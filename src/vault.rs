//! Encrypted on-disk storage.
//!
//! The whole library lives in one file. A random data key encrypts the body
//! with XChaCha20-Poly1305. That key is wrapped once per unlock method
//! ("slot"): Argon2id for the password, HKDF for the KeePassXC secret. The
//! header is authenticated as associated data of the body, so slots or
//! parameters cannot be altered without detection after unlock.
//!
//! Layout: `MAGIC | u32 LE header length | header JSON | 24-byte nonce | ciphertext`

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

const MAGIC: &[u8; 8] = b"GOTODO\x00\x01";
const MAX_HEADER: usize = 64 * 1024;
const PAD_BLOCK: usize = 4096;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;

pub const MIN_PASSWORD_CHARS: usize = 10;
pub const KEEPASS_SECRET_LEN: usize = 32;

/// Argon2id cost for new vaults: above the RFC 9106 low-memory profile.
const DEFAULT_KDF: KdfParams = KdfParams { m_kib: 128 * 1024, t: 3, p: 1 };

#[derive(Debug)]
pub enum VaultError {
    WrongSecret,
    Corrupt,
    NoKeePassSlot,
    NoDeviceSlot,
    LastSlot,
    AlreadyOpen,
    WeakPassword,
    Io(std::io::Error),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongSecret => write!(f, "wrong password or key"),
            Self::Corrupt => write!(f, "the vault file is damaged or was tampered with"),
            Self::NoKeePassSlot => write!(f, "KeePassXC unlock is not set up for this vault"),
            Self::NoDeviceSlot => write!(f, "device unlock is not set up for this vault"),
            Self::LastSlot => write!(f, "this is the only way to open the vault; set a password first"),
            Self::AlreadyOpen => write!(f, "the vault is already open in another process"),
            Self::WeakPassword => {
                write!(f, "the password needs at least {MIN_PASSWORD_CHARS} characters")
            }
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// Bounds on values read from an unauthenticated header.
    fn is_sane(&self) -> bool {
        (8 * 1024..=4 * 1024 * 1024).contains(&self.m_kib)
            && (1..=16).contains(&self.t)
            && (1..=16).contains(&self.p)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SlotKind {
    Password { salt: String, kdf: KdfParams },
    Keepass,
    /// A random secret sealed to this machine (TPM or device key).
    Device,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Slot {
    #[serde(flatten)]
    kind: SlotKind,
    nonce: String,
    wrapped: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Header {
    vault_id: String,
    slots: Vec<Slot>,
}

pub enum Secret {
    Password(Zeroizing<String>),
    KeePass(Zeroizing<Vec<u8>>),
    Device(Zeroizing<Vec<u8>>),
}

type Key = Zeroizing<[u8; KEY_LEN]>;

pub struct Vault {
    path: PathBuf,
    header: Header,
    header_bytes: Vec<u8>,
    key: Box<Key>,
    kdf: KdfParams,
    _lock: File,
}

impl Drop for Vault {
    fn drop(&mut self) {
        unlock_memory(self.key.as_ref());
    }
}

fn lock_memory(key: &Key) {
    // Best effort: keeps the data key out of swap when RLIMIT_MEMLOCK allows.
    unsafe {
        libc::mlock(key.as_ptr().cast(), KEY_LEN);
    }
}

fn unlock_memory(key: &Key) {
    unsafe {
        libc::munlock(key.as_ptr().cast(), KEY_LEN);
    }
}

fn random<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    getrandom::fill(&mut out).expect("operating system random source failed");
    out
}

fn new_key() -> Box<Key> {
    let key = Box::new(Zeroizing::new(random::<KEY_LEN>()));
    lock_memory(&key);
    key
}

pub fn new_keepass_secret() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(random::<KEEPASS_SECRET_LEN>().to_vec())
}

pub fn new_device_secret() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(random::<KEEPASS_SECRET_LEN>().to_vec())
}

fn decode(s: &str) -> Result<Vec<u8>, VaultError> {
    B64.decode(s).map_err(|_| VaultError::Corrupt)
}

fn derive_password(password: &str, salt: &[u8], kdf: KdfParams) -> Result<Key, VaultError> {
    if !kdf.is_sane() || salt.len() != 16 {
        return Err(VaultError::Corrupt);
    }
    let params =
        Params::new(kdf.m_kib, kdf.t, kdf.p, Some(KEY_LEN)).map_err(|_| VaultError::Corrupt)?;
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, out.as_mut())
        .map_err(|_| VaultError::Corrupt)?;
    Ok(out)
}

fn derive_with(secret: &[u8], vault_id: &str, info: &[u8]) -> Result<Key, VaultError> {
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    Hkdf::<Sha256>::new(Some(vault_id.as_bytes()), secret)
        .expand(info, out.as_mut())
        .map_err(|_| VaultError::Corrupt)?;
    Ok(out)
}

fn derive_keepass(secret: &[u8], vault_id: &str) -> Result<Key, VaultError> {
    derive_with(secret, vault_id, b"gotodo keepass slot v1")
}

fn derive_device(secret: &[u8], vault_id: &str) -> Result<Key, VaultError> {
    derive_with(secret, vault_id, b"gotodo device slot v1")
}

fn slot_aad(vault_id: &str, kind: &SlotKind) -> Vec<u8> {
    let mut aad = b"gotodo slot v1\0".to_vec();
    aad.extend_from_slice(vault_id.as_bytes());
    aad.push(0);
    aad.extend_from_slice(&serde_json::to_vec(kind).expect("slot kind serializes"));
    aad
}

fn wrap(kek: &Key, vault_id: &str, kind: SlotKind, data_key: &Key) -> Slot {
    let nonce = random::<NONCE_LEN>();
    let aad = slot_aad(vault_id, &kind);
    let wrapped = XChaCha20Poly1305::new(kek.as_ref().into())
        .encrypt(XNonce::from_slice(&nonce), Payload { msg: data_key.as_ref(), aad: &aad })
        .expect("encrypting a fixed-size key cannot fail");
    Slot { kind, nonce: B64.encode(nonce), wrapped: B64.encode(wrapped) }
}

fn unwrap(kek: &Key, vault_id: &str, slot: &Slot) -> Result<Box<Key>, VaultError> {
    let nonce = decode(&slot.nonce)?;
    if nonce.len() != NONCE_LEN {
        return Err(VaultError::Corrupt);
    }
    let wrapped = decode(&slot.wrapped)?;
    let aad = slot_aad(vault_id, &slot.kind);
    let plain = Zeroizing::new(
        XChaCha20Poly1305::new(kek.as_ref().into())
            .decrypt(XNonce::from_slice(&nonce), Payload { msg: &wrapped, aad: &aad })
            .map_err(|_| VaultError::WrongSecret)?,
    );
    if plain.len() != KEY_LEN {
        return Err(VaultError::Corrupt);
    }
    let mut key = new_key();
    key.copy_from_slice(&plain);
    Ok(key)
}

fn password_slot(password: &str, vault_id: &str, kdf: KdfParams, key: &Key) -> Result<Slot, VaultError> {
    let salt = random::<16>();
    let kek = derive_password(password, &salt, kdf)?;
    Ok(wrap(&kek, vault_id, SlotKind::Password { salt: B64.encode(salt), kdf }, key))
}

fn keepass_slot(secret: &[u8], vault_id: &str, key: &Key) -> Result<Slot, VaultError> {
    let kek = derive_keepass(secret, vault_id)?;
    Ok(wrap(&kek, vault_id, SlotKind::Keepass, key))
}

fn device_slot(secret: &[u8], vault_id: &str, key: &Key) -> Result<Slot, VaultError> {
    let kek = derive_device(secret, vault_id)?;
    Ok(wrap(&kek, vault_id, SlotKind::Device, key))
}

fn encode_header(header: &Header) -> Vec<u8> {
    serde_json::to_vec(header).expect("header serializes")
}

fn body_aad(header_bytes: &[u8]) -> Vec<u8> {
    let mut aad = MAGIC.to_vec();
    aad.extend_from_slice(header_bytes);
    aad
}

fn pad(plain: &[u8]) -> Zeroizing<Vec<u8>> {
    let total = (plain.len() + 8).div_ceil(PAD_BLOCK).max(1) * PAD_BLOCK;
    let mut out = Zeroizing::new(Vec::with_capacity(total));
    out.extend_from_slice(&(plain.len() as u64).to_le_bytes());
    out.extend_from_slice(plain);
    out.resize(total, 0);
    out
}

fn unpad(padded: &[u8]) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    let (len, rest) = padded.split_first_chunk::<8>().ok_or(VaultError::Corrupt)?;
    let len = usize::try_from(u64::from_le_bytes(*len)).map_err(|_| VaultError::Corrupt)?;
    if len > rest.len() {
        return Err(VaultError::Corrupt);
    }
    Ok(Zeroizing::new(rest[..len].to_vec()))
}

fn acquire_lock(path: &Path) -> Result<File, VaultError> {
    let lock_path = path.with_extension("lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(0o600)
        .open(lock_path)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(fs::TryLockError::WouldBlock) => Err(VaultError::AlreadyOpen),
        Err(fs::TryLockError::Error(e)) => Err(e.into()),
    }
}

fn ensure_private_dir(path: &Path) -> Result<(), VaultError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Replaces `path` so a crash leaves either the old or the new file, never a partial one.
fn write_atomic(path: &Path, data: &[u8]) -> Result<(), VaultError> {
    let tmp = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp, path)?;
    if let Some(dir) = path.parent() {
        File::open(dir)?.sync_all()?;
    }
    Ok(())
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Reads the vault id and whether KeePassXC unlock is configured, without unlocking.
pub fn peek(path: &Path) -> Result<(String, bool), VaultError> {
    let raw = fs::read(path)?;
    let (header, _, _) = split(&raw)?;
    let has_keepass = header.slots.iter().any(|s| s.kind == SlotKind::Keepass);
    Ok((header.vault_id, has_keepass))
}

/// Which unlock methods a vault offers, without unlocking it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Slots {
    pub password: bool,
    pub keepass: bool,
    pub device: bool,
}

pub fn peek_slots(path: &Path) -> Result<Slots, VaultError> {
    let raw = fs::read(path)?;
    let (header, _, _) = split(&raw)?;
    let mut slots = Slots::default();
    for slot in &header.slots {
        match slot.kind {
            SlotKind::Password { .. } => slots.password = true,
            SlotKind::Keepass => slots.keepass = true,
            SlotKind::Device => slots.device = true,
        }
    }
    Ok(slots)
}

fn split(raw: &[u8]) -> Result<(Header, &[u8], &[u8]), VaultError> {
    let rest = raw.strip_prefix(MAGIC.as_slice()).ok_or(VaultError::Corrupt)?;
    let (len, rest) = rest.split_first_chunk::<4>().ok_or(VaultError::Corrupt)?;
    let len = u32::from_le_bytes(*len) as usize;
    if len > MAX_HEADER || len > rest.len() {
        return Err(VaultError::Corrupt);
    }
    let (header_bytes, body) = rest.split_at(len);
    let header = serde_json::from_slice(header_bytes).map_err(|_| VaultError::Corrupt)?;
    Ok((header, header_bytes, body))
}

impl Vault {
    pub fn create(path: &Path, password: &str, plain: &[u8]) -> Result<Self, VaultError> {
        Self::create_with(path, password, plain, DEFAULT_KDF)
    }

    #[cfg(test)]
    pub fn create_for_tests(path: &Path, password: &str, plain: &[u8]) -> Result<Self, VaultError> {
        Self::create_with(path, password, plain, KdfParams { m_kib: 8 * 1024, t: 1, p: 1 })
    }

    fn create_with(
        path: &Path,
        password: &str,
        plain: &[u8],
        kdf: KdfParams,
    ) -> Result<Self, VaultError> {
        if password.chars().count() < MIN_PASSWORD_CHARS {
            return Err(VaultError::WeakPassword);
        }
        ensure_private_dir(path)?;
        let lock = acquire_lock(path)?;
        let vault_id = uuid::Uuid::new_v4().to_string();
        let key = new_key();
        let slot = password_slot(password, &vault_id, kdf, &key)?;
        let header = Header { vault_id, slots: vec![slot] };
        let header_bytes = encode_header(&header);
        let vault = Self { path: path.to_owned(), header, header_bytes, key, kdf, _lock: lock };
        vault.save(plain)?;
        Ok(vault)
    }

    /// A new vault that opens with a device-sealed secret and no password.
    pub fn create_device(path: &Path, secret: &[u8], plain: &[u8]) -> Result<Self, VaultError> {
        ensure_private_dir(path)?;
        let lock = acquire_lock(path)?;
        let vault_id = uuid::Uuid::new_v4().to_string();
        let key = new_key();
        let slot = device_slot(secret, &vault_id, &key)?;
        let header = Header { vault_id, slots: vec![slot] };
        let header_bytes = encode_header(&header);
        let vault = Self { path: path.to_owned(), header, header_bytes, key, kdf: DEFAULT_KDF, _lock: lock };
        vault.save(plain)?;
        Ok(vault)
    }

    pub fn open(path: &Path, secret: &Secret) -> Result<(Self, Zeroizing<Vec<u8>>), VaultError> {
        ensure_private_dir(path)?;
        let lock = acquire_lock(path)?;
        let raw = fs::read(path)?;
        let (header, header_bytes, body) = split(&raw)?;
        let (key, kdf) = match secret {
            Secret::Password(password) => {
                let (slot, salt, kdf) = header
                    .slots
                    .iter()
                    .find_map(|s| match &s.kind {
                        SlotKind::Password { salt, kdf } => Some((s, salt, *kdf)),
                        SlotKind::Keepass | SlotKind::Device => None,
                    })
                    .ok_or(VaultError::Corrupt)?;
                let kek = derive_password(password, &decode(salt)?, kdf)?;
                (unwrap(&kek, &header.vault_id, slot)?, kdf)
            }
            Secret::KeePass(value) => {
                let slot = header
                    .slots
                    .iter()
                    .find(|s| s.kind == SlotKind::Keepass)
                    .ok_or(VaultError::NoKeePassSlot)?;
                let kek = derive_keepass(value, &header.vault_id)?;
                let kdf = header
                    .slots
                    .iter()
                    .find_map(|s| match s.kind {
                        SlotKind::Password { kdf, .. } => Some(kdf),
                        SlotKind::Keepass | SlotKind::Device => None,
                    })
                    .unwrap_or(DEFAULT_KDF);
                (unwrap(&kek, &header.vault_id, slot)?, kdf)
            }
            Secret::Device(value) => {
                let slot = header
                    .slots
                    .iter()
                    .find(|s| s.kind == SlotKind::Device)
                    .ok_or(VaultError::NoDeviceSlot)?;
                let kek = derive_device(value, &header.vault_id)?;
                let kdf = header
                    .slots
                    .iter()
                    .find_map(|s| match s.kind {
                        SlotKind::Password { kdf, .. } => Some(kdf),
                        SlotKind::Keepass | SlotKind::Device => None,
                    })
                    .unwrap_or(DEFAULT_KDF);
                (unwrap(&kek, &header.vault_id, slot)?, kdf)
            }
        };
        if body.len() < NONCE_LEN {
            return Err(VaultError::Corrupt);
        }
        let (nonce, ciphertext) = body.split_at(NONCE_LEN);
        let aad = body_aad(header_bytes);
        let padded = Zeroizing::new(
            XChaCha20Poly1305::new(key.as_ref().as_ref().into())
                .decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad: &aad })
                .map_err(|_| VaultError::Corrupt)?,
        );
        let plain = unpad(&padded)?;
        let header_bytes = header_bytes.to_vec();
        Ok((Self { path: path.to_owned(), header, header_bytes, key, kdf, _lock: lock }, plain))
    }

    pub fn vault_id(&self) -> &str {
        &self.header.vault_id
    }

    pub fn has_device(&self) -> bool {
        self.header.slots.iter().any(|s| s.kind == SlotKind::Device)
    }

    pub fn has_password(&self) -> bool {
        self.header.slots.iter().any(|s| matches!(s.kind, SlotKind::Password { .. }))
    }

    pub fn add_device(&mut self, secret: &[u8], plain: &[u8]) -> Result<(), VaultError> {
        let slot = device_slot(secret, &self.header.vault_id, &self.key)?;
        let mut header = self.header.clone();
        header.slots.retain(|s| s.kind != SlotKind::Device);
        header.slots.push(slot);
        self.replace_header(header);
        self.save(plain)
    }

    /// Refuses to remove the last way in.
    pub fn remove_device(&mut self, plain: &[u8]) -> Result<(), VaultError> {
        let mut header = self.header.clone();
        header.slots.retain(|s| s.kind != SlotKind::Device);
        if header.slots.is_empty() {
            return Err(VaultError::LastSlot);
        }
        self.replace_header(header);
        self.save(plain)
    }

    /// Adds (or replaces) the password slot, e.g. as a recovery option.
    pub fn add_password(&mut self, password: &str, plain: &[u8]) -> Result<(), VaultError> {
        if password.chars().count() < MIN_PASSWORD_CHARS {
            return Err(VaultError::WeakPassword);
        }
        let slot = password_slot(password, &self.header.vault_id, self.kdf, &self.key)?;
        let mut header = self.header.clone();
        header.slots.retain(|s| !matches!(s.kind, SlotKind::Password { .. }));
        header.slots.push(slot);
        self.replace_header(header);
        self.save(plain)
    }

    pub fn has_keepass(&self) -> bool {
        self.header.slots.iter().any(|s| s.kind == SlotKind::Keepass)
    }

    pub fn save(&self, plain: &[u8]) -> Result<(), VaultError> {
        let padded = pad(plain);
        let nonce = random::<NONCE_LEN>();
        let aad = body_aad(&self.header_bytes);
        let ciphertext = XChaCha20Poly1305::new(self.key.as_ref().as_ref().into())
            .encrypt(XNonce::from_slice(&nonce), Payload { msg: &padded, aad: &aad })
            .map_err(|_| VaultError::Corrupt)?;
        let mut out = Vec::with_capacity(16 + self.header_bytes.len() + NONCE_LEN + ciphertext.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.header_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.header_bytes);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        write_atomic(&self.path, &out)
    }

    fn replace_header(&mut self, header: Header) {
        self.header_bytes = encode_header(&header);
        self.header = header;
    }

    pub fn add_keepass(&mut self, secret: &[u8], plain: &[u8]) -> Result<(), VaultError> {
        let slot = keepass_slot(secret, &self.header.vault_id, &self.key)?;
        let mut header = self.header.clone();
        header.slots.retain(|s| s.kind != SlotKind::Keepass);
        header.slots.push(slot);
        self.replace_header(header);
        self.save(plain)
    }

    pub fn remove_keepass(&mut self, plain: &[u8]) -> Result<(), VaultError> {
        let mut header = self.header.clone();
        header.slots.retain(|s| s.kind != SlotKind::Keepass);
        if header.slots.is_empty() {
            return Err(VaultError::LastSlot);
        }
        self.replace_header(header);
        self.save(plain)
    }

    /// Rotates the data key too, so an old copy of the file plus the old
    /// password cannot decrypt anything written from now on. The KeePassXC
    /// slot survives only if its secret is supplied.
    pub fn change_password(
        &mut self,
        current: &str,
        new: &str,
        keepass: Option<&[u8]>,
        device: Option<&[u8]>,
        plain: &[u8],
    ) -> Result<(), VaultError> {
        if new.chars().count() < MIN_PASSWORD_CHARS {
            return Err(VaultError::WeakPassword);
        }
        let (slot, salt, kdf) = self
            .header
            .slots
            .iter()
            .find_map(|s| match &s.kind {
                SlotKind::Password { salt, kdf } => Some((s, salt, *kdf)),
                SlotKind::Keepass | SlotKind::Device => None,
            })
            .ok_or(VaultError::Corrupt)?;
        let kek = derive_password(current, &decode(salt)?, kdf)?;
        unwrap(&kek, &self.header.vault_id, slot)?;

        let vault_id = self.header.vault_id.clone();
        let key = new_key();
        let mut slots = vec![password_slot(new, &vault_id, self.kdf, &key)?];
        if let Some(secret) = keepass {
            slots.push(keepass_slot(secret, &vault_id, &key)?);
        }
        if let Some(secret) = device {
            slots.push(device_slot(secret, &vault_id, &key)?);
        }
        let mut old = std::mem::replace(&mut self.key, key);
        unlock_memory(&old);
        old.zeroize();
        self.replace_header(Header { vault_id, slots });
        self.save(plain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAST: KdfParams = KdfParams { m_kib: 8 * 1024, t: 1, p: 1 };
    const PASSWORD: &str = "correct horse battery";

    fn password(p: &str) -> Secret {
        Secret::Password(Zeroizing::new(p.to_owned()))
    }

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/vault.gotodo");
        (dir, path)
    }

    fn reopen(path: &Path, secret: &Secret) -> Result<Vec<u8>, VaultError> {
        Vault::open(path, secret).map(|(_, plain)| plain.to_vec())
    }

    #[test]
    fn round_trip_with_password() {
        let (_d, path) = setup();
        drop(Vault::create_with(&path, PASSWORD, b"hello", FAST).unwrap());
        assert_eq!(reopen(&path, &password(PASSWORD)).unwrap(), b"hello");
    }

    #[test]
    fn plaintext_never_reaches_the_file() {
        let (_d, path) = setup();
        let secret = b"very-secret-todo-title";
        drop(Vault::create_with(&path, PASSWORD, secret, FAST).unwrap());
        let raw = fs::read(&path).unwrap();
        assert!(!raw.windows(secret.len()).any(|w| w == secret));
    }

    #[test]
    fn file_is_private() {
        let (_d, path) = setup();
        drop(Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap());
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        let dir = path.parent().unwrap();
        assert_eq!(fs::metadata(dir).unwrap().permissions().mode() & 0o777, 0o700);
    }

    #[test]
    fn wrong_password_is_rejected() {
        let (_d, path) = setup();
        drop(Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap());
        assert!(matches!(reopen(&path, &password("not the password")), Err(VaultError::WrongSecret)));
    }

    #[test]
    fn any_flipped_bit_is_detected() {
        let (_d, path) = setup();
        drop(Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap());
        let original = fs::read(&path).unwrap();
        for offset in [0, 9, original.len() / 2, original.len() - 1] {
            let mut raw = original.clone();
            raw[offset] ^= 1;
            fs::write(&path, &raw).unwrap();
            assert!(reopen(&path, &password(PASSWORD)).is_err(), "offset {offset}");
        }
    }

    #[test]
    fn body_padding_hides_exact_size() {
        let (_d, path) = setup();
        let vault = Vault::create_with(&path, PASSWORD, b"a", FAST).unwrap();
        let small = fs::metadata(&path).unwrap().len();
        vault.save(&[b'a'; 2000]).unwrap();
        assert_eq!(small, fs::metadata(&path).unwrap().len());
    }

    #[test]
    fn keepass_slot_unlocks_and_can_be_removed() {
        let (_d, path) = setup();
        let mut vault = Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap();
        let secret = new_keepass_secret();
        vault.add_keepass(&secret, b"y").unwrap();
        drop(vault);
        let via_keepass = Secret::KeePass(secret.clone());
        assert_eq!(reopen(&path, &via_keepass).unwrap(), b"y");
        let wrong = Secret::KeePass(new_keepass_secret());
        assert!(matches!(reopen(&path, &wrong), Err(VaultError::WrongSecret)));

        let (mut vault, _) = Vault::open(&path, &password(PASSWORD)).unwrap();
        vault.remove_keepass(b"z").unwrap();
        drop(vault);
        assert!(matches!(reopen(&path, &via_keepass), Err(VaultError::NoKeePassSlot)));
    }

    #[test]
    fn stripping_a_slot_is_detected() {
        let (_d, path) = setup();
        let mut vault = Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap();
        vault.add_keepass(&new_keepass_secret(), b"x").unwrap();
        drop(vault);
        let raw = fs::read(&path).unwrap();
        let (mut header, _, body) = split(&raw).unwrap();
        header.slots.retain(|s| s.kind != SlotKind::Keepass);
        let new_header = encode_header(&header);
        let mut forged = MAGIC.to_vec();
        forged.extend_from_slice(&(new_header.len() as u32).to_le_bytes());
        forged.extend_from_slice(&new_header);
        forged.extend_from_slice(body);
        fs::write(&path, forged).unwrap();
        assert!(matches!(reopen(&path, &password(PASSWORD)), Err(VaultError::Corrupt)));
    }

    #[test]
    fn change_password_rotates_key_and_keeps_keepass_when_given() {
        let (_d, path) = setup();
        let mut vault = Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap();
        let secret = new_keepass_secret();
        vault.add_keepass(&secret, b"x").unwrap();
        assert!(matches!(
            vault.change_password("wrong wrong wrong", "another long password", None, None, b"x"),
            Err(VaultError::WrongSecret)
        ));
        let old_key = **vault.key;
        vault.change_password(PASSWORD, "another long password", Some(&secret), None, b"new").unwrap();
        assert_ne!(old_key, **vault.key);
        drop(vault);
        assert!(reopen(&path, &password(PASSWORD)).is_err());
        assert_eq!(reopen(&path, &password("another long password")).unwrap(), b"new");
        assert_eq!(reopen(&path, &Secret::KeePass(secret)).unwrap(), b"new");
    }

    #[test]
    fn device_vault_opens_without_a_password_and_can_gain_one() {
        let (_d, path) = setup();
        let secret = new_device_secret();
        let mut vault = Vault::create_device(&path, &secret, b"x").unwrap();
        assert!(vault.has_device() && !vault.has_password());
        assert!(matches!(vault.remove_device(b"x"), Err(VaultError::LastSlot)));
        vault.add_password(PASSWORD, b"y").unwrap();
        vault.remove_device(b"y").unwrap();
        drop(vault);
        assert!(matches!(reopen(&path, &Secret::Device(secret)), Err(VaultError::NoDeviceSlot)));
        assert_eq!(reopen(&path, &password(PASSWORD)).unwrap(), b"y");
    }

    #[test]
    fn device_secret_unlocks_and_wrong_one_is_refused() {
        let (_d, path) = setup();
        let secret = new_device_secret();
        drop(Vault::create_device(&path, &secret, b"x").unwrap());
        assert_eq!(reopen(&path, &Secret::Device(secret)).unwrap(), b"x");
        assert!(matches!(reopen(&path, &Secret::Device(new_device_secret())), Err(VaultError::WrongSecret)));
    }

    #[test]
    fn short_password_is_refused() {
        let (_d, path) = setup();
        assert!(matches!(
            Vault::create_with(&path, "short", b"x", FAST),
            Err(VaultError::WeakPassword)
        ));
    }

    #[test]
    fn second_open_is_refused_while_held() {
        let (_d, path) = setup();
        let held = Vault::create_with(&path, PASSWORD, b"x", FAST).unwrap();
        assert!(matches!(reopen(&path, &password(PASSWORD)), Err(VaultError::AlreadyOpen)));
        drop(held);
        assert!(reopen(&path, &password(PASSWORD)).is_ok());
    }

    #[test]
    fn insane_kdf_parameters_are_refused() {
        let huge = KdfParams { m_kib: u32::MAX, t: 1, p: 1 };
        assert!(matches!(derive_password("p", &[0; 16], huge), Err(VaultError::Corrupt)));
    }
}
