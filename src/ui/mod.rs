//! Application state and message handling.

mod keys;
mod motion;
mod sheet;
mod style;
mod view;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate, Utc};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{Id as WidgetId, operation, text_editor};
use iced::{Event, Point, Size, Subscription, Task as IcedTask, Theme, event, mouse, time, window};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::i18n::t;
use crate::ipc;
use crate::keepass;
use crate::model::{
    ChecklistItem, Id, Kind, Library, MoveTarget, Schedule, Section, Status, Task, View, local_today,
};
use crate::parse;
use crate::session::{self, Session};
use crate::vault::{self, MIN_PASSWORD_CHARS, Secret, VaultError};

use keys::Cmd;
use motion::Spring;
use sheet::{Act, Prompt, Sheet};
use style::Palette;

pub use style::{DARK, LIGHT};

pub fn default_font() -> iced::Font {
    style::FONT
}

type Task2 = IcedTask<Message>;
type Slot<T> = Arc<Mutex<Option<T>>>;

const QUICK_SIZE: Size = Size::new(580.0, 102.0);
const LOG_DELAY: Duration = Duration::from_millis(1100);

#[derive(Clone)]
pub enum Message {
    Opened(window::Id),
    Closed(window::Id),
    Ipc(ipc::Command),
    Frame(Instant),
    Tick,
    Key(KeyPress),
    ModifiersChanged(Modifiers),
    CursorMoved(Point),
    ButtonReleased,
    SystemDark(bool),

    AuthPassword(String),
    AuthRepeat(String),
    AuthSubmit,
    AuthKeePass,
    AuthDone(Slot<Result<Session, String>>),
    KeePassProbe(keepass::Status, bool),

    Go(View),
    ToggleArea(Id),
    TagFilter(Option<Id>),

    RowPress(Id),
    RowOpen(Id),
    RowMenu(Id),
    RowHover(Id),
    SideHover(View),
    Toggle(Id),
    HeadingRename(Id),
    HeadingArchive(Id),
    HeadingDelete(Id),

    Title(String),
    TitleDone,
    Notes(text_editor::Action),
    ProjectNotes(text_editor::Action),
    ChecklistInput(String),
    ChecklistAdd,
    ChecklistToggle(Id),
    ChecklistEdit(Id, String),
    ChecklistRemove(Id),

    Command(Cmd),
    SheetQuery(String),
    SheetSubmit,
    SheetPick(usize),
    SheetClose,

    Setting(SettingChange),
    PasswordOld(String),
    PasswordNew(String),
    PasswordChange,
    SecurityDone(Slot<Result<String, String>>),
    KeePassConnect,
    KeePassDisconnect,

    QuickInput(String),
    QuickSubmit,
    QuickComplete(usize),
    Ignored,
}

#[derive(Clone, Debug)]
pub struct KeyPress {
    window: window::Id,
    key: Key,
    latin: Option<char>,
    modifiers: Modifiers,
    text: Option<String>,
    captured: bool,
}

#[derive(Clone, Copy)]
pub enum SettingChange {
    Appearance(crate::model::Appearance),
    Scale(f32),
    Log(crate::model::LogInterval),
    GroupToday(bool),
    ReduceMotion(bool),
    AutoLock(u32),
    ReminderTitles(bool),
    QuickToday(bool),
}

pub enum Auth {
    Setup,
    Locked { has_keepass: bool },
    Open,
}

#[derive(Clone, Copy)]
pub struct RowAnim {
    pub appear: Spring,
    pub expand: Spring,
    pub check: Spring,
}

impl RowAnim {
    fn rest(done: bool) -> Self {
        Self {
            appear: Spring::at(1.0),
            expand: Spring::at(0.0),
            check: Spring::at(if done { 1.0 } else { 0.0 }),
        }
    }

    fn settled(&self) -> bool {
        self.appear.settled() && self.expand.settled() && self.check.settled()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum DropTarget {
    Row(Id),
    Side(View),
}

pub struct App {
    pub main: Option<window::Id>,
    pub quick: Option<window::Id>,
    pub auth: Auth,
    pub session: Option<Session>,
    pub system_dark: bool,

    pub password: String,
    pub repeat: String,
    pub auth_error: Option<String>,
    pub auth_busy: bool,
    pub keepass: keepass::Status,
    skip_auto_unlock: bool,

    pub view: View,
    pub tag_filter: Option<Id>,
    pub sections: Vec<Section>,
    pub flat: Vec<Id>,
    pub selection: Vec<Id>,
    pub anchor: Option<Id>,
    pub expanded: Option<Id>,
    new_item: Option<Id>,
    pub title: String,
    pub notes: text_editor::Content,
    pub project_notes: text_editor::Content,
    pub checklist_input: String,

    pub rows: HashMap<Id, RowAnim>,
    pub ghosts: HashSet<Id>,
    pub view_anim: Spring,
    pub sheet: Option<Sheet>,
    pub sheet_anim: Spring,
    pub quick_anim: Spring,
    quick_closing: bool,
    pub query: String,
    pub sheet_index: usize,

    pub drag: Option<Vec<Id>>,
    pub drop_target: Option<DropTarget>,
    pressed: Option<(Id, Option<Point>)>,
    pub cursor: Point,

    pub quick_text: String,
    pub modifiers: Modifiers,

    pub password_old: String,
    pub password_new: String,
    pub security_note: Option<String>,

    last_frame: Option<Instant>,
    last_activity: Instant,
    last_probe: Instant,
    pending_log: Option<Instant>,
    today: NaiveDate,
}

fn title_id() -> WidgetId {
    WidgetId::new("todo-title")
}

fn sheet_id() -> WidgetId {
    WidgetId::new("sheet-input")
}

fn auth_id() -> WidgetId {
    WidgetId::new("auth-password")
}

fn quick_id() -> WidgetId {
    WidgetId::new("quick-input")
}

fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    map: impl Fn(T) -> Message + Send + 'static,
) -> Task2 {
    IcedTask::perform(
        async move { tokio::task::spawn_blocking(work).await.expect("background task panicked") },
        map,
    )
}

fn unfocus() -> Task2 {
    iced::advanced::widget::operate(iced::advanced::widget::operation::focusable::unfocus())
}

fn slot<T>(value: T) -> Slot<T> {
    Arc::new(Mutex::new(Some(value)))
}

#[cfg(target_os = "linux")]
fn platform(app_id: &str) -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific { application_id: app_id.to_owned(), ..Default::default() }
}

#[cfg(not(target_os = "linux"))]
fn platform(_: &str) -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

fn main_settings() -> window::Settings {
    window::Settings {
        size: Size::new(980.0, 680.0),
        min_size: Some(Size::new(560.0, 380.0)),
        platform_specific: platform("gotodo"),
        ..window::Settings::default()
    }
}

/// Fixed-size and undecorated. Sway and i3 float a toplevel whose minimum and
/// maximum size match, so this opens as a small centered popup instead of
/// being tiled. It stays opaque because software rendering has no alpha.
fn quick_settings() -> window::Settings {
    window::Settings {
        size: QUICK_SIZE,
        min_size: Some(QUICK_SIZE),
        max_size: Some(QUICK_SIZE),
        resizable: false,
        decorations: false,
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        platform_specific: platform("gotodo-quick"),
        ..window::Settings::default()
    }
}

impl App {
    pub fn boot(start_quick: bool) -> (Self, Task2) {
        let path = session::vault_path();
        let auth = if vault::exists(&path) {
            Auth::Locked { has_keepass: vault::peek(&path).map(|(_, k)| k).unwrap_or(false) }
        } else {
            Auth::Setup
        };
        let mut app = Self {
            main: None,
            quick: None,
            auth,
            session: None,
            system_dark: false,
            password: String::new(),
            repeat: String::new(),
            auth_error: None,
            auth_busy: false,
            keepass: keepass::Status::Missing,
            skip_auto_unlock: false,
            view: View::Today,
            tag_filter: None,
            sections: Vec::new(),
            flat: Vec::new(),
            selection: Vec::new(),
            anchor: None,
            expanded: None,
            new_item: None,
            title: String::new(),
            notes: text_editor::Content::new(),
            project_notes: text_editor::Content::new(),
            checklist_input: String::new(),
            rows: HashMap::new(),
            ghosts: HashSet::new(),
            view_anim: Spring::at(1.0),
            sheet: None,
            sheet_anim: Spring::at(0.0),
            quick_anim: Spring::at(1.0),
            quick_closing: false,
            query: String::new(),
            sheet_index: 0,
            drag: None,
            drop_target: None,
            pressed: None,
            cursor: Point::ORIGIN,
            quick_text: String::new(),
            modifiers: Modifiers::default(),
            password_old: String::new(),
            password_new: String::new(),
            security_note: None,
            last_frame: None,
            last_activity: Instant::now(),
            last_probe: Instant::now(),
            pending_log: None,
            today: local_today(),
        };
        let needs_setup = matches!(app.auth, Auth::Setup);
        let open = if start_quick && !needs_setup { app.open_quick() } else { app.open_main() };
        let theme = iced::system::theme().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark));
        (app, IcedTask::batch([open, theme, Self::probe(true)]))
    }

    fn open_main(&mut self) -> Task2 {
        let (id, open) = window::open(main_settings());
        self.main = Some(id);
        open.map(Message::Opened)
    }

    fn open_quick(&mut self) -> Task2 {
        if self.quick.is_some() {
            return IcedTask::none();
        }
        let (id, open) = window::open(quick_settings());
        self.quick = Some(id);
        self.quick_text.clear();
        self.quick_closing = false;
        self.quick_anim = Spring::at(0.0);
        self.quick_anim.to(1.0);
        open.map(Message::Opened)
    }

    fn probe(auto: bool) -> Task2 {
        blocking(keepass::status, move |status| Message::KeePassProbe(status, auto))
    }

    pub fn palette(&self) -> Palette {
        use crate::model::Appearance as A;
        let dark = match self.session.as_ref().map(|s| s.lib.settings.appearance) {
            Some(A::Light) => false,
            Some(A::Dark) => true,
            _ => self.system_dark,
        };
        if dark { DARK } else { LIGHT }
    }

    pub fn theme(&self, _: window::Id) -> Theme {
        style::base_theme(&self.palette())
    }

    pub fn scale(&self, _: window::Id) -> f32 {
        self.session.as_ref().map_or(1.0, |s| s.lib.settings.text_scale)
    }

    pub fn window_title(&self, id: window::Id) -> String {
        if Some(id) == self.quick { "gotodo quick entry".to_owned() } else { "gotodo".to_owned() }
    }

    pub fn lib(&self) -> Option<&Library> {
        self.session.as_ref().map(|s| &s.lib)
    }

    fn reduce_motion(&self) -> bool {
        self.lib().is_some_and(|l| l.settings.reduce_motion)
    }

    pub fn today(&self) -> NaiveDate {
        self.today
    }

    pub fn anim(&self, id: Id) -> RowAnim {
        self.rows.get(&id).copied().unwrap_or_else(|| {
            RowAnim::rest(self.lib().and_then(|l| l.task(id)).is_some_and(|t| !t.is_open()))
        })
    }

    fn anim_mut(&mut self, id: Id) -> &mut RowAnim {
        let rest = self.anim(id);
        self.rows.entry(id).or_insert(rest)
    }

    fn animating(&self) -> bool {
        !self.view_anim.settled()
            || !self.sheet_anim.settled()
            || !self.quick_anim.settled()
            || !self.ghosts.is_empty()
            || self.rows.values().any(|a| !a.settled())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            event::listen_with(global_event),
            window::close_events().map(Message::Closed),
            time::every(Duration::from_millis(500)).map(|_| Message::Tick),
            Subscription::run(ipc::listen).map(Message::Ipc),
            iced::system::theme_changes().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark)),
        ];
        if self.animating() {
            subs.push(window::frames().map(Message::Frame));
        }
        if self.pressed.is_some() {
            subs.push(event::listen_with(cursor_event));
        }
        Subscription::batch(subs)
    }

    /// Recomputes the visible list, animating rows in and out when it is the same view.
    fn refresh(&mut self, animate: bool) {
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

    fn go(&mut self, view: View) -> Task2 {
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

    fn load_project_notes(&mut self) {
        let notes = match self.view {
            View::Project(id) => self.lib().and_then(|l| l.task(id)).map(|t| t.notes.clone()),
            _ => None,
        };
        self.project_notes = text_editor::Content::with_text(&notes.unwrap_or_default());
    }

    fn collapse(&mut self) {
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

    fn expand(&mut self, id: Id) -> Task2 {
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

    /// The items an action applies to: the selection, the open row, or the open project.
    fn targets(&self) -> Vec<Id> {
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

    fn edit_targets(&mut self, change: impl Fn(&mut Library, Id, NaiveDate)) {
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

    fn new_todo(&mut self) -> Task2 {
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

    fn open_sheet(&mut self, sheet: Sheet) -> Task2 {
        self.sheet = Some(sheet);
        self.sheet_anim.to(1.0);
        self.query.clear();
        self.sheet_index = 0;
        self.security_note = None;
        let focus = operation::focus(sheet_id());
        if matches!(sheet, Sheet::KeePass | Sheet::Settings) {
            return IcedTask::batch([focus, Self::probe(false)]);
        }
        focus
    }

    fn close_sheet(&mut self) -> Task2 {
        self.sheet_anim.to(0.0);
        self.query.clear();
        if self.expanded.is_some() {
            operation::focus(title_id())
        } else {
            unfocus()
        }
    }

    fn lock(&mut self) {
        let has_keepass = self
            .session
            .as_ref()
            .map(|s| s.vault.lock().expect("vault lock").has_keepass())
            .unwrap_or(false);
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
        self.title.zeroize();
        self.query.zeroize();
        self.checklist_input.zeroize();
        self.quick_text.zeroize();
        self.notes = text_editor::Content::new();
        self.project_notes = text_editor::Content::new();
    }

    fn finish_unlock(&mut self, session: Session) -> Task2 {
        self.session = Some(session);
        self.auth = Auth::Open;
        self.auth_busy = false;
        self.auth_error = None;
        self.password.zeroize();
        self.repeat.zeroize();
        self.today = local_today();
        self.last_activity = Instant::now();
        self.run_daily();
        self.refresh(false);
        self.view_anim = Spring::at(0.0);
        self.view_anim.to(1.0);
        if self.quick.is_some() { operation::focus(quick_id()) } else { IcedTask::none() }
    }

    fn run_daily(&mut self) {
        let today = self.today;
        if let Some(session) = &mut self.session {
            session.edit_silent(|lib| {
                let spawned = lib.spawn_repeats(today);
                lib.sweep_logbook(today) || spawned
            });
        }
    }

    fn unlock_with_password(&mut self) -> Task2 {
        if self.auth_busy || self.password.is_empty() {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
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

    fn unlock_with_keepass(&mut self) -> Task2 {
        if self.auth_busy {
            return IcedTask::none();
        }
        self.auth_busy = true;
        self.auth_error = None;
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

    fn create_vault(&mut self) -> Task2 {
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
        let password = Zeroizing::new(std::mem::take(&mut self.password));
        self.repeat.zeroize();
        blocking(
            move || Session::create(&password).map_err(|e| e.to_string()),
            |result| Message::AuthDone(slot(result)),
        )
    }

    fn view_for(&self, task: &Task) -> View {
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

    fn apply_act(&mut self, act: Act) -> Task2 {
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

    fn create_from_prompt(&mut self, prompt: Prompt, text: String) -> Task2 {
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

    fn set_status_on_targets(&mut self, status: Status) {
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

    fn sync_checks(&mut self, ids: &[Id]) {
        for id in ids {
            let done = self.lib().and_then(|l| l.task(*id)).is_some_and(|t| !t.is_open());
            self.anim_mut(*id).check.to(if done { 1.0 } else { 0.0 });
        }
    }

    fn reorder(&mut self, delta: isize) {
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

    fn run(&mut self, cmd: Cmd) -> Task2 {
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
            Cmd::Complete => self.set_status_on_targets(Status::Completed),
            Cmd::Cancel => self.set_status_on_targets(Status::Canceled),
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
                self.expanded = None;
                self.new_item = None;
                self.edit_targets(|lib, id, _| lib.set_trashed(id, true));
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
            Cmd::ConnectKeePass => return self.open_sheet(Sheet::KeePass),
            Cmd::DisconnectKeePass => return self.update(Message::KeePassDisconnect),
            Cmd::Go(view) => return self.go(view),
        }
        IcedTask::none()
    }

    fn on_key(&mut self, press: KeyPress) -> Task2 {
        self.last_activity = Instant::now();
        if Some(press.window) == self.quick {
            return self.on_quick_key(&press);
        }
        if !matches!(self.auth, Auth::Open) {
            if matches!(press.key.as_ref(), Key::Named(Named::Tab)) {
                return operation::focus_next();
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

    fn step_selection(&mut self, delta: isize, extend: bool) {
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

    fn on_quick_key(&mut self, press: &KeyPress) -> Task2 {
        match press.key.as_ref() {
            Key::Named(Named::Escape) => self.close_quick(),
            Key::Named(Named::Tab) => {
                if matches!(self.auth, Auth::Open) { self.complete_quick(0) } else { operation::focus_next() }
            }
            _ => IcedTask::none(),
        }
    }

    pub fn quick_completions(&self) -> Vec<(String, String)> {
        self.lib().map_or_else(Vec::new, |lib| parse::completions(&self.quick_text, lib))
    }

    fn complete_quick(&mut self, index: usize) -> Task2 {
        if let Some((replacement, _)) = self.quick_completions().get(index) {
            self.quick_text = parse::apply_completion(&self.quick_text, replacement);
        }
        IcedTask::batch([operation::focus(quick_id()), operation::move_cursor_to_end(quick_id())])
    }

    fn close_quick(&mut self) -> Task2 {
        let Some(id) = self.quick else { return IcedTask::none() };
        if self.reduce_motion() {
            return window::close(id);
        }
        // Ease out first; on_frame closes the window once the spring settles.
        if !self.quick_closing {
            self.quick_closing = true;
            self.quick_anim.to(0.0);
        }
        IcedTask::none()
    }

    fn submit_quick(&mut self) -> Task2 {
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

    fn drop_on(&mut self, target: DropTarget, ids: Vec<Id>) {
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

    fn drop_on_row(&mut self, over: Id, ids: Vec<Id>) {
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

    fn on_tick(&mut self) -> Task2 {
        if self.session.is_none() {
            if self.last_probe.elapsed() >= Duration::from_secs(3) {
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
        if !due.is_empty() {
            let show_titles = self.lib().is_some_and(|l| l.settings.reminder_shows_title);
            let body = due
                .iter()
                .filter_map(|id| self.lib().and_then(|l| l.task(*id)))
                .map(|task| if show_titles { task.title.clone() } else { t("A to-do is due").to_owned() })
                .collect::<Vec<_>>();
            tasks.push(blocking(move || notify(&body), |()| Message::Ignored));
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

    fn on_frame(&mut self, now: Instant) -> Task2 {
        let dt = self.last_frame.map_or(1.0 / 60.0, |last| (now - last).as_secs_f32());
        self.last_frame = Some(now);
        let reduce = self.reduce_motion();
        self.view_anim.step(dt, reduce);
        self.sheet_anim.step(dt, reduce);
        self.quick_anim.step(dt, reduce);
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
        if self.quick_closing && self.quick_anim.settled() {
            self.quick_closing = false;
            if let Some(id) = self.quick {
                return window::close(id);
            }
        }
        IcedTask::none()
    }

    fn change_setting(&mut self, change: SettingChange) {
        let Some(session) = &mut self.session else { return };
        session.edit_silent(|lib| {
            let s = &mut lib.settings;
            match change {
                SettingChange::Appearance(v) => s.appearance = v,
                SettingChange::Scale(v) => s.text_scale = v.clamp(0.85, 1.4),
                SettingChange::Log(v) => s.log_interval = v,
                SettingChange::GroupToday(v) => s.group_today_by_parent = v,
                SettingChange::ReduceMotion(v) => s.reduce_motion = v,
                SettingChange::AutoLock(v) => s.auto_lock_minutes = v,
                SettingChange::ReminderTitles(v) => s.reminder_shows_title = v,
                SettingChange::QuickToday(v) => s.quick_entry_to_today = v,
            }
            true
        });
        self.refresh(false);
    }

    fn security_task(
        &mut self,
        work: impl FnOnce(Arc<Mutex<vault::Vault>>, Zeroizing<Vec<u8>>) -> Result<String, String> + Send + 'static,
    ) -> Task2 {
        let Some(session) = &self.session else { return IcedTask::none() };
        let vault = Arc::clone(&session.vault);
        let plain = session.plaintext();
        blocking(move || work(vault, plain), |result| Message::SecurityDone(slot(result)))
    }

    fn with_task_edit(&mut self, change: impl FnOnce(&mut Task)) {
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

    pub fn update(&mut self, message: Message) -> Task2 {
        match message {
            Message::Opened(id) => {
                if Some(id) == self.quick {
                    return if matches!(self.auth, Auth::Open) {
                        operation::focus(quick_id())
                    } else {
                        operation::focus(auth_id())
                    };
                }
                if !matches!(self.auth, Auth::Open) {
                    return operation::focus(auth_id());
                }
            }
            Message::Closed(id) => {
                if Some(id) == self.quick {
                    self.quick = None;
                    self.quick_closing = false;
                    self.quick_text.zeroize();
                }
                if Some(id) == self.main {
                    self.main = None;
                }
                if self.main.is_none() && self.quick.is_none() {
                    if let Some(session) = self.session.take() {
                        session.close();
                    }
                    return iced::exit();
                }
            }
            Message::Ipc(ipc::Command::QuickEntry) if !matches!(self.auth, Auth::Setup) => {
                // The same shortcut that opens Quick Entry also dismisses it.
                if self.quick.is_some() {
                    return self.close_quick();
                }
                return self.open_quick();
            }
            Message::Ipc(ipc::Command::QuickEntry) => {}
            Message::Ipc(ipc::Command::Show) => {
                return match self.main {
                    Some(id) => window::gain_focus(id),
                    None => self.open_main(),
                };
            }
            Message::Frame(now) => return self.on_frame(now),
            Message::Tick => return self.on_tick(),
            Message::Key(press) => return self.on_key(press),
            Message::ModifiersChanged(modifiers) => self.modifiers = modifiers,
            Message::CursorMoved(point) => {
                if let Some((_, start)) = &mut self.pressed {
                    let origin = *start.get_or_insert(point);
                    self.cursor = point;
                    if self.drag.is_none() && origin.distance(point) > 6.0 {
                        let (id, _) = self.pressed.expect("checked above");
                        let ids = if self.selection.contains(&id) { self.selection.clone() } else { vec![id] };
                        self.drag = Some(ids);
                    }
                }
            }
            Message::ButtonReleased => {
                self.last_activity = Instant::now();
                self.pressed = None;
                let target = self.drop_target.take();
                if let (Some(ids), Some(target)) = (self.drag.take(), target) {
                    self.drop_on(target, ids);
                }
            }
            Message::SystemDark(dark) => self.system_dark = dark,

            Message::AuthPassword(value) => self.password = value,
            Message::AuthRepeat(value) => self.repeat = value,
            Message::AuthSubmit => {
                return match self.auth {
                    Auth::Setup => self.create_vault(),
                    Auth::Locked { .. } => self.unlock_with_password(),
                    Auth::Open => IcedTask::none(),
                };
            }
            Message::AuthKeePass => return self.unlock_with_keepass(),
            Message::AuthDone(result) => {
                self.auth_busy = false;
                match result.lock().expect("result slot").take() {
                    Some(Ok(session)) => {
                        // A brand-new vault starts with a short, plain KeePassXC explanation.
                        let first_run = matches!(self.auth, Auth::Setup);
                        let unlocked = self.finish_unlock(session);
                        if first_run {
                            return IcedTask::batch([unlocked, self.open_sheet(Sheet::KeePass)]);
                        }
                        return unlocked;
                    }
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
            }

            Message::Go(view) => return self.go(view),
            Message::ToggleArea(id) => {
                if let Some(session) = &mut self.session {
                    session.edit_silent(|lib| {
                        if let Some(area) = lib.areas.iter_mut().find(|a| a.id == id) {
                            area.collapsed = !area.collapsed;
                        }
                        true
                    });
                }
            }
            Message::TagFilter(tag) => {
                self.tag_filter = if self.tag_filter == tag { None } else { tag };
                self.refresh(true);
            }

            Message::RowPress(id) => {
                self.last_activity = Instant::now();
                if self.expanded != Some(id) {
                    self.collapse();
                }
                self.pressed = Some((id, None));
                if self.modifiers.control() {
                    if let Some(index) = self.selection.iter().position(|x| *x == id) {
                        self.selection.remove(index);
                    } else {
                        self.selection.push(id);
                    }
                } else if self.modifiers.shift() {
                    let from = self.anchor.and_then(|a| self.flat.iter().position(|x| *x == a));
                    let to = self.flat.iter().position(|x| *x == id);
                    if let (Some(from), Some(to)) = (from, to) {
                        let (low, high) = (from.min(to), from.max(to));
                        self.selection = self.flat[low..=high].to_vec();
                    }
                } else if !self.selection.contains(&id) {
                    self.selection = vec![id];
                    self.anchor = Some(id);
                }
                return unfocus();
            }
            Message::RowOpen(id) => return self.expand(id),
            Message::RowMenu(id) => {
                if !self.selection.contains(&id) {
                    self.selection = vec![id];
                    self.anchor = Some(id);
                }
                return self.open_sheet(Sheet::Palette);
            }
            Message::RowHover(id) => {
                if self.drag.is_some() {
                    self.drop_target = Some(DropTarget::Row(id));
                }
            }
            Message::SideHover(view) => {
                if self.drag.is_some() {
                    self.drop_target = Some(DropTarget::Side(view));
                }
            }
            Message::Toggle(id) => {
                let today = self.today;
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

            Message::Command(cmd) => return self.run(cmd),
            Message::SheetQuery(value) => {
                self.query = value;
                self.sheet_index = 0;
            }
            Message::SheetSubmit => return self.pick(self.sheet_index),
            Message::SheetPick(index) => return self.pick(index),
            Message::SheetClose => return self.close_sheet(),

            Message::Setting(change) => self.change_setting(change),
            Message::PasswordOld(value) => self.password_old = value,
            Message::PasswordNew(value) => self.password_new = value,
            Message::PasswordChange => {
                let old = Zeroizing::new(std::mem::take(&mut self.password_old));
                let new = Zeroizing::new(std::mem::take(&mut self.password_new));
                return self.security_task(move |vault, plain| {
                    let mut vault = vault.lock().expect("vault lock");
                    let secret = if vault.has_keepass() { keepass::read_secret(vault.vault_id()) } else { None };
                    vault
                        .change_password(&old, &new, secret.as_ref().map(|s| s.as_slice()), &plain)
                        .map_err(|e| e.to_string())?;
                    Ok(t("Password changed").to_owned())
                });
            }
            Message::SecurityDone(result) => {
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

            Message::QuickInput(value) => self.quick_text = value,
            Message::QuickSubmit => return self.submit_quick(),
            Message::QuickComplete(index) => return self.complete_quick(index),
            Message::Ignored => {}
        }
        IcedTask::none()
    }

    fn pick(&mut self, index: usize) -> Task2 {
        if matches!(self.sheet, Some(Sheet::Settings | Sheet::Help | Sheet::KeePass)) {
            return IcedTask::none();
        }
        match self.sheet_items().into_iter().nth(index).map(|item| item.act) {
            Some(act) => self.apply_act(act),
            None => IcedTask::none(),
        }
    }

    pub fn has_vault_keepass(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.vault.lock().expect("vault lock").has_keepass())
    }

    pub fn save_error(&self) -> Option<String> {
        self.session.as_ref().and_then(|s| s.save_error.lock().expect("error lock").clone())
    }
}

fn global_event(event: Event, status: event::Status, window: window::Id) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, physical_key, modifiers, text, .. }) => {
            let latin = key.to_latin(physical_key).map(|c| c.to_ascii_lowercase());
            Some(Message::Key(KeyPress {
                window,
                key,
                latin,
                modifiers,
                text: text.map(|t| t.to_string()),
                captured: status == event::Status::Captured,
            }))
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => Some(Message::ModifiersChanged(modifiers)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(Message::ButtonReleased),
        _ => None,
    }
}

fn cursor_event(event: Event, _: event::Status, _: window::Id) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
        _ => None,
    }
}

/// Desktop notification over D-Bus; works with mako, dunst and friends.
fn notify(bodies: &[String]) {
    use std::collections::HashMap;
    use zbus::zvariant::Value;

    let Ok(connection) = zbus::blocking::Connection::session() else { return };
    for body in bodies {
        let hints: HashMap<&str, Value<'_>> = HashMap::new();
        let _ = connection.call_method(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            Some("org.freedesktop.Notifications"),
            "Notify",
            &("gotodo", 0u32, "gotodo", t("Reminder"), body.as_str(), Vec::<&str>::new(), hints, -1i32),
        );
    }
}

#[cfg(test)]
mod tests;
