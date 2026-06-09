use crate::backend::Backend;
use crate::engine_bridge::EngineEvent;
use crate::panels::log_console;
use crate::panels::models::has_trusted_checksum;
use crate::state::{AppMsg, AppState, Tab};
use crate::widgets::{fieldset, nav_button, status_chip};
use eframe::egui::{self, Align, Color32, RichText};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

pub struct PrismSplitApp {
    pub(crate) backend: Arc<Backend>,
    pub(crate) runtime: Arc<tokio::runtime::Runtime>,
    pub(crate) state: AppState,
    pub(crate) playback_controller: crate::preview::PlaybackController,
    pub(crate) save_handle: Option<tokio::task::JoinHandle<()>>,
}

impl PrismSplitApp {
    pub fn new(
        backend: Arc<Backend>,
        runtime: Arc<tokio::runtime::Runtime>,
        storage: Option<&dyn eframe::Storage>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut state = AppState::new(tx, rx);

        if let Some(storage) = storage {
            if let Some(tab) = eframe::get_value::<Tab>(storage, "active_tab") {
                state.active_tab = tab;
            }
            if let Some(enable_preview) = eframe::get_value::<bool>(storage, "enable_preview") {
                state.enable_preview = enable_preview;
            }
        }

        let playback_controller = crate::preview::PlaybackController::new();
        let app = Self {
            backend,
            runtime,
            state,
            playback_controller,
            save_handle: None,
        };
        app.refresh_boot_state();
        app
    }

    fn refresh_boot_state(&self) {
        self.load_config();
        self.refresh_health();
    }

    fn load_config(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        self.runtime.spawn(async move {
            let config = backend.load_config();
            let _ = tx.send(AppMsg::ConfigLoaded(config));
        });
    }



    pub(crate) fn refresh_health(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        self.runtime.spawn(async move {
            let result = backend.get_engine_health().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::HealthLoaded(result));
        });
    }

    pub(crate) fn load_catalog(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        self.runtime.spawn(async move {
            let result = backend
                .list_model_catalog()
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::CatalogLoaded(result));
        });
    }

    pub(crate) fn prepare_engine(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let _ = tx.send(AppMsg::Log("INIT: Preparing embedded engine...".into()));
        self.runtime.spawn(async move {
            let result = backend.prepare_engine().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::SetupFinished(result));
        });
    }

    pub(crate) fn repair_engine(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let _ = tx.send(AppMsg::Log("INIT: Starting Smart Doctor Repair for engine...".into()));
        self.runtime.spawn(async move {
            let result = backend.repair_engine().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::SetupFinished(result));
        });
    }

    pub(crate) fn download_model(&self, model_id: String) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let tx_progress = tx.clone();
        let _ = tx.send(AppMsg::DownloadStarted(model_id.clone()));
        self.runtime.spawn(async move {
            let result = backend
                .download_model(model_id.clone(), move |progress| {
                    let _ = tx_progress.send(AppMsg::DownloadProgress {
                        model_id: model_id.clone(),
                        progress,
                    });
                })
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::DownloadFinished(result));
        });
    }

    pub(crate) fn sync_catalog(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let _ = tx.send(AppMsg::Log(
            "INIT: Synchronizing model catalog with UVR sources...".into(),
        ));
        self.runtime.spawn(async move {
            let result = backend.sync_uvr_catalog().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::CatalogSynced(result));
        });
    }

    pub(crate) fn scan_local_models(&self, path: String) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let _ = tx.send(AppMsg::Log(format!(
            "INIT: Scanning local directory [{}] for known models...",
            path
        )));
        self.runtime.spawn(async move {
            let result = backend
                .scan_local_models(path)
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::LocalScanFinished(result));
        });
    }

    pub(crate) fn process_audio(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let input_file = self.state.input_file.clone();
        let model = self.state.selected_model.clone();
        let output_dir = if self.state.enable_preview {
            std::env::temp_dir()
                .join("PrismSplit_Preview")
                .to_string_lossy()
                .to_string()
        } else if self.state.output_dir.trim().is_empty() {
            default_output_dir(&input_file)
        } else {
            self.state.output_dir.clone()
        };
        let quality = self.state.quality.clone();

        let _ = tx.send(AppMsg::Log(format!(
            "INIT: Starting separation for <{}>.",
            std::path::Path::new(&input_file)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(&input_file)
        )));

        self.runtime.spawn(async move {
            let tx_events = tx.clone();
            let result = backend
                .process_audio(input_file, model, output_dir, quality, move |event: &EngineEvent| {
                    if event.event == "progress" {
                        let _ = tx_events.send(AppMsg::ProcessProgress {
                            message: event
                                .message
                                .clone()
                                .unwrap_or_else(|| "Working...".into()),
                            percent: event.percent.unwrap_or_default(),
                        });
                    } else if let Some(message) = &event.message {
                        let _ = tx_events.send(AppMsg::Log(format!("ENGINE: {}", message)));
                    }
                })
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::ProcessFinished(result));
        });
    }

    pub(crate) fn browse_input_file(&mut self) {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("Audio", &["wav", "mp3", "flac", "m4a", "aac", "ogg"])
            .pick_file()
        {
            self.state.input_file = file.display().to_string();
            self.state
                .push_log(format!("FILE LOADED: {}", self.state.input_file));
        }
    }

    pub(crate) fn browse_output_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.output_dir = folder.display().to_string();
            self.state
                .push_log(format!("DIR SELECTED: {}", self.state.output_dir));
        }
    }

    pub(crate) fn browse_models_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.config.models_dir = Some(folder.display().to_string());
        }
    }

    pub(crate) fn browse_cache_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.config.cache_dir = Some(folder.display().to_string());
        }
    }

    pub(crate) fn apply_settings(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let config = self.state.config.clone();
        self.runtime.spawn(async move {
            let result = backend.update_config(config).map_err(|error| error.to_string());
            match result {
                Ok(()) => {
                    let _ = tx.send(AppMsg::Log(
                        "SUCCESS: Settings applied. Some changes may require engine prepare again.".into(),
                    ));
                    let health = backend.get_engine_health().await.map_err(|error| error.to_string());
                    let _ = tx.send(AppMsg::HealthLoaded(health));
                }
                Err(error) => {
                    let _ = tx.send(AppMsg::Log(format!("ERROR: Failed to apply settings: {}", error)));
                }
            }
        });
    }

    fn auto_save_config(&mut self) {
        let mut config = self.state.config.clone();
        let mut changed = false;

        let input = if self.state.input_file.is_empty() { None } else { Some(self.state.input_file.clone()) };
        if config.last_input_file != input {
            config.last_input_file = input;
            changed = true;
        }

        let output = if self.state.output_dir.is_empty() { None } else { Some(self.state.output_dir.clone()) };
        if config.last_output_dir != output {
            config.last_output_dir = output;
            changed = true;
        }

        let model = if self.state.selected_model.is_empty() { None } else { Some(self.state.selected_model.clone()) };
        if config.last_selected_model != model {
            config.last_selected_model = model;
            changed = true;
        }

        if config.last_quality != Some(self.state.quality.clone()) {
            config.last_quality = Some(self.state.quality.clone());
            changed = true;
        }

        if config.last_export_format != Some(self.state.export_format.clone()) {
            config.last_export_format = Some(self.state.export_format.clone());
            changed = true;
        }

        if changed {
            self.state.config = config.clone();
            let backend = Arc::clone(&self.backend);
            if let Some(handle) = self.save_handle.take() {
                handle.abort();
            }
            self.save_handle = Some(self.runtime.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let _ = backend.update_config(config);
            }));
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        if dropped.is_empty() {
            return;
        }

        if let Some(path) = dropped
            .first()
            .and_then(|file| file.path.as_ref())
            .map(|path| path.display().to_string())
        {
            self.state.input_file = path.clone();
            self.state
                .push_log(format!("FILE LOADED via DnD: {}", path));
        }
    }

    fn drain_messages(&mut self) {
        while let Ok(message) = self.state.rx.try_recv() {
            match message {
                AppMsg::ConfigLoaded(config) => {
                    self.state.config = config.clone();
                    if let Some(val) = config.last_input_file {
                        self.state.input_file = val;
                    }
                    if let Some(val) = config.last_output_dir {
                        self.state.output_dir = val;
                    }
                    if let Some(val) = config.last_selected_model {
                        self.state.selected_model = val;
                    }
                    if let Some(val) = config.last_quality {
                        self.state.quality = val;
                    }
                    if let Some(val) = config.last_export_format {
                        self.state.export_format = val;
                    }
                }
                AppMsg::HealthLoaded(result) => {
                    self.state.is_initializing = false;
                    match result {
                        Ok(health) => {
                            let ready = health.runtime_ready
                                && health.dependencies_ready
                                && health.model_catalog_ready;
                            self.state.health = Some(health);
                            if ready {
                                self.load_catalog();
                            }
                        }
                        Err(error) => self.state.push_log(format!("ERROR: Health check failed: {}", error)),
                    }
                }
                AppMsg::CatalogLoaded(result) => match result {
                    Ok(catalog) => {
                        self.state.catalog = catalog;
                        if self.state.selected_model.is_empty() {
                            if let Some(first) = self.state.catalog.first() {
                                self.state.selected_model = first.id.clone();
                            }
                        }
                        self.state.push_log(format!(
                            "SUCCESS: Loaded {} models into registry view.",
                            self.state.catalog.len()
                        ));
                    }
                    Err(error) => self
                        .state
                        .push_log(format!("ERROR: Failed to load model catalog: {}", error)),
                },
                AppMsg::SetupFinished(result) => match result {
                    Ok(status) => {
                        self.state.setup_status = Some(status.clone());
                        if status.ready {
                            self.state
                                .push_log("SUCCESS: Engine preparation completed.".to_string());
                            self.refresh_health();
                        }
                    }
                    Err(error) => self.state.push_log(format!("ERROR: Setup failed: {}", error)),
                },
                AppMsg::DownloadStarted(model_id) => {
                    self.state.downloading_id = Some(model_id.clone());
                    self.state.download_progress = 0.0;
                    self.state
                        .push_log(format!("INIT: Download started for model [{}].", model_id));
                }
                AppMsg::DownloadProgress { model_id: _, progress } => {
                    self.state.download_progress = progress;
                }
                AppMsg::DownloadFinished(result) => {
                    self.state.downloading_id = None;
                    self.state.download_progress = 0.0;
                    match result {
                        Ok(model) => {
                            let verified = has_trusted_checksum(&model.sha256);
                            let status = if verified {
                                "installed and verified"
                            } else {
                                "installed without checksum verification"
                            };
                            self.state
                                .push_log(format!("SUCCESS: Model [{}] {}.", model.name, status));
                            self.load_catalog();
                        }
                        Err(error) => self
                            .state
                            .push_log(format!("ERROR: Model download failed: {}", error)),
                    }
                }
                AppMsg::CatalogSynced(result) => match result {
                    Ok(added) => {
                        self.state.push_log(format!(
                            "SUCCESS: Catalog synchronized. {} new models added.",
                            added
                        ));
                        self.load_catalog();
                    }
                    Err(error) => self
                        .state
                        .push_log(format!("ERROR: Catalog sync failed: {}", error)),
                },
                AppMsg::LocalScanFinished(result) => match result {
                    Ok(added) => {
                        self.state.push_log(format!(
                            "SUCCESS: Local scan complete. {} models registered.",
                            added
                        ));
                        self.load_catalog();
                    }
                    Err(error) => self.state.push_log(format!("ERROR: Scan failed: {}", error)),
                },
                AppMsg::ProcessProgress { message, percent } => {
                    self.state.is_processing = true;
                    self.state.process_progress = percent;
                    self.state
                        .push_log(format!("ENGINE: {} ({:.0}%)", message, percent));
                }
                AppMsg::ProcessFinished(result) => {
                    self.state.is_processing = false;
                    self.state.process_progress = 0.0;
                    match result {
                        Ok(response) => {
                            self.state.push_log(format!(
                                "SUCCESS: Separation complete via {}. Vocals: {} | Instrumental: {}",
                                response.backend, response.vocals_path, response.instrumental_path
                            ));

                            if self.state.enable_preview {
                                let tx = self.state.tx.clone();
                                let vocals_path = response.vocals_path.clone();
                                let instrumental_path = response.instrumental_path.clone();

                                self.runtime.spawn(async move {
                                    let _ = tx.send(AppMsg::Log("PREVIEW: Analyzing spectral data...".into()));
                                    
                                    let analysis_future = async {
                                        let mut stems = Vec::new();
                                        if let Ok(peaks) = crate::preview::analyze_audio_peaks(&vocals_path, 180) {
                                            stems.push(crate::preview::StemPreview {
                                                id: "vocals".into(),
                                                name: "VOCALS (Voz)".into(),
                                                file_path: vocals_path,
                                                peaks,
                                                is_playing: false,
                                            });
                                        }
                                        if let Ok(peaks) = crate::preview::analyze_audio_peaks(&instrumental_path, 180) {
                                            stems.push(crate::preview::StemPreview {
                                                id: "instrumental".into(),
                                                name: "INSTRUMENTAL (Música)".into(),
                                                file_path: instrumental_path,
                                                peaks,
                                                is_playing: false,
                                            });
                                        }
                                        stems
                                    };

                                    match tokio::time::timeout(std::time::Duration::from_secs(30), analysis_future).await {
                                        Ok(stems) => {
                                            let _ = tx.send(AppMsg::PreviewStemsLoaded(stems));
                                        }
                                        Err(_) => {
                                            let _ = tx.send(AppMsg::Log("WARNING: Preview analysis timed out".into()));
                                            let _ = tx.send(AppMsg::PreviewStemsLoaded(Vec::new()));
                                        }
                                    }
                                });
                            }
                        }
                        Err(error) => self
                            .state
                            .push_log(format!("ERROR: Separation halted: {}", error)),
                    }
                }
                AppMsg::PreviewStemsLoaded(stems) => {
                    self.state.active_preview_stems = Some(stems);
                    self.state.push_log("PREVIEW: Waveforms analyzed and ready for preview.");
                }
                AppMsg::Log(line) => self.state.push_log(line),

            }
        }
    }

    pub fn browse_local_scan(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.scan_local_models(path.display().to_string());
        }
    }

    pub(crate) fn save_all_preview_stems(&mut self, stems: &[crate::preview::StemPreview]) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let dest_dir = folder.display().to_string();
            let source_paths: Vec<String> = stems.iter().map(|s| s.file_path.clone()).collect();
            match self.backend.export_all_stems(&source_paths, &dest_dir) {
                Ok(()) => {
                    self.state.push_log(format!("SUCCESS: Exported all stems to folder [{}]", dest_dir));
                }
                Err(e) => {
                    self.state.push_log(format!("ERROR: Failed to export stems: {}", e));
                }
            }
        }
    }

    pub(crate) fn close_preview_window(&mut self, stems: &[crate::preview::StemPreview]) {
        self.playback_controller.stop();
        self.state.current_playing_stem = None;
        
        let source_paths: Vec<String> = stems.iter().map(|s| s.file_path.clone()).collect();
        self.backend.cleanup_previews(&source_paths);
        
        self.state.active_preview_stems = None;
        self.state.push_log("PREVIEW: Preview session closed and temporary assets destroyed.");
    }
}

impl eframe::App for PrismSplitApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "active_tab", &self.state.active_tab);
        eframe::set_value(storage, "enable_preview", &self.state.enable_preview);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_messages();
        self.handle_dropped_files(ctx);

        egui::TopBottomPanel::top("top_bar")
            .exact_height(42.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                    ui.label(
                        RichText::new("PRISMSPLIT // EGUi ALPHA")
                            .monospace()
                            .strong()
                            .color(Color32::from_rgb(34, 211, 238)),
                    );
                    ui.separator();

                    if nav_button(ui, self.state.active_tab == Tab::Separate, "[1] EXTRACTION") {
                        self.state.active_tab = Tab::Separate;
                    }
                    if nav_button(ui, self.state.active_tab == Tab::Models, "[2] REGISTRY") {
                        self.state.active_tab = Tab::Models;
                    }
                    if nav_button(ui, self.state.active_tab == Tab::Settings, "[3] CONFIG") {
                        self.state.active_tab = Tab::Settings;
                    }

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let gpu_text = self
                            .state
                            .health
                            .as_ref()
                            .and_then(|health| health.gpu_devices.first().cloned())
                            .unwrap_or_else(|| "CPU_ONLY".into());
                        status_chip(ui, "GPU:", &gpu_text, false);
                    });
                });
            });

        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui| {
                let active_procs = self.backend.list_active_processes();
                log_console::show(ui, &self.state.log, &active_procs);
            });

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let status = if self.state.is_processing {
                        "ENGINE_BUSY"
                    } else {
                        "ENGINE_READY"
                    };
                    status_chip(ui, "STATUS:", status, self.state.is_processing);
                    if self.state.downloading_id.is_some() {
                        ui.separator();
                        status_chip(
                            ui,
                            "DOWNLOAD:",
                            &format!("{:.0}%", self.state.download_progress),
                            true,
                        );
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let total_rect = ui.max_rect();
            let painter = ui.painter();
            
            // Draw rack ears on left and right
            let ear_width = 24.0;
            let left_ear = egui::Rect::from_min_max(
                total_rect.left_top(),
                egui::pos2(total_rect.left() + ear_width, total_rect.bottom())
            );
            let right_ear = egui::Rect::from_min_max(
                egui::pos2(total_rect.right() - ear_width, total_rect.top()),
                total_rect.right_bottom()
            );
            
            // Fill ears
            let ear_color = Color32::from_rgb(20, 20, 20);
            painter.rect_filled(left_ear, 0.0, ear_color);
            painter.rect_filled(right_ear, 0.0, ear_color);
            
            // Bevel edges dividing the ears from the main chassis panel
            let highlight = Color32::from_rgb(45, 45, 45);
            let shadow = Color32::from_rgb(5, 5, 5);
            
            painter.line_segment([left_ear.right_top(), left_ear.right_bottom()], egui::Stroke::new(1.5, shadow));
            painter.line_segment([left_ear.right_top() + egui::vec2(1.0, 0.0), left_ear.right_bottom() + egui::vec2(1.0, 0.0)], egui::Stroke::new(1.0, highlight));
            
            painter.line_segment([right_ear.left_top(), right_ear.left_bottom()], egui::Stroke::new(1.5, highlight));
            painter.line_segment([right_ear.left_top() + egui::vec2(1.0, 0.0), right_ear.left_bottom() + egui::vec2(1.0, 0.0)], egui::Stroke::new(1.0, shadow));
            
            // Draw rack mounting screws
            crate::widgets::draw_screw(ui, egui::pos2(left_ear.center().x, left_ear.top() + 30.0));
            crate::widgets::draw_screw(ui, egui::pos2(left_ear.center().x, left_ear.bottom() - 30.0));
            
            crate::widgets::draw_screw(ui, egui::pos2(right_ear.center().x, right_ear.top() + 30.0));
            crate::widgets::draw_screw(ui, egui::pos2(right_ear.center().x, right_ear.bottom() - 30.0));
            
            // Render actual content in the middle
            ui.allocate_ui_at_rect(
                egui::Rect::from_min_max(
                    egui::pos2(total_rect.left() + ear_width + 12.0, total_rect.top() + 12.0),
                    egui::pos2(total_rect.right() - ear_width - 12.0, total_rect.bottom() - 12.0)
                ),
                |ui| {
                    if self.state.is_initializing {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new("PRISMSPLIT_CORE_BOOTSTRAP...")
                                    .monospace()
                                    .strong(),
                            );
                        });
                        return;
                    }

                    let engine_ready = self
                        .state
                        .health
                        .as_ref()
                        .map(|health| health.runtime_ready && health.dependencies_ready)
                        .unwrap_or(false);

                    if !engine_ready {
                        crate::panels::setup::show(self, ui);
                        return;
                    }

                    match self.state.active_tab {
                        Tab::Separate => crate::panels::separate::show(self, ui),
                        Tab::Models => crate::panels::models::show(self, ui),
                        Tab::Settings => crate::panels::settings::show(self, ui),
                    }
                }
            );
        });

        self.auto_save_config();

        // Render live stem preview modal if active
        crate::panels::preview::show(self, ctx);

        if self.state.is_processing || self.state.downloading_id.is_some() || self.state.active_preview_stems.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.backend.kill_all_active_processes();
    }
}

fn default_output_dir(input_file: &str) -> String {
    std::path::Path::new(input_file)
        .parent()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".into())
}
