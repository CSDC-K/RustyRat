use crate::libs::hardware_lib;
pub fn _get_func(msg: &str) {
    if msg == "ping" {
        println!("PONG");
    } else if msg == "hardware"{
        
    }  
    else {
        println!("Unknown command: {}", msg);
    }
}
