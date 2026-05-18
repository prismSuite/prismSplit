use egui::{vec2, Color32, Stroke, Rounding, epaint::Shadow, Margin};

pub fn apply_prismsplit_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // MonolithUI Tokens
    // --ui-surface-0: #060608
    // --ui-surface-1: #0c0e12
    // --ui-surface-2: #131720
    // --ui-surface-3: #1c2130
    // --ui-surface-4: #252a3a
    // --ui-surface-5: #2e3445
    // --ui-surface-6: #3a4055
    // brand-plasma-core primary: #00d2ff

    // Backgrounds & Surfaces
    visuals.panel_fill = Color32::from_rgb(12, 14, 18); // surface-1 (panels, nav)
    visuals.window_fill = Color32::from_rgb(6, 6, 8); // surface-0 (deepest)
    visuals.extreme_bg_color = Color32::from_rgb(19, 23, 32); // surface-2 (inputs)
    visuals.faint_bg_color = Color32::from_rgb(28, 33, 48); // surface-3 

    // Text
    visuals.override_text_color = Some(Color32::from_rgb(230, 230, 235));

    // Selection / Brand
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 210, 255, 30);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 210, 255));

    // Widgets inactive (Surface 4)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(37, 42, 58); // surface-4
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(28, 33, 48); // surface-3
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 15)); // top edge light approximation
    visuals.widgets.inactive.rounding = Rounding::same(6.0); // ui-r-md

    // Widgets hovered (Surface 5)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(46, 52, 69); // surface-5
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(37, 42, 58); // surface-4
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25));
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    // Widgets active (Sunken)
    visuals.widgets.active.bg_fill = Color32::from_rgb(19, 23, 32); // surface-2
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(19, 23, 32); 
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 210, 255, 255)); // brand glow
    visuals.widgets.active.rounding = Rounding::same(6.0);

    // Shadows
    visuals.window_shadow = Shadow {
        offset: vec2(0.0, 8.0),
        blur: 64.0, // spread/blur
        spread: 0.0,
        color: Color32::from_rgba_unmultiplied(0, 0, 0, 50),
    };
    visuals.popup_shadow = Shadow {
        offset: vec2(0.0, 4.0),
        blur: 48.0,
        spread: 0.0,
        color: Color32::from_rgba_unmultiplied(0, 0, 0, 40),
    };

    style.visuals = visuals;
    
    // Spacing & Padding
    style.spacing.item_spacing = vec2(12.0, 12.0);
    style.spacing.button_padding = vec2(16.0, 8.0);
    style.spacing.window_margin = Margin::same(16.0);
    
    // Animation/Motion - Try to make it feel slightly more responsive
    style.animation_time = 0.16; // 160ms for fast UI feedback

    ctx.set_style(style);
}