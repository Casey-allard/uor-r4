//! Freezes the m1-address-as-read oracle panel, panel-v2.
//!
//! Each row is one dialogue. `writes` are its stated facts in sequence order: the evaluator
//! renders each `statement` as dialogue text, and the directory stores each `value` at the
//! write's prime address. `query` is the question asked after them. It names an entity and a
//! slot only, never an answer or a row family. The scorer-only fields (`family`, the oracle
//! address, `outcome`, `expected_bytes`, `evicted_at_sequence`) are never shown to the model.
//!
//! Arm B's address comes from the query's words: split the query on ASCII whitespace, trim
//! trailing `?`, `.` and `,`, keep the words found in the panel lexicon (`terms`) in order, take
//! their primes, and pair each kept word with the next. panel-v1 (`0712371d…`) regenerates from
//! commit `1a70a9ac3`.

use sha2::{Digest, Sha256};
use std::{error::Error, fs::OpenOptions, io::Write, path::Path};
use uor_r4_object_store::{
    address_read::{
        AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
    },
    Limits, Store,
};

const PANEL: &str = "m1-address-as-read/panel-v2";
const ENTITIES: [(&str, u32); 10] = [
    ("entity-00", 2),
    ("entity-01", 5),
    ("entity-02", 11),
    ("entity-03", 17),
    ("entity-04", 23),
    ("entity-05", 29),
    ("entity-06", 37),
    ("entity-07", 43),
    ("entity-08", 53),
    ("entity-09", 59),
];
const SLOTS: [(&str, u32); 6] = [
    ("fact-a", 3),
    ("fact-b", 7),
    ("fact-c", 13),
    ("fact-d", 19),
    ("status-a", 31),
    ("status-b", 41),
];

struct Fact {
    sequence: u64,
    /// The slot version `AddressDirectory::write` returns: 1 for a new address, 2 after an
    /// overwrite.
    version: u8,
    entity: usize,
    slot: usize,
    statement: String,
    value: String,
}

#[derive(Clone, Copy)]
enum Select {
    Current,
    Previous,
}

enum Expected {
    Hit(String),
    Previous(String),
    Absent,
    Evicted(u64),
}

struct Row {
    id: String,
    family: &'static str,
    capacity: usize,
    writes: Vec<Fact>,
    entity: usize,
    slot: usize,
    select: Select,
    expected: Expected,
}

/// Eight lowercase letters from SHA-256, so a value is neither guessable from the query's
/// words nor shared by two different facts.
fn value(entity: usize, slot: usize, version: u8) -> String {
    let seed = format!(
        "{PANEL}|{}|{}|v{version}",
        ENTITIES[entity].0, SLOTS[slot].0
    );
    Sha256::digest(seed.as_bytes())
        .iter()
        .take(8)
        .map(|byte| char::from(b'a' + byte % 26))
        .collect()
}

/// Builds a row's dialogue from `(entity, slot, version)` triples, numbering writes from 1 as
/// the directory's sequence does.
fn dialogue(facts: &[(usize, usize, u8)]) -> Vec<Fact> {
    facts
        .iter()
        .zip(1u64..)
        .map(|(&(entity, slot, version), sequence)| {
            let value = value(entity, slot, version);
            let (e, s) = (ENTITIES[entity].0, SLOTS[slot].0);
            let statement = if version == 1 {
                format!("{e} {s} is {value}.")
            } else {
                format!("{e} {s} is now {value}.")
            };
            Fact {
                sequence,
                version,
                entity,
                slot,
                statement,
                value,
            }
        })
        .collect()
}

fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    for i in 0..10 {
        let j = (i + 1) % 10;
        let facts: Vec<_> = (0..4).flat_map(|s| [(i, s, 1), (j, s, 1)]).collect();
        for k in 0..4 {
            rows.push(Row {
                id: format!("stated-{:03}", i * 4 + k),
                family: "stated",
                capacity: 8,
                writes: dialogue(&facts),
                entity: i,
                slot: k,
                select: Select::Current,
                expected: Expected::Hit(value(i, k, 1)),
            });
        }
    }
    for i in 0..10 {
        let j = (i + 1) % 10;
        for m in 0..2 {
            let s = 4 + m;
            rows.push(Row {
                id: format!("historical-{:03}", i * 2 + m),
                family: "historical",
                capacity: 8,
                writes: dialogue(&[(i, s, 1), (j, s, 1), (i, 0, 1), (i, s, 2), (j, 0, 1)]),
                entity: i,
                slot: s,
                select: Select::Previous,
                expected: Expected::Previous(value(i, s, 1)),
            });
        }
    }
    for i in 0..10 {
        let j = (i + 1) % 10;
        let facts: Vec<_> = (0..3).flat_map(|s| [(i, s, 1), (j, s, 1)]).collect();
        rows.push(Row {
            id: format!("unwritten-{i:03}"),
            family: "unwritten",
            capacity: 8,
            writes: dialogue(&facts),
            entity: i,
            slot: 3,
            select: Select::Current,
            expected: Expected::Absent,
        });
    }
    for i in 0..10 {
        let j = (i + 1) % 10;
        // Capacity 4: the fifth write evicts the oldest slot, this row's target.
        rows.push(Row {
            id: format!("evicted-{i:03}"),
            family: "evicted",
            capacity: 4,
            writes: dialogue(&[
                (i, 0, 1),
                (j, 0, 1),
                (j, 1, 1),
                (j, 2, 1),
                (j, 3, 1),
                (i, 1, 1),
            ]),
            entity: i,
            slot: 0,
            select: Select::Current,
            expected: Expected::Evicted(5),
        });
    }
    rows
}

fn query(row: &Row) -> String {
    let (e, s) = (ENTITIES[row.entity].0, SLOTS[row.slot].0);
    match row.select {
        Select::Current => format!("What is {e} {s}?"),
        Select::Previous => format!("What was {e} {s} before it changed?"),
    }
}

fn address(entity: usize, slot: usize) -> Result<PrimeAddress, Box<dyn Error>> {
    let (e, s) = (ENTITIES[entity].1, SLOTS[slot].1);
    PrimeAddress::with_pairs(&[e, s], &[(e, s)])
        .map_err(|error| format!("invalid address {e}/{s}: {error}").into())
}

/// Replays a row's writes into a fresh store and directory and checks the oracle read gives
/// the expected outcome, so a row is emitted only if its recorded state is real.
fn check(row: &Row) -> Result<(), Box<dyn Error>> {
    let mut store = Store::new(Limits::default());
    let mut dir = AddressDirectory::new(row.capacity)?;
    for fact in &row.writes {
        let folded = store.fold(fact.value.as_bytes())?;
        let version = dir.write(
            address(fact.entity, fact.slot)?,
            ObjectTarget::WholeObject {
                manifest: folded.manifest,
            },
        )?;
        if version != u64::from(fact.version) {
            return Err(
                format!("{}: write {} gave version {version}", row.id, fact.sequence).into(),
            );
        }
    }
    let select = match row.select {
        Select::Current => VersionSelect::Current,
        Select::Previous => VersionSelect::Previous,
    };
    let outcome = dir.read_exact(&store, None, &address(row.entity, row.slot)?, select)?;
    let ok = match (&row.expected, &outcome) {
        (
            Expected::Hit(want),
            AddressReadOutcome::Hit {
                bytes,
                whole_object_verified,
                ..
            },
        )
        | (
            Expected::Previous(want),
            AddressReadOutcome::Previous {
                bytes,
                whole_object_verified,
                ..
            },
        ) => bytes == want.as_bytes() && *whole_object_verified,
        (Expected::Absent, AddressReadOutcome::Absent) => true,
        (
            Expected::Evicted(want),
            AddressReadOutcome::Evicted {
                evicted_at_sequence,
                ..
            },
        ) => evicted_at_sequence == want,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(format!("{}: oracle read mismatch: {outcome:?}", row.id).into())
    }
}

fn generate_panel() -> Result<String, Box<dyn Error>> {
    let mut panel = String::new();
    for row in rows() {
        check(&row)?;
        panel.push_str(&json_row(&row));
    }
    Ok(panel)
}

fn json_terms(entity: usize, slot: usize) -> String {
    let (e, s) = (ENTITIES[entity], SLOTS[slot]);
    format!(
        "[{{\"word\":{},\"prime\":{}}},{{\"word\":{},\"prime\":{}}}]",
        json_string(e.0),
        e.1,
        json_string(s.0),
        s.1
    )
}

/// The canonical address fields: primes ascending, then the one ordered `(entity, slot)` pair.
fn json_address(entity: usize, slot: usize) -> (String, String) {
    let (e, s) = (ENTITIES[entity].1, SLOTS[slot].1);
    (
        format!("[{},{}]", e.min(s), e.max(s)),
        format!("[[{e},{s}]]"),
    )
}

fn json_row(row: &Row) -> String {
    let writes = row
        .writes
        .iter()
        .map(|fact| {
            let (primes, pairs) = json_address(fact.entity, fact.slot);
            format!(
                "{{\"sequence\":{},\"version\":{},\"terms\":{},\"primes\":{primes},\"ordered_pairs\":{pairs},\"statement\":{},\"value\":{}}}",
                fact.sequence,
                fact.version,
                json_terms(fact.entity, fact.slot),
                json_string(&fact.statement),
                json_string(&fact.value),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let (primes, pairs) = json_address(row.entity, row.slot);
    let (version, outcome, expected, evicted_at) = match &row.expected {
        Expected::Hit(v) => ("current", "hit", json_string(v), "null".to_string()),
        Expected::Previous(v) => ("previous", "previous", json_string(v), "null".to_string()),
        Expected::Absent => ("current", "absent", "null".to_string(), "null".to_string()),
        Expected::Evicted(seq) => ("current", "evicted", "null".to_string(), seq.to_string()),
    };
    format!(
        "{{\"panel\":{},\"row_id\":{},\"family\":{},\"capacity\":{},\"writes\":[{writes}],\"query\":{},\"query_terms\":{},\"oracle_primes\":{primes},\"oracle_ordered_pairs\":{pairs},\"version_select\":{},\"outcome\":{},\"expected_bytes\":{expected},\"evicted_at_sequence\":{evicted_at}}}\n",
        json_string(PANEL),
        json_string(&row.id),
        json_string(row.family),
        row.capacity,
        json_string(&query(row)),
        json_terms(row.entity, row.slot),
        json_string(version),
        json_string(outcome),
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('\"');
    out
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let output = args.next().ok_or("usage: address-panel-freeze OUT")?;
    if args.next().is_some() {
        return Err("usage: address-panel-freeze OUT".into());
    }
    let panel = generate_panel()?;
    let path = Path::new(&output);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(panel.as_bytes())?;
    file.sync_all()?;
    println!("panel {PANEL}");
    println!("rows {}", panel.lines().count());
    println!("bytes {}", panel.len());
    println!("sha256 {}", hex(&Sha256::digest(panel.as_bytes())));
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::{BTreeMap, BTreeSet};

    const FAMILY_WORDS: [&str; 7] = [
        "stated",
        "historical",
        "unwritten",
        "evicted",
        "previous",
        "absent",
        "hit",
    ];

    /// The panel as an evaluator reads it: parsed from the emitted bytes, not the generator's
    /// structs.
    fn parsed() -> Vec<Value> {
        generate_panel()
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn numbers(value: &Value) -> Vec<u32> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|n| u32::try_from(n.as_u64().unwrap()).unwrap())
            .collect()
    }

    fn pairs(value: &Value) -> Vec<(u32, u32)> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| {
                let pair = numbers(pair);
                (pair[0], pair[1])
            })
            .collect()
    }

    fn row_address(primes: &Value, ordered_pairs: &Value) -> PrimeAddress {
        PrimeAddress::with_pairs(&numbers(primes), &pairs(ordered_pairs)).unwrap()
    }

    fn terms(value: &Value) -> Vec<(String, u32)> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                let prime = u32::try_from(t["prime"].as_u64().unwrap()).unwrap();
                (t["word"].as_str().unwrap().to_string(), prime)
            })
            .collect()
    }

    fn lexicon(panel: &[Value]) -> BTreeMap<String, u32> {
        let mut lexicon = BTreeMap::new();
        let mut owner = BTreeMap::new();
        for row in panel {
            let writes = row["writes"].as_array().unwrap();
            let all = writes
                .iter()
                .flat_map(|w| terms(&w["terms"]))
                .chain(terms(&row["query_terms"]));
            for (word, prime) in all {
                assert_eq!(
                    *lexicon.entry(word.clone()).or_insert(prime),
                    prime,
                    "{word}"
                );
                assert_eq!(*owner.entry(prime).or_insert(word.clone()), word, "{prime}");
            }
        }
        lexicon
    }

    /// Arm B's rule from the module docs, applied to a query string.
    fn arm_b(query: &str, lexicon: &BTreeMap<String, u32>) -> (Vec<u32>, Vec<(u32, u32)>) {
        let kept: Vec<u32> = query
            .split_ascii_whitespace()
            .map(|w| w.trim_end_matches(['?', '.', ',']))
            .filter_map(|w| lexicon.get(w).copied())
            .collect();
        let pairs = kept.windows(2).map(|w| (w[0], w[1])).collect();
        (kept, pairs)
    }

    #[test]
    fn registered_composition_and_byte_reproduction() {
        let first = generate_panel().unwrap();
        assert_eq!(first, generate_panel().unwrap());
        let panel = parsed();
        assert_eq!(panel.len(), 80);
        let count = |field: &str, want: &str| panel.iter().filter(|r| r[field] == want).count();
        assert_eq!(count("family", "stated"), 40);
        assert_eq!(count("family", "historical"), 20);
        assert_eq!(count("family", "unwritten"), 10);
        assert_eq!(count("family", "evicted"), 10);
        assert_eq!(count("outcome", "hit"), 40);
        assert_eq!(count("outcome", "previous"), 20);
        assert_eq!(count("outcome", "absent"), 10);
        assert_eq!(count("outcome", "evicted"), 10);
        assert!(panel.iter().all(|r| r["panel"] == PANEL));
        let ids: BTreeSet<_> = panel
            .iter()
            .map(|r| r["row_id"].as_str().unwrap())
            .collect();
        assert_eq!(ids.len(), 80);
        let dialogues: BTreeSet<_> = panel
            .iter()
            .map(|r| (r["writes"].to_string(), r["query"].to_string()))
            .collect();
        assert_eq!(dialogues.len(), 80, "two rows share a dialogue and a query");
    }

    #[test]
    fn queries_hold_no_answer_and_no_family_label() {
        for row in parsed() {
            let id = row["row_id"].as_str().unwrap();
            let query = row["query"].as_str().unwrap();
            for write in row["writes"].as_array().unwrap() {
                let value = write["value"].as_str().unwrap();
                assert!(!query.contains(value), "{id}: query holds a dialogue value");
            }
            if let Some(expected) = row["expected_bytes"].as_str() {
                assert!(!query.contains(expected), "{id}: query holds its answer");
            }
            let lower = query.to_ascii_lowercase();
            for word in FAMILY_WORDS {
                assert!(!lower.contains(word), "{id}: query names `{word}`");
            }
            let t = terms(&row["query_terms"]);
            let template = if row["version_select"] == "previous" {
                format!("What was {} {} before it changed?", t[0].0, t[1].0)
            } else {
                format!("What is {} {}?", t[0].0, t[1].0)
            };
            assert_eq!(query, template, "{id}: query departs from its template");
        }
    }

    #[test]
    fn replaying_the_writes_reproduces_every_oracle_outcome() {
        for row in parsed() {
            let id = row["row_id"].as_str().unwrap();
            let capacity = usize::try_from(row["capacity"].as_u64().unwrap()).unwrap();
            let mut store = Store::new(Limits::default());
            let mut dir = AddressDirectory::new(capacity).unwrap();
            for write in row["writes"].as_array().unwrap() {
                let folded = store
                    .fold(write["value"].as_str().unwrap().as_bytes())
                    .unwrap();
                let target = ObjectTarget::WholeObject {
                    manifest: folded.manifest,
                };
                let address = row_address(&write["primes"], &write["ordered_pairs"]);
                let version = dir.write(address, target).unwrap();
                assert_eq!(Some(version), write["version"].as_u64(), "{id}");
            }
            let oracle = row_address(&row["oracle_primes"], &row["oracle_ordered_pairs"]);
            let select = match row["version_select"].as_str().unwrap() {
                "current" => VersionSelect::Current,
                "previous" => VersionSelect::Previous,
                other => panic!("{id}: version_select {other}"),
            };
            let outcome = dir.read_exact(&store, None, &oracle, select).unwrap();
            let expected = row["expected_bytes"].as_str().map(str::as_bytes);
            match (row["outcome"].as_str().unwrap(), outcome) {
                (
                    "hit",
                    AddressReadOutcome::Hit {
                        bytes,
                        whole_object_verified,
                        ..
                    },
                )
                | (
                    "previous",
                    AddressReadOutcome::Previous {
                        bytes,
                        whole_object_verified,
                        ..
                    },
                ) => {
                    assert_eq!(Some(bytes.as_slice()), expected, "{id}");
                    assert!(whole_object_verified, "{id}");
                }
                ("absent", AddressReadOutcome::Absent) => assert_eq!(expected, None, "{id}"),
                (
                    "evicted",
                    AddressReadOutcome::Evicted {
                        evicted_at_sequence,
                        ..
                    },
                ) => assert_eq!(
                    Some(evicted_at_sequence),
                    row["evicted_at_sequence"].as_u64(),
                    "{id}"
                ),
                (want, got) => panic!("{id}: recorded {want}, replay gave {got:?}"),
            }
        }
    }

    #[test]
    fn arm_b_query_words_derive_the_oracle_address() {
        let panel = parsed();
        let lexicon = lexicon(&panel);
        assert_eq!(lexicon.len(), ENTITIES.len() + SLOTS.len());
        for row in &panel {
            let id = row["row_id"].as_str().unwrap();
            let (primes, ordered_pairs) = arm_b(row["query"].as_str().unwrap(), &lexicon);
            let derived = PrimeAddress::with_pairs(&primes, &ordered_pairs).unwrap();
            let oracle = row_address(&row["oracle_primes"], &row["oracle_ordered_pairs"]);
            assert_eq!(derived, oracle, "{id}: Arm B misses the oracle address");
            let query_terms: Vec<u32> = terms(&row["query_terms"]).iter().map(|t| t.1).collect();
            assert_eq!(
                primes, query_terms,
                "{id}: query_terms disagree with the query"
            );
            for write in row["writes"].as_array().unwrap() {
                let (primes, ordered_pairs) = arm_b(write["statement"].as_str().unwrap(), &lexicon);
                let derived = PrimeAddress::with_pairs(&primes, &ordered_pairs).unwrap();
                assert_eq!(
                    derived,
                    row_address(&write["primes"], &write["ordered_pairs"]),
                    "{id}: a statement's words disagree with its write address"
                );
            }
        }
    }

    #[test]
    fn values_are_distinct_and_unguessable() {
        let panel = parsed();
        let lexicon = lexicon(&panel);
        let mut fact_of = BTreeMap::new();
        for row in &panel {
            for write in row["writes"].as_array().unwrap() {
                let value = write["value"].as_str().unwrap();
                assert_eq!(value.len(), 8);
                assert!(value.bytes().all(|b| b.is_ascii_lowercase()), "{value}");
                assert!(lexicon.keys().all(|word| !value.contains(word.as_str())));
                let statement = write["statement"].as_str().unwrap();
                let fact = (write["primes"].to_string(), statement.replace(value, ""));
                assert_eq!(
                    *fact_of.entry(value.to_string()).or_insert(fact.clone()),
                    fact
                );
            }
        }
        let facts: BTreeSet<_> = fact_of.values().collect();
        assert_eq!(facts.len(), fact_of.len(), "two facts share a value");
    }

    #[test]
    fn stated_rows_resolve_through_the_sieve() {
        let panel = parsed();
        let lexicon = lexicon(&panel);
        for row in panel.iter().filter(|r| r["family"] == "stated") {
            let id = row["row_id"].as_str().unwrap();
            let mut store = Store::new(Limits::default());
            let mut dir = AddressDirectory::new(8).unwrap();
            for write in row["writes"].as_array().unwrap() {
                let folded = store
                    .fold(write["value"].as_str().unwrap().as_bytes())
                    .unwrap();
                let target = ObjectTarget::WholeObject {
                    manifest: folded.manifest,
                };
                dir.write(
                    row_address(&write["primes"], &write["ordered_pairs"]),
                    target,
                )
                .unwrap();
            }
            let (primes, ordered_pairs) = arm_b(row["query"].as_str().unwrap(), &lexicon);
            let query = PrimeAddress::with_pairs(&primes, &ordered_pairs).unwrap();
            match dir.read_sieve(&store, None, &query).unwrap() {
                AddressReadOutcome::Hit { bytes, .. } => assert_eq!(
                    Some(bytes.as_slice()),
                    row["expected_bytes"].as_str().map(str::as_bytes),
                    "{id}"
                ),
                other => panic!("{id}: sieve gave {other:?}"),
            }
        }
    }
}
