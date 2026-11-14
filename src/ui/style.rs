//! Colors, fonts, widget styles and icons.

use std::sync::OnceLock;

use iced::widget::{button, container, svg, text_editor, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector, color, font};

pub const FAMILY: &str = "Inter";

pub const FONT: Font = Font::with_name(FAMILY);
pub const MEDIUM: Font = Font { weight: font::Weight::Medium, ..FONT };
pub const SEMIBOLD: Font = Font { weight: font::Weight::Semibold, ..FONT };

/// The type scale. Every text size in the app is one of these steps.
pub mod size {
    /// Key caps, counts and the smallest captions.
    pub const XS: f32 = 11.0;
    /// Meta lines, chips, helper text.
    pub const SM: f32 = 12.0;
    /// Secondary text: sidebar, settings, section headers.
    pub const MD: f32 = 13.0;
    /// To-dos, list items and inputs.
    pub const BODY: f32 = 14.0;
    /// Quick Entry and the sheet filter field.
    pub const LG: f32 = 16.0;
    /// Sheet titles and day numbers.
    pub const XL: f32 = 20.0;
    /// The lock screen heading.
    pub const XXL: f32 = 24.0;
    /// Page titles.
    pub const DISPLAY: f32 = 28.0;
}

/// Corner radii, from chips up to cards.
pub mod radius {
    /// Key caps.
    pub const XS: f32 = 4.0;
    /// Chips and tags.
    pub const SM: f32 = 6.0;
    /// Buttons, inputs, list rows, tooltips.
    pub const MD: f32 = 8.0;
    /// The open to-do and other raised rows.
    pub const LG: f32 = 12.0;
    /// Sheets, cards and the lock mark.
    pub const XL: f32 = 16.0;
    /// Pills and circles.
    pub const FULL: f32 = 999.0;
}

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

impl Palette {
    /// `tint` as text: bright hues get darker on light pages, dark hues
    /// lighter on dark ones, until they read at 4.5:1 on a raised surface.
    pub fn ink(&self, tint: Color) -> Color {
        let toward = if self.dark { Color::WHITE } else { Color::BLACK };
        let mut k = 0.0;
        let mut ink = tint;
        while contrast(ink, self.raised) < 4.5 && k < 0.7 {
            k += 0.04;
            ink = Color {
                r: tint.r + (toward.r - tint.r) * k,
                g: tint.g + (toward.g - tint.g) * k,
                b: tint.b + (toward.b - tint.b) * k,
                a: tint.a,
            };
        }
        ink
    }
}

fn luminance(c: Color) -> f32 {
    let f = |v: f32| if v <= 0.039_28 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
}

/// WCAG contrast ratio of two opaque colors.
pub fn contrast(a: Color, b: Color) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

pub const LIGHT: Palette = Palette {
    dark: false,
    page: color!(0xffffff),
    sidebar: color!(0xf5f4f1),
    raised: color!(0xffffff),
    text: color!(0x25272b),
    // Text contrast on page and sidebar: muted at least 4.5:1, faint at least 3:1.
    muted: color!(0x676b72),
    faint: color!(0x868a91),
    line: color!(0xe8e8e6),
    hover: Color::from_rgba(0.0, 0.0, 0.0, 0.045),
    selected: color!(0xd7e8fd),
    accent: color!(0x276ad6),
    on_accent: color!(0xffffff),
    gold: color!(0xf2b92c),
    red: color!(0xcf3550),
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
    faint: color!(0x7a7f87),
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
        border: Border { radius: radius::XL.into(), width: 1.0, color: p.line },
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
        border: Border { radius: radius::LG.into(), width: 1.0, color: p.line },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.4 } else { 0.1 }),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 9.0,
        },
        ..container::Style::default()
    }
}

/// The soft tinted tile behind the lock icon.
pub fn mark(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Color { a: if p.dark { 0.16 } else { 0.11 }, ..p.accent }.into()),
        border: Border { radius: radius::XL.into(), ..Border::default() },
        ..container::Style::default()
    }
}

/// The Quick Entry card. `a` fades the whole card, shadow included.
/// No border, radius or shadow of our own: the compositor frames the window.
pub fn quick_card(p: Palette, a: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Color { a: p.raised.a * a, ..p.raised }.into()),
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
        border: Border { radius: radius::MD.into(), width: 1.0, color: p.line },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.45 } else { 0.12 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
}

/// A key on the keyboard, as shown next to a command.
pub fn keycap(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.hover.into()),
        border: Border { radius: radius::XS.into(), width: 1.0, color: p.line },
        ..container::Style::default()
    }
}

pub fn solid_chip(tint: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(tint.into()),
        border: rounded(radius::FULL),
        ..container::Style::default()
    }
}

pub fn row_bg(background: Option<Color>) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: background.map(Background::Color),
        border: rounded(radius::MD),
        ..container::Style::default()
    }
}

pub fn chip(p: Palette, tint: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(p.ink(tint)),
        background: Some(Color { a: if p.dark { 0.2 } else { 0.13 }, ..tint }.into()),
        border: rounded(radius::SM),
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
        border: rounded(radius::MD),
        ..button::Style::default()
    }
}

/// The rounded box that groups related setting rows.
pub fn group(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(if p.dark { Color::from_rgba(1.0, 1.0, 1.0, 0.025) } else { Color::from_rgba(0.0, 0.0, 0.0, 0.018) }.into()),
        border: Border { radius: radius::LG.into(), width: 1.0, color: p.line },
        ..container::Style::default()
    }
}

/// The track behind a segmented control.
pub fn segment_track(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(p.hover.into()),
        border: rounded(radius::MD),
        ..container::Style::default()
    }
}

/// One option of a segmented control; the chosen one sits raised on the track.
pub fn segment(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: match (active, status) {
            (true, _) => Some(p.raised.into()),
            (false, button::Status::Hovered) => Some(p.hover.into()),
            _ => None,
        },
        text_color: if active { p.text } else { p.muted },
        border: Border { radius: (radius::MD - 2.0).into(), width: if active { 1.0 } else { 0.0 }, color: p.line },
        shadow: if active {
            Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, if p.dark { 0.35 } else { 0.08 }), offset: Vector::new(0.0, 1.0), blur_radius: 2.0 }
        } else {
            Shadow::default()
        },
        ..button::Style::default()
    }
}

/// A plain bordered button for secondary actions.
pub fn secondary_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: Some(match status {
            button::Status::Hovered | button::Status::Pressed => p.hover,
            _ => Color::TRANSPARENT,
        }.into()),
        text_color: if matches!(status, button::Status::Disabled) { p.faint } else { p.text },
        border: Border { radius: radius::MD.into(), width: 1.0, color: p.line },
        ..button::Style::default()
    }
}

/// Red text and outline, for the second click of something hard to undo.
pub fn danger_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: Some(Color { a: if matches!(status, button::Status::Hovered) { 0.16 } else { 0.1 }, ..p.red }.into()),
        text_color: p.red,
        border: Border { radius: radius::MD.into(), width: 1.0, color: Color { a: 0.5, ..p.red } },
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
        border: rounded(radius::MD),
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

/// A boxed field that turns red while `error` is set.
pub fn field(p: Palette, error: bool) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: p.raised.into(),
            border: Border {
                radius: radius::MD.into(),
                width: if error { 1.5 } else { 1.0 },
                color: match (error, focused) {
                    (true, _) => p.red,
                    (false, true) => p.accent,
                    (false, false) => p.line,
                },
            },
            icon: p.muted,
            placeholder: p.faint,
            value: p.text,
            selection: p.selected,
        }
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
    Left,
    Right,
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
    Eye,
    EyeOff,
    Caps,
    Alert,
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
            Self::Left => r#"<path d="m15 5-7 7 7 7"/>"#,
            Self::Right => r#"<path d="m9 5 7 7-7 7"/>"#,
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
            Self::Eye => r#"<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="3"/>"#,
            Self::EyeOff => r#"<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="3"/><path d="M4 4l16 16"/>"#,
            Self::Caps => r#"<path d="M12 4 4.5 12h4v4h7v-4h4z" fill="black" fill-opacity=".18" stroke="none"/><path d="M12 4 4.5 12h4v4h7v-4h4zM8.5 20h7"/>"#,
            Self::Alert => r#"<circle cx="12" cy="12" r="8.5" fill="black" fill-opacity=".16" stroke="none"/><circle cx="12" cy="12" r="8.5"/><path d="M12 7.8v4.9M12 16.2v.01"/>"#,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_colors_meet_contrast() {
        for p in [LIGHT, DARK] {
            for surface in [p.page, p.sidebar, p.raised] {
                assert!(contrast(p.text, surface) >= 7.0);
                assert!(contrast(p.muted, surface) >= 4.5, "muted on {surface:?}: {}", contrast(p.muted, surface));
                assert!(contrast(p.faint, surface) >= 3.0, "faint on {surface:?}: {}", contrast(p.faint, surface));
            }
            for tint in [p.gold, p.teal, p.red, p.accent, p.green, p.violet] {
                assert!(contrast(p.ink(tint), p.raised) >= 4.4, "{tint:?}");
            }
        }
    }
}
