//! The clock: animation frames, reminders, auto-lock, settings writes, status line.

use super::*;

impl App {
    /// The line `gotodo status` prints: today's count, overdue as its class.
    pub(super) fn publish_status(&mut self) {
        let line = match self.lib() {
            None => ipc::status_line("", t("gotodo is locked"), "locked"),
            Some(lib) => {
                let today = lib.today_count(self.today);
                let overdue = lib.overdue_count(self.today);
                let tooltip = if overdue > 0 {
                    format!("{today} {} · {overdue} {}", t("today"), t("overdue"))
                } else {
                    format!("{today} {}", t("today"))
                };
                let class = if overdue > 0 { "overdue" } else if today > 0 { "today" } else { "empty" };
                ipc::status_line(&today.to_string(), &tooltip, class)
            }
        };
        if line != self.published {
            ipc::publish(line.clone());
            self.published = line;
        }
    }

    pub(super) fn on_tick(&mut self) -> Task2 {
        self.publish_status();
        if self.booting && self.boot_started.elapsed() >= Duration::from_secs(5) {
            self.booting = false;
        }
        if self.session.is_none() {
            // Only talk to the Secret Service when KeePassXC unlock is actually set up.
            if self.slots.keepass && self.last_probe.elapsed() >= Duration::from_secs(3) {
                self.last_probe = Instant::now();
                return Self::probe(false);
            }
            return IcedTask::none();
        }
        let mut tasks = Vec::new();

        let auto_lock = self.lib().map_or(0, |l| l.settings.auto_lock_minutes);
        if auto_lock > 0 && self.last_activity.elapsed() >= Duration::from_secs(u64::from(auto_lock) * 60) {
            self.lock();
            return operation::focus(auth_id());
        }

        let now_day = local_today();
        if now_day != self.today {
            self.today = now_day;
            self.run_daily();
            self.refresh(true);
        }

        if self.pending_log.is_some_and(|at| at.elapsed() >= LOG_DELAY) {
            self.pending_log = None;
            if let Some(session) = &mut self.session {
                let today = self.today;
                if session.edit_silent(|lib| lib.sweep_logbook(today)) {
                    self.refresh(true);
                }
            }
        }

        let now = Local::now().time();
        let today = self.today;
        let mut due = Vec::new();
        if let Some(session) = &mut self.session {
            session.edit_silent(|lib| {
                due = lib.take_due_reminders(today, now);
                !due.is_empty()
            });
            session.save_if_due();
        }
        let lead = self.lib().map_or(0, |l| l.settings.deadline_alert_minutes);
        let mut closing = Vec::new();
        if let Some(session) = &mut self.session {
            session.edit_silent(|lib| {
                closing = lib.take_due_deadlines(today, now, lead);
                !closing.is_empty()
            });
        }
        let show_titles = self.lib().is_some_and(|l| l.settings.reminder_shows_title);
        let mut notes = Vec::new();
        for (id, left) in closing {
            let Some(task) = self.lib().and_then(|l| l.task(id)) else { continue };
            let when = if left <= 0 {
                t("Due now").to_owned()
            } else if left < 90 {
                format!("{} {left} {}", t("Due in"), t("min"))
            } else {
                format!("{} {} {}", t("Due in"), (left + 30) / 60, t("h"))
            };
            let body = if show_titles { format!("{}: {when}", task.title) } else { when };
            notes.push(crate::notify::Note { body, task: Some(id) });
        }
        for id in due {
            let Some(task) = self.lib().and_then(|l| l.task(id)) else { continue };
            let body = if show_titles { task.title.clone() } else { t("A to-do is due").to_owned() };
            notes.push(crate::notify::Note { body, task: Some(id) });
        }
        if !notes.is_empty() {
            tasks.push(blocking(move || crate::notify::show(notes), Message::Notified));
        }
        if self.toast.as_ref().is_some_and(|toast| toast.at.elapsed() >= TOAST_FOR) {
            self.toast = None;
        }

        let watching = matches!(self.sheet, Some(Sheet::KeePass | Sheet::Settings));
        let every = if watching { Duration::from_millis(1200) } else { Duration::from_secs(3) };
        if (watching || self.session.as_ref().is_some_and(|s| s.via_keepass))
            && self.last_probe.elapsed() >= every
        {
            self.last_probe = Instant::now();
            tasks.push(Self::probe(false));
        }
        IcedTask::batch(tasks)
    }

    pub(super) fn on_frame(&mut self, now: Instant) -> Task2 {
        let dt = self.last_frame.map_or(1.0 / 60.0, |last| (now - last).as_secs_f32());
        self.last_frame = Some(now);
        let reduce = self.reduce_motion();
        self.view_anim.step(dt, reduce);
        self.sheet_anim.step(dt, reduce);
        self.quick_anim.step(dt, reduce);
        self.quick_cal_slide.step(dt, reduce);
        if self.auth_busy || self.booting {
            self.spin = (self.spin + dt * 1.1).fract();
        }
        if self.sheet_anim.settled() && self.sheet_anim.target < 0.5 {
            self.sheet = None;
        }
        for anim in self.rows.values_mut() {
            anim.appear.step(dt, reduce);
            anim.expand.step(dt, reduce);
            anim.check.step(dt, reduce);
        }
        let finished: Vec<Id> = self
            .ghosts
            .iter()
            .copied()
            .filter(|id| self.rows.get(id).is_none_or(|a| a.appear.settled()))
            .collect();
        if !finished.is_empty() {
            for id in &finished {
                self.ghosts.remove(id);
                self.rows.remove(id);
            }
            for section in &mut self.sections {
                section.rows.retain(|id| !finished.contains(id));
            }
            self.flat.retain(|id| !finished.contains(id));
        }
        if !self.animating() {
            self.last_frame = None;
        }
        IcedTask::none()
    }

    pub(super) fn change_setting(&mut self, change: SettingChange) {
        match change {
            SettingChange::Appearance(v) => self.prefs.appearance = v,
            SettingChange::Scale(v) => self.prefs.text_scale = v.clamp(0.7, 2.0),
            SettingChange::ReduceMotion(v) => self.prefs.reduce_motion = v,
            SettingChange::LockNeedsSecret(v) => self.prefs.lock_needs_secret = v && self.can_pause_device(),
            SettingChange::Colors(v) => {
                self.prefs.colors = v;
                // Read again, so edits to gtk.css show up without a restart.
                self.gtk = if v == crate::prefs::ColorSource::Gotodo { None } else { super::gtk::GtkColors::load() };
            }
            SettingChange::Accent(v) => self.prefs.accent = v,
            SettingChange::KeepRunning(v) => self.prefs.keep_running = v,
            SettingChange::QuickWhileLocked(v) => self.prefs.quick_while_locked = v,
            SettingChange::PageOpacity(v) => self.prefs.page_opacity = v.clamp(crate::prefs::MIN_OPACITY, 1.0),
            SettingChange::SidebarOpacity(v) => self.prefs.sidebar_opacity = v.clamp(crate::prefs::MIN_OPACITY, 1.0),
            SettingChange::QuickOpacity(v) => self.prefs.quick_opacity = v.clamp(crate::prefs::MIN_OPACITY, 1.0),
            _ => {}
        }
        self.prefs.save();
        let Some(session) = &mut self.session else { return };
        session.edit_silent(|lib| {
            let s = &mut lib.settings;
            match change {
                SettingChange::Appearance(v) => s.appearance = v,
                SettingChange::Scale(v) => s.text_scale = v.clamp(0.7, 2.0),
                SettingChange::Log(v) => s.log_interval = v,
                SettingChange::GroupToday(v) => s.group_today_by_parent = v,
                SettingChange::ReduceMotion(v) => s.reduce_motion = v,
                SettingChange::AutoLock(v) => s.auto_lock_minutes = v,
                SettingChange::ReminderTitles(v) => s.reminder_shows_title = v,
                SettingChange::QuickToday(v) => s.quick_entry_to_today = v,
                SettingChange::DeadlineAlert(v) => s.deadline_alert_minutes = v,
                SettingChange::LockNeedsSecret(_)
                | SettingChange::Colors(_)
                | SettingChange::Accent(_)
                | SettingChange::PageOpacity(_)
                | SettingChange::SidebarOpacity(_)
                | SettingChange::QuickOpacity(_)
                | SettingChange::KeepRunning(_)
                | SettingChange::QuickWhileLocked(_) => return false,
            }
            true
        });
        self.refresh(false);
    }
}
