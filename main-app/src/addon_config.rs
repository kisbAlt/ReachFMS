use std::fs;
use std::fs::File;
use std::time::Duration;
use reqwest::blocking::Client;
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
    button_actions: Vec<ButtonAction>,
    svg_image: String,
    output_vars: Vec<String>
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
    pub fn load() -> Self {
        let stored = Self::get_stored().unwrap();


        return match Self::get_from_server() {
            Ok(res) => {
                if res.version > stored.version {
                    Self::write_config(&res);
                    Self::download_svgs(&res);
                }
                res
            }
            Err(_) => {
                println!("Returning stored AircraftAddon");
                stored
            }
        }
    }
    
    fn download_svgs(&self) {
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
            let resp = client.get(base.clone()+&*addon.svg_image).send();
            match resp {
                Ok(resp) => {
                    let txt = resp.text().unwrap();
                    downloaded_svgs.push(&*addon.svg_image);
                    let filename  =get_file_in_exe_folder(vec!["static", &*addon.svg_image]);
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

    fn get_from_server() -> Result<AddonConfig, bool> {
        println!("getting from server...");
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let resp = client.get(SERVER_ADDR).send();
        return match resp {
            Ok(resp) => {
                println!("got serv resp");
                let txt = resp.text().unwrap();
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
