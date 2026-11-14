//! Where gotodo keeps its files. Tests get a private directory per process,
//! so they never read or write the real library.

use std::path::PathBuf;

/// `~/.local/share/gotodo`: the vault, the sealed device key and the backups.
pub fn data_dir() -> PathBuf {
    #[cfg(test)]
    {
        test_root().join("data")
    }
    #[cfg(not(test))]
    {
        directories::ProjectDirs::from("org", "gotodo", "gotodo")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// `~/.config/gotodo`: preferences that must be readable while the vault is locked.
pub fn config_dir() -> PathBuf {
    #[cfg(test)]
    {
        test_root().join("config")
    }
    #[cfg(not(test))]
    {
        directories::ProjectDirs::from("org", "gotodo", "gotodo")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

pub fn vault() -> PathBuf {
    data_dir().join("vault.gotodo")
}

pub fn prefs() -> PathBuf {
    config_dir().join("prefs.json")
}

#[cfg(test)]
fn test_root() -> PathBuf {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| std::env::temp_dir().join(format!("gotodo-test-{}", std::process::id()))).clone()
}
