//! Tests for the capability API (`IntegerCapabilityApi`).
//!
//! The synthetic byte-vocabulary bundle covers the API contract on every run.
//! The width-576 tests need the local `dialogue-child-bundle-1` fixture; they
//! are ignored by default and fail when run without it. The width-576
//! continuation test is also a known failure: the session schema omits the
//! 576-wide value stores, so a restored width-576 session diverges.

use std::path::Path;
use uor_r4_integer::capability_api::{
    IntegerCapabilityApi, SessionConfig, CAPABILITY_API_SCHEMA, DECLARED_NUMERICAL_CONTRACT,
};
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::sampling::SamplePolicy;
use uor_r4_integer::{create_test_bundle_with_byte_vocab, Bundle, IntegerError, Result};

const DIALOGUE576_BUNDLE_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1";

fn require_fixture(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(IntegerError::Invalid(format!(
            "required fixture {} is missing; this ignored test needs the local dialogue-child-bundle-1 fixture",
            path.display()
        )))
    }
}

fn saved_json(api: &IntegerCapabilityApi, session_id: u32) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(api.save_session(session_id)?)?)
}

/// Restore a saved session: its telemetry and its saved form must equal the
/// original's. Returns the restored session.
fn restore_and_compare_saved_state(api: &IntegerCapabilityApi, original: u32) -> Result<u32> {
    let saved = api.save_session(original)?;
    let saved_value = serde_json::to_value(&saved)?;
    let restored = api.restore_session(saved)?;
    let before = api.get_telemetry(original)?;
    let after = api.get_telemetry(restored)?;
    assert_eq!(after.read_mode, before.read_mode);
    assert_eq!(after.policy, before.policy);
    assert_eq!(after.prng_state, before.prng_state);
    assert_eq!(after.active_tokens, before.active_tokens);
    assert_eq!(saved_json(api, restored)?, saved_value);
    Ok(restored)
}

/// Save a session after it advanced its sampler, restore it, and continue both
/// copies with the same user turns: generated text and the complete saved
/// state must stay identical.
fn assert_restore_continues_identically(api: &IntegerCapabilityApi, original: u32) -> Result<()> {
    let restored = restore_and_compare_saved_state(api, original)?;
    for turn in ["What did I say first?", "Say it again."] {
        api.ingest_user_turn(original, turn)?;
        api.ingest_user_turn(restored, turn)?;
        let original_text = api.generate_text(original, 12)?;
        let restored_text = api.generate_text(restored, 12)?;
        assert_eq!(
            restored_text, original_text,
            "continuation text after '{turn}'"
        );
        assert_eq!(
            saved_json(api, restored)?,
            saved_json(api, original)?,
            "saved state after '{turn}'"
        );
    }
    Ok(())
}

#[test]
fn metadata_declares_the_contract_without_certifying_it() -> Result<()> {
    let api = IntegerCapabilityApi::new(create_test_bundle_with_byte_vocab());
    let meta = api.metadata();
    assert_eq!(meta.api_schema_version, CAPABILITY_API_SCHEMA);
    assert_eq!(
        meta.declared_numerical_contract,
        DECLARED_NUMERICAL_CONTRACT
    );
    assert!(meta.contract_qualification.contains("not certified"));
    assert!(meta
        .contract_qualification
        .contains("audit_zero_matmul_serving.py"));
    let matrix = serde_json::to_string(&meta.truth_matrix)?;
    for claim in ["Certified", "Qualified", "ms/tok", "MiB"] {
        assert!(!matrix.contains(claim), "truth matrix states '{claim}'");
    }
    Ok(())
}

#[test]
fn restored_session_keeps_sampler_policy_and_read_mode_and_continues_identically() -> Result<()> {
    let api = IntegerCapabilityApi::new(create_test_bundle_with_byte_vocab());
    let original = api.create_session(SessionConfig {
        system_prompt: Some("Persistent persona.".into()),
        seed: 1234,
        read_mode: ReadMode::NoRead,
        policy: SamplePolicy::Categorical { top_k: 0 },
    })?;
    api.ingest_user_turn(original, "Remember the code alpha-7.")?;
    api.generate_text(original, 8)?;
    let telemetry = api.get_telemetry(original)?;
    assert_eq!(telemetry.read_mode, ReadMode::NoRead);
    assert_eq!(telemetry.policy, SamplePolicy::Categorical { top_k: 0 });
    assert_ne!(
        telemetry.prng_state, 1234,
        "generation must advance the sampler so its restoration is tested"
    );
    assert_restore_continues_identically(&api, original)
}

#[test]
fn empty_prompt_is_rejected_without_changing_the_session() -> Result<()> {
    let api = IntegerCapabilityApi::new(create_test_bundle_with_byte_vocab());
    let id = api.create_session(SessionConfig::default())?;
    api.ingest_user_turn(id, "hello")?;
    let before = saved_json(&api, id)?;
    for prompt in ["", "   ", "\n\t"] {
        assert!(api.ingest_user_turn(id, prompt).is_err());
    }
    assert_eq!(saved_json(&api, id)?, before);
    Ok(())
}

/// Load the width-576 fixture and run one turn in a new session.
fn dialogue576_session() -> Result<(IntegerCapabilityApi, u32)> {
    let bundle_path = Path::new(DIALOGUE576_BUNDLE_PATH);
    require_fixture(bundle_path)?;
    let api = IntegerCapabilityApi::new(Bundle::load(bundle_path)?);
    let session_id = api.create_session(SessionConfig {
        system_prompt: Some("You are a helpful mathematical assistant.".into()),
        policy: SamplePolicy::Greedy,
        read_mode: ReadMode::Enabled,
        seed: 42,
    })?;
    let ingested = api.ingest_user_turn(session_id, "What is the square root of 64?")?;
    assert!(ingested > 0);
    api.generate_text(session_id, 16)?;
    Ok((api, session_id))
}

#[test]
#[ignore = "requires the local dialogue-child-bundle-1 fixture; run with --ignored"]
fn capability_api_lifecycle_on_dialogue576_bundle() -> Result<()> {
    let (api, session_id) = dialogue576_session()?;
    let meta = api.metadata();
    assert_eq!(meta.api_schema_version, CAPABILITY_API_SCHEMA);
    assert_eq!(meta.width, 576);
    assert_eq!(meta.vocab_size, 4096);
    assert_eq!(meta.context_capacity, 256);
    assert_eq!(
        meta.declared_numerical_contract,
        DECLARED_NUMERICAL_CONTRACT
    );

    let telemetry = api.get_telemetry(session_id)?;
    assert_eq!(telemetry.session_id, session_id);
    assert!(telemetry.active_tokens > 0);
    assert_eq!(telemetry.context_capacity, 256);

    let saved = api.save_session(session_id)?;
    assert_eq!(saved.schema, "uor-r4.integer-session/1");
    assert_eq!(saved.session_state.state.len(), 576);
    restore_and_compare_saved_state(&api, session_id)?;
    Ok(())
}

#[test]
#[ignore = "known failure, tracked in #1476: uor-r4.integer-session/1 omits the width-576 value stores (persistent_values_576, dialogue_values_576, l2_pages_576), so a restored width-576 session diverges at its first step; also requires the local dialogue-child-bundle-1 fixture"]
fn restored_dialogue576_session_continues_identically() -> Result<()> {
    let (api, session_id) = dialogue576_session()?;
    assert_restore_continues_identically(&api, session_id)
}
