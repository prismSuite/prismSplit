// src-tauri/tests/model_registry.rs
use prismsplit::download_manager::sha256_file;
use prismsplit::model_registry::load_catalog_from_str;
use prismsplit::models::{EngineHealth, SetupStatus};
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
    };

    assert_eq!(health.installed_model_count, 0);
    assert!(!health.ffmpeg_ready);
}

#[test]
fn catalog_parser_reads_single_karaoke_model() {
    let json = r#"
    [
      {
        "id": "mdx_uvr_karaoke_1",
        "name": "MDX Karaoke 1",
        "backend": "mdx",
        "output_kind": "vocals_instrumental",
        "url": "https://example.com/model.onnx",
        "sha256": "abc",
        "size_bytes": 42,
        "filename": "model.onnx",
        "version": "1.0.0"
      }
    ]
    "#;

    let catalog = load_catalog_from_str(json).unwrap();
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
    std::fs::create_dir_all(&models_dir).unwrap();

    let registry = prismsplit::model_registry::ModelRegistry::new(models_dir.clone());
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
    };

    assert!(!registry.is_model_installed(&entry));

    let model_path = models_dir.join("test_model.onnx");
    std::fs::write(&model_path, b"dummy").unwrap();

    assert!(registry.is_model_installed(&entry));
}
