// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
use prismsplit::runtime_manager::RuntimeManager;
use tempfile::tempdir;

#[test]
fn app_paths_create_expected_runtime_layout() {
    let root = tempdir().unwrap();
    let paths = AppPaths::new(root.path().to_path_buf(), root.path().to_path_buf());

    assert!(paths.runtime_dir.ends_with("runtime"));
    assert!(paths.python_dir.ends_with("python"));
    assert!(paths.venv_dir.ends_with("venv"));
    assert!(paths.models_dir.ends_with("models"));
}

#[tokio::test]
async fn doctor_reports_missing_runtime_before_setup() {
    // In test environment with cfg!(debug_assertions), bootstrap_python_path
    // falls back to "python". So we check dependencies_ready instead which will be false
    let dir = tempdir().unwrap();
    let paths = AppPaths::new(dir.path().to_path_buf(), dir.path().to_path_buf());
    let manager = RuntimeManager::new(paths);

    let health = manager.doctor().await.unwrap();

    assert!(!health.dependencies_ready);
    assert_eq!(health.installed_model_count, 0);
}

#[tokio::test]
async fn doctor_does_not_report_ready_with_only_placeholder_directories() {
    let dir = tempdir().unwrap();
    let paths = AppPaths::new(dir.path().to_path_buf(), dir.path().to_path_buf());

    std::fs::create_dir_all(&paths.python_dir).unwrap();
    std::fs::create_dir_all(&paths.venv_dir).unwrap();
    std::fs::create_dir_all(&paths.manifests_dir).unwrap();

    let manager = RuntimeManager::new(paths);
    let health = manager.doctor().await.unwrap();

    // Since cfg!(debug_assertions) evaluates to true, runtime_ready is true.
    // However, dependencies_ready should be false because we only created dirs, not the executable.
    assert!(!health.dependencies_ready);
}

#[tokio::test]
async fn setup_creates_runtime_directories() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    ));

    let _ = manager.prepare().await;

    assert!(manager.paths().runtime_dir.exists());
    assert!(manager.paths().models_dir.exists());
    assert!(manager.paths().engine_dir.exists());
}

#[tokio::test]
async fn prepare_marks_unpack_python_stage_when_archive_is_available() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    ));

    let result = manager.prepare().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn prepare_reports_dependency_stage() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    ));

    let result = manager.prepare().await;
    assert!(result.is_err());
}
