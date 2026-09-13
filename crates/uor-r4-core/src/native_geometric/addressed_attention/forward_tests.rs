use super::*;
use crate::native_geometric::addressed_attention::artifact::Provenance;
use crate::native_geometric::addressed_attention::policy::{
    implementation_digest, CompiledPolicy, InterpretedPolicy,
};
use crate::report_output;
use std::fs;
use std::path::Path;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const DATA: &[u8] = b"Name: Ada. Place: Lima.\nlet a = 19; let b = 7; a - b;\nNext: answer.\n";
fn build(
    parameters: &Parameters,
    data: &[u8],
    config: &[u8],
) -> std::result::Result<Model, Box<dyn std::error::Error>> {
    Ok(Model::new(
        parameters.compile()?,
        Provenance {
            seed: parameters.seed(),
            training_data_digest: *blake3::hash(data).as_bytes(),
            training_config_digest: *blake3::hash(config).as_bytes(),
            parameter_digest: parameters.digest(),
            source_digest: implementation_digest(),
            parent: None,
        },
    )?)
}
#[test]
fn complete_deterministic_parameter_export_trace_and_pending_snapshot_parity() -> TestResult {
    let params = Parameters::seeded(7341)?;
    let model = build(&params, DATA, b"parity-test-only")?;
    let restored_model = Model::decode(&model.encode())?;
    assert_eq!(model, restored_model);
    let mut compiled = CompiledPolicy {
        compiled: model.compiled(),
    };
    let mut interpreted = InterpretedPolicy {
        parameters: &params,
    };
    let mut a = RuntimeSession::new(model.id(), model.geometry(), 77)?;
    let mut b = a.clone();
    for (i, &byte) in DATA.iter().enumerate() {
        if i == 16 {
            a.begin_turn()?;
            b.begin_turn()?;
        }
        let offered = a.predict(model.geometry(), &mut compiled)?;
        let other = b.predict(model.geometry(), &mut interpreted)?;
        assert_eq!(offered, other);
        assert_eq!(a.snapshot()?, b.snapshot()?);
        let pending = a.snapshot()?;
        let mut restored = RuntimeSession::restore(&pending, model.id(), model.geometry())?;
        assert_eq!(restored.predict(model.geometry(), &mut compiled)?, offered);
        let ta = a.observe(
            Symbol::Byte(byte),
            offered.offer.id,
            model.geometry(),
            &mut compiled,
        )?;
        let tb = b.observe(
            Symbol::Byte(byte),
            other.offer.id,
            model.geometry(),
            &mut interpreted,
        )?;
        let tc = restored.observe(
            Symbol::Byte(byte),
            offered.offer.id,
            model.geometry(),
            &mut compiled,
        )?;
        assert_eq!(ta, tb);
        assert_eq!(ta, tc);
        assert_eq!(a, b);
        assert_eq!(a, restored);
        assert!(ta.circuit_calls() <= 8);
        assert!(ta.candidate_scores <= 530);
    }
    let mut changed = params.values().to_vec();
    changed[0] = f64::NAN;
    assert!(Parameters::from_values(1, changed).is_err());
    Ok(())
}
#[test]
fn current_target_affects_only_direct_ce_not_offer_or_path_scores() -> TestResult {
    let params = Parameters::seeded(7341)?;
    let model = build(&params, DATA, b"target-causality-test-only")?;
    let mut a = RuntimeSession::new(model.id(), model.geometry(), 1)?;
    let mut b = a.clone();
    let mut pa = SampledPolicy::new(&params, 973, 0, 0);
    let mut pb = SampledPolicy::new(&params, 973, 0, 0);
    pa.set_target(65, 64)?;
    pb.set_target(66, 64)?;
    let oa = a.predict(model.geometry(), &mut pa)?;
    let ob = b.predict(model.geometry(), &mut pb)?;
    assert_eq!(oa, ob);
    assert_eq!(a, b);
    assert_eq!(pa.score(), pb.score());
    assert_ne!(pa.direct(), pb.direct());
    assert_eq!(pa.counts().scored_positions, 1);
    assert_eq!(pb.counts().scored_positions, 1);
    let counts = pa.counts();
    assert_eq!(a.predict(model.geometry(), &mut pa)?, oa);
    assert_eq!(pa.counts().gate_events, counts.gate_events);
    assert_eq!(pa.counts().head_events, counts.head_events);
    Ok(())
}

/// The executable report uses its own exclusively claimed directory and is
/// deliberately excluded from ordinary test runs. Root invokes it once after
/// unit parity passes, with UOR_ADDRESSED_FORWARD_REPORT set to a new path.
#[test]
#[ignore = "requires an exclusive report path and the recorded dry-run allowance"]
fn frozen_forward_backward_report() -> TestResult {
    let path = std::env::var("UOR_ADDRESSED_FORWARD_REPORT")?;
    let root = Path::new(&path);
    report_output::claim(root)?;
    let result = (|| -> TestResult {
        let started = Instant::now();
        let config = serde_json::json!({"schema":"uor-r4.addressed-forward-dry-run/1","parameter_seed":7341,"event_seed":973,"wiring_seed":super::super::policy::WIRING_SEED,"particles":4,"positions":64,"initial_logits":"uniform[-.25,.25)","parameter_updates":0,"gradient":"complete mean loss, serial LOO, all invocation scores plus direct CE","construction_only":true});
        let config_bytes = serde_json::to_vec_pretty(&config)?;
        let data = &DATA[..64];
        fs::write(root.join("config.json"), &config_bytes)?;
        fs::write(root.join("input.bin"), data)?;
        let params = Parameters::seeded(7341)?;
        let model = build(&params, data, &config_bytes)?;
        let encoded = model.encode();
        fs::write(root.join("initialized-model.bin"), &encoded)?;
        let loaded = Model::decode(&fs::read(root.join("initialized-model.bin"))?)?;
        if loaded != model {
            return Err("actual artifact roundtrip mismatch".into());
        }
        let prepare_us = started.elapsed().as_micros();
        let report = frozen_four_particle(&params, &loaded, data, 973)?;
        if report.particles.len() != 4
            || report.particles.iter().any(|p| p.positions != 64)
            || report
                .particles
                .iter()
                .map(|p| p.eight_phase_ticks)
                .sum::<u64>()
                == 0
            || report.gradient_nonzero == 0
        {
            return Err("dry-run execution accounting".into());
        }
        fs::write(
            root.join("dry-run.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        fs::write(
            root.join("summary.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"status":"PASS_ADDRESSED_ATTENTION_FORWARD_INTEGRATION_GATE","model_id":hex::encode(model.id()),"model_bytes":encoded.len(),"source_digest":hex::encode(implementation_digest()),"prepare_and_export_reload_us":prepare_us,"total_us":started.elapsed().as_micros(),"dry_run_us":report.elapsed_us,"updates":0,"fit_calls":0,"dataset_scope":"64-byte construction-only instrumentation input; not held-out language evidence"}),
            )?,
        )?;

        println!(
            "{}",
            serde_json::to_string(
                &serde_json::json!({"status":report.status,"particles":4,"positions_per_particle":64,"model_bytes":report.model_bytes,"dense_training_bytes":report.max_dense_training_bytes,"elapsed_us":report.elapsed_us,"gradient_nonzero":report.gradient_nonzero,"gradient_l2":report.gradient_l2,"mean_ce":report.initialization_mean_conditional_ce})
            )?
        );
        Ok(())
    })();
    if let Err(error) = &result {
        fs::write(
            root.join("incomplete.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"status":"INCOMPLETE_FORWARD_DRY_RUN","error":error.to_string(),"parameter_updates":0,"fit_calls":0}),
            )?,
        )?;
    }
    report_output::seal(root)?;
    report_output::verify(root)?;
    result
}
