use crate::config_handler;
use crate::debug_logger::{show_fatal_error, show_warning_dialog};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

// check if mobiflight wasm module is installed, if not install it.

pub fn get_community_folder() -> Result<String, bool> {
    #[cfg(windows)]
    let mut file_path = std::env::var("APPDATA").expect("No APP_DATA1 directory")
        + "\\Microsoft Flight Simulator\\UserCfg.opt";

    if !std::path::Path::new(&file_path).exists() {
        file_path = std::env::var("LOCALAPPDATA").expect("No APP_DATA1 directory")
            + "\\Packages\\Microsoft.FlightSimulator_8wekyb3d8bbwe\\LocalCache\\UserCfg.opt";
        if !std::path::Path::new(&file_path).exists() {
            return Err(false);
        }
    }

    let content = fs::read_to_string(&file_path).expect("Cant read config file");

    for line in content.lines() {
        if line.contains("InstalledPackagesPath") {
            let community_path =
                line.split("\"").collect::<Vec<&str>>()[1].to_string() + "\\Community";
            return Ok(community_path);
        }
    }
    return Err(false);
}

pub fn get_2024_community_folder() -> Result<String, bool> {
    #[cfg(windows)]
    let mut file_path = std::env::var("APPDATA").expect("No APP_DATA1 directory")
        + "\\Microsoft Flight Simulator 2024\\UserCfg.opt";

    if !std::path::Path::new(&file_path).exists() {
        file_path = std::env::var("LOCALAPPDATA").expect("No APP_DATA1 directory")
            + "\\Packages\\Microsoft.Limitless_8wekyb3d8bbwe\\LocalCache\\UserCfg.opt";
        if !std::path::Path::new(&file_path).exists() {
            return Err(false);
        }
    }

    let content = fs::read_to_string(&file_path).expect("Cant read config file");

    for line in content.lines() {
        if line.contains("InstalledPackagesPath") {
            let community_path =
                line.split("\"").collect::<Vec<&str>>()[1].to_string() + "\\Community";
            return Ok(community_path);
        }
    }
    return Err(false);
}

pub fn mobiflight_installed(show_msg: &bool) -> bool {
    mobiflight_2020_installed(&show_msg) && mobiflight_2024_installed(&show_msg)
}

fn mobiflight_2020_installed(show_msg: &bool) -> bool {
    match get_community_folder() {
        Ok(community_2020_folder) => {
            std::path::Path::new(&(community_2020_folder.to_owned() + "\\mobiflight-event-module"))
                .exists()
        }
        Err(e) => {
            if *show_msg {
                show_warning_dialog("Can't find your MSFS2020 community folder! If you don't have it installed ignore this message. Otherwise install the mobiflight wasm module manually!");
            }
            false
        }
    }
}

fn mobiflight_2024_installed(show_msg: &bool) -> bool {
    match get_2024_community_folder() {
        Ok(community_2024_folder) => {
            std::path::Path::new(&(community_2024_folder.to_owned() + "\\mobiflight-event-module"))
                .exists()
        }
        Err(e) => {
            if *show_msg {
                show_warning_dialog("Can't find your MSFS2024 community folder! If you don't have it installed ignore this message. Otherwise install the mobiflight wasm module manually!");
            }
            false
        }
    }
}

pub fn install_mobiflight() {
    download_package();
    // manual download: https://github.com/MobiFlight/MobiFlight-WASM-Module/releases/latest/
}
const FALLBACK_URL: &str = "https://github.com/MobiFlight/MobiFlight-WASM-Module/releases/download/1.0.1/mobiflight-event-module.1.0.1.zip";

pub fn download_package() {
    let resp = reqwest::blocking::get(FALLBACK_URL);
    match &resp {
        Ok(..) => {
            let bytes_vector = resp.unwrap().bytes().unwrap().to_vec();

            let temp_dir = config_handler::get_file_in_exe_folder(vec!["temp"]);
            let filename =
                config_handler::get_file_in_exe_folder(vec!["temp", "mobiflight-event-module.zip"]);
            if !std::path::Path::new(&temp_dir).exists() {
                fs::create_dir(temp_dir).expect("Cant create temp dir");
            }
            if !std::path::Path::new(&filename).exists() {
                File::create(&filename).expect("Cant create temp mobiflight-event file");
            }

            let mut file = fs::OpenOptions::new()
                // .create(true) // To create a new file
                .write(true)
                // either use the ? operator or unwrap since it returns a Result
                .open(&filename)
                .unwrap();

            file.write_all(&bytes_vector)
                .expect("Cant write downloaded mobiflight-event");

            extract_mobi(&filename);
        }
        Err(..) => {}
    }
}

fn extract_mobi(zip_file: &String) {
    let mut community_path;
    let fname = std::path::Path::new(&zip_file);
    let file = fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for v in 0..2 {
        if (v == 0) {
            // install mobi to msfs2020 if not installed already
            if (mobiflight_2020_installed(&false)) {
                continue;
            }
            match get_community_folder() {
                Ok(community_2020_folder) => {
                    community_path = PathBuf::from(community_2020_folder);
                }
                Err(e) => continue,
            }
        } else {
            // install mobi to msfs2024 if not installed already
            match get_2024_community_folder() {
                Ok(community_2024_folder) => {
                    community_path = PathBuf::from(community_2024_folder);
                }
                Err(e) => continue,
            }
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let new_path = community_path.join(path);
                    new_path
                }
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {}
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
}
