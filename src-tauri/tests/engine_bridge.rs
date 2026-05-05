// src-tauri/tests/engine_bridge.rs
use prismsplit::engine_bridge::parse_event_line;

#[test]
fn parse_event_line_reads_progress_message() {
    let line = r#"{"event":"progress","job_id":"1","message":"Loading","percent":50.0}"#;
    let event = parse_event_line(line).unwrap();
    assert_eq!(event.event, "progress");
    assert_eq!(event.job_id.as_deref(), Some("1"));
}
