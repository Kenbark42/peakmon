use super::history::History;
use sysinfo::System;

pub struct MemoryMetrics {
    pub total_ram: u64,
    pub used_ram: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub ram_percent: f64,
    pub swap_percent: f64,
    pub ram_history: History,
    pub swap_history: History,
}

impl MemoryMetrics {
    pub fn new() -> Self {
        Self {
            total_ram: 0,
            used_ram: 0,
            total_swap: 0,
            used_swap: 0,
            ram_percent: 0.0,
            swap_percent: 0.0,
            ram_history: History::new(),
            swap_history: History::new(),
        }
    }

    pub fn update(&mut self, sys: &System) {
        self.total_ram = sys.total_memory();
        self.used_ram = sys.used_memory();
        self.total_swap = sys.total_swap();
        self.used_swap = sys.used_swap();

        self.ram_percent = if self.total_ram > 0 {
            (self.used_ram as f64 / self.total_ram as f64) * 100.0
        } else {
            0.0
        };

        self.swap_percent = if self.total_swap > 0 {
            (self.used_swap as f64 / self.total_swap as f64) * 100.0
        } else {
            0.0
        };

        self.ram_history.push(self.ram_percent);
        self.swap_history.push(self.swap_percent);
    }
}
