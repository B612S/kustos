// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Tiny C++ bridge for the two things Rust cannot reach through
//! qmetaobject: installing a QTranslator and using the Qt clipboard.

fn main() {
    println!("cargo:rerun-if-changed=src/bridge.cpp");
    let qt = pkg_config::Config::new()
        .probe("Qt6Core Qt6Gui Qt6Qml Qt6Quick")
        .expect("Qt 6 development files are required");
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .file("src/bridge.cpp")
        .warnings(false);
    for path in &qt.include_paths {
        build.include(path);
    }
    build.compile("kustos_bridge");
}
