use lz4_flex::compress_prepend_size;
use scrap::Frame;

// Scale factor - 2 means half resolution (much faster)
const SCALE: usize = 2;

pub fn encode_frame(frames: Frame, w: usize, h: usize) -> (Vec<u8>, u16, u16) {
    let raw_data = frames.as_ref();
    
    // Calculate scaled dimensions
    let new_w = w / SCALE;
    let new_h = h / SCALE;
    
    // Scale down and convert BGRA to RGB in one pass
    let mut rgb_data = Vec::with_capacity(new_w * new_h * 3);
    
    for y in 0..new_h {
        let src_y = y * SCALE;
        for x in 0..new_w {
            let src_x = x * SCALE;
            let src_idx = (src_y * w + src_x) * 4;
            // BGRA -> RGB
            rgb_data.push(raw_data[src_idx + 2]); // R
            rgb_data.push(raw_data[src_idx + 1]); // G
            rgb_data.push(raw_data[src_idx]);     // B
        }
    }
    
    // Compress with LZ4 (very fast)
    let compressed = compress_prepend_size(&rgb_data);
    
    (compressed, new_w as u16, new_h as u16)
}