use crate::models::{AppConfig, EngineHealth, ModelCatalogEntry, ProcessAudioResponse, SetupStatus};
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};

pub const QUALITY_PRESETS: &[&str] = &[
    "Fast (CPU)",
    "Normal (CUDA)",
    "High Quality (Overlap)",
    "Extreme (Aggressive Math)",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tab {
    Separate,
    Models,
    Settings,
}

#[derive(Debug)]
pub enum AppMsg {
    ConfigLoaded(AppConfig),
    HealthLoaded(Result<EngineHealth, String>),
    CatalogLoaded(Result<Vec<ModelCatalogEntry>, String>),
    SetupFinished(Result<SetupStatus, String>),
    DownloadStarted(String),
    DownloadProgress { model_id: String, progress: f32 },
    DownloadFinished(Result<ModelCatalogEntry, String>),
    CatalogSynced(Result<usize, String>),
    LocalScanFinished(Result<usize, String>),
    ProcessProgress { message: String, percent: f32 },
    ProcessFinished(Result<ProcessAudioResponse, String>),
    Log(String),
    PreviewStemsLoaded(Vec<crate::preview::StemPreview>),
}

pub struct AppState {
    pub health: Option<EngineHealth>,
    pub is_initializing: bool,
    pub setup_status: Option<SetupStatus>,
    pub config: AppConfig,
    pub catalog: Vec<ModelCatalogEntry>,
    pub downloading_id: Option<String>,
    pub download_progress: f32,
    pub input_file: String,
    pub output_dir: String,
    pub selected_model: String,
    pub quality: String,
    pub export_format: String,
    pub is_processing: bool,
    pub process_progress: f32,
    pub log: VecDeque<String>,
    pub active_tab: Tab,
    pub is_dragging: bool,
    pub tx: Sender<AppMsg>,
    pub rx: Receiver<AppMsg>,
    // Preview feature state
    pub enable_preview: bool,
    pub active_preview_stems: Option<Vec<crate::preview::StemPreview>>,
    pub current_playing_stem: Option<String>,
}

impl AppState {
    pub fn new(tx: Sender<AppMsg>, rx: Receiver<AppMsg>) -> Self {
        let mut log = VecDeque::new();
        log.push_back("PrismSplit native boot sequence loaded.".into());
        log.push_back("Egui renderer online.".into());
        log.push_back("Waiting for engine health check...".into());

        Self {
            health: None,
            is_initializing: true,
            setup_status: None,
            config: AppConfig::default(),
            catalog: Vec::new(),
            downloading_id: None,
            download_progress: 0.0,
            input_file: String::new(),
            output_dir: String::new(),
            selected_model: String::new(),
            quality: QUALITY_PRESETS[1].into(),
            export_format: "WAV".into(),
            is_processing: false,
            process_progress: 0.0,
            log,
            active_tab: Tab::Separate,
            is_dragging: false,
            tx,
            rx,
            enable_preview: false,
            active_preview_stems: None,
            current_playing_stem: None,
        }
    }

    pub fn push_log(&mut self, message: impl Into<String>) {
        self.log.push_back(message.into());
        while self.log.len() > 500 {
            self.log.pop_front();
        }
    }
}
