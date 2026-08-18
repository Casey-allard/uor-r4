//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! The library holds the pieces both sides share:
//!
//! - [`OpKernel`]: the COMPLETE arithmetic interface of the runtime. Its
//!   method set — add, sub, shift, xor, compare, table read — is enumerated
//!   here and contains no multiply. Every arithmetic operation the runtime
//!   performs goes through this kernel and increments its census, so the
//!   multiplication-free claim is by construction (no multiply exists in the
//!   interface) and by measurement (the census is printed per run).
//! - the derived popcount table: Hamming distance between bit signatures is
//!   XOR then table reads then adds — the stratum observable of the byte
//!   plane, repurposed as the runtime's only metric arithmetic.
//! - bit signatures: "a vector at each bit" — bit b of a signature records
//!   which side of threshold b the content falls on; a prefix of bits is an
//!   intersection of regions; prefix depth is resolution.
//!
//! Multiplication is confined to the COMPILER (offline, once, κ-pinned
//! outputs) and to the CERTIFIER (instrumentation). See docs/transformerless/PROOF.md.

// teacher/compiler are portable (math, types, containers); only their
// fs-dependent functions are cfg-gated per item (see those files).
pub mod bott_fock;
pub mod cd_space;
pub mod endomorphism;
pub mod lie_jordan;
pub mod region_store;

pub use reference_state::{ActiveFrontier, ActiveFrontierEntry, PackedEdgeRanges};
pub use runtime::{derive_popcount_table, hamming, sign_signature, OpKernel};
pub use score_q::ScoreQ;
pub use uor_r4_graph_runtime::runtime_state::{
    ReservedState, ReservedStateUpdate, RuntimeState, RuntimeStateLevel, SemanticStateSlot,
    TokenState, LOCAL_STATE_CAPACITY, SEGMENT_STATE_CAPACITY, SESSION_STATE_CAPACITY,
    TOKEN_STATE_CAPACITY,
};

#[cfg(test)]
mod witnesses {
    use super::runtime::{derive_popcount_table, hamming, sign_signature, OpKernel};

    // #787 E-c: the scanner implementation moved to the shared
    // `super::source_scan` module (one canonical copy, with falsifiers
    // and the explicit allowance mechanism); these witnesses now consume
    // it instead of carrying their own copy.
    use super::source_scan::scan_for_forbidden_arith;

    /// P-1: the popcount table matches its definition on all 256 bytes and
    /// carries the stratum partition sizes C(8,k).
    #[test]
    fn p1_popcount_table() {
        let t = derive_popcount_table();
        let mut sizes = [0usize; 9];
        for x in 0..=255u8 {
            assert_eq!(t[x as usize], x.count_ones() as u8);
            sizes[t[x as usize] as usize] += 1;
        }
        assert_eq!(sizes, [1, 8, 28, 56, 70, 56, 28, 8, 1]);
    }

    /// P-2: kernel Hamming equals the direct definition on random pairs,
    /// and the census records only kernel ops.
    #[test]
    fn p2_hamming_exact() {
        let pop = derive_popcount_table();
        let mut s = 0x1234u64;
        let mut rng = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s & 0xff) as u8
        };
        for _ in 0..64 {
            let a: Vec<u8> = (0..36).map(|_| rng()).collect();
            let b: Vec<u8> = (0..36).map(|_| rng()).collect();
            let direct: u32 = a.iter().zip(&b).map(|(x, y)| (x ^ y).count_ones()).sum();
            let mut k = OpKernel::default();
            assert_eq!(hamming(&mut k, &pop, &a, &b), direct as i64);
            assert_eq!(k.xors, 36);
            assert_eq!(k.table_reads, 36);
            assert_eq!(k.adds, 36);
        }
    }

    /// P-4: the runtime module's source contains no multiplication,
    /// division, or modulo operator on values. Doc lines and comments are
    /// stripped; dereference `*x` (star not preceded by an operand) is not
    /// an arithmetic operator and does not match. This makes the
    /// "no source-level mul/div/mod in the runtime" claim machine-checked
    /// on every test run rather than a review assertion.
    #[test]
    fn p4_runtime_source_scan() {
        let src = include_str!("runtime.rs");
        let outcome = scan_for_forbidden_arith(src);
        assert!(
            outcome.offenders.is_empty(),
            "value arithmetic in runtime.rs:\n{}",
            outcome.offenders.join("\n")
        );
        assert!(
            outcome.allowed.is_empty(),
            "runtime.rs is deployed hot path — no allowances:\n{}",
            outcome.allowed.join("\n")
        );
    }

    /// #787 E-b: `runtime.rs` delegates its SIMD dispatch to `simd.rs`,
    /// which is deliberately NOT arithmetic-scanned — it holds the
    /// vectorized kernels whose intrinsics necessarily encode multiply
    /// semantics. Its exclusion is sanctioned by per-kernel
    /// scalar-equivalence witnesses instead: `test_simd_hamming_equivalence`
    /// and `simd_dot_argmax_matches_scalar_reference` pin every dispatcher
    /// to the scalar reference the P-4 scan DOES cover. This test pins the
    /// exclusion itself: the set of runtime→simd delegation points must
    /// not grow without this witness being revisited.
    #[test]
    fn p4_simd_exclusion_is_pinned_to_its_equivalence_witnesses() {
        let src = include_str!("runtime.rs");
        let delegations = src.matches("simd::").count();
        assert!(
            delegations > 0,
            "runtime.rs no longer delegates to simd.rs — remove this pin"
        );
        assert_eq!(
            delegations, 11,
            "the runtime.rs simd:: reference count changed ({delegations} vs 11); \
             add or revisit the scalar-equivalence witnesses for the affected \
             kernels, then re-pin this count deliberately"
        );
    }

    /// P-4 extension: ALL graph-runtime modules are scanned with the same
    /// arithmetic restrictions until machine-code audit (issue #160)
    /// supersedes source-level witnessing. #787 E-b closed the gap the
    /// 2026-08-18 audit found: `lib.rs`, `packed_kernels.rs`,
    /// `patch_chain.rs`, `scoring.rs`, and `vp_tree.rs` were previously
    /// unscanned. Load-time construction code inside scanned modules may
    /// carry an explicit `p4-allow(load-time)` marker; the exact allowance
    /// list is pinned below so it can never grow silently.
    #[test]
    fn p4_contract_owned_graph_runtime_source_scan() {
        let modules = [
            (
                "engine.rs",
                include_str!("../../../uor-r4-graph-runtime/src/engine.rs"),
            ),
            (
                "lib.rs",
                include_str!("../../../uor-r4-graph-runtime/src/lib.rs"),
            ),
            (
                "msa_selector.rs",
                include_str!("../../../uor-r4-graph-runtime/src/msa_selector.rs"),
            ),
            (
                "packed_kernels.rs",
                include_str!("../../../uor-r4-graph-runtime/src/packed_kernels.rs"),
            ),
            (
                "patch_chain.rs",
                include_str!("../../../uor-r4-graph-runtime/src/patch_chain.rs"),
            ),
            (
                "route_attention.rs",
                include_str!("../../../uor-r4-graph-runtime/src/route_attention.rs"),
            ),
            (
                "routing.rs",
                include_str!("../../../uor-r4-graph-runtime/src/routing.rs"),
            ),
            (
                "runtime_state.rs",
                include_str!("../../../uor-r4-graph-runtime/src/runtime_state.rs"),
            ),
            (
                "scoring.rs",
                include_str!("../../../uor-r4-graph-runtime/src/scoring.rs"),
            ),
            (
                "status.rs",
                include_str!("../../../uor-r4-graph-runtime/src/status.rs"),
            ),
            (
                "vp_tree.rs",
                include_str!("../../../uor-r4-graph-runtime/src/vp_tree.rs"),
            ),
            // #787 E-b: the two ACTIVITY_OWNERS modules the audit found
            // unscanned — the active-frontier owner in this crate and the
            // wasm-router decode-support owner.
            ("reference_state.rs", include_str!("reference_state.rs")),
            (
                "wasm-router/r4g1.rs",
                include_str!("../../../../src/r4g1.rs"),
            ),
        ];
        let mut offenders = Vec::new();
        let mut allowed = Vec::new();
        for (name, src) in modules {
            let outcome = scan_for_forbidden_arith(src);
            for offender in outcome.offenders {
                offenders.push(format!("{name}: {offender}"));
            }
            for allowance in outcome.allowed {
                allowed.push(format!("{name}: {allowance}"));
            }
        }
        assert!(
            offenders.is_empty(),
            "value arithmetic in contract modules:\n{}",
            offenders.join("\n")
        );
        // The complete, reviewed allowance set. Adding a `p4-allow` marker
        // anywhere in these modules fails this witness until the new line
        // is justified here. All three sites are VP-tree CONSTRUCTION code
        // that runs at graph load, before the steady-state boundary: two
        // overflow-checked word-to-byte conversions (the query path uses
        // shifts for the same conversion) and the median split. The query
        // path is division- and multiplication-free.
        let expected_allowed = vec![
            "vp_tree.rs: line 59: let proto_start = \
             (node.prototype_word_start as usize).checked_mul(8)?; "
                .to_string(),
            "vp_tree.rs: line 60: let mask_start = \
             (node.mask_word_start as usize).checked_mul(8)?; "
                .to_string(),
            "vp_tree.rs: line 142: let split = distances.len() / 2; ".to_string(),
        ];
        assert_eq!(
            allowed, expected_allowed,
            "the p4-allow allowance set changed — review and re-pin it"
        );
    }

    /// P-3: sign signatures agree with the direct definition, bit for bit.
    #[test]
    fn p3_sign_signature() {
        let vals: Vec<i64> = (0..288).map(|i| (i as i64 * 7919) % 1000 - 500).collect();
        let thr: Vec<i64> = (0..288).map(|i| (i as i64 * 104729) % 1000 - 500).collect();
        let mut k = OpKernel::default();
        let sig = sign_signature(&mut k, &vals, &thr);
        for i in 0..288 {
            let want = vals[i] > thr[i];
            let got = sig[i / 8] >> (i % 8) & 1 == 1;
            assert_eq!(want, got, "bit {}", i);
        }
        assert_eq!(k.compares, 288);
    }
}

// Portable items are available on all targets; fs-dependent functions
// (corpus load/generate, artifact save/load) are cfg-gated per item.
pub mod code_sidecar;
pub mod compiler;
#[cfg(not(target_arch = "wasm32"))]
pub mod convert_r4g1;
#[cfg(not(target_arch = "wasm32"))]
pub mod graph_patch;
pub mod hf_bpe;
pub mod reference_state;
pub mod resolution_status;
pub mod runtime;
pub mod scenarios;
pub mod score_q;
pub mod source_scan;
// Host/compile-side tokenizer source adapter: parses `spiece.model` and
// reports refusals on the `SourceUnavailable` surface, which is itself
// gated off wasm32 (the deployed wasm runtime uses the exported tokenizer,
// never the source parser). Gated to match, like `hf_bpe::adapter_constructor`.
#[cfg(not(target_arch = "wasm32"))]
pub mod sentencepiece;
pub mod simd;
pub mod transitions;

/// #243 dot-assignment tests live HERE rather than in runtime.rs: the
/// P-4 scan covers all of runtime.rs including its test code, and test
/// fixture arithmetic (K * D, %, /) must not appear in that file.
#[cfg(test)]
mod dot_assignment_tests {
    use super::{compiler, runtime};

    fn synthetic_dot_art() -> compiler::Compiled {
        let mut art = compiler::Compiled {
            token_codes: vec![0u8; compiler::STAGES],
            stage_books: (0..compiler::STAGES)
                .map(|_| vec![0i8; compiler::K * compiler::D])
                .collect(),
            stage_shifts: vec![0u8; compiler::STAGES],
            thresholds: (0..compiler::D as i64).map(|d| (d % 7) - 3).collect(),
            class_sigs: (0..compiler::STAGES)
                .map(|_| vec![0u8; compiler::K * compiler::D / 8])
                .collect(),
            ctx_cb: Vec::new(),
            token_stage_kappas: Vec::new(),
            dot_cb: Vec::new(),
            resid_cb: Vec::new(),
            resid_scale_shifts: Vec::new(),
            norm_fold_const: 0,
        };
        art.dot_cb = (0..compiler::STAGES)
            .map(|st| {
                (0..compiler::K * compiler::D)
                    .map(|i| {
                        let v = ((i + st) % 13) as f32 - 6.0;
                        compiler::pack_dot_entry(v / 8.0)
                    })
                    .collect()
            })
            .collect();
        art
    }

    /// The synthetic dot artifact plus the #318 Phase B residual
    /// sections: i8 centroid copies and decode shifts shaped like the
    /// compiler's `quantize_resid_copies` output, and a norm-fold CONST
    /// exercising the right-shift branch on the ramp bundle below.
    fn synthetic_resid_art() -> compiler::Compiled {
        let mut art = synthetic_dot_art();
        art.resid_cb = (0..compiler::STAGES)
            .map(|st| {
                (0..compiler::K * compiler::D)
                    .map(|i| (((i + st) % 13) as i8) - 6)
                    .collect()
            })
            .collect();
        art.resid_scale_shifts = vec![4u8; compiler::STAGES];
        art.norm_fold_const = 10;
        art
    }

    #[test]
    fn pack_dot_entry_round_trips_powers_of_two() {
        let one = compiler::pack_dot_entry(1.0).to_le_bytes();
        assert_eq!(one[1] & 0x40, 0x40, "nonzero flag");
        assert_eq!(one[1] & 0x80, 0, "positive");
        assert_eq!((one[1] & 0x3F) as i32 - 32, 0, "exponent 0");
        assert_eq!(one[0], 0, "no residual term");
        let v = compiler::pack_dot_entry(-0.375).to_le_bytes();
        assert_eq!((v[1] & 0x3F) as i32 - 32, -1);
        assert_eq!(v[1] & 0x80, 0x80, "first term negative");
        // Phase C decision (issue #243): DOT_TERMS = 1 — no residual
        // term is emitted. Two-term slots from Phase B era artifacts
        // still decode (dot_term_apply skips nothing it shouldn't).
        assert_eq!(compiler::DOT_TERMS, 1, "Phase C pins 1-term emission");
        assert_eq!(v[0], 0, "no residual term under 1-term emission");
        assert_eq!(compiler::pack_dot_entry(0.0), 0);
    }

    #[test]
    fn dot_kernel_equals_plain_on_synthetic_artifact() {
        let art = synthetic_dot_art();
        let mut bundle = [0i64; compiler::D];
        for (d, slot) in bundle.iter_mut().enumerate() {
            *slot = ((d as i64) % 97) - 48;
        }
        let plain = runtime::assign_for_bundle(&art, &bundle);
        let mut rt = runtime::Runtime::new(&art);
        let kernel = rt.code_from_bundle_dot(&bundle);
        assert_eq!(plain, kernel, "kernel/plain dot assignment divergence");
        assert!(rt.kernel.shifts > 0, "dot path must count shifts");
        assert!(rt.kernel.adds > 0, "dot path must count adds");
    }

    /// #318 Phase B equality witness: the residual-wired kernel form
    /// (`code_from_bundle_resid`) computes the plain form's
    /// (`assign_code_for_bundle_resid`) code exactly, across bundles
    /// exercising every norm-fold branch (s ≥ 0, s < 0, L1 = 0) and the
    /// per-stage copy subtraction.
    #[test]
    fn resid_kernel_equals_plain_on_synthetic_artifact() {
        let art = synthetic_resid_art();
        let bundles: Vec<[i64; compiler::D]> = vec![
            // ramp: large L1, fold shifts right (s > 0)
            std::array::from_fn(|d| ((d as i64 * 97) % 193) - 96),
            // tiny: L1 below the CONST scale, fold shifts left (s < 0)
            std::array::from_fn(|d| ((d as i64) % 3) - 1),
            // centered exactly: L1 = 0, fold leaves the zero vector
            std::array::from_fn(|d| (d as i64 % 7) - 3),
            // negative-heavy ramp
            std::array::from_fn(|d| -((d as i64 % 89) + 1)),
        ];
        let mut rt = runtime::Runtime::new(&art);
        for bundle in &bundles {
            let plain = runtime::assign_code_for_bundle_resid(&art, bundle);
            assert_eq!(
                plain,
                runtime::assign_code_for_bundle(&art, bundle),
                "assign_code_for_bundle routes TLA7 artifacts to the residual path"
            );
            let kernel = rt.code_from_bundle_resid(bundle);
            assert_eq!(
                plain, kernel,
                "kernel/plain residual divergence on {bundle:?}"
            );
        }
        assert!(rt.kernel.shifts > 0, "residual path must count shifts");
        assert!(rt.kernel.adds > 0, "residual path must count adds");
        assert!(rt.kernel.compares > 0, "residual path must count compares");
        assert!(
            rt.kernel.table_reads > 0,
            "residual path must count table reads"
        );
    }

    /// #318 Phase B routing witness: the corpus-free kernel entry point
    /// takes the residual path for TLA7 artifacts, identical to the
    /// plain window path.
    #[test]
    fn resid_assign_window_matches_plain_window_path() {
        let art = synthetic_resid_art();
        let rot = compiler::derive_rotations();
        let window = [0u32];
        let plain = {
            let b = runtime::bundle_window_plain(&art, &rot, &window);
            runtime::assign_code_for_bundle(&art, &b)
        };
        let mut rt = runtime::Runtime::new(&art);
        assert_eq!(rt.assign_window(&window), plain);
    }

    /// #318 Phase B routing consistency: EVERY bundle-holding entry
    /// point takes the residual path for TLA7 artifacts — plain beam
    /// form, plain allocation-free serving form, membership primary,
    /// and kernel form must all agree. Regression test for the first
    /// quality run's divergence: `assign_for_bundle` kept the
    /// non-residual dot path while the kernel took the residual path,
    /// and the resid-vs-resid synthetic witness could not see it.
    #[test]
    fn resid_routing_consistent_across_all_entry_points() {
        let art = synthetic_resid_art();
        let mut art6 = synthetic_resid_art();
        art6.resid_cb = Vec::new(); // non-residual reference (TLA6 shape)
        let mut rt = runtime::Runtime::new(&art);
        let mut saw_residual_effect = false;
        for seed in 0..8usize {
            let bundle: [i64; compiler::D] =
                std::array::from_fn(|d| (((d * 31 + seed * 17) % 211) as i64) - 105);
            let resid = runtime::assign_code_for_bundle_resid(&art, &bundle);
            assert_eq!(
                runtime::assign_for_bundle(&art, &bundle),
                resid,
                "beam form must route to the residual path"
            );
            assert_eq!(
                runtime::assign_code_for_bundle(&art, &bundle),
                resid,
                "serving form must route to the residual path"
            );
            assert_eq!(
                runtime::assign_memberships_for_bundle(&art, &bundle).0,
                resid,
                "membership primary must route to the residual path"
            );
            assert_eq!(
                rt.code_from_bundle_resid(&bundle),
                resid,
                "kernel form agrees with all plain forms"
            );
            if runtime::assign_for_bundle(&art6, &bundle) != resid {
                saw_residual_effect = true;
            }
        }
        assert!(
            saw_residual_effect,
            "the synthetic fixture must make the residual update change at least one code, \
             or this consistency check passes vacuously"
        );
    }

    /// #318 Phase B container eras: TLA7 round-trips the residual
    /// sections byte-identically; a dot-only artifact still emits TLA6
    /// with no residual sections (pre-TLA7 loads unchanged).
    #[test]
    fn tla7_container_roundtrip() {
        let art = synthetic_resid_art();
        let bytes = compiler::artifact_bytes(&art);
        assert!(bytes.starts_with(b"TLA7"), "residual artifact emits TLA7");
        let parsed = compiler::parse_artifacts(&bytes).expect("TLA7 parses");
        assert_eq!(parsed.resid_cb, art.resid_cb, "centroid copies round-trip");
        assert_eq!(
            parsed.resid_scale_shifts, art.resid_scale_shifts,
            "decode shifts round-trip"
        );
        assert_eq!(
            parsed.norm_fold_const, art.norm_fold_const,
            "norm-fold CONST round-trips"
        );
        assert_eq!(parsed.dot_cb, art.dot_cb, "dot tables round-trip");
        assert_eq!(
            compiler::artifact_bytes(&parsed),
            bytes,
            "TLA7 parse → serialize is byte-identical"
        );
        assert!(
            compiler::parse_artifacts(&bytes[..bytes.len() - 1]).is_none(),
            "truncated TLA7 container rejected"
        );

        let dot_only = synthetic_dot_art();
        let tla6 = compiler::artifact_bytes(&dot_only);
        assert!(tla6.starts_with(b"TLA6"), "dot-only artifact emits TLA6");
        let parsed6 = compiler::parse_artifacts(&tla6).expect("TLA6 parses");
        assert!(parsed6.resid_cb.is_empty(), "TLA6 loads carry no residual");
        assert_eq!(parsed6.norm_fold_const, 0);
        assert_eq!(
            compiler::artifact_bytes(&parsed6),
            tla6,
            "TLA6 parse → serialize is byte-identical"
        );
    }
}
