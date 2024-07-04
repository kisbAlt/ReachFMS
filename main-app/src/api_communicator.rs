use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use crate::config_handler::get_simconnector_exe;
use crate::http_streamer::BridgeStatus;

const LOCALHOST_PORT: &str = "5273";

const DETACHED_PROCESS: u32 = 0x00000008;
pub fn check_bridge_process() -> bool {
    let resp = reqwest::blocking::get(format!("http://localhost:{}", LOCALHOST_PORT));
    return false;
    match &resp {
        Ok(..) => {
            return true;
        }
        Err(..) => {
            println!("Not avialable");
            return false;
        }
    }
    true
}

pub fn start_bridge_process() -> Child {

    let connector_path: String = get_simconnector_exe();
    println!("starting app...: {}", &connector_path);
    
    let mut child = Command::new(connector_path)
        .arg("hide").spawn().expect("failed to execute exe");
    child
}

pub fn stop_bridge_process() -> bool {
    let resp = reqwest::blocking::get(format!("http://localhost:{}/stop_server", LOCALHOST_PORT));
    match &resp {
        Ok(..) => {
            false
        }
        Err(..) => {
            println!("Not avialable");
            true
        }
    }
}


pub fn send_btn_action(btn_id: String, use_fo: bool) -> String {
    let mut btn_var = btn_id_decoder(btn_id);

    if btn_var.contains("CDU1") & use_fo {
        btn_var = btn_var.replace("CDU1", "CDU2")
    }

    let resp = reqwest::blocking::get(format!("http://localhost:{}/btn_action?var={}", LOCALHOST_PORT, btn_var));
    match &resp {
        Ok(..) => {
            resp.unwrap().text().unwrap();
        }
        Err(..) => {
            println!("Not avialable");
        }
    }
    btn_var
}


pub fn reconnect() -> String {
    let resp = reqwest::blocking::get(format!("http://localhost:{}/connect", LOCALHOST_PORT));
    match &resp {
        Ok(..) => {
            let rspval = resp.unwrap().text().unwrap();
            rspval
        }
        Err(..) => {
            println!("Not avialable");
            "Error".to_string()
        }
    }
}


fn btn_id_decoder(btn_id: String) -> String {
    let s: String = btn_id.to_owned();
    let s_slice: &str = &s[..];
    let btn_var = match s_slice {
        "65" => "S_CDU1_KEY_A",
        "66" => "S_CDU1_KEY_B",
        "67" => "S_CDU1_KEY_C",
        "68" => "S_CDU1_KEY_D",
        "69" => "S_CDU1_KEY_E",
        "70" => "S_CDU1_KEY_F",
        "71" => "S_CDU1_KEY_G",
        "72" => "S_CDU1_KEY_H",
        "73" => "S_CDU1_KEY_I",
        "74" => "S_CDU1_KEY_J",
        "75" => "S_CDU1_KEY_K",
        "76" => "S_CDU1_KEY_L",
        "77" => "S_CDU1_KEY_M",
        "78" => "S_CDU1_KEY_N",
        "79" => "S_CDU1_KEY_O",
        "80" => "S_CDU1_KEY_P",
        "81" => "S_CDU1_KEY_Q",
        "82" => "S_CDU1_KEY_R",
        "83" => "S_CDU1_KEY_S",
        "84" => "S_CDU1_KEY_T",
        "85" => "S_CDU1_KEY_U",
        "86" => "S_CDU1_KEY_V",
        "87" => "S_CDU1_KEY_W",
        "88" => "S_CDU1_KEY_X",
        "89" => "S_CDU1_KEY_Y",
        "90" => "S_CDU1_KEY_Z",

        "47" => "S_CDU1_KEY_SLASH",
        "32" => "S_CDU1_KEY_SPACE",
        "110" => "S_CDU1_KEY_OVFLY",
        "8" => "S_CDU1_KEY_CLEAR",

        "112" => "S_CDU1_KEY_LSK1L",
        "113" => "S_CDU1_KEY_LSK2L",
        "114" => "S_CDU1_KEY_LSK3L",
        "115" => "S_CDU1_KEY_LSK4L",
        "116" => "S_CDU1_KEY_LSK5L",
        "117" => "S_CDU1_KEY_LSK6L",
        "120" => "S_CDU1_KEY_LSK1R",
        "121" => "S_CDU1_KEY_LSK2R",
        "122" => "S_CDU1_KEY_LSK3R",
        "123" => "S_CDU1_KEY_LSK4R",
        "124" => "S_CDU1_KEY_LSK5R",
        "125" => "S_CDU1_KEY_LSK6R",

        "98" => "S_CDU1_KEY_DIR",
        "99" => "S_CDU1_KEY_PROG",
        "100" => "S_CDU1_KEY_PERF",
        "101" => "S_CDU1_KEY_INIT",
        "102" => "S_CDU1_KEY_DATA",
        "103" => "S_CDU1_KEY_FPLN",
        "104" => "S_CDU1_KEY_RAD_NAV",
        "105" => "S_CDU1_KEY_FUEL_PRED",
        "106" => "S_CDU1_KEY_SEC_FPLN",
        "107" => "S_CDU1_KEY_ATC_COM",
        "108" => "S_CDU1_KEY_AIRPORT",
        "96" => "S_CDU1_KEY_MENU",

        "94" => "S_CDU1_KEY_ARROW_UP",
        "95" => "S_CDU1_KEY_ARROW_LEFT",
        "30" => "S_CDU1_KEY_ARROW_DOWN",
        "31" => "S_CDU1_KEY_ARROW_RIGHT",

        "49" => "S_CDU1_KEY_1",
        "50" => "S_CDU1_KEY_2",
        "51" => "S_CDU1_KEY_3",
        "52" => "S_CDU1_KEY_4",
        "53" => "S_CDU1_KEY_5",
        "54" => "S_CDU1_KEY_6",
        "55" => "S_CDU1_KEY_7",
        "56" => "S_CDU1_KEY_8",
        "57" => "S_CDU1_KEY_9",
        "48" => "S_CDU1_KEY_0",
        "46" => "S_CDU1_KEY_DOT",
        "36" => "S_CDU1_KEY_MINUS",

        "201" => "S_ECAM_TO",
        "200" => "S_ECAM_ENGINE",
        "202" => "S_ECAM_BLEED",
        "203" => "S_ECAM_CAB_PRESS",
        "204" => "S_ECAM_ELEC",
        "205" => "S_ECAM_HYD",
        "206" => "S_ECAM_FUEL",
        "207" => "S_ECAM_APU",
        "208" => "S_ECAM_COND",
        "209" => "S_ECAM_DOOR",
        "210" => "S_ECAM_WHEEL",
        "211" => "S_ECAM_FCTL",
        "212" => "S_ECAM_ALL",
        "213" => "S_ECAM_STATUS",
        "214" => "S_ECAM_RCL",
        "216" => "S_ECAM_CLR_LEFT",
        "215" => "S_ECAM_CLR_RIGHT",

        _ => ""
    };
    btn_var.to_string()
}