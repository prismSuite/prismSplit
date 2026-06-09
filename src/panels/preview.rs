use egui::{Color32, RichText};
use crate::app::PrismSplitApp;

pub fn show(app: &mut PrismSplitApp, ctx: &egui::Context) {
    let Some(stems) = app.state.active_preview_stems.clone() else {
        return;
    };

    egui::Window::new("▸ PRISMPREVIEW // LIVE STEMS VISUALIZER")
        .collapsible(false)
        .resizable(true)
        .default_width(850.0)
        .default_height(480.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.colored_label(
                    Color32::from_rgb(34, 211, 238),
                    RichText::new("● PREVIEW MODE ACTIVE").monospace().strong(),
                );
                ui.separator();
                ui.label(RichText::new("Stems separated in temporary environment.").monospace().small());
            });
            
            ui.add_space(10.0);

            let mut stem_to_play: Option<String> = None;
            let mut stem_to_save: Option<(String, String)> = None;

            egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                for stem in stems.iter() {
                    crate::widgets::fieldset(ui, &stem.name, |ui| {
                        ui.horizontal(|ui| {
                            let btn_label = if stem.is_playing { "⏸ PAUSE" } else { "▶ PLAY" };
                            let btn_color = if stem.is_playing {
                                Color32::from_rgb(245, 158, 11)
                            } else {
                                Color32::from_rgb(34, 211, 238)
                            };
                            
                            let play_btn = egui::Button::new(RichText::new(btn_label).monospace().strong().color(Color32::from_rgb(5, 5, 5)))
                                .fill(btn_color)
                                .min_size(egui::vec2(85.0, 34.0));
                            
                            if ui.add(play_btn).clicked() {
                                  stem_to_play = Some(stem.id.clone());
                            }

                            ui.add_space(6.0);

                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width() - 95.0, 48.0),
                                egui::Sense::hover(),
                            );
                            
                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 0.0, Color32::from_rgb(17, 17, 17));
                            
                            let grid_stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 4));
                            for x in (rect.left() as i32..rect.right() as i32).step_by(24) {
                                painter.line_segment([egui::pos2(x as f32, rect.top()), egui::pos2(x as f32, rect.bottom())], grid_stroke);
                            }
                            painter.line_segment(
                                [egui::pos2(rect.left(), rect.center().y), egui::pos2(rect.right(), rect.center().y)],
                                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 8)),
                            );

                            let peak_count = stem.peaks.len();
                            if peak_count > 0 {
                                let active_color = if stem.is_playing {
                                    Color32::from_rgb(34, 211, 238)
                                } else {
                                    Color32::from_rgb(120, 130, 145)
                                };

                                let shapes = {
                                    let mut cache = stem.cached_shapes.lock().unwrap();
                                    let needs_rebuild = match &*cache {
                                        Some((cached_rect, cached_playing, _)) => *cached_rect != rect || *cached_playing != stem.is_playing,
                                        None => true,
                                    };
                                    if needs_rebuild {
                                        let bar_width = rect.width() / (peak_count as f32);
                                        let mut new_shapes = Vec::with_capacity(peak_count);
                                        for (i, &amp) in stem.peaks.iter().enumerate() {
                                            let x = rect.left() + (i as f32) * bar_width + (bar_width / 2.0);
                                            let half_height = (amp * (rect.height() / 2.0)) * 0.95;
                                            let top_y = rect.center().y - half_height;
                                            let bottom_y = rect.center().y + half_height;
                                            new_shapes.push(egui::Shape::line_segment(
                                                [egui::pos2(x, top_y), egui::pos2(x, bottom_y)],
                                                egui::Stroke::new(bar_width * 0.8, active_color),
                                            ));
                                        }
                                        *cache = Some((rect, stem.is_playing, new_shapes.clone()));
                                        new_shapes
                                    } else {
                                        cache.as_ref().unwrap().2.clone()
                                    }
                                };
                                
                                painter.extend(shapes);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let save_btn = egui::Button::new(RichText::new("💾 SAVE").monospace().small())
                                    .min_size(egui::vec2(65.0, 34.0));
                                if ui.add(save_btn).clicked() {
                                    stem_to_save = Some((stem.id.clone(), stem.file_path.clone()));
                                }
                            });
                        });
                    });
                    ui.add_space(6.0);
                }
            });

            ui.separator();
            ui.add_space(6.0);

            ui.columns(2, |columns| {
                if crate::widgets::custom_button(
                    &mut columns[0],
                    "💾 EXPORT ALL STEMS (GUARDAR)",
                    true,
                    Color32::from_rgb(34, 211, 238),
                ) {
                    app.save_all_preview_stems(&stems);
                }

                if crate::widgets::custom_button(
                    &mut columns[1],
                    "✕ CLOSE PREVIEW WINDOW",
                    true,
                    Color32::from_rgb(180, 187, 193),
                ) {
                    app.close_preview_window(&stems);
                }
            });

            if let Some(id) = stem_to_play {
                let mut updated_stems = stems.clone();
                let mut current_playing_id = app.state.current_playing_stem.clone();
                
                for s in &mut updated_stems {
                    if s.id == id {
                        if s.is_playing {
                            s.is_playing = false;
                            app.playback_controller.stop();
                            current_playing_id = None;
                        } else {
                            s.is_playing = true;
                            if let Err(e) = app.playback_controller.play(&s.file_path) {
                                app.state.push_log(format!("ERROR: Failed to play audio: {}", e));
                            } else {
                                current_playing_id = Some(id.clone());
                            }
                        }
                    } else {
                        s.is_playing = false;
                    }
                }
                
                app.state.active_preview_stems = Some(updated_stems);
                app.state.current_playing_stem = current_playing_id;
            }

            if let Some((id, path)) = stem_to_save {
                let extension = std::path::Path::new(&path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("wav");
                let default_filename = if id == "vocals" {
                    "Separated_Vocals"
                } else {
                    "Separated_Instrumental"
                };

                if let Some(target) = rfd::FileDialog::new()
                    .set_file_name(format!("{}.{}", default_filename, extension))
                    .add_filter("Audio", &[extension])
                    .save_file()
                {
                    let target_path = target.display().to_string();
                    match app.backend.copy_file(&path, &target_path) {
                        Ok(()) => {
                            app.state.push_log(format!("SUCCESS: Saved stem to <{}>.", target_path));
                        }
                        Err(e) => {
                            app.state.push_log(format!("ERROR: Failed to save stem: {}", e));
                        }
                    }
                }
            }
        });
}
