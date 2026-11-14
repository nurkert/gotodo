//! Application state and message handling.

mod actions;
mod auth;
mod gtk;
mod input;
mod keys;
mod motion;
mod navigate;
mod quick;
mod sheet;
mod style;
mod tick;
mod view;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{Datelike, Local, Months, NaiveDate, Utc};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{Id as WidgetId, operation, text_editor};
use iced::{Event, Point, Size, Subscription, Task as IcedTask, Theme, event, mouse, time, window};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::i18n::t;
use crate::ipc;
use crate::device;
use crate::keepass;
use crate::model::{
    ChecklistItem, Id, Kind, Library, MoveTarget, Schedule, Section, Status, Task, View, local_today,
};
use crate::parse;
use crate::prefs::Prefs;
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

/// The visible popup card.
pub const QUICK_CARD: Size = Size::new(580.0, 112.0);
/// The window is exactly the card, so compositor effects (blur, border,
/// shadow, rounded corners) wrap the popup and nothing else.
const QUICK_SIZE: Size = QUICK_CARD;

/// Windows are see-through so the Quick Entry card can fade in and the
/// background opacity settings can show a blurred desktop behind them.
/// Set GOTODO_OPAQUE=1 on a setup whose renderer has no alpha channel.
pub fn quick_transparent() -> bool {
    static SEE_THROUGH: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SEE_THROUGH.get_or_init(|| std::env::var_os("GOTODO_OPAQUE").is_none())
}
/// The most see-through the Quick Entry card gets.
const QUICK_MIN_OPACITY: f32 = 0.72;
pub const SIDEBAR_MIN: f32 = 180.0;
const COMPACT_BELOW: f32 = 700.0;
const RAIL_WIDTH: f32 = 56.0;
pub const SIDEBAR_MAX: f32 = 420.0;
const LOG_DELAY: Duration = Duration::from_millis(1100);

#[derive(Clone)]
pub enum Message {
    Opened(window::Id),
    Closed(window::Id),
    Resized(window::Id, Size),
    QuickUnlock,
    Ipc(ipc::Command),
    Frame(Instant),
    Tick,
    Key(KeyPress),
    ModifiersChanged(Modifiers),
    CursorMoved(Point),
    ButtonReleased,
    SystemDark(bool),
    ShowPassword(bool),
    BackupExport,
    Notified(Vec<(u32, Id)>),
    NotificationAction(u32, String),
    ToastUndo,
    BackupRestore(std::path::PathBuf),

    AuthPassword(String),
    AuthRepeat(String),
    AuthSubmit,
    AuthKeePass,
    AuthDevice,
    SetupDevice,
    SetupPassword(bool),
    UnlockMode(bool),
    DeviceEnable,
    DeviceDisable,
    AuthDone(Slot<Result<Session, String>>),
    KeePassProbe(keepass::Status, bool),

    Go(View),
    ToggleArea(Id),
    TagFilter(Option<Id>),

    RowPress(Id),
    RowOpen(Id),
    RowMenu(Id),
    RowHover(Id),
    RowLeave(Id),
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
    PasswordRepeat(String),
    PasswordChange,
    SecurityDone(Slot<Result<String, String>>),
    KeePassConnect,
    KeePassDisconnect,

    QuickInput(String),
    QuickSubmit,
    QuickComplete(usize),
    QuickToken(char),
    SidebarGrab,
    CalShift(i32),
    QuickCal(char),
    QuickCalShift(i32),
    QuickCalPick(NaiveDate),
    QuickPreset(char, &'static str),
    CalPick(NaiveDate),
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
    DeadlineAlert(u32),
    LockNeedsSecret(bool),
    Colors(crate::prefs::ColorSource),
    Accent(crate::prefs::Accent),
    PageOpacity(f32),
    SidebarOpacity(f32),
    QuickOpacity(f32),
    KeepRunning(bool),
    QuickWhileLocked(bool),
}

/// How the current unlock attempt opens the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Password,
    KeePass,
    Device,
}

/// How the process was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Main,
    Quick,
    /// No window; the process stays when windows close.
    Background,
}

/// A short note after something that is easy to regret, with a way back.
pub struct Toast {
    pub text: String,
    pub at: Instant,
}

const TOAST_FOR: Duration = Duration::from_secs(6);

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
    /// Started with `--background`: closing the last window keeps the process.
    background: bool,
    pub auth: Auth,
    pub session: Option<Session>,
    pub system_dark: bool,
    /// Colors from the GTK theme, read at start and when the setting is chosen.
    gtk: Option<gtk::GtkColors>,
    /// Look and lock behaviour, readable while the vault is closed.
    pub prefs: Prefs,
    /// Startup with an automatic way in: show a quiet page instead of the lock
    /// screen until that attempt succeeds or fails.
    pub booting: bool,
    boot_started: Instant,
    unlock_method: Option<Method>,
    /// Best guess from typed letters, since the toolkit does not report Caps Lock.
    pub caps_lock: bool,
    pub show_password: bool,
    /// Phase of the busy spinner, in turns.
    pub spin: f32,

    pub password: String,
    pub repeat: String,
    pub auth_error: Option<String>,
    pub auth_busy: bool,
    pub keepass: keepass::Status,
    skip_auto_unlock: bool,
    pub slots: vault::Slots,
    pub cal_month: NaiveDate,
    /// The open mini calendar in Quick Entry: the token it edits and its first day.
    pub quick_cal: Option<(char, NaiveDate)>,
    /// Days the strip still has to slide to reach its new first day.
    pub quick_cal_slide: Spring,
    pub setup_password: bool,
    /// On the lock screen: show the password field instead of the device button.
    pub unlock_password: bool,

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
    sidebar_drag: bool,
    sidebar_live: Option<f32>,
    pub query: String,
    pub sheet_index: usize,

    pub drag: Option<Vec<Id>>,
    pub drop_target: Option<DropTarget>,
    pub hovered_row: Option<Id>,
    pressed: Option<(Id, Option<Point>)>,
    pub cursor: Point,
    /// The main window's size in logical pixels.
    pub main_size: Size,

    pub quick_text: String,
    /// Quick Entry text typed while locked, added on the next unlock.
    pub pending_quick: Vec<Zeroizing<String>>,
    /// The locked Quick Entry shows the unlock card instead of the input.
    quick_unlock: bool,
    pub modifiers: Modifiers,

    pub password_old: String,
    pub password_new: String,
    pub password_repeat: String,
    pub security_note: Option<String>,
    /// Daily copies, listed while Settings is open.
    pub backups: Vec<crate::backup::Backup>,
    /// A restore waits for a second click on the same copy.
    pub restore_armed: Option<std::path::PathBuf>,
    /// Shown on the lock screen once, e.g. after a restore.
    pub lock_notice: Option<String>,

    last_frame: Option<Instant>,
    last_activity: Instant,
    last_probe: Instant,
    pending_log: Option<Instant>,
    today: NaiveDate,
    published: String,
    /// Shown notifications that carry buttons, by the server's id.
    notifications: HashMap<u32, Id>,
    pub toast: Option<Toast>,
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
        transparent: quick_transparent(),
        platform_specific: platform("gotodo"),
        ..window::Settings::default()
    }
}

/// Fixed-size and undecorated. Sway and i3 float a toplevel whose minimum and
/// maximum size match, so this opens as a small centered popup instead of
/// being tiled.
fn quick_settings() -> window::Settings {
    window::Settings {
        size: QUICK_SIZE,
        min_size: Some(QUICK_SIZE),
        max_size: Some(QUICK_SIZE),
        resizable: false,
        decorations: false,
        transparent: quick_transparent(),
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        platform_specific: platform("gotodo-quick"),
        ..window::Settings::default()
    }
}

impl App {
    pub fn boot(start: Start) -> (Self, Task2) {
        let path = session::vault_path();
        let slots = if vault::exists(&path) { vault::peek_slots(&path).unwrap_or_default() } else { vault::Slots::default() };
        let auth = if vault::exists(&path) { Auth::Locked { has_keepass: slots.keepass } } else { Auth::Setup };
        let prefs = Prefs::load().unwrap_or_default();
        let device_ok = slots.device && !prefs.device_paused;
        let booting = matches!(auth, Auth::Locked { .. }) && (device_ok || slots.keepass);
        let mut app = Self {
            main: None,
            quick: None,
            background: false,
            auth,
            session: None,
            system_dark: false,
            gtk: if prefs.colors == crate::prefs::ColorSource::Gotodo { None } else { gtk::GtkColors::load() },
            prefs,
            booting,
            boot_started: Instant::now(),
            unlock_method: None,
            caps_lock: false,
            show_password: false,
            spin: 0.0,
            password: String::new(),
            repeat: String::new(),
            auth_error: None,
            auth_busy: false,
            keepass: keepass::Status::Missing,
            skip_auto_unlock: false,
            slots,
            cal_month: local_today().with_day(1).unwrap_or_else(local_today),
            quick_cal: None,
            quick_cal_slide: Spring::at(0.0),
            setup_password: false,
            unlock_password: false,
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
            sidebar_drag: false,
            sidebar_live: None,
            query: String::new(),
            sheet_index: 0,
            drag: None,
            drop_target: None,
            hovered_row: None,
            pressed: None,
            cursor: Point::ORIGIN,
            main_size: Size::new(980.0, 680.0),
            quick_text: String::new(),
            pending_quick: Vec::new(),
            quick_unlock: false,
            modifiers: Modifiers::default(),
            password_old: String::new(),
            password_new: String::new(),
            password_repeat: String::new(),
            security_note: None,
            backups: Vec::new(),
            restore_armed: None,
            lock_notice: None,
            last_frame: None,
            last_activity: Instant::now(),
            last_probe: Instant::now(),
            pending_log: None,
            today: local_today(),
            published: String::new(),
            notifications: HashMap::new(),
            toast: None,
        };
        let needs_setup = matches!(app.auth, Auth::Setup);
        app.background = start == Start::Background;
        let open = match start {
            Start::Quick if !needs_setup => app.open_quick(),
            Start::Background if !needs_setup => IcedTask::none(),
            _ => app.open_main(),
        };
        let theme = iced::system::theme().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark));
        app.view_anim = Spring::at(0.0);
        app.view_anim.to(1.0);
        let device = if matches!(app.auth, Auth::Locked { .. }) && app.device_allowed() {
            app.unlock_with_device()
        } else {
            IcedTask::none()
        };
        let probe = if app.slots.keepass { Self::probe(true) } else { IcedTask::none() };
        (app, IcedTask::batch([open, theme, probe, device]))
    }

    fn open_main(&mut self) -> Task2 {
        let (id, open) = window::open(main_settings());
        self.main = Some(id);
        open.map(Message::Opened)
    }

    pub fn palette(&self) -> Palette {
        use crate::model::Appearance as A;
        let dark = match self.prefs.appearance {
            A::Light => false,
            A::Dark => true,
            A::Auto => self.system_dark,
        };
        use crate::prefs::ColorSource;
        let base = if dark { DARK } else { LIGHT };
        match (self.prefs.colors, &self.gtk) {
            (ColorSource::Gtk, Some(gtk)) => gtk.palette(dark),
            (ColorSource::GtkAccent, Some(gtk::GtkColors { accent: Some(accent), .. })) => style::with_accent(base, *accent),
            _ => style::with_accent(base, style::accent_color(self.prefs.accent, dark)),
        }
    }

    /// The main window's background with the chosen opacity.
    pub fn page_bg(&self) -> iced::Color {
        let p = self.palette();
        if !quick_transparent() {
            return p.page;
        }
        iced::Color { a: self.prefs.page_opacity.clamp(crate::prefs::MIN_OPACITY, 1.0), ..p.page }
    }

    pub fn sidebar_bg(&self) -> iced::Color {
        let p = self.palette();
        if !quick_transparent() {
            return p.sidebar;
        }
        iced::Color { a: self.prefs.sidebar_opacity.clamp(crate::prefs::MIN_OPACITY, 1.0), ..p.sidebar }
    }

    /// Quick Entry floats over anything, often a bright browser page, and its
    /// text is the whole point. So the slider only spans 72 to 100 % of real
    /// opacity: enough to see the blur, never enough to wash out the text.
    pub fn quick_opacity(&self) -> f32 {
        if !quick_transparent() {
            return 1.0;
        }
        let chosen = self.prefs.quick_opacity.clamp(crate::prefs::MIN_OPACITY, 1.0);
        let k = (chosen - crate::prefs::MIN_OPACITY) / (1.0 - crate::prefs::MIN_OPACITY);
        QUICK_MIN_OPACITY + (1.0 - QUICK_MIN_OPACITY) * k
    }

    pub fn theme(&self, _: window::Id) -> Theme {
        style::base_theme(&self.palette())
    }

    pub fn scale(&self, _: window::Id) -> f32 {
        self.prefs.text_scale
    }

    pub fn window_title(&self, id: window::Id) -> String {
        if Some(id) == self.quick { "gotodo quick entry".to_owned() } else { "gotodo".to_owned() }
    }

    pub fn lib(&self) -> Option<&Library> {
        self.session.as_ref().map(|s| &s.lib)
    }

    fn reduce_motion(&self) -> bool {
        self.prefs.reduce_motion
    }

    pub fn boot_started_elapsed(&self) -> f32 {
        self.boot_started.elapsed().as_secs_f32()
    }

    /// Tallest a sheet may be in the main window.
    pub fn sheet_max_height(&self) -> f32 {
        (self.main_size.height - 96.0).max(260.0)
    }

    /// The device key may open the vault: it exists and no lock has paused it.
    pub fn device_allowed(&self) -> bool {
        self.slots.device && !self.prefs.device_paused
    }

    /// A lock can pause the device key only when another way in exists.
    pub fn can_pause_device(&self) -> bool {
        self.slots.password || self.slots.keepass
    }

    /// Below this width the sidebar folds into a rail of icons.
    pub fn compact(&self) -> bool {
        self.main_size.width < COMPACT_BELOW
    }

    /// Sidebar width in logical pixels, live while the divider is being dragged.
    pub fn sidebar_width(&self) -> f32 {
        if self.compact() {
            return RAIL_WIDTH;
        }
        let saved = self.lib().map_or(246.0, |l| l.settings.sidebar_width);
        self.sidebar_live.unwrap_or(saved).clamp(SIDEBAR_MIN, SIDEBAR_MAX)
    }

    fn zoom(&mut self, to: Option<f32>, step: f32) {
        let current = self.prefs.text_scale;
        let next = to.unwrap_or(((current + step) * 20.0).round() / 20.0);
        self.change_setting(SettingChange::Scale(next));
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
            || self.auth_busy
            || self.booting
            || !self.sheet_anim.settled()
            || !self.quick_cal_slide.settled()
            || !self.ghosts.is_empty()
            || self.rows.values().any(|a| !a.settled())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            event::listen_with(global_event),
            window::close_events().map(Message::Closed),
            time::every(Duration::from_millis(500)).map(|_| Message::Tick),
            Subscription::run(ipc::listen).map(Message::Ipc),
            Subscription::run(crate::notify::actions).map(|(id, key)| Message::NotificationAction(id, key)),
            iced::system::theme_changes().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark)),
        ];
        if self.animating() {
            subs.push(window::frames().map(Message::Frame));
        }
        if self.pressed.is_some() || self.sidebar_drag {
            subs.push(event::listen_with(cursor_event));
        }
        Subscription::batch(subs)
    }

    pub fn update(&mut self, message: Message) -> Task2 {
        match message {
            message @ (Message::AuthPassword(..) | Message::AuthRepeat(..) | Message::AuthSubmit | Message::AuthKeePass | Message::AuthDevice | Message::SetupDevice | Message::UnlockMode(..) | Message::SetupPassword(..) | Message::DeviceEnable | Message::DeviceDisable | Message::AuthDone(..) | Message::KeePassProbe(..) | Message::ShowPassword(..) | Message::PasswordOld(..) | Message::PasswordNew(..) | Message::PasswordRepeat(..) | Message::PasswordChange | Message::SecurityDone(..) | Message::KeePassConnect | Message::KeePassDisconnect | Message::BackupExport | Message::BackupRestore(..)) => return self.update_security(message),
            message @ (Message::QuickCal(..) | Message::QuickCalShift(..) | Message::QuickCalPick(..) | Message::QuickPreset(..) | Message::QuickInput(..) | Message::QuickSubmit | Message::QuickComplete(..) | Message::QuickToken(..)) => return self.update_quick(message),
            message @ (Message::Title(..) | Message::TitleDone | Message::Notes(..) | Message::ProjectNotes(..) | Message::ChecklistInput(..) | Message::ChecklistAdd | Message::ChecklistToggle(..) | Message::ChecklistEdit(..) | Message::ChecklistRemove(..) | Message::Toggle(..) | Message::HeadingRename(..) | Message::HeadingArchive(..) | Message::HeadingDelete(..) | Message::Notified(..) | Message::NotificationAction(..) | Message::ToastUndo) => return self.update_edit(message),
            Message::Opened(id) => {
                if Some(id) == self.quick {
                    return if matches!(self.auth, Auth::Open) || self.quick_queues() {
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
                    self.quick_cal = None;
                    self.quick_text.zeroize();
                }
                if Some(id) == self.main {
                    self.main = None;
                }
                // To-dos waiting for an unlock live only in this process, so it stays.
                let stay = self.background || self.prefs.keep_running || !self.pending_quick.is_empty();
                if self.main.is_none() && self.quick.is_none() && !stay {
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
            Message::QuickUnlock => {
                self.quick_unlock = true;
                return operation::focus(auth_id());
            }
            Message::Ipc(ipc::Command::Show) => {
                return match self.main {
                    Some(id) => window::gain_focus(id),
                    None => self.open_main(),
                };
            }
            Message::Resized(id, size) => {
                if Some(id) == self.main {
                    self.main_size = size;
                }
            }
            Message::Frame(now) => return self.on_frame(now),
            Message::Tick => return self.on_tick(),
            Message::Key(press) => return self.on_key(press),
            Message::ModifiersChanged(modifiers) => self.modifiers = modifiers,
            Message::SidebarGrab => self.sidebar_drag = true,
            Message::CalShift(delta) => {
                let shifted = if delta >= 0 {
                    self.cal_month.checked_add_months(Months::new(delta as u32))
                } else {
                    self.cal_month.checked_sub_months(Months::new(delta.unsigned_abs()))
                };
                self.cal_month = shifted.unwrap_or(self.cal_month);
            }
            Message::CalPick(date) => {
                let act = match self.sheet {
                    Some(Sheet::Deadline) => Act::Deadline(Some(date)),
                    Some(Sheet::When) => Act::When(parse::When {
                        schedule: Schedule::On(date),
                        evening: false,
                        reminder: None,
                    }),
                    _ => return IcedTask::none(),
                };
                return self.apply_act(act);
            }
            Message::CursorMoved(point) => {
                if self.sidebar_drag {
                    self.sidebar_live = Some(point.x.clamp(SIDEBAR_MIN, SIDEBAR_MAX));
                }
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
                if std::mem::take(&mut self.sidebar_drag) {
                    if let (Some(width), Some(session)) = (self.sidebar_live.take(), &mut self.session) {
                        session.edit_silent(|lib| {
                            lib.settings.sidebar_width = width;
                            true
                        });
                    }
                }
                self.pressed = None;
                let target = self.drop_target.take();
                if let (Some(ids), Some(target)) = (self.drag.take(), target) {
                    self.drop_on(target, ids);
                }
            }
            Message::SystemDark(dark) => self.system_dark = dark,
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
            Message::RowLeave(id) => {
                if self.hovered_row == Some(id) {
                    self.hovered_row = None;
                }
            }
            Message::RowHover(id) => {
                self.hovered_row = Some(id);
                if self.drag.is_some() {
                    self.drop_target = Some(DropTarget::Row(id));
                }
            }
            Message::SideHover(view) => {
                if self.drag.is_some() {
                    self.drop_target = Some(DropTarget::Side(view));
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
            Message::Ignored => {}
        }
        IcedTask::none()
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
        Event::Window(window::Event::Resized(size)) => Some(Message::Resized(window, size)),
        _ => None,
    }
}

fn cursor_event(event: Event, _: event::Status, _: window::Id) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
