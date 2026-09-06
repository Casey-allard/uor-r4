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
