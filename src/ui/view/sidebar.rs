//! The list of lists on the left, or the rail of icons in narrow windows.

use super::*;
use iced::widget::{column, row, stack};

impl App {
    pub(super) fn side_row<'a>(&'a self, view: View, text_label: String, tint_icon: El<'a>, count: Option<(usize, bool)>) -> El<'a> {
        let p = self.palette();
        let dropping = self.drop_target == Some(DropTarget::Side(view));
        let active = self.view == view || dropping;
        let count = count.filter(|(v, _)| *v > 0);
        if self.compact() {
            // The narrow rail: the icon alone, a dot for anything overdue, the name on hover.
            let mut mark = stack![container(tint_icon).width(Length::Fill).center_x(Length::Fill)];
            if let Some((_, true)) = count {
                mark = mark.push(
                    container(container(space()).width(7).height(7).style(style::solid_chip(p.red)))
                        .width(Length::Fill)
                        .align_x(Horizontal::Right)
                        .padding(Padding { right: 6.0, ..Padding::ZERO }),
                );
            }
            let control = mouse_area(
                button(mark).width(Length::Fill).padding([7, 0]).on_press(Message::Go(view)).style(style::quiet_button(p, active)),
            )
            .on_enter(Message::SideHover(view));
            return tooltip(control, container(label(text_label, size::SM, p.text)).padding([5, 9]), tooltip::Position::Right)
                .style(style::tip(p))
                .gap(6)
                .into();
        }
        let mut line = Row::new()
            .spacing(9)
            .align_y(Alignment::Center)
            .push(tint_icon)
            .push(label(text_label, size::MD, p.text).font(if active { MEDIUM } else { FONT }).wrapping(text::Wrapping::None).width(Length::Fill));
        // Holding Ctrl swaps the counts for the keys that reach each list.
        let chord = chord_for(Cmd::Go(view)).filter(|_| self.modifiers.control());
        if let Some(chord) = chord {
            line = line.push(keycap(chord.trim_start_matches("Ctrl+").to_owned(), p));
        } else if let Some((value, alert)) = count {
            line = line.push(if alert {
                Element::from(
                    container(label(value.to_string(), size::XS, p.on_accent).font(SEMIBOLD))
                        .padding([1, 6])
                        .style(style::solid_chip(p.red)),
                )
            } else {
                label(value.to_string(), size::SM, p.muted).into()
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

    pub(super) fn nav_row(&self, view: View, count: Option<(usize, bool)>) -> El<'_> {
        let p = self.palette();
        let (which, tint) = view_icon(view, p);
        let title = self.lib().map_or_else(String::new, |lib| view_title(lib, view));
        self.side_row(view, title, icon(which, 16, tint), count)
    }

    pub(super) fn sidebar(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let today = self.today();
        let overdue = lib.overdue_count(today);
        let side = if self.compact() { 8.0 } else { 10.0 };
        let mut list = Column::new()
            .spacing(1)
            .padding(Padding { top: 16.0, right: side, bottom: 10.0, left: side })
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

        let footer: El<'_> = if self.compact() {
            column![self.tool_button(Icon::Plus, Cmd::NewProject), self.tool_button(Icon::Lock, Cmd::Lock), self.tool_button(Icon::Gear, Cmd::Settings)]
                .spacing(2)
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .padding([6, 0])
                .into()
        } else {
            let new_list = button(
                row![icon(Icon::Plus, 15, p.muted), label(t("New List"), size::MD, p.muted)].spacing(7).align_y(Alignment::Center),
            )
            .padding([6, 9])
            .on_press(Message::Command(Cmd::NewProject))
            .style(style::quiet_button(p, false));
            row![
                self.with_tip(new_list.into(), tip_text(Cmd::NewProject)),
                space().width(Length::Fill),
                self.tool_button(Icon::Lock, Cmd::Lock),
                self.tool_button(Icon::Gear, Cmd::Settings),
            ]
            .spacing(2)
            .align_y(Alignment::Center)
            .padding([6, 8])
            .into()
        };

        column![quiet_scroll(list, p).height(Length::Fill), footer].into()
    }
}
