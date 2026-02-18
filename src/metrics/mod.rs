pub mod cpu;
pub mod disk;
pub mod history;
pub mod memory;
pub mod network;
pub mod process;
pub mod temperature;

use cpu::CpuMetrics;
use disk::DiskMetrics;
use memory::MemoryMetrics;
use network::NetworkMetrics;
use process::ProcessMetrics;
use temperature::TemperatureMetrics;
use sysinfo::{Components, Disks, Networks, System};

pub struct MetricsCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub processes: ProcessMetrics,
    pub temperature: TemperatureMetrics,
    pub boot_time: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let core_count = sys.cpus().len();
        let boot_time = System::boot_time();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        Self {
            sys,
            disks,
            networks,
            components,
            cpu: CpuMetrics::new(core_count),
            memory: MemoryMetrics::new(),
            disk: DiskMetrics::new(),
            network: NetworkMetrics::new(),
            processes: ProcessMetrics::new(),
            temperature: TemperatureMetrics::new(),
            boot_time,
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);

        self.cpu.update(&self.sys);
        self.memory.update(&self.sys);
        self.disk.update(&self.disks);
        self.network.update(&self.networks);
        self.processes.update(&self.sys);
        self.temperature.update(&self.components);
    }

    pub fn uptime(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.boot_time)
    }
}
