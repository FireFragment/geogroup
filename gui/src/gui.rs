//! The core of this crate, the code rendering all the GUI

use egui_transition_animation::animated_pager;
use egui_transition_animation::TransitionStyle;
use std::ops::RangeInclusive;
use std::thread;

use super::*;

pub(crate) fn ribbon_slider<Num: emath::Numeric>(
    ui: &mut Ui,
    slider: egui::Slider,
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

        let mut changed = ui.add(slider).labelled_by(label.id).changed();

        //TODO
        /*#[allow(clippy::collapsible_if)]
        if *value != default {
            if ui.button("⟳").clicked() {
                *value = default;
                changed = true;
            }
        }*/
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
    pub(crate) fn main_page(&mut self, ctx: &Context) {
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

        egui::TopBottomPanel::top("ribbon content").max_height(96.0).min_height(96.0).show_separator_line(false).show_animated(ctx, main_page.pane.is_some(), |ui| {
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
                                                    sort_btn_clicked = ui.add_sized(ui.available_size(), Button::new("⛭ Sort").fill(ui.style().visuals.selection.bg_fill)).clicked();
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
                                    egui::Slider::new(
                                        &mut main_page.operation_config.depth,
                                        0..=backend::algorithm::MAX_DEPTH
                                    ).custom_formatter(|num, range|  format!(".{:>2}", num*100.0 / *range.end() as f64)), // TODO: Add also custom parser
                                    backend::algorithm::Params::default().depth,
                                    "Depth",
                                    "High values yield deeply nested folder structure. Low values lead to shallow structures",
                                    Some(&mut cfg_changed),
                                );
                        }
                        PaneContent::Naming => {}
                        PaneContent::ManualEdit => {
                            todo!()
                            /*if main_page.auto_sort {
                                ui.vertical(|ui| {
                                    ui.strong("Automatic sorting is enabled");
                                    ui.label("To make manual changes to the hiearchy, please disable automatic sorting.");
                                    if ui.button("Disable automatic sorting").clicked() {
                                        main_page.auto_sort = false;
                                    }
                                });
                            } else if let Some(selected_item) = main_page.selected_item_mut() {
                                /*match selected_item {
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
                                }*/
                            }*/
                        }
                        PaneContent::Apply => {
                            /*if ui.button("Apply by copying files").clicked() {
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

                                    let hiearchy = backend::HiearchyItem::Group(main_page.hiearchy.clone(), hiearchy::GroupData::new()).map_leafs(&|file| backend::apply::ApplyLeaf {
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
                            };*/
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
                                egui::Slider::new(&mut main_page.image_scale,
                                32..=128,),
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

pub fn show_hiearchy(
    ui: &mut Ui,
    hiearchy: &TemplateHiearchy,
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
    hiearchy: &TemplateHiearchy,
    selected_vec: &mut Vec<usize>,
    current_depth: usize,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
) {
    todo!()
    /*
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
        let group_row_size =
            ui.style().text_styles[&TextStyle::Body].size + ui.style().spacing.item_spacing.y * 2.0;
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
                            HI::Group(_, _) => group_row_size,
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
    }*/
}

pub(crate) fn error_ui(ui: &mut Ui, error: &str) {
    ui.colored_label(ui.visuals().error_fg_color, format!("⊗ {}", error));
}

fn widgetvisuals_to_frame(
    visuals: &egui::style::WidgetVisuals,
    button_padding: Vec2,
) -> egui::Frame {
    egui::Frame {
        fill: visuals.bg_fill,
        stroke: visuals.bg_stroke,
        inner_margin: Margin::symmetric(button_padding.x, button_padding.y),
        rounding: visuals.rounding,
        ..egui::Frame::none()
    }
}

pub fn big_btn(ui: &mut Ui, icon: &str, heading: &str, description: &str) -> Response {
    ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
        let response = ui.response();
        let visuals = ui.style().interact(&response);

        widgetvisuals_to_frame(visuals, ui.style().spacing.button_padding).show(ui, |ui| {
            ui.set_width(256.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icon)
                        .size(32.0)
                        .color(ui.style().visuals.selection.bg_fill),
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(heading);
                    ui.weak(description);
                });
            });
        });
    })
    .response
}

impl WelcomePage {
    fn draw(&mut self, ctx: &egui::Context, inbox: &UiInbox<Message>) {
        egui::SidePanel::left("recents")
            .frame(Frame::default().inner_margin(Margin::same(32.0)))
            .show(ctx, |ui| {
                ui.style_mut().spacing.button_padding *= 2.0;

                /*ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::BLACK;
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    ui.style_mut().visuals.widgets.inactive.bg_fill;
                ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke {
                    color: ui.style().visuals.selection.bg_fill,
                    width: 2.0,
                };*/

                //ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                /*ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::BLACK;
                ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke {
                    color: ui.style().visuals.selection.bg_fill,
                    width: 1.0,
                };*/

                //ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::BLACK;
                ui.style_mut().visuals.widgets.inactive.bg_fill = ui
                    .style_mut()
                    .visuals
                    .widgets
                    .inactive
                    .bg_fill
                    .gamma_multiply(0.7);

                ui.style_mut().visuals.widgets.hovered.bg_fill =
                    ui.style_mut().visuals.widgets.inactive.bg_fill;
                ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke {
                    color: ui.style().visuals.selection.bg_fill,
                    width: 1.0,
                };

                /*ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
                ui.style().visuals.selection.bg_fill;*/

                /*ui.style_mut().visuals.widgets.hovered = ui.style_mut().visuals.widgets.inactive;
                ui.style_mut().visuals.widgets.hovered.bg_fill = ui
                    .style_mut()
                    .visuals
                    .widgets
                    .hovered
                    .bg_fill
                    .lerp_to_gamma(Color32::WHITE, 0.3);*/

                /*ui.style_mut().visuals.widgets.hovered.bg_fill = ui.style().visuals.selection.bg_fill;
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    ui.style_mut().visuals.widgets.hovered.bg_fill;
                ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                ui.style_mut().visuals.widgets.hovered.expansion = 1.0;*/

                ui.label(RichText::new("Welcome to Geogroup").font(FontId {
                    size: 48.0,
                    family: FontFamily::Name("Light".into()),
                }));
                ui.add_space(32.0);

                if matches!(self, Self::Normal | Self::Error(_)) {
                    //ui.add_space(ctx.style().text_styles[&TextStyle::Heading].size);

                    self.pick_file_btn(ui, inbox);
                    let clicked = big_btn(
                        ui,
                        "🗋",
                        "Open GEGR file",
                        "Continue working on your saved project",
                    )
                    .clicked();

                    if clicked {
                        todo!()
                    }
                };

                ui.horizontal(|ui| ui.link(" GitHub"));

                match self {
                    Self::Normal => (),
                    Self::Loading(msg) => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(&*msg);
                        });
                    }
                    Self::Error(err) => {
                        gui::error_ui(ui, err);
                    }
                }
            });

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(Margin::same(32.0)),
            )
            .show(ctx, |ui| {
                ui.heading("Recent projects");

                ui.label("No recently opened projects\nUse the left panel to open a folder with your photos")
            });
    }

    fn pick_file_btn(&mut self, ui: &mut Ui, inbox: &UiInbox<Message>) {
        let clicked = big_btn(
            ui,
            "🗁",
            "Pick a folder",
            "Locate the photo collection you wish to sort",
        )
        .clicked();
        if clicked {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.action_load_dir(inbox, folder);
            }
        };
    }

    fn action_load_dir(&mut self, inbox: &UiInbox<Message>, folder: PathBuf) {
        todo!()

        /*let sender = inbox.sender();

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

        *self = Self::Loading("Loading files".into());*/
    }
}

#[derive(Debug)]
pub enum Message {
    SetContent(AppContent),
    Sorted {
        new_hiearchy: hiearchy::TemplateHiearchy,
    },
    SetProgress(Option<Progress>),
}

pub struct App {
    pub content: AppContent,
    pub inbox: UiInbox<Message>,
    pub args: CliArgs,
    pub style_params: style::Params,
    pub style_changed: bool,
}

impl App {
    pub fn new(args: CliArgs) -> Self {
        let mut app = Self {
            content: gui::AppContent::default(),
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

        app
    }
}

#[derive(Debug)]
pub enum AppContent {
    WelcomePage(WelcomePage),
    MainPage(MainPage),
}

#[derive(Debug)]
struct MainPage {
    pane: Option<PaneContent>,
    src_dir: PathBuf,
    hiearchy: hiearchy::TemplateHiearchy,
    flatten_mode: Option<FlattenMode>,
    selection: Vec<usize>,
    image_scale: u16,
    progress: Option<Progress>,
    /// Config controlling the entire operation, including sorting, naming, etc.
    operation_config: backend::algorithm::Params,
    auto_sort: bool,
    /// Whether sorting should be rerun once `sort_process` completes.
    /// This may happen when user changes configuration during sorting. In that case, the already running `sort_process` uses outdated configuration.
    sort_pending: bool,
    sort_process: Option<thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub struct Progress {
    pub action: ProgressAction,
    pub progress: Option<u16>,
}

#[derive(Debug)]
pub enum ProgressAction {
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

impl MainPage {
    fn new(
        //hiearchy: Vec<geogroup_backend::HiearchyItem<PathBuf, PathBuf>>,
        folder: PathBuf,
    ) -> MainPage {
        MainPage {
            hiearchy: todo!(), /*hiearchy
                               .into_iter()
                               .map(|h| {
                                   h.map_group_data(&|path: PathBuf| filename_to_string(path.file_name()))
                                       .map_leafs(&|path| {
                                           let loc_data = backend::loaders::GeneralLoader.get_data(&path).ok(); // TODO: Do something with unexpected errors
                                           hiearchy::FileInfo {
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
                               .collect(),*/
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
    /*pub fn selected_item(&self) -> Option<&backend::HiearchyItem<hiearchy::FileInfo, String>> {
        let mut idx_iter = self.selection.iter();
        let Some(first_idx) = idx_iter.next() else {
            return None;
        };
        let mut current_hiearchy = &self.hiearchy[*first_idx];

        for idx in idx_iter {
            let hiearchy::template::Node::Group(group) = current_hiearchy else {
                panic!("Too many indices in selection - tried to probe contents of an item, it should be a group.");
            };
            current_hiearchy = &group.body[*idx];
        }

        Some(current_hiearchy)
    } TODO*/

    /// Returns [None] if either:
    ///  - Nothing is selected
    /*pub fn selected_item_mut(
        &mut self,
    ) -> Option<&mut backend::HiearchyItem<hiearchy::FileInfo, String>> {
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
    }*/

    fn is_sort_process_idle(self: &mut MainPage) -> bool {
        self.sort_process.as_ref().is_none_or(|p| p.is_finished())
    }

    /// Returns [false] if nothing was dissolved because a group was not selected
    fn dissolve_selected(&mut self) -> bool {
        todo!()
        /*let idx_of_dissolved = self.selection.pop().unwrap();

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

        return true;*/
    }
}

impl Default for AppContent {
    fn default() -> Self {
        Self::WelcomePage(WelcomePage::Normal)
    }
}
