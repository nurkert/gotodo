//! Which list is shown, which to-do is open, which sheet is up.

use super::*;

impl App {
    /// Recomputes the visible list, animating rows in and out when it is the same view.
    pub(super) fn refresh(&mut self, animate: bool) {
        let Some(lib) = self.lib() else { return };
        let mut fresh = lib.sections(self.view, self.today);
        if let Some(tag) = self.tag_filter {
            let wanted = lib.tag_and_descendants(tag);
            for section in &mut fresh {
                section.rows.retain(|id| {
                    lib.task(*id).is_some_and(|t| t.tags.iter().any(|tag| wanted.contains(tag)))
                });
            }
        }
        let new_ids: HashSet<Id> = fresh.iter().flat_map(|s| s.rows.iter().copied()).collect();
        let old_ids: HashSet<Id> = self.flat.iter().copied().collect();

        if animate {
            let exists: HashSet<Id> = lib.tasks.iter().map(|t| t.id).collect();
            let mut leaving = Vec::new();
            for (index, section) in self.sections.iter().enumerate() {
                for (position, id) in section.rows.iter().enumerate() {
                    if !new_ids.contains(id) && exists.contains(id) {
                        leaving.push((index, position, *id));
                    }
                }
            }
            for (index, position, id) in leaving {
                if let Some(section) = fresh.get_mut(index) {
                    let at = position.min(section.rows.len());
                    section.rows.insert(at, id);
                    self.ghosts.insert(id);
                    self.anim_mut(id).appear.to(0.0);
                }
            }
            for id in new_ids.difference(&old_ids) {
                let row = self.anim_mut(*id);
                row.appear = Spring::at(0.0);
                row.appear.to(1.0);
            }
            for id in &new_ids {
                if self.ghosts.remove(id) {
                    self.anim_mut(*id).appear.to(1.0);
                }
            }
        } else {
            self.ghosts.clear();
        }

        self.flat = fresh.iter().flat_map(|s| s.rows.iter().copied()).collect();
        self.sections = fresh;
        self.selection.retain(|id| new_ids.contains(id));
        if let Some(id) = self.expanded.filter(|id| !new_ids.contains(id)) {
            self.expanded = None;
            self.anim_mut(id).expand.to(0.0);
        }
    }

    pub(super) fn go(&mut self, view: View) -> Task2 {
        if view == self.view {
            return IcedTask::none();
        }
        self.collapse();
        self.view = view;
        self.tag_filter = None;
        self.selection.clear();
        self.anchor = None;
        self.flat.clear();
        self.sections.clear();
        self.rows.clear();
        self.ghosts.clear();
        self.refresh(false);
        self.load_project_notes();
        self.view_anim = Spring::at(0.0);
        self.view_anim.to(1.0);
        IcedTask::none()
    }

    pub(super) fn load_project_notes(&mut self) {
        let notes = match self.view {
            View::Project(id) => self.lib().and_then(|l| l.task(id)).map(|t| t.notes.clone()),
            _ => None,
        };
        self.project_notes = text_editor::Content::with_text(&notes.unwrap_or_default());
    }

    pub(super) fn collapse(&mut self) {
        let Some(id) = self.expanded.take() else { return };
        self.anim_mut(id).expand.to(0.0);
        if self.new_item.take() == Some(id) && self.title.trim().is_empty() {
            if let Some(session) = &mut self.session {
                session.edit_silent(|lib| {
                    lib.tasks.retain(|t| t.id != id);
                    true
                });
            }
            self.rows.remove(&id);
            self.refresh(true);
        }
    }

    pub(super) fn expand(&mut self, id: Id) -> Task2 {
        self.collapse();
        let Some(task) = self.lib().and_then(|l| l.task(id)).cloned() else {
            return IcedTask::none();
        };
        if task.kind != Kind::Todo {
            return self.go(View::Project(id));
        }
        self.expanded = Some(id);
        self.selection = vec![id];
        self.anchor = Some(id);
        self.title = task.title;
        self.notes = text_editor::Content::with_text(&task.notes);
        self.checklist_input.clear();
        self.anim_mut(id).expand.to(1.0);
        IcedTask::batch([operation::focus(title_id()), operation::move_cursor_to_end(title_id())])
    }

    pub(super) fn open_sheet(&mut self, sheet: Sheet) -> Task2 {
        self.sheet = Some(sheet);
        self.sheet_anim.to(1.0);
        self.query.clear();
        self.sheet_index = 0;
        self.security_note = None;
        self.restore_armed = None;
        if sheet == Sheet::Settings {
            self.backups = crate::backup::list(&session::vault_path());
        }
        if matches!(sheet, Sheet::When | Sheet::Deadline) {
            self.cal_month = self.today.with_day(1).unwrap_or(self.today);
        }
        let focus = operation::focus(sheet_id());
        if matches!(sheet, Sheet::KeePass | Sheet::Settings) {
            return IcedTask::batch([focus, Self::probe(false)]);
        }
        focus
    }

    pub(super) fn close_sheet(&mut self) -> Task2 {
        self.sheet_anim.to(0.0);
        self.query.clear();
        if self.expanded.is_some() {
            operation::focus(title_id())
        } else {
            unfocus()
        }
    }
}
