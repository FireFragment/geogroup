#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use egui_inbox::UiInbox;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<App>::default())
        }),
    )
}

type Hiearchy = Vec<backend::HiearchyItem<PathBuf, String>>;

enum Message {
    SetContent(AppContent),
    SetHiearchy(Hiearchy),
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
        src_dir: PathBuf,
        hiearchy: Hiearchy,
        flatten_mode: Option<FlattenMode>,
        selection: Vec<usize>,
        image_scale: u16,
    },
}

enum PaneContent {
    Sort,
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
        for message in self.inbox.read_without_ctx() {
            message.perform(self);
        }

        match &mut self.content {
            AppContent::WelcomePage(welcome_page) => welcome_page.draw(ctx, &self.inbox),

            AppContent::MainPage {
                hiearchy,
                pane,
                src_dir,
                flatten_mode,
                selection,
                image_scale,
            } => {
                egui::TopBottomPanel::bottom("the wizard pane").show(ctx, |ui| match pane {
                    PaneContent::Sort => {
                        ui.heading("Sort files");

                        ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                            let clicked = ui.add(Button::new("Sort")).clicked();

                            if clicked {
                                let sender = self.inbox.sender();
                                let cloned_src_dir = src_dir.clone();

                                std::thread::spawn(move || {
                                    let sorted = backend::sort_from_fs_to_mem(&cloned_src_dir);
                                    sender.send(Message::SetHiearchy(vec![sorted.map_group_data(&|_| String::from("Group"))]))
                                });
                            }
                        });
                    }
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("View", |ui| {
                            egui::Slider::new(image_scale, 32..=128)
                                .text("Image preview height")
                                .ui(ui);
                        });
                    });

                    show_hiearchy(ui, hiearchy, selection, flatten_mode, *image_scale)
                });
            }
        }
    }
}

impl Message {
    pub fn perform(self, app: &mut App) {
        match self {
            Message::SetContent(content) => app.content = content,
            Message::SetHiearchy(hiearchy_to_set) => {
                if let AppContent::MainPage {
                    pane: _,
                    src_dir: _,
                    ref mut hiearchy,
                    flatten_mode: _,
                    ref mut selection,
                    image_scale: _,
                } = app.content
                {
                    *selection = Vec::new();
                    *hiearchy = hiearchy_to_set;
                } else { /* TODO: Warning */
                }
            }
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
                    use geogroup_backend::HiearchyItem as HI;
                    let res = backend::load_directory(&folder);
                    // Send will return an error if the receiver has been dropped
                    // but unless you have a long running task that will send multiple messages
                    // you can just ignore the error
                    sender
                        .send(Message::SetContent(match res {
                            Ok(HI::Group(hiearchy, _)) => AppContent::MainPage {
                                hiearchy: hiearchy
                                    .into_iter()
                                    .map(|h| {
                                        h.map_group_data(&|path: PathBuf| {
                                            path.file_name()
                                                .map(|path| path.to_str())
                                                .flatten()
                                                .unwrap_or("[invalid filename]")
                                                .into()
                                        })
                                    })
                                    .collect(),
                                pane: PaneContent::Sort,
                                src_dir: folder,
                                flatten_mode: None,
                                selection: Vec::new(),
                                image_scale: 48,
                            },
                            Ok(HI::Item(_)) => AppContent::WelcomePage(WelcomePage::Error(
                                String::from("Please choose a directory"),
                            )),
                            Err(err) => {
                                AppContent::WelcomePage(WelcomePage::Error(err.to_string()))
                            }
                        }))
                        .ok();
                });

                *self = Self::Loading("Loading files".into());
            }
        };
    }
}

fn show_hiearchy(
    ui: &mut Ui,
    hiearchy: &Hiearchy,
    selected_vec: &mut Vec<usize>,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
) {
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal_centered(|ui| {
            show_hiearchy_inner(ui, hiearchy, selected_vec, 0, flatten_mode, image_scale)
        });
    });
}

fn show_hiearchy_inner(
    ui: &mut Ui,
    hiearchy: &Hiearchy,
    selected_vec: &mut Vec<usize>,
    current_depth: usize,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
) {
    assert!(selected_vec.len() >= current_depth);

    use egui_extras::{Column, TableBuilder};

    let selected_group = if let Some(selection_idx) = selected_vec.get(current_depth) {
        if let geogroup_backend::HiearchyItem::Group(g, _) = &hiearchy[*selection_idx] {
            Some(g)
        } else {
            None
        }
    } else {
        None
    };

    ui.push_id(current_depth, |ui| {
        TableBuilder::new(ui)
            .column(if selected_group.is_some() {
                Column::exact(256.0)
            } else {
                Column::remainder()
            })
            .sense(Sense::click())
            .body(|body| {
                body.heterogeneous_rows(
                    hiearchy.iter().map(|item| {
                        use geogroup_backend::HiearchyItem as HI;
                        match item {
                            HI::Group(_, _) => 16.0,
                            HI::Item(_) => image_scale as f32,
                        }
                    }),
                    |mut row| {
                        let idx = row.index();

                        row.set_selected(
                            selected_vec
                                .get(current_depth)
                                .map(|s| *s == row.index())
                                .unwrap_or(false),
                        );

                        row.col(|ui| {
                            match &hiearchy[idx] {
                                geogroup_backend::HiearchyItem::Group(_, name) => {
                                    ui.add(
                                        Label::new(format!(
                                            "🗁 {name}"
                                        ))
                                        .selectable(false),
                                    );
                                }
                                geogroup_backend::HiearchyItem::Item(path) => {
                                    ui.horizontal_top(|ui| {
                                        if let Some(file_path) = path.to_str() {
                                            Image::new(format!("file://{file_path}")).ui(ui);
                                        }

                                        ui.add(
                                            Label::new(
                                                path.file_name()
                                                    .unwrap_or(OsStr::new("[invalid filename]"))
                                                    .to_str()
                                                    .unwrap_or("[invalid filename]"),
                                            )
                                            .selectable(false),
                                        )
                                    });
                                }
                            };
                        });

                        if row.response().clicked() {
                            if let Some(selection_idx) = selected_vec.get_mut(current_depth) {
                                *selection_idx = idx;
                                selected_vec.truncate(current_depth + 1);
                            } else {
                                selected_vec.push(idx);
                            }
                        }
                    },
                );
            })
    });

    if let Some(g) = selected_group {
        show_hiearchy_inner(
            ui,
            g,
            selected_vec,
            current_depth + 1,
            flatten_mode,
            image_scale,
        )
    }
}

fn error_ui(ui: &mut Ui, error: &str) {
    ui.colored_label(ui.visuals().error_fg_color, format!("⊗ {}", error));
}
