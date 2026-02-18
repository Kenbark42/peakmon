use sysinfo::System;

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory: u64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProcessSortField {
    Pid,
    Name,
    Cpu,
    Memory,
}

pub struct ProcessMetrics {
    pub processes: Vec<ProcessInfo>,
    pub sort_field: ProcessSortField,
    pub sort_ascending: bool,
    pub filter: String,
}

impl ProcessMetrics {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            sort_field: ProcessSortField::Cpu,
            sort_ascending: false,
            filter: String::new(),
        }
    }

    pub fn update(&mut self, sys: &System) {
        self.processes = sys
            .processes()
            .iter()
            .map(|(&pid, proc_info)| {
                let pid_val: usize = pid.into();
                ProcessInfo {
                    pid: pid_val as u32,
                    name: proc_info.name().to_string_lossy().to_string(),
                    cpu_usage: proc_info.cpu_usage() as f64,
                    memory: proc_info.memory(),
                }
            })
            .collect();

        self.sort();
    }

    pub fn sort(&mut self) {
        let ascending = self.sort_ascending;
        match self.sort_field {
            ProcessSortField::Pid => {
                self.processes.sort_by(|a, b| {
                    if ascending { a.pid.cmp(&b.pid) } else { b.pid.cmp(&a.pid) }
                });
            }
            ProcessSortField::Name => {
                self.processes.sort_by(|a, b| {
                    if ascending {
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    } else {
                        b.name.to_lowercase().cmp(&a.name.to_lowercase())
                    }
                });
            }
            ProcessSortField::Cpu => {
                self.processes.sort_by(|a, b| {
                    if ascending {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            ProcessSortField::Memory => {
                self.processes.sort_by(|a, b| {
                    if ascending { a.memory.cmp(&b.memory) } else { b.memory.cmp(&a.memory) }
                });
            }
        }
    }

    pub fn set_sort_field(&mut self, field: ProcessSortField) {
        if self.sort_field == field {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_field = field;
            self.sort_ascending = false;
        }
        self.sort();
    }

    pub fn filtered_processes(&self) -> Vec<&ProcessInfo> {
        if self.filter.is_empty() {
            self.processes.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.processes
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }
}
