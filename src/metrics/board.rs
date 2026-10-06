use std::fs;


#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub description: String,
    pub product: String,
    pub vendor: String,
    pub version: String,
    pub serial: String,
    pub width: String,
    pub capabilities: String,
    pub configuration: String,
}


#[derive(Debug, Clone, Default)]
pub struct MotheboardInfo {
    pub description: String,
    pub product: String,
    pub vendor: String,
    pub physical_id: u64,
    pub version: String,
    pub serial: String,
    pub slot: String,
}


#[derive(Debug, Clone, Default)]
pub struct FirmwareInfo {
    pub description: String,
    pub vendor: String,
    pub version: String,
    pub date: String,
    pub capabilities: String,
}


#[derive(Debug, Clone, Default)]
pub struct BoardSnapshot {
    pub sys: SystemInfo,
    pub mb: MotheboardInfo,
    pub fw: FirmwareInfo,
}


pub struct BoardCollector;


impl BoardCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn map_chassis_type(raw_val: &str) -> String {
        // Parse the trimmed text string into an integer safely
        match raw_val.parse::<u32>() {
            Ok(3) => String::from("Desktop"),
            Ok(6) | Ok(7) => String::from("Tower"),
            Ok(9) | Ok(10) => String::from("Notebook"),
            Ok(17) | Ok(23) => String::from("Server"),
            Ok(31) | Ok(32) => String::from("Convertible/Tablet"),
            _ => format!("Unknown Chassis ({})", raw_val),
        }
    }

    pub fn collect(&self) -> BoardSnapshot {
        let mut snapshot = BoardSnapshot::default();

        // Helper closure to read, trim, and handle errors cleanly
        let read_sys_file = |path: &str| -> String {
            fs::read_to_string(path)
                .map(|s| s.trim().to_string()) // Trim trailing newlines and convert to String
                .unwrap_or_else(|_| String::from("Unknown")) // Fallback if file read fails
        };

        // Safely populate the System block
        let raw_chassis = read_sys_file("/sys/class/dmi/id/chassis_type");
        snapshot.sys.description = Self::map_chassis_type(&raw_chassis);
        snapshot.sys.vendor = read_sys_file("/sys/class/dmi/id/sys_vendor");
        snapshot.sys.product = read_sys_file("/sys/class/dmi/id/product_name");
        snapshot.sys.version = read_sys_file("/sys/class/dmi/id/product_version");
        snapshot.sys.serial = read_sys_file("/sys/class/dmi/id/product_serial");

        // Dynamic detection of architecture compilation targets
        snapshot.sys.width = if cfg!(target_pointer_width = "64") {
            String::from("64 bits")
        } else {
            String::from("32 bits")
        };

        // Safely populate the Motherboard
        snapshot.mb.description = String::from("Motherboard");
        snapshot.mb.vendor = read_sys_file("/sys/class/dmi/id/board_vendor");
        snapshot.mb.product = read_sys_file("/sys/class/dmi/id/board_name");
        snapshot.mb.version = read_sys_file("/sys/class/dmi/id/board_version");
        snapshot.mb.serial = read_sys_file("/sys/class/dmi/id/board_serial");
        snapshot.mb.slot = read_sys_file("/sys/class/dmi/id/board_asset_tag");


        // Safely populate the Firmware
        snapshot.fw.description = String::from("BIOS");
        snapshot.fw.vendor = read_sys_file("/sys/class/dmi/id/bios_vendor");
        snapshot.fw.version = read_sys_file("/sys/class/dmi/id/bios_version");
        snapshot.fw.date = read_sys_file("/sys/class/dmi/id/bios_date");


        snapshot
    }
}