//! Every keyboard shortcut lives in this one table; the help sheet and the
//! command palette are generated from it. Super and Alt are left alone so
//! they stay free for the window manager.

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::model::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    NewTodo,
    NewProject,
    NewHeading,
    NewArea,
    QuickEntry,
    Find,
    Palette,
    Help,
    Settings,
    Lock,
    Complete,
    Cancel,
    SetToday,
    SetEvening,
    SetAnytime,
    SetSomeday,
    When,
    Deadline,
    MoveTo,
    Tags,
    Repeat,
    Duplicate,
    ConvertToProject,
    Delete,
    PutBack,
    Rename,
    Undo,
    Redo,
    MoveUp,
    MoveDown,
    SelectAll,
    EmptyTrash,
    LogNow,
    DeleteList,
    ConnectKeePass,
    DisconnectKeePass,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Go(View),
}

#[derive(Debug, Clone, Copy)]
pub enum K {
    Char(char),
    Named(Named),
}

#[derive(Debug, Clone, Copy)]
pub struct Binding {
    pub ctrl: bool,
    pub shift: bool,
    pub key: K,
    pub cmd: Cmd,
    pub label: &'static str,
}

const fn ctrl(c: char, cmd: Cmd, label: &'static str) -> Binding {
    Binding { ctrl: true, shift: false, key: K::Char(c), cmd, label }
}

const fn ctrl_shift(c: char, cmd: Cmd, label: &'static str) -> Binding {
    Binding { ctrl: true, shift: true, key: K::Char(c), cmd, label }
}

const fn ctrl_named(key: Named, cmd: Cmd, label: &'static str) -> Binding {
    Binding { ctrl: true, shift: false, key: K::Named(key), cmd, label }
}

const fn plain_named(key: Named, cmd: Cmd, label: &'static str) -> Binding {
    Binding { ctrl: false, shift: false, key: K::Named(key), cmd, label }
}

pub const BINDINGS: &[Binding] = &[
    ctrl('n', Cmd::NewTodo, "New To-Do"),
    ctrl_shift('n', Cmd::QuickEntry, "Quick Entry"),
    ctrl_shift('p', Cmd::NewProject, "New Project"),
    ctrl_shift('h', Cmd::NewHeading, "New Heading"),
    ctrl_shift('a', Cmd::NewArea, "New Area"),
    ctrl_shift('m', Cmd::MoveTo, "Move"),
    ctrl('s', Cmd::When, "When"),
    ctrl('t', Cmd::SetToday, "Today"),
    ctrl('e', Cmd::SetEvening, "This Evening"),
    ctrl('r', Cmd::SetAnytime, "Anytime"),
    ctrl('o', Cmd::SetSomeday, "Someday"),
    ctrl_shift('d', Cmd::Deadline, "Deadline"),
    ctrl_shift('t', Cmd::Tags, "Tags"),
    ctrl_shift('r', Cmd::Repeat, "Repeat"),
    ctrl('k', Cmd::Complete, "Complete"),
    ctrl_shift('k', Cmd::Cancel, "Cancel to-do"),
    ctrl('d', Cmd::Duplicate, "Duplicate"),
    ctrl_named(Named::Delete, Cmd::Delete, "Delete"),
    plain_named(Named::Delete, Cmd::Delete, "Delete"),
    plain_named(Named::F2, Cmd::Rename, "Rename"),
    ctrl('z', Cmd::Undo, "Undo"),
    ctrl_shift('z', Cmd::Redo, "Redo"),
    ctrl_named(Named::ArrowUp, Cmd::MoveUp, "Move Up"),
    ctrl_named(Named::ArrowDown, Cmd::MoveDown, "Move Down"),
    ctrl('a', Cmd::SelectAll, "Select all"),
    ctrl('f', Cmd::Find, "Quick Find"),
    ctrl('p', Cmd::Palette, "Commands"),
    ctrl(',', Cmd::Settings, "Settings"),
    ctrl('l', Cmd::Lock, "Lock"),
    ctrl('+', Cmd::ZoomIn, "Zoom in"),
    ctrl_shift('+', Cmd::ZoomIn, "Zoom in"),
    ctrl('=', Cmd::ZoomIn, "Zoom in"),
    ctrl('-', Cmd::ZoomOut, "Zoom out"),
    ctrl_shift('_', Cmd::ZoomOut, "Zoom out"),
    ctrl('0', Cmd::ZoomReset, "Reset zoom"),
    plain_named(Named::F1, Cmd::Help, "Shortcuts"),
    ctrl('1', Cmd::Go(View::Inbox), "Inbox"),
    ctrl('2', Cmd::Go(View::Today), "Today"),
    ctrl('3', Cmd::Go(View::Upcoming), "Upcoming"),
    ctrl('4', Cmd::Go(View::Anytime), "Anytime"),
    ctrl('5', Cmd::Go(View::Someday), "Someday"),
    ctrl('6', Cmd::Go(View::Logbook), "Logbook"),
    ctrl('7', Cmd::Go(View::Trash), "Trash"),
];

/// Commands reachable only from the palette.
pub const PALETTE_EXTRAS: &[(Cmd, &str)] = &[
    (Cmd::ConvertToProject, "Convert to Project"),
    (Cmd::PutBack, "Put Back"),
    (Cmd::EmptyTrash, "Empty Trash"),
    (Cmd::LogNow, "Log Completed"),
    (Cmd::DeleteList, "Delete List"),
    (Cmd::ConnectKeePass, "Connect KeePassXC"),
    (Cmd::DisconnectKeePass, "Disconnect KeePassXC"),
    (Cmd::Go(View::Deadlines), "Deadlines"),
    (Cmd::Go(View::Repeating), "Repeating"),
    (Cmd::Go(View::AllProjects), "All Projects"),
];

pub fn lookup(key: &Key, physical_latin: Option<char>, modifiers: Modifiers) -> Option<Cmd> {
    if modifiers.alt() || modifiers.logo() {
        return None;
    }
    let pressed_char = match key.as_ref() {
        Key::Character(s) => s.chars().next().map(|c| c.to_ascii_lowercase()),
        _ => None,
    };
    BINDINGS
        .iter()
        .find(|b| {
            b.ctrl == modifiers.control()
                && b.shift == modifiers.shift()
                && match b.key {
                    K::Char(c) => pressed_char == Some(c) || physical_latin == Some(c),
                    K::Named(named) => matches!(key.as_ref(), Key::Named(n) if n == named),
                }
        })
        .map(|b| b.cmd)
}

pub fn chord_label(binding: &Binding) -> String {
    let key = match binding.key {
        K::Char(c) => c.to_ascii_uppercase().to_string(),
        K::Named(Named::ArrowUp) => "↑".to_owned(),
        K::Named(Named::ArrowDown) => "↓".to_owned(),
        K::Named(Named::Delete) => "Del".to_owned(),
        K::Named(named) => format!("{named:?}"),
    };
    let mut label = String::new();
    if binding.ctrl {
        label.push_str("Ctrl+");
    }
    if binding.shift {
        label.push_str("Shift+");
    }
    label + &key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_two_bindings_share_a_chord() {
        let labels: Vec<String> = BINDINGS.iter().map(chord_label).collect();
        for (i, label) in labels.iter().enumerate() {
            assert!(!labels[i + 1..].contains(label), "duplicate chord {label}");
        }
    }

    #[test]
    fn shift_produces_uppercase_but_still_matches() {
        let key = Key::Character("M".into());
        let mods = Modifiers::CTRL | Modifiers::SHIFT;
        assert_eq!(lookup(&key, None, mods), Some(Cmd::MoveTo));
    }

    #[test]
    fn window_manager_modifiers_are_ignored() {
        let key = Key::Character("n".into());
        assert_eq!(lookup(&key, None, Modifiers::CTRL | Modifiers::LOGO), None);
        assert_eq!(lookup(&key, None, Modifiers::CTRL | Modifiers::ALT), None);
    }

    #[test]
    fn non_latin_layouts_use_the_physical_key() {
        let key = Key::Character("т".into());
        assert_eq!(lookup(&key, Some('n'), Modifiers::CTRL), Some(Cmd::NewTodo));
    }
}
