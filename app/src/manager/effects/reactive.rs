use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{Inner, profile::Profile};
use device_query::{DeviceQuery, DeviceState, Keycode};

pub fn play(manager: &mut Inner, profile: &mut Profile, typing_color: [u8; 3], bg_color: [u8; 3]) {
    let device_state = DeviceState::new();
    let mut last_keys: Vec<Keycode> = vec![];
    
    // Smooth transition variables
    let mut current_colors = [0.0; 12];
    for z in 0..4 {
        for c in 0..3 {
            current_colors[z * 3 + c] = bg_color[c] as f32;
        }
    }
    
    // We can define 'speed' to control fade speed (higher speed = faster fade out)
    let fade_factor = match profile.speed {
        0..=1 => 0.05,
        2 => 0.1,
        3 => 0.2,
        4..=255 => 0.3,
    };

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let keys = device_state.get_keys();
        let mut pressed = false;
        
        for key in &keys {
            if !last_keys.contains(key) {
                pressed = true;
                break;
            }
        }
        last_keys = keys;

        if pressed {
            // Splash!
            for z in 0..4 {
                for c in 0..3 {
                    current_colors[z * 3 + c] = typing_color[c] as f32;
                }
            }
        } else {
            // Fade out
            for z in 0..4 {
                for c in 0..3 {
                    current_colors[z * 3 + c] += (bg_color[c] as f32 - current_colors[z * 3 + c]) * fade_factor;
                }
            }
        }

        let mut target = [0; 12];
        for i in 0..12 {
            target[i] = current_colors[i] as u8;
        }

        let _ = manager.keyboard.transition_colors_to(&target, 5, 1);
        thread::sleep(Duration::from_millis(30)); // 33 fps approx
    }
}
