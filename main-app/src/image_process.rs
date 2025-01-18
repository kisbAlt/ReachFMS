use std::{mem};
use std::sync::{Arc, Mutex};
use fltk::draw::width;
use image::{RgbaImage};
use win_screenshot::prelude::*;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, GetWindow, GetWindowRect, GW_OWNER, SetWindowPos, SM_CYVIRTUALSCREEN, SWP_NOACTIVATE, SWP_NOREDRAW, SWP_NOZORDER};
use crate::debug_logger;

#[derive(Serialize, Deserialize, Clone)]
pub struct InstrumentRgb {
    pub hwnd: isize,
    pub height: u16,
    pub width: u16,
    pub instrument: String,
    pub jpeg_bytes: Vec<u8>,
    //[[crop_x,cropy_y],[crop_w, crop_h]]
    pub excluded: bool,
    pub auto_hide: bool,
    pub selected: bool,
    pub(crate) crop: [[i32; 2]; 2],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InstrumentResponse {
    pub hwnd: isize,
    pub height: u16,
    pub width: u16,
    pub instrument: String,
    pub excluded: bool,
    pub auto_hide: bool,
    pub selected: bool,
    pub jpeg_bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct PopOutWindow {
    pub title: String,
    pub hwnd: isize,
}


const USING: Using = Using::PrintWindow;
const AREA: Area = Area::ClientOnly;
const MSFS_TITLE: &str = "Microsoft Flight Simulator - ";
const DEFAULT_TITLE: &str = "WASMINSTRUMENT";
pub(crate) const UNKNOWN_TITLE: &str = "UNKNOWN";
pub(crate) const MCDU_TITLE: &str = "FMS";

pub const POPOUT_WIDTH: i32 = 700;
pub const POPOUT_HEIGHT: i32 = 700;


pub struct ImageProcess {}

pub struct WindowCapture {
    pub buf: Vec<u8>,
    pub width: u16,
    pub height: u16
}




impl ImageProcess {
    // pub fn init() -> Self {
    //     Self {}
    // }
    pub fn start(auto_hide: Option<bool>,
                 selected_hwnd: Option<isize>, log_str: &Option<Arc<Mutex<String>>>
    ) -> Vec<InstrumentRgb> {
        let mut rgb_list: Vec<InstrumentRgb> = Vec::new();
        let av_hw = match ImageProcess::find_popup_windows() {
            Ok(res) => {
                debug_logger::log(format!("Found pop-outs: {}", 
                                          serde_json::to_string(&res).unwrap()).as_str(), &log_str);
                res
            }
            Err(_) => {
                debug_logger::log("Can't find MSFS window! Found window handles:", &log_str);
                let ls = window_list().unwrap();
                for ls_c in ls {
                    debug_logger::log(format!("HWND: {}, TITLE: '{}'",
                                              &ls_c.hwnd, &ls_c.window_name).as_str(), &log_str);
                }
                //show_warning_dialog("Cant find MSFS window!");
                vec![]
            }
        };

        if av_hw.len() == 0 {
            return rgb_list;
        }


        for hw in av_hw.iter() {
            let capture = ImageProcess::capture_instrument(
                hw.hwnd.clone(), [[0, 0], [0, 0]]).unwrap();


            // let buf = capture_window_ex(hw.hwnd.clone(), Using::PrintWindow,
            //                             Area::ClientOnly, None, None).unwrap();
            // let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();

            let current = InstrumentRgb {
                hwnd: hw.hwnd.clone(),
                height: capture.height,
                width: capture.width,
                crop: ImageProcess::find_crop_for_instruments(capture.width as u32, capture.height as u32),
                instrument: hw.title.clone(),
                jpeg_bytes: capture.buf,
                auto_hide: true,
                excluded: false,
                selected: false,
            };
            rgb_list.push(current);
        }
        if av_hw.len() == 1 || selected_hwnd.is_some() {
            match auto_hide {
                None => {}
                Some(hide_res) => unsafe {
                    let hw: isize;
                    if av_hw.len() == 1 {
                        hw = av_hw[0].hwnd;
                    } else {
                        hw = selected_hwnd.unwrap_or(0);
                    }
                    for rgb in &mut rgb_list {
                        if rgb.hwnd == hw {
                            debug_logger::log(format!("Setting selected hwnd: {}",
                                                      &rgb.hwnd).as_str(), &log_str);
                            if rgb.instrument == UNKNOWN_TITLE {
                                rgb.instrument = MCDU_TITLE.to_string();
                            }
                            rgb.selected = true;
                            if hide_res {
                                ImageProcess::hide_window(hw)
                            } else {
                                let wsize = ImageProcess::get_window_pos(hw);
                                ImageProcess::move_window(hw, wsize.top, wsize.left, POPOUT_WIDTH, POPOUT_HEIGHT);
                            }
                        }
                    }
                }
            }
        }
        ;
        rgb_list
    }

    fn find_crop_for_instruments(width: u32, height: u32) -> [[i32; 2]; 2] {
        let crop: [[i32; 2]; 2] = [[0, 0], [width as i32, height as i32]];


        return if width == height {
            crop
        } else if width > height {
            let crop_pixels = width - height;
            [[(crop_pixels / 2) as i32, 0], [height as i32, height as i32]]
        } else {
            let crop_pixels = height - width;
            [[0, (crop_pixels / 2) as i32], [width as i32, width as i32]]
        };
    }

    pub unsafe fn hide_window(hwnd_in: isize) {
        if hwnd_in != 0 {
            let hwnda_to_move: HWND = mem::transmute(hwnd_in);
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            SetWindowPos(hwnda_to_move, hwnda_to_move,
                         0, height + 50, POPOUT_WIDTH, POPOUT_HEIGHT, SWP_NOREDRAW | SWP_NOZORDER | SWP_NOACTIVATE).expect("Cant set window pos");
        }
    }


    pub fn get_window_parent(hwnd: isize) -> HWND {
        unsafe {
            let hwnda_to_move: HWND = mem::transmute(hwnd);

            let owner = GetWindow(hwnda_to_move, GW_OWNER);

            //let prnt = GetAncestor(hwnda_to_move, GA_PARENT);
            return owner;
        }
    }

    pub unsafe fn get_window_pos(hwnd_in: isize) -> RECT {
        let hwnda_to_move: HWND = mem::transmute(hwnd_in);
        let mut rrval: RECT = RECT::default();
        GetWindowRect(hwnda_to_move, &mut rrval).expect("Cant get window rect");
        rrval
    }

    pub unsafe fn move_window(hwnd_in: isize, move_top: i32, move_left: i32, move_width: i32, move_height: i32) {
        let hwnda_to_move: HWND = mem::transmute(hwnd_in);
        SetWindowPos(hwnda_to_move, hwnda_to_move,
                     move_left, move_top, move_width, move_height,
                     SWP_NOREDRAW | SWP_NOZORDER | SWP_NOACTIVATE).expect("Cant move window!");
    }

    pub fn get_sim_hwnd(window_ls: &Vec<HwndName>) -> isize {
        //let mut current_hwnd: HWND;
        for i in window_ls {
            if i.window_name.contains(MSFS_TITLE) {
                return i.hwnd;
            }
        }
        return 0;
    }


    pub fn find_popup_windows() -> Result<Vec<PopOutWindow>, bool> {
        let mut process_ls = Vec::<PopOutWindow>::new();
        let ls = window_list().unwrap();
        let fs_hwnd = Self::get_sim_hwnd(&ls);
        if fs_hwnd == 0 {
            return Err(false);
        }

        for i in ls {
            let prnt_id = Self::get_window_parent(i.hwnd);

            if i.window_name == DEFAULT_TITLE || prnt_id.0 == fs_hwnd {
                let hwnd = i.hwnd;
                process_ls.push(PopOutWindow {
                    hwnd,
                    title: match i.window_name.as_str() {
                        DEFAULT_TITLE => { UNKNOWN_TITLE.to_string() }
                        _ => { i.window_name }
                    },
                })
            }
        }
        return Ok(process_ls);
    }
    pub fn window_to_string(input: &Vec<InstrumentRgb>) -> String {
        let mut string_instruments: Vec<InstrumentResponse> = Vec::new();

        for i in input {
            string_instruments.push(InstrumentResponse {
                hwnd: i.hwnd,
                height: i.height,
                width: i.width,
                instrument: i.instrument.clone(),
                auto_hide: i.auto_hide,
                excluded: i.excluded,
                jpeg_bytes: i.jpeg_bytes.clone(),
                selected: i.selected,
            })
        }
        return serde_json::to_string(&string_instruments).unwrap();
    }


    pub fn capture_instrument(hw_id: isize, crop: [[i32; 2]; 2]) -> Result<WindowCapture, u8> {
        let cropxy: Option<[i32; 2]> = match crop[0] {
            [0, 0] => Option::None,
            _ => { Option::from(crop[0]) }
        };

        let cropwh: Option<[i32; 2]> = match crop[1] {
            [0, 0] => Option::None,
            _ => { Option::from(crop[1]) }
        };


        let buf = match capture_window_ex(hw_id, USING, AREA,
                                          cropxy, cropwh) {
            Ok(tempbuf) => tempbuf,
            Err(..) => {
                return Err(0);
            }
        };
        // let ref mut w = BufWriter::new(Cursor::new(Vec::new()));
        let mut outputbuf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut outputbuf, buf.width, buf.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            writer.write_image_data(&buf.pixels).unwrap(); // Save
        }
        let capture = WindowCapture{
            buf: outputbuf,
            width: buf.width as u16,
            height: buf.height as u16
        };
        Ok(capture)
    }

    pub fn restore_all() -> bool {
        let poputs = match ImageProcess::find_popup_windows() {
            Ok(res) => { res }
            Err(_) => {
                return false
            }
        };
        for popout in poputs {
            unsafe { ImageProcess::move_window(popout.hwnd, 0, 0, POPOUT_WIDTH, POPOUT_HEIGHT) }
        }
        return true;
    }

    pub fn hide_all() -> bool {
        let poputs = match ImageProcess::find_popup_windows() {
            Ok(res) => { res }
            Err(_) => {
                return false
            }
        };
        for popout in poputs {
            unsafe { Self::hide_window(popout.hwnd) }
        }
        return true;
    }
}