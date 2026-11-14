//! The Quick Entry popup: opening, typing, saving, closing.

use super::*;

impl App {
    pub(super) fn open_quick(&mut self) -> Task2 {
        if self.quick.is_some() {
            return IcedTask::none();
        }
        let (id, open) = window::open(quick_settings());
        self.quick = Some(id);
        self.quick_text.clear();
        self.quick_cal = None;
        self.quick_anim = Spring::quick(0.0);
        self.quick_anim.to(1.0);
        open.map(Message::Opened)
    }

    pub(super) fn on_quick_key(&mut self, press: &KeyPress) -> Task2 {
        if !matches!(self.auth, Auth::Open) {
            self.note_caps(press);
        }
        match press.key.as_ref() {
            Key::Named(Named::Escape) => {
                // Esc first closes the mini calendar, then the popup.
                if self.quick_cal.take().is_some() {
                    return IcedTask::none();
                }
                self.close_quick()
            }
            Key::Named(Named::Tab) => {
                if matches!(self.auth, Auth::Open) { self.complete_quick(0) } else { operation::focus_next() }
            }
            _ => IcedTask::none(),
        }
    }

    pub fn quick_completions(&self) -> Vec<(String, String)> {
        self.lib().map_or_else(Vec::new, |lib| parse::completions(&self.quick_text, lib))
    }

    /// Replaces any `marker` token in the Quick Entry text with `marker` + `value`.
    pub(super) fn set_quick_token(&mut self, marker: char, value: &str) {
        let kept: Vec<&str> = self.quick_text.split_whitespace().filter(|w| !w.starts_with(marker)).collect();
        let mut text = kept.join(" ");
        if !text.is_empty() {
            text.push(' ');
        }
        text.push(marker);
        text.push_str(value);
        text.push(' ');
        self.quick_text = text;
    }

    pub(super) fn complete_quick(&mut self, index: usize) -> Task2 {
        if let Some((replacement, _)) = self.quick_completions().get(index) {
            self.quick_text = parse::apply_completion(&self.quick_text, replacement);
        }
        IcedTask::batch([operation::focus(quick_id()), operation::move_cursor_to_end(quick_id())])
    }

    /// Quick Entry closes only on Esc, its shortcut, or after saving. Losing
    /// focus does not close it: with focus-follows-mouse, merely moving the
    /// pointer across it would.
    pub(super) fn close_quick(&mut self) -> Task2 {
        // Dismiss in one cut: the compositor keeps blur and border for as long as
        // the window exists, so fading the contents would leave an empty box behind.
        match self.quick {
            Some(id) => window::close(id),
            None => IcedTask::none(),
        }
    }

    pub(super) fn submit_quick(&mut self) -> Task2 {
        let today = self.today;
        let keep_open = self.modifiers.shift();
        let text = std::mem::take(&mut self.quick_text);
        let Some(session) = &mut self.session else { return IcedTask::none() };
        let draft = parse::parse_draft(&text, &session.lib, today);
        if draft.title.is_empty() {
            self.quick_text = text;
            return IcedTask::none();
        }
        session.edit(|lib| {
            let fallback = if lib.settings.quick_entry_to_today { Schedule::On(today) } else { Schedule::Inbox };
            let mut task = Task::new(Kind::Todo, draft.title.clone(), fallback);
            task.deadline = draft.deadline;
            task.deadline_time = draft.deadline_time;
            for (existing, name) in &draft.tags {
                let tag = existing.unwrap_or_else(|| lib.ensure_tag(name));
                task.tags.push(tag);
            }
            let id = lib.insert(task);
            if let Some((target, _)) = draft.target {
                lib.move_to(id, target);
            }
            if let (Some(when), Some(task)) = (draft.when, lib.task_mut(id)) {
                task.schedule = when.schedule;
                task.evening = when.evening;
                task.reminder = when.reminder;
            }
        });
        self.refresh(true);
        if keep_open { operation::focus(quick_id()) } else { self.close_quick() }
    }

    /// Messages about the Quick Entry popup.
    pub(super) fn update_quick(&mut self, message: Message) -> Task2 {
        match message {
            Message::QuickCal(marker) => {
                self.quick_cal = match self.quick_cal {
                    Some((open, _)) if open == marker => None,
                    _ => Some((marker, self.today)),
                };
                self.quick_cal_slide = Spring::at(0.0);
            }
            Message::QuickCalShift(weeks) => {
                if let Some((_, start)) = &mut self.quick_cal {
                    let days = chrono::Duration::days(i64::from(weeks) * 7);
                    *start = start.checked_add_signed(days).unwrap_or(*start);
                    // The old days stay on screen first, then glide out of the way.
                    self.quick_cal_slide.nudge(-7.0 * weeks as f32);
                    self.quick_cal_slide.to(0.0);
                }
            }
            Message::QuickCalPick(date) => {
                if let Some((marker, _)) = self.quick_cal.take() {
                    self.set_quick_token(marker, &date.format("%Y-%m-%d").to_string());
                }
                return IcedTask::batch([operation::focus(quick_id()), operation::move_cursor_to_end(quick_id())]);
            }
            Message::QuickPreset(marker, word) => {
                self.set_quick_token(marker, word);
                return IcedTask::batch([operation::focus(quick_id()), operation::move_cursor_to_end(quick_id())]);
            }
            Message::QuickInput(value) => self.quick_text = value,
            Message::QuickSubmit => return self.submit_quick(),
            Message::QuickComplete(index) => return self.complete_quick(index),
            Message::QuickToken(marker) => {
                if !self.quick_text.is_empty() && !self.quick_text.ends_with(' ') {
                    self.quick_text.push(' ');
                }
                self.quick_text.push(marker);
                return IcedTask::batch([operation::focus(quick_id()), operation::move_cursor_to_end(quick_id())]);
            }
            _ => {}
        }
        IcedTask::none()
    }
}
