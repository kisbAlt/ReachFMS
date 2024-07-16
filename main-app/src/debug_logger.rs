use std::sync::{Arc, Mutex};
use std::time::Instant;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct DebugLogger {
    pub log_text: Arc<Mutex<String>>,
    pub last_written: Arc<Mutex<Instant>>
}

impl DebugLogger {
    pub fn init() -> Self {
        let init_var = DebugLogger {
            log_text: Arc::new(Mutex::from("".to_string())),
            last_written: Arc::from(Mutex::from(Instant::now()))
        };
        init_var
    }
    pub fn log(&mut self, text: &str) {
        let current_time: DateTime<Utc> = Utc::now();
        let log_text = format!("[{}]: {}", current_time.to_string(), text);
        let mut old_log = self.log_text.lock().unwrap();
        *old_log += &*(log_text + "\n");
        
        println!("LOG TEXT START: {} \n END", old_log);
        drop(old_log);
        
        let mut last_write = self.last_written.lock().unwrap();
        if last_write.elapsed().as_millis() > 30000  {
            *last_write = Instant::now();
            self.write_log();
        }
    }
    
    pub fn write_log(&self) {
        
    }
}
