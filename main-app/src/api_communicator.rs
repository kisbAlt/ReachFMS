use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use crate::config_handler::{get_simconnector_exe, get_simconnector_folder, SIMCONNECTOR_RELATIVE_DIR};
use crate::debug_logger::show_fatal_error;

const DETACHED_PROCESS: u32 = 0x00000008;

pub fn start_bridge_process() -> Child {
    if !std::path::Path::new(&get_simconnector_exe()).exists() {
        show_fatal_error("Can't find SimConnector.exe. Did you fully extract the the archive? Try reinstalling the app.");
    }

    let child: Child;
    if cfg!(debug_assertions) {
        child = Command::new(get_simconnector_exe()).current_dir(get_simconnector_folder())
            .spawn().expect("failed to execute exe");
    }else {
        child = Command::new(get_simconnector_exe()).current_dir(get_simconnector_folder())
            .creation_flags(DETACHED_PROCESS).arg("hide").spawn().expect("failed to execute exe");
    }
    child
}
