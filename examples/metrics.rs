use lsys_info::LsysInfo;


fn main() {
    let mut sys = LsysInfo::new();

    let mem_snapshot = sys.memory();
    let cpu_snapshot = sys.cpu();
    let network_snapshot = sys.network();
    let disk_snapshot = sys.disk();
    let board_snapshot = sys.board();

    println!("==================================================================");
    println!("                      SYSTEM MONITOR REPORT                       ");
    println!("==================================================================");

    // ---------------- Memory ---------------- //
    println!("\n🔹 [MEMORY METRICS]");
    println!("------------------------------------------------------------------");
    println!("↳ RAM");
    println!("  ├─ total_kb         : {} KB", mem_snapshot.ram.total_kb);
    println!("  └─ available_kb     : {} KB", mem_snapshot.ram.available_kb);
    println!("↳ SWAP");
    println!("  ├─ total_kb         : {} KB", mem_snapshot.swap.total_kb);
    println!("  └─ free_kb          : {} KB", mem_snapshot.swap.free_kb);
    println!("\n------------------------------------------------------------------");

    // ----------------- CPU info ----------------- //
    println!("\n🔹 [CPU METRICS]");
    println!("------------------------------------------------------------------");
    println!("↳ Hardware Information");
    println!("  ├─ vendor           : {}", cpu_snapshot.main.vendor);
    println!("  ├─ product          : {}", cpu_snapshot.main.product);
    match &cpu_snapshot.main.serial {
        Some(serial) => println!("  ├─ serial           : {}", serial),
        None         => println!("  ├─ serial           : None"),
    }
    println!("  ├─ slot             : {}", cpu_snapshot.main.slot);
    println!("  ├─ version          : {}", cpu_snapshot.main.version);
    println!("  ├─ bit_width        : {} bits", cpu_snapshot.main.bit_width);
    println!("  ├─ bus_clock_mhz    : {} MHz", cpu_snapshot.main.bus_clock_mhz);
    println!("  ├─ current_speed_mhz: {} MHz", cpu_snapshot.main.current_speed_mhz);
    println!("  ├─ max_speed_mhz    : {} MHz", cpu_snapshot.main.max_speed_mhz);
    println!("  ├─ physical_cores   : {}", cpu_snapshot.main.physical_cores);
    println!("  └─ logical_cores    : {}", cpu_snapshot.main.logical_cores);
    
    println!("↳ Load Stats");
    println!("  └─ system_total     : {:?}", cpu_snapshot.system_total);

    println!("↳ Per-Core Load State");
    let cores = cpu_snapshot.cores();
    for (i, core) in cores.iter().enumerate() {
        let prefix = if i == cores.len() - 1 { "  └─" } else { "  ├─" };
        println!("{} core_{:<2}        : idle={}", prefix, i, core.idle);
    }
    println!("\n------------------------------------------------------------------");

    // ------------------- Network ------------------- //
    println!("\n🔹 [NETWORK METRICS]");
    println!("------------------------------------------------------------------");
    if network_snapshot.interfaces().is_empty() {
        println!("  (No active network interfaces detected)");
    }
    for (name, metrics) in network_snapshot.iter() {
        println!("↳ Interface: {}", name);
        println!("  ├─ rx_bytes         : {}", metrics.rx_bytes);
        println!("  ├─ tx_bytes         : {}", metrics.tx_bytes);
        println!("  ├─ rx_packets       : {}", metrics.rx_packets);
        println!("  ├─ tx_packets       : {}", metrics.tx_packets);
        println!("  ├─ rx_errors        : {}", metrics.rx_errors);
        println!("  └─ tx_errors        : {}", metrics.tx_errors);
        println!();
    }
    println!("------------------------------------------------------------------");

    // --------------------- Disk --------------------- //
    println!("\n🔹 [DISK METRICS]");
    println!("------------------------------------------------------------------");
    if disk_snapshot.metrics().is_empty() {
        println!("  (No active block devices detected)");
    }
    for (name, metrics) in disk_snapshot.iter() {
        println!("↳ Device: {}", name);
        println!("  ├─ reads_completed  : {}", metrics.reads_completed);
        println!("  ├─ sectors_read     : {}", metrics.sectors_read);
        println!("  ├─ writes_completed : {}", metrics.writes_completed);
        println!("  └─ sectors_written  : {}", metrics.sectors_written);
        println!();
    }
    println!("------------------------------------------------------------------");

    // -------------------- Board -------------------- //
    println!("\n🔹 [BOARD & SYSTEM METRICS]");
    println!("------------------------------------------------------------------");

    // Render the Primary System block
    println!("↳ System metrics");
    println!("  ├─ description      : {}", board_snapshot.sys.description);
    println!("  ├─ product          : {}", board_snapshot.sys.product);
    println!("  ├─ vendor           : {}", board_snapshot.sys.vendor);
    println!("  ├─ version          : {}", board_snapshot.sys.version);
    println!("  ├─ serial           : {}", board_snapshot.sys.serial);
    println!("  ├─ width            : {}", board_snapshot.sys.width);
    println!("  ├─ capabilities     : {}", board_snapshot.sys.capabilities);
    println!("  └─ configuration    : {}", board_snapshot.sys.configuration);
    println!();

    // Render the Motherboard block
    println!("↳ Motherboard metrics");
    println!("  ├─ description      : {}", board_snapshot.mb.description);
    println!("  ├─ product          : {}", board_snapshot.mb.product);
    println!("  ├─ vendor           : {}", board_snapshot.mb.vendor);
    println!("  ├─ physical_id      : {}", board_snapshot.mb.physical_id);
    println!("  ├─ version          : {}", board_snapshot.mb.version);
    println!("  ├─ serial           : {}", board_snapshot.mb.serial);
    println!("  └─ slot             : {}", board_snapshot.mb.slot);
    println!();

    // Render the Firmware block
    println!("↳ Firmware metrics");
    println!("  ├─ description      : {}", board_snapshot.fw.description);
    println!("  ├─ vendor           : {}", board_snapshot.fw.vendor);
    println!("  ├─ version          : {}", board_snapshot.fw.version);
    println!("  ├─ date             : {}", board_snapshot.fw.date);
    println!("  └─ capabilities     : {}", board_snapshot.fw.capabilities);

    println!("\n==================================================================\n");
}