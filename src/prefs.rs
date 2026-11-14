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
}

impl Default for Prefs {
    fn default() -> Self {
        Self { appearance: Appearance::Auto, text_scale: 1.0, reduce_motion: false, lock_needs_secret: false, device_paused: false }
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
