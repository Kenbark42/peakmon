use super::history::History;
use std::process::Command;

pub struct GpuMetrics {
    pub model: String,
    pub core_count: u32,
    pub device_utilization: f64,
    pub renderer_utilization: f64,
    pub tiler_utilization: f64,
    pub in_use_memory: u64,
    pub alloc_memory: u64,
    pub utilization_history: History,
}

impl GpuMetrics {
    pub fn new() -> Self {
        let (model, core_count) = Self::detect_gpu();
        Self {
            model,
            core_count,
            device_utilization: 0.0,
            renderer_utilization: 0.0,
            tiler_utilization: 0.0,
            in_use_memory: 0,
            alloc_memory: 0,
            utilization_history: History::new(),
        }
    }

    fn detect_gpu() -> (String, u32) {
        let output = Command::new("ioreg")
            .args(["-r", "-d", "1", "-c", "IOAccelerator"])
            .output();

        let output = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return ("Unknown GPU".to_string(), 0),
        };

        let model = Self::extract_string(&output, "\"model\" = \"")
            .unwrap_or_else(|| "Unknown GPU".to_string());
        let core_count = Self::extract_number(&output, "\"gpu-core-count\" = ").unwrap_or(0);

        (model, core_count)
    }

    pub fn update(&mut self) {
        let output = Command::new("ioreg")
            .args(["-r", "-d", "1", "-c", "IOAccelerator"])
            .output();

        let output = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return,
        };

        self.device_utilization =
            Self::extract_number(&output, "\"Device Utilization %\" = ").unwrap_or(0) as f64;
        self.renderer_utilization =
            Self::extract_number(&output, "\"Renderer Utilization %\" = ").unwrap_or(0) as f64;
        self.tiler_utilization =
            Self::extract_number(&output, "\"Tiler Utilization %\" = ").unwrap_or(0) as f64;
        self.in_use_memory =
            Self::extract_number::<u64>(&output, "\"In use system memory\" = ").unwrap_or(0);
        self.alloc_memory =
            Self::extract_number::<u64>(&output, "\"Alloc system memory\" = ").unwrap_or(0);

        self.utilization_history.push(self.device_utilization);
    }

    fn extract_string(text: &str, prefix: &str) -> Option<String> {
        let start = text.find(prefix)? + prefix.len();
        let end = text[start..].find('"')? + start;
        Some(text[start..end].to_string())
    }

    fn extract_number<T: std::str::FromStr>(text: &str, prefix: &str) -> Option<T> {
        let start = text.find(prefix)? + prefix.len();
        let rest = &text[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    }
}
