use std::ffi::CString;
use std::fs::{OpenOptions};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::config_handler::get_log_file;
use std::io::prelude::*;
use windows::core::PCSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONWARNING, MB_OK, MessageBoxA};

pub fn log(new_log: &str, log_str: &Option<Arc<Mutex<String>>>) {
    let log_text = format!("[{}] {}\\n", Utc::now().to_string(), new_log);
    match log_str {
        None => {println!("{}", log_text)}
        Some(log) => {
            // DEBUG
            println!("{}", log_text);
            let mut old_log = log.lock().expect("Cant unwrap old_str in debug_logger");
            
            *old_log+= &*(log_text);
            
            if old_log.chars().count() > 1024 {
                write_file(&old_log);
                *old_log = "".to_string(); 
            }
            drop(old_log);
            
        }
    }
}

pub fn log_and_write(new_log: &str, log_str: &Option<Arc<Mutex<String>>>) {
    let log_text = format!("[{}] {}\\n", Utc::now().to_string(), new_log);
    match log_str {
        None => {println!("{}", log_text)}
        Some(log) => {
            let mut old_log = log.lock().expect("Cant unwrap old_str in debug_logger");

            *old_log+= &*(log_text);
            write_file(&old_log);
            *old_log = "".to_string();
            drop(old_log);

        }
    }
}


pub fn write_file(write_log: &String) {
    
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&get_log_file()).unwrap();

    write!(f, "{}", write_log).unwrap();
    
}

pub fn clone_log(log_str: &Option<Arc<Mutex<String>>>) -> Option<Arc<Mutex<String>>>{
    return match log_str {
        None => { Option::None }
        Some(cnt) => {
            Option::from(Arc::clone(cnt))
        }
    }
}

pub fn show_error_dialog(message: &str) {
    unsafe {
        let lp_text = CString::new(message).unwrap();
        let lp_caption = CString::new("Error while running the app...").unwrap();
        let text_pcstr: PCSTR = PCSTR(lp_text.as_ptr() as _);
        let caption_pcstr: PCSTR = PCSTR(lp_caption.as_ptr() as _);
        MessageBoxA(HWND::default(), text_pcstr, caption_pcstr, MB_OK | MB_ICONERROR);

    }
}

pub fn show_warning_dialog(message: &str) {
    unsafe {
        let lp_text = CString::new(message).unwrap();
        let lp_caption = CString::new("Warning while running the app...").unwrap();
        let text_pcstr: PCSTR = PCSTR(lp_text.as_ptr() as _);
        let caption_pcstr: PCSTR = PCSTR(lp_caption.as_ptr() as _);
        MessageBoxA(HWND::default(), text_pcstr, caption_pcstr, MB_OK | MB_ICONWARNING);

    }
}

pub fn show_fatal_error(message: &str) {
    show_error_dialog(message);
    std::process::exit(0);
}
