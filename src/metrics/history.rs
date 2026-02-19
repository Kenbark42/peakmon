use std::collections::VecDeque;

const DEFAULT_CAPACITY: usize = 300; // 5 min at 1s intervals

#[derive(Clone)]
pub struct History {
    data: VecDeque<f64>,
    capacity: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            data: VecDeque::with_capacity(DEFAULT_CAPACITY),
            capacity: DEFAULT_CAPACITY,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn max(&self) -> f64 {
        self.data.iter().copied().fold(0.0_f64, f64::max)
    }

    pub fn as_u64_vec(&self, count: usize) -> Vec<u64> {
        let len = self.data.len();
        let skip = len.saturating_sub(count);
        self.data.iter().skip(skip).map(|&v| v as u64).collect()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
