//! Repeat rules for templates that spawn to-do instances.

use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    Never,
    AfterCount(u32),
    OnDate(NaiveDate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatRule {
    pub freq: Freq,
    pub interval: u32,
    /// Monday = 0. Only used by weekly rules; empty means the anchor's weekday.
    pub weekdays: Vec<u8>,
    /// Count from the previous completion instead of the calendar.
    pub after_completion: bool,
    pub end: End,
    /// The instance's deadline, in days after its start date.
    pub deadline_days: Option<u32>,
    /// How many days before its date a copy appears.
    pub lead_days: u32,
    pub paused: bool,
    /// Date of the next copy. `None` while an after-completion rule waits.
    pub next: Option<NaiveDate>,
    pub spawned: u32,
}

impl RepeatRule {
    pub fn new(freq: Freq, first: NaiveDate) -> Self {
        Self {
            freq,
            interval: 1,
            weekdays: Vec::new(),
            after_completion: false,
            end: End::Never,
            deadline_days: None,
            lead_days: 0,
            paused: false,
            next: Some(first),
            spawned: 0,
        }
    }

    /// The first occurrence strictly after `from`.
    pub fn following(&self, from: NaiveDate) -> NaiveDate {
        let n = self.interval.max(1);
        match self.freq {
            Freq::Daily => from + Days::new(u64::from(n)),
            Freq::Weekly if self.weekdays.is_empty() => from + Days::new(u64::from(n) * 7),
            Freq::Weekly => self.next_weekday(from, n),
            Freq::Monthly => add_months_clamped(from, n),
            Freq::Yearly => add_months_clamped(from, n.saturating_mul(12)),
        }
    }

    fn next_weekday(&self, from: NaiveDate, interval: u32) -> NaiveDate {
        let today = from.weekday().num_days_from_monday() as u8;
        if let Some(day) = self.weekdays.iter().copied().filter(|d| *d > today).min() {
            return from + Days::new(u64::from(day - today));
        }
        let first = self.weekdays.iter().copied().min().unwrap_or(today);
        let monday = from - Days::new(u64::from(today));
        monday + Days::new(u64::from(interval) * 7 + u64::from(first))
    }

    pub fn is_finished(&self, date: NaiveDate) -> bool {
        match self.end {
            End::Never => false,
            End::AfterCount(max) => self.spawned >= max,
            End::OnDate(last) => date > last,
        }
    }

    pub fn is_due(&self, today: NaiveDate) -> bool {
        !self.paused
            && self.next.is_some_and(|date| {
                !self.is_finished(date) && date - Days::new(u64::from(self.lead_days)) <= today
            })
    }

    /// Records one spawned copy and moves `next` forward.
    pub fn advance(&mut self) {
        self.spawned += 1;
        self.next = if self.after_completion {
            None
        } else {
            self.next.map(|date| self.following(date))
        };
    }

    pub fn on_instance_completed(&mut self, completed: NaiveDate) {
        if self.after_completion && self.next.is_none() {
            self.next = Some(self.following(completed));
        }
    }

    /// Up to `limit` future dates, for the Upcoming list.
    pub fn preview(&self, until: NaiveDate, limit: usize) -> Vec<NaiveDate> {
        let mut out = Vec::new();
        if self.paused {
            return out;
        }
        let mut probe = self.clone();
        while let Some(date) = probe.next {
            if out.len() >= limit || date > until || probe.is_finished(date) {
                break;
            }
            out.push(date);
            probe.advance();
        }
        out
    }
}

fn add_months_clamped(date: NaiveDate, months: u32) -> NaiveDate {
    date.checked_add_months(Months::new(months)).unwrap_or(date)
}

pub fn weekday_index(day: Weekday) -> u8 {
    day.num_days_from_monday() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn daily_with_interval() {
        let mut rule = RepeatRule::new(Freq::Daily, date(2026, 1, 1));
        rule.interval = 3;
        assert_eq!(rule.following(date(2026, 1, 1)), date(2026, 1, 4));
    }

    #[test]
    fn weekly_picks_the_next_selected_day() {
        let mut rule = RepeatRule::new(Freq::Weekly, date(2026, 11, 2));
        rule.weekdays = vec![0, 3];
        assert_eq!(rule.following(date(2026, 11, 2)), date(2026, 11, 5));
        assert_eq!(rule.following(date(2026, 11, 5)), date(2026, 11, 9));
        rule.interval = 2;
        assert_eq!(rule.following(date(2026, 11, 5)), date(2026, 11, 16));
    }

    #[test]
    fn monthly_clamps_to_month_end() {
        let rule = RepeatRule::new(Freq::Monthly, date(2026, 1, 31));
        assert_eq!(rule.following(date(2026, 1, 31)), date(2026, 2, 28));
    }

    #[test]
    fn yearly_handles_leap_day() {
        let rule = RepeatRule::new(Freq::Yearly, date(2028, 2, 29));
        assert_eq!(rule.following(date(2028, 2, 29)), date(2029, 2, 28));
    }

    #[test]
    fn end_after_count_stops() {
        let mut rule = RepeatRule::new(Freq::Daily, date(2026, 1, 1));
        rule.end = End::AfterCount(2);
        assert_eq!(rule.preview(date(2026, 2, 1), 10).len(), 2);
    }

    #[test]
    fn end_on_date_stops() {
        let mut rule = RepeatRule::new(Freq::Daily, date(2026, 1, 1));
        rule.end = End::OnDate(date(2026, 1, 3));
        assert_eq!(rule.preview(date(2026, 2, 1), 10).len(), 3);
    }

    #[test]
    fn after_completion_waits_for_completion() {
        let mut rule = RepeatRule::new(Freq::Daily, date(2026, 1, 1));
        rule.after_completion = true;
        rule.interval = 2;
        rule.advance();
        assert_eq!(rule.next, None);
        assert!(!rule.is_due(date(2026, 1, 20)));
        rule.on_instance_completed(date(2026, 1, 10));
        assert_eq!(rule.next, Some(date(2026, 1, 12)));
    }

    #[test]
    fn lead_days_bring_the_copy_forward() {
        let mut rule = RepeatRule::new(Freq::Weekly, date(2026, 1, 10));
        rule.lead_days = 3;
        assert!(!rule.is_due(date(2026, 1, 6)));
        assert!(rule.is_due(date(2026, 1, 7)));
    }

    #[test]
    fn paused_rule_is_never_due() {
        let mut rule = RepeatRule::new(Freq::Daily, date(2026, 1, 1));
        rule.paused = true;
        assert!(!rule.is_due(date(2026, 6, 1)));
        assert!(rule.preview(date(2026, 6, 1), 5).is_empty());
    }
}
