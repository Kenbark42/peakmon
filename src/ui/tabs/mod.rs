#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tab {
    Dashboard,
    Cpu,
    Gpu,
    Ai,
    Memory,
    Disk,
    Network,
    Processes,
    Logs,
    Temperatures,
}

impl Tab {
    pub const ALL: [Tab; 10] = [
        Tab::Dashboard,
        Tab::Cpu,
        Tab::Gpu,
        Tab::Ai,
        Tab::Memory,
        Tab::Disk,
        Tab::Network,
        Tab::Processes,
        Tab::Logs,
        Tab::Temperatures,
    ];

    pub fn label(&self) -> &str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Cpu => "CPU",
            Tab::Gpu => "GPU",
            Tab::Ai => "AI",
            Tab::Memory => "Memory",
            Tab::Disk => "Disk",
            Tab::Network => "Network",
            Tab::Processes => "Processes",
            Tab::Logs => "Logs",
            Tab::Temperatures => "Temps",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Dashboard => 0,
            Tab::Cpu => 1,
            Tab::Gpu => 2,
            Tab::Ai => 3,
            Tab::Memory => 4,
            Tab::Disk => 5,
            Tab::Network => 6,
            Tab::Processes => 7,
            Tab::Logs => 8,
            Tab::Temperatures => 9,
        }
    }

    pub fn from_index(i: usize) -> Option<Tab> {
        Tab::ALL.get(i).copied()
    }

    pub fn next(&self) -> Tab {
        let idx = (self.index() + 1) % Tab::ALL.len();
        Tab::ALL[idx]
    }

    pub fn prev(&self) -> Tab {
        let idx = if self.index() == 0 {
            Tab::ALL.len() - 1
        } else {
            self.index() - 1
        };
        Tab::ALL[idx]
    }
}

pub mod ai_detail;
pub mod cpu_detail;
pub mod dashboard;
pub mod disk_detail;
pub mod gpu_detail;
pub mod logs;
pub mod memory_detail;
pub mod network_detail;
pub mod processes;
pub mod temperatures;
