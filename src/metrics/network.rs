use std::fs;
use std::array;


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


/// Combined telemetry data and identity layout for a single interface.
#[derive(Debug, Clone, Default)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub metrics: InterfaceMetric,
}


#[derive(Debug)]
pub struct NetworkSnapshot<const N: usize = 16> {
    pub items: [NetworkInterface; N],
    pub count: usize,
}


impl<const N: usize> Default for NetworkSnapshot<N> {
    fn default() -> Self {
        Self {
            items: array::from_fn(|_| NetworkInterface::default()),
            count: 0,
        }
    }
}


impl<const N: usize> NetworkSnapshot<N> {
    /// Returns a slice containing only the actively populated interfaces
    pub fn interfaces(&self) -> &[NetworkInterface] {
        &self.items[..self.count]
    }

    /// Exposes a clean, direct iterator over the active interface items.
    pub fn iter(&self) -> impl Iterator<Item = &NetworkInterface> {
        self.items[..self.count].iter()
    }

    /// Returns a lazy iterator yielding only the names of active interfaces
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.items[..self.count].iter().map(|item| &item.name)
    }

    /// Returns a lazy iterator yielding only the MAC addresses of active interfaces
    pub fn macs(&self) -> impl Iterator<Item = &String> {
        self.items[..self.count].iter().map(|item| &item.mac)
    }
}


pub struct NetworkCollector;


impl NetworkCollector {

    pub fn new() -> Self {
        Self
    }

    pub fn collect<const N: usize>(&self) -> NetworkSnapshot<N> {
        let mut snapshot = NetworkSnapshot::<N>::default();

        if let Ok(interfaces) = fs::read_to_string("/proc/net/dev") {
            // Skip the first 2 lines containing system headers
            for interface in interfaces.lines().skip(2) {

                // Safety boundary check to prevent out-of-bounds stack arrays panic
                if snapshot.count >= N {
                    break;
                }

                // Split into: interface name (before ':') and metrics (after ':')
                let mut parts = interface.splitn(2, ':');
                let interface_name = parts.next().map(|s| s.trim());
                let metrics_part = parts.next().map(|s| s.trim());

                if let (Some(name), Some(metrics_str)) = (interface_name, metrics_part) {
                    if !name.is_empty() {
                        let mut value = metrics_str.split_whitespace();
                        let mut interface_item = NetworkInterface::default();

                        interface_item.name = name.to_string();

                        // Resolve MAC address natively from sysfs
                        let mac_path = format!("/sys/class/net/{}/address", name);
                        interface_item.mac = fs::read_to_string(mac_path)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| String::from("00:00:00:00:00:00"));

                        // Parse performance counters safely using combinators
                        let mut metric = InterfaceMetric::default();
                        metric.rx_bytes   = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.rx_packets = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.rx_errors  = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);

                        // Skip the 5 unnecessary internal intermediate fields
                        let _ = value.nth(4);

                        metric.tx_bytes   = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.tx_packets = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                        metric.tx_errors  = value.next().and_then(|v| v.parse().ok()).unwrap_or(0);

                        interface_item.metrics = metric;

                        // Securely commit to stack array
                        snapshot.items[snapshot.count] = interface_item;
                        snapshot.count += 1;
                    }
                }

            }
        }

        snapshot
    }
}