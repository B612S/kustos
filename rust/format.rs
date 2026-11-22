// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

/// Human-readable sizes, binary units (like Finder and Time Machine).
pub fn bytes(value: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if value < 1024 {
        return format!("{value} B");
    }
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    let digits = if size < 10.0 { 2 } else { 1 };
    format!("{size:.digits$} {}", UNITS[unit])
}

pub fn duration(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{seconds} s");
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{} min {} s", minutes, seconds % 60);
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{} h {} min", hours, minutes % 60);
    }
    format!("{} d {} h", hours / 24, hours % 24)
}

/// Speed text for the status bar, computed from bytes and elapsed time.
pub fn speed(bytes_done: u64, seconds: f64) -> String {
    if seconds <= 0.5 {
        return String::new();
    }
    // Convert the rate to an integer only after the division: casting 0.1 to
    // u64 first would round it down to zero.
    let rate = (bytes_done as f64 / seconds.max(0.1)) as u64;
    format!("{}/s", bytes(rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_use_binary_units() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(1023), "1023 B");
        assert_eq!(bytes(1024), "1.00 KB");
        assert_eq!(bytes(1536), "1.50 KB");
        assert_eq!(bytes(10 * 1024 * 1024), "10.0 MB");
    }

    #[test]
    fn speed_handles_short_elapsed_times() {
        assert_eq!(speed(0, 0.6), "0 B/s");
        assert_eq!(speed(1, 0.6), "1 B/s");
        assert_eq!(speed(1024, 0.6), "1.67 KB/s");
        assert_eq!(speed(1024, 0.4), "");
    }

    #[test]
    fn duration_is_human_readable() {
        assert_eq!(duration(42), "42 s");
        assert_eq!(duration(90), "1 min 30 s");
        assert_eq!(duration(3 * 3600 + 120), "3 h 2 min");
    }
}
