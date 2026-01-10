use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod libs;
use crate::libs::capture_image::ScreenCapturer;

const SERVER_ADDR: &str = "0.0.0.0:9999";

fn handle_client(mut stream: TcpStream) {
    println!("Client connected: {:?}", stream.peer_addr());
    
    // Disable Nagle's algorithm for lower latency
    let _ = stream.set_nodelay(true);
    
    let mut capturer = ScreenCapturer::new();
    
    loop {
        // Capture and encode the frame
        if let Some((data, width, height)) = capturer.capture_frame() {
            // Send header: size (4 bytes) + width (2 bytes) + height (2 bytes)
            let size = data.len() as u32;
            if let Err(e) = stream.write_all(&size.to_be_bytes()) {
                eprintln!("Failed to send frame size: {}", e);
                break;
            }
            if let Err(e) = stream.write_all(&width.to_be_bytes()) {
                eprintln!("Failed to send width: {}", e);
                break;
            }
            if let Err(e) = stream.write_all(&height.to_be_bytes()) {
                eprintln!("Failed to send height: {}", e);
                break;
            }
            
            // Send the frame data
            if let Err(e) = stream.write_all(&data) {
                eprintln!("Failed to send frame data: {}", e);
                break;
            }
            
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream: {}", e);
                break;
            }
        } else {
            eprintln!("Failed to capture frame");
            break;
        }
    }
    
    println!("Client disconnected: {:?}", stream.peer_addr());
}

fn main() {
    println!("Live Server is starting on {}...", SERVER_ADDR);
    
    let listener = TcpListener::bind(SERVER_ADDR).expect("Failed to bind to address");
    println!("Server listening on {}", SERVER_ADDR);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each client
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
