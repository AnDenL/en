use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Stroke, Visuals};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(30, 27, 46); 
pub const CARD_BG: Color32 = Color32::from_rgb(42, 38, 64);
pub const CARD_HOVER: Color32 = Color32::from_rgb(56, 51, 85);
pub const ACCENT: Color32 = Color32::from_rgb(242, 166, 90);
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(255, 192, 133);
pub const TEXT_MAIN: Color32 = Color32::from_rgb(234, 230, 240);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(154, 147, 166);

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    
    fonts.font_data.insert(
        "my_font".to_owned(),
        Arc::new(FontData::from_static(include_bytes!("../assets/Nunito-Medium.ttf"))),
    );

    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "my_font".to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
}

pub fn configure_styles(ctx: &Context) {
    let mut visuals = Visuals::dark();
    
    visuals.panel_fill = BG;
    visuals.window_fill = CARD_BG;
    
    visuals.widgets.inactive.weak_bg_fill = CARD_BG;
    visuals.widgets.inactive.bg_fill = CARD_BG;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MAIN);
    
    visuals.widgets.hovered.weak_bg_fill = CARD_HOVER;
    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, ACCENT);
    
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    visuals.selection.bg_fill = ACCENT.linear_multiply(0.5);
    
    ctx.set_visuals(visuals);
}

pub fn setup(ctx: &Context) {
    setup_custom_fonts(ctx);
    configure_styles(ctx);
}