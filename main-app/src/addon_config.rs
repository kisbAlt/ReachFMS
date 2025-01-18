use std::fs;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config_handler::{get_addon_config, get_file_in_exe_folder, get_static_folder};
use crate::debug_logger;
use crate::debug_logger::{show_fatal_error, show_warning_dialog};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ButtonAction {
    button: String,
    lvar: String,
}
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct AircraftAddon {
    title: String,
    display: String,
    button_actions: Vec<ButtonAction>,
    svg_image: String,
    output_vars: Vec<String>,
    // aspect: width/height
    fms_aspect: f64,
    display_width: u16,
    display_top: i16,
    display_left: i16,
    custom_popout: Vec<String>,
    touch_enabled: bool,
    last_updated: String,
}
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct AddonConfig {
    aircraft_addons: Vec<AircraftAddon>,
    version: u32,
    app_version: u32,
    updated: String,
    #[serde(skip_serializing, skip_deserializing)]
    log_str: Option<Arc<Mutex<String>>>,
}
const SERVER_ADDR: &str = "http://airportfinder.us.to/reachfms/addon_config.json";
const SERVER_BASE_ADDR: &str = "http://airportfinder.us.to/reachfms/";
impl AddonConfig {
    pub async fn load(log_str: Option<Arc<Mutex<String>>>) -> Self {
        let stored = Self::get_stored();

        match Self::get_from_server(debug_logger::clone_log(&log_str)).await {
            Ok(mut res) => {
                return match stored {
                    Ok(mut stored) => {
                        if res.version > stored.version && res.app_version <= get_app_version() {
                            Self::write_config(&res);
                            Self::download_svgs(&res).await;
                            res.log_str = log_str;
                            return res;
                        }
                        if res.app_version > get_app_version() {
                            show_warning_dialog("A new app version is released, and is needed for the newest version of the config file to work. Head over to flightsim.to to download.");
                        }
                        stored.log_str = log_str;
                        stored
                    }
                    Err(err) => {
                        println!("error while opening: {}", err);
                        Self::write_config(&res);
                        Self::download_svgs(&res).await;
                        res
                    }
                }
            }
            Err(_) => {
                return match stored {
                    Ok(stored_u) => {
                        stored_u
                    }
                    Err(_) => {
                        show_fatal_error("Can't reach server, and there is no static folder. Did you extract the program correctly? Try to reinstall the app!");
                        AddonConfig {
                            aircraft_addons: vec![],
                            version: 0,
                            app_version: 0,
                            updated: "".to_string(),
                            log_str,
                        }
                    }
                }
            }
        };
    }

    async fn download_svgs(&self) {
        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();
        debug_logger::log("Downloading svg files...", &self.log_str);
        let base: String = SERVER_BASE_ADDR.to_string();
        let mut downloaded_svgs: Vec<&str> = vec![];
        for addon in &self.aircraft_addons {
            if downloaded_svgs.contains(&&*addon.svg_image) {
                continue;
            }
            debug_logger::log(&*format!("Downloading svg: {}", &addon.svg_image), &self.log_str);
            let resp = client.get(base.clone() + &*addon.svg_image).send().await;
            match resp {
                Ok(resp) => {
                    let txt = resp.text().await.unwrap();
                    downloaded_svgs.push(&*addon.svg_image);
                    let filename = get_file_in_exe_folder(vec!["static", &*addon.svg_image]);
                    File::create(filename.clone())
                        .expect("Error encountered while creating file!");
                    fs::write(&filename, txt).expect("Unable to write file");
                    debug_logger::log(&*format!("Svg downloaded successfully: {}", &addon.svg_image), &self.log_str);
                }
                Err(er) => {
                    debug_logger::log(&*format!("error downloading svg files: {}", &er), &self.log_str);
                }
            };
        }
    }

    fn get_stored() -> Result<AddonConfig, bool> {
        if !std::path::Path::new(&get_static_folder()).exists() {
            fs::create_dir(&get_static_folder()).expect("Cant create static.");
        }

        return match fs::read_to_string(get_addon_config()) {
            Ok(string_data) => {
                return match serde_json::from_str(&string_data) {
                    Ok(deserialized) => {
                        Ok(deserialized)
                    }
                    Err(err) => {
                        println!("{}", err);
                        Err(false)
                    }
                }
            }
            Err(_) => {
                Err(false)
            }
        };
    }

    async fn get_from_server(log_str: Option<Arc<Mutex<String>>>) -> Result<AddonConfig, bool> {
        debug_logger::log("Getting addon config from server", &log_str);
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let resp = client.get(SERVER_ADDR).send().await;
        return match resp {
            Ok(resp) => {
                debug_logger::log("Server responded successfully", &log_str);
                return match resp.text().await {
                    Ok(txt) => {
                        let deserialized: AddonConfig = serde_json::from_str(&txt).unwrap();
                        Ok(deserialized)
                    }
                    Err(_) => { Err(false) }
                };
            }
            Err(e) => {
                debug_logger::log(format!("Error while getting addon config from server: {}", e).as_str(), &log_str);
                Err(false)
            }
        };
    }

    fn write_config(&self) {
        let filename = get_addon_config();
        if std::path::Path::new(&get_static_folder()).exists() {
            match File::create(&filename) {
                Ok(_) => {}
                Err(_) => { debug_logger::log("Error while creating static folder", &self.log_str); }
            }
            let json_string = serde_json::to_string(self).unwrap();
            fs::write(&filename, json_string).expect("Unable to write file");
        }
    }

    pub fn get_aircraft_config(&self, aircraft_filename: &String) -> Option<&AircraftAddon> {
        let lowered = aircraft_filename.to_lowercase();
        for aircraft_addon in &self.aircraft_addons {
            if lowered.contains(&aircraft_addon.title) {
                return Option::from(aircraft_addon);
            }
        }
        Option::None
    }

    pub fn get_var(&self, btn: String, aircraft_filename: &String) -> &str {
        return match self.get_aircraft_config(&aircraft_filename) {
            None => {
                debug_logger::log("Cant find aircraft config for this aircraft!", &self.log_str);

                ""
            }
            Some(aircraft_addon) => {
                for button_action in &aircraft_addon.button_actions {
                    if &button_action.button == &btn {
                        return &button_action.lvar;
                    }
                }
                debug_logger::log("Cant find this button config for this known aircraft!", &self.log_str);
                ""
            }
        };
    }

    pub fn popout_list(&self) -> Vec<String> {
        let mut popout_list: Vec<String> = Vec::new();
        for aircraft_addon in &self.aircraft_addons {
            if aircraft_addon.custom_popout.len() > 0 {
                popout_list.extend(aircraft_addon.custom_popout.clone())
            }
        }
        return popout_list;
    }

    pub fn calculate_crop(&self, aircraft_filename: &String, width: i32, height: i32) -> [[i32; 2]; 2] {
        // [[cropx, cropy],[cropwidth, cropheight]]

        let mut crop: [[i32; 2]; 2] = [[0, 0], [0, 0]];
        match self.get_aircraft_config(&aircraft_filename) {
            None => {}
            Some(aircraft_addon) => {
                let img_aspect: f64 = (width as f64) / (height as f64);
                if img_aspect > aircraft_addon.fms_aspect {
                    // magassag a fix
                    let new_width = aircraft_addon.fms_aspect * (height as f64);
                    crop[1][0] = new_width as i32;
                    crop[1][1] = height as i32;
                    crop[0][0] = ((width - (new_width as i32)) / 2) as i32
                } else {
                    // szelesseg a fix
                    let new_height = (width as f64) / aircraft_addon.fms_aspect;
                    crop[1][0] = width as i32;
                    crop[1][1] = new_height as i32;
                    crop[0][1] = ((height - (new_height as i32)) / 2) as i32
                }
            }
        };
        crop
    }
}
fn get_app_version() -> u32 {
    // example : v0.2.0 => 2000 (every part is max. 2 digits)
    // example : v0.21.11 => 2111
    let str_ver = env!("CARGO_PKG_VERSION");
    let version_split = str_ver.split(".").collect::<Vec<&str>>();
    let mut ver_num: u32 = 0;
    let mut cntr: u32 = (version_split.len() + 3) as u32;
    for ver in version_split {
        cntr -= 2;
        ver_num += ver.parse::<u32>().unwrap_or(0) * u32::pow(10, cntr);
    }
    ver_num
}