use std::{fs};
use std::fs::File;
use std::sync::{Arc, Mutex};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use crate::debug_logger;

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ConfigHandler {
    pub local_ip: String,
    pub auto_hide: bool,
    pub max_fps: bool,
    pub refresh_rate: u16,
    pub cpu_displays: bool,
    pub multiple_displays: bool,
    pub auto_start: bool,
    pub calibrated: bool,
    pub log_enabled: bool,
    #[serde(skip_serializing, skip_deserializing)]
    log_str: Option<Arc<Mutex<String>>>
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct DebugSave {
    pub instrument_list: String,
    pub config: String,
    pub status: String,
}

impl ConfigHandler {
    pub fn init(log_str: Option<Arc<Mutex<String>>>) -> Self {
        let host = ConfigHandler::get_localhost();
        let local_ip = host.clone();
        let default_config = ConfigHandler {
            local_ip,
            auto_hide: true,
            max_fps: true,
            refresh_rate: 200,
            cpu_displays: false,
            multiple_displays: true,
            auto_start: false,
            calibrated: false,
            log_str,
            log_enabled: false
        };

        if !ConfigHandler::is_data_created() {
            debug_logger::log("creating data folder...", &default_config.log_str);
            fs::create_dir(&get_file_in_exe_folder(vec!["data"])).unwrap();

        }

        if !ConfigHandler::is_config_created() {
            debug_logger::log("creating config.json...", &default_config.log_str);
            File::create(get_config_file())
                .expect("Error encountered while creating file!");
            let json_string = serde_json::to_string(&default_config).unwrap();
            fs::write(get_config_file(), json_string).expect("Unable to write file");
        }
        qrcode_generator::to_png_to_file(host, QrCodeEcc::Low, 1024, get_qr_file()).unwrap();
        default_config
    }

    pub fn read_config(&mut self) {
        let string_data = fs::read_to_string(get_config_file()).expect("Unable to read file");
        let deserialized: ConfigHandler = serde_json::from_str(&string_data).unwrap();
        self.auto_hide = deserialized.auto_hide;
        self.refresh_rate = deserialized.refresh_rate;
        self.max_fps = deserialized.max_fps;
        self.auto_start = deserialized.auto_start;
        self.multiple_displays = deserialized.multiple_displays;
        self.cpu_displays = deserialized.cpu_displays;
        self.calibrated = deserialized.calibrated;
        self.log_enabled = deserialized.log_enabled
    }

    pub fn get_all_local_ip() -> Vec<String> {
        let mut url_list: Vec<String> = Vec::new();
        let raw_list = local_ip_address::list_afinet_netifas().unwrap();
        for ip in raw_list {
            let cur_ip = ip.1.to_string();
            if cur_ip.contains(":") { continue; }
            url_list.push(format!("http://{}:5273", cur_ip))
        }
        url_list
    }

    pub fn write_config(&self) {
        let filename = get_config_file();
        if std::path::Path::new(&filename).exists() {
            File::create(&filename)
                .expect("Error encountered while creating file!");
            let json_string = serde_json::to_string(self).unwrap();
            fs::write(&filename, json_string).expect("Unable to write file");
        }
    }

    pub fn get_localhost() -> String {
        format!("http://{}:5273", local_ip_address::local_ip().unwrap().to_string())
    }

    pub fn get_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn is_data_created() -> bool {
        std::path::Path::new(&get_file_in_exe_folder(vec!["data"])).exists()
    }
    pub fn is_config_created() -> bool {
        std::path::Path::new(&get_config_file()).exists()
    }

}
pub fn get_file_in_exe_folder(path_inside: Vec<&str>) -> String {
    return match std::env::current_exe() {
        Ok(mut res) => {
            res.pop();

            for item in path_inside {
                res.push(item);
            }

            res.into_os_string().into_string().unwrap_or("".to_string())

        }
        Err(_) => {
            "".to_string()
        }
    };
}


pub fn get_static_folder() -> String {
    return get_file_in_exe_folder(vec!["static"])
}

pub fn get_addon_config() -> String {
    return get_file_in_exe_folder(vec!["static", "addon_config.json"])
}

pub const SIMCONNECTOR_RELATIVE_DIR: &str = "SimConnector";
pub fn get_simconnector_exe() -> String {
    return get_file_in_exe_folder(vec![SIMCONNECTOR_RELATIVE_DIR, "SimConnector.exe"])
}

pub fn get_simconnector_folder() -> String {
    return get_file_in_exe_folder(vec![SIMCONNECTOR_RELATIVE_DIR])
}

pub fn get_config_file() -> String {
    return get_file_in_exe_folder(vec!["data", "config.json"])
}
pub fn get_qr_file() -> String {
    return get_file_in_exe_folder(vec!["data", "qr.png"])
}
pub fn get_log_file() -> String {
    return get_file_in_exe_folder(vec!["reachfms.log"])
}