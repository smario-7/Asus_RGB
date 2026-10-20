//! The last applied look (`~/.config/a-rgb/last`) — for `restore` and the relative commands.

use crate::model::Settings;
use std::{env, fs, path::PathBuf};

fn path() -> Option<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(x) if !x.is_empty() => PathBuf::from(x),
        _ => PathBuf::from(env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("a-rgb").join("last"))
}

pub fn load() -> Option<Settings> {
    Settings::deserialize(&fs::read_to_string(path()?).ok()?).ok()
}

pub fn save(s: &Settings) -> Result<(), String> {
    let p = path().ok_or("cannot determine the configuration directory (HOME is not set)")?;
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&p, s.serialize()).map_err(|e| format!("{}: {e}", p.display()))
}
