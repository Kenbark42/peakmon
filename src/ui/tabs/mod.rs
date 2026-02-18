#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tab {
    Dashboard,
    Cpu,
    Memory,
    Disk,
    Network,
    Processes,
    Logs,
    Temperatures,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::Dashboard,
        Tab::Cpu,
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
            Tab::Memory => 2,
            Tab::Disk => 3,
            Tab::Network => 4,
            Tab::Processes => 5,
            Tab::Logs => 6,
            Tab::Temperatures => 7,
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

pub mod dashboard;
pub mod cpu_detail;
pub mod memory_detail;
pub mod disk_detail;
pub mod network_detail;
pub mod processes;
pub mod logs;
pub mod temperatures;
