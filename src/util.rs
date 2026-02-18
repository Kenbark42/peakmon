/// Format bytes into a human-readable string (e.g., "1.23 GiB").
pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;

    let b = bytes as f64;
    if b >= TIB {
        format!("{:.2} TiB", b / TIB)
    } else if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.2} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KiB", b / KIB)
    } else {
        format!("{bytes} B")
    }
}

/// Format bytes-per-second into a rate string (e.g., "1.23 MiB/s").
pub fn format_rate(bytes_per_sec: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let b = bytes_per_sec.abs();
    if b >= GIB {
        format!("{:.2} GiB/s", bytes_per_sec / GIB)
    } else if b >= MIB {
        format!("{:.2} MiB/s", bytes_per_sec / MIB)
    } else if b >= KIB {
        format!("{:.1} KiB/s", bytes_per_sec / KIB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

/// Format a duration in seconds into "Xd Xh Xm" or "Xh Xm Xs".
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        let s = secs % 60;
        format!("{mins}m {s}s")
    }
}

/// Format a percentage with one decimal (e.g., "45.2%").
pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", value)
}
