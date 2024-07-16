use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use crate::config_handler::get_simconnector_exe;
const DETACHED_PROCESS: u32 = 0x00000008;
const LOCALHOST_PORT: &str = "5273";
pub fn check_bridge_process() -> bool {
    return false;
}

pub fn start_bridge_process() -> Child {

    let connector_path: String = get_simconnector_exe();
    println!("starting app...: {}", &connector_path);
    
    let child = Command::new(connector_path)
        .creation_flags(DETACHED_PROCESS).arg("hide").spawn().expect("failed to execute exe");
    child
}
