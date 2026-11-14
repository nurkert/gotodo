//! The lock and welcome screen, and Quick Entry while locked.

use super::*;
use iced::widget::{column, row, stack};

impl App {
    /// The password field with its show/hide switch inside the right edge.
    pub(super) fn password_field<'a>(&self, placeholder: &'static str, value: &'a str, error: bool, on_input: fn(String) -> Message, id: bool) -> El<'a> {
        let p = self.palette();
        let mut input = text_input(t(placeholder), value)
            .secure(!self.show_password)
            .on_submit(Message::AuthSubmit)
            .padding(Padding { top: 11.0, bottom: 11.0, left: 12.0, right: 40.0 })
            .size(size::BODY)
            .style(style::field(p, error));
        if !self.auth_busy {
            input = input.on_input(on_input);
        }
        if id {
            input = input.id(auth_id());
        }
        let (which, words) = if self.show_password { (Icon::EyeOff, "Hide password") } else { (Icon::Eye, "Show password") };
        let eye = self.with_tip(
            button(icon(which, 16, p.muted))
                .padding(6)
                .on_press(Message::ShowPassword(!self.show_password))
                .style(style::quiet_button(p, false))
                .into(),
            t(words).to_owned(),
        );
        stack![
            input,
            container(eye).width(Length::Fill).height(Length::Fill).align_x(Horizontal::Right).align_y(Vertical::Center).padding([0, 6]),
        ]
        .into()
    }

    /// The main button of an auth card. While busy it keeps its color and shows a spinner.
    pub(super) fn auth_button(&self, words: &'static str, busy_words: &'static str, ready: bool, message: Message, height: f32) -> El<'_> {
        let p = self.palette();
        let content: El<'_> = if self.auth_busy {
            row![spinner(self.spin, p.on_accent, 14.0), text(t(busy_words)).size(size::BODY).font(MEDIUM)]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
        } else {
            text(t(words)).size(size::BODY).font(MEDIUM).into()
        };
        let press = if self.auth_busy { Some(Message::Ignored) } else { ready.then_some(message) };
        button(center(content)).height(height).width(Length::Fill).on_press_maybe(press).style(style::primary_button(p)).into()
    }

    /// One line under a field, always the same height so nothing jumps.
    pub(super) fn hint_line(&self, which: Option<Icon>, words: String, tint: Color) -> El<'_> {
        let mut line = Row::new().spacing(6).align_y(Alignment::Center).height(18);
        if let Some(which) = which {
            line = line.push(icon(which, 13, tint));
        }
        line.push(label(words, size::SM, tint)).into()
    }

    /// Under the unlock field: the error, else a Caps Lock warning, else nothing.
    pub(super) fn unlock_hint(&self) -> El<'_> {
        let p = self.palette();
        if let Some(error) = &self.auth_error {
            return self.hint_line(Some(Icon::Alert), error.clone(), p.red);
        }
        if self.caps_lock {
            return self.hint_line(Some(Icon::Caps), t("Caps Lock is on").to_owned(), p.gold);
        }
        space().height(18).into()
    }

    /// Under the new password: how far from long enough, or Caps Lock.
    pub(super) fn length_hint(&self) -> El<'_> {
        let p = self.palette();
        if self.caps_lock {
            return self.hint_line(Some(Icon::Caps), t("Caps Lock is on").to_owned(), p.gold);
        }
        let count = self.password.chars().count();
        if count >= MIN_PASSWORD_CHARS {
            return self.hint_line(Some(Icon::Check), t("Long enough").to_owned(), p.green);
        }
        let words = if count == 0 {
            format!("{} {MIN_PASSWORD_CHARS} {}", t("At least"), t("characters"))
        } else {
            format!("{} {} {}", MIN_PASSWORD_CHARS - count, t("more"), t("characters"))
        };
        self.hint_line(None, words, p.muted)
    }

    /// The repeat field is wrong once it stops being the start of the password.
    pub(super) fn repeat_mismatch(&self) -> bool {
        !self.repeat.is_empty() && !self.password.starts_with(self.repeat.as_str())
    }

    pub(super) fn repeat_hint(&self) -> El<'_> {
        let p = self.palette();
        if self.repeat_mismatch() {
            return self.hint_line(Some(Icon::Alert), t("Passwords do not match").to_owned(), p.red);
        }
        if !self.repeat.is_empty() && self.repeat == self.password {
            return self.hint_line(Some(Icon::Check), t("Passwords match").to_owned(), p.green);
        }
        if let Some(error) = &self.auth_error {
            return self.hint_line(Some(Icon::Alert), error.clone(), p.red);
        }
        space().height(18).into()
    }

    /// Startup with an automatic way in: a quiet page, and only after a
    /// moment a small sign that something is happening.
    pub(super) fn boot_screen(&self) -> El<'_> {
        let p = self.palette();
        if self.boot_started_elapsed() < 0.6 {
            return space().width(Length::Fill).height(Length::Fill).into();
        }
        center(
            column![spinner(self.spin, p.muted, 18.0), label(t("Unlocking…"), size::SM, p.muted)]
                .spacing(10)
                .align_x(Alignment::Center),
        )
        .into()
    }

    /// The full-window lock and welcome screen: one clear action, the other
    /// ways in tucked into quiet links underneath.
    pub(super) fn lock_screen<'a>(&'a self) -> El<'a> {
        let p = self.palette();
        if self.booting {
            return self.boot_screen();
        }
        let setup = matches!(self.auth, Auth::Setup);
        let device_first = !setup && self.device_allowed() && !self.unlock_password;
        let paused = !setup && self.slots.device && self.prefs.device_paused;

        let centered = |content: iced::widget::Text<'a>| -> El<'a> {
            container(content.align_x(Horizontal::Center)).center_x(Length::Fill).into()
        };
        let link = |words: &'static str, message: Option<Message>| -> El<'static> {
            button(text(t(words)).size(size::SM).color(if message.is_some() { p.muted } else { p.faint }))
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
        let notice = self.lock_notice.as_deref().filter(|_| !setup);
        let subtitle = if let Some(notice) = notice {
            notice
        } else if setup && !self.setup_password {
            t("Your to-dos stay on this computer, encrypted. The key is sealed to this machine, so there is no password to type.")
        } else if setup {
            t("Everything you add stays on this device, encrypted with this password. It cannot be recovered if you lose it.")
        } else if paused {
            t("You locked gotodo, so this device cannot open it on its own. Unlock once with your password or KeePassXC.")
        } else if device_first {
            t("Your to-dos are encrypted on this device.")
        } else {
            t("Enter your password to continue.")
        };

        let mut card = Column::new()
            .spacing(12)
            .width(340)
            .align_x(Alignment::Center)
            .push(container(icon(Icon::Lock, 26, p.accent)).padding(17).style(style::mark(p)))
            .push(space().height(4))
            .push(centered(label(title, size::XXL, p.text).font(SEMIBOLD)))
            .push(centered(label(subtitle, size::MD, p.muted).line_height(1.45)))
            .push(space().height(8));

        if setup && !self.setup_password {
            card = card.push(self.auth_button("Get started", "Setting up…", true, Message::SetupDevice, 42.0));
            if let Some(error) = &self.auth_error {
                card = card.push(self.hint_line(Some(Icon::Alert), error.clone(), p.red));
            }
        } else if device_first {
            card = card.push(self.auth_button("Unlock", "Unlocking…", true, Message::AuthDevice, 42.0));
            card = card.push(self.unlock_hint());
        } else if setup {
            let enough = self.password.chars().count() >= MIN_PASSWORD_CHARS;
            let ready = enough && self.repeat == self.password;
            let fields = column![
                self.password_field("Password", &self.password, false, Message::AuthPassword, true),
                self.length_hint(),
                self.password_field("Repeat password", &self.repeat, self.repeat_mismatch(), Message::AuthRepeat, false),
                self.repeat_hint(),
            ]
            .spacing(6);
            card = card.push(fields).push(self.auth_button("Create vault", "Setting up…", ready, Message::AuthSubmit, 42.0));
        } else {
            let fields = column![
                self.password_field("Password", &self.password, self.auth_error.is_some(), Message::AuthPassword, true),
                self.unlock_hint(),
            ]
            .spacing(6);
            card = card.push(fields).push(self.auth_button("Unlock", "Unlocking…", !self.password.is_empty(), Message::AuthSubmit, 42.0));
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
            if self.device_allowed() && !device_first {
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
            card = card.push(container(others).center_x(Length::Fill));
        }

        let v = self.view_anim.value.clamp(0.0, 1.0);
        let shown = reveal(container(card).center_x(Length::Fill)).fade(1.0 - v, self.page_bg()).slide((1.0 - v) * 10.0);
        let footer = container(
            row![icon(Icon::Lock, 12, p.faint), label(t("Encrypted on this device"), size::SM, p.faint)]
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

    /// Quick Entry while locked: the same lock mark and words as the window,
    /// with the one way in on the right.
    pub(super) fn quick_lock_card(&self) -> El<'_> {
        let p = self.palette();
        if self.booting {
            return row![spinner(self.spin, p.muted, 16.0), label(t("Unlocking…"), size::BODY, p.muted)]
                .spacing(10)
                .align_y(Alignment::Center)
                .into();
        }
        let device_first = self.device_allowed() && (!self.slots.password || !self.unlock_password);
        let mark = container(icon(Icon::Lock, 18, p.accent)).padding(9).style(style::mark(p));
        let (sub, tint) = match &self.auth_error {
            Some(error) => (error.clone(), p.red),
            None if self.caps_lock && !device_first => (t("Caps Lock is on").to_owned(), p.gold),
            None if device_first => (t("Your to-dos are encrypted on this device.").to_owned(), p.muted),
            None => (t("Enter your password to continue.").to_owned(), p.muted),
        };
        let heading = row![
            mark,
            column![label(t("gotodo is locked"), size::BODY, p.text).font(SEMIBOLD), label(sub, size::SM, tint)].spacing(2),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        if device_first {
            return row![
                heading,
                space().width(Length::Fill),
                container(self.auth_button("Unlock", "Unlocking…", true, Message::AuthDevice, 36.0)).width(120),
            ]
            .align_y(Alignment::Center)
            .into();
        }
        column![
            heading,
            row![
                container(self.password_field("Password", &self.password, self.auth_error.is_some(), Message::AuthPassword, true)).width(Length::Fill),
                container(self.auth_button("Unlock", "Unlocking…", !self.password.is_empty(), Message::AuthSubmit, 40.0)).width(110),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12)
        .into()
    }
}
