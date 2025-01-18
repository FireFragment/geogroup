#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod style;
mod tabbar;
use tabbar::tabbar;

use clap::Parser;
use egui_extras::Size;
use egui_inbox::UiInbox;
use egui_transition_animation::{animated_pager, TransitionStyle};
use std::{
    ffi::OsStr,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, RangeInclusive},
    path::{Path, PathBuf},
    thread,
};

use eframe::*;
use egui::Frame;
use egui::*;
use geogroup_backend::{
    self as backend, loaders::DataLoader as _, naming::RevGeocoder, HiearchyItem,
};
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

    let args = CliArgs::parse();
    if !args.geocoding_cache.exists() {
        std::fs::create_dir_all(&args.geocoding_cache).unwrap();
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let mut app = App {
        content: AppContent::default(),
        inbox: UiInbox::default(),
        style_params: Default::default(),
        style_changed: false,
        args,
    };

    let AppContent::WelcomePage(ref mut welcome_page) = app.content else {
        panic!(
            "App::default().content has is not a welcome page, it's {:?}",
            app.content
        )
    };

    if let Some(ref dir) = app.args.dir {
        welcome_page.action_load_dir(&app.inbox, dir.to_owned());
    }

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

type FileTime = chrono::DateTime<chrono::FixedOffset>;

#[derive(Debug, Clone)]
pub struct FileInHiearchy {
    pub name: String,
    pub path: PathBuf,
    pub pos: Option<backend::geo_lib::Point>,
    pub date: Option<FileTime>,
}

//impl From<FileInHiearchy> for backend::algorithm::SortItem<backend::geo_lib::Point, > {}

impl FileInHiearchy {
    pub fn transform_for_sorting(
        self,
    ) -> Option<(backend::geo_lib::Point, (String, PathBuf, FileTime))> {
        self.pos
            .and_then(|pos| self.date.map(|date| (pos, (self.name, self.path, date))))
    }

    pub fn transform_after_sorting(
        (pos, (name, path, date)): (backend::geo_lib::Point, (String, PathBuf, FileTime)),
    ) -> Self {
        Self {
            name,
            path,
            pos: Some(pos),
            date: Some(date),
        }
    }
}

type Hiearchy = Vec<backend::HiearchyItem<FileInHiearchy, String>>;

#[derive(Debug)]
enum Message {
    SetContent(AppContent),
    Sorted { new_hiearchy: Hiearchy },
    SetProgress(Option<Progress>),
}

struct App {
    content: AppContent,
    inbox: UiInbox<Message>,
    args: CliArgs,
    style_params: style::Params,
    style_changed: bool,
}

#[derive(Debug)]
enum AppContent {
    WelcomePage(WelcomePage),
    MainPage(MainPage),
}

#[derive(Debug)]
struct MainPage {
    pane: Option<PaneContent>,
    src_dir: PathBuf,
    hiearchy: Hiearchy,
    flatten_mode: Option<FlattenMode>,
    selection: Vec<usize>,
    image_scale: u16,
    progress: Option<Progress>,
    /// Config controlling the entire operation, including sorting, naming, etc.
    operation_config: backend::SortingCfg,
    auto_sort: bool,
    /// Whether sorting should be rerun once `sort_process` completes.
    /// This may happen when user changes configuration during sorting. In that case, the already running `sort_process` uses outdated configuration.
    sort_pending: bool,
    sort_process: Option<thread::JoinHandle<()>>,
}

#[derive(Debug)]
struct Progress {
    pub action: ProgressAction,
    pub progress: Option<u16>,
}

#[derive(Debug)]
enum ProgressAction {
    Geocoding,
    Grouping,
    Naming,

    Applying,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum PaneContent {
    Grouping,
    Naming,
    ManualEdit,
    Apply,
    Home,
    View,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum FlattenMode {
    Flatten,
    OnlyRoot,
}

#[derive(Debug)]
enum WelcomePage {
    Normal,
    Loading(String),
    Error(String),
}

pub fn filename_to_string(os_str: Option<&OsStr>) -> String {
    os_str
        .map(|s| s.to_string_lossy().deref().to_string())
        .unwrap_or(String::from("Invalid filename"))
}

impl Default for AppContent {
    fn default() -> Self {
        Self::WelcomePage(WelcomePage::Normal)
    }
}

fn ribbon_slider<Num: emath::Numeric>(
    ui: &mut Ui,
    value: &mut Num,
    range: RangeInclusive<Num>,
    default: Num,
    name: &str,
    help_text: &str,
    cfg_changed: Option<&mut bool>,
) -> Response {
    ui.horizontal(|ui| {
        let label = ui
            .label(name)
            .on_hover_cursor(CursorIcon::Help)
            .on_hover_text(help_text);

        let mut changed = ui
            .add(egui::Slider::new(value, range))
            .labelled_by(label.id)
            .changed();

        #[allow(clippy::collapsible_if)]
        if *value != default {
            if ui.button("⟳").clicked() {
                *value = default;
                changed = true;
            }
        }

        if let Some(cfg_changed) = cfg_changed {
            *cfg_changed |= changed;
        }
    })
    .response
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.style_changed {
            self.style_changed = false;
            style::apply(ctx, &self.style_params);
        }

        for message in self.inbox.read_without_ctx() {
            message.perform(self, ctx);
        }

        match &mut self.content {
            AppContent::WelcomePage(welcome_page) => welcome_page.draw(ctx, &self.inbox),

            AppContent::MainPage(_) => {
                self.main_page(ctx);
            }
        }
    }
}

impl App {
    /// Panics if `content` is not [`AppContent::MainPage`]
    fn main_page(&mut self, ctx: &Context) {
        let AppContent::MainPage(ref mut main_page) = self.content else {
            panic!(
                "AppContent is not `MainPage` when `App::main_page` was called.
                It's the following instead: {:#?}",
                self.content
            )
        };

        if main_page.progress.is_some() {
            ctx.request_repaint_after_secs(0.1);
        }

        egui::TopBottomPanel::top("ribbon tab bar")
            .frame(Frame::none())
            .show_separator_line(false)
            .show(ctx, |ui| {
                tabbar(
                    ui,
                    &mut main_page.pane,
                    [
                        (PaneContent::Grouping, "🗁 Grouping"),
                        (PaneContent::Naming, "🏷 Naming"),
                        (PaneContent::ManualEdit, "✏ Manual edits"),
                        (PaneContent::Apply, "☑ Apply"),
                        (PaneContent::Home, "🏠 Home"),
                        (PaneContent::View, "👁 View"),
                    ],
                );
            });

        egui::TopBottomPanel::top("ribbon content").min_height(64.0).show_separator_line(false).show_animated(ctx, main_page.pane.is_some(), |ui| {
            let Some(pane) = main_page.pane.clone() else { return };

            ui.add_space(4.0);
            let mut cfg_changed = false;

            ui.with_layout(Layout::left_to_right(Align::TOP).with_cross_justify(true), |ui| {
                animated_pager(ui, pane, &TransitionStyle::horizontal(ui), Id::from("ribbon"), |ui, pane| {
                    match pane {
                        PaneContent::Grouping => {
                            let mut sort_btn_clicked = false;

                                ui.scope(|ui| {
                                    ui.set_max_width(128.0);
                                    egui_extras::StripBuilder::new(ui).size(Size::remainder()).size(Size::exact(24.0)).vertical(|mut strip| {

                                        strip.cell(|ui| {
                                            match main_page.is_sort_process_idle() {
                                                true => {
                                                    if main_page.auto_sort {
                                                        ui.disable();
                                                    }
                                                    sort_btn_clicked = ui.add_sized(ui.available_size(), Button::new("⛭ Sort").selected(true)).clicked();
                                                },
                                                false => {
                                                    ui.horizontal_centered(|ui| {
                                                        ui.spinner(); //.labelled_by(label.id);
                                                        ui.label("Sorting...")
                                                    });
                                                }
                                            }
                                        });

                                        strip.cell(|ui| {
                                            cfg_changed |= ui.checkbox(&mut main_page.auto_sort, "Sort automatically").changed();
                                        });
                                    });
                                });


                                ui.separator();

                                ribbon_slider(
                                    ui,
                                    &mut main_page.operation_config.geogroup_params.depth,
                                    0..=u8::MAX,
                                    backend::algorithm::Params::default().depth,
                                    "Depth",
                                    "High values yield deeply nested folder structure. Low values lead to shallow structures",
                                    Some(&mut cfg_changed),
                                );


                                if (cfg_changed && main_page.auto_sort)
                                    | sort_btn_clicked
                                    | main_page.sort_pending
                                {

                                    if main_page.is_sort_process_idle() {
                                        main_page.sort_process = Some(action_sort(
                                            &mut main_page.operation_config,
                                            main_page.hiearchy.to_owned(),
                                            &self.args,
                                            &self.inbox,
                                        ));
                                        main_page.sort_pending = false;
                                    } else {
                                        main_page.sort_pending = true;
                                    }
                                }
                        }
                        PaneContent::Naming => {}
                        PaneContent::ManualEdit => {
                            if main_page.auto_sort {
                                ui.vertical(|ui| {
                                    ui.strong("Automatic sorting is enabled");
                                    ui.label("To make manual changes to the hiearchy, please disable automatic sorting.");
                                    if ui.button("Disable automatic sorting").clicked() {
                                        main_page.auto_sort = false;
                                    }
                                });
                            } else if let Some(selected_item) = main_page.selected_item_mut() {
                                match selected_item {
                                    HiearchyItem::Group(_, name) => {
                                        ui.text_edit_singleline(name);
                                        if ui.button("Dissolve").clicked() {
                                            // TODO: Report failure
                                            main_page.dissolve_selected();
                                        }
                                    }
                                    HiearchyItem::Item(it) => {
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(&mut it.name);
                                        });
                                    }
                                }
                            }
                        }
                        PaneContent::Apply => {
                            if ui.button("Apply by copying files").clicked() {
                                let target_dir = rfd::FileDialog::new().pick_folder();

                                if let Some(target_dir) = target_dir {
                                    // This is here, because if there already was some progress, it would be erased by this
                                    // TODO: Prevent this
                                    debug_assert!(main_page.progress.is_none(), "There is some progress already, but we are trying to overwrite it by applying");

                                    self.inbox.sender().send(
                                        Message::SetProgress(Some(Progress {
                                            action: ProgressAction::Applying,
                                            progress: None,
                                        }))
                                    ).unwrap();

                                    let hiearchy = backend::HiearchyItem::Group(main_page.hiearchy.clone(), String::new()).map_leafs(&|file| backend::apply::ApplyLeaf {
                                        target_name: file.name,
                                        original_path: file.path
                                    });

                                    let inbox_sender = self.inbox.sender();

                                    thread::spawn(move || {
                                        backend::apply::apply_by_copy(hiearchy, target_dir).unwrap();

                                        inbox_sender.send(
                                            Message::SetProgress(None)
                                        ).unwrap();
                                    });

                                }
                            };
                        }
                        PaneContent::Home => {
                            #[cfg(target_os = "linux")]
                            if ui.button("🗖 New window").clicked() {
                                std::process::Command::new("/proc/self/exe").spawn().expect("failed to start myself");
                            }

                            if ui.button("❌ Close directory").clicked() {
                                self.inbox.sender().send(Message::SetContent(AppContent::WelcomePage(WelcomePage::Normal))).unwrap();
                            };
                            if ui.button("❎ Quit Geogroup").clicked() {
                                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                            };
                        }
                        PaneContent::View => {
                            ribbon_slider(
                                ui,
                                &mut main_page.image_scale,
                                32..=128,
                                48,
                                "Image size",
                                "Height of image previews",
                                Some(&mut cfg_changed),
                            );

                            ui.separator();

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Accent color");
                                    if ui.color_edit_button_srgba(&mut self.style_params.accent_color).changed() {
                                        self.style_changed = true;
                                    }
                                });

                                ui.horizontal_centered(|ui| {
                                    ui.label("Color theme");
                                    egui_theme_switch::global_theme_switch(ui)
                                });
                            });

                        }
                    }
                });

            });
        });

        egui::TopBottomPanel::bottom("bottom statusbar").show(ctx, |ui| {
            if let Some(ref progress) = main_page.progress {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(match progress.action {
                        ProgressAction::Geocoding => "Geocoding",
                        ProgressAction::Grouping => "Grouping",
                        ProgressAction::Naming => "Naming",

                        ProgressAction::Applying => "Applying",
                    });

                    if let Some(progress) = progress.progress {
                        ProgressBar::new(progress as f32 / u16::MAX as f32)
                            .show_percentage()
                            .ui(ui);
                    }
                });
            } else {
                ui.label("Idle");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            show_hiearchy(
                ui,
                &main_page.hiearchy,
                &mut main_page.selection,
                &main_page.flatten_mode,
                main_page.image_scale,
            )
        });
    }
}

impl CliArgs {
    pub fn get_geocoder(&self) -> RevGeocoder<'static> {
        RevGeocoder::from_env_key(self.geocoding_cache.to_owned())
    }
}

fn action_sort(
    operation_config: &mut geogroup_backend::SortingCfg,
    hiearchy: Vec<geogroup_backend::HiearchyItem<FileInHiearchy, String>>,
    args: &CliArgs,
    inbox: &UiInbox<Message>,
) -> thread::JoinHandle<()> {
    let sender = inbox.sender();
    let op_config = operation_config.geogroup_params.to_owned();

    let mut flattened = hiearchy
        .iter()
        .flat_map(|h| h.leaves_cloned())
        .collect::<Vec<_>>();
    flattened.sort_by_key(|it| it.date);
    let ready_for_sorting = flattened
        .into_iter()
        .filter_map(|leaf| leaf.transform_for_sorting()) // TODO: Don't just filter out items without a position
        .collect();

    let geocoder = args.get_geocoder();

    std::thread::spawn(move || {
        sender
            .send(Message::SetProgress(Some(Progress {
                action: ProgressAction::Grouping,
                progress: None,
            })))
            .unwrap();

        let sorted_hiearchy = backend::algorithm::sort(ready_for_sorting, &op_config)
            .map_leafs(&|leaf| FileInHiearchy::transform_after_sorting(leaf))
            .map_group_data(&|()| String::from("Group"));

        println!("Sorted!");

        sender
            .send(Message::SetProgress(Some(Progress {
                action: ProgressAction::Geocoding,
                progress: None,
            })))
            .unwrap();

        geocoder
            .prefetch_places(
                &sorted_hiearchy
                    .leaves()
                    .filter_map(|file| file.pos)
                    .collect(),
                |prog| {
                    sender
                        .send(Message::SetProgress(Some(Progress {
                            action: ProgressAction::Geocoding,
                            progress: Some(
                                (u16::MAX as f32 * prog.done as f32 / prog.total as f32) as u16,
                            ),
                        })))
                        .unwrap();
                },
            )
            .unwrap();

        sender
            .send(Message::SetProgress(Some(Progress {
                action: ProgressAction::Naming,
                progress: None,
            })))
            .unwrap();

        let named_hiearchy = geocoder
            .name_hiearchy(
                sorted_hiearchy
                    .map_leafs(&|file| {
                        (
                            file.pos
                                .expect("Missing point, TODO: Handle this correctly"),
                            file,
                        )
                    })
                    .map_group_data(&|_| ()),
            )
            .map_leafs(&|(name, file)| FileInHiearchy { name, ..file });

        sender.send(Message::SetProgress(None)).unwrap();

        sender
            .send(Message::Sorted {
                new_hiearchy: match named_hiearchy {
                    backend::HiearchyItem::Group(g, _) => g,
                    backend::HiearchyItem::Item(it) => vec![backend::HiearchyItem::Item(it)],
                },
            })
            .unwrap();
    })
}

impl Message {
    pub fn perform(self, app: &mut App, ctx: &egui::Context) {
        match self {
            Message::SetContent(content) => app.content = content,
            Message::Sorted { new_hiearchy } => {
                if let AppContent::MainPage(ref mut main_page) = app.content {
                    main_page.selection = Vec::new();
                    main_page.hiearchy = new_hiearchy;
                } else { /* TODO: Warning */
                }
            }
            Message::SetProgress(p) => {
                if let AppContent::MainPage(ref mut main_page) = app.content {
                    main_page.progress = p;
                    ctx.request_repaint();
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
                self.action_load_dir(inbox, folder);
            }
        };
    }

    fn action_load_dir(&mut self, inbox: &UiInbox<Message>, folder: PathBuf) {
        let sender = inbox.sender();

        std::thread::spawn(move || {
            use geogroup_backend::HiearchyItem as HI;
            let res = backend::load_directory(&folder);
            // Send will return an error if the receiver has been dropped
            // but unless you have a long running task that will send multiple messages
            // you can just ignore the error
            sender
                .send(Message::SetContent(match res {
                    Ok(HI::Group(hiearchy, _)) => {
                        AppContent::MainPage(MainPage::new(hiearchy, folder))
                    }
                    Ok(HI::Item(_)) => AppContent::WelcomePage(WelcomePage::Error(String::from(
                        "Please choose a directory",
                    ))),
                    Err(err) => AppContent::WelcomePage(WelcomePage::Error(err.to_string())),
                }))
                .ok();
        });

        *self = Self::Loading("Loading files".into());
    }
}

impl MainPage {
    fn new(
        hiearchy: Vec<geogroup_backend::HiearchyItem<PathBuf, PathBuf>>,
        folder: PathBuf,
    ) -> MainPage {
        MainPage {
            hiearchy: hiearchy
                .into_iter()
                .map(|h| {
                    h.map_group_data(&|path: PathBuf| filename_to_string(path.file_name()))
                        .map_leafs(&|path| {
                            let loc_data = backend::loaders::GeneralLoader.get_data(&path).ok(); // TODO: Do something with unexpected errors
                            FileInHiearchy {
                                name: filename_to_string(path.file_name()),
                                path,
                                pos: loc_data.as_ref().and_then(|loc_data| {
                                    loc_data
                                        .location
                                        .as_ref()
                                        .ok()
                                        .map(|rect| rect.center().into())
                                }),
                                date: loc_data.as_ref().and_then(|loc_data| {
                                    loc_data.time.as_ref().ok().map(|dates| dates[0])
                                    // TODO: Don't use just the first one
                                }),
                            }
                        })
                })
                .collect(),
            pane: Some(PaneContent::Grouping),
            src_dir: folder,
            flatten_mode: None,
            selection: Vec::new(),
            image_scale: 48,
            progress: None,
            operation_config: Default::default(),
            auto_sort: true,
            sort_process: None,
            // True to perform an initial sort
            sort_pending: true,
        }
    }

    /// Returns [None] if either:
    ///  - Nothing is selected
    pub fn selected_item(&self) -> Option<&backend::HiearchyItem<FileInHiearchy, String>> {
        let mut idx_iter = self.selection.iter();
        let Some(first_idx) = idx_iter.next() else {
            return None;
        };
        let mut current_hiearchy = &self.hiearchy[*first_idx];

        for idx in idx_iter {
            let HiearchyItem::Group(children, _) = current_hiearchy else {
                panic!("Too many indices in selection - tried to probe contents of an item, it should be a group.");
            };
            current_hiearchy = &children[*idx];
        }

        Some(current_hiearchy)
    }

    /// Returns [None] if either:
    ///  - Nothing is selected
    pub fn selected_item_mut(
        &mut self,
    ) -> Option<&mut backend::HiearchyItem<FileInHiearchy, String>> {
        let mut idx_iter = self.selection.iter();
        let Some(first_idx) = idx_iter.next() else {
            return None;
        };
        let mut current_hiearchy = &mut self.hiearchy[*first_idx];

        for idx in idx_iter {
            let HiearchyItem::Group(children, _) = current_hiearchy else {
                panic!("Too many indices in selection - tried to probe contents of an item, it should be a group.");
            };
            current_hiearchy = &mut children[*idx];
        }

        Some(current_hiearchy)
    }

    fn is_sort_process_idle(self: &mut MainPage) -> bool {
        self.sort_process.as_ref().is_none_or(|p| p.is_finished())
    }

    /// Returns [false] if nothing was dissolved because a group was not selected
    fn dissolve_selected(&mut self) -> bool {
        let idx_of_dissolved = self.selection.pop().unwrap();

        let target_items =
            if let Some(HiearchyItem::Group(target_items, _)) = self.selected_item_mut() {
                target_items
            } else {
                &mut self.hiearchy
            };

        let HiearchyItem::Group(dissolved_items, _) = target_items.remove(idx_of_dissolved) else {
            return false;
        };

        target_items.reserve(dissolved_items.len());

        let mut v = target_items.split_off(idx_of_dissolved);
        target_items.extend_from_slice(&dissolved_items);
        target_items.append(&mut v);

        return true;
    }
}

fn show_hiearchy(
    ui: &mut Ui,
    hiearchy: &Hiearchy,
    selected_vec: &mut Vec<usize>,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
) {
    egui::ScrollArea::horizontal()
        .stick_to_right(true)
        .show(ui, |ui| {
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
                                    ui.add(Label::new(format!("🗁 {name}")).selectable(false));
                                }
                                geogroup_backend::HiearchyItem::Item(item) => {
                                    ui.horizontal_top(|ui| {
                                        if let Some(file_path) = item.path.to_str() {
                                            Image::new(format!("file://{file_path}")).ui(ui);
                                        }

                                        ui.add(Label::new(&item.name).selectable(false))
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
