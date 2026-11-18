// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Translations are Qt `.qm` files embedded as resources (`:/i18n/<lang>.qm`).
//! A `QTranslator` is installed through the small C++ bridge, so QML keeps
//! using plain `qsTr()` and re-evaluates its bindings when the language
//! changes.

use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn kustos_install_translator(path: *const c_char) -> i32;
    fn kustos_set_clipboard(text: *const c_char);
    fn kustos_quit();
}

pub fn install(language: &str) -> bool {
    let Ok(path) = CString::new(format!(":/i18n/{language}.qm")) else {
        return false;
    };
    unsafe { kustos_install_translator(path.as_ptr()) != 0 }
}

pub fn set_clipboard(text: &str) {
    let Ok(text) = CString::new(text) else { return };
    unsafe { kustos_set_clipboard(text.as_ptr()) };
}

pub fn quit() {
    unsafe { kustos_quit() };
}
