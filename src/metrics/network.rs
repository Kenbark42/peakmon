use super::history::History;
use sysinfo::Networks;

pub struct InterfaceMetrics {
    pub name: String,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rx_history: History,
    pub tx_history: History,
    prev_rx: u64,
    prev_tx: u64,
}

pub struct NetworkMetrics {
    pub interfaces: Vec<InterfaceMetrics>,
    pub total_rx_rate: f64,
    pub total_tx_rate: f64,
    pub total_rx_history: History,
    pub total_tx_history: History,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
            total_rx_rate: 0.0,
            total_tx_rate: 0.0,
            total_rx_history: History::new(),
            total_tx_history: History::new(),
        }
    }

    pub fn update(&mut self, networks: &Networks) {
        let mut total_rx: f64 = 0.0;
        let mut total_tx: f64 = 0.0;

        for (name, data) in networks.list() {
            let rx = data.total_received();
            let tx = data.total_transmitted();

            if let Some(iface) = self.interfaces.iter_mut().find(|i| i.name == *name) {
                iface.rx_rate = rx.saturating_sub(iface.prev_rx) as f64;
                iface.tx_rate = tx.saturating_sub(iface.prev_tx) as f64;
                iface.prev_rx = rx;
                iface.prev_tx = tx;
                iface.rx_history.push(iface.rx_rate);
                iface.tx_history.push(iface.tx_rate);
                total_rx += iface.rx_rate;
                total_tx += iface.tx_rate;
            } else {
                let mut iface = InterfaceMetrics {
                    name: name.clone(),
                    rx_rate: 0.0,
                    tx_rate: 0.0,
                    rx_history: History::new(),
                    tx_history: History::new(),
                    prev_rx: rx,
                    prev_tx: tx,
                };
                iface.rx_history.push(0.0);
                iface.tx_history.push(0.0);
                self.interfaces.push(iface);
            }
        }

        // Filter out loopback and inactive interfaces for display
        self.interfaces.retain(|i| {
            // Keep interfaces that have seen some traffic
            i.prev_rx > 0 || i.prev_tx > 0
        });

        self.total_rx_rate = total_rx;
        self.total_tx_rate = total_tx;
        self.total_rx_history.push(total_rx);
        self.total_tx_history.push(total_tx);
    }
}
