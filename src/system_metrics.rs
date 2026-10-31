use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub uptime: String,
    pub load: String,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_available_mb: u64,
    pub ram_percent: u8,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            uptime: "N/A".to_string(),
            load: "0.0, 0.0, 0.0".to_string(),
            ram_total_mb: 1024,
            ram_used_mb: 512,
            ram_available_mb: 512,
            ram_percent: 50,
            swap_total_mb: 2048,
            swap_used_mb: 0,
        }
    }
}

pub fn collect_system_metrics() -> SystemMetrics {
    let mut m = SystemMetrics::default();

    // 1. Meminfo
    if let Ok(data) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        let mut swap_total_kb = 0u64;
        let mut swap_free_kb = 0u64;

        for line in data.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            if let Ok(val) = parts[1].parse::<u64>() {
                match parts[0] {
                    "MemTotal:" => total_kb = val,
                    "MemAvailable:" => avail_kb = val,
                    "SwapTotal:" => swap_total_kb = val,
                    "SwapFree:" => swap_free_kb = val,
                    _ => {}
                }
            }
        }

        if total_kb > 0 {
            let total_mb = total_kb / 1024;
            let avail_mb = avail_kb / 1024;
            let used_mb = total_mb.saturating_sub(avail_mb);
            m.ram_total_mb = total_mb;
            m.ram_available_mb = avail_mb;
            m.ram_used_mb = used_mb;
            m.ram_percent = ((used_mb as f64 / total_mb as f64) * 100.0) as u8;
        }

        if swap_total_kb > 0 {
            m.swap_total_mb = swap_total_kb / 1024;
            m.swap_used_mb = (swap_total_kb.saturating_sub(swap_free_kb)) / 1024;
        }
    }

    // 2. Loadavg
    if let Ok(data) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = data.split_whitespace().collect();
        if parts.len() >= 3 {
            m.load = format!("{}, {}, {}", parts[0], parts[1], parts[2]);
        }
    }

    // 3. Uptime
    if let Ok(data) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = data.split_whitespace().next() {
            if let Ok(sec) = first.parse::<f64>() {
                let hours = (sec / 3600.0) as u64;
                let mins = ((sec % 3600.0) / 60.0) as u64;
                m.uptime = format!("{hours}h {mins}m");
            }
        }
    }

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_metrics() {
        let m = collect_system_metrics();
        // Em qualquer ambiente deve retornar um struct válido sem pânico
        assert!(m.ram_percent <= 100);
    }
}
