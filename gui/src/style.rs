use std::{sync::{mpsc, Arc}, time::Duration};

const DEFAULT_ACCENT_COLOR: Color32 = Color32::from_rgb(67, 100, 188);

use eframe::egui::{
    self, Color32, Context, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Theme, Vec2,
};

pub struct Manager {
    accent_color_stream: mpsc::Receiver<Color32>,
    subscription: mundy::Subscription
}

impl Manager {
    pub fn new_from_os() -> Self {
        let (tx, rx) = mpsc::channel();
        Manager {
        /*accent_color: mundy::Preferences::once_blocking(mundy::Interest::AccentColor, Duration::from_millis(500))
                .map(|preferences| preferences.accent_color.0)
                .flatten()
                .map(|c| Color32::from_rgb(
                    (c.red * u8::MAX as f64) as u8,
                    (c.green * u8::MAX as f64) as u8,
                    (c.blue * u8::MAX as f64) as u8
                ))
                .unwrap_or_else(|| Color32::from_rgb(67, 100, 188)),*/
            accent_color_stream: rx,
            subscription: mundy::Preferences::subscribe(mundy::Interest::AccentColor, move |preferences| {
                let Some(c) = preferences.accent_color.0 else {
                    log::error!("Got no accent color");
                    return
                };
                tx.send(Color32::from_rgb(
                    (c.red * u8::MAX as f64) as u8,
                    (c.green * u8::MAX as f64) as u8,
                    (c.blue * u8::MAX as f64) as u8
                )).unwrap_or_else(|err| {
                    log::error!("Error sending a new accent color: {err}");
                });
            })
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

fn load_fonts(ctx: &Context) {
    const FONT_REGULAR: &str = "System Sans Serif";
    const FONT_BOLD: &str = "System Sans Serif Bold";
    const FONT_LIGHT: &str = "System Sans Serif Light";

    use eframe::{
        egui::{FontData, FontDefinitions},
        epaint::FontFamily,
    };
    use font_kit::{
        family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
    };

    let mut fonts = FontDefinitions::default();

    {
        let handle = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title("Segoe UI".into()), FamilyName::SansSerif],
                &Properties::new(),
            )
            .unwrap();

        let buf: Vec<u8> = match handle {
            Handle::Memory { bytes, .. } => bytes.to_vec(),
            Handle::Path { path, .. } => std::fs::read(path).unwrap(),
        };

        fonts
            .font_data
            .insert(FONT_REGULAR.to_owned(), FontData::from_owned(buf).into());
    }
    {
        let handle = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title("Segoe UI".into()), FamilyName::SansSerif],
                Properties::new().weight(font_kit::properties::Weight(700.0)),
            )
            .unwrap();

        let buf: Vec<u8> = match handle {
            Handle::Memory { bytes, .. } => bytes.to_vec(),
            Handle::Path { path, .. } => std::fs::read(path).unwrap(),
        };

        fonts
            .font_data
            .insert(FONT_BOLD.to_owned(), FontData::from_owned(buf).into());
    }
    {
        let handle = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title("Segoe UI".into()), FamilyName::SansSerif],
                Properties::new().weight(font_kit::properties::Weight(200.0)),
            )
            .unwrap();

        let buf: Vec<u8> = match handle {
            Handle::Memory { bytes, .. } => bytes.to_vec(),
            Handle::Path { path, .. } => std::fs::read(path).unwrap(),
        };

        fonts
            .font_data
            .insert(FONT_LIGHT.to_owned(), FontData::from_owned(buf).into());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.insert(0, FONT_REGULAR.to_owned());
    }

    fonts.families.insert(
        FontFamily::Name("Bold".into()),
        vec![
            FONT_BOLD.into(),
            FONT_REGULAR.into(),
            "emoji-icon-font".into(),
        ],
    );

    fonts.families.insert(
        FontFamily::Name("Light".into()),
        vec![
            FONT_LIGHT.into(),
            FONT_REGULAR.into(),
            "emoji-icon-font".into(),
        ],
    );

    ctx.set_fonts(fonts);
}

/// Apply only if needed, ie. something changed from the last call
pub fn possible_apply(ctx: &Context, params: &Manager) {
    if let Ok(color) = params.accent_color_stream.try_recv() {
        apply(ctx, color);
    }
}

/// Block and apply after having received a new accent color
///
/// Will not block for longer than 100ms
pub fn initial_apply(ctx: &Context, params: &Manager) {
    match params.accent_color_stream.recv_timeout(Duration::from_millis(100)) {
        Ok(color) => apply(ctx, color),
        Err(err) => {
            log::warn!("Failed to get accent color: {err}");
            apply(ctx, DEFAULT_ACCENT_COLOR)
        },
    }

}

pub fn apply(ctx: &Context, accent_color: Color32) {

    //ctx.set_fonts(fonts);

    load_fonts(ctx);

    ctx.all_styles_mut(|style| {
        *style.text_styles.get_mut(&TextStyle::Heading).unwrap() = FontId {
            size: 24.0,
            family: FontFamily::Name("Bold".into()),
        };

        for ts in [TextStyle::Body, TextStyle::Button, TextStyle::Monospace] {
            if let Some(s) = style.text_styles.get_mut(&ts) {
                s.size = 16.0;
            } else {
                debug_assert!(false, "`ts` is missing in style??")
            }
        }
        if style.visuals.dark_mode {
            style.visuals.panel_fill = Color32::from_rgb(28, 28, 33);

            style.visuals.widgets.inactive.bg_fill =
                Color32::from_rgb(28, 28, 38).gamma_multiply(1.5); //Color32::from_rgb(42, 42, 64);
            style.visuals.widgets.inactive.weak_bg_fill = style.visuals.widgets.inactive.bg_fill;

            style.visuals.widgets.hovered.bg_fill = style.visuals.widgets.inactive.bg_fill;
            style.visuals.widgets.hovered.weak_bg_fill = style.visuals.widgets.hovered.bg_fill;
            style.visuals.widgets.hovered.bg_stroke.color =
                style.visuals.widgets.inactive.bg_fill.gamma_multiply(1.5);

            style.visuals.faint_bg_color = Color32::GRAY.gamma_multiply(0.5);

            set_fg_color(style, Color32::WHITE);
        } else {
            set_fg_color(style, Color32::BLACK);
        }

        //style.visuals.widgets.hovered.expansion = -2.0;
        //style.visuals.widgets.hovered.bg_stroke.width = 2.0;
        //style.visuals.widgets.active.expansion = 0.0;
        style.visuals.widgets.active.bg_fill = style.visuals.widgets.hovered.bg_stroke.color;
        style.visuals.widgets.active.weak_bg_fill = style.visuals.widgets.active.bg_fill;
        style.visuals.widgets.active.bg_stroke.color = Color32::TRANSPARENT;

        style.visuals.selection.bg_fill = accent_color;
        style.visuals.hyperlink_color = accent_color;
        //style.visuals.selection.bg_fill = Color32::from_rgb(193, 113, 34);
        //style.visuals.selection.bg_fill = Color32::from_rgb(135, 8, 131);

        style.visuals.selection.stroke.color = if accent_color.g() > 128 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };

        //style.spacing.button_padding = Vec2::new(8.0, 4.0);

        // ROUNDING
        style.visuals.window_corner_radius = Rounding::ZERO;
        style.visuals.menu_corner_radius = Rounding::ZERO;
        style.visuals.widgets.active.corner_radius = Rounding::ZERO;
        style.visuals.widgets.hovered.corner_radius = Rounding::ZERO;
        style.visuals.widgets.inactive.corner_radius = Rounding::ZERO;
        style.visuals.widgets.noninteractive.corner_radius = Rounding::ZERO;
        style.visuals.widgets.open.corner_radius = Rounding::ZERO;

        // MISCELLANEOUS
        style.interaction.selectable_labels = false;
        style.interaction.tooltip_delay = 0.0;
        style.animation_time = 0.2;
        style.visuals.slider_trailing_fill = true;
    });
}
