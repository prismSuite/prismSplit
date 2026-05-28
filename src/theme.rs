use egui::{vec2, Color32, Stroke, Rounding, epaint::Shadow, Margin};

pub fn apply_prismsplit_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // MonolithUI v1.0 Tokens
    // --ui-surface-0 (Pure Deep): #050505
    // --ui-surface-1 (Panel): #0a0a0a
    // --ui-surface-2 (Wells/Inputs): #111111
    // --ui-surface-3 (Faint BG): #181818
    // --ui-surface-4 (Widgets Inactive): #202020
    // --ui-surface-5 (Widgets Hovered): #282828
    // --ui-surface-6 (Highlight Borders): #323232
    // brand-plasma-core primary (Electric Cyan): #22d3ee

    // Backgrounds & Surfaces
    visuals.panel_fill = Color32::from_rgb(10, 10, 10); // surface-1 (panels, nav)
    visuals.window_fill = Color32::from_rgb(5, 5, 5); // surface-0 (deepest)
    visuals.extreme_bg_color = Color32::from_rgb(17, 17, 17); // surface-2 (inputs)
    visuals.faint_bg_color = Color32::from_rgb(24, 24, 24); // surface-3 

    // Text
    visuals.override_text_color = Some(Color32::from_rgb(235, 235, 240));

    // Selection / Brand
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(34, 211, 238, 30);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(34, 211, 238));

    // Widgets inactive (Surface 4)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 32, 32); // surface-4
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(24, 24, 24); // surface-3
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12)); // subtle top lit edge
    visuals.widgets.inactive.rounding = Rounding::ZERO; // retro Win32 90 deg corners

    // Widgets hovered (Surface 5)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 40); // surface-5
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(32, 32, 32); // surface-4
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    visuals.widgets.hovered.rounding = Rounding::ZERO;

    // Widgets active (Sunken)
    visuals.widgets.active.bg_fill = Color32::from_rgb(17, 17, 17); // surface-2
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(17, 17, 17); 
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(34, 211, 238, 255)); // brand glow
    visuals.widgets.active.rounding = Rounding::ZERO;

    // Shadows (Disabled for flat retro-brutalist DAW skeuomorphism)
    visuals.window_shadow = Shadow {
        offset: vec2(0.0, 0.0),
        blur: 0.0, 
        spread: 0.0,
        color: Color32::TRANSPARENT,
    };
    visuals.popup_shadow = Shadow {
        offset: vec2(0.0, 0.0),
        blur: 0.0,
        spread: 0.0,
        color: Color32::TRANSPARENT,
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