pub mod footer;
pub mod header;
pub mod layout;
pub mod tabs;
pub mod theme;
pub mod widgets;

use ratatui::Frame;

use crate::app::App;
use tabs::Tab;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let app_layout = layout::compute_layout(area);

    // Header
    let hostname = app.hostname.as_str();
    let uptime = app.metrics.uptime();
    header::render(frame, app_layout.header, app.current_tab, hostname, uptime);

    // Body - dispatch to current tab
    match app.current_tab {
        Tab::Dashboard => tabs::dashboard::render(frame, app_layout.body, &app.metrics),
        Tab::Cpu => tabs::cpu_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Memory => tabs::memory_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Disk => tabs::disk_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Network => tabs::network_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Processes => tabs::processes::render(
            frame,
            app_layout.body,
            &app.metrics,
            app.scroll_offset,
        ),
        Tab::Logs => tabs::logs::render(
            frame,
            app_layout.body,
            &app.log_stream,
            app.scroll_offset,
        ),
        Tab::Temperatures => tabs::temperatures::render(frame, app_layout.body, &app.metrics),
    }

    // Footer
    footer::render(frame, app_layout.footer, app.current_tab, app.filter_mode);
}
