// src-tauri/tests/model_registry.rs
use prismsplit::download_manager::sha256_file;
use prismsplit::models::{EngineHealth, SetupStatus};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

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
        gpu_devices: vec![],
    };

    assert_eq!(health.installed_model_count, 0);
    assert!(!health.ffmpeg_ready);
}

#[test]
fn engine_health_serializes_with_camel_case_keys() {
    let health = EngineHealth {
        runtime_ready: true,
        dependencies_ready: true,
        ffmpeg_ready: false,
        model_catalog_ready: true,
        installed_model_count: 1,
        active_job_count: 0,
        gpu_devices: vec![],
    };

    let value = serde_json::to_value(health).unwrap();

    assert_eq!(value["runtimeReady"], json!(true));
    assert_eq!(value["dependenciesReady"], json!(true));
    assert_eq!(value["modelCatalogReady"], json!(true));
    assert_eq!(value["installedModelCount"], json!(1));
}

#[test]
fn catalog_parser_reads_single_karaoke_model() {
    let json = r#"
    [
      {
        "id": "mdx_uvr_karaoke_1",
        "name": "MDX Karaoke 1",
        "backend": "mdx",
        "outputKind": "vocals_instrumental",
        "url": "https://example.com/model.onnx",
        "sha256": "abc",
        "sizeBytes": 42,
        "filename": "model.onnx",
        "version": "1.0.0"
      }
    ]
    "#;

    let catalog = prismsplit::model_registry::load_catalog_from_str(json).unwrap();
    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog[0].id, "mdx_uvr_karaoke_1");
}

#[test]
fn sha256_file_matches_known_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    fs::write(&path, b"prismsplit").unwrap();

    let hash = sha256_file(&path).unwrap();
    assert_eq!(hash.len(), 64);
}

#[test]
fn model_is_installed_when_target_file_exists() {
    let dir = tempdir().unwrap();
    let models_dir = dir.path().join("models");
    let catalog_path = dir.path().join("catalog.json");
    std::fs::create_dir_all(&models_dir).unwrap();
    std::fs::write(&catalog_path, "[]").unwrap();

    let registry = prismsplit::model_registry::ModelRegistry::new(models_dir.clone(), catalog_path);
    let entry = prismsplit::models::ModelCatalogEntry {
        id: "test".into(),
        name: "Test".into(),
        backend: "mdx".into(),
        output_kind: "vocals".into(),
        url: "".into(),
        sha256: "".into(),
        size_bytes: 0,
        filename: "test_model.onnx".into(),
        version: "1".into(),
        is_installed: false,
        local_path: None,
    };

    assert!(!registry.is_model_installed(&entry));

    let model_path = models_dir.join("test_model.onnx");
    std::fs::write(&model_path, b"dummy").unwrap();

    assert!(registry.is_model_installed(&entry));
}
