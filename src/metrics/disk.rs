use super::history::History;
use std::collections::HashMap;
use sysinfo::Disks;

pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_percent: f64,
    pub read_rate: f64,
    pub write_rate: f64,
}

pub struct DiskIoSnapshot {
    pub read_bytes: u64,
    pub written_bytes: u64,
}

pub struct DiskMetrics {
    pub disks: Vec<DiskInfo>,
    pub read_rate: f64,
    pub write_rate: f64,
    pub read_history: History,
    pub write_history: History,
    prev_snapshot: Option<DiskIoSnapshot>,
    prev_per_disk: HashMap<String, DiskIoSnapshot>,
}

impl DiskMetrics {
    pub fn new() -> Self {
        Self {
            disks: Vec::new(),
            read_rate: 0.0,
            write_rate: 0.0,
            read_history: History::new(),
            write_history: History::new(),
            prev_snapshot: None,
            prev_per_disk: HashMap::new(),
        }
    }

    pub fn update(&mut self, sysinfo_disks: &Disks) {
        self.disks.clear();
        let mut total_read: u64 = 0;
        let mut total_written: u64 = 0;
        let mut new_per_disk = HashMap::new();

        for disk in sysinfo_disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let used_pct = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let name = disk.name().to_string_lossy().to_string();
            let mount = disk.mount_point().to_string_lossy().to_string();

            // Skip tiny/virtual filesystems
            if total < 1_000_000 {
                continue;
            }

            let disk_read = disk.usage().read_bytes;
            let disk_written = disk.usage().written_bytes;

            let display_name = if name.is_empty() { mount.clone() } else { name };

            // Compute per-disk rates
            let (per_read_rate, per_write_rate) =
                if let Some(prev) = self.prev_per_disk.get(&display_name) {
                    (
                        disk_read.saturating_sub(prev.read_bytes) as f64,
                        disk_written.saturating_sub(prev.written_bytes) as f64,
                    )
                } else {
                    (0.0, 0.0)
                };

            new_per_disk.insert(
                display_name.clone(),
                DiskIoSnapshot {
                    read_bytes: disk_read,
                    written_bytes: disk_written,
                },
            );

            self.disks.push(DiskInfo {
                name: display_name,
                mount_point: mount,
                total_space: total,
                available_space: available,
                used_percent: used_pct,
                read_rate: per_read_rate,
                write_rate: per_write_rate,
            });

            total_read = total_read.wrapping_add(disk_read);
            total_written = total_written.wrapping_add(disk_written);
        }

        self.prev_per_disk = new_per_disk;

        let current = DiskIoSnapshot {
            read_bytes: total_read,
            written_bytes: total_written,
        };

        if let Some(prev) = &self.prev_snapshot {
            self.read_rate = total_read.saturating_sub(prev.read_bytes) as f64;
            self.write_rate = total_written.saturating_sub(prev.written_bytes) as f64;
        }

        self.prev_snapshot = Some(current);
        self.read_history.push(self.read_rate);
        self.write_history.push(self.write_rate);
    }
}
