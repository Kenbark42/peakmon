use super::history::History;
use sysinfo::Components;

pub struct SensorInfo {
    pub label: String,
    pub temperature: f64,
    pub max_temperature: f64,
    pub history: History,
}

pub struct TemperatureMetrics {
    pub sensors: Vec<SensorInfo>,
    pub selected_sensor: usize,
}

impl TemperatureMetrics {
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            selected_sensor: 0,
        }
    }

    pub fn update(&mut self, components: &Components) {
        for component in components.list() {
            let label = component.label().to_string();
            let temp = component.temperature().unwrap_or(0.0) as f64;
            let max = component.max().unwrap_or(0.0) as f64;

            if let Some(sensor) = self.sensors.iter_mut().find(|s| s.label == label) {
                sensor.temperature = temp;
                sensor.max_temperature = max;
                sensor.history.push(temp);
            } else {
                let mut sensor = SensorInfo {
                    label,
                    temperature: temp,
                    max_temperature: max,
                    history: History::new(),
                };
                sensor.history.push(temp);
                self.sensors.push(sensor);
            }
        }

        // Sort sensors by label for consistent display
        self.sensors.sort_by(|a, b| a.label.cmp(&b.label));
    }

    pub fn select_next(&mut self) {
        if !self.sensors.is_empty() {
            self.selected_sensor = (self.selected_sensor + 1) % self.sensors.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.sensors.is_empty() {
            self.selected_sensor = if self.selected_sensor == 0 {
                self.sensors.len() - 1
            } else {
                self.selected_sensor - 1
            };
        }
    }
}
