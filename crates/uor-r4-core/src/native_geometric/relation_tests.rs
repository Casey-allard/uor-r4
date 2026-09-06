//! Causal storage checks; learned complete-answer behavior is evaluated separately.
use super::relation::*;
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueState, ValueWork};
use super::*;

fn atom(name: &str, ordinal: u64) -> WordAtom {
    let mut w = WordAtom {
        len: name.len() as u8,
        end: ordinal,
        byte_end: ordinal,
        ..WordAtom::default()
    };
    w.bytes[..name.len()].copy_from_slice(name.as_bytes());
    w
}
#[test]
fn native_relation_versions_preserve_unrelated_members_and_conflicts() {
    println!(
        "relation_layout record={} state={}",
        std::mem::size_of::<RelationRecord>(),
        std::mem::size_of::<RelationState>()
    );
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom("ada", 8), atom("Rome", 16), 1, &mut work);
    state.commit(atom("cyra", 24), atom("Perth", 32), 1, &mut work);
    state.commit(atom("ada", 40), atom("Dover", 48), 2, &mut work);
    assert_eq!(state.record(1).unwrap().value, atom("Rome", 16));
    assert!(state.directory.contains(&2));
    assert!(state.directory.contains(&3));
    assert!(!state.directory.contains(&1));
    assert_eq!(state.record(3).unwrap().previous, 1);
    assert!(!state.record(3).unwrap().conflict);
    state.commit(atom("ada", 56), atom("Cairo", 64), 1, &mut work);
    assert!(state.record(4).unwrap().conflict);
    state.commit(atom("ada", 72), atom("Dover", 80), 2, &mut work);
    assert!(!state.record(5).unwrap().conflict);
    assert_eq!(state.record(2).unwrap().value, atom("Perth", 32));
    let mut other = state.clone();
    other.commit(atom("cyra", 88), atom("Cairo", 96), 2, &mut work);
    assert_eq!(state.record(2).unwrap().value, atom("Perth", 32));
    assert_eq!(state.next_id, 6);
    for i in 0..24 {
        other.commit(
            atom("extra", 104 + i * 16),
            atom("place", 112 + i * 16),
            2,
            &mut work,
        );
    }
    assert_eq!(other.records.iter().filter(|r| r.id != 0).count(), 16);
    assert!(other.record(1).is_none());
    assert!(work.relations.record_evictions > 0);
}

#[test]
fn native_relation_snapshot_rejects_dangling_or_rewritten_current_versions() {
    let docs = vec![Document {
        id: "relation-shape".into(),
        text: "ada Rome cyra Perth".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut values = ValueState::new(&model);
    values.seen = 1000;
    values.lexemes = Some(LexemeState {
        source_bytes_seen: 1000,
        ..LexemeState::default()
    });
    let mut state = RelationState::default();
    let mut work = ValueWork::default();
    state.commit(atom("ada", 8), atom("Rome", 16), 1, &mut work);
    state.commit(atom("ada", 24), atom("Perth", 32), 2, &mut work);
    assert!(state.validate(&values, &model).is_ok());
    let mut bad = state.clone();
    bad.directory[0] = 1;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.directory[0] = 999;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.records[1].previous = 2;
    assert!(bad.validate(&values, &model).is_err());
    let mut bad = state.clone();
    bad.records[1].conflict = true;
    assert!(bad.validate(&values, &model).is_err());
}

#[test]
fn native_relation_role_features_ignore_payload_renaming_and_global_pose() {
    let docs = vec![Document {
        id: "role-transport".into(),
        text: "in now not holds".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let mut context = vec![
        model.geometry.tokens[7].clone(),
        model.geometry.tokens[11].clone(),
    ];
    context[0].prime = 13;
    context[1].prime = 17;
    let words = [
        atom("oldvalue", 32),
        atom("in", 24),
        atom("now", 16),
        atom("owner", 8),
    ];
    let addr = [3, 13, 17, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // Every offered pair is invariant to its own participant payloads, including
    // pairs where the neighbor feature addresses the other participant.
    for o in 0..4 {
        for v in 0..4 {
            if o == v || (o != 0 && v != 0) {
                continue;
            }
            let mut renamed = words;
            let mut changed_addr = addr;
            for (i, name) in [(o, "differentowner"), (v, "differentvalue")] {
                renamed[i] = atom(name, 80);
                renamed[i].pose = 37;
                renamed[i].phases = [62000; PHASE_CHANNELS];
                changed_addr[i] = 0;
            }
            let a = write_features_with_context(
                &model,
                &words,
                &addr,
                o,
                v,
                Some(&context),
                &mut ValueWork::default(),
            );
            let b = write_features_with_context(
                &model,
                &renamed,
                &changed_addr,
                o,
                v,
                Some(&context),
                &mut ValueWork::default(),
            );
            assert_eq!(a, b);
        }
    }
}

#[test]
fn native_relation_role_path_keeps_context_order_and_direction() {
    let docs = vec![Document {
        id: "role-path-order".into(),
        text: "context".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
    trainer.train_documents(&docs).unwrap();
    let model = trainer.compile().unwrap();
    let g = &model.geometry;
    let (a, b) = (0..g.inverses.len())
        .flat_map(|a| (0..g.inverses.len()).map(move |b| (a, b)))
        .find(|&(a, b)| g.products[g.row_bases[a] + b] != g.products[g.row_bases[b] + a])
        .unwrap();
    let context = [
        TokenGeometry {
            prime: 13,
            leaf: a as u16,
            phases: [8192; PHASE_CHANNELS],
        },
        TokenGeometry {
            prime: 17,
            leaf: b as u16,
            phases: [4096; PHASE_CHANNELS],
        },
    ];
    let words = [
        atom("value", 32),
        atom("contexta", 24),
        atom("contextb", 16),
        atom("owner", 8),
    ];
    let mut addr = [0; 16];
    addr[1] = 13;
    addr[2] = 17;
    let (f, n) = write_features_with_context(
        &model,
        &words,
        &addr,
        3,
        0,
        Some(&context),
        &mut ValueWork::default(),
    );
    let mut reversed = addr;
    reversed.swap(1, 2);
    let (r, m) = write_features_with_context(
        &model,
        &words,
        &reversed,
        3,
        0,
        Some(&context),
        &mut ValueWork::default(),
    );
    let root = |features: &[super::value_types::ValueFeature]| {
        features.iter().find(|f| f.kind == 10).unwrap().a as usize
    };
    assert_ne!(root(&f[..n]), root(&r[..m]));
    let (back, k) = write_features_with_context(
        &model,
        &words,
        &addr,
        0,
        3,
        Some(&context),
        &mut ValueWork::default(),
    );
    assert_eq!(root(&back[..k]), usize::from(g.inverses[root(&f[..n])]));
}
