use std::{fs};
use std::fs::File;
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct DebugSave {
    pub instrument_list: String,
    pub config: String,
    pub status: String,
}

impl ConfigHandler {
    pub fn init() -> Self {
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
        };

        if !ConfigHandler::is_data_created() {
            fs::create_dir("data").unwrap();

        }
        if !std::path::Path::new("data/samples").exists() {
            fs::create_dir("data/samples").unwrap();
        }

        if !ConfigHandler::is_config_created() {
            File::create("data/config.json")
                .expect("Error encountered while creating file!");
            let json_string = serde_json::to_string(&default_config).unwrap();
            fs::write("data/config.json", json_string).expect("Unable to write file");
        }
        qrcode_generator::to_png_to_file(host, QrCodeEcc::Low, 1024, "data/qr.png").unwrap();
        default_config
    }

    pub fn read_config(&mut self) {
        let string_data = fs::read_to_string("data/config.json").expect("Unable to read file");
        let deserialized: ConfigHandler = serde_json::from_str(&string_data).unwrap();
        self.auto_hide = deserialized.auto_hide;
        self.refresh_rate = deserialized.refresh_rate;
        self.max_fps = deserialized.max_fps;
        self.auto_start = deserialized.auto_start;
        self.multiple_displays = deserialized.multiple_displays;
        self.cpu_displays = deserialized.cpu_displays;
        self.calibrated = deserialized.calibrated;
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
        if std::path::Path::new("data/config.json").exists() {
            File::create("data/config.json")
                .expect("Error encountered while creating file!");
            let json_string = serde_json::to_string(self).unwrap();
            fs::write("data/config.json", json_string).expect("Unable to write file");
        }
    }

    pub fn get_localhost() -> String {
        format!("http://{}:5273", local_ip_address::local_ip().unwrap().to_string())
    }

    pub fn get_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn is_data_created() -> bool {
        std::path::Path::new("data").exists()
    }
    pub fn is_config_created() -> bool {
        std::path::Path::new("data/config.json").exists()
    }

}