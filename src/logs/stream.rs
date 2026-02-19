use super::{LogEntry, LogLevel};
use crate::util::contains_ignore_ascii_case;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub struct LogStream {
    pub entries: VecDeque<LogEntry>,
    pub max_entries: usize,
    pub auto_scroll: bool,
    pub level_filter: Option<LogLevel>,
    pub text_filter: String,
    receiver: mpsc::Receiver<LogEntry>,
    _handle: Option<thread::JoinHandle<()>>,
}

impl LogStream {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let max_entries = 5000;

        let handle = thread::spawn(move || {
            let child = Command::new("log")
                .args(["stream", "--style=compact", "--level=default"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();

            let child = match child {
                Ok(c) => c,
                Err(_) => return,
            };

            let stdout = match child.stdout {
                Some(s) => s,
                None => return,
            };

            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                if line.trim().is_empty() || line.starts_with("Filtering") {
                    continue;
                }

                let entry = parse_log_line(&line);
                if tx.send(entry).is_err() {
                    break;
                }
            }
        });

        Self {
            entries: VecDeque::new(),
            max_entries,
            auto_scroll: true,
            level_filter: None,
            text_filter: String::new(),
            receiver: rx,
            _handle: Some(handle),
        }
    }

    pub fn poll(&mut self) {
        while let Ok(entry) = self.receiver.try_recv() {
            self.entries.push_back(entry);
            if self.entries.len() > self.max_entries {
                self.entries.pop_front();
            }
        }
    }

    pub fn filtered_entries(&self) -> Vec<&LogEntry> {
        let has_text_filter = !self.text_filter.is_empty();
        self.entries
            .iter()
            .filter(|e| {
                if let Some(ref level) = self.level_filter {
                    if &e.level != level {
                        return false;
                    }
                }
                if has_text_filter
                    && !contains_ignore_ascii_case(&e.message, &self.text_filter)
                    && !contains_ignore_ascii_case(&e.process, &self.text_filter)
                {
                    return false;
                }
                true
            })
            .collect()
    }

    pub fn cycle_level_filter(&mut self) {
        self.level_filter = match &self.level_filter {
            None => Some(LogLevel::Error),
            Some(LogLevel::Error) => Some(LogLevel::Fault),
            Some(LogLevel::Fault) => Some(LogLevel::Info),
            Some(LogLevel::Info) => Some(LogLevel::Debug),
            Some(LogLevel::Debug) => Some(LogLevel::Default),
            Some(LogLevel::Default) => None,
        };
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }
}

fn parse_log_line(line: &str) -> LogEntry {
    // Compact format: "2024-01-01 12:00:00.000 Df processname[pid]: message"
    // The level indicator is a two-char code after timestamp
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() >= 4 {
        let timestamp = format!("{} {}", parts[0], parts[1]);
        let level_and_rest = parts[2];

        // The level code is typically the first 2 chars
        let level = if level_and_rest.starts_with("Df") || level_and_rest.starts_with("df") {
            LogLevel::Default
        } else if level_and_rest.starts_with("In") || level_and_rest.starts_with("in") {
            LogLevel::Info
        } else if level_and_rest.starts_with("Db") || level_and_rest.starts_with("db") {
            LogLevel::Debug
        } else if level_and_rest.starts_with("Er") || level_and_rest.starts_with("er") {
            LogLevel::Error
        } else if level_and_rest.starts_with("Ft") || level_and_rest.starts_with("ft") {
            LogLevel::Fault
        } else {
            LogLevel::Default
        };

        // Extract process name from the rest
        let rest = parts[3];
        let (process, message) = if let Some(colon_pos) = rest.find(": ") {
            let proc_part = &rest[..colon_pos];
            // Strip PID bracket if present
            let proc_name = if let Some(bracket) = proc_part.find('[') {
                &proc_part[..bracket]
            } else {
                proc_part
            };
            (proc_name.to_string(), rest[colon_pos + 2..].to_string())
        } else {
            ("unknown".to_string(), rest.to_string())
        };

        LogEntry {
            timestamp,
            level,
            process,
            message,
        }
    } else {
        LogEntry {
            timestamp: String::new(),
            level: LogLevel::Default,
            process: String::new(),
            message: line.to_string(),
        }
    }
}
