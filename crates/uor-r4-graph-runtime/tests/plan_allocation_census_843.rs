//! Allocation census and P-4 operation scan for the deployed bounded planner
//! (#843 §6).
//!
//! A counting global allocator measures what a planning episode actually
//! allocates. One `#[test]` for the census by design: the gate and counters are
//! thread-local and libtest runs tests in parallel, so a second measured test
//! could let one episode's bookkeeping land in another's census. The fixture is
//! built with the gate closed; only the episode itself is measured.
//!
//! Run with:
//! `cargo test -p uor-r4-graph-runtime --test plan_allocation_census_843 -- --nocapture`

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_graph_format::plan::{CompareOp, EffectDelta, PreconditionMask, SlotVec};
use uor_r4_graph_format::plan_sections::{
    PackedRule, PlanSchema, PredicateSet, RuleTable, WitnessDraft, WitnessStep,
    build_predicate_set, build_rule_table, build_schema, encode_witness_into,
};
use uor_r4_graph_runtime::plan::{
    PlanBudget, PlanOutcome, PlanQuery, PlanScratch, PlanStrategy, plan,
};

struct CountingAlloc;

thread_local! {
    static GATE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let _ = GATE.try_with(|gate| {
                if gate.get() {
                    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
                    let _ = BYTES.try_with(|n| n.set(n.get() + layout.size()));
                }
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

/// Reset the counters, open the gate, run `body`, close the gate, and return
/// `(allocations, bytes)`. Reporting happens with the gate closed.
fn measure<T>(body: impl FnOnce() -> T) -> (usize, usize, T) {
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    GATE.with(|g| g.set(true));
    let value = body();
    GATE.with(|g| g.set(false));
    (
        ALLOCATIONS.with(|n| n.get()),
        BYTES.with(|n| n.get()),
        value,
    )
}

fn effect(x: i16, y: i16) -> EffectDelta {
    EffectDelta::from_slice(&[x, y]).unwrap()
}

fn cell_predicate(x: i16, y: i16) -> PreconditionMask {
    PreconditionMask::unconditional()
        .reading(0, CompareOp::Equal, x)
        .unwrap()
        .reading(1, CompareOp::Equal, y)
        .unwrap()
}

#[test]
fn a_planning_episode_and_its_witness_are_allocation_free() {
    // Fixture: built offline, with the gate closed.
    let vocabulary = vec![effect(1, 0), effect(0, 1), effect(-1, 0), effect(0, -1)];
    let schema_bytes = build_schema(2, &vocabulary, (1, 4, 16)).unwrap();
    let schema = PlanSchema::parse(&schema_bytes).unwrap();
    let rules: Vec<PackedRule> = (0..schema.operator_count())
        .map(|index| PackedRule {
            operator: index as u16,
            precondition: PreconditionMask::unconditional(),
            effect: schema.operator(index).unwrap(),
            support: 8,
            band: 2,
        })
        .collect();
    let rule_bytes = build_rule_table(2, schema.operator_count() as u16, &rules).unwrap();
    let rule_table = RuleTable::parse(&rule_bytes, &schema).unwrap();
    let predicate_bytes =
        build_predicate_set(2, &[cell_predicate(3, 0)], &[cell_predicate(2, 0)]).unwrap();
    let predicates = PredicateSet::parse(&predicate_bytes, &schema).unwrap();
    let mut scratch = Box::new(PlanScratch::new());
    let mut witness = vec![0u8; 4096];
    let constraints = [cell_predicate(2, 0)];
    let goal = cell_predicate(3, 0);
    let initial = SlotVec::from_slice(&[0, 0]).unwrap();
    let query = PlanQuery {
        strategy: PlanStrategy::BreadthFirst,
        schema: &schema,
        rules: &rule_table,
        predicates: &predicates,
        initial,
        available: 0b1111,
        budget: PlanBudget {
            horizon: 8,
            ..PlanBudget::frozen()
        },
    };
    // Warm the paths once outside the census.
    let _ = plan(&query, &mut scratch);

    // Measured: one whole episode, on an already-allocated scratch.
    let (episode_allocations, episode_bytes, outcome) =
        measure(|| plan(&query, &mut scratch).outcome);
    assert!(
        matches!(outcome, PlanOutcome::Plan { .. }),
        "the measured episode must actually plan, or the census is vacuous"
    );

    // Measured: emitting the witness into caller-owned bytes.
    let mut steps: [WitnessStep; 16] = [(EffectDelta::EMPTY, SlotVec::empty(), 0, 0); 16];
    let step_count = scratch.path_len();
    for (index, slot) in steps.iter_mut().enumerate().take(step_count) {
        *slot = scratch.path_step(index).unwrap();
    }
    let considered_per_step = scratch.considered_per_step() as u8;
    let considered = scratch.considered();
    let (witness_allocations, witness_bytes, written) = measure(|| {
        encode_witness_into(
            &WitnessDraft {
                slot_count: 2,
                initial,
                goal,
                constraints: &constraints,
                steps: &steps[..step_count],
                considered,
                considered_per_step,
                decline: None,
                verdict: (0, 0),
            },
            &mut witness,
        )
    });

    println!(
        "planning episode: {episode_allocations} allocations, {episode_bytes} bytes\n\
         witness emit:     {witness_allocations} allocations, {witness_bytes} bytes\n\
         witness size:     {written:?} bytes"
    );
    assert!(written.is_some(), "the witness must encode");
    assert_eq!(
        episode_allocations, 0,
        "a planning episode must be allocation-free in steady state"
    );
    assert_eq!(
        witness_allocations, 0,
        "emitting a witness into caller-owned bytes must not allocate"
    );
}

/// A machine-checked source scan of the deployed planner: the P-4 permitted
/// classes only. No multiply, no divide, no float — the guarantee §6 states,
/// asserted against the source rather than described in a comment.
#[test]
fn the_deployed_planner_uses_only_p4_operations() {
    let source = include_str!("../src/plan.rs");
    let mut offenders = Vec::new();
    let mut const_folded: Vec<String> = Vec::new();
    for (number, line) in source.lines().enumerate() {
        let code = match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        };
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        for forbidden in ["f32", "f64", " as f", "sqrt", "powi", "powf"] {
            if trimmed.contains(forbidden) {
                offenders.push(format!("{}: float `{forbidden}` — {trimmed}", number + 1));
            }
        }
        // A bare `*` or `/` that is not a dereference, a reference, a
        // generic bound, or a path separator.
        let bytes = trimmed.as_bytes();
        for (at, byte) in bytes.iter().enumerate() {
            if *byte != b'*' && *byte != b'/' {
                continue;
            }
            let previous = at.checked_sub(1).map(|i| bytes[i]);
            let next = bytes.get(at + 1).copied();
            // `*self`, `&mut *`, `*=`, `//` and `/*` are not arithmetic; an
            // operator with whitespace on both sides is.
            if previous != Some(b' ') || next != Some(b' ') {
                continue;
            }
            let name = if *byte == b'*' { "multiply" } else { "divide" };
            // A compile-time constant expression emits no instruction, so it is
            // not a deployed-kernel operation. Exempt it only when BOTH
            // operands are constants - a SCREAMING_SNAKE identifier or an
            // integer literal - and record the exemption so it is visible
            // rather than silent.
            if is_constant(operand_left(trimmed, at)) && is_constant(operand_right(trimmed, at)) {
                const_folded.push(format!("{}: {name} — {trimmed}", number + 1));
                continue;
            }
            offenders.push(format!("{}: {name} — {trimmed}", number + 1));
        }
    }
    println!(
        "P-4 scan: {} compile-time constant expressions exempted (they emit no instruction):\n{}",
        const_folded.len(),
        const_folded.join("\n")
    );
    assert!(
        offenders.is_empty(),
        "the deployed planner must execute only P-4 operations:\n{}",
        offenders.join("\n")
    );
    // The scan must be able to fire, or it is not evidence: a runtime multiply
    // between two non-constant operands is detected.
    assert!(
        source.contains("saturating_add"),
        "the scan is reading the planner source"
    );
    assert!(
        !const_folded.is_empty(),
        "the constant-expression exemption must be exercised, or it hides nothing"
    );
    let planted = "    let poisoned = width * depth;";
    let mut caught = false;
    for (at, byte) in planted.as_bytes().iter().enumerate() {
        if *byte == b'*'
            && !(is_constant(operand_left(planted.trim(), at - 4))
                && is_constant(operand_right(planted.trim(), at - 4)))
        {
            caught = true;
        }
    }
    assert!(caught, "a runtime multiply must still be caught");
}

/// The identifier or literal immediately left of `at`, ignoring whitespace.
fn operand_left(line: &str, at: usize) -> &str {
    let bytes = line.as_bytes();
    let mut end = at;
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    &line[start..end]
}

/// The identifier or literal immediately right of `at`, ignoring whitespace.
fn operand_right(line: &str, at: usize) -> &str {
    let bytes = line.as_bytes();
    let mut start = at + 1;
    while start < bytes.len() && bytes[start] == b' ' {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    &line[start..end]
}

/// Whether an operand is a compile-time constant: a SCREAMING_SNAKE identifier
/// or an integer literal.
fn is_constant(operand: &str) -> bool {
    !operand.is_empty()
        && operand
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}
