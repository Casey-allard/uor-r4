mod common;

use serde_json::{json, Value};
use uor_r4_studio_agent::{
    event_id, validate_approval, validate_event, validate_run_start_pair, ActiveRun, AgentCursor,
    AgentErrorCode, RunState, TaskState, TaskSummary, AGENT_EVENT_SCHEMA,
};

#[test]
fn contract_schema_id_is_stable() {
    assert_eq!(AGENT_EVENT_SCHEMA, "uor-r4.studio.agent-event/1");
}

#[test]
fn event_id_is_deterministic() {
    assert_eq!(
        event_id("run_7f3a2c1b9d4e6a80", 2, 1),
        "evt_run_7f3a2c1b9d4e6a80_2_000000000001"
    );
    assert_eq!(
        event_id("run_7f3a2c1b9d4e6a80", 2, 42),
        "evt_run_7f3a2c1b9d4e6a80_2_000000000042"
    );
}

#[test]
fn event_envelope_requires_run_scope() {
    let value = json!({"schema": AGENT_EVENT_SCHEMA, "task_id": "tsk_1"});
    let error = validate_event(&value).unwrap_err();
    assert_eq!(error.code(), "malformed_event");
}

#[test]
fn unknown_event_type_preserves_opaque_data() {
    let value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 3),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 3,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "future.unknown",
        "data": {"x": 1}
    });
    let event = validate_event(&value).unwrap();
    assert_eq!(event.event_type, "future.unknown");
    assert_eq!(event.data, json!({"x": 1}));
    assert!(!event.is_authoritative_terminal());
}

#[test]
fn cursor_token_round_trips() {
    let event = common::event(4);
    let cursor = AgentCursor::from_event(&event).unwrap();
    assert_eq!(cursor.generation, 1);
    assert_eq!(cursor.sequence, 4);
    let token = cursor.encode();
    assert!(token.starts_with("uorse1."));
    assert_eq!(AgentCursor::parse(&token).unwrap(), cursor);
    assert_eq!(cursor.last_event_id(), event.event_id);
    assert!(AgentCursor::parse("uorse1.not-a-cursor").is_err());
    assert!(AgentCursor::parse(&token.replacen("uorse1", "uorse2", 1)).is_err());
}

#[test]
fn cursor_rejects_sequence_gaps() {
    let cursor = AgentCursor::from_event(&common::event(2)).unwrap();
    assert!(cursor.check_event(&common::event(3)).is_ok());

    let gap = common::event(5);
    assert_eq!(cursor.check_event(&gap).unwrap_err().code(), "out_of_order");

    let same_position = common::event(2);
    assert_eq!(
        cursor.check_event(&same_position).unwrap_err().code(),
        "out_of_order"
    );
}

#[test]
fn approval_object_requires_digest_and_expiry() {
    let requested_at_ms = 1_794_283_200_123u64;
    let approval = json!({
        "approval_id": "apr_5c2e8a1f9b7d3e40",
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "tool_call_id": "tc_02",
        "tool": "git.commit",
        "scope": "write",
        "request_digest": format!("sha256:{}", "0f".repeat(32)),
        "requested_at_ms": requested_at_ms,
        "expires_at_ms": requested_at_ms + 600_000
    });
    assert!(validate_approval(&approval).is_ok());

    for field in ["request_digest", "expires_at_ms"] {
        let mut broken = approval.clone();
        if let Some(map) = broken.as_object_mut() {
            map.remove(field);
        }
        assert!(
            validate_approval(&broken).is_err(),
            "{field} must be required"
        );
    }

    let mut bad_digest = approval.clone();
    bad_digest["request_digest"] = json!("sha256:nothex");
    assert!(validate_approval(&bad_digest).is_err());

    let mut expired_window = approval.clone();
    expired_window["expires_at_ms"] = json!(requested_at_ms);
    assert!(validate_approval(&expired_window).is_err());
}

#[test]
fn error_codes_include_runtime_and_host_unavailable() {
    assert_eq!(
        AgentErrorCode::RuntimeUnavailable.as_str(),
        "runtime_unavailable"
    );
    assert_eq!(AgentErrorCode::HostUnavailable.as_str(), "host_unavailable");
    assert_eq!(
        AgentErrorCode::StopUnacknowledged.as_str(),
        "stop_unacknowledged"
    );
    assert_ne!(
        AgentErrorCode::RuntimeUnavailable.as_str(),
        AgentErrorCode::HostUnavailable.as_str()
    );
}

#[test]
fn fixture_run_queued_and_started_satisfy_contract() {
    let queued_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-queued.json")).unwrap();
    let started_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-started.json")).unwrap();

    let (queued, started) = validate_run_start_pair(&queued_value, &started_value).unwrap();
    assert_eq!(queued.event_type, "run.queued");
    assert_eq!(queued.sequence, 0);
    assert_eq!(started.event_type, "run.started");
    assert_eq!(started.sequence, 1);
    assert_eq!(queued.run_id, started.run_id);
    assert_eq!(queued.task_id, started.task_id);
    assert_eq!(queued.generation, started.generation);
    assert_eq!(started.data["retry_of_run_id"], Value::Null);
    assert_eq!(started.data["mode"], "agent");

    assert!(validate_run_start_pair(&started_value, &queued_value).is_err());
    assert!(validate_run_start_pair(&queued_value, &queued_value).is_err());
}

#[test]
fn resume_uses_next_generation_without_repeating_start_markers() {
    let queued_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-queued.json")).unwrap();
    let started_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-started.json")).unwrap();
    let (_, started) = validate_run_start_pair(&queued_value, &started_value).unwrap();

    let resumed_at_ms = common::RECORDED_AT_MS + 7_000;
    let snapshot_digest = format!("sha256:{}", "ab".repeat(32));
    let resumed_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 2, 2),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 2,
        "sequence": 2,
        "recorded_at_ms": resumed_at_ms,
        "type": "run.resumed",
        "data": {
            "resumed_at_ms": resumed_at_ms,
            "previous_generation": 1,
            "snapshot_digest": snapshot_digest.clone()
        }
    });

    let cursor = AgentCursor::from_event(&started).unwrap();
    let resumed_cursor = cursor.resumed().unwrap();
    assert_eq!((resumed_cursor.generation, resumed_cursor.sequence), (2, 2));

    let resumed = validate_event(&resumed_value).unwrap();
    assert_eq!(resumed.event_type, "run.resumed");
    assert_eq!(resumed.generation, started.generation + 1);
    assert_eq!(resumed.sequence, started.sequence + 1);
    assert_ne!(resumed.event_id, started.event_id);
    assert_eq!(resumed_cursor, AgentCursor::from_event(&resumed).unwrap());
    assert_eq!(resumed_cursor, cursor.check_resume(&resumed).unwrap());
    assert_eq!(resumed_cursor.last_event_id(), resumed.event_id);

    let mut wrong_previous = resumed_value.clone();
    wrong_previous["data"]["previous_generation"] = json!(2);
    assert!(validate_event(&wrong_previous).is_err());

    let mut early_sequence = resumed_value.clone();
    early_sequence["sequence"] = json!(1);
    early_sequence["event_id"] = json!(event_id(common::RUN_ID, 2, 1));
    assert!(validate_event(&early_sequence).is_err());

    let repeated_started_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 2, 1),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 2,
        "sequence": 1,
        "recorded_at_ms": resumed_at_ms,
        "type": "run.started",
        "data": {
            "started_at_ms": resumed_at_ms,
            "model": {"id": "replay", "kind": "fixture"},
            "mode": "agent",
            "environment": {"kind": "local-browser"},
            "retry_of_run_id": null
        }
    });
    let repeated_started = validate_event(&repeated_started_value).unwrap();
    assert_eq!(repeated_started.sequence, 1);
    assert_eq!(
        resumed_cursor
            .check_event(&repeated_started)
            .unwrap_err()
            .code(),
        "out_of_order"
    );

    let repeated_queued_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 2, 0),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 2,
        "sequence": 0,
        "recorded_at_ms": resumed_at_ms,
        "type": "run.queued",
        "data": {"queued_at_ms": resumed_at_ms}
    });
    let repeated_queued = validate_event(&repeated_queued_value).unwrap();
    assert_eq!(repeated_queued.sequence, 0);
    assert_eq!(
        resumed_cursor
            .check_event(&repeated_queued)
            .unwrap_err()
            .code(),
        "out_of_order"
    );

    let stale = common::event(3);
    assert_eq!(
        resumed_cursor.check_event(&stale).unwrap_err().code(),
        "stale_generation"
    );
    let cross_task = common::event_for(common::OTHER_TASK_ID, 3);
    assert_eq!(
        resumed_cursor.check_event(&cross_task).unwrap_err().code(),
        "cross_task_event"
    );
}

#[test]
fn check_resume_rejects_a_sequence_jump() {
    let queued_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-queued.json")).unwrap();
    let started_value: Value =
        serde_json::from_str(include_str!("fixtures/event-run-started.json")).unwrap();
    let (_, started) = validate_run_start_pair(&queued_value, &started_value).unwrap();
    let cursor = AgentCursor::from_event(&started).unwrap();

    let resumed_at_ms = common::RECORDED_AT_MS + 8_000;
    let snapshot_digest = format!("sha256:{}", "cd".repeat(32));

    let jumped_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 2, 5),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 2,
        "sequence": 5,
        "recorded_at_ms": resumed_at_ms,
        "type": "run.resumed",
        "data": {
            "resumed_at_ms": resumed_at_ms,
            "previous_generation": 1,
            "snapshot_digest": snapshot_digest.clone()
        }
    });
    let jumped = validate_event(&jumped_value).unwrap();
    assert_eq!(jumped.sequence, 5);
    assert_eq!(
        cursor.check_resume(&jumped).unwrap_err().code(),
        "out_of_order"
    );

    let valid_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 2, 2),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 2,
        "sequence": 2,
        "recorded_at_ms": resumed_at_ms,
        "type": "run.resumed",
        "data": {
            "resumed_at_ms": resumed_at_ms,
            "previous_generation": 1,
            "snapshot_digest": snapshot_digest
        }
    });
    let valid = validate_event(&valid_value).unwrap();
    let resumed = cursor.check_resume(&valid).unwrap();
    assert_eq!(resumed, cursor.resumed().unwrap());
    assert_eq!((resumed.generation, resumed.sequence), (2, 2));

    let mut wrong_previous = valid.clone();
    wrong_previous.data["previous_generation"] = json!(0);
    assert!(cursor.check_resume(&wrong_previous).is_err());

    let mut wrong_type = valid.clone();
    wrong_type.event_type = "run.status".to_string();
    assert!(cursor.check_resume(&wrong_type).is_err());
}

#[test]
fn terminal_events_are_authoritative_over_run_status() {
    let status_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 4),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 4,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "run.status",
        "data": {"status": "finished"}
    });
    let status = validate_event(&status_value).unwrap();
    assert_eq!(status.data["status"], "finished");
    assert!(!status.is_authoritative_terminal());

    let finished_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 5),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 5,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "run.finished",
        "data": {"status": "finished", "finished_at_ms": common::RECORDED_AT_MS, "duration_ms": 5}
    });
    assert!(validate_event(&finished_value)
        .unwrap()
        .is_authoritative_terminal());

    let cancelled_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 6),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 6,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "run.cancelled",
        "data": {
            "safe_boundary": true,
            "stopped_at_tool_id": "tc_02",
            "finished_at_ms": common::RECORDED_AT_MS
        }
    });
    assert!(validate_event(&cancelled_value)
        .unwrap()
        .is_authoritative_terminal());

    let failed_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 7),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 7,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "run.failed",
        "data": {
            "code": "stop_unacknowledged",
            "message": "the host never acknowledged the stop",
            "retryable": false,
            "stop_outcome": "ack_timeout"
        }
    });
    let failed = validate_event(&failed_value).unwrap();
    assert!(failed.is_authoritative_terminal());
    assert_eq!(failed.data["stop_outcome"], "ack_timeout");

    let error_value = json!({
        "schema": AGENT_EVENT_SCHEMA,
        "event_id": event_id(common::RUN_ID, 1, 8),
        "project_id": common::PROJECT_ID,
        "task_id": common::TASK_ID,
        "run_id": common::RUN_ID,
        "generation": 1,
        "sequence": 8,
        "recorded_at_ms": common::RECORDED_AT_MS,
        "type": "error",
        "data": {"code": "host_unavailable", "message": "companion not connected", "retryable": true}
    });
    assert!(!validate_event(&error_value)
        .unwrap()
        .is_authoritative_terminal());
}

#[test]
fn task_summary_derives_state_from_active_run() {
    let run_id = common::RUN_ID.to_string();
    let mut summary = TaskSummary {
        task_id: common::TASK_ID.to_string(),
        project_id: common::PROJECT_ID.to_string(),
        title: "Contract task".to_string(),
        repository: None,
        branch: None,
        pinned: false,
        archived: false,
        hold: false,
        active_run: Some(ActiveRun {
            run_id: run_id.clone(),
            state: RunState::Running,
        }),
    };

    assert_eq!(summary.state(), TaskState::Running);

    summary.archived = true;
    assert_eq!(summary.state(), TaskState::Archived);
    summary.archived = false;

    summary.active_run = Some(ActiveRun {
        run_id: run_id.clone(),
        state: RunState::WaitingForApproval,
    });
    assert_eq!(summary.state(), TaskState::WaitingForApproval);

    summary.active_run = Some(ActiveRun {
        run_id: run_id.clone(),
        state: RunState::Finished,
    });
    assert_eq!(summary.state().as_str(), "finished");

    for (state, label) in [
        (RunState::Failed, "failed"),
        (RunState::Cancelled, "cancelled"),
        (RunState::Expired, "expired"),
    ] {
        summary.active_run = Some(ActiveRun {
            run_id: run_id.clone(),
            state,
        });
        assert_eq!(
            summary.state().as_str(),
            label,
            "a terminal run must not project as idle"
        );
    }

    summary.hold = true;
    summary.active_run = Some(ActiveRun {
        run_id,
        state: RunState::Finished,
    });
    assert_eq!(summary.state(), TaskState::WaitingForBackground);

    summary.active_run = None;
    assert_eq!(summary.state(), TaskState::WaitingForBackground);

    summary.hold = false;
    assert_eq!(summary.state(), TaskState::Idle);

    assert!(RunState::Cancelled.is_terminal());
    assert!(RunState::Expired.is_terminal());
    assert!(!RunState::Cancelling.is_terminal());
}
