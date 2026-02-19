use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

use crate::event::{self, AppEvent};
use crate::logs::stream::LogStream;
use crate::metrics::ai::ChatMessage;
use crate::metrics::process::ProcessSortField;
use crate::metrics::MetricsCollector;
use crate::ui::tabs::Tab;

#[derive(Clone, Copy, PartialEq)]
pub enum AiInputMode {
    Normal,
    PullPrompt,
    ChatInput,
    SearchInput,
}

pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub metrics: MetricsCollector,
    pub log_stream: LogStream,
    pub hostname: String,
    pub refresh_rate: Duration,
    pub scroll_offset: usize,
    pub filter_mode: bool,
    pub filter_buffer: String,
    pub viewport_height: usize,
    pub process_selected: usize,
    pub confirm_kill: Option<(u32, String)>,
    pub show_help: bool,
    pub ai_input_mode: AiInputMode,
    pub ai_input_buffer: String,
    pub ai_confirm_delete: Option<String>,
    pub ai_chat_scroll: usize,
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
            viewport_height: 24,
            process_selected: 0,
            confirm_kill: None,
            show_help: false,
            ai_input_mode: AiInputMode::Normal,
            ai_input_buffer: String::new(),
            ai_confirm_delete: None,
            ai_chat_scroll: 0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> color_eyre::Result<()> {
        let mut last_refresh = Instant::now();
        let poll_timeout = Duration::from_millis(250);

        // Initial metrics refresh — refresh all subsystems
        self.metrics.refresh(Tab::Dashboard);

        while self.running {
            // Render
            terminal.draw(|frame| {
                let area = frame.area();
                // body = total height - 2 (header) - 1 (footer) - 2 (borders)
                self.viewport_height = area.height.saturating_sub(5) as usize;
                crate::ui::render(frame, self);
            })?;

            // Poll events
            match event::poll_event(poll_timeout)? {
                AppEvent::Key(key) => self.handle_key(key),
                AppEvent::Mouse(mouse) => self.handle_mouse(mouse),
                AppEvent::Resize => {}
                AppEvent::Tick => {}
            }

            // Periodic refresh
            if last_refresh.elapsed() >= self.refresh_rate {
                self.metrics.refresh(self.current_tab);
                self.log_stream.poll();
                last_refresh = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Kill confirmation mode
        if let Some((pid, _)) = &self.confirm_kill {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let pid = *pid;
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                    self.confirm_kill = None;
                }
                _ => {
                    self.confirm_kill = None;
                }
            }
            return;
        }

        // AI delete confirmation mode
        if let Some(ref model_name) = self.ai_confirm_delete.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.metrics.ai.delete_model(model_name);
                    self.ai_confirm_delete = None;
                }
                _ => {
                    self.ai_confirm_delete = None;
                }
            }
            return;
        }

        // Help overlay
        if self.show_help {
            self.show_help = false;
            return;
        }

        // AI search results overlay
        if self.metrics.ai.show_search {
            match key.code {
                KeyCode::Esc => {
                    self.metrics.ai.dismiss_search();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.metrics.ai.search_select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.metrics.ai.search_select_prev();
                }
                KeyCode::Enter => {
                    if let Some(name) = self.metrics.ai.selected_search_model() {
                        self.metrics.ai.start_pull(name);
                        self.metrics.ai.dismiss_search();
                    }
                }
                _ => {}
            }
            return;
        }

        // AI pull prompt input
        if self.ai_input_mode == AiInputMode::PullPrompt {
            match key.code {
                KeyCode::Esc => {
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Enter => {
                    let name = self.ai_input_buffer.trim().to_string();
                    if !name.is_empty() {
                        self.metrics.ai.start_pull(name);
                    }
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.ai_input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.ai_input_buffer.push(c);
                }
                _ => {}
            }
            return;
        }

        // AI chat input mode
        if self.ai_input_mode == AiInputMode::ChatInput {
            match key.code {
                KeyCode::Esc => {
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Enter => {
                    let prompt = self.ai_input_buffer.trim().to_string();
                    if !prompt.is_empty() {
                        if let Some(model) = self.metrics.ai.first_loaded_model_name() {
                            // Add user message
                            self.metrics.ai.chat_messages.push(ChatMessage {
                                role: "user".to_string(),
                                content: prompt,
                            });
                            // Add empty assistant placeholder
                            self.metrics.ai.chat_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                            });
                            // Start streaming chat
                            let messages = self.metrics.ai.chat_messages.clone();
                            self.metrics.ai.start_chat(&model, &messages);
                        }
                    }
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.ai_input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.ai_input_buffer.push(c);
                }
                _ => {}
            }
            return;
        }

        // AI search input mode
        if self.ai_input_mode == AiInputMode::SearchInput {
            match key.code {
                KeyCode::Esc => {
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Enter => {
                    let query = self.ai_input_buffer.trim().to_string();
                    if !query.is_empty() {
                        self.metrics.ai.start_search(query);
                    }
                    self.ai_input_mode = AiInputMode::Normal;
                    self.ai_input_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.ai_input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.ai_input_buffer.push(c);
                }
                _ => {}
            }
            return;
        }

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
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            _ => {}
        }

        match key.code {
            // Tab selection by number (1-9 + 0 for 10 tabs)
            KeyCode::Char('1') => self.switch_tab(Tab::Dashboard),
            KeyCode::Char('2') => self.switch_tab(Tab::Cpu),
            KeyCode::Char('3') => self.switch_tab(Tab::Gpu),
            KeyCode::Char('4') => self.switch_tab(Tab::Ai),
            KeyCode::Char('5') => self.switch_tab(Tab::Memory),
            KeyCode::Char('6') => self.switch_tab(Tab::Disk),
            KeyCode::Char('7') => self.switch_tab(Tab::Network),
            KeyCode::Char('8') => self.switch_tab(Tab::Processes),
            KeyCode::Char('9') => self.switch_tab(Tab::Logs),
            KeyCode::Char('0') => self.switch_tab(Tab::Temperatures),

            // Tab cycling
            KeyCode::Tab => self.switch_tab(self.current_tab.next()),
            KeyCode::BackTab => self.switch_tab(self.current_tab.prev()),

            // Function keys
            KeyCode::F(n) if (1..=10).contains(&n) => {
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

            // Scroll / selection
            KeyCode::Char('j') | KeyCode::Down => match self.current_tab {
                Tab::Temperatures => self.metrics.temperature.select_next(),
                Tab::Ai => self.metrics.ai.select_next(),
                Tab::Processes => {
                    let count = self.metrics.processes.filtered_processes().len();
                    if count > 0 {
                        self.process_selected = (self.process_selected + 1).min(count - 1);
                        // Auto-scroll to keep selection visible
                        if self.process_selected >= self.scroll_offset + self.viewport_height {
                            self.scroll_offset = self.process_selected - self.viewport_height + 1;
                        }
                    }
                }
                _ => self.scroll_offset = self.scroll_offset.saturating_add(1),
            },
            KeyCode::Char('k') | KeyCode::Up => match self.current_tab {
                Tab::Temperatures => self.metrics.temperature.select_prev(),
                Tab::Ai => self.metrics.ai.select_prev(),
                Tab::Processes => {
                    self.process_selected = self.process_selected.saturating_sub(1);
                    if self.process_selected < self.scroll_offset {
                        self.scroll_offset = self.process_selected;
                    }
                }
                _ => self.scroll_offset = self.scroll_offset.saturating_sub(1),
            },
            KeyCode::Char('g') => {
                self.scroll_offset = 0;
                if self.current_tab == Tab::Processes {
                    self.process_selected = 0;
                }
            }
            KeyCode::Char('G') => {
                self.scroll_offset = usize::MAX;
                if self.current_tab == Tab::Processes {
                    let count = self.metrics.processes.filtered_processes().len();
                    self.process_selected = count.saturating_sub(1);
                }
            }
            KeyCode::PageDown => {
                if self.current_tab == Tab::Processes {
                    let count = self.metrics.processes.filtered_processes().len();
                    self.process_selected =
                        (self.process_selected + self.viewport_height).min(count.saturating_sub(1));
                    self.scroll_offset = self.scroll_offset.saturating_add(self.viewport_height);
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_add(self.viewport_height);
                }
            }
            KeyCode::PageUp => {
                if self.current_tab == Tab::Processes {
                    self.process_selected =
                        self.process_selected.saturating_sub(self.viewport_height);
                }
                self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
            }

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

            // Tree view toggle
            KeyCode::Char('t') if self.current_tab == Tab::Processes => {
                self.metrics.processes.toggle_tree_mode();
                self.process_selected = 0;
                self.scroll_offset = 0;
            }

            // Kill process
            KeyCode::Char('K') if self.current_tab == Tab::Processes => {
                let filtered = self.metrics.processes.filtered_processes();
                if let Some(proc) = filtered.get(self.process_selected) {
                    self.confirm_kill = Some((proc.pid, proc.name.clone()));
                }
            }

            // AI tab keys
            KeyCode::Char('P') if self.current_tab == Tab::Ai => {
                if self.metrics.ai.ollama_available {
                    self.ai_input_mode = AiInputMode::PullPrompt;
                    self.ai_input_buffer.clear();
                }
            }
            KeyCode::Char('D') if self.current_tab == Tab::Ai => {
                if let Some(name) = self.metrics.ai.selected_model_name() {
                    self.ai_confirm_delete = Some(name);
                }
            }
            KeyCode::Enter if self.current_tab == Tab::Ai => {
                if let Some(name) = self.metrics.ai.selected_model_name() {
                    self.metrics.ai.load_model(&name);
                }
            }
            KeyCode::Char('U') if self.current_tab == Tab::Ai => {
                if let Some(name) = self.metrics.ai.selected_model_name() {
                    self.metrics.ai.unload_model(&name);
                }
            }
            KeyCode::Char('i') if self.current_tab == Tab::Ai => {
                if self.metrics.ai.has_loaded_model() {
                    self.ai_input_mode = AiInputMode::ChatInput;
                    self.ai_input_buffer.clear();
                }
            }
            KeyCode::Char('S') if self.current_tab == Tab::Ai => {
                if self.metrics.ai.ollama_available {
                    self.ai_input_mode = AiInputMode::SearchInput;
                    self.ai_input_buffer.clear();
                }
            }
            KeyCode::Char('C') if self.current_tab == Tab::Ai => {
                self.metrics.ai.clear_chat();
                self.ai_chat_scroll = 0;
            }
            KeyCode::Esc if self.current_tab == Tab::Ai => {
                self.metrics.ai.cancel_chat();
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
        if tab == Tab::Processes {
            self.process_selected = 0;
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        // Dismiss overlays on any click
        if self.show_help
            || self.confirm_kill.is_some()
            || self.ai_confirm_delete.is_some()
            || self.metrics.ai.show_search
        {
            if matches!(mouse.kind, MouseEventKind::Down(_)) {
                self.show_help = false;
                self.confirm_kill = None;
                self.ai_confirm_delete = None;
                self.metrics.ai.dismiss_search();
            }
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.current_tab == Tab::Processes {
                    self.process_selected = self.process_selected.saturating_sub(3);
                    if self.process_selected < self.scroll_offset {
                        self.scroll_offset = self.process_selected;
                    }
                } else if self.current_tab == Tab::Temperatures {
                    self.metrics.temperature.select_prev();
                } else if self.current_tab == Tab::Ai {
                    self.metrics.ai.select_prev();
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if self.current_tab == Tab::Processes {
                    let count = self.metrics.processes.filtered_processes().len();
                    if count > 0 {
                        self.process_selected = (self.process_selected + 3).min(count - 1);
                        if self.process_selected >= self.scroll_offset + self.viewport_height {
                            self.scroll_offset = self.process_selected - self.viewport_height + 1;
                        }
                    }
                } else if self.current_tab == Tab::Temperatures {
                    self.metrics.temperature.select_next();
                } else if self.current_tab == Tab::Ai {
                    self.metrics.ai.select_next();
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_add(3);
                }
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let row = mouse.row;
                let col = mouse.column;

                // Tab bar is on row 1 (second row of header)
                if row == 1 {
                    self.handle_tab_click(col);
                }
            }
            _ => {}
        }
    }

    fn handle_tab_click(&mut self, col: u16) {
        // Tab bar format: " N:Label  N:Label  ..."
        // Each tab is roughly: 1 space + "N:Label" + 1 space
        let mut x: u16 = 1; // initial space
        for tab in &Tab::ALL {
            let num = tab.index() + 1;
            let display_num = if num == 10 {
                "0".to_string()
            } else {
                num.to_string()
            };
            let label = format!(" {display_num}:{} ", tab.label());
            let width = label.len() as u16;
            if col >= x && col < x + width {
                self.switch_tab(*tab);
                return;
            }
            x += width + 1; // +1 for the gap space
        }
    }
}
