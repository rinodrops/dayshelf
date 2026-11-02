use serde::{Deserialize, Serialize};
use time::Date;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Auto,
    Countdown,
    Countup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardOrder {
    PastToFuture,
    FutureToPast,
}

impl Default for CardOrder {
    fn default() -> Self {
        Self::PastToFuture
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    pub lang: Option<String>,
    pub anchor: Date,
    pub direction: Direction,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
    pub device_id: String,
}

impl Card {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// `incoming` replaces `current` when it is newer, or when the timestamps match
/// and its device id sorts later.
pub fn prefer(current: &Card, incoming: &Card) -> bool {
    match incoming.updated_at.cmp(&current.updated_at) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => incoming.device_id > current.device_id,
    }
}

pub fn sort_cards(cards: &mut [Card], order: CardOrder) {
    cards.sort_by(|a, b| {
        let by_date = a.anchor.cmp(&b.anchor);
        let by_date = match order {
            CardOrder::PastToFuture => by_date,
            CardOrder::FutureToPast => by_date.reverse(),
        };
        by_date.then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn card(id: &str, y: i32, m: Month, d: u8, device: &str, updated: i64) -> Card {
        Card {
            id: Uuid::parse_str(id).unwrap(),
            title: String::new(),
            lang: None,
            anchor: Date::from_calendar_date(y, m, d).unwrap(),
            direction: Direction::Auto,
            updated_at: OffsetDateTime::from_unix_timestamp(updated).unwrap(),
            deleted_at: None,
            device_id: device.to_owned(),
        }
    }

    #[test]
    fn newer_timestamp_wins() {
        let a = card(
            "00000000-0000-0000-0000-000000000001",
            2026,
            Month::January,
            1,
            "a",
            10,
        );
        let b = card(
            "00000000-0000-0000-0000-000000000001",
            2026,
            Month::January,
            1,
            "a",
            11,
        );
        assert!(prefer(&a, &b));
        assert!(!prefer(&b, &a));
    }

    #[test]
    fn equal_timestamp_uses_device_id_order() {
        let low = card(
            "00000000-0000-0000-0000-000000000001",
            2026,
            Month::January,
            1,
            "aaaa",
            10,
        );
        let high = card(
            "00000000-0000-0000-0000-000000000001",
            2026,
            Month::January,
            1,
            "bbbb",
            10,
        );
        assert!(prefer(&low, &high));
        assert!(!prefer(&high, &low));
    }

    #[test]
    fn sort_runs_from_far_past_to_far_future_and_back() {
        let early = card(
            "00000000-0000-0000-0000-000000000002",
            2020,
            Month::January,
            1,
            "a",
            1,
        );
        let later_same = card(
            "00000000-0000-0000-0000-000000000003",
            2024,
            Month::May,
            1,
            "a",
            1,
        );
        let earlier_same = card(
            "00000000-0000-0000-0000-000000000001",
            2024,
            Month::May,
            1,
            "a",
            1,
        );
        let future = card(
            "00000000-0000-0000-0000-000000000004",
            2030,
            Month::January,
            1,
            "a",
            1,
        );
        let mut cards = vec![
            future.clone(),
            later_same.clone(),
            early.clone(),
            earlier_same.clone(),
        ];
        sort_cards(&mut cards, CardOrder::PastToFuture);
        assert_eq!(
            cards.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![early.id, earlier_same.id, later_same.id, future.id]
        );
        sort_cards(&mut cards, CardOrder::FutureToPast);
        assert_eq!(
            cards.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![future.id, earlier_same.id, later_same.id, early.id]
        );
    }
}
