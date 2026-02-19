pub mod ai;
pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod history;
pub mod memory;
pub mod network;
pub mod process;
pub mod temperature;

use ai::AiMetrics;
use battery::BatteryMetrics;
use cpu::CpuMetrics;
use disk::DiskMetrics;
use gpu::GpuMetrics;
use memory::MemoryMetrics;
use network::NetworkMetrics;
use process::ProcessMetrics;
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};
use temperature::TemperatureMetrics;

use crate::ui::tabs::Tab;

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
    pub gpu: GpuMetrics,
    pub ai: AiMetrics,
    pub battery: BatteryMetrics,
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
            gpu: GpuMetrics::new(),
            ai: AiMetrics::new(),
            battery: BatteryMetrics::new(),
            boot_time,
        }
    }

    pub fn refresh(&mut self, active_tab: Tab) {
        // Always refresh CPU and memory (cheap)
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.cpu.update(&self.sys);
        self.memory.update(&self.sys);

        // Only refresh expensive subsystems when their tab is visible
        let needs_processes = matches!(active_tab, Tab::Dashboard | Tab::Processes | Tab::Ai);
        let needs_disk = matches!(active_tab, Tab::Dashboard | Tab::Disk);
        let needs_network = matches!(active_tab, Tab::Dashboard | Tab::Network);
        let needs_temps = matches!(active_tab, Tab::Temperatures);
        let needs_gpu = matches!(active_tab, Tab::Dashboard | Tab::Gpu | Tab::Ai);
        let needs_ai = matches!(active_tab, Tab::Ai);
        let needs_battery = matches!(active_tab, Tab::Dashboard);

        if needs_processes {
            self.sys.refresh_processes(ProcessesToUpdate::All, true);
            self.processes.update(&self.sys);
        }

        if needs_disk {
            self.disks.refresh(true);
            self.disk.update(&self.disks);
        }

        if needs_network {
            self.networks.refresh(true);
            self.network.update(&self.networks);
        }

        if needs_temps {
            self.components.refresh(true);
            self.temperature.update(&self.components);
        }

        if needs_gpu {
            self.gpu.update();
        }

        if needs_ai {
            self.ai.update(&self.processes.processes);
        }

        if needs_battery {
            self.battery.update();
        }
    }

    pub fn uptime(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.boot_time)
    }
}
