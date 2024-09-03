use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use crate::config_handler::{get_simconnector_exe, SIMCONNECTOR_RELATIVE_DIR};
use crate::debug_logger::show_fatal_error;

const DETACHED_PROCESS: u32 = 0x00000008;

pub fn start_bridge_process() -> Child {
    if !std::path::Path::new(&get_simconnector_exe()).exists() {
        show_fatal_error("Can't find SimConnector.exe. Did you fully extract the the archive? Try reinstalling the app.");
    }
    let connector_path: String = get_simconnector_exe();

    let child: Child;
    if cfg!(debug_assertions) {
        child = Command::new(connector_path)
            .spawn().expect("failed to execute exe");
    }else {
        child = Command::new(connector_path).current_dir(SIMCONNECTOR_RELATIVE_DIR)
            .creation_flags(DETACHED_PROCESS).arg("hide").spawn().expect("failed to execute exe");
        
    }
    
    child
}
