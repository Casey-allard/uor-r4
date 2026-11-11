#![allow(dead_code)]

use serde_json::{json, Value};
use uor_r4_studio_agent::{event_id, validate_event, AgentEvent, AGENT_EVENT_SCHEMA};

pub const PROJECT_ID: &str = "prj_4b1d9e0c7a2f6e51";
pub const TASK_ID: &str = "tsk_9a3c5e7b1d2f4a60";
pub const OTHER_TASK_ID: &str = "tsk_0d1e2f3a4b5c6d7e";
pub const RUN_ID: &str = "run_7f3a2c1b9d4e6a80";
pub const RECORDED_AT_MS: u64 = 1_794_283_200_123;

pub fn event(sequence: u64) -> AgentEvent {
    event_for(TASK_ID, sequence)
}

pub fn event_for(task_id: &str, sequence: u64) -> AgentEvent {
    let value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(RUN_ID, 1, sequence),
        "project_id": PROJECT_ID,
        "task_id": task_id,
        "run_id": RUN_ID,
        "generation": 1,
        "sequence": sequence,
        "recorded_at_ms": RECORDED_AT_MS,
        "type": "assistant.delta",
        "data": {
            "message_id": "msg_1f2e3d4c5b6a7988",
            "text_delta": "hello"
        }
    });
    validate_event(&value).expect("common event fixture is a valid contract event")
}

pub fn last_event(sequence: u64) -> AgentEvent {
    let value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(RUN_ID, 1, sequence),
        "project_id": PROJECT_ID,
        "task_id": TASK_ID,
        "run_id": RUN_ID,
        "generation": 1,
        "sequence": sequence,
        "recorded_at_ms": RECORDED_AT_MS,
        "type": "run.finished",
        "data": {
            "status": "finished",
            "finished_at_ms": RECORDED_AT_MS,
            "duration_ms": 42
        }
    });
    validate_event(&value).expect("common terminal event fixture is valid")
}

pub fn event_with_order(reversed: bool) -> AgentEvent {
    let data: Value = serde_json::from_str(if reversed {
        r#"{"text":"ordered","message_id":"msg_1f2e3d4c5b6a7988"}"#
    } else {
        r#"{"message_id":"msg_1f2e3d4c5b6a7988","text":"ordered"}"#
    })
    .expect("common data fixture is valid JSON");
    let value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(RUN_ID, 1, 6),
        "project_id": PROJECT_ID,
        "task_id": TASK_ID,
        "run_id": RUN_ID,
        "generation": 1,
        "sequence": 6,
        "recorded_at_ms": RECORDED_AT_MS,
        "type": "assistant.message",
        "data": data
    });
    validate_event(&value).expect("common insertion-order fixture is valid")
}
