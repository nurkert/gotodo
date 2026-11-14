//! Sheets over the window: pickers, the calendar, shortcuts, settings, KeePassXC.

use super::*;
use iced::widget::{column, row, stack};

impl App {
    pub(super) fn sheet_layer(&self) -> El<'_> {
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

    pub(super) fn picker_sheet(&self) -> El<'_> {
        let p = self.palette();
        let input = text_input(self.sheet_placeholder(), &self.query)
            .id(sheet_id())
            .on_input(Message::SheetQuery)
            .on_submit(Message::SheetSubmit)
            .padding(14)
            .size(size::LG)
            .style(style::bare_input(p));
        let is_prompt = matches!(self.sheet, Some(Sheet::Prompt(_)));
        let mut list = Column::new().spacing(1).padding(6);
        if !is_prompt {
            for (index, item) in self.sheet_items().into_iter().enumerate().take(60) {
                let mut line = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(icon(item.icon, 15, (item.tint)(&p)))
                    .push(label(item.label, size::MD, p.text).wrapping(text::Wrapping::None).width(Length::Fill));
                if item.checked {
                    line = line.push(icon(Icon::Check, 14, p.accent));
                }
                if let Some(hint) = item.hint {
                    line = line.push(label(hint, size::SM, p.muted));
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
        let with_calendar = matches!(self.sheet, Some(Sheet::Deadline | Sheet::When)) && self.main_size.width >= 820.0;
        let mut sheet = Column::new().width(self.sheet_width(if with_calendar { 760.0 } else { 480.0 })).push(input);
        if !is_prompt {
            let results = quiet_scroll(list, p).height(Length::Shrink);
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
    pub(super) fn calendar(&self) -> El<'_> {
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
            label(format!("{} {}", month_name(month.month()), month.year()), size::BODY, p.text).font(SEMIBOLD).width(Length::Fill),
            nav(Icon::Left, -1),
            nav(Icon::Right, 1),
        ]
        .align_y(Alignment::Center);

        let mut names = Row::new();
        for index in 0..7 {
            let short: String = weekday_name(index).chars().take(2).collect();
            names = names.push(container(label(short, size::XS, p.faint).font(MEDIUM)).width(CELL).center_x(CELL));
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
                    button(center(label(date.day().to_string(), size::MD, color).font(if is_today { SEMIBOLD } else { FONT })))
                        .width(CELL)
                        .height(30)
                        .padding(0)
                        .on_press(Message::CalPick(date))
                        .style(move |theme: &iced::Theme, status| {
                            if is_today {
                                let mut base = style::primary_button(p)(theme, status);
                                base.border.radius = radius::FULL.into();
                                base
                            } else {
                                let mut base = style::quiet_button(p, false)(theme, status);
                                base.border.radius = radius::FULL.into();
                                base
                            }
                        }),
                );
            }
            grid = grid.push(line);
        }
        container(column![head, names, grid].spacing(8)).padding(14).width(CELL * 7.0 + 12.0 + 28.0).into()
    }

    /// All shortcuts, grouped by what they are for, as key caps.
    pub(super) fn help_sheet(&self) -> El<'_> {
        let p = self.palette();
        let entry = |words: &'static str, chord: String| -> El<'static> {
            let mut keys = Row::new().spacing(3).align_y(Alignment::Center);
            for part in chord_parts(&chord) {
                keys = keys.push(keycap(part, p));
            }
            row![label(t(words), size::MD, p.text).width(Length::Fill), keys].spacing(12).align_y(Alignment::Center).into()
        };
        let bound = |cmd: Cmd| chord_for(cmd).unwrap_or_default();
        let group = |title: &'static str, rows: Vec<El<'static>>| -> El<'static> {
            let mut column = Column::new().spacing(7).push(
                container(label(t(title), size::SM, p.muted).font(SEMIBOLD)).padding(Padding { bottom: 2.0, ..Padding::ZERO }),
            );
            for row in rows {
                column = column.push(row);
            }
            column.into()
        };
        let create = group(
            "Create",
            vec![
                entry("New To-Do", bound(Cmd::NewTodo)),
                entry("Quick Entry", bound(Cmd::QuickEntry)),
                entry("New Project", bound(Cmd::NewProject)),
                entry("New Heading", bound(Cmd::NewHeading)),
                entry("New Area", bound(Cmd::NewArea)),
                entry("Duplicate", bound(Cmd::Duplicate)),
            ],
        );
        let schedule = group(
            "Schedule",
            vec![
                entry("When", bound(Cmd::When)),
                entry("Today", bound(Cmd::SetToday)),
                entry("This Evening", bound(Cmd::SetEvening)),
                entry("Anytime", bound(Cmd::SetAnytime)),
                entry("Someday", bound(Cmd::SetSomeday)),
                entry("Deadline", bound(Cmd::Deadline)),
                entry("Repeat", bound(Cmd::Repeat)),
            ],
        );
        let organize = group(
            "Organize",
            vec![
                entry("Complete", bound(Cmd::Complete)),
                entry("Cancel to-do", bound(Cmd::Cancel)),
                entry("Move", bound(Cmd::MoveTo)),
                entry("Tags", bound(Cmd::Tags)),
                entry("Rename", bound(Cmd::Rename)),
                entry("Move Up", bound(Cmd::MoveUp)),
                entry("Move Down", bound(Cmd::MoveDown)),
                entry("Select all", bound(Cmd::SelectAll)),
                entry("Delete", "Del".to_owned()),
            ],
        );
        let navigate = group(
            "Navigate",
            vec![
                entry("Go to a list", "Ctrl+1–7".to_owned()),
                entry("Quick Find", bound(Cmd::Find)),
                entry("Commands", bound(Cmd::Palette)),
                entry("Open", "Enter".to_owned()),
                entry("Close", "Esc".to_owned()),
            ],
        );
        let app = group(
            "App",
            vec![
                entry("Undo", bound(Cmd::Undo)),
                entry("Redo", bound(Cmd::Redo)),
                entry("Zoom", "Ctrl+± 0".to_owned()),
                entry("Settings", bound(Cmd::Settings)),
                entry("Lock", bound(Cmd::Lock)),
                entry("Shortcuts", bound(Cmd::Help)),
            ],
        );
        let syntax = column![
            label(t("Quick Entry"), size::SM, p.muted).font(SEMIBOLD),
            row![
                keycap("#", p),
                label(t("project"), size::SM, p.text),
                keycap("@", p),
                label(t("tag"), size::SM, p.text),
                keycap("!", p),
                label(t("when"), size::SM, p.text),
                keycap("^", p),
                label(t("deadline"), size::SM, p.text),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            label("Kaufen #Haus @laden !morgen-9:00 ^fr", size::SM, p.muted),
        ]
        .spacing(7);

        let wide = self.main_size.width >= 900.0;
        let columns: El<'_> = if wide {
            row![
                column![create, schedule].spacing(20).width(Length::FillPortion(1)),
                column![organize, navigate].spacing(20).width(Length::FillPortion(1)),
                column![app, syntax].spacing(20).width(Length::FillPortion(1)),
            ]
            .spacing(32)
            .into()
        } else {
            column![create, schedule, organize, navigate, app, syntax].spacing(20).into()
        };
        let body = column![label(t("Shortcuts"), size::XL, p.text).font(SEMIBOLD), columns].spacing(18);
        container(quiet_scroll(body.padding(Padding { right: 10.0, ..Padding::ZERO }), p))
            .width(self.sheet_width(if wide { 820.0 } else { 420.0 }))
            .max_height(self.sheet_max_height())
            .padding(Padding { top: 22.0, right: 12.0, bottom: 22.0, left: 22.0 })
            .style(style::card(p))
            .into()
    }

    pub(super) fn settings_sheet(&self) -> El<'_> {
        let p = self.palette();
        let Some(lib) = self.lib() else { return space().into() };
        let s = &lib.settings;
        let look = self.prefs;
        let set = |change: SettingChange| Message::Setting(change);

        let appearance = segmented(
            vec![
                (t("Automatic"), look.appearance == Appearance::Auto, set(SettingChange::Appearance(Appearance::Auto))),
                (t("Light"), look.appearance == Appearance::Light, set(SettingChange::Appearance(Appearance::Light))),
                (t("Dark"), look.appearance == Appearance::Dark, set(SettingChange::Appearance(Appearance::Dark))),
            ],
            p,
        );
        let scale = row![
            slider(0.7..=2.0, look.text_scale, |v| Message::Setting(SettingChange::Scale(v))).step(0.05_f32).width(160),
            container(label(format!("{:.0} %", look.text_scale * 100.0), size::SM, p.muted)).width(44).align_x(Horizontal::Right),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let logging = segmented(
            vec![
                (t("Immediately"), s.log_interval == LogInterval::Immediately, set(SettingChange::Log(LogInterval::Immediately))),
                (t("Daily"), s.log_interval == LogInterval::Daily, set(SettingChange::Log(LogInterval::Daily))),
                (t("Manually"), s.log_interval == LogInterval::Manually, set(SettingChange::Log(LogInterval::Manually))),
            ],
            p,
        );
        let minutes = |m: u32| -> String {
            match m {
                0 => t("Off").to_owned(),
                m if m % 60 == 0 => format!("{} h", m / 60),
                m => format!("{m} min"),
            }
        };
        let alert = segmented(
            [0, 15, 60, 180]
                .into_iter()
                .map(|m| (minutes(m), s.deadline_alert_minutes == m, set(SettingChange::DeadlineAlert(m))))
                .collect(),
            p,
        );
        let lock = segmented(
            [5, 15, 60, 0]
                .into_iter()
                .map(|m| (if m == 0 { t("Never").to_owned() } else { minutes(m) }, s.auto_lock_minutes == m, set(SettingChange::AutoLock(m))))
                .collect(),
            p,
        );
        let switch = |value: bool, change: fn(bool) -> SettingChange| -> El<'static> {
            toggler(value).on_toggle(move |v| Message::Setting(change(v))).size(18).into()
        };
        let action = |words: &'static str, message: Message| -> El<'static> {
            button(text(t(words)).size(size::MD)).padding([6, 12]).on_press(message).style(style::secondary_button(p)).into()
        };

        let general = settings_group(
            vec![
                setting_row("Appearance", None, appearance, p),
                setting_row("Text size", None, scale.into(), p),
                setting_row("Reduce motion", None, switch(look.reduce_motion, SettingChange::ReduceMotion), p),
            ],
            p,
        );
        let lists = settings_group(
            vec![
                setting_row("Move completed to the Logbook", None, logging, p),
                setting_row("Group Today by project", None, switch(s.group_today_by_parent, SettingChange::GroupToday), p),
                setting_row("Quick Entry adds to Today", Some(t("Otherwise new to-dos land in the Inbox.")), switch(s.quick_entry_to_today, SettingChange::QuickToday), p),
                setting_row("Notify before a timed deadline", None, alert, p),
            ],
            p,
        );

        let connected = self.has_vault_keepass();
        let device_on = self.slots.device;
        let mut security_rows = vec![
            setting_row("Lock after inactivity", None, lock, p),
            setting_row(
                "Show titles in notifications",
                Some(t("Off: a notification only says that a to-do is due.")),
                switch(s.reminder_shows_title, SettingChange::ReminderTitles),
                p,
            ),
            setting_row(
                "Unlock with this device",
                Some(if device_on { t("On. The key is sealed to this computer.") } else { t("Off. A password or KeePassXC opens gotodo.") }),
                action(if device_on { "Turn off" } else { "Turn on" }, if device_on { Message::DeviceDisable } else { Message::DeviceEnable }),
                p,
            ),
            setting_row(
                "Unlock with KeePassXC",
                Some(if connected { t("Connected") } else { keepass_status_text(self.keepass) }),
                action(if connected { "Manage" } else { "Set up" }, Message::Command(Cmd::ConnectKeePass)),
                p,
            ),
        ];
        if device_on && self.can_pause_device() {
            security_rows.push(setting_row(
                "Require password after locking",
                Some(t("The device key stays unused until you unlock with your password or KeePassXC.")),
                switch(look.lock_needs_secret, SettingChange::LockNeedsSecret),
                p,
            ));
        }
        let security = settings_group(security_rows, p);

        let has_password = self.slots.password;
        let new_count = self.password_new.chars().count();
        let repeat_wrong = !self.password_repeat.is_empty() && self.password_repeat != self.password_new;
        let change_ready = (!has_password || !self.password_old.is_empty())
            && new_count >= MIN_PASSWORD_CHARS
            && self.password_repeat == self.password_new;
        let small_field = |placeholder: &'static str, value: &str, error: bool, on_input: fn(String) -> Message| -> El<'static> {
            text_input(t(placeholder), value)
                .secure(true)
                .on_input(on_input)
                .padding([7, 10])
                .size(size::MD)
                .width(240)
                .style(style::field(p, error))
                .into()
        };
        let mut password_rows = Vec::new();
        if has_password {
            password_rows.push(setting_row("Current password", None, small_field("Current password", &self.password_old, false, Message::PasswordOld), p));
        }
        let new_hint = if new_count == 0 || new_count >= MIN_PASSWORD_CHARS {
            None
        } else {
            Some(format!("{} {} {}", MIN_PASSWORD_CHARS - new_count, t("more"), t("characters")))
        };
        password_rows.push(setting_row_owned(
            t(if has_password { "New password" } else { "Recovery password" }),
            new_hint.or_else(|| (!has_password).then(|| t("Opens gotodo if this computer or KeePassXC cannot.").to_owned())),
            small_field(if has_password { "New password" } else { "Recovery password" }, &self.password_new, false, Message::PasswordNew),
            p,
        ));
        password_rows.push(setting_row(
            "Repeat",
            repeat_wrong.then(|| t("Passwords do not match")),
            small_field("Repeat password", &self.password_repeat, repeat_wrong, Message::PasswordRepeat),
            p,
        ));
        let change = button(text(t(if has_password { "Change password" } else { "Set password" })).size(size::MD).font(MEDIUM))
            .padding([7, 14])
            .on_press_maybe(change_ready.then_some(Message::PasswordChange))
            .style(style::primary_button(p));
        password_rows.push(
            container(change).width(Length::Fill).align_x(Horizontal::Right).padding([10, 14]).into(),
        );
        let password = settings_group(password_rows, p);

        let mut backup_rows: Vec<El<'_>> = Vec::new();
        backup_rows.push(setting_row(
            "Export a copy",
            Some(t("Saves the encrypted file to your Documents folder.")),
            action("Export", Message::BackupExport),
            p,
        ));
        if self.backups.is_empty() {
            backup_rows.push(setting_row("Daily copies", Some(t("The first copy is made the next time something changes.")), space().into(), p));
        }
        for backup in self.backups.iter().take(5) {
            let armed = self.restore_armed.as_ref() == Some(&backup.path);
            let restore: El<'_> = button(text(t(if armed { "Restore now" } else { "Restore" })).size(size::MD))
                .padding([6, 12])
                .on_press(Message::BackupRestore(backup.path.clone()))
                .style(move |theme: &iced::Theme, status| {
                    if armed { style::danger_button(p)(theme, status) } else { style::secondary_button(p)(theme, status) }
                })
                .into();
            let sub = armed.then(|| t("Locks gotodo and replaces your to-dos with this copy. Today's state is kept.").to_owned());
            backup_rows.push(setting_row_owned(date_label(backup.date, self.today()), sub, restore, p));
        }
        let backups = settings_group(backup_rows, p);

        let mut body = Column::new().spacing(8).width(self.sheet_width(560.0) - 34.0).push(label(t("Settings"), size::XL, p.text).font(SEMIBOLD));
        if let Some(note) = &self.security_note {
            body = body.push(
                container(row![icon(Icon::Check, 14, p.accent), label(note.as_str(), size::SM, p.text)].spacing(8).align_y(Alignment::Center))
                    .padding([8, 12])
                    .width(Length::Fill)
                    .style(style::chip(p, p.accent)),
            );
        }
        body = body
            .push(section_label(t("General"), p))
            .push(general)
            .push(section_label(t("Lists"), p))
            .push(lists)
            .push(section_label(t("Security"), p))
            .push(security)
            .push(section_label(t("Password"), p))
            .push(password)
            .push(section_label(t("Backups"), p))
            .push(label(
                format!("{} {} {}", t("gotodo keeps an encrypted copy from each of the last"), crate::backup::KEEP, t("days. Changing the password starts over.")),
                size::SM,
                p.muted,
            ))
            .push(backups);
        container(quiet_scroll(body.padding(Padding { right: 10.0, ..Padding::ZERO }), p))
            .max_height(self.sheet_max_height())
            .padding(Padding { top: 22.0, right: 12.0, bottom: 22.0, left: 22.0 })
            .style(style::card(p))
            .into()
    }

    /// Plain-language walkthrough for the KeePassXC connection. Each step ticks
    /// itself as soon as the machine is ready, so it reads as progress, not a manual.
    pub(super) fn keepass_sheet(&self) -> El<'_> {
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
                    label(t(title), size::BODY, if done { p.muted } else { p.text }).font(MEDIUM),
                    label(t(detail), size::MD, p.muted).line_height(1.4),
                ]
                .spacing(2)
                .width(Length::Fill),
            ]
            .spacing(12)
            .into()
        };

        let action: El<'_> = if connected {
            button(center(text(t("Disconnect KeePassXC")).size(size::BODY)))
                .height(38)
                .width(Length::Fill)
                .on_press(Message::KeePassDisconnect)
                .style(style::quiet_button(p, false))
                .into()
        } else {
            button(center(text(t("Connect KeePassXC")).size(size::BODY).font(MEDIUM)))
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
            .width(self.sheet_width(460.0) - 48.0)
            .push(
                row![
                    container(icon(Icon::Key, 22, p.accent)).padding(10).style(style::chip(p, p.accent)),
                    column![
                        label(t("Unlock with KeePassXC"), size::XL, p.text).font(SEMIBOLD),
                        label(t("No password to type. gotodo opens whenever your KeePassXC database is unlocked."), size::MD, p.muted)
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
            .push(label(hint, size::MD, if status == keepass::Status::OtherProvider { p.red } else { p.muted }))
            .push(action);
        if let Some(note) = &self.security_note {
            body = body.push(label(note.as_str(), size::MD, p.muted));
        }
        body = body.push(
            label(
                t("Lock KeePassXC and gotodo locks with it. Your password keeps working either way."),
                size::SM,
                p.faint,
            )
            .line_height(1.45),
        );
        container(body).padding(24).style(style::card(p)).into()
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

fn section_label<'a>(name: &'a str, p: Palette) -> El<'a> {
    container(label(name, size::SM, p.muted).font(SEMIBOLD)).padding(Padding { top: 12.0, left: 4.0, ..Padding::ZERO }).into()
}

/// Options side by side on a track; the chosen one sits raised.
fn segmented<'a, S: Into<String>>(options: Vec<(S, bool, Message)>, p: Palette) -> El<'a> {
    let mut line = Row::new().spacing(2);
    for (words, active, message) in options {
        line = line.push(
            button(label(words.into(), size::SM, if active { p.text } else { p.muted }).font(if active { MEDIUM } else { FONT }))
                .padding([4, 10])
                .on_press(message)
                .style(style::segment(p, active)),
        );
    }
    container(line).padding(2).style(style::segment_track(p)).into()
}

fn setting_row<'a>(title: &'static str, sub: Option<&'a str>, control: El<'a>, p: Palette) -> El<'a> {
    setting_row_owned(t(title), sub.map(str::to_owned), control, p)
}

/// One line of a settings group: what it is on the left, the control on the right.
fn setting_row_owned<'a>(title: impl Into<String>, sub: Option<String>, control: El<'a>, p: Palette) -> El<'a> {
    let mut words = Column::new().spacing(2).width(Length::Fill).push(label(title.into(), size::MD, p.text));
    if let Some(sub) = sub {
        words = words.push(label(sub, size::SM, p.muted).line_height(1.35));
    }
    row![words, control].spacing(16).align_y(Alignment::Center).padding([10, 14]).into()
}

/// Rows in one rounded box, with hairlines between them.
fn settings_group<'a>(rows: Vec<El<'a>>, p: Palette) -> El<'a> {
    let mut column = Column::new();
    let count = rows.len();
    for (index, row) in rows.into_iter().enumerate() {
        column = column.push(row);
        if index + 1 < count {
            column = column.push(container(space()).height(1).width(Length::Fill).style(style::panel(p.line)));
        }
    }
    container(column).width(Length::Fill).style(style::group(p)).into()
}
