//! The few preferences gotodo needs before the vault is open: how the lock
//! screen looks, and whether the device key may open it. Stored unencrypted,
//! so nothing personal belongs here.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use serde::{Deserialize, Serialize};

use crate::model::Appearance;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub appearance: Appearance,
    pub text_scale: f32,
    pub reduce_motion: bool,
    /// After gotodo locks, the device key stays unused until the next
    /// password or KeePassXC unlock.
    pub lock_needs_secret: bool,
    /// Set when gotodo locks while `lock_needs_secret` is on.
    pub device_paused: bool,
    /// Where the colors come from.
    pub colors: ColorSource,
    /// The highlight color when `colors` is gotodo's own palette.
    pub accent: Accent,
    /// Opacity of the main window's content background, 0.3 to 1.0. Below 1.0
    /// a compositor with blur (SwayFX, KWin, Hyprland) shows the desktop through.
    pub page_opacity: f32,
    pub sidebar_opacity: f32,
    pub quick_opacity: f32,
    /// Closing the last window keeps gotodo running, and unlocked until
    /// auto-lock, so Quick Entry opens at once.
    pub keep_running: bool,
    /// Quick Entry takes to-dos while gotodo is locked. They wait in memory,
    /// never on disk, and are added on the next unlock.
    pub quick_while_locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorSource {
    /// gotodo's palette with the chosen accent.
    #[default]
    Gotodo,
    /// gotodo's palette with the GTK theme's accent.
    GtkAccent,
    /// Every color the GTK theme defines.
    Gtk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Accent {
    #[default]
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Green,
    Teal,
    Graphite,
}

impl Accent {
    pub const ALL: [Accent; 8] =
        [Accent::Blue, Accent::Purple, Accent::Pink, Accent::Red, Accent::Orange, Accent::Green, Accent::Teal, Accent::Graphite];
}

/// Opacity values outside this range are clamped.
pub const MIN_OPACITY: f32 = 0.3;

impl Default for Prefs {
    fn default() -> Self {
        Self {
            appearance: Appearance::Auto,
            text_scale: 1.0,
            reduce_motion: false,
            lock_needs_secret: false,
            device_paused: false,
            colors: ColorSource::Gotodo,
            accent: Accent::Blue,
            page_opacity: 1.0,
            sidebar_opacity: 1.0,
            quick_opacity: 1.0,
            keep_running: true,
            quick_while_locked: true,
        }
    }
}

impl Prefs {
    /// `None` when there is no file yet, so the caller can migrate old settings.
    pub fn load() -> Option<Self> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            let raw = fs::read(crate::paths::prefs()).ok()?;
            serde_json::from_slice(&raw).ok()
        }
    }

    pub fn save(&self) {
        #[cfg(not(test))]
        {
            let _ = write(self);
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
fn write(prefs: &Prefs) -> std::io::Result<()> {
    let path = crate::paths::prefs();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
    file.write_all(&serde_json::to_vec_pretty(prefs).expect("prefs serialize"))?;
    file.sync_all()?;
    fs::rename(tmp, path)
}
