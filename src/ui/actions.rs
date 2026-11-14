//! What commands do to the library: scheduling, moving, completing, dropping.

use super::*;

impl App {
    /// The items an action applies to: the selection, the open row, or the open project.
    pub(super) fn targets(&self) -> Vec<Id> {
        if !self.selection.is_empty() {
            return self.selection.clone();
        }
        if let Some(id) = self.expanded {
            return vec![id];
        }
        match self.view {
            View::Project(id) => vec![id],
            _ => Vec::new(),
        }
    }

    pub(super) fn edit_targets(&mut self, change: impl Fn(&mut Library, Id, NaiveDate)) {
        let targets = self.targets();
        let today = self.today;
        if targets.is_empty() {
            return;
        }
        if let Some(session) = &mut self.session {
            session.edit(|lib| {
                for id in targets {
                    change(lib, id, today);
                    lib.touch(id);
                }
            });
        }
        self.refresh(true);
    }

    pub(super) fn new_todo(&mut self) -> Task2 {
        let today = self.today;
        let view = self.view;
        let schedule = match view {
            View::Today => Schedule::On(today),
            View::Someday => Schedule::Someday,
            View::Anytime | View::Project(_) | View::Area(_) | View::Tag(_) => Schedule::Anytime,
            _ => Schedule::Inbox,
        };
        let target = if matches!(view, View::Inbox | View::Today | View::Anytime | View::Someday | View::Project(_) | View::Area(_) | View::Tag(_)) {
            view
        } else {
            View::Inbox
        };
        let mut task = Task::new(Kind::Todo, "", schedule);
        let id = task.id;
        let after = self.selection.last().copied();
        match view {
            View::Project(project) => {
                task.project = Some(project);
                task.heading = after.and_then(|a| self.lib().and_then(|l| l.task(a))).and_then(|t| t.heading);
            }
            View::Area(area) => task.area = Some(area),
            View::Tag(tag) => task.tags.push(tag),
            _ => {}
        }
        if let Some(tag) = self.tag_filter {
            task.tags.push(tag);
        }
        let mut order = self.flat.clone();
        let position = after.and_then(|a| order.iter().position(|x| *x == a)).map_or(order.len(), |p| p + 1);
        order.insert(position, id);
        if let Some(session) = &mut self.session {
            session.edit(|lib| {
                lib.insert(task);
                lib.apply_order(&order, view == View::Today);
            });
        }
        let go = self.go(target);
        self.refresh(true);
        self.new_item = None;
        let focus = self.expand(id);
        self.new_item = Some(id);
        IcedTask::batch([go, focus])
    }

    pub(super) fn view_for(&self, task: &Task) -> View {
        if task.logged {
            return View::Logbook;
        }
        if let Some(project) = task.project {
            return View::Project(project);
        }
        if task.is_today(self.today) {
            return View::Today;
        }
        match task.schedule {
            Schedule::Inbox => View::Inbox,
            Schedule::Anytime => View::Anytime,
            Schedule::Someday => View::Someday,
            Schedule::On(_) => View::Upcoming,
        }
    }

    pub(super) fn apply_act(&mut self, act: Act) -> Task2 {
        let close = self.close_sheet();
        let followup = match act {
            Act::When(when) => {
                self.edit_targets(move |lib, id, _| {
                    if let Some(task) = lib.task_mut(id) {
                        task.schedule = when.schedule;
                        task.evening = when.evening;
                        task.reminder = when.reminder;
                        task.reminded_on = None;
                    }
                });
                IcedTask::none()
            }
            Act::ClearReminder => {
                self.edit_targets(|lib, id, _| {
                    if let Some(task) = lib.task_mut(id) {
                        task.reminder = None;
                    }
                });
                IcedTask::none()
            }
            Act::Deadline(date) => {
                self.edit_targets(move |lib, id, _| {
                    if let Some(task) = lib.task_mut(id) {
                        task.deadline = date;
                        task.deadline_time = None;
                        task.deadline_alerted = false;
                    }
                });
                IcedTask::none()
            }
            Act::DeadlineAt(date, time) => {
                self.edit_targets(move |lib, id, _| {
                    if let Some(task) = lib.task_mut(id) {
                        task.deadline = Some(date);
                        task.deadline_time = Some(time);
                        task.deadline_alerted = false;
                    }
                });
                IcedTask::none()
            }
            Act::Move(target) => {
                self.edit_targets(move |lib, id, _| lib.move_to(id, target));
                IcedTask::none()
            }
            Act::ToggleTag(existing, name) => {
                let targets = self.targets();
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        let tag = existing.unwrap_or_else(|| lib.ensure_tag(&name));
                        let all_have = targets
                            .iter()
                            .all(|id| lib.task(*id).is_some_and(|t| t.tags.contains(&tag)));
                        for id in &targets {
                            if let Some(task) = lib.task_mut(*id) {
                                task.tags.retain(|t| *t != tag);
                                if !all_have {
                                    task.tags.push(tag);
                                }
                                task.modified = Utc::now();
                            }
                        }
                    });
                }
                self.refresh(true);
                self.query.clear();
                self.sheet_anim.to(1.0);
                return operation::focus(sheet_id());
            }
            Act::Go(view) => self.go(view),
            Act::Reveal(id) => {
                let target = self.lib().and_then(|l| l.task(id)).map(|task| self.view_for(task));
                let go = target.map_or_else(IcedTask::none, |view| self.go(view));
                self.selection = vec![id];
                self.anchor = Some(id);
                go
            }
            Act::Repeat(rule) => {
                let today = self.today;
                let targets = self.targets();
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        for id in targets {
                            let Some(task) = lib.task_mut(id) else { continue };
                            match &rule {
                                Some(rule) => {
                                    task.rule = Some(rule.clone());
                                    task.schedule = Schedule::Someday;
                                }
                                None => task.rule = None,
                            }
                        }
                        lib.spawn_repeats(today);
                    });
                }
                self.refresh(true);
                IcedTask::none()
            }
            Act::Run(cmd) => self.run(cmd),
            Act::Create(prompt, text) => self.create_from_prompt(prompt, text),
        };
        IcedTask::batch([close, followup])
    }

    pub(super) fn create_from_prompt(&mut self, prompt: Prompt, text: String) -> Task2 {
        let name = text.trim().to_owned();
        if name.is_empty() {
            return IcedTask::none();
        }
        let view = self.view;
        let area = if let View::Area(id) = view { Some(id) } else { None };
        let Some(session) = &mut self.session else { return IcedTask::none() };
        let mut go_to = None;
        match prompt {
            Prompt::NewProject => session.edit(|lib| {
                let mut project = Task::new(Kind::Project, name, Schedule::Anytime);
                project.area = area;
                go_to = Some(View::Project(lib.insert(project)));
            }),
            Prompt::NewArea => session.edit(|lib| go_to = Some(View::Area(lib.add_area(name)))),
            Prompt::NewHeading => {
                if let View::Project(project) = view {
                    session.edit(|lib| {
                        let mut heading = Task::new(Kind::Heading, name, Schedule::Anytime);
                        heading.project = Some(project);
                        lib.insert(heading);
                    });
                }
            }
            Prompt::Rename(id) => session.edit(|lib| {
                if let Some(task) = lib.task_mut(id) {
                    task.title = name;
                } else if let Some(area) = lib.areas.iter_mut().find(|a| a.id == id) {
                    area.title = name;
                } else if let Some(tag) = lib.tags.iter_mut().find(|t| t.id == id) {
                    tag.title = name;
                }
            }),
        }
        self.refresh(true);
        go_to.map_or_else(IcedTask::none, |view| self.go(view))
    }

    pub(super) fn set_status_on_targets(&mut self, status: Status) {
        let today = self.today;
        let targets = self.targets();
        if let Some(session) = &mut self.session {
            session.edit(|lib| {
                let all_set = targets.iter().all(|id| lib.task(*id).is_some_and(|t| t.status == status));
                for id in &targets {
                    lib.set_status(*id, if all_set { Status::Open } else { status }, today);
                }
            });
        }
        self.sync_checks(&targets);
        self.pending_log = Some(Instant::now());
        self.refresh(true);
    }

    pub(super) fn sync_checks(&mut self, ids: &[Id]) {
        for id in ids {
            let done = self.lib().and_then(|l| l.task(*id)).is_some_and(|t| !t.is_open());
            self.anim_mut(*id).check.to(if done { 1.0 } else { 0.0 });
        }
    }

    pub(super) fn reorder(&mut self, delta: isize) {
        let selected: HashSet<Id> = self.selection.iter().copied().collect();
        if selected.is_empty() {
            return;
        }
        let mut order = self.flat.clone();
        let indices: Vec<usize> = (0..order.len()).filter(|i| selected.contains(&order[*i])).collect();
        let walk: Vec<usize> = if delta < 0 { indices } else { indices.into_iter().rev().collect() };
        for index in walk {
            let Some(next) = index.checked_add_signed(delta).filter(|n| *n < order.len()) else { return };
            if selected.contains(&order[next]) {
                return;
            }
            order.swap(index, next);
        }
        let today_list = self.view == View::Today;
        if let Some(session) = &mut self.session {
            session.edit(|lib| lib.apply_order(&order, today_list));
        }
        self.refresh(false);
    }

    pub(super) fn run(&mut self, cmd: Cmd) -> Task2 {
        match cmd {
            Cmd::NewTodo => return self.new_todo(),
            Cmd::QuickEntry => return self.open_quick(),
            Cmd::NewProject => return self.open_sheet(Sheet::Prompt(Prompt::NewProject)),
            Cmd::NewArea => return self.open_sheet(Sheet::Prompt(Prompt::NewArea)),
            Cmd::NewHeading => {
                if matches!(self.view, View::Project(_)) {
                    return self.open_sheet(Sheet::Prompt(Prompt::NewHeading));
                }
            }
            Cmd::Find => return self.open_sheet(Sheet::Find),
            Cmd::Palette => return self.open_sheet(Sheet::Palette),
            Cmd::Help => return self.open_sheet(Sheet::Help),
            Cmd::Settings => return self.open_sheet(Sheet::Settings),
            Cmd::Lock => {
                self.lock();
                return operation::focus(auth_id());
            }
            Cmd::Complete => {
                let count = self.targets().len();
                self.set_status_on_targets(Status::Completed);
                self.say(count, "Completed", "to-dos completed");
            }
            Cmd::Cancel => {
                let count = self.targets().len();
                self.set_status_on_targets(Status::Canceled);
                self.say(count, "Canceled", "to-dos canceled");
            }
            Cmd::SetToday | Cmd::SetEvening => {
                let evening = cmd == Cmd::SetEvening;
                self.edit_targets(move |lib, id, today| {
                    if let Some(task) = lib.task_mut(id) {
                        task.schedule = Schedule::On(today);
                        task.evening = evening;
                    }
                });
            }
            Cmd::SetAnytime | Cmd::SetSomeday => {
                let schedule = if cmd == Cmd::SetAnytime { Schedule::Anytime } else { Schedule::Someday };
                self.edit_targets(move |lib, id, _| {
                    if let Some(task) = lib.task_mut(id) {
                        task.schedule = schedule;
                        task.evening = false;
                    }
                });
            }
            Cmd::When | Cmd::Deadline | Cmd::MoveTo | Cmd::Tags | Cmd::Repeat => {
                if self.targets().is_empty() {
                    return IcedTask::none();
                }
                return self.open_sheet(match cmd {
                    Cmd::When => Sheet::When,
                    Cmd::Deadline => Sheet::Deadline,
                    Cmd::MoveTo => Sheet::MoveTo,
                    Cmd::Tags => Sheet::Tags,
                    _ => Sheet::Repeat,
                });
            }
            Cmd::Duplicate => {
                let targets = self.targets();
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        for id in targets {
                            lib.duplicate(id);
                        }
                    });
                }
                self.refresh(true);
            }
            Cmd::ConvertToProject => self.edit_targets(|lib, id, _| lib.convert_to_project(id)),
            Cmd::Delete => {
                let count = self.targets().len();
                self.expanded = None;
                self.new_item = None;
                self.edit_targets(|lib, id, _| lib.set_trashed(id, true));
                self.say(count, "Moved to Trash", "to-dos moved to Trash");
            }
            Cmd::PutBack => self.edit_targets(|lib, id, _| lib.set_trashed(id, false)),
            Cmd::Rename => {
                let id = match self.view {
                    View::Project(id) | View::Area(id) | View::Tag(id) if self.selection.is_empty() => Some(id),
                    _ => self.selection.first().copied(),
                };
                if let Some(id) = id {
                    return self.open_sheet(Sheet::Prompt(Prompt::Rename(id)));
                }
            }
            Cmd::Undo | Cmd::Redo => {
                self.toast = None;
                if let Some(session) = &mut self.session {
                    if cmd == Cmd::Undo { session.undo() } else { session.redo() };
                }
                self.expanded = None;
                self.new_item = None;
                self.rows.clear();
                self.refresh(true);
                self.load_project_notes();
            }
            Cmd::MoveUp => self.reorder(-1),
            Cmd::MoveDown => self.reorder(1),
            Cmd::SelectAll => self.selection = self.flat.clone(),
            Cmd::EmptyTrash => {
                if let Some(session) = &mut self.session {
                    session.edit(Library::empty_trash);
                }
                self.refresh(true);
            }
            Cmd::LogNow => {
                if let Some(session) = &mut self.session {
                    session.edit(Library::log_now);
                }
                self.refresh(true);
            }
            Cmd::DeleteList => {
                let view = self.view;
                if let Some(session) = &mut self.session {
                    match view {
                        View::Area(id) => session.edit(|lib| lib.delete_area(id)),
                        View::Tag(id) => session.edit(|lib| lib.delete_tag(id)),
                        View::Project(id) => session.edit(|lib| lib.set_trashed(id, true)),
                        _ => return IcedTask::none(),
                    }
                }
                return self.go(View::Inbox);
            }
            Cmd::ZoomIn => self.zoom(None, 0.1),
            Cmd::ZoomOut => self.zoom(None, -0.1),
            Cmd::ZoomReset => self.zoom(Some(1.0), 0.0),
            Cmd::ConnectKeePass => return self.open_sheet(Sheet::KeePass),
            Cmd::DisconnectKeePass => return self.update(Message::KeePassDisconnect),
            Cmd::Go(view) => return self.go(view),
        }
        IcedTask::none()
    }

    /// Shows the undo note: "Completed" for one to-do, "3 to-dos completed" for more.
    pub(super) fn say(&mut self, count: usize, one: &'static str, many: &'static str) {
        if count == 0 {
            return;
        }
        let text = if count == 1 { t(one).to_owned() } else { format!("{count} {}", t(many)) };
        self.toast = Some(Toast { text, at: Instant::now() });
    }

    pub(super) fn drop_on(&mut self, target: DropTarget, ids: Vec<Id>) {
        let today = self.today;
        match target {
            DropTarget::Row(over) => self.drop_on_row(over, ids),
            DropTarget::Side(view) => {
                let Some(session) = &mut self.session else { return };
                session.edit(|lib| {
                    for id in ids {
                        match view {
                            View::Inbox => lib.move_to(id, MoveTarget::Inbox),
                            View::Project(project) => lib.move_to(id, MoveTarget::Project(project, None)),
                            View::Area(area) => lib.move_to(id, MoveTarget::Area(area)),
                            View::Trash => lib.set_trashed(id, true),
                            View::Logbook => lib.set_status(id, Status::Completed, today),
                            View::Today | View::Anytime | View::Someday | View::Upcoming => {
                                if let Some(task) = lib.task_mut(id) {
                                    task.evening = false;
                                    task.schedule = match view {
                                        View::Today => Schedule::On(today),
                                        View::Someday => Schedule::Someday,
                                        _ => Schedule::Anytime,
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                });
                self.refresh(true);
            }
        }
    }

    pub(super) fn drop_on_row(&mut self, over: Id, ids: Vec<Id>) {
        if ids.contains(&over) {
            return;
        }
        let section = self.sections.iter().find(|s| s.rows.contains(&over)).cloned();
        let mut order: Vec<Id> = self.flat.iter().copied().filter(|id| !ids.contains(id)).collect();
        let at = order.iter().position(|id| *id == over).unwrap_or(order.len());
        order.splice(at..at, ids.iter().copied());
        let today_list = self.view == View::Today;
        let in_project = matches!(self.view, View::Project(_));
        if let Some(session) = &mut self.session {
            session.edit(|lib| {
                lib.apply_order(&order, today_list);
                for id in &ids {
                    let Some(task) = lib.task_mut(*id) else { continue };
                    if today_list {
                        task.evening = section.as_ref().is_some_and(|s| s.evening);
                    }
                    if in_project {
                        task.heading = section.as_ref().and_then(|s| s.heading);
                    }
                }
            });
        }
        self.refresh(false);
    }

    pub(super) fn with_task_edit(&mut self, change: impl FnOnce(&mut Task)) {
        let Some(id) = self.expanded else { return };
        if let Some(session) = &mut self.session {
            session.edit_coalesced(format!("detail:{id}"), |lib| {
                if let Some(task) = lib.task_mut(id) {
                    change(task);
                    task.modified = Utc::now();
                }
            });
        }
    }

    pub(super) fn pick(&mut self, index: usize) -> Task2 {
        if matches!(self.sheet, Some(Sheet::Settings | Sheet::Help | Sheet::KeePass)) {
            return IcedTask::none();
        }
        match self.sheet_items().into_iter().nth(index).map(|item| item.act) {
            Some(act) => self.apply_act(act),
            None => IcedTask::none(),
        }
    }

    /// Messages about editing to-dos, and the ways back from it.
    pub(super) fn update_edit(&mut self, message: Message) -> Task2 {
        match message {
            Message::Toggle(id) => {
                let today = self.today;
                if self.lib().and_then(|l| l.task(id)).is_some_and(|t| t.status == Status::Open) {
                    self.say(1, "Completed", "to-dos completed");
                }
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        let next = match lib.task(id).map(|t| t.status) {
                            Some(Status::Open) => Status::Completed,
                            _ => Status::Open,
                        };
                        lib.set_status(id, next, today);
                    });
                }
                self.sync_checks(&[id]);
                self.pending_log = Some(Instant::now());
                self.refresh(true);
            }
            Message::HeadingRename(id) => return self.open_sheet(Sheet::Prompt(Prompt::Rename(id))),
            Message::HeadingArchive(id) => {
                let today = self.today;
                if let Some(session) = &mut self.session {
                    session.edit(|lib| lib.set_status(id, Status::Completed, today));
                }
                self.pending_log = Some(Instant::now());
                self.refresh(true);
            }
            Message::HeadingDelete(id) => {
                if let Some(session) = &mut self.session {
                    session.edit(|lib| lib.set_trashed(id, true));
                }
                self.refresh(true);
            }
            Message::Title(value) => {
                self.with_task_edit(|task| task.title.clone_from(&value));
                self.title = value;
            }
            Message::TitleDone => {
                self.collapse();
                return unfocus();
            }
            Message::Notes(action) => {
                let edit = action.is_edit();
                self.notes.perform(action);
                if edit {
                    let text = self.notes.text().trim_end_matches('\n').to_owned();
                    self.with_task_edit(|task| task.notes = text);
                }
            }
            Message::ProjectNotes(action) => {
                let edit = action.is_edit();
                self.project_notes.perform(action);
                if edit {
                    if let (View::Project(id), Some(session)) = (self.view, &mut self.session) {
                        let text = self.project_notes.text().trim_end_matches('\n').to_owned();
                        session.edit_coalesced(format!("project:{id}"), |lib| {
                            if let Some(task) = lib.task_mut(id) {
                                task.notes = text;
                            }
                        });
                    }
                }
            }
            Message::ChecklistInput(value) => self.checklist_input = value,
            Message::ChecklistAdd => {
                let title = std::mem::take(&mut self.checklist_input);
                if !title.trim().is_empty() {
                    let Some(id) = self.expanded else { return IcedTask::none() };
                    if let Some(session) = &mut self.session {
                        session.edit(|lib| {
                            if let Some(task) = lib.task_mut(id) {
                                task.checklist.push(ChecklistItem { id: Uuid::new_v4(), title, done: false });
                            }
                        });
                    }
                }
            }
            Message::ChecklistToggle(item) => {
                let Some(id) = self.expanded else { return IcedTask::none() };
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        let entry = lib.task_mut(id).and_then(|t| t.checklist.iter_mut().find(|c| c.id == item));
                        if let Some(entry) = entry {
                            entry.done = !entry.done;
                        }
                    });
                }
            }
            Message::ChecklistEdit(item, value) => {
                self.with_task_edit(|task| {
                    if let Some(entry) = task.checklist.iter_mut().find(|c| c.id == item) {
                        entry.title = value;
                    }
                });
            }
            Message::ChecklistRemove(item) => {
                let Some(id) = self.expanded else { return IcedTask::none() };
                if let Some(session) = &mut self.session {
                    session.edit(|lib| {
                        if let Some(task) = lib.task_mut(id) {
                            task.checklist.retain(|c| c.id != item);
                        }
                    });
                }
            }
            Message::Notified(sent) => self.notifications.extend(sent),
            Message::NotificationAction(id, key) => {
                let Some(task) = self.notifications.remove(&id) else { return IcedTask::none() };
                let (today, now) = (self.today, Local::now().time());
                if let Some(session) = &mut self.session {
                    match key.as_str() {
                        crate::notify::DONE => session.edit(|lib| lib.set_status(task, Status::Completed, today)),
                        crate::notify::LATER => session.edit(|lib| lib.snooze(task, today, now, 60)),
                        _ => {}
                    }
                    self.sync_checks(&[task]);
                    self.pending_log = Some(Instant::now());
                    self.refresh(true);
                }
            }
            Message::ToastUndo => {
                self.toast = None;
                return self.run(Cmd::Undo);
            }
            _ => {}
        }
        IcedTask::none()
    }
}
