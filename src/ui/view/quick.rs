//! The Quick Entry popup.

use super::*;
use iced::widget::{column, row};

impl App {
    pub(super) fn quick_view(&self) -> El<'_> {
        let p = self.palette();
        // No entrance animation: the compositor shows the window and its blur
        // in the first frame, so the content has to be there in that frame too.
        let (card_a, input_a, meta_a) = (1.0, 1.0, 1.0);

        let queues = self.quick_queues();
        let content: El<'_> = if !matches!(self.auth, Auth::Open) && !queues {
            self.quick_lock_card()
        } else {
            let input = text_input(t("New to-do"), &self.quick_text)
                .id(quick_id())
                .on_input(Message::QuickInput)
                .on_submit(Message::QuickSubmit)
                .padding(0)
                .size(size::LG)
                .font(MEDIUM)
                .style(style::fading_input(p, input_a));
            let meta: El<'_> = if meta_a < 0.03 {
                space().height(if self.quick_cal.is_some() { 40 } else { 28 }).into()
            } else {
                let details = if queues { self.quick_queue_note() } else { self.quick_meta() };
                reveal(details).fade(1.0 - meta_a, iced::Color { a: self.quick_opacity(), ..p.raised }).slide((1.0 - meta_a) * 4.0).into()
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
            .style(style::quick_card(p, card_a * self.quick_opacity()))
            .into()
    }

    /// Under the input while locked: where the to-do goes, and a way to unlock now.
    fn quick_queue_note(&self) -> El<'_> {
        let p = self.palette();
        let waiting = self.pending_quick.len();
        let note = if waiting == 0 {
            t("Locked. Added when you next unlock gotodo.").to_owned()
        } else {
            format!("{} {waiting} {}", t("Locked. Added when you next unlock gotodo."), t("waiting."))
        };
        row![
            icon(Icon::Lock, 13, p.muted),
            label(note, size::SM, p.muted),
            space().width(Length::Fill),
            button(label(t("Unlock now"), size::SM, p.accent)).padding([2, 6]).on_press(Message::QuickUnlock).style(style::quiet_button(p, false)),
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .height(28)
        .into()
    }

    /// Twelve days in one row, for picking a date without leaving Quick Entry.
    pub(super) fn quick_strip(&self, marker: char, start: NaiveDate) -> El<'_> {
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
            container(label(month_name(start.month()), size::SM, p.muted).font(MEDIUM).wrapping(text::Wrapping::None)).width(66),
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
                    column![label(top, size::XS, top_color), label(date.day().to_string(), size::MD, number_color).font(if is_today { SEMIBOLD } else { FONT })]
                        .spacing(1)
                        .align_x(Alignment::Center),
                )
                .width(33)
                .height(40)
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
    pub(super) fn quick_presets(&self) -> El<'_> {
        let p = self.palette();
        let marker = match self.quick_cal {
            Some(('^', _)) => '^',
            _ => '!',
        };
        let words: Vec<(&'static str, &'static str)> = if marker == '^' {
            vec![("today", t("Today")), ("tomorrow", t("Tomorrow")), ("monday", weekday_name(0)), ("friday", weekday_name(4))]
        } else {
            vec![("today", t("Today")), ("evening", t("This Evening")), ("tomorrow", t("Tomorrow")), ("someday", t("Someday"))]
        };
        let mut line = Row::new().spacing(4).align_y(Alignment::Center);
        for (word, title) in words {
            line = line.push(
                button(label(title, size::SM, p.muted))
                    .padding([4, 9])
                    .on_press(Message::QuickPreset(marker, word))
                    .style(style::quiet_button(p, false)),
            );
        }
        line.into()
    }

    pub(super) fn quick_meta(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        if let Some((marker, start)) = self.quick_cal {
            return self.quick_strip(marker, start);
        }
        let completions = self.quick_completions();
        if !completions.is_empty() {
            let mut line = Row::new().spacing(4).align_y(Alignment::Center);
            for (index, (_, title)) in completions.into_iter().enumerate() {
                line = line.push(
                    button(label(title, size::SM, if index == 0 { p.accent } else { p.text }).font(MEDIUM))
                        .padding([4, 9])
                        .on_press(Message::QuickComplete(index))
                        .style(style::quiet_button(p, index == 0)),
                );
            }
            return line
                .push(space().width(Length::Fill))
                .push(keycap("Tab", p))
                .push(label(t("completes"), size::XS, p.faint))
                .spacing(6)
                .into();
        }
        let typed = !self.quick_text.trim().is_empty();
        let left: El<'_> = if typed { self.quick_chips(lib) } else { self.quick_presets() };
        row![left, space().width(Length::Fill), self.quick_tokens()].align_y(Alignment::Center).into()
    }

    /// What the text already says, as colored chips.
    pub(super) fn quick_chips(&self, lib: &Library) -> El<'_> {
        let p = self.palette();
        let today = self.today();
        let draft = parse::parse_draft(&self.quick_text, lib, today);
        let mut chips: Vec<El<'_>> = Vec::new();
        let mut add = |which: Icon, words: String, tint: Color| {
            chips.push(
                container(
                    row![icon(which, 12, p.ink(tint)), label(words, size::SM, p.ink(tint)).font(MEDIUM).wrapping(text::Wrapping::None)]
                        .spacing(4)
                        .align_y(Alignment::Center),
                )
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
            let words = match when.reminder {
                Some(time) => format!("{words} {}", time.format("%H:%M")),
                None => words,
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
        if chips.is_empty() {
            let place = if lib.settings.quick_entry_to_today { t("Today") } else { t("Inbox") };
            return row![icon(if lib.settings.quick_entry_to_today { Icon::Star } else { Icon::Inbox }, 13, p.faint), label(place, size::SM, p.muted)]
                .spacing(6)
                .align_y(Alignment::Center)
                .into();
        }
        Row::with_children(chips).spacing(5).align_y(Alignment::Center).into()
    }

    /// The four token keys, icon plus the character to type. Names on hover.
    pub(super) fn quick_tokens(&self) -> El<'_> {
        let p = self.palette();
        let token = |which: Icon, words: &'static str, marker: char, message: Message| -> El<'static> {
            let control = button(
                row![icon(which, 13, p.muted), label(marker.to_string(), size::SM, p.faint).font(MEDIUM)]
                    .spacing(3)
                    .align_y(Alignment::Center),
            )
            .padding([4, 6])
            .on_press(message)
            .style(style::quiet_button(p, false))
            .into();
            tipped(control, format!("{}   {marker}", t(words)), p)
        };
        row![
            token(Icon::Calendar, "When", '!', Message::QuickCal('!')),
            token(Icon::Flag, "Deadline", '^', Message::QuickCal('^')),
            token(Icon::Tag, "Tags", '@', Message::QuickToken('@')),
            token(Icon::Box, "Move", '#', Message::QuickToken('#')),
        ]
        .spacing(1)
        .align_y(Alignment::Center)
        .into()
    }
}
