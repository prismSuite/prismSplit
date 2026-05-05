// src-tauri/tests/model_registry.rs
use prismsplit::models::{EngineHealth, SetupStatus};

#[test]
fn setup_status_defaults_to_not_ready() {
    let status = SetupStatus::default();
    assert!(!status.ready);
    assert!(status.current_stage.is_none());
}

#[test]
fn engine_health_exposes_runtime_and_model_flags() {
    let health = EngineHealth {
        runtime_ready: false,
        dependencies_ready: false,
        ffmpeg_ready: false,
        model_catalog_ready: false,
        installed_model_count: 0,
        active_job_count: 0,
    };

    assert_eq!(health.installed_model_count, 0);
    assert!(!health.ffmpeg_ready);
}
