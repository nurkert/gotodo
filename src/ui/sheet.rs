//! The keyboard-driven sheets. Every picker is the same thing: a filter field
//! over a list of actions.

use chrono::{Datelike, Days, NaiveDate, Weekday};

use super::App;
use super::keys::{self, Cmd};
use super::style::{Icon, Palette};
use crate::i18n::{month_name, t, weekday_name};
use crate::model::{Id, MoveTarget, Schedule, View};
use crate::parse::{self, When, normalize};
use crate::recurrence::{Freq, RepeatRule};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    NewProject,
    NewArea,
    NewHeading,
    Rename(Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sheet {
    When,
    Deadline,
    MoveTo,
    Tags,
    Repeat,
    Find,
    Palette,
    Prompt(Prompt),
    Settings,
    Help,
}

#[derive(Clone)]
pub enum Act {
    When(When),
    ClearReminder,
    Deadline(Option<NaiveDate>),
    Move(MoveTarget),
    ToggleTag(Option<Id>, String),
    Go(View),
    Reveal(Id),
    Repeat(Option<RepeatRule>),
    Run(Cmd),
    Create(Prompt, String),
}

#[derive(Clone)]
pub struct Item {
    pub icon: Icon,
    pub tint: fn(&Palette) -> iced::Color,
    pub label: String,
    pub hint: Option<String>,
    pub checked: bool,
    pub act: Act,
}

fn muted(p: &Palette) -> iced::Color {
    p.muted
}

fn gold(p: &Palette) -> iced::Color {
    p.gold
}

fn red(p: &Palette) -> iced::Color {
    p.red
}

fn accent(p: &Palette) -> iced::Color {
    p.accent
}

fn teal(p: &Palette) -> iced::Color {
    p.teal
}

fn sand(p: &Palette) -> iced::Color {
    p.sand
}

fn item(icon: Icon, tint: fn(&Palette) -> iced::Color, label: impl Into<String>, act: Act) -> Item {
    Item { icon, tint, label: label.into(), hint: None, checked: false, act }
}

fn matches(label: &str, query: &str) -> bool {
    normalize(label).contains(&normalize(query))
}

pub fn date_label(date: NaiveDate, today: NaiveDate) -> String {
    let diff = (date - today).num_days();
    match diff {
        0 => t("Today").to_owned(),
        1 => t("Tomorrow").to_owned(),
        _ => {
            let weekday = weekday_name(date.weekday().num_days_from_monday() as usize);
            format!("{weekday}, {}. {}", date.day(), month_name(date.month()))
        }
    }
}

fn when_label(when: When, today: NaiveDate) -> String {
    let base = match when.schedule {
        Schedule::Inbox => t("Inbox").to_owned(),
        Schedule::Anytime => t("Anytime").to_owned(),
        Schedule::Someday => t("Someday").to_owned(),
        Schedule::On(_) if when.evening => t("This Evening").to_owned(),
        Schedule::On(date) => date_label(date, today),
    };
    match when.reminder {
        Some(time) => format!("{base}  {}", time.format("%H:%M")),
        None => base,
    }
}

fn next_monday(today: NaiveDate) -> NaiveDate {
    (1..=7)
        .map(|n| today + Days::new(n))
        .find(|d| d.weekday() == Weekday::Mon)
        .unwrap_or(today)
}

impl App {
    pub fn sheet_placeholder(&self) -> &'static str {
        match self.sheet {
            Some(Sheet::When) => t("Type a date: fri, 24.12., in 3 days, tomorrow 9:00"),
            Some(Sheet::Deadline) => t("Type a date"),
            Some(Sheet::Repeat) => t("every 2 weeks, after 3 days, until 2027-01-01"),
            Some(Sheet::Find) => t("Search everything"),
            Some(Sheet::Prompt(Prompt::NewProject)) => t("Project name"),
            Some(Sheet::Prompt(Prompt::NewArea)) => t("Area name"),
            Some(Sheet::Prompt(Prompt::NewHeading)) => t("Heading name"),
            Some(Sheet::Prompt(Prompt::Rename(_))) => t("New name"),
            _ => t("Type to filter"),
        }
    }

    pub fn sheet_items(&self) -> Vec<Item> {
        let Some(lib) = self.lib() else { return Vec::new() };
        let today = self.today();
        let query = self.query.trim();
        let filter = |items: Vec<Item>| -> Vec<Item> {
            if query.is_empty() {
                items
            } else {
                items.into_iter().filter(|i| matches(&i.label, query)).collect()
            }
        };
        match self.sheet {
            Some(Sheet::When) => {
                let mut out = Vec::new();
                if let Some(when) = parse::parse_when(query, today).filter(|_| !query.is_empty()) {
                    out.push(item(Icon::Calendar, accent, when_label(when, today), Act::When(when)));
                }
                let plain = |schedule| When { schedule, evening: false, reminder: None };
                let presets = vec![
                    item(Icon::Star, gold, t("Today"), Act::When(plain(Schedule::On(today)))),
                    item(Icon::Moon, accent, t("This Evening"), Act::When(When { schedule: Schedule::On(today), evening: true, reminder: None })),
                    item(Icon::Calendar, red, t("Tomorrow"), Act::When(plain(Schedule::On(today + Days::new(1))))),
                    item(Icon::Calendar, red, t("Next week"), Act::When(plain(Schedule::On(next_monday(today))))),
                    item(Icon::Layers, teal, t("Anytime"), Act::When(plain(Schedule::Anytime))),
                    item(Icon::Archive, sand, t("Someday"), Act::When(plain(Schedule::Someday))),
                    item(Icon::Inbox, accent, t("Inbox"), Act::When(plain(Schedule::Inbox))),
                    item(Icon::Bell, muted, t("Clear reminder"), Act::ClearReminder),
                ];
                out.extend(filter(presets));
                out
            }
            Some(Sheet::Deadline) => {
                let mut out = Vec::new();
                if let Some(date) = parse::parse_deadline(query, today).filter(|_| !query.is_empty()) {
                    out.push(item(Icon::Flag, red, date_label(date, today), Act::Deadline(Some(date))));
                }
                let presets = vec![
                    item(Icon::Flag, red, t("Today"), Act::Deadline(Some(today))),
                    item(Icon::Flag, red, t("Tomorrow"), Act::Deadline(Some(today + Days::new(1)))),
                    item(Icon::Flag, red, t("Next week"), Act::Deadline(Some(next_monday(today)))),
                    item(Icon::Flag, muted, t("Remove deadline"), Act::Deadline(None)),
                ];
                out.extend(filter(presets));
                out
            }
            Some(Sheet::MoveTo) => {
                let mut items = vec![
                    item(Icon::Inbox, accent, t("Inbox"), Act::Move(MoveTarget::Inbox)),
                    item(Icon::Box, muted, t("No Project"), Act::Move(MoveTarget::None)),
                ];
                for project in lib.all_open_projects() {
                    items.push(item(Icon::Box, accent, project.title.clone(), Act::Move(MoveTarget::Project(project.id, None))));
                    for heading in lib.tasks.iter().filter(|t| {
                        t.project == Some(project.id) && t.kind == crate::model::Kind::Heading && t.is_open() && !t.trashed
                    }) {
                        let mut entry = item(
                            Icon::Arrow,
                            muted,
                            format!("{} · {}", project.title, heading.title),
                            Act::Move(MoveTarget::Project(project.id, Some(heading.id))),
                        );
                        entry.hint = Some(t("Heading name").to_owned());
                        items.push(entry);
                    }
                }
                for area in lib.areas_sorted() {
                    items.push(item(Icon::Layers, teal, area.title.clone(), Act::Move(MoveTarget::Area(area.id))));
                }
                filter(items)
            }
            Some(Sheet::Tags) => {
                let targets = self.selection.first().or(self.expanded.as_ref()).copied();
                let assigned = targets.and_then(|id| lib.task(id)).map(|t| t.tags.clone()).unwrap_or_default();
                let mut items: Vec<Item> = lib
                    .tags_sorted()
                    .into_iter()
                    .map(|tag| {
                        let mut entry = item(Icon::Tag, teal, tag.title.clone(), Act::ToggleTag(Some(tag.id), tag.title.clone()));
                        entry.checked = assigned.contains(&tag.id);
                        entry
                    })
                    .collect();
                items = filter(items);
                if !query.is_empty() && lib.tag_named(query).is_none() {
                    items.push(item(
                        Icon::Plus,
                        accent,
                        format!("{}: {query}", t("Create tag")),
                        Act::ToggleTag(None, query.to_owned()),
                    ));
                }
                items
            }
            Some(Sheet::Repeat) => {
                let mut out = Vec::new();
                if let Some(rule) = parse::parse_repeat(query, today).filter(|_| !query.is_empty()) {
                    out.push(item(Icon::Repeat, accent, query.to_owned(), Act::Repeat(Some(rule))));
                }
                let rule = |freq| RepeatRule::new(freq, today);
                let after = |freq| RepeatRule { after_completion: true, ..RepeatRule::new(freq, today) };
                let presets = vec![
                    item(Icon::Repeat, accent, t("Daily"), Act::Repeat(Some(rule(Freq::Daily)))),
                    item(Icon::Repeat, accent, t("Every weekday"), Act::Repeat(Some(RepeatRule { weekdays: vec![0, 1, 2, 3, 4], ..rule(Freq::Weekly) }))),
                    item(Icon::Repeat, accent, t("Weekly"), Act::Repeat(Some(rule(Freq::Weekly)))),
                    item(Icon::Repeat, accent, t("Every 2 weeks"), Act::Repeat(Some(RepeatRule { interval: 2, ..rule(Freq::Weekly) }))),
                    item(Icon::Repeat, accent, t("Monthly"), Act::Repeat(Some(rule(Freq::Monthly)))),
                    item(Icon::Repeat, accent, t("Yearly"), Act::Repeat(Some(rule(Freq::Yearly)))),
                    item(Icon::Check, teal, t("Daily after completion"), Act::Repeat(Some(after(Freq::Daily)))),
                    item(Icon::Check, teal, t("Weekly after completion"), Act::Repeat(Some(after(Freq::Weekly)))),
                    item(Icon::Check, teal, t("Monthly after completion"), Act::Repeat(Some(after(Freq::Monthly)))),
                    item(Icon::Repeat, muted, t("Stop repeating"), Act::Repeat(None)),
                ];
                out.extend(filter(presets));
                out
            }
            Some(Sheet::Find) => {
                let mut out = filter(self.navigation_items());
                if query.is_empty() {
                    return out;
                }
                for id in lib.search(query, 40) {
                    let Some(task) = lib.task(id) else { continue };
                    let icon = if task.kind == crate::model::Kind::Project { Icon::Box } else { Icon::Check };
                    out.push(item(icon, muted, task.title.clone(), Act::Reveal(id)));
                }
                out
            }
            Some(Sheet::Palette) => {
                let mut items: Vec<Item> = keys::BINDINGS
                    .iter()
                    .filter(|b| !matches!(b.cmd, Cmd::Palette | Cmd::Go(_)))
                    .map(|binding| {
                        let mut entry = item(Icon::Command, muted, t(binding.label), Act::Run(binding.cmd));
                        entry.hint = Some(keys::chord_label(binding));
                        entry
                    })
                    .collect();
                items.dedup_by_key(|i| i.label.clone());
                items.extend(
                    keys::PALETTE_EXTRAS
                        .iter()
                        .map(|(cmd, label)| item(Icon::Command, muted, t(label), Act::Run(*cmd))),
                );
                filter(items)
            }
            Some(Sheet::Prompt(prompt)) => {
                vec![item(Icon::Plus, accent, query.to_owned(), Act::Create(prompt, query.to_owned()))]
            }
            _ => Vec::new(),
        }
    }

    fn navigation_items(&self) -> Vec<Item> {
        let Some(lib) = self.lib() else { return Vec::new() };
        let mut items = vec![
            item(Icon::Inbox, accent, t("Inbox"), Act::Go(View::Inbox)),
            item(Icon::Star, gold, t("Today"), Act::Go(View::Today)),
            item(Icon::Calendar, red, t("Upcoming"), Act::Go(View::Upcoming)),
            item(Icon::Layers, teal, t("Anytime"), Act::Go(View::Anytime)),
            item(Icon::Archive, sand, t("Someday"), Act::Go(View::Someday)),
            item(Icon::Book, teal, t("Logbook"), Act::Go(View::Logbook)),
            item(Icon::Trash, muted, t("Trash"), Act::Go(View::Trash)),
            item(Icon::Flag, red, t("Deadlines"), Act::Go(View::Deadlines)),
            item(Icon::Repeat, accent, t("Repeating"), Act::Go(View::Repeating)),
            item(Icon::Box, accent, t("All Projects"), Act::Go(View::AllProjects)),
        ];
        items.extend(lib.all_open_projects().into_iter().map(|p| item(Icon::Box, accent, p.title.clone(), Act::Go(View::Project(p.id)))));
        items.extend(lib.areas_sorted().into_iter().map(|a| item(Icon::Layers, teal, a.title.clone(), Act::Go(View::Area(a.id)))));
        items.extend(lib.tags_sorted().into_iter().map(|tag| item(Icon::Tag, teal, tag.title.clone(), Act::Go(View::Tag(tag.id)))));
        items
    }
}
