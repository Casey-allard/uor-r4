//! Direct allocation census for successful native session observe/predict.
//! Fitting, model construction, tokenization, session allocation, and reports
//! are deliberately outside the measured section. TLS excludes libtest noise.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_core::native_geometric::{
    Config, Control, Document, MemoryReadFitConfig, MemoryReadSchedule, MemoryReadTrainer,
    ReadoutFitConfig, Trainer, BOS,
};

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
    for (name, source) in [
        ("native runtime", kernel),
        ("native Feature helpers", features),
        ("native memory runtime", memory),
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
    assert!(
        memory_fit.target_in_memory > 0,
        "fixture must exercise fitted memory alternatives"
    );
    let tokens = model.encode(&documents[0].text).unwrap();
    assert!(!tokens.is_empty());
    for (readout, model) in [
        ("fixed_v1", model),
        ("learned_v1", learned),
        ("learned_memory_exact", exact_memory),
        ("learned_memory_with_aliases", with_memory),
        ("query_context_memory_with_aliases", query_context_memory),
        ("occurrence_composition", occurrence_memory),
    ] {
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
        ] {
            let mut session = model.session(control).unwrap();
            session.observe(&model, BOS).unwrap();
            let before = session.state();
            ALLOCATIONS.with(|count| count.set(0));
            BYTES.with(|count| count.set(0));
            MEASURING.with(|enabled| enabled.set(true));
            let result = (|| {
                for index in 0..1024 {
                    session.predict(&model)?;
                    session.observe(&model, tokens[index % tokens.len()])?;
                }
                Ok::<_, uor_r4_core::native_geometric::Error>(())
            })();
            MEASURING.with(|enabled| enabled.set(false));
            let allocations = ALLOCATIONS.with(Cell::get);
            let bytes = BYTES.with(Cell::get);
            result.unwrap();
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
                    session.work.memory_candidates <= 1024 * memory_config.candidate_limit as u64
                );
                assert!(
                    session.work.memory_index_reads
                        <= 1025
                            * (memory_config.source_offsets
                                * (memory_config.postings_per_address - 1))
                                as u64
                            + 1024 * memory_config.candidate_limit as u64
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
                if word_cues {
                    assert!(session.work.memory_cue_reads > 0);
                } else {
                    assert_eq!(session.work.memory_cue_reads, 0);
                }
                assert!(
                    session.work.memory_cue_reads
                        <= 1025 * memory_config.source_offsets as u64
                            + 1024 * memory_config.candidate_limit as u64
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
                after.ring_storage_bytes + after.index_storage_bytes + after.candidate_storage_bytes
            } else {
                assert!(model.memory_read_version().is_none());
                assert_eq!(session.work.memory_candidates, 0);
                assert_eq!(session.work.memory_cue_reads, 0);
                0
            };
            println!("{readout} {control:?}: decisions=1024 evictions=993 allocations={allocations} bytes={bytes} ring_bytes={} candidate_bytes={} offers={} evaluations={} memory_bytes={memory_bytes} memory_candidates={} memory_score_lookups={} memory_cue_reads={}",
            after.ring_storage_bytes, after.candidate_storage_bytes, session.work.candidate_offers, session.work.candidate_evaluations,
            session.work.memory_candidates,session.work.memory_score_lookups,session.work.memory_cue_reads);
        }
    }
}
