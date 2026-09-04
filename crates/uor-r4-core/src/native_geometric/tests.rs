use super::*;

fn documents() -> Vec<Document> {
    vec![
        Document {
            id: "prose-red".into(),
            text: "Alice saved a red gem. Much later the vault held a red gem.".into(),
        },
        Document {
            id: "prose-blue".into(),
            text: "Alice saved a blue gem. Much later the vault held a blue gem.".into(),
        },
        Document {
            id: "rust-add".into(),
            text: "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n".into(),
        },
        Document {
            id: "rust-sub".into(),
            text: "fn subtract(a: i32, b: i32) -> i32 {\n    a - b\n}\n".into(),
        },
    ]
}

fn fitted(context: usize) -> Model {
    let documents = documents();
    let mut trainer = Trainer::new(
        Config {
            context_tokens: context,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    trainer.compile().unwrap()
}

#[test]
fn native_codec_preserves_code_whitespace_and_unseen_unicode() {
    let model = fitted(32);
    let source = "fn λ(unknown: &str) {\n\tprintln!(\"🦀 {}\", unknown);\n}\n  ";
    assert_eq!(
        model.decode(&model.encode(source).unwrap()).unwrap(),
        source.as_bytes()
    );
}

#[test]
fn native_resume_and_artifact_reload_are_identical() {
    let docs = documents();
    let mut uninterrupted = Trainer::new(Config::default(), &docs).unwrap();
    uninterrupted.train_documents(&docs).unwrap();
    let mut interrupted = Trainer::new(Config::default(), &docs).unwrap();
    interrupted.train_documents(&docs[..2]).unwrap();
    let mut resumed = Trainer::from_bytes(&interrupted.to_bytes().unwrap()).unwrap();
    resumed.train_documents(&docs[2..]).unwrap();
    let expected = uninterrupted.compile().unwrap().to_bytes().unwrap();
    assert_eq!(expected, resumed.compile().unwrap().to_bytes().unwrap());
    let loaded = Model::from_bytes(&expected).unwrap();
    assert_eq!(loaded.to_bytes().unwrap(), expected);
    assert_eq!(
        loaded
            .generate("fn add(a: i32, b: i32)", 12, Control::Full)
            .unwrap(),
        uninterrupted
            .compile()
            .unwrap()
            .generate("fn add(a: i32, b: i32)", 12, Control::Full)
            .unwrap()
    );
    let mut tampered: serde_json::Value = serde_json::from_slice(&expected).unwrap();
    tampered["config"]["candidate_limit"] = serde_json::json!(256);
    assert!(Model::from_bytes(&serde_json::to_vec(&tampered).unwrap()).is_err());
}

#[test]
fn native_geometry_is_read_by_learned_scores_and_controls_remove_its_features() {
    let model = fitted(32);
    let mut full = model.session(Control::Full).unwrap();
    let mut off = model.session(Control::GeometryDisabled).unwrap();
    for token in [BOS]
        .into_iter()
        .chain(model.encode("Alice saved a red gem.").unwrap())
    {
        full.observe(&model, token).unwrap();
        off.observe(&model, token).unwrap();
    }
    let enabled = full.predict(&model).unwrap();
    let disabled = off.predict(&model).unwrap();
    assert!(enabled.geometric_rows > 0);
    assert_eq!(disabled.geometric_rows, 0);
    assert_ne!(full.candidates(), off.candidates());
    assert_eq!(full.state().h4_index, off.state().h4_index);
    assert_eq!(full.state().phase_turns_u16, off.state().phase_turns_u16);
    assert!(!Feature { kind: 5, value: 0 }.admitted(Control::ZetaDisabled));
    assert!(!Feature { kind: 8, value: 0 }.admitted(Control::ZetaDisabled));
    assert!(Feature { kind: 16, value: 0 }.admitted(Control::ZetaDisabled));
    assert!(Feature { kind: 23, value: 0 }.admitted(Control::ZetaDisabled));
}

#[test]
fn native_context_eviction_matches_exact_ordered_tail_and_has_fixed_storage() {
    let model = fitted(32);
    let tokens = model
        .encode("Alice saved a red gem. Much later the vault held a blue gem. ")
        .unwrap();
    let mut session = model.session(Control::Full).unwrap();
    let original_bytes = session.state().ring_storage_bytes;
    let mut observed = Vec::new();
    for token in tokens.iter().cycle().take(100) {
        session.observe(&model, *token).unwrap();
        observed.push(*token);
    }
    let mut tail = model.session(Control::Full).unwrap();
    for token in &observed[observed.len() - 32..] {
        tail.observe(&model, *token).unwrap();
    }
    assert_eq!(session.state().h4_index, tail.state().h4_index);
    assert_eq!(
        session.state().phase_turns_u16,
        tail.state().phase_turns_u16
    );
    assert_eq!(
        session.state().paired_h4_coefficients,
        tail.state().paired_h4_coefficients
    );
    assert_eq!(
        session.state().radial_squared_zphi_numerator,
        tail.state().radial_squared_zphi_numerator
    );
    assert_eq!(session.state().ring_storage_bytes, original_bytes);
    assert_eq!(session.state().retained_tokens, 32);
    assert_eq!(session.work.evictions, 68);
    assert!(session.predict(&model).unwrap().candidate_count <= model.config().candidate_limit);
}

#[test]
fn native_evaluation_refuses_id_and_exact_text_leakage() {
    let model = fitted(32);
    assert!(model.evaluate(&documents(), Control::Full).is_err());
    let mut copied = documents()[0].clone();
    copied.id = "renamed-heldout".into();
    assert!(model.evaluate(&[copied], Control::Full).is_err());
    let report = model
        .evaluate(
            &[Document {
                id: "new".into(),
                text: "Alice saved a green gem.".into(),
            }],
            Control::Full,
        )
        .unwrap();
    assert!(report.positions > 0);
    assert!(report.candidate_hits <= report.positions);
}

#[test]
fn native_session_restores_after_eviction_and_rejects_state_tampering() {
    let model = fitted(32);
    let tokens = model.encode("Alice saved a red gem. ").unwrap();
    let mut session = model.session(Control::Full).unwrap();
    for &token in tokens.iter().cycle().take(81) {
        session.observe(&model, token).unwrap();
    }
    let snapshot = session.checkpoint().unwrap();
    let mut replay = model.restore_session(&snapshot).unwrap();
    assert_eq!(session.state(), replay.state());
    assert_eq!(session.checkpoint().unwrap(), replay.checkpoint().unwrap());
    for _ in 0..12 {
        let left = session.predict(&model).unwrap();
        let right = replay.predict(&model).unwrap();
        assert_eq!(left, right);
        session.observe(&model, left.token).unwrap();
        replay.observe(&model, right.token).unwrap();
    }
    let mut bad: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    bad["phases"][0] = serde_json::json!(bad["phases"][0].as_u64().unwrap() ^ 1);
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
}

#[test]
fn native_readout_fits_separate_labels_and_preserves_fixed_baseline() {
    let model = fitted(32);
    let baseline = model.to_bytes().unwrap();
    assert!(model
        .fit_readout(&documents(), ReadoutFitConfig::default())
        .is_err());
    let fit = vec![Document { id: "readout-only".into(), text:
        "Alice saved a blue gem. Alice saved a red gem. fn add(a: i32, b: i32) -> i32 {\n    a - b\n}\n".repeat(3) }];
    let config = ReadoutFitConfig {
        max_positions: 256,
        epochs: 3,
        max_queries: 8,
    };
    let (learned, report) = model.fit_readout(&fit, config).unwrap();
    assert_eq!(model.readout_version(), "fixed_v1");
    assert_eq!(model.to_bytes().unwrap(), baseline);
    assert_eq!(learned.readout_version(), "learned_mixture_v1");
    assert_ne!(learned.artifact_cid(), model.artifact_cid());
    assert!(report.target_in_shortlist > 0);
    assert!(report.candidate_cross_entropy_before.is_finite());
    assert!(report.candidate_cross_entropy_after.is_finite());
    assert!(report
        .global_weights_eighths
        .iter()
        .all(|&weight| weight <= 16));
    assert!(learned.evaluate(&fit, Control::Full).is_err());
    let loaded = Model::from_bytes(&learned.to_bytes().unwrap()).unwrap();
    assert_eq!(
        learned.generate("Alice saved a", 8, Control::Full).unwrap(),
        loaded.generate("Alice saved a", 8, Control::Full).unwrap()
    );
    assert_eq!(
        learned.to_bytes().unwrap(),
        model
            .fit_readout(&fit, config)
            .unwrap()
            .0
            .to_bytes()
            .unwrap()
    );
}

#[test]
fn native_memory_read_learns_query_relative_copy_and_restores_after_eviction() {
    let catalog = [Document { id: "memory-codec-catalog".into(), text:
        "The orb is red. The cube is blue. The key is green. The coin is gold. Now the orb is gold. Question: What color is the orb? Answer: red. wind moves.\n".into() }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 64,
            candidate_limit: 16,
            max_lexical_pieces: 256,
            ..Config::default()
        },
        &catalog,
    )
    .unwrap();
    trainer.train_documents(&catalog).unwrap();
    let baseline = trainer.compile().unwrap();
    let original_bytes = baseline.to_bytes().unwrap();
    assert!(!String::from_utf8_lossy(&original_bytes).contains("memory_read"));
    let names = ["orb", "cube", "key", "coin"];
    let colors = ["red", "blue", "green", "gold"];
    let mut fit = Vec::new();
    for index in 0..128 {
        let first = names[index % 4];
        let second = names[(index % 4 + 1) % 4];
        let old = colors[(index / 4) % 4];
        let replacement = colors[(index / 16) % 4];
        let other = colors[((index / 4) + 1) % 4];
        let (query, answer) = if index < 64 {
            (first, replacement)
        } else {
            (second, other)
        };
        fit.push(Document { id: format!("memory-fit-{index}"), text: format!(
            "The {first} is {old}. The {second} is {other}. Now the {first} is {replacement}. Question: What color is the {query}? Answer: {answer}.\n") });
    }
    let (model, report) = baseline
        .fit_memory_read(
            &fit,
            MemoryReadFitConfig {
                max_positions: 2048,
                epochs: 12,
                ..MemoryReadFitConfig::default()
            },
        )
        .unwrap();
    assert!(report.target_in_memory > 0);
    assert_eq!(report.cue_identity, memory_types::EXACT_CUE_SCHEMA);
    assert_eq!(
        model.memory_cue_identity(),
        Some(memory_types::EXACT_CUE_SCHEMA)
    );
    assert!(model.memory_read.as_ref().unwrap().cue_aliases.is_none());
    assert!(report.candidate_cross_entropy_after < report.candidate_cross_entropy_before);
    assert!(model.evaluate(&fit, Control::Full).is_err());
    assert_eq!(baseline.to_bytes().unwrap(), original_bytes);
    let loaded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let prompts = [
        ("The orb is green. The key is blue. Now the orb is red. Question: What color is the orb? Answer:", " red"),
        ("The orb is green. The key is blue. Now the orb is gold. Question: What color is the orb? Answer:", " gold"),
    ];
    let mut predictions = Vec::new();
    for (prompt, expected) in prompts {
        let mut session = loaded.session(Control::Full).unwrap();
        // Force several evictions before the supplied facts; replay must be
        // independent of the removed prefix's H4/phase gauge.
        for token in loaded.encode(&"wind moves. ".repeat(80)).unwrap() {
            session.observe(&loaded, token).unwrap();
        }
        for token in loaded.encode(prompt).unwrap() {
            session.observe(&loaded, token).unwrap();
        }
        let prediction = session.predict(&loaded).unwrap();
        assert_eq!(
            loaded.decode(&[prediction.token]).unwrap(),
            expected.as_bytes()
        );
        predictions.push(prediction.token);
        let mut restored = loaded
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        assert_eq!(session.state(), restored.state());
        for _ in 0..12 {
            assert_eq!(
                session.predict(&loaded).unwrap(),
                restored.predict(&loaded).unwrap()
            );
            assert_eq!(session.candidates(), restored.candidates());
            session.observe(&loaded, prediction.token).unwrap();
            restored.observe(&loaded, prediction.token).unwrap();
        }
        let mut off = loaded.session(Control::MemoryDisabled).unwrap();
        let mut original = baseline.session(Control::Full).unwrap();
        for token in baseline.encode(prompt).unwrap() {
            off.observe(&loaded, token).unwrap();
            original.observe(&baseline, token).unwrap();
        }
        assert_eq!(
            off.predict(&loaded).unwrap(),
            original.predict(&baseline).unwrap()
        );
        assert_eq!(off.candidates(), original.candidates());
    }
    assert_ne!(predictions[0], predictions[1]);
}

#[test]
fn native_legacy_session_checkpoint_defaults_new_memory_counters() {
    let model = fitted(32);
    let mut session = model.session(Control::Full).unwrap();
    for token in model.encode("Alice saved a red gem.").unwrap() {
        session.observe(&model, token).unwrap();
    }
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    checkpoint["work"]
        .as_object_mut()
        .unwrap()
        .retain(|key, _| !key.starts_with("memory_"));
    let restored = model
        .restore_session(&serde_json::to_vec(&checkpoint).unwrap())
        .unwrap();
    assert_eq!(restored.state(), session.state());
    assert_eq!(restored.work, session.work);
}

#[test]
fn native_memory_cue_aliases_preserve_outputs_and_validate_complete_mapping() {
    let catalog = [Document {
        id: "cue-catalog".into(),
        text: "alpha beta. alpha beta.\nalpha beta.\talpha beta. Alpha beta.! \n! gamma.".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &catalog).unwrap();
    trainer.train_documents(&catalog).unwrap();
    let baseline = trainer.compile().unwrap();
    let (model, report) = baseline
        .fit_memory_read_with_word_cues(
            &[Document {
                id: "cue-fit".into(),
                text: "alpha beta. alpha beta. alpha beta.".into(),
            }],
            MemoryReadFitConfig {
                query_tokens: 1,
                source_offsets: 1,
                candidate_limit: 8,
                max_positions: 32,
                epochs: 1,
                ..MemoryReadFitConfig::default()
            },
        )
        .unwrap();
    assert!(report.aliased_lexical_tokens >= 3);
    assert_eq!(model.memory_cue_identity(), Some(memory_types::CUE_SCHEMA));
    let token = |text: &str| {
        let encoded = model.encode(text).unwrap();
        assert_eq!(encoded.len(), 1, "{text:?}");
        encoded[0]
    };
    let bare = token("alpha");
    let memory = model.memory_read.as_ref().unwrap();
    let aliases = &memory.cue_aliases.as_ref().unwrap().representatives;
    for text in ["alpha", " alpha", "\nalpha", "\talpha"] {
        assert_eq!(aliases[token(text) as usize], bare);
        assert_eq!(model.decode(&[token(text)]).unwrap(), text.as_bytes());
    }
    assert_ne!(aliases[token(" Alpha") as usize], bare);
    for text in ["!", " \n!"] {
        assert_eq!(aliases[token(text) as usize], token(text));
    }
    for id in 0..LEXICAL_BASE {
        assert_eq!(aliases[id as usize], id);
    }
    let bytes = model.to_bytes().unwrap();
    assert_eq!(
        Model::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
    let mut tampered = model.clone();
    tampered
        .memory_read
        .as_mut()
        .unwrap()
        .cue_aliases
        .as_mut()
        .unwrap()
        .representatives[bare as usize] = token(" beta");
    tampered.refresh_identity().unwrap();
    assert!(Model::from_bytes(&tampered.to_bytes().unwrap()).is_err());

    // Old artifacts omit the table and retain exact-cue behavior and bytes.
    let mut legacy = model.clone();
    legacy.memory_read.as_mut().unwrap().schema = memory_types::LEGACY_MEMORY_SCHEMA.into();
    legacy.memory_read.as_mut().unwrap().cue_aliases = None;
    legacy.refresh_identity().unwrap();
    let legacy_bytes = legacy.to_bytes().unwrap();
    assert!(!String::from_utf8_lossy(&legacy_bytes).contains("cue_aliases"));
    assert_eq!(
        Model::from_bytes(&legacy_bytes)
            .unwrap()
            .to_bytes()
            .unwrap(),
        legacy_bytes
    );
    let prefix = model.encode("alpha beta.\nalpha").unwrap();
    let expected = token(" beta");
    for (artifact, expected_admission) in [(&model, true), (&legacy, false)] {
        let operator = artifact.memory_read.as_ref().unwrap();
        let mut state = memory_types::MemoryState::new(artifact, operator);
        let mut work = Work::default();
        for &value in &prefix {
            state.observe(artifact, operator, value, &mut work);
        }
        state.collect(artifact, operator, Control::Full, &mut work);
        assert_eq!(
            state
                .candidates
                .iter()
                .any(|candidate| candidate.token == expected),
            expected_admission
        );
        assert_eq!(work.memory_cue_reads > 0, expected_admission);
    }
}
