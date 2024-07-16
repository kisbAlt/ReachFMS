use std::{mem};
use image::{RgbaImage};
use win_screenshot::prelude::*;
use serde::{Deserialize, Serialize};
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

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
    pub jpeg_bytes: Vec<u8>,
}


const USING: Using = Using::PrintWindow;
const AREA: Area = Area::ClientOnly;

pub struct ImageProcess {}

impl ImageProcess {
    // pub fn init() -> Self {
    //     Self {}
    // }
    pub fn start(auto_hide: Option<bool>,
                 selected_hwnd: Option<isize>) -> Vec<InstrumentRgb> {
        let av_hw = ImageProcess::find_popup_windows();
        let mut rgb_list: Vec<InstrumentRgb> = Vec::new();
        if av_hw.len() == 0 { return rgb_list; }

        for hw in av_hw.iter() {
            let buf2 = ImageProcess::capture_instrument(
                hw.clone(), [[0, 0], [0, 0]]);


            let buf = capture_window_ex(hw.clone(), Using::PrintWindow,
                                        Area::ClientOnly, None, None).unwrap();
            let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();

            let current = InstrumentRgb {
                hwnd: *hw,
                height: img.height() as u16,
                width: img.width() as u16,
                crop: ImageProcess::find_crop_for_instruments(img.width(), img.height()),
                instrument: "UNKNOWN".to_string(),
                jpeg_bytes: buf2.unwrap(),
                auto_hide: true,
                excluded: false,
            };
            rgb_list.push(current);
        }
        if av_hw.len() == 1 || selected_hwnd.is_some() {
            match auto_hide {
                None => {}
                Some(hide_res) => unsafe {
                    let mut hw: isize = 0;
                    if av_hw.len() == 1 {
                        hw = av_hw[0];
                    } else {
                        hw = selected_hwnd.unwrap_or(0);
                    }
                    println!("selected hwnd: {}", hw);
                    for rgb in &mut rgb_list {
                        if rgb.hwnd == hw {
                            println!("setting MCDU:  {}", rgb.hwnd);
                            rgb.instrument = "MCDU".parse().unwrap();

                            if hide_res {
                                ImageProcess::hide_window(hw)
                            } else {
                                let wsize = ImageProcess::get_window_pos(hw);
                                ImageProcess::move_window(hw, wsize.top, wsize.left, 700, 700);
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
                         0, height + 50, 700, 700, SWP_NOREDRAW | SWP_NOZORDER | SWP_NOACTIVATE).expect("Cant set window pos");
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

    pub fn find_popup_windows() -> Vec<isize> {
        let mut process_ls = Vec::<isize>::new();
        println!("wndow llist");
        let ls = window_list().unwrap();
        println!("wndow llist done");
        for i in ls {
            if i.window_name == "WASMINSTRUMENT" {
                let hwnd = i.hwnd;
                process_ls.push(hwnd)
            }
        }
        process_ls
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
            })
        }
        return serde_json::to_string(&string_instruments).unwrap();
    }


    pub fn capture_instrument(hw_id: isize, crop: [[i32; 2]; 2]) -> Result<Vec<u8>, u8> {
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
        let mut outputasd = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut outputasd, buf.width, buf.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            writer.write_image_data(&buf.pixels).unwrap(); // Save
        }
        Ok(outputasd)
    }

    pub fn restore_all() {
        let poputs = ImageProcess::find_popup_windows();

        println!("moving windows");
        for popout in poputs {
            unsafe { ImageProcess::move_window(popout, 0, 0, 700, 700) }
        }
    }
}