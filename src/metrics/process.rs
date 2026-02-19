use std::collections::HashMap;
use sysinfo::{ProcessStatus, System};

use crate::util::contains_ignore_ascii_case;

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub cpu_usage: f64,
    pub memory: u64,
    pub status: ProcessState,
    pub depth: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProcessState {
    Run,
    Sleep,
    Idle,
    Zombie,
    Stop,
    Unknown,
}

impl ProcessState {
    pub fn from_sysinfo(status: ProcessStatus) -> Self {
        match status {
            ProcessStatus::Run => ProcessState::Run,
            ProcessStatus::Sleep => ProcessState::Sleep,
            ProcessStatus::Idle => ProcessState::Idle,
            ProcessStatus::Zombie => ProcessState::Zombie,
            ProcessStatus::Stop => ProcessState::Stop,
            _ => ProcessState::Unknown,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ProcessState::Run => "Run",
            ProcessState::Sleep => "Sleep",
            ProcessState::Idle => "Idle",
            ProcessState::Zombie => "Zombie",
            ProcessState::Stop => "Stop",
            ProcessState::Unknown => "?",
        }
    }
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
    pub tree_mode: bool,
}

impl ProcessMetrics {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            sort_field: ProcessSortField::Cpu,
            sort_ascending: false,
            filter: String::new(),
            tree_mode: false,
        }
    }

    pub fn update(&mut self, sys: &System) {
        self.processes = sys
            .processes()
            .iter()
            .map(|(&pid, proc_info)| {
                let pid_val: usize = pid.into();
                let ppid = proc_info.parent().map(|p| {
                    let v: usize = p.into();
                    v as u32
                });
                ProcessInfo {
                    pid: pid_val as u32,
                    parent_pid: ppid,
                    name: proc_info.name().to_string_lossy().to_string(),
                    cpu_usage: proc_info.cpu_usage() as f64,
                    memory: proc_info.memory(),
                    status: ProcessState::from_sysinfo(proc_info.status()),
                    depth: 0,
                }
            })
            .collect();

        self.sort();
    }

    pub fn sort(&mut self) {
        let ascending = self.sort_ascending;
        match self.sort_field {
            ProcessSortField::Pid => {
                self.processes.sort_unstable_by(|a, b| {
                    if ascending {
                        a.pid.cmp(&b.pid)
                    } else {
                        b.pid.cmp(&a.pid)
                    }
                });
            }
            ProcessSortField::Name => {
                // Cache lowercase keys to avoid allocating in comparator
                self.processes.sort_unstable_by(|a, b| {
                    let cmp = a
                        .name
                        .as_bytes()
                        .iter()
                        .map(|c| c.to_ascii_lowercase())
                        .cmp(b.name.as_bytes().iter().map(|c| c.to_ascii_lowercase()));
                    if ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            ProcessSortField::Cpu => {
                self.processes.sort_unstable_by(|a, b| {
                    if ascending {
                        a.cpu_usage
                            .partial_cmp(&b.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b.cpu_usage
                            .partial_cmp(&a.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            ProcessSortField::Memory => {
                self.processes.sort_unstable_by(|a, b| {
                    if ascending {
                        a.memory.cmp(&b.memory)
                    } else {
                        b.memory.cmp(&a.memory)
                    }
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
            self.processes
                .iter()
                .filter(|p| contains_ignore_ascii_case(&p.name, &self.filter))
                .collect()
        }
    }

    pub fn filtered_count(&self) -> usize {
        if self.filter.is_empty() {
            self.processes.len()
        } else {
            self.processes
                .iter()
                .filter(|p| contains_ignore_ascii_case(&p.name, &self.filter))
                .count()
        }
    }

    pub fn tree_view(&self) -> Vec<ProcessInfo> {
        let pid_map: HashMap<u32, &ProcessInfo> =
            self.processes.iter().map(|p| (p.pid, p)).collect();

        let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut roots = Vec::new();

        for proc in &self.processes {
            match proc.parent_pid {
                Some(ppid) if pid_map.contains_key(&ppid) && ppid != proc.pid => {
                    children_map.entry(ppid).or_default().push(proc.pid);
                }
                _ => {
                    roots.push(proc.pid);
                }
            }
        }

        // Sort roots by sort field
        roots.sort_unstable_by(|a, b| {
            let pa = pid_map[a];
            let pb = pid_map[b];
            pb.cpu_usage
                .partial_cmp(&pa.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut result = Vec::new();
        for pid in &roots {
            Self::build_tree(*pid, 0, &pid_map, &children_map, &mut result);
        }

        // Apply filter
        if !self.filter.is_empty() {
            result.retain(|p| contains_ignore_ascii_case(&p.name, &self.filter));
        }

        result
    }

    fn build_tree(
        pid: u32,
        depth: usize,
        pid_map: &HashMap<u32, &ProcessInfo>,
        children_map: &HashMap<u32, Vec<u32>>,
        result: &mut Vec<ProcessInfo>,
    ) {
        if let Some(proc) = pid_map.get(&pid) {
            let mut p = (*proc).clone();
            p.depth = depth;
            result.push(p);

            if let Some(children) = children_map.get(&pid) {
                for child_pid in children {
                    Self::build_tree(*child_pid, depth + 1, pid_map, children_map, result);
                }
            }
        }
    }

    pub fn toggle_tree_mode(&mut self) {
        self.tree_mode = !self.tree_mode;
    }
}
