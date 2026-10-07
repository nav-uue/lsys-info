mod metrics;

// Export the block structures publicly
pub use metrics::memory::{MemorySnapshot, RamSpaceInfo, SwapSpaceInfo};
pub use metrics::cpu::CpuSnapshot;
pub use metrics::network::NetworkSnapshot;
pub use metrics::disk::DiskSnapshot;
pub use metrics::board::{BoardSnapshot, MotheboardInfo, SystemInfo, FirmwareInfo};
pub use metrics::os::OsSnapshot;


pub struct LsysInfo {
    memory_collector: metrics::memory::MemoryCollector,
    cpu_collector: metrics::cpu::CpuCollector,
    network_collector: metrics::network::NetworkCollector,
    disk_collector: metrics::disk::DiskCollector,
    board_collector: metrics::board::BoardCollector,
    os_collector: metrics::os::OsCollector,
}


impl LsysInfo {
    pub fn new() -> Self {
        Self {
            memory_collector: metrics::memory::MemoryCollector::new(),
            cpu_collector: metrics::cpu::CpuCollector::new(),
            network_collector: metrics::network::NetworkCollector::new(),
            disk_collector: metrics::disk::DiskCollector::new(),
            board_collector: metrics::board::BoardCollector::new(),
            os_collector: metrics::os::OsCollector::new(),
        }
    }

    // Returns the grouped memory blocks
    pub fn memory(&mut self) -> MemorySnapshot {
        self.memory_collector.collect()
    }

    pub fn cpu(&mut self) -> CpuSnapshot {
        self.cpu_collector.collect()
    }

    pub fn network(&mut self) -> NetworkSnapshot {
        self.network_collector.collect()
    }

    pub fn disk(&mut self) -> DiskSnapshot {
        self.disk_collector.collect()
    }

    pub fn board(&mut self) -> BoardSnapshot {
        self.board_collector.collect()
    }

    pub fn os(&mut self) -> OsSnapshot {
        self.os_collector.collect()
    }

}
