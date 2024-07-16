#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_communicator;
mod image_process;
mod http_streamer;
mod config_handler;
mod comm_sender;
mod addon_config;
mod mobiflight_installer;
mod debug_logger;

use std::{thread, time};
use std::os::windows::process::CommandExt;
use std::process::{Command};
use fltk::{enums::{Color, Font, FrameType, Cursor}, prelude::*, *};
use fltk::app::{screen_size};
use fltk::enums::{Event};
use crate::config_handler::ConfigHandler;
use crate::image_process::ImageProcess;


// TODO:
// logging to file

#[derive(Copy, Clone)]
enum Message {
    Start,
    Url,
    Continue,
    ShowAllUrl,
    CloseAllUrl,
    AutomaticInstall,
    ManualInstall,
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
    install_manually: button::Button,
    install_automatically: button::Button,
    continue_button: button::Button,
    instructions: frame::Frame,
    mobi_text: frame::Frame,
    first_welcome_text: frame::Frame,
}

impl McduApp {
    pub fn new() -> Self {
        let app = app::App::default();
        let (s, receiver) = app::channel();
        let monitor_size = screen_size();
        let center_pos: (i32, i32) = (monitor_size.0 as i32 / 2 - 300, monitor_size.1 as i32 / 2 - 300);
        let mut main_win = window::Window::new(center_pos.0, center_pos.1,
                                               600, 400, "ReachFMS");
        main_win.set_color(Color::from_rgb(21, 26, 32));
        let mut start_button = button::Button::new(0, 350, 160, 40, "Start server").center_x(&main_win);

        let qr_frame = frame::Frame::new(0, 150, 200, 200, "").center_x(&main_win);

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


        let mut logo_frame = frame::Frame::new(175, 25, 30, 30, "").center_x(&main_win);
        let logo_image = image::PngImage::from_data(include_bytes!("../../svg/reachfms_white220.png")).unwrap();
        //logo_image.scale(50, 50, true, true);
        logo_frame.set_image(Some(logo_image));

        main_win.end();
        main_win.show();

        main_win.set_callback(|_| {
            if fltk::app::event() == fltk::enums::Event::Close {
                ImageProcess::restore_all();
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
        main_win.set_icon(Some(icon_image.clone()));
        
        
        let mut first_window = window::Window::new(center_pos.0, center_pos.1, 600, 400, "ReachFMS First start");
        first_window.set_icon(Some(icon_image));
        first_window.set_color(Color::from_rgb(21, 26, 32));
        let mut continue_button = button::Button::new(0, 270, 160, 40, "Continue").center_x(&first_window);
        let mut install_automatically = button::Button::new(120, 200, 160, 40, "Install automatically");
        let mut install_manually = button::Button::new(320, 200, 160, 40, "Install manually");
        let mut mobi_text = frame::Frame::new(0, 60, 400, 150, "")
            .center_x(&first_window)
            .with_label("Welcome to the ReachFMS app!");
        mobi_text.hide();

        let mut first_welcome_text = frame::Frame::new(0, 50, 100, 40, "")
            .center_x(&first_window)
            .with_label("Welcome to the ReachFMS app!");
        let mut instructions = frame::Frame::new(0, 100, 600, 100, "")
            .with_label("1. Make sure that you have at least the SU12 update.\n\n2. The mobiflight wasm module is need to be installed for the app to run.\nIn the next step this will be checked, and you can choose\nif you want it to be installed automatically");

        let mut reachfms_logo = frame::Frame::new(0, 10, 220, 39, "").center_x(&first_window);
        let mut reachfms_image = image::PngImage::from_data(include_bytes!("../../svg/reachfms_white220.png")).unwrap();
        //reachfms_image.scale(220, 39, true, true);
        reachfms_logo.set_image(Some(reachfms_image));
        
        if !ConfigHandler::is_data_created() {

            // first window starts here
            install_automatically.handle(move |b, event| match event {
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
                    b.emit(s, Message::AutomaticInstall);
                    true
                }
                _ => false,
            });

            install_manually.handle(move |b, event| match event {
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
                    b.emit(s, Message::ManualInstall);
                    true
                }
                _ => false,
            });

            install_automatically.hide();
            install_manually.hide();



            first_window.end();
            main_win.hide();
            first_window.show();
            install_automatically.set_color(Color::from_rgb(47, 53, 67));
            install_automatically.set_label_color(Color::Green);
            install_automatically.set_frame(FrameType::FlatBox);


            install_manually.set_color(Color::from_rgb(47, 53, 67));
            install_manually.set_label_color(Color::Green);
            install_manually.set_frame(FrameType::FlatBox);

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

            mobi_text.set_label_size(17);
            mobi_text.set_label_color(Color::White);
            mobi_text.set_label_font(Font::HelveticaBold);


            if mobiflight_installer::mobiflight_installed() {
                mobi_text.set_label("It seems like that you have the mobiflight wasm module installed. \n The mobiflight wasm module is needed for the app to function properly. \n If you installed the module a a while ago consider updating it.")
            } else {
                mobi_text.set_label("It seems like that you don't have mobiflight installed.\n The mobiflight wasm module is needed for the app to function properly.\n You can install the event_module manuall, or the app can do it automatically.")
            }
        } else {
            let mut config = ConfigHandler::init();
            config.read_config();
            if config.auto_start {
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
            install_automatically,
            install_manually,
            continue_button,
            instructions,
            mobi_text,
            first_welcome_text,
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
                            ImageProcess::restore_all();
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
                        if !self.mobi_text.visible() && self.instructions.visible() {
                            if !mobiflight_installer::mobiflight_installed() {
                                self.continue_button.hide();
                                self.install_automatically.show();
                                self.install_manually.show();
                            }
                            self.instructions.hide();
                            self.mobi_text.show();
                            self.first_welcome_text.set_label("ReachFMS Setup");
                        } else if self.continue_button.label().contains("Check") {
                            if !mobiflight_installer::mobiflight_installed() {
                                self.mobi_text.set_label("It seems like the wasm module is still not installed. \n\n Please try again!");
                                self.mobi_text.redraw_label();
                                self.mobi_text.redraw()
                            } else {
                                self.first_window.hide();
                                self.main_win.show();
                            }
                        }else {
                            self.first_window.hide();
                            self.main_win.show();
                        }
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
                    Message::AutomaticInstall => {
                        self.install_automatically.hide();
                        self.install_manually.hide();
                        self.install_automatically.hide();
                        self.install_manually.hide();
                        self.mobi_text.set_label("The mobiflight wasm module is now installing automatically. \n\n Please wait!");
                        self.mobi_text.redraw_label();
                        self.mobi_text.redraw();
                        
                        mobiflight_installer::install_mobiflight();

                        self.first_window.hide();
                        self.main_win.show();
                    }
                    Message::ManualInstall => {
                        self.install_automatically.hide();
                        self.install_manually.hide();
                        
                        self.continue_button.set_label("Check installation");
                        self.mobi_text.set_label("Now install the wasm module to your community folder.\n Once it is installed, click the Check button to continue.");
                        self.continue_button.show();
                        
                        if let Ok(mut child) = Command::new("cmd.exe").creation_flags(0x00000008u32)
                            .arg("/C").arg("start").arg("").arg("https://github.com/MobiFlight/MobiFlight-WASM-Module/releases/latest/").spawn() {
                            thread::sleep(time::Duration::new(3, 0)); // On windows need to allow time for browser to start
                            if let Ok(..) = child.wait() {
                                println!("ok")
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    //mobiflight_installer::download_package();

    let a = McduApp::new();
    a.run();
}