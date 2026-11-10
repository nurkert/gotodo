//! Everything the user sees.

use chrono::{Datelike, Days, NaiveDate};
use iced::widget::{
    Column, Row, button, center, column, container, hover, mouse_area, opaque, pin, row, scrollable, slider, space, stack,
    text, text_editor, text_input, toggler, tooltip,
};
use iced::alignment::{Horizontal, Vertical};
use iced::{Alignment, Color, Element, Length, Padding, Point, mouse, window};

use super::keys::{self, Cmd};
use super::motion::{CheckMark, check_mark, hslide, pie, reveal};
use super::sheet::{Sheet, date_label};
use super::style::{self, FONT, Icon, MEDIUM, Palette, SEMIBOLD, icon};
use super::{App, Auth, DropTarget, Message, SettingChange, auth_id, quick_id, sheet_id, title_id};
use crate::i18n::{month_name, t, weekday_name};
use crate::keepass;
use crate::model::{Appearance, Kind, Library, LogInterval, Schedule, Section, Status, Task, View};
use crate::parse;
use crate::vault::MIN_PASSWORD_CHARS;

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

    fn auth_card(&self, compact: bool) -> El<'_> {
        let p = self.palette();
        let setup = matches!(self.auth, Auth::Setup);
        let heading = if setup { t("Create a password") } else { t("gotodo is locked") };
        let password = text_input(t("Password"), &self.password)
            .id(auth_id())
            .secure(true)
            .on_input(Message::AuthPassword)
            .on_submit(Message::AuthSubmit)
            .padding(10)
            .size(14)
            .style(style::boxed_input(p));

        let device_button = |primary: bool| -> El<'static> {
            let words = if self.auth_busy { t("Working…") } else { t("Unlock") };
            button(center(text(words).size(14).font(MEDIUM)))
                .height(38)
                .width(Length::Fill)
                .on_press_maybe((!self.auth_busy).then_some(Message::AuthDevice))
                .style(move |theme: &iced::Theme, status| {
                    if primary {
                        style::primary_button(p)(theme, status)
                    } else {
                        style::quiet_button(p, false)(theme, status)
                    }
                })
                .into()
        };

        if compact {
            let error = self.auth_error.as_deref().unwrap_or(if self.slots.password { heading } else { t("gotodo is locked") });
            let color = if self.auth_error.is_some() { p.red } else { p.muted };
            if !self.slots.password && !setup {
                return column![device_button(true), label(error, 12, color)].spacing(8).into();
            }
            return column![password, label(error, 12, color)].spacing(8).into();
        }

        // Only the Quick Entry popup reaches this; the window uses `lock_screen`.
        column![password, label(heading, 12, p.muted)].spacing(8).into()
    }

    /// The full-window lock and welcome screen: one clear action, the other
    /// ways in tucked into quiet links underneath.
    fn lock_screen(&self) -> El<'_> {
        let p = self.palette();
        let setup = matches!(self.auth, Auth::Setup);
        let device_first = !setup && self.slots.device && !self.unlock_password;

        let centered = |content: iced::widget::Text<'static>| -> El<'static> {
            container(content.align_x(Horizontal::Center)).center_x(Length::Fill).into()
        };
        let primary = |words: &'static str, busy_words: &'static str, ready: bool, message: Message| -> El<'static> {
            button(center(text(if self.auth_busy { busy_words } else { words }).size(14).font(MEDIUM)))
                .height(42)
                .width(Length::Fill)
                .on_press_maybe((ready && !self.auth_busy).then_some(message))
                .style(style::primary_button(p))
                .into()
        };
        let link = |words: &'static str, message: Option<Message>| -> El<'static> {
            button(text(t(words)).size(12.5).color(if message.is_some() { p.muted } else { p.faint }))
                .padding([4, 8])
                .on_press_maybe(message)
                .style(style::quiet_button(p, false))
                .into()
        };

        let title = if setup {
            if self.setup_password { t("Create a password") } else { t("Welcome to gotodo") }
        } else {
            t("gotodo is locked")
        };
        let subtitle = if setup && !self.setup_password {
            t("Your to-dos stay on this computer, encrypted. The key is sealed to this machine, so there is no password to type.")
        } else if setup {
            t("Everything you add stays on this device, encrypted with this password. It cannot be recovered if you lose it.")
        } else if device_first {
            t("Your to-dos are encrypted on this device.")
        } else {
            t("Enter your password to continue.")
        };

        let mut card = Column::new()
            .spacing(14)
            .width(340)
            .align_x(Alignment::Center)
            .push(container(icon(Icon::Lock, 28, p.accent)).padding(18).style(style::mark(p)))
            .push(space().height(2))
            .push(centered(label(title, 23, p.text).font(SEMIBOLD)))
            .push(centered(label(subtitle, 13, p.muted).line_height(1.45)))
            .push(space().height(4));

        if setup && !self.setup_password {
            card = card.push(primary("Get started", "Setting up…", true, Message::SetupDevice));
        } else if device_first {
            card = card.push(primary("Unlock", "Unlocking…", true, Message::AuthDevice));
        } else {
            let enough = self.password.chars().count() >= MIN_PASSWORD_CHARS;
            card = card.push(
                text_input(t("Password"), &self.password)
                    .id(auth_id())
                    .secure(true)
                    .on_input(Message::AuthPassword)
                    .on_submit(Message::AuthSubmit)
                    .padding(11)
                    .size(14)
                    .style(style::boxed_input(p)),
            );
            if setup {
                card = card.push(
                    text_input(t("Repeat password"), &self.repeat)
                        .secure(true)
                        .on_input(Message::AuthRepeat)
                        .on_submit(Message::AuthSubmit)
                        .padding(11)
                        .size(14)
                        .style(style::boxed_input(p)),
                );
                card = card.push(primary("Create vault", "Setting up…", enough, Message::AuthSubmit));
            } else {
                card = card.push(primary("Unlock", "Unlocking…", true, Message::AuthSubmit));
            }
        }

        if let Some(error) = &self.auth_error {
            card = card.push(
                row![icon(Icon::Flag, 13, p.red), label(error.as_str(), 12.5, p.red)]
                    .spacing(6)
                    .align_y(Alignment::Center),
            );
        }

        // Other ways in, only the ones that exist, only as quiet text.
        let mut others = Row::new().spacing(2).align_y(Alignment::Center);
        let mut any = false;
        if setup {
            others = others.push(link(
                if self.setup_password { "Use the device key instead" } else { "Use a password instead" },
                Some(Message::SetupPassword(!self.setup_password)),
            ));
            any = true;
        } else {
            if self.slots.password && device_first {
                others = others.push(link("Use password", Some(Message::UnlockMode(true))));
                any = true;
            }
            if self.slots.device && !device_first {
                others = others.push(link("Use this device", Some(Message::UnlockMode(false))));
                any = true;
            }
            if let Auth::Locked { has_keepass: true } = self.auth {
                let ready = self.keepass == keepass::Status::Unlocked;
                others = others.push(if ready {
                    link("Use KeePassXC", Some(Message::AuthKeePass))
                } else {
                    link("KeePassXC is locked", None)
                });
                any = true;
            }
        }
        if any {
            card = card.push(space().height(2)).push(container(others).center_x(Length::Fill));
        }

        let v = self.view_anim.value.clamp(0.0, 1.0);
        let shown = reveal(card).fade(1.0 - v, p.page).slide((1.0 - v) * 10.0);
        let footer = container(
            row![icon(Icon::Lock, 12, p.faint), label(t("Encrypted on this device"), 11.5, p.faint)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(20);
        stack![center(shown), footer].into()
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
        let split = row![
            container(self.sidebar()).width(self.sidebar_width()).height(Length::Fill).style(style::panel(p.sidebar)),
            divider,
            self.main_pane(),
        ];
        let mut layers = stack![split];
        if self.sheet.is_some() {
            layers = layers.push(self.sheet_layer());
        }
        if let Some(ids) = &self.drag {
            let name = if ids.len() == 1 {
                self.lib().and_then(|l| l.task(ids[0])).map_or_else(String::new, |task| task.title.clone())
            } else {
                format!("{} ×", ids.len())
            };
            let ghost = container(label(name, 13, p.text).font(MEDIUM)).padding([5, 10]).style(style::card(p));
            let at = Point::new(self.cursor.x + 12.0, self.cursor.y + 6.0);
            layers = layers.push(pin(ghost).x(at.x).y(at.y));
        }
        layers.into()
    }

    fn side_row<'a>(&'a self, view: View, text_label: String, tint_icon: El<'a>, count: Option<(usize, bool)>) -> El<'a> {
        let p = self.palette();
        let dropping = self.drop_target == Some(DropTarget::Side(view));
        let active = self.view == view || dropping;
        let mut line = Row::new()
            .spacing(9)
            .align_y(Alignment::Center)
            .push(tint_icon)
            .push(label(text_label, 13.5, p.text).font(if active { MEDIUM } else { FONT }).wrapping(text::Wrapping::None).width(Length::Fill));
        // The keyboard way to this list stays visible on the right.
        if let Some(chord) = chord_for(Cmd::Go(view)) {
            line = line.push(label(chord, 11, p.faint).wrapping(text::Wrapping::None));
        }
        if let Some((value, alert)) = count.filter(|(v, _)| *v > 0) {
            line = line.push(if alert {
                Element::from(
                    container(label(value.to_string(), 11, p.on_accent).font(SEMIBOLD))
                        .padding([1, 6])
                        .style(style::solid_chip(p.red)),
                )
            } else {
                label(value.to_string(), 12, p.muted).into()
            });
        }
        mouse_area(
            button(line)
                .width(Length::Fill)
                .padding([5, 9])
                .on_press(Message::Go(view))
                .style(style::quiet_button(p, active)),
        )
        .on_enter(Message::SideHover(view))
        .into()
    }

    fn nav_row(&self, view: View, count: Option<(usize, bool)>) -> El<'_> {
        let p = self.palette();
        let (which, tint) = view_icon(view, p);
        let title = self.lib().map_or_else(String::new, |lib| view_title(lib, view));
        self.side_row(view, title, icon(which, 16, tint), count)
    }

    fn sidebar(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let today = self.today();
        let overdue = lib.overdue_count(today);
        let mut list = Column::new()
            .spacing(1)
            .padding(Padding { top: 16.0, right: 10.0, bottom: 10.0, left: 10.0 })
            .push(self.nav_row(View::Inbox, Some((lib.inbox_count(), false))))
            .push(space().height(10))
            .push(self.nav_row(View::Today, Some((lib.today_count(today), overdue > 0))))
            .push(self.nav_row(View::Upcoming, None))
            .push(self.nav_row(View::Anytime, None))
            .push(self.nav_row(View::Someday, None))
            .push(space().height(10))
            .push(self.nav_row(View::Logbook, None))
            .push(self.nav_row(View::Trash, None))
            .push(space().height(14));

        let project_row = |project: &Task| {
            let share = lib.progress(project.id);
            self.side_row(
                View::Project(project.id),
                project.title.clone(),
                pie(share, p.accent, 14.0),
                Some((lib.open_children(project.id), false)),
            )
        };
        for project in lib.sidebar_projects(None, today) {
            list = list.push(project_row(project));
        }
        for area in lib.areas_sorted() {
            let fold = mouse_area(icon(Icon::Layers, 15, p.teal))
                .on_press(Message::ToggleArea(area.id))
                .interaction(mouse::Interaction::Pointer);
            list = list.push(space().height(10)).push(self.side_row(
                View::Area(area.id),
                area.title.clone(),
                fold.into(),
                None,
            ));
            if !area.collapsed {
                for project in lib.sidebar_projects(Some(area.id), today) {
                    list = list.push(project_row(project));
                }
            }
        }

        let (theme_icon, next_look) = if p.dark { (Icon::Sun, Appearance::Light) } else { (Icon::Moon, Appearance::Dark) };
        let theme_toggle = self.with_tip(
            button(icon(theme_icon, 17, p.muted))
                .padding(7)
                .on_press(Message::Setting(SettingChange::Appearance(next_look)))
                .style(style::quiet_button(p, false))
                .into(),
            t(if p.dark { "Light" } else { "Dark" }).to_owned(),
        );
        let key_toggle = self.with_tip(
            button(icon(Icon::Key, 17, if self.has_vault_keepass() { p.green } else { p.muted }))
                .padding(7)
                .on_press(Message::Command(Cmd::ConnectKeePass))
                .style(style::quiet_button(p, false))
                .into(),
            t("Unlock with KeePassXC").to_owned(),
        );
        let footer = row![
            self.tool_button(Icon::Plus, Cmd::NewProject),
            space().width(Length::Fill),
            key_toggle,
            theme_toggle,
            self.tool_button(Icon::Lock, Cmd::Lock),
            self.tool_button(Icon::Gear, Cmd::Settings),
        ]
        .spacing(2)
        .padding([6, 8]);

        column![scrollable(list).height(Length::Fill).style(hidden_scrollbar(p)), footer].into()
    }

    fn tool_button(&self, which: Icon, cmd: Cmd) -> El<'_> {
        let p = self.palette();
        let words = keys::BINDINGS
            .iter()
            .find(|b| b.cmd == cmd)
            .map_or_else(String::new, |b| format!("{}   {}", t(b.label), keys::chord_label(b)));
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
        tooltip(control, container(label(words, 12, p.text)).padding([5, 9]), tooltip::Position::Top)
            .style(style::tip(p))
            .gap(6)
            .into()
    }

    fn main_pane(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let (which, tint) = view_icon(self.view, p);
        let mut head = Row::new().spacing(12).align_y(Alignment::Center).padding(Padding { left: ROW_INSET, ..Padding::ZERO });
        head = head.push(match self.view {
            View::Project(id) => pie(lib.progress(id), p.accent, 26.0),
            _ => icon(which, 28, tint),
        });
        head = head.push(label(view_title(lib, self.view), 28, p.text).font(SEMIBOLD));

        let mut page = Column::new()
            .spacing(2)
            .max_width(CONTENT_MAX)
            .padding(Padding { top: 34.0, right: 28.0, bottom: 40.0, left: 28.0 })
            .push(head);

        if let View::Project(_) = self.view {
            page = page.push(space().height(6)).push(
                text_editor(&self.project_notes)
                    .placeholder(t("Notes"))
                    .on_action(Message::ProjectNotes)
                    .size(14)
                    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: ROW_INSET })
                    .style(style::bare_editor(p)),
            );
        }
        if let Some(bar) = self.tag_bar() {
            page = page.push(space().height(8)).push(bar);
        }
        page = page.push(space().height(16));

        let empty = self.flat.is_empty() && self.sections.iter().all(|s| s.previews.is_empty() && s.heading.is_none());
        if empty {
            page = page.push(
                container(column![icon(which, 44, Color { a: 0.32, ..tint }), label(t("Nothing here"), 14, p.muted), label(t("Press Ctrl+N to add a to-do"), 12.5, p.faint)].spacing(8).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .padding(Padding { top: 90.0, ..Padding::ZERO })
                    .center_x(Length::Fill),
            );
        } else {
            for section in &self.sections {
                if let Some(element) = self.section_view(lib, section) {
                    page = page.push(element);
                }
            }
        }

        let t_value = self.view_anim.value;
        let centered = container(page).center_x(Length::Fill);
        let animated = reveal(centered).fade(1.0 - t_value, p.page).slide((1.0 - t_value) * 12.0);
        let mut column = Column::new().push(scrollable(animated).height(Length::Fill).style(hidden_scrollbar(p)));
        if let Some(error) = self.save_error() {
            column = column.push(container(label(format!("{}: {error}", t("Could not save")), 12, p.red)).padding([6, 16]));
        }
        column
            .push(container(space()).height(1).width(Length::Fill).style(style::panel(p.line)))
            .push(
                container(
                    row![
                        self.tool_button(Icon::Plus, Cmd::NewTodo),
                        space().width(Length::Fill),
                        self.tool_button(Icon::Calendar, Cmd::When),
                        self.tool_button(Icon::Arrow, Cmd::MoveTo),
                        self.tool_button(Icon::Search, Cmd::Find),
                    ]
                    .spacing(4),
                )
                .padding([6, 12]),
            )
            .width(Length::Fill)
            .into()
    }

    fn tag_bar(&self) -> Option<El<'_>> {
        let p = self.palette();
        let lib = self.lib()?;
        let mut used: Vec<_> = lib
            .tags_sorted()
            .into_iter()
            .filter(|tag| self.flat.iter().filter_map(|id| lib.task(*id)).any(|t| t.tags.contains(&tag.id)))
            .collect();
        if let Some(active) = self.tag_filter {
            if !used.iter().any(|tag| tag.id == active) {
                used.extend(lib.tag(active));
            }
        }
        if used.is_empty() {
            return None;
        }
        let mut bar = Row::new().spacing(5);
        for tag in used {
            let active = self.tag_filter == Some(tag.id);
            bar = bar.push(
                button(label(tag.title.as_str(), 12, if active { p.on_accent } else { p.muted }).font(MEDIUM))
                    .padding([3, 9])
                    .on_press(Message::TagFilter(Some(tag.id)))
                    .style(move |_, _| button::Style {
                        background: Some(if active { p.accent } else { p.hover }.into()),
                        border: iced::Border { radius: 11.0.into(), ..Default::default() },
                        ..button::Style::default()
                    }),
            );
        }
        Some(container(bar.wrap().vertical_spacing(5)).padding(Padding { left: ROW_INSET, ..Padding::ZERO }).into())
    }

    fn section_view<'a>(&'a self, lib: &'a Library, section: &'a Section) -> Option<El<'a>> {
        if section.rows.is_empty() && section.previews.is_empty() && section.heading.is_none() {
            return None;
        }
        let p = self.palette();
        let mut out = Column::new().spacing(1);
        if let Some(title) = &section.title {
            out = out.push(self.section_header(lib, section, title));
        }
        for id in &section.rows {
            if let Some(task) = lib.task(*id) {
                out = out.push(self.row_view(lib, task));
            }
        }
        for (template, _) in &section.previews {
            if let Some(task) = lib.task(*template) {
                out = out.push(
                    container(
                        row![icon(Icon::Repeat, 14, p.faint), label(task.title.as_str(), 14, p.muted)]
                            .spacing(11)
                            .align_y(Alignment::Center),
                    )
                    .padding([6, 12]),
                );
            }
        }
        Some(container(out).padding(Padding { bottom: 16.0, ..Padding::ZERO }).into())
    }

    fn section_header<'a>(&'a self, _: &'a Library, section: &'a Section, title: &'a str) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        if section.evening {
            return row![icon(Icon::Moon, 15, p.accent), label(t("This Evening"), 14, p.text).font(SEMIBOLD)]
                .spacing(7)
                .padding(Padding { top: 8.0, bottom: 7.0, left: ROW_INSET, right: 0.0 })
                .align_y(Alignment::Center)
                .into();
        }
        if self.view == View::Upcoming {
            if let Ok(date) = NaiveDate::parse_from_str(title, "%Y-%m-%d") {
                return upcoming_header(date, today, p);
            }
        }
        if self.view == View::Logbook {
            if let Ok(date) = NaiveDate::parse_from_str(&format!("{title}-01"), "%Y-%m-%d") {
                return group_header(format!("{} {}", month_name(date.month()), date.year()), p, None);
            }
        }
        match section.heading {
            Some(id) => heading_header(id, title.to_owned(), p),
            None => group_header(title.to_owned(), p, section.link),
        }
    }

    fn row_view<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        let id = task.id;
        let anim = self.anim(id);
        let is_open_row = self.expanded == Some(id);
        let selected = self.selection.contains(&id);

        let leading: El<'a> = if task.kind == Kind::Project {
            container(pie(lib.progress(id), p.accent, 17.0)).padding(CHECK_PAD).into()
        } else {
            mouse_area(check_mark(
                CheckMark {
                    progress: anim.check.value,
                    canceled: task.status == Status::Canceled,
                    round: false,
                    outline: if task.deadline.is_some_and(|d| d <= today) && task.is_open() { p.red } else { p.faint },
                    fill: if task.status == Status::Canceled { p.faint } else { p.accent },
                    tick: p.on_accent,
                    pad: CHECK_PAD,
                    rising: anim.check.target > 0.5,
                },
                17.0,
            ))
            .on_press(Message::Toggle(id))
            .interaction(mouse::Interaction::Pointer)
            .into()
        };

        let mut head = Row::new().spacing(11.0 - CHECK_PAD).align_y(Alignment::Center).push(leading);
        if is_open_row {
            head = head.push(
                text_input(t("New to-do"), &self.title)
                    .id(title_id())
                    .on_input(Message::Title)
                    .on_submit(Message::TitleDone)
                    .padding(0)
                    .size(14.5)
                    .font(MEDIUM)
                    .style(style::bare_input(p)),
            );
        } else {
            if self.view != View::Today && task.is_today(today) && task.is_open() {
                head = head.push(icon(if task.evening { Icon::Moon } else { Icon::Star }, 13, if task.evening { p.accent } else { p.gold }));
            }
            let title = if task.title.is_empty() { t("New to-do") } else { task.title.as_str() };
            // The title settles into its faded look together with the check mark.
            let color = if task.title.is_empty() { p.faint } else { mix(p.text, p.faint, anim.check.value) };
            head = head.push(label(title, 14, color).wrapping(text::Wrapping::None));
            head = head.push(self.row_badges(lib, task));
        }

        let progress = anim.expand.value.clamp(0.0, 1.05);
        let mut body = Column::new().push(head);
        if progress > 0.01 {
            let details = if is_open_row { self.details(lib, task) } else { self.closing_details(task) };
            body = body.push(reveal(details).progress(progress).fade(1.0 - progress.min(1.0), p.raised));
        }

        let inset = ROW_INSET - CHECK_PAD;
        let vertical = 6.5 + 7.0 * progress.min(1.0);
        let inner = container(body).padding(Padding { top: vertical, bottom: vertical, left: inset, right: ROW_INSET }).width(Length::Fill);
        let dropping = self.drop_target == Some(DropTarget::Row(id));
        let styled: El<'a> = if progress > 0.5 {
            container(inner).style(style::open_row(p)).into()
        } else {
            container(inner).style(style::row_bg(selected.then_some(p.selected))).into()
        };
        let margin = 11.0 * progress.min(1.0);
        let mut wrapped = Column::new();
        let side = 11.0 * progress.min(1.0);
        if dropping {
            wrapped = wrapped.push(container(space()).height(2).width(Length::Fill).style(style::drop_line(p)));
        }
        wrapped = wrapped.push(container(styled).padding(Padding { top: margin, bottom: margin, left: side, right: side }));

        let area = mouse_area(wrapped)
            .on_press(Message::RowPress(id))
            .on_double_click(Message::RowOpen(id))
            .on_right_press(Message::RowMenu(id))
            .on_enter(Message::RowHover(id));
        reveal(area).progress(anim.appear.value).fade(1.0 - anim.appear.value.min(1.0), p.page).into()
    }

    fn row_badges<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        let mut badges = Row::new().spacing(7).align_y(Alignment::Center);
        for tag in task.tags.iter().filter_map(|id| lib.tag(*id)).take(3) {
            badges = badges.push(container(label(tag.title.as_str(), 11, p.muted)).padding([1, 6]).style(style::chip(p, p.muted)));
        }
        if self.view != View::Project(task.project.unwrap_or_default()) {
            let parent = task
                .project
                .and_then(|id| lib.task(id))
                .map(|project| project.title.as_str())
                .or_else(|| task.area.and_then(|id| lib.area(id)).map(|a| a.title.as_str()));
            if let (Some(parent), true) = (parent, matches!(self.view, View::Today | View::Upcoming | View::Tag(_) | View::Deadlines)) {
                badges = badges.push(label(parent, 12, p.faint).wrapping(text::Wrapping::None));
            }
        }
        if !task.notes.is_empty() {
            badges = badges.push(icon(Icon::Notes, 13, p.faint));
        }
        if !task.checklist.is_empty() {
            let done = task.checklist.iter().filter(|c| c.done).count();
            badges = badges.push(label(format!("{done}/{}", task.checklist.len()), 11, p.faint));
        }
        if task.rule.is_some() || task.template.is_some() {
            badges = badges.push(icon(Icon::Repeat, 13, p.faint));
        }
        if task.reminder.is_some() {
            badges = badges.push(icon(Icon::Bell, 13, p.faint));
        }
        let mut line = Row::new().spacing(8).align_y(Alignment::Center).width(Length::Fill).push(badges);
        line = line.push(space().width(Length::Fill));
        if let Some(deadline) = task.deadline {
            let days = (deadline - today).num_days();
            let urgent = days <= 0 && task.is_open();
            let tint = if urgent { p.red } else { p.muted };
            let words = if days < 0 {
                format!("{} · {}d", t("Overdue"), -days)
            } else if days == 0 {
                t("Today").to_owned()
            } else {
                format!("{days}d {}", t("left"))
            };
            let words = match task.deadline_time {
                Some(time) => format!("{words} {}", time.format("%H:%M")),
                None => words,
            };
            line = line.push(row![icon(Icon::Flag, 12, tint), label(words, 12, tint)].spacing(4).align_y(Alignment::Center));
        }
        line.into()
    }

    fn closing_details<'a>(&'a self, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let mut out = Column::new().spacing(6).padding(Padding { top: 8.0, left: 32.0, ..Padding::ZERO });
        if !task.notes.is_empty() {
            out = out.push(label(task.notes.as_str(), 14, p.muted));
        }
        for item in &task.checklist {
            out = out.push(label(item.title.as_str(), 13, p.muted));
        }
        out = out.push(space().height(22));
        out.into()
    }

    fn details<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        let mut out = Column::new()
            .spacing(8)
            .padding(Padding { top: 8.0, right: 0.0, bottom: 2.0, left: 32.0 })
            .push(
                text_editor(&self.notes)
                    .placeholder(t("Notes"))
                    .on_action(Message::Notes)
                    .size(14)
                    .padding(0)
                    .style(style::bare_editor(p)),
            );

        for item in &task.checklist {
            let id = item.id;
            let tick = mouse_area(check_mark(
                CheckMark {
                    progress: if item.done { 1.0 } else { 0.0 },
                    canceled: false,
                    round: true,
                    outline: p.faint,
                    fill: p.accent,
                    tick: p.on_accent,
                    pad: 0.0,
                    rising: false,
                },
                14.0,
            ))
            .on_press(Message::ChecklistToggle(id));
            out = out.push(
                row![
                    tick,
                    text_input(t("Checklist"), &item.title)
                        .on_input(move |value| Message::ChecklistEdit(id, value))
                        .padding(0)
                        .size(13.5)
                        .style(style::bare_input(p)),
                    button(icon(Icon::Trash, 13, p.faint)).padding(2).on_press(Message::ChecklistRemove(id)).style(style::quiet_button(p, false)),
                ]
                .spacing(9)
                .align_y(Alignment::Center),
            );
        }
        out = out.push(
            row![
                icon(Icon::Plus, 14, p.faint),
                text_input(t("Add item"), &self.checklist_input)
                    .on_input(Message::ChecklistInput)
                    .on_submit(Message::ChecklistAdd)
                    .padding(0)
                    .size(13.5)
                    .style(style::bare_input(p)),
            ]
            .spacing(9)
            .align_y(Alignment::Center),
        );

        let mut chips = Row::new().spacing(6).align_y(Alignment::Center);
        let when_text = match task.schedule {
            Schedule::Inbox => t("When").to_owned(),
            Schedule::Anytime => t("Anytime").to_owned(),
            Schedule::Someday => t("Someday").to_owned(),
            Schedule::On(_) if task.evening => t("This Evening").to_owned(),
            Schedule::On(date) => date_label(date, today),
        };
        let when_text = match task.reminder {
            Some(time) => format!("{when_text} {}", time.format("%H:%M")),
            None => when_text,
        };
        chips = chips.push(detail_chip(Icon::Calendar, when_text, Cmd::When, p));
        let tags = task.tags.iter().filter_map(|id| lib.tag(*id)).map(|tag| tag.title.as_str()).collect::<Vec<_>>().join(", ");
        chips = chips.push(detail_chip(Icon::Tag, if tags.is_empty() { t("Tags").to_owned() } else { tags }, Cmd::Tags, p));
        chips = chips.push(detail_chip(Icon::Flag, task.deadline.map_or_else(|| t("Deadline").to_owned(), |d| date_label(d, today)), Cmd::Deadline, p));
        let place = task
            .project
            .and_then(|id| lib.task(id))
            .map(|project| project.title.clone())
            .or_else(|| task.area.and_then(|id| lib.area(id)).map(|a| a.title.clone()))
            .unwrap_or_else(|| t("Move").to_owned());
        chips = chips.push(detail_chip(Icon::Box, place, Cmd::MoveTo, p));
        chips = chips.push(detail_chip(Icon::Repeat, t("Repeat").to_owned(), Cmd::Repeat, p));
        out.push(chips.wrap().vertical_spacing(5)).into()
    }

    fn sheet_layer(&self) -> El<'_> {
        let p = self.palette();
        let t_value = self.sheet_anim.value.clamp(0.0, 1.0);
        let backdrop_color = Color { a: p.backdrop.a * t_value, ..p.backdrop };
        let backdrop = mouse_area(
            container(space())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style::panel(backdrop_color)),
        )
        .on_press(Message::SheetClose);

        let card: El<'_> = match self.sheet {
            Some(Sheet::Settings) => self.settings_sheet(),
            Some(Sheet::Help) => self.help_sheet(),
            Some(Sheet::KeePass) => self.keepass_sheet(),
            _ => self.picker_sheet(),
        };
        let top = 64.0 - (1.0 - t_value) * 14.0;
        let placed = container(opaque(card)).center_x(Length::Fill).padding(Padding { top, ..Padding::ZERO });
        stack![backdrop, placed].into()
    }

    fn picker_sheet(&self) -> El<'_> {
        let p = self.palette();
        let input = text_input(self.sheet_placeholder(), &self.query)
            .id(sheet_id())
            .on_input(Message::SheetQuery)
            .on_submit(Message::SheetSubmit)
            .padding(14)
            .size(15)
            .style(style::bare_input(p));
        let is_prompt = matches!(self.sheet, Some(Sheet::Prompt(_)));
        let mut list = Column::new().spacing(1).padding(6);
        if !is_prompt {
            for (index, item) in self.sheet_items().into_iter().enumerate().take(60) {
                let mut line = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(icon(item.icon, 15, (item.tint)(&p)))
                    .push(label(item.label, 13.5, p.text).wrapping(text::Wrapping::None).width(Length::Fill));
                if item.checked {
                    line = line.push(icon(Icon::Check, 14, p.accent));
                }
                if let Some(hint) = item.hint {
                    line = line.push(label(hint, 11.5, p.muted));
                }
                list = list.push(
                    button(line)
                        .width(Length::Fill)
                        .padding([6, 9])
                        .on_press(Message::SheetPick(index))
                        .style(style::quiet_button(p, index == self.sheet_index)),
                );
            }
        }
        let with_calendar = matches!(self.sheet, Some(Sheet::Deadline | Sheet::When));
        let mut sheet = Column::new().width(if with_calendar { 760 } else { 480 }).push(input);
        if !is_prompt {
            let results = scrollable(list).height(Length::Shrink).style(hidden_scrollbar(p));
            let body: El<'_> = if with_calendar {
                row![
                    container(results).width(Length::Fill),
                    container(space()).width(1).height(Length::Fill).style(style::panel(p.line)),
                    self.calendar(),
                ]
                .into()
            } else {
                results.into()
            };
            sheet = sheet
                .push(container(space()).height(1).width(Length::Fill).style(style::panel(p.line)))
                .push(body);
        }
        container(sheet).max_height(420).style(style::card(p)).into()
    }

    /// A month grid for picking a date with the mouse. Weeks start on Monday.
    fn calendar(&self) -> El<'_> {
        let p = self.palette();
        let today = self.today();
        let month = self.cal_month;
        let nav = |which: Icon, delta: i32| -> El<'static> {
            button(icon(which, 15, p.muted))
                .padding(6)
                .on_press(Message::CalShift(delta))
                .style(style::quiet_button(p, false))
                .into()
        };
        let head = row![
            label(format!("{} {}", month_name(month.month()), month.year()), 14, p.text).font(SEMIBOLD).width(Length::Fill),
            nav(Icon::Left, -1),
            nav(Icon::Right, 1),
        ]
        .align_y(Alignment::Center);

        let mut names = Row::new();
        for index in 0..7 {
            let short: String = weekday_name(index).chars().take(2).collect();
            names = names.push(container(label(short, 11, p.faint).font(MEDIUM)).width(CELL).center_x(CELL));
        }

        let offset = u64::from(month.weekday().num_days_from_monday());
        let start = month - Days::new(offset);
        let mut grid = Column::new().spacing(2);
        for week in 0..6u64 {
            let mut line = Row::new().spacing(2);
            for weekday in 0..7u64 {
                let date = start + Days::new(week * 7 + weekday);
                let inside = date.month() == month.month();
                let is_today = date == today;
                let color = if is_today {
                    p.on_accent
                } else if inside {
                    p.text
                } else {
                    p.faint
                };
                line = line.push(
                    button(center(label(date.day().to_string(), 12.5, color).font(if is_today { SEMIBOLD } else { FONT })))
                        .width(CELL)
                        .height(30)
                        .padding(0)
                        .on_press(Message::CalPick(date))
                        .style(move |theme: &iced::Theme, status| {
                            if is_today {
                                let mut base = style::primary_button(p)(theme, status);
                                base.border.radius = 15.0.into();
                                base
                            } else {
                                let mut base = style::quiet_button(p, false)(theme, status);
                                base.border.radius = 15.0.into();
                                base
                            }
                        }),
                );
            }
            grid = grid.push(line);
        }
        container(column![head, names, grid].spacing(8)).padding(14).width(CELL * 7.0 + 12.0 + 28.0).into()
    }

    fn help_sheet(&self) -> El<'_> {
        let p = self.palette();
        let mut list = Column::new().spacing(5);
        let mut seen = Vec::new();
        for binding in keys::BINDINGS {
            if seen.contains(&binding.label) {
                continue;
            }
            seen.push(binding.label);
            list = list.push(
                row![
                    label(t(binding.label), 13, p.text).width(Length::Fill),
                    label(keys::chord_label(binding), 12, p.muted).font(MEDIUM)
                ]
                .align_y(Alignment::Center),
            );
        }
        let syntax = column![
            label(t("Quick Entry"), 13, p.text).font(SEMIBOLD),
            label("#project   @tag   !when   ^deadline", 12.5, p.muted),
            label("Kaufen #Haus @laden !morgen ^fr", 12.5, p.faint),
        ]
        .spacing(5);
        let body = column![
            label(t("Shortcuts"), 19, p.text).font(SEMIBOLD),
            scrollable(list.padding(Padding { right: 12.0, ..Padding::ZERO })).height(380).style(hidden_scrollbar(p)),
            syntax,
        ]
        .spacing(10)
        .width(440);
        container(body).padding(20).style(style::card(p)).into()
    }

    fn settings_sheet(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let s = &lib.settings;

        let appearance = row![
            segment(t("Automatic"), s.appearance == Appearance::Auto, SettingChange::Appearance(Appearance::Auto), p),
            segment(t("Light"), s.appearance == Appearance::Light, SettingChange::Appearance(Appearance::Light), p),
            segment(t("Dark"), s.appearance == Appearance::Dark, SettingChange::Appearance(Appearance::Dark), p),
        ]
        .spacing(4);
        let logging = row![
            segment(t("Immediately"), s.log_interval == LogInterval::Immediately, SettingChange::Log(LogInterval::Immediately), p),
            segment(t("Daily"), s.log_interval == LogInterval::Daily, SettingChange::Log(LogInterval::Daily), p),
            segment(t("Manually"), s.log_interval == LogInterval::Manually, SettingChange::Log(LogInterval::Manually), p),
        ]
        .spacing(4);
        let alert = row![
            segment(t("Off"), s.deadline_alert_minutes == 0, SettingChange::DeadlineAlert(0), p),
            segment("15 min", s.deadline_alert_minutes == 15, SettingChange::DeadlineAlert(15), p),
            segment("1 h", s.deadline_alert_minutes == 60, SettingChange::DeadlineAlert(60), p),
            segment("3 h", s.deadline_alert_minutes == 180, SettingChange::DeadlineAlert(180), p),
        ]
        .spacing(4);
        let lock = row![
            segment("5", s.auto_lock_minutes == 5, SettingChange::AutoLock(5), p),
            segment("15", s.auto_lock_minutes == 15, SettingChange::AutoLock(15), p),
            segment("60", s.auto_lock_minutes == 60, SettingChange::AutoLock(60), p),
            segment(t("Never"), s.auto_lock_minutes == 0, SettingChange::AutoLock(0), p),
        ]
        .spacing(4);

        let setting_line = |name: &'static str, control: El<'static>| -> El<'static> {
            row![label(t(name), 13, p.text).width(Length::Fill), control].align_y(Alignment::Center).into()
        };
        let toggle = |name: &'static str, value: bool, change: fn(bool) -> SettingChange| -> El<'static> {
            setting_line(name, toggler(value).on_toggle(move |v| Message::Setting(change(v))).size(18).into())
        };

        let connected = self.has_vault_keepass();
        let keepass_line: El<'_> = row![
            icon(Icon::Key, 16, if connected { p.green } else { p.muted }),
            column![
                label(t("Unlock with KeePassXC"), 13, p.text),
                label(if connected { t("Connected") } else { keepass_status_text(self.keepass) }, 12, p.muted),
            ]
            .spacing(2)
            .width(Length::Fill),
            button(text(if connected { t("Manage") } else { t("Set up") }).size(13))
                .padding([6, 12])
                .on_press(Message::Command(Cmd::ConnectKeePass))
                .style(style::quiet_button(p, false)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into();

        let device_on = self.slots.device;
        let device_line: El<'_> = row![
            icon(Icon::Lock, 16, if device_on { p.green } else { p.muted }),
            column![
                label(t("Unlock with this device"), 13, p.text),
                label(
                    if device_on { t("On. The key is sealed to this computer.") } else { t("Off. A password or KeePassXC opens gotodo.") },
                    12,
                    p.muted,
                ),
            ]
            .spacing(2)
            .width(Length::Fill),
            button(text(if device_on { t("Turn off") } else { t("Turn on") }).size(13))
                .padding([6, 12])
                .on_press(if device_on { Message::DeviceDisable } else { Message::DeviceEnable })
                .style(style::quiet_button(p, false)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into();

        let has_password = self.slots.password;
        let change_ready = (!has_password || !self.password_old.is_empty()) && self.password_new.chars().count() >= MIN_PASSWORD_CHARS;
        let mut security = Column::new()
            .spacing(9)
            .push(heading_label(t("Security"), p))
            .push(setting_line("Auto-lock after", lock.into()))
            .push(toggle("Show titles in reminders", s.reminder_shows_title, SettingChange::ReminderTitles))
            .push(device_line)
            .push(keepass_line);
        let mut password_row = Row::new().spacing(6);
        if has_password {
            password_row = password_row.push(
                text_input(t("Current password"), &self.password_old).secure(true).on_input(Message::PasswordOld).padding(7).size(13).style(style::boxed_input(p)),
            );
        }
        password_row = password_row
            .push(text_input(if has_password { t("New password") } else { t("Recovery password") }, &self.password_new).secure(true).on_input(Message::PasswordNew).padding(7).size(13).style(style::boxed_input(p)))
            .push(
                button(text(if has_password { t("Change password") } else { t("Set password") }).size(13).color(p.on_accent))
                    .padding([7, 12])
                    .on_press_maybe(change_ready.then_some(Message::PasswordChange))
                    .style(style::primary_button(p)),
            );
        security = security.push(password_row);
        if let Some(note) = &self.security_note {
            security = security.push(label(note.as_str(), 12, p.muted));
        }

        let body = column![
            label(t("Settings"), 19, p.text).font(SEMIBOLD),
            heading_label(t("General"), p),
            setting_line("Appearance", appearance.into()),
            setting_line("Text size", slider(0.7..=2.0, s.text_scale, |v| Message::Setting(SettingChange::Scale(v))).step(0.05_f32).width(200).into()),
            setting_line("Log completed items", logging.into()),
            toggle("Group Today by project", s.group_today_by_parent, SettingChange::GroupToday),
            toggle("Quick Entry adds to Today", s.quick_entry_to_today, SettingChange::QuickToday),
            setting_line("Notify before a timed deadline", alert.into()),
            toggle("Reduce motion", s.reduce_motion, SettingChange::ReduceMotion),
            space().height(6),
            security,
        ]
        .spacing(9)
        .width(520);
        container(scrollable(body).style(hidden_scrollbar(p))).max_height(560).padding(22).style(style::card(p)).into()
    }

    /// Plain-language walkthrough for the KeePassXC connection. Each step ticks
    /// itself as soon as the machine is ready, so it reads as progress, not a manual.
    fn keepass_sheet(&self) -> El<'_> {
        let p = self.palette();
        let status = self.keepass;
        let connected = self.has_vault_keepass();
        let running = matches!(status, keepass::Status::Locked | keepass::Status::Unlocked);
        let unlocked = status == keepass::Status::Unlocked;

        let step = |done: bool, title: &'static str, detail: &'static str| -> El<'static> {
            row![
                check_mark(
                    CheckMark {
                        progress: if done { 1.0 } else { 0.0 },
                        canceled: false,
                        round: true,
                        outline: p.faint,
                        fill: p.green,
                        tick: p.on_accent,
                        pad: 0.0,
                        rising: false,
                    },
                    20.0,
                ),
                column![
                    label(t(title), 14, if done { p.muted } else { p.text }).font(MEDIUM),
                    label(t(detail), 12.5, p.muted).line_height(1.4),
                ]
                .spacing(2)
                .width(Length::Fill),
            ]
            .spacing(12)
            .into()
        };

        let action: El<'_> = if connected {
            button(center(text(t("Disconnect KeePassXC")).size(14)))
                .height(38)
                .width(Length::Fill)
                .on_press(Message::KeePassDisconnect)
                .style(style::quiet_button(p, false))
                .into()
        } else {
            button(center(text(t("Connect KeePassXC")).size(14).font(MEDIUM)))
                .height(38)
                .width(Length::Fill)
                .on_press_maybe(unlocked.then_some(Message::KeePassConnect))
                .style(style::primary_button(p))
                .into()
        };

        let hint = if connected {
            t("Done. Next time, gotodo opens by itself while KeePassXC is unlocked.")
        } else if status == keepass::Status::OtherProvider {
            t("GNOME Keyring holds the Secret Service. Quit it (killall gnome-keyring-daemon), then turn Secret Service Integration off and on again in KeePassXC.")
        } else if unlocked {
            t("Everything is ready. One click connects it.")
        } else {
            t("This page updates by itself while you set things up in KeePassXC.")
        };

        let mut body = Column::new()
            .spacing(16)
            .width(460)
            .push(
                row![
                    container(icon(Icon::Key, 22, p.accent)).padding(10).style(style::chip(p, p.accent)),
                    column![
                        label(t("Unlock with KeePassXC"), 19, p.text).font(SEMIBOLD),
                        label(t("No password to type. gotodo opens whenever your KeePassXC database is unlocked."), 13, p.muted)
                            .line_height(1.45),
                    ]
                    .spacing(3)
                    .width(Length::Fill),
                ]
                .spacing(14)
                .align_y(Alignment::Center),
            )
            .push(
                column![
                    step(
                        running,
                        "Turn on Secret Service in KeePassXC",
                        "Tools › Settings › Secret Service Integration, then expose a group for this database.",
                    ),
                    step(unlocked, "Unlock your database", "Open it the way you always do."),
                    step(connected, "Connect", "gotodo stores a random key in that group. It never leaves KeePassXC."),
                ]
                .spacing(14),
            )
            .push(label(hint, 12.5, if status == keepass::Status::OtherProvider { p.red } else { p.muted }))
            .push(action);
        if let Some(note) = &self.security_note {
            body = body.push(label(note.as_str(), 12.5, p.muted));
        }
        body = body.push(
            label(
                t("Lock KeePassXC and gotodo locks with it. Your password keeps working either way."),
                12,
                p.faint,
            )
            .line_height(1.45),
        );
        container(body).padding(24).style(style::card(p)).into()
    }

    fn quick_view(&self) -> El<'_> {
        let p = self.palette();
        let see_through = super::quick_transparent();
        let k = self.quick_anim.value.clamp(0.0, 1.0);
        // One spring drives three staggered stages: the card, the input, the details.
        let card_a = if see_through { (k / 0.4).min(1.0) } else { 1.0 };
        let input_a = ((k - 0.12) / 0.55).clamp(0.0, 1.0);
        let meta_a = ((k - 0.30) / 0.60).clamp(0.0, 1.0);

        let content: El<'_> = if !matches!(self.auth, Auth::Open) {
            self.auth_card(true)
        } else {
            let input = text_input(t("New to-do"), &self.quick_text)
                .id(quick_id())
                .on_input(Message::QuickInput)
                .on_submit(Message::QuickSubmit)
                .padding(0)
                .size(17)
                .font(MEDIUM)
                .style(style::fading_input(p, input_a));
            let meta: El<'_> = if meta_a < 0.03 {
                space().height(if self.quick_cal.is_some() { 72 } else { 58 }).into()
            } else {
                reveal(self.quick_meta()).fade(1.0 - meta_a, p.raised).slide((1.0 - meta_a) * 4.0).into()
            };
            column![
                // Icons cannot fade, so the plus switches on and off mid-way instead.
                row![
                    if input_a > 0.5 { icon(Icon::Plus, 19, p.accent) } else { space().width(19).height(19).into() },
                    input
                ]
                .spacing(11)
                .align_y(Alignment::Center),
                meta,
            ]
            .spacing(10)
            .into()
        };

        container(content)
            .padding([0, 18])
            .width(Length::Fill)
            .center_y(Length::Fill)
            .style(style::quick_card(p, card_a))
            .into()
    }

    /// Twelve days in one row, for picking a date without leaving Quick Entry.
    fn quick_strip(&self, marker: char, start: NaiveDate) -> El<'_> {
        let p = self.palette();
        let today = self.today();
        let chosen = self.lib().and_then(|lib| {
            let draft = parse::parse_draft(&self.quick_text, lib, today);
            if marker == '^' {
                draft.deadline
            } else {
                draft.when.and_then(|w| match w.schedule {
                    Schedule::On(date) => Some(date),
                    _ => None,
                })
            }
        });
        let nav = |which: Icon, weeks: i32| -> El<'static> {
            button(icon(which, 14, p.muted))
                .padding([8, 6])
                .on_press(Message::QuickCalShift(weeks))
                .style(style::quiet_button(p, false))
                .into()
        };
        let mut line = Row::new().spacing(2).align_y(Alignment::Center);
        line = line.push(
            container(label(month_name(start.month()), 12, p.muted).font(MEDIUM).wrapping(text::Wrapping::None)).width(66),
        );
        line = line.push(nav(Icon::Left, -1));
        // A week of spare days on each side lets the strip glide between pages.
        let mut days = Row::new().spacing(2).align_y(Alignment::Center);
        for offset in 0..26u64 {
            let date = start + chrono::Days::new(offset) - chrono::Days::new(7);
            let is_today = date == today;
            let is_chosen = chosen == Some(date);
            let top = if date.day() == 1 {
                month_name(date.month()).chars().take(3).collect::<String>()
            } else {
                weekday_name(date.weekday().num_days_from_monday() as usize).chars().take(2).collect::<String>()
            };
            let number_color = if is_chosen { p.on_accent } else if is_today { p.accent } else { p.text };
            let top_color = if is_chosen { p.on_accent } else { p.faint };
            days = days.push(
                button(
                    column![label(top, 9.5, top_color), label(date.day().to_string(), 13, number_color).font(if is_today { SEMIBOLD } else { FONT })]
                        .spacing(1)
                        .align_x(Alignment::Center),
                )
                .width(33)
                .height(38)
                .padding(0)
                .on_press(Message::QuickCalPick(date))
                .style(move |theme: &iced::Theme, status| {
                    if is_chosen {
                        style::primary_button(p)(theme, status)
                    } else {
                        style::quiet_button(p, false)(theme, status)
                    }
                }),
            );
        }
        let pitch = 33.0 + 2.0;
        line = line.push(hslide(days, (7.0 + self.quick_cal_slide.value) * pitch));
        line = line.push(nav(Icon::Right, 1));
        line.into()
    }

    /// The one-click schedule words that stay visible while typing.
    fn quick_presets(&self) -> El<'_> {
        let p = self.palette();
        let marker = match self.quick_cal {
            Some(('^', _)) => '^',
            _ => '!',
        };
        let words: Vec<(&'static str, &'static str)> = if marker == '^' {
            vec![("today", t("Today")), ("tomorrow", t("Tomorrow")), ("monday", weekday_name(0)), ("friday", weekday_name(4))]
        } else {
            vec![
                ("today", t("Today")),
                ("tomorrow", t("Tomorrow")),
                ("evening", t("This Evening")),
                ("someday", t("Someday")),
                ("anytime", t("Anytime")),
            ]
        };
        let mut line = Row::new().spacing(4).align_y(Alignment::Center);
        for (word, title) in words {
            line = line.push(
                button(label(title, 12.5, p.text))
                    .padding([3, 8])
                    .on_press(Message::QuickPreset(marker, word))
                    .style(style::quiet_button(p, false)),
            );
        }
        line.into()
    }

    fn quick_meta(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let completions = self.quick_completions();
        let upper: El<'_> = if !completions.is_empty() {
            let mut line = Row::new().spacing(6).align_y(Alignment::Center);
            for (index, (_, title)) in completions.into_iter().enumerate() {
                line = line.push(
                    button(label(title, 12.5, if index == 0 { p.accent } else { p.text }).font(MEDIUM))
                        .padding([3, 8])
                        .on_press(Message::QuickComplete(index))
                        .style(style::quiet_button(p, index == 0)),
                );
            }
            line.push(space().width(Length::Fill)).push(label(t("Tab completes"), 11.5, p.faint)).into()
        } else {
            self.quick_presets()
        };
        let lower: El<'_> = match self.quick_cal {
            Some((marker, start)) => self.quick_strip(marker, start),
            None => self.quick_details(lib),
        };
        column![upper, lower].spacing(10).into()
    }

    /// What the text already says, plus the buttons that add more.
    fn quick_details(&self, lib: &Library) -> El<'_> {
        let p = self.palette();
        let today = self.today();
        let draft = parse::parse_draft(&self.quick_text, lib, today);
        let mut chips: Vec<El<'_>> = Vec::new();
        let mut add = |which: Icon, words: String, tint: Color| {
            chips.push(
                container(row![icon(which, 12, tint), label(words, 12, tint).font(MEDIUM)].spacing(4).align_y(Alignment::Center))
                    .padding([3, 7])
                    .style(style::chip(p, tint))
                    .into(),
            );
        };
        if let Some((_, title)) = draft.target {
            add(Icon::Box, title, p.accent);
        }
        if let Some(when) = draft.when {
            let words = match when.schedule {
                Schedule::On(_) if when.evening => t("This Evening").to_owned(),
                Schedule::On(date) => date_label(date, today),
                Schedule::Someday => t("Someday").to_owned(),
                Schedule::Anytime => t("Anytime").to_owned(),
                Schedule::Inbox => t("Inbox").to_owned(),
            };
            add(Icon::Calendar, words, p.gold);
        }
        if let Some(deadline) = draft.deadline {
            let words = match draft.deadline_time {
                Some(time) => format!("{} {}", date_label(deadline, today), time.format("%H:%M")),
                None => date_label(deadline, today),
            };
            add(Icon::Flag, words, p.red);
        }
        for (_, name) in draft.tags {
            add(Icon::Tag, name, p.teal);
        }
        // The typed marker is part of each label, so the keyboard way is always visible.
        let token = |which: Icon, words: &'static str, marker: char, message: Message| -> El<'static> {
            button(
                row![icon(which, 13, p.muted), label(t(words), 12, p.muted), label(marker.to_string(), 11, p.faint)]
                    .spacing(5)
                    .align_y(Alignment::Center),
            )
            .padding([3, 7])
            .on_press(message)
            .style(style::quiet_button(p, false))
            .into()
        };
        let options = Row::new()
            .spacing(2)
            .align_y(Alignment::Center)
            .push(token(Icon::Calendar, "When", '!', Message::QuickCal('!')))
            .push(token(Icon::Tag, "Tags", '@', Message::QuickToken('@')))
            .push(token(Icon::Flag, "Deadline", '^', Message::QuickCal('^')))
            .push(token(Icon::Box, "Move", '#', Message::QuickToken('#')));
        let mut line = Row::with_children(chips).spacing(6).align_y(Alignment::Center);
        line = line.push(space().width(Length::Fill)).push(options);
        line.into()
    }
}

fn keepass_status_text(status: keepass::Status) -> &'static str {
    match status {
        keepass::Status::Unlocked => t("KeePassXC is unlocked"),
        keepass::Status::Locked => t("KeePassXC is locked"),
        keepass::Status::Missing => t("KeePassXC not found"),
        keepass::Status::OtherProvider => t("Another secret service is running, not KeePassXC"),
    }
}

fn heading_label<'a>(name: &'static str, p: Palette) -> iced::widget::Text<'a> {
    label(name, 12, p.muted).font(SEMIBOLD)
}

fn segment(name: &'static str, active: bool, change: SettingChange, p: Palette) -> El<'static> {
    button(label(name, 12.5, p.text))
        .padding([4, 9])
        .on_press(Message::Setting(change))
        .style(style::quiet_button(p, active))
        .into()
}

fn chord_for(cmd: Cmd) -> Option<String> {
    keys::BINDINGS.iter().find(|b| b.cmd == cmd).map(keys::chord_label)
}

fn tip_text(cmd: Cmd) -> String {
    keys::BINDINGS
        .iter()
        .find(|b| b.cmd == cmd)
        .map_or_else(String::new, |b| format!("{}   {}", t(b.label), keys::chord_label(b)))
}

fn tipped(control: El<'static>, words: String, p: Palette) -> El<'static> {
    if words.is_empty() {
        return control;
    }
    tooltip(control, container(label(words, 12, p.text)).padding([5, 9]), tooltip::Position::Top)
        .style(style::tip(p))
        .gap(6)
        .into()
}

fn detail_chip(which: Icon, words: String, cmd: Cmd, p: Palette) -> El<'static> {
    let chip = button(row![icon(which, 13, p.muted), label(words, 12, p.muted)].spacing(5).align_y(Alignment::Center))
        .padding([3, 7])
        .on_press(Message::Command(cmd))
        .style(style::quiet_button(p, false))
        .into();
    tipped(chip, tip_text(cmd), p)
}

fn upcoming_header<'a>(date: NaiveDate, today: NaiveDate, p: Palette) -> El<'a> {
    let name = if (date - today).num_days() == 1 {
        t("Tomorrow").to_owned()
    } else {
        weekday_name(date.weekday().num_days_from_monday() as usize).to_owned()
    };
    container(
        row![
            label(date.day().to_string(), 22, p.text).font(SEMIBOLD),
            label(name, 13.5, p.muted).font(MEDIUM),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(Padding { top: 6.0, bottom: 6.0, left: ROW_INSET, right: 0.0 })
    .into()
}

fn heading_header<'a>(id: crate::model::Id, title: String, p: Palette) -> El<'a> {
    let action = |which: Icon, message: Message| -> El<'a> {
        button(icon(which, 13, p.faint)).padding(3).on_press(message).style(style::quiet_button(p, false)).into()
    };
    let line = row![
        label(title, 13.5, p.accent).font(SEMIBOLD).width(Length::Fill),
        hover(
            space().width(66).height(20),
            row![
                action(Icon::Notes, Message::HeadingRename(id)),
                action(Icon::Archive, Message::HeadingArchive(id)),
                action(Icon::Trash, Message::HeadingDelete(id)),
            ]
            .spacing(2),
        ),
    ]
    .align_y(Alignment::Center);
    column![
        container(line).padding(Padding { top: 8.0, bottom: 4.0, left: ROW_INSET, right: 0.0 }),
        container(space()).height(1).width(Length::Fill).style(style::panel(p.line)),
        space().height(4),
    ]
    .into()
}

fn group_header<'a>(title: String, p: Palette, link: Option<View>) -> El<'a> {
    let name = label(title, 13.5, p.text).font(SEMIBOLD);
    let content: El<'a> = match link {
        Some(view) => button(name).padding([2, 4]).on_press(Message::Go(view)).style(style::quiet_button(p, false)).into(),
        None => name.into(),
    };
    let left = if link.is_some() { ROW_INSET - 4.0 } else { ROW_INSET };
    column![
        container(content).padding(Padding { top: 8.0, bottom: 5.0, left, right: 0.0 }),
        container(space()).height(1).width(Length::Fill).style(style::panel(p.line)),
        space().height(4),
    ]
    .into()
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

