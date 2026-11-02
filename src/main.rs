// Pas de fenêtre console en version release sur Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod capture;
mod config;
mod convert;

use eframe::egui::{self, ColorImage};
use std::sync::Arc;

static LOGO_PNG: &[u8] = include_bytes!("../assets/LOGO.png");

fn main() -> eframe::Result<()> {
    let cfg = config::load();

    // Initialise Media Foundation une fois sur le thread principal.
    let _ = capture::list_devices();

    let logo = image::load_from_memory(LOGO_PNG).ok().map(|i| i.to_rgba8());
    let icon = logo.as_ref().map(|i| {
        Arc::new(egui::IconData {
            rgba: i.as_raw().clone(),
            width: i.width(),
            height: i.height(),
        })
    });
    let logo_img = logo.map(|i| {
        ColorImage::from_rgba_unmultiplied([i.width() as usize, i.height() as usize], i.as_raw())
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("HDMI Splitscreen")
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([320.0, 180.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        vsync: cfg.vsync,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    let result = eframe::run_native(
        "HDMI Splitscreen",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, cfg, logo_img)))),
    );
    // Quitte tout de suite même si une carte de capture bloque son thread.
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}
