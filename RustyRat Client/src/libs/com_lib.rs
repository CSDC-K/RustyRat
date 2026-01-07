use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::Arc;
use std::thread;
use std::io;

use crate::libs::debug_info::writeDebugInfo;

pub fn start_communication(stream : TcpStream) {
    // Placeholder for communication logic
    let stream = Arc::new(stream);
    let write_thread_handle = Arc::clone(&stream);
    let read_thread_handle = Arc::clone(&stream);
    writeDebugInfo("Communication started");
    
    // Reading Handle Thread
    thread::spawn(move || {
        let reader = BufReader::new(&*read_thread_handle);
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("Received: {}", msg),
                Err(_) => break,
            }
        }
    });

    // Writing Handle Thread
    thread::spawn(move || {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            print!("[COM_LIB] Enter message to send: ");
            io::stdout().flush().expect("Error at flushing stdout");
            io::stdin().read_line(&mut buffer).expect("Error at reading from stdin");
            buffer = buffer.trim().to_string();
            //buffer.push('\n');
            let mut thread_guard = write_thread_handle.try_clone().expect("Error at writing thread handle");
            thread_guard.write_all(buffer.as_bytes()).expect("Error at writing thread");
        }
    });

    loop {
        thread::park();
    }


}