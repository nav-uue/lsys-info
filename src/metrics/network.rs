use std::fs;
use std::array;


pub const MAX_INTERFACES: usize = 16;


/// Network traffic and error statistics for a single network interface.
#[derive(Debug, Clone, Default)]
pub struct InterfaceMetric {
    /// Total bytes received by the interface
    pub rx_bytes: u64,
    /// Total bytes transmitted by the interface
    pub tx_bytes: u64,
    /// Total packets received by the interface
    pub rx_packets: u64,
    /// Total packets transmitted by the interface
    pub tx_packets: u64,
    /// Total receive errors encountered
    pub rx_errors: u64,
    /// Total transmit errors encountered
    pub tx_errors: u64,
}


#[derive(Debug)]
pub struct NetworkSnapshot {
    pub interfaces: [InterfaceMetric; MAX_INTERFACES],
    pub interfaces_names: [String; MAX_INTERFACES],
    pub count: usize,
}


impl Default for NetworkSnapshot {
    fn default() -> Self {
        Self {
            interfaces: array::from_fn(|_| InterfaceMetric::default()),
            interfaces_names: array::from_fn(|_| String::new()),
            count: 0,
        }
    }
}


impl NetworkSnapshot {
    /// Returns a slice containing only the actively populated interface metrics
    pub fn interfaces(&self) -> &[InterfaceMetric] {
        &self.interfaces[..self.count]
    }

    /// Returns a slice containing only the actively populated interface names
    pub fn names(&self) -> &[String] {
        &self.interfaces_names[..self.count]
    }

    /// Returns an iterator that yields a tuple of (&String, &InterfaceMetric) for each active interface
    pub fn iter(&self) -> impl Iterator<Item = (&String, &InterfaceMetric)> {
        self.interfaces_names[..self.count]
            .iter()
            .zip(self.interfaces[..self.count].iter())
    }
}


pub struct NetworkCollector;


impl NetworkCollector {

    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> NetworkSnapshot {
        let mut snapshot = NetworkSnapshot::default();

        if let Ok(interfaces) = fs::read_to_string("/proc/net/dev") {
            for interface in interfaces.lines().skip(2) {

                // Break early if we have filled the stack-allocated array capacity
                if snapshot.count >= MAX_INTERFACES {
                    break;
                }

                // Split the line into exactly 2 parts: before and after the colon
                let mut parts = interface.splitn(2, ':');

                // Take the first part (before ':') and clean up whitespace
                let interface_name = parts.next().map(|s| s.trim());

                // Take the second part (after ':') containing all the numeric tokens
                let metrics_part = parts.next().map(|s| s.trim());

                if let (Some(name), Some(metrics_str)) = (interface_name, metrics_part) {
                    if !name.is_empty() {
                        let mut value = metrics_str.split_whitespace();

                        let mut metric = InterfaceMetric::default();

                        // Parse Receive (RX) metrics (columns 0, 1, 2)
                        metric.rx_bytes   = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.rx_packets = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.rx_errors  = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);

                        // Skip the next 5 unnecessary RX columns (drop, fifo, frame, compressed, multicast)
                        // .nth(4) skips 4 elements and returns the 5th one, effectively advancing the iterator
                        let _ = value.nth(4);

                        // Parse Transmit (TX) metrics (columns 8, 9, 10)
                        metric.tx_bytes   = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.tx_packets = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.tx_errors  = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);

                        snapshot.interfaces_names[snapshot.count] = name.to_string();
                        snapshot.interfaces[snapshot.count] = metric;

                        // Increment the counter to track active entries and point to the next free cell
                        snapshot.count += 1;
                    }
                }

            }
        }

        snapshot
    }
}