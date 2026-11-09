//! KeePassXC integration through the freedesktop Secret Service API.
//!
//! The vault's unlock secret lives in the KeePassXC database, so it is only
//! reachable while that database is open. gotodo never asks the service to
//! unlock anything: a locked KeePassXC simply means this route is unavailable.

use std::collections::HashMap;

use secret_service::EncryptionType;
use secret_service::blocking::SecretService;
use zeroize::Zeroizing;

const BUS_NAME: &str = "org.freedesktop.secrets";
const LABEL: &str = "gotodo vault key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// No Secret Service is running on the session bus.
    Missing,
    /// Something else, e.g. GNOME Keyring, owns the Secret Service name.
    OtherProvider,
    Locked,
    Unlocked,
}

fn provider_is_keepassxc() -> Option<bool> {
    use zbus::blocking::{Connection, fdo::DBusProxy};
    use zbus::names::{BusName, UniqueName};

    let connection = Connection::session().ok()?;
    let proxy = DBusProxy::new(&connection).ok()?;
    let name = BusName::try_from(BUS_NAME).ok()?;
    let owner = proxy.get_name_owner(name).ok()?;
    let owner: UniqueName<'_> = owner.into();
    let pid = proxy.get_connection_unix_process_id(BusName::from(owner)).ok()?;
    // KeePassXC marks itself non-dumpable, which hides /proc/<pid>/exe from us.
    let name = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    Some(name.trim().to_lowercase().starts_with("keepassxc"))
}

fn attributes(vault_id: &str) -> HashMap<&str, &str> {
    HashMap::from([("application", "gotodo"), ("vault", vault_id)])
}

pub fn status() -> Status {
    match provider_is_keepassxc() {
        None => return Status::Missing,
        Some(false) => return Status::OtherProvider,
        Some(true) => {}
    }
    let Ok(service) = SecretService::connect(EncryptionType::Dh) else {
        return Status::Missing;
    };
    match service.get_any_collection().and_then(|c| c.is_locked()) {
        Ok(false) => Status::Unlocked,
        _ => Status::Locked,
    }
}

/// Returns the stored secret, or `None` when KeePassXC is locked or holds no entry.
pub fn read_secret(vault_id: &str) -> Option<Zeroizing<Vec<u8>>> {
    if status() != Status::Unlocked {
        return None;
    }
    let service = SecretService::connect(EncryptionType::Dh).ok()?;
    let found = service.search_items(attributes(vault_id)).ok()?;
    let item = found.unlocked.first()?;
    item.get_secret().ok().map(Zeroizing::new)
}

pub fn store_secret(vault_id: &str, secret: &[u8]) -> Result<(), String> {
    if status() != Status::Unlocked {
        return Err("KeePassXC is not unlocked".into());
    }
    let service = SecretService::connect(EncryptionType::Dh).map_err(|e| e.to_string())?;
    let collection = service.get_any_collection().map_err(|e| e.to_string())?;
    collection
        .create_item(LABEL, attributes(vault_id), secret, true, "application/octet-stream")
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn delete_secret(vault_id: &str) -> Result<(), String> {
    let service = SecretService::connect(EncryptionType::Dh).map_err(|e| e.to_string())?;
    let found = service.search_items(attributes(vault_id)).map_err(|e| e.to_string())?;
    for item in &found.unlocked {
        item.delete().map_err(|e| e.to_string())?;
    }
    Ok(())
}
