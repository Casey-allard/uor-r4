//! Behavioral contract for the catamorphism evaluator (wiki ADR-029).
//!
//! Per ADR-029, `pipeline::run` evaluates the route's Term tree as a
//! structural fold with per-variant fold-rules. Foundation exposes
//! `evaluate_term_tree` as the catamorphism's evaluation entry point;
//! `pipeline::run_route` calls it after validating the `CompileUnit`,
//! and the resulting bytes flow into the `Grounded`'s output payload
//! (per ADR-028).
//!
//! This test pins:
//!
//! 1. `evaluate_term_tree` is reachable through `pipeline::*`.
//! 2. The empty arena (identity route) returns the input bytes.
//! 3. `Term::Literal` evaluates to its literal value's big-endian bytes.
//! 4. `Term::Application` over `PrimitiveOp::Add` adds the args' values.
//! 5. `Term::AxisInvocation` folds the input through the selected Hasher
//!    (axis 0, kernel 0 = the canonical hash projection).
//! 6. ADR-060: the source-polymorphic `TermValue<'a, N>` carrier replaced the
//!    fixed 4096-byte ceiling; its inline width is `carrier_inline_bytes::<B>()`.

use uor_foundation::enforcement::{Hasher, Term, TermList};
use uor_foundation::pipeline::{evaluate_term_tree, NullResolverTuple, TermValue};
use uor_foundation::{PipelineFailure, PrimitiveOp, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

/// Thin wrapper around `evaluate_term_tree` that defaults the resolver
/// tuple to `NullResolverTuple` — keeps these test bodies focused on the
/// catamorphism's term-tree fold-rules (the resolver-bound ψ-Term variants
/// are exercised by dedicated tests that supply real resolvers).
fn eval_zero<'a>(
    arena: &'a [Term<'a, N>],
    input_bytes: &'a [u8],
) -> Result<TermValue<'a, N>, PipelineFailure> {
    evaluate_term_tree::<ZeroHasher, NullResolverTuple, N, 32>(
        arena,
        TermValue::borrowed(input_bytes),
        &NullResolverTuple,
    )
}

#[derive(Debug, Clone, Copy, Default)]
struct ZeroHasher;

impl Hasher for ZeroHasher {
    const OUTPUT_BYTES: usize = 4;
    fn initial() -> Self {
        Self
    }
    fn fold_byte(self, _: u8) -> Self {
        self
    }
    fn finalize(self) -> [u8; 32] {
        // Deterministic, distinguishable digest so the assertion below can
        // distinguish the hasher's output from the input bytes.
        let mut out = [0u8; 32];
        out[0] = 0xab;
        out[1] = 0xcd;
        out[2] = 0xef;
        out[3] = 0x01;
        out
    }
}

#[test]
fn evaluator_surface_resolves_at_crate_root() {
    // The function exists at the foundation public path and evaluates the
    // identity (empty) route to the threaded input bytes.
    let out = evaluate_term_tree::<ZeroHasher, NullResolverTuple, N, 32>(
        &[],
        TermValue::borrowed(&[0xaa]),
        &NullResolverTuple,
    )
    .expect("identity route resolves at crate root");
    assert_eq!(out.bytes(), &[0xaa][..]);
    // ADR-060 removed the fixed `TERM_VALUE_MAX_BYTES` / `ROUTE_*_BUFFER_BYTES`
    // 4096-byte ceiling in favour of the source-polymorphic `TermValue<'a, N>`
    // carrier whose inline width is the application's `carrier_inline_bytes`.
    // The empty inline carrier and a foundation-derived inline width pin that
    // the source-polymorphic carrier is reachable at the crate root.
    let empty = TermValue::<N>::empty();
    assert_eq!(empty.bytes().len(), 0);
    const {
        assert!(
            N > 0,
            "carrier_inline_bytes must be a positive inline width"
        )
    };
}

#[test]
fn empty_arena_evaluates_to_input_bytes() {
    // ADR-029 / wiki ADR-022 D5 corner case: the foundation-sanctioned
    // identity route has an empty term arena. The catamorphism must
    // pass the input through to the output unchanged.
    let input = [0xde, 0xad, 0xbe, 0xef];
    let result = eval_zero(&[], &input).expect("identity route succeeds");
    assert_eq!(result.bytes(), &input[..]);
}

#[test]
fn literal_term_evaluates_to_value_bytes() {
    // uor_foundation::pipeline::literal_u64(0x42, W8) → single-byte 0x42.
    let arena = [uor_foundation::pipeline::literal_u64(0x42, WittLevel::W8)];
    let result = eval_zero(&arena, &[]).expect("literal evaluates");
    assert_eq!(result.bytes(), &[0x42][..]);
}

#[test]
fn application_add_combines_args() {
    // [Literal(2), Literal(3), Application(Add, [0..2])] → 5 (1 byte).
    let arena = [
        uor_foundation::pipeline::literal_u64(2, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(3, WittLevel::W8),
        Term::Application {
            operator: PrimitiveOp::Add,
            args: TermList { start: 0, len: 2 },
        },
    ];
    let result = eval_zero(&arena, &[]).expect("addition evaluates");
    assert_eq!(result.bytes(), &[5u8][..]);
}

#[test]
fn hasher_projection_delegates_to_substitution_axis() {
    // Term::AxisInvocation { axis_index: 0, kernel_id: 0, input_index: 0 }
    // applied to a Variable input — the catamorphism reaches the Hasher
    // axis and emits the digest. ZeroHasher returns a fixed pattern so
    // the assertion is distinguishable from input bytes.
    let arena = [
        Term::Variable { name_index: 0 },
        Term::AxisInvocation {
            axis_index: 0,
            kernel_id: 0,
            input_index: 0,
        },
    ];
    let input = [0x11, 0x22, 0x33];
    let result = eval_zero(&arena, &input).expect("hash projection evaluates");
    // Per ADR-029, the hasher's OUTPUT_BYTES width prefix is taken.
    assert_eq!(result.bytes(), &[0xab, 0xcd, 0xef, 0x01][..]);
}

#[test]
fn term_value_carries_active_prefix_only() {
    // `TermValue::inline_from_slice` copies up to `INLINE_BYTES` bytes
    // and reports the active prefix length via `bytes()`.
    let v = TermValue::<N>::inline_from_slice(&[1, 2, 3, 4, 5]);
    assert_eq!(v.bytes(), &[1, 2, 3, 4, 5][..]);
    assert_eq!(v.bytes().len(), 5);
    let empty = TermValue::<N>::empty();
    assert_eq!(empty.bytes().len(), 0);
}

// ── Per-variant fold-rule coverage (ADR-029) ────────────────────────────

#[test]
fn variable_term_routes_input_bytes() {
    // ADR-022 D3 G2: name_index = 0 is the route input slot. The
    // catamorphism's Variable handler returns the threaded input bytes.
    let arena = [Term::Variable { name_index: 0 }];
    let input = [0xca, 0xfe];
    let result = eval_zero(&arena, &input).expect("variable evaluates to input");
    assert_eq!(result.bytes(), &input[..]);
}

#[test]
fn lift_term_zero_extends_to_target_width() {
    // Term::Lift big-endian zero-extends a narrower value to the target
    // Witt level's byte width.
    let arena = [
        uor_foundation::pipeline::literal_u64(0x42, WittLevel::W8),
        Term::Lift {
            operand_index: 0,
            target: WittLevel::W32,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("lift evaluates");
    // W32 is 4 bytes; the W8 value 0x42 zero-extends to [0x00, 0x00, 0x00, 0x42].
    assert_eq!(result.bytes(), &[0x00, 0x00, 0x00, 0x42][..]);
}

#[test]
fn project_term_truncates_to_target_width() {
    // Term::Project takes the trailing `target_width` bytes of the operand.
    let arena = [
        uor_foundation::pipeline::literal_u64(0xdeadbeef, WittLevel::W32),
        Term::Project {
            operand_index: 0,
            target: WittLevel::W8,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("project evaluates");
    // 0xdeadbeef projected to W8 (1 byte) keeps the trailing byte 0xef.
    assert_eq!(result.bytes(), &[0xef][..]);
}

#[test]
fn match_term_dispatches_on_literal_pattern() {
    // arena layout (indexes shown):
    //   0: Literal(7, W8)             scrutinee
    //   1: Literal(7, W8)             pattern arm 1
    //   2: Literal(0xaa, W8)          body arm 1 (matches)
    //   3: Literal(9, W8)             pattern arm 2
    //   4: Literal(0xbb, W8)          body arm 2 (does not match)
    //   5: Match { scrutinee: 0, arms: [1..5] }
    let arena = [
        uor_foundation::pipeline::literal_u64(7, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(7, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(0xaa, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(9, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(0xbb, WittLevel::W8),
        Term::Match {
            scrutinee_index: 0,
            arms: TermList { start: 1, len: 4 },
        },
    ];
    let result = eval_zero(&arena, &[]).expect("match evaluates");
    assert_eq!(result.bytes(), &[0xaa][..]);
}

#[test]
fn match_term_falls_through_to_wildcard_arm() {
    // Wildcard pattern is `Variable { name_index: u32::MAX }`. When the
    // scrutinee matches no literal arm, the wildcard's body is taken.
    let arena = [
        uor_foundation::pipeline::literal_u64(99, WittLevel::W8),
        Term::Variable {
            name_index: u32::MAX,
        },
        uor_foundation::pipeline::literal_u64(0xfa, WittLevel::W8),
        Term::Match {
            scrutinee_index: 0,
            arms: TermList { start: 1, len: 2 },
        },
    ];
    let result = eval_zero(&arena, &[]).expect("wildcard match evaluates");
    assert_eq!(result.bytes(), &[0xfa][..]);
}

#[test]
fn recurse_term_iterates_step_n_times() {
    // ADR-029 recursive fold: `recurse(measure=3, base=10, |_self, _x| add(x, 1))`
    // should compute 10 + 1 + 1 + 1 = 13 by iterating step 3 times with
    // the recursive-call placeholder bound to the previous iteration's
    // result.
    use uor_foundation::pipeline::RECURSE_PLACEHOLDER_NAME_INDEX;
    let arena = [
        // 0: measure literal — 3 (one byte)
        uor_foundation::pipeline::literal_u64(3, WittLevel::W8),
        // 1: base literal — 10 (one byte)
        uor_foundation::pipeline::literal_u64(10, WittLevel::W8),
        // 2: step body — Add(placeholder, 1)
        Term::Variable {
            name_index: RECURSE_PLACEHOLDER_NAME_INDEX,
        },
        uor_foundation::pipeline::literal_u64(1, WittLevel::W8),
        Term::Application {
            operator: PrimitiveOp::Add,
            args: TermList { start: 2, len: 2 },
        },
        // 5: Recurse { measure: 0, base: 1, step: 4 }
        Term::Recurse {
            measure_index: 0,
            base_index: 1,
            step_index: 4,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("recurse evaluates");
    assert_eq!(result.bytes(), &[13u8][..]);
}

#[test]
fn recurse_zero_measure_returns_base() {
    // measure = 0 → base case.
    let arena = [
        uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(0xbe, WittLevel::W8),
        // Step: identity (will not run).
        Term::Variable { name_index: 0 },
        Term::Recurse {
            measure_index: 0,
            base_index: 1,
            step_index: 2,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("recurse base evaluates");
    assert_eq!(result.bytes(), &[0xbe][..]);
}

#[test]
fn unfold_term_iterates_to_kleene_fixpoint() {
    // ADR-029 anamorphism: unfold(seed=0, |s| or(s, 0xff)) reaches the
    // fixpoint at 0xff after one step (or(0xff, 0xff) == 0xff).
    use uor_foundation::pipeline::UNFOLD_PLACEHOLDER_NAME_INDEX;
    let arena = [
        // 0: seed literal — 0
        uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
        // 1: state placeholder
        Term::Variable {
            name_index: UNFOLD_PLACEHOLDER_NAME_INDEX,
        },
        // 2: 0xff literal
        uor_foundation::pipeline::literal_u64(0xff, WittLevel::W8),
        // 3: Or(state, 0xff)
        Term::Application {
            operator: PrimitiveOp::Or,
            args: TermList { start: 1, len: 2 },
        },
        // 4: Unfold { seed: 0, step: 3 }
        Term::Unfold {
            seed_index: 0,
            step_index: 3,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("unfold evaluates");
    assert_eq!(result.bytes(), &[0xff][..]);
}

#[test]
fn try_term_propagates_success() {
    // Body succeeds → handler is not invoked.
    let arena = [
        uor_foundation::pipeline::literal_u64(0x77, WittLevel::W8),
        Term::Try {
            body_index: 0,
            handler_index: u32::MAX,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("try success evaluates");
    assert_eq!(result.bytes(), &[0x77][..]);
}

#[test]
fn recurse_two_param_form_binds_iteration_counter() {
    // ADR-034 Mechanism 1: the two-parameter `recurse(measure, base,
    // |self, idx| step)` form binds the iteration counter to
    // RECURSE_IDX_NAME_INDEX. At iter=0 the descent measure is N,
    // decreasing each step. With measure=3, base=0, step = idx (just
    // return the iteration counter), the final value is 1 (the descent
    // measure at the LAST iteration before zero).
    use uor_foundation::pipeline::RECURSE_IDX_NAME_INDEX;
    let arena = [
        // 0: measure literal — 3
        uor_foundation::pipeline::literal_u64(3, WittLevel::W8),
        // 1: base literal — 0
        uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
        // 2: step body — Variable referencing the iteration counter
        Term::Variable {
            name_index: RECURSE_IDX_NAME_INDEX,
        },
        // 3: Recurse { measure: 0, base: 1, step: 2 }
        Term::Recurse {
            measure_index: 0,
            base_index: 1,
            step_index: 2,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("recurse with idx binding evaluates");
    // The descent at the final iteration (iter=2 → measure=N-iter=1) is
    // the last value the step body sees; that's what becomes the result.
    // The 8-byte BE encoding of 1u64 ends in 0x01.
    assert_eq!(result.bytes().last(), Some(&1));
}

#[test]
fn first_admit_returns_found_coproduct_on_admission() {
    // ADR-034 Mechanism 2: first_admit iterates idx in 0..N and returns
    // the first idx for which predicate evaluates non-zero. Result
    // shape: (0x01, idx_bytes). With domain = 5 (1 idx byte, fits W8),
    // predicate = `idx`, the first non-zero idx is 1 → result =
    // [0x01, 0x01].
    use uor_foundation::pipeline::FIRST_ADMIT_IDX_NAME_INDEX;
    let arena = [
        // 0: domain size = 5
        uor_foundation::pipeline::literal_u64(5, WittLevel::W8),
        // 1: predicate body = Variable referencing the candidate idx
        Term::Variable {
            name_index: FIRST_ADMIT_IDX_NAME_INDEX,
        },
        // 2: FirstAdmit { domain_size_index: 0, predicate_index: 1 }
        Term::FirstAdmit {
            domain_size_index: 0,
            predicate_index: 1,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("first_admit evaluates");
    // 5 fits in 1 byte → idx_byte_width = 1 → result is 2 bytes:
    //   discriminator 0x01 ("found") + idx 1.
    assert_eq!(result.bytes(), &[0x01, 0x01][..]);
}

#[test]
fn first_admit_returns_not_found_coproduct_on_exhausted_search() {
    // ADR-034: when no idx admits across the full domain, emit the
    // not-found coproduct value (0x00, idx-width zero padding).
    let arena = [
        // 0: domain size = 4
        uor_foundation::pipeline::literal_u64(4, WittLevel::W8),
        // 1: predicate body = Literal(0) — always rejects.
        uor_foundation::pipeline::literal_u64(0, WittLevel::W8),
        // 2: FirstAdmit
        Term::FirstAdmit {
            domain_size_index: 0,
            predicate_index: 1,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("first_admit (no admission) evaluates");
    assert_eq!(result.bytes(), &[0x00, 0x00][..]);
}

#[test]
fn axis_invocation_canonical_hash_routes_through_axis_tuple() {
    // ADR-030: Term::AxisInvocation { axis_index: 0, kernel_id: 0, ... }
    // dispatches via the application's AxisTuple. Foundation's blanket
    // `impl<H: Hasher> AxisTuple for H` routes the canonical (0, 0)
    // dispatch through the legacy Hasher API. This test pins that the
    // catamorphism's AxisInvocation arm correctly forwards to AxisTuple's
    // dispatch surface (per the wiki's evaluate_term_tree<A: AxisTuple>).
    let arena = [
        Term::Variable { name_index: 0 },
        Term::AxisInvocation {
            axis_index: 0,
            kernel_id: 0,
            input_index: 0,
        },
    ];
    let input = [0xaa, 0xbb];
    let result = eval_zero(&arena, &input).expect("axis invocation evaluates");
    // ZeroHasher's AxisTuple dispatch returns its OUTPUT_BYTES (4) bytes
    // of the canonical pattern.
    assert_eq!(result.bytes(), &[0xab, 0xcd, 0xef, 0x01][..]);
}

#[test]
fn axis_invocation_non_canonical_dispatch_rejects() {
    // Non-canonical (axis_index, kernel_id) combinations against the
    // foundation-built blanket AxisTuple-for-Hasher impl produce a
    // ShapeViolation. User-declared AxisTuple impls (via the `axis!`
    // SDK macro and tuple composition) extend the dispatch surface.
    let arena = [
        Term::Variable { name_index: 0 },
        Term::AxisInvocation {
            axis_index: 1, // out of range for the blanket 1-axis dispatcher
            kernel_id: 0,
            input_index: 0,
        },
    ];
    let result = eval_zero(&arena, &[1, 2, 3]);
    assert!(
        result.is_err(),
        "non-canonical axis dispatch must produce a ShapeViolation"
    );
}

#[test]
fn project_field_term_slices_source_bytes() {
    // ADR-033 G20: Term::ProjectField { source_index, byte_offset,
    // byte_length } slices the source's evaluated bytes per the offset
    // and length the proc-macro computes from PartitionProductFields.
    let arena = [
        // 0: Source — a 4-byte literal (treated as a partition_product
        // of two 2-byte halves at byte offsets 0 and 2).
        uor_foundation::pipeline::literal_u64(0xdeadbeef, WittLevel::W32),
        // 1: Project field 1 → bytes [2..4] = [0xbe, 0xef].
        Term::ProjectField {
            source_index: 0,
            byte_offset: 2,
            byte_length: 2,
        },
    ];
    let result = eval_zero(&arena, &[]).expect("project_field evaluates");
    assert_eq!(result.bytes(), &[0xbe, 0xef][..]);
}

#[test]
fn project_field_out_of_bounds_rejects() {
    // ADR-033: byte_offset + byte_length > source length must produce a
    // ShapeViolation rather than panic. The proc-macro generates valid
    // offsets/lengths from PartitionProductFields, but hand-built arenas
    // and replay-from-trace paths may exercise this guard.
    let arena = [
        uor_foundation::pipeline::literal_u64(0x42, WittLevel::W8),
        Term::ProjectField {
            source_index: 0,
            byte_offset: 0,
            byte_length: 16, // way beyond the 1-byte source
        },
    ];
    let result = eval_zero(&arena, &[]);
    assert!(
        result.is_err(),
        "ProjectField overflowing source must produce ShapeViolation"
    );
}

// ── PrimitiveOp coverage (ADR-013/TR-08 substrate amendment) ─────────────

fn binary_op_arena(op: PrimitiveOp, lhs: u64, rhs: u64) -> [Term<'static, N>; 3] {
    [
        uor_foundation::pipeline::literal_u64(lhs, WittLevel::W8),
        uor_foundation::pipeline::literal_u64(rhs, WittLevel::W8),
        Term::Application {
            operator: op,
            args: TermList { start: 0, len: 2 },
        },
    ]
}

#[test]
fn comparison_primitives_emit_zero_or_one() {
    // ADR-013/TR-08: Le, Lt, Ge, Gt fold to a single 0/1-valued byte.
    let cases = [
        (PrimitiveOp::Le, 5u64, 7u64, 1u8),
        (PrimitiveOp::Le, 7, 7, 1),
        (PrimitiveOp::Le, 9, 7, 0),
        (PrimitiveOp::Lt, 5, 7, 1),
        (PrimitiveOp::Lt, 7, 7, 0),
        (PrimitiveOp::Ge, 9, 7, 1),
        (PrimitiveOp::Ge, 7, 7, 1),
        (PrimitiveOp::Ge, 5, 7, 0),
        (PrimitiveOp::Gt, 9, 7, 1),
        (PrimitiveOp::Gt, 7, 7, 0),
    ];
    for (op, lhs, rhs, expected) in cases {
        let arena = binary_op_arena(op, lhs, rhs);
        let result = eval_zero(&arena, &[]).expect("comparison op evaluates");
        assert_eq!(
            result.bytes(),
            &[expected][..],
            "{op:?}({lhs}, {rhs}) expected {expected}"
        );
    }
}

#[test]
fn concat_primitive_packs_byte_sequences() {
    // ADR-013/TR-08: Concat appends rhs's bytes to lhs's.
    let arena = [
        uor_foundation::pipeline::literal_u64(0xabcd, WittLevel::W16),
        uor_foundation::pipeline::literal_u64(0x1234, WittLevel::W16),
        Term::Application {
            operator: PrimitiveOp::Concat,
            args: TermList { start: 0, len: 2 },
        },
    ];
    let result = eval_zero(&arena, &[]).expect("concat evaluates");
    assert_eq!(result.bytes(), &[0xab, 0xcd, 0x12, 0x34][..]);
}

#[test]
fn arithmetic_primitives_match_ring_semantics() {
    // Sub, Mul, Xor, And, Or — paired with their ring-eval reductions.
    let cases = [
        (PrimitiveOp::Sub, 10u64, 3u64, 7u8),
        (PrimitiveOp::Mul, 6, 7, 42),
        (PrimitiveOp::Xor, 0b1010, 0b1100, 0b0110),
        (PrimitiveOp::And, 0b1010, 0b1100, 0b1000),
        (PrimitiveOp::Or, 0b1010, 0b1100, 0b1110),
    ];
    for (op, lhs, rhs, expected) in cases {
        let arena = binary_op_arena(op, lhs, rhs);
        let result = eval_zero(&arena, &[]).expect("binary op evaluates");
        assert_eq!(result.bytes(), &[expected][..], "{op:?}({lhs}, {rhs})");
    }
}

#[test]
fn unary_primitives_match_ring_semantics() {
    // Neg, Bnot, Succ, Pred (1-arg forms).
    let cases = [
        (PrimitiveOp::Neg, 1u64, 255u8), // 0 - 1 in u8
        (PrimitiveOp::Bnot, 0u64, 0xff),
        (PrimitiveOp::Succ, 41, 42),
        (PrimitiveOp::Pred, 1, 0),
    ];
    for (op, operand, expected) in cases {
        let arena = [
            uor_foundation::pipeline::literal_u64(operand, WittLevel::W8),
            Term::Application {
                operator: op,
                args: TermList { start: 0, len: 1 },
            },
        ];
        let result = eval_zero(&arena, &[]).expect("unary op evaluates");
        assert_eq!(result.bytes(), &[expected][..], "{op:?}({operand})");
    }
}

// ── ADR-036: resolver-bound ψ-Term fold-rules consult the application's
// ResolverTuple at evaluation time ──────────────────────────────────────

#[test]
fn resolver_bound_psi_term_consults_resolver_tuple() {
    // Wiki ADR-036: the eight resolver-bound ψ-Term fold-rules
    // (Nerve, ChainComplex, HomologyGroups, CochainComplex,
    // CohomologyGroups, PostnikovTower, HomotopyGroups, KInvariants)
    // dispatch the operand bytes to the application's resolver. With
    // the NullResolverTuple default, `resolve` returns the
    // RESOLVER_ABSENT shape violation, which the catamorphism
    // propagates as `PipelineFailure::ShapeViolation`. This test pins
    // that the resolver IS consulted (rather than the variant emitting
    // a fixed pass-through), by asserting the propagated violation's
    // `shape_iri` carries the `RESOLVER_ABSENT` discriminator.
    let arena = [
        Term::Variable { name_index: 0 },
        Term::Nerve { value_index: 0 },
    ];
    let input = [0x11u8, 0x22, 0x33];
    let result = evaluate_term_tree::<ZeroHasher, NullResolverTuple, N, 32>(
        &arena,
        TermValue::borrowed(&input),
        &NullResolverTuple,
    );
    match result {
        Err(PipelineFailure::ShapeViolation { report }) => {
            assert_eq!(
                report.shape_iri, "https://uor.foundation/resolver/RESOLVER_ABSENT",
                "Null resolver's `resolve` must emit RESOLVER_ABSENT",
            );
        }
        other => panic!(
            "Term::Nerve under NullResolverTuple must propagate RESOLVER_ABSENT, got {other:?}"
        ),
    }
}

#[test]
fn betti_term_is_resolver_free_passthrough() {
    // Wiki ADR-035 ψ_4: Betti-extraction is pure byte projection — the
    // catamorphism does NOT consult any resolver. With
    // NullResolverTuple, evaluating Term::Betti over a literal homology
    // payload must succeed and return the bytes unchanged.
    let arena = [
        uor_foundation::pipeline::literal_u64(0x42, WittLevel::W8),
        Term::Betti { homology_index: 0 },
    ];
    let result = eval_zero(&arena, &[]).expect("betti is resolver-free");
    assert_eq!(result.bytes(), &[0x42][..]);
}
