use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::Inner;
use sysinfo::{System, CpuRefreshKind, RefreshKind};

pub fn play(manager: &mut Inner) {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()).with_memory(),
    );
    
    // Need to sleep once before getting cpu usage
    sys.refresh_cpu_usage();
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

    let mut current_colors = [0.0; 12];

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        
        let cpu_usage = sys.global_cpu_usage(); // 0.0 to 100.0
        let mem_total = sys.total_memory() as f32;
        let mem_used = sys.used_memory() as f32;
        let mem_usage = if mem_total > 0.0 { (mem_used / mem_total) * 100.0 } else { 0.0 };
        
        // We can map Load (0.0 to 100.0) to colors:
        // Green -> Yellow -> Red
        let get_color_for_load = |load: f32| -> [f32; 3] {
            if load < 50.0 {
                // Green to Yellow
                let percent = load / 50.0;
                [percent * 255.0, 255.0, 0.0]
            } else {
                // Yellow to Red
                let percent = (load - 50.0) / 50.0;
                [255.0, 255.0 - (percent * 255.0), 0.0]
            }
        };

        // Zone 1 and 2 = CPU
        // Zone 3 and 4 = RAM
        let cpu_color = get_color_for_load(cpu_usage);
        let mem_color = get_color_for_load(mem_usage);

        for c in 0..3 {
            current_colors[0 + c] = cpu_color[c];
            current_colors[3 + c] = cpu_color[c];
            current_colors[6 + c] = mem_color[c];
            current_colors[9 + c] = mem_color[c];
        }

        let mut target = [0; 12];
        for i in 0..12 {
            target[i] = current_colors[i] as u8;
        }

        let _ = manager.keyboard.transition_colors_to(&target, 5, 1);
        thread::sleep(Duration::from_millis(500)); // Update every 500ms
    }
}
