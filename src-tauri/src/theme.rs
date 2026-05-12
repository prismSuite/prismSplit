use egui::{vec2, Color32, Stroke};

pub fn apply_prismsplit_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = Color32::from_rgb(18, 18, 18);
    visuals.window_fill = Color32::from_rgb(10, 10, 10);
    visuals.extreme_bg_color = Color32::from_rgb(24, 24, 24);
    visuals.faint_bg_color = Color32::from_rgb(16, 16, 16);
    visuals.override_text_color = Some(Color32::from_rgb(206, 214, 219));
    visuals.selection.bg_fill = Color32::from_rgb(45, 80, 22);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 255, 102));

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(44, 44, 44);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(32, 32, 32);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(86, 86, 86));
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(54, 54, 54);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(38, 38, 38);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(110, 110, 110));
    visuals.widgets.active.bg_fill = Color32::from_rgb(36, 36, 36);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(30, 30, 30);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 255, 102));

    style.visuals = visuals;
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(12.0);

    ctx.set_style(style);
}
