use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

pub const RAIL: Color32 = Color32::from_rgb(0x13, 0x14, 0x18);
pub const SURFACE: Color32 = Color32::from_rgb(0x1E, 0x20, 0x27);
pub const RAISED: Color32 = Color32::from_rgb(0x2B, 0x2E, 0x38);
pub const HOVER: Color32 = Color32::from_rgb(0x25, 0x27, 0x30);
pub const ACCENT: Color32 = Color32::from_rgb(0x6C, 0x80, 0xF5);
pub const MENTION: Color32 = Color32::from_rgb(0xA3, 0xB1, 0xFF);
pub const TEXT: Color32 = Color32::from_rgb(0xF2, 0xF3, 0xF5);
pub const MUTED: Color32 = Color32::from_rgb(0x8E, 0x92, 0x9E);

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

/// Usa a fonte do sistema quando existe, para não embutir megabytes no binário.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    let mut proportional = fonts.families[&FontFamily::Proportional].clone();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts
            .font_data
            .insert("ui".into(), Arc::new(FontData::from_owned(bytes)));
        proportional.insert(0, "ui".into());
    }
    let mut strong = proportional.clone();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\seguisb.ttf") {
        fonts
            .font_data
            .insert("ui-semibold".into(), Arc::new(FontData::from_owned(bytes)));
        strong.insert(0, "ui-semibold".into());
    }
    fonts
        .families
        .insert(FontFamily::Proportional, proportional);
    fonts
        .families
        .insert(FontFamily::Name("semibold".into()), strong);
    ctx.set_fonts(fonts);

    ctx.set_visuals(egui::Visuals::dark());
}
