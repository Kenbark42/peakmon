use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

use crate::event::{self, AppEvent};
use crate::logs::stream::LogStream;
use crate::metrics::process::ProcessSortField;
use crate::metrics::MetricsCollector;
use crate::ui::tabs::Tab;

pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub metrics: MetricsCollector,
    pub log_stream: LogStream,
    pub hostname: String,
    pub refresh_rate: Duration,
    pub scroll_offset: usize,
    pub filter_mode: bool,
    filter_buffer: String,
}

impl App {
    pub fn new(refresh_rate_ms: u64) -> Self {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            running: true,
            current_tab: Tab::Dashboard,
            metrics: MetricsCollector::new(),
            log_stream: LogStream::new(),
            hostname,
            refresh_rate: Duration::from_millis(refresh_rate_ms),
            scroll_offset: 0,
            filter_mode: false,
            filter_buffer: String::new(),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> color_eyre::Result<()> {
        let mut last_refresh = Instant::now();
        let poll_timeout = Duration::from_millis(250);

        // Initial metrics refresh
        self.metrics.refresh();

        while self.running {
            // Render
            terminal.draw(|frame| crate::ui::render(frame, self))?;

            // Poll events
            match event::poll_event(poll_timeout)? {
                AppEvent::Key(key) => self.handle_key(key),
                AppEvent::Resize => {}
                AppEvent::Tick => {}
            }

            // Periodic refresh
            if last_refresh.elapsed() >= self.refresh_rate {
                self.metrics.refresh();
                self.log_stream.poll();
                last_refresh = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Filter mode input handling
        if self.filter_mode {
            match key.code {
                KeyCode::Esc => {
                    self.filter_mode = false;
                    self.filter_buffer.clear();
                }
                KeyCode::Enter => {
                    self.filter_mode = false;
                    match self.current_tab {
                        Tab::Processes => {
                            self.metrics.processes.filter = self.filter_buffer.clone();
                        }
                        Tab::Logs => {
                            self.log_stream.text_filter = self.filter_buffer.clone();
                        }
                        _ => {}
                    }
                    self.filter_buffer.clear();
                    self.scroll_offset = 0;
                }
                KeyCode::Backspace => {
                    self.filter_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.filter_buffer.push(c);
                }
                _ => {}
            }
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
                return;
            }
            _ => {}
        }

        match key.code {
            // Tab selection by number
            KeyCode::Char('1') => self.switch_tab(Tab::Dashboard),
            KeyCode::Char('2') => self.switch_tab(Tab::Cpu),
            KeyCode::Char('3') => self.switch_tab(Tab::Memory),
            KeyCode::Char('4') => self.switch_tab(Tab::Disk),
            KeyCode::Char('5') => self.switch_tab(Tab::Network),
            KeyCode::Char('6') => self.switch_tab(Tab::Processes),
            KeyCode::Char('7') => self.switch_tab(Tab::Logs),
            KeyCode::Char('8') => self.switch_tab(Tab::Temperatures),

            // Tab cycling
            KeyCode::Tab => self.switch_tab(self.current_tab.next()),
            KeyCode::BackTab => self.switch_tab(self.current_tab.prev()),

            // Function keys
            KeyCode::F(n) if (1..=8).contains(&n) => {
                if let Some(tab) = Tab::from_index(n as usize - 1) {
                    self.switch_tab(tab);
                }
            }

            // Refresh rate
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let ms = self.refresh_rate.as_millis() as u64;
                let new_ms = ms.saturating_sub(250).max(250);
                self.refresh_rate = Duration::from_millis(new_ms);
            }
            KeyCode::Char('-') => {
                let ms = self.refresh_rate.as_millis() as u64;
                let new_ms = (ms + 250).min(10000);
                self.refresh_rate = Duration::from_millis(new_ms);
            }

            // Scroll
            KeyCode::Char('j') | KeyCode::Down => {
                match self.current_tab {
                    Tab::Temperatures => self.metrics.temperature.select_next(),
                    _ => self.scroll_offset = self.scroll_offset.saturating_add(1),
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.current_tab {
                    Tab::Temperatures => self.metrics.temperature.select_prev(),
                    _ => self.scroll_offset = self.scroll_offset.saturating_sub(1),
                }
            }
            KeyCode::Char('g') => self.scroll_offset = 0,
            KeyCode::Char('G') => self.scroll_offset = usize::MAX,
            KeyCode::PageDown => self.scroll_offset = self.scroll_offset.saturating_add(20),
            KeyCode::PageUp => self.scroll_offset = self.scroll_offset.saturating_sub(20),

            // Tab-specific keys
            KeyCode::Char('/') => {
                if matches!(self.current_tab, Tab::Processes | Tab::Logs) {
                    self.filter_mode = true;
                    self.filter_buffer.clear();
                }
            }

            // Process sort keys
            KeyCode::Char('c') if self.current_tab == Tab::Processes => {
                self.metrics.processes.set_sort_field(ProcessSortField::Cpu);
            }
            KeyCode::Char('m') if self.current_tab == Tab::Processes => {
                self.metrics
                    .processes
                    .set_sort_field(ProcessSortField::Memory);
            }
            KeyCode::Char('p') if self.current_tab == Tab::Processes => {
                self.metrics.processes.set_sort_field(ProcessSortField::Pid);
            }
            KeyCode::Char('n') if self.current_tab == Tab::Processes => {
                self.metrics
                    .processes
                    .set_sort_field(ProcessSortField::Name);
            }

            // Log keys
            KeyCode::Char('l') if self.current_tab == Tab::Logs => {
                self.log_stream.cycle_level_filter();
            }
            KeyCode::Char('a') if self.current_tab == Tab::Logs => {
                self.log_stream.toggle_auto_scroll();
            }

            _ => {}
        }
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
        self.scroll_offset = 0;
    }
}
