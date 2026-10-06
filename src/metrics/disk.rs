use std::array;
use std::fs;
use std::str::FromStr;

/// Maximum number of physical block devices we can track simultaneously on the stack.
pub const MAX_DISKS: usize = 16;

/// Primitive telemetry values extracted from block devices.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiskMetric {
    pub reads_completed: u64,
    pub sectors_read: u64,
    pub writes_completed: u64,
    pub sectors_written: u64,
}

/// A fixed-size stack container holding live disk snapshots.
#[derive(Debug)]
pub struct DiskSnapshot {
    metrics: [DiskMetric; MAX_DISKS],
    names: [String; MAX_DISKS],
    count: usize,
}

impl Default for DiskSnapshot {
    fn default() -> Self {
        Self {
            metrics: [DiskMetric::default(); MAX_DISKS],
            names: array::from_fn(|_| String::new()),
            count: 0,
        }
    }
}

impl DiskSnapshot {
    pub fn metrics(&self) -> &[DiskMetric] {
        &self.metrics[..self.count]
    }

    pub fn names(&self) -> &[String] {
        &self.names[..self.count]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &DiskMetric)> {
        self.names[..self.count]
            .iter()
            .zip(self.metrics[..self.count].iter())
    }
}

pub struct DiskCollector;

impl DiskCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> DiskSnapshot {
        let mut snapshot = DiskSnapshot::default();

        if let Ok(disks) = fs::read_to_string("/proc/diskstats") {
            for disk in disks.lines() {
                // Break early if we have filled the stack-allocated array capacity
                if snapshot.count >= MAX_DISKS {
                    break;
                }

                let mut metrics = disk.split_whitespace();

                // Skip major and minor numbers (Columns 1 & 2)
                let _major = metrics.next();
                let _minor = metrics.next();

                // Extract device name (Column 3)
                let Some(device_name) = metrics.next() else { continue; };

                // Kernel device filtering layer
                if device_name.starts_with("loop") || device_name.starts_with("ram") {
                    continue;
                }

                // Sequential token processing to handle cross-kernel field shifting
                let reads_completed = metrics.next().and_then(|t| u64::from_str(t).ok());
                let _reads_merged = metrics.next(); // Skip column 5
                let sectors_read = metrics.next().and_then(|t| u64::from_str(t).ok());
                let _time_reading = metrics.next(); // Skip column 7
                let writes_completed = metrics.next().and_then(|t| u64::from_str(t).ok());
                let _writes_merged = metrics.next(); // Skip column 9
                let sectors_written = metrics.next().and_then(|t| u64::from_str(t).ok());

                // Safe structural unwrap validation
                if let (Some(r_comp), Some(s_read), Some(w_comp), Some(s_writ)) = 
                    (reads_completed, sectors_read, writes_completed, sectors_written) 
                {
                    // Assign properties into preallocated positions using simple heap allocations
                    snapshot.names[snapshot.count] = device_name.to_string();
                    snapshot.metrics[snapshot.count] = DiskMetric {
                        reads_completed: r_comp,
                        sectors_read: s_read,
                        writes_completed: w_comp,
                        sectors_written: s_writ,
                    };
                    
                    snapshot.count += 1;
                }
            }
        }

        snapshot
    }
}