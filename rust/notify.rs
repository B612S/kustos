// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Desktop notifications through `notify-send`. Missing tools are not an
//! error: a backup must never fail because a notification could not be shown.

pub fn send(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "Kustos", "-i", "kustos", title, body])
        .spawn();
}
