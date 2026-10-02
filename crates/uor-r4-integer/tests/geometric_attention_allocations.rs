//! Allocation census for the actual persistent integer attention session.
//!
//! This external test binary owns its System allocator wrapper. The portable
//! integer library retains forbid(unsafe_code). Unsafe below only forwards the
//! GlobalAlloc pointer/layout contract; it implements no attention arithmetic.
//! Admitted fixture construction, TLS initialization, warm-up, assertions and
//! reporting are outside measurement. Thread-local tracking excludes libtest
//! and other test-thread traffic. This is a synthetic-path allocation census,
//! not a model-quality, compiled-opcode or complete-model serving result.

#![deny(unsafe_op_in_unsafe_fn)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use native::geometric_attention::{
    NativeAttentionComponents, NativeAttentionSession, HEADS, LANES, LANES_PER_HEAD, MAX_CONTEXT,
};
use native::geometric_composition::NativeGeometricComposition;
use native::geometric_context::NativeContextTables;
use native::geometric_context_q4::{ContextQ4Config, NativeContextQ4};
use native::geometric_event::NativeEventTables;
use native::geometric_no_read::{pack_coefficients, NativeGeometricNoRead, NoReadConfig};
use native::geometric_potential::{
    NativePotentialTables, CONTENT_PRESENCE_OFFSET, CONTEXT_PRESENCE_OFFSET, ENTRIES_PER_LANE,
};
use native::geometric_span::{SpanAction, TokenActionDictionary};
use native::geometric_value::ValueState;
use native::geometric_value_producer::NativeValueProducer;
use native::geometric_value_q4::{NativeValueQ4, ValueQ4Config};
use native::h4_tables::HistoricalH4Tables;
use uor_r4_integer as native;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    allocations: usize,
    allocated_bytes: usize,
    reallocations: usize,
    deallocations: usize,
}
const ZERO: Counts = Counts {
    allocations: 0,
    allocated_bytes: 0,
    reallocations: 0,
    deallocations: 0,
};
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static COUNTS: Cell<Counts> = const { Cell::new(ZERO) };
}

fn record(bytes: usize, reallocation: bool, deallocation: bool) {
    let _ = MEASURING.try_with(|enabled| {
        if enabled.get() {
            let _ = COUNTS.try_with(|counter| {
                let mut count = counter.get();
                if deallocation {
                    count.deallocations = count.deallocations.saturating_add(1);
                } else {
                    count.allocations = count.allocations.saturating_add(1);
                    count.allocated_bytes = count.allocated_bytes.saturating_add(bytes);
                    if reallocation {
                        count.reallocations = count.reallocations.saturating_add(1);
                    }
                }
                counter.set(count);
            });
        }
    });
}

struct CountingAllocator;
// SAFETY: Every allocation operation delegates its unchanged pointer/layout
// contract to System. TLS accounting never dereferences an allocation pointer.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc's caller supplies a valid allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size(), false, false);
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Same unchanged layout contract as alloc.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size(), false, false);
        }
        pointer
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: The caller supplies System's live pointer, its layout, and a
        // valid new size; all are forwarded without modification.
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        if !replacement.is_null() {
            record(size, true, false);
        }
        replacement
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The live pointer and matching layout are passed unchanged.
        unsafe { System.dealloc(pointer, layout) };
        record(0, false, true);
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Also disarm during unwinding so panic diagnostics cannot contaminate later
/// measurements. Normal assertions and error rendering occur after stop().
struct Measurement;
impl Measurement {
    fn start() -> Self {
        MEASURING.with(|enabled| enabled.set(false));
        COUNTS.with(|counter| counter.set(ZERO));
        MEASURING.with(|enabled| enabled.set(true));
        Self
    }
    fn stop(self) -> Counts {
        MEASURING.with(|enabled| enabled.set(false));
        COUNTS.with(Cell::get)
    }
}
impl Drop for Measurement {
    fn drop(&mut self) {
        let _ = MEASURING.try_with(|enabled| enabled.set(false));
    }
}

// Only construction inputs are reused from geometric_attention.rs's admitted
// fixture. The runtime implementation is exercised through its public API.
struct Fixture {
    events: NativeEventTables,
    context: NativeContextTables,
    span: TokenActionDictionary,
    potential: NativePotentialTables,
    values: NativeValueProducer,
    null: NativeGeometricNoRead,
    composition: NativeGeometricComposition,
    geometry: HistoricalH4Tables,
    ages: Vec<i64>,
    exp: Vec<u32>,
}
impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let payload = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))?;
        let geometry = HistoricalH4Tables::from_bytes(&payload)?;
        let mut transition = vec![0; 4 * 128];
        let mut event = vec![0; 4 * 4];
        for (token, action) in [1, 2, 3, 0].into_iter().enumerate() {
            transition[token * 128 + 1] = 1;
            event[token * 4 + action] = 1;
        }
        let events = NativeEventTables::new(
            4,
            1,
            &transition,
            &vec![0; 128 * 128],
            None,
            &event,
            &vec![0; 128 * 4],
        )?;
        let context_cfg = ContextQ4Config {
            vocab_size: 4,
            heads: 2,
            lanes_per_head: 4,
        };
        let mut coefficients = Vec::new();
        for (name, shape) in context_cfg.coefficient_shapes()? {
            let mut q = vec![0i8; shape.iter().product()];
            if name.starts_with("token_") {
                let classes = if name == "token_category" { 33 } else { 120 };
                for (row, scores) in q.chunks_exact_mut(classes).enumerate() {
                    let token = row / LANES;
                    let winner = match name.as_str() {
                        "token_transition" if token == 1 => 3,
                        "token_category" if token == 2 => 0,
                        "token_category" => 17,
                        _ => 1,
                    };
                    scores[winner] = 4;
                }
            }
            coefficients.extend(q);
        }
        let context =
            NativeContextQ4::new(context_cfg, &pack_coefficients(&coefficients)?)?.into_native()?;
        let mut dictionary = vec![1; 4 * LANES];
        dictionary[LANES..2 * LANES].fill(3);
        let span = TokenActionDictionary::new(LANES, &dictionary)?;
        let mut potentials = vec![0; HEADS * LANES_PER_HEAD * ENTRIES_PER_LANE];
        potentials[CONTENT_PRESENCE_OFFSET + 2] = 1 << 24;
        potentials[LANES_PER_HEAD * ENTRIES_PER_LANE + CONTEXT_PRESENCE_OFFSET + 3] = 1 << 24;
        let potential = NativePotentialTables::new(HEADS, LANES_PER_HEAD, &potentials)?;
        let value_cfg = ValueQ4Config {
            vocab_size: 4,
            heads: 2,
            latent_lanes_per_head: 4,
        };
        let mut coefficients = Vec::new();
        for (name, shape) in value_cfg.coefficient_shapes()? {
            let mut q = vec![0i8; shape.iter().product()];
            if name == "token_root" || name == "token_category" {
                let classes = if name == "token_root" { 120 } else { 32 };
                for (row, scores) in q.chunks_exact_mut(classes).enumerate() {
                    let token = row / 16;
                    let atom = row % 2;
                    let winner = if name == "token_root" {
                        if token == 3 && atom == 1 {
                            0
                        } else {
                            1
                        }
                    } else if token == 2 || (atom == 1 && token != 3) {
                        0
                    } else {
                        17
                    };
                    scores[winner] = 4;
                }
            }
            coefficients.extend(q);
        }
        let values =
            NativeValueQ4::new(value_cfg, &pack_coefficients(&coefficients)?)?.into_native()?;
        let null_cfg = NoReadConfig {
            vocabulary: 4,
            heads: 2,
            latent_lanes_per_head: 4,
        };
        let mut q = vec![0; null_cfg.coefficient_count()];
        q[null_cfg.coefficients_per_head()] = 4;
        let null = NativeGeometricNoRead::new(null_cfg, &pack_coefficients(&q)?)?;
        let mut gains = vec![0; 128];
        for h in 0..2 {
            for o in 0..8 {
                gains[((h * 8 + o) * 4) * 2] = 4;
            }
        }
        let composition = NativeGeometricComposition::new(
            &[1; 128],
            &[1; 128],
            &native::geometric_composition::pack_gains(&gains)?,
            &payload,
        )?;
        // Monotone integer-only fixture, not an exponential-accuracy claim.
        let exp = (0..native::geometric_read::EXP_TABLE_LEN)
            .map(|i| ((1u64 << 31) >> (i / 256).min(32)) as u32)
            .collect();
        Ok(Self {
            events,
            context,
            span,
            potential,
            values,
            null,
            composition,
            geometry,
            ages: vec![0; HEADS * MAX_CONTEXT],
            exp,
        })
    }
    fn components(&self) -> NativeAttentionComponents<'_> {
        NativeAttentionComponents {
            events: &self.events,
            context: &self.context,
            span: &self.span,
            potential: &self.potential,
            values: &self.values,
            no_read: &self.null,
            composition: &self.composition,
            geometry: &self.geometry,
            age_q24: &self.ages,
            age_context: MAX_CONTEXT,
            exp_q31: &self.exp,
        }
    }
}

#[test]
fn persistent_attention_successful_push_and_reset_have_zero_allocations() -> TestResult {
    let fixture = Fixture::new()?;
    let mut session = NativeAttentionSession::new(fixture.components(), MAX_CONTEXT)?;

    // Initialize TLS and traverse every fixture event/value branch before the
    // census. Construction and first-use costs are deliberately excluded.
    MEASURING.with(|enabled| enabled.set(false));
    COUNTS.with(|counter| counter.set(ZERO));
    for token in [0, 1, 2, 3] {
        std::hint::black_box(session.push(token)?);
    }
    session.reset();

    let mut successful_pushes = 0usize;
    let mut resets = 0usize;
    let mut functional = true;
    let mut nonzero_rows = 0usize;
    let mut explicit_zero_rows = 0usize;
    let mut cancellation_rows = 0usize;
    let mut checksum = 0i64;
    let mut failure = None;
    let mut first_output = None;
    let measurement = Measurement::start();
    'runs: for run in 0..2 {
        for position in 0..MAX_CONTEXT {
            // OPEN, APPEND, COMMIT, then HOLD to full capacity. The committed
            // span first appears at position3; HOLD's two atoms cancel exactly.
            let token = if position < 3 { position } else { 3 };
            let step = match session.push(std::hint::black_box(token)) {
                Ok(step) => step,
                Err(error) => {
                    failure = Some(error);
                    break 'runs;
                }
            };
            successful_pushes += 1;
            functional &= step.position == position && step.values.occurrence_valid;
            functional &= step.heads.iter().all(|head| {
                head.occurrence_weights_q31.len() == position + 1
                    && head.total_weight_q31 > 0
                    && head.occurrence_weights_q31[position] > 0
            });
            let expected_event = match position {
                0 => SpanAction::Open,
                1 => SpanAction::Append,
                2 => SpanAction::Commit,
                _ => SpanAction::Hold,
            };
            functional &= step.event.event == expected_event;
            if position < 3 {
                functional &= step.held_before.is_none();
                functional &= step.content_codes.iter().all(|code| !code.present());
            } else {
                functional &= step
                    .held_before
                    .is_some_and(|codes| codes.iter().all(|code| code.index() == 3));
            }
            if position < 2 {
                let nonzero = step.values.values_q16.iter().any(|&x| x != 0);
                functional &= nonzero;
                nonzero_rows += usize::from(nonzero);
            } else if position == 2 {
                let zero = step
                    .values
                    .packets
                    .iter()
                    .flatten()
                    .all(|packet| packet.state() == ValueState::PresentZero)
                    && step.values.values_q16.iter().all(|&x| x == 0);
                functional &= zero;
                explicit_zero_rows += usize::from(zero);
            } else {
                let cancelled = step
                    .values
                    .packets
                    .iter()
                    .flatten()
                    .all(|packet| packet.state() == ValueState::PresentNonzero)
                    && step.values.values_q16.iter().all(|&x| x == 0)
                    && step.composed_q16.iter().flatten().all(|&x| x == 0);
                functional &= cancelled;
                cancellation_rows += usize::from(cancelled);
            }
            if position == 0 {
                if run == 0 {
                    first_output = Some(*step.output_q16);
                } else {
                    functional &= first_output.as_ref() == Some(step.output_q16);
                }
            }
            checksum ^= std::hint::black_box(step.output_q16[0]);
        }
        functional &= session.len() == MAX_CONTEXT;
        session.reset();
        resets += 1;
        functional &= session.is_empty() && session.last().is_none();
    }
    let counts = measurement.stop();

    // Functional completion and allocator results are separate obligations.
    // Nothing below executes with tracking enabled, including error formatting.
    assert!(failure.is_none(), "measured push failed: {failure:?}");
    assert!(
        functional,
        "the measured branches did not retain their declared behavior"
    );
    assert_eq!(successful_pushes, 2 * MAX_CONTEXT);
    assert_eq!(resets, 2);
    assert_eq!(nonzero_rows, 4);
    assert_eq!(explicit_zero_rows, 2);
    assert_eq!(cancellation_rows, 2 * (MAX_CONTEXT - 3));
    std::hint::black_box(checksum);
    eprintln!(
        "persistent attention allocation census: pushes={successful_pushes}, resets={resets}, nonzero={nonzero_rows}, present_zero={explicit_zero_rows}, cancelled_nonzero={cancellation_rows}, counts={counts:?}"
    );
    assert_eq!(counts.allocations, 0, "successful push/reset allocated");
    assert_eq!(
        counts.allocated_bytes, 0,
        "successful push/reset allocated bytes"
    );
    assert_eq!(counts.reallocations, 0, "successful push/reset reallocated");
    assert_eq!(
        counts.deallocations, 0,
        "successful push/reset freed heap storage"
    );
    Ok(())
}
