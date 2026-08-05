use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use scrap::{Capturer, Display, Frame, TraitCapturer, TraitPixelBuffer};

use crate::manager::Inner;

#[derive(Clone, Copy)]
struct ScreenDimensions {
    src: (u32, u32),
    dest: (u32, u32),
}

pub fn play(manager: &mut Inner, fps: u8, saturation_boost: f32, smoothness: bool) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        //Display setup
        let display = Display::all().unwrap().remove(0);

        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

        let dimensions = ScreenDimensions {
            src: (capturer.width() as u32, capturer.height() as u32),
            dest: (4, 1),
        };

        let seconds_per_frame = Duration::from_nanos(1_000_000_000 / u64::from(fps));

        #[cfg(target_os = "windows")]
        let mut try_gdi = 1;
        
        let mut last_update = Instant::now();
        let mut current_colors = [0f32; 12];
        let mut first_frame = true;

        while !manager.stop_signals.keyboard_stop_signal.load(Ordering::SeqCst) {
            #[allow(clippy::single_match)]
            match capturer.frame(seconds_per_frame) {
                Ok(frame) => {
                    // Drain the frame queue to prevent 1-second lag, but only update keyboard at the requested FPS
                    if last_update.elapsed() >= seconds_per_frame {
                        let rgb = process_frame(frame, dimensions, saturation_boost);
                        
                        if first_frame || !smoothness {
                            for i in 0..12 {
                                current_colors[i] = rgb[i] as f32;
                            }
                            first_frame = false;
                        } else {
                            // Exponential moving average for smooth color transitions
                            let alpha = 0.3; // Smoothing factor for buttery smooth transitions
                            for i in 0..12 {
                                current_colors[i] = current_colors[i] + alpha * (rgb[i] as f32 - current_colors[i]);
                            }
                        }
                        
                        let mut final_rgb = [0u8; 12];
                        for i in 0..12 {
                            final_rgb[i] = current_colors[i].round() as u8;
                        }

                        manager.keyboard.set_colors_to(&final_rgb).unwrap();
                        last_update = Instant::now();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        try_gdi = 0;
                    }
                }
                Err(error) => match error.kind() {
                    std::io::ErrorKind::WouldBlock =>
                    {
                        #[cfg(target_os = "windows")]
                        if try_gdi > 0 && !capturer.is_gdi() {
                            if try_gdi > 3 {
                                capturer.set_gdi();
                                try_gdi = 0;
                            }
                            try_gdi += 1;
                        }
                    }
                    _ =>
                    {
                        #[cfg(windows)]
                        if !capturer.is_gdi() {
                            capturer.set_gdi();
                            continue;
                        }
                    }
                },
            }
        }
    }
}

fn process_frame(frame: Frame, dimensions: ScreenDimensions, saturation_boost: f32) -> [u8; 12] {
    let Frame::PixelBuffer(buf) = frame else {
        unreachable!("Attempted to extract vec from Texture variant in the Ambient effect");
    };

    let frame_vec = buf.data();
    let width = dimensions.src.0 as usize;
    let height = dimensions.src.1 as usize;
    let slice_width = width / 4;
    
    // Sample pixels across the horizontal center of the screen
    let y = height / 2;
    let mut rgba = [0u8; 16];
    
    for zone in 0..4 {
        // Find the center X coordinate of the zone
        let x = zone * slice_width + (slice_width / 2);
        
        // Calculate the flat array index (BGRA format is 4 bytes per pixel)
        let idx = (y * width + x) * 4;
        
        // Bounds check just in case
        if idx + 2 < frame_vec.len() {
            let b = frame_vec[idx];
            let g = frame_vec[idx + 1];
            let r = frame_vec[idx + 2];
            
            rgba[zone * 4] = r;
            rgba[zone * 4 + 1] = g;
            rgba[zone * 4 + 2] = b;
            rgba[zone * 4 + 3] = 255;
        }
    }

    let mut img = photon_rs::PhotonImage::new(rgba.to_vec(), 4, 1);
    photon_rs::colour_spaces::saturate_hsv(&mut img, saturation_boost);

    let raw = img.get_raw_pixels();
    let mut rgb: [u8; 12] = [0; 12];
    for (src, dst) in raw.chunks_exact(4).zip(rgb.chunks_exact_mut(3)) {
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
    }

    rgb
}
