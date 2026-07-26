use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::Inner;

#[allow(unused_mut, unused_variables)]
fn get_colors(use_accent: bool, hot_color: [u8; 3], cool_color: [u8; 3]) -> ([f32; 12], [f32; 12]) {
    let mut hot = hot_color;
    #[cfg(target_os = "windows")]
    if use_accent {
        if let Some(accent) = crate::util::get_windows_accent_color() {
            hot = accent;
        }
    }
    
    let mut temp_cool = [0.0; 12];
    let mut temp_hot = [0.0; 12];
    for z in 0..4 {
        for c in 0..3 {
            temp_hot[z * 3 + c] = hot[c] as f32;
            temp_cool[z * 3 + c] = cool_color[c] as f32;
        }
    }
    (temp_hot, temp_cool)
}

#[cfg(target_os = "linux")]
pub fn play(manager: &mut Inner, use_accent: bool, hot_color: [u8; 3], cool_color: [u8; 3]) {
    use sysinfo::{Components, System};
    let safe_temp = 20.0;
    let ramp_boost = 1.6;
    let (temp_hot, temp_cool) = get_colors(use_accent, hot_color, cool_color);

    let mut color_differences: [f32; 12] = [0.0; 12];
    for index in 0..12 {
        color_differences[index] = temp_hot[index] - temp_cool[index];
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut components = Components::new_with_refreshed_list();

    for component in components.iter_mut() {
        if component.label().contains("Tctl") {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                component.refresh();
                let temp = component.temperature();
                if let Some(temperature) = temp {
                    let mut adjusted_temp = temperature - safe_temp;
                    if adjusted_temp < 0.0 {
                        adjusted_temp = 0.0;
                    }
                    let temp_percent = (adjusted_temp / 100.0) * ramp_boost;

                    let mut target = [0.0; 12];
                    for index in 0..12 {
                        target[index] = color_differences[index].mul_add(temp_percent, temp_cool[index]);
                    }
                    let _ = manager.keyboard.transition_colors_to(&target.map(|val| val as u8), 5, 1);
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn play(manager: &mut Inner, use_accent: bool, hot_color: [u8; 3], cool_color: [u8; 3]) {
    use std::os::windows::process::CommandExt;
    
    let safe_temp = 20.0;
    let ramp_boost = 1.6;
    let (temp_hot, temp_cool) = get_colors(use_accent, hot_color, cool_color);

    let mut color_differences: [f32; 12] = [0.0; 12];
    for index in 0..12 {
        color_differences[index] = temp_hot[index] - temp_cool[index];
    }

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-WmiObject MSAcpi_ThermalZoneTemperature -Namespace 'root/wmi').CurrentTemperature",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        let mut temperature = None;
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().next() {
                if let Ok(temp_kelvin_tenths) = line.trim().parse::<f32>() {
                    // Convert from tenths of degrees Kelvin to Celsius
                    temperature = Some((temp_kelvin_tenths / 10.0) - 273.15);
                }
            }
        }

        let temp = temperature.unwrap_or(20.0); // Default to cool if unable to read
        let mut adjusted_temp = temp - safe_temp;
        if adjusted_temp < 0.0 {
            adjusted_temp = 0.0;
        }
        let temp_percent = (adjusted_temp / 100.0) * ramp_boost;

        let mut target = [0.0; 12];
        for index in 0..12 {
            target[index] = color_differences[index].mul_add(temp_percent, temp_cool[index]);
        }
        let _ = manager.keyboard.transition_colors_to(&target.map(|val| val as u8), 5, 1);
        
        // Wait longer on Windows because spawning powershell is expensive
        thread::sleep(Duration::from_millis(1000));
    }
}
