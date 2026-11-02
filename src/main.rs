fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([960.0, 640.0])
        .with_title("DayShelf");
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "DayShelf",
        options,
        Box::new(|cc| Ok(Box::new(dayshelf::ui::DayShelfApp::new(cc)))),
    )
}
