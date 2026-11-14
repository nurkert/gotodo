//! Keys in the main window.

use super::*;

impl App {
    pub(super) fn on_key(&mut self, press: KeyPress) -> Task2 {
        self.last_activity = Instant::now();
        if Some(press.window) == self.quick {
            return self.on_quick_key(&press);
        }
        if !matches!(self.auth, Auth::Open) {
            self.note_caps(&press);
            if matches!(press.key.as_ref(), Key::Named(Named::Tab)) {
                return operation::focus_next();
            }
            // With the device button as the main action, Enter presses it.
            if matches!(press.key.as_ref(), Key::Named(Named::Enter))
                && matches!(self.auth, Auth::Locked { .. })
                && self.device_allowed()
                && !self.unlock_password
            {
                return self.unlock_with_device();
            }
            return IcedTask::none();
        }
        let named = match press.key.as_ref() {
            Key::Named(named) => Some(named),
            _ => None,
        };
        let plain = !press.modifiers.control() && !press.modifiers.alt() && !press.modifiers.logo();

        if named == Some(Named::Escape) {
            if self.sheet.is_some() && self.sheet_anim.target > 0.5 {
                return self.close_sheet();
            }
            if self.expanded.is_some() {
                self.collapse();
                return unfocus();
            }
            if !press.captured {
                self.selection.clear();
            }
            return IcedTask::none();
        }

        if self.sheet.is_some() && self.sheet_anim.target > 0.5 {
            match named {
                Some(Named::ArrowDown) => self.sheet_index = self.sheet_index.saturating_add(1),
                Some(Named::ArrowUp) => self.sheet_index = self.sheet_index.saturating_sub(1),
                Some(Named::Tab) => return operation::focus_next(),
                _ => {}
            }
            let total = self.sheet_items().len();
            self.sheet_index = self.sheet_index.min(total.saturating_sub(1));
            return IcedTask::none();
        }

        if let Some(cmd) = keys::lookup(&press.key, press.latin, press.modifiers) {
            // A focused text field reports every key as captured, so Ctrl chords
            // still run; only keys that mean something while typing are held back.
            let text_key = !press.modifiers.control() || cmd == Cmd::SelectAll;
            if !(text_key && press.captured) {
                return self.run(cmd);
            }
        }
        if press.captured {
            return IcedTask::none();
        }

        match named {
            Some(Named::ArrowDown | Named::ArrowUp) if self.expanded.is_some() => {}
            Some(Named::ArrowDown) => self.step_selection(1, press.modifiers.shift()),
            Some(Named::ArrowUp) => self.step_selection(-1, press.modifiers.shift()),
            Some(Named::Enter) if plain => {
                if let Some(id) = self.selection.last().copied() {
                    return self.expand(id);
                }
            }
            Some(Named::Space) if plain => return self.new_todo(),
            None if plain => {
                if let Some(text) = press.text.filter(|t| t.chars().all(|c| !c.is_control())) {
                    let open = self.open_sheet(Sheet::Find);
                    self.query = text;
                    return IcedTask::batch([open, operation::move_cursor_to_end(sheet_id())]);
                }
            }
            _ => {}
        }
        IcedTask::none()
    }

    pub(super) fn step_selection(&mut self, delta: isize, extend: bool) {
        if self.flat.is_empty() {
            return;
        }
        let current = self.selection.last().and_then(|id| self.flat.iter().position(|x| x == id));
        let next = match current {
            Some(index) => index.saturating_add_signed(delta).min(self.flat.len() - 1),
            None if delta > 0 => 0,
            None => self.flat.len() - 1,
        };
        let id = self.flat[next];
        if extend {
            if !self.selection.contains(&id) {
                self.selection.push(id);
            } else if let Some(last) = self.selection.pop() {
                if self.selection.is_empty() {
                    self.selection.push(last);
                }
            }
        } else {
            self.selection = vec![id];
            self.anchor = Some(id);
        }
    }
}
