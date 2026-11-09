//! Colors, fonts, widget styles and icons.

use std::sync::OnceLock;

use iced::widget::{button, container, svg, text_editor, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector, color, font};

pub const FAMILY: &str = "Inter";

pub const FONT: Font = Font::with_name(FAMILY);
pub const MEDIUM: Font = Font { weight: font::Weight::Medium, ..FONT };
pub const SEMIBOLD: Font = Font { weight: font::Weight::Semibold, ..FONT };

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub page: Color,
    pub sidebar: Color,
    pub raised: Color,
    pub text: Color,
    pub muted: Color,
    pub faint: Color,
    pub line: Color,
    pub hover: Color,
    pub selected: Color,
    pub accent: Color,
    pub on_accent: Color,
    pub gold: Color,
    pub red: Color,
    pub teal: Color,
    pub violet: Color,
    pub green: Color,
    pub sand: Color,
    pub backdrop: Color,
}

pub const LIGHT: Palette = Palette {
    dark: false,
    page: color!(0xffffff),
    sidebar: color!(0xf5f4f1),
    raised: color!(0xffffff),
    text: color!(0x25272b),
    muted: color!(0x74787f),
    faint: color!(0xb4b7bc),
    line: color!(0xe8e8e6),
    hover: Color::from_rgba(0.0, 0.0, 0.0, 0.045),
    selected: color!(0xd7e8fd),
    accent: color!(0x2f7bf0),
    on_accent: color!(0xffffff),
    gold: color!(0xf2b92c),
    red: color!(0xe2445c),
    teal: color!(0x2fa394),
    violet: color!(0x7b5cdc),
    green: color!(0x4bae6e),
    sand: color!(0xb99a5b),
    backdrop: Color::from_rgba(0.1, 0.11, 0.14, 0.22),
};

pub const DARK: Palette = Palette {
    dark: true,
    page: color!(0x1b1d21),
    sidebar: color!(0x23252a),
    raised: color!(0x2a2d33),
    text: color!(0xe9eaec),
    muted: color!(0x9a9ea6),
    faint: color!(0x62666e),
    line: color!(0x33363c),
    hover: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
    selected: color!(0x23406a),
    accent: color!(0x4c91f7),
    on_accent: color!(0xffffff),
    gold: color!(0xf4c54a),
    red: color!(0xf0607a),
    teal: color!(0x4fc0b4),
    violet: color!(0xa08cf7),
    green: color!(0x62c285),
    sand: color!(0xc9ab70),
    backdrop: Color::from_rgba(0.0, 0.0, 0.0, 0.42),
};

fn rounded(radius: f32) -> Border {
    Border { radius: radius.into(), ..Border::default() }
}

pub fn panel(background: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style { background: Some(background.into()), ..container::Style::default() }
}

pub fn card(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.raised.into()),
        border: Border { radius: 14.0.into(), width: 1.0, color: p.line },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.5 } else { 0.14 }),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 30.0,
        },
        ..container::Style::default()
    }
}

/// The open to-do. Its shadow stays inside the row's margin, which clips it.
pub fn open_row(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.raised.into()),
        border: Border { radius: 12.0.into(), width: 1.0, color: p.line },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.4 } else { 0.1 }),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 9.0,
        },
        ..container::Style::default()
    }
}

/// The Quick Entry card. `a` fades the whole card, shadow included.
pub fn quick_card(p: Palette, a: f32, see_through: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Color { a: p.raised.a * a, ..p.raised }.into()),
        border: Border {
            radius: if see_through { 16.0 } else { 0.0 }.into(),
            width: 1.0,
            color: Color { a: p.line.a * a, ..p.line },
        },
        shadow: if see_through {
            Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, (if p.dark { 0.5 } else { 0.2 }) * a),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 22.0,
            }
        } else {
            Shadow::default()
        },
        ..container::Style::default()
    }
}

/// Like `bare_input`, with every color scaled by `a` so text can fade in.
pub fn fading_input(p: Palette, a: f32) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    let fade = move |c: Color| Color { a: c.a * a, ..c };
    move |_, _| text_input::Style {
        background: Color::TRANSPARENT.into(),
        border: Border::default(),
        icon: fade(p.muted),
        placeholder: fade(p.faint),
        value: fade(p.text),
        selection: fade(p.selected),
    }
}

/// Small hover label for icon-only buttons.
pub fn tip(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.raised.into()),
        border: Border { radius: 8.0.into(), width: 1.0, color: p.line },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.45 } else { 0.12 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
}

pub fn solid_chip(tint: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(tint.into()),
        border: rounded(8.0),
        ..container::Style::default()
    }
}

pub fn row_bg(background: Option<Color>) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: background.map(Background::Color),
        border: rounded(9.0),
        ..container::Style::default()
    }
}

pub fn chip(p: Palette, tint: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(tint),
        background: Some(Color { a: if p.dark { 0.2 } else { 0.13 }, ..tint }.into()),
        border: rounded(6.0),
        ..container::Style::default()
    }
}

pub fn drop_line(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.accent.into()),
        border: rounded(1.0),
        ..container::Style::default()
    }
}

pub fn quiet_button(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: match (active, status) {
            (true, _) => Some(p.selected.into()),
            (false, button::Status::Hovered | button::Status::Pressed) => Some(p.hover.into()),
            _ => None,
        },
        text_color: p.text,
        border: rounded(8.0),
        ..button::Style::default()
    }
}

pub fn primary_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: Some(
            match status {
                button::Status::Hovered => Color { a: 0.9, ..p.accent },
                button::Status::Disabled => Color { a: 0.45, ..p.accent },
                _ => p.accent,
            }
            .into(),
        ),
        text_color: p.on_accent,
        border: rounded(9.0),
        ..button::Style::default()
    }
}

/// A field that reads as plain text until it has focus.
pub fn bare_input(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, _| text_input::Style {
        background: Color::TRANSPARENT.into(),
        border: Border::default(),
        icon: p.muted,
        placeholder: p.faint,
        value: p.text,
        selection: p.selected,
    }
}

pub fn boxed_input(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| text_input::Style {
        background: p.raised.into(),
        border: Border {
            radius: 9.0.into(),
            width: 1.0,
            color: if matches!(status, text_input::Status::Focused { .. }) { p.accent } else { p.line },
        },
        icon: p.muted,
        placeholder: p.faint,
        value: p.text,
        selection: p.selected,
    }
}

pub fn bare_editor(p: Palette) -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
    move |_, _| text_editor::Style {
        background: Color::TRANSPARENT.into(),
        border: Border::default(),
        placeholder: p.faint,
        value: p.text,
        selection: p.selected,
    }
}

/// Base theme, so stock widgets such as sliders and togglers share the accent.
pub fn base_theme(p: &Palette) -> Theme {
    static THEMES: OnceLock<(Theme, Theme)> = OnceLock::new();
    let make = |name: &'static str, p: &Palette| {
        Theme::custom(
            name,
            iced::theme::Palette {
                background: p.page,
                text: p.text,
                primary: p.accent,
                success: p.green,
                warning: p.gold,
                danger: p.red,
            },
        )
    };
    let (light, dark) = THEMES.get_or_init(|| (make("gotodo light", &LIGHT), make("gotodo dark", &DARK)));
    if p.dark { dark.clone() } else { light.clone() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Inbox,
    Star,
    Calendar,
    Layers,
    Archive,
    Book,
    Trash,
    Plus,
    Tag,
    Flag,
    Moon,
    Sun,
    Repeat,
    Search,
    Arrow,
    Gear,
    Lock,
    Key,
    Notes,
    Box,
    Bell,
    Command,
    Check,
}

impl Icon {
    /// Outline plus a soft fill layer. The tint recolors both, so the fill
    /// layer (alpha .2) reads as a lighter shade of the same hue.
    fn body(self) -> &'static str {
        match self {
            Self::Inbox => r#"<path d="M3 13.5 5.5 5h13l2.5 8.5V19H3z" fill="black" fill-opacity=".18" stroke="none"/><path d="M3 13.5 5.5 5h13l2.5 8.5V19H3z"/><path d="M3 13.5h5.5l1 2.5h5l1-2.5H21"/>"#,
            Self::Star => r#"<path d="m12 3.2 2.7 5.7 6.3.8-4.6 4.3 1.2 6.2-5.6-3.1-5.6 3.1 1.2-6.2L3 9.7l6.3-.8z" fill="black"/>"#,
            Self::Calendar => r#"<rect x="3.5" y="5" width="17" height="15.5" rx="3" fill="black" fill-opacity=".18" stroke="none"/><rect x="3.5" y="5" width="17" height="15.5" rx="3"/><path d="M3.5 10h17M8 3v4M16 3v4"/>"#,
            Self::Layers => r#"<path d="m12 3 9 5-9 5-9-5z" fill="black" fill-opacity=".22" stroke="none"/><path d="m12 3 9 5-9 5-9-5z"/><path d="m3 12.5 9 5 9-5"/><path d="m3 16.5 9 5 9-5" opacity=".5"/>"#,
            Self::Archive => r#"<rect x="3" y="4" width="18" height="5" rx="1.6" fill="black" fill-opacity=".22" stroke="none"/><rect x="3" y="4" width="18" height="5" rx="1.6"/><path d="M5 9v10.5h14V9M10 13h4"/>"#,
            Self::Book => r#"<path d="M5 4.5A1.5 1.5 0 0 1 6.5 3H19v18H6.5A1.5 1.5 0 0 1 5 19.5z" fill="black" fill-opacity=".18" stroke="none"/><path d="M5 4.5A1.5 1.5 0 0 1 6.5 3H19v18H6.5A1.5 1.5 0 0 1 5 19.5z"/><path d="m9 11 2.2 2.2L15.5 9"/>"#,
            Self::Trash => r#"<path d="M6 6.5l1 14h10l1-14z" fill="black" fill-opacity=".15" stroke="none"/><path d="M4 6.5h16M9 6.5V4h6v2.5M6 6.5l1 14h10l1-14"/>"#,
            Self::Plus => r#"<path d="M12 5v14M5 12h14"/>"#,
            Self::Tag => r#"<path d="M3.5 4h8l9 9-8 8-9-9z" fill="black" fill-opacity=".18" stroke="none"/><path d="M3.5 4h8l9 9-8 8-9-9z"/><circle cx="8" cy="8.5" r="1.2" fill="black"/>"#,
            Self::Flag => r#"<path d="M5.5 4.5h12.5l-2.8 4.2 2.8 4.3H5.5z" fill="black" fill-opacity=".2" stroke="none"/><path d="M5.5 21V4M5.5 4.5h12.5l-2.8 4.2 2.8 4.3H5.5"/>"#,
            Self::Moon => r#"<path d="M20 14.5A8.5 8.5 0 1 1 9.5 4a7 7 0 0 0 10.5 10.5z" fill="black"/>"#,
            Self::Sun => r#"<circle cx="12" cy="12" r="4.2" fill="black"/><path d="M12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M18.5 5.5l-1.7 1.7M7.2 16.8l-1.7 1.7"/>"#,
            Self::Repeat => r#"<path d="M4 11V9.5A3.5 3.5 0 0 1 7.5 6H20m0 0-3-3m3 3-3 3M20 13v1.5a3.5 3.5 0 0 1-3.5 3.5H4m0 0 3-3m-3 3 3 3"/>"#,
            Self::Search => r#"<circle cx="10.5" cy="10.5" r="6.5" fill="black" fill-opacity=".12" stroke="none"/><circle cx="10.5" cy="10.5" r="6.5"/><path d="m15.5 15.5 5 5"/>"#,
            Self::Arrow => r#"<path d="M5 12h14m-5.5-5.5L19 12l-5.5 5.5"/>"#,
            Self::Gear => r#"<circle cx="12" cy="12" r="4" fill="black" fill-opacity=".18" stroke="none"/><circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M18.7 5.3l-2.1 2.1M7.4 16.6l-2.1 2.1"/>"#,
            Self::Lock => r#"<rect x="4.5" y="10.5" width="15" height="10.5" rx="2.6" fill="black" fill-opacity=".2" stroke="none"/><rect x="4.5" y="10.5" width="15" height="10.5" rx="2.6"/><path d="M8 10.5V7a4 4 0 0 1 8 0v3.5"/>"#,
            Self::Key => r#"<circle cx="8" cy="15" r="4.5" fill="black" fill-opacity=".2" stroke="none"/><circle cx="8" cy="15" r="4.5"/><path d="m11.2 11.8 8.3-8.3M16 7l2.5 2.5M13.5 9.5 15.5 11.5"/>"#,
            Self::Notes => r#"<path d="M5 6h14M5 11h14M5 16h9"/>"#,
            Self::Box => r#"<circle cx="12" cy="12" r="8.5" fill="black" fill-opacity=".14" stroke="none"/><circle cx="12" cy="12" r="8.5"/>"#,
            Self::Bell => r#"<path d="M6 16.5V11a6 6 0 0 1 12 0v5.5l1.5 2h-15z" fill="black" fill-opacity=".18" stroke="none"/><path d="M6 16.5V11a6 6 0 0 1 12 0v5.5l1.5 2h-15zM10 21h4"/>"#,
            Self::Command => r#"<path d="m5 7 5 5-5 5M12.5 18H19"/>"#,
            Self::Check => r#"<path d="m5 12.5 4.5 4.5L19 7.5"/>"#,
        }
    }

    pub fn handle(self) -> svg::Handle {
        let source = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
            self.body()
        );
        svg::Handle::from_memory(source.into_bytes())
    }
}

pub fn icon<'a, Message: 'a>(which: Icon, size: impl Into<iced::Pixels>, tint: Color) -> iced::Element<'a, Message> {
    let side = size.into().0;
    svg(which.handle())
        .width(side)
        .height(side)
        .style(move |_, _| svg::Style { color: Some(tint) })
        .into()
}
