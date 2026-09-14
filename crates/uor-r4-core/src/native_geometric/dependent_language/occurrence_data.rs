//! Frozen raw-text occurrence diagnostic. The oracle uses only the authored
//! relation protocol, never a geometric match, candidate or learned prediction.
use super::phrase_data::intervals;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub pattern: String,
    pub overlapping: bool,
    pub subject_answer: bool,
    pub depth: usize,
    pub rotation: usize,
    pub variant: String,
    pub records: [Vec<u8>; 4],
    pub prompt: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_path: Vec<[usize; 2]>,
    pub expected_spans: Vec<[usize; 3]>,
    pub expected_bounds: Vec<[usize; 2]>,
    pub expected_queries: Vec<Vec<u8>>,
    pub expected_payloads: Vec<Vec<u8>>,
}
const PATTERNS: [(&str, &str, &str, &str); 8] = [
    ("disjoint", "ruby amber", "helen birch", "oscar birch"),
    (
        "disjoint_repeat_known",
        "ruby ruby",
        "helen birch",
        "oscar birch",
    ),
    (
        "disjoint_repeat_answer",
        "ruby amber",
        "helen helen",
        "oscar oscar",
    ),
    (
        "overlap_first_first",
        "ruby amber",
        "ruby birch",
        "ruby cedar",
    ),
    (
        "overlap_last_first",
        "amber ruby",
        "ruby birch",
        "ruby cedar",
    ),
    (
        "overlap_first_last",
        "ruby amber",
        "birch ruby",
        "cedar ruby",
    ),
    (
        "overlap_repeat_known",
        "ruby ruby",
        "ruby birch",
        "ruby cedar",
    ),
    (
        "overlap_repeat_answer",
        "ruby amber",
        "ruby ruby",
        "amber amber",
    ),
];
const VARIANTS: [&str; 3] = ["baseline", "active", "inactive"];
/// Separate fixed guard, outside the balanced valid-answer corpus. The raw
/// oracle must resolve the first clause and reject the full continuation.
pub fn multiplicity_guard() -> Result<Value, String> {
    let records = [
        b"alice did guide ruby ruby.".to_vec(),
        b"ruby did help clara.".to_vec(),
        b"felix did trust oscar.".to_vec(),
        b"bruno did help helen.".to_vec(),
    ];
    let first = b"who did alice guide?";
    let prompt = b"who did alice guide? who did they help?";
    let resolved = oracle(&records, first)?;
    if resolved.len() != 1 || resolved[0].value != b"ruby ruby" || oracle(&records, prompt).is_ok()
    {
        return Err("multiplicity guard does not isolate missing full endpoint".into());
    }
    Ok(
        json!({"id":"multiplicity-not-two-occurrences","records":records,"prompt":prompt.as_slice(),"expected_first_payload":b"ruby ruby".as_slice(),"expected_second_query":b"who did ruby ruby help?".as_slice(),"expected_unresolved_clause":1,"expected_reason":"NoCompatibleCandidate","scope":"one source occurrence of ruby cannot satisfy two distinct query occurrences; unresolved routing is not global absence","part_of_384_row_balance":false}),
    )
}
fn words(raw: &[u8]) -> Vec<&[u8]> {
    intervals(raw).iter().map(|[a, b]| &raw[*a..*b]).collect()
}
fn fact(known: &str, verb: &str, answer: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("{answer} did {verb} {known}.").into_bytes()
    } else {
        format!("{known} did {verb} {answer}.").into_bytes()
    }
}
fn question(known: &str, verb: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("who did {verb} {known}?").into_bytes()
    } else {
        format!("who did {known} {verb}?").into_bytes()
    }
}
struct Resolved {
    source: usize,
    span: [usize; 2],
    bounds: [usize; 2],
    query: Vec<u8>,
    value: Vec<u8>,
}
fn is_verb(w: &[u8]) -> bool {
    matches!(w, b"guide" | b"help" | b"trust")
}
fn oracle(records: &[Vec<u8>; 4], prompt: &[u8]) -> Result<Vec<Resolved>, String> {
    let mut out: Vec<Resolved> = Vec::new();
    let clauses: Vec<_> = prompt
        .split_inclusive(|b| *b == b'?')
        .map(|q| q.strip_prefix(b" ").unwrap_or(q))
        .collect();
    if !(1..=2).contains(&clauses.len()) || prompt.len() > 256 {
        return Err("outside occurrence clause window".into());
    }
    for clause in clauses {
        if clause.last() != Some(&b'?') {
            return Err("question terminator missing".into());
        }
        let refs: Vec<_> = intervals(clause)
            .into_iter()
            .filter(|[a, b]| matches!(&clause[*a..*b], b"they" | b"them"))
            .collect();
        let query = match (out.last(), refs.as_slice()) {
            (None, []) => clause.to_vec(),
            (Some(previous), [[a, b]]) => {
                let mut query = clause[..*a].to_vec();
                query.extend(&previous.value);
                query.extend(&clause[*b..]);
                query
            }
            _ => return Err("invalid reference occurrence".into()),
        };
        let q = words(&query);
        if q.len() < 4 || q[..2] != [b"who".as_slice(), b"did".as_slice()] {
            return Err("outside occurrence question grammar".into());
        }
        let subject = is_verb(q[2]);
        let (verb, known) = if subject {
            (q[2], q[3..].join(&b' '))
        } else {
            (q[q.len() - 1], q[2..q.len() - 1].join(&b' '))
        };
        if !is_verb(verb) || known.is_empty() {
            return Err("invalid query relation".into());
        }
        let mut matches = Vec::new();
        for (source, record) in records.iter().enumerate() {
            let r = words(record);
            let aux: Vec<_> = r
                .iter()
                .enumerate()
                .filter(|(_, w)| **w == b"did")
                .map(|(i, _)| i)
                .collect();
            if aux.len() != 1 || record.last() != Some(&b'.') {
                return Err("invalid fact delimiter".into());
            }
            let a = aux[0];
            if a == 0 || a + 2 >= r.len() || !is_verb(r[a + 1]) {
                return Err("outside occurrence fact grammar".into());
            }
            let (answer_span, known_span) = if subject {
                ([0, a], [a + 2, r.len()])
            } else {
                ([a + 2, r.len()], [0, a])
            };
            if r[a + 1] == verb && r[known_span[0]..known_span[1]].join(&b' ') == known {
                let spans = intervals(record);
                let bounds = [spans[answer_span[0]][0], spans[answer_span[1] - 1][1]];
                matches.push(Resolved {
                    source,
                    span: answer_span,
                    bounds,
                    query: query.clone(),
                    value: record[bounds[0]..bounds[1]].to_vec(),
                });
            }
        }
        if matches.len() != 1 {
            return Err("raw relation answer missing or ambiguous".into());
        }
        out.push(matches.pop().ok_or("resolved answer disappeared")?);
    }
    Ok(out)
}
pub fn corpus() -> Result<Vec<Example>, String> {
    let mut rows = Vec::with_capacity(384);
    for (pattern, known, baseline, active) in PATTERNS {
        for subject in [false, true] {
            for depth in 1..=2 {
                let family = format!(
                    "occurrence-{pattern}-subject{}-depth{depth}",
                    usize::from(subject)
                );
                let prompt = if depth == 1 {
                    question(known, "help", subject)
                } else {
                    [
                        question("alice", "guide", subject),
                        question(if subject { "them" } else { "they" }, "help", subject),
                    ]
                    .join(&b' ')
                };
                for rotation in 0..4 {
                    for variant in VARIANTS {
                        let answer = if variant == "active" {
                            active
                        } else {
                            baseline
                        };
                        let target = fact(known, "help", answer, subject);
                        let first = if depth == 1 {
                            target.clone()
                        } else {
                            fact("alice", "guide", known, subject)
                        };
                        let second = if depth == 1 {
                            fact("clara", "guide", "dylan", subject)
                        } else {
                            target
                        };
                        let logical = [
                            first,
                            second,
                            fact(known, "trust", "oscar", subject),
                            fact(
                                "felix",
                                "help",
                                if variant == "inactive" {
                                    "dylan"
                                } else {
                                    "bruno"
                                },
                                subject,
                            ),
                        ];
                        let records =
                            std::array::from_fn(|slot| logical[(slot + 4 - rotation) % 4].clone());
                        let resolved = oracle(&records, &prompt)?;
                        if resolved.last().ok_or("empty path")?.value != answer.as_bytes() {
                            return Err("independent oracle differs from authored answer".into());
                        }
                        rows.push(Example {
                            id: format!("{family}-r{rotation}-{variant}"),
                            family: family.clone(),
                            pattern: pattern.into(),
                            overlapping: pattern.starts_with("overlap_"),
                            subject_answer: subject,
                            depth,
                            rotation,
                            variant: variant.into(),
                            records,
                            prompt: prompt.clone(),
                            answer: answer.as_bytes().to_vec(),
                            expected_path: resolved.iter().map(|r| [r.source, r.span[0]]).collect(),
                            expected_spans: resolved
                                .iter()
                                .map(|r| [r.source, r.span[0], r.span[1]])
                                .collect(),
                            expected_bounds: resolved.iter().map(|r| r.bounds).collect(),
                            expected_queries: resolved.iter().map(|r| r.query.clone()).collect(),
                            expected_payloads: resolved.iter().map(|r| r.value.clone()).collect(),
                        });
                    }
                }
            }
        }
    }
    validate(&rows)?;
    Ok(rows)
}
fn check_row(e: &Example) -> Result<(), String> {
    let out = oracle(&e.records, &e.prompt)?;
    if e.depth != out.len()
        || e.answer != out.last().ok_or("empty oracle")?.value
        || e.expected_path
            != out
                .iter()
                .map(|r| [r.source, r.span[0]])
                .collect::<Vec<_>>()
        || e.expected_spans
            != out
                .iter()
                .map(|r| [r.source, r.span[0], r.span[1]])
                .collect::<Vec<_>>()
        || e.expected_bounds != out.iter().map(|r| r.bounds).collect::<Vec<_>>()
        || e.expected_queries != out.iter().map(|r| r.query.clone()).collect::<Vec<_>>()
        || e.expected_payloads != out.iter().map(|r| r.value.clone()).collect::<Vec<_>>()
    {
        return Err("occurrence oracle metadata differs".into());
    }
    for raw in e.records.iter().chain(e.expected_queries.iter()) {
        let w = words(raw);
        if raw.len() > 128
            || w.len() > 16
            || w.is_empty()
            || w.iter()
                .any(|w| w.len() > 16 || !w.iter().all(u8::is_ascii_lowercase))
        {
            return Err("occurrence lexical window exceeded".into());
        }
    }
    let (_, known, _, _) = PATTERNS
        .iter()
        .find(|p| p.0 == e.pattern)
        .ok_or("unknown pattern")?;
    let k = words(known.as_bytes());
    let a = words(&e.answer);
    if e.overlapping != k.iter().any(|w| a.contains(w))
        || a.len() != 2
        || e.pattern.contains("repeat_known") && k[0] != k[1]
        || e.pattern.contains("repeat_answer") && a[0] != a[1]
    {
        return Err("overlap/repetition classification differs".into());
    }
    Ok(())
}
pub fn validate(rows: &[Example]) -> Result<Value, String> {
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut families = BTreeMap::<&str, BTreeMap<(usize, &str), &Example>>::new();
    let mut patterns = BTreeMap::<&str, usize>::new();
    let mut overlaps = [0usize; 2];
    let mut roles = [0usize; 2];
    let mut depths = [0usize; 2];
    for e in rows {
        check_row(e)?;
        if e.rotation >= 4
            || !VARIANTS.contains(&e.variant.as_str())
            || !(1..=2).contains(&e.depth)
            || !ids.insert(&e.id)
            || !inputs.insert((&e.records, &e.prompt))
            || families
                .entry(&e.family)
                .or_default()
                .insert((e.rotation, &e.variant), e)
                .is_some()
        {
            return Err("duplicate or malformed occurrence cell".into());
        }
        *patterns.entry(&e.pattern).or_default() += 1;
        overlaps[usize::from(e.overlapping)] += 1;
        roles[usize::from(e.subject_answer)] += 1;
        depths[e.depth - 1] += 1;
    }
    if rows.len() != 384
        || families.len() != 32
        || overlaps != [144, 240]
        || roles != [192; 2]
        || depths != [192; 2]
        || PATTERNS.iter().any(|p| patterns.get(p.0) != Some(&48))
    {
        return Err("occurrence balance differs".into());
    }
    for cells in families.values() {
        if cells.len() != 12 {
            return Err("incomplete occurrence family".into());
        }
        let reference = cells.get(&(0, "baseline")).ok_or("baseline missing")?;
        for rotation in 0..4 {
            let base = cells.get(&(rotation, "baseline")).ok_or("baseline cell")?;
            let active = cells.get(&(rotation, "active")).ok_or("active cell")?;
            let inactive = cells.get(&(rotation, "inactive")).ok_or("inactive cell")?;
            if base.prompt != reference.prompt
                || active.prompt != base.prompt
                || inactive.prompt != base.prompt
                || base.answer != reference.answer
                || inactive.answer != base.answer
                || active.answer == base.answer
                || (0..4).any(|s| base.records[(s + rotation) % 4] != reference.records[s])
            {
                return Err("rotation or intervention identity differs".into());
            }
            let source = base.expected_path[base.depth - 1][0];
            let [a, b] = base.expected_bounds[base.depth - 1];
            let mut expected = base.records.clone();
            expected[source] = [
                &expected[source][..a],
                active.answer.as_slice(),
                &expected[source][b..],
            ]
            .concat();
            if expected != active.records || active.expected_path != base.expected_path {
                return Err("active intervention changes unrelated bytes/path".into());
            }
            let unused = (3 + rotation) % 4;
            let mut expected = base.records.clone();
            expected[unused] = fact("felix", "help", "dylan", base.subject_answer);
            if expected != inactive.records
                || inactive.expected_path != base.expected_path
                || inactive.expected_queries != base.expected_queries
                || base.expected_path.iter().any(|p| p[0] == unused)
            {
                return Err("inactive intervention changes active path".into());
            }
        }
    }
    Ok(
        json!({"examples":384,"families":32,"rows_per_family":12,"patterns":patterns,"disjoint_rows":144,"overlap_rows":240,"roles":roles,"depths":depths,"source_rotations":4,"variants":VARIANTS,"inputs":["records","prompt"],"evaluation_only":["answer","expected_path","expected_spans","expected_bounds","expected_queries","expected_payloads"],"oracle":"independent raw relation grammar with exact full-phrase endpoint identity and sequential pronoun substitution","training":"NOT_RUN","model_filtering":false,"final_holdout":"NOT_RUN"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn occurrence_oracle_preserves_repetition_roles_and_interventions() -> Result<(), String> {
        let rows = corpus()?;
        let mut bad = rows
            .iter()
            .find(|e| e.pattern == "overlap_repeat_answer" && e.depth == 2)
            .ok_or("repeated row")?
            .clone();
        bad.answer = b"ruby".to_vec();
        assert!(check_row(&bad).is_err());
        let mut bad = rows.clone();
        bad[0].expected_bounds[0][0] += 1;
        assert!(validate(&bad).is_err());
        let mut bad = rows.clone();
        let active = bad
            .iter_mut()
            .find(|e| e.variant == "active")
            .ok_or("active row")?;
        active.records[(3 + active.rotation) % 4].insert(0, b' ');
        assert!(validate(&bad).is_err());
        Ok(())
    }
}
