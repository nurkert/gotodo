//! Everything the user sees.

use chrono::{Datelike, Days, NaiveDate};
use iced::widget::{
    Column, Row, button, center, container, hover, mouse_area, opaque, pin, row, scrollable, slider, space, stack,
    text, text_editor, text_input, toggler, tooltip,
};
use iced::alignment::{Horizontal, Vertical};
use iced::{Alignment, Color, Element, Length, Padding, Point, mouse, window};

use super::keys::{self, Cmd};
use super::motion::{CheckMark, check_mark, hslide, pie, reveal, spinner};
use super::sheet::{Sheet, date_label};
use super::style::{self, FONT, Icon, MEDIUM, Palette, SEMIBOLD, icon, radius, size};
use super::{App, Auth, DropTarget, Message, SettingChange, auth_id, quick_id, sheet_id, title_id};
use crate::i18n::{month_name, t, weekday_name};
use crate::keepass;
use crate::model::{Appearance, Kind, Library, LogInterval, Schedule, Section, Status, Task, View};
use crate::parse;
use crate::vault::MIN_PASSWORD_CHARS;

mod list;
mod lock;
mod quick;
mod sheets;
mod sidebar;

type El<'a> = Element<'a, Message>;

const CONTENT_MAX: f32 = 720.0;
/// Left edge shared by the page title, section headers and the check boxes.
const ROW_INSET: f32 = 12.0;
/// Width of one day cell in the date picker.
const CELL: f32 = 34.0;
/// Room around each check box for the completion ring.
const CHECK_PAD: f32 = 4.0;

fn view_title(lib: &Library, view: View) -> String {
    match view {
        View::Inbox => t("Inbox").to_owned(),
        View::Today => t("Today").to_owned(),
        View::Upcoming => t("Upcoming").to_owned(),
        View::Anytime => t("Anytime").to_owned(),
        View::Someday => t("Someday").to_owned(),
        View::Logbook => t("Logbook").to_owned(),
        View::Trash => t("Trash").to_owned(),
        View::Deadlines => t("Deadlines").to_owned(),
        View::Repeating => t("Repeating").to_owned(),
        View::AllProjects => t("All Projects").to_owned(),
        View::Project(id) => lib.task(id).map_or_else(String::new, |p| p.title.clone()),
        View::Area(id) => lib.area(id).map_or_else(String::new, |a| a.title.clone()),
        View::Tag(id) => lib.tag(id).map_or_else(String::new, |tag| tag.title.clone()),
    }
}

fn view_icon(view: View, p: Palette) -> (Icon, Color) {
    match view {
        View::Inbox => (Icon::Inbox, p.accent),
        View::Today => (Icon::Star, p.gold),
        View::Upcoming => (Icon::Calendar, p.violet),
        View::Anytime => (Icon::Layers, p.teal),
        View::Someday => (Icon::Archive, p.sand),
        View::Logbook => (Icon::Book, p.green),
        View::Trash => (Icon::Trash, p.muted),
        View::Deadlines => (Icon::Flag, p.red),
        View::Repeating => (Icon::Repeat, p.violet),
        View::AllProjects | View::Project(_) => (Icon::Box, p.accent),
        View::Area(_) => (Icon::Layers, p.teal),
        View::Tag(_) => (Icon::Tag, p.teal),
    }
}

fn mix(from: Color, to: Color, k: f32) -> Color {
    let k = k.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * k,
        g: from.g + (to.g - from.g) * k,
        b: from.b + (to.b - from.b) * k,
        a: from.a + (to.a - from.a) * k,
    }
}

fn label<'a>(content: impl text::IntoFragment<'a>, size: impl Into<iced::Pixels>, color: Color) -> iced::widget::Text<'a> {
    text(content).size(size).color(color)
}

impl App {
    pub fn view(&self, window_id: window::Id) -> El<'_> {
        if Some(window_id) == self.quick {
            return self.quick_view();
        }
        let p = self.palette();
        let body: El<'_> = match self.auth {
            Auth::Open => self.workspace(),
            _ => self.lock_screen(),
        };
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::panel(p.page))
            .into()
    }

    fn workspace(&self) -> El<'_> {
        let p = self.palette();
        // The line sits flush against the sidebar; the strip beside it only widens the grab area.
        let divider = mouse_area(row![
            container(space()).width(1).height(Length::Fill).style(style::panel(p.line)),
            container(space()).width(5).height(Length::Fill),
        ])
        .on_press(Message::SidebarGrab)
        .interaction(mouse::Interaction::ResizingHorizontally);
        let divider: El<'_> = if self.compact() {
            container(space()).width(1).height(Length::Fill).style(style::panel(p.line)).into()
        } else {
            divider.into()
        };
        let split = row![
            container(self.sidebar()).width(self.sidebar_width()).height(Length::Fill).style(style::panel(p.sidebar)),
            divider,
            self.main_pane(),
        ];
        let mut layers = stack![split];
        if self.sheet.is_some() {
            layers = layers.push(self.sheet_layer());
        }
        if let Some(toast) = &self.toast {
            let note = container(
                row![
                    icon(Icon::Check, 14, p.green),
                    label(toast.text.as_str(), size::MD, p.text),
                    space().width(8),
                    button(label(t("Undo"), size::MD, p.ink(p.accent)).font(MEDIUM))
                        .padding([3, 8])
                        .on_press(Message::ToastUndo)
                        .style(style::quiet_button(p, false)),
                    keycap("Ctrl", p),
                    keycap("Z", p),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding { top: 6.0, bottom: 6.0, left: 14.0, right: 10.0 })
            .style(style::card(p));
            layers = layers.push(
                container(note)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Bottom)
                    .padding(Padding { bottom: 64.0, left: self.sidebar_width(), ..Padding::ZERO }),
            );
        }
        if let Some(ids) = &self.drag {
            let name = if ids.len() == 1 {
                self.lib().and_then(|l| l.task(ids[0])).map_or_else(String::new, |task| task.title.clone())
            } else {
                format!("{} ×", ids.len())
            };
            let ghost = container(label(name, size::MD, p.text).font(MEDIUM)).padding([5, 10]).style(style::card(p));
            let at = Point::new(self.cursor.x + 12.0, self.cursor.y + 6.0);
            layers = layers.push(pin(ghost).x(at.x).y(at.y));
        }
        layers.into()
    }

    fn tool_button(&self, which: Icon, cmd: Cmd) -> El<'_> {
        let p = self.palette();
        let words = keys::BINDINGS
            .iter()
            .find(|b| b.cmd == cmd)
            .map_or_else(String::new, |b| format!("{}   {}", t(b.label), crate::i18n::keys(&keys::chord_label(b))));
        let control = button(icon(which, 17, p.muted))
            .padding(7)
            .on_press(Message::Command(cmd))
            .style(style::quiet_button(p, false));
        self.with_tip(control.into(), words)
    }

    /// Icon-only controls name themselves, and their shortcut, on hover.
    fn with_tip<'a>(&self, control: El<'a>, words: String) -> El<'a> {
        let p = self.palette();
        if words.is_empty() {
            return control;
        }
        tooltip(control, container(label(words, size::SM, p.text)).padding([5, 9]), tooltip::Position::Top)
            .style(style::tip(p))
            .gap(6)
            .into()
    }

    /// `wanted`, unless the window is too narrow for it.
    fn sheet_width(&self, wanted: f32) -> f32 {
        wanted.min(self.main_size.width - 32.0)
    }

}

/// A key name drawn like a key: a small bordered box.
fn keycap<'a>(name: impl Into<String>, p: Palette) -> El<'a> {
    container(label(crate::i18n::keys(&name.into()), size::XS, p.muted).font(MEDIUM))
        .padding([1, 5])
        .style(style::keycap(p))
        .into()
}

/// "Ctrl+Shift+N" as separate keys; "Ctrl+1–7" keeps its range.
fn chord_parts(chord: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for piece in chord.split('+') {
        if piece.is_empty() {
            parts.push("+".to_owned());
        } else {
            parts.push(piece.to_owned());
        }
    }
    parts.dedup_by(|a, b| a == "+" && b == "+");
    parts
}

fn chord_for(cmd: Cmd) -> Option<String> {
    keys::BINDINGS.iter().find(|b| b.cmd == cmd).map(keys::chord_label)
}

fn tip_text(cmd: Cmd) -> String {
    keys::BINDINGS
        .iter()
        .find(|b| b.cmd == cmd)
        .map_or_else(String::new, |b| format!("{}   {}", t(b.label), crate::i18n::keys(&keys::chord_label(b))))
}

fn tipped(control: El<'static>, words: String, p: Palette) -> El<'static> {
    if words.is_empty() {
        return control;
    }
    tooltip(control, container(label(words, size::SM, p.text)).padding([5, 9]), tooltip::Position::Top)
        .style(style::tip(p))
        .gap(6)
        .into()
}

/// A scrollable with a thin bar that only takes a few pixels at the edge.
fn quiet_scroll<'a>(content: impl Into<El<'a>>, p: Palette) -> scrollable::Scrollable<'a, Message> {
    scrollable(content)
        .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new().width(4).scroller_width(4).margin(2)))
        .style(hidden_scrollbar(p))
}

fn hidden_scrollbar(p: Palette) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style {
    move |theme, status| {
        let mut base = scrollable::default(theme, status);
        let quiet = Color { a: 0.16, ..p.text };
        for rail in [&mut base.vertical_rail, &mut base.horizontal_rail] {
            rail.background = None;
            rail.border = iced::Border::default();
            rail.scroller.background = quiet.into();
            rail.scroller.border = iced::Border { radius: 3.0.into(), ..Default::default() };
        }
        base
    }
}

