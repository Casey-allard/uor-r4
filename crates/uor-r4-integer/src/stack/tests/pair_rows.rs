//! claude-384: the pair rows in a served stack's artifact.

use serde_json::json;

use super::super as stack;
use super::{data_bytes, drawn_address, served_read_trunk, with_header, IntegerStackModel};

/// Pair rows of three pairs over the stack's vocabulary at `width`, the rows drawn from a small
/// integer walk.
fn drawn_pair_rows(vocab: usize, width: usize) -> stack::PairRowsImage {
    let primes = crate::fbs_pair_rows::token_primes(vocab).expect("the stack's vocabulary");
    let mut addresses = vec![
        primes[1] * primes[2],
        primes[2] * primes[5],
        primes[3] * primes[7],
    ];
    addresses.sort_unstable();
    let rows = (0..addresses.len() * width)
        .map(|i| ((i * 37) % 401) as i16 - 200)
        .collect();
    let marks = stack::PairMarks {
        user: vec![1, 2],
        assistant: vec![3, 4],
    };
    stack::PairRowsImage::checked(vocab, width, marks, addresses, rows)
        .expect("pair rows of the stack's size")
}

/// Pair rows round-trip. A stack without them inspects with no `pair_rows` key and reads no
/// image. Added, the image read back equals the one written bit for bit, under `lut-stack/15`,
/// every earlier section keeps its bytes, and inspect shows the record.
#[test]
fn pair_rows_round_trip_and_inspect() {
    let (_, plain, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    for (name, bytes) in [("no reads", &plain), ("reads", &with_read)] {
        assert_eq!(
            stack::read_pair_rows_uorlut01(bytes).expect("parse"),
            None,
            "{name}"
        );
        let before = stack::inspect_uorlut01(bytes).expect("inspect");
        let written = serde_json::to_value(&before).expect("the metadata is written");
        assert!(
            written.get("pair_rows").is_none(),
            "{name}: a stack without pair rows inspects as before"
        );
        let image = drawn_pair_rows(before.shape.vocab, before.shape.width);
        let out = stack::add_pair_rows_uorlut01(bytes, &image).expect("the rows are added");
        let back = stack::read_pair_rows_uorlut01(&out)
            .expect("parse")
            .expect("the rows are read");
        assert_eq!(back, image, "{name}");
        assert_eq!(back.to_bytes(), image.to_bytes(), "{name}: bit for bit");
        assert!(IntegerStackModel::parse(&out).is_ok(), "{name}");
        let after = stack::inspect_uorlut01(&out).expect("inspect");
        assert_eq!(after.schema, stack::STACK_FBS_PAIR_ROWS_SCHEMA, "{name}");
        assert_eq!(
            after.pair_rows,
            Some(stack::StackPairRows::of(&image)),
            "{name}"
        );
        let shown = serde_json::to_value(&after).expect("the metadata is written");
        assert_eq!(
            shown["pair_rows"]["sha256"],
            json!(image.sha256()),
            "{name}"
        );
        assert_eq!(
            shown["pair_rows"]["row_exp"],
            json!(stack::PAIR_ROW_EXP),
            "{name}"
        );
        let (old, new) = (data_bytes(bytes), data_bytes(&out));
        assert_eq!(
            new[..old.len()],
            old[..],
            "{name}: every earlier section keeps its bytes"
        );
    }
}

/// The values table of the wrong length is refused.
#[test]
fn pair_rows_refuse_a_short_values_table() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    // A header rewritten with no edit is still read: a refusal below is the edit's own.
    assert!(stack::read_pair_rows_uorlut01(&with_header(&out, |_| {}))
        .expect("read")
        .is_some());
    let short = with_header(&out, |h| {
        for t in h["tables"].as_array_mut().unwrap() {
            if t["name"] == json!("pair_rows.values") {
                let len = t["len"].as_u64().unwrap() - shape.width as u64;
                t["len"] = json!(len);
                t["span"]["bytes"] = json!(len * 2);
            }
        }
    });
    assert!(stack::read_pair_rows_uorlut01(&short).is_err());
}

/// A record whose hash is not the rows file's is refused.
#[test]
fn pair_rows_refuse_a_changed_sha256() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    let hashed = with_header(&out, |h| h["pair_rows"]["sha256"] = json!("00".repeat(32)));
    assert!(
        stack::inspect_uorlut01(&hashed).is_ok(),
        "the header itself still parses"
    );
    assert!(stack::read_pair_rows_uorlut01(&hashed).is_err());
}

/// A values table with one byte changed is refused: the record's hash covers the rows.
#[test]
fn pair_rows_refuse_a_changed_value() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    let declared = u64::from_le_bytes(out[8..16].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&out[16..16 + declared]).unwrap();
    let values = header["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == json!("pair_rows.values"))
        .expect("the values table");
    let at =
        (16 + declared).div_ceil(64) * 64 + values["span"]["offset"].as_u64().unwrap() as usize;
    assert!(values["span"]["bytes"].as_u64().unwrap() > 0);
    let mut flipped = out.clone();
    flipped[at] ^= 1;
    let refused = stack::read_pair_rows_uorlut01(&flipped).expect_err("a changed value");
    assert!(
        refused
            .to_string()
            .contains("pair rows are the rows file the record hashes"),
        "{refused}"
    );
    assert!(IntegerStackModel::parse(&flipped).is_err());
}

/// The record is read strictly and with its own schema: it is refused under `/11` and under the
/// `/6` and under the stack's own earlier schema, with an unknown key, or at another exponent.
#[test]
fn pair_rows_record_and_schema_come_together() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let before = stack::inspect_uorlut01(&plain).expect("inspect");
    let image = drawn_pair_rows(before.shape.vocab, before.shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    let earlier = before.schema.as_str();
    for schema in [
        stack::STACK_FBS_ADDRESS_SCHEMA,
        stack::STACK_FBS_HAMMING_SCHEMA,
        earlier,
    ] {
        let bytes = with_header(&out, |h| h["schema"] = json!(schema));
        assert!(
            stack::inspect_uorlut01(&bytes).is_err(),
            "the record under {schema}"
        );
        assert!(
            IntegerStackModel::parse(&bytes).is_err(),
            "the record under {schema}"
        );
    }
    let unknown = with_header(&out, |h| h["pair_rows"]["extra"] = json!(1));
    assert!(
        stack::inspect_uorlut01(&unknown).is_err(),
        "a key the reader does not know"
    );
    let exp = with_header(&out, |h| h["pair_rows"]["row_exp"] = json!(-16));
    assert!(stack::inspect_uorlut01(&exp).is_err(), "another exponent");
}

/// Pair rows are added once, never beside an address table, and only at the stack's own width
/// and vocabulary.
#[test]
fn pair_rows_are_added_once_and_alone() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let out = stack::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
    assert!(
        stack::add_pair_rows_uorlut01(&out, &image).is_err(),
        "a second add"
    );
    let table = drawn_address(stack::AddressArm::E8, shape.width, 3, 9);
    let address = stack::add_address_uorlut01(&plain, &table).expect("the address table is added");
    assert!(
        stack::add_pair_rows_uorlut01(&address, &image).is_err(),
        "an address-table stack"
    );
    let record = serde_json::to_value(stack::StackPairRows::of(&image)).unwrap();
    let both = with_header(&address, |h| {
        h["pair_rows"] = record;
        h["schema"] = json!(stack::STACK_FBS_PAIR_ROWS_SCHEMA);
    });
    let refused = stack::inspect_uorlut01(&both).expect_err("both records together");
    assert!(
        refused
            .to_string()
            .contains("an address table and pair rows are not served together"),
        "{refused}"
    );
    assert!(
        IntegerStackModel::parse(&both).is_err(),
        "both records together"
    );
    let narrow = drawn_pair_rows(shape.vocab, shape.width - 8);
    assert!(
        stack::add_pair_rows_uorlut01(&plain, &narrow).is_err(),
        "another width"
    );
    let wider = drawn_pair_rows(shape.vocab + 1, shape.width);
    assert!(
        stack::add_pair_rows_uorlut01(&plain, &wider).is_err(),
        "another vocabulary"
    );
}

/// claude-384: pair rows of the same three pairs as [`drawn_pair_rows`], every value `value`.
fn flat_pair_rows(vocab: usize, width: usize, value: i16) -> stack::PairRowsImage {
    let drawn = drawn_pair_rows(vocab, width);
    let (dict, marks) = (drawn.dict(), drawn.marks().clone());
    let rows = vec![value; dict.len() * width];
    stack::PairRowsImage::checked(vocab, width, marks, dict.addresses().to_vec(), rows)
        .expect("pair rows of the stack's size")
}

/// claude-384: two turns. The first user turn `5 9 2 7 3` holds the pairs (2, 5) and (3, 7), rows
/// 1 and 2; its reply runs from the header's last token (position 8) to the second user marker.
/// The second user turn `6 8` holds no dictionary pair.
const TURNS: [u32; 22] = [
    1, 2, 5, 9, 2, 7, 3, 3, 4, 6, 6, 9, 8, 1, 2, 6, 8, 3, 4, 7, 7, 7,
];

fn steps(bytes: &[u8], tokens: &[u32]) -> Vec<stack::NativeStep> {
    let model = IntegerStackModel::parse(bytes).expect("served");
    let mut session = model.session();
    tokens
        .iter()
        .map(|&token| session.step_native(token).expect("step"))
        .collect()
}

/// claude-384: served with all rows zero, every step is the stack's without rows; with rows set,
/// the steps before the first reply position are unchanged and the reply's steps change. Dropped
/// rows serve the stack without them. A stack with pair rows has no trunk image.
#[test]
fn the_served_step_adds_the_pair_rows_at_reply_positions_only() {
    let (_, plain, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let first_reply = 8;
    for (name, bytes) in [("no reads", &plain), ("reads", &with_read)] {
        let want = steps(bytes, &TURNS);
        let zero =
            stack::add_pair_rows_uorlut01(bytes, &flat_pair_rows(shape.vocab, shape.width, 0))
                .expect("added");
        assert_eq!(
            steps(&zero, &TURNS),
            want,
            "{name}: rows of zero change no step"
        );
        let set =
            stack::add_pair_rows_uorlut01(bytes, &flat_pair_rows(shape.vocab, shape.width, 20_000))
                .expect("added");
        let got = steps(&set, &TURNS);
        assert_eq!(
            got[..first_reply],
            want[..first_reply],
            "{name}: no bag before the reply"
        );
        assert_ne!(
            got[first_reply], want[first_reply],
            "{name}: the reply's first step reads the bag"
        );
        let mut model = IntegerStackModel::parse(&set).expect("served");
        assert!(model.pair_rows().is_some());
        assert!(model.fbs_trunk_image().is_none(), "{name}: no trunk image");
        assert!(model.drop_pair_rows(), "{name}: the rows are dropped");
        assert!(!model.drop_pair_rows(), "{name}: once");
        let mut session = model.session();
        let dropped: Vec<_> = TURNS
            .iter()
            .map(|&token| session.step_native(token).expect("step"))
            .collect();
        assert_eq!(
            dropped, want,
            "{name}: dropped rows serve the stack without them"
        );
    }
}

/// claude-384: the session's picks, fed token by token, are the whole stream's open picks at
/// every position, and stay so across a reset and a restored state. A model without pair rows
/// reports none.
#[test]
fn the_session_picks_are_the_open_picks_of_the_whole_stream() {
    let (_, plain, _) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&plain).expect("inspect").shape;
    let image = drawn_pair_rows(shape.vocab, shape.width);
    let bytes = stack::add_pair_rows_uorlut01(&plain, &image).expect("added");
    let model = IntegerStackModel::parse(&bytes).expect("served");
    let open =
        |upto: usize| crate::fbs_pair_rows::open_picks(image.dict(), image.marks(), &TURNS[..upto]);
    let mut session = model.session();
    assert_eq!(session.last_pair_picks(), None, "no picks before a step");
    let mut saved = None;
    for (t, &token) in TURNS.iter().enumerate() {
        session.step_native(token).expect("step");
        assert_eq!(
            session.last_pair_picks().map(<[u32]>::to_vec),
            open(t + 1),
            "position {t}"
        );
        if t == 10 {
            saved = Some(session.save_state());
        }
    }
    assert_eq!(
        open(9),
        Some(vec![1, 2]),
        "the first reply reads rows 1 and 2"
    );
    assert_eq!(
        open(13),
        Some(vec![1, 2]),
        "the reply reads up to the second user marker"
    );
    assert_eq!(open(14), None, "a user marker under way closes the reply");
    session.reset();
    session.step_native(TURNS[0]).expect("step");
    assert_eq!(
        session.last_pair_picks().map(<[u32]>::to_vec),
        open(1),
        "after a reset"
    );
    session
        .restore_state(&saved.expect("saved"))
        .expect("restored");
    for (t, &token) in TURNS.iter().enumerate().skip(11) {
        session.step_native(token).expect("step");
        assert_eq!(
            session.last_pair_picks().map(<[u32]>::to_vec),
            open(t + 1),
            "restored, position {t}"
        );
    }
    // A restored stream of the same length as the session's own, but another open turn: the
    // next step's picks are the restored stream's, not the ones carried from before.
    let mut other = TURNS[..9].to_vec();
    other.resize(TURNS.len(), 6);
    let mut source = model.session();
    for &token in &other {
        source.step_native(token).expect("step");
    }
    assert_eq!(
        session.last_pair_picks().map(<[u32]>::to_vec),
        open(TURNS.len())
    );
    session
        .restore_state(&source.save_state())
        .expect("restored");
    session.step_native(7).expect("step");
    other.push(7);
    assert_eq!(
        session.last_pair_picks().map(<[u32]>::to_vec),
        crate::fbs_pair_rows::open_picks(image.dict(), image.marks(), &other),
        "a restored stream of the same length"
    );
    assert_ne!(
        open(TURNS.len()),
        crate::fbs_pair_rows::open_picks(image.dict(), image.marks(), &other[..TURNS.len()]),
        "the two streams' open turns differ"
    );
    let plain_model = IntegerStackModel::parse(&plain).expect("served");
    let mut session = plain_model.session();
    session.step_native(TURNS[0]).expect("step");
    assert_eq!(session.last_pair_picks(), None, "a model without pair rows");
}

/// claude-384: the stage- and part-timed steps serve the pair rows as the plain step does, and
/// leave the session in the same state.
#[test]
fn the_timed_fbs_steps_serve_the_pair_rows() {
    let (_, _, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let shape = stack::inspect_uorlut01(&with_read).expect("inspect").shape;
    let bytes = stack::add_pair_rows_uorlut01(
        &with_read,
        &flat_pair_rows(shape.vocab, shape.width, 20_000),
    )
    .expect("added");
    let model = IntegerStackModel::parse(&bytes).expect("served");
    let (mut plain, mut stages, mut parts) = (model.session(), model.session(), model.session());
    let mut ns = Vec::new();
    for (t, &token) in TURNS.iter().enumerate() {
        let want = plain.step_native(token).expect("step");
        let (got, _) = stages
            .measure_fbs_step_stages(token, &mut ns)
            .expect("stage-timed step");
        assert_eq!(got, want, "stages, position {t}");
        let (got, _) = parts
            .measure_fbs_step_parts(token, &mut ns)
            .expect("part-timed step");
        assert_eq!(got, want, "parts, position {t}");
    }
    assert_eq!(plain.save_state(), stages.save_state());
    assert_eq!(plain.save_state(), parts.save_state());
}
