use std::time::Instant;
use crossbeam_channel::{bounded, select, after, SendTimeoutError};
use crate::http_streamer::BridgeStatus;

pub fn get_aircraft(command_sender: &crossbeam_channel::Sender<String>,
                    comm_receiver: &crossbeam_channel::Receiver<String>) -> String {
    let timeout = std::time::Duration::from_millis(100);
    command_sender.send("BridgeStatus".to_string()).expect("Can't send.");
    let mut aircraft: String = "".to_string();
    let started: Instant = Instant::now();
    let mut got_info = false;
    while !got_info {
        select! {
            recv(comm_receiver) -> msg => {
                let resp = msg.unwrap_or("".to_string());
                if resp.contains("AIRCRAFT:"){
                    aircraft = resp.replace("AIRCRAFT:", "");
                    got_info = true;
                }
        },
            recv(after(timeout)) -> _ => {
                    if started.elapsed().as_millis() > 201 {
                    break;
                }
                }
        }
    }
    return aircraft;
}

pub fn get_status(brid_status: &mut BridgeStatus, command_sender: crossbeam_channel::Sender<String>,
                  comm_receiver: crossbeam_channel::Receiver<String>) {
    let timeout = std::time::Duration::from_millis(100);

    let started: Instant = Instant::now();
    match command_sender.send_timeout("BridgeStatus".to_string(), timeout) {
        Ok(_) => {}
        Err(_) => {}
    };
    loop {
        select! {
            recv(comm_receiver) -> msg => {
                let resp = msg.unwrap_or("".to_string());
                brid_status.comm = true;
                if resp == "STATUS:TRUE"{
                    brid_status.connected = true;
                }else if resp == "STATUS:FALSE"{
                    brid_status.connected = false;
                }
                return;
            },
            recv(after(timeout)) -> _ => {
                if(started.elapsed().as_millis() > 201){
                    brid_status.comm = false;
                    return;
                }
             }
            }
    }
}
