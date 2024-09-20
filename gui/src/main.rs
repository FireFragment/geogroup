#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::*;
use egui::*;
use glow::RED;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::<App>::default())),
    )
}

enum App {
    WelcomePage(),
}

impl Default for App {
    fn default() -> Self {
        Self::WelcomePage()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui_extras::StripBuilder::new(ui)
                .size(egui_extras::Size::remainder())
                .size(egui_extras::Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                            ui.label("Locate your photo collection to get started");
                            ui.heading("Welcome to Geogroup");
                        });
                    });

                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(16.0);
                            ui.button("🗁 Pick folder");
                        });
                    });
                });
        });
    }
}
