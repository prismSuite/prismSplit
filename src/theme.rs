use egui::{vec2, Color32, Stroke, Rounding, epaint::Shadow, Margin};
use std::path::PathBuf;

fn get_system_font_data() -> Option<Vec<u8>> {
    let paths = if cfg!(target_os = "windows") {
        vec![
            PathBuf::from("C:\\Windows\\Fonts\\tahoma.ttf"),
            PathBuf::from("C:\\Windows\\Fonts\\segoeui.ttf"),
            PathBuf::from("C:\\Windows\\Fonts\\arial.ttf"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/System/Library/Fonts/LucidaGrande.ttc"),
            PathBuf::from("/System/Library/Fonts/Helvetica.ttc"),
            PathBuf::from("/Library/Fonts/Arial.ttf"),
        ]
    } else {
        // Linux
        vec![
            PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
            PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"),
        ]
    };

    for path in paths {
        if path.is_file() {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }
    }
    None
}

pub fn apply_prismsplit_theme(ctx: &egui::Context, dark_mode: bool) {
    // 1. Setup system fonts
    if let Some(font_data) = get_system_font_data() {
        let mut fonts = egui::FontDefinitions::default();
        
        fonts.font_data.insert(
            "system_font".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "system_font".to_owned());
            
        fonts.families.get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "system_font".to_owned());
            
        ctx.set_fonts(fonts);
    }

    let mut style = (*ctx.style()).clone();
    
    // Set Tahoma-like sizes
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(12.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(12.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Small, egui::FontId::new(11.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(16.0, egui::FontFamily::Proportional));

    // 2. Setup Visuals
    let mut visuals = if dark_mode {
        let mut vis = egui::Visuals::dark();
        
        // Backgrounds & Surfaces (Dark Monolith)
        vis.panel_fill = Color32::from_rgb(10, 10, 10);
        vis.window_fill = Color32::from_rgb(5, 5, 5);
        vis.extreme_bg_color = Color32::from_rgb(17, 17, 17);
        vis.faint_bg_color = Color32::from_rgb(24, 24, 24);
        
        // Text
        vis.override_text_color = Some(Color32::from_rgb(235, 235, 240));
        
        // Selection / Brand
        vis.selection.bg_fill = Color32::from_rgba_unmultiplied(34, 211, 238, 30);
        vis.selection.stroke = Stroke::new(1.0, Color32::from_rgb(34, 211, 238));
        
        // Widgets inactive
        vis.widgets.inactive.bg_fill = Color32::from_rgb(32, 32, 32);
        vis.widgets.inactive.weak_bg_fill = Color32::from_rgb(24, 24, 24);
        vis.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12));
        vis.widgets.inactive.rounding = Rounding::ZERO;
        
        // Widgets hovered
        vis.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 40);
        vis.widgets.hovered.weak_bg_fill = Color32::from_rgb(32, 32, 32);
        vis.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
        vis.widgets.hovered.rounding = Rounding::ZERO;
        
        // Widgets active
        vis.widgets.active.bg_fill = Color32::from_rgb(17, 17, 17);
        vis.widgets.active.weak_bg_fill = Color32::from_rgb(17, 17, 17);
        vis.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(34, 211, 238, 255));
        vis.widgets.active.rounding = Rounding::ZERO;
        
        vis
    } else {
        let mut vis = egui::Visuals::light();
        
        // Backgrounds & Surfaces (Light Monolith / Industrial Brutalism)
        vis.panel_fill = Color32::from_rgb(240, 240, 240);
        vis.window_fill = Color32::from_rgb(250, 250, 250);
        vis.extreme_bg_color = Color32::from_rgb(255, 255, 255);
        vis.faint_bg_color = Color32::from_rgb(225, 225, 225);
        
        // Text (Dark contrast)
        vis.override_text_color = Some(Color32::from_rgb(15, 15, 20));
        
        // Selection / Brand
        vis.selection.bg_fill = Color32::from_rgba_unmultiplied(34, 211, 238, 50);
        vis.selection.stroke = Stroke::new(1.0, Color32::from_rgb(8, 145, 178));
        
        // Widgets inactive
        vis.widgets.inactive.bg_fill = Color32::from_rgb(220, 220, 220);
        vis.widgets.inactive.weak_bg_fill = Color32::from_rgb(230, 230, 230);
        vis.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 30));
        vis.widgets.inactive.rounding = Rounding::ZERO;
        
        // Widgets hovered
        vis.widgets.hovered.bg_fill = Color32::from_rgb(205, 205, 205);
        vis.widgets.hovered.weak_bg_fill = Color32::from_rgb(215, 215, 215);
        vis.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 50));
        vis.widgets.hovered.rounding = Rounding::ZERO;
        
        // Widgets active
        vis.widgets.active.bg_fill = Color32::from_rgb(245, 245, 245);
        vis.widgets.active.weak_bg_fill = Color32::from_rgb(245, 245, 245);
        vis.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(8, 145, 178, 255));
        vis.widgets.active.rounding = Rounding::ZERO;
        
        vis
    };

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
    
    // Animation/Motion
    style.animation_time = 0.16;

    ctx.set_style(style);
}