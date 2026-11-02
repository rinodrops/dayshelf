use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use time::{Month, OffsetDateTime, Weekday};
use uuid::Uuid;

use crate::calendar::{add_months, month_cells, parse_span, visible_months};
use crate::card::{sort_cards, Card, Direction};
use crate::config::{load_config, load_device_id, save_config, LocalConfig};
use crate::days::{signed_days, CountKind};
use crate::store::{is_conflict_name, is_ignored_name, Store};
use crate::watch::FolderWatch;

pub struct DayShelfApp {
    config: LocalConfig,
    device_id: String,
    cards: Vec<Card>,
    message: String,
    editor: Option<Card>,
    anchor_text: String,
    view_year: i32,
    view_month: u8,
    watch: Option<FolderWatch>,
    watched: Option<PathBuf>,
    self_writes: HashMap<PathBuf, String>,
}

impl DayShelfApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let config = load_config();
        let device_id = load_device_id();
        let cards = visible_cards(&config);
        let today = today();
        Self {
            config,
            device_id,
            cards,
            message: String::new(),
            editor: None,
            anchor_text: String::new(),
            view_year: today.year(),
            view_month: today.month() as u8,
            watch: None,
            watched: None,
            self_writes: HashMap::new(),
        }
    }

    fn open_new_on(&mut self, anchor: time::Date) {
        self.anchor_text = anchor.to_string();
        self.editor = Some(Card {
            id: Uuid::new_v4(),
            title: String::new(),
            lang: None,
            anchor,
            direction: Direction::Auto,
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
            device_id: self.device_id.clone(),
        });
    }

    fn open_new(&mut self) {
        self.open_new_on(today());
    }

    fn open_edit(&mut self, card: Card) {
        self.anchor_text = card.anchor.to_string();
        self.editor = Some(card);
    }

    fn save_editor(&mut self) {
        let Some(folder) = self.config.folder().map(|path| path.to_path_buf()) else {
            return;
        };
        let Some(card) = self.editor.as_mut() else {
            return;
        };
        let Ok(anchor) = time::Date::parse(
            &self.anchor_text,
            &time::macros::format_description!("[year]-[month]-[day]"),
        ) else {
            self.message = "Use a date like 2026-11-03.".to_owned();
            return;
        };
        card.anchor = anchor;
        card.updated_at = OffsetDateTime::now_utc();
        card.device_id = self.device_id.clone();
        card.deleted_at = None;
        let store = Store::new(&folder);
        match store.save(card) {
            Ok(text) => {
                let path = store.cards_dir().join(format!("{}.json", card.id));
                self.self_writes.insert(path, text);
                self.editor = None;
                self.message.clear();
                self.reload();
            }
            Err(error) => self.message = format!("{error:?}"),
        }
    }

    fn delete_card(&mut self, mut card: Card) {
        let Some(folder) = self.config.folder().map(|path| path.to_path_buf()) else {
            return;
        };
        card.deleted_at = Some(OffsetDateTime::now_utc());
        card.updated_at = OffsetDateTime::now_utc();
        card.device_id = self.device_id.clone();
        let store = Store::new(&folder);
        match store.save(&card) {
            Ok(text) => {
                let path = store.cards_dir().join(format!("{}.json", card.id));
                self.self_writes.insert(path, text);
            }
            Err(error) => {
                self.message = format!("{error:?}");
                return;
            }
        }
        if self.editor.as_ref().is_some_and(|open| open.id == card.id) {
            self.editor = None;
        }
        self.reload();
    }

    fn choose_folder(&mut self, create: bool) {
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        if let Err(error) = prepare_folder(&folder, create) {
            self.message = error;
            return;
        }
        self.config.sync_folder = folder.display().to_string();
        if let Err(error) = save_config(&self.config) {
            self.message = error.to_string();
            return;
        }
        self.message.clear();
        self.reload();
    }

    fn reload(&mut self) {
        self.cards = visible_cards(&self.config);
    }

    fn poll_folder(&mut self) {
        let wanted = self.config.folder().map(crate::store::cards_dir);
        if self.watched.as_ref() != wanted.as_ref() {
            self.watch = wanted
                .as_ref()
                .and_then(|dir| FolderWatch::open(dir.clone()).ok());
            self.watched = wanted;
        }
        let Some(watch) = &self.watch else {
            return;
        };
        let paths = watch.poll();
        if paths.is_empty() {
            return;
        }
        let Some(folder) = self.config.folder().map(|path| path.to_path_buf()) else {
            return;
        };
        let store = Store::new(&folder);
        let mut changed = false;
        for path in paths {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if is_ignored_name(name) {
                continue;
            }
            if is_conflict_name(name) {
                let _ = store.absorb_conflict(&path);
                changed = true;
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                if self.self_writes.get(&path) == Some(&text) {
                    self.self_writes.remove(&path);
                    continue;
                }
            }
            changed = true;
        }
        if changed {
            self.reload();
        }
    }
}

impl eframe::App for DayShelfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_folder();
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.config.folder().is_none() {
                ui.heading("DayShelf");
                ui.label(
                    "Choose a folder for the cards. A cloud-storage folder works the same way.",
                );
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {
                        self.choose_folder(true);
                    }
                    if ui.button("Existing").clicked() {
                        self.choose_folder(false);
                    }
                });
                if !self.message.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.message);
                }
                return;
            }

            ui.horizontal(|ui| {
                ui.heading("DayShelf");
                if ui.button("New card").clicked() {
                    self.open_new();
                }
                if ui.button("Change folder").clicked() {
                    self.config.sync_folder.clear();
                    let _ = save_config(&self.config);
                    self.cards.clear();
                    self.editor = None;
                }
            });
            if !self.message.is_empty() {
                ui.label(&self.message);
            }
            if let Some(card) = self.editor.clone() {
                self.editor_ui(ui, card);
                return;
            }
            if self.cards.is_empty() {
                ui.label("No cards yet.");
            }
            let mut action: Option<(Uuid, bool)> = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                let width = ui.available_width();
                let card_w = 200.0;
                let cols = ((width / card_w).floor() as usize).max(1);
                egui::Grid::new("cards")
                    .num_columns(cols)
                    .spacing([12.0, 12.0])
                    .show(ui, |ui| {
                        for (index, card) in self.cards.iter().enumerate() {
                            if let Some(delete) = card_face(ui, card) {
                                action = Some((card.id, delete));
                            }
                            if (index + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
            if let Some((id, delete)) = action {
                if let Some(card) = self.cards.iter().find(|card| card.id == id).cloned() {
                    if delete {
                        self.delete_card(card);
                    } else {
                        self.open_edit(card);
                    }
                }
            }
            self.calendar_ui(ui);
        });
    }
}

impl DayShelfApp {
    fn calendar_ui(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("◀").clicked() {
                if let Some((year, month)) = add_months(self.view_year, self.view_month, -1) {
                    self.view_year = year;
                    self.view_month = month;
                }
            }
            if ui.button("▶").clicked() {
                if let Some((year, month)) = add_months(self.view_year, self.view_month, 1) {
                    self.view_year = year;
                    self.view_month = month;
                }
            }
            ui.label(format!("{:04}-{:02}", self.view_year, self.view_month));
        });
        let marked: HashSet<_> = self.cards.iter().map(|card| card.anchor).collect();
        let months = visible_months(
            (self.view_year, self.view_month),
            self.config.month_count,
            parse_span(&self.config.span),
        );
        let mut create_on = None;
        ui.horizontal(|ui| {
            for (year, month) in months {
                create_on = create_on.or(month_grid(ui, year, month, &marked));
            }
        });
        if let Some(date) = create_on {
            self.open_new_on(date);
        }
    }
}

impl DayShelfApp {
    fn editor_ui(&mut self, ui: &mut egui::Ui, mut card: Card) {
        ui.label("Title");
        ui.text_edit_singleline(&mut card.title);
        ui.label("Anchor date");
        ui.text_edit_singleline(&mut self.anchor_text);
        ui.label("Direction");
        egui::ComboBox::from_id_salt("direction")
            .selected_text(direction_label(card.direction))
            .show_ui(ui, |ui| {
                for direction in [Direction::Auto, Direction::Countdown, Direction::Countup] {
                    ui.selectable_value(&mut card.direction, direction, direction_label(direction));
                }
            });
        let mut save = false;
        let mut cancel = false;
        let mut delete = false;
        ui.horizontal(|ui| {
            save = ui.button("Save").clicked();
            cancel = ui.button("Cancel").clicked();
            delete = ui
                .add(egui::Button::new("Delete").fill(egui::Color32::from_rgb(180, 40, 40)))
                .clicked();
        });
        self.editor = Some(card.clone());
        if save {
            self.save_editor();
        } else if cancel {
            self.editor = None;
            self.message.clear();
        } else if delete {
            self.delete_card(card);
        }
    }
}

fn today() -> time::Date {
    OffsetDateTime::now_local()
        .map(|time| time.date())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().date())
}

fn month_grid(
    ui: &mut egui::Ui,
    year: i32,
    month: u8,
    marked: &HashSet<time::Date>,
) -> Option<time::Date> {
    let mut create_on = None;
    ui.vertical(|ui| {
        let name = Month::try_from(month)
            .map(|month| month.to_string())
            .unwrap_or_default();
        ui.label(format!("{name} {year}"));
        ui.horizontal(|ui| {
            for label in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
                ui.add_sized([28.0, 16.0], egui::Label::new(label));
            }
        });
        let cells = month_cells(year, month, Weekday::Monday);
        for week in cells.chunks(7) {
            ui.horizontal(|ui| {
                for date in week {
                    let in_month = date.month() as u8 == month;
                    let mut text = date.day().to_string();
                    if marked.contains(date) {
                        text.push('*');
                    }
                    let mut button = egui::Button::new(text);
                    if !in_month {
                        button = button.frame(false);
                    }
                    let response = ui.add_sized([28.0, 24.0], button);
                    response.context_menu(|ui| {
                        if ui.button("New card").clicked() {
                            create_on = Some(*date);
                            ui.close_menu();
                        }
                    });
                }
            });
        }
    });
    create_on
}

fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::Auto => "Auto",
        Direction::Countdown => "Countdown",
        Direction::Countup => "Countup",
    }
}

/// Returns `Some(true)` to delete and `Some(false)` to edit.
fn card_face(ui: &mut egui::Ui, card: &Card) -> Option<bool> {
    let today = OffsetDateTime::now_local()
        .map(|time| time.date())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().date());
    let (days, kind) = signed_days(today, card.anchor, card.direction);
    let kind_label = match kind {
        CountKind::Countdown => "Countdown",
        CountKind::Countup => "Countup",
        CountKind::Today => "Today",
    };
    let mut action = None;
    let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_size(egui::vec2(188.0, 120.0));
        ui.set_max_width(188.0);
        ui.label(if card.title.is_empty() {
            "Untitled"
        } else {
            &card.title
        });
        ui.heading(days.to_string());
        ui.label(kind_label);
        ui.label(card.anchor.to_string());
    });
    frame.response.context_menu(|ui| {
        if ui.button("Edit").clicked() {
            action = Some(false);
            ui.close_menu();
        }
        if ui.button("Delete").clicked() {
            action = Some(true);
            ui.close_menu();
        }
    });
    action
}

fn visible_cards(config: &LocalConfig) -> Vec<Card> {
    let Some(folder) = config.folder() else {
        return Vec::new();
    };
    let mut cards = Store::new(folder)
        .load()
        .unwrap_or_default()
        .into_iter()
        .filter(|card| !card.is_deleted())
        .collect::<Vec<_>>();
    sort_cards(&mut cards, config.cards_order);
    cards
}

fn prepare_folder(folder: &std::path::Path, create: bool) -> Result<(), String> {
    let cards = crate::store::cards_dir(folder);
    if create {
        std::fs::create_dir_all(&cards).map_err(|error| error.to_string())?;
    } else if !cards.is_dir() {
        return Err("That folder has no cards directory.".to_owned());
    }
    Ok(())
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in system_font_candidates() {
        if let Ok(bytes) = std::fs::read(&path) {
            fonts
                .font_data
                .insert(path.clone(), egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(path.clone());
            }
        }
    }
    ctx.set_fonts(fonts);
}

fn system_font_candidates() -> Vec<String> {
    [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
        "C:/Windows/Fonts/msgothic.ttc",
        "C:/Windows/Fonts/arial.ttf",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
