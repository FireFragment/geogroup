#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod action;
mod gui;
mod hiearchy;
//mod long_running_container;
mod style;
mod tabbar;
use backend::gg_prelude::*;
use gui::Message;
use hiearchy::TemplateHiearchy;
use tabbar::tabbar;

use backend::gg_prelude::*;
use clap::Parser;
use egui_extras::Size;
use egui_inbox::UiInbox;
use std::{
    ffi::OsStr,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    path::{Path, PathBuf},
};

use eframe::{egui, emath, glow};
use egui::Frame;
use geogroup_backend::{self as backend, loaders::DataLoader as _, naming::RevGeocoder};
use glow::{FALSE, RED};

#[derive(clap::Parser)]
#[command(name = "geogroup", version, about)]
struct CliArgs {
    /// Directory that should be opened on startup.
    /// If not specified, user will be prompted to choose directory in a GUI.
    #[arg(value_hint = clap::ValueHint::DirPath)]
    dir: Option<PathBuf>,

    /// Directory to cache reverse-geocoded locations in
    #[arg(
        long,
        env = "GEOGROUP_GEOCODING_CACHE_DIR",
        default_value = "/tmp/geogroup_reverse_geocoding_cache" // TODO: What on other platforms than linux?
    )]
    geocoding_cache: PathBuf,
    /* Clap doesn't support env-only arguments, see https://github.com/clap-rs/clap/discussions/5432
    /// API key for [Opencage](https://opencagedata.com/) used for reverse geocoding
    #[arg(env = "OPENCAGE_API_KEY")]
    opencage_api_key: Option<String>,
    */
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    if let Err(error) = dotenv::dotenv() {
        log::warn!("Failed to load .env file: {error}\nDeveloper info: {error:?}")
    }

    let args = CliArgs::parse();
    if !args.geocoding_cache.exists() {
        std::fs::create_dir_all(&args.geocoding_cache).unwrap();
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let app = gui::App::new(args);

    eframe::run_native(
        "Geogroup",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            style::apply(&cc.egui_ctx, &app.style_params);
            Ok(Box::new(app))
        }),
    )
}

pub fn filename_to_string(os_str: Option<&OsStr>) -> String {
    os_str
        .map(|s| s.to_string_lossy().deref().to_string())
        .unwrap_or(String::from("Invalid filename"))
}

impl CliArgs {
    pub fn get_geocoder(&self) -> RevGeocoder<'static> {
        RevGeocoder::from_env_key(self.geocoding_cache.to_owned())
    }
}
