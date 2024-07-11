use std::fs;
use std::fs::File;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config_handler::{get_addon_config, get_file_in_exe_folder, get_static_folder};

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
    display_top: u16,
}
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct AddonConfig {
    aircraft_addons: Vec<AircraftAddon>,
    version: u16,
    app_version: u16,
    updated: String,
}
const SERVER_ADDR: &str = "http://airportfinder.us.to/reachfms/addon_config.json";
const SERVER_BASE_ADDR: &str = "http://airportfinder.us.to/reachfms/";
impl AddonConfig {
    pub async fn load() -> Self {
        let stored = Self::get_stored().unwrap();


        return match Self::get_from_server().await {
            Ok(res) => {
                if res.version > stored.version {
                    Self::write_config(&res);
                    Self::download_svgs(&res);
                    return res
                }
                stored
            }
            Err(_) => {
                println!("Returning stored AircraftAddon");
                stored
            }
        };
    }

    async fn download_svgs(&self) {
        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();
        println!("UPDATING SVGS:");
        let base: String = SERVER_BASE_ADDR.to_string();
        let mut downloaded_svgs: Vec<&str> = vec![];
        for addon in &self.aircraft_addons {
            if downloaded_svgs.contains(&&*addon.svg_image) {
                continue;
            }
            println!("Downloading {}", addon.svg_image);
            let resp = client.get(base.clone() + &*addon.svg_image).send().await;
            match resp {
                Ok(resp) => {
                    let txt = resp.text().await.unwrap();
                    downloaded_svgs.push(&*addon.svg_image);
                    let filename = get_file_in_exe_folder(vec!["static", &*addon.svg_image]);
                    File::create(filename.clone())
                        .expect("Error encountered while creating file!");
                    fs::write(&filename, txt).expect("Unable to write file");
                }
                Err(er) => {
                    println!("error downloading svg: {}", er)
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
                let deserialized: AddonConfig = serde_json::from_str(&string_data).unwrap();
                Ok(deserialized)
            }
            Err(_) => {
                Ok(AddonConfig {
                    aircraft_addons: vec![],
                    version: 0,
                    app_version: 0,
                    updated: "".to_string(),
                })
            }
        };
    }

    async fn get_from_server() -> Result<AddonConfig, bool> {
        println!("getting from server...");
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let resp = client.get(SERVER_ADDR).send().await;
        return match resp {
            Ok(resp) => {
                println!("got serv resp");
                let txt = resp.text().await.unwrap();
                let deserialized: AddonConfig = serde_json::from_str(&txt).unwrap();
                Ok(deserialized)
            }
            Err(..) => {
                Err(false)
            }
        };
    }

    fn write_config(&self) {
        let filename = get_addon_config();
        if std::path::Path::new(&get_static_folder()).exists() {
            File::create(&filename)
                .expect("Error encountered while creating file!");
            let json_string = serde_json::to_string(self).unwrap();
            fs::write(&filename, json_string).expect("Unable to write file");
        }
    }

    fn get_aircraft_config(&self, aircraft_filename: String) -> Option<&AircraftAddon> {
        for aircraft_addon in &self.aircraft_addons {
            if aircraft_filename.contains(&aircraft_addon.title) {
                return Option::from(aircraft_addon);
            }
        }
        Option::None
    }

    pub fn get_var(&self, btn: String, aircraft_filename: String) -> &str {
        return match self.get_aircraft_config(aircraft_filename) {
            None => { "" }
            Some(aircraft_addon) => {
                for button_action in &aircraft_addon.button_actions {
                    if button_action.button == btn {
                        return &*button_action.lvar;
                    }
                }
                ""
            }
        };
    }

    pub fn calculate_crop(&self, aircraft_filename: String, width: isize, height: isize) -> [[i32; 2]; 2] {
        // [[cropx, cropy],[cropwidth, cropheight]]
        
        let mut crop: [[i32; 2]; 2] = [[0,0], [0,0]];
        match self.get_aircraft_config(aircraft_filename) {
            None => {}
            Some(aircraft_addon) => {
                let img_aspect: f64 = (width as f64) / (height as f64);
                if img_aspect > aircraft_addon.fms_aspect {
                    // magassag a fix
                    let new_width = aircraft_addon.fms_aspect * (height as f64);
                    crop[1][0] = new_width as i32;
                    crop[1][1] = height as i32;
                    crop[0][0] = ((width - (new_width as isize)) / 2) as i32
                } else {
                    // szelesseg a fix
                    let new_height = (width as f64) / aircraft_addon.fms_aspect;
                    crop[1][0] = width as i32;
                    crop[1][1] = new_height as i32;
                    crop[0][1] = ((height - (new_height as isize)) / 2) as i32
                }
            }
        };
        crop
    }
}
// pub fn test() {
//     let btn = ButtonAction { button: "btn1".to_string(), lvar: "lvar1".to_string() };
//     let btn2 = ButtonAction { button: "btn2".to_string(), lvar: "lvar2".to_string() };
//     let arcrft = AircraftAddon {
//         title: "aircraft1".to_string(),
//         button_actions: vec![btn, btn2],
//         svg_image: "a321.svg".to_string(),
//     };
//     let addn = AddonConfig {
//         aircraft_addons: vec![arcrft],
//         version: 0,
//         app_version: 0,
//         updated: "date".to_string(),
//     };
//     let filename = get_addon_config();
//         File::create(&filename)
//             .expect("Error encountered while creating file!");
//         let json_string = serde_json::to_string(&addn).unwrap();
//         fs::write(&filename, json_string).expect("Unable to write file");
//     
//  
// }
