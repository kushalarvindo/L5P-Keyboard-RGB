use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use device_query::{DeviceQuery, DeviceState, Keycode};
use rand::Rng;

use crate::manager::Inner;

struct Ripple {
    x_origin: f32,
    color: [f32; 3],
    start_time: Instant,
}

fn key_x(key: &Keycode) -> f32 {
    use Keycode::*;
    match key {
        Escape | Grave | Tab | CapsLock | LShift | LControl | LMeta | LAlt => 0.05,
        Q | A | Z | Key1 | F1 => 0.15,
        W | S | X | Key2 | F2 => 0.2,
        E | D | C | Key3 | F3 => 0.25,
        R | F | V | Key4 | F4 => 0.3,
        T | G | B | Key5 | F5 => 0.35,
        Y | H | N | Key6 | F6 => 0.45,
        U | J | M | Key7 | F7 => 0.5,
        I | K | Comma | Key8 | F8 => 0.55,
        O | L | Dot | Key9 | F9 => 0.6,
        P | Semicolon | Slash | Key0 | F10 => 0.65,
        LeftBracket | RightBracket | Minus | Equal | F11 | F12 => 0.7,
        Enter | BackSlash | Backspace | RShift | RControl | RAlt | RMeta => 0.75,
        Up | Down | Left | Right | Insert | Delete | Home | End | PageUp | PageDown => 0.8,
        Numpad0 | Numpad1 | Numpad4 | Numpad7 => 0.85,
        Numpad2 | Numpad5 | Numpad8 | NumpadDivide => 0.9,
        Numpad3 | Numpad6 | Numpad9 | NumpadMultiply | NumpadSubtract | NumpadAdd | NumpadEnter => 0.95,
        Space => 0.4,
        _ => 0.5,
    }
}

pub fn play(manager: &mut Inner, bg_color: [u8; 3], speed: u8, width: f32) {
    let device_state = DeviceState::new();
    let mut last_keys: Vec<Keycode> = vec![];
    let mut ripples: Vec<Ripple> = Vec::new();
    let mut rng = rand::thread_rng();

    // Speed multiplier, speed is 0-255
    let speed_mult = 0.5 + (speed as f32 / 255.0) * 2.0;

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let keys = device_state.get_keys();
        
        for key in &keys {
            if !last_keys.contains(key) {
                // Spawn a new ripple!
                let color = [
                    rng.gen_range(50.0..255.0),
                    rng.gen_range(50.0..255.0),
                    rng.gen_range(50.0..255.0),
                ];
                
                ripples.push(Ripple {
                    x_origin: key_x(key),
                    color,
                    start_time: Instant::now(),
                });
                
                manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
            }
        }
        
        last_keys = keys;

        // Calculate colors for each zone
        let mut target = [0; 12];
        let now = Instant::now();
        let zones_x = [0.125, 0.375, 0.625, 0.875];
        
        for (z_idx, z_x) in zones_x.iter().enumerate() {
            let mut final_r = bg_color[0] as f32;
            let mut final_g = bg_color[1] as f32;
            let mut final_b = bg_color[2] as f32;

            for ripple in &ripples {
                let elapsed = now.duration_since(ripple.start_time).as_secs_f32();
                // Radius of the ripple expands over time
                let radius = elapsed * speed_mult;
                
                // Distance from ripple origin to this zone
                let dist = (z_x - ripple.x_origin).abs();
                
                // If the ripple's edge is currently passing over this zone
                // The width defines how thick the ring is
                let dist_to_edge = (radius - dist).abs();
                
                if dist_to_edge < width {
                    // Calculate intensity (1.0 at exactly the edge, fading out up to `width`)
                    let intensity = 1.0 - (dist_to_edge / width);
                    // Also fade the whole ripple out over time (max 2 seconds)
                    let time_fade = (1.0 - (elapsed / 2.0)).clamp(0.0, 1.0);
                    let actual_intensity = intensity * time_fade;
                    
                    final_r += (ripple.color[0] - final_r) * actual_intensity;
                    final_g += (ripple.color[1] - final_g) * actual_intensity;
                    final_b += (ripple.color[2] - final_b) * actual_intensity;
                }
            }
            
            target[z_idx * 3 + 0] = final_r.clamp(0.0, 255.0) as u8;
            target[z_idx * 3 + 1] = final_g.clamp(0.0, 255.0) as u8;
            target[z_idx * 3 + 2] = final_b.clamp(0.0, 255.0) as u8;
        }

        // Clean up old ripples
        ripples.retain(|r| now.duration_since(r.start_time).as_secs_f32() < 2.0);

        let _ = manager.keyboard.transition_colors_to(&target, 5, 1);
        thread::sleep(Duration::from_millis(30)); // ~33 fps
    }
}
