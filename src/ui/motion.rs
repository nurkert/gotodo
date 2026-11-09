//! Springs and the small custom widgets that give the app its motion.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay, renderer};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, stroke};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Radians, Rectangle, Renderer, Shadow,
    Size, Theme, Vector, mouse,
};

const STIFFNESS: f32 = 320.0;
const DAMPING: f32 = 30.0;
const STEP: f32 = 1.0 / 240.0;

/// A critically-leaning spring: settles in about a third of a second with
/// only a trace of overshoot, and picks up velocity when retargeted mid-flight.
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub value: f32,
    velocity: f32,
    pub target: f32,
    stiffness: f32,
    damping: f32,
}

impl Spring {
    pub fn at(value: f32) -> Self {
        Self { value, velocity: 0.0, target: value, stiffness: STIFFNESS, damping: DAMPING }
    }

    /// A snappier spring for small surfaces such as the Quick Entry popup.
    pub fn quick(value: f32) -> Self {
        Self { stiffness: 1100.0, damping: 62.0, ..Self::at(value) }
    }

    pub fn to(&mut self, target: f32) {
        self.target = target;
    }

    pub fn snap(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
    }

    pub fn settled(&self) -> bool {
        (self.value - self.target).abs() < 0.001 && self.velocity.abs() < 0.01
    }

    pub fn step(&mut self, dt: f32, reduce_motion: bool) {
        if reduce_motion {
            self.snap();
            return;
        }
        let mut left = dt.min(0.1);
        while left > 0.0 {
            let h = left.min(STEP);
            let force = self.stiffness * (self.target - self.value) - self.damping * self.velocity;
            self.velocity += force * h;
            self.value += self.velocity * h;
            left -= h;
        }
        if self.settled() {
            self.snap();
        }
    }
}

/// Shows its content at a fraction of its natural height, clipped, with an
/// optional fade (painted as a veil in `veil` color) and vertical slide.
pub struct Reveal<'a, Message> {
    content: Element<'a, Message>,
    progress: f32,
    fade: f32,
    slide: f32,
    veil: Color,
}

pub fn reveal<'a, Message>(content: impl Into<Element<'a, Message>>) -> Reveal<'a, Message> {
    Reveal { content: content.into(), progress: 1.0, fade: 0.0, slide: 0.0, veil: Color::TRANSPARENT }
}

impl<Message> Reveal<'_, Message> {
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.max(0.0);
        self
    }

    pub fn fade(mut self, fade: f32, veil: Color) -> Self {
        self.fade = fade.clamp(0.0, 1.0);
        self.veil = veil;
        self
    }

    pub fn slide(mut self, slide: f32) -> Self {
        self.slide = slide;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Reveal<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let inner = layout::Limits::new(Size::ZERO, Size::new(limits.max().width, f32::INFINITY));
        let child = self.content.as_widget_mut().layout(tree, renderer, &inner);
        let size = Size::new(limits.max().width, child.size().height * self.progress);
        layout::Node::with_children(size, vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            tree,
            layout.children().next().expect("reveal has one child"),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.progress < 0.02 {
            return;
        }
        let cursor = if cursor.is_over(layout.bounds()) { cursor } else { mouse::Cursor::Unavailable };
        self.content.as_widget_mut().update(
            tree,
            event,
            layout.children().next().expect("reveal has one child"),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::default();
        }
        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().expect("reveal has one child"),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.height < 0.5 {
            return;
        }
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(Vector::new(0.0, self.slide), |renderer| {
                self.content.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    style,
                    layout.children().next().expect("reveal has one child"),
                    cursor,
                    viewport,
                );
            });
        });
        if self.fade > 0.005 {
            let veil = Color { a: self.veil.a * self.fade, ..self.veil };
            renderer.with_layer(bounds, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(veil),
                );
            });
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().expect("reveal has one child"),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Reveal<'a, Message>> for Element<'a, Message> {
    fn from(reveal: Reveal<'a, Message>) -> Self {
        Element::new(reveal)
    }
}

/// The to-do check box. `progress` runs 0..1 as the tick draws itself in.
#[derive(Debug, Clone, Copy)]
pub struct CheckMark {
    pub progress: f32,
    pub canceled: bool,
    pub round: bool,
    pub outline: Color,
    pub fill: Color,
    pub tick: Color,
    /// Spare room around the box, so the completion ring can spread outside it.
    pub pad: f32,
    /// True while the box is being checked; the ring only plays in that direction.
    pub rising: bool,
}

impl<Message> canvas::Program<Message> for CheckMark {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let side = bounds.width.min(bounds.height) - 2.0 * self.pad;
        let t = self.progress.clamp(0.0, 1.0);
        // A small press-in that releases as the fill arrives.
        let pop = 1.0 - 0.14 * (t * std::f32::consts::PI).sin();
        let size = (side - 2.0) * pop;
        let origin = Point::new((bounds.width - size) / 2.0, (bounds.height - size) / 2.0);
        let radius = if self.round { size / 2.0 } else { 4.0 };
        let shape = Path::rounded_rectangle(origin, Size::new(size, size), radius.into());
        if self.rising && self.pad > 0.0 && t > 0.05 && t < 0.98 {
            // One soft ring leaves the box as the fill arrives.
            let k = (t - 0.05) / 0.93;
            let reach = size / 2.0 + 1.0 + self.pad * k;
            let fade = (1.0 - k) * (1.0 - k);
            let middle = frame.center();
            frame.stroke(
                &Path::circle(middle, reach),
                Stroke {
                    style: stroke::Style::Solid(Color { a: self.fill.a * 0.55 * fade, ..self.fill }),
                    width: 1.6 * (1.0 - 0.6 * k),
                    ..Stroke::default()
                },
            );
        }
        if t > 0.0 {
            frame.fill(&shape, Color { a: self.fill.a * t, ..self.fill });
        }
        frame.stroke(
            &shape,
            Stroke {
                style: stroke::Style::Solid(if t > 0.5 { self.fill } else { self.outline }),
                width: 1.25,
                ..Stroke::default()
            },
        );
        let drawn = ((t - 0.25) / 0.75).clamp(0.0, 1.0);
        if drawn > 0.0 {
            let at = |x: f32, y: f32| Point::new(origin.x + size * x, origin.y + size * y);
            let (a, b, c) = if self.canceled {
                (at(0.3, 0.3), at(0.5, 0.5), at(0.7, 0.7))
            } else {
                (at(0.26, 0.52), at(0.44, 0.7), at(0.76, 0.32))
            };
            let lerp = |p: Point, q: Point, k: f32| Point::new(p.x + (q.x - p.x) * k, p.y + (q.y - p.y) * k);
            let path = Path::new(|builder| {
                builder.move_to(a);
                if drawn < 0.45 {
                    builder.line_to(lerp(a, b, drawn / 0.45));
                } else {
                    builder.line_to(b);
                    builder.line_to(lerp(b, c, (drawn - 0.45) / 0.55));
                }
            });
            frame.stroke(
                &path,
                Stroke {
                    style: stroke::Style::Solid(self.tick),
                    width: 1.8,
                    line_cap: stroke::LineCap::Round,
                    line_join: stroke::LineJoin::Round,
                    ..Stroke::default()
                },
            );
            if self.canceled {
                let across = Path::line(at(0.7, 0.3), at(0.3, 0.7));
                frame.stroke(
                    &across,
                    Stroke {
                        style: stroke::Style::Solid(Color { a: drawn, ..self.tick }),
                        width: 1.8,
                        line_cap: stroke::LineCap::Round,
                        ..Stroke::default()
                    },
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

pub fn check_mark<'a, Message: 'a>(mark: CheckMark, side: f32) -> Element<'a, Message> {
    let full = side + 2.0 * mark.pad;
    Canvas::new(mark).width(full).height(full).into()
}

/// A project's progress ring and pie.
#[derive(Debug, Clone, Copy)]
pub struct Pie {
    pub share: f32,
    pub color: Color,
}

impl<Message> canvas::Program<Message> for Pie {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = bounds.width.min(bounds.height) / 2.0 - 1.0;
        frame.stroke(
            &Path::circle(center, radius),
            Stroke { style: stroke::Style::Solid(self.color), width: 1.4, ..Stroke::default() },
        );
        if self.share > 0.0 {
            let start = -std::f32::consts::FRAC_PI_2;
            let pie = Path::new(|builder| {
                builder.move_to(center);
                builder.arc(canvas::path::Arc {
                    center,
                    radius: radius - 2.2,
                    start_angle: Radians(start),
                    end_angle: Radians(start + std::f32::consts::TAU * self.share.min(1.0)),
                });
                builder.close();
            });
            frame.fill(&pie, self.color);
        }
        vec![frame.into_geometry()]
    }
}

pub fn pie<'a, Message: 'a>(share: f32, color: Color, side: f32) -> Element<'a, Message> {
    Canvas::new(Pie { share, color }).width(side).height(side).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_settles_quickly_on_target() {
        let mut spring = Spring::at(0.0);
        spring.to(1.0);
        let mut elapsed = 0.0;
        while !spring.settled() && elapsed < 2.0 {
            spring.step(1.0 / 60.0, false);
            elapsed += 1.0 / 60.0;
        }
        assert!(elapsed < 0.8, "took {elapsed}s");
        assert_eq!(spring.value, 1.0);
    }

    #[test]
    fn overshoot_stays_subtle() {
        let mut spring = Spring::at(0.0);
        spring.to(1.0);
        let mut peak: f32 = 0.0;
        for _ in 0..120 {
            spring.step(1.0 / 60.0, false);
            peak = peak.max(spring.value);
        }
        assert!(peak < 1.06, "peak {peak}");
    }

    #[test]
    fn reduced_motion_snaps() {
        let mut spring = Spring::at(0.0);
        spring.to(1.0);
        spring.step(1.0 / 60.0, true);
        assert_eq!(spring.value, 1.0);
    }
}
