use std::fs;


/// Holds runtime RAM metrics parsed from /proc/meminfo
#[derive(Debug, Clone, Copy, Default)]
pub struct RamInfo {
    pub total_kb: u64,
    pub free_kb: u64,
    pub available_kb: u64
}


/// Holds runtime Swap metrics parsed from /proc/meminfo
#[derive(Debug, Clone, Copy, Default)]
pub struct SwapInfo {
    pub total_kb: u64,
    pub free_kb: u64
}


/// A complete point-in-time snapshot of the system memory state
#[derive(Debug, Clone, Copy, Default)]
pub struct MemorySnapshot {
    pub ram: RamInfo,
    pub swap: SwapInfo
}


/// Stateless collector responsible for gathering system memory metrics
pub struct MemoryCollector;


impl MemoryCollector {
    /// Initializes a new memory collector instance
    pub fn new() -> Self {
        Self
    }

    /// Generates a fresh point-in-time snapshot of RAM and Swap memory usage
    pub fn collect(&self) -> MemorySnapshot {
        let mut snapshot = MemorySnapshot::default();

        // Atomically read the system memory file into memory
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                let mut parts = line.split_whitespace();
                let key = parts.next();
                let val = parts.next().and_then(|v| v.parse::<u64>().ok());

                match (key, val) {
                    // Populate core RAM metrics
                    (Some("MemTotal:"), Some(v)) => snapshot.ram.total_kb = v,
                    (Some("MemFree:"), Some(v)) => snapshot.ram.free_kb = v,
                    (Some("MemAvailable:"), Some(v)) => snapshot.ram.available_kb = v,
                    
                    // Populate swap space metrics
                    (Some("SwapTotal:"), Some(v)) => snapshot.swap.total_kb = v,
                    (Some("SwapFree:"), Some(v)) => snapshot.swap.free_kb = v,
                    
                    _ => {}
                }
            }
        }

        snapshot
    }
}