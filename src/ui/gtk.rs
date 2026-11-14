//! Colors from the user's GTK theme, so gotodo can match the rest of the desktop.
//!
//! Reads the `@define-color` lines of `~/.config/gtk-4.0/gtk.css` and
//! `~/.config/gtk-3.0/gtk.css` (the libadwaita names first, then the classic
//! GTK 3 ones). Without an accent there, the GNOME accent setting is used.
//! Anything missing falls back to gotodo's own palette, and text colors are
//! nudged until they stay readable on the theme's surfaces.

use std::collections::HashMap;

use iced::Color;

use super::style::{DARK, LIGHT, Palette, contrast};

/// The colors a GTK theme defines. Each is optional.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GtkColors {
    pub page: Option<Color>,
    pub text: Option<Color>,
    pub raised: Option<Color>,
    pub sidebar: Option<Color>,
    pub line: Option<Color>,
    pub accent: Option<Color>,
    pub on_accent: Option<Color>,
    pub red: Option<Color>,
    pub gold: Option<Color>,
    pub green: Option<Color>,
}

impl GtkColors {
    /// Reads the user's GTK settings. `None` when nothing usable is defined.
    pub fn load() -> Option<Self> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            let config = std::env::var_os("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config")))?;
            let mut css = String::new();
            // GTK 4 last, so its definitions win.
            for file in ["gtk-3.0/gtk.css", "gtk-4.0/gtk.css"] {
                if let Ok(text) = std::fs::read_to_string(config.join(file)) {
                    css.push_str(&text);
                    css.push('\n');
                }
            }
            let mut colors = Self::from_css(&css);
            if colors.accent.is_none() {
                colors.accent = gnome_accent();
            }
            (colors != Self::default()).then_some(colors)
        }
    }

    pub fn from_css(css: &str) -> Self {
        let defs = definitions(css);
        let pick = |names: &[&str]| names.iter().find_map(|name| resolve(&format!("@{name}"), &defs, 0));
        Self {
            page: pick(&["window_bg_color", "theme_bg_color", "bg_color"]),
            text: pick(&["window_fg_color", "theme_fg_color", "fg_color"]),
            raised: pick(&["view_bg_color", "card_bg_color", "theme_base_color", "base_color"]),
            sidebar: pick(&["sidebar_bg_color", "headerbar_bg_color"]),
            line: pick(&["borders", "border_color", "unfocused_borders"]),
            accent: pick(&["accent_bg_color", "accent_color", "theme_selected_bg_color", "selected_bg_color"]),
            on_accent: pick(&["accent_fg_color", "theme_selected_fg_color", "selected_fg_color"]),
            red: pick(&["error_color", "destructive_color", "error_bg_color"]),
            gold: pick(&["warning_color", "warning_bg_color"]),
            green: pick(&["success_color", "success_bg_color"]),
        }
    }

    /// A full palette: the theme's colors where it has them, gotodo's elsewhere.
    /// `fallback_dark` decides light or dark when the theme names no background.
    pub fn palette(&self, fallback_dark: bool) -> Palette {
        let opaque = |c: Color| Color { a: 1.0, ..c };
        let dark = self.page.map_or(fallback_dark, |page| luminance(page) < 0.4);
        let base = if dark { DARK } else { LIGHT };
        let page = self.page.map_or(base.page, opaque);
        let text = self.text.map_or(base.text, opaque);
        let raised = self.raised.map(opaque).unwrap_or(if self.page.is_some() {
            if dark { mix(page, Color::WHITE, 0.05) } else { Color::WHITE }
        } else {
            base.raised
        });
        let sidebar = self.sidebar.map(opaque).unwrap_or(if self.page.is_some() { mix(page, raised, 0.5) } else { base.sidebar });
        let accent = self.accent.map_or(base.accent, opaque);
        let surfaces = [page, sidebar, raised];
        let on_accent = self.on_accent.map(opaque).unwrap_or_else(|| {
            if contrast(Color::WHITE, accent) >= contrast(Color::BLACK, accent) { Color::WHITE } else { Color::BLACK }
        });
        Palette {
            dark,
            page,
            sidebar,
            raised,
            text,
            muted: readable(mix(text, page, 0.4), text, &surfaces, 4.5),
            faint: readable(mix(text, page, 0.55), text, &surfaces, 3.0),
            line: self.line.map(opaque).unwrap_or_else(|| mix(page, text, if dark { 0.12 } else { 0.1 })),
            hover: Color { a: if dark { 0.06 } else { 0.045 }, ..text },
            selected: mix(page, accent, if dark { 0.32 } else { 0.18 }),
            accent,
            on_accent,
            red: self.red.map_or(base.red, opaque),
            gold: self.gold.map_or(base.gold, opaque),
            green: self.green.map_or(base.green, opaque),
            ..base
        }
    }
}

/// Moves `color` toward `text` until it reads at `ratio` on every surface.
fn readable(color: Color, text: Color, surfaces: &[Color], ratio: f32) -> Color {
    let mut k = 0.0;
    let mut out = color;
    while surfaces.iter().any(|&s| contrast(out, s) < ratio) && k < 1.0 {
        k += 0.05;
        out = mix(color, text, k);
    }
    out
}

fn mix(a: Color, b: Color, k: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * k, g: a.g + (b.g - a.g) * k, b: a.b + (b.b - a.b) * k, a: a.a + (b.a - a.a) * k }
}

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// The `@define-color` lines, comments removed.
fn definitions(css: &str) -> HashMap<String, String> {
    let mut clean = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        clean.push_str(&rest[..start]);
        rest = rest[start + 2..].find("*/").map_or("", |end| &rest[start + 2 + end + 2..]);
    }
    clean.push_str(rest);
    clean
        .split(';')
        .filter_map(|statement| {
            let statement = statement.trim();
            let at = statement.rfind("@define-color")?;
            let mut parts = statement[at + "@define-color".len()..].trim().splitn(2, char::is_whitespace);
            Some((parts.next()?.to_owned(), parts.next()?.trim().to_owned()))
        })
        .collect()
}

/// Evaluates a GTK color expression: names, hex, rgb()/rgba(), and the
/// alpha(), shade(), mix(), lighter() and darker() functions.
fn resolve(expr: &str, defs: &HashMap<String, String>, depth: u8) -> Option<Color> {
    if depth > 16 {
        return None;
    }
    let expr = expr.trim();
    if let Some(name) = expr.strip_prefix('@') {
        return resolve(defs.get(name)?, defs, depth + 1);
    }
    if let Some(hex) = expr.strip_prefix('#') {
        return parse_hex(hex);
    }
    match expr {
        "white" => return Some(Color::WHITE),
        "black" => return Some(Color::BLACK),
        "transparent" => return Some(Color::TRANSPARENT),
        _ => {}
    }
    let open = expr.find('(')?;
    let inner = expr[open + 1..].strip_suffix(')')?;
    let args = split_args(inner);
    let color = |i: usize| args.get(i).and_then(|a| resolve(a, defs, depth + 1));
    let number = |i: usize| args.get(i).and_then(|a| a.trim().parse::<f32>().ok());
    match expr[..open].trim() {
        "rgb" | "rgba" => {
            let channel = |i: usize| {
                let a = args.get(i)?.trim();
                match a.strip_suffix('%') {
                    Some(p) => p.trim().parse::<f32>().ok().map(|v| v / 100.0),
                    None => a.parse::<f32>().ok().map(|v| v / 255.0),
                }
            };
            Some(Color::from_rgba(channel(0)?, channel(1)?, channel(2)?, number(3).unwrap_or(1.0)))
        }
        "alpha" => {
            let c = color(0)?;
            Some(Color { a: (c.a * number(1)?).clamp(0.0, 1.0), ..c })
        }
        "shade" => Some(shade(color(0)?, number(1)?)),
        "lighter" => Some(shade(color(0)?, 1.3)),
        "darker" => Some(shade(color(0)?, 0.7)),
        "mix" => Some(mix(color(0)?, color(1)?, number(2)?)),
        _ => None,
    }
}

/// Close to GTK's shade(): scales lightness, keeping the hue.
fn shade(c: Color, factor: f32) -> Color {
    if factor >= 1.0 {
        mix(c, Color { a: c.a, ..Color::WHITE }, ((factor - 1.0) / factor).min(1.0))
    } else {
        Color { r: c.r * factor, g: c.g * factor, b: c.b * factor, a: c.a }
    }
}

fn split_args(inner: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (i, ch) in inner.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                args.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    args.push(&inner[start..]);
    args
}

fn parse_hex(hex: &str) -> Option<Color> {
    let digits: Vec<u8> = hex.chars().map(|c| c.to_digit(16).map(|d| d as u8)).collect::<Option<_>>()?;
    let byte = |hi: u8, lo: u8| f32::from(hi * 16 + lo) / 255.0;
    match digits.as_slice() {
        [r, g, b] => Some(Color::from_rgb(byte(*r, *r), byte(*g, *g), byte(*b, *b))),
        [r1, r2, g1, g2, b1, b2] => Some(Color::from_rgb(byte(*r1, *r2), byte(*g1, *g2), byte(*b1, *b2))),
        [r1, r2, g1, g2, b1, b2, a1, a2] => Some(Color::from_rgba(byte(*r1, *r2), byte(*g1, *g2), byte(*b1, *b2), byte(*a1, *a2))),
        _ => None,
    }
}

/// GNOME's accent color setting, as libadwaita draws it.
#[cfg(not(test))]
fn gnome_accent() -> Option<Color> {
    let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.interface", "accent-color"]).output().ok()?;
    let name = String::from_utf8(out.stdout).ok()?;
    let hex = match name.trim().trim_matches('\'') {
        "blue" => "3584e4",
        "teal" => "2190a4",
        "green" => "3a944a",
        "yellow" => "c88800",
        "orange" => "ed5b00",
        "red" => "e62d42",
        "pink" => "d56199",
        "purple" => "9141ac",
        "slate" => "6f8396",
        _ => return None,
    };
    parse_hex(hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSS: &str = "
        /* a comment with @define-color fake #000000; */
        @define-color accent_color #b162d5;
        @define-color accent_bg_color @accent_color;
        @define-color wofi_bg #15151a;
        @define-color wofi_surface #1c1c24;
        @define-color window_bg_color @wofi_bg;
        @define-color window_fg_color #e8e8ed;
        @define-color base_color @wofi_surface;
        @define-color borders #34343d;
        @define-color insensitive_fg_color alpha(@window_fg_color, 0.45);
        @define-color error_color rgb(213, 86, 114);
        window { background-color: @wofi_bg; }
    ";

    #[test]
    fn reads_references_and_functions() {
        let colors = GtkColors::from_css(CSS);
        assert_eq!(colors.accent, parse_hex("b162d5"));
        assert_eq!(colors.page, parse_hex("15151a"));
        assert_eq!(colors.raised, parse_hex("1c1c24"));
        assert_eq!(colors.line, parse_hex("34343d"));
        let red = colors.red.unwrap();
        assert!((red.r - 213.0 / 255.0).abs() < 1e-4);
        let defs = definitions(CSS);
        assert!((resolve("@insensitive_fg_color", &defs, 0).unwrap().a - 0.45).abs() < 1e-4);
        assert_eq!(resolve("mix(#000000, #ffffff, 0.5)", &defs, 0).map(|c| (c.r * 100.0).round()), Some(50.0));
        assert!(defs.get("fake").is_none());
    }

    #[test]
    fn theme_palette_stays_readable() {
        for css in [CSS, "@define-color window_bg_color #fafafa; @define-color window_fg_color #999999;", ""] {
            let p = GtkColors::from_css(css).palette(true);
            for surface in [p.page, p.sidebar, p.raised] {
                assert!(contrast(p.muted, surface) >= 4.5 || contrast(p.text, surface) < 4.5);
                assert!(contrast(p.faint, surface) >= 3.0 || contrast(p.text, surface) < 3.0);
            }
        }
        let p = GtkColors::from_css(CSS).palette(false);
        assert!(p.dark, "a dark GTK background makes a dark palette");
        assert_eq!(p.accent, parse_hex("b162d5").unwrap());
    }
}
