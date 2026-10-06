use std::fs;


/// Maximum number of CPU cores supported by the stack array
pub const MAX_CORES: usize = 128;


/// Holds CPU time metrics (ticks) extracted from /proc/stat
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuTime {
    /// Time spent in user space (normal programs, browsers, games).
    pub user: u64,
    /// Time spent in user space with low priority (changed via `nice` command).
    pub nice: u64,
    /// Time spent in kernel space (system calls, drivers, kernel tasks).
    pub system: u64,
    /// Time spent doing nothing (CPU was idle).
    pub idle: u64,
    /// Time spent idling while waiting for I/O operations (e.g., disk storage).
    pub iowait: u64,
    /// Time spent servicing hardware interrupts.
    pub irq: u64,
    /// Time spent servicing software interrupts.
    pub softirq: u64,
    /// Stolen time spent waiting for the hypervisor to allocate physical CPU cycles to this VM.
    pub steal: u64,
}


/// Static hardware specifications of the CPU
#[derive(Debug, Clone, Default)]
pub struct CpuHardwareInfo {
    pub vendor: String,
    pub product: String,
    pub serial: Option<String>,
    pub slot: String,
    pub version: String,
    pub current_speed_mhz: u64,
    pub max_speed_mhz: u64,
    pub bit_width: u8,
    pub bus_clock_mhz: u64,
    pub logical_cores: usize,
    pub physical_cores: usize,
}


/// A complete point-in-time snapshot of the CPU state
#[derive(Debug, Clone)]
pub struct CpuSnapshot {
    pub main: CpuHardwareInfo,
    pub system_total: CpuTime,
    pub cores_array: [CpuTime; MAX_CORES],
    pub cores_count: usize,
}


impl Default for CpuSnapshot {
    fn default() -> Self {
        Self {
            main: CpuHardwareInfo::default(),
            system_total: CpuTime::default(),
            cores_array: [CpuTime::default(); MAX_CORES],
            cores_count: 0,
        }
    }
}


impl CpuSnapshot {
    // Returns an iterator for safely traversing the actively used cores
    pub fn cores(&self) -> &[CpuTime] {
        &self.cores_array[..self.cores_count]
    }
}

pub struct CpuCollector {
    hardware_info: CpuHardwareInfo,
}

impl CpuCollector {
    /// Initializes the collector and loads immutable hardware data
    pub fn new() -> Self {
        let mut hw = CpuHardwareInfo::default();
        
        Self::parse_cpuinfo(&mut hw);
        Self::parse_sys_dmi(&mut hw);
        Self::parse_max_frequency(&mut hw);
        
        // Contextual hardware defaults
        hw.bit_width = if cfg!(target_arch = "x86_64") { 64 } else { 32 };
        hw.bus_clock_mhz = 100;

        Self { hardware_info: hw }
    }

    /// Parses /proc/cpuinfo once to calculate core counts and identity strings
    fn parse_cpuinfo(hw: &mut CpuHardwareInfo) {
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let mut unique_cores: [(u16, u16); MAX_CORES] = [(0, 0); MAX_CORES];
            let mut unique_count = 0;
            let mut current_physical_id = 0u16;

            for line in cpuinfo.lines() {
                let mut parts = line.splitn(2, ':');
                let key = parts.next().map(|s| s.trim());
                let val = parts.next().map(|s| s.trim());

                match (key, val) {
                    (Some("vendor_id"), Some(v)) => hw.vendor = v.to_string(),
                    (Some("model name"), Some(v)) => hw.product = v.to_string(),
                    (Some("processor"), _) => hw.logical_cores += 1,
                    (Some("physical id"), Some(v)) => current_physical_id = v.parse().unwrap_or(0),
                    (Some("core id"), Some(v)) => {
                        let current_core_id = v.parse().unwrap_or(0);
                        let pair = (current_physical_id, current_core_id);
                        
                        // Avoid core duplicates (e.g. due to Hyper-Threading / SMT)
                        if !unique_cores[..unique_count].contains(&pair) && unique_count < MAX_CORES {
                            unique_cores[unique_count] = pair;
                            unique_count += 1;
                        }
                    }
                    _ => {}
                }
            }
            hw.physical_cores = unique_count;
        }
    }

    /// Reads static DMI parameters directly from the sysfs tree
    fn parse_sys_dmi(hw: &mut CpuHardwareInfo) {
        hw.slot = fs::read_to_string("/sys/class/dmi/id/processor_version")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
            
        hw.version = fs::read_to_string("/sys/class/dmi/id/bios_version")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
            
        hw.serial = fs::read_to_string("/sys/class/dmi/id/product_serial")
            .map(|s| s.trim().to_string())
            .ok()
            .filter(|s| !s.is_empty() && s != "None");
    }

    /// Reads the hardware scale capacity frequency
    fn parse_max_frequency(hw: &mut CpuHardwareInfo) {
        if let Ok(max_freq) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
            hw.max_speed_mhz = max_freq.trim().parse::<u64>().unwrap_or(0) / 1000;
        }
    }

    /// Generates a fresh point-in-time snapshot with up-to-date metrics
    pub fn collect(&self) -> CpuSnapshot {
        let mut snapshot = CpuSnapshot {
            main: self.hardware_info.clone(),
            ..CpuSnapshot::default()
        };

        // Query the active core clock speed on demand
        if let Ok(cur_freq) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
            snapshot.main.current_speed_mhz = cur_freq.trim().parse::<u64>().unwrap_or(0) / 1000;
        }

        // Process runtime time stat tokens atomically
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            for line in stat.lines() {
                if !line.starts_with("cpu") { break; }

                let mut parts = line.split_whitespace();
                let cpu_id = parts.next();

                let mut t = CpuTime::default();
                t.user    = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.nice    = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.system  = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.idle    = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.iowait  = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.irq     = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.softirq = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                t.steal   = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);

                match cpu_id {
                    Some("cpu") => snapshot.system_total = t,
                    Some(id) if id.starts_with("cpu") => {
                        if snapshot.cores_count < MAX_CORES {
                            snapshot.cores_array[snapshot.cores_count] = t;
                            snapshot.cores_count += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        snapshot
    }
}