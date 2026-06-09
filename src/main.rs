#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use prismsplit::app::PrismSplitApp;
use prismsplit::backend::Backend;
use prismsplit::theme::apply_prismsplit_theme;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    dotenvy::dotenv().ok();

    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime"),
    );

    let app_root = app_data_root();
    let config_path = app_config_root().join("config.json");
    let resource_dir = resource_engine_dir();
    let backend = Arc::new(Backend::new(app_root, resource_dir, config_path));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PrismSplit // Native Egui")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PrismSplit",
        native_options,
        Box::new(move |cc| {
            apply_prismsplit_theme(&cc.egui_ctx);
            Ok(Box::new(PrismSplitApp::new(
                Arc::clone(&backend),
                Arc::clone(&runtime),
                cc.storage,
            )))
        }),
    )
}

fn app_data_root() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(std::env::temp_dir)
        .join("PrismSplit")
}

fn app_config_root() -> PathBuf {
    dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .or_else(dirs::data_dir)
        .unwrap_or_else(std::env::temp_dir)
        .join("PrismSplit")
}

fn resource_engine_dir() -> PathBuf {
    // 1. Try local workspace (for development/cargo run)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_engine = manifest_dir.join("engine");
    if workspace_engine.is_dir() {
        return workspace_engine;
    }

    // 2. Try current executable's sibling (for portable/zip/nsis release installs)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let engine_dir = exe_dir.join("engine");
            if engine_dir.is_dir() {
                return engine_dir;
            }
            // 3. Try under 'resources' subdirectory (for cargo-packager / bundle standard)
            let resources_engine = exe_dir.join("resources").join("engine");
            if resources_engine.is_dir() {
                return resources_engine;
            }
        }
    }

    // Fallback guess
    manifest_dir.join("engine")
}
