#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_communicator;
mod image_process;
mod http_streamer;
mod config_handler;
mod comm_sender;
mod addon_config;

use std::{thread, time};
use std::os::windows::process::CommandExt;
use std::process::{Command};
use fltk::{enums::{Color, Font, FrameType, Cursor}, prelude::*, *};
use fltk::app::{screen_size};
use fltk::enums::{Event};
use crate::addon_config::AddonConfig;
use crate::config_handler::ConfigHandler;
use crate::image_process::ImageProcess;


// TODO:
// add window excluding
// add multiple localhost option display
// restore windows on app exit or server stop

#[derive(Copy, Clone)]
enum Message {
    Start,
    Url,
    Continue,
    ShowAllUrl,
    CloseAllUrl,
}

struct McduApp {
    app: app::App,
    main_win: window::Window,
    first_window: window::Window,
    status_text: frame::Frame,
    start_button: button::Button,
    receiver: app::Receiver<Message>,
    bridge_started: bool,
    qr_frame: frame::Frame,
    url_text: button::Button,
    get_started: frame::Frame,
    url_not_working: button::Button,
    all_ip_pack: group::Pack,
    adress_list: frame::Frame,
}

impl McduApp {
    pub fn new() -> Self {
        let app = app::App::default();
        let (s, receiver) = app::channel();
        let monitor_size = screen_size();
        let center_pos: (i32, i32) = (monitor_size.0 as i32 / 2 - 300, monitor_size.1 as i32 / 2 - 300);
        let mut main_win = window::Window::new(center_pos.0, center_pos.1,
                                               600, 400, "A320 Remote MCDU");
        main_win.set_color(Color::from_rgb(21, 26, 32));
        let mut start_button = button::Button::new(0, 350, 160, 40, "Start server").center_x(&main_win);

        let qr_frame = frame::Frame::new(0, 150, 200, 200, "").center_x(&main_win);

        let mut welcome_text = frame::Frame::new(0, 14, 60, 40, "")
            .center_x(&main_win)
            .with_label("Remote MCDU for\nthe Fenix A320");
        let mut status_text = frame::Frame::new(0, 60, 140, 20, "")
            .center_x(&main_win)
            .with_label("Status: stopped");

        let mut url_text = button::Button::new(0, 80, 250, 20, "")
            .center_x(&main_win)
            .with_label(&ConfigHandler::get_localhost());

        let mut url_not_working = button::Button::new(0, 110, 250, 20, "")
            .center_x(&main_win)
            .with_label("Displayed ip adress is not correct?");

        let version_string: String = format!("v{}", env!("CARGO_PKG_VERSION"));
        let mut version_label = frame::Frame::new(5, 5, 30, 20, "")
            .with_label(&*version_string);

        let mut get_started = frame::Frame::new(0, 150, 140, 100, "")
            .center_x(&main_win)
            .with_label("To get started, simply click the 'Start server' button below to launch the server.\nOnce the server is up and running, you can connect to it from any device\non your home network by scanning the QR code or by opening a web browser\nand navigating to the displayed IP address and port.");

        let mut all_ip_pack = group::Pack::new(0, 130, 250, 210, "").center_x(&main_win);
        all_ip_pack.set_type(group::PackType::Vertical);
        all_ip_pack.set_spacing(10);
        let mut url_list_title = frame::Frame::new(0, 0, 140, 20, "")
            .center_x(&main_win)
            .with_label("Try these url adresses too:");
        let mut adress_list = frame::Frame::new(0, 0, 140, 150, "")
            .center_x(&main_win)
            .with_label("iplist");
        let mut close_all_ip = button::Button::new(0, 0, 50, 20, "").center_x(&main_win).with_label("Close");
        all_ip_pack.end();


        let mut logo_frame = frame::Frame::new(175, 25, 30, 30, "");
        let mut logo_image = image::PngImage::from_data(include_bytes!("../../transparent-compressed.png")).unwrap();
        logo_image.scale(50, 50, true, true);
        logo_frame.set_image(Some(logo_image));

        main_win.end();
        main_win.show();

        main_win.set_callback(|_| {
            if fltk::app::event() == fltk::enums::Event::Close {
                match reqwest::blocking::get("http://localhost:5273/stop_server") {
                    Ok(..) => {
                        println!("server closed, closing the app...");
                        app::quit();
                    }
                    Err(..) => {
                        println!("server can't be reached but closing the app...");
                        app::quit();
                    }
                }
                // Which would close using the close button. You can also assign other keys to close the application
            }
        });

        app::set_visible_focus(false);

        status_text.set_label_size(18);
        status_text.set_label_color(Color::Red);
        status_text.set_label_font(Font::Helvetica);

        get_started.set_label_size(16);
        get_started.set_label_color(Color::from_rgb(169, 169, 169));
        get_started.set_label_font(Font::Helvetica);

        url_list_title.set_label_size(16);
        url_list_title.set_label_color(Color::White);
        url_list_title.set_label_font(Font::Helvetica);

        version_label.set_label_size(11);
        version_label.set_label_color(Color::White);
        version_label.set_label_font(Font::Helvetica);

        adress_list.set_label_size(16);
        adress_list.set_label_color(Color::from_rgb(0, 150, 255));
        adress_list.set_label_font(Font::Helvetica);

        close_all_ip.set_label_size(18);
        close_all_ip.set_label_color(Color::from_rgb(100, 149, 237));
        close_all_ip.set_label_font(Font::Helvetica);
        close_all_ip.handle(move |b, event| match event {
            Event::Enter => {
                b.set_label_color(Color::from_rgb(255, 69, 0));
                b.redraw();
                true
            }
            Event::Leave => {
                b.set_label_color(Color::from_rgb(100, 149, 237));
                b.redraw();
                true
            }
            Event::Push => {
                b.emit(s, Message::CloseAllUrl);
                true
            }
            _ => false,
        });
        close_all_ip.set_frame(FrameType::NoBox);

        url_text.set_label_size(18);
        url_text.set_label_color(Color::from_rgb(100, 149, 237));
        url_text.set_label_font(Font::Helvetica);
        url_text.handle(move |b, event| match event {
            Event::Enter => {
                b.set_label_color(Color::from_rgb(0, 255, 255));
                b.redraw();
                true
            }
            Event::Leave => {
                b.set_label_color(Color::from_rgb(100, 149, 237));
                b.redraw();
                true
            }
            Event::Push => {
                b.emit(s, Message::Url);
                true
            }
            _ => false,
        });
        url_text.set_frame(FrameType::NoBox);
        url_text.hide();

        url_not_working.set_label_size(15);
        url_not_working.set_label_color(Color::from_rgb(169, 169, 169));
        url_not_working.set_label_font(Font::Helvetica);
        url_not_working.handle(move |b, event| match event {
            Event::Enter => {
                b.set_label_color(Color::from_rgb(0, 255, 255));
                b.redraw();
                true
            }
            Event::Leave => {
                b.set_label_color(Color::from_rgb(169, 169, 169));
                b.redraw();
                true
            }
            Event::Push => {
                b.emit(s, Message::ShowAllUrl);
                true
            }
            _ => false,
        });
        url_not_working.set_frame(FrameType::NoBox);
        url_not_working.hide();

        welcome_text.set_label_size(18);
        welcome_text.set_label_color(Color::White);
        welcome_text.set_label_font(Font::Helvetica);
        //welcome_text.set_align(Align::Right);

        //all_ip_pack.set_color(Color::from_rgb(128, 128, 128));
        all_ip_pack.set_frame(FrameType::NoBox);
        all_ip_pack.hide();

        start_button.set_color(Color::from_rgb(47, 53, 67));
        start_button.set_label_color(Color::Green);
        start_button.set_label_font(Font::HelveticaBold);
        start_button.set_frame(FrameType::GtkDownBox);
        start_button.handle(move |b, event| match event {
            Event::Enter => {
                b.set_color(Color::from_rgb(96, 130, 182));
                b.redraw();
                true
            }
            Event::Leave => {
                b.set_color(Color::from_rgb(47, 53, 67));
                b.redraw();
                true
            }
            Event::Push => {
                b.emit(s, Message::Start);
                true
            }
            _ => false,
        });
        //start_button.emit(s, Message::Start);

        let icon_image = image::PngImage::from_data(include_bytes!("../../transparent-compressed.png")).unwrap();
        main_win.set_icon(Some(icon_image));


        let mut first_window = window::Window::new(center_pos.0, center_pos.1, 600, 400, "A320 Remote MCDU First start");
        first_window.set_color(Color::from_rgb(21, 26, 32));
        let mut continue_button = button::Button::new(0, 350, 160, 40, "Continue").center_x(&first_window);
        let mut first_welcome_text = frame::Frame::new(0, 30, 100, 40, "")
            .center_x(&first_window)
            .with_label("Welcome to the Remote A320 MCDU app!");
        let mut instructions = frame::Frame::new(0, 100, 600, 100, "")
            .with_label("1. Make sure that you have at least the SU12 update.\n\n2. You should now install the wasm module\nto your Community directory:\nThe wasm module can be found in the downloaded .rar file.\nMove the remotemcdu-wasm folder to your Community folder.");

        first_window.end();
        if !ConfigHandler::is_data_created() {
            main_win.hide();
            first_window.show();
            continue_button.set_color(Color::from_rgb(47, 53, 67));
            continue_button.set_label_color(Color::Green);
            continue_button.set_frame(FrameType::FlatBox);
            continue_button.handle(move |b, event| match event {
                Event::Enter => {
                    b.set_color(Color::from_rgb(96, 130, 182));
                    b.redraw();
                    true
                }
                Event::Leave => {
                    b.set_color(Color::from_rgb(47, 53, 67));
                    b.redraw();
                    true
                }
                Event::Push => {
                    b.emit(s, Message::Continue);
                    true
                }
                _ => false,
            });

            first_welcome_text.set_label_size(18);
            first_welcome_text.set_label_color(Color::from_rgb(80, 200, 120));
            first_welcome_text.set_label_font(Font::HelveticaBold);
            instructions.set_label_size(15);
            instructions.set_label_color(Color::White);
            instructions.set_label_font(Font::Helvetica);
        }else {
            let mut config = ConfigHandler::init();
            config.read_config();
            if config.auto_start{
                drop(config);
                s.send(Message::Start);
            }
        }




        Self {
            app,
            main_win,
            first_window,
            status_text,
            start_button,
            receiver,
            bridge_started: false,
            qr_frame,
            url_text,
            get_started,
            url_not_working,
            all_ip_pack,
            adress_list,
        }
    }

    pub fn run(mut self) {
        thread::spawn(|| {
            match http_streamer::main() {
                Ok(..) => println!("server started"),
                Err(..) => println!("error starting the server")
            };
        });
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Start => {
                        self.main_win.set_cursor(Cursor::Wait);
                        if self.bridge_started {
                            let resp = reqwest::blocking::get("http://localhost:5273/stop_server");
                            match &resp {
                                Ok(..) => {
                                    resp.unwrap().text().unwrap();
                                    self.start_button.set_label_color(Color::Green);
                                    self.start_button.set_label("Start server");

                                    self.status_text.set_label("Status: stopped");
                                    self.status_text.set_label_color(Color::Red);
                                    self.bridge_started = false;

                                    self.qr_frame.hide();
                                    self.url_text.hide();
                                    self.all_ip_pack.hide();
                                    self.url_not_working.hide();

                                    self.get_started.show();
                                }
                                Err(..) => {
                                    println!("Not avialable");
                                }
                            }
                        } else {
                            if api_communicator::check_bridge_process() == false {
                                let resp = reqwest::blocking::get("http://localhost:5273/start_server");
                                match &resp {
                                    Ok(..) => {
                                        resp.unwrap().text().unwrap();
                                        self.get_started.hide();

                                        self.start_button.set_label_color(Color::Red);
                                        self.start_button.set_label("Stop server");

                                        self.status_text.set_label("Status: running");
                                        self.status_text.set_label_color(Color::Green);
                                        self.bridge_started = true;
                                        //self.hpack.show();
                                        self.qr_frame.show();
                                        let mut qr_image = image::PngImage::load(config_handler::get_qr_file()).unwrap();
                                        qr_image.scale(200, 200, true, true);
                                        self.qr_frame.set_image(Some(qr_image));
                                        self.qr_frame.redraw();

                                        self.url_text.show();
                                        self.url_not_working.show();
                                    }
                                    Err(..) => {
                                        println!("Not avialable");
                                    }
                                }
                            }
                        }
                        self.main_win.set_cursor(Cursor::Default);
                    }
                    Message::Url => {
                        if let Ok(mut child) = Command::new("cmd.exe").creation_flags(0x00000008u32)
                            .arg("/C").arg("start").arg("").arg(&self.url_text.label()).spawn() {
                            thread::sleep(time::Duration::new(3, 0)); // On windows need to allow time for browser to start
                            if let Ok(..) = child.wait() {
                                println!("ok")
                            }
                        }
                    }
                    Message::Continue => {
                        self.first_window.hide();
                        self.main_win.show();
                    }
                    Message::ShowAllUrl => {
                        let mut addr_str: String = "".to_string();
                        let addr_ls = ConfigHandler::get_all_local_ip();
                        for addr in addr_ls {
                            addr_str += &addr;
                            addr_str += &"\n".to_string();
                        }
                        self.adress_list.set_label(&addr_str);

                        self.qr_frame.hide();
                        self.all_ip_pack.show();
                        println!("{}", &addr_str)
                    }
                    Message::CloseAllUrl => {
                        self.all_ip_pack.hide();
                        self.qr_frame.show();
                    }
                }
            }
        }
    }
}

fn main() {
    // let hostfxr = nethost::load_hostfxr().unwrap();
    // let context = hostfxr.initialize_for_dotnet_command_line(pdcstr!("Test.dll")).unwrap();
    // let result = context.run_app().value();
    let addon_config = AddonConfig::load();
    let a = McduApp::new();
    a.run();
}