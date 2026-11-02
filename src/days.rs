use time::Date;

use crate::card::Direction;

/// How a signed day count should be described.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountKind {
    Countdown,
    Countup,
    Today,
}

/// Calendar-day difference for a card.
///
/// Countdown is `anchor - today`. Countup is `today - anchor`.
/// `auto` uses countdown for a future anchor, countup for a past anchor, and 0 today.
/// A forced direction that points the other way is negative. Today is always 0.
pub fn signed_days(today: Date, anchor: Date, direction: Direction) -> (i64, CountKind) {
    if today == anchor {
        let kind = match direction {
            Direction::Countup => CountKind::Countup,
            _ => CountKind::Countdown,
        };
        return (
            0,
            if direction == Direction::Auto {
                CountKind::Today
            } else {
                kind
            },
        );
    }
    let until = day_span(anchor, today);
    let since = -until;
    match direction {
        Direction::Countdown => (until, CountKind::Countdown),
        Direction::Countup => (since, CountKind::Countup),
        Direction::Auto => {
            if until > 0 {
                (until, CountKind::Countdown)
            } else {
                (since, CountKind::Countup)
            }
        }
    }
}

fn day_span(later_or_anchor: Date, today: Date) -> i64 {
    (later_or_anchor - today).whole_days()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn date(y: i32, m: Month, d: u8) -> Date {
        Date::from_calendar_date(y, m, d).unwrap()
    }

    #[test]
    fn auto_future_is_countdown() {
        let today = date(2026, Month::November, 3);
        let anchor = date(2026, Month::November, 10);
        assert_eq!(
            signed_days(today, anchor, Direction::Auto),
            (7, CountKind::Countdown)
        );
    }

    #[test]
    fn auto_past_is_countup() {
        let today = date(2026, Month::November, 3);
        let anchor = date(2026, Month::October, 31);
        assert_eq!(
            signed_days(today, anchor, Direction::Auto),
            (3, CountKind::Countup)
        );
    }

    #[test]
    fn today_is_zero_for_every_direction() {
        let today = date(2026, Month::November, 3);
        for direction in [Direction::Auto, Direction::Countdown, Direction::Countup] {
            let (n, _) = signed_days(today, today, direction);
            assert_eq!(n, 0);
        }
        assert_eq!(
            signed_days(today, today, Direction::Auto).1,
            CountKind::Today
        );
    }

    #[test]
    fn forced_countdown_on_a_past_date_is_negative() {
        let today = date(2026, Month::November, 3);
        let anchor = date(2020, Month::January, 1);
        let (n, kind) = signed_days(today, anchor, Direction::Countdown);
        assert_eq!(kind, CountKind::Countdown);
        assert!(n < 0);
        assert_eq!(n, day_span(anchor, today));
    }

    #[test]
    fn forced_countup_on_a_future_date_is_negative() {
        let today = date(2026, Month::November, 3);
        let anchor = date(2030, Month::January, 1);
        let (n, kind) = signed_days(today, anchor, Direction::Countup);
        assert_eq!(kind, CountKind::Countup);
        assert!(n < 0);
    }
}
