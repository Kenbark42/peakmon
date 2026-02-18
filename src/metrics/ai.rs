use super::history::History;
use super::process::ProcessInfo;
use crate::util::format_bytes;
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

// --- Service Detection ---

#[derive(Clone)]
pub struct AiService {
    pub name: &'static str,
    pub detected: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
}

struct ServicePattern {
    name: &'static str,
    process_patterns: &'static [&'static str],
}

const SERVICE_PATTERNS: &[ServicePattern] = &[
    ServicePattern {
        name: "Ollama",
        process_patterns: &["ollama"],
    },
    ServicePattern {
        name: "LM Studio",
        process_patterns: &["LM Studio", "lmstudio"],
    },
    ServicePattern {
        name: "llama.cpp",
        process_patterns: &["llama-server", "llama-cli"],
    },
    ServicePattern {
        name: "Claude Code",
        process_patterns: &["claude"],
    },
    ServicePattern {
        name: "MLX",
        process_patterns: &["mlx"],
    },
    ServicePattern {
        name: "vLLM",
        process_patterns: &["vllm"],
    },
    ServicePattern {
        name: "Open WebUI",
        process_patterns: &["open-webui"],
    },
    ServicePattern {
        name: "GPT4All",
        process_patterns: &["gpt4all"],
    },
    ServicePattern {
        name: "Whisper",
        process_patterns: &["whisper"],
    },
    ServicePattern {
        name: "Stable Diffusion",
        process_patterns: &["stable-diffusion", "comfy"],
    },
];

// All process name patterns for AI filtering
const AI_PROCESS_PATTERNS: &[&str] = &[
    "ollama",
    "lm studio",
    "lmstudio",
    "llama-server",
    "llama-cli",
    "llama",
    "claude",
    "mlx",
    "vllm",
    "open-webui",
    "gpt4all",
    "whisper",
    "stable-diffusion",
    "comfy",
];

// --- Ollama API types ---

#[derive(Deserialize)]
struct OllamaVersion {
    version: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModel>>,
}

#[derive(Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub details: Option<OllamaModelDetails>,
}

#[derive(Deserialize, Clone)]
pub struct OllamaModelDetails {
    pub quantization_level: Option<String>,
}

#[derive(Deserialize)]
struct OllamaPsResponse {
    models: Option<Vec<OllamaRunningModel>>,
}

#[derive(Deserialize, Clone)]
pub struct OllamaRunningModel {
    pub name: String,
    pub size_vram: u64,
}

// --- Pull Progress ---

#[derive(Clone)]
pub enum PullStatus {
    Progress {
        status: String,
        percent: Option<f64>,
    },
    Done,
    Error(String),
}

#[derive(Deserialize)]
struct PullProgressLine {
    status: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
}

// --- Main AI Metrics Struct ---

pub struct AiMetrics {
    pub services: Vec<AiService>,
    pub ollama_available: bool,
    pub ollama_version: Option<String>,
    pub ollama_models: Vec<OllamaModel>,
    pub ollama_running: Vec<OllamaRunningModel>,
    pub ai_processes: Vec<ProcessInfo>,
    pub aggregate_cpu: f64,
    pub aggregate_memory: u64,
    pub cpu_history: History,
    pub model_selected: usize,
    pub pull_status: Option<PullStatus>,
    pull_receiver: Option<mpsc::Receiver<PullStatus>>,
    last_api_check: Option<Instant>,
    api_cache_secs: u64,
}

impl AiMetrics {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            ollama_available: false,
            ollama_version: None,
            ollama_models: Vec::new(),
            ollama_running: Vec::new(),
            ai_processes: Vec::new(),
            aggregate_cpu: 0.0,
            aggregate_memory: 0,
            cpu_history: History::new(),
            model_selected: 0,
            pull_status: None,
            pull_receiver: None,
            last_api_check: None,
            api_cache_secs: 5,
        }
    }

    pub fn update(&mut self, processes: &[ProcessInfo]) {
        self.detect_services(processes);
        self.filter_ai_processes(processes);
        self.poll_pull_status();

        let should_check_api = match self.last_api_check {
            Some(t) => t.elapsed().as_secs() >= self.api_cache_secs,
            None => true,
        };

        if should_check_api {
            self.refresh_ollama_api();
            self.last_api_check = Some(Instant::now());
        }

        self.cpu_history.push(self.aggregate_cpu);
    }

    fn detect_services(&mut self, processes: &[ProcessInfo]) {
        self.services = SERVICE_PATTERNS
            .iter()
            .map(|sp| {
                let matched = processes.iter().find(|p| {
                    let name_lower = p.name.to_lowercase();
                    sp.process_patterns
                        .iter()
                        .any(|pat| name_lower.contains(&pat.to_lowercase()))
                });

                AiService {
                    name: sp.name,
                    detected: matched.is_some(),
                    version: if sp.name == "Ollama" {
                        self.ollama_version.clone()
                    } else {
                        None
                    },
                    pid: matched.map(|p| p.pid),
                }
            })
            .collect();

        self.ollama_available = self
            .services
            .iter()
            .any(|s| s.name == "Ollama" && s.detected);
    }

    fn filter_ai_processes(&mut self, processes: &[ProcessInfo]) {
        self.ai_processes = processes
            .iter()
            .filter(|p| {
                let name_lower = p.name.to_lowercase();
                AI_PROCESS_PATTERNS
                    .iter()
                    .any(|pat| name_lower.contains(pat))
            })
            .cloned()
            .collect();

        self.ai_processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.aggregate_cpu = self.ai_processes.iter().map(|p| p.cpu_usage).sum();
        self.aggregate_memory = self.ai_processes.iter().map(|p| p.memory).sum();
    }

    fn refresh_ollama_api(&mut self) {
        if !self.ollama_available {
            self.ollama_models.clear();
            self.ollama_running.clear();
            self.ollama_version = None;
            return;
        }

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_millis(200))
            .timeout_read(std::time::Duration::from_secs(1))
            .build();

        // Version
        if let Ok(resp) = agent.get("http://localhost:11434/api/version").call() {
            if let Ok(v) = resp.into_json::<OllamaVersion>() {
                self.ollama_version = Some(v.version);
            }
        }

        // Tags (available models)
        if let Ok(resp) = agent.get("http://localhost:11434/api/tags").call() {
            if let Ok(tags) = resp.into_json::<OllamaTagsResponse>() {
                self.ollama_models = tags.models.unwrap_or_default();
            }
        }

        // Running models
        if let Ok(resp) = agent.get("http://localhost:11434/api/ps").call() {
            if let Ok(ps) = resp.into_json::<OllamaPsResponse>() {
                self.ollama_running = ps.models.unwrap_or_default();
            }
        }
    }

    // --- Model Management ---

    pub fn start_pull(&mut self, model_name: String) {
        let (tx, rx) = mpsc::channel();
        self.pull_receiver = Some(rx);
        self.pull_status = Some(PullStatus::Progress {
            status: "Starting pull...".to_string(),
            percent: None,
        });

        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(2000))
                .timeout_read(std::time::Duration::from_secs(300))
                .build();

            let body = serde_json::json!({ "name": model_name, "stream": true });

            match agent
                .post("http://localhost:11434/api/pull")
                .send_json(&body)
            {
                Ok(resp) => {
                    let reader = resp.into_reader();
                    let buf_reader = std::io::BufReader::new(reader);
                    use std::io::BufRead;
                    for line in buf_reader.lines() {
                        let line = match line {
                            Ok(l) => l,
                            Err(_) => break,
                        };
                        if line.trim().is_empty() {
                            continue;
                        }
                        if let Ok(progress) = serde_json::from_str::<PullProgressLine>(&line) {
                            let percent = match (progress.total, progress.completed) {
                                (Some(total), Some(completed)) if total > 0 => {
                                    Some(completed as f64 / total as f64 * 100.0)
                                }
                                _ => None,
                            };
                            let status = progress.status.unwrap_or_default();
                            if status.contains("success") {
                                let _ = tx.send(PullStatus::Done);
                                return;
                            }
                            let _ = tx.send(PullStatus::Progress { status, percent });
                        }
                    }
                    let _ = tx.send(PullStatus::Done);
                }
                Err(e) => {
                    let _ = tx.send(PullStatus::Error(format!("Pull failed: {e}")));
                }
            }
        });
    }

    pub fn delete_model(&mut self, model_name: &str) {
        let name = model_name.to_string();
        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(200))
                .timeout_read(std::time::Duration::from_secs(10))
                .build();
            let body = serde_json::json!({ "name": name });
            let _ = agent
                .delete("http://localhost:11434/api/delete")
                .send_json(&body);
        });
        // Remove from local list immediately
        self.ollama_models.retain(|m| m.name != model_name);
        if self.model_selected > 0 && self.model_selected >= self.ollama_models.len() {
            self.model_selected = self.ollama_models.len().saturating_sub(1);
        }
        // Force re-check on next update
        self.last_api_check = None;
    }

    pub fn load_model(&self, model_name: &str) {
        let name = model_name.to_string();
        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(200))
                .timeout_read(std::time::Duration::from_secs(60))
                .build();
            let body = serde_json::json!({ "model": name, "prompt": "" });
            let _ = agent
                .post("http://localhost:11434/api/generate")
                .send_json(&body);
        });
    }

    pub fn unload_model(&self, model_name: &str) {
        let name = model_name.to_string();
        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(200))
                .timeout_read(std::time::Duration::from_secs(5))
                .build();
            let body = serde_json::json!({ "model": name, "keep_alive": 0 });
            let _ = agent
                .post("http://localhost:11434/api/generate")
                .send_json(&body);
        });
    }

    pub fn select_next(&mut self) {
        let count = self.ollama_models.len();
        if count > 0 {
            self.model_selected = (self.model_selected + 1).min(count - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.model_selected = self.model_selected.saturating_sub(1);
    }

    pub fn selected_model_name(&self) -> Option<String> {
        self.ollama_models
            .get(self.model_selected)
            .map(|m| m.name.clone())
    }

    fn poll_pull_status(&mut self) {
        if let Some(ref rx) = self.pull_receiver {
            while let Ok(status) = rx.try_recv() {
                match &status {
                    PullStatus::Done | PullStatus::Error(_) => {
                        self.pull_status = Some(status);
                        self.pull_receiver = None;
                        self.last_api_check = None; // Force refresh
                        return;
                    }
                    PullStatus::Progress { .. } => {
                        self.pull_status = Some(status);
                    }
                }
            }
        }
    }

    pub fn model_vram(&self, model_name: &str) -> Option<String> {
        self.ollama_running
            .iter()
            .find(|r| r.name == model_name)
            .map(|r| format_bytes(r.size_vram))
    }

    pub fn model_status(&self, model_name: &str) -> &str {
        if self.ollama_running.iter().any(|r| r.name == model_name) {
            "Loaded"
        } else {
            "Ready"
        }
    }
}
