use std::{fs, io};
use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;
use crate::config_handler;

// check if mobiflight wasm module is installed, if not install it.
pub fn check_mobiflight() {
    let community: String = get_community_folder().unwrap();

    if !mobiflight_installed() {
        install_mobiflight();
    }
}


pub fn get_community_folder() -> Result<String, bool> {
    #[cfg(windows)]
    let mut file_path = std::env::var("APPDATA").expect("No APP_DATA1 directory") + "\\Microsoft Flight Simulator\\UserCfg.opt";


    if !std::path::Path::new(&file_path).exists() {
        file_path = std::env::var("LOCALAPPDATA").expect("No APP_DATA1 directory")
            + "\\Packages\\Microsoft.FlightSimulator_8wekyb3d8bbwe\\LocalCache\\UserCfg.opt";
        if !std::path::Path::new(&file_path).exists() {
            return Err(false);
        }
    }
    println!("1:{}", &file_path);
    let content = fs::read_to_string(&file_path)
        .expect("Cant read config file");

    for line in content.lines() {
        if line.contains("InstalledPackagesPath") {
            let community_path = line.split("\"").collect::<Vec<&str>>()[1].to_string()
                + "\\Community";
            println!("{}", community_path);
            return Ok(community_path);
        }
    }
    return Err(false);
}

pub fn mobiflight_installed() -> bool {
    let community_folder = get_community_folder().unwrap();
    std::path::Path::new(&(community_folder.to_owned() + "\\mobiflight-event-module")).exists()
    
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
            let filename = config_handler::get_file_in_exe_folder(vec!["temp","mobiflight-event-module.zip"]);
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
                .open(&filename).unwrap();

            file.write_all(&bytes_vector).expect("Cant write downloaded mobiflight-event");

            extract_mobi(&filename);
        }
        Err(..) => {
            println!("Not avialable");
        }
    }
}

fn extract_mobi(zip_file: &String) {
    let community_path = PathBuf::from(get_community_folder().unwrap());
    let fname = std::path::Path::new(&zip_file);
    let file = fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => {
                let new_path = community_path.join(path);
                new_path
            },
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {i} comment: {comment}");
            }
        }

        if file.is_dir() {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
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