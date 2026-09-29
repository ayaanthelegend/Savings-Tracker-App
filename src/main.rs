#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ledger;
mod models;
mod notification;
mod savings;
mod storage;
mod subscriptions;
mod ui;

use ui::theme::Theme;
use ui::SavingsTrackerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Savings Tracker")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([800.0, 520.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Savings Tracker",
        native_options,
        Box::new(|cc| {
            // Load and register the exact fonts: Space Grotesk, Inter, and IBM Plex Mono
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "SpaceGrotesk".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/fonts/SpaceGrotesk.ttf")),
            );
            fonts.font_data.insert(
                "Inter".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/fonts/Inter.ttf")),
            );
            fonts.font_data.insert(
                "IBMPlexMono".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/fonts/IBMPlexMono.ttf")),
            );

            // Register named font families
            fonts.families.insert(
                egui::FontFamily::Name("SpaceGrotesk".into()),
                vec!["SpaceGrotesk".to_owned(), "Inter".to_owned()],
            );
            fonts.families.insert(
                egui::FontFamily::Name("Inter".into()),
                vec!["Inter".to_owned()],
            );
            fonts.families.insert(
                egui::FontFamily::Name("IBMPlexMono".into()),
                vec!["IBMPlexMono".to_owned()],
            );

            // Proportional: Inter
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "Inter".to_owned());

            // Monospace: IBM Plex Mono
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "IBMPlexMono".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            // Configure explicit TextStyle mappings so headings and body use custom fonts
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::new(19.0, egui::FontFamily::Name("SpaceGrotesk".into()))),
                (egui::TextStyle::Name("TopTitle".into()), egui::FontId::new(15.5, egui::FontFamily::Name("SpaceGrotesk".into()))),
                (egui::TextStyle::Name("Brand".into()), egui::FontId::new(16.0, egui::FontFamily::Name("SpaceGrotesk".into()))),
                (egui::TextStyle::Name("CardTitle".into()), egui::FontId::new(15.0, egui::FontFamily::Name("SpaceGrotesk".into()))),
                (egui::TextStyle::Body, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Small, egui::FontId::new(11.5, egui::FontFamily::Proportional)),
                (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Name("IBMPlexMono".into()))),
                (egui::TextStyle::Name("Money".into()), egui::FontId::new(13.0, egui::FontFamily::Name("IBMPlexMono".into()))),
                (egui::TextStyle::Name("MoneyLarge".into()), egui::FontId::new(16.0, egui::FontFamily::Name("IBMPlexMono".into()))),
                (egui::TextStyle::Name("MoneyStat".into()), egui::FontId::new(18.0, egui::FontFamily::Name("IBMPlexMono".into()))),
            ].into();

            // Configure exact visuals matching approved reference
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Theme::BG;
            visuals.window_fill = Theme::PANEL;
            visuals.window_stroke = Theme::stroke_border();
            visuals.window_rounding = egui::Rounding::same(Theme::RADIUS_PANEL);
            visuals.override_text_color = Some(Theme::TEXT);

            // Selection & interaction accents
            visuals.selection.bg_fill = Theme::ACCENT_DIM;
            visuals.selection.stroke = egui::Stroke::new(1.0_f32, Theme::ACCENT);

            // Widget styling (all buttons and controls) - sleek, dark, soft with clear readable text
            visuals.widgets.inactive.bg_fill = Theme::PANEL2;
            visuals.widgets.inactive.bg_stroke = Theme::stroke_border();
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, Theme::TEXT);
            visuals.widgets.inactive.rounding = egui::Rounding::same(Theme::RADIUS_BTN);

            visuals.widgets.hovered.bg_fill = Theme::ACCENT_DIM;
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Theme::ACCENT);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.hovered.rounding = egui::Rounding::same(Theme::RADIUS_BTN);

            visuals.widgets.active.bg_fill = Theme::ACCENT_DIM;
            visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, Theme::ACCENT);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.active.rounding = egui::Rounding::same(Theme::RADIUS_BTN);

            style.visuals = visuals;
            cc.egui_ctx.set_style(style);

            Ok(Box::new(SavingsTrackerApp::new(cc)))
        }),
    )
}
