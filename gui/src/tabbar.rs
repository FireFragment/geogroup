use super::*;
use egui::Frame;

pub fn tabbar<'a, TabId: PartialEq>(
    ui: &mut Ui,
    selected_tab: &mut Option<TabId>,
    tabs: impl IntoIterator<Item = (TabId, &'a str)>,
) {
    let tab_bar_bg = ui.style().visuals.selection.bg_fill;

    egui::Frame::none().fill(tab_bar_bg).show(ui, |ui| {
        let style = ui.style_mut();
        style::set_fg_color(style, style.visuals.selection.stroke.color);

        style.visuals.selection.bg_fill = style.visuals.panel_fill;
        style.visuals.selection.stroke.color = tab_bar_bg/*.lerp_to_gamma(Color32::WHITE, 0.5)*/; // TODO: Light mode

        style.visuals.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.hovered.expansion = 0.0;
        style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        style.visuals.widgets.hovered.fg_stroke.color = {
            let [r, g, b, _] = style.visuals.widgets.hovered.fg_stroke.color.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, u8::MAX / 4)
        };

        style.visuals.widgets.active.bg_fill = {
            let [r, g, b, _] = style.visuals.panel_fill.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, u8::MAX / 4)
        };
        style.visuals.widgets.active.weak_bg_fill = style.visuals.widgets.active.bg_fill;
        style.visuals.widgets.active.expansion = 0.0;
        style.visuals.widgets.active.bg_stroke = Stroke::NONE;

        ui.horizontal(|ui| {
            ui.add_space(ui.style().spacing.item_spacing.x);
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.heading("Geogroup");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    for (tab_id, label) in tabs {
                        ui.selectable_value(selected_tab, Some(tab_id), label);
                    }

                    if selected_tab.is_some() {
                        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                            if ui.add(Button::new("⬆").frame(false)).clicked() {
                                *selected_tab = None;
                            }
                        });
                    }

                    // Make the ribbon go full width even if there's no tab open
                    ui.allocate_space(ui.available_size());
                });
            })
        });
    });
}
