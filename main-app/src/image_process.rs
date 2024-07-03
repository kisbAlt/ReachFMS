use std::io::{BufWriter, Cursor};
use std::{mem};
use image::{ImageFormat, Luma, RgbaImage};
use win_screenshot::prelude::*;
use serde::{Deserialize, Serialize};
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct InstrumentRgb {
    pub hwnd: isize,
    height: u16,
    width: u16,
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

pub struct InstrumentPosition {
    top: i32,
    left: i32,
    width: i32,
    height: i32,
    hwnd: isize,
}

const USING: Using = Using::PrintWindow;
const AREA: Area = Area::ClientOnly;

pub struct ImageProcess {}

impl ImageProcess {
    // pub fn init() -> Self {
    //     Self {}
    // }
    pub fn start(previous_state: Option<Vec<InstrumentRgb>>, autohide: bool, force_refresh: bool) -> Vec<InstrumentRgb> {
        let av_hw = ImageProcess::find_popup_windows();
        let mut rgb_list: Vec<InstrumentRgb> = Vec::new();
        if av_hw.len() == 0 { return rgb_list; }

        // let mut local_prev_state: Vec<InstrumentRgb> = Vec::new();
        // if previous_state.is_some() && !force_refresh {
        //     local_prev_state = previous_state.unwrap();
        // }

        let mut needs_update: Vec<InstrumentPosition> = Vec::new();
        for hw in av_hw.iter() {
            unsafe {
                let win_pos = ImageProcess::get_window_pos(hw.clone());
                needs_update.push(InstrumentPosition {
                    top: win_pos.top,
                    left: win_pos.left,
                    width: win_pos.right - win_pos.left,
                    height: win_pos.bottom - win_pos.top,
                    hwnd: *hw,
                });
                ImageProcess::hide_window(hw.clone());
            }
        }
        // if needs_update.len() > 0 {
        //     std::thread::spawn(move || {
        //         api_communicator::brightness_full();
        //     }).join().unwrap();
        // }
        //thread::sleep(std::time::Duration::from_millis(500));
        for current_instr in needs_update {
            let buf = capture_window_ex(current_instr.hwnd.clone(), Using::PrintWindow,
                                        Area::ClientOnly, None, None).unwrap();
            let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();

            let resized = image::imageops::resize(&img, 120,
                                                  120, image::imageops::FilterType::Nearest);
            
            let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
            resized.write_to(&mut buffer, ImageFormat::Jpeg).unwrap();
            let bytes: Vec<u8> = buffer.into_inner().unwrap().into_inner();


            if !autohide {
                unsafe {
                    ImageProcess::move_window(current_instr.hwnd.clone(), current_instr.top,
                                              current_instr.left, current_instr.width, current_instr.height);
                }
            }
            let instr: String = "PFD".to_string();
            let current = InstrumentRgb {
                hwnd: current_instr.hwnd,
                height: img.height() as u16,
                width: img.width() as u16,
                crop: ImageProcess::find_crop_for_instruments(
                    instr.clone(), img.width(), img.height()),
                instrument: instr,
                jpeg_bytes: bytes,
                auto_hide: true,
                excluded: false,
            };
            rgb_list.push(current);
        }
        rgb_list
    }
    // fn get_system(pxels: Pixels<Rgba<u8>>, sure: bool) -> String {
    //     return if sure {
    //         for p in pxels {
    //             let rgba: [u8; 4] = p.0;
    //             let [r, g, b, _a] = rgba;
    //             let clr: [u8; 3] = [r, g, b];
    //             if clr == [111, 50, 0] || clr == [110, 51, 0] {
    //                 return "PFD".to_string();
    //             } else if clr == [70, 60, 35] || clr == [70, 61, 36] || clr == [71, 60, 35] {
    //                 return "SIS".to_string();
    //             } else if clr == [172, 201, 253] {
    //                 return "L_ECAM".to_string();
    //             } else if clr == [246, 222, 179] || clr == [246, 222, 179] || clr == [245, 223, 180] {
    //                 return "SEGMENTS".to_string();
    //             }
    //         }
    //         "".to_string()
    //     } else {
    //         let mut has_nd_yellow = false;
    //         // let mut num_of_white_row: u8 = 0;
    //         // let mut max_white = 0;
    //         for p in pxels {
    //             let rgba: [u8; 4] = p.0;
    //             let [r, g, b, _a] = rgba;
    //             let clr: [u8; 3] = [r, g, b];
    //             if clr == [227, 18, 0] || clr == [81, 104, 127] || clr == [228, 18, 0]
    //                 || clr == [82, 102, 126] || clr == [82, 103, 126] || clr == [79, 100, 122] {
    //                 return "U_ECAM".to_string();
    //             } else if clr == [255, 255, 0] {
    //                 has_nd_yellow = true;
    //             }
    //         }
    //         if has_nd_yellow { return "ND".to_string(); }
    //
    //         "MCDU".to_string()
    //     };
    // }

    fn find_crop_for_instruments(instrument: String, width: u32, height: u32) -> [[i32; 2]; 2] {
        let crop: [[i32; 2]; 2] = [[0, 0], [width as i32, height as i32]];
        if instrument == "".to_string() { return crop; }

        if instrument == "U_ECAM".to_string() || instrument == "L_ECAM".to_string()
            || instrument == "ND".to_string() || instrument == "PFD".to_string()
            || instrument == "SEGMENTS".to_string() || instrument == "SIS".to_string() {
            return if width == height {
                crop
            } else if width > height {
                let crop_pixels = width - height;
                [[(crop_pixels / 2) as i32, 0], [height as i32, height as i32]]
            } else {
                let crop_pixels = height - width;
                [[0, (crop_pixels / 2) as i32], [width as i32, width as i32]]
            };
        } else if instrument == "MCDU".to_string() {
            // } else if (img_height / img_width) > 1.3 {
            let mcdu_ratio: f32 = 0.866;
            if width as f32 * mcdu_ratio < height as f32 {
                let correct_height = width as f32 * mcdu_ratio;
                let gap = height as f32 - correct_height;
                return [[0, (gap / 2.0) as i32], [width as i32, correct_height as i32]];
            } else if height as f32 / mcdu_ratio < width as f32 {
                let correct_width = height as f32 / mcdu_ratio;
                let gap = width as f32 - correct_width;
                return [[(gap / 2.0) as i32, 0], [correct_width as i32, height as i32]];
            }
        }

        crop
    }

    // fn template_match_instrument(bw_img: ImageBuffer<Luma<f32>, Vec<f32>>) -> String {
    //     let path_list = Vec::from(["data/samples/template_L_ECAM.png", "data/samples/template_U_ECAM.png", "data/samples/template_PFD.png", "data/samples/template_ND.png"]);
    // 
    //     let t_definitions: [String; 4] =
    //         ["L_ECAM".to_string(), "U_ECAM".to_string(), "PFD".to_string(), "ND".to_string()];
    // 
    //     let mut match_num = 0;
    //     for t in path_list {
    //         if t == "" {continue}
    //         if Path::new(t).exists(){
    //             // let template = image::open(t).unwrap().to_luma32f();
    //             // let result = match_template(&bw_img, &template, MatchTemplateMethod::SumOfSquaredDifferences);
    //             // let extremes = find_extremes(&result);
    //             // if extremes.min_value < 1.0 {
    //             //     break;
    //             // }
    //             break;
    // 
    //         }
    //         match_num += 1
    //     }
    //     if match_num > t_definitions.len() - 1 {
    //         return "".to_string();
    //     }
    //     t_definitions[match_num].clone()
    // }

    pub unsafe fn hide_window(hwnd_in: isize) {
        let hwnda_to_move: HWND = mem::transmute(hwnd_in);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        SetWindowPos(hwnda_to_move, hwnda_to_move,
                     0, height + 50, 700, 700, SWP_NOREDRAW | SWP_NOZORDER | SWP_NOACTIVATE);
    }

    pub unsafe fn get_window_pos(hwnd_in: isize) -> RECT {
        let hwnda_to_move: HWND = mem::transmute(hwnd_in);
        let mut rrval: RECT = RECT::default();
        GetWindowRect(hwnda_to_move, &mut rrval);
        rrval
    }

    pub unsafe fn move_window(hwnd_in: isize, move_top: i32, move_left: i32, move_width: i32, move_height: i32) {
        let hwnda_to_move: HWND = mem::transmute(hwnd_in);
        SetWindowPos(hwnda_to_move, hwnda_to_move,
                     move_left, move_top, move_width, move_height,
                     SWP_NOREDRAW | SWP_NOZORDER | SWP_NOACTIVATE);
    }

    pub fn find_popup_windows() -> Vec<isize> {
        let mut process_ls = Vec::<isize>::new();
        let ls = window_list().unwrap();
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

    pub fn capture_instrument_sample(hwnd: isize, instrument: &str) -> &str {
        if instrument == "MCDU" {
            return "success";
        }

        if hwnd == 0 {
            return "error";
        }
        let crop: [[i32; 2]; 2] = match instrument {
            "PFD" => [[284, 142], [29, 15]],
            "ND" => [[12, 2], [34, 24]],
            "L_ECAM" => [[227, 566], [230, 23]],
            "U_ECAM" => [[18, 398], [79, 26]],
            _ => [[0, 0], [0, 0]],
        };
        let buf = match ImageProcess::capture_instrument(hwnd, crop) {
            Ok(res) => res,
            Err(..) => return "error"
        };

        std::fs::write(format!("data/samples/template_{}.png", instrument), buf).unwrap();
        "ok"
    }
}