use chrono::Local;

pub fn writeDebugInfo(info: &str) {
    println!("[{}] [DEBUG] {}", Local::now().format("%H:%M"), info);
}