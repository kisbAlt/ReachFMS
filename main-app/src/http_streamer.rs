use std::{fs, thread};
use std::fs::File;
use std::process::Child;
use std::sync::Mutex;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, Error};
use actix_web_actors::ws;
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use win_screenshot::prelude::*;
use crate::{api_communicator, comm_sender, ImageProcess};
use image::{RgbaImage};
use image;
use qstring::QString;
use std::time::{Instant};
use actix_files::Files;
use crossbeam_channel::{bounded, select, after};
use crate::config_handler::{ConfigHandler, DebugSave, get_static_folder};
use crate::image_process::InstrumentRgb;
use serde::{Deserialize, Serialize};
use crate::addon_config::AddonConfig;

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    bridge_status: String,
    settings: ConfigHandler,
}
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct BridgeStatus {
    pub started: bool,
    pub connected: bool,
    pub comm: bool,
}


struct AppState {
    last_bytes: Mutex<Vec<u8>>,
    last_update: Mutex<Instant>,
    main_html_string: &'static str,
    mcdu_svg: &'static str,
    ecam_svg: &'static str,
    tiff_js: &'static str,
    icon_png: &'static [u8],
    instrument_list: Mutex<Vec<InstrumentRgb>>,
    list_of_allowed_hwnd: Mutex<Vec<isize>>,
    config: Mutex<ConfigHandler>,
    child_process: Mutex<Option<Child>>,
    selected_hwnd: Mutex<isize>,
    command_sender: crossbeam_channel::Sender<String>,
    command_receiver: crossbeam_channel::Receiver<String>,
    comm_sender: crossbeam_channel::Sender<String>,
    comm_receiver: crossbeam_channel::Receiver<String>,
    bridge_status: Mutex<BridgeStatus>,
}


#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let child_proc = data.child_process.lock().unwrap();
    if child_proc.is_none() {
        drop(child_proc);
        return HttpResponse::Ok().body("Bridge not running");
    } else {
        drop(child_proc);
        HttpResponse::Ok().body(data.main_html_string)
    }
}


#[get("/ecam.svg")]
async fn ecam_svg(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.ecam_svg)
}

#[get("/tiff.min.js")]
async fn tiffjs(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.tiff_js)
}

#[get("/icon.png")]
async fn icon_png(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.icon_png)
}


#[get("/start_server")]
async fn start_server(data: web::Data<AppState>) -> impl Responder {
    let mut child_proc = data.child_process.lock().unwrap();
    if child_proc.is_none() {
        *child_proc = std::thread::spawn(move || {
            Option::from(api_communicator::start_bridge_process())
        }).join().unwrap();
    } else {
        drop(child_proc);
        return HttpResponse::Ok().body("Already running");
    }
    drop(child_proc);
    let mut brstat = data.bridge_status.lock().unwrap();
    brstat.started = true;
    drop(brstat);

    HttpResponse::Ok().body("started")
}

#[get("/bridge_status")]
async fn bridge_status(data: web::Data<AppState>) -> impl Responder {
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    comm_sender::get_status(&mut brid_status, data.command_sender.clone(), data.comm_receiver.clone());

    return HttpResponse::Ok().body(serde_json::to_string(&brid_status).unwrap());
}

#[get("/stop_server")]
async fn stop_server(data: web::Data<AppState>) -> impl Responder {
    let mut child_proc = data.child_process.lock().unwrap();
    let state_instruments = data.instrument_list.lock().unwrap();
    //let list_clone = hw_list.clone();
    let config = data.config.lock().unwrap();
    let do_restore = config.clone().auto_hide;
    if !child_proc.is_none() {
        let mut windows_to_hide: Vec<isize> = Vec::new();
        if do_restore {
            for state in state_instruments.iter() {
                if state.auto_hide {
                    windows_to_hide.push(state.hwnd)
                }
            }
        }
        if do_restore {
            for wn in windows_to_hide {
                unsafe { ImageProcess::move_window(wn, 0, 0, 700, 700) }
            }
        }
        data.command_sender.send("CloseBridge".to_string()).expect("cannot send CloseBridge");

        // std::thread::spawn(move || {
        //     api_communicator::stop_bridge_process()
        // }).join().unwrap();

        *child_proc = Option::None;
    } else {
        drop(child_proc);
        return HttpResponse::Ok().body("Not running");
    }
    drop(child_proc);
    drop(state_instruments);
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    brid_status.started = false;
    drop(brid_status);
    
    HttpResponse::Ok().body("stopped")
}

#[get("/reconnect")]
async fn bridge_reconnect() -> impl Responder {
    let resp = std::thread::spawn(move || {
        api_communicator::reconnect()
    }).join().unwrap();
    HttpResponse::Ok().body(resp)
}

#[get("/settings")]
async fn settings(data: web::Data<AppState>) -> impl Responder {
    let sting = data.config.lock().unwrap();
    let str_sting = sting.get_string();
    drop(sting);
    HttpResponse::Ok().body(str_sting)
}

#[get("/set_hwnd_settings")]
async fn set_hwnd_settings(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let for_hwnd = qs.clone().get("hwnd").unwrap_or("0")
        .parse::<isize>().unwrap_or(0);
    let auto_hide = match qs.clone().get("autohide").unwrap_or("none") {
        "true" => Option::from(true),
        "false" => Option::from(false),
        _ => { None }
    };
    let excluded = match qs.clone().get("excluded").unwrap_or("none") {
        "true" => Option::from(true),
        "false" => Option::from(false),
        _ => { None }
    };
    let mut state_instruments = data.instrument_list.lock().unwrap();
    let mut count: usize = 0;
    for instr in state_instruments.iter() {
        if instr.hwnd == for_hwnd {
            if auto_hide.is_some() {
                state_instruments[count].auto_hide = auto_hide.unwrap();
            }
            if excluded.is_some() {
                state_instruments[count].excluded = excluded.unwrap();
            }
            return HttpResponse::Ok().body("ok");
        }
        count += 1;
    }
    drop(state_instruments);
    HttpResponse::Ok().body("can't find hwnd")
}

#[get("/set_settings")]
async fn set_settings(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let mut refresh = qs.clone().get("refresh").unwrap_or("200")
        .parse::<u16>().unwrap_or(200);
    let auto_hide = match qs.clone().get("autohide").unwrap_or("true") {
        "true" => true,
        "false" => false,
        _ => { true }
    };

    let max_fps = match qs.clone().get("maxfps").unwrap_or("true") {
        "true" => true,
        "false" => false,
        _ => { true }
    };

    let multiple = match qs.clone().get("multiple").unwrap_or("true") {
        "true" => true,
        "false" => false,
        _ => { true }
    };

    let cpu_disp = match qs.clone().get("alternate").unwrap_or("false") {
        "true" => true,
        "false" => false,
        _ => { false }
    };

    let auto_start = match qs.clone().get("autostart").unwrap_or("false") {
        "true" => true,
        "false" => false,
        _ => { false }
    };

    if refresh < 10 {
        refresh = 10;
    }

    let mut conf = data.config.lock().unwrap();
    conf.refresh_rate = refresh;
    conf.auto_hide = auto_hide;
    conf.max_fps = max_fps;
    conf.multiple_displays = multiple;
    conf.cpu_displays = cpu_disp;
    conf.auto_start = auto_start;
    conf.write_config();
    drop(conf);
    HttpResponse::Ok().body("ok")
}

#[get("/save_debug")]
async fn save_debug(data: web::Data<AppState>) -> impl Responder {
    if std::path::Path::new("debug").exists() {
        fs::remove_dir_all("debug").unwrap();
    }
    fs::create_dir("debug").unwrap();

    if std::path::Path::new("debug.tar").exists() {
        fs::remove_file("debug.tar").unwrap()
    }
    let file = File::create("debug.tar").unwrap();
    let mut a = tar::Builder::new(file);

    
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    comm_sender::get_status(&mut brid_status, data.command_sender.clone(), data.comm_receiver.clone());
    let temp_status: String = serde_json::to_string(&brid_status).unwrap_or("".to_string());
    drop(brid_status);
    
    
    let conf = data.config.lock().unwrap();
    let temp_instr = ImageProcess::start(None, false, false);

    let debug: DebugSave = DebugSave {
        instrument_list: ImageProcess::window_to_string(&temp_instr),
        config: conf.get_string(),
        status: temp_status,
    };
    drop(conf);

    File::create("debug/debug.json")
        .expect("Error encountered while creating file!");
    let json_string = serde_json::to_string(&debug).unwrap();
    fs::write("debug/debug.json", json_string).expect("Unable to write file");
    a.append_path("debug/debug.json").unwrap();

    if std::path::Path::new("data/samples").exists() {
        let sample_paths = fs::read_dir("data/samples").unwrap();
        for path in sample_paths {
            a.append_path(path.unwrap().path()).expect("Cant append sample!");
        }
    }

    for instr in temp_instr.iter() {
        let buf = capture_window_ex(instr.hwnd, Using::PrintWindow,
                                    Area::ClientOnly, None, None).unwrap();
        let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
        img.save(format!("debug/{}_{}.jpg", instr.instrument, instr.hwnd)).unwrap();
        a.append_path(format!("debug/{}_{}.jpg", instr.instrument, instr.hwnd)).unwrap();
    }

    if std::path::Path::new("WASimClient.log").exists() {
        a.append_path("WASimClient.log").unwrap();
    }

    if std::path::Path::new("debug").exists() {
        fs::remove_dir_all("debug").unwrap();
    }

    HttpResponse::Ok().body("ok")
}

#[get("/status")]
async fn status(data: web::Data<AppState>) -> impl Responder {
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    comm_sender::get_status(&mut brid_status, data.command_sender.clone(), data.comm_receiver.clone());
    let resp: String = serde_json::to_string(&brid_status).unwrap_or("".to_string());
    drop(brid_status);
    let sting = data.config.lock().unwrap();
    let resp: StatusResponse = StatusResponse {
        settings: sting.clone(),
        bridge_status: resp,
    };
    drop(sting);


    HttpResponse::Ok().body(serde_json::to_string(&resp).unwrap())
}


#[get("/mcdu_btn_press")]
async fn mcdu_btn(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let btn_id = qs.clone().get("btn").unwrap_or("").to_string();


    data.command_sender.send(format!("SM_SEND:CMD_BTN:{}", btn_id)).expect("ERROR SENDING MESSAGE");

    HttpResponse::Ok().body("ok")
}

#[get("/restore_windows")]
async fn restore_windows(data: web::Data<AppState>) -> HttpResponse {
    let hw_list = data.list_of_allowed_hwnd.lock().unwrap();
    let list_clone = hw_list.clone();
    for wn in list_clone {
        unsafe { ImageProcess::move_window(wn, 0, 0, 700, 700) }
    }

    HttpResponse::Ok().body("ok")
}

#[get("/hide_windows")]
async fn hide_popout_windows(data: web::Data<AppState>) -> HttpResponse {
    let hw_list = data.list_of_allowed_hwnd.lock().unwrap();
    let list_clone = hw_list.clone();
    for wn in list_clone {
        unsafe { ImageProcess::hide_window(wn) }
    }

    HttpResponse::Ok().body("ok")
}


#[get("/get_windows")]
async fn get_windows(data: web::Data<AppState>) -> HttpResponse {
    let mut state_instruments = data.instrument_list.lock().unwrap();
    let mut hw_list = data.list_of_allowed_hwnd.lock().unwrap();
    let conf = data.config.lock().unwrap();

    let wndows = ImageProcess::start(
        Option::from(state_instruments.clone()), conf.auto_hide, false);

    let mut local_hwnd_list: Vec<isize> = Vec::new();
    for i in &wndows {
        local_hwnd_list.push(i.hwnd);
    }
    *hw_list = local_hwnd_list.clone();

    let resp = ImageProcess::window_to_string(&wndows);
    *state_instruments = wndows;
    drop(state_instruments);

    HttpResponse::Ok().body(resp)
}

#[get("/get_aircraft")]
async fn get_aircraft(data: web::Data<AppState>) -> HttpResponse {
    let aircraft: String = comm_sender::get_aircraft(&data.command_sender, &data.comm_receiver);
    
    return HttpResponse::Ok().body(aircraft);
}

#[get("/force_rescan")]
async fn force_rescan(data: web::Data<AppState>) -> HttpResponse {
    let mut state_instruments = data.instrument_list.lock().unwrap();
    let mut hw_list = data.list_of_allowed_hwnd.lock().unwrap();
    let conf = data.config.lock().unwrap();

    let wndows = ImageProcess::start(
        Option::from(state_instruments.clone()), conf.auto_hide, true);

    let mut local_hwnd_list: Vec<isize> = Vec::new();
    for i in &wndows {
        local_hwnd_list.push(i.hwnd);
    }
    *hw_list = local_hwnd_list.clone();

    let resp = ImageProcess::window_to_string(&wndows);
    *state_instruments = wndows;
    drop(state_instruments);

    HttpResponse::Ok().body(resp)
}


#[get("/set_min_capture_ms")]
async fn set_min_capture_ms(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let mut ms_param = qs.get("ms").unwrap().parse::<u32>().unwrap_or(0) as u16;

    if ms_param < 10
    {
        ms_param = 10;
    }

    let mut min_ms = data.config.lock().unwrap();
    min_ms.refresh_rate = ms_param;
    min_ms.write_config();
    drop(min_ms);
    HttpResponse::Ok()
        .body("ok")
}

#[get("/image_state")]
async fn image_state(data: web::Data<AppState>) -> HttpResponse {
    let last_bytes = data.last_bytes.lock().unwrap();
    let bytes = last_bytes.clone();
    drop(last_bytes);
    return HttpResponse::Ok()
        .body(bytes);
}

#[get("/set_hwnd")]
async fn set_hwnd(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let mut hwnd = data.selected_hwnd.lock().unwrap();
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let hw_id = qs.get("hwnd").unwrap().parse::<u32>().unwrap_or(0) as isize;

    let allowed_hwnd = data.list_of_allowed_hwnd.lock().unwrap();
    if !allowed_hwnd.contains(&hw_id) {
        return HttpResponse::Ok()
            .body("Not allowed");
    }
    drop(allowed_hwnd);

    *hwnd = hw_id;
    drop(hwnd);
    HttpResponse::Ok()
        .body("ok")
}


#[get("/calibrate_displays")]
async fn calibrate_displays(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let hw_id = qs.get("calibratestr").unwrap_or("");
    // str format: hwnd:INSTRUMENT;hwnd:INSTRUMENT


    if hw_id == "" {
        return HttpResponse::Ok()
            .body("error");
    }

    let calibrate_list = hw_id.split(";");
    for istr in calibrate_list {
        let hwnd: isize = istr.split(":").collect::<Vec<&str>>()[0].parse::<isize>().unwrap_or(0);
        println!("currently: {}", hwnd);
        let instrument: &str = istr.split(":").collect::<Vec<&str>>()[1];
        ImageProcess::capture_instrument_sample(hwnd, instrument);
    }
    let mut conf = data.config.lock().unwrap();
    conf.calibrated = true;
    conf.write_config();
    drop(conf);


    HttpResponse::Ok()
        .body("ok")
}


#[get("/get_image")]
async fn get_image(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let mut lastupdate = data.last_update.lock().unwrap();
    let min_ms = data.config.lock().unwrap();
    // let send_format = match min_ms.max_fps {
    //     true => ImageFormat::Jpeg,
    //     false => ImageFormat::Tiff
    // };
    if !min_ms.multiple_displays && lastupdate.elapsed().as_millis() < min_ms.refresh_rate as u128 {
        let last_bytes = data.last_bytes.lock().unwrap();
        let bytes = last_bytes.clone();
        drop(last_bytes);
        return HttpResponse::Ok()
            .body(bytes);
    }
    *lastupdate = Instant::now();

    drop(lastupdate);
    drop(min_ms);

    //let hw_id = data.selected_hwnd.lock().unwrap().clone();
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let hw_id: isize = qs.clone().get("hwnd").unwrap_or("0")
        .parse::<isize>().unwrap_or(0);

    let mut crop = [[0, 0], [0, 0]];

    let insr_list = data.instrument_list.lock().unwrap();
    for instr in insr_list.iter() {
        if instr.hwnd == hw_id {
            crop = instr.crop;
        }
    }

    let resp = match ImageProcess::capture_instrument(hw_id, crop) {
        Ok(tempresp) => tempresp,
        Err(..) => return HttpResponse::Ok().body("HwndNotFound")
    };
    let mut last_bytes = data.last_bytes.lock().unwrap();
    *last_bytes = resp.clone();
    drop(last_bytes);


    HttpResponse::Ok()
        .body(resp)
}

#[get("/test_ws")]
async fn test_ws(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    // test

    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let cmd: &str = qs.get("cmd").unwrap_or("");

    let snd = data.command_sender.clone();
    snd.send(format!("SM_SEND:{cmd}").parse().unwrap()).expect("ERROR SENDING STAT MSG");

    // end test
    return HttpResponse::Ok()
        .body("sent.");
}

struct MyWs {
    pub command_receiver: crossbeam_channel::Receiver<String>,
    pub comm_sender: crossbeam_channel::Sender<String>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}
#[derive(Message)]
#[rtype(result = "()")]
struct StringConnectMessage(String);

impl Handler<StringConnectMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: StringConnectMessage, ctx: &mut Self::Context) {
        // Send the string message to the WebSocket client
        println!("sending StringConnectMessage: {}", msg.0);
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("msg");
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("ws message: {}", text);

                if (text == "ConnectWSClient") {
                    let rx = self.command_receiver.clone();
                    let sn = self.comm_sender.clone();
                    let addr = ctx.address().clone();
                    //let ctx_clone = ctx.deref().clone();
                    thread::spawn(move || {
                        println!("Recv thread spawned!");
                        loop {
                            let value = rx.recv().expect("Unable to receive from channel");

                            // COMMUNICATION ON THE CHANNELS:
                            // CloseBridge  => send close cmd
                            // SM_SEND:CMD_BTN:EXAMPLE_LVAR send msg to Simconnector to press the EXAMLE_LVAR btn
                            if value == "CloseBridge" {
                                addr.do_send(StringConnectMessage("CLOSE".to_string()));
                            }
                            if (value == "BridgeStatus") {
                                println!("Getting status...");
                                addr.do_send(StringConnectMessage("STATUS".to_string()));
                            } else if (value.contains("SM_SEND:")) {
                                //let cmnd: &str = value.split(":").collect::<Vec<&str>>()[1];
                                addr.do_send(StringConnectMessage(value.replace("SM_SEND:", "")))
                            }

                            println!("Got channel val: {value}");
                        }
                    });
                    ctx.text("CONNECTED");
                } else if (text.contains("STATUS:")) {
                    let stat = text.replace("STATUS:", "");
                    println!("GOT STATUS: {}", stat);
                    if (stat == "TRUE") {
                        self.comm_sender.send(stat).unwrap()
                    } else {
                        self.comm_sender.send("FALSE".to_string()).unwrap()
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn ws_index(req: HttpRequest, stream: web::Payload, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let rec = data.command_receiver.clone();
    let sndr = data.comm_sender.clone();
    let resp = ws::start(MyWs {
        command_receiver: rec,
        comm_sender: sndr,
    }, &req, stream);

    println!("{:?}", resp);
    resp
}

// #[get("/jpeg_test")]
// async fn jpeg_test(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
//     println!("start");
//     let mut now = Instant::now();
//     let mut strres = "".to_string();
//     for i in 0..10 {
//         let buf = capture_display().unwrap();
//         let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
//         let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
//         img.write_to(&mut buffer, ImageFormat::Jpeg).unwrap();
//         let bytes: Vec<u8> = buffer.into_inner().unwrap().into_inner();
//     };
//
//     println!("end");
//     println!("Elapsed first: {:.2?}", now.elapsed());
//     strres += &*format!("Elapsed first: {:.2?}", now.elapsed()).to_string();
//
//     now = Instant::now();
//
//     use std::io::BufWriter;
//     for i in 0..10 {
//         let buf = capture_display().unwrap();
//
//         let ref mut w = BufWriter::new(Cursor::new(Vec::new()));
//         let mut outputasd = Vec::new();
//         {
//             let mut encoder = png::Encoder::new(&mut outputasd, buf.width, buf.height);
//             encoder.set_color(png::ColorType::Rgba);
//             encoder.set_depth(png::BitDepth::Eight);
//             let mut writer = encoder.write_header().unwrap();
//
//             writer.write_image_data(&buf.pixels).unwrap(); // Save
//         }
//     };
//     println!("end");
//     println!("Elapsed second: {:.2?}", now.elapsed());
//     strres += &*format!("   Elapsed second: {:.2?}", now.elapsed()).to_string();
//
//     HttpResponse::Ok()
//         .body(strres)
// }

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let mut config = ConfigHandler::init();
    config.read_config();
    let (s, r) = bounded::<String>(0);
    let (sc, rc) = bounded::<String>(0);
    let state = web::Data::new(AppState {
        last_bytes: Mutex::from(Vec::new()),
        last_update: Mutex::from(Instant::now()),
        main_html_string: include_str!("../../frontend/build/index.html"),
        mcdu_svg: "",
        ecam_svg: include_str!("../../ecam_controls.svg"),
        tiff_js: include_str!("../tiff.min.js"),
        icon_png: include_bytes!("../../icon_compressed.png"),
        instrument_list: Mutex::from(vec![]),
        list_of_allowed_hwnd: Mutex::new(Vec::new()),
        config: Mutex::from(config),
        child_process: Mutex::from(Option::None),
        selected_hwnd: Mutex::from(0),
        command_sender: s,
        command_receiver: r,
        comm_sender: sc,
        comm_receiver: rc,
        bridge_status: Mutex::from(BridgeStatus {
            connected: false,
            started: false,
            comm: false
        }),
    });
    let static_path: String = get_static_folder();
    
    
    
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(index)
            .service(ecam_svg)
            .service(tiffjs)
            .service(icon_png)
            .service(mcdu_btn)
            .service(get_image)
            .service(save_debug)
            .service(set_hwnd_settings)
            .service(get_windows)
            .service(image_state)
            .service(set_min_capture_ms)
            .service(restore_windows)
            .service(hide_popout_windows)
            .service(settings)
            .service(status)
            .service(start_server)
            .service(set_settings)
            .service(stop_server)
            .service(set_hwnd)
            .service(bridge_reconnect)
            .service(force_rescan)
            .service(calibrate_displays)
            .service(test_ws)
            .service(bridge_status)
            .service(get_aircraft)
            .service(Files::new("/static", static_path.clone()))
            .route("/ws/", web::get().to(ws_index))
        //.service(jpeg_test)
    })
        .bind(("0.0.0.0", 5273))?
        .run()
        .await
}