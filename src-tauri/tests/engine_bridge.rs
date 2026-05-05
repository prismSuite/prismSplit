// src-tauri/tests/engine_bridge.rs
use prismsplit::engine_bridge::parse_event_line;
use prismsplit::job_manager::validate_request;
use prismsplit::models::SeparationRequest;

#[test]
fn parse_event_line_reads_progress_message() {
    let line = r#"{"event":"progress","job_id":"1","message":"Loading","percent":50.0}"#;
    let event = parse_event_line(line).unwrap();
    assert_eq!(event.event, "progress");
    assert_eq!(event.job_id.as_deref(), Some("1"));
}

#[test]
fn validate_request_rejects_missing_input_path() {
    let request = SeparationRequest {
        input_path: "".into(),
        model_id: "mdx_uvr_karaoke_1".into(),
        output_dir: "C:/out".into(),
        format: "wav".into(),
    };

    assert!(validate_request(&request).is_err());
}
