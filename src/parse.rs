//! Natural-language dates (English and German) and the quick-entry syntax:
//! `#project`, `@tag`, `!when`, `^deadline`. Use `-` for spaces inside tokens,
//! e.g. `!next-week`.

use chrono::{Datelike, Days, Months, NaiveDate, NaiveTime, Weekday};

use crate::i18n::{t, weekday_name};
use crate::model::{Id, Library, MoveTarget, Schedule};
use crate::recurrence::{End, Freq, RepeatRule, weekday_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct When {
    pub schedule: Schedule,
    pub evening: bool,
    pub reminder: Option<NaiveTime>,
}

impl When {
    fn plain(schedule: Schedule) -> Self {
        Self { schedule, evening: false, reminder: None }
    }
}

fn weekday(word: &str) -> Option<Weekday> {
    Some(match word {
        "mon" | "monday" | "mo" | "montag" => Weekday::Mon,
        "tue" | "tues" | "tuesday" | "di" | "dienstag" => Weekday::Tue,
        "wed" | "wednesday" | "mi" | "mittwoch" => Weekday::Wed,
        "thu" | "thur" | "thurs" | "thursday" | "do" | "donnerstag" => Weekday::Thu,
        "fri" | "friday" | "fr" | "freitag" => Weekday::Fri,
        "sat" | "saturday" | "sa" | "samstag" => Weekday::Sat,
        "sun" | "sunday" | "so" | "sonntag" => Weekday::Sun,
        _ => return None,
    })
}

fn month(word: &str) -> Option<u32> {
    Some(match word {
        "jan" | "january" | "januar" => 1,
        "feb" | "february" | "februar" => 2,
        "mar" | "march" | "mär" | "märz" | "maerz" => 3,
        "apr" | "april" => 4,
        "may" | "mai" => 5,
        "jun" | "june" | "juni" => 6,
        "jul" | "july" | "juli" => 7,
        "aug" | "august" => 8,
        "sep" | "sept" | "september" => 9,
        "oct" | "october" | "okt" | "oktober" => 10,
        "nov" | "november" => 11,
        "dec" | "december" | "dez" | "dezember" => 12,
        _ => return None,
    })
}

fn next_weekday(today: NaiveDate, target: Weekday) -> NaiveDate {
    let now = i64::from(today.weekday().num_days_from_monday());
    let goal = i64::from(target.num_days_from_monday());
    let ahead = (goal - now).rem_euclid(7);
    today + chrono::Duration::days(if ahead == 0 { 7 } else { ahead })
}

fn offset(today: NaiveDate, amount: u32, unit: &str) -> Option<NaiveDate> {
    match unit {
        "d" | "day" | "days" | "tag" | "tage" | "tagen" => today.checked_add_days(Days::new(amount.into())),
        "w" | "week" | "weeks" | "woche" | "wochen" => today.checked_add_days(Days::new(u64::from(amount) * 7)),
        "m" | "mo" | "month" | "months" | "monat" | "monate" | "monaten" => {
            today.checked_add_months(Months::new(amount))
        }
        "y" | "j" | "year" | "years" | "jahr" | "jahre" | "jahren" => {
            today.checked_add_months(Months::new(amount.checked_mul(12)?))
        }
        _ => None,
    }
}

/// A year-less date is taken as its next occurrence.
fn month_day(today: NaiveDate, m: u32, d: u32) -> Option<NaiveDate> {
    let this_year = NaiveDate::from_ymd_opt(today.year(), m, d)?;
    if this_year >= today {
        Some(this_year)
    } else {
        NaiveDate::from_ymd_opt(today.year() + 1, m, d)
    }
}

fn numeric_date(word: &str, today: NaiveDate) -> Option<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(word, "%Y-%m-%d") {
        return Some(date);
    }
    let dotted: Vec<&str> = word.trim_end_matches('.').split('.').collect();
    match dotted.as_slice() {
        [d, m] => month_day(today, m.parse().ok()?, d.parse().ok()?),
        [d, m, y] => {
            let year: i32 = y.parse().ok()?;
            let year = if year < 100 { 2000 + year } else { year };
            NaiveDate::from_ymd_opt(year, m.parse().ok()?, d.parse().ok()?)
        }
        _ => None,
    }
}

fn parse_time(word: &str) -> Option<NaiveTime> {
    let word = word.trim_end_matches("uhr");
    if let Ok(time) = NaiveTime::parse_from_str(word, "%H:%M") {
        return Some(time);
    }
    for (suffix, shift) in [("am", 0), ("pm", 12)] {
        if let Some(hour) = word.strip_suffix(suffix).and_then(|h| h.parse::<u32>().ok()) {
            if (1..=12).contains(&hour) {
                return NaiveTime::from_hms_opt(hour % 12 + shift, 0, 0);
            }
        }
    }
    None
}

fn parse_day(words: &[&str], today: NaiveDate) -> Option<When> {
    let date = |d| Some(When::plain(Schedule::On(d)));
    match words {
        ["today" | "heute"] => date(today),
        ["tomorrow" | "morgen" | "tom"] => date(today + Days::new(1)),
        ["evening" | "tonight" | "abend" | "heute-abend"]
        | ["this" | "heute", "evening" | "abend"] => {
            Some(When { schedule: Schedule::On(today), evening: true, reminder: None })
        }
        ["someday" | "irgendwann"] => Some(When::plain(Schedule::Someday)),
        ["anytime" | "jederzeit"] => Some(When::plain(Schedule::Anytime)),
        ["inbox" | "eingang"] => Some(When::plain(Schedule::Inbox)),
        ["next" | "nächste" | "nächster" | "nächsten" | "naechste", unit] => match *unit {
            "week" | "woche" => date(next_weekday(today, Weekday::Mon)),
            "month" | "monat" => {
                let first = today.with_day(1)?;
                date(first.checked_add_months(Months::new(1))?)
            }
            word => date(next_weekday(today, weekday(word)?)),
        },
        ["in", amount, unit] => date(offset(today, amount.parse().ok()?, unit)?),
        [word] => {
            if let Some(day) = weekday(word) {
                return date(next_weekday(today, day));
            }
            if let Some(split) = word.find(|c: char| !c.is_ascii_digit()).filter(|i| *i > 0) {
                let (amount, unit) = word.split_at(split);
                if let Some(result) = amount.parse().ok().and_then(|n| offset(today, n, unit)) {
                    return date(result);
                }
            }
            date(numeric_date(word, today)?)
        }
        [a, b] => {
            if let (Some(m), Ok(d)) = (month(a), b.trim_end_matches('.').parse()) {
                return date(month_day(today, m, d)?);
            }
            if let (Ok(d), Some(m)) = (a.trim_end_matches('.').parse(), month(b)) {
                return date(month_day(today, m, d)?);
            }
            None
        }
        _ => None,
    }
}

pub fn parse_when(input: &str, today: NaiveDate) -> Option<When> {
    let lowered = input.trim().to_lowercase();
    let mut words: Vec<&str> = lowered.split_whitespace().filter(|w| *w != "at" && *w != "um").collect();
    let reminder = words.last().and_then(|w| parse_time(w));
    if reminder.is_some() {
        words.pop();
    }
    if words.is_empty() {
        return reminder.map(|r| When { schedule: Schedule::On(today), evening: false, reminder: Some(r) });
    }
    let mut when = parse_day(&words, today)?;
    if let (Some(time), Schedule::On(_)) = (reminder, when.schedule) {
        when.reminder = Some(time);
    }
    Some(when)
}

/// A deadline with an optional time of day, e.g. `fri 17:00`.
pub fn parse_deadline_at(input: &str, today: NaiveDate) -> Option<(NaiveDate, Option<NaiveTime>)> {
    let when = parse_when(input, today)?;
    match when.schedule {
        Schedule::On(date) => Some((date, when.reminder)),
        _ => None,
    }
}

pub fn parse_deadline(input: &str, today: NaiveDate) -> Option<NaiveDate> {
    match parse_when(input, today)?.schedule {
        Schedule::On(date) => Some(date),
        _ => None,
    }
}

/// Reads a repeat description such as `every 2 weeks`, `after 3 days`,
/// `every monday until 2027-01-01`, `daily x5`, `monthly deadline 3 lead 2`.
pub fn parse_repeat(input: &str, today: NaiveDate) -> Option<RepeatRule> {
    let lowered = input.trim().to_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    let (first, rest) = words.split_first()?;
    let (after_completion, spec) = match *first {
        "after" | "nach" => (true, rest),
        "every" | "jede" | "jeden" | "jedes" | "alle" => (false, rest),
        _ => (false, words.as_slice()),
    };
    let (amount, spec) = match spec.first().and_then(|w| w.parse::<u32>().ok()) {
        Some(n) if n > 0 => (n, &spec[1..]),
        _ => (1, spec),
    };
    let (unit, options) = spec.split_first()?;
    let mut weekdays = Vec::new();
    let freq = match *unit {
        "d" | "day" | "days" | "daily" | "tag" | "tage" | "tagen" | "täglich" => Freq::Daily,
        "w" | "week" | "weeks" | "weekly" | "woche" | "wochen" | "wöchentlich" => Freq::Weekly,
        "m" | "month" | "months" | "monthly" | "monat" | "monate" | "monaten" | "monatlich" => Freq::Monthly,
        "y" | "year" | "years" | "yearly" | "jahr" | "jahre" | "jahren" | "jährlich" => Freq::Yearly,
        word => {
            weekdays.push(weekday_index(weekday(word)?));
            Freq::Weekly
        }
    };
    let first_date = match weekdays.first() {
        Some(day) => (1..=7)
            .map(|n| today + Days::new(n))
            .find(|d| weekday_index(d.weekday()) == *day)?,
        None => today,
    };
    let mut rule = RepeatRule::new(freq, first_date);
    rule.interval = amount;
    rule.weekdays = weekdays;
    rule.after_completion = after_completion;
    let mut options = options.iter();
    while let Some(word) = options.next() {
        match *word {
            "until" | "bis" => rule.end = End::OnDate(parse_deadline(options.next()?, today)?),
            "deadline" | "frist" => rule.deadline_days = Some(options.next()?.parse().ok()?),
            "lead" | "vorlauf" => rule.lead_days = options.next()?.parse().ok()?,
            count => rule.end = End::AfterCount(count.strip_prefix('x')?.parse().ok()?),
        }
    }
    Some(rule)
}

/// Removes everything but letters and digits, so `#HausUmbau` matches "Haus Umbau".
pub fn normalize(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Possible destinations, best match first.
pub fn match_targets(lib: &Library, query: &str) -> Vec<(MoveTarget, String)> {
    let needle = normalize(query);
    let mut scored: Vec<(u8, MoveTarget, String)> = Vec::new();
    let mut consider = |target: MoveTarget, title: &str| {
        let hay = normalize(title);
        let score = if hay == needle {
            0
        } else if hay.starts_with(&needle) {
            1
        } else if hay.contains(&needle) {
            2
        } else {
            return;
        };
        scored.push((score, target, title.to_owned()));
    };
    for project in lib.all_open_projects() {
        consider(MoveTarget::Project(project.id, None), &project.title);
    }
    for area in lib.areas_sorted() {
        consider(MoveTarget::Area(area.id), &area.title);
    }
    scored.sort_by_key(|(score, ..)| *score);
    scored.into_iter().map(|(_, target, title)| (target, title)).collect()
}

fn match_tags(lib: &Library, query: &str) -> Vec<(Id, String)> {
    let needle = normalize(query);
    let mut hits: Vec<(bool, Id, String)> = lib
        .tags_sorted()
        .into_iter()
        .filter(|t| normalize(&t.title).contains(&needle))
        .map(|t| (!normalize(&t.title).starts_with(&needle), t.id, t.title.clone()))
        .collect();
    hits.sort_by_key(|(later, ..)| *later);
    hits.into_iter().map(|(_, id, title)| (id, title)).collect()
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Draft {
    pub title: String,
    pub target: Option<(MoveTarget, String)>,
    pub tags: Vec<(Option<Id>, String)>,
    pub when: Option<When>,
    pub deadline: Option<NaiveDate>,
    pub deadline_time: Option<NaiveTime>,
}

/// Splits raw quick-entry text into a title and metadata. Tokens that resolve
/// to nothing stay in the title, so an ordinary `#1` is never swallowed.
pub fn parse_draft(input: &str, lib: &Library, today: NaiveDate) -> Draft {
    let mut draft = Draft::default();
    let mut title = Vec::new();
    for word in input.split_whitespace() {
        let mut chars = word.chars();
        let marker = chars.next();
        let rest = chars.as_str();
        let spaced = rest.replace('-', " ");
        let consumed = match marker {
            Some('#') if !rest.is_empty() => match match_targets(lib, rest).into_iter().next() {
                Some(found) => {
                    draft.target = Some(found);
                    true
                }
                None => false,
            },
            Some('@') if !rest.is_empty() => {
                let tag = match_tags(lib, rest)
                    .into_iter()
                    .find(|(_, name)| normalize(name) == normalize(rest))
                    .map_or((None, rest.to_owned()), |(id, name)| (Some(id), name));
                draft.tags.push(tag);
                true
            }
            Some('!') if !rest.is_empty() => match parse_when(&spaced, today) {
                Some(when) => {
                    draft.when = Some(when);
                    true
                }
                None => false,
            },
            Some('^') if !rest.is_empty() => match parse_deadline_at(&spaced, today) {
                Some((date, time)) => {
                    draft.deadline = Some(date);
                    draft.deadline_time = time;
                    true
                }
                None => false,
            },
            _ => false,
        };
        if !consumed {
            title.push(word);
        }
    }
    draft.title = title.join(" ");
    draft
}

/// The completion for the token under the end of `input`: `(replacement, label)`.
pub fn completions(input: &str, lib: &Library) -> Vec<(String, String)> {
    let Some(word) = input.rsplit(char::is_whitespace).next() else { return Vec::new() };
    let mut chars = word.chars();
    let marker = chars.next();
    let rest = chars.as_str();
    let compact = |title: &str| title.split_whitespace().collect::<String>();
    match marker {
        Some('#') => match_targets(lib, rest)
            .into_iter()
            .take(5)
            .map(|(_, title)| (format!("#{}", compact(&title)), title))
            .collect(),
        Some('@') => match_tags(lib, rest)
            .into_iter()
            .take(5)
            .map(|(_, title)| (format!("@{}", compact(&title)), title))
            .collect(),
        Some('!') => preset_words(rest, &[
            ("today", t("Today")),
            ("tomorrow", t("Tomorrow")),
            ("evening", t("This Evening")),
            ("someday", t("Someday")),
            ("anytime", t("Anytime")),
        ])
        .into_iter()
        .map(|(word, title)| (format!("!{word}"), title))
        .collect(),
        Some('^') => preset_words(rest, &[
            ("today", t("Today")),
            ("tomorrow", t("Tomorrow")),
            ("monday", weekday_name(0)),
            ("friday", weekday_name(4)),
        ])
        .into_iter()
        .map(|(word, title)| (format!("^{word}"), title))
        .collect(),
        _ => Vec::new(),
    }
}

fn preset_words(rest: &str, presets: &[(&'static str, &'static str)]) -> Vec<(&'static str, String)> {
    let rest = rest.to_lowercase();
    presets
        .iter()
        .filter(|(word, _)| word.starts_with(&rest))
        .map(|(word, title)| (*word, (*title).to_owned()))
        .collect()
}

pub fn apply_completion(input: &str, replacement: &str) -> String {
    let start = input.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    format!("{}{replacement} ", &input[..start])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Kind, Task};

    // A Sunday.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 8).unwrap()
    }

    fn on(m: u32, d: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(2026, m, d)
    }

    fn date_of(input: &str) -> Option<NaiveDate> {
        parse_deadline(input, today())
    }

    #[test]
    fn relative_words_in_both_languages() {
        assert_eq!(date_of("today"), on(11, 8));
        assert_eq!(date_of("heute"), on(11, 8));
        assert_eq!(date_of("tomorrow"), on(11, 9));
        assert_eq!(date_of("Morgen"), on(11, 9));
    }

    #[test]
    fn weekdays_are_the_next_occurrence() {
        assert_eq!(date_of("fri"), on(11, 13));
        assert_eq!(date_of("freitag"), on(11, 13));
        assert_eq!(date_of("sunday"), on(11, 15));
    }

    #[test]
    fn offsets() {
        assert_eq!(date_of("in 3 days"), on(11, 11));
        assert_eq!(date_of("in 2 wochen"), on(11, 22));
        assert_eq!(date_of("3d"), on(11, 11));
        assert_eq!(date_of("2w"), on(11, 22));
        assert_eq!(date_of("1m"), on(12, 8));
    }

    #[test]
    fn next_units() {
        assert_eq!(date_of("next week"), on(11, 9));
        assert_eq!(date_of("nächste woche"), on(11, 9));
        assert_eq!(date_of("next month"), on(12, 1));
    }

    #[test]
    fn absolute_dates() {
        assert_eq!(date_of("2026-12-24"), on(12, 24));
        assert_eq!(date_of("24.12."), on(12, 24));
        assert_eq!(date_of("24.12.2026"), on(12, 24));
        assert_eq!(date_of("dec 24"), on(12, 24));
        assert_eq!(date_of("24. dezember"), on(12, 24));
    }

    #[test]
    fn past_month_day_rolls_to_next_year() {
        assert_eq!(date_of("1.1."), NaiveDate::from_ymd_opt(2027, 1, 1));
    }

    #[test]
    fn special_schedules() {
        assert_eq!(parse_when("someday", today()).unwrap().schedule, Schedule::Someday);
        assert_eq!(parse_when("irgendwann", today()).unwrap().schedule, Schedule::Someday);
        assert!(parse_when("this evening", today()).unwrap().evening);
        assert!(parse_when("heute abend", today()).unwrap().evening);
    }

    #[test]
    fn times_become_reminders() {
        let when = parse_when("tomorrow 9:30", today()).unwrap();
        assert_eq!(when.schedule, Schedule::On(on(11, 9).unwrap()));
        assert_eq!(when.reminder, NaiveTime::from_hms_opt(9, 30, 0));
        assert_eq!(parse_when("fri 5pm", today()).unwrap().reminder, NaiveTime::from_hms_opt(17, 0, 0));
    }

    #[test]
    fn nonsense_is_rejected() {
        assert_eq!(parse_when("blorp", today()), None);
        assert_eq!(parse_when("31.2.", today()), None);
    }

    #[test]
    fn repeat_descriptions() {
        let rule = parse_repeat("every 2 weeks", today()).unwrap();
        assert_eq!((rule.freq, rule.interval, rule.after_completion), (Freq::Weekly, 2, false));
        let rule = parse_repeat("after 3 days", today()).unwrap();
        assert_eq!((rule.freq, rule.interval, rule.after_completion), (Freq::Daily, 3, true));
        let rule = parse_repeat("every monday until 31.12.2026", today()).unwrap();
        assert_eq!(rule.weekdays, vec![0]);
        assert_eq!(rule.next, on(11, 9));
        assert_eq!(rule.end, End::OnDate(on(12, 31).unwrap()));
        let rule = parse_repeat("daily x5 deadline 2 lead 1", today()).unwrap();
        assert_eq!(rule.end, End::AfterCount(5));
        assert_eq!((rule.deadline_days, rule.lead_days), (Some(2), 1));
        assert!(parse_repeat("nonsense", today()).is_none());
    }

    fn library() -> (Library, Id) {
        let mut lib = Library::default();
        let project = lib.insert(Task::new(Kind::Project, "Haus Umbau", Schedule::Anytime));
        lib.ensure_tag("Errand");
        (lib, project)
    }

    #[test]
    fn draft_extracts_all_tokens() {
        let (lib, project) = library();
        let draft = parse_draft("Fliesen kaufen #haus @errand !morgen ^fri", &lib, today());
        assert_eq!(draft.title, "Fliesen kaufen");
        assert_eq!(draft.target.unwrap().0, MoveTarget::Project(project, None));
        assert_eq!(draft.tags[0].1, "Errand");
        assert_eq!(draft.when.unwrap().schedule, Schedule::On(on(11, 9).unwrap()));
        assert_eq!(draft.deadline, on(11, 13));
    }

    #[test]
    fn hyphens_stand_for_spaces() {
        let (lib, _) = library();
        let draft = parse_draft("x !next-week", &lib, today());
        assert_eq!(draft.when.unwrap().schedule, Schedule::On(on(11, 9).unwrap()));
    }

    #[test]
    fn unresolved_tokens_stay_in_the_title() {
        let (lib, _) = library();
        let draft = parse_draft("Fix issue #1 !blorp", &lib, today());
        assert_eq!(draft.title, "Fix issue #1 !blorp");
        assert!(draft.target.is_none());
    }

    #[test]
    fn unknown_tag_is_created_later() {
        let (lib, _) = library();
        let draft = parse_draft("x @neu", &lib, today());
        assert_eq!(draft.tags, vec![(None, "neu".to_owned())]);
    }

    #[test]
    fn completion_replaces_the_last_token() {
        let (lib, _) = library();
        let found = completions("kaufen #ha", &lib);
        assert_eq!(found[0], ("#HausUmbau".to_owned(), "Haus Umbau".to_owned()));
        assert_eq!(apply_completion("kaufen #ha", &found[0].0), "kaufen #HausUmbau ");
        let draft = parse_draft("kaufen #HausUmbau", &lib, today());
        assert!(draft.target.is_some());
    }
}
