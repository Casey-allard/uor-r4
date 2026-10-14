//! Prime-Zeta Complex Routing finite construction qualification driver.
//!
//! Authority:
//! - Decision D32 October 14 Addendum (#2398)
//! - Mechanism brief: `docs/mechanisms/prime-zeta-complex-routing.md`
//! - Pre-registration sheet: `docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`
//!   (TEST FITNESS: FIT, #2433, sha256 `26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7`)
//! - Owner release: #2029 comment 6170312990
//!
//! Scope: finite construction qualification, zero training, zero corpus dose,
//! zero `.apply` calls. Host-side `f64` arithmetic is strictly confined to
//! one-time offline codebook generation up to the integer quantization boundary.
//! All model forward, routing, flock reads, counterfactual proposals and
//! snapshot validations run exclusively on CUDA (`--features cuda`).

#[path = "../prime_zeta.rs"]
mod prime_zeta;

use prime_zeta::{
    compare_codebooks, generate_codebook, token_witness, CodebookMode, QuantizedCodebook,
    TANGENT_NULL_DELTA,
};
use serde_json::{json, Value};
use std::path::Path;
use uor_r4_discrete::{Error, Result, Spec, SCHEMA};

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(invalid(why))
    }
}

fn write_pretty_json(path: &Path, payload: &Value) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    serde_json::to_writer_pretty(&file, payload).map_err(|e| invalid(e.to_string()))?;
    file.sync_all().map_err(|e| invalid(e.to_string()))
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
struct HostVerificationBundle {
    b4_seed0: QuantizedCodebook,
    b4_tangent: QuantizedCodebook,
    b4_normal0: QuantizedCodebook,
    b4_normal1: QuantizedCodebook,
    b4_normal2: QuantizedCodebook,
    b2_seed0: QuantizedCodebook,
    b2_tangent: QuantizedCodebook,
    b2_normal0: QuantizedCodebook,
    summary: Value,
    witnesses: Value,
}

fn run_host_codebook_verification() -> Result<HostVerificationBundle> {
    let spec_b4 = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 64,
        kept_cells: 8,
    };
    let spec_b2 = Spec {
        vocabulary: 4096,
        blocks: 2,
        context: 64,
        kept_cells: 8,
    };
    let spec_v256_b2 = Spec {
        vocabulary: 256,
        blocks: 2,
        context: 64,
        kept_cells: 8,
    };

    // 1. Generate V4096 B4 across seeds 0, 1, 2
    let b4_seed0 = generate_codebook(spec_b4, 0, CodebookMode::PrimeZeta)?;
    let b4_seed1 = generate_codebook(spec_b4, 1, CodebookMode::PrimeZeta)?;
    let b4_seed2 = generate_codebook(spec_b4, 2, CodebookMode::PrimeZeta)?;
    require(
        b4_seed0.table == b4_seed1.table && b4_seed1.table == b4_seed2.table,
        "V4096 B4 PrimeZeta codebook must be bit-exact across seeds 0, 1, 2",
    )?;
    require(
        b4_seed0.unique_phase_vectors == 4096
            && b4_seed0.unique_spinor_vectors == 4096
            && b4_seed0.unique_table_vectors == 4096
            && b4_seed0.occupied_writer_cells == 256,
        "V4096 B4 PrimeZeta codebook must have 0 full-vector collisions (4096/4096) and 256/256 writer cells",
    )?;

    // 2. Generate V4096 B2 across seeds 0, 1, 2 (sheet split: fixture-tokens V4096 B2 context64)
    let b2_seed0 = generate_codebook(spec_b2, 0, CodebookMode::PrimeZeta)?;
    let b2_seed1 = generate_codebook(spec_b2, 1, CodebookMode::PrimeZeta)?;
    let b2_seed2 = generate_codebook(spec_b2, 2, CodebookMode::PrimeZeta)?;
    require(
        b2_seed0.table == b2_seed1.table && b2_seed1.table == b2_seed2.table,
        "V4096 B2 PrimeZeta codebook must be bit-exact across seeds 0, 1, 2",
    )?;
    require(
        b2_seed0.unique_phase_vectors == 4096
            && b2_seed0.unique_spinor_vectors == 4096
            && b2_seed0.unique_table_vectors == 256
            && b2_seed0.occupied_writer_cells == 256,
        "V4096 B2 PrimeZeta codebook must have 4096/4096 unique PhaseQ29/UnitS3Q30 vectors and saturate all 256/256 ROOTS^2 cells",
    )?;

    // 3. Generate V256 B2 to verify 0 full-vector collisions at B=2 within 16^2=256 orbit capacity
    let v256_b2 = generate_codebook(spec_v256_b2, 0, CodebookMode::PrimeZeta)?;
    require(
        v256_b2.unique_table_vectors == 256 && v256_b2.occupied_writer_cells == 256,
        "V256 B2 PrimeZeta codebook must have 256/256 unique table vectors (0 collisions)",
    )?;

    // 4. Construct Tangent Null (delta = 1.0) and Normal Nulls (seeds 0, 1, 2)
    let b4_tangent = generate_codebook(
        spec_b4,
        0,
        CodebookMode::TangentNull {
            delta: TANGENT_NULL_DELTA,
        },
    )?;
    let b4_normal0 = generate_codebook(spec_b4, 0, CodebookMode::NormalNull { seed: 0 })?;
    let b4_normal1 = generate_codebook(spec_b4, 1, CodebookMode::NormalNull { seed: 1 })?;
    let b4_normal2 = generate_codebook(spec_b4, 2, CodebookMode::NormalNull { seed: 2 })?;
    let b4_normal0_repeat = generate_codebook(spec_b4, 0, CodebookMode::NormalNull { seed: 0 })?;
    require(
        b4_normal0.table == b4_normal0_repeat.table
            && b4_normal0.table != b4_normal1.table
            && b4_normal1.table != b4_normal2.table,
        "NormalNull must be bit-exact reproducible per seed and distinct across seeds 0, 1, 2",
    )?;
    require(
        b4_tangent.unique_table_vectors == 4096
            && b4_normal0.unique_table_vectors == 4096
            && b4_normal1.unique_table_vectors == 4096
            && b4_normal2.unique_table_vectors == 4096,
        "TangentNull and NormalNull V4096 B4 codebooks must also have 4096/4096 unique table vectors",
    )?;

    let b2_tangent = generate_codebook(
        spec_b2,
        0,
        CodebookMode::TangentNull {
            delta: TANGENT_NULL_DELTA,
        },
    )?;
    let b2_normal0 = generate_codebook(spec_b2, 0, CodebookMode::NormalNull { seed: 0 })?;

    // 5. Structural Hamming and L1 separation checks
    let b4_vs_tangent = compare_codebooks(&b4_seed0, &b4_tangent)?;
    let b4_vs_normal0 = compare_codebooks(&b4_seed0, &b4_normal0)?;
    let b4_vs_normal1 = compare_codebooks(&b4_seed0, &b4_normal1)?;
    let b4_vs_normal2 = compare_codebooks(&b4_seed0, &b4_normal2)?;
    let b4_tangent_vs_normal0 = compare_codebooks(&b4_tangent, &b4_normal0)?;

    let b2_vs_tangent = compare_codebooks(&b2_seed0, &b2_tangent)?;
    let b2_vs_normal0 = compare_codebooks(&b2_seed0, &b2_normal0)?;

    require(
        b4_vs_tangent.differing_tokens > 3900
            && b4_vs_normal0.differing_tokens > 3900
            && b4_tangent_vs_normal0.differing_tokens > 3900
            && b2_vs_tangent.differing_tokens > 3500
            && b2_vs_normal0.differing_tokens > 3500,
        "PrimeZeta codebook failed structural Hamming separation bar against TangentNull or NormalNull",
    )?;

    // 6. Analytic witness tokens: 0 (p=2), 1 (p=3), 2 (p=5), 3 (p=7), 255 (p=1619), 4095 (p=38873)
    let witness_ids = [0u32, 1, 2, 3, 255, 1023, 2047, 4095];
    let mut b4_witnesses = Vec::with_capacity(witness_ids.len());
    let mut b2_witnesses = Vec::with_capacity(witness_ids.len());
    for &tid in &witness_ids {
        b4_witnesses.push(token_witness(&b4_seed0, tid)?);
        b2_witnesses.push(token_witness(&b2_seed0, tid)?);
    }

    let summary = json!({
        "schema": SCHEMA,
        "mechanism": "prime-zeta-complex-routing",
        "sheet_path": "docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md",
        "sheet_sha256": "26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7",
        "driver_source_sha256": uor_r4_discrete::training::driver_source_sha256(),
        "operator_kappa": prime_zeta::geometry::OPERATOR_KAPPA,
        "composition_kappa": prime_zeta::geometry::COMPOSITION_KAPPA,
        "float_scope": "host_only_one_time_offline_pre_quantization",
        "seed_reproducibility_0_1_2": {
            "v4096_b4_bit_exact": true,
            "v4096_b2_bit_exact": true,
            "normal_null_per_seed_bit_exact": true,
            "normal_null_across_seeds_distinct": true
        },
        "codebooks": {
            "v4096_b4_prime_zeta": {
                "table_sha256": b4_seed0.table_sha256,
                "phases_sha256": b4_seed0.phases_sha256,
                "spinors_sha256": b4_seed0.spinors_sha256,
                "unique_phase_vectors": b4_seed0.unique_phase_vectors,
                "unique_spinor_vectors": b4_seed0.unique_spinor_vectors,
                "unique_table_vectors": b4_seed0.unique_table_vectors,
                "full_vector_collisions": 4096 - b4_seed0.unique_table_vectors,
                "occupied_writer_cells": b4_seed0.occupied_writer_cells
            },
            "v4096_b4_tangent_null_delta_1_0": {
                "table_sha256": b4_tangent.table_sha256,
                "phases_sha256": b4_tangent.phases_sha256,
                "spinors_sha256": b4_tangent.spinors_sha256,
                "unique_table_vectors": b4_tangent.unique_table_vectors,
                "occupied_writer_cells": b4_tangent.occupied_writer_cells
            },
            "v4096_b4_normal_null_seed_0": {
                "table_sha256": b4_normal0.table_sha256,
                "phases_sha256": b4_normal0.phases_sha256,
                "spinors_sha256": b4_normal0.spinors_sha256,
                "unique_table_vectors": b4_normal0.unique_table_vectors,
                "occupied_writer_cells": b4_normal0.occupied_writer_cells
            },
            "v4096_b4_normal_null_seed_1": {
                "table_sha256": b4_normal1.table_sha256,
                "unique_table_vectors": b4_normal1.unique_table_vectors
            },
            "v4096_b4_normal_null_seed_2": {
                "table_sha256": b4_normal2.table_sha256,
                "unique_table_vectors": b4_normal2.unique_table_vectors
            },
            "v4096_b2_prime_zeta": {
                "table_sha256": b2_seed0.table_sha256,
                "phases_sha256": b2_seed0.phases_sha256,
                "spinors_sha256": b2_seed0.spinors_sha256,
                "unique_phase_vectors": b2_seed0.unique_phase_vectors,
                "unique_spinor_vectors": b2_seed0.unique_spinor_vectors,
                "unique_table_vectors": b2_seed0.unique_table_vectors,
                "occupied_writer_cells": b2_seed0.occupied_writer_cells,
                "note": "At B=2, discrete_validate restricts each 8D block to the 16-root orbit so ROOTS^2 has 256 total vectors; all 256/256 cells are populated with exact quota 16, while PhaseQ29 and UnitS3Q30 have 4096/4096 unique vectors (0 collisions)."
            },
            "v256_b2_prime_zeta": {
                "table_sha256": v256_b2.table_sha256,
                "unique_table_vectors": v256_b2.unique_table_vectors,
                "full_vector_collisions": 256 - v256_b2.unique_table_vectors,
                "occupied_writer_cells": v256_b2.occupied_writer_cells
            }
        },
        "structural_separation": {
            "v4096_b4_prime_zeta_vs_tangent_null": b4_vs_tangent,
            "v4096_b4_prime_zeta_vs_normal_null_seed0": b4_vs_normal0,
            "v4096_b4_prime_zeta_vs_normal_null_seed1": b4_vs_normal1,
            "v4096_b4_prime_zeta_vs_normal_null_seed2": b4_vs_normal2,
            "v4096_b4_tangent_null_vs_normal_null_seed0": b4_tangent_vs_normal0,
            "v4096_b2_prime_zeta_vs_tangent_null": b2_vs_tangent,
            "v4096_b2_prime_zeta_vs_normal_null_seed0": b2_vs_normal0
        },
        "training": "HELD",
        "applied_updates": 0
    });

    let witnesses = json!({
        "schema": SCHEMA,
        "mechanism": "prime-zeta-complex-routing",
        "v4096_b4_table_sha256": b4_seed0.table_sha256,
        "v4096_b2_table_sha256": b2_seed0.table_sha256,
        "v4096_b4_witnesses": b4_witnesses,
        "v4096_b2_witnesses": b2_witnesses
    });

    Ok(HostVerificationBundle {
        b4_seed0,
        b4_tangent,
        b4_normal0,
        b4_normal1,
        b4_normal2,
        b2_seed0,
        b2_tangent,
        b2_normal0,
        summary,
        witnesses,
    })
}

#[cfg(feature = "cuda")]
mod cuda_qualify {
    use super::*;
    use prime_zeta::verify_zero_float_ptx;
    use uor_r4_discrete::{
        device::{compiled_identity, Device, Snapshot},
        Identity, Parameter, ReadSpec,
    };

    const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/discrete.ptx"));

    fn identity(token: u32, occurrence: u64) -> Identity {
        Identity {
            token,
            occurrence,
            record: 2433,
            version: 1,
        }
    }

    fn build_snapshot(
        spec: Spec,
        read_spec: ReadSpec,
        seed: u32,
        table: &[i32],
    ) -> Result<Snapshot> {
        let base = Device::new_with_read(spec, read_spec, seed, 0)?;
        let mut snap = base.snapshot()?;
        require(
            snap.table.len() == table.len(),
            "codebook table length mismatch against device snapshot",
        )?;
        snap.table.copy_from_slice(table);
        Ok(snap)
    }

    fn sample_windows_v4096() -> (Vec<Vec<Identity>>, Vec<u32>) {
        let windows = vec![
            vec![
                identity(0, 0),
                identity(1, 1),
                identity(2, 2),
                identity(3, 3),
                identity(255, 4),
                identity(1023, 5),
                identity(2047, 6),
                identity(4095, 7),
            ],
            vec![
                identity(2, 8),
                identity(5, 9),
                identity(11, 10),
                identity(23, 11),
                identity(47, 12),
                identity(97, 13),
            ],
            vec![identity(4095, 14), identity(2047, 15), identity(1023, 16)],
            vec![
                identity(13, 17),
                identity(29, 18),
                identity(59, 19),
                identity(113, 20),
            ],
        ];
        let targets = vec![17u32, 199, 511, 3001];
        (windows, targets)
    }

    pub fn run_full_cuda_qualification(report_dir: &Path) -> Result<()> {
        std::fs::create_dir(report_dir)
            .map_err(|e| invalid(format!("exclusive report directory: {e}")))?;

        // Step 1: Verify zero floating-point instructions in AOT-compiled PTX
        let ptx_entries = verify_zero_float_ptx(PTX)?;
        require(
            ptx_entries == 11,
            "AOT PTX must expose all 11 integer kernel entries with 0 float instructions",
        )?;

        // Step 2: Run host-side offline codebook quantization and null separation checks
        let bundle = run_host_codebook_verification()?;
        write_pretty_json(
            &report_dir.join("host-codebook-qualification.json"),
            &bundle.summary,
        )?;
        write_pretty_json(&report_dir.join("v4096-witnesses.json"), &bundle.witnesses)?;

        // Step 3: Run the 17 base discrete CUDA construction controls (11 Route + 6 Flock)
        let route_dir = report_dir.join("route-controls");
        uor_r4_discrete::qualification::run(&route_dir)?;
        let flock_dir = report_dir.join("flock-controls");
        uor_r4_discrete::flock_qualification::run(&flock_dir)?;

        // Step 4: Run the 6 Prime-Zeta CUDA construction controls (C18..C23)
        let mut pz_controls = Vec::new();

        // C18: V4096 B4 CUDA snapshot validation (discrete_validate kernel) and round-trip
        let snap_b4_s0 = build_snapshot(
            bundle.b4_seed0.spec,
            ReadSpec::Route,
            0,
            &bundle.b4_seed0.table,
        )?;
        let dev_b4_s0 = Device::from_snapshot(snap_b4_s0.clone(), 0)?;
        let roundtrip_b4 = dev_b4_s0.snapshot()?;
        require(
            roundtrip_b4.table == bundle.b4_seed0.table && roundtrip_b4.epoch == 0,
            "C18: V4096 B4 CUDA snapshot round-trip altered PrimeZeta codebook or epoch",
        )?;
        pz_controls.push(json!({
            "index": 18,
            "name": "prime_zeta_v4096_b4_cuda_snapshot_validation_and_zero_collision",
            "status": "PASS",
            "unique_table_vectors": bundle.b4_seed0.unique_table_vectors,
            "occupied_writer_cells": bundle.b4_seed0.occupied_writer_cells
        }));

        // C19: V4096 B2 CUDA snapshot validation (sheet split: fixture-tokens V4096 B2 context64)
        let snap_b2_s0 = build_snapshot(
            bundle.b2_seed0.spec,
            ReadSpec::Route,
            0,
            &bundle.b2_seed0.table,
        )?;
        let dev_b2_s0 = Device::from_snapshot(snap_b2_s0, 0)?;
        let roundtrip_b2 = dev_b2_s0.snapshot()?;
        require(
            roundtrip_b2.table == bundle.b2_seed0.table && roundtrip_b2.epoch == 0,
            "C19: V4096 B2 CUDA snapshot round-trip altered PrimeZeta codebook or epoch",
        )?;
        let (windows, targets) = sample_windows_v4096();
        let carried_b2 = dev_b2_s0.carry_windows(&windows)?;
        let fwd_b2 = dev_b2_s0.forward(&carried_b2, &targets)?;
        let eval_b2 = dev_b2_s0.evaluate(&carried_b2, &targets)?;
        require(
            fwd_b2.predictions.len() == 4 && eval_b2.aggregate.scored_rows == 4,
            "C19: V4096 B2 CUDA forward/evaluate row count mismatch",
        )?;
        pz_controls.push(json!({
            "index": 19,
            "name": "prime_zeta_v4096_b2_cuda_snapshot_validation",
            "status": "PASS",
            "predictions": fwd_b2.predictions,
            "losses": fwd_b2.losses
        }));

        // C20: Bit-exact CUDA forward/evaluate reproducibility across seeds 0, 1, 2
        let dev_b4_s1 = Device::from_snapshot(
            build_snapshot(
                bundle.b4_seed0.spec,
                ReadSpec::Route,
                1,
                &bundle.b4_seed0.table,
            )?,
            0,
        )?;
        let dev_b4_s2 = Device::from_snapshot(
            build_snapshot(
                bundle.b4_seed0.spec,
                ReadSpec::Route,
                2,
                &bundle.b4_seed0.table,
            )?,
            0,
        )?;
        let c0 = dev_b4_s0.carry_windows(&windows)?;
        let c1 = dev_b4_s1.carry_windows(&windows)?;
        let c2 = dev_b4_s2.carry_windows(&windows)?;
        let f0 = dev_b4_s0.forward(&c0, &targets)?;
        let f1 = dev_b4_s1.forward(&c1, &targets)?;
        let f2 = dev_b4_s2.forward(&c2, &targets)?;
        let e0 = dev_b4_s0.evaluate(&c0, &targets)?;
        let e1 = dev_b4_s1.evaluate(&c1, &targets)?;
        let e2 = dev_b4_s2.evaluate(&c2, &targets)?;
        require(
            c0.values()? == c1.values()?
                && c1.values()? == c2.values()?
                && f0.vectors()? == f1.vectors()?
                && f1.vectors()? == f2.vectors()?
                && f0.predictions == f1.predictions
                && f1.predictions == f2.predictions
                && f0.losses == f1.losses
                && f1.losses == f2.losses
                && e0.aggregate == e1.aggregate
                && e1.aggregate == e2.aggregate,
            "C20: CUDA forward/evaluate differed across seeds 0, 1, 2",
        )?;
        pz_controls.push(json!({
            "index": 20,
            "name": "prime_zeta_seed_reproducibility_0_1_2_cuda",
            "status": "PASS",
            "predictions": f0.predictions,
            "losses": f0.losses,
            "total_loss_numerator_sum": e0.aggregate.total_loss_numerator_sum
        }));

        // C21: CUDA validation and forward/writer separation against TangentNull and NormalNulls
        let dev_tangent = Device::from_snapshot(
            build_snapshot(
                bundle.b4_tangent.spec,
                ReadSpec::Route,
                0,
                &bundle.b4_tangent.table,
            )?,
            0,
        )?;
        let dev_normal0 = Device::from_snapshot(
            build_snapshot(
                bundle.b4_normal0.spec,
                ReadSpec::Route,
                0,
                &bundle.b4_normal0.table,
            )?,
            0,
        )?;
        let dev_normal1 = Device::from_snapshot(
            build_snapshot(
                bundle.b4_normal1.spec,
                ReadSpec::Route,
                1,
                &bundle.b4_normal1.table,
            )?,
            0,
        )?;
        let dev_normal2 = Device::from_snapshot(
            build_snapshot(
                bundle.b4_normal2.spec,
                ReadSpec::Route,
                2,
                &bundle.b4_normal2.table,
            )?,
            0,
        )?;
        let ct = dev_tangent.carry_windows(&windows)?;
        let cn0 = dev_normal0.carry_windows(&windows)?;
        let cn1 = dev_normal1.carry_windows(&windows)?;
        let cn2 = dev_normal2.carry_windows(&windows)?;
        let ft = dev_tangent.forward(&ct, &targets)?;
        let fn0 = dev_normal0.forward(&cn0, &targets)?;
        let fn1 = dev_normal1.forward(&cn1, &targets)?;
        let fn2 = dev_normal2.forward(&cn2, &targets)?;
        require(
            c0.values()? != ct.values()?
                && c0.values()? != cn0.values()?
                && ct.values()? != cn0.values()?
                && f0.vectors()? != ft.vectors()?
                && f0.vectors()? != fn0.vectors()?
                && (f0.losses != ft.losses || f0.admitted_cells != ft.admitted_cells)
                && (f0.losses != fn0.losses || f0.admitted_cells != fn0.admitted_cells),
            "C21: CUDA carried vectors/forward outputs failed to separate PrimeZeta from TangentNull and NormalNull",
        )?;

        // Also validate B=2 TangentNull and NormalNull on CUDA
        let _dev_b2_tangent = Device::from_snapshot(
            build_snapshot(
                bundle.b2_tangent.spec,
                ReadSpec::Route,
                0,
                &bundle.b2_tangent.table,
            )?,
            0,
        )?;
        let _dev_b2_normal0 = Device::from_snapshot(
            build_snapshot(
                bundle.b2_normal0.spec,
                ReadSpec::Route,
                0,
                &bundle.b2_normal0.table,
            )?,
            0,
        )?;
        pz_controls.push(json!({
            "index": 21,
            "name": "prime_zeta_tangent_and_normal_null_cuda_separation",
            "status": "PASS",
            "prime_zeta_losses": f0.losses,
            "tangent_null_losses": ft.losses,
            "normal_null_seed0_losses": fn0.losses,
            "normal_null_seed1_losses": fn1.losses,
            "normal_null_seed2_losses": fn2.losses
        }));

        // C22: Route counterfactual proposal non-vacuity and zero state mutation on V4096 B4
        let prop_vec = dev_b4_s0
            .propose(&c0, &targets, Parameter::VectorBlock(0))?
            .observation()?;
        let prop_vec_repeat = dev_b4_s0
            .propose(&c0, &targets, Parameter::VectorBlock(0))?
            .observation()?;
        let prop_route = dev_b4_s0
            .propose(&c0, &targets, Parameter::Route(0))?
            .observation()?;
        let after_propose = dev_b4_s0.snapshot()?;
        require(
            prop_vec == prop_vec_repeat
                && prop_vec.epoch == 0
                && prop_route.epoch == 0
                && after_propose.table == bundle.b4_seed0.table
                && after_propose.epoch == 0,
            "C22: counterfactual proposal on V4096 B4 was non-deterministic or mutated state",
        )?;
        pz_controls.push(json!({
            "index": 22,
            "name": "prime_zeta_route_counterfactual_nonvacuity_v4096",
            "status": "PASS",
            "vector_block0_proposal": {
                "candidate": prop_vec.candidate,
                "before": prop_vec.before,
                "after": prop_vec.after,
                "sensitive": prop_vec.any_candidate_cost_differs_from_identity
            },
            "route0_proposal": {
                "candidate": prop_route.candidate,
                "before": prop_route.before,
                "after": prop_route.after,
                "sensitive": prop_route.any_candidate_cost_differs_from_identity
            }
        }));

        // C23: Flock read and Gate counterfactual on V4096 B4 PrimeZeta codebook
        let dev_flock = Device::from_snapshot(
            build_snapshot(
                bundle.b4_seed0.spec,
                ReadSpec::Flock { kept: 4 },
                0,
                &bundle.b4_seed0.table,
            )?,
            0,
        )?;
        let cf = dev_flock.carry_windows(&windows)?;
        let ff = dev_flock.forward_traced(&cf, &targets)?;
        let trace = ff
            .flock_trace()?
            .ok_or_else(|| invalid("C23: flock forward_traced returned no trace"))?;
        let gate_prop = dev_flock
            .propose(&cf, &targets, Parameter::Gate(0))?
            .observation()?;
        let after_flock = dev_flock.snapshot()?;
        require(
            !trace.is_empty()
                && gate_prop.epoch == 0
                && after_flock.table == bundle.b4_seed0.table
                && after_flock.epoch == 0,
            "C23: flock read or gate counterfactual failed on V4096 B4 PrimeZeta codebook",
        )?;
        pz_controls.push(json!({
            "index": 23,
            "name": "prime_zeta_flock_read_and_gate_counterfactual_v4096",
            "status": "PASS",
            "flock_predictions": ff.predictions,
            "flock_losses": ff.losses,
            "gate0_proposal": {
                "candidate": gate_prop.candidate,
                "before": gate_prop.before,
                "after": gate_prop.after,
                "sensitive": gate_prop.any_candidate_cost_differs_from_identity
            }
        }));

        let executable = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
        let executable_sha256 = uor_r4_discrete::corpus::sha256(
            &std::fs::read(&executable).map_err(|e| invalid(e.to_string()))?,
        );
        let mem = dev_b4_s0.resource_metadata()?;

        let final_report = json!({
            "schema": SCHEMA,
            "status": "PASS",
            "mechanism": "prime-zeta-complex-routing",
            "sheet_path": "docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md",
            "sheet_sha256": "26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7",
            "owner_release_comment": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990",
            "compiled_identity": compiled_identity(),
            "driver_source_sha256": uor_r4_discrete::training::driver_source_sha256(),
            "executable_sha256": executable_sha256,
            "admit_ptx": {
                "zero_float_instructions": true,
                "verified_entries": ptx_entries
            },
            "base_controls": {
                "route_controls": "11/11 PASS",
                "flock_controls": "6/6 PASS",
                "total_base_controls": "17/17 PASS"
            },
            "prime_zeta_cuda_controls": {
                "registered": 6,
                "passed": 6,
                "controls": pz_controls
            },
            "device_resources": mem,
            "training": "HELD",
            "applied_updates": 0,
            "corpus_rows_consumed": 0
        });

        write_pretty_json(
            &report_dir.join("prime-zeta-qualification.json"),
            &final_report,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&final_report).map_err(|e| invalid(e.to_string()))?
        );
        Ok(())
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("host-verify") if args.len() == 2 => {
            let dir = Path::new(&args[1]);
            std::fs::create_dir(dir)
                .map_err(|e| invalid(format!("exclusive report directory: {e}")))?;
            let bundle = run_host_codebook_verification()?;
            write_pretty_json(&dir.join("host-codebook-qualification.json"), &bundle.summary)?;
            write_pretty_json(&dir.join("v4096-witnesses.json"), &bundle.witnesses)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle.summary)
                    .map_err(|e| invalid(e.to_string()))?
            );
            Ok(())
        }
        Some("qualify") if args.len() == 2 => {
            #[cfg(feature = "cuda")]
            {
                cuda_qualify::run_full_cuda_qualification(Path::new(&args[1]))
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err(invalid(
                    "prime-zeta-qualify qualify requires --features cuda; no CPU fallback (D24/D32)",
                ))
            }
        }
        _ => Err(invalid(
            "usage: prime-zeta-qualify host-verify NEW_REPORT_DIR | prime-zeta-qualify qualify NEW_REPORT_DIR (--features cuda)",
        )),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
