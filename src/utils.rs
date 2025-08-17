use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }

    if parts.is_empty() {
        "just now".to_string()
    } else {
        format!("{} ago", parts.join(" "))
    }
}

pub fn format_base_text(folder_name: &str, dir: &str, duplicates: &HashSet<&str>) -> String {
    if duplicates.contains(folder_name) {
        format!("       > {} ({})", folder_name, dir)
    } else {
        format!("       > {}", folder_name)
    }
}

pub fn get_folder_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
}
