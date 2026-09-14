//! Whole-phrase intermediate references in a bounded authored query protocol.
//! Raw relation resolution is independent of model routing and update operators.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub placement: String,
    pub depth: usize,
    pub length: usize,
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
const NAMES: [&str; 8] = [
    "alice", "bruno", "ruby", "helen", "oscar", "felix", "clara", "dylan",
];
const PLACEMENTS: [&str; 3] = ["first", "middle", "both"];
const VARIANTS: [&str; 3] = ["baseline", "active", "inactive"];
pub fn intervals(bytes: &[u8]) -> Vec<[usize; 2]> {
    let mut out = Vec::new();
    let mut start = None;
    for (i, byte) in bytes
        .iter()
        .copied()
        .chain(std::iter::once(b'.'))
        .enumerate()
    {
        if byte.is_ascii_alphabetic() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(a) = start.take() {
            out.push([a, i]);
        }
    }
    out
}
fn words(bytes: &[u8]) -> Vec<&[u8]> {
    intervals(bytes)
        .iter()
        .map(|[a, b]| &bytes[*a..*b])
        .collect()
}
fn render(parts: &[&[u8]], punctuation: u8) -> Vec<u8> {
    let mut out = parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(&b' ');
    out.push(punctuation);
    out
}
fn fact(styled: bool, known: &[u8], verb: &[u8], answer: &[u8], subject: bool) -> Vec<u8> {
    let (s, o) = if subject {
        (answer, known)
    } else {
        (known, answer)
    };
    if styled {
        render(&[b"today", s, b"can", b"quietly", verb, o], b'.')
    } else {
        render(&[s, b"did", verb, o], b'.')
    }
}
fn question(styled: bool, known: &[u8], verb: &[u8], subject: bool) -> Vec<u8> {
    let (prefix, auxiliary, adverb): (&[u8], &[u8], &[u8]) = if styled {
        (b"which person", b"can", b"quietly")
    } else {
        (b"who", b"did", b"")
    };
    if subject {
        render(&[prefix, auxiliary, adverb, verb, known], b'?')
    } else {
        render(&[prefix, auxiliary, known, adverb, verb], b'?')
    }
}
fn phrase(name: &[u8], length: usize, active: bool, alternate: &[u8]) -> Vec<u8> {
    match length {
        1 => {
            if active {
                alternate.to_vec()
            } else {
                name.to_vec()
            }
        }
        2 => [name, if active { b"birch" } else { b"amber" }].join(&b' '),
        _ => [name, b"amber", if active { b"cedar" } else { b"birch" }].join(&b' '),
    }
}
fn questions(prompt: &[u8]) -> Result<Vec<&[u8]>, String> {
    let q: Vec<_> = prompt
        .split_inclusive(|&b| b == b'?')
        .map(|s| s.strip_prefix(b" ").unwrap_or(s))
        .collect();
    if prompt.len() > 256
        || !(2..=3).contains(&q.len())
        || q.iter().any(|p| p.last() != Some(&b'?'))
    {
        return Err("invalid authored question protocol".into());
    }
    Ok(q)
}
struct Fact {
    subject: [usize; 2],
    object: [usize; 2],
    verb: usize,
}
fn parse_fact(record: &[u8], styled: bool) -> Result<Fact, String> {
    let r = words(record);
    let aux = if styled {
        b"can".as_slice()
    } else {
        b"did".as_slice()
    };
    let indices: Vec<_> = r
        .iter()
        .enumerate()
        .filter(|(_, w)| **w == aux)
        .map(|(i, _)| i)
        .collect();
    if indices.len() != 1 || record.last() != Some(&b'.') {
        return Err("invalid fact auxiliary/punctuation".into());
    }
    let start = usize::from(styled);
    let a = indices[0];
    let verb = a + 1 + usize::from(styled);
    if a <= start
        || verb + 1 >= r.len()
        || (styled && (r[0] != b"today" || r[a + 1] != b"quietly"))
        || !matches!(r[verb], b"guide" | b"help" | b"trust")
    {
        return Err("outside fact grammar".into());
    }
    Ok(Fact {
        subject: [start, a],
        object: [verb + 1, r.len()],
        verb,
    })
}
#[derive(Clone, Debug)]
struct Resolved {
    source: usize,
    span: [usize; 2],
    bounds: [usize; 2],
    value: Vec<u8>,
    query: Vec<u8>,
    subject: bool,
}
fn expand(query: &[u8], previous: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let spans = intervals(query);
    let refs: Vec<_> = spans
        .iter()
        .copied()
        .filter(|[a, b]| matches!(&query[*a..*b], b"they" | b"them"))
        .collect();
    match previous {
        None if refs.is_empty() => Ok(query.to_vec()),
        Some(value) if refs.len() == 1 => {
            let [a, b] = refs[0];
            let mut out = query[..a].to_vec();
            out.extend(value);
            out.extend(&query[b..]);
            Ok(out)
        }
        _ => Err("invalid sequential reference occurrence".into()),
    }
}
fn resolve(
    records: &[Vec<u8>; 4],
    query: &[u8],
    previous: Option<&[u8]>,
) -> Result<Resolved, String> {
    let expanded = expand(query, previous)?;
    let q = words(&expanded);
    let styled = expanded.starts_with(b"which person ");
    let prefix: &[&[u8]] = if styled {
        &[b"which", b"person"]
    } else {
        &[b"who"]
    };
    let lead = prefix.len() + 1;
    let aux = if styled {
        b"can".as_slice()
    } else {
        b"did".as_slice()
    };
    if q.len() < lead + 2 + usize::from(styled)
        || q[..prefix.len()] != *prefix
        || q[prefix.len()] != aux
    {
        return Err("outside question grammar".into());
    }
    let si = lead + usize::from(styled);
    let subject =
        matches!(q[si], b"guide" | b"help" | b"trust") && (!styled || q[lead] == b"quietly");
    let (verb, known) = if subject {
        (q[si], q[si + 1..].join(&b' '))
    } else {
        let vi = q.len() - 1;
        let end = vi - usize::from(styled);
        if end <= lead
            || (styled && q[vi - 1] != b"quietly")
            || !matches!(q[vi], b"guide" | b"help" | b"trust")
        {
            return Err("outside object question grammar".into());
        }
        (q[vi], q[lead..end].join(&b' '))
    };
    let mut results = Vec::new();
    for (source, record) in records.iter().enumerate() {
        let f = parse_fact(record, styled)?;
        let r = words(record);
        let (answer, k) = if subject {
            (f.subject, f.object)
        } else {
            (f.object, f.subject)
        };
        if r[f.verb] == verb && r[k[0]..k[1]].join(&b' ') == known {
            let spans = intervals(record);
            let bounds = [spans[answer[0]][0], spans[answer[1] - 1][1]];
            results.push(Resolved {
                source,
                span: answer,
                bounds,
                value: record[bounds[0]..bounds[1]].to_vec(),
                query: expanded.clone(),
                subject,
            });
        }
    }
    if results.len() != 1 {
        return Err("raw query has missing or ambiguous full-phrase answer".into());
    }
    results.pop().ok_or_else(|| "answer disappeared".into())
}
fn oracle(records: &[Vec<u8>; 4], prompt: &[u8]) -> Result<Vec<Resolved>, String> {
    let mut path: Vec<Resolved> = Vec::new();
    for q in questions(prompt)? {
        path.push(resolve(
            records,
            q,
            path.last().map(|r| r.value.as_slice()),
        )?);
    }
    Ok(path)
}
fn authored() -> Result<Vec<Example>, String> {
    let mut rows = Vec::with_capacity(864);
    for offset in 0..2 {
        let n: [&[u8]; 8] = std::array::from_fn(|i| NAMES[(offset + i) % 8].as_bytes());
        for subject in [false, true] {
            for styled in [false, true] {
                for length in 1..=3 {
                    for placement in PLACEMENTS {
                        let family = format!(
                            "phrase-n{offset}-v{}-s{}-l{length}-{placement}",
                            usize::from(subject),
                            usize::from(styled)
                        );
                        let alias = phrase(n[2], length, false, n[6]);
                        let changed = phrase(n[2], length, true, n[6]);
                        let bridge = if placement == "both" {
                            [n[7], n[5], n[1]][..length].join(&b' ')
                        } else {
                            n[1].to_vec()
                        };
                        let reference = if subject {
                            b"them".as_slice()
                        } else {
                            b"they".as_slice()
                        };
                        let depth = if placement == "first" { 2 } else { 3 };
                        let mut qs = vec![
                            question(styled, n[0], b"guide", subject),
                            question(styled, reference, b"help", subject),
                        ];
                        if depth == 3 {
                            qs.push(question(styled, reference, b"trust", subject));
                        }
                        let prompt = qs.join(&b' ');
                        for rotation in 0..4 {
                            for variant in VARIANTS {
                                let selected = if variant == "active" {
                                    &changed
                                } else {
                                    &alias
                                };
                                let right_answer = if variant == "inactive" { n[5] } else { n[4] };
                                let logical = if depth == 2 {
                                    [
                                        fact(styled, n[0], b"guide", selected, subject),
                                        fact(styled, &alias, b"help", n[3], subject),
                                        fact(styled, &changed, b"help", right_answer, subject),
                                        fact(styled, n[5], b"trust", n[1], subject),
                                    ]
                                } else {
                                    [
                                        fact(styled, n[0], b"guide", &bridge, subject),
                                        fact(styled, &bridge, b"help", selected, subject),
                                        fact(styled, &alias, b"trust", n[3], subject),
                                        fact(styled, &changed, b"trust", right_answer, subject),
                                    ]
                                };
                                let records = std::array::from_fn(|slot| {
                                    logical[(slot + 4 - rotation) % 4].clone()
                                });
                                let resolved = oracle(&records, &prompt)?;
                                let expected_answer = if variant == "active" { n[4] } else { n[3] };
                                let expected_sources: Vec<_> = if depth == 2 {
                                    vec![0, if variant == "active" { 2 } else { 1 }]
                                } else {
                                    vec![0, 1, if variant == "active" { 3 } else { 2 }]
                                };
                                if resolved.last().ok_or("empty path")?.value != expected_answer
                                    || resolved.iter().map(|r| r.source).collect::<Vec<_>>()
                                        != expected_sources
                                            .iter()
                                            .map(|s| (s + rotation) % 4)
                                            .collect::<Vec<_>>()
                                {
                                    return Err(
                                        "authored branch differs from independent oracle".into()
                                    );
                                }
                                rows.push(Example {
                                    id: format!("{family}-r{rotation}-{variant}"),
                                    family: family.clone(),
                                    placement: placement.into(),
                                    depth,
                                    length,
                                    variant: variant.into(),
                                    records,
                                    prompt: prompt.clone(),
                                    answer: expected_answer.to_vec(),
                                    expected_path: resolved
                                        .iter()
                                        .map(|r| [r.source, r.span[0]])
                                        .collect(),
                                    expected_spans: resolved
                                        .iter()
                                        .map(|r| [r.source, r.span[0], r.span[1]])
                                        .collect(),
                                    expected_bounds: resolved.iter().map(|r| r.bounds).collect(),
                                    expected_queries: resolved
                                        .iter()
                                        .map(|r| r.query.clone())
                                        .collect(),
                                    expected_payloads: resolved
                                        .iter()
                                        .map(|r| r.value.clone())
                                        .collect(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(rows)
}
fn check_row(e: &Example) -> Result<(usize, usize), String> {
    let qs = questions(&e.prompt)?;
    for raw in e
        .records
        .iter()
        .map(Vec::as_slice)
        .chain(qs.iter().copied())
        .chain(e.expected_queries.iter().map(Vec::as_slice))
    {
        let w = words(raw);
        if raw.len() > 128
            || w.is_empty()
            || w.len() > 16
            || w.iter()
                .any(|x| x.len() > 16 || !x.iter().all(u8::is_ascii_lowercase))
        {
            return Err("phrase lexical window exceeded".into());
        }
    }
    let out = oracle(&e.records, &e.prompt)?;
    if e.depth != out.len()
        || e.answer != out.last().ok_or("empty path")?.value
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
        return Err("phrase oracle metadata differs".into());
    }
    let expected_lengths: Vec<_> = match e.placement.as_str() {
        "first" => vec![e.length, 1],
        "middle" => vec![1, e.length, 1],
        "both" => vec![e.length, e.length, 1],
        _ => return Err("unknown placement".into()),
    };
    if !(1..=3).contains(&e.length)
        || out
            .iter()
            .map(|r| r.span[1] - r.span[0])
            .collect::<Vec<_>>()
            != expected_lengths
        || out.iter().any(|r| r.subject != out[0].subject)
    {
        return Err("phrase placement/role differs".into());
    }
    Ok((
        usize::from(e.prompt.starts_with(b"which person ")),
        usize::from(out[0].subject),
    ))
}
fn replace(record: &[u8], bounds: [usize; 2], payload: &[u8]) -> Vec<u8> {
    let mut out = record[..bounds[0]].to_vec();
    out.extend(payload);
    out.extend(&record[bounds[1]..]);
    out
}
pub fn corpus() -> Result<Vec<Example>, String> {
    let rows = authored()?;
    validate(&rows)?;
    Ok(rows)
}
pub fn validate(rows: &[Example]) -> Result<Value, String> {
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut families: BTreeMap<&str, BTreeMap<(usize, &str), &Example>> = BTreeMap::new();
    let mut styles = [0usize; 2];
    let mut roles = [0usize; 2];
    let mut lengths = [0usize; 3];
    let mut placements = BTreeMap::<&str, usize>::new();
    let mut variants = BTreeMap::<&str, usize>::new();
    for e in rows {
        let (s, r) = check_row(e)?;
        let suffix =
            e.id.strip_prefix(&format!("{}-r", e.family))
                .ok_or("family identity")?;
        let (rot, var) = suffix.split_once('-').ok_or("rotation variant")?;
        let rot: usize = rot.parse().map_err(|_| "rotation")?;
        if rot >= 4
            || var != e.variant
            || !VARIANTS.contains(&var)
            || !ids.insert(&e.id)
            || !inputs.insert((&e.records, &e.prompt))
        {
            return Err("duplicate or malformed phrase input".into());
        }
        if families
            .entry(&e.family)
            .or_default()
            .insert((rot, var), e)
            .is_some()
        {
            return Err("duplicate matched cell".into());
        }
        styles[s] += 1;
        roles[r] += 1;
        lengths[e.length - 1] += 1;
        *placements.entry(&e.placement).or_default() += 1;
        *variants.entry(&e.variant).or_default() += 1;
    }
    if rows.len() != 864
        || families.len() != 72
        || styles != [432; 2]
        || roles != [432; 2]
        || lengths != [288; 3]
        || PLACEMENTS.iter().any(|p| placements.get(p) != Some(&288))
        || VARIANTS.iter().any(|v| variants.get(v) != Some(&288))
    {
        return Err("phrase corpus count/balance differs".into());
    }
    for (family, cells) in &families {
        if cells.len() != 12 {
            return Err("incomplete phrase family".into());
        }
        let reference = cells.get(&(0, "baseline")).ok_or("unrotated baseline")?;
        for rotation in 0..4 {
            let get = |v| {
                cells
                    .get(&(rotation, v))
                    .copied()
                    .ok_or("missing matched variant")
            };
            let base = get("baseline")?;
            let active = get("active")?;
            let inactive = get("inactive")?;
            if base.prompt != reference.prompt
                || base.answer != reference.answer
                || (0..4).any(|s| base.records[(s + rotation) % 4] != reference.records[s])
                || active.prompt != base.prompt
                || inactive.prompt != base.prompt
                || inactive.answer != base.answer
                || active.answer == base.answer
            {
                return Err(format!("{family}: rotation/variant identity differs"));
            }
            let stage = base.depth - 2;
            let source = base.expected_path[stage][0];
            let mut expected = base.records.clone();
            expected[source] = replace(
                &expected[source],
                base.expected_bounds[stage],
                &active.expected_payloads[stage],
            );
            if active.records != expected {
                return Err("active intervention changes unrelated bytes".into());
            }
            let old = words(&base.expected_payloads[stage]);
            let new = words(&active.expected_payloads[stage]);
            if old.len() != new.len()
                || (old.len() > 1
                    && (old[..old.len() - 1] != new[..new.len() - 1] || old.last() == new.last()))
            {
                return Err("active phrase must retain first components and change last".into());
            }
            let unused = ((if base.depth == 2 { 2 } else { 3 }) + rotation) % 4;
            let styled = base.prompt.starts_with(b"which person ");
            let p = parse_fact(&base.records[unused], styled)?;
            let subject = base.family.contains("-v1-");
            let span = if subject { p.subject } else { p.object };
            let before = intervals(&base.records[unused]);
            let after = intervals(&inactive.records[unused]);
            let changed = parse_fact(&inactive.records[unused], styled)?;
            let c = if subject {
                changed.subject
            } else {
                changed.object
            };
            let payload = &inactive.records[unused][after[c[0]][0]..after[c[1] - 1][1]];
            let mut expected = base.records.clone();
            expected[unused] = replace(
                &expected[unused],
                [before[span[0]][0], before[span[1] - 1][1]],
                payload,
            );
            if expected != inactive.records
                || inactive.records == base.records
                || inactive.expected_path != base.expected_path
                || inactive.expected_queries != base.expected_queries
                || base.expected_path.iter().any(|p| p[0] == unused)
            {
                return Err("inactive branch intervention changed active path".into());
            }
        }
    }
    Ok(
        json!({"examples":rows.len(),"families":families.len(),"rows_per_family":12,"source_rotations":4,"name_cohorts":2,"styles":styles,"answer_roles":roles,"intermediate_lengths":lengths,"placements":placements,"variants":variants,
        "inputs":["records","prompt"],"evaluation_only":["answer","expected_path","expected_spans","expected_bounds","expected_queries","expected_payloads"],"training":"NOT_RUN","model_filtering":false,
        "scope":"authored whole-phrase sequential references and same-first-word branch competitors; shared role orientation along a chain; familiar vocabulary, open development","final_holdout":"NOT_RUN"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_phrase_oracle_rejects_truncation_and_unrelated_intervention() -> Result<(), String> {
        let rows = corpus()?;
        assert_eq!(rows.len(), 864);
        let mut e = rows
            .iter()
            .find(|e| e.length == 3 && e.depth == 3)
            .ok_or("long row")?
            .clone();
        e.expected_queries[2] = e.expected_queries[1].clone();
        assert!(check_row(&e).is_err());
        let mut e = rows
            .iter()
            .find(|e| e.length == 3)
            .ok_or("phrase row")?
            .clone();
        e.expected_bounds[0][1] -= 1;
        assert!(check_row(&e).is_err());
        let mut corrupt = rows.clone();
        let e = corrupt
            .iter_mut()
            .find(|e| e.variant == "active")
            .ok_or("active")?;
        let s = (e.expected_path[0][0] + 3) % 4;
        e.records[s].insert(0, b' ');
        assert!(validate(&corrupt).is_err());
        Ok(())
    }
}
