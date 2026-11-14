//! Daily copies of the vault file, kept next to it in `backups/`.
//!
//! A copy is the encrypted file exactly as it was before the first write of
//! a day, so it opens with whatever unlocked the vault on that day. Nothing
//! is ever decrypted here. Changing the password removes the old copies,
//! because they would still open with the old one.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

/// How many daily copies stay around.
pub const KEEP: usize = 14;
const PREFIX: &str = "vault-";
const SUFFIX: &str = ".gotodo";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    pub date: NaiveDate,
    pub path: PathBuf,
}

pub fn dir_for(vault: &Path) -> PathBuf {
    vault.with_file_name("backups")
}

fn copy_private(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    let data = fs::read(from)?;
    let tmp = to.with_extension("tmp");
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
    file.write_all(&data)?;
    file.sync_all()?;
    fs::rename(tmp, to)
}

/// Saves today's copy unless there is one. Call it before writing the vault.
pub fn daily(vault: &Path, today: NaiveDate) -> std::io::Result<()> {
    if !vault.exists() {
        return Ok(());
    }
    let target = dir_for(vault).join(format!("{PREFIX}{today}{SUFFIX}"));
    if target.exists() {
        return Ok(());
    }
    copy_private(vault, &target)?;
    prune(vault)
}

/// Newest first.
pub fn list(vault: &Path) -> Vec<Backup> {
    let Ok(entries) = fs::read_dir(dir_for(vault)) else { return Vec::new() };
    let mut found: Vec<Backup> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let date = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
            let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
            Some(Backup { date, path: entry.path() })
        })
        .collect();
    found.sort_by(|a, b| b.date.cmp(&a.date));
    found
}

fn prune(vault: &Path) -> std::io::Result<()> {
    for old in list(vault).into_iter().skip(KEEP) {
        fs::remove_file(old.path)?;
    }
    Ok(())
}

/// Drops every daily copy, then keeps the current file as today's.
pub fn restart(vault: &Path, today: NaiveDate) -> std::io::Result<()> {
    for old in list(vault) {
        fs::remove_file(old.path)?;
    }
    daily(vault, today)
}

/// Puts `backup` in place of the vault. The current file is kept beside the
/// daily copies first, so a restore can itself be undone. The vault must be closed.
pub fn restore(vault: &Path, backup: &Path, now: chrono::NaiveDateTime) -> std::io::Result<PathBuf> {
    let kept = dir_for(vault).join(format!("before-restore-{}{SUFFIX}", now.format("%Y-%m-%d-%H%M%S")));
    if vault.exists() {
        copy_private(vault, &kept)?;
    }
    copy_private(backup, vault)?;
    Ok(kept)
}

/// Writes a copy of the encrypted vault into `dir` and returns its path.
pub fn export(vault: &Path, dir: &Path, today: NaiveDate) -> std::io::Result<PathBuf> {
    let mut target = dir.join(format!("gotodo-{today}{SUFFIX}"));
    let mut n = 2;
    while target.exists() {
        target = dir.join(format!("gotodo-{today}-{n}{SUFFIX}"));
        n += 1;
    }
    copy_private(vault, &target)?;
    Ok(target)
}

/// Where exports go: the Documents folder, or home when there is none.
pub fn export_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.document_dir().map(Path::to_path_buf).or_else(|| Some(dirs.home_dir().to_path_buf())))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(n: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, n).unwrap()
    }

    #[test]
    fn one_copy_per_day_and_old_ones_go() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.gotodo");
        fs::write(&vault, b"first").unwrap();
        daily(&vault, day(1)).unwrap();
        fs::write(&vault, b"second").unwrap();
        daily(&vault, day(1)).unwrap();
        assert_eq!(fs::read(&list(&vault)[0].path).unwrap(), b"first", "the day's copy is the state before its first write");

        for n in 2..=20 {
            daily(&vault, day(n)).unwrap();
        }
        let kept = list(&vault);
        assert_eq!(kept.len(), KEEP);
        assert_eq!(kept[0].date, day(20));
        let mode = fs::metadata(&kept[0].path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn restore_keeps_the_replaced_file() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.gotodo");
        fs::write(&vault, b"old").unwrap();
        daily(&vault, day(1)).unwrap();
        fs::write(&vault, b"new").unwrap();
        let backup = list(&vault)[0].path.clone();
        let kept = restore(&vault, &backup, day(2).and_hms_opt(9, 0, 0).unwrap()).unwrap();
        assert_eq!(fs::read(&vault).unwrap(), b"old");
        assert_eq!(fs::read(kept).unwrap(), b"new");
    }

    #[test]
    fn restart_forgets_copies_of_the_old_password() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.gotodo");
        fs::write(&vault, b"a").unwrap();
        daily(&vault, day(1)).unwrap();
        daily(&vault, day(2)).unwrap();
        fs::write(&vault, b"b").unwrap();
        restart(&vault, day(3)).unwrap();
        let kept = list(&vault);
        assert_eq!(kept.len(), 1);
        assert_eq!(fs::read(&kept[0].path).unwrap(), b"b");
    }
}
