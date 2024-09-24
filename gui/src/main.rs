#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use egui_inbox::UiInbox;
use std::path::PathBuf;

use eframe::*;
use egui::*;
use geogroup_backend as backend;
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

enum Message {
    SetContent(AppContent),
}

#[derive(Default)]
struct App {
    content: AppContent,
    inbox: UiInbox<Message>,
}

enum AppContent {
    WelcomePage(WelcomePage),
    MainPage {
        pane: PaneContent,
        hiearchy: backend::HiearchyItem<PathBuf>,
        flatten_mode: Option<FlattenMode>,
    },
}

enum PaneContent {
    Flatten,
    LoadData,
}

#[derive(PartialEq, Eq, Clone)]
enum FlattenMode {
    Flatten,
    OnlyRoot,
}

enum WelcomePage {
    Normal,
    Loading(String),
    Error(String),
}

impl Default for AppContent {
    fn default() -> Self {
        Self::WelcomePage(WelcomePage::Normal)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.inbox.set_ctx(ctx);
        for message in self.inbox.read_without_ctx() {
            message.perform(self);
        }

        match &mut self.content {
            AppContent::WelcomePage(welcome_page) => welcome_page.draw(ctx, &self.inbox),

            AppContent::MainPage {
                hiearchy,
                pane,
                flatten_mode,
            } => {
                egui::SidePanel::left("the wizard pane").show(ctx, |ui| match pane {
                    PaneContent::Flatten => {
                        ui.heading("Step 1: Subdirectories");
                        ui.label("In the directory you've chosen, there are some subdirectories. Please choose what to do with files in those subdirectories");

                        ui.radio_value(flatten_mode, Some(FlattenMode::Flatten), "Use all files");
                        ui.radio_value(flatten_mode, Some(FlattenMode::OnlyRoot), "Skip files in subdirectories");

                        ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                            let clicked = ui.add_enabled(flatten_mode.is_some(), Button::new("Next")).clicked();

                            if clicked {
                                *pane = PaneContent::LoadData;
                            }
                        });
                    },
                    PaneContent::LoadData => {
                        ui.heading("Step 2: Loading data");

                        ui.horizontal(|ui| {ui.spinner(); ui.label("Loading data from files...")});
                    },
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.centered_and_justified(|ui| show_hiearchy(ui, hiearchy, flatten_mode))
                });
            }
        }
    }
}

impl Message {
    pub fn perform(self, app: &mut App) {
        match self {
            Message::SetContent(content) => app.content = content,
        }
    }
}

impl WelcomePage {
    fn draw(&mut self, ctx: &egui::Context, inbox: &UiInbox<Message>) {
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

                            // I can't use error from pattern match directly because of single-mut rule :(
                            /*if let Self::WelcomePage {
                                error: Some(ref error),
                            } = *self
                            {
                                error_ui(ui, error);
                            }*/

                            if matches!(self, Self::Normal | Self::Error(_)) {
                                self.pick_file_btn(ui, inbox);
                            };

                            match self {
                                Self::Normal => (),
                                Self::Loading(msg) => {
                                    ui.spinner();
                                    ui.label(&*msg);
                                }
                                Self::Error(err) => {
                                    error_ui(ui, err);
                                }
                            }
                        });
                    });
                });
        });
    }

    fn pick_file_btn(&mut self, ui: &mut Ui, inbox: &UiInbox<Message>) {
        let clicked = ui.button("🗁 Pick folder").clicked();
        if clicked {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let sender = inbox.sender();

                std::thread::spawn(move || {
                    let res = backend::load_directory(folder);
                    // Send will return an error if the receiver has been dropped
                    // but unless you have a long running task that will send multiple messages
                    // you can just ignore the error
                    sender
                        .send(Message::SetContent(match res {
                            Ok(hiearchy) => AppContent::MainPage {
                                hiearchy,
                                pane: PaneContent::Flatten,
                                flatten_mode: None,
                            },
                            Err(err) => {
                                AppContent::WelcomePage(WelcomePage::Error(err.to_string()))
                            }
                        }))
                        .ok()
                });

                *self = Self::Loading("Loading files".into());
            }
        };
    }
}

fn show_hiearchy(
    ui: &mut Ui,
    hiearchy: &backend::HiearchyItem<PathBuf>,
    flatten_mode: &Option<FlattenMode>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered_justified(|ui| show_hiearchy_inner(ui, hiearchy, flatten_mode, true));
    });
}

fn show_hiearchy_inner(
    ui: &mut Ui,
    hiearchy: &backend::HiearchyItem<PathBuf>,
    flatten_mode: &Option<FlattenMode>,
    root: bool,
) {
    match hiearchy {
        geogroup_backend::HiearchyItem::Group(group) => {
            match flatten_mode {
                None | Some(FlattenMode::Flatten) => {
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        Id::with(ui.id(), "hiearchy_dir_collapsing"),
                        true,
                    )
                    .show_header(ui, |ui| {
                        hiearchy_row(ui, "Directory");
                    })
                    .body(|ui| {
                        for (id, item) in group.iter().enumerate() {
                            ui.push_id(id, |ui| show_hiearchy_inner(ui, item, flatten_mode, false));
                        }
                    });
                }
                Some(FlattenMode::OnlyRoot) => {
                    if root {
                        for (id, item) in group.iter().enumerate() {
                            ui.push_id(id, |ui| show_hiearchy_inner(ui, item, flatten_mode, false));
                        }
                    }
                }
            };
        }
        geogroup_backend::HiearchyItem::Item(path) => {
            hiearchy_row(ui, path.file_name().unwrap().to_str().unwrap());
        }
    }
}

fn hiearchy_row(ui: &mut Ui, item: &str) {
    ui.horizontal(|ui| {
        ui.selectable_label(false, item);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 0.0),
            Layout::right_to_left(Align::Center),
            |ui| ui.button("Dissolve"),
        );
    });
}

fn error_ui(ui: &mut Ui, error: &str) {
    ui.colored_label(ui.visuals().error_fg_color, format!("⊗ {}", error));
}
