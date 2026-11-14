//! An unlocked library: undo history plus background saving to the vault.

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use zeroize::Zeroizing;

use crate::model::Library;
use crate::vault::{Secret, Vault, VaultError};

const UNDO_LIMIT: usize = 100;
const SAVE_DELAY: Duration = Duration::from_millis(400);

enum SaveJob {
    Write(Zeroizing<Vec<u8>>),
    Stop,
}

pub struct Session {
    pub lib: Library,
    pub vault: Arc<Mutex<Vault>>,
    pub via_keepass: bool,
    pub save_error: Arc<Mutex<Option<String>>>,
    undo: Vec<Library>,
    redo: Vec<Library>,
    coalesce: Option<String>,
    dirty_since: Option<Instant>,
    jobs: Sender<SaveJob>,
    worker: Option<JoinHandle<()>>,
}

pub fn vault_path() -> PathBuf {
    crate::paths::vault()
}

fn serialize(lib: &Library) -> Zeroizing<Vec<u8>> {
    Zeroizing::new(serde_json::to_vec(lib).expect("library serializes"))
}

impl Session {
    fn start(lib: Library, vault: Vault, via_keepass: bool) -> Self {
        let vault = Arc::new(Mutex::new(vault));
        let save_error = Arc::new(Mutex::new(None));
        let (jobs, inbox) = mpsc::channel();
        let worker = {
            let vault = Arc::clone(&vault);
            let save_error = Arc::clone(&save_error);
            thread::spawn(move || {
                while let Ok(job) = inbox.recv() {
                    let SaveJob::Write(mut latest) = job else { break };
                    let mut stop = false;
                    for next in inbox.try_iter() {
                        match next {
                            SaveJob::Write(newer) => latest = newer,
                            SaveJob::Stop => stop = true,
                        }
                    }
                    let vault = vault.lock().expect("vault lock");
                    // A failed copy must not stop the save itself.
                    let _ = crate::backup::daily(vault.path(), crate::model::local_today());
                    let result = vault.save(&latest);
                    drop(vault);
                    *save_error.lock().expect("error lock") = result.err().map(|e| e.to_string());
                    if stop {
                        break;
                    }
                }
            })
        };
        Self {
            lib,
            vault,
            via_keepass,
            save_error,
            undo: Vec::new(),
            redo: Vec::new(),
            coalesce: None,
            dirty_since: None,
            jobs,
            worker: Some(worker),
        }
    }

    #[cfg(test)]
    pub fn for_tests(lib: Library, path: &std::path::Path) -> Self {
        let plain = serialize(&lib);
        let vault = Vault::create_for_tests(path, "correct horse battery", &plain).expect("test vault");
        Self::start(lib, vault, false)
    }

    pub fn create(password: &str) -> Result<Self, VaultError> {
        let lib = Library::default();
        let vault = Vault::create(&vault_path(), password, &serialize(&lib))?;
        Ok(Self::start(lib, vault, false))
    }

    /// A new vault that opens with the sealed device secret, no password.
    pub fn create_device(secret: &[u8]) -> Result<Self, VaultError> {
        let lib = Library::default();
        let vault = Vault::create_device(&vault_path(), secret, &serialize(&lib))?;
        Ok(Self::start(lib, vault, false))
    }

    pub fn open(secret: &Secret) -> Result<Self, VaultError> {
        let (vault, plain) = Vault::open(&vault_path(), secret)?;
        let lib = serde_json::from_slice(&plain).map_err(|_| VaultError::Corrupt)?;
        Ok(Self::start(lib, vault, matches!(secret, Secret::KeePass(_))))
    }

    /// Applies an undoable change.
    pub fn edit(&mut self, change: impl FnOnce(&mut Library)) {
        self.coalesce = None;
        self.checkpoint();
        change(&mut self.lib);
        self.mark_dirty();
    }

    /// Like `edit`, but a run of changes with the same key is one undo step.
    pub fn edit_coalesced(&mut self, key: String, change: impl FnOnce(&mut Library)) {
        if self.coalesce.as_ref() != Some(&key) {
            self.checkpoint();
            self.coalesce = Some(key);
        }
        change(&mut self.lib);
        self.mark_dirty();
    }

    /// A change that is not a user action, such as logging or spawning repeats.
    pub fn edit_silent(&mut self, change: impl FnOnce(&mut Library) -> bool) -> bool {
        let changed = change(&mut self.lib);
        if changed {
            self.mark_dirty();
        }
        changed
    }

    fn checkpoint(&mut self) {
        self.undo.push(self.lib.clone());
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn mark_dirty(&mut self) {
        self.dirty_since.get_or_insert_with(Instant::now);
    }

    pub fn undo(&mut self) -> bool {
        self.swap_history(true)
    }

    pub fn redo(&mut self) -> bool {
        self.swap_history(false)
    }

    fn swap_history(&mut self, undo: bool) -> bool {
        self.coalesce = None;
        let (from, to) = if undo { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        let Some(state) = from.pop() else { return false };
        to.push(std::mem::replace(&mut self.lib, state));
        self.mark_dirty();
        true
    }

    pub fn save_if_due(&mut self) {
        if self.dirty_since.is_some_and(|since| since.elapsed() >= SAVE_DELAY) {
            self.queue_save();
        }
    }

    fn queue_save(&mut self) {
        self.dirty_since = None;
        let _ = self.jobs.send(SaveJob::Write(serialize(&self.lib)));
    }

    pub fn plaintext(&self) -> Zeroizing<Vec<u8>> {
        serialize(&self.lib)
    }

    /// Writes any pending change and waits for the worker to finish.
    pub fn close(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if self.dirty_since.is_some() {
            self.queue_save();
        }
        let _ = self.jobs.send(SaveJob::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.shutdown();
    }
}
