//! Real-fit state/serialization checks; these are construction laws, not transfer.
use super::*;
#[allow(dead_code)]
mod fixture {
    use crate::native_geometric as native;
    include!("../../tests/support/native_word_copy_fixture.rs");
}

fn fitted() -> &'static Model {
    static MODEL: std::sync::OnceLock<Model> = std::sync::OnceLock::new();
    MODEL.get_or_init(|| {
        let mut docs = Vec::new();
        for (i, name) in ["alpha", "bravo", "cedar", "delta"].iter().enumerate() {
            for (j, prompt, response) in [
                (
                    0,
                    format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    "),
                    format!("{name}\n}}\n"),
                ),
                (
                    1,
                    format!("holder in {name}. Where is holder? Answer:"),
                    format!(" {name}.\n"),
                ),
                (
                    2,
                    format!("{name} in city. Where is missing? Answer:"),
                    " Unknown.\n".into(),
                ),
            ] {
                docs.push(ValueExample {
                    id: format!("role-state-{i}-{j}"),
                    prompt,
                    response,
                });
            }
        }
        let (model, report) = fixture::fitted_composed()
            .fit_role_read(
                &docs,
                ResponseEntryFitConfig {
                    epochs: 64,
                    ..ResponseEntryFitConfig::default()
                },
            )
            .unwrap();
        assert_eq!(report["fit_correct"], report["frames"]);
        Model::from_bytes(&model.to_bytes().unwrap()).unwrap()
    })
}

fn begin(model: &Model, prompt: &str) -> Session {
    let mut s = model.session(Control::Full).unwrap();
    s.observe(model, BOS).unwrap();
    for t in model.encode(prompt).unwrap() {
        s.observe(model, t).unwrap();
    }
    s.begin_response(model).unwrap();
    s
}

#[test]
fn native_role_read_commits_only_observed_entry_and_restores_before_byte_zero() {
    let model = fitted();
    for (prompt, action) in [
        (fixture::COPY_PROMPT, WordCopyAction::Read),
        (
            "holder in alpha. Where is holder? Answer:",
            WordCopyAction::Prepare,
        ),
        (
            "alpha in city. Where is missing? Answer:",
            WordCopyAction::NoRead,
        ),
    ] {
        let mut s = begin(model, prompt);
        let before = s.checkpoint().unwrap();
        let first = s.predict(model).unwrap();
        let decision = s.word_copy_decision().unwrap();
        assert_eq!(decision.action, action);
        let before_wire: serde_json::Value = serde_json::from_slice(&before).unwrap();
        let after_wire: serde_json::Value =
            serde_json::from_slice(&s.checkpoint().unwrap()).unwrap();
        assert_eq!(
            after_wire["word_copy"], before_wire["word_copy"],
            "selection is transient; work counters still advance"
        );
        let mut mismatch = model.restore_session(&before).unwrap();
        mismatch.predict(model).unwrap();
        mismatch.observe(model, u32::from(b'?') + 2).unwrap();
        assert!(mismatch.word_copy.as_ref().unwrap().read_commit.is_none());
        assert!(mismatch.word_copy.as_ref().unwrap().origin.is_none());
        s.observe(model, first.token).unwrap();
        let snapshot = s.checkpoint().unwrap();
        let mut restored = model.restore_session(&snapshot).unwrap();
        assert_eq!(restored.word_copy, s.word_copy);
        let mut bad: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
        bad["word_copy"]["read_commit"]["source_end"] = serde_json::json!(u64::MAX);
        assert!(model
            .restore_session(&serde_json::to_vec(&bad).unwrap())
            .is_err());
        let mut missing: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
        missing["word_copy"]
            .as_object_mut()
            .unwrap()
            .remove("read_commit");
        assert!(model
            .restore_session(&serde_json::to_vec(&missing).unwrap())
            .is_err());
        for _ in 0..3 {
            let next = s.predict(model).unwrap();
            assert_eq!(restored.predict(model).unwrap(), next);
            assert_eq!(restored.word_copy_decision(), s.word_copy_decision());
            s.observe(model, next.token).unwrap();
            restored.observe(model, next.token).unwrap();
            assert_eq!(restored.word_copy, s.word_copy);
            model.restore_session(&s.checkpoint().unwrap()).unwrap();
            if next.token == EOS {
                break;
            }
        }
    }
}

#[test]
fn native_role_read_validates_quantized_rows_and_parent_identity() {
    let model = fitted();
    for change in 0..3 {
        let mut bad = model.clone();
        let head = bad
            .response_entry
            .as_mut()
            .unwrap()
            .copy
            .as_mut()
            .unwrap()
            .role_read
            .as_mut()
            .unwrap();
        match change {
            0 => head.rows[0].weight = 1_000_001,
            1 => head.baseline_artifact = "bad".into(),
            _ => head.dictionary[0].prime += 1,
        }
        bad.refresh_identity().unwrap();
        assert!(bad.validate().is_err());
    }
    let mut changed = model.clone();
    changed
        .response_entry
        .as_mut()
        .unwrap()
        .copy
        .as_mut()
        .unwrap()
        .role_read
        .as_mut()
        .unwrap()
        .rows[0]
        .weight += 1;
    changed.refresh_identity().unwrap();
    assert_ne!(changed.artifact_cid(), model.artifact_cid());
    assert_eq!(
        Model::from_bytes(&changed.to_bytes().unwrap()).unwrap(),
        changed
    );
}
