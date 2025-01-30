use std::sync::Arc;

use eframe::egui::{
    self, Color32, Context, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Theme, Vec2,
};

#[derive(Debug)]
pub struct Params {
    pub accent_color: Color32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            accent_color: Color32::from_rgb(186, 32, 68),
        }
    }
}

pub fn set_fg_color(style: &mut Style, color: Color32) {
    style.visuals.widgets.active.fg_stroke.color = color;
    style.visuals.widgets.hovered.fg_stroke.color = color;
    style.visuals.widgets.inactive.fg_stroke.color = color;
    style.visuals.widgets.noninteractive.fg_stroke.color = color;
    style.visuals.widgets.open.fg_stroke.color = color;
}

pub fn apply(ctx: &Context, params: &Params) {
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "Segoe UI".to_owned(),
        // .ttf and .otf supported
        egui::FontData::from_static(include_bytes!(
            "/nix/store/p5sizlrzxqxa2jp194xwzpgzka7mhkmf-Segoe-UI/share/fonts/Segoe UI/segoe-ui.otf"
        )),
    );

    fonts.font_data.insert(
        "Segoe UI Bold".to_owned(),
        // .ttf and .otf supported
        egui::FontData::from_static(include_bytes!(
            "/nix/store/p5sizlrzxqxa2jp194xwzpgzka7mhkmf-Segoe-UI/share/fonts/Segoe UI/segoe-ui-bold.otf"
        )),
    );

    fonts.font_data.insert(
        "Segoe UI Light".to_owned(),
        // .ttf and .otf supported
        egui::FontData::from_static(include_bytes!(
            "/nix/store/p5sizlrzxqxa2jp194xwzpgzka7mhkmf-Segoe-UI/share/fonts/Segoe UI/segoe-ui-light-2.ttf"
        )),
    );
    fonts.font_data.insert(
        "Figtree".to_owned(),
        // .ttf and .otf supported
        egui::FontData::from_static(include_bytes!(
            "/nix/store/frkc22297c6z2rrl10m9p47lixl303fs-figtree/share/fonts/figtree/figtree-v6-latin_latin-ext-regular.ttf"
        )),
    );

    fonts.font_data.insert(
        "Figtree Bold".to_owned(),
        // .ttf and .otf supported
        egui::FontData::from_static(include_bytes!(
            "/nix/store/frkc22297c6z2rrl10m9p47lixl303fs-figtree/share/fonts/figtree/figtree-v6-latin_latin-ext-800.ttf"
        )),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "Segoe UI".to_owned());

    fonts.families.insert(
        egui::FontFamily::Name("Light".into()),
        vec!["Segoe UI Light".to_owned()],
    );
    fonts.families.insert(
        egui::FontFamily::Name("Bold".into()),
        vec!["Segoe UI Bold".to_owned()],
    );

    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Name("Bold".into())),
        );

        if style.visuals.dark_mode {
            style.visuals.panel_fill = Color32::BLACK; //Color32::from_rgb(28, 28, 38);

            style.visuals.widgets.inactive.bg_fill =
                Color32::from_rgb(28, 28, 38).gamma_multiply(1.5); //Color32::from_rgb(42, 42, 64);
            style.visuals.widgets.inactive.weak_bg_fill = style.visuals.widgets.inactive.bg_fill;

            style.visuals.widgets.hovered.bg_fill = style.visuals.widgets.inactive.bg_fill;
            style.visuals.widgets.hovered.weak_bg_fill = style.visuals.widgets.hovered.bg_fill;
            style.visuals.widgets.hovered.bg_stroke.color =
                style.visuals.widgets.inactive.bg_fill.gamma_multiply(1.5);

            set_fg_color(style, Color32::WHITE);
        } else {
            set_fg_color(style, Color32::BLACK);
        }

        style.visuals.widgets.hovered.expansion = -2.0;
        style.visuals.widgets.hovered.bg_stroke.width = 2.0;
        style.visuals.widgets.active.expansion = 0.0;
        style.visuals.widgets.active.bg_fill = style.visuals.widgets.hovered.bg_stroke.color;
        style.visuals.widgets.active.weak_bg_fill = style.visuals.widgets.active.bg_fill;
        style.visuals.widgets.active.bg_stroke.color = Color32::TRANSPARENT;

        style.visuals.selection.bg_fill = params.accent_color;
        //style.visuals.selection.bg_fill = Color32::from_rgb(193, 113, 34);
        //style.visuals.selection.bg_fill = Color32::from_rgb(135, 8, 131);

        style.visuals.selection.stroke.color = if params.accent_color.g() > 128 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };

        style.spacing.button_padding = Vec2::new(12.0, 6.0);

        // ROUNDING
        style.visuals.window_rounding = Rounding::ZERO;
        style.visuals.menu_rounding = Rounding::ZERO;
        style.visuals.widgets.active.rounding = Rounding::ZERO;
        style.visuals.widgets.hovered.rounding = Rounding::ZERO;
        style.visuals.widgets.inactive.rounding = Rounding::ZERO;
        style.visuals.widgets.noninteractive.rounding = Rounding::ZERO;
        style.visuals.widgets.open.rounding = Rounding::ZERO;

        // MISCELLANEOUS
        style.interaction.selectable_labels = false;
        style.animation_time = 0.2;
        style.visuals.slider_trailing_fill = true;
    });
}
