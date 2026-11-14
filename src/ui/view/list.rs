//! The page on the right: title, rows, the open to-do and empty states.

use super::*;
use iced::widget::{column, row};

impl App {
    pub(super) fn main_pane(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let (which, tint) = view_icon(self.view, p);
        let mut head = Row::new().spacing(12).align_y(Alignment::Center).padding(Padding { left: ROW_INSET, ..Padding::ZERO });
        head = head.push(match self.view {
            View::Project(id) => pie(lib.progress(id), p.accent, 26.0),
            _ => icon(which, 28, tint),
        });
        head = head.push(label(view_title(lib, self.view), size::DISPLAY, p.text).font(SEMIBOLD));

        let mut page = Column::new()
            .spacing(2)
            .max_width(CONTENT_MAX)
            .padding(if self.compact() {
                Padding { top: 24.0, right: 14.0, bottom: 32.0, left: 14.0 }
            } else {
                Padding { top: 34.0, right: 28.0, bottom: 40.0, left: 28.0 }
            })
            .push(head);

        if let View::Project(_) = self.view {
            page = page.push(space().height(6)).push(
                text_editor(&self.project_notes)
                    .placeholder(t("Notes"))
                    .on_action(Message::ProjectNotes)
                    .size(size::BODY)
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
            page = page.push(self.empty_state(which, tint));
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
        let mut column = Column::new().push(quiet_scroll(animated, p).height(Length::Fill));
        if let Some(error) = self.save_error() {
            column = column.push(container(label(format!("{}: {error}", t("Could not save")), size::SM, p.red)).padding([6, 16]));
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

    /// What an empty list says: why it is empty, and the one key that fills it.
    pub(super) fn empty_state(&self, which: Icon, tint: Color) -> El<'_> {
        let p = self.palette();
        let (title, detail, key): (&str, &str, Option<&str>) = match self.view {
            View::Inbox => ("Inbox is empty", "New to-dos land here until you give them a place.", Some("Ctrl+N")),
            View::Today => ("Nothing planned for today", "Plan a to-do for today with Ctrl+T.", Some("Ctrl+N")),
            View::Upcoming => ("Nothing coming up", "To-dos with a date show up here, day by day.", None),
            View::Anytime => ("Nothing in Anytime", "To-dos you can do whenever you like.", Some("Ctrl+N")),
            View::Someday => ("Nothing for someday", "Park ideas here that are not for now.", Some("Ctrl+N")),
            View::Logbook => ("The Logbook is empty", "Completed to-dos collect here.", None),
            View::Trash => ("Trash is empty", "Deleted to-dos wait here until you empty it.", None),
            View::Deadlines => ("No deadlines", "To-dos with a deadline show up here.", None),
            View::Repeating => ("Nothing repeats", "Make a to-do repeat with Ctrl+Shift+R.", None),
            _ => ("Nothing here yet", "Add the first to-do to this list.", Some("Ctrl+N")),
        };
        let mut content = Column::new()
            .spacing(8)
            .align_x(Alignment::Center)
            .push(icon(which, 40, Color { a: 0.35, ..tint }))
            .push(space().height(4))
            .push(label(t(title), size::BODY, p.text).font(MEDIUM))
            .push(label(t(detail), size::SM, p.muted));
        if let Some(key) = key {
            let mut keys = Row::new().spacing(3).align_y(Alignment::Center);
            for part in chord_parts(key) {
                keys = keys.push(keycap(part, p));
            }
            content = content
                .push(space().height(4))
                .push(row![keys, label(t("new to-do"), size::SM, p.faint)].spacing(8).align_y(Alignment::Center));
        }
        container(content).width(Length::Fill).padding(Padding { top: 90.0, ..Padding::ZERO }).center_x(Length::Fill).into()
    }

    pub(super) fn tag_bar(&self) -> Option<El<'_>> {
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
                button(label(tag.title.as_str(), size::SM, if active { p.on_accent } else { p.muted }).font(MEDIUM))
                    .padding([3, 9])
                    .on_press(Message::TagFilter(Some(tag.id)))
                    .style(move |_, _| button::Style {
                        background: Some(if active { p.accent } else { p.hover }.into()),
                        border: iced::Border { radius: radius::FULL.into(), ..Default::default() },
                        ..button::Style::default()
                    }),
            );
        }
        Some(container(bar.wrap().vertical_spacing(5)).padding(Padding { left: ROW_INSET, ..Padding::ZERO }).into())
    }

    pub(super) fn section_view<'a>(&'a self, lib: &'a Library, section: &'a Section) -> Option<El<'a>> {
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
                        row![icon(Icon::Repeat, size::BODY, p.faint), label(task.title.as_str(), size::BODY, p.muted)]
                            .spacing(11)
                            .align_y(Alignment::Center),
                    )
                    .padding([6, 12]),
                );
            }
        }
        Some(container(out).padding(Padding { bottom: 16.0, ..Padding::ZERO }).into())
    }

    pub(super) fn section_header<'a>(&'a self, _: &'a Library, section: &'a Section, title: &'a str) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        if section.evening {
            return row![icon(Icon::Moon, size::LG, p.accent), label(t("This Evening"), size::BODY, p.text).font(SEMIBOLD)]
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

    pub(super) fn row_view<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
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
                    .size(size::BODY)
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
            head = head.push(label(title, size::BODY, color).wrapping(text::Wrapping::None));
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
            let hovered = self.hovered_row == Some(id) && self.drag.is_none();
            let background = if selected { Some(p.selected) } else if hovered { Some(p.hover) } else { None };
            container(inner).style(style::row_bg(background)).into()
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
            .on_enter(Message::RowHover(id))
            .on_exit(Message::RowLeave(id));
        reveal(area).progress(anim.appear.value).fade(1.0 - anim.appear.value.min(1.0), p.page).into()
    }

    pub(super) fn row_badges<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        let mut badges = Row::new().spacing(7).align_y(Alignment::Center);
        for tag in task.tags.iter().filter_map(|id| lib.tag(*id)).take(3) {
            badges = badges.push(container(label(tag.title.as_str(), size::XS, p.muted)).padding([1, 6]).style(style::chip(p, p.muted)));
        }
        if self.view != View::Project(task.project.unwrap_or_default()) {
            let parent = task
                .project
                .and_then(|id| lib.task(id))
                .map(|project| project.title.as_str())
                .or_else(|| task.area.and_then(|id| lib.area(id)).map(|a| a.title.as_str()));
            if let (Some(parent), true) = (parent, matches!(self.view, View::Today | View::Upcoming | View::Tag(_) | View::Deadlines)) {
                badges = badges.push(label(parent, size::SM, p.faint).wrapping(text::Wrapping::None));
            }
        }
        if !task.notes.is_empty() {
            badges = badges.push(icon(Icon::Notes, 13, p.faint));
        }
        if !task.checklist.is_empty() {
            let done = task.checklist.iter().filter(|c| c.done).count();
            badges = badges.push(label(format!("{done}/{}", task.checklist.len()), size::XS, p.faint));
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
            line = line.push(row![icon(Icon::Flag, size::SM, tint), label(words, size::SM, tint)].spacing(4).align_y(Alignment::Center));
        }
        line.into()
    }

    pub(super) fn closing_details<'a>(&'a self, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let mut out = Column::new().spacing(6).padding(Padding { top: 8.0, left: 32.0, ..Padding::ZERO });
        if !task.notes.is_empty() {
            out = out.push(label(task.notes.as_str(), size::BODY, p.muted));
        }
        for item in &task.checklist {
            out = out.push(label(item.title.as_str(), size::MD, p.muted));
        }
        out = out.push(space().height(22));
        out.into()
    }

    pub(super) fn details<'a>(&'a self, lib: &'a Library, task: &'a Task) -> El<'a> {
        let p = self.palette();
        let today = self.today();
        let mut out = Column::new()
            .spacing(8)
            .padding(Padding { top: 8.0, right: 0.0, bottom: 2.0, left: 32.0 })
            .push(
                text_editor(&self.notes)
                    .placeholder(t("Notes"))
                    .on_action(Message::Notes)
                    .size(size::BODY)
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
                        .size(size::MD)
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
                    .size(size::MD)
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
}

fn detail_chip(which: Icon, words: String, cmd: Cmd, p: Palette) -> El<'static> {
    let chip = button(row![icon(which, size::MD, p.muted), label(words, size::SM, p.muted)].spacing(5).align_y(Alignment::Center))
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
            label(date.day().to_string(), size::XL, p.text).font(SEMIBOLD),
            label(name, size::MD, p.muted).font(MEDIUM),
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
        label(title, size::MD, p.accent).font(SEMIBOLD).width(Length::Fill),
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
    let name = label(title, size::MD, p.text).font(SEMIBOLD);
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
