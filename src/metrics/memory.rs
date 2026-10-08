use std::fs;


/// Stores the system's physical memory layout and motherboard capacity metrics.
#[derive(Debug, Clone, Default)]
pub struct RamSystemInfo {
    pub location: u8,
    pub use_case: u8,
    pub max_capacity_kb: u64,
    pub total_slots: u16,
    pub installed_capacity_kb: u64,
    pub populated_slots: u16,
    pub slots: Vec<SlotDetail>,
}


/// Provides human-readable string translations for raw SMBIOS memory fields.
impl RamSystemInfo {
    pub fn location_str(&self) -> &'static str {
        match self.location {
            0x03 => "System board or motherboard",
            0x04 => "ISA add-on card",
            0x06 => "PCI add-on card",
            _ => "Other / Integrated Layout",
        }
    }

    pub fn use_case_str(&self) -> &'static str {
        match self.use_case {
            0x03 => "System Memory",
            0x04 => "Video Memory",
            _ => "General Storage Memory",
        }
    }
}


#[derive(Debug, Default, Clone)]
pub struct SlotDetail {
    pub slot_label: String,
    pub vendor: String,
    pub serial: String,
    pub product: String,
    pub size_kb: u64,
    pub speed_mhz: u16,
    pub width_bits: u16,
}


/// Holds runtime RAM metrics parsed from /proc/meminfo
#[derive(Debug, Clone, Copy, Default)]
pub struct RamSpaceInfo {
    pub total_kb: u64,
    pub free_kb: u64,
    pub available_kb: u64
}


/// Holds runtime Swap metrics parsed from /proc/meminfo
#[derive(Debug, Clone, Copy, Default)]
pub struct SwapSpaceInfo {
    pub total_kb: u64,
    pub free_kb: u64
}


/// A complete point-in-time snapshot of the system memory state
#[derive(Debug, Clone, Default)]
pub struct MemorySnapshot {
    pub ram_sys: RamSystemInfo, 
    pub ram_space: RamSpaceInfo,
    pub swap_space: SwapSpaceInfo
}


/// Stateless collector responsible for gathering system memory metrics
pub struct MemoryCollector;


impl MemoryCollector {
    /// Initializes a new memory collector instance
    pub fn new() -> Self {
        Self
    }

    /// Highly efficient, zero-allocation extraction helper targeting the null-separated text string pool.
    fn extract_smbios_string(raw_strings: &[u8], index: usize) -> String {
        if index == 0 {
            return "Unknown".to_string();
        }
        let mut current_idx = 1;
        let mut start = 0;

        for (i, &b) in raw_strings.iter().enumerate() {
            if b == 0 {
                if current_idx == index {
                    return String::from_utf8_lossy(&raw_strings[start..i]).trim().to_string();
                }
                start = i + 1;
                current_idx += 1;
            }
        }
        "Unknown".to_string()
    }

    fn parse_system_memory_info(data: &[u8]) -> RamSystemInfo {
        let mut info = RamSystemInfo::default();
        let mut cursor = 0;

        while cursor < data.len() {
            if cursor + 4 > data.len() { break; }

            let struct_type = data[cursor];
            let length = data[cursor + 1] as usize;

            if cursor + length > data.len() { break; }

            // Extract the string boundary locations right now before advancing the cursor
            let string_section_start = cursor + length;
            let mut string_section_end = string_section_start;
            
            while string_section_end + 1 < data.len() {
                if data[string_section_end] == 0 && data[string_section_end + 1] == 0 {
                    string_section_end += 2;
                    break;
                }
                string_section_end += 1;
            }
            let raw_strings = &data[string_section_start..string_section_end];

            match struct_type {
                // --- Type 16: Physical Memory Array ---
                16 => {
                    if length >= 0x0F {
                        info.location = data[cursor + 4];
                        info.use_case = data[cursor + 5];
                        
                        let mut max_kb = u32::from_le_bytes([
                            data[cursor + 7],  data[cursor + 8], 
                            data[cursor + 9],  data[cursor + 10]
                        ]) as u64;

                        // Handle 64-bit extended capacity for modern large motherboards
                        if max_kb == 0x80000000 && length >= 0x17 {
                            max_kb = u64::from_le_bytes([
                                data[cursor + 0x0F], data[cursor + 0x10], data[cursor + 0x11], data[cursor + 0x12],
                                data[cursor + 0x13], data[cursor + 0x14], data[cursor + 0x15], data[cursor + 0x16],
                            ]);
                        }
                        info.max_capacity_kb = max_kb;
                        info.total_slots = u16::from_le_bytes([data[cursor + 0x0D], data[cursor + 0x0E]]);
                    }
                }
                // --- Type 17: Memory Device (Individual Slots) ---
                17 => {
                    if length >= 0x15 {
                        let raw_size = u16::from_le_bytes([data[cursor + 0x0C], data[cursor + 0x0D]]);
                        
                        // 0xFFFF means the slot is completely empty
                        if raw_size != 0xFFFF && raw_size != 0x0000 {
                            info.populated_slots += 1;
                            
                            let mut slot_size_kb: u64 = 0;

                            if raw_size == 0x7FFF && length >= 0x20 {
                                // Value is 32 GiB or greater, read the extended 4-byte field (in MB)
                                let ext_size_mb = u32::from_le_bytes([
                                    data[cursor + 0x1C], data[cursor + 0x1D], 
                                    data[cursor + 0x1E], data[cursor + 0x1F]
                                ]) as u64;
                                slot_size_kb = ext_size_mb * 1024;
                            } else {
                                let is_kb = (raw_size & 0x8000) != 0;
                                let val = (raw_size & 0x7FFF) as u64;
                                if is_kb {
                                    slot_size_kb = val;
                                } else {
                                    slot_size_kb = val * 1024; // Convert MB to KB
                                }
                            }
                            info.installed_capacity_kb += slot_size_kb;

                            // EXTRACT EXTRA SLOT METADATA WITH ZERO COPIES
                            let mut slot = SlotDetail::default();
                            slot.size_kb = slot_size_kb;

                            // Extract String Table Indices from Type 17 offsets
                            let device_locator_idx = data[cursor + 0x10] as usize;
                            let manufacturer_idx   = data[cursor + 0x17] as usize;
                            let serial_number_idx  = data[cursor + 0x18] as usize;
                            let part_number_idx    = data[cursor + 0x1A] as usize;

                            slot.slot_label = Self::extract_smbios_string(raw_strings, device_locator_idx);
                            slot.vendor     = Self::extract_smbios_string(raw_strings, manufacturer_idx);
                            slot.serial     = Self::extract_smbios_string(raw_strings, serial_number_idx);
                            slot.product    = Self::extract_smbios_string(raw_strings, part_number_idx);

                            // Extract Data Width (Offset 0x0A, 2 bytes)
                            slot.width_bits = u16::from_le_bytes([data[cursor + 0x0A], data[cursor + 0x0B]]);

                            // Extract Configured Clock Speed (Offset 0x22, 2 bytes)
                            if length >= 0x24 {
                                slot.speed_mhz = u16::from_le_bytes([data[cursor + 0x22], data[cursor + 0x23]]);
                            }

                            info.slots.push(slot);
                        }
                    }
                }
                _ => {}
            }

            // Directly jump to the end of the text strings region we scanned earlier
            cursor = string_section_end;
        }
        info
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
                    (Some("MemTotal:"), Some(v)) => snapshot.ram_space.total_kb = v,
                    (Some("MemFree:"), Some(v)) => snapshot.ram_space.free_kb = v,
                    (Some("MemAvailable:"), Some(v)) => snapshot.ram_space.available_kb = v,
                    
                    // Populate swap space metrics
                    (Some("SwapTotal:"), Some(v)) => snapshot.swap_space.total_kb = v,
                    (Some("SwapFree:"), Some(v)) => snapshot.swap_space.free_kb = v,
                    
                    _ => {}
                }
            }
        }

        if let Ok(raw_table) = fs::read("/sys/firmware/dmi/tables/DMI") {
            let info = Self::parse_system_memory_info(&raw_table);

            snapshot.ram_sys = info;
        }

        snapshot
    }
}