//! Drives the real app state through messages, and renders it headlessly.
//! Set GOTODO_SNAPSHOTS to a directory to also write PNG screenshots.

use std::time::{Duration, Instant};

use chrono::Days;
use iced::widget::text_editor;
use iced::{Settings, Size, window};

use super::*;
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
    let (mut app, _) = App::boot(false);
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

fn snapshot(app: &App, window: window::Id, name: &str, size: Size, dark: bool) {
    let Ok(dir) = std::env::var("GOTODO_SNAPSHOTS") else { return };
    let path = format!("{dir}/{name}.png");
    let settings = Settings {
        default_font: style::FONT,
        fonts: vec![
            include_bytes!("../../assets/fonts/Inter-Regular.ttf").as_slice().into(),
            include_bytes!("../../assets/fonts/Inter-Medium.ttf").as_slice().into(),
            include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").as_slice().into(),
        ],
        ..Settings::default()
    };
    let _ = dark;
    let theme = app.theme(window);
    let mut ui = iced_test::Simulator::with_size(settings, size, app.view(window));
    ui.snapshot(&theme).unwrap().matches_image(&path).unwrap();
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
fn manual_lock_is_not_undone_by_an_unlocked_keepass() {
    let mut f = fixture();
    run(&mut f.app, Cmd::Lock);
    f.app.auth = Auth::Locked { has_keepass: true };
    let _ = f.app.update(Message::KeePassProbe(keepass::Status::Unlocked, true));
    assert!(!f.app.auth_busy, "must wait for an explicit unlock");
}

#[test]
fn every_screen_renders() {
    let main = Size::new(980.0, 680.0);
    let sheet_cases = [(Cmd::MoveTo, "move"), (Cmd::Find, "find"), (Cmd::Settings, "settings"), (Cmd::Help, "help")];
    for dark in [false, true] {
        let tag = if dark { "dark" } else { "light" };
        let mut f = fixture();
        if let Some(session) = &mut f.app.session {
            session.lib.settings.appearance =
                if dark { crate::model::Appearance::Dark } else { crate::model::Appearance::Light };
        }
        let window = f.window;
        snapshot(&f.app, window, &format!("{tag}-today"), main, dark);

        let id = titled(&f.app, "Paket zur Post bringen");
        let _ = f.app.update(Message::RowOpen(id));
        f.app.notes = text_editor::Content::with_text("Adresse steht in der Mail.");
        settle(&mut f.app);
        snapshot(&f.app, window, &format!("{tag}-expanded"), main, dark);
        let _ = f.app.update(Message::TitleDone);
        settle(&mut f.app);

        f.app.selection = vec![id];
        for (cmd, name) in sheet_cases {
            run(&mut f.app, cmd);
            settle(&mut f.app);
            snapshot(&f.app, window, &format!("{tag}-{name}"), main, dark);
            let _ = f.app.update(Message::SheetClose);
            settle(&mut f.app);
        }

        let _ = f.app.update(Message::Go(View::Project(f.project)));
        settle(&mut f.app);
        snapshot(&f.app, window, &format!("{tag}-project"), main, dark);
        let _ = f.app.update(Message::Go(View::Upcoming));
        settle(&mut f.app);
        snapshot(&f.app, window, &format!("{tag}-upcoming"), main, dark);

        let popup = window::Id::unique();
        f.app.quick = Some(popup);
        f.app.quick_text = "Dübel kaufen #haus @besorgung !morgen".into();
        snapshot(&f.app, popup, &format!("{tag}-quick"), QUICK_SIZE, dark);
        f.app.quick_text = "Dübel kaufen #ha".into();
        snapshot(&f.app, popup, &format!("{tag}-quick-complete"), QUICK_SIZE, dark);
        f.app.quick = None;

        run(&mut f.app, Cmd::Lock);
        snapshot(&f.app, window, &format!("{tag}-locked"), main, dark);
        f.app.auth = Auth::Setup;
        snapshot(&f.app, window, &format!("{tag}-setup"), main, dark);
    }
}
