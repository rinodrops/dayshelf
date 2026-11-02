use time::{Date, Month, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    IncludePrevious,
    StartAtCurrent,
}

pub fn parse_span(value: &str) -> Span {
    if value == "start_at_current" {
        Span::StartAtCurrent
    } else {
        Span::IncludePrevious
    }
}

pub fn add_months(year: i32, month: u8, delta: i32) -> Option<(i32, u8)> {
    let index = year * 12 + (month as i32 - 1) + delta;
    if !(12..=9999 * 12 + 11).contains(&index) {
        return None;
    }
    let year = index.div_euclid(12);
    let month = index.rem_euclid(12) + 1;
    Some((year, month as u8))
}

pub fn visible_months(anchor: (i32, u8), count: u8, span: Span) -> Vec<(i32, u8)> {
    let count = count.clamp(1, 3);
    let start = match (count, span) {
        (1, _) => 0,
        (_, Span::IncludePrevious) => -1,
        (_, Span::StartAtCurrent) => 0,
    };
    (0..count)
        .filter_map(|step| add_months(anchor.0, anchor.1, start + i32::from(step)))
        .collect()
}

/// Six rows of seven days, beginning on `week_start`. Days outside the month are included
/// so the grid stays rectangular.
pub fn month_cells(year: i32, month: u8, week_start: Weekday) -> Vec<Date> {
    let Ok(month) = Month::try_from(month) else {
        return Vec::new();
    };
    let Some(first) = Date::from_calendar_date(year, month, 1).ok() else {
        return Vec::new();
    };
    let mut cursor = first;
    while cursor.weekday() != week_start {
        let Some(prev) = cursor.previous_day() else {
            break;
        };
        cursor = prev;
    }
    let mut days = Vec::with_capacity(42);
    for _ in 0..42 {
        days.push(cursor);
        let Some(next) = cursor.next_day() else {
            break;
        };
        cursor = next;
    }
    days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_month_spans() {
        let anchor = (2026, 11);
        assert_eq!(
            visible_months(anchor, 3, Span::IncludePrevious),
            vec![(2026, 10), (2026, 11), (2026, 12)]
        );
        assert_eq!(
            visible_months(anchor, 3, Span::StartAtCurrent),
            vec![(2026, 11), (2026, 12), (2027, 1)]
        );
    }

    #[test]
    fn two_month_spans() {
        let anchor = (2026, 1);
        assert_eq!(
            visible_months(anchor, 2, Span::IncludePrevious),
            vec![(2025, 12), (2026, 1)]
        );
        assert_eq!(
            visible_months(anchor, 2, Span::StartAtCurrent),
            vec![(2026, 1), (2026, 2)]
        );
    }

    #[test]
    fn one_month_ignores_span_and_stays_inside_the_year_range() {
        assert_eq!(
            visible_months((2026, 6), 1, Span::IncludePrevious),
            vec![(2026, 6)]
        );
        assert!(add_months(1, 1, -1).is_none());
        assert!(add_months(9999, 12, 1).is_none());
    }

    #[test]
    fn monday_grid_starts_on_monday() {
        let cells = month_cells(2026, 11, Weekday::Monday);
        assert_eq!(cells.len(), 42);
        assert_eq!(cells[0].weekday(), Weekday::Monday);
        assert!(cells.contains(&Date::from_calendar_date(2026, Month::November, 3).unwrap()));
    }
}
