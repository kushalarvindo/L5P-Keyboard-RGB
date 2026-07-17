use std::{sync::atomic::Ordering, thread, time::Duration};
use chrono::{Local, Timelike};
use crate::manager::Inner;

pub fn play(manager: &mut Inner) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let now = Local::now();
        let seconds_since_midnight = now.hour() * 3600 + now.minute() * 60 + now.second();
        
        let mut target = [0; 12];
        
        let sun_color = [255.0, 180.0, 0.0];
        let day_sky = [60.0, 150.0, 255.0];
        let moon_color = [255.0, 255.0, 255.0];
        let night_sky = [5.0, 5.0, 40.0];
        
        // 6 AM is 21600 seconds, 6 PM is 64800 seconds
        let is_day = seconds_since_midnight >= 21600 && seconds_since_midnight < 64800;
        
        let (fg_color, bg_color, progress) = if is_day {
            let progress = (seconds_since_midnight - 21600) as f32 / 43200.0;
            (sun_color, day_sky, progress)
        } else {
            let mut shifted = seconds_since_midnight;
            if shifted < 21600 {
                shifted += 86400; // Add 24 hours if it's past midnight
            }
            let progress = (shifted - 64800) as f32 / 43200.0;
            (moon_color, night_sky, progress)
        };
        
        // Progress goes from 0.0 to 1.0. Map it across the 4 zones (0 to 3).
        let position = progress * 3.0;
        
        for zone in 0..4 {
            // Distance from the celestial body to this zone
            let distance = (position - zone as f32).abs();
            // Brightness factor (1.0 when perfectly centered, drops to 0.0 when 1 zone away)
            let blend = (1.0 - distance).max(0.0).min(1.0);
            
            // Blend foreground and background
            for color_idx in 0..3 {
                let r = bg_color[color_idx] + (fg_color[color_idx] - bg_color[color_idx]) * blend;
                target[zone * 3 + color_idx] = r as u8;
            }
        }
        
        let _ = manager.keyboard.transition_colors_to(&target, 50, 1);
        
        // Update every 1 second
        thread::sleep(Duration::from_millis(1000));
    }
}
