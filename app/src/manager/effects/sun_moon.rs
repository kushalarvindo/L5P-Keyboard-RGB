use std::{
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use chrono::{Local, Timelike};
use crate::manager::Inner;

pub fn play(manager: &mut Inner) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let second = now.second();
        
        let time_in_hours = hour as f32 + (minute as f32 / 60.0) + (second as f32 / 3600.0);
        
        let is_daytime = time_in_hours >= 6.0 && time_in_hours < 18.0;
        
        let mut target = [0; 12];
        
        // Background color
        let (bg_r, bg_g, bg_b) = if is_daytime {
            (135, 206, 235) // Sky Blue
        } else {
            (10, 10, 40) // Dark Blue
        };
        
        for i in 0..4 {
            target[i * 3] = bg_r;
            target[i * 3 + 1] = bg_g;
            target[i * 3 + 2] = bg_b;
        }

        // Celestial body color (Sun = Yellow/Orange, Moon = White/Silver)
        let (body_r, body_g, body_b) = if is_daytime {
            (255, 200, 0) // Sun
        } else {
            (200, 200, 255) // Moon
        };

        // Calculate position based on the 12-hour period
        // For day: 6:00 is pos 0, 18:00 is pos 4
        // For night: 18:00 is pos 0, 6:00 is pos 4
        let pos = if is_daytime {
            ((time_in_hours - 6.0) / 12.0) * 4.0
        } else {
            let mut night_hours = time_in_hours - 18.0;
            if night_hours < 0.0 {
                night_hours += 24.0;
            }
            (night_hours / 12.0) * 4.0
        };

        // Draw the body smoothly across zones
        for z in 0..4 {
            let dist = (z as f32 + 0.5 - pos).abs();
            // width of celestial body
            let width = 0.8;
            
            if dist < width {
                let intensity = 1.0 - (dist / width);
                
                let cur_r = target[z * 3] as f32;
                let cur_g = target[z * 3 + 1] as f32;
                let cur_b = target[z * 3 + 2] as f32;
                
                target[z * 3] = (cur_r + (body_r as f32 - cur_r) * intensity) as u8;
                target[z * 3 + 1] = (cur_g + (body_g as f32 - cur_g) * intensity) as u8;
                target[z * 3 + 2] = (cur_b + (body_b as f32 - cur_b) * intensity) as u8;
            }
        }

        let _ = manager.keyboard.transition_colors_to(&target, 5, 1);
        
        // We only need to update occasionally since it's based on time
        thread::sleep(Duration::from_millis(500));
    }
}
