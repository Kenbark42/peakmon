use super::history::History;
use super::process::ProcessInfo;
use crate::util::{contains_ignore_ascii_case, format_bytes};
use serde::Deserialize;
use std::collections::HashMap;
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
    error: Option<String>,
}

// --- Chat Types ---

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, PartialEq)]
pub enum ChatStatus {
    Idle,
    Generating,
    Done,
    Error(String),
}

#[derive(Clone)]
pub struct ChatMetrics {
    pub tokens_per_sec: f64,
    pub ttft_ms: f64,
    pub prompt_tokens: u64,
    pub gen_tokens: u64,
    pub total_duration_ms: f64,
    pub load_duration_ms: f64,
}

pub enum ChatToken {
    Token(String),
    FirstToken(String, f64), // token text, TTFT in ms
    Done(ChatMetrics),
    Error(String),
}

// --- Chat streaming response types ---

#[derive(Deserialize)]
struct ChatResponseLine {
    message: Option<ChatResponseMessage>,
    done: Option<bool>,
    eval_count: Option<u64>,
    eval_duration: Option<u64>,
    prompt_eval_count: Option<u64>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

// --- Search Types ---

#[derive(Clone)]
pub struct SearchResult {
    pub name: String,
    pub description: String,
    pub sizes: Vec<String>,
    pub pulls: String,
}

pub enum SearchStatus {
    Results(Vec<SearchResult>),
    Error(String),
}

/// Parse model names, descriptions, sizes, and pull counts from ollama.com/search HTML.
fn parse_search_html(html: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let bytes = html.as_bytes();

    // Each model card starts with <li x-test-model ...>
    let li_needle = b"x-test-model";
    let mut pos = 0;

    while pos < bytes.len() {
        let Some(li_start) = find_bytes(bytes, li_needle, pos) else {
            break;
        };
        // Find the end of this <li> block (next <li x-test-model or end of file)
        let block_end =
            find_bytes(bytes, li_needle, li_start + li_needle.len()).unwrap_or(bytes.len());

        let block = &html[li_start..block_end];
        let block_bytes = block.as_bytes();
        pos = block_end;

        // Extract model name from href="/library/<name>"
        let href_needle = b"href=\"/library/";
        let Some(href_start) = find_bytes(block_bytes, href_needle, 0) else {
            continue;
        };
        let name_start = href_start + href_needle.len();
        let Some(quote_end) = find_byte(block_bytes, b'"', name_start) else {
            continue;
        };
        let name = &block[name_start..quote_end];
        if name.contains('/') || name.is_empty() {
            continue;
        }

        // Avoid duplicates
        if results.iter().any(|r: &SearchResult| r.name == name) {
            continue;
        }

        // Extract description from <p class="max-w-lg ...">...</p>
        let desc_needle = b"max-w-lg";
        let description = if let Some(desc_start) = find_bytes(block_bytes, desc_needle, 0) {
            if let Some(gt) = find_byte(block_bytes, b'>', desc_start) {
                if let Some(p_end) = find_bytes(block_bytes, b"</p>", gt + 1) {
                    decode_html_entities(strip_html_tags(&block[gt + 1..p_end]).trim())
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Extract parameter sizes from <span x-test-size ...>8b</span>
        let size_needle = b"x-test-size";
        let mut sizes = Vec::new();
        let mut spos = 0;
        while let Some(s_start) = find_bytes(block_bytes, size_needle, spos) {
            if let Some(gt) = find_byte(block_bytes, b'>', s_start) {
                if let Some(s_end) = find_bytes(block_bytes, b"</span>", gt + 1) {
                    let size_text = block[gt + 1..s_end].trim().to_string();
                    if !size_text.is_empty() {
                        sizes.push(size_text);
                    }
                    spos = s_end;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Extract pull count from <span x-test-pull-count>110.4M</span>
        let pull_needle = b"x-test-pull-count";
        let pulls = if let Some(p_start) = find_bytes(block_bytes, pull_needle, 0) {
            if let Some(gt) = find_byte(block_bytes, b'>', p_start) {
                if let Some(p_end) = find_bytes(block_bytes, b"</span>", gt + 1) {
                    block[gt + 1..p_end].trim().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        results.push(SearchResult {
            name: name.to_string(),
            description,
            sizes,
            pulls,
        });
    }

    results
}

fn find_bytes(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start + needle.len() > haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + start)
}

fn find_byte(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    haystack[start..]
        .iter()
        .position(|&b| b == needle)
        .map(|p| p + start)
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(ch);
        }
    }
    out
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
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
    pub pull_model_name: Option<String>,
    pull_receiver: Option<mpsc::Receiver<PullStatus>>,
    last_api_check: Option<Instant>,
    api_cache_secs: u64,

    // Chat state
    pub chat_messages: Vec<ChatMessage>,
    pub chat_status: ChatStatus,
    pub chat_metrics: Option<ChatMetrics>,
    pub chat_model: Option<String>,
    pub tps_history: History,
    pub last_tps: HashMap<String, f64>,
    chat_receiver: Option<mpsc::Receiver<ChatToken>>,
    pub chat_scroll: usize,

    // Search state
    pub search_results: Vec<SearchResult>,
    pub search_status: Option<String>,
    pub search_selected: usize,
    pub show_search: bool,
    search_receiver: Option<mpsc::Receiver<SearchStatus>>,
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
            pull_model_name: None,
            pull_receiver: None,
            last_api_check: None,
            api_cache_secs: 5,

            chat_messages: Vec::new(),
            chat_status: ChatStatus::Idle,
            chat_metrics: None,
            chat_model: None,
            tps_history: History::new(),
            last_tps: HashMap::new(),
            chat_receiver: None,
            chat_scroll: 0,

            search_results: Vec::new(),
            search_status: None,
            search_selected: 0,
            show_search: false,
            search_receiver: None,
        }
    }

    pub fn update(&mut self, processes: &[ProcessInfo]) {
        self.detect_services(processes);
        self.filter_ai_processes(processes);
        self.poll_pull_status();
        self.poll_chat();
        self.poll_search();

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
        // Resize services vec to match patterns (reuse allocation)
        self.services
            .resize_with(SERVICE_PATTERNS.len(), || AiService {
                name: "",
                detected: false,
                version: None,
                pid: None,
            });

        for (i, sp) in SERVICE_PATTERNS.iter().enumerate() {
            // Patterns are already lowercase, so use case-insensitive search
            let matched = processes.iter().find(|p| {
                sp.process_patterns
                    .iter()
                    .any(|pat| contains_ignore_ascii_case(&p.name, pat))
            });

            self.services[i].name = sp.name;
            self.services[i].detected = matched.is_some();
            self.services[i].version = if sp.name == "Ollama" {
                self.ollama_version.clone()
            } else {
                None
            };
            self.services[i].pid = matched.map(|p| p.pid);
        }

        self.ollama_available = self
            .services
            .iter()
            .any(|s| s.name == "Ollama" && s.detected);
    }

    fn filter_ai_processes(&mut self, processes: &[ProcessInfo]) {
        self.ai_processes.clear();
        self.ai_processes.extend(
            processes
                .iter()
                .filter(|p| {
                    AI_PROCESS_PATTERNS
                        .iter()
                        .any(|pat| contains_ignore_ascii_case(&p.name, pat))
                })
                .cloned(),
        );

        self.ai_processes.sort_unstable_by(|a, b| {
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
        self.pull_model_name = Some(model_name.clone());
        self.pull_status = Some(PullStatus::Progress {
            status: "Starting pull...".to_string(),
            percent: None,
        });

        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(5000))
                .timeout_read(std::time::Duration::from_secs(600))
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
                    let mut got_any_status = false;
                    for line in buf_reader.lines() {
                        let line = match line {
                            Ok(l) => l,
                            Err(e) => {
                                let _ = tx.send(PullStatus::Error(format!("Read error: {e}")));
                                return;
                            }
                        };
                        if line.trim().is_empty() {
                            continue;
                        }
                        if let Ok(progress) = serde_json::from_str::<PullProgressLine>(&line) {
                            // Check for error in response
                            if let Some(err) = progress.error {
                                let _ = tx.send(PullStatus::Error(err));
                                return;
                            }
                            let percent = match (progress.total, progress.completed) {
                                (Some(total), Some(completed)) if total > 0 => {
                                    Some(completed as f64 / total as f64 * 100.0)
                                }
                                _ => None,
                            };
                            let status = progress.status.unwrap_or_default();
                            got_any_status = true;
                            if status.contains("success") {
                                let _ = tx.send(PullStatus::Done);
                                return;
                            }
                            let _ = tx.send(PullStatus::Progress { status, percent });
                        }
                    }
                    if got_any_status {
                        let _ = tx.send(PullStatus::Done);
                    } else {
                        let _ = tx.send(PullStatus::Error(
                            "No response from Ollama — is the model name valid?".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    // ureq returns non-2xx as errors; extract body if available
                    let msg = match e {
                        ureq::Error::Status(code, resp) => {
                            let body = resp.into_string().unwrap_or_default();
                            if let Ok(parsed) = serde_json::from_str::<PullProgressLine>(&body) {
                                if let Some(err) = parsed.error {
                                    format!("Pull failed ({code}): {err}")
                                } else {
                                    format!("Pull failed: HTTP {code}")
                                }
                            } else {
                                format!("Pull failed: HTTP {code}")
                            }
                        }
                        other => format!("Pull failed: {other}"),
                    };
                    let _ = tx.send(PullStatus::Error(msg));
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

    // --- Chat ---

    pub fn start_chat(&mut self, model: &str, messages: &[ChatMessage]) {
        let (tx, rx) = mpsc::channel();
        self.chat_receiver = Some(rx);
        self.chat_status = ChatStatus::Generating;
        self.chat_model = Some(model.to_string());

        let model = model.to_string();
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(2000))
                .timeout_read(std::time::Duration::from_secs(300))
                .build();

            let body = serde_json::json!({
                "model": model,
                "messages": msgs,
                "stream": true,
            });

            let start_time = Instant::now();
            let mut first_token = true;

            match agent
                .post("http://localhost:11434/api/chat")
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

                        let parsed: ChatResponseLine = match serde_json::from_str(&line) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };

                        let done = parsed.done.unwrap_or(false);

                        if done {
                            // Final chunk with metrics
                            let eval_count = parsed.eval_count.unwrap_or(0);
                            let eval_duration = parsed.eval_duration.unwrap_or(1);
                            let tps = eval_count as f64 / eval_duration as f64 * 1e9;
                            let total_dur = parsed.total_duration.unwrap_or(0) as f64 / 1_000_000.0;
                            let load_dur = parsed.load_duration.unwrap_or(0) as f64 / 1_000_000.0;
                            let ttft = if first_token {
                                start_time.elapsed().as_secs_f64() * 1000.0
                            } else {
                                0.0
                            };

                            let metrics = ChatMetrics {
                                tokens_per_sec: tps,
                                ttft_ms: ttft,
                                prompt_tokens: parsed.prompt_eval_count.unwrap_or(0),
                                gen_tokens: eval_count,
                                total_duration_ms: total_dur,
                                load_duration_ms: load_dur,
                            };
                            let _ = tx.send(ChatToken::Done(metrics));
                            return;
                        }

                        // Streaming token
                        if let Some(msg) = parsed.message {
                            if let Some(content) = msg.content {
                                if !content.is_empty() {
                                    if first_token {
                                        let ttft = start_time.elapsed().as_secs_f64() * 1000.0;
                                        first_token = false;
                                        let _ = tx.send(ChatToken::FirstToken(content, ttft));
                                    } else {
                                        let _ = tx.send(ChatToken::Token(content));
                                    }
                                }
                            }
                        }
                    }

                    // Stream ended without explicit done
                    if first_token {
                        let _ = tx.send(ChatToken::Error("No response received".to_string()));
                    }
                }
                Err(e) => {
                    let _ = tx.send(ChatToken::Error(format!("Chat failed: {e}")));
                }
            }
        });
    }

    fn poll_chat(&mut self) {
        if self.chat_receiver.is_none() {
            return;
        }

        let rx = self.chat_receiver.as_ref().unwrap();
        loop {
            match rx.try_recv() {
                Ok(ChatToken::Token(text)) => {
                    if let Some(last) = self.chat_messages.last_mut() {
                        if last.role == "assistant" {
                            last.content.push_str(&text);
                        }
                    }
                }
                Ok(ChatToken::FirstToken(text, ttft)) => {
                    // Store TTFT for metrics that will be finalized in Done
                    if let Some(last) = self.chat_messages.last_mut() {
                        if last.role == "assistant" {
                            last.content.push_str(&text);
                        }
                    }
                    // Store preliminary TTFT
                    if let Some(ref mut m) = self.chat_metrics {
                        m.ttft_ms = ttft;
                    } else {
                        self.chat_metrics = Some(ChatMetrics {
                            tokens_per_sec: 0.0,
                            ttft_ms: ttft,
                            prompt_tokens: 0,
                            gen_tokens: 0,
                            total_duration_ms: 0.0,
                            load_duration_ms: 0.0,
                        });
                    }
                }
                Ok(ChatToken::Done(metrics)) => {
                    // Preserve TTFT from FirstToken if the Done metrics has 0
                    let ttft = if metrics.ttft_ms == 0.0 {
                        self.chat_metrics.as_ref().map(|m| m.ttft_ms).unwrap_or(0.0)
                    } else {
                        metrics.ttft_ms
                    };

                    let final_metrics = ChatMetrics {
                        ttft_ms: ttft,
                        ..metrics
                    };

                    if let Some(ref model) = self.chat_model {
                        self.last_tps
                            .insert(model.clone(), final_metrics.tokens_per_sec);
                    }
                    self.tps_history.push(final_metrics.tokens_per_sec);
                    self.chat_metrics = Some(final_metrics);
                    self.chat_status = ChatStatus::Done;
                    self.chat_receiver = None;
                    return;
                }
                Ok(ChatToken::Error(err)) => {
                    self.chat_status = ChatStatus::Error(err);
                    self.chat_receiver = None;
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => {
                    if self.chat_status == ChatStatus::Generating {
                        self.chat_status = ChatStatus::Done;
                    }
                    self.chat_receiver = None;
                    return;
                }
            }
        }
    }

    pub fn cancel_chat(&mut self) {
        self.chat_receiver = None;
        if self.chat_status == ChatStatus::Generating {
            self.chat_status = ChatStatus::Idle;
        }
    }

    pub fn clear_chat(&mut self) {
        self.chat_messages.clear();
        self.chat_metrics = None;
        self.chat_status = ChatStatus::Idle;
        self.chat_scroll = 0;
    }

    // --- Search ---

    pub fn start_search(&mut self, query: String) {
        let (tx, rx) = mpsc::channel();
        self.search_receiver = Some(rx);
        self.search_status = Some("Searching...".to_string());
        self.search_results.clear();
        self.search_selected = 0;
        self.show_search = true;

        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_millis(3000))
                .timeout_read(std::time::Duration::from_secs(10))
                .build();

            let encoded: String = query
                .bytes()
                .map(|b| match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                        (b as char).to_string()
                    }
                    b' ' => "+".to_string(),
                    _ => format!("%{b:02X}"),
                })
                .collect();
            let url = format!("https://ollama.com/search?q={encoded}");

            match agent.get(&url).call() {
                Ok(resp) => match resp.into_string() {
                    Ok(html) => {
                        let results = parse_search_html(&html);
                        let _ = tx.send(SearchStatus::Results(results));
                    }
                    Err(e) => {
                        let _ =
                            tx.send(SearchStatus::Error(format!("Failed to read response: {e}")));
                    }
                },
                Err(e) => {
                    let _ = tx.send(SearchStatus::Error(format!("Search failed: {e}")));
                }
            }
        });
    }

    fn poll_search(&mut self) {
        if self.search_receiver.is_none() {
            return;
        }

        let rx = self.search_receiver.as_ref().unwrap();
        match rx.try_recv() {
            Ok(SearchStatus::Results(results)) => {
                if results.is_empty() {
                    self.search_status = Some("No results found".to_string());
                } else {
                    self.search_status = None;
                }
                self.search_results = results;
                self.search_receiver = None;
            }
            Ok(SearchStatus::Error(err)) => {
                self.search_status = Some(err);
                self.search_receiver = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.search_receiver = None;
            }
        }
    }

    pub fn search_select_next(&mut self) {
        let count = self.search_results.len();
        if count > 0 {
            self.search_selected = (self.search_selected + 1).min(count - 1);
        }
    }

    pub fn search_select_prev(&mut self) {
        self.search_selected = self.search_selected.saturating_sub(1);
    }

    pub fn selected_search_model(&self) -> Option<String> {
        self.search_results.get(self.search_selected).map(|r| {
            // Append the first (smallest) size tag so the pull targets a specific variant
            if let Some(first_size) = r.sizes.first() {
                format!("{}:{}", r.name, first_size)
            } else {
                r.name.clone()
            }
        })
    }

    pub fn dismiss_search(&mut self) {
        self.show_search = false;
        self.search_results.clear();
        self.search_status = None;
        self.search_selected = 0;
    }

    pub fn has_loaded_model(&self) -> bool {
        !self.ollama_running.is_empty()
    }

    pub fn first_loaded_model_name(&self) -> Option<String> {
        // Prefer the selected model if it's loaded, otherwise first loaded
        if let Some(name) = self.selected_model_name() {
            if self.ollama_running.iter().any(|r| r.name == name) {
                return Some(name);
            }
        }
        self.ollama_running.first().map(|r| r.name.clone())
    }
}
