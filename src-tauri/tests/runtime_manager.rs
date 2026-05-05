// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
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
