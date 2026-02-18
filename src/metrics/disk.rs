use super::history::History;
use sysinfo::Disks;

pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_percent: f64,
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
        }
    }

    pub fn update(&mut self, sysinfo_disks: &Disks) {
        self.disks.clear();
        let mut total_read: u64 = 0;
        let mut total_written: u64 = 0;

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

            self.disks.push(DiskInfo {
                name: if name.is_empty() {
                    mount.clone()
                } else {
                    name
                },
                mount_point: mount,
                total_space: total,
                available_space: available,
                used_percent: used_pct,
            });

            total_read = total_read.wrapping_add(disk.usage().read_bytes);
            total_written = total_written.wrapping_add(disk.usage().written_bytes);
        }

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
