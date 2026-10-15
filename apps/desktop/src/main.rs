//! Точка входа desktop-приложения karta.

use anyhow::Result;

fn main() -> Result<()> {
    // Инициализация логирования (уровень можно задать через RUST_LOG).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Запуск karta desktop…");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("karta — карта для туристов"),
        ..Default::default()
    };

    eframe::run_native(
        "karta — карта для туристов",
        options,
        Box::new(|cc| Ok(Box::new(karta_desktop::app::KartaApp::new(cc)))),
    )?;

    Ok(())
}
