use std::{sync::atomic::Ordering, thread, time::Duration};
use crate::manager::Inner;
use crossbeam_channel::bounded;

pub fn play(manager: &mut Inner) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            return;
        }
    };
    
    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(_) => {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            return;
        }
    };

    let (tx, rx) = bounded(1);
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                let sum: f32 = data.iter().map(|s| s.abs()).sum();
                let avg = if data.is_empty() { 0.0 } else { sum / data.len() as f32 };
                let _ = tx.try_send(avg);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                let sum: f32 = data.iter().map(|&s| (s as f32 / i16::MAX as f32).abs()).sum();
                let avg = if data.is_empty() { 0.0 } else { sum / data.len() as f32 };
                let _ = tx.try_send(avg);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _: &_| {
                let sum: f32 = data.iter().map(|&s| ((s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0)).abs()).sum();
                let avg = if data.is_empty() { 0.0 } else { sum / data.len() as f32 };
                let _ = tx.try_send(avg);
            },
            err_fn,
            None,
        ),
        _ => {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            return;
        }
    };

    let stream = match stream {
        Ok(s) => s,
        Err(_) => {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            return;
        }
    };

    let _ = stream.play();

    let mut smoothed_intensity = 0.0;
    
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut level = 0.0;
        while let Ok(new_level) = rx.try_recv() {
            level = new_level;
        }

        let intensity = (level * 20.0).clamp(0.0, 1.0);
        
        if intensity > smoothed_intensity {
            smoothed_intensity = intensity;
        } else {
            smoothed_intensity -= 0.05;
            if smoothed_intensity < 0.0 {
                smoothed_intensity = 0.0;
            }
        }
        
        let mut target = [0; 12];
        
        let r = (smoothed_intensity * 255.0) as u8;
        let g = ((1.0 - (smoothed_intensity - 0.5).abs() * 2.0) * 255.0).clamp(0.0, 255.0) as u8;
        let b = ((1.0 - smoothed_intensity) * 255.0) as u8;

        for z in 0..4 {
            let mut z_r = r;
            let mut z_g = g;
            let mut z_b = b;
            
            if (z == 0 || z == 3) && smoothed_intensity < 0.5 {
                z_r = 0; z_g = 0; z_b = 0;
            } else if (z == 1 || z == 2) && smoothed_intensity < 0.1 {
                z_r = 0; z_g = 0; z_b = 0;
            }
            
            target[z * 3 + 0] = z_r;
            target[z * 3 + 1] = z_g;
            target[z * 3 + 2] = z_b;
        }
        
        let _ = manager.keyboard.transition_colors_to(&target, 5, 1);
        thread::sleep(Duration::from_millis(30));
    }
}
