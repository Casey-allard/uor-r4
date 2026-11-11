mod common;

use serde_json::{json, Value};
use uor_r4_studio_agent::{
    canonical_json, digest_event, parse_canonical_json, schema_document, AGENT_ERROR_SCHEMA,
    AGENT_EVENT_SCHEMA, CANONICAL_JSON_SCHEMA,
};

#[test]
fn keys_are_sorted_ascii() {
    let value = json!({"b": 2, "a": 1, "a_b": 5, "a.b": 3, "a-b": 4});
    assert_eq!(
        canonical_json(&value).unwrap(),
        r#"{"a":1,"a-b":4,"a.b":3,"a_b":5,"b":2}"#
    );

    let nested = json!({"z": {"b": true, "a": null}, "y": [3, 2, 1]});
    assert_eq!(
        canonical_json(&nested).unwrap(),
        r#"{"y":[3,2,1],"z":{"a":null,"b":true}}"#
    );
}

#[test]
fn insertion_order_does_not_change_digest() {
    let first = digest_event(&common::event_with_order(true)).unwrap();
    let second = digest_event(&common::event_with_order(false)).unwrap();
    assert_eq!(first, second);

    let hex = first.strip_prefix("sha256:").unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));

    let ordered_first = parse_canonical_json(r#"{"b":2,"a":{"d":4,"c":3}}"#).unwrap();
    let ordered_second = parse_canonical_json(r#"{"a":{"c":3,"d":4},"b":2}"#).unwrap();
    assert_eq!(ordered_first, ordered_second);
}

#[test]
fn floats_are_rejected() {
    assert_eq!(
        canonical_json(&json!({"x": 1.5})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"x": 1.0})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":1.5}"#).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":1e3}"#).unwrap_err().code(),
        "invalid_event"
    );
}

#[test]
fn non_ascii_keys_are_rejected() {
    assert_eq!(
        canonical_json(&json!({"é": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"A": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        canonical_json(&json!({"$schema": 1})).unwrap_err().code(),
        "invalid_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"é":1}"#).unwrap_err().code(),
        "invalid_event"
    );
    assert!(canonical_json(&json!({"ok": "é"})).is_ok());
}

#[test]
fn schema_document_is_stable_sorted() {
    let first = schema_document();
    let second = schema_document();
    assert_eq!(first, second);
    assert!(first.starts_with(r#"{"$id":"#));
    assert!(first.contains("\"run.queued\""));
    assert!(first.contains("\"run.started\""));
    assert!(first.contains("\"run.resumed\""));
    assert!(first.contains("\"retry_of_run_id\""));
    assert!(first.contains("\"runtime_unavailable\""));
    assert!(first.contains("\"host_unavailable\""));
    assert!(first.contains(AGENT_EVENT_SCHEMA));
    assert!(first.contains(AGENT_ERROR_SCHEMA));
    assert!(first.contains(CANONICAL_JSON_SCHEMA));
    assert!(first.contains("\"uorse1\""));
    assert!(!first.contains(": "));

    let parsed: Value = serde_json::from_str(&first).unwrap();
    assert!(parsed.get("event_types").is_some());
    assert!(parsed.get("error_codes").is_some());
}

#[test]
fn raw_parser_rejects_duplicate_keys_at_every_depth() {
    for text in [
        r#"{"a":1,"a":2}"#,
        r#"{"outer":{"a":1,"a":2}}"#,
        r#"[{"x":1,"x":1}]"#,
        r#"{"a":{"b":{"c":1,"c":1}}}"#,
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "invalid_event",
            "{text}"
        );
    }
    assert_eq!(
        parse_canonical_json(r#"{"a":1,"b":2}"#).unwrap(),
        r#"{"a":1,"b":2}"#
    );
}

#[test]
fn raw_parser_rejects_floats_negative_and_unsafe_integers() {
    for text in [
        r#"{"x":1.5}"#,
        r#"{"x":-1}"#,
        r#"{"x":-0}"#,
        r#"{"x":9007199254740992}"#,
        r#"{"x":1e3}"#,
        r#"{"x":-9007199254740991}"#,
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "invalid_event",
            "{text}"
        );
    }
    assert_eq!(
        parse_canonical_json(r#"{"x":0,"y":9007199254740991}"#).unwrap(),
        r#"{"x":0,"y":9007199254740991}"#
    );
}

#[test]
fn raw_parser_rejects_malformed_text_and_unpaired_surrogates() {
    for text in [
        "{\"x\":1",
        "\u{feff}{\"x\":1}",
        "{\"x\":1} trailing",
        "{\"x\":\"\\ud800\"}",
        "",
        "[] []",
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            "malformed_event",
            "{text:?}"
        );
    }
}

#[test]
fn cross_runtime_vectors_are_pinned() {
    // The same literals are asserted by
    // web/studio/src/agent/canonical.test.ts. Changing one side requires
    // changing the other; this is the Rust half of the cross-runtime proof.
    assert_eq!(
        canonical_json(&json!({"b": 2, "a": 1, "a_b": 5, "a.b": 3, "a-b": 4})).unwrap(),
        r#"{"a":1,"a-b":4,"a.b":3,"a_b":5,"b":2}"#
    );
    assert_eq!(
        canonical_json(&json!({"z": {"b": true, "a": null}, "y": [3, 2, 1]})).unwrap(),
        r#"{"y":[3,2,1],"z":{"a":null,"b":true}}"#
    );
    assert_eq!(
        canonical_json(&json!({"": 1, "a": 2})).unwrap(),
        r#"{"":1,"a":2}"#
    );
    assert_eq!(
        canonical_json(&json!({"a": "\u{0}\u{b}\u{1f}"})).unwrap(),
        r#"{"a":"\u0000\u000b\u001f"}"#
    );
    let controls: String = (0u32..0x20)
        .map(|code| char::from_u32(code).expect("control code is a scalar"))
        .collect();
    let escaped: String = (0u32..0x20)
        .map(|code| format!("\\u00{code:02x}"))
        .collect();
    assert_eq!(
        canonical_json(&json!({"a": controls})).unwrap(),
        format!("{{\"a\":\"{escaped}\"}}")
    );
    assert_eq!(
        canonical_json(&json!({"a": "\u{7f}"})).unwrap(),
        "{\"a\":\"\u{7f}\"}"
    );
    assert_eq!(
        canonical_json(&json!({"a": "café ☃"})).unwrap(),
        "{\"a\":\"café ☃\"}"
    );
    assert_eq!(canonical_json(&json!("\u{1f600}")).unwrap(), "\"😀\"");

    assert_eq!(
        parse_canonical_json(r#"{"a":"caf\u00e9 \"q\" \\ line\nend","z":"\u0000\u001f"}"#).unwrap(),
        "{\"a\":\"café \\\"q\\\" \\\\ line\\u000aend\",\"z\":\"\\u0000\\u001f\"}"
    );
    assert_eq!(
        parse_canonical_json(r#"{"b":2,"a":{"d":4,"c":3}}"#).unwrap(),
        r#"{"a":{"c":3,"d":4},"b":2}"#
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":"\ud83d\ude00"}"#).unwrap(),
        "{\"x\":\"😀\"}"
    );
    assert_eq!(parse_canonical_json("\"caf\\u00e9\"").unwrap(), "\"café\"");
    assert_eq!(
        parse_canonical_json("{\"a\":1} \n\t").unwrap(),
        r#"{"a":1}"#
    );
    assert_eq!(parse_canonical_json("null").unwrap(), "null");
    assert_eq!(parse_canonical_json("0").unwrap(), "0");
    assert_eq!(
        parse_canonical_json(r#"{"__proto__":{"polluted":true},"a":1}"#).unwrap(),
        r#"{"__proto__":{"polluted":true},"a":1}"#
    );
    let depth_127 = format!("{}0{}", "[".repeat(127), "]".repeat(127));
    assert_eq!(parse_canonical_json(&depth_127).unwrap(), depth_127);
    let depth_128 = format!("{}0{}", "[".repeat(128), "]".repeat(128));
    assert_eq!(
        parse_canonical_json(&depth_128).unwrap_err().code(),
        "malformed_event"
    );
    assert_eq!(
        parse_canonical_json(r#"{"x":1e308}"#).unwrap_err().code(),
        "invalid_event"
    );
    let huge_integer = "9".repeat(400);
    for text in [
        r#"{"x":1e400}"#.to_string(),
        r#"{"x":-1e400}"#.to_string(),
        format!("{{\"x\":{huge_integer}}}"),
    ] {
        assert_eq!(
            parse_canonical_json(&text).unwrap_err().code(),
            "malformed_event",
            "{text}"
        );
    }

    let event = common::event_with_order(false);
    let canonical = canonical_json(&serde_json::to_value(&event).unwrap()).unwrap();
    assert_eq!(
        canonical,
        r#"{"data":{"message_id":"msg_1f2e3d4c5b6a7988","text":"ordered"},"event_id":"evt_run_7f3a2c1b9d4e6a80_1_000000000006","generation":1,"project_id":"prj_4b1d9e0c7a2f6e51","recorded_at_ms":1794283200123,"run_id":"run_7f3a2c1b9d4e6a80","schema":"uor-r4.studio.agent-event/1","sequence":6,"task_id":"tsk_9a3c5e7b1d2f4a60","type":"assistant.message"}"#
    );
    assert_eq!(
        digest_event(&event).unwrap(),
        "sha256:fac58990349c5d556f0d14f927d9a37eacfe2a8d90683fcbb04a146b086e6565"
    );
    assert_eq!(
        digest_event(&common::event_with_order(true)).unwrap(),
        digest_event(&event).unwrap()
    );

    for (text, code) in [
        (r#"{"a":1,"a":2}"#, "invalid_event"),
        (r#"{"outer":{"a":1,"a":2}}"#, "invalid_event"),
        (r#"[{"x":1,"x":1}]"#, "invalid_event"),
        (r#"{"a":{"b":{"c":1,"c":1}}}"#, "invalid_event"),
        (r#"{"a":1,"\u0061":2}"#, "invalid_event"),
        (r#"{"x":1.5}"#, "invalid_event"),
        (r#"{"x":1.0}"#, "invalid_event"),
        (r#"{"x":-1}"#, "invalid_event"),
        (r#"{"x":-0}"#, "invalid_event"),
        (r#"{"x":9007199254740992}"#, "invalid_event"),
        (r#"{"x":1e3}"#, "invalid_event"),
        (r#"{"x":-9007199254740991}"#, "invalid_event"),
        (r#"{"é":1}"#, "invalid_event"),
        ("{\"x\":1", "malformed_event"),
        ("\u{feff}{\"x\":1}", "malformed_event"),
        ("{\"x\":1} trailing", "malformed_event"),
        ("{\"x\":\"\\ud800\"}", "malformed_event"),
        ("{\"x\":\"\\udc00\"}", "malformed_event"),
        ("{\"x\":1,}", "malformed_event"),
        ("", "malformed_event"),
        ("[] []", "malformed_event"),
        ("{\"x\":01}", "malformed_event"),
    ] {
        assert_eq!(
            parse_canonical_json(text).unwrap_err().code(),
            code,
            "{text:?}"
        );
    }
}

#[test]
fn raw_parser_escapes_minimally_and_preserves_utf8() {
    let text = r#"{"a":"caf\u00e9 \"q\" \\ line\nend","z":"\u0000\u001f"}"#;
    assert_eq!(
        parse_canonical_json(text).unwrap(),
        "{\"a\":\"café \\\"q\\\" \\\\ line\\u000aend\",\"z\":\"\\u0000\\u001f\"}"
    );

    assert_eq!(
        parse_canonical_json(r#"{"a":"é ☃"}"#).unwrap(),
        "{\"a\":\"é ☃\"}"
    );
    assert_eq!(
        parse_canonical_json(r#"{"é":1}"#).unwrap_err().code(),
        "invalid_event"
    );
}
