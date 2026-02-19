use super::history::History;
use std::process::Command;
use std::time::Instant;

pub struct BatteryMetrics {
    pub available: bool,
    pub percent: f64,
    pub is_charging: bool,
    pub external_connected: bool,
    pub fully_charged: bool,
    pub cycle_count: u32,
    pub health_percent: f64,
    pub temperature: f64,
    pub voltage_mv: u32,
    pub amperage_ma: i32,
    pub watts: f64,
    pub design_capacity_mah: u32,
    pub max_capacity_mah: u32,
    pub current_capacity_mah: u32,
    pub time_to_empty_mins: Option<u32>,
    pub time_to_full_mins: Option<u32>,
    pub percent_history: History,
    last_update: Option<Instant>,
}

impl BatteryMetrics {
    pub fn new() -> Self {
        let mut m = Self {
            available: false,
            percent: 0.0,
            is_charging: false,
            external_connected: false,
            fully_charged: false,
            cycle_count: 0,
            health_percent: 0.0,
            temperature: 0.0,
            voltage_mv: 0,
            amperage_ma: 0,
            watts: 0.0,
            design_capacity_mah: 0,
            max_capacity_mah: 0,
            current_capacity_mah: 0,
            time_to_empty_mins: None,
            time_to_full_mins: None,
            percent_history: History::new(),
            last_update: None,
        };
        m.detect();
        m
    }

    fn detect(&mut self) {
        if let Some(output) = Self::run_ioreg() {
            self.available = extract_bool(&output, "\"BatteryInstalled\"").unwrap_or(false);
        }
    }

    pub fn update(&mut self) {
        if !self.available {
            return;
        }

        // Throttle ioreg to every 5 seconds
        if let Some(last) = self.last_update {
            if last.elapsed().as_secs() < 5 {
                return;
            }
        }
        self.last_update = Some(Instant::now());

        let Some(output) = Self::run_ioreg() else {
            return;
        };

        self.is_charging = extract_bool(&output, "\"IsCharging\"").unwrap_or(false);
        self.external_connected = extract_bool(&output, "\"ExternalConnected\"").unwrap_or(false);
        self.fully_charged = extract_bool(&output, "\"FullyCharged\"").unwrap_or(false);

        self.current_capacity_mah = extract_number(&output, "\"CurrentCapacity\"").unwrap_or(0);
        self.max_capacity_mah = extract_number(&output, "\"MaxCapacity\"").unwrap_or(0);
        self.design_capacity_mah = extract_number(&output, "\"DesignCapacity\"").unwrap_or(0);
        self.cycle_count = extract_number(&output, "\"CycleCount\"").unwrap_or(0);
        self.voltage_mv = extract_number(&output, "\"Voltage\"").unwrap_or(0);
        self.amperage_ma = extract_signed(&output, "\"Amperage\"").unwrap_or(0);

        let temp_raw: u32 = extract_number(&output, "\"Temperature\"").unwrap_or(0);
        self.temperature = temp_raw as f64 / 100.0;

        let time_empty: u32 = extract_number(&output, "\"AvgTimeToEmpty\"").unwrap_or(65535);
        self.time_to_empty_mins = if time_empty == 65535 {
            None
        } else {
            Some(time_empty)
        };

        let time_full: u32 = extract_number(&output, "\"AvgTimeToFull\"").unwrap_or(65535);
        self.time_to_full_mins = if time_full == 65535 {
            None
        } else {
            Some(time_full)
        };

        // Derived: percentage
        self.percent = if self.max_capacity_mah > 0 {
            (self.current_capacity_mah as f64 / self.max_capacity_mah as f64) * 100.0
        } else {
            0.0
        };

        // Derived: health
        self.health_percent = if self.design_capacity_mah > 0 {
            (self.max_capacity_mah as f64 / self.design_capacity_mah as f64) * 100.0
        } else {
            0.0
        };

        // Derived: watts (mV * mA = µW, / 1_000_000 = W)
        self.watts =
            (self.voltage_mv as f64 * self.amperage_ma.unsigned_abs() as f64) / 1_000_000.0;

        self.percent_history.push(self.percent);
    }

    fn run_ioreg() -> Option<String> {
        Command::new("ioreg")
            .args(["-rd1", "-c", "AppleSmartBattery"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    }
}

fn extract_number<T: std::str::FromStr>(text: &str, key: &str) -> Option<T> {
    let idx = text.find(key)?;
    let after_key = &text[idx + key.len()..];
    let eq_pos = after_key.find('=')?;
    let after_eq = after_key[eq_pos + 1..].trim_start();
    let end = after_eq
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after_eq.len());
    after_eq[..end].parse().ok()
}

fn extract_signed(text: &str, key: &str) -> Option<i32> {
    let idx = text.find(key)?;
    let after_key = &text[idx + key.len()..];
    let eq_pos = after_key.find('=')?;
    let after_eq = after_key[eq_pos + 1..].trim_start();
    let end = after_eq
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(after_eq.len());
    after_eq[..end].parse().ok()
}

fn extract_bool(text: &str, key: &str) -> Option<bool> {
    let idx = text.find(key)?;
    let after_key = &text[idx + key.len()..];
    let eq_pos = after_key.find('=')?;
    let after_eq = after_key[eq_pos + 1..].trim_start();
    if after_eq.starts_with("Yes") {
        Some(true)
    } else if after_eq.starts_with("No") {
        Some(false)
    } else {
        None
    }
}
