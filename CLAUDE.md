# Peakmon - Terminal System Monitor

## Build & Run
- `cargo build` - Debug build
- `cargo run` - Run in debug mode
- `cargo build --release` - Release build (optimized, stripped)
- `cargo clippy -- -D warnings` - Lint check
- `cargo fmt -- --check` - Format check

## Architecture
- Synchronous event loop with 250ms poll timeout
- Metrics collected via `sysinfo` crate, refreshed on configurable interval
- History stored in VecDeque ring buffers (300 samples = 5 min at 1s)
- Log streaming via child process (`log stream --style=compact`)
- TUI built with `ratatui` + `crossterm` backend

## Module Layout
- `src/main.rs` - Entry point, terminal init/restore
- `src/app.rs` - App state, event loop, key handling
- `src/config.rs` - CLI args via clap
- `src/event.rs` - Crossterm event polling
- `src/util.rs` - Byte/rate/uptime formatting
- `src/metrics/` - System metrics (CPU, memory, disk, network, process, temperature, history)
- `src/logs/` - macOS log stream subprocess and parsing
- `src/ui/` - TUI rendering (theme, layout, header, footer, tab dispatch)
- `src/ui/tabs/` - Individual tab views (dashboard, cpu, memory, disk, network, processes, logs, temps)
- `src/ui/widgets/` - Reusable composite widgets (sparkline_panel, metric_gauge, sortable_table)

## Conventions
- Edition 2021, targets macOS (Apple Silicon)
- Catppuccin Mocha color palette for the TUI theme
- Tab navigation: 1-8 keys, Tab/Shift-Tab, F1-F8
- Keep `cargo clippy -- -D warnings` clean
