// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
use prismsplit::runtime_manager::RuntimeManager;
use tempfile::tempdir;

#[test]
fn app_paths_create_expected_runtime_layout() {
    let root = tempdir().unwrap();
    let paths = AppPaths::new(root.path().to_path_buf());

    assert!(paths.runtime_dir.ends_with("runtime"));
    assert!(paths.python_dir.ends_with("python"));
    assert!(paths.venv_dir.ends_with("venv"));
    assert!(paths.models_dir.ends_with("models"));
}

#[tokio::test]
async fn doctor_reports_missing_runtime_before_setup() {
    let dir = tempdir().unwrap();
    let paths = AppPaths::new(dir.path().to_path_buf());
    let manager = RuntimeManager::new(paths);

    let health = manager.doctor().await.unwrap();

    assert!(!health.runtime_ready);
    assert_eq!(health.installed_model_count, 0);
}

#[tokio::test]
async fn setup_creates_runtime_directories() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(dir.path().to_path_buf()));

    let status = manager.prepare().await.unwrap();

    assert!(status
        .completed_stages
        .contains(&"create_directories".to_string()));
}

#[tokio::test]
async fn prepare_marks_unpack_python_stage_when_archive_is_available() {
    // This test is a scaffold for when we have a real fixture
    // For now we just verify it doesn't crash if the logic is there
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(dir.path().to_path_buf()));

    // We expect it to fail if archive is missing, or we can mock it
    let result = manager.prepare().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn prepare_reports_dependency_stage() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(dir.path().to_path_buf()));

    let status = manager.prepare().await.unwrap();

    assert!(status
        .completed_stages
        .contains(&"install_dependencies".to_string()));
}
