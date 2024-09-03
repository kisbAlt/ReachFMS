use std::{mem, thread};
use std::process::Child;
use std::sync::{Arc, Mutex};
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, Error};
use actix_web_actors::ws;
use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use crate::{api_communicator, comm_sender, debug_logger, ImageProcess};
use qstring::QString;
use actix_files::Files;
use crossbeam_channel::{bounded};
use crate::config_handler::{ConfigHandler, get_static_folder};
use crate::image_process::{InstrumentRgb, POPOUT_HEIGHT, POPOUT_WIDTH};
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowRect, SetCursorPos, SetForegroundWindow, SM_CXSCREEN, SM_CYSCREEN};
use windows::Win32::UI::Input::KeyboardAndMouse::{mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, SetFocus};
use crate::addon_config::{AddonConfig};
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
    pub instrument_search: Mutex<String>,
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
    log_str: Option<Arc<Mutex<String>>>,
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
    debug_logger::log("Starting SimConnector", &data.log_str);
    let mut child_proc = data.child_process.lock().unwrap();
    if child_proc.is_none() {
        *child_proc = std::thread::spawn(move || {
            Option::from(api_communicator::start_bridge_process())
        }).join().unwrap();
        debug_logger::log("SimConnector started", &data.log_str);
    } else {
        drop(child_proc);
        debug_logger::log("SimConnector was already running...", &data.log_str);
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
    debug_logger::log("Getting bridge status", &data.log_str);
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    comm_sender::get_status(&mut brid_status, data.command_sender.clone(), data.comm_receiver.clone());

    debug_logger::log(&*format!("Getting comm_sender status connected: {}, conn: {}",
                                &brid_status.connected, &brid_status.comm), &data.log_str);

    return HttpResponse::Ok().body(serde_json::to_string(&brid_status).unwrap());
}

#[get("/stop_server")]
async fn stop_server(data: web::Data<AppState>) -> impl Responder {
    debug_logger::log("Stopping SimConnector...", &data.log_str);

    let mut child_proc = data.child_process.lock().unwrap();
    if !child_proc.is_none() {
        data.command_sender.send("CloseBridge".to_string()).expect("cannot send CloseBridge");
        *child_proc = Option::None;
    } else {
        debug_logger::log("Simconnector wasn't running...", &data.log_str);
        drop(child_proc);
        return HttpResponse::Ok().body("Not running");
    }
    drop(child_proc);
    let mut brid_status = data.bridge_status.lock().unwrap().clone();
    brid_status.started = false;
    drop(brid_status);

    HttpResponse::Ok().body("stopped")
}

#[get("/reconnect")]
async fn bridge_reconnect(data: web::Data<AppState>) -> impl Responder {
    let resp: String = comm_sender::reconnect(data.command_sender.clone(), data.comm_receiver.clone());
    debug_logger::log("reconnecting server...", &data.log_str);
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
    debug_logger::log("Setting hwnd settings... (deprecated)", &data.log_str);
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
    debug_logger::log("Editing settings...", &data.log_str);
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
    debug_logger::log("saving debug data (deprecated)", &data.log_str);
    let mut conf = data.config.lock().unwrap();
    conf.log_enabled = true;
    conf.write_config();
    drop(conf);
    // if std::path::Path::new("debug").exists() {
    //     fs::remove_dir_all("debug").unwrap();
    // }
    // fs::create_dir("debug").unwrap();
    // 
    // if std::path::Path::new("debug.tar").exists() {
    //     fs::remove_file("debug.tar").unwrap()
    // }
    // let file = File::create("debug.tar").unwrap();
    // let mut a = tar::Builder::new(file);
    // 
    // 
    // let mut brid_status = data.bridge_status.lock().unwrap().clone();
    // comm_sender::get_status(&mut brid_status, data.command_sender.clone(), data.comm_receiver.clone());
    // let temp_status: String = serde_json::to_string(&brid_status).unwrap_or("".to_string());
    // drop(brid_status);
    // 
    // 
    // let conf = data.config.lock().unwrap();
    // let temp_instr = ImageProcess::start(Option::None, Option::None);
    // let debug: DebugSave = DebugSave {
    //     instrument_list: ImageProcess::window_to_string(&temp_instr),
    //     config: conf.get_string(),
    //     status: temp_status,
    // };
    // drop(conf);
    // 
    // File::create("debug/debug.json")
    //     .expect("Error encountered while creating file!");
    // let json_string = serde_json::to_string(&debug).unwrap();
    // fs::write("debug/debug.json", json_string).expect("Unable to write file");
    // a.append_path("debug/debug.json").unwrap();
    // 
    // if std::path::Path::new("data/samples").exists() {
    //     let sample_paths = fs::read_dir("data/samples").unwrap();
    //     for path in sample_paths {
    //         a.append_path(path.unwrap().path()).expect("Cant append sample!");
    //     }
    // }
    // 
    // for instr in temp_instr.iter() {
    //     let buf = capture_window_ex(instr.hwnd, Using::PrintWindow,
    //                                 Area::ClientOnly, None, None).unwrap();
    //     let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    //     img.save(format!("debug/{}_{}.jpg", instr.instrument, instr.hwnd)).unwrap();
    //     a.append_path(format!("debug/{}_{}.jpg", instr.instrument, instr.hwnd)).unwrap();
    // }
    // 
    // if std::path::Path::new("WASimClient.log").exists() {
    //     a.append_path("WASimClient.log").unwrap();
    // }
    // 
    // if std::path::Path::new("debug").exists() {
    //     fs::remove_dir_all("debug").unwrap();
    // }

    HttpResponse::Ok().body("log enabled")
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
    let instr = data.img_sub_status.instrument_search.lock().unwrap().clone();

    if instr.is_empty() {}

    let aircraft_var = data.addon_config.get_var(btn_id.replace("BTN:", ""), instr);

    if aircraft_var == "" {
        debug_logger::log(&*format!("Cant find lvar for: {}", &btn_id), &data.log_str);
        return HttpResponse::Ok().body("Cant find lvar");
    }

    if aircraft_var.contains(">") || aircraft_var.contains("K:") || aircraft_var.contains("H:") {
        data.command_sender.send(format!("SM_SEND:CUSTOM_WASM:{}", aircraft_var)).expect("ERROR SENDING MESSAGE");
    } else {
        data.command_sender.send(format!("SM_SEND:CMD_BTN:{}", aircraft_var)).expect("ERROR SENDING MESSAGE");
    }


    HttpResponse::Ok().body("ok")
}


#[get("/touch_event")]
async fn touch_event(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    // let hwnd = qs.clone().get("hwnd").unwrap_or("0")
    //     .parse::<isize>().unwrap_or(0);
    let x_pos = qs.clone().get("x_pos").unwrap_or("0").parse::<u16>().unwrap_or(0);
    let y_pos = qs.clone().get("y_pos").unwrap_or("0").parse::<u16>().unwrap_or(0);
    let sleep_ms: u64 = qs.clone().get("sleep_ms").unwrap_or("100").
        parse::<u64>().unwrap_or(100);

    let sleep_time: core::time::Duration = core::time::Duration::from_millis(sleep_ms);

    let hwnd = data.img_sub_status.selected_hwnd.lock().unwrap().clone();


    //thread::sleep(std::time::Duration::from_millis(4000));
    debug_logger::log(&*format!("Touch event: x:{} y:{} hwnd:{}, sleep_ms:{}",
                                &x_pos, &y_pos, &hwnd, sleep_ms), &data.log_str);

    if hwnd != 0 {
        unsafe {
            let hwnda_to_use: HWND = mem::transmute(hwnd);
            let mut original_pos = POINT { x: 0, y: 0 };
            if let Ok(_) = GetCursorPos(&mut original_pos as *mut POINT) {
                debug_logger::log(&*format!("Original cursor position: x:{} y:{}", &original_pos.x, &original_pos.y), &data.log_str);

                // Convert client coordinates (x, y) to screen coordinates
                let mut click_point = POINT { x: x_pos as i32, y: y_pos as i32 };
                let _ = ClientToScreen(hwnda_to_use, &mut click_point);
                debug_logger::log(&*format!("Target ursor position: x:{} y:{}", &click_point.x, &click_point.y), &data.log_str);

                let focused_window = GetForegroundWindow();
                if focused_window != hwnda_to_use {
                    let _ = SetForegroundWindow(hwnda_to_use);
                    SetFocus(hwnda_to_use);
                }

                let mut window_rect: RECT = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetWindowRect(hwnda_to_use, &mut window_rect).expect("Can't get window rect");

                let sx = GetSystemMetrics(SM_CXSCREEN);
                let sy = GetSystemMetrics(SM_CYSCREEN);
                let move_x = window_rect.left + x_pos as i32;
                let move_y = window_rect.top + y_pos as i32;
                let absolute_x = click_point.x * 65536 / sx;
                let absolute_y = click_point.y * 65536 / sy;
                debug_logger::log(&*format!("WNDOW POS: x:{} y:{}", &window_rect.left, &window_rect.top), &data.log_str);
                debug_logger::log(&*format!("MOVING TO: x:{} y:{}", &move_x, &move_y), &data.log_str);

                SetCursorPos(move_x, move_y).expect("Can't set cursor pos");
                thread::sleep(sleep_time);
                mouse_event(MOUSEEVENTF_LEFTDOWN
                            , absolute_x, absolute_y, 0, 0);
                thread::sleep(sleep_time);
                mouse_event(MOUSEEVENTF_LEFTUP
                            , absolute_x, absolute_y, 0, 0);


                thread::sleep(core::time::Duration::from_millis(50));
                SetCursorPos(original_pos.x, original_pos.y).expect("Can't set cursor pos");
            }
        }
        return HttpResponse::Ok().body("ok");
    }
    HttpResponse::Ok().body("not executed")
}

#[get("/var_test")]
async fn var_test(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let btn_id = qs.clone().get("btn").unwrap_or("").to_string();


    if btn_id.contains(">") || btn_id.contains("K:") || btn_id.contains("H:") {
        data.command_sender.send(format!("SM_SEND:CUSTOM_WASM:{}", btn_id)).expect("ERROR SENDING MESSAGE");
    } else {
        data.command_sender.send(format!("SM_SEND:CMD_BTN:{}", btn_id)).expect("ERROR SENDING MESSAGE");
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
    ImageProcess::hide_all();

    HttpResponse::Ok().body("ok")
}


#[get("/get_windows")]
async fn get_windows(data: web::Data<AppState>) -> HttpResponse {
    let mut state_instruments = data.instrument_list.lock().unwrap();
    let conf = data.config.lock().unwrap();

    let popout_lst = data.addon_config.popout_list();
    let aircraft: String = comm_sender::get_aircraft(&data.command_sender, &data.comm_receiver);


    let mut saved = data.current_aircraft.lock().unwrap();
    *saved = aircraft.clone();
    drop(saved);
    let sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap().clone();
    let wndows = ImageProcess::start(Option::from(conf.auto_hide),
                                     Option::from(sub_hwnd));
    let resp = ImageProcess::window_to_string(&wndows);
    for img in &wndows {
        if img.instrument == "MCDU" || popout_lst.contains(&img.instrument) || wndows.len() == 1 {
            let mut instr_search = data.img_sub_status.instrument_search.lock().unwrap();
            if img.instrument != crate::image_process::UNKNOWN_TITLE &&
                img.instrument != crate::image_process::MCDU_TITLE{
                *instr_search = img.instrument.clone();
            } else {
                *instr_search = aircraft.clone();
            }
            drop(instr_search);
            let mut sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap();
            *sub_hwnd = img.hwnd;
            let find_crop = data.addon_config.calculate_crop(&aircraft,
                                                             POPOUT_WIDTH, POPOUT_HEIGHT);
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


#[get("/get_simvars")]
async fn get_simvars(data: web::Data<AppState>) -> HttpResponse {
    let resp: String = comm_sender::get_vars(data.command_sender.clone(), data.comm_receiver.clone());
    return HttpResponse::Ok()
        .body(resp);
}

#[get("/get_simvar")]
async fn get_simvar(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let simvar = qs.get("var").unwrap_or("");

    let resp: String = comm_sender::get_var(simvar, data.command_sender.clone(), data.comm_receiver.clone());
    return HttpResponse::Ok()
        .body(resp);
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
        if img.hwnd == hw_id {
            let mut instr_search = data.img_sub_status.instrument_search.lock().unwrap();
            if img.instrument != crate::image_process::UNKNOWN_TITLE &&
                img.instrument != crate::image_process::MCDU_TITLE{
                *instr_search = img.instrument.clone();
            } else {
                *instr_search = aircraft.clone();
            }
            drop(instr_search);
            
            let mut sub_hwnd = data.img_sub_status.selected_hwnd.lock().unwrap();
            *sub_hwnd = img.hwnd;
            let find_crop = data.addon_config.calculate_crop(&aircraft,
                                                             POPOUT_WIDTH, POPOUT_HEIGHT);
            let mut crop = data.img_sub_status.display_crop.lock().unwrap();
            *crop = find_crop;
            let mut state_instruments = data.instrument_list.lock().unwrap();
            *state_instruments = wndows;
            
            
            return HttpResponse::Ok()
                .body("ok");
        }
    }

    HttpResponse::Ok()
        .body("error")
}

pub struct MyWs {
    pub command_receiver: crossbeam_channel::Receiver<String>,
    pub comm_sender: crossbeam_channel::Sender<String>,
    pub img_subscribers: Arc<Mutex<Vec<Addr<MyWs>>>>,
    pub sub_started: Arc<Mutex<bool>>,
    pub sub_hwnd: Arc<Mutex<isize>>,
    pub img_crop: Arc<Mutex<[[i32; 2]; 2]>>,
    pub config: Arc<Mutex<ConfigHandler>>,
    pub log_str: Option<Arc<Mutex<String>>>,
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
                    let log_inner = debug_logger::clone_log(&self.log_str);
                    thread::spawn(move || {
                        debug_logger::log("Spawning recv thread...", &log_inner);
                        loop {
                            let value = rx.recv().expect("Unable to receive from channel");

                            // COMMUNICATION ON THE CHANNELS:
                            // CloseBridge  => send close cmd
                            // SM_SEND:CMD_BTN:EXAMPLE_LVAR send msg to Simconnector to press the EXAMLE_LVAR btn
                            if value == "CloseBridge" {
                                debug_logger::log("Sending bridge CLOSE command", &log_inner);
                                addr.do_send(StringConnectMessage("CLOSE".to_string()));
                                break;
                            }
                            if value == "BridgeStatus" {
                                debug_logger::log("Sending bridge STATUS command", &log_inner);

                                addr.do_send(StringConnectMessage("STATUS".to_string()));
                            } else if value == "GetAircraft" {
                                debug_logger::log("Sending bridge GET_AIRCRAFT command", &log_inner);
                                addr.do_send(StringConnectMessage("GET_AIRCRAFT".to_string()));
                            } else if value.contains("SM_SEND:") {
                                //let cmnd: &str = value.split(":").collect::<Vec<&str>>()[1];
                                debug_logger::log(&*format!("Sending bridge SM_SEND:{}", &value), &log_inner);

                                addr.do_send(StringConnectMessage(value.replace("SM_SEND:", "")))
                            }
                        }
                        debug_logger::log("Recv thread exiting...", &log_inner);
                    });
                    ctx.text("CONNECTED");
                } else if text == "IMAGESUBSCRIBE" {
                    debug_logger::log("New IMAGE_SUBSCRIBE", &self.log_str);

                    let mut started = self.sub_started.lock().unwrap();
                    self.img_subscribers.lock().unwrap().push(ctx.address());

                    if !*started {
                        debug_logger::log("Adding subscription thread...", &self.log_str);

                        *started = true;
                        drop(started);
                        let sbs = Arc::clone(&self.img_subscribers);
                        let cnfig_inner = Arc::clone(&self.config);
                        let hwnd = Arc::clone(&self.sub_hwnd);
                        let crop = Arc::clone(&self.img_crop);
                        let inner_log = debug_logger::clone_log(&self.log_str);
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
                                            debug_logger::log("Removing a subscriber from img thread",
                                                &inner_log);
                                            subs_locked.remove(i);
                                        } else {
                                            i += 1;
                                        }
                                    }
                                    inner_subs = subs_locked.clone();
                                    inner_hwnd = Arc::clone(&hwnd).lock().unwrap().clone();
                                    inner_crop = Arc::clone(&crop).lock().unwrap().clone();
                                }
                                if inner_subs.len() > 0 {
                                    let img =
                                        match ImageProcess::capture_instrument(
                                            inner_hwnd, inner_crop) {
                                            Ok(res) => {res}
                                            Err(e) => {
                                                debug_logger::log(
                                                    format!("Cant capture instrument!: {}", e).as_str(),
                                                    &inner_log);
                                                vec![]
                                            }
                                        };

                                    for i in 0..inner_subs.len() {
                                        if inner_hwnd != 0 && img.len() > 0 {
                                            match inner_subs[i].try_send(BinaryMessage(img.clone())) {
                                                Ok(_) => {},
                                                Err(e) => {
                                                    debug_logger::log(
                                                        format!("Cant send img from sub thread: {}", e).as_str(),
                                                        &inner_log);
                                                    
                                                    
                                                },
                                            }

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
    debug_logger::log("ws_index, starting communications...", &data.log_str);
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
        log_str: debug_logger::clone_log(&data.log_str),
    }, &req, stream);

    resp
}

#[actix_web::main]
pub async fn main(log_str: Option<Arc<Mutex<String>>>) -> std::io::Result<()> {
    debug_logger::log("Initializing http server...", &log_str);

    let mut config = ConfigHandler::init(debug_logger::clone_log(&log_str));
    config.read_config();
    let (s, r) = bounded::<String>(0);
    let (sc, rc) = bounded::<String>(0);
    let addon_config = AddonConfig::load(debug_logger::clone_log(&log_str)).await;
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
            instrument_search: Mutex::from("".to_string()),
        },
        addon_config,
        log_str,
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
            .service(var_test)
            .service(get_simvars)
            .service(get_simvar)
            .service(touch_event)
            .service(Files::new("/static", static_path.clone()))
            .route("/ws/", web::get().to(ws_index))
        //.service(jpeg_test)
    })
        .bind(("0.0.0.0", 5273))?
        .run()
        .await
}