use eframe::egui;
use lz4_flex::decompress_size_prepended;
use std::io::{BufReader, Read};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

const SERVER_ADDR: &str = "127.0.0.1:9999";

struct LiveViewApp {
    texture: Option<egui::TextureHandle>,
    image_data: Arc<Mutex<Option<ImageData>>>,
    connected: Arc<Mutex<bool>>,
    error_message: Arc<Mutex<Option<String>>>,
}

struct ImageData {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl LiveViewApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let image_data: Arc<Mutex<Option<ImageData>>> = Arc::new(Mutex::new(None));
        let connected = Arc::new(Mutex::new(false));
        let error_message: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let image_data_clone = Arc::clone(&image_data);
        let connected_clone = Arc::clone(&connected);
        let error_message_clone = Arc::clone(&error_message);
        let ctx = cc.egui_ctx.clone();

        thread::spawn(move || {
            Self::network_loop(image_data_clone, connected_clone, error_message_clone, ctx);
        });

        Self {
            texture: None,
            image_data,
            connected,
            error_message,
        }
    }

    fn network_loop(
        image_data: Arc<Mutex<Option<ImageData>>>,
        connected: Arc<Mutex<bool>>,
        error_message: Arc<Mutex<Option<String>>>,
        ctx: egui::Context,
    ) {
        let mut frame_buf: Vec<u8> = Vec::with_capacity(1024 * 1024);
        
        loop {
            println!("Connecting to server at {}...", SERVER_ADDR);
            
            match TcpStream::connect(SERVER_ADDR) {
                Ok(stream) => {
                    println!("Connected to server!");
                    let _ = stream.set_nodelay(true);
                    let mut reader = BufReader::with_capacity(512 * 1024, stream);
                    
                    *connected.lock().unwrap() = true;
                    *error_message.lock().unwrap() = None;
                    ctx.request_repaint();

                    loop {
                        // Read header: size (4) + width (2) + height (2)
                        let mut header = [0u8; 8];
                        if let Err(e) = reader.read_exact(&mut header) {
                            eprintln!("Failed to read header: {}", e);
                            *error_message.lock().unwrap() = Some(format!("Connection lost: {}", e));
                            break;
                        }
                        
                        let frame_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
                        let width = u16::from_be_bytes([header[4], header[5]]) as usize;
                        let height = u16::from_be_bytes([header[6], header[7]]) as usize;

                        // Read compressed frame data
                        frame_buf.resize(frame_size, 0);
                        if let Err(e) = reader.read_exact(&mut frame_buf) {
                            eprintln!("Failed to read frame data: {}", e);
                            *error_message.lock().unwrap() = Some(format!("Connection lost: {}", e));
                            break;
                        }

                        // Decompress LZ4
                        match decompress_size_prepended(&frame_buf) {
                            Ok(rgb_data) => {
                                // Convert RGB to RGBA for egui
                                let mut rgba = Vec::with_capacity(width * height * 4);
                                for chunk in rgb_data.chunks(3) {
                                    rgba.push(chunk[0]); // R
                                    rgba.push(chunk[1]); // G
                                    rgba.push(chunk[2]); // B
                                    rgba.push(255);      // A
                                }
                                
                                *image_data.lock().unwrap() = Some(ImageData {
                                    pixels: rgba,
                                    width,
                                    height,
                                });
                                ctx.request_repaint();
                            }
                            Err(e) => {
                                eprintln!("Failed to decompress: {}", e);
                            }
                        }
                    }

                    *connected.lock().unwrap() = false;
                    ctx.request_repaint();
                }
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                    *error_message.lock().unwrap() = Some(format!("Failed to connect: {}", e));
                    *connected.lock().unwrap() = false;
                    ctx.request_repaint();
                }
            }

            thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

impl eframe::App for LiveViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let connected = *self.connected.lock().unwrap();
                if connected {
                    ui.label(egui::RichText::new("● Connected").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("● Disconnected").color(egui::Color32::RED));
                }
                
                if let Some(ref err) = *self.error_message.lock().unwrap() {
                    ui.label(egui::RichText::new(err).color(egui::Color32::YELLOW));
                }
            });
            
            ui.separator();

            let img_opt = self.image_data.lock().unwrap().take();
            if let Some(img_data) = img_opt {
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [img_data.width, img_data.height],
                    &img_data.pixels,
                );
                
                match &mut self.texture {
                    Some(texture) => {
                        texture.set(color_image, egui::TextureOptions::LINEAR);
                    }
                    None => {
                        self.texture = Some(ctx.load_texture(
                            "screen_capture",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                }
            }
            
            if let Some(ref texture) = self.texture {
                let available_size = ui.available_size();
                let img_size = texture.size_vec2();
                let scale = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                let display_size = img_size * scale;
                
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), display_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for stream...");
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("Live Screen Viewer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Live Screen Viewer",
        options,
        Box::new(|cc| Ok(Box::new(LiveViewApp::new(cc)))),
    )
}
