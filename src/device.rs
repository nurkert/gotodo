//! Seals the "this device" unlock secret to the computer with `systemd-creds`.
//!
//! The secret is a random value that only unwraps the vault's device slot. It
//! is sealed to the TPM when one is usable, and otherwise to a per-user key
//! kept by systemd. The sealed blob sits next to the vault; without this
//! machine it cannot be opened.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use zeroize::Zeroizing;

const NAME: &str = "--name=gotodo-vault";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Tpm,
    HostKey,
}

fn file() -> PathBuf {
    crate::session::vault_path().with_file_name("device.cred")
}

fn run(args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    let mut child = Command::new("systemd-creds")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("systemd-creds: {e}"))?;
    {
        let mut stdin = child.stdin.take().expect("stdin is piped");
        stdin.write_all(input).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let message = String::from_utf8_lossy(&out.stderr).trim().to_owned();
        Err(if message.is_empty() { "systemd-creds failed".to_owned() } else { message })
    }
}

/// Seals `secret` and stores it next to the vault. TPM first, per-user key second.
pub fn seal(secret: &[u8]) -> Result<Backend, String> {
    let mut last = String::new();
    // The TPM works for a normal user in system mode; `--user` refuses it.
    let attempts: [(Backend, &[&str], &str); 2] = [
        (Backend::Tpm, &["encrypt", "--with-key=tpm2", NAME, "-", "-"], "tpm2"),
        (Backend::HostKey, &["--user", "encrypt", "--with-key=host", NAME, "-", "-"], "host"),
    ];
    for (backend, args, tag) in attempts {
        match run(args, secret) {
            Ok(blob) => {
                let mut data = format!("{tag}\n").into_bytes();
                data.extend_from_slice(&blob);
                let path = file();
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                }
                std::fs::write(&path, data).map_err(|e| e.to_string())?;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
                return Ok(backend);
            }
            Err(error) => last = error,
        }
    }
    Err(last)
}

fn read() -> Option<(Backend, Vec<u8>)> {
    let raw = std::fs::read(file()).ok()?;
    let split = raw.iter().position(|b| *b == b'\n')?;
    let backend = match &raw[..split] {
        b"tpm2" => Backend::Tpm,
        b"host" => Backend::HostKey,
        _ => return None,
    };
    Some((backend, raw[split + 1..].to_vec()))
}

#[allow(dead_code)]
pub fn backend() -> Option<Backend> {
    read().map(|(backend, _)| backend)
}

/// Opens the sealed secret again. `None` when it is missing or the machine refuses.
pub fn unseal() -> Option<Zeroizing<Vec<u8>>> {
    let (backend, blob) = read()?;
    let args: &[&str] = match backend {
        Backend::Tpm => &["decrypt", NAME, "-", "-"],
        Backend::HostKey => &["--user", "decrypt", NAME, "-", "-"],
    };
    run(args, &blob).ok().map(Zeroizing::new)
}

pub fn remove() {
    let _ = std::fs::remove_file(file());
}
