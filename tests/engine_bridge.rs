// src-tauri/tests/engine_bridge.rs
use prismsplit::engine_bridge::parse_event_line;

#[test]
fn parse_event_line_reads_progress_message() {
    let line = r#"{"event":"progress","job_id":"1","message":"Loading","percent":50.0}"#;
    let event = parse_event_line(line).unwrap();
    assert_eq!(event.event, "progress");
    assert_eq!(event.job_id.as_deref(), Some("1"));
}

#[tokio::test]
async fn bridge_reads_result_event_from_python_process() {
    let python_exe = std::path::PathBuf::from("python3");
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let engine_script = manifest_dir.join("tests/fixtures/mock_engine.py");

    let bridge = prismsplit::engine_bridge::EngineBridge::new(python_exe, engine_script);
    let payload = serde_json::json!({
        "ping": true
    });

    let (events, mut child) = bridge
        .run_command_collect("doctor", payload)
        .await
        .expect("Failed to run doctor");

    assert!(!events.is_empty(), "Expected at least one event");
    let last_event = events.last().unwrap();
    assert_eq!(last_event.event, "result");
    assert_eq!(last_event.message.as_deref(), Some("doctor_ok"));
    
    let payload_val = last_event.payload.as_ref().expect("Expected payload");
    assert_eq!(payload_val.get("ping"), Some(&serde_json::json!(true)));

    let _ = child.kill().await;
}

#[tokio::test]
async fn running_job_can_be_cancelled() {
    let python_exe = std::path::PathBuf::from("python3");
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let engine_script = manifest_dir.join("tests/fixtures/mock_engine.py");

    let bridge = prismsplit::engine_bridge::EngineBridge::new(python_exe, engine_script);
    let payload = serde_json::json!({});

    let mut child = bridge.spawn_command("slow_job", payload).await.expect("Failed to spawn command");
    let stdout = child.stdout.take().expect("Failed to take stdout");

    // Spawn a task to stream output
    let handle = tokio::spawn(async move {
        prismsplit::engine_bridge::EngineBridge::stream_stdout(stdout, |_event| {}).await
    });

    // Let it run briefly
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Cancel/kill child process
    child.kill().await.expect("Failed to kill child process");

    // The stream task should exit with error because process was killed (pipe closed)
    let res = handle.await.expect("Task join failed");
    assert!(res.is_err(), "Expected error after child process kill");
}

#[tokio::test]
#[ignore = "requires embedded runtime and real model fixture"]
async fn end_to_end_karaoke_separation_produces_two_output_files() {
    // 1. setup on fresh runtime root
    // 2. install one model
    // 3. run one short separation fixture
    // 4. verify two outputs exist
    // 5. verify logs contain progress and result events
}
