use crate::backend::Backend;
use crate::engine_bridge::EngineEvent;
use crate::panels::log_console;
use crate::state::{AppMsg, AppState, Tab};
use crate::widgets::{fieldset, nav_button, status_chip};
use eframe::egui::{self, Align, Color32, RichText};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

pub struct PrismSplitApp {
    backend: Arc<Backend>,
    runtime: Arc<tokio::runtime::Runtime>,
    state: AppState,
}

impl PrismSplitApp {
    pub fn new(backend: Arc<Backend>, runtime: Arc<tokio::runtime::Runtime>) -> Self {
        let (tx, rx) = mpsc::channel();
        let state = AppState::new(tx, rx);
        let app = Self {
            backend,
            runtime,
            state,
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

    fn refresh_health(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        self.runtime.spawn(async move {
            let result = backend.get_engine_health().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::HealthLoaded(result));
        });
    }

    fn load_catalog(&self) {
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

    fn prepare_engine(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let _ = tx.send(AppMsg::Log("INIT: Preparing embedded engine...".into()));
        self.runtime.spawn(async move {
            let result = backend.prepare_engine().await.map_err(|error| error.to_string());
            let _ = tx.send(AppMsg::SetupFinished(result));
        });
    }

    fn download_model(&self, model_id: String) {
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

    fn sync_catalog(&self) {
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

    fn scan_local_models(&self, path: String) {
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

    fn process_audio(&self) {
        let tx = self.state.tx.clone();
        let backend = Arc::clone(&self.backend);
        let input_file = self.state.input_file.clone();
        let model = self.state.selected_model.clone();
        let output_dir = if self.state.output_dir.trim().is_empty() {
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

    fn browse_input_file(&mut self) {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("Audio", &["wav", "mp3", "flac", "m4a", "aac", "ogg"])
            .pick_file()
        {
            self.state.input_file = file.display().to_string();
            self.state
                .push_log(format!("FILE LOADED: {}", self.state.input_file));
        }
    }

    fn browse_output_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.output_dir = folder.display().to_string();
            self.state
                .push_log(format!("DIR SELECTED: {}", self.state.output_dir));
        }
    }

    fn browse_models_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.config.models_dir = Some(folder.display().to_string());
        }
    }

    fn browse_cache_dir(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.state.config.cache_dir = Some(folder.display().to_string());
        }
    }

    fn apply_settings(&self) {
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
            self.runtime.spawn(async move {
                let _ = backend.update_config(config);
            });
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
                        Ok(response) => self.state.push_log(format!(
                            "SUCCESS: Separation complete via {}. Vocals: {} | Instrumental: {}",
                            response.backend, response.vocals_path, response.instrumental_path
                        )),
                        Err(error) => self
                            .state
                            .push_log(format!("ERROR: Separation halted: {}", error)),
                    }
                }
                AppMsg::Log(line) => self.state.push_log(line),
            }
        }
    }

    fn render_setup(&mut self, ui: &mut egui::Ui) {
        fieldset(ui, "CORE_SETUP", |ui| {
            if let Some(health) = &self.state.health {
                ui.label(format!("Runtime ready: {}", health.runtime_ready));
                ui.label(format!("Dependencies ready: {}", health.dependencies_ready));
                ui.label(format!("Catalog ready: {}", health.model_catalog_ready));
                ui.label(format!("Installed models: {}", health.installed_model_count));
            } else {
                ui.label("No engine health data yet.");
            }

            if let Some(status) = &self.state.setup_status {
                ui.separator();
                ui.label(RichText::new("Last prepare status").strong());
                ui.label(format!("Ready: {}", status.ready));
                if !status.completed_stages.is_empty() {
                    ui.label(format!(
                        "Stages: {}",
                        status.completed_stages.join(", ")
                    ));
                }
                if let Some(error) = &status.last_error {
                    ui.colored_label(Color32::from_rgb(242, 139, 130), error);
                }
            }

            ui.add_space(8.0);
            if ui.button("PREPARE ENGINE").clicked() {
                self.prepare_engine();
            }
            if ui.button("REFRESH HEALTH").clicked() {
                self.refresh_health();
            }
        });
    }

    fn render_separation(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            fieldset(&mut columns[0], "I/O TARGETS", |ui| {
                ui.label("INPUT FILE");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.state.input_file);
                    if ui.button("BROWSE").clicked() {
                        self.browse_input_file();
                    }
                });

                ui.label("OUTPUT DIRECTORY");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.state.output_dir);
                    if ui.button("BROWSE").clicked() {
                        self.browse_output_dir();
                    }
                });

                if !self.state.input_file.is_empty() {
                    ui.label(
                        RichText::new("Tip: you can also drag a file onto the window.")
                            .italics()
                            .small(),
                    );
                }
            });

            fieldset(&mut columns[1], "ENGINE_PARAMS", |ui| {
                ui.label("MODEL");
                egui::ComboBox::from_id_source("model_selector")
                    .selected_text(selected_model_label(&self.state))
                    .show_ui(ui, |ui| {
                        for entry in &self.state.catalog {
                            ui.selectable_value(
                                &mut self.state.selected_model,
                                entry.id.clone(),
                                format!("{} [{}]", entry.name, entry.backend),
                            );
                        }
                    });

                ui.label("EXPORT FORMAT");
                egui::ComboBox::from_id_source("export_format")
                    .selected_text(self.state.export_format.as_str())
                    .show_ui(ui, |ui| {
                        for option in ["WAV", "FLAC", "MP3"] {
                            ui.selectable_value(
                                &mut self.state.export_format,
                                option.to_string(),
                                option,
                            );
                        }
                    });

                ui.label("QUALITY");
                egui::ComboBox::from_id_source("quality")
                    .selected_text(self.state.quality.as_str())
                    .show_ui(ui, |ui| {
                        for option in [
                            "Fast (CPU)",
                            "Normal (CUDA)",
                            "High Quality (Overlap)",
                            "Extreme (Aggressive Math)",
                        ] {
                            ui.selectable_value(&mut self.state.quality, option.to_string(), option);
                        }
                    });
            });
        });

        ui.add_space(12.0);
        fieldset(ui, "EXECUTION_NODE", |ui| {
            if self.state.is_processing {
                ui.add(
                    egui::ProgressBar::new(self.state.process_progress / 100.0)
                        .text(format!("{:.0}% active", self.state.process_progress)),
                );
            }

            let can_start = !self.state.input_file.trim().is_empty()
                && !self.state.selected_model.trim().is_empty()
                && !self.state.is_processing;

            if ui
                .add_enabled(can_start, egui::Button::new("START SEPARATION"))
                .clicked()
            {
                self.state.is_processing = true;
                self.process_audio();
            }
        });
    }

    fn render_models(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("SYNC UVR CATALOG").clicked() {
                self.sync_catalog();
            }
            if ui.button("SCAN LOCAL MODELS").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.scan_local_models(path.display().to_string());
                }
            }
            if ui.button("REFRESH").clicked() {
                self.load_catalog();
            }
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in self.state.catalog.clone() {
                fieldset(ui, &entry.name, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Backend: {}", entry.backend));
                        ui.separator();
                        ui.label(format!("Version: {}", entry.version));
                        ui.separator();
                        ui.label(format!("Installed: {}", entry.is_installed));
                    });
                    ui.label(format!("File: {}", entry.filename));
                    if entry.size_bytes > 0 {
                        ui.label(format!(
                            "Size: {:.2} MB",
                            entry.size_bytes as f64 / (1024.0 * 1024.0)
                        ));
                    }

                    if self.state.downloading_id.as_deref() == Some(entry.id.as_str()) {
                        ui.add(
                            egui::ProgressBar::new(self.state.download_progress / 100.0)
                                .text(format!("{:.0}%", self.state.download_progress)),
                        );
                    }

                    let label = if entry.is_installed {
                        "INSTALLED"
                    } else {
                        "DOWNLOAD"
                    };
                    if ui
                        .add_enabled(
                            !entry.is_installed
                                && self.state.downloading_id.is_none(),
                            egui::Button::new(label),
                        )
                        .clicked()
                    {
                        self.download_model(entry.id.clone());
                    }
                });
                ui.add_space(6.0);
            }
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            fieldset(&mut columns[0], "SYSTEM_PATHS", |ui| {
                ui.label("MODELS DIRECTORY");
                ui.horizontal(|ui| {
                    let value = self.state.config.models_dir.get_or_insert_with(String::new);
                    ui.text_edit_singleline(value);
                    if ui.button("BROWSE").clicked() {
                        self.browse_models_dir();
                    }
                });

                ui.label("CACHE DIRECTORY");
                ui.horizontal(|ui| {
                    let value = self.state.config.cache_dir.get_or_insert_with(String::new);
                    ui.text_edit_singleline(value);
                    if ui.button("BROWSE").clicked() {
                        self.browse_cache_dir();
                    }
                });
            });

            fieldset(&mut columns[1], "HARDWARE_ACCEL", |ui| {
                if let Some(health) = &self.state.health {
                    if health.gpu_devices.is_empty() {
                        ui.label("No GPU devices detected.");
                    } else {
                        for gpu in &health.gpu_devices {
                            ui.label(gpu);
                        }
                    }
                } else {
                    ui.label("Health data unavailable.");
                }

                ui.add_space(8.0);
                if ui.button("APPLY SETTINGS").clicked() {
                    self.apply_settings();
                }
            });
        });
    }
}

impl eframe::App for PrismSplitApp {
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
                            .color(Color32::from_rgb(0, 210, 255)),
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
                log_console::show(ui, &self.state.log);
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
                self.render_setup(ui);
                return;
            }

            match self.state.active_tab {
                Tab::Separate => self.render_separation(ui),
                Tab::Models => self.render_models(ui),
                Tab::Settings => self.render_settings(ui),
            }
        });

        self.auto_save_config();

        if self.state.is_processing || self.state.downloading_id.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn default_output_dir(input_file: &str) -> String {
    std::path::Path::new(input_file)
        .parent()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".into())
}

fn selected_model_label(state: &AppState) -> String {
    state
        .catalog
        .iter()
        .find(|entry| entry.id == state.selected_model)
        .map(|entry| format!("{} [{}]", entry.name, entry.backend))
        .unwrap_or_else(|| "Select a model".into())
}

fn has_trusted_checksum(sha256: &str) -> bool {
    let value = sha256.trim();
    !value.is_empty() && !value.eq_ignore_ascii_case("replace-with-real-sha256")
}
