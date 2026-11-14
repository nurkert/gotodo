//! Locking and every way of unlocking: password, device key, KeePassXC.

use super::*;

impl App {
    pub(super) fn probe(auto: bool) -> Task2 {
        blocking(keepass::status, move |status| Message::KeePassProbe(status, auto))
    }

    pub(super) fn lock(&mut self) {
        self.refresh_slots();
        if self.prefs.lock_needs_secret && self.slots.device && self.can_pause_device() && !self.prefs.device_paused {
            self.prefs.device_paused = true;
            self.prefs.save();
        }
        self.unlock_password = false;
        self.show_password = false;
        self.booting = false;
        self.view_anim = Spring::at(0.0);
        self.view_anim.to(1.0);
        let has_keepass = self.slots.keepass;
        if let Some(session) = self.session.take() {
            session.close();
        }
        self.auth = Auth::Locked { has_keepass };
        self.skip_auto_unlock = true;
        self.auth_error = None;
        self.expanded = None;
        self.new_item = None;
        self.selection.clear();
        self.sections.clear();
        self.flat.clear();
        self.rows.clear();
        self.ghosts.clear();
        self.sheet = None;
        self.sheet_anim = Spring::at(0.0);
        self.toast = None;
        self.notifications.clear();
        self.title.zeroize();
        self.query.zeroize();
        self.checklist_input.zeroize();
        self.quick_text.zeroize();
        self.notes = text_editor::Content::new();
        self.project_notes = text_editor::Content::new();
    }

    pub(super) fn finish_unlock(&mut self, session: Session) -> Task2 {
        self.session = Some(session);
        self.refresh_slots();
        self.auth = Auth::Open;
        self.auth_busy = false;
        self.auth_error = None;
        self.password.zeroize();
        self.repeat.zeroize();
        self.booting = false;
        self.show_password = false;
        self.caps_lock = false;
        self.lock_notice = None;
        let method = self.unlock_method.take();
        if matches!(method, Some(Method::Password | Method::KeePass)) && self.prefs.device_paused {
            self.prefs.device_paused = false;
            self.prefs.save();
        }
        if !crate::paths::prefs().exists() {
            // First start after the look moved out of the vault: carry it over.
            if let Some(s) = self.lib().map(|lib| lib.settings.clone()) {
                self.prefs.appearance = s.appearance;
                self.prefs.text_scale = s.text_scale;
                self.prefs.reduce_motion = s.reduce_motion;
            }
            self.prefs.save();
        }
        self.today = local_today();
        self.last_activity = Instant::now();
        self.run_daily();
        self.refresh(false);
        self.view_anim = Spring::at(0.0);
        self.view_anim.to(1.0);
        if self.quick.is_some() { operation::focus(quick_id()) } else { IcedTask::none() }
    }

    pub(super) fn run_daily(&mut self) {
        let today = self.today;
        if let Some(session) = &mut self.session {
            session.edit_silent(|lib| {
                let spawned = lib.spawn_repeats(today);
                lib.sweep_logbook(today) || spawned
            });
        }
    }

    pub(super) fn unlock_with_password(&mut self) -> Task2 {
        if self.auth_busy || self.password.is_empty() {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.unlock_method = Some(Method::Password);
        let secret = Secret::Password(Zeroizing::new(std::mem::take(&mut self.password)));
        blocking(
            move || {
                Session::open(&secret).map_err(|e| match e {
                    VaultError::WrongSecret => t("Wrong password").to_owned(),
                    other => other.to_string(),
                })
            },
            |result| Message::AuthDone(slot(result)),
        )
    }

    /// Re-reads which unlock methods the open vault offers.
    pub(super) fn refresh_slots(&mut self) {
        if let Some(session) = &self.session {
            let vault = session.vault.lock().expect("vault lock");
            self.slots = vault::Slots {
                password: vault.has_password(),
                keepass: vault.has_keepass(),
                device: vault.has_device(),
            };
        }
    }

    pub(super) fn unlock_with_device(&mut self) -> Task2 {
        if self.auth_busy || !self.device_allowed() {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.unlock_method = Some(Method::Device);
        blocking(
            || {
                let secret = device::unseal().ok_or_else(|| t("This computer could not open the device key").to_owned())?;
                Session::open(&Secret::Device(secret)).map_err(|e| e.to_string())
            },
            |result| Message::AuthDone(slot(result)),
        )
    }

    pub(super) fn create_device_vault(&mut self) -> Task2 {
        if self.auth_busy {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
        blocking(
            || {
                let secret = vault::new_device_secret();
                device::seal(&secret)?;
                Session::create_device(&secret).map_err(|e| {
                    device::remove();
                    e.to_string()
                })
            },
            |result| Message::AuthDone(slot(result)),
        )
    }

    pub(super) fn unlock_with_keepass(&mut self) -> Task2 {
        if self.auth_busy {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.unlock_method = Some(Method::KeePass);
        blocking(
            || {
                let (vault_id, _) = vault::peek(&session::vault_path()).map_err(|e| e.to_string())?;
                let secret = keepass::read_secret(&vault_id)
                    .ok_or_else(|| t("KeePassXC is locked").to_owned())?;
                Session::open(&Secret::KeePass(secret)).map_err(|e| e.to_string())
            },
            |result| Message::AuthDone(slot(result)),
        )
    }

    pub(super) fn create_vault(&mut self) -> Task2 {
        if self.auth_busy {
            return IcedTask::none();
        }
        if self.password.chars().count() < MIN_PASSWORD_CHARS {
            self.auth_error = Some(VaultError::WeakPassword.to_string());
            return IcedTask::none();
        }
        if self.password != self.repeat {
            self.auth_error = Some(t("Passwords do not match").to_owned());
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.unlock_method = Some(Method::Password);
        let password = Zeroizing::new(std::mem::take(&mut self.password));
        self.repeat.zeroize();
        blocking(
            move || Session::create(&password).map_err(|e| e.to_string()),
            |result| Message::AuthDone(slot(result)),
        )
    }

    /// Caps Lock is on when a letter arrives in the other case than Shift asks for.
    pub(super) fn note_caps(&mut self, press: &KeyPress) {
        if matches!(press.key.as_ref(), Key::Named(Named::CapsLock)) {
            self.caps_lock = !self.caps_lock;
            return;
        }
        let Some(letter) = press.text.as_deref().and_then(|t| t.chars().next()).filter(|c| c.is_alphabetic()) else {
            return;
        };
        if letter.is_uppercase() != letter.is_lowercase() {
            self.caps_lock = letter.is_uppercase() != press.modifiers.shift();
        }
    }

    pub(super) fn security_task(
        &mut self,
        work: impl FnOnce(Arc<Mutex<vault::Vault>>, Zeroizing<Vec<u8>>) -> Result<String, String> + Send + 'static,
    ) -> Task2 {
        let Some(session) = &self.session else { return IcedTask::none() };
        let vault = Arc::clone(&session.vault);
        let plain = session.plaintext();
        blocking(move || work(vault, plain), |result| Message::SecurityDone(slot(result)))
    }

    /// Messages about locking, unlocking, passwords and backups.
    pub(super) fn update_security(&mut self, message: Message) -> Task2 {
        match message {
            Message::ShowPassword(on) => {
                self.show_password = on;
                return operation::focus(auth_id());
            }
            Message::AuthPassword(value) => {
                self.password = value;
                self.auth_error = None;
            }
            Message::AuthRepeat(value) => self.repeat = value,
            Message::AuthSubmit => {
                return match self.auth {
                    Auth::Setup => self.create_vault(),
                    Auth::Locked { .. } if !self.slots.password || (self.device_allowed() && !self.unlock_password) => {
                        self.unlock_with_device()
                    }
                    Auth::Locked { .. } => self.unlock_with_password(),
                    Auth::Open => IcedTask::none(),
                };
            }
            Message::AuthKeePass => return self.unlock_with_keepass(),
            Message::AuthDevice => return self.unlock_with_device(),
            Message::SetupDevice => return self.create_device_vault(),
            Message::UnlockMode(password) => {
                self.unlock_password = password;
                self.auth_error = None;
                return if password { operation::focus(auth_id()) } else { IcedTask::none() };
            }
            Message::SetupPassword(on) => {
                self.setup_password = on;
                self.auth_error = None;
            }
            Message::DeviceEnable => {
                return self.security_task(|vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    let secret = vault::new_device_secret();
                    device::seal(&secret)?;
                    vault.add_device(&secret, &plain).map_err(|e| e.to_string())?;
                    Ok(t("Device unlock is on").to_owned())
                });
            }
            Message::DeviceDisable => {
                return self.security_task(|vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    vault.remove_device(&plain).map_err(|e| e.to_string())?;
                    device::remove();
                    Ok(t("Device unlock is off").to_owned())
                });
            }
            Message::AuthDone(result) => {
                self.auth_busy = false;
                self.booting = false;
                match result.lock().expect("result slot").take() {
                    Some(Ok(session)) => return self.finish_unlock(session),
                    Some(Err(error)) => {
                        self.auth_error = Some(error);
                        return operation::focus(auth_id());
                    }
                    None => {}
                }
            }
            Message::KeePassProbe(status, auto) => {
                self.keepass = status;
                if self.session.as_ref().is_some_and(|s| s.via_keepass) && status != keepass::Status::Unlocked {
                    self.lock();
                    return operation::focus(auth_id());
                }
                let can_auto = auto && !self.skip_auto_unlock;
                if can_auto
                    && status == keepass::Status::Unlocked
                    && matches!(self.auth, Auth::Locked { has_keepass: true })
                {
                    return self.unlock_with_keepass();
                }
                if auto && !self.auth_busy {
                    self.booting = false;
                }
            }
            Message::PasswordOld(value) => self.password_old = value,
            Message::PasswordNew(value) => self.password_new = value,
            Message::PasswordRepeat(value) => self.password_repeat = value,
            Message::PasswordChange => {
                let old = Zeroizing::new(std::mem::take(&mut self.password_old));
                let new = Zeroizing::new(std::mem::take(&mut self.password_new));
                self.password_repeat.zeroize();
                return self.security_task(move |vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    let secret = if vault.has_keepass() { keepass::read_secret(vault.vault_id()) } else { None };
                    if !vault.has_password() {
                        vault.add_password(&new, &plain).map_err(|e| e.to_string())?;
                        return Ok(t("Password set").to_owned());
                    }
                    let device = if vault.has_device() { device::unseal() } else { None };
                    vault
                        .change_password(
                            &old,
                            &new,
                            secret.as_ref().map(|s| s.as_slice()),
                            device.as_ref().map(|s| s.as_slice()),
                            &plain,
                        )
                        .map_err(|e| e.to_string())?;
                    // Old copies still open with the old password, so they go.
                    let _ = crate::backup::restart(vault.path(), local_today());
                    Ok(t("Password changed").to_owned())
                });
            }
            Message::SecurityDone(result) => {
                self.refresh_slots();
                self.security_note = result
                    .lock()
                    .expect("result slot")
                    .take()
                    .map(|outcome| outcome.unwrap_or_else(|error| error));
            }
            Message::KeePassConnect => {
                return self.security_task(|vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    let secret = vault::new_keepass_secret();
                    keepass::store_secret(vault.vault_id(), &secret)?;
                    vault.add_keepass(&secret, &plain).map_err(|e| e.to_string())?;
                    Ok(t("KeePassXC connected").to_owned())
                });
            }
            Message::KeePassDisconnect => {
                return self.security_task(|vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    vault.remove_keepass(&plain).map_err(|e| e.to_string())?;
                    let _ = keepass::delete_secret(vault.vault_id());
                    Ok(t("Done").to_owned())
                });
            }
            Message::BackupExport => {
                let source = session::vault_path();
                let today = self.today;
                return blocking(
                    move || {
                        crate::backup::export(&source, &crate::backup::export_dir(), today)
                            .map(|path| format!("{} {}", t("Saved to"), path.display()))
                            .map_err(|e| e.to_string())
                    },
                    |result| Message::SecurityDone(slot(result)),
                );
            }
            Message::BackupRestore(path) => {
                if self.restore_armed.as_ref() != Some(&path) {
                    self.restore_armed = Some(path);
                    return IcedTask::none();
                }
                self.restore_armed = None;
                let date = self.backups.iter().find(|b| b.path == path).map(|b| b.date);
                self.lock();
                let vault = session::vault_path();
                match crate::backup::restore(&vault, &path, Local::now().naive_local()) {
                    Ok(_) => {
                        self.slots = vault::peek_slots(&vault).unwrap_or_default();
                        self.auth = Auth::Locked { has_keepass: self.slots.keepass };
                        let day = date.map_or_else(String::new, |d| format!("{}. {}", d.day(), crate::i18n::month_name(d.month())));
                        self.lock_notice = Some(format!("{} {day}. {}", t("Restored the copy from"), t("Unlock it the way you did then.")));
                    }
                    Err(error) => self.auth_error = Some(error.to_string()),
                }
                return operation::focus(auth_id());
            }
            _ => {}
        }
        IcedTask::none()
    }
}
