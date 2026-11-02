use time::OffsetDateTime;

use crate::card::{sort_cards, Card};
use crate::config::{load_config, load_device_id, save_config, LocalConfig};
use crate::days::{signed_days, CountKind};
use crate::store::Store;

pub struct DayShelfApp {
    config: LocalConfig,
    #[allow(dead_code)]
    device_id: String,
    cards: Vec<Card>,
    message: String,
}

impl DayShelfApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let config = load_config();
        let device_id = load_device_id();
        let cards = visible_cards(&config);
        Self {
            config,
            device_id,
            cards,
            message: String::new(),
        }
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
}

impl eframe::App for DayShelfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                if ui.button("Change folder").clicked() {
                    self.config.sync_folder.clear();
                    let _ = save_config(&self.config);
                    self.cards.clear();
                }
            });
            if !self.message.is_empty() {
                ui.label(&self.message);
            }
            if self.cards.is_empty() {
                ui.label("No cards yet.");
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                let width = ui.available_width();
                let card_w = 200.0;
                let cols = ((width / card_w).floor() as usize).max(1);
                egui::Grid::new("cards")
                    .num_columns(cols)
                    .spacing([12.0, 12.0])
                    .show(ui, |ui| {
                        for (index, card) in self.cards.iter().enumerate() {
                            card_face(ui, card);
                            if (index + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });
    }
}

fn card_face(ui: &mut egui::Ui, card: &Card) {
    let today = OffsetDateTime::now_local()
        .map(|time| time.date())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().date());
    let (days, kind) = signed_days(today, card.anchor, card.direction);
    let kind_label = match kind {
        CountKind::Countdown => "Countdown",
        CountKind::Countup => "Countup",
        CountKind::Today => "Today",
    };
    egui::Frame::group(ui.style()).show(ui, |ui| {
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
