//! The library: to-dos, projects, headings, areas and tags, plus every list
//! the app can show. Pure data and logic, no UI and no I/O.

use chrono::{DateTime, Datelike, Days, Local, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::recurrence::RepeatRule;

pub type Id = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Todo,
    Project,
    Heading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Open,
    Completed,
    Canceled,
}

/// When an item should be worked on. A date on or before today means Today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Schedule {
    Inbox,
    Anytime,
    Someday,
    On(NaiveDate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Id,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub kind: Kind,
    pub title: String,
    pub notes: String,
    pub status: Status,
    pub stopped: Option<DateTime<Utc>>,
    pub logged: bool,
    pub trashed: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub schedule: Schedule,
    pub evening: bool,
    pub reminder: Option<NaiveTime>,
    pub reminded_on: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub area: Option<Id>,
    pub project: Option<Id>,
    pub heading: Option<Id>,
    pub tags: Vec<Id>,
    pub checklist: Vec<ChecklistItem>,
    pub order: f64,
    pub today_order: f64,
    pub rule: Option<RepeatRule>,
    pub template: Option<Id>,
}

impl Task {
    pub fn new(kind: Kind, title: impl Into<String>, schedule: Schedule) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            kind,
            title: title.into(),
            notes: String::new(),
            status: Status::Open,
            stopped: None,
            logged: false,
            trashed: false,
            created: now,
            modified: now,
            schedule,
            evening: false,
            reminder: None,
            reminded_on: None,
            deadline: None,
            area: None,
            project: None,
            heading: None,
            tags: Vec::new(),
            checklist: Vec::new(),
            order: 0.0,
            today_order: 0.0,
            rule: None,
            template: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.status == Status::Open
    }

    pub fn is_today(&self, today: NaiveDate) -> bool {
        matches!(self.schedule, Schedule::On(date) if date <= today)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub id: Id,
    pub title: String,
    pub collapsed: bool,
    pub tags: Vec<Id>,
    pub order: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: Id,
    pub title: String,
    pub shortcut: Option<char>,
    pub parent: Option<Id>,
    pub order: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogInterval {
    Immediately,
    Daily,
    Manually,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub log_interval: LogInterval,
    pub group_today_by_parent: bool,
    pub appearance: Appearance,
    pub text_scale: f32,
    pub reduce_motion: bool,
    pub auto_lock_minutes: u32,
    pub reminder_shows_title: bool,
    pub quick_entry_to_today: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_interval: LogInterval::Daily,
            group_today_by_parent: false,
            appearance: Appearance::Auto,
            text_scale: 1.0,
            reduce_motion: false,
            auto_lock_minutes: 15,
            reminder_shows_title: false,
            quick_entry_to_today: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Library {
    pub tasks: Vec<Task>,
    pub areas: Vec<Area>,
    pub tags: Vec<Tag>,
    pub settings: Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Someday,
    Logbook,
    Trash,
    Deadlines,
    Repeating,
    AllProjects,
    Project(Id),
    Area(Id),
    Tag(Id),
}

/// A group of rows in a list. `heading` is set for real heading items.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub title: Option<String>,
    pub heading: Option<Id>,
    pub link: Option<View>,
    pub evening: bool,
    pub rows: Vec<Id>,
    /// Future copies of repeating templates, shown read-only.
    pub previews: Vec<(Id, NaiveDate)>,
}

impl Section {
    fn plain(rows: Vec<Id>) -> Self {
        Self { title: None, heading: None, link: None, evening: false, rows, previews: Vec::new() }
    }

    fn titled(title: impl Into<String>, rows: Vec<Id>) -> Self {
        Self { title: Some(title.into()), ..Self::plain(rows) }
    }
}

pub fn local_today() -> NaiveDate {
    Local::now().date_naive()
}

impl Library {
    pub fn task(&self, id: Id) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn task_mut(&mut self, id: Id) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn area(&self, id: Id) -> Option<&Area> {
        self.areas.iter().find(|a| a.id == id)
    }

    pub fn tag(&self, id: Id) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == id)
    }

    pub fn touch(&mut self, id: Id) {
        if let Some(task) = self.task_mut(id) {
            task.modified = Utc::now();
        }
    }

    fn next_order(&self) -> f64 {
        self.tasks.iter().map(|t| t.order).fold(0.0, f64::max) + 1.0
    }

    fn next_today_order(&self) -> f64 {
        self.tasks.iter().map(|t| t.today_order).fold(0.0, f64::max) + 1.0
    }

    pub fn insert(&mut self, mut task: Task) -> Id {
        task.order = self.next_order();
        task.today_order = self.next_today_order();
        let id = task.id;
        self.tasks.push(task);
        id
    }

    pub fn add_area(&mut self, title: impl Into<String>) -> Id {
        let id = Uuid::new_v4();
        let order = self.areas.iter().map(|a| a.order).fold(0.0, f64::max) + 1.0;
        self.areas.push(Area { id, title: title.into(), collapsed: false, tags: Vec::new(), order });
        id
    }

    pub fn tag_named(&self, name: &str) -> Option<Id> {
        self.tags.iter().find(|t| t.title.eq_ignore_ascii_case(name)).map(|t| t.id)
    }

    pub fn ensure_tag(&mut self, name: &str) -> Id {
        if let Some(id) = self.tag_named(name) {
            return id;
        }
        let id = Uuid::new_v4();
        let order = self.tags.iter().map(|t| t.order).fold(0.0, f64::max) + 1.0;
        self.tags.push(Tag { id, title: name.to_owned(), shortcut: None, parent: None, order });
        id
    }

    pub fn tag_and_descendants(&self, id: Id) -> Vec<Id> {
        let mut out = vec![id];
        let mut index = 0;
        while index < out.len() {
            let current = out[index];
            out.extend(self.tags.iter().filter(|t| t.parent == Some(current)).map(|t| t.id));
            index += 1;
        }
        out
    }

    fn parent_of(&self, task: &Task) -> Option<&Task> {
        task.project.and_then(|id| self.task(id))
    }

    fn live(&self, task: &Task) -> bool {
        !task.trashed
            && !task.logged
            && task.rule.is_none()
            && self.parent_of(task).is_none_or(|p| !p.trashed && !p.logged)
    }

    fn active_parent(&self, task: &Task, today: NaiveDate) -> bool {
        self.parent_of(task).is_none_or(|p| match p.schedule {
            Schedule::Someday => false,
            Schedule::On(date) => date <= today,
            _ => true,
        })
    }

    fn collect(&self, predicate: impl Fn(&Task) -> bool) -> Vec<&Task> {
        let mut rows: Vec<&Task> = self.tasks.iter().filter(|t| predicate(t)).collect();
        rows.sort_by(|a, b| a.order.total_cmp(&b.order));
        rows
    }

    fn ids(rows: Vec<&Task>) -> Vec<Id> {
        rows.into_iter().map(|t| t.id).collect()
    }

    pub fn open_children(&self, project: Id) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.kind == Kind::Todo && t.project == Some(project) && !t.trashed && t.is_open())
            .count()
    }

    /// Share of a project's to-dos that are done.
    pub fn progress(&self, project: Id) -> f32 {
        let (mut done, mut total) = (0, 0);
        for task in &self.tasks {
            if task.kind == Kind::Todo && task.project == Some(project) && !task.trashed {
                total += 1;
                done += usize::from(!task.is_open());
            }
        }
        if total == 0 { 0.0 } else { done as f32 / total as f32 }
    }

    pub fn inbox_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| {
                t.kind == Kind::Todo && t.is_open() && t.schedule == Schedule::Inbox && self.live(t)
            })
            .count()
    }

    pub fn today_rows(&self, today: NaiveDate) -> Vec<&Task> {
        let mut rows: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                t.kind != Kind::Heading
                    && self.live(t)
                    && (t.is_today(today) || (t.is_open() && t.deadline.is_some_and(|d| d <= today)))
            })
            .collect();
        rows.sort_by(|a, b| a.today_order.total_cmp(&b.today_order));
        rows
    }

    pub fn today_count(&self, today: NaiveDate) -> usize {
        self.today_rows(today).iter().filter(|t| t.is_open()).count()
    }

    pub fn overdue_count(&self, today: NaiveDate) -> usize {
        self.today_rows(today).iter().filter(|t| t.deadline.is_some_and(|d| d <= today) && t.is_open()).count()
    }

    pub fn sections(&self, view: View, today: NaiveDate) -> Vec<Section> {
        match view {
            View::Inbox => vec![Section::plain(Self::ids(self.collect(|t| {
                t.kind == Kind::Todo && t.schedule == Schedule::Inbox && t.project.is_none() && self.live(t)
            })))],
            View::Today => self.today_sections(today),
            View::Upcoming => self.upcoming_sections(today),
            View::Anytime => self.anytime_sections(today),
            View::Someday => self.grouped(|t| {
                t.schedule == Schedule::Someday && t.is_open() && t.kind != Kind::Heading && self.live(t)
            }),
            View::Logbook => self.logbook_sections(),
            View::Trash => vec![Section::plain(Self::ids(self.collect(|t| {
                t.trashed && t.kind != Kind::Heading && self.parent_of(t).is_none_or(|p| !p.trashed)
            })))],
            View::Deadlines => {
                let mut rows =
                    self.collect(|t| t.deadline.is_some() && t.is_open() && t.kind != Kind::Heading && self.live(t));
                rows.sort_by_key(|t| t.deadline);
                vec![Section::plain(Self::ids(rows))]
            }
            View::Repeating => vec![Section::plain(Self::ids(
                self.collect(|t| t.rule.is_some() && !t.trashed),
            ))],
            View::AllProjects => self.all_projects(),
            View::Project(id) => self.project_sections(id),
            View::Area(id) => self.area_sections(id, today),
            View::Tag(id) => {
                let wanted = self.tag_and_descendants(id);
                vec![Section::plain(Self::ids(self.collect(|t| {
                    t.kind != Kind::Heading
                        && t.is_open()
                        && self.live(t)
                        && t.tags.iter().any(|tag| wanted.contains(tag))
                })))]
            }
        }
    }

    fn today_sections(&self, today: NaiveDate) -> Vec<Section> {
        let rows = self.today_rows(today);
        let (evening, day): (Vec<&Task>, Vec<&Task>) = rows.into_iter().partition(|t| t.evening);
        let mut out = if self.settings.group_today_by_parent {
            self.group_rows(day)
        } else {
            vec![Section::plain(Self::ids(day))]
        };
        if !evening.is_empty() {
            out.push(Section {
                evening: true,
                ..Section::titled("This Evening", Self::ids(evening))
            });
        }
        out
    }

    fn upcoming_sections(&self, today: NaiveDate) -> Vec<Section> {
        let horizon = today + Days::new(366);
        let mut by_day: std::collections::BTreeMap<NaiveDate, (Vec<Id>, Vec<(Id, NaiveDate)>)> =
            Default::default();
        for task in self.collect(|t| {
            t.is_open() && t.kind != Kind::Heading && self.live(t) && matches!(t.schedule, Schedule::On(d) if d > today)
        }) {
            if let Schedule::On(date) = task.schedule {
                by_day.entry(date).or_default().0.push(task.id);
            }
        }
        for template in self.tasks.iter().filter(|t| t.rule.is_some() && !t.trashed) {
            let rule = template.rule.as_ref().expect("filtered on rule");
            for date in rule.preview(horizon, 60) {
                if date > today {
                    by_day.entry(date).or_default().1.push((template.id, date));
                }
            }
        }
        by_day
            .into_iter()
            .map(|(date, (rows, previews))| Section {
                previews,
                ..Section::titled(date.format("%Y-%m-%d").to_string(), rows)
            })
            .collect()
    }

    fn anytime_sections(&self, today: NaiveDate) -> Vec<Section> {
        self.grouped(|t| {
            t.is_open()
                && t.kind == Kind::Todo
                && self.live(t)
                && self.active_parent(t, today)
                && match t.schedule {
                    Schedule::Anytime => true,
                    Schedule::On(date) => date <= today,
                    _ => false,
                }
        })
    }

    fn grouped(&self, predicate: impl Fn(&Task) -> bool) -> Vec<Section> {
        self.group_rows(self.collect(predicate))
    }

    /// Unfiled rows first, then one section per area or project.
    fn group_rows(&self, rows: Vec<&Task>) -> Vec<Section> {
        let mut unfiled = Vec::new();
        let mut groups: Vec<(View, String, Vec<Id>)> = Vec::new();
        for task in rows {
            let (view, title) = if let Some(project) = self.parent_of(task) {
                (View::Project(project.id), project.title.clone())
            } else if let Some(area) = task.area.and_then(|id| self.area(id)) {
                (View::Area(area.id), area.title.clone())
            } else {
                unfiled.push(task.id);
                continue;
            };
            match groups.iter_mut().find(|(v, ..)| *v == view) {
                Some((_, _, ids)) => ids.push(task.id),
                None => groups.push((view, title, vec![task.id])),
            }
        }
        let mut out = vec![Section::plain(unfiled)];
        out.extend(groups.into_iter().map(|(view, title, rows)| Section {
            link: Some(view),
            ..Section::titled(title, rows)
        }));
        out
    }

    fn logbook_sections(&self) -> Vec<Section> {
        let mut rows = self.collect(|t| t.logged && !t.trashed && t.kind != Kind::Heading);
        rows.sort_by_key(|t| std::cmp::Reverse(t.stopped));
        let mut out: Vec<(String, Vec<Id>)> = Vec::new();
        for task in rows {
            let local = task.stopped.unwrap_or(task.modified).with_timezone(&Local);
            let label = format!("{:04}-{:02}", local.year(), local.month());
            match out.last_mut() {
                Some((last, ids)) if *last == label => ids.push(task.id),
                _ => out.push((label, vec![task.id])),
            }
        }
        out.into_iter().map(|(label, rows)| Section::titled(label, rows)).collect()
    }

    fn all_projects(&self) -> Vec<Section> {
        self.grouped(|t| t.kind == Kind::Project && t.is_open() && self.live(t))
    }

    fn project_sections(&self, id: Id) -> Vec<Section> {
        let in_project = |t: &Task| t.project == Some(id) && !t.trashed && !t.logged && t.rule.is_none();
        let loose = self.collect(|t| in_project(t) && t.kind == Kind::Todo && t.heading.is_none());
        let mut out = vec![Section::plain(Self::ids(loose))];
        for heading in self.collect(|t| in_project(t) && t.kind == Kind::Heading && t.is_open()) {
            let rows = self.collect(|t| in_project(t) && t.kind == Kind::Todo && t.heading == Some(heading.id));
            out.push(Section {
                heading: Some(heading.id),
                ..Section::titled(heading.title.clone(), Self::ids(rows))
            });
        }
        out
    }

    fn area_sections(&self, id: Id, today: NaiveDate) -> Vec<Section> {
        let todos = self.collect(|t| {
            t.area == Some(id) && t.kind == Kind::Todo && t.project.is_none() && t.is_open() && self.live(t)
                && t.schedule != Schedule::Someday
                && !matches!(t.schedule, Schedule::On(d) if d > today)
        });
        let projects = self.collect(|t| {
            t.area == Some(id) && t.kind == Kind::Project && t.is_open() && self.live(t)
        });
        vec![Section::plain(Self::ids(projects)), Section::plain(Self::ids(todos))]
    }

    pub fn sidebar_projects(&self, area: Option<Id>, today: NaiveDate) -> Vec<&Task> {
        self.collect(|t| {
            t.kind == Kind::Project
                && t.area == area
                && t.is_open()
                && self.live(t)
                && t.schedule != Schedule::Someday
                && !matches!(t.schedule, Schedule::On(d) if d > today)
        })
    }

    pub fn all_open_projects(&self) -> Vec<&Task> {
        self.collect(|t| t.kind == Kind::Project && t.is_open() && self.live(t))
    }

    pub fn areas_sorted(&self) -> Vec<&Area> {
        let mut out: Vec<&Area> = self.areas.iter().collect();
        out.sort_by(|a, b| a.order.total_cmp(&b.order));
        out
    }

    pub fn tags_sorted(&self) -> Vec<&Tag> {
        let mut out: Vec<&Tag> = self.tags.iter().collect();
        out.sort_by(|a, b| a.order.total_cmp(&b.order));
        out
    }

    /// Case-insensitive search across titles, notes, checklists and tags.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Id> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(u8, &Task)> = self
            .tasks
            .iter()
            .filter(|t| !t.trashed && t.kind != Kind::Heading)
            .filter_map(|t| {
                let title = t.title.to_lowercase();
                let score = if title.starts_with(&needle) {
                    0
                } else if title.contains(&needle) {
                    1
                } else if t.notes.to_lowercase().contains(&needle)
                    || t.checklist.iter().any(|c| c.title.to_lowercase().contains(&needle))
                    || t.tags.iter().filter_map(|id| self.tag(*id)).any(|tag| tag.title.to_lowercase().contains(&needle))
                {
                    2
                } else {
                    return None;
                };
                Some((score + 3 * u8::from(!t.is_open() || t.logged), t))
            })
            .collect();
        scored.sort_by_key(|(score, task)| (*score, std::cmp::Reverse(task.modified)));
        scored.into_iter().take(limit).map(|(_, t)| t.id).collect()
    }

    pub fn set_status(&mut self, id: Id, status: Status, today: NaiveDate) {
        let was_open;
        let (kind, template) = {
            let Some(task) = self.task_mut(id) else { return };
            was_open = task.is_open();
            task.status = status;
            task.stopped = (status != Status::Open).then(Utc::now);
            if status == Status::Open {
                task.logged = false;
            }
            task.modified = Utc::now();
            (task.kind, task.template)
        };
        if kind == Kind::Project && status != Status::Open {
            for child in &mut self.tasks {
                if child.project == Some(id) && child.kind == Kind::Todo && child.is_open() {
                    child.status = status;
                    child.stopped = Some(Utc::now());
                }
            }
        }
        if kind == Kind::Heading && status != Status::Open {
            for child in &mut self.tasks {
                if child.heading == Some(id) && child.is_open() {
                    child.status = status;
                    child.stopped = Some(Utc::now());
                }
            }
        }
        if was_open && status != Status::Open {
            if let Some(rule) = template.and_then(|t| self.task_mut(t)).and_then(|t| t.rule.as_mut()) {
                rule.on_instance_completed(today);
            }
        }
    }

    pub fn set_trashed(&mut self, id: Id, trashed: bool) {
        let is_heading = self.task(id).is_some_and(|t| t.kind == Kind::Heading);
        for task in &mut self.tasks {
            if task.id == id || (is_heading && task.heading == Some(id)) {
                task.trashed = trashed;
                task.modified = Utc::now();
            }
        }
    }

    pub fn empty_trash(&mut self) {
        let doomed: Vec<Id> = self.tasks.iter().filter(|t| t.trashed).map(|t| t.id).collect();
        self.tasks.retain(|t| {
            !t.trashed && t.project.is_none_or(|p| !doomed.contains(&p)) && t.heading.is_none_or(|h| !doomed.contains(&h))
        });
    }

    pub fn delete_area(&mut self, id: Id) {
        self.areas.retain(|a| a.id != id);
        for task in &mut self.tasks {
            if task.area == Some(id) {
                task.area = None;
            }
        }
    }

    pub fn delete_tag(&mut self, id: Id) {
        let doomed = self.tag_and_descendants(id);
        self.tags.retain(|t| !doomed.contains(&t.id));
        for task in &mut self.tasks {
            task.tags.retain(|t| !doomed.contains(t));
        }
        for area in &mut self.areas {
            area.tags.retain(|t| !doomed.contains(t));
        }
    }

    /// Files a to-do under a project (with optional heading), an area, or nothing.
    pub fn move_to(&mut self, id: Id, target: MoveTarget) {
        let inbox_to_anytime = |task: &mut Task| {
            if task.schedule == Schedule::Inbox {
                task.schedule = Schedule::Anytime;
            }
        };
        let Some(task) = self.task_mut(id) else { return };
        match target {
            MoveTarget::Inbox => {
                task.project = None;
                task.heading = None;
                task.area = None;
                task.schedule = Schedule::Inbox;
                task.evening = false;
            }
            MoveTarget::Project(project, heading) => {
                task.project = Some(project);
                task.heading = heading;
                task.area = None;
                inbox_to_anytime(task);
            }
            MoveTarget::Area(area) => {
                task.project = None;
                task.heading = None;
                task.area = Some(area);
                inbox_to_anytime(task);
            }
            MoveTarget::None => {
                task.project = None;
                task.heading = None;
                task.area = None;
                inbox_to_anytime(task);
            }
        }
        task.modified = Utc::now();
    }

    pub fn duplicate(&mut self, id: Id) -> Option<Id> {
        let mut copy = self.task(id)?.clone();
        let now = Utc::now();
        copy.id = Uuid::new_v4();
        copy.created = now;
        copy.modified = now;
        copy.status = Status::Open;
        copy.stopped = None;
        copy.logged = false;
        for item in &mut copy.checklist {
            item.id = Uuid::new_v4();
            item.done = false;
        }
        let new_id = self.insert(copy);
        if self.task(id)?.kind == Kind::Project {
            let mut heading_map = std::collections::HashMap::new();
            let headings: Vec<Task> = self
                .tasks
                .iter()
                .filter(|t| t.project == Some(id) && t.kind == Kind::Heading)
                .cloned()
                .collect();
            for mut heading in headings {
                let old = heading.id;
                heading.id = Uuid::new_v4();
                heading.project = Some(new_id);
                heading_map.insert(old, heading.id);
                self.insert(heading);
            }
            let children: Vec<Task> = self
                .tasks
                .iter()
                .filter(|t| t.project == Some(id) && t.kind == Kind::Todo && !t.trashed)
                .cloned()
                .collect();
            for mut child in children {
                child.id = Uuid::new_v4();
                child.project = Some(new_id);
                child.status = Status::Open;
                child.stopped = None;
                child.heading = child.heading.and_then(|h| heading_map.get(&h).copied());
                self.insert(child);
            }
        }
        Some(new_id)
    }

    pub fn convert_to_project(&mut self, id: Id) {
        let Some(task) = self.task_mut(id) else { return };
        if task.kind != Kind::Todo {
            return;
        }
        task.kind = Kind::Project;
        task.project = None;
        task.heading = None;
        if task.schedule == Schedule::Inbox {
            task.schedule = Schedule::Anytime;
        }
        let items = std::mem::take(&mut task.checklist);
        let schedule = task.schedule;
        for item in items {
            let mut child = Task::new(Kind::Todo, item.title, schedule);
            child.project = Some(id);
            if item.done {
                child.status = Status::Completed;
                child.stopped = Some(Utc::now());
            }
            self.insert(child);
        }
    }

    /// Rewrites list positions so `ids` appear in the given order.
    pub fn apply_order(&mut self, ids: &[Id], today_list: bool) {
        for (index, id) in ids.iter().enumerate() {
            if let Some(task) = self.task_mut(*id) {
                if today_list {
                    task.today_order = index as f64;
                } else {
                    task.order = index as f64;
                }
            }
        }
    }

    /// Moves finished items into the Logbook according to the log interval.
    pub fn sweep_logbook(&mut self, today: NaiveDate) -> bool {
        let interval = self.settings.log_interval;
        let mut changed = false;
        for task in &mut self.tasks {
            if task.is_open() || task.logged || task.trashed {
                continue;
            }
            let due = match interval {
                LogInterval::Immediately => true,
                LogInterval::Daily => {
                    task.stopped.is_some_and(|s| s.with_timezone(&Local).date_naive() < today)
                }
                LogInterval::Manually => false,
            };
            if due {
                task.logged = true;
                changed = true;
            }
        }
        changed
    }

    pub fn log_now(&mut self) {
        for task in &mut self.tasks {
            if !task.is_open() && !task.trashed {
                task.logged = true;
            }
        }
    }

    /// Creates the instances that repeating templates owe as of `today`.
    pub fn spawn_repeats(&mut self, today: NaiveDate) -> bool {
        let mut changed = false;
        loop {
            let Some(index) = self.tasks.iter().position(|t| {
                !t.trashed && t.rule.as_ref().is_some_and(|r| r.is_due(today))
            }) else {
                return changed;
            };
            let template = self.tasks[index].clone();
            let rule = template.rule.as_ref().expect("position checked rule");
            let date = rule.next.expect("due rule has a date");
            let mut copy = Task::new(template.kind, template.title.clone(), Schedule::On(date));
            copy.notes = template.notes.clone();
            copy.evening = template.evening;
            copy.reminder = template.reminder;
            copy.area = template.area;
            copy.project = template.project;
            copy.heading = template.heading;
            copy.tags = template.tags.clone();
            copy.checklist = template
                .checklist
                .iter()
                .map(|item| ChecklistItem { id: Uuid::new_v4(), title: item.title.clone(), done: false })
                .collect();
            copy.deadline = rule.deadline_days.map(|days| date + Days::new(u64::from(days)));
            copy.template = Some(template.id);
            self.insert(copy);
            self.tasks[index].rule.as_mut().expect("position checked rule").advance();
            changed = true;
        }
    }

    /// To-dos whose reminder should fire now; marks them so each fires once a day.
    pub fn take_due_reminders(&mut self, today: NaiveDate, now: NaiveTime) -> Vec<Id> {
        let mut out = Vec::new();
        for task in &mut self.tasks {
            let fires = task.is_open()
                && !task.trashed
                && task.rule.is_none()
                && matches!(task.schedule, Schedule::On(d) if d == today)
                && task.reminder.is_some_and(|time| time <= now)
                && task.reminded_on != Some(today);
            if fires {
                task.reminded_on = Some(today);
                out.push(task.id);
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    Inbox,
    Project(Id, Option<Id>),
    Area(Id),
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recurrence::Freq;

    fn day(n: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, n).unwrap()
    }

    fn rows(lib: &Library, view: View, today: NaiveDate) -> Vec<Id> {
        lib.sections(view, today).into_iter().flat_map(|s| s.rows).collect()
    }

    fn todo(lib: &mut Library, schedule: Schedule) -> Id {
        lib.insert(Task::new(Kind::Todo, "t", schedule))
    }

    #[test]
    fn schedule_decides_the_list() {
        let mut lib = Library::default();
        let inbox = todo(&mut lib, Schedule::Inbox);
        let today = todo(&mut lib, Schedule::On(day(8)));
        let future = todo(&mut lib, Schedule::On(day(20)));
        let someday = todo(&mut lib, Schedule::Someday);
        assert_eq!(rows(&lib, View::Inbox, day(8)), vec![inbox]);
        assert_eq!(rows(&lib, View::Today, day(8)), vec![today]);
        assert_eq!(rows(&lib, View::Upcoming, day(8)), vec![future]);
        assert_eq!(rows(&lib, View::Someday, day(8)), vec![someday]);
        assert_eq!(rows(&lib, View::Anytime, day(8)), vec![today]);
    }

    #[test]
    fn scheduled_item_rolls_into_today_and_stays() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::On(day(10)));
        assert!(rows(&lib, View::Today, day(9)).is_empty());
        assert_eq!(rows(&lib, View::Today, day(10)), vec![id]);
        assert_eq!(rows(&lib, View::Today, day(12)), vec![id]);
    }

    #[test]
    fn deadline_pulls_item_into_today() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::Anytime);
        lib.task_mut(id).unwrap().deadline = Some(day(8));
        assert_eq!(rows(&lib, View::Today, day(8)), vec![id]);
        assert_eq!(lib.overdue_count(day(8)), 1);
    }

    #[test]
    fn evening_gets_its_own_section() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::On(day(8)));
        lib.task_mut(id).unwrap().evening = true;
        let sections = lib.sections(View::Today, day(8));
        assert!(sections.iter().any(|s| s.evening && s.rows == vec![id]));
    }

    #[test]
    fn someday_project_hides_children_from_anytime() {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Someday));
        let id = todo(&mut lib, Schedule::Anytime);
        lib.task_mut(id).unwrap().project = Some(project);
        assert!(rows(&lib, View::Anytime, day(8)).is_empty());
        lib.task_mut(project).unwrap().schedule = Schedule::Anytime;
        assert_eq!(rows(&lib, View::Anytime, day(8)), vec![id]);
    }

    #[test]
    fn moving_out_of_inbox_makes_it_anytime() {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Anytime));
        let id = todo(&mut lib, Schedule::Inbox);
        lib.move_to(id, MoveTarget::Project(project, None));
        let task = lib.task(id).unwrap();
        assert_eq!(task.schedule, Schedule::Anytime);
        assert_eq!(task.project, Some(project));
        assert_eq!(rows(&lib, View::Project(project), day(8)), vec![id]);
    }

    #[test]
    fn completing_a_project_completes_children() {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Anytime));
        let id = todo(&mut lib, Schedule::Anytime);
        lib.task_mut(id).unwrap().project = Some(project);
        lib.set_status(project, Status::Completed, day(8));
        assert_eq!(lib.task(id).unwrap().status, Status::Completed);
    }

    #[test]
    fn progress_counts_done_children() {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Anytime));
        let a = todo(&mut lib, Schedule::Anytime);
        let b = todo(&mut lib, Schedule::Anytime);
        lib.task_mut(a).unwrap().project = Some(project);
        lib.task_mut(b).unwrap().project = Some(project);
        lib.set_status(a, Status::Completed, day(8));
        assert_eq!(lib.progress(project), 0.5);
    }

    #[test]
    fn immediate_logging_moves_to_logbook() {
        let mut lib = Library::default();
        lib.settings.log_interval = LogInterval::Immediately;
        let id = todo(&mut lib, Schedule::Anytime);
        lib.set_status(id, Status::Completed, day(8));
        assert!(lib.sweep_logbook(day(8)));
        assert_eq!(rows(&lib, View::Logbook, day(8)), vec![id]);
    }

    #[test]
    fn manual_logging_waits() {
        let mut lib = Library::default();
        lib.settings.log_interval = LogInterval::Manually;
        let id = todo(&mut lib, Schedule::Anytime);
        lib.set_status(id, Status::Completed, day(8));
        assert!(!lib.sweep_logbook(day(9)));
        lib.log_now();
        assert_eq!(rows(&lib, View::Logbook, day(9)), vec![id]);
    }

    #[test]
    fn trash_hides_and_empties() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::Inbox);
        lib.set_trashed(id, true);
        assert!(rows(&lib, View::Inbox, day(8)).is_empty());
        assert_eq!(rows(&lib, View::Trash, day(8)), vec![id]);
        lib.empty_trash();
        assert!(lib.tasks.is_empty());
    }

    #[test]
    fn repeating_spawns_once_per_occurrence() {
        let mut lib = Library::default();
        let mut template = Task::new(Kind::Todo, "water plants", Schedule::Someday);
        template.rule = Some(RepeatRule::new(Freq::Daily, day(8)));
        template.checklist.push(ChecklistItem { id: Uuid::new_v4(), title: "a".into(), done: true });
        let template = lib.insert(template);
        assert!(lib.spawn_repeats(day(9)));
        assert!(!lib.spawn_repeats(day(9)));
        let copies: Vec<&Task> = lib.tasks.iter().filter(|t| t.template == Some(template)).collect();
        assert_eq!(copies.len(), 2);
        assert!(!copies[0].checklist[0].done);
        assert_eq!(rows(&lib, View::Repeating, day(9)), vec![template]);
    }

    #[test]
    fn upcoming_previews_repeating_copies() {
        let mut lib = Library::default();
        let mut template = Task::new(Kind::Todo, "weekly", Schedule::Someday);
        template.rule = Some(RepeatRule::new(Freq::Weekly, day(15)));
        lib.insert(template);
        let sections = lib.sections(View::Upcoming, day(8));
        assert!(sections.iter().any(|s| !s.previews.is_empty()));
    }

    #[test]
    fn reminder_fires_once() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::On(day(8)));
        lib.task_mut(id).unwrap().reminder = NaiveTime::from_hms_opt(9, 0, 0);
        let early = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let late = NaiveTime::from_hms_opt(9, 1, 0).unwrap();
        assert!(lib.take_due_reminders(day(8), early).is_empty());
        assert_eq!(lib.take_due_reminders(day(8), late), vec![id]);
        assert!(lib.take_due_reminders(day(8), late).is_empty());
    }

    #[test]
    fn search_finds_titles_notes_and_tags() {
        let mut lib = Library::default();
        let a = lib.insert(Task::new(Kind::Todo, "Buy milk", Schedule::Inbox));
        let b = lib.insert(Task::new(Kind::Todo, "other", Schedule::Inbox));
        lib.task_mut(b).unwrap().notes = "needs MILK too".into();
        assert_eq!(lib.search("milk", 10), vec![a, b]);
        assert!(lib.search("  ", 10).is_empty());
    }

    #[test]
    fn convert_checklist_to_child_todos() {
        let mut lib = Library::default();
        let id = todo(&mut lib, Schedule::Inbox);
        lib.task_mut(id).unwrap().checklist.push(ChecklistItem { id: Uuid::new_v4(), title: "step".into(), done: false });
        lib.convert_to_project(id);
        assert_eq!(lib.task(id).unwrap().kind, Kind::Project);
        assert_eq!(lib.open_children(id), 1);
    }

    #[test]
    fn duplicate_project_copies_children_and_headings() {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Anytime));
        let mut heading = Task::new(Kind::Heading, "h", Schedule::Anytime);
        heading.project = Some(project);
        let heading = lib.insert(heading);
        let mut child = Task::new(Kind::Todo, "c", Schedule::Anytime);
        child.project = Some(project);
        child.heading = Some(heading);
        lib.insert(child);
        let copy = lib.duplicate(project).unwrap();
        let sections = lib.sections(View::Project(copy), day(8));
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[1].rows.len(), 1);
    }

    fn project_with_heading(lib: &mut Library) -> (Id, Id, Id) {
        let project = lib.insert(Task::new(Kind::Project, "p", Schedule::Anytime));
        let mut heading = Task::new(Kind::Heading, "h", Schedule::Anytime);
        heading.project = Some(project);
        let heading = lib.insert(heading);
        let mut child = Task::new(Kind::Todo, "c", Schedule::Anytime);
        child.project = Some(project);
        child.heading = Some(heading);
        (project, heading, lib.insert(child))
    }

    #[test]
    fn archiving_a_heading_finishes_its_todos() {
        let mut lib = Library::default();
        let (project, heading, child) = project_with_heading(&mut lib);
        lib.set_status(heading, Status::Completed, day(8));
        assert_eq!(lib.task(child).unwrap().status, Status::Completed);
        assert_eq!(lib.sections(View::Project(project), day(8)).len(), 1);
    }

    #[test]
    fn trashing_a_heading_takes_its_todos_along() {
        let mut lib = Library::default();
        let (_, heading, child) = project_with_heading(&mut lib);
        lib.set_trashed(heading, true);
        assert!(lib.task(child).unwrap().trashed);
        assert!(rows(&lib, View::Anytime, day(8)).is_empty());
    }

    #[test]
    fn nested_tags_match_through_parent() {
        let mut lib = Library::default();
        let parent = lib.ensure_tag("work");
        let child = lib.ensure_tag("calls");
        lib.tags.iter_mut().find(|t| t.id == child).unwrap().parent = Some(parent);
        let id = todo(&mut lib, Schedule::Anytime);
        lib.task_mut(id).unwrap().tags.push(child);
        assert_eq!(rows(&lib, View::Tag(parent), day(8)), vec![id]);
    }
}
