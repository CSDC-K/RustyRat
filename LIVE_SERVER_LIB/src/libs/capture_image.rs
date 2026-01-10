use scrap::{Capturer, Display};
use std::{io::ErrorKind, thread, time::Duration};

use crate::libs::encode_image;

pub struct ScreenCapturer {
    capturer: Capturer,
    width: usize,
    height: usize,
}

impl ScreenCapturer {
    pub fn new() -> Self {
        let display = Display::primary().expect("Couldn't find primary display.");
        let capturer = Capturer::new(display).expect("Couldn't begin capture.");
        let width = capturer.width();
        let height = capturer.height();
        
        ScreenCapturer {
            capturer,
            width,
            height,
        }
    }

    /// Captures a single frame and returns (compressed_data, width, height)
    pub fn capture_frame(&mut self) -> Option<(Vec<u8>, u16, u16)> {
        loop {
            match self.capturer.frame() {
                Ok(frame) => {
                    let (data, w, h) = encode_image::encode_frame(frame, self.width, self.height);
                    return Some((data, w, h));
                }
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        // Wait and try again.
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    } else {
                        eprintln!("Capture error: {}", error);
                        return None;
                    }
                }
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}