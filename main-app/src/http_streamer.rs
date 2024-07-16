use std::{fs, thread};
use std::fs::File;
use std::process::Child;
use std::sync::{Arc, Mutex};
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, Error};
use actix_web_actors::ws;
use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use win_screenshot::prelude::*;
use crate::{api_communicator, comm_sender, ImageProcess};
use image::{RgbaImage};
use image;
use qstring::QString;
use actix_files::Files;
use crossbeam_channel::{bounded};
use crate::config_handler::{ConfigHandler, DebugSave, get_static_folder};
use crate::image_process::InstrumentRgb;
use serde::{Deserialize, Serialize};
use crate::addon_config::AddonConfig;
#[derive(Serialize, Deserialize)]
struct StatusResponse {
    bridge_status: BridgeStatus,
    settings: ConfigHandler,
}
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct BridgeStatus {
    pub started: bool,
    pub connected: bool,
    pub comm: bool,
}


pub struct ImageSubscriptionStatus {
    pub thread_started: Arc<Mutex<bool>>,
    pub selected_hwnd: Arc<Mutex<isize>>,
    pub img_sub_list: Arc<Mutex<Vec<Addr<MyWs>>>>,
    pub display_crop: Arc<Mutex<[[i32; 2]; 2]>>,
}


struct AppState {
    last_bytes: Mutex<Vec<u8>>,
    main_html_string: &'static str,
    icon_png: &'static [u8],
    instrument_list: Mutex<Vec<InstrumentRgb>>,
    config: Arc<Mutex<ConfigHandler>>,
    child_process: Mutex<Option<Child>>,
    //selected_hwnd: Mutex<isize>,
    command_sender: crossbeam_channel::Sender<String>,
    command_receiver: crossbeam_channel::Receiver<String>,
    comm_sender: crossbeam_channel::Sender<String>,
    comm_receiver: crossbeam_channel::Receiver<String>,
    bridge_status: Mutex<BridgeStatus>,
    img_sub_status: ImageSubscriptionStatus,
    current_aircraft: Mutex<String>,
    addon_config: AddonConfig,
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
    
    println!("locking child_proc");
    let mut child_proc = data.child_process.lock().unwrap();
    if !child_proc.is_none() {
        println!("child_running");
        data.command_sender.send("CloseBridge".to_string()).expect("cannot send CloseBridge");
        *child_proc = Option::None;
    } else {
        println!("child not running");
        drop(child_proc);
        return HttpResponse::Ok().body("Not running");
    }
    drop(child_proc);
    println!("Setting status");
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    brid_status.started = false;
    drop(brid_status);

    println!("returning resp");
    HttpResponse::Ok().body("stopped")
}

#[get("/reconnect")]
async fn bridge_reconnect(data: web::Data<AppState>) -> impl Responder {
    let resp: String = comm_sender::reconnect(data.command_sender.clone(), data.comm_receiver.clone());
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

    if refresh < 50 {
        refresh = 50;
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
    let temp_instr = ImageProcess::start(Option::None, Option::None);
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


    let sting = data.config.lock().unwrap();
    let resp: StatusResponse = StatusResponse {
        settings: sting.clone(),
        bridge_status: brid_status.clone(),
    };
    drop(sting);
    drop(brid_status);

    HttpResponse::Ok().body(serde_json::to_string(&resp).unwrap())
}


#[get("/mcdu_btn_press")]
async fn mcdu_btn(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let btn_id = qs.clone().get("btn").unwrap_or("").to_string();
    let aircraft = data.current_aircraft.lock().unwrap().clone();

    if aircraft.is_empty() {}

    let lvar = data.addon_config.get_var(btn_id.replace("BTN:", ""), aircraft);

    if lvar.contains("K:") {
        data.command_sender.send(format!("SM_SEND:CUSTOM_WASM:{}", lvar)).expect("ERROR SENDING MESSAGE");
    } else {
        data.command_sender.send(format!("SM_SEND:CMD_BTN:{}", lvar)).expect("ERROR SENDING MESSAGE");
    }


    HttpResponse::Ok().body("ok")
}

#[get("/restore_windows")]
async fn restore_windows() -> HttpResponse {
    ImageProcess::restore_all();

    HttpResponse::Ok().body("ok")
}

#[get("/hide_windows")]
async fn hide_popout_windows(data: web::Data<AppState>) -> HttpResponse {
    let hw = data.img_sub_status.selected_hwnd.lock().unwrap();
    unsafe { ImageProcess::hide_window(*hw); }

    HttpResponse::Ok().body("ok")
}


#[get("/get_windows")]
async fn get_windows(data: web::Data<AppState>) -> HttpResponse {
    let mut state_instruments = data.instrument_list.lock().unwrap();
    let conf = data.config.lock().unwrap();

    
    let aircraft: String = comm_sender::get_aircraft(&data.command_sender, &data.comm_receiver);
    let mut saved = data.current_aircraft.lock().unwrap();
    *saved = aircraft.clone();
    drop(saved);
    let sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap().clone();
    let wndows = ImageProcess::start(Option::from(conf.auto_hide), Option::from(sub_hwnd));
    let resp = ImageProcess::window_to_string(&wndows);

    for img in &wndows {
        if img.instrument == "MCDU" {
            let mut sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap();
            *sub_hwnd = img.hwnd;
            let find_crop = data.addon_config.calculate_crop(&aircraft,
                                                             700, 700);
            let mut crop = data.img_sub_status.display_crop.lock().unwrap();
            *crop = find_crop;
        }
    }
    
    *state_instruments = wndows;
    drop(state_instruments);
    
    HttpResponse::Ok().body(resp)
}

#[get("/get_aircraft")]
async fn get_aircraft(data: web::Data<AppState>) -> HttpResponse {
    let aircraft: String = comm_sender::get_aircraft(&data.command_sender, &data.comm_receiver);
    let mut saved = data.current_aircraft.lock().unwrap();
    *saved = aircraft.clone();
    return HttpResponse::Ok().body(aircraft);
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
    
    let conf = data.config.lock().unwrap();

    let aircraft: String = comm_sender::get_aircraft(&data.command_sender, &data.comm_receiver);
    let mut saved = data.current_aircraft.lock().unwrap();
    *saved = aircraft.clone();
    drop(saved);
    
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let hw_id = qs.get("hwnd").unwrap_or("0")
        .parse::<isize>().unwrap_or(0);

    let wndows = ImageProcess::start(Option::from(conf.auto_hide), Option::from(hw_id));
    

    for img in &wndows {
        if img.instrument == "MCDU" {
            let mut sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap();
            *sub_hwnd = img.hwnd;
            let find_crop = data.addon_config.calculate_crop(&aircraft,
                                                             700, 700);
            let mut crop = data.img_sub_status.display_crop.lock().unwrap();
            *crop = find_crop;
            let mut state_instruments = data.instrument_list.lock().unwrap();
            *state_instruments = wndows;
            return HttpResponse::Ok()
                .body("ok")
        }
    }
    
    HttpResponse::Ok()
        .body("error")
}
// 
// 
// #[get("/calibrate_displays")]
// async fn calibrate_displays(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
//     let query_str = req.query_string(); // "name=ferret"
//     let qs = QString::from(query_str);
//     let hw_id = qs.get("calibratestr").unwrap_or("");
//     // str format: hwnd:INSTRUMENT;hwnd:INSTRUMENT
// 
// 
//     if hw_id == "" {
//         return HttpResponse::Ok()
//             .body("error");
//     }
// 
//     let calibrate_list = hw_id.split(";");
//     for istr in calibrate_list {
//         let hwnd: isize = istr.split(":").collect::<Vec<&str>>()[0].parse::<isize>().unwrap_or(0);
//         
//         let instrument: &str = istr.split(":").collect::<Vec<&str>>()[1];
//         ImageProcess::capture_instrument_sample(hwnd, instrument);
//     }
//     let mut conf = data.config.lock().unwrap();
//     conf.calibrated = true;
//     conf.write_config();
//     drop(conf);
// 
// 
//     HttpResponse::Ok()
//         .body("ok")
// }

// 
// #[get("/get_image")]
// async fn get_image(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
//     let mut lastupdate = data.last_update.lock().unwrap();
//     let min_ms = data.config.lock().unwrap();
// 
//     if !min_ms.multiple_displays && lastupdate.elapsed().as_millis() < min_ms.refresh_rate as u128 {
//         let last_bytes = data.last_bytes.lock().unwrap();
//         let bytes = last_bytes.clone();
//         drop(last_bytes);
//         return HttpResponse::Ok()
//             .body(bytes);
//     }
//     *lastupdate = Instant::now();
// 
//     drop(lastupdate);
//     drop(min_ms);
// 
//     //let hw_id = data.selected_hwnd.lock().unwrap().clone();
//     let query_str = req.query_string();
//     let qs = QString::from(query_str);
//     let hw_id: isize = qs.clone().get("hwnd").unwrap_or("0")
//         .parse::<isize>().unwrap_or(0);
// 
//     let mut crop = [[0, 0], [0, 0]];
// 
//     let insr_list = data.instrument_list.lock().unwrap();
//     for instr in insr_list.iter() {
//         if instr.hwnd == hw_id {
//             crop = instr.crop;
//         }
//     }
// 
//     let resp = match ImageProcess::capture_instrument(hw_id, crop) {
//         Ok(tempresp) => tempresp,
//         Err(..) => return HttpResponse::Ok().body("HwndNotFound")
//     };
//     let mut last_bytes = data.last_bytes.lock().unwrap();
//     *last_bytes = resp.clone();
//     drop(last_bytes);
// 
// 
//     HttpResponse::Ok()
//         .body(resp)
// }


pub struct MyWs {
    pub command_receiver: crossbeam_channel::Receiver<String>,
    pub comm_sender: crossbeam_channel::Sender<String>,
    pub img_subscribers: Arc<Mutex<Vec<Addr<MyWs>>>>,
    pub sub_started: Arc<Mutex<bool>>,
    pub sub_hwnd: Arc<Mutex<isize>>,
    pub img_crop: Arc<Mutex<[[i32; 2]; 2]>>,
    pub config: Arc<Mutex<ConfigHandler>>,
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
        ctx.text(msg.0);
    }
}


// Define a custom message type for sending binary data
#[derive(Message)]
#[rtype(result = "()")]
struct BinaryMessage(Vec<u8>);

impl Handler<BinaryMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: BinaryMessage, ctx: &mut Self::Context) {
        ctx.binary(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                if text == "ConnectWSClient" {
                    let rx = self.command_receiver.clone();
                    //let sn = self.comm_sender.clone();
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
                            if value == "BridgeStatus" {
                                println!("Getting status...");
                                addr.do_send(StringConnectMessage("STATUS".to_string()));
                            } else if value == "GetAircraft" {
                                addr.do_send(StringConnectMessage("GET_AIRCRAFT".to_string()));
                            } else if value.contains("SM_SEND:") {
                                //let cmnd: &str = value.split(":").collect::<Vec<&str>>()[1];
                                println!("SM: {}", value.replace("SM_SEND:", ""));
                                addr.do_send(StringConnectMessage(value.replace("SM_SEND:", "")))
                            }
                        }
                    });
                    ctx.text("CONNECTED");
                } else if text == "IMAGESUBSCRIBE" {
                    let mut started = self.sub_started.lock().unwrap();
                    self.img_subscribers.lock().unwrap().push(ctx.address());

                    if !*started {
                        println!("Adding sub thread");
                        *started = true;
                        drop(started);
                        let sbs = Arc::clone(&self.img_subscribers);
                        let cnfig_inner = Arc::clone(&self.config);
                        let hwnd = Arc::clone(&self.sub_hwnd);
                        let crop = Arc::clone(&self.img_crop);
                        thread::spawn(move || {
                            let mut refresh = cnfig_inner.lock().unwrap().refresh_rate.clone();
                            let mut inner_subs = Arc::clone(&sbs).lock().unwrap().clone();
                            let mut inner_hwnd = Arc::clone(&hwnd).lock().unwrap().clone();
                            let mut inner_crop = Arc::clone(&crop).lock().unwrap().clone();

                            let mut counter = 500;
                            loop {
                                if counter % 10 == 0 {
                                    refresh = cnfig_inner.lock().unwrap().refresh_rate.clone();
                                    let current_subs = Arc::clone(&sbs);
                                    let mut subs_locked = current_subs.lock().unwrap();
                                    let mut i = 0;
                                    while i < subs_locked.len() {
                                        if !subs_locked[i].connected() {
                                            subs_locked.remove(i);
                                        } else {
                                            i += 1;
                                        }
                                    }
                                    inner_subs = subs_locked.clone();
                                    inner_hwnd = Arc::clone(&hwnd).lock().unwrap().clone();
                                    inner_crop = Arc::clone(&crop).lock().unwrap().clone();
                                    //println!("CROP: {:?}", inner_crop);
                                    //inner_hwnd = Arc::clone(&hwnd).lock().unwrap().clone();
                                    //println!("sub count: {}", inner_subs.len())
                                }
                                if inner_subs.len() > 0 {
                                    let img =
                                        ImageProcess::capture_instrument(
                                            inner_hwnd, inner_crop).unwrap();

                                    for i in 0..inner_subs.len() {
                                        if inner_hwnd != 0 {
                                            let req =
                                                inner_subs[i].try_send(BinaryMessage(img.clone()));
                                            
                                        }
                                    }
                                }
                                thread::sleep(std::time::Duration::from_millis(refresh as u64));
                                counter += 1;
                            }
                        });
                    }
                } else {
                    self.comm_sender.send(text.to_string()).unwrap()
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
        img_subscribers: Arc::clone(&data.img_sub_status.img_sub_list),
        sub_started: Arc::clone(&data.img_sub_status.thread_started),
        config: data.config.clone(),
        sub_hwnd: Arc::clone(&data.img_sub_status.selected_hwnd),
        img_crop: Arc::clone(&data.img_sub_status.display_crop),
    }, &req, stream);

    resp
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let mut config = ConfigHandler::init();
    config.read_config();
    let (s, r) = bounded::<String>(0);
    let (sc, rc) = bounded::<String>(0);
    let addon_config = AddonConfig::load().await;
    let state = web::Data::new(AppState {
        last_bytes: Mutex::from(Vec::new()),
        main_html_string: include_str!("../../frontend/build/index.html"),
        icon_png: include_bytes!("../../svg/reachfms_white.png"),
        instrument_list: Mutex::from(vec![]),
        config: Arc::new(Mutex::from(config)),
        child_process: Mutex::from(Option::None),
        //selected_hwnd: Mutex::from(0),
        command_sender: s,
        command_receiver: r,
        comm_sender: sc,
        comm_receiver: rc,
        current_aircraft: Mutex::new("".to_string()),
        bridge_status: Mutex::from(BridgeStatus {
            connected: false,
            started: false,
            comm: false,
        }),
        img_sub_status: ImageSubscriptionStatus {
            thread_started: Arc::new(Mutex::new(false)),
            img_sub_list: Arc::new(Mutex::new(vec![])),
            selected_hwnd: Arc::new(Mutex::new(0)),
            display_crop: Arc::new(Mutex::new([[0, 0], [0, 0]])),
        },
        addon_config,
    });
    let static_path: String = get_static_folder();
    

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(index)
            .service(icon_png)
            .service(mcdu_btn)
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