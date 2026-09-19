//! Direct allocation census for successful native session observe/predict.
//! Fitting, model construction, tokenization, session allocation, and reports
//! are deliberately outside the measured section. TLS excludes libtest noise.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_core::native_geometric::{
    Config, Control, Document, MemoryReadFitConfig, MemoryReadSchedule, MemoryReadSupervision,
    MemoryReadTokenSpan, MemoryReadTrainer, ReadoutFitConfig, ResponseEntryFitConfig, Trainer,
    ValueCompletionFitConfig, ValueExample, ValueFitConfig, WordCopyAction, BOS, EOS,
};

mod copy_fixture {
    use uor_r4_core::native_geometric as native;
    include!("support/native_word_copy_fixture.rs");
}

struct CountingAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            let _ = MEASURING.try_with(|enabled| {
                if enabled.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                    let _ = BYTES.try_with(|count| count.set(count.get() + layout.size()));
                }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// The experimental core uses its own discrete byte path, with no parent
/// predictor. Construction/checkpoint/report allocation is outside this census.
#[test]
fn shared_core_observe_predict_has_zero_allocations() {
    use uor_r4_core::native_geometric::shared_core::{Intervention, SharedCore};
    let model = SharedCore::initialized(7341).unwrap();
    let mut session = model.session(Intervention::Full);
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let mut valid = true;
    for position in 0..512_u16 {
        valid &= session.observe(position as u8).is_ok();
        valid &= std::hint::black_box(session.predict()) <= 256;
    }
    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
}

#[test]
fn shared_core_calibrated_observe_predict_has_zero_allocations() {
    use uor_r4_core::native_geometric::shared_core::{Intervention, SharedCore};
    let model = SharedCore::initialized(7341)
        .unwrap()
        .upgrade_calibrated()
        .unwrap();
    let mut session = model.session(Intervention::Full);
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let mut valid = true;
    for position in 0..512_u16 {
        valid &= session.observe(position as u8).is_ok();
        valid &= std::hint::black_box(session.predict()) <= 256;
    }
    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
}

#[test]
#[ignore = "Requires an actual block-calibrated or joint artifact path"]
fn shared_core_saved_block_candidate_has_zero_allocations() {
    use uor_r4_core::native_geometric::shared_core::{Intervention, SharedCore};
    let path = std::env::var("UOR_SHARED_CORE_CANDIDATE").expect("actual candidate path");
    let raw = std::fs::read(path).expect("candidate bytes");
    let model = SharedCore::from_bytes(&raw).expect("source-bound candidate");
    assert_eq!(model.to_bytes().unwrap(), raw);
    let mut session = model.session(Intervention::Full);
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let mut valid = true;
    for position in 0..512_u16 {
        valid &= session.observe(position as u8).is_ok();
        valid &= std::hint::black_box(session.predict()) <= 256;
    }
    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
}

/// Source operator/type-token guard for the actual native kernel and its
/// project-defined feature helpers. This does not inspect transitive standard
/// library implementations or generated machine code. Runtime allocation is
/// measured independently by the census below. Session setup, state rendering,
/// model construction/fitting, encoding and checkpoints are explicit host work.
#[test]
fn native_kernel_source_has_no_forbidden_arithmetic_or_float_types() {
    use uor_r4_core::transformerless::source_scan::{
        scan_for_forbidden_arith_and_floats, ALLOW_MARKER,
    };

    fn region<'a>(source: &'a str, begin: &str, end: &str) -> (&'a str, &'a str, &'a str) {
        assert_eq!(
            source.matches(begin).count(),
            1,
            "unique begin marker {begin}"
        );
        assert_eq!(source.matches(end).count(), 1, "unique end marker {end}");
        let (before, rest) = source.split_once(begin).unwrap();
        let (body, after) = rest.split_once(end).unwrap();
        (before, body, after)
    }

    let runtime = include_str!("../src/native_geometric/runtime.rs");
    let (host, kernel, after) = region(
        runtime,
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    assert!(
        after.trim().is_empty(),
        "runtime helpers must remain inside the scanned kernel region"
    );
    // The three existing constructor/report accessors are the only function
    // bodies before the runtime boundary; adding another requires reviewing
    // whether it belongs to host setup or the scanned prediction path.
    let host_functions: Vec<_> = host
        .lines()
        .map(str::trim)
        .filter(|line| line.contains("fn "))
        .collect();
    assert_eq!(
        host_functions,
        [
            "pub(super) fn new(model: &Model, control: Control) -> Self {",
            "pub fn state(&self) -> StateView {",
            "pub fn candidates(&self) -> &[Candidate] {",
        ]
    );
    for function in [
        "fn check_model(",
        "fn product(",
        "fn observe(",
        "fn begin_response(",
        "fn end_response(",
        "fn needs_input_boundary(",
        "fn response_decision(",
        "fn value_decision(",
        "fn completion_decision(",
        "fn response_entry_decision(",
        "fn word_copy_decision(",
        "fn recent(",
        "fn features(",
        "fn score_candidate(",
        "fn offer(",
        "fn offer_memory(",
        "fn predict(",
        "fn gate_eighths(",
    ] {
        assert_eq!(
            kernel.matches(function).count(),
            1,
            "kernel coverage includes {function}"
        );
    }
    let (_, features, _) = region(
        include_str!("../src/native_geometric/mod.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_END",
    );
    for function in ["fn group(", "fn shift(", "fn admitted("] {
        assert_eq!(
            features.matches(function).count(),
            1,
            "feature coverage includes {function}"
        );
    }
    // MemoryState allocation lives in memory_types.rs. The entire separate
    // memory runtime (including its private feature-admission helper) is scanned.
    let memory = include_str!("../src/native_geometric/memory_runtime.rs");
    for function in [
        "fn cue_identity(",
        "fn pack_query_occurrence(",
        "fn admitted(",
        "fn state(",
        "fn recent(",
        "fn observe(",
        "fn collect(",
    ] {
        assert_eq!(
            memory.matches(function).count(),
            1,
            "memory kernel coverage includes {function}"
        );
    }
    let response = include_str!("../src/native_geometric/response_runtime.rs");
    for function in [
        "fn response_view(",
        "fn begin_response(",
        "fn end_response(",
        "fn reference_for_sequence(",
        "fn select_response(",
        "fn commit_response(",
        "fn collect_continuation(",
    ] {
        assert_eq!(
            response.matches(function).count(),
            1,
            "response kernel coverage includes {function}"
        );
    }
    let value_runtime = include_str!("../src/native_geometric/value_runtime.rs");
    for function in [
        "fn observe(",
        "fn begin(",
        "fn end(",
        "fn proposal(",
        "fn features(",
        "fn offer(",
        "fn selected(",
        "fn commit(",
        "fn execute(",
    ] {
        assert_eq!(
            value_runtime.matches(function).count(),
            1,
            "typed-value kernel coverage includes {function}"
        );
    }
    let completion = include_str!("../src/native_geometric/completion_runtime.rs");
    for function in [
        "fn reset(",
        "fn observe(",
        "fn features(",
        "fn offer(",
        "fn selected(",
        "fn candidates(",
        "fn offer_token(",
        "fn score_candidate(",
    ] {
        assert_eq!(
            completion.matches(function).count(),
            1,
            "completion kernel coverage includes {function}"
        );
    }
    let completion_types = include_str!("../src/native_geometric/completion_types.rs");
    let response_entry = include_str!("../src/native_geometric/response_entry_runtime.rs");
    let word_copy = include_str!("../src/native_geometric/word_copy_runtime.rs");
    for function in [
        "fn eligible(",
        "fn reset(",
        "fn begin(",
        "fn observe(",
        "fn features(",
        "fn offer(",
        "fn selected(",
    ] {
        assert_eq!(
            response_entry.matches(function).count(),
            1,
            "response-entry kernel coverage includes {function}"
        );
    }
    let seed = completion_types
        .split_once("impl From<&ValueDecision> for CompletionSeed {")
        .unwrap()
        .1
        .split_once("#[derive")
        .unwrap()
        .0;
    let (_, numeral, _) = region(
        include_str!("../src/native_geometric/numeral.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    let (_, lexemes, _) = region(
        include_str!("../src/native_geometric/value_lexemes.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    for function in ["fn from_zphi(", "fn digit(", "fn feed(", "fn finish("] {
        assert_eq!(
            numeral.matches(function).count(),
            1,
            "numeral kernel coverage includes {function}"
        );
    }
    // The only external project arithmetic called by typed execution is the
    // existing exact checked coefficient addition; scan that actual method.
    let zphi_source = include_str!("../src/prime_route_attention.rs");
    let zphi = zphi_source.split_once("impl ZPhi {").unwrap().1;
    let zphi_add = zphi
        .split_once("pub fn checked_add(")
        .unwrap()
        .1
        .split_once("/// Exact checked multiplication")
        .unwrap()
        .0;
    let (_, operation_transition, _) = region(
        include_str!("../src/native_geometric/operation_transition.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    for (name, source) in [
        ("native operation transition", operation_transition),
        (
            "native lexical emission",
            region(
                include_str!("../src/native_geometric/lexical_emission.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native completed-word lexical emission",
            region(
                include_str!("../src/native_geometric/word_emission.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native writer lexical residual",
            region(
                include_str!("../src/native_geometric/writer_lexical.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native owner/value field composition",
            region(
                include_str!("../src/native_geometric/field_composition.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native historical owner/value field admission",
            region(
                include_str!("../src/native_geometric/historical_field_composition.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native contextual writer choice",
            region(
                include_str!("../src/native_geometric/writer_choice.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native historical source selection",
            region(
                include_str!("../src/native_geometric/historical_read.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native current-version source context",
            region(
                include_str!("../src/native_geometric/current_source.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native shared writer-role correction",
            region(
                include_str!("../src/native_geometric/writer_role.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        ("native runtime", kernel),
        ("native Feature helpers", features),
        ("native memory runtime", memory),
        ("native response runtime", response),
        ("native typed-value runtime", value_runtime),
        ("native completion runtime", completion),
        ("native response-entry runtime", response_entry),
        ("native retained-word copy runtime", word_copy),
        (
            "native learned routing runtime",
            include_str!("../src/native_geometric/learned_routing.rs"),
        ),
        (
            "native recurrent routing output",
            include_str!("../src/native_geometric/recurrent_routing.rs"),
        ),
        (
            "native retained-source routing",
            include_str!("../src/native_geometric/source_routing.rs"),
        ),
        (
            "native dependent read",
            include_str!("../src/native_geometric/dependent_read.rs"),
        ),
        (
            "native typed routing",
            include_str!("../src/native_geometric/typed_routing.rs"),
        ),
        (
            "native joint admission",
            region(
                include_str!("../src/native_geometric/joint_admission.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native source span",
            include_str!("../src/native_geometric/source_span.rs"),
        ),
        (
            "native relation memory",
            region(
                include_str!("../src/native_geometric/relation.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native retained relation extent",
            region(
                include_str!("../src/native_geometric/relation_span.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native learned relation start",
            region(
                include_str!("../src/native_geometric/relation_start.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native historical version intent",
            region(
                include_str!("../src/native_geometric/historical_version_intent.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        (
            "native current query handoff",
            region(
                include_str!("../src/native_geometric/current_query_handoff.rs"),
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
                "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
            )
            .1,
        ),
        ("native completion seed", seed),
        ("native numeral codec", numeral),
        ("native whole-word codec", lexemes),
        ("exact ZPhi checked addition", zphi_add),
        (
            "native vsa context engine",
            include_str!("../src/native_geometric/vsa/context_engine.rs"),
        ),
        (
            "native vsa attention",
            include_str!("../src/native_geometric/vsa/attention.rs"),
        ),
    ] {
        assert!(
            !source.contains(ALLOW_MARKER),
            "{name} permits no source-arithmetic exceptions"
        );
        let outcome = scan_for_forbidden_arith_and_floats(source);
        assert!(
            outcome.offenders.is_empty(),
            "{name}: {:?}",
            outcome.offenders
        );
        assert!(
            outcome.allowed.is_empty(),
            "{name}: no kernel allowances are permitted"
        );
    }
    let (_, hopf_kernel, _) = region(
        include_str!("../src/native_geometric/hopf_metric.rs"),
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN",
        "// NATIVE_GEOMETRIC_INTEGER_KERNEL_END",
    );
    assert!(
        !hopf_kernel.contains("f32") && !hopf_kernel.contains("f64"),
        "hopf integer serving kernel must contain zero float types"
    );
    for function in [
        "fn isqrt_u64(",
        "fn isqrt_u128(",
        "fn atan2_q30(",
        "fn dot_fiber_q30(",
        "fn to_unit_s3_q30(",
        "fn metric_distance_q30(",
        "fn from_hopf_fiber_q30(",
        "fn hopf_project(",
        "fn fiber_u1_q30(",
        "fn fiber_phase_q30(",
        "fn hopf_fiber_project(",
        "fn mul_q30(",
        "fn norm_q30(",
        "fn dot_q30(",
        "fn nearest_root_q30(",
        "fn step_q30(",
    ] {
        assert_eq!(
            hopf_kernel.matches(function).count(),
            1,
            "hopf integer serving kernel coverage includes {function}"
        );
    }
    let engram_source = include_str!("../src/native_geometric/engram.rs");
    assert!(
        !engram_source.contains("f32") && !engram_source.contains("f64"),
        "engram serving module must contain zero float types"
    );
    for function in [
        "fn hash_bigram(",
        "fn hash_trigram(",
        "fn hash_skip(",
        "fn lookup(",
        "fn lookup_skip(",
    ] {
        assert!(
            engram_source.contains(function),
            "engram integer serving module must include {function}"
        );
    }
}

/// Per-module multiplier census.
///
/// `cheap` / `dense` count binary `*` with a constant operand (split by set-bit count);
/// `variable` counts binary `*` with two non-literal operands; `deref` counts `*` used as a
/// dereference, which is not a multiply at all. The `method_*` fields count `*_mul(` calls and
/// their `pow(` equivalents, classified by their argument whether literal or not.
#[derive(Default, Debug, PartialEq, Eq)]
struct MulCensus {
    cheap: usize,
    dense: usize,
    variable: usize,
    deref: usize,
    method_cheap: usize,
    method_dense: usize,
    method_variable: usize,
}

impl MulCensus {
    /// Sites that engage a multiplier circuit: two runtime operands, a dense constant, or a
    /// method-form multiply whose argument is not a cheap constant.
    fn gated(&self) -> usize {
        self.variable + self.dense + self.method_dense + self.method_variable
    }
}

/// Blank out comments and string/char literals, preserving byte offsets.
fn mask_literals(src: &str) -> Vec<u8> {
    let b = src.as_bytes();
    let mut m = b.to_vec();
    let mut i = 0usize;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                m[i] = b' ';
                i += 1;
            }
        } else if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            // Rust block comments nest.
            let mut depth = 0usize;
            while i < b.len() {
                if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
                    depth += 1;
                    m[i] = b' ';
                    m[i + 1] = b' ';
                    i += 2;
                } else if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b'/' {
                    depth -= 1;
                    m[i] = b' ';
                    m[i + 1] = b' ';
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    m[i] = b' ';
                    i += 1;
                }
            }
        } else if b[i] == b'"' {
            m[i] = b' ';
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    m[i] = b' ';
                    if i + 1 < b.len() {
                        m[i + 1] = b' ';
                    }
                    i += 2;
                } else if b[i] == b'"' {
                    m[i] = b' ';
                    i += 1;
                    break;
                } else {
                    m[i] = b' ';
                    i += 1;
                }
            }
        } else if b[i] == b'\'' {
            // A char literal has its closing quote within a few bytes; a lifetime does not.
            let mut j = i + 1;
            let mut closed = None;
            while j < b.len() && j < i + 6 {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == b'\'' {
                    closed = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(end) = closed {
                for k in i..=end {
                    m[k] = b' ';
                }
                i = end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    m
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Parse the integer literal starting at `start`, returning its value and end offset.
fn literal_at(src: &[u8], start: usize) -> Option<(u64, usize)> {
    let mut j = start;
    if !src.get(j).map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return None;
    }
    if src.get(j) == Some(&b'0') && matches!(src.get(j + 1), Some(b'x') | Some(b'X')) {
        j += 2;
        let s = j;
        while src
            .get(j)
            .map(|c| c.is_ascii_hexdigit() || *c == b'_')
            .unwrap_or(false)
        {
            j += 1;
        }
        if j == s {
            return None;
        }
        let digits: String = src[s..j]
            .iter()
            .map(|&c| c as char)
            .filter(|c| *c != '_')
            .collect();
        return u64::from_str_radix(&digits, 16).ok().map(|v| (v, j));
    }
    let s = j;
    while src
        .get(j)
        .map(|c| c.is_ascii_digit() || *c == b'_')
        .unwrap_or(false)
    {
        j += 1;
    }
    let digits: String = src[s..j]
        .iter()
        .map(|&c| c as char)
        .filter(|c| *c != '_')
        .collect();
    digits.parse::<u64>().ok().map(|v| (v, j))
}

fn census(src: &str) -> MulCensus {
    let m = mask_literals(src);
    let mut c = MulCensus::default();

    for i in 0..m.len() {
        if m[i] != b'*' {
            continue;
        }
        // Previous non-whitespace byte decides dereference vs binary operator.
        let mut p = i;
        while p > 0 && (m[p - 1] as char).is_whitespace() {
            p -= 1;
        }
        let prev = if p == 0 { None } else { Some(m[p - 1]) };
        let binary = matches!(prev, Some(x) if is_ident_byte(x) || x == b')' || x == b']');
        if !binary {
            c.deref += 1;
            continue;
        }
        // Literal on either side?
        let mut q = i + 1;
        while q < m.len() && (m[q] as char).is_whitespace() {
            q += 1;
        }
        let right = literal_at(&m, q);
        // The token ending just before the operator.
        let mut e = p;
        while e > 0 && is_ident_byte(m[e - 1]) {
            e -= 1;
        }
        let left = literal_at(&m, e);
        match (left, right) {
            (Some(_), _) | (_, Some(_)) => {
                let v = left.or(right).map(|(v, _)| v).unwrap_or(0);
                if v.count_ones() <= 4 {
                    c.cheap += 1;
                } else {
                    c.dense += 1;
                }
            }
            (None, None) => c.variable += 1,
        }
    }

    // Method forms: `*_mul(` and `pow(`. Their first argument decides the class.
    for name in [
        "wrapping_mul(",
        "saturating_mul(",
        "checked_mul(",
        "overflowing_mul(",
        "unchecked_mul(",
        "wrapping_pow(",
        "saturating_pow(",
        "checked_pow(",
    ] {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(name) {
            let at = from + rel + name.len();
            let arg = literal_at(&m, at);
            match arg {
                Some((v, _)) if v.count_ones() <= 4 => c.method_cheap += 1,
                Some(_) => c.method_dense += 1,
                None => c.method_variable += 1,
            }
            from = at;
        }
    }
    c
}

/// Ratcheting multiplier census over the serving path.
///
/// # Why this exists
///
/// The multiplier-free invariant was enforced against a *named subset* of the kernel, so
/// `hopf_metric.rs` was float-checked but never arithmetic-scanned — and its 82 raw multiplies,
/// on the serving hot path, were invisible. A guard that covers a list cannot report what it
/// does not cover. This test makes the count explicit per module and ratcheting: a module may not
/// ADD multiplies, and each ceiling carries its target.
///
/// Ceilings may only be LOWERED as multiplies are removed. Raising one weakens the D0-a serving
/// contract and requires an owner decision.
#[test]
fn serving_path_multiplier_census_is_ratcheting() {
    use uor_r4_core::transformerless::source_scan::scan_for_forbidden_arith;

    let hopf_src = include_str!("../src/native_geometric/hopf_metric.rs");
    let begin = hopf_src
        .find("// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN")
        .unwrap_or(0);
    let end = hopf_src
        .find("// NATIVE_GEOMETRIC_INTEGER_KERNEL_END")
        .unwrap_or(hopf_src.len());
    let kernel = &hopf_src[begin..end];

    // (label, source, raw ceiling, target). Ceilings may only be LOWERED; raising one weakens
    // D0-a and needs an owner decision.
    let modules: [(&str, &str, usize, &str); 7] = [
        ("hopf_metric.rs integer kernel", kernel, 43, "zero"),
        (
            "engram.rs",
            include_str!("../src/native_geometric/engram.rs"),
            40,
            "zero",
        ),
        (
            "lattice_table.rs",
            include_str!("../src/native_geometric/lattice_table.rs"),
            98,
            "zero",
        ),
        (
            "vsa/hypervector.rs",
            include_str!("../src/native_geometric/vsa/hypervector.rs"),
            18,
            "zero",
        ),
        (
            "vsa/attention.rs",
            include_str!("../src/native_geometric/vsa/attention.rs"),
            0,
            "zero",
        ),
        (
            "vsa/hierarchical.rs",
            include_str!("../src/native_geometric/vsa/hierarchical.rs"),
            6,
            "zero",
        ),
        (
            "learner/binary_model.rs",
            include_str!("../src/native_geometric/learner/binary_model.rs"),
            78,
            "zero",
        ),
    ];

    let mut counts: Vec<(&str, usize, MulCensus)> = Vec::with_capacity(modules.len());
    for (label, src, _ceiling, _target) in modules {
        counts.push((
            label,
            scan_for_forbidden_arith(src).offenders.len(),
            census(src),
        ));
    }

    // Controls. `vsa/attention.rs` is written with `square_u64` and `div_small_positive`, so it is
    // the negative control and must classify completely clean; `mul_q30` multiplies two Q1.30
    // runtime operands, so it is the positive control.
    let attention = census(include_str!("../src/native_geometric/vsa/attention.rs"));
    assert_eq!(
        (
            attention.cheap,
            attention.dense,
            attention.variable,
            attention.method_cheap,
            attention.method_dense,
            attention.method_variable,
        ),
        (0, 0, 0, 0, 0, 0),
        "control failed: vsa/attention.rs uses shift-add idioms and must have no multiplying \
         form at all (dereferences are expected and are not multiplies): {attention:?}"
    );
    let mul_fn = hopf_src
        .split("pub fn mul_q30")
        .nth(1)
        .and_then(|s| s.split("pub fn norm_q30").next())
        .unwrap_or("");
    let mq = census(mul_fn);
    assert!(
        mq.variable > 0,
        "control failed: mul_q30 must classify as variable, got {mq:?}"
    );

    println!("multiplier census (token-level classifier):");
    println!(
        "  {:<32} {:>5} {:>6} {:>6} {:>8} {:>6} {:>8} {:>8} {:>7}",
        "module", "raw", "cheap", "dense", "variable", "deref", "m_cheap", "m_dense", "m_var"
    );
    let mut raw_total = 0usize;
    let mut totals = MulCensus::default();
    for (label, raw, c) in &counts {
        println!(
            "  {label:<32} {raw:>5} {:>6} {:>6} {:>8} {:>6} {:>8} {:>8} {:>7}",
            c.cheap,
            c.dense,
            c.variable,
            c.deref,
            c.method_cheap,
            c.method_dense,
            c.method_variable
        );
        raw_total += raw;
        totals.cheap += c.cheap;
        totals.dense += c.dense;
        totals.variable += c.variable;
        totals.deref += c.deref;
        totals.method_cheap += c.method_cheap;
        totals.method_dense += c.method_dense;
        totals.method_variable += c.method_variable;
    }
    println!(
        "  {:<32} {raw_total:>5} {:>6} {:>6} {:>8} {:>6} {:>8} {:>8} {:>7}  gated={}",
        "TOTAL",
        totals.cheap,
        totals.dense,
        totals.variable,
        totals.deref,
        totals.method_cheap,
        totals.method_dense,
        totals.method_variable,
        totals.gated()
    );
    println!(
        "  NOTE: `deref` counts are excluded from the gate; the classifier is a token scan, so a"
    );
    println!("  construct it mis-tokenises is possible. The controls above bound that risk.");

    let mut breaches = Vec::new();
    for (i, (label, raw, _c)) in counts.iter().enumerate() {
        let ceiling = modules[i].2;
        if *raw > ceiling {
            breaches.push(format!(
                "{label}: {raw} raw multiplying operators, ceiling {ceiling}"
            ));
        }
    }
    assert!(
        breaches.is_empty(),
        "serving-path multiplier ceilings exceeded (a ceiling may only be LOWERED as multiplies \
         are removed; raising one weakens D0-a and needs an owner decision):\n{}",
        breaches.join("\n")
    );
}

#[test]
fn native_learned_routing_selection_and_transformation_are_allocation_free() {
    routing_allocation(false);
}

#[test]
fn native_recurrent_routing_joint_output_is_allocation_free() {
    routing_allocation(true);
}

fn routing_allocation(recurrent: bool) {
    use uor_r4_core::native_geometric::{RoutingFitConfig, RoutingMode};
    let docs = [Document {
        id: "routing-allocation".into(),
        text: "Alice saved red. Bob saved blue. Alice gave Bob red.".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 16,
            ..Config::default()
        },
        &docs,
    )
    .unwrap();
    trainer.train_documents(&docs).unwrap();
    let parent = trainer.compile().unwrap();
    let config = RoutingFitConfig {
        max_positions: 32,
        learned_tokens: 2,
        passes: 1,
        mode: RoutingMode::Angular,
        ..RoutingFitConfig::default()
    };
    let model = if recurrent {
        parent
            .fit_recurrent_routing(
                &docs,
                &[uor_r4_core::native_geometric::ValueExample {
                    id: "routing-allocation-response".into(),
                    prompt: "Alice saved".into(),
                    response: " red.".into(),
                }],
                config,
            )
            .unwrap()
            .0
    } else {
        parent.fit_routing_block(&docs, config).unwrap().0
    };
    let tokens = model.encode(&docs[0].text).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        for _ in 0..3 {
            for &token in &tokens {
                session.observe(&model, token)?;
                session.predict(&model)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
    assert!(session.work.learned_routing.payload_gathers > 0);
    assert!(session.work.learned_routing.operator_executions > 0);
}

#[test]
fn native_typed_value_ingest_write_emission_and_eviction_are_allocation_free() {
    typed_value_allocation_census(false);
    typed_value_allocation_census(true);
}

fn typed_value_allocation_census(lexeme_cues: bool) {
    let documents = [Document {
        id: "typed-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17 unknown".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let baseline = trainer.compile().unwrap();
    let source = [ValueExample {
        id: "typed-allocation-fit".into(),
        prompt: "left = 13; right = 4; total:".into(),
        response: "17".into(),
    }];
    let config = ValueFitConfig {
        epochs: 64,
        learning_rate: 0.25,
        max_features: 4096,
    };
    let (model, _) = if lexeme_cues {
        baseline.fit_values_with_lexeme_cues(&source, config)
    } else {
        baseline.fit_values(&source, config)
    }
    .unwrap();
    let tokens = model.encode(&source[0].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let before = session.state().values.unwrap().storage_bytes;

    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for &token in &tokens {
            session.observe(&model, token)?;
        }
        session.begin_response(&model)?;
        let first = session.predict(&model)?;
        let selected = session.value_decision().is_some();
        session.observe(&model, first.token)?;
        // Interrupt a committed multi-byte value using a different teacher
        // byte. It must clear the cursor without an allocating recovery path.
        session.predict(&model)?;
        session.observe(&model, u32::from(b'x') + 2)?;
        session.end_response(&model)?;
        for _ in 0..64 {
            for &token in &tokens {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..2 {
                let prediction = session.predict(&model)?;
                session.observe(&model, prediction.token)?;
            }
            session.refresh_value_sources(&model)?;
            session.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(selected)
    })();
    MEASURING.with(|enabled| enabled.set(false));
    let allocations = ALLOCATIONS.with(Cell::get);
    let bytes = BYTES.with(Cell::get);
    assert!(result.unwrap(), "fixture must select a fitted typed result");
    assert_eq!((allocations, bytes), (0, 0));
    assert_eq!(session.state().values.unwrap().storage_bytes, before);
    assert!(session.work.values.input_bytes > 0);
    assert!(session.work.values.literal_writes > 16);
    assert!(session.work.values.record_evictions > 0);
    assert!(session.work.values.derived_writes > 0);
    assert!(session.work.values.source_refreshes > 0);
    assert!(session.work.values.emission_commits > 0);
    assert!(session.work.values.emission_mismatches > 0);
    assert!(session.work.evictions > 0);
    println!(
        "typed-value census allocations={allocations} bytes={bytes} work={:?}",
        session.work.values
    );
}

#[test]
fn native_value_completion_commit_mismatch_and_limit_are_allocation_free() {
    let documents = [Document {
        id: "completion-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17.\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let baseline = trainer.compile().unwrap();
    let source = [ValueExample {
        id: "completion-allocation-fit".into(),
        prompt: "left = 13; right = 4; total:".into(),
        response: "17.\n".into(),
    }];
    let (typed, _) = baseline
        .fit_values_with_lexeme_cues(
            &source,
            ValueFitConfig {
                epochs: 64,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    let (model, report) = typed
        .fit_value_completion(&source, ValueCompletionFitConfig::default())
        .unwrap();
    assert_eq!(report.matched_numeric, 1);
    let prompt = model.encode(&source[0].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let storage = session.state().completion.unwrap().storage_bytes;
    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for _ in 0..4 {
            session.end_response(&model)?;
            for &token in &prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..2 {
                let next = session.predict(&model)?.token;
                session.observe(&model, next)?;
            }
            // Commit one learned suffix token, then deliberately observe a
            // different byte and exercise the finite progress limit.
            let next = session.predict(&model)?.token;
            session.observe(&model, next)?;
            session.predict(&model)?;
            session.observe(&model, u32::from(b'x') + 2)?;
            for _ in 0..31 {
                session.predict(&model)?;
                session.observe(&model, u32::from(b'x') + 2)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|enabled| enabled.set(false));
    result.unwrap();
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(session.state().completion.unwrap().storage_bytes, storage);
    assert_eq!(session.work.completion.anchors, 4);
    assert!(session.work.completion.commits > 0);
    assert!(session.work.completion.mismatches > 0);
    assert_eq!(session.work.completion.step_limits, 4);
    assert!(session.work.evictions > 0);
}

#[test]
fn native_response_entry_commit_mismatch_and_limit_are_allocation_free() {
    let catalog = [Document {
        id: "entry-allocation-catalog".into(),
        text: "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 8,
            ..Config::default()
        },
        &catalog,
    )
    .unwrap();
    trainer.train_documents(&catalog).unwrap();
    let examples = (0..4)
        .flat_map(|index| {
            [
                ValueExample {
                    id: format!("entry-allocation-numeric-{index}"),
                    prompt: format!("left = {}; right = 4; total:", 13 + index),
                    response: format!("{}.\n", 17 + index),
                },
                ValueExample {
                    id: format!("entry-allocation-no-write-{index}"),
                    prompt: format!("left = {}; right = 4; reply:", 13 + index),
                    response: " Unknown.\n".into(),
                },
            ]
        })
        .collect::<Vec<_>>();
    let (typed, _) = trainer
        .compile()
        .unwrap()
        .fit_values_with_lexeme_cues(
            &examples,
            ValueFitConfig {
                epochs: 64,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    let (completion, _) = typed
        .fit_value_completion(&examples, ValueCompletionFitConfig::default())
        .unwrap();
    let (model, _) = completion
        .fit_response_entry(&examples, ResponseEntryFitConfig::default())
        .unwrap();
    let prompt = model.encode(&examples[1].prompt).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let storage = session.state().response_entry.unwrap().storage_bytes;
    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|count| count.set(0));
    MEASURING.with(|enabled| enabled.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        let mut selected_entry = true;
        for _ in 0..4 {
            session.end_response(&model)?;
            for &token in &prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            let first = session.predict(&model)?.token;
            selected_entry &= session.response_entry_decision().is_some();
            // A repeated prediction must preserve the pending first choice.
            selected_entry &= session.predict(&model)?.token == first;
            session.observe(&model, first)?;
            // A mismatching observed byte follows actual history. Repeated
            // observations then exercise the bounded entry-progress limit.
            for _ in 0..40 {
                session.predict(&model)?;
                session.observe(&model, u32::from(b'x') + 2)?;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(selected_entry)
    })();
    MEASURING.with(|enabled| enabled.set(false));
    assert!(
        result.unwrap(),
        "fixture must select its learned lexical entry"
    );
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(
        session.state().response_entry.unwrap().storage_bytes,
        storage
    );
    assert!(session.work.response_entry.commits > 0);
    assert!(session.work.response_entry.mismatches > 0);
    assert_eq!(session.work.response_entry.step_limits, 4);
    assert_eq!(session.work.values.derived_writes, 0);
    assert!(session.work.evictions > 0);
}

#[test]
fn native_word_copy_selection_bytes_mismatch_and_cap_are_allocation_free() {
    let (_, original) = copy_fixture::fitted();
    let completed_word = copy_fixture::fitted_completed_word();
    let composed = copy_fixture::fitted_composed();
    let binding = copy_fixture::fitted_shared_binding();
    for (variant, model) in [
        ("actual-byte suffix", original),
        ("completed-word suffix", completed_word),
        ("composed dispatch", composed),
        ("shared binding entry", binding),
    ] {
        let prompt = model.encode(copy_fixture::COPY_PROMPT).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        let storage = session.state().word_copy.unwrap().storage_bytes;
        let mut selected = false;
        let mut repeat_stable = true;
        let mut copied_bytes = 0;
        let mut suffix_decisions = 0;
        let mut observed_eos = false;
        ALLOCATIONS.with(|count| count.set(0));
        BYTES.with(|count| count.set(0));
        MEASURING.with(|enabled| enabled.set(true));
        let result = (|| {
            session.observe(model, BOS)?;
            for &token in &prompt {
                session.observe(model, token)?;
            }
            session.begin_response(model)?;
            for _ in 0..32 {
                let next = session.predict(model)?;
                let decision = session.word_copy_decision();
                selected |= decision.is_some();
                copied_bytes += usize::from(decision.is_some_and(|choice| {
                    matches!(choice.action, WordCopyAction::Start | WordCopyAction::Byte)
                }));
                suffix_decisions += usize::from(decision.is_some_and(|choice| {
                    matches!(choice.action, WordCopyAction::Emit | WordCopyAction::Stop)
                }));
                let repeated = session.predict(model)?;
                repeat_stable &= repeated == next;
                session.observe(model, next.token)?;
                if next.token == EOS {
                    observed_eos = true;
                    break;
                }
            }
            session.end_response(model)?;
            for &token in &prompt {
                session.observe(model, token)?;
            }
            session.begin_response(model)?;
            let first = session.predict(model)?.token;
            session.observe(model, first)?;
            for _ in 0..40 {
                session.predict(model)?;
                session.observe(model, u32::from(b'?') + 2)?;
            }
            session.end_response(model)?;
            Ok::<_, uor_r4_core::native_geometric::Error>(())
        })();
        MEASURING.with(|enabled| enabled.set(false));
        result.expect(variant);
        assert!(
            selected,
            "{variant}: fixture must execute learned retained-word selection"
        );
        assert!(
            repeat_stable,
            "{variant}: repeated prediction must retain the selected byte"
        );
        assert!(
            copied_bytes >= 5,
            "{variant}: fixture must execute its multi-byte cursor"
        );
        assert!(
            suffix_decisions > 0 && observed_eos,
            "{variant}: fixture must execute its suffix through EOS"
        );
        assert_eq!(
            (ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)),
            (0, 0),
            "{variant}"
        );
        assert_eq!(session.state().word_copy.unwrap().storage_bytes, storage);
        assert!(session.work.word_copy.word_candidates > 0);
        assert!(session.work.word_copy.dictionary_comparisons > 0);
        assert!(session.work.word_copy.byte_reads > 0);
        if variant == "composed dispatch" {
            assert!(session.work.word_copy.forced_dispatches > 0);
        }
        assert!(session.work.word_copy.selector.mismatches > 0);
        assert!(session.work.response_entry.step_limits > 0);
        assert_eq!(session.work.values.derived_writes, 0);
    }
}

#[test]
fn native_observe_predict_stays_allocation_free_through_evictions() {
    let documents = [
        Document {
            id: "a".into(),
            text: "red fox finds green stone. red fox keeps green stone.".into(),
        },
        Document {
            id: "b".into(),
            text: "blue bird finds amber seed. blue bird keeps amber seed.".into(),
        },
        Document {
            id: "c".into(),
            text: "green turtle gives red fox a blue shell.".into(),
        },
    ];
    let config = Config {
        context_tokens: 32,
        candidate_limit: 3,
        postings_per_row: 4,
        ..Config::default()
    };
    let mut trainer = Trainer::new(config, &documents).unwrap();
    trainer.train_documents(&documents).unwrap();
    let model = trainer.compile().unwrap();
    let readout_documents = [Document {
        id: "readout-fit".into(),
        text: "red fox gives blue bird green stone. blue bird finds red shell.".into(),
    }];
    let (learned, _) = model
        .fit_readout(
            &readout_documents,
            ReadoutFitConfig {
                max_positions: 128,
                epochs: 2,
                max_queries: 16,
            },
        )
        .unwrap();
    let memory_config = MemoryReadFitConfig {
        advance_response_path: false,
        query_tokens: 4,
        source_offsets: 2,
        postings_per_address: 2,
        candidate_limit: 16,
        max_positions: 128,
        epochs: 2,
        max_features: 1024,
    };
    let (with_memory, memory_fit) = learned
        .fit_memory_read_with_word_cues(&readout_documents, memory_config)
        .unwrap();
    let (exact_memory, _) = learned
        .fit_memory_read(&readout_documents, memory_config)
        .unwrap();
    let (query_context_memory, _) = learned
        .fit_memory_read_with_query_context(&readout_documents, memory_config, true)
        .unwrap();
    let mut composition_trainer = MemoryReadTrainer::new_with_occurrence_composition(
        &learned,
        &readout_documents,
        memory_config,
        MemoryReadSchedule {
            total_positions: 128,
            batch_positions: 128,
        },
        true,
        None,
    )
    .unwrap();
    while !composition_trainer.is_complete() {
        composition_trainer
            .advance(16, std::time::Duration::from_secs(10))
            .unwrap();
    }
    let (occurrence_memory, _) = composition_trainer.finish().unwrap();
    let response_prompts = [
        "red fox finds green stone. blue bird finds amber seed. red fox keeps",
        "blue bird finds amber seed. red fox finds green stone. blue bird keeps",
    ];
    let response_documents: Vec<_> = response_prompts
        .iter()
        .enumerate()
        .map(|(index, prompt)| Document {
            id: format!("response-allocation-fit-{index}"),
            text: format!(
                "{prompt} {}.",
                if index == 0 {
                    "green stone"
                } else {
                    "amber seed"
                }
            ),
        })
        .collect();
    let response_spans = response_documents
        .iter()
        .zip(response_prompts)
        .map(|(document, prompt)| {
            let prefix = learned.encode(prompt).unwrap();
            let full = learned.encode(&document.text).unwrap();
            assert!(full.starts_with(&prefix));
            vec![MemoryReadTokenSpan {
                start: prefix.len(),
                end: full.len() + 1,
            }]
        })
        .collect();
    let supervision =
        MemoryReadSupervision::new(&learned, &response_documents, response_spans).unwrap();
    let fit_response = |advance_response_path| {
        let mut response_trainer = MemoryReadTrainer::new_with_response_state(
            &learned,
            &response_documents,
            MemoryReadFitConfig {
                advance_response_path,
                ..memory_config
            },
            MemoryReadSchedule {
                total_positions: 128,
                batch_positions: 128,
            },
            true,
            supervision.clone(),
        )
        .unwrap();
        while !response_trainer.is_complete() {
            response_trainer
                .advance(16, std::time::Duration::from_secs(10))
                .unwrap();
        }
        response_trainer.finish().unwrap().0
    };
    let response_memory = fit_response(false);
    let advancing_response_memory = fit_response(true);
    assert!(
        !response_memory
            .memory_read_config()
            .unwrap()
            .advance_response_path
    );
    assert!(
        advancing_response_memory
            .memory_read_config()
            .unwrap()
            .advance_response_path
    );
    assert_ne!(
        response_memory.memory_read_feature_layout(),
        advancing_response_memory.memory_read_feature_layout()
    );
    assert!(
        memory_fit.target_in_memory > 0,
        "fixture must exercise fitted memory alternatives"
    );
    let tokens = model.encode(&documents[0].text).unwrap();
    assert!(!tokens.is_empty());
    let mut measured_decisions = 0;
    for (readout, model) in [
        ("fixed_v1", model),
        ("learned_v1", learned),
        ("learned_memory_exact", exact_memory),
        ("learned_memory_with_aliases", with_memory),
        ("query_context_memory_with_aliases", query_context_memory),
        ("occurrence_composition", occurrence_memory),
        ("persistent_response", response_memory),
        ("persistent_advancing_response", advancing_response_memory),
    ] {
        let persistent = model
            .memory_read_version()
            .is_some_and(|version| version.ends_with("/5"));
        let word_cues =
            model.memory_cue_identity() == Some("leading-unicode-whitespace-word-equivalence/1");
        for control in [
            Control::Full,
            Control::GeometryDisabled,
            Control::ZetaDisabled,
            Control::H4Disabled,
            Control::OrientationDisabled,
            Control::PairedDisabled,
            Control::RadialDisabled,
            Control::HeatmapDisabled,
            Control::MemoryDisabled,
            Control::ResponseStateDisabled,
        ] {
            let active_response = persistent
                && control != Control::MemoryDisabled
                && control != Control::ResponseStateDisabled;
            let mut session = model.session(control).unwrap();
            session.observe(&model, BOS).unwrap();
            let before = session.state();
            ALLOCATIONS.with(|count| count.set(0));
            BYTES.with(|count| count.set(0));
            MEASURING.with(|enabled| enabled.set(true));
            let result = (|| {
                let mut stopped = false;
                for index in 0..1024 {
                    let response_step = active_response && index % 32 >= 16;
                    if persistent && index % 32 == 0 {
                        session.end_response(&model)?;
                    }
                    if response_step && (index % 32 == 16 || stopped) {
                        session.begin_response(&model)?;
                    }
                    let prediction = session.predict(&model)?;
                    let token = if response_step {
                        prediction.token
                    } else {
                        tokens[index % tokens.len()]
                    };
                    stopped = response_step && token == EOS;
                    session.observe(&model, token)?;
                }
                Ok::<_, uor_r4_core::native_geometric::Error>(())
            })();
            MEASURING.with(|enabled| enabled.set(false));
            let allocations = ALLOCATIONS.with(Cell::get);
            let bytes = BYTES.with(Cell::get);
            result.unwrap();
            measured_decisions += 1024;
            assert_eq!((allocations, bytes), (0, 0), "control {control:?}");
            let after = session.state();
            assert_eq!(after.context_capacity, 32);
            assert_eq!(after.retained_tokens, 32);
            assert_eq!(after.tokens_seen, 1025);
            assert_eq!(session.work.evictions, 993);
            assert_eq!(after.ring_storage_bytes, before.ring_storage_bytes);
            assert_eq!(
                after.candidate_storage_bytes,
                before.candidate_storage_bytes
            );
            assert!(session.candidates().len() <= 3);
            assert!(session.work.candidate_offers <= 1024 * (3 + 26 * 4));
            assert!(session.work.candidate_evaluations <= session.work.candidate_offers);
            if active_response {
                assert!(session.work.response_query_captures >= 32);
                assert!(session.work.response_query_captures <= 512);
                assert_eq!(
                    session.work.response_commits + session.work.response_stops,
                    512
                );
                assert!(session.work.response_reference_reads > 0);
                assert_eq!(session.work.response_mismatches, 0);
            } else {
                assert_eq!(session.work.response_query_captures, 0);
                assert_eq!(session.work.response_commits, 0);
                assert_eq!(session.work.response_reference_reads, 0);
            }
            let captures = session.work.response_query_captures;
            let memory_bytes = if let (Some(before), Some(after)) =
                (before.memory_read, after.memory_read)
            {
                assert_eq!(after.retained_tokens, 32);
                assert_eq!(before.ring_storage_bytes, after.ring_storage_bytes);
                assert_eq!(before.index_storage_bytes, after.index_storage_bytes);
                assert_eq!(
                    before.candidate_storage_bytes,
                    after.candidate_storage_bytes
                );
                assert!(session.work.memory_index_writes > 0);
                assert!(
                    session.work.memory_candidates
                        <= 1024 * (memory_config.candidate_limit + usize::from(persistent)) as u64
                );
                assert!(
                    session.work.memory_index_reads
                        <= 1025
                            * (memory_config.source_offsets
                                * (memory_config.postings_per_address - 1))
                                as u64
                            + 1024 * memory_config.candidate_limit as u64
                            + captures * memory_config.candidate_limit as u64
                );
                assert!(
                    session.work.memory_index_writes
                        <= 1025
                            * (memory_config.source_offsets * memory_config.postings_per_address)
                                as u64
                );
                assert!(session.work.memory_score_lookups <= session.work.memory_candidates * 18);
                assert_eq!(
                    before.composed_candidate_storage_bytes,
                    after.composed_candidate_storage_bytes
                );
                assert_eq!(
                    before.composition_feature_storage_bytes,
                    after.composition_feature_storage_bytes
                );
                assert_eq!(before.response_storage_bytes, after.response_storage_bytes);
                assert_eq!(after.response_storage_bytes > 0, persistent);
                if word_cues {
                    assert!(session.work.memory_cue_reads > 0);
                } else {
                    assert_eq!(session.work.memory_cue_reads, 0);
                }
                assert!(
                    session.work.memory_cue_reads
                        <= 1025 * memory_config.source_offsets as u64
                            + 1024 * memory_config.candidate_limit as u64
                            + captures * memory_config.candidate_limit as u64
                );
                if control == Control::MemoryDisabled {
                    assert_eq!(session.work.memory_candidates, 0);
                    assert_eq!(session.work.memory_score_lookups, 0);
                    // Observation still updates the same memory state; only
                    // prediction-side cue reads disappear under this control.
                    assert!(
                        session.work.memory_cue_reads <= 1025 * memory_config.source_offsets as u64
                    );
                } else {
                    assert!(session.work.memory_candidates > 0);
                    assert!(session.work.memory_score_lookups > 0);
                }
                after.ring_storage_bytes
                    + after.index_storage_bytes
                    + after.candidate_storage_bytes
                    + after.composed_candidate_storage_bytes
                    + after.composition_feature_storage_bytes
                    + after.response_storage_bytes
            } else {
                assert!(model.memory_read_version().is_none());
                assert_eq!(session.work.memory_candidates, 0);
                assert_eq!(session.work.memory_cue_reads, 0);
                0
            };
            println!("{readout} {control:?}: decisions=1024 evictions=993 allocations={allocations} bytes={bytes} ring_bytes={} candidate_bytes={} offers={} evaluations={} memory_bytes={memory_bytes} memory_candidates={} memory_score_lookups={} memory_cue_reads={} response_captures={} response_commits={} response_requeries={} response_continuations={} response_stops={}",
            after.ring_storage_bytes, after.candidate_storage_bytes, session.work.candidate_offers, session.work.candidate_evaluations,
            session.work.memory_candidates,session.work.memory_score_lookups,session.work.memory_cue_reads,
            captures, session.work.response_commits, session.work.response_requeries, session.work.response_continuations, session.work.response_stops);
        }
    }
    assert_eq!(measured_decisions, 8 * 10 * 1024);
}

#[test]
fn source_routing_observe_select_and_copy_are_allocation_free() {
    use uor_r4_core::native_geometric::SourceRoutingConfig;
    let docs = [
        ValueExample {
            id: "source-allocation-copy".into(),
            prompt: "holder in alpha. Where is holder? Answer:".into(),
            response: " alpha.\n".into(),
        },
        ValueExample {
            id: "source-allocation-none".into(),
            prompt: "alpha in city. Where is missing? Answer:".into(),
            response: " Unknown.\n".into(),
        },
    ];
    let (parent, _) = copy_fixture::fitted_composed()
        .fit_role_read(
            &docs,
            ResponseEntryFitConfig {
                epochs: 8,
                ..ResponseEntryFitConfig::default()
            },
        )
        .unwrap();
    let (model, _) = parent
        .fit_source_routing(
            &docs,
            SourceRoutingConfig {
                learned_features: 8,
                passes: 1,
                proposals: 2,
                role_context_only: true,
                ..SourceRoutingConfig::default()
            },
        )
        .unwrap();
    let tokens = model.encode(&docs[0].prompt).unwrap();
    let mut s = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|n| n.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for &t in &tokens {
            s.observe(&model, t)?;
        }
        s.begin_response(&model)?;
        for _ in 0..24 {
            let p = s.predict(&model)?;
            s.observe(&model, p.token)?;
            if p.token == EOS {
                break;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|n| n.set(false));
    result.unwrap();
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
    assert!(s.work.word_copy.routing.predictions > 0);
}

/// Actual retained artifact exercise. The large local learned artifact is not a
/// repository fixture; CI's small-fixture census does not claim this execution.
#[test]
#[ignore = "requires R4_DEPENDENT_READ_MODEL retained development artifact"]
fn native_dependent_artifact_copy_is_allocation_free() {
    let path = std::env::var("R4_DEPENDENT_READ_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tokens = model
        .encode(
            "casket in elvin. elvin in Bremen. Question: Where is the location of casket? Answer:",
        )
        .unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let mut selected = false;
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for &token in &tokens {
            session.observe(&model, token)?;
        }
        session.begin_response(&model)?;
        for _ in 0..32 {
            let p = session.predict(&model)?;
            selected |= session
                .word_copy_decision()
                .is_some_and(|d| d.dependency.is_some_and(|ids| ids[1] != 0));
            session.observe(&model, p.token)?;
            if p.token == EOS {
                break;
            }
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert!(selected, "dependent path was not exercised");
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    println!("dependent exact two-link ingest/select/observe/copy: allocations=0 bytes=0");
}

#[test]
#[ignore = "requires R4_TYPED_ROUTING_MODEL learned development artifact"]
fn native_typed_artifact_routing_is_allocation_free() {
    let path = std::env::var("R4_TYPED_ROUTING_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let first=model.encode("User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:").unwrap();
    let second = model
        .encode("User: There are 5 new coins. Add the new coins to the previous total.\nAssistant:")
        .unwrap();
    let mut session = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        session.observe(&model, BOS)?;
        for prompt in [&first, &second] {
            for &token in prompt {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            for _ in 0..32 {
                let token = session.predict(&model)?.token;
                session.observe(&model, token)?;
                if token == EOS {
                    break;
                }
            }
            session.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert!(session.work.values.routing.predictions > 0);
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    println!("actual typed geometric routing/commit: allocations=0 bytes=0");
}

#[test]
#[ignore = "requires R4_TYPED_ROLES_MODEL retained role selection artifact"]
fn native_typed_role_artifact_is_allocation_free() {
    let path = std::env::var("R4_TYPED_ROLES_MODEL").expect("named model path");
    let model =
        uor_r4_core::native_geometric::Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let prompts=[
        "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
        "User: There are 5 new coins. Add the new coins to the previous total.\nAssistant:",
        "User: Repeat the original total.\nAssistant:",
        "User: There are 2 extra coins. Add the extra coins to the updated total.\nAssistant:",
    ].map(|p|model.encode(p).unwrap());
    let mut s = model.session(Control::Full).unwrap();
    let mut answers = [None; 4];
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for (i, prompt) in prompts.iter().enumerate() {
            for &token in prompt {
                s.observe(&model, token)?;
            }
            s.begin_response(&model)?;
            for _ in 0..32 {
                let token = s.predict(&model)?.token;
                if let Some(d) = s.value_decision().filter(|d| d.cursor == 0) {
                    answers[i] = Some(d.value);
                }
                s.observe(&model, token)?;
                if token == EOS {
                    break;
                }
            }
            s.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!(answers, [Some(17), Some(22), Some(17), Some(24)]);
    assert!(s.work.values.alias_self_add_rejections > 0);
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    let checkpoint = s.checkpoint().unwrap();
    let restored = model.restore_session(&checkpoint).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), checkpoint);
    let mut forged: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    assert!(forged["values"]["query_boundary"].is_number());
    forged["values"]
        .as_object_mut()
        .unwrap()
        .remove("query_boundary");
    assert!(model
        .restore_session(&serde_json::to_vec(&forged).unwrap())
        .is_err());
    forged["values"]["query_boundary"] = serde_json::json!(u64::MAX);
    assert!(model
        .restore_session(&serde_json::to_vec(&forged).unwrap())
        .is_err());
    println!("actual competing typed roles: four committed values correct; allocations=0 bytes=0; boundary restore and rejection checks PASS");
}

#[test]
#[ignore = "requires R4_INDEPENDENT_MODEL retained operand-provenance artifact"]
fn native_independent_artifact_is_allocation_free() {
    use uor_r4_core::native_geometric::Model;
    let model =
        Model::from_bytes(&std::fs::read(std::env::var("R4_INDEPENDENT_MODEL").unwrap()).unwrap())
            .unwrap();
    let prompts = [
        "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
        "User: mira has 5 coins. neri has 9 coins.\nUser: What is the sum of mira's and neri's coins?\nAssistant:",
        "User: Repeat the total for mira and neri.\nAssistant:",
    ].map(|p|model.encode(p).unwrap());
    let expected = ["17.\n", "14.\n", "14.\n"];
    let mut outputs = [[0; 32]; 3];
    let mut lengths = [0; 3];
    let mut ended = [false; 3];
    let mut s = model.session(Control::Full).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    let result = (|| {
        s.observe(&model, BOS)?;
        for (i, prompt) in prompts.iter().enumerate() {
            for &t in prompt {
                s.observe(&model, t)?;
            }
            s.begin_response(&model)?;
            for _ in 0..32 {
                let token = s.predict(&model)?.token;
                s.observe(&model, token)?;
                if token == EOS {
                    ended[i] = true;
                    break;
                }
                outputs[i][lengths[i]] = token;
                lengths[i] += 1;
            }
            s.end_response(&model)?;
        }
        Ok::<_, uor_r4_core::native_geometric::Error>(())
    })();
    MEASURING.with(|v| v.set(false));
    result.unwrap();
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    for i in 0..3 {
        assert!(ended[i]);
        assert_eq!(
            model.decode(&outputs[i][..lengths[i]]).unwrap(),
            expected[i].as_bytes()
        );
    }
    let checkpoint = s.checkpoint().unwrap();
    assert_eq!(
        model
            .restore_session(&checkpoint)
            .unwrap()
            .checkpoint()
            .unwrap(),
        checkpoint
    );
    let mut wire: serde_json::Value = serde_json::from_slice(&model.to_bytes().unwrap()).unwrap();
    if wire.get("typed_literals").is_some() {
        let parent_bytes = std::fs::read(std::env::var("R4_LITERAL_PARENT").unwrap()).unwrap();
        let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
        assert_eq!(
            wire["typed_literals"]["router"]["parent_artifact"],
            parent["artifact_cid"]
        );
        let mut protected = wire.clone();
        // A source-only refinement retains the exact earlier router as its
        // provenance witness; restore it before checking the literal parent.
        if let Some(witness) = protected
            .as_object_mut()
            .unwrap()
            .remove("source_routing_refinement")
        {
            protected["source_routing"] = witness["previous"].clone();
        }
        protected.as_object_mut().unwrap().remove("typed_literals");
        protected
            .as_object_mut()
            .unwrap()
            .remove("no_read_completion");
        for key in ["artifact_cid", "uor_model_address"] {
            protected.as_object_mut().unwrap().remove(key);
            parent.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(
            protected, parent,
            "every inherited model field must remain unchanged"
        );
        let mut invalid = wire.clone();
        invalid["typed_literals"]["literal_answers"] = serde_json::json!(false);
        assert!(Model::from_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
        println!("literal admission: entire parent unchanged; literal flag mutation rejected");
    }
    wire["typed_roles"]
        .as_object_mut()
        .unwrap()
        .remove("operand_provenance");
    assert!(Model::from_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    println!("actual independent sums and named Copy: all bytes and EOS correct; allocations=0 bytes=0; checkpoint roundtrip and unbound flag rejection PASS");
}

#[test]
#[ignore = "requires the explicitly supplied accepted NoRead completion artifact"]
fn native_no_read_artifact_is_allocation_free() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_NO_READ_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut outer: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(outer
        .as_object_mut()
        .unwrap()
        .remove("no_read_completion")
        .is_some());
    let mut parent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("R4_NO_READ_PARENT").unwrap()).unwrap(),
    )
    .unwrap();
    for doc in [&mut outer, &mut parent] {
        for k in ["artifact_cid", "uor_model_address"] {
            doc.as_object_mut().unwrap().remove(k);
        }
    }
    assert_eq!(outer, parent, "every parent parameter must be unchanged");
    let prompt=model.encode("User: leni has 17 coins. varo has -5 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:").unwrap();
    let mut s = model.session(Control::Full).unwrap();
    let mut out = [EOS; 32];
    let mut n = 0;
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    s.observe(&model, BOS).unwrap();
    for t in prompt {
        s.observe(&model, t).unwrap();
    }
    s.begin_response(&model).unwrap();
    for slot in &mut out {
        let p = s.predict(&model).unwrap();
        *slot = p.token;
        n += 1;
        s.observe(&model, p.token).unwrap();
        if p.token == EOS {
            break;
        }
    }
    MEASURING.with(|v| v.set(false));
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    assert_eq!(out[n - 1], EOS);
    assert_eq!(model.decode(&out[..n]).unwrap(), b" Unknown.\n");
    let checkpoint = s.checkpoint().unwrap();
    assert_eq!(
        model
            .restore_session(&checkpoint)
            .unwrap()
            .checkpoint()
            .unwrap(),
        checkpoint
    );
    println!("actual NoRead answer: exact bytes and EOS; allocations=0 bytes=0; complete parent equality and checkpoint roundtrip PASS");
}

/// Exercises the fitted replacement router itself, including the observation
/// boundary that turns a transient source choice into a committed copy/NoRead.
#[test]
#[ignore = "requires R4_SOURCE_NO_READ_MODEL and R4_SOURCE_NO_READ_PARENT artifacts"]
fn native_source_no_read_artifact_is_allocation_free_and_causal() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_NO_READ_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("R4_SOURCE_NO_READ_PARENT").unwrap()).unwrap(),
    )
    .unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("source_routing_refinement")
        .expect("actual replacement-router provenance");
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    assert_eq!(witness["previous"], parent["source_routing"]);
    wire["source_routing"] = witness["previous"].clone();
    for document in [&mut wire, &mut parent] {
        for key in ["artifact_cid", "uor_model_address"] {
            document.as_object_mut().unwrap().remove(key);
        }
    }
    assert_eq!(
        wire, parent,
        "all fields outside the replaced router are unchanged"
    );

    let cases = [
        (
            "User: varo has -5 coins. leni has 17 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:",
            b" Unknown.\n".as_slice(),
            false,
        ),
        (
            "User: varo has stones. leni has coins.\nUser: What does leni have?\nAssistant:",
            b" coins.\n".as_slice(),
            true,
        ),
    ];
    for (prompt, expected, supported) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = (|| {
            session.observe(&model, BOS)?;
            for &token in &tokens {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        ingest.unwrap();
        let boundary = session.checkpoint().unwrap();
        let before: serde_json::Value = serde_json::from_slice(&boundary).unwrap();
        assert!(before["word_copy"]["read_commit"].is_null());
        let commits = session.work.word_copy.selector.commits;

        MEASURING.with(|v| v.set(true));
        let first = session.predict(&model);
        let selected = session.word_copy_decision();
        let repeated = session.predict(&model);
        let repeated_choice = session.word_copy_decision();
        MEASURING.with(|v| v.set(false));
        let first = first.unwrap();
        assert_eq!(first, repeated.unwrap());
        let selected = selected.expect("joint source or NoRead selection exercised");
        assert_eq!(Some(selected), repeated_choice);
        assert_eq!(session.work.word_copy.selector.commits, commits);
        assert_eq!(selected.action == WordCopyAction::NoRead, !supported);
        if supported {
            assert!(matches!(
                selected.action,
                WordCopyAction::Read | WordCopyAction::Prepare
            ));
        }
        let transient: serde_json::Value =
            serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
        assert_eq!(
            transient["word_copy"], before["word_copy"],
            "prediction cannot commit a source"
        );

        MEASURING.with(|v| v.set(true));
        let observed = session.observe(&model, first.token);
        MEASURING.with(|v| v.set(false));
        observed.unwrap();
        assert_eq!(session.work.word_copy.selector.commits, commits + 1);
        let committed = session.checkpoint().unwrap();
        let state: serde_json::Value = serde_json::from_slice(&committed).unwrap();
        let read = &state["word_copy"]["read_commit"];
        assert!(read.is_object());
        assert_eq!(read["token"], serde_json::json!(first.token));
        assert_eq!(read["source_end"], serde_json::json!(selected.source_end));
        assert_eq!(
            read["source_byte_end"],
            serde_json::json!(selected.source_byte_end)
        );
        if supported {
            assert_eq!(read["source"], serde_json::json!(selected.word_index));
        } else {
            assert!(read["source"].is_null());
        }
        assert_eq!(
            model
                .restore_session(&committed)
                .unwrap()
                .checkpoint()
                .unwrap(),
            committed
        );

        let mut output = [EOS; 32];
        output[0] = first.token;
        let mut length = 1;
        MEASURING.with(|v| v.set(true));
        let generated = (|| {
            while output[length - 1] != EOS && length < output.len() {
                let token = session.predict(&model)?.token;
                session.observe(&model, token)?;
                output[length] = token;
                length += 1;
            }
            session.end_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        generated.unwrap();
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        assert_eq!(output[length - 1], EOS, "complete answer must terminate");
        assert_eq!(model.decode(&output[..length]).unwrap(), expected);

        // Restore an uncommitted boundary and observe a different legal byte.
        // Checkpoint parsing/serialization are deliberately outside the census.
        let mut mismatch = model.restore_session(&boundary).unwrap();
        let predicted = mismatch.predict(&model).unwrap();
        let mismatch_commits = mismatch.work.word_copy.selector.commits;
        let other = if predicted.token == u32::from(b'x') + 2 {
            u32::from(b'y') + 2
        } else {
            u32::from(b'x') + 2
        };
        mismatch.observe(&model, other).unwrap();
        assert_eq!(mismatch.work.word_copy.selector.commits, mismatch_commits);
        let mismatch_state: serde_json::Value =
            serde_json::from_slice(&mismatch.checkpoint().unwrap()).unwrap();
        assert!(mismatch_state["word_copy"]["read_commit"].is_null());
    }
    println!("actual source/NoRead: unsupported and supported complete answers; allocations=0 bytes=0; transient/matching/mismatched commit, checkpoint restore and full parent lineage PASS");
}

#[test]
#[ignore = "requires R4_JOINT_ADMISSION_MODEL and R4_JOINT_ADMISSION_PARENT"]
fn native_joint_admission_preserves_parent_and_executes_only_admitted_payloads() {
    use std::time::Instant;
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_JOINT_ADMISSION_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_JOINT_ADMISSION_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let parent_model = Model::from_bytes(&parent_bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let gate = wire
        .as_object_mut()
        .unwrap()
        .remove("joint_admission")
        .unwrap();
    assert_eq!(gate["router"]["parent_artifact"], parent["artifact_cid"]);
    for w in [&mut wire, &mut parent] {
        w.as_object_mut().unwrap().remove("artifact_cid");
        w.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(wire, parent, "complete inherited model equality");
    for field in 0..4 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match field {
            0 => bad["joint_admission"]["router"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["joint_admission"]["router"]["codes"][0]["roots"][0] = 120.into(),
            2 => bad["joint_admission"]["router"]["config"]["role_context_only"] = true.into(),
            _ => bad["typed_literals"]["router"]["biases"][0] = 32.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: cyra has 13 coins. cyra lives in Paris.\nUser: Where is cyra?\nAssistant:"," Paris.\n",false),
        ("User: ada has 13 coins. other has 7 coins. ada lives in Rome.\nUser: How many coins does ada have?\nAssistant:","13.\n",true),
        ("User: varo has -5 coins. leni has 17 coins. tavi has 301 coins.\nUser: Where is the location of leni?\nAssistant:"," Unknown.\n",false),
    ];
    let mut times = Vec::with_capacity(96);
    let mut total_rejections = 0;
    for (prompt, expected, numeric) in cases {
        assert_eq!(
            model
                .generate(prompt, 32, Control::JointAdmissionDisabled)
                .unwrap()
                .text,
            parent_model
                .generate(prompt, 32, Control::Full)
                .unwrap()
                .text
        );
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = (|| {
            session.observe(&model, BOS)?;
            for &t in &tokens {
                session.observe(&model, t)?;
            }
            session.begin_response(&model)
        })();
        MEASURING.with(|v| v.set(false));
        ingest.unwrap();
        let boundary = session.checkpoint().unwrap();
        MEASURING.with(|v| v.set(true));
        let first = session.predict(&model);
        let repeated = session.predict(&model);
        MEASURING.with(|v| v.set(false));
        assert_eq!(first.unwrap(), repeated.unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        assert_eq!(session.work.values.emission_commits, 0);
        if !numeric {
            assert!(session.value_decision().is_none());
            assert_eq!(
                session.work.values.operator_executions, 0,
                "declined arithmetic was never executed"
            );
        } else {
            assert!(session.value_decision().is_some());
        }
        total_rejections += session.work.values.admission_rejections;
        // A different observation may not commit the predicted numeric value or source.
        let mut mismatch = model.restore_session(&boundary).unwrap();
        let p = mismatch.predict(&model).unwrap();
        let wrong = if p.token == u32::from(b'x') + 2 {
            u32::from(b'y') + 2
        } else {
            u32::from(b'x') + 2
        };
        mismatch.observe(&model, wrong).unwrap();
        assert_eq!(mismatch.work.values.derived_writes, 0);
        assert_eq!(mismatch.work.word_copy.selector.commits, 0);
        let mut output = [EOS; 32];
        let mut length = 0;
        loop {
            MEASURING.with(|v| v.set(true));
            let started = Instant::now();
            let step = (|| {
                let t = session.predict(&model)?.token;
                session.observe(&model, t)?;
                Ok::<_, uor_r4_core::native_geometric::Error>(t)
            })();
            let ns = started.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            let t = step.unwrap();
            times.push(ns);
            output[length] = t;
            length += 1;
            if length == 1 {
                let checkpoint = session.checkpoint().unwrap();
                assert_eq!(
                    model
                        .restore_session(&checkpoint)
                        .unwrap()
                        .checkpoint()
                        .unwrap(),
                    checkpoint
                );
                if numeric {
                    assert_eq!(session.work.values.derived_writes, 1);
                }
            }
            if t == EOS || length == output.len() {
                break;
            }
        }
        assert_eq!(output[length - 1], EOS);
        assert_eq!(
            model.decode(&output[..length]).unwrap(),
            expected.as_bytes()
        );
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    assert!(
        total_rejections > 0,
        "actual learned declining path executed"
    );
    times.sort_unstable();
    println!("joint admission: exact parent, valid/invalid load, numeric and word/NoRead outputs, observation-only commitment, checkpoint and zero allocations PASS; warm predict+observe samples={}, median_ns={}, max_ns={} (encoding/loading/ingestion/checkpoints excluded)",times.len(),times[times.len()/2],times.last().unwrap());
}

#[test]
#[ignore = "requires R4_LITERAL_REFINEMENT_MODEL and R4_LITERAL_REFINEMENT_PARENT"]
fn native_literal_refinement_preserves_parent_and_binds_operands() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_LITERAL_REFINEMENT_MODEL").unwrap()).unwrap();
    let parent_bytes =
        std::fs::read(std::env::var("R4_LITERAL_REFINEMENT_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let old = Model::from_bytes(&parent_bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("literal_routing_refinement")
        .unwrap();
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    wire["typed_literals"]["router"] = witness["previous"].clone();
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "all other parameters, dictionary and parent lineage are fixed"
    );
    for k in 0..3 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match k {
            0 => bad["literal_routing_refinement"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["typed_literals"]["router"]["codes"][0]["roots"][0] = 120.into(),
            _ => bad["joint_admission"]["router"]["biases"][0] = 1000.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: adara lives in Rome. adara has 13 coins. cyris has 7 coins.\nUser: How many coins does adara have?\nAssistant:","13.\n",true),
        ("User: cyris has 7 coins. adara has 13 coins. adara lives in Rome.\nUser: How many coins does cyris have?\nAssistant:","7.\n",true),
        ("User: cyra has 13 coins. cyra lives in Paris.\nUser: Where is cyra?\nAssistant:"," Paris.\n",false),
    ];
    let mut times = [0_u128; 96];
    let mut time_len = 0;
    for (prompt, expected, numeric) in cases {
        assert_eq!(
            model
                .generate(prompt, 32, Control::LiteralRefinementDisabled)
                .unwrap()
                .text,
            old.generate(prompt, 32, Control::Full).unwrap().text
        );
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        MEASURING.with(|v| v.set(false));
        let checkpoint = session.checkpoint().unwrap();
        let mut wrong = model.restore_session(&checkpoint).unwrap();
        let p = wrong.predict(&model).unwrap();
        wrong
            .observe(&model, if p.token == 2 { 3 } else { 2 })
            .unwrap();
        assert_eq!(wrong.work.values.derived_writes, 0);
        let mut out = [EOS; 32];
        let mut len = 0;
        MEASURING.with(|v| v.set(true));
        loop {
            let started = std::time::Instant::now();
            let t = session.predict(&model).unwrap().token;
            session.observe(&model, t).unwrap();
            times[time_len] = started.elapsed().as_nanos();
            time_len += 1;
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        MEASURING.with(|v| v.set(false));
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, u64::from(numeric));
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
    }
    times[..time_len].sort_unstable();
    println!("warm predict+observe samples={}, median_ns={}, max_ns={} (loading/encoding/ingestion/checkpoints excluded)",time_len,times[time_len/2],times[time_len-1]);
    println!("literal refinement: exact parent restoration, malformed-artifact rejection, disabled-parent behavior, changed-order operands, source preservation, observation-only writes, checkpoints and zero allocations PASS");
}

#[test]
#[ignore = "requires R4_SOURCE_CONTEXT_MODEL and R4_SOURCE_CONTEXT_PARENT"]
fn native_source_context_retains_owner_and_preserves_causal_copy() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_CONTEXT_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_SOURCE_CONTEXT_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let witness = wire
        .as_object_mut()
        .unwrap()
        .remove("source_context")
        .unwrap();
    assert_eq!(witness["parent_artifact"], parent["artifact_cid"]);
    wire["source_routing"] = witness["previous"].clone();
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "all other parameters, dictionary and parent lineage are fixed"
    );
    for k in 0..3 {
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match k {
            0 => bad["source_context"]["parent_artifact"] = "wrong-parent".into(),
            1 => bad["source_routing"]["codes"][0]["roots"][0] = 120.into(),
            _ => bad["joint_admission"]["router"]["biases"][0] = 1000.into(),
        }
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let cases = [
        ("User: ada lives in Rome. ada has 13 coins. other has 7 coins.\nUser: Where is the location of ada?\nAssistant:"," Rome.\n",false),
        ("User: cyra lives in Rome. ada has 13 coins. other has 7 coins.\nUser: Where is the location of ada?\nAssistant:"," Unknown.\n",false),
        ("User: adara lives in Rome. adara has 13 coins. cyris has 7 coins.\nUser: How many coins does adara have?\nAssistant:","13.\n",true),
    ];
    let mut times = [0_u128; 96];
    let mut time_len = 0;
    for (prompt, expected, numeric) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.work.values.derived_writes, 0);
        MEASURING.with(|v| v.set(false));
        let checkpoint = session.checkpoint().unwrap();
        let mut wrong = model.restore_session(&checkpoint).unwrap();
        let p = wrong.predict(&model).unwrap();
        wrong
            .observe(&model, if p.token == 2 { 3 } else { 2 })
            .unwrap();
        assert_eq!(wrong.work.values.derived_writes, 0);
        let mut out = [EOS; 32];
        let mut len = 0;
        MEASURING.with(|v| v.set(true));
        loop {
            let started = std::time::Instant::now();
            let t = session.predict(&model).unwrap().token;
            session.observe(&model, t).unwrap();
            times[time_len] = started.elapsed().as_nanos();
            time_len += 1;
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        MEASURING.with(|v| v.set(false));
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, u64::from(numeric));
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
    }
    times[..time_len].sort_unstable();
    println!("warm predict+observe samples={}, median_ns={}, max_ns={} (loading/encoding/ingestion/checkpoints excluded)",time_len,times[time_len/2],times[time_len-1]);
    println!("source context: exact parent restoration, malformed-artifact rejection, evicted-owner contrast, numeric binding, source preservation, observation-only writes, checkpoints and zero allocations PASS");
}

#[test]
#[ignore = "requires R4_SOURCE_SPAN_MODEL and R4_SOURCE_SPAN_PARENT"]
fn native_source_span_preserves_commit_checkpoints_and_zero_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_SOURCE_SPAN_MODEL").unwrap()).unwrap();
    let parent_bytes = std::fs::read(std::env::var("R4_SOURCE_SPAN_PARENT").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let contextual = wire["source_span_context"].is_array();
    wire.as_object_mut().unwrap().remove("source_span_context");
    let block = wire.as_object_mut().unwrap().remove("source_span").unwrap();
    assert_eq!(block["parent_artifact"], parent["artifact_cid"]);
    for j in [&mut wire, &mut parent] {
        j.as_object_mut().unwrap().remove("artifact_cid");
        j.as_object_mut().unwrap().remove("uor_model_address");
    }
    assert_eq!(
        wire, parent,
        "the complete selector and parent parameters are unchanged"
    );
    let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    invalid["source_span"]["codes"][0]["roots"][0] = 120.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
    let mut times = [0_u128; 96];
    let mut samples = 0;
    let mut cases = vec![
        (
            "User: ada lives in New York.\nUser: Where is ada?\nAssistant:",
            " New York.\n",
            1,
        ),
        (
            "User: ada lives in Rio de Janeiro.\nUser: Where is ada?\nAssistant:",
            " Rio de Janeiro.\n",
            2,
        ),
    ];
    if contextual {
        cases.push((
            "Record: Dover holds cyra. Where is cyra? Answer:",
            " Dover.\n",
            0,
        ));
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"] = false.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"][0]["prime"] = 0.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["source_span_context"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (prompt, expected, extra) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        session.observe(&model, BOS).unwrap();
        for t in tokens {
            session.observe(&model, t).unwrap();
        }
        session.begin_response(&model).unwrap();
        let first = session.predict(&model).unwrap();
        assert_eq!(first, session.predict(&model).unwrap());
        assert_eq!(session.word_copy_decision().unwrap().span_words, extra);
        assert_eq!(session.work.word_copy.selector.commits, 0);
        MEASURING.with(|v| v.set(false));
        let mut out = [EOS; 32];
        let mut len = 0;
        loop {
            // Pending predictions are omitted and reconstructed by checkpoints.
            let checkpoint = session.checkpoint().unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let replay_prediction = restored.predict(&model).unwrap();
            if len == 5 {
                let mut bad: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
                bad["word_copy"]["span_words"] = 15.into();
                assert!(model
                    .restore_session(&serde_json::to_vec(&bad).unwrap())
                    .is_err());
                let commits = restored.work.word_copy.selector.commits;
                let p = restored.predict(&model).unwrap();
                restored
                    .observe(&model, if p.token == 2 { 3 } else { 2 })
                    .unwrap();
                assert_eq!(restored.work.word_copy.selector.commits, commits);
                assert!(restored.work.word_copy.selector.mismatches > 0);
            }
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let prediction = session.predict(&model).unwrap();
            let t = prediction.token;
            session.observe(&model, t).unwrap();
            let elapsed = start.elapsed().as_nanos();
            // The first step was predicted above to inspect its pending source.
            // Subsequent steps start without a pending prediction.
            if len != 0 {
                times[samples] = elapsed;
                samples += 1;
            }
            MEASURING.with(|v| v.set(false));
            assert_eq!(prediction, replay_prediction);
            out[len] = t;
            len += 1;
            if t == EOS || len == 32 {
                break;
            }
        }
        assert_eq!(out[len - 1], EOS);
        assert_eq!(model.decode(&out[..len]).unwrap(), expected.as_bytes());
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        let checkpoint = session.checkpoint().unwrap();
        assert_eq!(
            model
                .restore_session(&checkpoint)
                .unwrap()
                .checkpoint()
                .unwrap(),
            checkpoint
        );
        if extra > 0 {
            assert!(
                model
                    .generate(prompt, 32, Control::SourceSpanDisabled)
                    .unwrap()
                    .text
                    .len()
                    < expected.len()
            );
        } else if contextual {
            assert_ne!(
                model
                    .generate(prompt, 32, Control::SourceSpanContextDisabled)
                    .unwrap()
                    .text,
                expected
            );
        }
    }
    times[..samples].sort_unstable();
    println!("span uncached warm predict+observe samples={samples} median_ns={} max_ns={} (loading, ingestion and checkpoints excluded)", times[samples/2], times[samples-1]);
    println!("parent restoration, malformed code rejection, source commitment, interrupted copy, every-byte checkpoint and zero allocations PASS");
}

#[test]
#[ignore = "requires the retained relation span artifact; charged model work"]
fn native_retained_relation_span_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let path = std::env::var("R4_RETAINED_SPAN_MODEL").unwrap();
    let model = Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, 0).unwrap();
    for token in model.encode("ada in New ").unwrap() {
        session.observe(&model, token).unwrap();
    }
    let checkpoint = session.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    assert!(wire["values"]["relations"]["pending"].is_object());
    assert_eq!(wire["values"]["relations"]["next_id"], 1);
    session = model.restore_session(&checkpoint).unwrap();
    let rest = format!("York. {}Where is ada? Answer:", "quiet sky. ".repeat(96));
    let tokens = model.encode(&rest).unwrap();
    ALLOCATIONS.with(|v| v.set(0));
    BYTES.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
    for token in tokens {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    MEASURING.with(|v| v.set(false));
    let saved = session.checkpoint().unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    bad["values"]["relations"]["records"][0]["span"]["len"] = 29.into();
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut out = [1; 32];
    let mut used = 0;
    let mut times = [0u128; 32];
    loop {
        let mut restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        let predicted = restored.predict(&model).unwrap();
        MEASURING.with(|v| v.set(true));
        let start = std::time::Instant::now();
        let actual = session.predict(&model).unwrap();
        session.observe(&model, actual.token).unwrap();
        times[used] = start.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        assert_eq!(actual, predicted);
        out[used] = actual.token;
        used += 1;
        if actual.token == 1 || used == 32 {
            break;
        }
    }
    assert_eq!(out[used - 1], 1);
    assert_eq!(model.decode(&out[..used]).unwrap(), b" New York.\n");
    assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    times[..used].sort_unstable();
    println!("retained span pending/atomic write, malformed extent, eviction, each-step checkpoint and zero allocation PASS; uncached predict+observe samples={used} median_ns={} max_ns={} (includes initial selection; excludes load/encoding/ingestion/checkpoints)",times[used/2],times[used-1]);
}

#[test]
#[ignore = "requires the reverse relation span artifact; charged model work"]
fn native_reverse_relation_span_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let bytes = std::fs::read(std::env::var("R4_REVERSE_SPAN_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let mut wrong: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wrong["relation_reverse_spans"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&wrong).unwrap()).is_err());
    let mut samples = Vec::new();
    for padding in [String::new(), "quiet sky. ".repeat(96)] {
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, 0).unwrap();
        let prompt = format!("Orin Grove holds nelra. {padding}Where is nelra? Answer:");
        let tokens = model.encode(&prompt).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        for token in tokens {
            s.observe(&model, token).unwrap();
        }
        s.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        let checkpoint = s.checkpoint().unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
        let record = &wire["values"]["relations"]["records"][0];
        assert!(record["span"]["start"].is_object());
        assert_eq!(record["span"]["terminal"], record["value"]);
        let mut malformed = wire.clone();
        malformed["values"]["relations"]["records"][0]["span"]["start"]["byte_end"] = 999999.into();
        assert!(model
            .restore_session(&serde_json::to_vec(&malformed).unwrap())
            .is_err());
        let mut output = [1; 32];
        let mut used = 0;
        loop {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            let expected = restored.predict(&model).unwrap();
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = s.predict(&model).unwrap();
            s.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            samples.push(elapsed);
            assert_eq!(actual, expected);
            output[used] = actual.token;
            used += 1;
            if actual.token == 1 || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], 1);
        assert_eq!(model.decode(&output[..used]).unwrap(), b" Orin Grove.\n");
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    samples.sort_unstable();
    println!("reverse span parent rejection, preserved terminal anchor, malformed start rejection, short/evicted reads, per-step checkpoints and zero allocations PASS; samples={} median_ns={} max_ns={} (uncached predict+observe includes initial selection; excludes load/encoding/ingestion/checkpoints)",samples.len(),samples[samples.len()/2],samples[samples.len()-1]);
}

#[test]
#[ignore = "requires learned relation-start artifact; charged model execution"]
fn native_relation_start_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Control, Model};
    let bytes = std::fs::read(std::env::var("R4_RELATION_START_MODEL").unwrap()).unwrap();
    let loading = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = loading.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut bad = wire.clone();
    bad["relation_start"]["codes"][0]["roots"][0] = 120.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_start"]["parent_artifact"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let contextual = wire.get("relation_start_context").is_some();
    if contextual {
        let mut bad = wire.clone();
        bad["relation_start_context"][0]["prime"] = 0.into();
        assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let control = model.without_relation_start_context().unwrap();
        let restored = Model::from_bytes(&control.to_bytes().unwrap()).unwrap();
        assert_eq!(control.artifact_cid(), restored.artifact_cid());
    }
    let mut times = Vec::new();
    let mut input_times = Vec::new();
    let mut cases = vec![
        (
            "Notes say Amber Meadow holds telra.",
            "telra",
            " Amber Meadow.\n",
            false,
        ),
        (
            "Notes say Amber Meadow holds telra.",
            "telra",
            " Amber Meadow.\n",
            true,
        ),
        ("Notes say Amber holds telra.", "telra", " Amber.\n", false),
    ];
    if contextual {
        cases.extend([
            (
                "notes say fine sand holds calvi.",
                "calvi",
                " fine sand.\n",
                false,
            ),
            (
                "notes say fine sand holds calvi.",
                "calvi",
                " fine sand.\n",
                true,
            ),
        ]);
    }
    for (fact, owner, expected, long) in cases {
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, 0).unwrap();
        let padding = if long {
            "quiet sky. ".repeat(96)
        } else {
            String::new()
        };
        let encoding = std::time::Instant::now();
        let tokens = model
            .encode(&format!("{fact} {padding}Where is {owner}? Answer:"))
            .unwrap();
        let encode_ns = encoding.elapsed().as_nanos();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = std::time::Instant::now();
        for token in tokens {
            s.observe(&model, token).unwrap();
        }
        s.begin_response(&model).unwrap();
        let ingest_ns = ingest.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        input_times.push((long, encode_ns, ingest_ns));
        let mut output = [1; 32];
        let mut used = 0;
        loop {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            let predicted = restored.predict(&model).unwrap();
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = s.predict(&model).unwrap();
            s.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, predicted);
            times.push(elapsed);
            output[used] = actual.token;
            used += 1;
            if actual.token == 1 || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], 1);
        assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    times.sort_unstable();
    println!("relation-start load_ns={load_ns}; (evicted,encode_ns,ingest_begin_response_ns)={input_times:?}; ingestion includes start scoring, excludes emission/checkpoints; sampled wall times, no energy measurement");
    println!("relation-start model/root rejection, short/evicted/singleton outputs, per-step checkpoints and zero allocations PASS; samples={} median_ns={} max_ns={} (uncached predict+observe including initial selection; excludes loading/encoding/ingestion/checkpoints)",times.len(),times[times.len()/2],times[times.len()-1]);
}

#[test]
#[ignore = "requires refined writer artifact; charged model execution"]
fn native_writer_refinement_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_WRITER_REFINEMENT_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_ne!(
        wire["relation_writer"]["rows"],
        wire["relation_writer_refinement"]["previous"]["rows"]
    );
    let mut bad = wire.clone();
    bad["relation_writer_refinement"]["parent_artifact"] = "wrong-parent".into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_writer"]["dictionary"][0]["prime"] = 0.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = wire.clone();
    bad["relation_writer"]["admission"] =
        wire["relation_writer_refinement"]["previous"]["admission"].clone();
    assert!(Model::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut emission_ns = Vec::new();
    let mut inputs = Vec::new();
    for (fact, owner, expected, writes, evicted) in [
        (
            "notes say quiet harbor holds ranvi.",
            "ranvi",
            " quiet harbor.\n",
            1,
            false,
        ),
        (
            "notes say quiet harbor holds ranvi.",
            "ranvi",
            " quiet harbor.\n",
            1,
            true,
        ),
        ("notes say quiet harbor.", "ranvi", " Unknown.\n", 0, false),
        (
            "Report notes Cobalt Field holds vorin.",
            "vorin",
            " Cobalt Field.\n",
            1,
            false,
        ),
    ] {
        let padding = if evicted {
            "quiet sky. ".repeat(96)
        } else {
            String::new()
        };
        let prompt = format!("{fact} {padding}Where is {owner}? Answer:");
        let encoding = std::time::Instant::now();
        let tokens = model.encode(&prompt).unwrap();
        let encode_ns = encoding.elapsed().as_nanos();
        let mut session = model.session(Control::Full).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        let ingest = std::time::Instant::now();
        session.observe(&model, BOS).unwrap();
        for token in tokens {
            session.observe(&model, token).unwrap();
        }
        session.begin_response(&model).unwrap();
        let ingest_ns = ingest.elapsed().as_nanos();
        MEASURING.with(|v| v.set(false));
        assert_eq!(session.work.values.relations.record_writes, writes);
        inputs.push((evicted, encode_ns, ingest_ns));
        let mut output = [EOS; 32];
        let mut used = 0;
        loop {
            let mut restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            let expected_prediction = restored.predict(&model).unwrap();
            assert_eq!(expected_prediction, restored.predict(&model).unwrap());
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = session.predict(&model).unwrap();
            session.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, expected_prediction);
            emission_ns.push(elapsed);
            output[used] = actual.token;
            used += 1;
            if actual.token == EOS || used == 32 {
                break;
            }
        }
        assert_eq!(output[used - 1], EOS);
        assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.relations.record_writes, writes);
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
    }
    emission_ns.sort_unstable();
    println!("writer-refinement load_ns={load_ns}; (evicted,encode_ns,ingest_begin_response_ns)={inputs:?}; ingestion includes changed writer/cache and phrase selection; measured wall samples, no energy result");
    println!("writer-refinement parent/dictionary/stale-cache rejection, exact assertion/nonassertion writes and output, evicted reads, every-step checkpoints and zero allocations PASS; predict_observe samples={} median_ns={} max_ns={} (loading, encoding, ingestion and checkpoints excluded)",emission_ns.len(),emission_ns[emission_ns.len()/2],emission_ns.last().unwrap());
}

#[test]
#[ignore = "requires learned operation artifact; charged actual-model execution"]
fn native_operation_transition_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_OPERATION_TRANSITION_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["operation_transition"].is_object());
    let mut times = Vec::new();
    let mut positions = 0;
    for (a, b, extra) in [(13, 4, 3), (-19, 4, 7)] {
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        for (prompt,expected) in [
            (format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),format!("{}.\n",a+b)),
            (format!("User: There are {extra} extra coins. Add the extra coins to the original total. Again.\nAssistant:"),format!("{}.\n{}.\n",a+b+extra,a+b+extra+extra)),
        ] {
            let tokens=model.encode(&prompt).unwrap();
            ALLOCATIONS.with(|v|v.set(0));BYTES.with(|v|v.set(0));
            MEASURING.with(|v|v.set(true));
            for token in tokens {session.observe(&model,token).unwrap();}
            session.begin_response(&model).unwrap();
            MEASURING.with(|v|v.set(false));
            let mut output=[EOS;64];let mut used=0;
            loop {
                let mut restored=model.restore_session(&session.checkpoint().unwrap()).unwrap();
                let prediction=restored.predict(&model).unwrap();
                assert_eq!(restored.predict(&model).unwrap(),prediction);
                MEASURING.with(|v|v.set(true));
                let start=std::time::Instant::now();
                let actual=session.predict(&model).unwrap();
                session.observe(&model,actual.token).unwrap();
                let elapsed=start.elapsed().as_nanos();
                MEASURING.with(|v|v.set(false));
                assert_eq!(actual,prediction);times.push(elapsed);positions+=1;
                output[used]=actual.token;used+=1;
                if actual.token==EOS||used==64{break;}
            }
            assert_eq!(output[used-1],EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(),expected.as_bytes());
            assert_eq!((ALLOCATIONS.with(Cell::get),BYTES.with(Cell::get)),(0,0));
            session.end_response(&model).unwrap();
        }
    }
    times.sort_unstable();
    println!("actual learned operation artifact={}; load_ns={load_ns}; checkpoint_positions={positions}; allocations=0 bytes=0 for ingestion/begin/predict/observe; predict_observe median_ns={} max_ns={} (load/encode/session/checkpoint/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

#[test]
#[ignore = "requires lexical emission artifact; charged actual-model development cases"]
fn native_lexical_emission_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_LEXICAL_EMISSION_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["lexical_emission"].is_object());
    let mut times = Vec::new();
    let mut positions = 0;
    let mut cases = 0;
    for (a, b) in [(13, 4), (-19, 4)] {
        let mut requests = vec![
            ("sentence. ", "", format!("{} is {b} plus {a}.\n", a + b)),
            ("Rust. ", "", format!("{} == {b} + {a}\n", a + b)),
        ];
        if wire["instruction_binding"].is_object() {
            requests.extend([
                (
                    "In a sentence, ",
                    "",
                    format!("{} is {b} plus {a}.\n", a + b),
                ),
                (
                    "",
                    " Write a Rust equality.",
                    format!("{} == {b} + {a}\n", a + b),
                ),
            ]);
        }
        for (request, after, expected) in requests {
            cases += 1;
            // These are declared development examples, not the sealed fresh panel.
            let prompt = format!("User: suri has {a} coins. orin has {b} coins.\nUser: {request}What is the sum of suri's and orin's coins?{after}\nAssistant:");
            let tokens = model.encode(&prompt).unwrap();
            let mut session = model.session(Control::Full).unwrap();
            ALLOCATIONS.with(|v| v.set(0));
            BYTES.with(|v| v.set(0));
            MEASURING.with(|v| v.set(true));
            session.observe(&model, BOS).unwrap();
            for &token in &tokens {
                session.observe(&model, token).unwrap();
            }
            session.begin_response(&model).unwrap();
            MEASURING.with(|v| v.set(false));
            let mut output = [EOS; 64];
            let mut used = 0;
            loop {
                let mut restored = model
                    .restore_session(&session.checkpoint().unwrap())
                    .unwrap();
                let prediction = restored.predict(&model).unwrap();
                let completion = restored.completion_decision();
                let value = restored.value_decision();
                assert_eq!(restored.predict(&model).unwrap(), prediction);
                assert_eq!(restored.completion_decision(), completion);
                assert_eq!(restored.value_decision(), value);
                MEASURING.with(|v| v.set(true));
                let start = std::time::Instant::now();
                let actual = session.predict(&model).unwrap();
                let actual_completion = session.completion_decision();
                let actual_value = session.value_decision();
                session.observe(&model, actual.token).unwrap();
                let elapsed = start.elapsed().as_nanos();
                MEASURING.with(|v| v.set(false));
                assert_eq!(actual, prediction);
                assert_eq!(actual_completion, completion);
                assert_eq!(actual_value, value);
                restored.observe(&model, prediction.token).unwrap();
                // Byte equality alone cannot distinguish equal-valued record IDs.
                // Compare committed read and typed state after observation too.
                let actual_state: serde_json::Value =
                    serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
                let restored_state: serde_json::Value =
                    serde_json::from_slice(&restored.checkpoint().unwrap()).unwrap();
                assert_eq!(actual_state["completion"], restored_state["completion"]);
                assert_eq!(actual_state["values"], restored_state["values"]);
                times.push(elapsed);
                positions += 1;
                output[used] = actual.token;
                used += 1;
                if actual.token == EOS || used == output.len() {
                    break;
                }
            }
            assert_eq!(output[used - 1], EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
            assert_eq!(session.work.values.derived_writes, 1);
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        }
    }
    times.sort_unstable();
    println!("actual learned lexical artifact={}; load_ns={load_ns}; development_cases={cases}; checkpoint_positions={positions}; exactly one derived write per response; allocations=0 bytes=0 for ingestion/begin/predict/observe; predict_observe median_ns={} max_ns={} (load/encode/session/checkpoint/report excluded; no energy claim)", model.artifact_cid(), times[times.len() / 2], times[times.len() - 1]);
}

#[test]
#[ignore = "requires composed output artifact; charged actual-model development cases"]
fn native_composed_output_actual_checkpoint_and_allocation() {
    actual_composed_or_mixed_checkpoint_and_allocation("composed");
}

#[test]
#[ignore = "requires R4_MIXED_OPERATORS_MODEL; charged actual-model development cases"]
fn native_mixed_operators_actual_checkpoint_and_allocation() {
    actual_composed_or_mixed_checkpoint_and_allocation("mixed");
}

#[test]
#[ignore = "requires R4_ACTION_EMISSION_MODEL; charged actual-model development cases"]
fn native_action_emission_actual_checkpoint_and_allocation() {
    actual_composed_or_mixed_checkpoint_and_allocation("action");
}

#[test]
#[ignore = "requires R4_COPY_ADD_MODEL; charged actual-model development cases"]
fn native_copy_add_actual_checkpoint_and_allocation() {
    actual_composed_or_mixed_checkpoint_and_allocation("copy_add");
}

fn actual_composed_or_mixed_checkpoint_and_allocation(scope: &str) {
    use uor_r4_core::native_geometric::{Model, ValueAction};
    let (artifact_env, witness) = match scope {
        "copy_add" => ("R4_COPY_ADD_MODEL", "shared_operator_refinement"),
        "action" => ("R4_ACTION_EMISSION_MODEL", "action_emission"),
        "mixed" => ("R4_MIXED_OPERATORS_MODEL", "mixed_operators"),
        "composed" => ("R4_COMPOSED_OUTPUT_MODEL", "composed_output"),
        _ => panic!("unknown authored allocation panel"),
    };
    let bytes = std::fs::read(std::env::var(artifact_env).unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire[witness].is_object());
    if scope == "action" {
        assert_eq!(wire[witness]["context_enabled"], true);
    }
    if scope == "copy_add" {
        assert_eq!(wire[witness]["continuation_fitted"], true,
            "Copy-Add behavior requires a completed continuation fit, not a staged binding checkpoint");
    }
    let history = "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
    let mut times = Vec::new();
    let mut positions = 0;
    // Query, targets, exact occurrence IDs and expected lexical reads are
    // prepared outside every measured region. These are authored checks,
    // never serving inputs other than the query itself.
    let formatted_reads: &[(u64, u64)] = &[(4, 3), (4, 2), (5, 3), (5, 4)];
    let no_reads: &[(u64, u64)] = &[];
    let latest_reads: &[(u64, u64)] = &[(4, 3), (4, 2), (5, 4)];
    let original_reads: &[(u64, u64)] = &[(4, 3), (4, 2), (5, 2)];
    let copy_original_reads: &[(u64, u64)] = &[(4, 2), (5, 3), (5, 4)];
    let copy_extra_reads: &[(u64, u64)] = &[(4, 3), (5, 2), (5, 4)];
    let cases = if scope == "copy_add" {
        vec![
            ("sentence_original_add_extra", "Copy the original total. Add the extra to the copied result. Explain in a sentence.",
             "17 is 17.\n20 is 3 plus 17.\n", ValueAction::Add, 20, [3, 4], copy_original_reads),
            ("sentence_extra_add_original", "Copy the extra. Add the original total to the copied result. Explain in a sentence.",
             "3 is 3.\n20 is 17 plus 3.\n", ValueAction::Add, 20, [2, 4], copy_extra_reads),
            ("rust_original_add_extra", "Copy the original total. Add the extra to the copied result. Write a Rust equality.",
             "17 == 17\n20 == 3 + 17\n", ValueAction::Add, 20, [3, 4], copy_original_reads),
            ("rust_extra_add_original", "Copy the extra. Add the original total to the copied result. Write a Rust equality.",
             "3 == 3\n20 == 17 + 3\n", ValueAction::Add, 20, [2, 4], copy_extra_reads),
        ]
    } else if scope == "action" {
        vec![
            (
                "sentence_copy_latest",
                " Copy the latest result. Explain in a sentence.",
                "20 is 3 plus 17.\n20 is 20.\n",
                ValueAction::Copy,
                20,
                [4, 4],
                latest_reads,
            ),
            (
                "sentence_copy_original",
                " Copy the original total. Explain in a sentence.",
                "20 is 3 plus 17.\n17 is 17.\n",
                ValueAction::Copy,
                17,
                [2, 2],
                original_reads,
            ),
            (
                "rust_copy_latest",
                " Copy the latest result. Write a Rust equality.",
                "20 == 3 + 17\n20 == 20\n",
                ValueAction::Copy,
                20,
                [4, 4],
                latest_reads,
            ),
            (
                "rust_copy_original",
                " Copy the original total. Write a Rust equality.",
                "20 == 3 + 17\n17 == 17\n",
                ValueAction::Copy,
                17,
                [2, 2],
                original_reads,
            ),
        ]
    } else if scope == "mixed" {
        vec![
            (
                "copy_latest",
                " Copy the latest result.",
                "20.\n20.\n",
                ValueAction::Copy,
                20,
                [4, 4],
                no_reads,
            ),
            (
                "copy_original",
                " Copy the original total.",
                "20.\n17.\n",
                ValueAction::Copy,
                17,
                [2, 2],
                no_reads,
            ),
        ]
    } else {
        vec![
            (
                "sentence",
                " Again. Explain in a sentence.",
                "20 is 3 plus 17.\n23 is 3 plus 20.\n",
                ValueAction::Add,
                23,
                [3, 4],
                formatted_reads,
            ),
            (
                "rust",
                " Again. Write a Rust equality.",
                "20 == 3 + 17\n23 == 3 + 20\n",
                ValueAction::Add,
                23,
                [3, 4],
                formatted_reads,
            ),
        ]
    };
    let case_count = cases.len();
    for (label, request, expected, second_action, second_value, second_operands, expected_reads) in
        cases
    {
        let case_positions_start = positions;
        let query = if scope == "copy_add" {
            format!("User: There are 3 extra coins. {request}\nAssistant:")
        } else {
            format!("User: There are 3 extra coins. Add the extra coins to the original total.{request}\nAssistant:")
        };
        let (first_action, first_value, first_operands) = if scope == "copy_add" {
            match second_operands {
                [3, 4] => (ValueAction::Copy, 17, [2, 2]),
                [2, 4] => (ValueAction::Copy, 3, [3, 3]),
                _ => panic!("unknown authored Copy-Add case"),
            }
        } else {
            (ValueAction::Add, 20, [3, 2])
        };
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        let mut turns = vec![
            (history, "17.\n", false, 2),
            (query.as_str(), expected, true, 0),
        ];
        if matches!(scope, "action" | "copy_add") {
            turns.push((history, "17.\n", false, 8));
        }
        for (prompt, expected, composed, independent_id) in turns {
            let tokens = model.encode(prompt).unwrap();
            ALLOCATIONS.with(|v| v.set(0));
            BYTES.with(|v| v.set(0));
            MEASURING.with(|v| v.set(true));
            for &token in &tokens {
                session.observe(&model, token).unwrap();
            }
            session.begin_response(&model).unwrap();
            MEASURING.with(|v| v.set(false));
            let initial: serde_json::Value =
                serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
            let before_writes = session.work.values.derived_writes;
            let mut output = [EOS; 96];
            let mut used = 0;
            let mut writes = Vec::new();
            let mut reads = Vec::new();
            let mut last_read_start = None;
            loop {
                let mut restored = model
                    .restore_session(&session.checkpoint().unwrap())
                    .unwrap();
                let prediction = restored.predict(&model).unwrap();
                let completion = restored.completion_decision();
                let value = restored.value_decision();
                assert_eq!(restored.predict(&model).unwrap(), prediction);
                assert_eq!(restored.completion_decision(), completion);
                assert_eq!(restored.value_decision(), value);
                MEASURING.with(|v| v.set(true));
                let start = std::time::Instant::now();
                let actual = session.predict(&model).unwrap();
                let actual_completion = session.completion_decision();
                let actual_value = session.value_decision();
                session.observe(&model, actual.token).unwrap();
                let elapsed = start.elapsed().as_nanos();
                MEASURING.with(|v| v.set(false));
                assert_eq!(actual, prediction);
                assert_eq!(actual_completion, completion);
                assert_eq!(actual_value, value);
                if let Some(write) = actual_value.filter(|v| v.cursor == 0) {
                    writes.push(write);
                }
                restored.observe(&model, prediction.token).unwrap();
                let state: serde_json::Value =
                    serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
                let restored_state: serde_json::Value =
                    serde_json::from_slice(&restored.checkpoint().unwrap()).unwrap();
                assert_eq!(state["completion"], restored_state["completion"]);
                assert_eq!(state["values"], restored_state["values"]);
                // Capture observed read starts, not the evaluator's expected
                // derivation. Equal-valued records cannot substitute identities.
                if let Some(read) = state["completion"]
                    .get("lexical_read")
                    .filter(|r| r.is_object())
                {
                    let at = read["start_at"].as_u64().unwrap();
                    if last_read_start != Some(at) {
                        assert_eq!(read["cursor"], 1);
                        reads.push((
                            state["completion"]["anchor"]["write_id"].as_u64().unwrap(),
                            read["record_id"].as_u64().unwrap(),
                        ));
                        last_read_start = Some(at);
                    }
                }
                if actual.token != EOS {
                    // Later operations read committed records without replacing
                    // the original captured source/query state.
                    assert_eq!(state["values"]["sources"], initial["values"]["sources"]);
                    assert_eq!(state["values"]["queries"], initial["values"]["queries"]);
                }
                times.push(elapsed);
                positions += 1;
                output[used] = actual.token;
                used += 1;
                if actual.token == EOS || used == output.len() {
                    break;
                }
            }
            assert_eq!(output[used - 1], EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(), expected.as_bytes());
            if composed {
                assert_eq!(session.work.values.derived_writes - before_writes, 2);
                assert_eq!(writes.len(), 2);
                for (write, action, id, value, operands) in [
                    (&writes[0], first_action, 4, first_value, first_operands),
                    (&writes[1], second_action, 5, second_value, second_operands),
                ] {
                    assert_eq!(write.action, action);
                    assert_eq!(write.write_id, id);
                    assert_eq!(write.value, value);
                    assert_eq!(write.operands.map(|r| r.id), operands);
                }
                assert_eq!(reads.as_slice(), expected_reads);
            } else {
                assert_eq!(session.work.values.derived_writes - before_writes, 1);
                assert_eq!(writes.len(), 1);
                assert_eq!(writes[0].write_id, independent_id);
                assert_eq!(writes[0].action, ValueAction::Add);
                assert_eq!(
                    writes[0].operands.map(|r| r.id),
                    [independent_id - 1, independent_id - 2]
                );
                assert_eq!(writes[0].value, 17);
                assert!(reads.is_empty());
            }
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
            session.end_response(&model).unwrap();
        }
        println!("actual {scope} case={label}; checkpoint_positions={}; includes actual history plus bounded two-operation response; exact actions/write IDs/operand IDs/lexical reads; allocations=0 bytes=0 for measured ingestion/begin/predict/observe", positions - case_positions_start);
    }
    times.sort_unstable();
    println!("actual {scope} artifact={}; load_ns={load_ns}; development_cases={case_count}; checkpoint_positions={positions}; two typed writes and exact occurrence/read checks per requested response; allocations=0 bytes=0 for ingestion/begin/predict/observe; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/end-response/checkpoint/JSON/report excluded; no energy claim)", model.artifact_cid(), times[times.len() / 2], times[times.len() - 1]);
}

/// Actual development artifact: source copying must finish before the shared
/// learned lexical selector emits the sentence suffix. Targets are assertions,
/// never observations supplied to the serving session.
#[test]
#[ignore = "requires R4_WORD_SENTENCE_MODEL; charged actual-model development cases"]
fn native_word_sentence_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    let bytes = std::fs::read(std::env::var("R4_WORD_SENTENCE_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        wire["word_emission"].is_object(),
        "requires the learned word-emission artifact"
    );
    let cases = [
        ("span_place", "User: tilva lives in Ash Court.\nUser: Where is tilva?\nExplain in a sentence. Assistant:",
         " Ash Court", " Ash Court is the place.\n"),
        ("span_stop", "User: tilva lives in Ash Court.\nUser: Where is tilva?\nExplain the stop in a sentence. Assistant:",
         " Ash Court", " Ash Court is the stop.\n"),
        ("word_place", "Record: velra in Lodov. Where is velra? Explain in a sentence. Answer:",
         " Lodov", " Lodov is the place.\n"),
    ];
    let mut times = Vec::new();
    let mut positions = 0usize;
    for (label, prompt, copied_prefix, target) in cases {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        MEASURING.with(|v| v.set(true));
        for token in tokens {
            session.observe(&model, token).unwrap();
        }
        session.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        let before_writes = session.work.values.derived_writes;
        let mut output = [EOS; 96];
        let mut used = 0usize;
        let mut copied_boundary: Option<serde_json::Value> = None;
        let mut suffix_positions = 0usize;
        let case_start = positions;
        loop {
            let checkpoint = session.checkpoint().unwrap();
            let before: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let predicted = restored.predict(&model).unwrap();
            let word = restored.word_copy_decision();
            let completion = restored.completion_decision();
            let value = restored.value_decision();
            let entry = restored.response_entry_decision();
            assert_eq!(restored.predict(&model).unwrap(), predicted);
            assert_eq!(restored.word_copy_decision(), word);
            assert_eq!(restored.completion_decision(), completion);
            assert_eq!(restored.value_decision(), value);
            assert_eq!(restored.response_entry_decision(), entry);
            let routing_before = session.work.word_copy.routing;
            let candidate_evaluations_before =
                session.work.word_copy.selector.candidate_evaluations;
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = session.predict(&model).unwrap();
            let actual_word = session.word_copy_decision();
            let actual_completion = session.completion_decision();
            let actual_value = session.value_decision();
            let actual_entry = session.response_entry_decision();
            session.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, predicted);
            assert_eq!(actual_word, word);
            assert_eq!(actual_completion, completion);
            assert_eq!(actual_value, value);
            assert_eq!(actual_entry, entry);
            assert!(
                actual_value.is_none(),
                "word suffix must not invent a numeric write"
            );
            restored.observe(&model, predicted.token).unwrap();
            let after: serde_json::Value =
                serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
            let restored_after: serde_json::Value =
                serde_json::from_slice(&restored.checkpoint().unwrap()).unwrap();
            for key in ["values", "word_copy", "response_entry", "completion"] {
                assert_eq!(after[key], restored_after[key], "{label}: committed {key}");
            }
            assert_eq!(after["completion"]["active"], false);
            assert!(after["completion"]["anchor"].is_null());
            assert!(after["completion"]["lexical_read"].is_null());
            if before["word_copy"]["progress"] == "complete" {
                if suffix_positions == 0 {
                    for field in ["source_end", "source_byte_end", "relation_id"] {
                        let mut changed = before.clone();
                        let old = changed["word_copy"]["read_commit"][field]
                            .as_u64()
                            .unwrap_or(0);
                        changed["word_copy"]["read_commit"][field] = serde_json::json!(old + 1);
                        assert!(
                            model
                                .restore_session(&serde_json::to_vec(&changed).unwrap())
                                .is_err(),
                            "{label}: changed committed {field} must be rejected"
                        );
                    }
                    let mut changed = before.clone();
                    let span = changed["word_copy"]["span_words"].as_u64().unwrap_or(0);
                    changed["word_copy"]["span_words"] = serde_json::json!(span + 1);
                    assert!(
                        model
                            .restore_session(&serde_json::to_vec(&changed).unwrap())
                            .is_err(),
                        "{label}: changed span extent must be rejected"
                    );
                    let mut mismatch = model.restore_session(&checkpoint).unwrap();
                    let proposal = mismatch.predict(&model).unwrap();
                    let pending_word = mismatch
                        .word_copy_decision()
                        .expect("selected suffix copy decision");
                    let pending_entry = mismatch
                        .response_entry_decision()
                        .expect("selected suffix entry decision");
                    assert_eq!(pending_word.token, proposal.token);
                    assert_eq!(pending_entry.token, proposal.token);
                    assert!(matches!(
                        pending_word.action,
                        WordCopyAction::Emit | WordCopyAction::Stop
                    ));
                    let copy_work = mismatch.work.word_copy.selector;
                    let entry_work = mismatch.work.response_entry;
                    let wrong = if proposal.token == 2 { 3 } else { 2 };
                    mismatch.observe(&model, wrong).unwrap();
                    // The source copy is already complete. ResponseEntry owns
                    // non-EOS lexical mismatches; WordCopy's mismatch counter
                    // counts aborted in-progress copies (or EOS/reset cases).
                    // Reject the pending suffix without fabricating a commit,
                    // retaining the exact source for the observed Base step.
                    assert_eq!(mismatch.work.word_copy.selector.commits, copy_work.commits);
                    assert_eq!(
                        mismatch.work.word_copy.selector.mismatches,
                        copy_work.mismatches
                    );
                    assert_eq!(mismatch.work.response_entry.commits, entry_work.commits);
                    assert_eq!(
                        mismatch.work.response_entry.mismatches,
                        entry_work.mismatches + 1
                    );
                    assert_eq!(
                        mismatch.work.response_entry.base_steps,
                        entry_work.base_steps + 1
                    );
                    assert!(mismatch.word_copy_decision().is_none());
                    assert!(mismatch.response_entry_decision().is_none());
                    let mismatched_state: serde_json::Value =
                        serde_json::from_slice(&mismatch.checkpoint().unwrap()).unwrap();
                    assert_eq!(mismatched_state["response_entry"]["last_action"], "base");
                    assert_eq!(mismatched_state["response_entry"]["last"], wrong);
                    assert_eq!(
                        mismatched_state["response_entry"]["steps"],
                        pending_entry.step + 1
                    );
                    assert_eq!(mismatched_state["word_copy"], before["word_copy"]);
                }
                let boundary = copied_boundary
                    .as_ref()
                    .expect("observed copied prefix before suffix");
                for key in ["origin", "read_commit", "span_words"] {
                    assert_eq!(
                        before["word_copy"][key], boundary[key],
                        "{label}: exact copied source {key}"
                    );
                }
                assert!(
                    matches!(
                        actual_word.map(|w| w.action),
                        Some(WordCopyAction::Emit | WordCopyAction::Stop)
                    ),
                    "completed word/span suffix remains an observed WordCopy lexical decision"
                );
                assert!(
                    session.work.word_copy.routing.code_reads > routing_before.code_reads,
                    "completed-copy suffix must execute learned geometric lexical codes"
                );
                assert!(
                    session.work.word_copy.selector.candidate_evaluations
                        > candidate_evaluations_before,
                    "completed-copy suffix must evaluate shared lexical candidates"
                );
                suffix_positions += 1;
            }
            output[used] = actual.token;
            used += 1;
            if copied_boundary.is_none() && after["word_copy"]["progress"] == "complete" {
                assert_eq!(
                    model.decode(&output[..used]).unwrap(),
                    copied_prefix.as_bytes(),
                    "the lexical bridge begins only after the actual copied word/span"
                );
                let source = &after["word_copy"];
                assert!(source["origin"].as_u64().is_some());
                assert!(source["read_commit"]["source"].as_u64().is_some());
                assert!(source["read_commit"]["source_end"]
                    .as_u64()
                    .is_some_and(|v| v > 0));
                assert!(source["read_commit"]["source_byte_end"]
                    .as_u64()
                    .is_some_and(|v| v > 0));
                let decision = actual_word.expect("final copied byte has an exact source decision");
                assert_eq!(source["origin"], decision.word_index);
                assert_eq!(source["read_commit"]["source_end"], decision.source_end);
                assert_eq!(
                    source["read_commit"]["source_byte_end"],
                    decision.source_byte_end
                );
                copied_boundary = Some(source.clone());
            }
            positions += 1;
            times.push(elapsed);
            if actual.token == EOS || used == output.len() {
                break;
            }
        }
        assert_eq!(output[used - 1], EOS, "{label}: bounded learned stop");
        assert_eq!(
            model.decode(&output[..used]).unwrap(),
            target.as_bytes(),
            "{label}: generated bytes"
        );
        assert!(copied_boundary.is_some());
        assert!(
            suffix_positions > 1,
            "must exercise learned suffix and EOS after completed copy"
        );
        assert_eq!(session.work.values.derived_writes, before_writes);
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        println!("actual word-sentence case={label}; checkpoint_positions={}; suffix_positions={suffix_positions}; exact completed source identity; no numeric derived writes; allocations=0 bytes=0 for measured ingestion/begin/predict/observe",positions-case_start);
    }
    times.sort_unstable();
    println!("actual word-sentence artifact={}; load_ns={load_ns}; development_cases=3; checkpoint_positions={positions}; allocations=0 bytes=0 for ingestion/begin/predict/observe; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

/// Actual writer context intervention. Compare committed relations and input
/// checkpoint causality; generated text below is reported, not a prose target.
#[test]
#[ignore = "requires R4_WRITER_LEXICAL_MODEL; charged actual-model development cases"]
fn native_writer_lexical_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::Model;
    fn state(s: &uor_r4_core::native_geometric::Session) -> serde_json::Value {
        serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
    }
    fn atom_text(atom: &serde_json::Value) -> String {
        let len = atom["len"].as_u64().unwrap() as usize;
        let bytes = atom["bytes"].as_array().unwrap();
        String::from_utf8(
            bytes[..len]
                .iter()
                .map(|b| b.as_u64().unwrap() as u8)
                .collect(),
        )
        .unwrap()
    }
    fn same_state(left: &serde_json::Value, right: &serde_json::Value) {
        for key in ["values", "response_entry", "word_copy", "completion"] {
            assert_eq!(left[key], right[key], "writer checkpoint committed {key}");
        }
    }
    let bytes = std::fs::read(std::env::var("R4_WRITER_LEXICAL_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        wire["writer_lexical"].is_object(),
        "requires actual writer lexical artifact"
    );
    let mut input_positions = 0usize;
    let mut output_positions = 0usize;
    for (label, prompt, fact) in [
        (
            "instruction",
            "Where is selvi? Explain in a sentence. Answer:",
            false,
        ),
        ("fact", "Where is selvi? velra in Dusk Ridge. Answer:", true),
    ] {
        let tokens = model.encode(prompt).unwrap();
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        for token in tokens {
            let mut restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            MEASURING.with(|v| v.set(true));
            session.observe(&model, token).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.observe(&model, token).unwrap();
            same_state(&state(&session), &state(&restored));
            input_positions += 1;
        }
        let mut restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        MEASURING.with(|v| v.set(true));
        session.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        restored.begin_response(&model).unwrap();
        let initial = state(&session);
        same_state(&initial, &state(&restored));
        let records: Vec<_> = initial["values"]["relations"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
            .collect();
        assert!(
            records.iter().all(|r| atom_text(&r["owner"]) != "Explain"),
            "an instruction must not assert Explain as a fact owner"
        );
        if fact {
            let matches: Vec<_> = records
                .iter()
                .filter(|r| {
                    atom_text(&r["owner"]) == "velra"
                        && atom_text(&r["value"]) == "Dusk"
                        && atom_text(&r["span"]) == "Dusk Ridge"
                })
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "the matched fact must retain its exact owner/value/span"
            );
        }
        let mut output = [EOS; 96];
        let mut used = 0usize;
        loop {
            let mut restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            let prediction = restored.predict(&model).unwrap();
            let decisions = (
                restored.word_copy_decision(),
                restored.response_entry_decision(),
                restored.value_decision(),
                restored.completion_decision(),
            );
            assert_eq!(restored.predict(&model).unwrap(), prediction);
            assert_eq!(
                (
                    restored.word_copy_decision(),
                    restored.response_entry_decision(),
                    restored.value_decision(),
                    restored.completion_decision()
                ),
                decisions
            );
            MEASURING.with(|v| v.set(true));
            let actual = session.predict(&model).unwrap();
            let actual_decisions = (
                session.word_copy_decision(),
                session.response_entry_decision(),
                session.value_decision(),
                session.completion_decision(),
            );
            session.observe(&model, actual.token).unwrap();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, prediction);
            assert_eq!(actual_decisions, decisions);
            restored.observe(&model, prediction.token).unwrap();
            let current = state(&session);
            same_state(&current, &state(&restored));
            assert_eq!(
                current["values"]["relations"]["records"],
                initial["values"]["relations"]["records"],
                "response emission must preserve the observed input relation records"
            );
            output[used] = actual.token;
            used += 1;
            output_positions += 1;
            if actual.token == EOS || used == output.len() {
                break;
            }
        }
        assert_eq!(
            output[used - 1],
            EOS,
            "writer interface response must terminate within bounded check"
        );
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        println!("actual writer lexical case={label}; input_records={}; actual_text={:?}; allocations=0 bytes=0 for observe/begin/predict/observe; no authored prose-output claim",records.len(),String::from_utf8(model.decode(&output[..used]).unwrap()).unwrap());
    }
    println!("actual writer lexical artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; exact input-owner/value/span and pertoken committed-state checks; allocations=0 bytes=0 (load/encode/session/BOS/checkpoint/JSON/decode/report excluded)",model.artifact_cid());
}

/// Actual field-selection artifact: checkpoints span every observed input and
/// generated output token, including an independent arithmetic turn afterwards.
#[test]
#[ignore = "requires R4_FIELD_COMPOSITION_MODEL; charged actual-model development cases"]
fn native_field_composition_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(session: &Session) -> serde_json::Value {
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap()
    }
    fn same_committed_state(left: &Session, right: &Session) {
        let mut left = state(left);
        let mut right = state(right);
        // Diagnostic scoring work can differ after repeated predict. Every
        // persisted causal/field/memory state remains part of this comparison.
        left.as_object_mut().unwrap().remove("work");
        right.as_object_mut().unwrap().remove("work");
        assert_eq!(left, right, "complete committed checkpoint state");
    }
    let bytes = std::fs::read(std::env::var("R4_FIELD_COMPOSITION_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["field_composition"].is_object());
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    let mut input_positions = 0usize;
    let mut output_positions = 0usize;
    let mut active_field_positions = 0usize;
    let mut malformed_checks = 0usize;
    let mut mismatch_checks = 0usize;
    let default_fields = state(&session)["field_composition"].clone();
    let mut times = Vec::new();
    for (label, prompt, target) in [
        (
            "owner_value",
            "Record: selvi in Dusk Ridge. Where is selvi? Name the owner first. Answer:",
            " selvi is in Dusk Ridge.\n",
        ),
        (
            "independent_sum",
            "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
            "18.\n",
        ),
    ] {
        let tokens = model.encode(prompt).unwrap();
        if session.needs_input_boundary() {
            session.end_response(&model).unwrap();
        }
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        for token in tokens {
            let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
            MEASURING.with(|v| v.set(true));
            session.observe(&model, token).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.observe(&model, token).unwrap();
            same_committed_state(&session, &restored);
            input_positions += 1;
        }
        let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
        MEASURING.with(|v| v.set(true));
        session.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        restored.begin_response(&model).unwrap();
        same_committed_state(&session, &restored);
        let records = state(&session)["values"]["relations"]["records"].clone();
        let mut output = [EOS; 96];
        let mut used = 0usize;
        loop {
            let checkpoint = session.checkpoint().unwrap();
            let before: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let prediction = restored.predict(&model).unwrap();
            let mut after_predict = state(&restored);
            after_predict.as_object_mut().unwrap().remove("work");
            assert_eq!(restored.predict(&model).unwrap(), prediction);
            let mut after_repeated_predict = state(&restored);
            after_repeated_predict.as_object_mut().unwrap().remove("work");
            assert_eq!(after_repeated_predict, after_predict, "repeated predict preserves committed state");
            if before["field_composition"]["anchor"].is_object() {
                active_field_positions += 1;
                if malformed_checks == 0 {
                    for pointer in [
                        "/field_composition/anchor/relation_id",
                        "/field_composition/anchor/source_end",
                        "/field_composition/anchor/source_byte_end",
                        "/field_composition/previous2",
                    ] {
                        let mut changed = before.clone();
                        let field = changed.pointer_mut(pointer).expect("field anchor member");
                        *field = serde_json::json!(field.as_u64().unwrap() + 1);
                        assert!(model.restore_session(&serde_json::to_vec(&changed).unwrap()).is_err(),
                            "altered field anchor must be rejected: {pointer}");
                        malformed_checks += 1;
                    }
                }
            }
            if mismatch_checks == 0
                && before["field_composition"]["read"]["field"] == 1
                && before["field_composition"]["read"]["cursor"]
                    .as_u64()
                    .is_some_and(|cursor| cursor > 0)
            {
                // Fork an actual partially emitted owner. The wrong observed
                // byte must abort the pending field, not advance its cursor.
                let mut mismatched = model.restore_session(&checkpoint).unwrap();
                MEASURING.with(|v| v.set(true));
                let proposal = mismatched.predict(&model).unwrap();
                let wrong = if proposal.token == u32::from(b'x') + 2 {
                    u32::from(b'y') + 2
                } else {
                    u32::from(b'x') + 2
                };
                mismatched.observe(&model, wrong).unwrap();
                MEASURING.with(|v| v.set(false));
                let aborted = state(&mismatched);
                assert_eq!(aborted["field_composition"], default_fields);
                assert_eq!(aborted["response_entry"]["active"], false);
                assert_eq!(aborted["response_entry"]["steps"], 0);
                assert!(aborted["response_entry"]["boundary"].is_null());
                assert_eq!(aborted["response_entry"]["last_action"], "base");
                assert_eq!(aborted["response_entry"]["last"], wrong);
                assert_eq!(aborted["values"]["relations"]["records"], records);
                assert!(mismatched.field_composition_decision().is_none());
                assert!(mismatched.response_entry_decision().is_none());
                let mut resumed = model
                    .restore_session(&mismatched.checkpoint().unwrap())
                    .unwrap();
                same_committed_state(&mismatched, &resumed);
                if resumed.needs_input_boundary() {
                    resumed.end_response(&model).unwrap();
                }
                for token in model.encode("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:").unwrap() {
                    resumed.observe(&model, token).unwrap();
                }
                resumed.begin_response(&model).unwrap();
                let mut next = [EOS; 96];
                let mut next_used = 0usize;
                while next_used < next.len() {
                    let token = resumed.predict(&model).unwrap().token;
                    resumed.observe(&model, token).unwrap();
                    next[next_used] = token;
                    next_used += 1;
                    if token == EOS { break; }
                }
                assert_eq!(next[next_used - 1], EOS);
                assert_eq!(model.decode(&next[..next_used]).unwrap(), b"18.\n");
                mismatch_checks += 1;
                println!("actual field mismatch: observed partial-owner byte {wrong} instead of {}; default field/reset entry and checkpoint restoration preserved; independent next sum=18; mismatched predict/observe included in allocation census, recovery generation excluded", proposal.token);
            }
            MEASURING.with(|v| v.set(true));
            let started = std::time::Instant::now();
            let actual = session.predict(&model).unwrap();
            session.observe(&model, actual.token).unwrap();
            let elapsed = started.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            assert_eq!(actual, prediction);
            restored.observe(&model, prediction.token).unwrap();
            same_committed_state(&session, &restored);
            assert_eq!(state(&session)["values"]["relations"]["records"], records,
                "field emission preserves exact input records");
            output[used] = actual.token;
            used += 1;
            output_positions += 1;
            times.push(elapsed);
            if actual.token == EOS || used == output.len() { break; }
        }
        assert_eq!(output[used - 1], EOS, "{label}: bounded generated stop");
        assert_eq!(model.decode(&output[..used]).unwrap(), target.as_bytes(), "{label}");
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        println!("actual field composition case={label}; output={target:?}; allocations=0 bytes=0 for ingestion/begin/predict/observe");
    }
    assert!(
        active_field_positions > 1,
        "exercise committed field sequencing"
    );
    assert_eq!(malformed_checks, 4);
    assert_eq!(mismatch_checks, 1);
    times.sort_unstable();
    println!("actual field composition artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; active_field_positions={active_field_positions}; malformed_anchor_history_checks={malformed_checks}; mismatch_recovery_checks={mismatch_checks}; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/end-response/checkpoint/JSON/decode/report and mismatch recovery generation excluded; no energy claim)", model.artifact_cid(), times[times.len()/2], times[times.len()-1]);
}

/// Actual writer-choice artifact. Input checkpoints must preserve a revision's
/// exact owner and version link without globally excluding the lexical owner now.
#[test]
#[ignore = "requires R4_WRITER_CHOICE_MODEL; charged actual-model development cases"]
fn native_writer_choice_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(session: &Session) -> serde_json::Value {
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap()
    }
    fn same(left: &Session, right: &Session) {
        let mut left = state(left);
        let mut right = state(right);
        left.as_object_mut().unwrap().remove("work");
        right.as_object_mut().unwrap().remove("work");
        assert_eq!(left, right, "writer choice full causal checkpoint state");
    }
    fn atom(atom: &serde_json::Value) -> Vec<u8> {
        let len = atom["len"].as_u64().unwrap() as usize;
        atom["bytes"].as_array().unwrap()[..len]
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect()
    }
    let bytes = std::fs::read(std::env::var("R4_WRITER_CHOICE_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["writer_choice"].is_object());
    let mut input_positions = 0usize;
    let mut output_positions = 0usize;
    let mut times = Vec::new();
    for (label, prompt, target, revision) in [
        ("revision", "Record: selvi in Dusk Ridge. selvi now in Copper Vale. Where is selvi? Name the owner first. Answer:", " selvi is in Copper Vale.\n", true),
        ("literal_now", "Record: now in Copper Vale. Where is now? Name the owner first. Answer:", " now is in Copper Vale.\n", false),
        ("dependent_revision", "casket in elvin. elvin in Bremen. Now elvin in Zurich. Question: Where is the location of casket? Answer:", " Zurich.\n", false),
    ] {
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        let mut turns = vec![(prompt, target)];
        if revision || label == "dependent_revision" {
            turns.push(("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", "18.\n"));
        }
        for (turn_index, (prompt, target)) in turns.into_iter().enumerate() {
            if session.needs_input_boundary() { session.end_response(&model).unwrap(); }
            let tokens = model.encode(prompt).unwrap();
            ALLOCATIONS.with(|v| v.set(0));
            BYTES.with(|v| v.set(0));
            for token in tokens {
                let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
                MEASURING.with(|v| v.set(true));
                session.observe(&model, token).unwrap();
                MEASURING.with(|v| v.set(false));
                restored.observe(&model, token).unwrap();
                same(&session, &restored);
                input_positions += 1;
            }
            let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
            MEASURING.with(|v| v.set(true));
            session.begin_response(&model).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.begin_response(&model).unwrap();
            same(&session, &restored);
            let initial = state(&session);
            let relations = &initial["values"]["relations"];
            if turn_index == 0 {
                let records: Vec<_> = relations["records"].as_array().unwrap().iter()
                    .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0)).collect();
                let current: Vec<_> = relations["directory"].as_array().unwrap().iter()
                    .filter_map(|id| id.as_u64().filter(|id| *id != 0)).collect();
                if revision {
                    assert_eq!(records.len(), 2);
                    let old = records.iter().find(|r| r["id"] == 1).unwrap();
                    let new = records.iter().find(|r| r["id"] == 2).unwrap();
                    assert_eq!(atom(&old["owner"]), b"selvi");
                    assert_eq!(atom(&old["span"]), b"Dusk Ridge");
                    assert_eq!(old["action"], 1);
                    assert_eq!(old["previous"], 0);
                    assert_eq!(atom(&new["owner"]), b"selvi");
                    assert_eq!(atom(&new["span"]), b"Copper Vale");
                    assert_eq!(new["action"], 2);
                    assert_eq!(new["previous"], 1);
                    assert_eq!(current, vec![2]);
                } else if label == "dependent_revision" {
                    assert_eq!(records.len(), 3);
                    let first = records.iter().find(|r| r["id"] == 1).unwrap();
                    let old = records.iter().find(|r| r["id"] == 2).unwrap();
                    let new = records.iter().find(|r| r["id"] == 3).unwrap();
                    assert_eq!(atom(&first["owner"]), b"casket");
                    assert_eq!(atom(&first["value"]), b"elvin");
                    assert_eq!(first["action"], 1);
                    assert_eq!(first["previous"], 0);
                    assert_eq!(atom(&old["owner"]), b"elvin");
                    assert_eq!(atom(&old["value"]), b"Bremen");
                    assert_eq!(old["action"], 1);
                    assert_eq!(old["previous"], 0);
                    assert_eq!(atom(&new["owner"]), b"elvin");
                    assert_eq!(atom(&new["value"]), b"Zurich");
                    assert_eq!(new["action"], 2);
                    assert_eq!(new["previous"], 2);
                    assert_eq!(current, vec![1, 3]);
                } else {
                    assert_eq!(records.len(), 1);
                    assert_eq!(atom(&records[0]["owner"]), b"now");
                    assert_eq!(atom(&records[0]["span"]), b"Copper Vale");
                    assert_eq!(records[0]["action"], 1);
                    assert_eq!(records[0]["previous"], 0);
                    assert_eq!(current, vec![1]);
                }
            }
            let mut output = [EOS; 96];
            let mut used = 0;
            let mut saw_dependency = false;
            loop {
                let checkpoint = session.checkpoint().unwrap();
                let mut restored = model.restore_session(&checkpoint).unwrap();
                let predicted = restored.predict(&model).unwrap();
                assert_eq!(restored.predict(&model).unwrap(), predicted);
                MEASURING.with(|v| v.set(true));
                let start = std::time::Instant::now();
                let actual = session.predict(&model).unwrap();
                saw_dependency |= session.word_copy_decision()
                    .is_some_and(|decision| decision.dependency == Some([1, 3]));
                session.observe(&model, actual.token).unwrap();
                let elapsed = start.elapsed().as_nanos();
                MEASURING.with(|v| v.set(false));
                assert_eq!(actual, predicted);
                restored.observe(&model, predicted.token).unwrap();
                same(&session, &restored);
                assert_eq!(state(&session)["values"]["relations"], *relations);
                output[used] = actual.token;
                used += 1;
                output_positions += 1;
                times.push(elapsed);
                if actual.token == EOS || used == output.len() { break; }
            }
            assert_eq!(output[used - 1], EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(), target.as_bytes());
            if label == "dependent_revision" && turn_index == 0 {
                assert!(saw_dependency, "answer must use the legitimate revised dependency [1,3]");
            }
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
            println!("actual writer choice case={label}; turn={turn_index}; output={target:?}; exact records/version directory; allocations=0 bytes=0 for ingestion/begin/predict/observe");
        }
    }
    times.sort_unstable();
    println!("actual writer choice artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/end-response/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

/// Actual source-routing artifact: the selected field must bind to the current
/// explicit revision, while unsupported historical/raw requests preserve parent.
#[test]
#[ignore = "requires R4_CURRENT_SOURCE_MODEL; charged actual-model development cases"]
fn native_current_source_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(s: &Session) -> serde_json::Value {
        serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
    }
    fn same(left: &Session, right: &Session) {
        let mut left = state(left);
        let mut right = state(right);
        left.as_object_mut().unwrap().remove("work");
        right.as_object_mut().unwrap().remove("work");
        assert_eq!(left, right, "current-source causal checkpoint state");
    }
    fn atom(v: &serde_json::Value) -> Vec<u8> {
        v["bytes"].as_array().unwrap()[..v["len"].as_u64().unwrap() as usize]
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect()
    }
    fn raw(model: &Model, prompt: &str, control: Control) -> Vec<u32> {
        let mut s = model.session(control).unwrap();
        s.observe(model, BOS).unwrap();
        for token in model.encode(prompt).unwrap() {
            s.observe(model, token).unwrap();
        }
        s.begin_response(model).unwrap();
        let mut out = Vec::new();
        for _ in 0..96 {
            let token = s.predict(model).unwrap().token;
            s.observe(model, token).unwrap();
            out.push(token);
            if token == EOS {
                break;
            }
        }
        out
    }
    let bytes = std::fs::read(std::env::var("R4_CURRENT_SOURCE_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["current_source"].is_object());
    let integrated_writer = wire["writer_role"].is_object();
    drop(wire);
    drop(bytes);
    let mut input_positions = 0;
    let mut output_positions = 0;
    let mut times = Vec::new();
    let facts = "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.";
    let mut cases = vec![
        (
            "owner",
            String::new(),
            "Name the owner first.",
            " selvi is in Copper Vale.\n",
        ),
        ("value", String::new(), "", " Copper Vale.\n"),
        (
            "evicted_owner",
            "oak ash elm ".repeat(12),
            "Name the owner first.",
            " selvi is in Copper Vale.\n",
        ),
    ];
    if integrated_writer {
        cases.extend([
            (
                "latest_amber",
                String::new(),
                "Name the owner first.",
                " selvi is in Amber Field.\n",
            ),
            (
                "latest_known_prime",
                String::new(),
                "Name the owner first.",
                " selvi is in Ash Court.\n",
            ),
            (
                "literal_now",
                String::new(),
                "Name the owner first.",
                " now is in Amber Field.\n",
            ),
        ]);
    }
    for (label, padding, instruction, target) in cases {
        let (case_facts, owner, last_value, current_id) = match label {
            "latest_amber" => ("Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field.", "selvi", "Amber Field", 3),
            "latest_known_prime" => ("Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Ash Court.", "selvi", "Ash Court", 3),
            "literal_now" => ("Record: now in Amber Field.", "now", "Amber Field", 1),
            _ => (facts, "selvi", "Copper Vale", 2),
        };
        let prompt = format!("{case_facts} {padding}Where is {owner}? {instruction} Answer:");
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        let mut turns = vec![(prompt, target)];
        if label == "owner" {
            turns.push(("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:".into(), "18.\n"));
        }
        for (turn, (prompt, target)) in turns.into_iter().enumerate() {
            if session.needs_input_boundary() {
                session.end_response(&model).unwrap();
            }
            let tokens = model.encode(&prompt).unwrap();
            assert!(tokens.len() <= 512);
            ALLOCATIONS.with(|v| v.set(0));
            BYTES.with(|v| v.set(0));
            for token in tokens {
                let mut restored = model
                    .restore_session(&session.checkpoint().unwrap())
                    .unwrap();
                MEASURING.with(|v| v.set(true));
                session.observe(&model, token).unwrap();
                MEASURING.with(|v| v.set(false));
                restored.observe(&model, token).unwrap();
                same(&session, &restored);
                input_positions += 1;
            }
            let mut restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            MEASURING.with(|v| v.set(true));
            session.begin_response(&model).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.begin_response(&model).unwrap();
            same(&session, &restored);
            let initial = state(&session);
            let relations = &initial["values"]["relations"];
            let records = relations["records"].as_array().unwrap();
            let current = records.iter().find(|r| r["id"] == current_id).unwrap();
            if turn == 0 {
                let active: Vec<_> = records
                    .iter()
                    .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
                    .collect();
                assert_eq!(active.len(), current_id as usize);
                for id in 1..=current_id {
                    let r = active.iter().find(|r| r["id"] == id).unwrap();
                    let span = if id == current_id {
                        last_value
                    } else if id == 1 {
                        "Dusk Ridge"
                    } else {
                        "Copper Vale"
                    };
                    assert_eq!(atom(&r["owner"]), owner.as_bytes());
                    assert_eq!(atom(&r["span"]), span.as_bytes());
                    assert_eq!(r["action"], if id == 1 { 1 } else { 2 });
                    assert_eq!(r["previous"], id - 1);
                    assert_eq!(r["conflict"], false);
                }
                let directory: Vec<_> = relations["directory"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_u64().filter(|id| *id != 0))
                    .collect();
                assert_eq!(directory, vec![current_id]);
            }
            let endpoint = current["value"]["end"].as_u64().unwrap();
            let byte_endpoint = current["value"]["byte_end"].as_u64().unwrap();
            let mut owner_read = false;
            let mut value_read = false;
            let mut word_read = false;
            let mut output = [EOS; 96];
            let mut used = 0;
            loop {
                let mut restored = model
                    .restore_session(&session.checkpoint().unwrap())
                    .unwrap();
                let predicted = restored.predict(&model).unwrap();
                assert_eq!(restored.predict(&model).unwrap(), predicted);
                MEASURING.with(|v| v.set(true));
                let start = std::time::Instant::now();
                let actual = session.predict(&model).unwrap();
                let field = session.field_composition_decision();
                let word = session.word_copy_decision();
                session.observe(&model, actual.token).unwrap();
                let elapsed = start.elapsed().as_nanos();
                MEASURING.with(|v| v.set(false));
                if let Some(d) = field.filter(|d| d.field != 0) {
                    assert_eq!(d.anchor.relation_id, current_id);
                    assert_eq!(d.anchor.source_end, endpoint);
                    assert_eq!(d.anchor.source_byte_end, byte_endpoint);
                    owner_read |= d.field == 1;
                    value_read |= d.field == 2;
                }
                word_read |= word.is_some_and(|d| {
                    matches!(d.action, WordCopyAction::Read | WordCopyAction::Prepare)
                        && d.source_end == endpoint
                        && d.source_byte_end == byte_endpoint
                });
                assert_eq!(actual, predicted);
                restored.observe(&model, predicted.token).unwrap();
                same(&session, &restored);
                assert_eq!(state(&session)["values"]["relations"], *relations);
                output[used] = actual.token;
                used += 1;
                output_positions += 1;
                times.push(elapsed);
                if actual.token == EOS || used == output.len() {
                    break;
                }
            }
            assert_eq!(output[used - 1], EOS);
            assert_eq!(model.decode(&output[..used]).unwrap(), target.as_bytes());
            if turn == 0 {
                if label == "value" {
                    assert!(word_read);
                } else {
                    assert!(owner_read && value_read);
                }
            }
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
            println!("actual current source case={label}; turn={turn}; target={target:?}; exact current revision endpoint; allocations=0 bytes=0");
        }
    }
    let parent_start = std::time::Instant::now();
    let parent = model.without_current_source().unwrap();
    let parent_restore_ns = parent_start.elapsed().as_nanos();
    for query in [
        "Where was selvi before? Answer:",
        "Copy Dusk Ridge. Answer:",
    ] {
        let prompt = format!("{facts} {query}");
        assert_eq!(
            raw(&model, &prompt, Control::Full),
            raw(&parent, &prompt, Control::Full),
            "unqualified historical/raw parent control: {query}"
        );
    }
    let prompt = format!("{facts} Where is selvi? Name the owner first. Answer:");
    let reference = raw(&parent, &prompt, Control::Full);
    for control in [
        Control::CurrentSourceDisabled,
        Control::CurrentSourceVersionDisabled,
    ] {
        assert_eq!(
            raw(&model, &prompt, control),
            reference,
            "intervention restores exact parent bytes/EOS"
        );
    }
    times.sort_unstable();
    println!("actual current source artifact={}; load_ns={load_ns}; parent_restore_ns={parent_restore_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; historical_raw_controls=2; disabled_controls=2; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/parent restore/control generation/encode/session/BOS/end-response/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

/// Actual shared writer-role artifact: two revisions retain both prior versions,
/// and a literal factual owner now must remain available.
#[test]
#[ignore = "requires R4_WRITER_ROLE_MODEL; charged actual-model development cases"]
fn native_writer_role_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(s: &Session) -> serde_json::Value {
        serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
    }
    fn same(a: &Session, b: &Session) {
        let mut a = state(a);
        let mut b = state(b);
        a.as_object_mut().unwrap().remove("work");
        b.as_object_mut().unwrap().remove("work");
        assert_eq!(a, b, "writer-role full causal checkpoint state");
    }
    fn atom(v: &serde_json::Value) -> Vec<u8> {
        v["bytes"].as_array().unwrap()[..v["len"].as_u64().unwrap() as usize]
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect()
    }
    let bytes = std::fs::read(std::env::var("R4_WRITER_ROLE_MODEL").unwrap()).unwrap();
    let load = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = load.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["writer_role"].is_object());
    drop(wire);
    drop(bytes);
    let mut input_positions = 0;
    let mut output_positions = 0;
    let mut times = Vec::new();
    for (label, prompt, target, revisions) in [
        ("two_revisions", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field. Where is selvi? Name the owner first. Answer:", " selvi is in Amber Field.\n", true),
        ("literal_now", "Record: now in Amber Field. Where is now? Name the owner first. Answer:", " now is in Amber Field.\n", false),
    ] {
        let mut session = model.session(Control::Full).unwrap(); session.observe(&model, BOS).unwrap();
        let mut turns = vec![(prompt, target)];
        if revisions { turns.push(("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", "18.\n")); }
        for (turn, (prompt, target)) in turns.into_iter().enumerate() {
            if session.needs_input_boundary() { session.end_response(&model).unwrap(); }
            let tokens = model.encode(prompt).unwrap(); assert!(tokens.len() <= 512);
            ALLOCATIONS.with(|v| v.set(0)); BYTES.with(|v| v.set(0));
            for token in tokens {
                let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
                MEASURING.with(|v| v.set(true)); session.observe(&model, token).unwrap(); MEASURING.with(|v| v.set(false));
                restored.observe(&model, token).unwrap(); same(&session, &restored); input_positions += 1;
            }
            let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
            MEASURING.with(|v| v.set(true)); session.begin_response(&model).unwrap(); MEASURING.with(|v| v.set(false));
            restored.begin_response(&model).unwrap(); same(&session, &restored);
            let initial = state(&session); let relations = &initial["values"]["relations"];
            let records: Vec<_> = relations["records"].as_array().unwrap().iter().filter(|r| r["id"].as_u64().is_some_and(|id| id != 0)).collect();
            let current: Vec<_> = relations["directory"].as_array().unwrap().iter().filter_map(|v| v.as_u64().filter(|id| *id != 0)).collect();
            if turn == 0 {
                if revisions {
                    assert_eq!(records.len(), 3); assert_eq!(current, vec![3]);
                    for (id, span, action, previous) in [(1, b"Dusk Ridge".as_slice(), 1, 0), (2, b"Copper Vale".as_slice(), 2, 1), (3, b"Amber Field".as_slice(), 2, 2)] {
                        let record = records.iter().find(|r| r["id"] == id).unwrap();
                        assert_eq!(atom(&record["owner"]), b"selvi"); assert_eq!(atom(&record["span"]), span);
                        assert_eq!(record["action"], action); assert_eq!(record["previous"], previous); assert_eq!(record["conflict"], false);
                    }
                } else {
                    assert_eq!(records.len(), 1); assert_eq!(current, vec![1]);
                    assert_eq!(atom(&records[0]["owner"]), b"now"); assert_eq!(atom(&records[0]["span"]), b"Amber Field");
                    assert_eq!(records[0]["action"], 1); assert_eq!(records[0]["previous"], 0); assert_eq!(records[0]["conflict"], false);
                }
            }
            let mut output = [EOS; 96]; let mut used = 0;
            let mut owner_read = false; let mut value_read = false;
            loop {
                let mut restored = model.restore_session(&session.checkpoint().unwrap()).unwrap();
                let predicted = restored.predict(&model).unwrap(); assert_eq!(restored.predict(&model).unwrap(), predicted);
                MEASURING.with(|v| v.set(true)); let start = std::time::Instant::now();
                let actual = session.predict(&model).unwrap(); let decision = session.field_composition_decision();
                session.observe(&model, actual.token).unwrap(); let elapsed = start.elapsed().as_nanos(); MEASURING.with(|v| v.set(false));
                if let Some(d) = decision.filter(|d| d.field != 0) {
                    assert_eq!(d.anchor.relation_id, if revisions {3} else {1});
                    owner_read |= d.field == 1; value_read |= d.field == 2;
                }
                assert_eq!(actual, predicted); restored.observe(&model, predicted.token).unwrap(); same(&session, &restored);
                assert_eq!(state(&session)["values"]["relations"], *relations);
                output[used] = actual.token; used += 1; output_positions += 1; times.push(elapsed);
                if actual.token == EOS || used == output.len() { break; }
            }
            assert_eq!(output[used - 1], EOS); assert_eq!(model.decode(&output[..used]).unwrap(), target.as_bytes());
            if turn == 0 { assert!(owner_read && value_read); }
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
            println!("actual writer-role case={label}; turn={turn}; target={target:?}; exact current chain and observed field reads; allocations=0 bytes=0");
        }
    }
    times.sort_unstable();
    println!("actual writer-role artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/end-response/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

/// Opt-in two-owner source discrimination on the actual candidate; earlier
/// current-source artifacts are not subjected to this new acceptance panel.
#[test]
#[ignore = "requires R4_HISTORICAL_VERSION_MODEL; charged actual historical-version-intent artifact"]
fn native_historical_version_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(s: &Session) -> serde_json::Value {
        serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
    }
    fn same(a: &Session, b: &Session) {
        let mut a = state(a);
        let mut b = state(b);
        a.as_object_mut().unwrap().remove("work");
        b.as_object_mut().unwrap().remove("work");
        assert_eq!(a, b, "historical-version full causal checkpoint state");
    }
    let bytes = std::fs::read(std::env::var("R4_HISTORICAL_VERSION_MODEL").unwrap()).unwrap();
    let start = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = start.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["historical_version_intent"].is_object());
    drop(wire);
    drop(bytes);
    let mut input_positions = 0;
    let mut output_positions = 0;
    let mut times = Vec::new();
    let mut evicted = String::from(
        "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
    );
    for (k, name) in [
        "arbor", "brook", "cairn", "delta", "ember", "fjord", "glade", "haven", "islet", "jetty",
        "knoll", "lagoon", "marsh", "nadir",
    ]
    .iter()
    .enumerate()
    {
        evicted.push_str(&format!(" {name} in Zone{k}."));
    }
    // (facts, owner, request, expected, root id, head id, depth); root 0 means an abstention.
    let cases = [
        (evicted.as_str(), "selvi", "What was the initial location of selvi?", " Unknown.\n", 0, 3, 0),
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.", "selvi", "What was the initial location of selvi?", " Dusk Ridge.\n", 1, 3, 2),
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.", "selvi", "What was the initial location of selvi? Name the owner first.", " selvi was in Dusk Ridge.\n", 1, 3, 2),
        ("Record: moss dale holds tilva. tilva now in Birch Grove. tilva now in Pine Hollow. tilva now in Cedar Point.", "tilva", "Where was tilva originally? State the owner first.", " tilva was in moss dale.\n", 1, 4, 3),
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.", "selvi", "What was the previous location of selvi? Name the owner first.", " selvi was in Copper Vale.\n", 2, 3, 1),
        // Resident same-value reassertions are validated links: the root, not an abstention.
        ("Record: selvi in Dusk Ridge. selvi in Dusk Ridge. selvi now in Copper Vale.", "selvi", "What was the initial location of selvi?", " Dusk Ridge.\n", 1, 3, 2),
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi in Copper Vale. selvi now in Amber Field.", "selvi", "What was the initial location of selvi? Name the owner first.", " selvi was in Dusk Ridge.\n", 1, 4, 3),
        // A same-value reassertion head: the first hop reaches the head's previous record
        // and the root stays readable (record-hop semantics).
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi in Copper Vale.", "selvi", "What was the initial location of selvi?", " Dusk Ridge.\n", 1, 3, 2),
        ("Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi in Copper Vale.", "selvi", "What was the previous location of selvi? Name the owner first.", " selvi was in Copper Vale.\n", 2, 3, 1),
        // The smallest repeated-head chain: previous is record 1 through the head hop.
        ("Record: selvi in Dusk Ridge. selvi in Dusk Ridge.", "selvi", "What was the previous location of selvi?", " Dusk Ridge.\n", 1, 2, 1),
        ("Record: selvi in Dusk Ridge. selvi in Dusk Ridge.", "selvi", "What was the initial location of selvi? Name the owner first.", " selvi was in Dusk Ridge.\n", 1, 2, 1),
        // Reader-window contract: a current request whose two instructions follow the
        // question reads the live head (root == head, depth 0) through the owner-first
        // field path on a reverse-form record.
        ("Record: amber quay holds pemru. Record: amber quay holds pemru.", "pemru", "Where is pemru? Explain in a sentence. Name the owner first.", " pemru is in amber quay.\n", 2, 2, 0),
        ("Record: amber quay holds pemru.", "pemru", "Where is pemru? Name the owner first. Explain in a sentence.", " pemru is in amber quay.\n", 1, 1, 0),
    ];
    for (facts, owner, request, target, root, head, depth) in cases {
        let prompt = format!("{facts} {request} Answer:");
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, BOS).unwrap();
        let tokens = model.encode(&prompt).unwrap();
        assert!(tokens.len() <= 512);
        ALLOCATIONS.with(|v| v.set(0));
        BYTES.with(|v| v.set(0));
        for token in tokens {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            MEASURING.with(|v| v.set(true));
            s.observe(&model, token).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.observe(&model, token).unwrap();
            same(&s, &restored);
            input_positions += 1;
        }
        let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
        MEASURING.with(|v| v.set(true));
        s.begin_response(&model).unwrap();
        MEASURING.with(|v| v.set(false));
        restored.begin_response(&model).unwrap();
        same(&s, &restored);
        let initial = state(&s);
        let relations = &initial["values"]["relations"];
        let records = relations["records"].as_array().unwrap();
        let abstain = root == 0;
        let (endpoint, byte_endpoint) = if abstain {
            assert!(records.iter().all(|r| r["id"] != 1), "root must be evicted");
            (0, 0)
        } else {
            let selected_record = records.iter().find(|r| r["id"] == root).unwrap();
            if root != head {
                assert_eq!(
                    selected_record["previous"],
                    if depth == 1 && root != 1 { root - 1 } else { 0 }
                );
            } else {
                // A current target: the live head itself, resident in the directory.
                assert!(relations["directory"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(head)));
                assert_eq!(depth, 0);
            }
            (
                selected_record["value"]["end"].as_u64().unwrap(),
                selected_record["value"]["byte_end"].as_u64().unwrap(),
            )
        };
        let mut selected = false;
        let mut out = [EOS; 96];
        let mut used = 0;
        loop {
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            let predicted = restored.predict(&model).unwrap();
            assert_eq!(restored.predict(&model).unwrap(), predicted);
            MEASURING.with(|v| v.set(true));
            let start = std::time::Instant::now();
            let actual = s.predict(&model).unwrap();
            let word = s.word_copy_decision();
            let field = s.field_composition_decision();
            s.observe(&model, actual.token).unwrap();
            let elapsed = start.elapsed().as_nanos();
            MEASURING.with(|v| v.set(false));
            selected |= if abstain {
                word.is_some_and(|d| d.action == WordCopyAction::NoRead && d.word_index == 16)
            } else {
                word.is_some_and(|d| {
                    matches!(d.action, WordCopyAction::Prepare | WordCopyAction::Read)
                        && d.source_end == endpoint
                        && d.source_byte_end == byte_endpoint
                }) || field.is_some_and(|d| {
                    d.field != 0
                        && d.anchor.relation_id == root
                        && d.anchor.current_revision == (root != head).then_some(head)
                        && d.anchor.ancestor_depth == (depth > 1).then_some(depth)
                        && d.anchor.source_end == endpoint
                        && d.anchor.source_byte_end == byte_endpoint
                })
            };
            assert_eq!(actual, predicted);
            restored.observe(&model, predicted.token).unwrap();
            same(&s, &restored);
            assert_eq!(state(&s)["values"]["relations"], *relations);
            out[used] = actual.token;
            used += 1;
            output_positions += 1;
            times.push(elapsed);
            if actual.token == EOS || used == out.len() {
                break;
            }
        }
        assert_eq!(out[used - 1], EOS);
        assert_eq!(model.decode(&out[..used]).unwrap(), target.as_bytes());
        assert!(
            selected,
            "actual answer must select the exact ancestor endpoint or abstain for {owner}"
        );
        assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
        println!("historical-version owner={owner} root={root} head={head} depth={depth} output={target:?}; allocations=0 bytes=0");
    }
    times.sort_unstable();
    println!("actual historical-version artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

#[test]
#[ignore = "requires R4_QUERY_OWNER_MODEL; charged actual-model query-owner cases"]
fn native_query_owner_actual_checkpoint_and_allocation() {
    use uor_r4_core::native_geometric::{Model, Session};
    fn state(s: &Session) -> serde_json::Value {
        serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
    }
    fn same(a: &Session, b: &Session) {
        let mut a = state(a);
        let mut b = state(b);
        a.as_object_mut().unwrap().remove("work");
        b.as_object_mut().unwrap().remove("work");
        assert_eq!(a, b, "query-owner full causal checkpoint state");
    }
    fn atom(v: &serde_json::Value) -> Vec<u8> {
        v["bytes"].as_array().unwrap()[..v["len"].as_u64().unwrap() as usize]
            .iter()
            .map(|n| n.as_u64().unwrap() as u8)
            .collect()
    }
    let bytes = std::fs::read(std::env::var("R4_QUERY_OWNER_MODEL").unwrap()).unwrap();
    let start = std::time::Instant::now();
    let model = Model::from_bytes(&bytes).unwrap();
    let load_ns = start.elapsed().as_nanos();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["current_source"].is_object() && wire["writer_role"].is_object());
    drop(wire);
    drop(bytes);
    let mut input_positions = 0;
    let mut output_positions = 0;
    let mut times = Vec::new();
    for swapped in [false, true] {
        let chains = if swapped {
            [
                ("tilva", "Silver Cove", "Birch Grove"),
                ("selvi", "Dusk Ridge", "Copper Vale"),
            ]
        } else {
            [
                ("selvi", "Dusk Ridge", "Copper Vale"),
                ("tilva", "Silver Cove", "Birch Grove"),
            ]
        };
        let facts = format!(
            "Record: {} holds {}. {} now in {}. {} holds {}. {} now in {}.",
            chains[0].1,
            chains[0].0,
            chains[0].0,
            chains[0].2,
            chains[1].1,
            chains[1].0,
            chains[1].0,
            chains[1].2
        );
        for (chain, (owner, _, value)) in chains.iter().enumerate() {
            let prompt = format!("{facts} Where is {owner}?  Answer:");
            let target = format!(" {value}.\n");
            let mut s = model.session(Control::Full).unwrap();
            s.observe(&model, BOS).unwrap();
            let tokens = model.encode(&prompt).unwrap();
            assert!(tokens.len() <= 512);
            ALLOCATIONS.with(|v| v.set(0));
            BYTES.with(|v| v.set(0));
            for token in tokens {
                let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
                MEASURING.with(|v| v.set(true));
                s.observe(&model, token).unwrap();
                MEASURING.with(|v| v.set(false));
                restored.observe(&model, token).unwrap();
                same(&s, &restored);
                input_positions += 1;
            }
            let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
            MEASURING.with(|v| v.set(true));
            s.begin_response(&model).unwrap();
            MEASURING.with(|v| v.set(false));
            restored.begin_response(&model).unwrap();
            same(&s, &restored);
            let initial = state(&s);
            let relations = &initial["values"]["relations"];
            let records: Vec<_> = relations["records"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
                .collect();
            assert_eq!(records.len(), 4);
            for (i, (o, old, new)) in chains.iter().enumerate() {
                for (offset, span) in [(1, *old), (2, *new)] {
                    let id = (i as u64) * 2 + offset;
                    let r = records.iter().find(|r| r["id"] == id).unwrap();
                    assert_eq!(atom(&r["owner"]), o.as_bytes());
                    assert_eq!(atom(&r["span"]), span.as_bytes());
                    assert_eq!(r["action"], offset);
                    assert_eq!(r["previous"], if offset == 1 { 0 } else { id - 1 });
                    assert_eq!(r["conflict"], false);
                }
            }
            let directory: Vec<_> = relations["directory"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| n.as_u64().filter(|n| *n != 0))
                .collect();
            assert_eq!(directory, vec![2, 4]);
            let current_id = (chain as u64) * 2 + 2;
            let current = records.iter().find(|r| r["id"] == current_id).unwrap();
            let endpoint = current["value"]["end"].as_u64().unwrap();
            let byte_endpoint = current["value"]["byte_end"].as_u64().unwrap();
            let mut selected = false;
            let mut out = [EOS; 96];
            let mut used = 0;
            loop {
                let mut restored = model.restore_session(&s.checkpoint().unwrap()).unwrap();
                let predicted = restored.predict(&model).unwrap();
                assert_eq!(restored.predict(&model).unwrap(), predicted);
                MEASURING.with(|v| v.set(true));
                let start = std::time::Instant::now();
                let actual = s.predict(&model).unwrap();
                let word = s.word_copy_decision();
                let field = s.field_composition_decision();
                s.observe(&model, actual.token).unwrap();
                let elapsed = start.elapsed().as_nanos();
                MEASURING.with(|v| v.set(false));
                selected |= word.is_some_and(|d| {
                    matches!(d.action, WordCopyAction::Prepare | WordCopyAction::Read)
                        && d.source_end == endpoint
                        && d.source_byte_end == byte_endpoint
                }) || field.is_some_and(|d| {
                    d.field != 0
                        && d.anchor.relation_id == current_id
                        && d.anchor.source_end == endpoint
                        && d.anchor.source_byte_end == byte_endpoint
                });
                assert_eq!(actual, predicted);
                restored.observe(&model, predicted.token).unwrap();
                same(&s, &restored);
                assert_eq!(state(&s)["values"]["relations"], *relations);
                out[used] = actual.token;
                used += 1;
                output_positions += 1;
                times.push(elapsed);
                if actual.token == EOS || used == out.len() {
                    break;
                }
            }
            assert_eq!(out[used - 1], EOS);
            assert_eq!(model.decode(&out[..used]).unwrap(), target.as_bytes());
            assert!(
                selected,
                "actual answer must select queried owner's current endpoint"
            );
            assert_eq!((ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get)), (0, 0));
            println!("query-owner swapped={swapped} queried={owner} current_record={current_id} output={target:?}; allocations=0 bytes=0");
        }
    }
    times.sort_unstable();
    println!("actual query-owner artifact={}; load_ns={load_ns}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; allocations=0 bytes=0; predict_observe median_ns={} max_ns={} (load/encode/session/BOS/checkpoint/JSON/decode/report excluded; no energy claim)",model.artifact_cid(),times[times.len()/2],times[times.len()-1]);
}

#[test]
#[ignore = "Requires an actual block-calibrated or joint artifact path"]
fn shared_core_saved_angular_tree_has_bounded_zero_allocation_prediction() {
    use uor_r4_core::native_geometric::shared_core::{Intervention, SharedCore};
    let path = std::env::var("UOR_SHARED_CORE_CANDIDATE").expect("actual candidate path");
    let raw = std::fs::read(path).expect("candidate bytes");
    let model = SharedCore::from_bytes(&raw).expect("source-bound candidate");
    assert_eq!(model.to_bytes().unwrap(), raw);
    let mut session = model.session(Intervention::Full);
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));
    let mut valid = true;
    for position in 0..512_u16 {
        valid &= session.observe(position as u8).is_ok();
        let before = session.work();
        valid &= std::hint::black_box(session.predict()) <= 256;
        valid &= session.work().products - before.products <= 27;
    }
    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
}

#[test]
fn hopf_metric_trajectory_and_serving_projection_have_zero_allocations() {
    use uor_r4_core::native_geometric::hopf_metric::{HopfStateTrajectory, UnitS2Q30, UnitS3};
    let initial = UnitS3::IDENTITY;
    let mut traj = HopfStateTrajectory::new(initial);
    let delta = UnitS3::new(0.99, 0.01, 0.05, 0.02).unwrap();

    let roots = [
        UnitS2Q30::NORTH_POLE,
        UnitS2Q30::SOUTH_POLE,
        UnitS2Q30::EQUATOR_X,
        UnitS2Q30::EQUATOR_Y,
    ];

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut traj_q30 =
        uor_r4_core::native_geometric::hopf_metric::HopfStateTrajectoryQ30::new(initial.to_q30());
    let delta_q30 = delta.to_q30();

    let mut valid = true;
    for _ in 0..512 {
        let step = traj.step(&delta);
        valid &= step > 0.0;
        let s2_q30 = traj.current_s3.to_q30().hopf_project();
        let nearest = s2_q30.nearest_root_q30(&roots);
        valid &= nearest.is_some_and(|idx| idx < 4);

        // Exercise Q1.30 fiber-preserving holonomy methods
        let fiber_pt = traj.current_s3.to_q30().hopf_fiber_project();
        valid &= fiber_pt.base.0[2].abs()
            <= uor_r4_core::native_geometric::hopf_metric::Q30_SCALE as i32;
        valid &= fiber_pt.fiber_phase.abs()
            <= uor_r4_core::native_geometric::hopf_metric::Q30_SCALE as i32;

        traj_q30.step_q30(&delta_q30);
        valid &= traj_q30.step_count > 0;
    }

    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
}

#[test]
fn vsa_hot_path_operations_have_zero_allocations() {
    use uor_r4_core::native_geometric::vsa::{
        encode_multiscale_context, encode_ngram, Codebook, Hypervector4096, RollingVsaContext,
    };

    let codebook = Codebook::<64>::new(256, 0x1234_5678);
    let mut ctx = RollingVsaContext::<64, 64>::new();
    let query = Hypervector4096::from_seed(0xcafe, 0xbabe);
    let v1 = codebook.get(10);
    let v2 = codebook.get(20);
    let v3 = codebook.get(30);
    let v4 = codebook.get(40);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut valid = true;
    for step in 0..512_u32 {
        let tok = step % 256;
        ctx.push(tok);

        // 1. N-gram encoding
        let ng = ctx.current_ngram(&codebook, 4);
        valid &= ng.count_ones() > 0;

        // 2. Rolling multi-scale context bundling (non-empty for all length >= 1)
        let ctx_vec = ctx.current_context_hypervector(&codebook, 4);
        if ctx.len() >= 1 {
            valid &= ctx_vec.count_ones() > 0;
        }

        // 3. Zero-allocation top-K candidate search on stack
        let top = codebook.top_k_candidates_q15::<8>(&query);
        valid &= top[0].1 >= top[7].1;

        // 4. Candidate continuation scoring
        let score = ctx.score_continuation_q15(&codebook, &ctx_vec, tok);
        valid &= score >= 0;

        // 5. Primitive VSA algebraic ops
        let bound = v1.bind(&v2);
        let permuted = bound.permute(step as usize % 4096);
        let sim = permuted.similarity_q15(&v3);
        let corr = permuted.bipolar_correlation_q15(&v4);
        valid &= sim >= 0 && corr >= -32767;

        // 6. Majority bundling across active window
        let bundle = Hypervector4096::bundle(&[v1, v2, v3, v4]);
        valid &= bundle.count_ones() > 0;

        // 7. Pure functional sequence encoding
        let pure_ngram = encode_ngram(&[tok, tok.wrapping_add(1), tok.wrapping_add(2)], &codebook);
        let pure_multi = encode_multiscale_context(&[tok, tok.wrapping_add(1)], &codebook, 2);
        valid &= pure_ngram.count_ones() > 0 && pure_multi.count_ones() > 0;
    }

    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "VSA hot path must have zero heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "VSA hot path must allocate zero heap bytes"
    );
}

#[test]
fn engram_table_lookup_has_zero_allocations() {
    use uor_r4_core::native_geometric::engram::{hash_bigram, hash_trigram, EngramTable};

    let mut table = EngramTable::with_capacity(1024);
    let key1 = hash_bigram(10, 20);
    let key2 = hash_trigram(5, 10, 20);
    table.insert(key1, &[(30, 16384), (31, 8192)]);
    table.insert(key2, &[(40, 24576), (41, 12288)]);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut hits = 0;
    for i in 0..1024 {
        let w_curr = (i % 256) as u32;
        let w_prev = ((i + 1) % 256) as u32;
        let w_prev2 = ((i + 2) % 256) as u32;

        if let Some(cands) = table.lookup_bigram(w_prev, w_curr) {
            hits += cands.len();
        }
        if let Some(cands) = table.lookup_trigram(w_prev2, w_prev, w_curr) {
            hits += cands.len();
        }
        if let Some(cands) = table.lookup(key1) {
            hits += cands.len();
        }
    }

    MEASURING.with(|v| v.set(false));
    assert!(hits > 0);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "EngramTable lookup must have zero heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "EngramTable lookup must allocate zero heap bytes"
    );
}

#[test]
fn hierarchical_lattice_tables_lookup_has_zero_allocations() {
    use uor_r4_core::native_geometric::lattice_table::{
        HierarchicalLatticeTables, COARSE_TABLE_SIZE,
    };

    let vocab_size = 256;
    let num_clusters = 16;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let coarse = vec![5i8; COARSE_TABLE_SIZE];
    let fine = vec![12i16; num_clusters * num_clusters];
    let tables = HierarchicalLatticeTables::new(num_clusters, token_to_cluster, coarse, fine);

    // Warm up
    let _ = tables.score(1, 2, 3, 0, 1);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut accum = 0i64;
    for i in 0..1024 {
        let r_prev = (i * 7) % 120;
        let r_curr = (i * 13) % 120;
        let r_cand = (i * 23) % 120;
        let c_curr = (i * 3) % num_clusters;
        let c_cand = (i * 5) % num_clusters;
        accum += tables.score(r_prev, r_curr, r_cand, c_curr, c_cand) as i64;
    }
    std::hint::black_box(accum);

    MEASURING.with(|v| v.set(false));

    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "HierarchicalLatticeTables serving lookups must perform ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "HierarchicalLatticeTables serving lookups must allocate ZERO bytes"
    );
}

#[test]
fn mmap_corpus_streaming_reads_have_zero_allocations() {
    use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_zero_alloc_{}_{}.u16",
        std::process::id(),
        std::time::Instant::now().elapsed().as_nanos()
    ));

    let sample_tokens: Vec<u16> = (0..10_000).map(|i| (i % 4096) as u16).collect();
    CorpusWriter::write_file(&temp_file, 4096, &sample_tokens).expect("write_file must succeed");

    let reader = MmapCorpusReader::open(&temp_file).expect("MmapCorpusReader::open must succeed");
    assert_eq!(reader.len(), 10_000);

    // Warm up
    let _ = reader.get(0);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut accum: u64 = 0;

    // 1. Full slice streaming
    for &tok in reader.as_slice() {
        accum = accum.wrapping_add(tok as u64);
    }

    // 2. Direct random indexing
    for i in 0..1024 {
        accum = accum.wrapping_add(reader[i] as u64);
    }

    // 3. Streaming non-overlapping chunks
    for chunk in reader.chunks(64) {
        for &tok in chunk {
            accum = accum.wrapping_add(tok as u64);
        }
    }

    // 4. Streaming sliding windows
    let win_iter = reader.window_stride(128, 32);
    accum = accum.wrapping_add(win_iter.len() as u64);
    for window in win_iter {
        for &tok in window {
            accum = accum.wrapping_add(tok as u64);
        }
    }

    // 5. Inclusive range slicing and subslices
    for &tok in &reader[10..=50] {
        accum = accum.wrapping_add(tok as u64);
    }
    if let Some(sub) = reader.subslice(100, 50) {
        for &tok in sub {
            accum = accum.wrapping_add(tok as u64);
        }
    }

    std::hint::black_box(accum);

    MEASURING.with(|v| v.set(false));

    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "MmapCorpusReader streaming operations must perform ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "MmapCorpusReader streaming operations must allocate ZERO bytes"
    );

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn vsa_context_64_and_s3_normalization_have_zero_allocations() {
    use uor_r4_core::native_geometric::hopf_metric::UnitS3Q30;
    use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
    use uor_r4_core::native_geometric::vsa::{Codebook, RollingVsaContext};

    let codebook = Codebook::<64>::new(256, 0x1234_5678);
    let mut ctx = RollingVsaContext::<64, 64>::new();
    let token_to_root: Vec<u8> = (0..256).map(|t| (t % 120) as u8).collect();
    let model = ExportedGeometricModel {
        vocab_size: 256,
        token_to_root,
        discrete_tables: Vec::new(),
        discrete_bias: vec![0; 256],
        discrete_s2_readout: vec![[0; 5]; 256],
        discrete_jepa_w_state: [0; 9],
        discrete_jepa_w_token: [0; 9],
        discrete_jepa_bias: [0; 3],
        discrete_jepa_fiber_w_state: [0; 4],
        discrete_jepa_fiber_w_token: [0; 4],
        discrete_jepa_fiber_bias: [0; 2],
        vsa_seed: 0x1234_5678,
        vsa_scale_q15: 500,
        vsa_code_mode: 0,
        hierarchical_codebook: None,
        engram_table: None,
        hierarchical_lattice: None,
    };

    // Warm-up static once-locks outside measurement
    ctx.push(42);
    let _ = model.context_fiber_from_ring(&ctx.ring, ctx.cursor, ctx.length);
    let _ = model.context_vsa_from_ring(&codebook, &ctx.ring, ctx.cursor, ctx.length);
    let _ = UnitS3Q30::normalize([100, 200, 300, 400]);
    ctx.clear();

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut valid = true;
    for i in 0..1_000_u32 {
        let tok = i % 256;
        ctx.push(tok);

        let fiber = model.context_fiber_from_ring(&ctx.ring, ctx.cursor, ctx.length);
        valid &= fiber.base.0[2] >= -1073741824 && fiber.base.0[2] <= 1073741824;

        let vsa = model.context_vsa_from_ring(&codebook, &ctx.ring, ctx.cursor, ctx.length);
        if ctx.len() >= 1 {
            valid &= vsa.count_ones() > 0;
        }

        let raw = [
            (i as i64).wrapping_mul(1001),
            ((i + 1) as i64).wrapping_mul(2003),
            ((i + 2) as i64).wrapping_mul(3007),
            ((i + 3) as i64).wrapping_mul(4009),
        ];
        let norm_s3 = UnitS3Q30::normalize(raw);
        let n_q30 = norm_s3.norm_q30();
        valid &= n_q30 <= 1073741824 && n_q30 >= 1073741823;
    }

    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "64-token VSA context and S3 normalization must perform ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "64-token VSA context and S3 normalization must allocate ZERO heap bytes"
    );
}

#[test]
fn engram_32768_skip_bigram_lookup_and_syntactic_register_transitions_have_zero_allocations() {
    use uor_r4_core::native_geometric::engram::{
        hash_bigram, hash_skip, hash_trigram, EngramTable, DEFAULT_ENGRAM_CAPACITY,
    };
    use uor_r4_core::native_geometric::{Config, Control, Document, Trainer};

    let mut table = EngramTable::with_capacity(DEFAULT_ENGRAM_CAPACITY);
    let key1 = hash_bigram(10, 20);
    let key2 = hash_trigram(5, 10, 20);
    let key_skip1 = hash_skip(15, 20, 0);
    let key_skip2 = hash_skip(15, 20, 1);
    let key_skip3 = hash_skip(15, 20, 3);
    table.insert(key1, &[(30, 16384), (31, 8192)]);
    table.insert(key2, &[(40, 24576), (41, 12288)]);
    table.insert_conditioned(key_skip1, 0, &[(50, 20000), (51, 10000)]);
    table.insert_conditioned(key_skip2, 1, &[(36, 32767)]);
    table.insert_conditioned(key_skip3, 3, &[(60, 15000)]);

    let docs = [Document {
        id: "alloc-test".into(),
        text: "The quick brown fox jumps over the \"lazy\" dog. (Wait, what?) Yes!".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            ..Config::default()
        },
        &docs,
    )
    .unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut session = model.session(Control::default()).unwrap();

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut hits = 0;
    for i in 0..10_000_u32 {
        let w_curr = (i % 256) + 2;
        let w_prev = ((i + 1) % 256) + 2;
        let w_prev2 = ((i + 2) % 256) + 2;
        let cond = (i % 8) as u8;

        if let Some(cands) = table.lookup_bigram(w_prev, w_curr) {
            hits += cands.len();
        }
        if let Some(cands) = table.lookup_trigram(w_prev2, w_prev, w_curr) {
            hits += cands.len();
        }
        if let Some(cands) = table.lookup_skip(w_prev2, w_curr, cond) {
            hits += cands.len();
        }

        // Test syntactic register transition
        let tok = match i % 10 {
            0 => 36, // '"'
            1 => 42, // '('
            2 => 43, // ')'
            3 => 46, // ','
            4 => 48, // '.'
            _ => (i % 50) + 2,
        };
        let _ = session.observe(&model, tok);
        let syn = session.syntactic_state();
        let q = session.quotation_parity();
        let d = session.clause_depth();
        hits += (syn as usize) + (q as usize) + (d as usize);
    }

    MEASURING.with(|v| v.set(false));
    assert!(hits > 0);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "Engram 32,768 skip-bigram lookups and syntactic register updates must have zero heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "Engram 32,768 skip-bigram lookups and syntactic register updates must allocate zero heap bytes"
    );
}

#[test]
fn multi_head_vsa_attention_hot_path_has_zero_allocations() {
    use uor_r4_core::native_geometric::vsa::{Codebook, MultiHeadVsaAttention, RollingVsaContext};

    let codebook = Codebook::<64>::new(256, 0x1234_5678);
    let attn = MultiHeadVsaAttention::<64>::new();
    let mut ctx = RollingVsaContext::<64, 64>::new();

    for i in 0..32_u32 {
        ctx.push((i % 50) + 2);
    }

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut valid = true;
    for step in 0..256_u32 {
        ctx.push((step % 50) + 2);
        let res = ctx.current_multihead_attention(&codebook, &attn);
        valid &= res.active_heads > 0;

        let cand_vec = codebook.get((step % 50) + 2);
        let score = res.score_candidate_q15(&cand_vec);
        valid &= score != i16::MIN;
    }

    MEASURING.with(|v| v.set(false));
    assert!(valid);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "Multi-head VSA attention hot path must have zero heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "Multi-head VSA attention hot path must allocate zero heap bytes"
    );
}
