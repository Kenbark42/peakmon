use super::history::History;
use sysinfo::System;

pub struct CpuMetrics {
    pub aggregate_usage: f64,
    pub per_core_usage: Vec<f64>,
    pub aggregate_history: History,
    pub per_core_history: Vec<History>,
    pub core_count: usize,
    pub load_avg: [f64; 3],
}

impl CpuMetrics {
    pub fn new(core_count: usize) -> Self {
        Self {
            aggregate_usage: 0.0,
            per_core_usage: vec![0.0; core_count],
            aggregate_history: History::new(),
            per_core_history: (0..core_count).map(|_| History::new()).collect(),
            core_count,
            load_avg: [0.0; 3],
        }
    }

    pub fn update(&mut self, sys: &System) {
        let cpus = sys.cpus();
        self.core_count = cpus.len();

        // Per-core usage
        self.per_core_usage.clear();
        for cpu in cpus {
            self.per_core_usage.push(cpu.cpu_usage() as f64);
        }

        // Ensure history vectors match core count
        while self.per_core_history.len() < self.core_count {
            self.per_core_history.push(History::new());
        }

        for (i, &usage) in self.per_core_usage.iter().enumerate() {
            if i < self.per_core_history.len() {
                self.per_core_history[i].push(usage);
            }
        }

        // Aggregate
        self.aggregate_usage = if self.per_core_usage.is_empty() {
            0.0
        } else {
            self.per_core_usage.iter().sum::<f64>() / self.per_core_usage.len() as f64
        };
        self.aggregate_history.push(self.aggregate_usage);

        // Load average
        let mut loadavg = [0.0_f64; 3];
        unsafe {
            libc::getloadavg(loadavg.as_mut_ptr(), 3);
        }
        self.load_avg = loadavg;
    }
}
