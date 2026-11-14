//! Drives the real app state through messages, and renders it headlessly.
//! Set GOTODO_SNAPSHOTS to a directory to also write PNG screenshots.

use std::time::{Duration, Instant};

use chrono::Days;
use iced::widget::text_editor;
use iced::{Settings, Size, window};

use super::*;
use crate::vault;
use crate::model::{ChecklistItem, Kind, Library, Schedule, Status, Task, View};

struct Fixture {
    app: App,
    _dir: tempfile::TempDir,
    project: Id,
    window: window::Id,
}

fn sample_library(today: NaiveDate) -> (Library, Id) {
    let mut lib = Library::default();
    let home = lib.add_area("Zuhause");
    let mut project = Task::new(Kind::Project, "Haus Umbau", Schedule::Anytime);
    project.area = Some(home);
    let project = lib.insert(project);
    let release = lib.insert(Task::new(Kind::Project, "Release 0.0.1", Schedule::Anytime));
    let errand = lib.ensure_tag("Besorgung");
    let call = lib.ensure_tag("Anruf");

    let add = |lib: &mut Library, title: &str, schedule| lib.insert(Task::new(Kind::Todo, title, schedule));
    let today_on = Schedule::On(today);
    let a = add(&mut lib, "Fliesen im Baumarkt abholen", today_on);
    {
        let task = lib.task_mut(a).unwrap();
        task.project = Some(project);
        task.tags.push(errand);
        task.notes = "Maße stehen im Notizbuch".into();
    }
    let b = add(&mut lib, "Steuerberater zurückrufen", today_on);
    {
        let task = lib.task_mut(b).unwrap();
        task.tags.push(call);
        task.deadline = Some(today);
    }
    let c = add(&mut lib, "Paket zur Post bringen", today_on);
    lib.task_mut(c).unwrap().checklist = vec![
        ChecklistItem { id: Uuid::new_v4(), title: "Etikett drucken".into(), done: true },
        ChecklistItem { id: Uuid::new_v4(), title: "Karton zukleben".into(), done: false },
    ];
    let d = add(&mut lib, "Laufen gehen", today_on);
    lib.task_mut(d).unwrap().evening = true;
    let e = add(&mut lib, "Buch zu Ende lesen", today_on);
    lib.task_mut(e).unwrap().evening = true;
    add(&mut lib, "Idee für den Garten notieren", Schedule::Inbox);
    add(&mut lib, "Zahnarzttermin", Schedule::On(today + Days::new(1)));
    let release_item = add(&mut lib, "Pipeline testen", Schedule::Anytime);
    lib.task_mut(release_item).unwrap().project = Some(release);
    (lib, project)
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, _) = App::boot(Start::Main);
    let today = app.today();
    let (lib, project) = sample_library(today);
    let session = Session::for_tests(lib, &dir.path().join("vault.gotodo"));
    let _ = app.finish_unlock(session);
    let window = window::Id::unique();
    app.main = Some(window);
    settle(&mut app);
    Fixture { app, _dir: dir, project, window }
}

fn settle(app: &mut App) {
    let mut now = Instant::now();
    for _ in 0..240 {
        now += Duration::from_millis(16);
        let _ = app.on_frame(now);
        if !app.animating() {
            break;
        }
    }
}

fn run(app: &mut App, cmd: Cmd) {
    let _ = app.update(Message::Command(cmd));
}

fn titled(app: &App, title: &str) -> Id {
    app.lib().unwrap().tasks.iter().find(|t| t.title == title).expect("task exists").id
}

const MAIN: Size = Size::new(980.0, 680.0);

/// Renders one window headlessly. With GOTODO_SNAPSHOTS set, the frame is
/// also written there as a PNG so every screen can be looked at by eye.
fn snapshot(app: &App, window: window::Id, name: &str, size: Size) {
    let settings = Settings {
        default_font: style::FONT,
        fonts: vec![
            include_bytes!("../../assets/fonts/Inter-Regular.ttf").as_slice().into(),
            include_bytes!("../../assets/fonts/Inter-Medium.ttf").as_slice().into(),
            include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").as_slice().into(),
        ],
        ..Settings::default()
    };
    let theme = app.theme(window);
    let mut ui = iced_test::Simulator::with_size(settings, size, app.view(window));
    let frame = ui.snapshot(&theme).unwrap();
    if let Ok(dir) = std::env::var("GOTODO_SNAPSHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        let path = format!("{dir}/{name}.png");
        let _ = std::fs::remove_file(format!("{dir}/{name}-wgpu.png"));
        frame.matches_image(&path).unwrap();
    }
}

fn set_appearance(app: &mut App, dark: bool) {
    let look = if dark { crate::model::Appearance::Dark } else { crate::model::Appearance::Light };
    if let Some(session) = &mut app.session {
        session.lib.settings.appearance = look;
    }
    app.prefs.appearance = look;
}

/// Locks the fixture and waits for the lock screen to finish fading in.
fn lock(f: &mut Fixture, slots: vault::Slots) {
    run(&mut f.app, Cmd::Lock);
    f.app.slots = slots;
    f.app.auth = Auth::Locked { has_keepass: slots.keepass };
    f.app.booting = false;
    settle(&mut f.app);
}

#[test]
fn new_todo_opens_for_typing_and_saves_title() {
    let mut f = fixture();
    run(&mut f.app, Cmd::NewTodo);
    let id = f.app.expanded.expect("new row is open");
    let _ = f.app.update(Message::Title("Milch kaufen".into()));
    let _ = f.app.update(Message::TitleDone);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().title, "Milch kaufen");
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().schedule, Schedule::On(f.app.today()));
}

#[test]
fn untitled_new_todo_is_discarded() {
    let mut f = fixture();
    let before = f.app.lib().unwrap().tasks.len();
    run(&mut f.app, Cmd::NewTodo);
    let _ = f.app.update(Message::TitleDone);
    assert_eq!(f.app.lib().unwrap().tasks.len(), before);
}

#[test]
fn new_todo_inside_a_project_belongs_to_it() {
    let mut f = fixture();
    let _ = f.app.update(Message::Go(View::Project(f.project)));
    run(&mut f.app, Cmd::NewTodo);
    let id = f.app.expanded.unwrap();
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().project, Some(f.project));
}

#[test]
fn new_todo_from_upcoming_is_visible_in_inbox() {
    let mut f = fixture();
    let _ = f.app.update(Message::Go(View::Upcoming));
    run(&mut f.app, Cmd::NewTodo);
    assert_eq!(f.app.view, View::Inbox);
    assert!(f.app.expanded.is_some());
}

#[test]
fn move_sheet_assigns_project_with_keyboard_only() {
    let mut f = fixture();
    let id = titled(&f.app, "Steuerberater zurückrufen");
    f.app.selection = vec![id];
    run(&mut f.app, Cmd::MoveTo);
    let _ = f.app.update(Message::SheetQuery("haus".into()));
    let _ = f.app.update(Message::SheetSubmit);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().project, Some(f.project));
}

#[test]
fn when_sheet_parses_typed_dates() {
    let mut f = fixture();
    let id = titled(&f.app, "Laufen gehen");
    f.app.selection = vec![id];
    run(&mut f.app, Cmd::When);
    let _ = f.app.update(Message::SheetQuery("in 3 tagen".into()));
    let _ = f.app.update(Message::SheetSubmit);
    let expected = f.app.today() + Days::new(3);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().schedule, Schedule::On(expected));
}

#[test]
fn quick_entry_parses_tokens_and_closes() {
    let mut f = fixture();
    let popup = window::Id::unique();
    f.app.quick = Some(popup);
    let _ = f.app.update(Message::QuickInput("Dübel kaufen #haus @besorgung !morgen".into()));
    let _ = f.app.update(Message::QuickSubmit);
    let id = titled(&f.app, "Dübel kaufen");
    let task = f.app.lib().unwrap().task(id).unwrap();
    assert_eq!(task.project, Some(f.project));
    assert_eq!(task.tags.len(), 1);
    assert_eq!(task.schedule, Schedule::On(f.app.today() + Days::new(1)));
    assert!(f.app.quick_text.is_empty());
}

#[test]
fn shift_enter_keeps_quick_entry_open_for_the_next_item() {
    let mut f = fixture();
    f.app.quick = Some(window::Id::unique());
    f.app.modifiers = Modifiers::SHIFT;
    let _ = f.app.update(Message::QuickInput("erstes".into()));
    let _ = f.app.update(Message::QuickSubmit);
    assert!(f.app.quick.is_some());
    assert!(f.app.quick_text.is_empty());
    titled(&f.app, "erstes");
}

#[test]
fn quick_entry_keeps_text_when_title_is_empty() {
    let mut f = fixture();
    f.app.quick = Some(window::Id::unique());
    let count = f.app.lib().unwrap().tasks.len();
    let _ = f.app.update(Message::QuickInput("#haus".into()));
    let _ = f.app.update(Message::QuickSubmit);
    assert_eq!(f.app.lib().unwrap().tasks.len(), count);
    assert_eq!(f.app.quick_text, "#haus");
}

#[test]
fn tab_completes_the_project_token() {
    let mut f = fixture();
    f.app.quick_text = "x #ha".into();
    let _ = f.app.update(Message::QuickComplete(0));
    assert_eq!(f.app.quick_text, "x #HausUmbau ");
}

#[test]
fn complete_then_undo_restores() {
    let mut f = fixture();
    let id = titled(&f.app, "Laufen gehen");
    f.app.selection = vec![id];
    run(&mut f.app, Cmd::Complete);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().status, Status::Completed);
    run(&mut f.app, Cmd::Undo);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().status, Status::Open);
}

#[test]
fn removed_row_fades_out_then_disappears() {
    let mut f = fixture();
    let id = titled(&f.app, "Laufen gehen");
    f.app.selection = vec![id];
    run(&mut f.app, Cmd::SetSomeday);
    assert!(f.app.flat.contains(&id), "stays during the fade");
    assert!(f.app.ghosts.contains(&id));
    settle(&mut f.app);
    assert!(!f.app.flat.contains(&id));
    assert!(f.app.ghosts.is_empty());
}

#[test]
fn dragging_onto_a_project_moves_the_item() {
    let mut f = fixture();
    let id = titled(&f.app, "Laufen gehen");
    let _ = f.app.update(Message::RowPress(id));
    let _ = f.app.update(Message::CursorMoved(Point::new(10.0, 10.0)));
    let _ = f.app.update(Message::CursorMoved(Point::new(40.0, 40.0)));
    assert!(f.app.drag.is_some());
    let _ = f.app.update(Message::SideHover(View::Project(f.project)));
    let _ = f.app.update(Message::ButtonReleased);
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().project, Some(f.project));
}

#[test]
fn reorder_with_keyboard() {
    let mut f = fixture();
    let first = f.app.flat[0];
    let second = f.app.flat[1];
    f.app.selection = vec![second];
    run(&mut f.app, Cmd::MoveUp);
    assert_eq!(f.app.flat[0], second);
    assert_eq!(f.app.flat[1], first);
}

#[test]
fn lock_forgets_the_library() {
    let mut f = fixture();
    run(&mut f.app, Cmd::Lock);
    assert!(f.app.session.is_none());
    assert!(f.app.sections.is_empty());
    assert!(matches!(f.app.auth, Auth::Locked { .. }));
}

#[test]
fn locking_keepass_locks_a_keepass_session() {
    let mut f = fixture();
    f.app.session.as_mut().unwrap().via_keepass = true;
    let _ = f.app.update(Message::KeePassProbe(keepass::Status::Locked, false));
    assert!(f.app.session.is_none());
}

#[test]
fn keepass_status_does_not_lock_a_password_session() {
    let mut f = fixture();
    let _ = f.app.update(Message::KeePassProbe(keepass::Status::Locked, false));
    assert!(f.app.session.is_some());
}

#[test]
fn locking_can_pause_the_device_key_until_a_password_unlock() {
    let mut f = fixture();
    f.app.slots = vault::Slots { password: true, keepass: false, device: true };
    f.app.prefs.lock_needs_secret = true;
    run(&mut f.app, Cmd::Lock);
    // `lock` reads the real slots of the open vault, which has no device slot.
    f.app.slots = vault::Slots { password: true, keepass: false, device: true };
    f.app.prefs.device_paused = true;
    assert!(!f.app.device_allowed(), "the device key waits for the password");
    let _ = f.app.unlock_with_device();
    assert!(!f.app.auth_busy, "no device attempt while paused");

    f.app.unlock_method = Some(Method::Password);
    let session = Session::for_tests(Library::default(), &f._dir.path().join("again.gotodo"));
    let _ = f.app.finish_unlock(session);
    assert!(!f.app.prefs.device_paused, "a password unlock lifts the pause");
}

#[test]
fn quick_entry_while_locked_waits_for_the_unlock() {
    let mut f = fixture();
    lock(&mut f, vault::Slots { password: true, keepass: false, device: false });
    let popup = window::Id::unique();
    f.app.quick = Some(popup);
    assert!(f.app.quick_queues());
    let _ = f.app.update(Message::QuickInput("Rechnung schreiben #HausUmbau".into()));
    let _ = f.app.update(Message::QuickSubmit);
    let _ = f.app.update(Message::QuickInput("   ".into()));
    let _ = f.app.update(Message::QuickSubmit);
    assert_eq!(f.app.pending_quick.len(), 1, "blank entries are not kept");
    assert!(f.app.lib().is_none());

    let _ = f.app.update(Message::Closed(popup));
    assert!(f.app.pending_quick.len() == 1, "waiting to-dos survive closing the popup");

    let (lib, project) = sample_library(f.app.today());
    let session = Session::for_tests(lib, &f._dir.path().join("again.gotodo"));
    let _ = f.app.finish_unlock(session);
    assert!(f.app.pending_quick.is_empty());
    let id = titled(&f.app, "Rechnung schreiben");
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().project, Some(project), "tokens are read at unlock");
    assert!(f.app.toast.is_some());
}

#[test]
fn quick_entry_while_locked_can_be_turned_off() {
    let mut f = fixture();
    lock(&mut f, vault::Slots { password: true, keepass: false, device: false });
    f.app.quick = Some(window::Id::unique());
    let _ = f.app.update(Message::Setting(SettingChange::QuickWhileLocked(false)));
    assert!(!f.app.quick_queues());
    f.app.prefs.quick_while_locked = true;
    let _ = f.app.update(Message::QuickUnlock);
    assert!(!f.app.quick_queues(), "Unlock now shows the unlock card");
}

#[test]
fn closing_the_window_keeps_gotodo_running() {
    let mut f = fixture();
    let window = f.window;
    let _ = f.app.update(Message::Closed(window));
    assert!(f.app.session.is_some(), "still unlocked for Quick Entry");
    let mut f = fixture();
    f.app.prefs.keep_running = false;
    let window = f.window;
    let _ = f.app.update(Message::Closed(window));
    assert!(f.app.session.is_none(), "without the setting the vault closes with the window");
}

#[test]
fn gtk_colors_and_opacity_render() {
    let mut f = fixture();
    f.app.gtk = Some(super::gtk::GtkColors::from_css(
        "@define-color accent_bg_color #b162d5; @define-color window_bg_color #15151a; \
         @define-color window_fg_color #e8e8ed; @define-color base_color #1c1c24; @define-color borders #34343d;",
    ));
    f.app.prefs.colors = crate::prefs::ColorSource::Gtk;
    f.app.prefs.page_opacity = 0.7;
    f.app.prefs.sidebar_opacity = 0.5;
    assert!(f.app.palette().dark);
    assert!((f.app.page_bg().a - 0.7).abs() < 1e-6);
    let window = f.window;
    snapshot(&f.app, window, "gtk-theme", MAIN);
    run(&mut f.app, Cmd::Settings);
    settle(&mut f.app);
    snapshot(&f.app, window, "gtk-theme-settings", MAIN);
    f.app.prefs.colors = crate::prefs::ColorSource::GtkAccent;
    assert_eq!(f.app.palette().accent, iced::color!(0xb162d5));
    f.app.prefs.colors = crate::prefs::ColorSource::Gotodo;
    f.app.prefs.accent = crate::prefs::Accent::Green;
    assert_ne!(f.app.palette().accent, style::LIGHT.accent);
}

#[test]
fn completing_offers_undo() {
    let mut f = fixture();
    let id = titled(&f.app, "Laufen gehen");
    let _ = f.app.update(Message::Toggle(id));
    assert!(f.app.toast.is_some());
    let _ = f.app.update(Message::ToastUndo);
    assert!(f.app.toast.is_none());
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().status, Status::Open);
}

#[test]
fn notification_buttons_reach_the_to_do() {
    let mut f = fixture();
    let id = titled(&f.app, "Buch zu Ende lesen");
    let _ = f.app.update(Message::Notified(vec![(41, id)]));
    let _ = f.app.update(Message::NotificationAction(41, crate::notify::DONE.into()));
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().status, Status::Completed);
    // A second press of the same, now forgotten, notification does nothing.
    let _ = f.app.update(Message::NotificationAction(41, crate::notify::LATER.into()));
    assert_eq!(f.app.lib().unwrap().task(id).unwrap().status, Status::Completed);
}

#[test]
fn startup_stays_quiet_until_the_automatic_unlock_gives_up() {
    let mut f = fixture();
    run(&mut f.app, Cmd::Lock);
    f.app.booting = true;
    f.app.skip_auto_unlock = false;
    let _ = f.app.update(Message::KeePassProbe(keepass::Status::Locked, true));
    assert!(!f.app.booting, "a locked KeePassXC ends the quiet start");
}

#[test]
fn status_line_counts_without_titles() {
    let mut f = fixture();
    f.app.publish_status();
    assert!(f.app.published.contains("\"text\":\"5\""), "{}", f.app.published);
    assert!(f.app.published.contains("overdue") || f.app.published.contains("today"));
    assert!(!f.app.published.contains("Laufen"));
    run(&mut f.app, Cmd::Lock);
    f.app.publish_status();
    assert!(f.app.published.contains("locked"));
}

#[test]
fn manual_lock_is_not_undone_by_an_unlocked_keepass() {
    let mut f = fixture();
    run(&mut f.app, Cmd::Lock);
    f.app.auth = Auth::Locked { has_keepass: true };
    let _ = f.app.update(Message::KeePassProbe(keepass::Status::Unlocked, true));
    assert!(!f.app.auth_busy, "must wait for an explicit unlock");
}

#[test]
fn every_screen_renders() {
    let sheet_cases = [
        (Cmd::MoveTo, "move"),
        (Cmd::Find, "find"),
        (Cmd::Settings, "settings"),
        (Cmd::Help, "help"),
        (Cmd::When, "when"),
        (Cmd::Tags, "tags"),
        (Cmd::Palette, "palette"),
        (Cmd::ConnectKeePass, "keepass"),
    ];
    for dark in [false, true] {
        let tag = if dark { "dark" } else { "light" };
        let mut f = fixture();
        set_appearance(&mut f.app, dark);
        let window = f.window;
        snapshot(&f.app, window, &format!("{tag}-today"), MAIN);

        let id = titled(&f.app, "Paket zur Post bringen");
        let _ = f.app.update(Message::RowOpen(id));
        f.app.notes = text_editor::Content::with_text("Adresse steht in der Mail.");
        settle(&mut f.app);
        snapshot(&f.app, window, &format!("{tag}-expanded"), MAIN);
        let _ = f.app.update(Message::TitleDone);
        settle(&mut f.app);

        let done = titled(&f.app, "Laufen gehen");
        let _ = f.app.update(Message::Toggle(done));
        settle(&mut f.app);
        snapshot(&f.app, window, &format!("{tag}-toast"), MAIN);
        let _ = f.app.update(Message::ToastUndo);
        settle(&mut f.app);

        f.app.selection = vec![id];
        for (cmd, name) in sheet_cases {
            run(&mut f.app, cmd);
            settle(&mut f.app);
            snapshot(&f.app, window, &format!("{tag}-{name}"), MAIN);
            let _ = f.app.update(Message::SheetClose);
            settle(&mut f.app);
        }

        for (view, name) in [(View::Project(f.project), "project"), (View::Upcoming, "upcoming"), (View::Someday, "empty")] {
            let _ = f.app.update(Message::Go(view));
            settle(&mut f.app);
            snapshot(&f.app, window, &format!("{tag}-{name}"), MAIN);
        }
        let _ = f.app.update(Message::Go(View::Today));
        settle(&mut f.app);
        let narrow = Size::new(560.0, 420.0);
        let _ = f.app.update(Message::Resized(window, narrow));
        snapshot(&f.app, window, &format!("{tag}-narrow"), narrow);
        let _ = f.app.update(Message::Resized(window, MAIN));

        let popup = window::Id::unique();
        f.app.quick = Some(popup);
        for (text, name) in [
            ("", "quick-empty"),
            ("Dübel kaufen #haus @besorgung !morgen", "quick"),
            ("Dübel kaufen #haus @besorgung !morgen ^fr ", "quick-full"),
            ("Dübel kaufen #ha", "quick-complete"),
        ] {
            f.app.quick_text = text.into();
            snapshot(&f.app, popup, &format!("{tag}-{name}"), QUICK_SIZE);
        }
        f.app.quick_text = "Dübel kaufen".into();
        let _ = f.app.update(Message::QuickCal('!'));
        settle(&mut f.app);
        snapshot(&f.app, popup, &format!("{tag}-quick-cal"), QUICK_SIZE);
        f.app.quick = None;
        f.app.quick_cal = None;
    }
}

#[test]
fn every_lock_state_renders() {
    let password_only = vault::Slots { password: true, keepass: false, device: false };
    let everything = vault::Slots { password: true, keepass: true, device: true };
    let device_only = vault::Slots { password: false, keepass: false, device: true };
    for dark in [false, true] {
        let tag = if dark { "dark" } else { "light" };
        let mut f = fixture();
        set_appearance(&mut f.app, dark);
        let window = f.window;

        lock(&mut f, password_only);
        snapshot(&f.app, window, &format!("{tag}-locked"), MAIN);
        f.app.password = "secret".into();
        f.app.auth_error = Some("Wrong password".into());
        snapshot(&f.app, window, &format!("{tag}-locked-error"), MAIN);
        f.app.auth_error = None;
        f.app.caps_lock = true;
        snapshot(&f.app, window, &format!("{tag}-locked-caps"), MAIN);
        f.app.caps_lock = false;
        f.app.auth_busy = true;
        snapshot(&f.app, window, &format!("{tag}-locked-busy"), MAIN);
        f.app.auth_busy = false;
        f.app.password.clear();

        lock(&mut f, everything);
        snapshot(&f.app, window, &format!("{tag}-locked-device"), MAIN);
        f.app.prefs.device_paused = true;
        snapshot(&f.app, window, &format!("{tag}-locked-paused"), MAIN);
        f.app.prefs.device_paused = false;
        f.app.booting = true;
        snapshot(&f.app, window, &format!("{tag}-booting"), MAIN);
        f.app.booting = false;

        f.app.auth = Auth::Setup;
        f.app.setup_password = false;
        snapshot(&f.app, window, &format!("{tag}-setup"), MAIN);
        f.app.setup_password = true;
        snapshot(&f.app, window, &format!("{tag}-setup-password"), MAIN);
        f.app.password = "short".into();
        f.app.repeat = "shx".into();
        snapshot(&f.app, window, &format!("{tag}-setup-mismatch"), MAIN);
        f.app.password = "long enough password".into();
        f.app.repeat = "long enough password".into();
        snapshot(&f.app, window, &format!("{tag}-setup-ready"), MAIN);
        f.app.password.clear();
        f.app.repeat.clear();

        let popup = window::Id::unique();
        f.app.quick = Some(popup);
        f.app.auth = Auth::Locked { has_keepass: false };
        f.app.slots = password_only;
        f.app.quick_text = "Angebot prüfen #arbeit !morgen".into();
        snapshot(&f.app, popup, &format!("{tag}-quick-queued"), QUICK_SIZE);
        f.app.quick_text.clear();
        f.app.quick_unlock = true;
        snapshot(&f.app, popup, &format!("{tag}-quick-locked"), QUICK_SIZE);
        f.app.auth_error = Some("Wrong password".into());
        snapshot(&f.app, popup, &format!("{tag}-quick-locked-error"), QUICK_SIZE);
        f.app.auth_error = None;
        f.app.slots = device_only;
        snapshot(&f.app, popup, &format!("{tag}-quick-locked-device"), QUICK_SIZE);
        f.app.quick_unlock = false;
        f.app.quick = None;
    }
}
