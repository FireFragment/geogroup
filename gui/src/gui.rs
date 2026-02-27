//! The core of this crate, the code rendering all the GUI

use derive_more::From;
use eframe::egui::Color32;
use eframe::egui::TextStyle;
use egui::{
    Align, Button, CursorIcon, FontFamily, FontId, Layout, Margin, ProgressBar, RichText, Sense,
    Stroke, UiBuilder, Vec2, ViewportCommand, Widget,
};
use egui_extras::TableBuilder;
use egui_transition_animation::animated_pager;
use egui_transition_animation::TransitionStyle;
use geogroup_backend::lazy_hierarchy::concrete::Group;
use geogroup_backend::lazy_hierarchy::NodeRef;
use geogroup_backend::main_hierarchy;
use geogroup_backend::progress;
use geogroup_backend::ONE_METER_DISTANCE;
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use thiserror::Error;

use super::*;

pub(crate) fn ribbon_slider<Num: emath::Numeric>(
    ui: &mut egui::Ui,
    slider: egui::Slider,
    default: Num,
    name: &str,
    help_text: &str,
    cfg_changed: Option<&mut bool>,
) -> egui::Response {
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
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        match visuals.dark_mode {
            true => egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180).to_normalized_gamma_f32(),
            false => egui::Color32::from_rgba_unmultiplied(u8::MAX, u8::MAX, u8::MAX, 128).to_normalized_gamma_f32(),
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        style::possible_apply(ctx, &self.style_manager);
        for message in self.inbox.read_without_ctx() {
            message.perform(self, ctx);
        }

        /*
        if let Some(cpu_usage) = frame.info().cpu_usage {
            egui::TopBottomPanel::bottom("performance").show(ctx, |ui| {
                let since_last_frame = ctx.input(|input| input.unstable_dt);
                self.mean_cpu_usage = (self.mean_cpu_usage * 7.0 + (cpu_usage / (1.0 / 60.0))) / 8.0;

                egui::ProgressBar::new(self.mean_cpu_usage)
                    .rounding(egui::Rounding::ZERO)
                    .ui(ui);
                ctx.request_repaint();

                //std::thread::sleep(Duration::from_millis(1000 / 90));
            });
        };*/

        match &mut self.content {
            AppContent::WelcomePage(_) => self.draw_welcome_page(ctx),
            AppContent::MainPage(_) => self.draw_main_page(ctx),
        }
    }
}

impl App {
    fn load_dir(&mut self, dir: PathBuf) {
        // TODO: Check if it's not a file and a valid directory
        self.content = MainPage::new(dir).into();
        //

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

    /*fn get_concrete_parent(&self) -> & {
        todo!()
    }*/

    /// Panics if `content` is not [`AppContent::MainPage`]
    pub(crate) fn draw_main_page(&mut self, ctx: &egui::Context) {
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
            .frame(Frame::new())
            .show_separator_line(false)
            .show(ctx, |ui| {
                use tabbar::Item::*;
                enum Action { Deselect }
                let action = tabbar(
                    ui,
                    &mut main_page.pane,
                    [
                        (Tab(PaneContent::Grouping), "🗁 Grouping"),
                        (Tab(PaneContent::Naming), "🏷 Naming"),
                        (Tab(PaneContent::ManualEdit), "✏ Manual edits"),
                        (Tab(PaneContent::Apply), "☑ Apply"),
                        (Tab(PaneContent::Home), "🏠 Home"),
                        (Tab(PaneContent::View), "👁 View"),
                        //(Action(Action::Deselect), "Deselect all"),
                    ],
                );

                match action {
                    Some(Action::Deselect) => main_page.selection = Vec::new(),
                    None => {},
                }
            });

        egui::TopBottomPanel::top("ribbon content").max_height(96.0).min_height(96.0).show_separator_line(false).show_animated(ctx, main_page.pane.is_some(), |ui| {
            let Some(pane) = main_page.pane.clone() else { return };

            ui.add_space(4.0);

            ui.with_layout(Layout::left_to_right(Align::TOP).with_cross_justify(true), |ui| {
                animated_pager(ui, pane, &TransitionStyle::horizontal(ui).with_fade(), egui::Id::from("ribbon"), |ui, pane| {
                    match pane {
                        PaneContent::Grouping => {
                            let selected_static_id_opt = main_page.hiearchy.selection_to_static_id(main_page.selection.iter().cloned());

                            let lazy_hierarchy::concrete::Node::Leaf(ref mut leaf) = main_page.hiearchy.0.root_group_mut().children_mut()[0]
                                else { todo!() } ;

                            let main_hierarchy::InnerLeafData::LazySubgroup(subgroup) = leaf.leaf_data_mut() else { todo!() };

                            if let Some(main_hierarchy::lazy_group::dynamic::Finalized::Success(subgroup_final)) =
                                subgroup.get_final_mut()
                            {
                                match subgroup_final {
                                    main_hierarchy::lazy_group::Final::Sorted(subgroup_sorted) => {
                                        ui.vertical(|ui| {

                                            ribbon_slider(
                                                ui,
                                                egui::Slider::new(
                                                    &mut subgroup_sorted.params_mut().depth,
                                                    0..=backend::algorithm::MAX_DEPTH
                                                ).custom_formatter(|num, range| {
                                                    format!(".{:0>2}", ((num*100.0)/(backend::algorithm::MAX_DEPTH as f64)) as u8)
                                                }), // TODO: Add also custom parser
                                                backend::algorithm::Params::default().depth,
                                                "Depth",
                                                "High values yield deeply nested folder structure. Low values lead to shallow structures",
                                                None,
                                            );
                                            ui.horizontal(|ui| {
                                                let label = ui
                                                    .label("Minimum distance of separated items")
                                                    .on_hover_cursor(CursorIcon::Help)
                                                    .on_hover_text("The minimum distance of consecutive items that are not in the same group. No two items going right after each other that are closer than this distance will be separated into different groups.");

                                                let meters = subgroup_sorted.params_mut().minimum_distance / ONE_METER_DISTANCE;
                                                // TODO: Add custom parser
                                                egui::DragValue::new(&mut subgroup_sorted.params_mut().minimum_distance)
                                                    .speed({
                                                        ONE_METER_DISTANCE as f32 * match meters {
                                                            ..1000 => 1.0,
                                                            1000..5000 => 100.0,
                                                            5000.. => 1000.0
                                                        }
                                                    })
                                                    .custom_formatter(|dist, _| {
                                                        let meters = dist as u64 / ONE_METER_DISTANCE;
                                                        match meters {
                                                            ..1000 => {format!("{meters}m")},
                                                            1000..5000 => {format!("{:.1}km", meters as f32 / 1000.0)}
                                                            5000.. => {format!("{}km", (meters as f32 / 1000.0).round())}
                                                        }
                                                    })
                                                    .ui(ui);
                                            });
                                        });

                                        ui.separator();
                                        if let Some(static_id) = selected_static_id_opt {
                                                ui.vertical(|ui| {
                                                    ui.strong("Selected item:");
                                                    let is_retained = subgroup_sorted.sorter_mut().get_manual_modification(static_id) ==
                                                        Some(
                                                            algorithm::geogroup::ManualModification::Retain
                                                        );

                                                    if ui.add_enabled(!is_retained, egui::Button::new("Dissolve")).clicked() {
                                                        subgroup_sorted.sorter_mut().set_manual_modification(
                                                            static_id,
                                                            algorithm::geogroup::ManualModification::Dissolve
                                                        );
                                                    };

                                                    let clicked = egui::Button::new("Forcibly retain").selected(is_retained).ui(ui).clicked();
                                                    if clicked {
                                                        if is_retained {
                                                            subgroup_sorted.sorter_mut().remove_manual_modification(
                                                                static_id
                                                            );
                                                        } else {
                                                            subgroup_sorted.sorter_mut().set_manual_modification(
                                                                static_id,
                                                                algorithm::geogroup::ManualModification::Retain
                                                            );
                                                        }
                                                    }
                                                });
                                        }
                                    },
                                    _ => {} // TODO
                                }


                            }

                        }
                        PaneContent::Naming => {}
                        PaneContent::ManualEdit => {
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
                                None,
                            );

                            ui.separator();

                            ui.vertical(|ui| {
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

        /*egui::SidePanel::left("colors").show(ctx, |ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ctx.settings_ui(ui);
            });
        });*/

        egui::TopBottomPanel::bottom("bottom statusbar").show(ctx, |ui| {
            /*if let Some(ref progress) = main_page.progress {
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
            }*/

            if let Some(error) = self.errors.last() {
                error_ui(ui, &format!("{error}"));
                if ui.button("✔").clicked() {
                    self.errors.pop();
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            /*if let Some(ppos) = ui.input(|i| i.pointer.latest_pos()) {
                println!("Hovering");


                ui.add(egui::Label::new("Before"));

                let cursor = ui.cursor();
                //ui.put(egui::Rect::from_x_y_ranges(ppos.x.., ppos.y..), egui::Label::new("cursor1"));

                //ui.allocate_ui_at_rect(egui::Rect::from_x_y_ranges(ppos.x.., ppos.y..), |ui| ui.label("cursor2"));
                //ui.place(egui::Rect::from_x_y_ranges(ppos.x.., ppos.y..), egui::Label::new("cursor3"));
                //

                ui.new_child(
                    UiBuilder::new()
                        .max_rect(egui::Rect::from_x_y_ranges(ppos.x.., ppos.y..))
                )
                .add(egui::Label::new("Cursor 4"));

                //
                //ui.allocate_ui_at_rect(cursor, |ui| ui.label("After manipulated"));

                //ui.allocate_rect(cursor, Sense::all());

                ui.add(egui::Label::new("After ato"));
            }*/


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
    ui: &mut egui::Ui,
    hiearchy: &backend::main_hierarchy::TemplateHiearchy,
    selection: &mut Vec<backend::selection::PathComponent>,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
) {
    let mut selecion_indices = hiearchy.selection_to_indices(selection.iter().cloned()).collect_vec();

    let selection_modified = egui::ScrollArea::horizontal()
        .stick_to_right(true)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                show_hiearchy_inner(
                    ui,
                    &hiearchy.root_final(),
                    &mut selecion_indices,
                    0,
                    flatten_mode,
                    image_scale,
                    "show_hierarchy".into()
                )
            }).inner
        }).inner;

    if selection_modified {
        *selection = hiearchy.indices_to_selection(selecion_indices.into_iter()).collect();
    }
}

/// Returns true if selection was modified
fn show_hiearchy_list(
    table: TableBuilder,
    selected_indices: &mut Vec<usize>,
    current_depth: usize,
    group_row_size: f32,
    image_scale: u16,
    items: &Vec<NodeRef<impl main_hierarchy::ImplGroupRef>>,
    id: egui::Id
) -> bool {
    let mut selection_modified = false;

    table.body(|body| {
        body.heterogeneous_rows(
            // Row heights
            items.iter().map(|item| match item {
                lazy_hierarchy::NodeRef::Group(group) => {
                    /*if let main_hierarchy::GroupData::LazySubgroupRoot = group.group_data() {
                        group.get_children().expect("TODO").map(|child| {
                            match child {
                                NodeRef::Group(_) => group_row_size, // There should be no nested `LazySubgroupRoot`s
                                NodeRef::Leaf(_) => image_scale as f32,
                            }
                        }).sum()
                    } else {*/
                        group_row_size
                    //}
                },
                lazy_hierarchy::NodeRef::Leaf(_) => image_scale as f32,
            }),
            // Row contents
            |mut row| {
                let idx = row.index();
                let child = &items[idx];
                let child_data = child.node_data();

                let mut selected = selected_indices.get(current_depth) == Some(&row.index());
                // Don't highlight "initializing" cells
                if let lazy_hierarchy::NodeRef::Leaf(leaf) = child {
                    if let backend::main_hierarchy::LeafData::LazyGroupInitializing { .. } = leaf.leaf_data() {
                        selected = false;
                    }
                }
                row.set_selected(selected);

                if let lazy_hierarchy::NodeRef::Group(g) = child {
                    if let main_hierarchy::GroupData::LazySubgroupRoot = g.group_data() {
                        /*row.set_selected(false);
                        row.set_hovered(false);*/
                    }
                }

                row.col(|ui| {
                    match child {
                        lazy_hierarchy::NodeRef::Group(group) => {
                            /*
                            if let main_hierarchy::GroupData::LazySubgroupRoot = group.group_data() {
                                ui.vertical(|ui| {
                                    let sorter_color = if selected { ui.visuals().selection.bg_fill } else  {
                                        let c = ui.visuals().text_color();
                                        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 64)
                                    };

                                    // The label
                                    {
                                        let mut frame = Frame::none().inner_margin(Margin::symmetric(4.0, 0.0)).begin(ui);

                                        let text_color = if selected { frame.content_ui.visuals().selection.stroke.color }
                                            else { ui.visuals().text_color() };
                                        frame.content_ui.add(
                                            egui::Label::new(RichText::new(format!("⚙ Sorted automatically {child_name}"))
                                                .color(text_color))
                                                .selectable(false)
                                        );
                                        let response = frame.allocate_space(ui);

                                        if response.hovered() {
                                            frame.frame.fill = ui.visuals().widgets.hovered.bg_fill;
                                        }
                                        if selected {
                                            frame.frame.fill = ui.visuals().selection.bg_fill;
                                        }
                                        frame.paint(ui);
                                    }


                                    ui.horizontal_top(|ui| {
                                        use egui_extras::{Column, TableBuilder};

                                        let line_thickness = 0.0; //if selected { 4.0 } else { 2.0 };
                                        let line_margin = 5.0 - line_thickness / 2.0;

                                        ui.add_space(line_margin);
                                        egui::Frame::none()
                                            .fill(sorter_color)
                                            .show(ui, |ui| {
                                                //ui.add_space(ui.visuals().widgets.noninteractive.bg_stroke.width);
                                                ui.add_space(line_thickness);
                                                ui.set_height(ui.available_height());
                                            });
                                        ui.add_space(line_margin);

                                        show_hiearchy_list(
                                            TableBuilder::new(ui)
                                                .column(Column::remainder())
                                                .vscroll(false)
                                                .sense(Sense::click()),
                                            &mut None, // TODO: Allow selections here
                                            group_row_size,
                                            image_scale,
                                            &group.get_children().expect("TODO").collect(),
                                        );
                                    });
                                });
                            } else {*/
                                // TODO: Group names
                                name_label(ui, child_data, true, id.with(idx));
                            //}

                        }
                        lazy_hierarchy::NodeRef::Leaf(leaf) => {
                            use backend::main_hierarchy::LeafData;
                            match leaf.leaf_data() {
                                LeafData::File(file) => {
                                    ui.horizontal_top(|ui| {
                                        match thumbnails::get(&file.path) {
                                            thumbnails::TResult::CreationInProgress => {
                                                egui::Spinner::new().size(ui.available_height()).ui(ui);
                                            },
                                            thumbnails::TResult::Error(toe_error_type) => {},
                                            thumbnails::TResult::Thumbnail(thumb) => {
                                               egui::Image::new(format!(
                                                   "file://{}",
                                                   thumb.to_string_lossy()
                                               ))
                                               .ui(ui);
                                           },
                                        }
                                        name_label(ui, child_data, false, id.with(idx));
                                    });
                                }
                                LeafData::LazyGroupInitializing { message, progress } => {
                                    ui.vertical(|ui| {
                                        ui.horizontal_top(|ui| {
                                            egui::Spinner::new().ui(ui);
                                            if let Some(msg) = message {
                                                ui.label(msg);
                                            }
                                        });
                                        if let Some(progress) = progress {
                                            egui::ProgressBar::new(progress as f32 / u16::MAX as f32)
                                                .ui(ui);
                                        }
                                    });
                                },
                            }
                        }
                    };
                });

                if row.response().clicked() {
                    selected_indices.truncate(current_depth);
                    selected_indices.push(idx);
                    selection_modified = true;
                }

                //row.set_selected(selection_highlight);
            },
        );
    });

    selection_modified
}

fn name_label(ui: &mut egui::Ui, node_data: main_hierarchy::NodeData, is_group: bool, id: impl Into<egui::Id>) {

        ui.horizontal(|ui| {
            match node_data.name {
                main_hierarchy::NameStatus::Named(_) => {},
                main_hierarchy::NameStatus::InProgress(_) => {
                    ui.spinner();
                    //ui.small(RichText::new("Naming...").italics());
                },
                main_hierarchy::NameStatus::Error { ref err, name: _ } => {
                    ui.label(RichText::new("🏷 ").color(Color32::RED))
                        .on_hover_text(
                            RichText::new(format!("Naming failed: {err}")).color(Color32::RED));
                    //ui.small(RichText::new("Naming failed").color(Color32::RED));
                },
            }

            if is_group {
                ui.label("🗁");
            };

            ui.add(egui::Label::new(
                if let Some(label) = node_data.name.get_name() {
                    label.into()
                } else {
                    RichText::new(format!("Name missing")).italics()
                }
            ).selectable(false));
        });
}

/// Returns whether the selection was modified.
fn show_hiearchy_inner(
    ui: &mut egui::Ui,
    hiearchy: &impl main_hierarchy::ImplGroupRef,
    selected_indices: &mut Vec<usize>,
    current_depth: usize,
    flatten_mode: &Option<FlattenMode>,
    image_scale: u16,
    mut id: egui::Id
) -> bool {
    // TODO: Preseve selection through depth (and other algorithm parameters) changes
    // TODO: Maybe we don't have to crash so horribly?
    debug_assert!(selected_indices.len() >= current_depth);

    use egui_extras::{Column, TableBuilder};

    let children = hiearchy
        .get_children()
        .expect("TODO") //TODO: Handle
        .collect_vec();

    let selected_group = if let Some(selection_idx) = selected_indices.get(current_depth) {
        id = id.with(selection_idx);
        if let Some(lazy_hierarchy::NodeRef::Group(g)) = &children.get(*selection_idx) { // TODO: Make sure this always succesds
            Some(g)
        } else {
            None
        }
    } else {
        None
    };

    let selection_modified_rn = ui.push_id(current_depth, |ui| {
        let group_row_size =
            ui.style().text_styles[&TextStyle::Body].size + ui.style().spacing.item_spacing.y * 2.0;

        show_hiearchy_list(
            TableBuilder::new(ui)
                .column(if selected_group.is_some() {
                    Column::exact(256.0)
                } else {
                    Column::remainder().at_least(256.0)
                })
                .sense(Sense::click()),
            selected_indices,
            current_depth,
            group_row_size,
            image_scale,
            &children,
            id.with("list")
        )
    }).inner;

    let selection_modified_rec = if let Some(g) = selected_group {
        show_hiearchy_inner(
            ui,
            g,
            selected_indices,
            current_depth + 1,
            flatten_mode,
            image_scale,
            id.with("next")
        )
    } else { false };
    selection_modified_rec || selection_modified_rn
}

pub(crate) fn error_ui(ui: &mut egui::Ui, error: &str) {
    ui.colored_label(ui.visuals().error_fg_color, format!("⊗ {}", error));
}

fn widgetvisuals_to_frame(
    visuals: &egui::style::WidgetVisuals,
    button_padding: Vec2,
) -> egui::Frame {
    egui::Frame {
        fill: visuals.bg_fill,
        stroke: visuals.bg_stroke,
        inner_margin: Margin::symmetric(
            (button_padding.x - visuals.bg_stroke.width) as i8,
            (button_padding.y - visuals.bg_stroke.width) as i8,
        ),
        corner_radius: visuals.corner_radius,
        ..egui::Frame::NONE
    }
}

pub fn big_btn(ui: &mut egui::Ui, icon: &str, heading: &str, description: &str) -> egui::Response {
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

impl WelcomePage {}

impl App {
    /// Panics if `self.app_content` is not WelcomePage
    fn draw_welcome_page(&mut self, ctx: &egui::Context) {
        let AppContent::WelcomePage(ref welcome_page) = self.content else {
            panic!("`self.app_content` is not `WelcomePage`")
        };

        let welcome_page = welcome_page.to_owned();

        egui::SidePanel::left("recents")
            .frame(Frame::default().inner_margin(Margin::same(32)))
            .show(ctx, |ui| {
                ui.style_mut().spacing.button_padding = Vec2::new(32.0, 16.0);

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

                if !matches!(welcome_page, WelcomePage::Loading(_)) {
                    //ui.add_space(ctx.style().text_styles[&TextStyle::Heading].size);

                    self.pick_file_btn(ui);
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

                match welcome_page {
                    WelcomePage::Normal => {}
                    WelcomePage::Loading(msg) => { // TODO: Remove
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(&*msg);
                        });
                    }
                }
            });

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(Margin::same(32)),
            )
            .show(ctx, |ui| {
                // TODO: Actually implement
                ui.heading("Recent projects");

                ui.label("No recently opened projects\nUse the left panel to open a folder with your photos")
            });
    }
    fn pick_file_btn(&mut self, ui: &mut egui::Ui) {
        let clicked = big_btn(
            ui,
            "🗁",
            "Pick a folder",
            "Locate the photo collection you wish to sort",
        )
        .clicked();
        if clicked {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.load_dir(folder);
            }
        };
    }
}

#[derive(Debug)]
pub enum Message {
    SetContent(AppContent),
    // TODO: Remove
    Sorted {
        new_hiearchy: backend::main_hierarchy::TemplateHiearchy,
    },
    SetProgress(Option<Progress>),
}

#[derive(Error, Debug)]
pub enum AppWideError {
    #[error(transparent)]
    SelectedNotFileNorDir(#[from] backend::fs_hierarchy::ErrFileNorDir),
}

pub struct App {
    pub content: AppContent,
    pub inbox: UiInbox<Message>,
    pub style_manager: style::Manager,
    pub style_changed: bool,
    pub errors: Vec<AppWideError>, // TODO: Show it on every page (every variant of AppContent)
    mean_cpu_usage: f32
}

impl App {
    pub fn new(args: CliArgs) -> Self {
        let mut app = Self {
            content: gui::AppContent::default(),
            inbox: UiInbox::default(),
            style_manager: style::Manager::new_from_os(),
            style_changed: false,
            errors: Vec::new(),
            mean_cpu_usage: 1.0,
        };

        let AppContent::WelcomePage(ref mut welcome_page) = app.content else {
            panic!(
                "App::default().content has is not a welcome page, it's {:?}",
                app.content
            )
        };

        if let Some(dir) = args.dir {
            app.load_dir(dir);
        }

        app
    }
}

#[derive(Debug, From)]
pub enum AppContent {
    WelcomePage(WelcomePage),
    MainPage(MainPage),
}

#[derive(Debug)]
struct MainPage {
    pane: Option<PaneContent>,
    hiearchy: backend::main_hierarchy::TemplateHiearchy,
    flatten_mode: Option<FlattenMode>, // TODO: Remove
    selection: Vec<backend::selection::PathComponent>,
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

// NOTE: As of writing this, WelcomePage is cloned every frame in draw_welcome_page, be careful in increasing its size
#[derive(Debug, Clone)]
enum WelcomePage {
    Normal,
    Loading(String),
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
    /// Panics on `!folder.is_dir()`
    fn new(
        //hiearchy: Vec<geogroup_backend::HiearchyItem<PathBuf, PathBuf>>,
        folder: PathBuf,
    ) -> MainPage {
        assert!(
            folder.is_dir(),
            "MainPage::new called with non-directory as an argument: {}",
            folder.display()
        );

        let hierarchy_template = backend::main_hierarchy::lazy_group::Template::Sorted {
            item_source: ItemSource::EntireDir(folder.into()),
            params: algorithm::Params::default()
        };

        log::debug!("Created root group template");
        let root_group = main_hierarchy::lazy_group::Dynamic::new(hierarchy_template);

        log::debug!("Created root group");

        MainPage {
            hiearchy: root_group.into(),
            /*hiearchy
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
