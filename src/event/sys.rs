use anyhow::Result;
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
struct SysStats {
    cpu: String,
    cores: String,
    mem: String,
    mem_pct: String,
    mem_used: String,
    mem_total: String,
}

fn gather() -> SysStats {
    let mut sys = System::new();

    // CPU: need two refreshes with a gap for meaningful usage
    sys.refresh_cpu_usage();
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_usage();

    let cpu_avg: f32 = if sys.cpus().is_empty() {
        0.0
    } else {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
    };
    let cores = sys.cpus().len();

    // Memory
    sys.refresh_memory();
    let total_bytes = sys.total_memory();
    let used_bytes = sys.used_memory();
    let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_pct = if total_bytes > 0 {
        100.0 * used_bytes as f64 / total_bytes as f64
    } else {
        0.0
    };

    SysStats {
        cpu: format!("{:.0}%", cpu_avg),
        cores: cores.to_string(),
        mem: format!("{used_gb:.1}/{total_gb:.0}G ({used_pct:.0}%)"),
        mem_pct: format!("{used_pct:.0}%"),
        mem_used: format!("{used_gb:.0}G"),
        mem_total: format!("{total_gb:.0}G"),
    }
}

pub fn run() -> Result<()> {
    let data = gather();
    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}
