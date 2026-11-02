use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::card::Card;
use crate::card::Direction;

const DATE: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day]");

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Format(String),
}

impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn cards_dir(root: &Path) -> PathBuf {
    root.join("cards")
}

pub fn is_ignored_name(name: &str) -> bool {
    name.starts_with('.') || name.ends_with(".tmp") || name.ends_with(".json.tmp")
}

pub fn is_conflict_name(name: &str) -> bool {
    name.contains(".sync-conflict-") || name.contains("conflicted copy")
}

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn cards_dir(&self) -> PathBuf {
        cards_dir(&self.root)
    }

    pub fn ensure_cards_dir(&self) -> Result<PathBuf, StoreError> {
        let dir = self.cards_dir();
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn load(&self) -> Result<Vec<Card>, StoreError> {
        let dir = self.cards_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut cards = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if is_ignored_name(name) || is_conflict_name(name) {
                continue;
            }
            let Some(id) = id_from_card_name(name) else {
                continue;
            };
            let text = fs::read_to_string(entry.path())?;
            let Ok(card) = parse_card(&text) else {
                continue;
            };
            if card.id != id {
                continue;
            }
            cards.push(card);
        }
        Ok(cards)
    }

    pub fn save(&self, card: &Card) -> Result<String, StoreError> {
        let dir = self.ensure_cards_dir()?;
        let text = format_card(card)?;
        let final_path = dir.join(format!("{}.json", card.id));
        let tmp_path = dir.join(format!(".{}.json.tmp", card.id));
        fs::write(&tmp_path, &text)?;
        fs::rename(&tmp_path, &final_path)?;
        Ok(text)
    }

    /// Read a conflict copy, keep the newer card under `{id}.json`, and remove the copy.
    pub fn absorb_conflict(&self, path: &Path) -> Result<Option<Card>, StoreError> {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };
        if !is_conflict_name(name) {
            return Ok(None);
        }
        let Some(id) = conflict_id(name) else {
            return Ok(None);
        };
        let text = fs::read_to_string(path)?;
        let Ok(mut incoming) = parse_card(&text) else {
            return Ok(None);
        };
        incoming.id = id;
        let existing = self.load()?.into_iter().find(|card| card.id == id);
        let winner = match existing {
            Some(current) if !crate::card::prefer(&current, &incoming) => current,
            _ => incoming,
        };
        self.save(&winner)?;
        fs::remove_file(path)?;
        Ok(Some(winner))
    }
}

fn id_from_card_name(name: &str) -> Option<Uuid> {
    let stem = name.strip_suffix(".json")?;
    Uuid::parse_str(stem).ok()
}

fn conflict_id(name: &str) -> Option<Uuid> {
    let stem = name.strip_suffix(".json").unwrap_or(name);
    let head = stem
        .split(".sync-conflict-")
        .next()
        .unwrap_or(stem)
        .split(" (")
        .next()
        .unwrap_or(stem);
    Uuid::parse_str(head.trim()).ok()
}

#[derive(Serialize, Deserialize)]
struct CardFile {
    id: Uuid,
    title: String,
    lang: Option<String>,
    anchor: String,
    direction: Direction,
    updated_at: String,
    deleted_at: Option<String>,
    device_id: String,
}

pub fn parse_card(text: &str) -> Result<Card, StoreError> {
    let file: CardFile =
        serde_json::from_str(text).map_err(|e| StoreError::Format(e.to_string()))?;
    Ok(Card {
        id: file.id,
        title: file.title,
        lang: file.lang,
        anchor: Date::parse(&file.anchor, DATE).map_err(|e| StoreError::Format(e.to_string()))?,
        direction: file.direction,
        updated_at: OffsetDateTime::parse(&file.updated_at, &Rfc3339)
            .map_err(|e| StoreError::Format(e.to_string()))?,
        deleted_at: match file.deleted_at {
            Some(value) => Some(
                OffsetDateTime::parse(&value, &Rfc3339)
                    .map_err(|e| StoreError::Format(e.to_string()))?,
            ),
            None => None,
        },
        device_id: file.device_id,
    })
}

pub fn format_card(card: &Card) -> Result<String, StoreError> {
    let file = CardFile {
        id: card.id,
        title: card.title.clone(),
        lang: card.lang.clone(),
        anchor: card
            .anchor
            .format(DATE)
            .map_err(|e| StoreError::Format(e.to_string()))?,
        direction: card.direction,
        updated_at: card
            .updated_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::Format(e.to_string()))?,
        deleted_at: match &card.deleted_at {
            Some(value) => Some(
                value
                    .format(&Rfc3339)
                    .map_err(|e| StoreError::Format(e.to_string()))?,
            ),
            None => None,
        },
        device_id: card.device_id.clone(),
    };
    serde_json::to_string_pretty(&file).map_err(|e| StoreError::Format(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::prefer;
    use time::Month;

    fn sample(id: &str, title: &str, deleted: bool) -> Card {
        Card {
            id: Uuid::parse_str(id).unwrap(),
            title: title.to_owned(),
            lang: None,
            anchor: Date::from_calendar_date(2026, Month::November, 3).unwrap(),
            direction: Direction::Auto,
            updated_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
            deleted_at: deleted
                .then(|| OffsetDateTime::from_unix_timestamp(1_700_000_100).unwrap()),
            device_id: "device-a".to_owned(),
        }
    }

    #[test]
    fn roundtrip_keeps_a_tombstone() {
        let dir = std::env::temp_dir().join(format!("dayshelf-{}", Uuid::new_v4()));
        let store = Store::new(&dir);
        let card = sample("11111111-1111-1111-1111-111111111111", "Graduation", true);
        let text = store.save(&card).unwrap();
        assert!(text.contains("\"deleted_at\""));
        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_deleted());
        assert_eq!(loaded[0].title, "Graduation");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ignores_temp_files_and_keeps_the_newer_conflict() {
        let dir = std::env::temp_dir().join(format!("dayshelf-{}", Uuid::new_v4()));
        let store = Store::new(&dir);
        let id = "22222222-2222-2222-2222-222222222222";
        let mut current = sample(id, "Old", false);
        current.device_id = "aaaa".to_owned();
        store.save(&current).unwrap();

        let mut incoming = sample(id, "New", false);
        incoming.updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_050).unwrap();
        incoming.device_id = "bbbb".to_owned();
        let conflict = store
            .cards_dir()
            .join(format!("{id}.sync-conflict-20261103-120000-PHONE.json"));
        fs::write(&conflict, format_card(&incoming).unwrap()).unwrap();
        fs::write(store.cards_dir().join(format!(".{id}.json.tmp")), "{").unwrap();

        let winner = store.absorb_conflict(&conflict).unwrap().unwrap();
        assert_eq!(winner.title, "New");
        assert!(prefer(&current, &incoming));
        assert!(!conflict.exists());
        assert_eq!(
            store
                .load()
                .unwrap()
                .iter()
                .filter(|card| !card.is_deleted())
                .count(),
            1
        );
        let _ = fs::remove_dir_all(dir);
    }
}
