//! Frozen matched completion cases derived from the retained depth corpus.
//! Typed outcomes describe only these authored records, not global fact absence.
use super::depth_data;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub depth: usize,
    pub variant: String,
    pub records: [Vec<u8>; 4],
    pub prompt: Vec<u8>,
    pub answer: Option<Vec<u8>>,
    pub resolved_prefix: Vec<[usize; 2]>,
    pub blocked_at: Option<usize>,
    pub expected_reason: String,
}
const NAMES: [&str; 8] = [
    "ruby", "felix", "clara", "dylan", "alice", "bruno", "helen", "oscar",
];
struct Style {
    auxiliary: &'static str,
    adverb: &'static str,
    before: &'static str,
    prefix: &'static str,
}
const STYLES: [Style; 2] = [
    Style {
        auxiliary: "did",
        adverb: "",
        before: "",
        prefix: "who",
    },
    Style {
        auxiliary: "can",
        adverb: "quietly",
        before: "today",
        prefix: "which person",
    },
];
fn words(bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .split(|byte| !byte.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .collect()
}
fn questions(prompt: &[u8]) -> Result<Vec<&[u8]>, String> {
    let q: Vec<_> = prompt
        .split_inclusive(|&byte| byte == b'?')
        .map(|p| p.strip_prefix(b" ").unwrap_or(p))
        .collect();
    if prompt.len() > 256
        || q.is_empty()
        || q.len() > 3
        || q.iter().any(|p| p.is_empty() || p.last() != Some(&b'?'))
    {
        return Err("invalid question protocol".into());
    }
    Ok(q)
}
fn replace(record: &[u8], old: &[u8], value: &[u8]) -> Result<Vec<u8>, String> {
    let mut tokens = words(record);
    let positions: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, word)| **word == old)
        .map(|(i, _)| i)
        .collect();
    if positions.len() != 1 {
        return Err("known endpoint is not unique in its authored fact".into());
    }
    tokens[positions[0]] = value;
    let mut out = tokens.join(&b' ');
    out.push(b'.');
    Ok(out)
}
fn authored() -> Result<Vec<Example>, String> {
    let mut rows = Vec::with_capacity(1792);
    for base in depth_data::corpus()?
        .into_iter()
        .filter(|row| row.variant == "baseline")
    {
        let family = base
            .id
            .strip_suffix("-baseline")
            .ok_or("baseline id")?
            .to_string();
        let all_questions = questions(&base.prompt)?;
        let unused = NAMES
            .iter()
            .find(|name| {
                !base
                    .records
                    .iter()
                    .any(|r| words(r).contains(&name.as_bytes()))
            })
            .ok_or("unused name")?
            .as_bytes();
        let inactive = (0..4)
            .find(|source| !base.expected_path.iter().any(|p| p[0] == *source))
            .ok_or("inactive source")?;
        for depth in 1..=3 {
            let prompt = all_questions[..depth].join(&b' ');
            for variant in if depth == 1 {
                &["valid"][..]
            } else {
                &["valid", "missing", "conflict"][..]
            } {
                let mut records = base.records.clone();
                let blocked = *variant != "valid";
                if blocked {
                    let known = &base.intermediates[depth - 2];
                    if *variant == "missing" {
                        let required = base.expected_path[depth - 1][0];
                        records[required] = replace(&records[required], known, unused)?;
                    } else {
                        records[inactive] = replace(&records[inactive], &base.answer, known)?;
                    }
                }
                rows.push(Example {
                    id: format!("{family}-d{depth}-{variant}"),
                    family: family.clone(),
                    depth,
                    variant: (*variant).into(),
                    records,
                    prompt: prompt.clone(),
                    answer: if blocked {
                        None
                    } else if depth < 3 {
                        Some(base.intermediates[depth - 1].clone())
                    } else {
                        Some(base.answer.clone())
                    },
                    resolved_prefix: base.expected_path[..if blocked { depth - 1 } else { depth }]
                        .to_vec(),
                    blocked_at: blocked.then_some(depth - 1),
                    expected_reason: match *variant {
                        "missing" => "NoCompatibleCandidate",
                        "conflict" => "Ambiguous",
                        _ => "Completed",
                    }
                    .into(),
                });
            }
        }
    }
    Ok(rows)
}
type Resolved = (usize, usize, Vec<u8>);
/// Offline typed relation oracle, independent of learned rules and geometry.
fn resolve(
    style: &Style,
    records: &[Vec<u8>; 4],
    query: &[u8],
    previous: Option<&[u8]>,
) -> Result<Vec<Resolved>, String> {
    let mut q = words(query);
    let prefix: Vec<_> = style.prefix.split_whitespace().map(str::as_bytes).collect();
    let lead = prefix.len() + 1;
    if q.len() != lead + 2 + usize::from(!style.adverb.is_empty())
        || q[..prefix.len()] != prefix
        || q[prefix.len()] != style.auxiliary.as_bytes()
    {
        return Err("question is outside its authored construction".into());
    }
    let refs: Vec<_> = q
        .iter()
        .enumerate()
        .filter(|(_, word)| **word == b"they" || **word == b"them")
        .map(|(i, _)| i)
        .collect();
    match previous {
        Some(value) if refs.len() == 1 => q[refs[0]] = value,
        None if refs.is_empty() => (),
        _ => return Err("invalid reference occurrence".into()),
    }
    let subject = usize::from(!style.before.is_empty());
    let verb = subject + 2 + usize::from(!style.adverb.is_empty());
    let object = verb + 1;
    let mut matches = Vec::new();
    for (source, record) in records.iter().enumerate() {
        let r = words(record);
        if r.len() != object + 1
            || r[subject + 1] != style.auxiliary.as_bytes()
            || (!style.before.is_empty() && r[0] != style.before.as_bytes())
            || (!style.adverb.is_empty() && r[subject + 2] != style.adverb.as_bytes())
            || record.last() != Some(&b'.')
        {
            return Err("fact is outside its authored construction".into());
        }
        let si = lead + usize::from(!style.adverb.is_empty());
        let oi = lead + 1 + usize::from(!style.adverb.is_empty());
        if (style.adverb.is_empty() || q[lead] == style.adverb.as_bytes())
            && r[verb] == q[si]
            && r[object] == q[si + 1]
        {
            matches.push((source, subject, r[subject].to_vec()));
        }
        if (style.adverb.is_empty() || q[lead + 1] == style.adverb.as_bytes())
            && r[subject] == q[lead]
            && r[verb] == q[oi]
        {
            matches.push((source, object, r[object].to_vec()));
        }
    }
    Ok(matches)
}
fn check_row(row: &Example) -> Result<(), String> {
    let qs = questions(&row.prompt)?;
    if qs.len() != row.depth {
        return Err("depth metadata mismatch".into());
    }
    let style = if qs[0].starts_with(b"who ") {
        &STYLES[0]
    } else if qs[0].starts_with(b"which person ") {
        &STYLES[1]
    } else {
        return Err("unknown style".into());
    };
    for bytes in row
        .records
        .iter()
        .map(Vec::as_slice)
        .chain(qs.iter().copied())
    {
        let tokens = words(bytes);
        if bytes.len() > 128
            || tokens.is_empty()
            || tokens.len() > 16
            || tokens
                .iter()
                .any(|w| w.len() > 16 || !w.iter().all(u8::is_ascii_lowercase))
        {
            return Err("lexical window".into());
        }
    }
    let mut previous: Option<Vec<u8>> = None;
    let mut path = Vec::new();
    let mut blocked = None;
    let mut reason = "Completed";
    for (clause, q) in qs.iter().enumerate() {
        let found = resolve(style, &row.records, q, previous.as_deref())?;
        if found.len() != 1 {
            reason = if found.is_empty() {
                "NoCompatibleCandidate"
            } else {
                "Ambiguous"
            };
            if found.len() > 1
                && found
                    .iter()
                    .map(|(_, _, value)| value)
                    .collect::<BTreeSet<_>>()
                    .len()
                    < 2
            {
                return Err("conflict must have differing authored answers".into());
            }
            blocked = Some(clause);
            break;
        }
        let (source, word, value) = found.into_iter().next().ok_or("typed answer disappeared")?;
        path.push([source, word]);
        previous = Some(value);
    }
    let answer = if blocked.is_none() { previous } else { None };
    if path != row.resolved_prefix
        || blocked != row.blocked_at
        || reason != row.expected_reason
        || answer != row.answer
    {
        return Err(format!("{}: typed completion outcome mismatch", row.id));
    }
    Ok(())
}
pub fn corpus() -> Result<Vec<Example>, String> {
    let rows = authored()?;
    validate(&rows)?;
    Ok(rows)
}
pub fn validate(rows: &[Example]) -> Result<Value, String> {
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut groups: BTreeMap<&str, BTreeMap<(usize, &str), &Example>> = BTreeMap::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        check_row(row)?;
        if row.id != format!("{}-d{}-{}", row.family, row.depth, row.variant)
            || !ids.insert(&row.id)
            || !inputs.insert((&row.records, &row.prompt))
            || groups
                .entry(&row.family)
                .or_default()
                .insert((row.depth, &row.variant), row)
                .is_some()
        {
            return Err("duplicate or inconsistent completion row".into());
        }
        *counts
            .entry(format!("depth{}/{}", row.depth, row.variant))
            .or_default() += 1;
    }
    if rows.len() != 1792
        || groups.len() != 256
        || counts.len() != 7
        || counts.values().any(|&count| count != 256)
    {
        return Err("completion corpus count mismatch".into());
    }
    for (family, group) in &groups {
        if group.len() != 7 {
            return Err(format!("{family}: incomplete matched family"));
        }
        let baseline = *group.get(&(3, "valid")).ok_or("missing depth3 baseline")?;
        let first_value = words(&baseline.records[baseline.resolved_prefix[0][0]])
            [baseline.resolved_prefix[0][1]];
        let inactive = (0..4)
            .find(|source| !baseline.resolved_prefix.iter().any(|p| p[0] == *source))
            .ok_or("inactive source")?;
        let terminal = baseline.answer.as_ref().ok_or("baseline answer")?;
        for depth in 1..=3 {
            let valid = *group.get(&(depth, "valid")).ok_or("valid case missing")?;
            if valid.records != baseline.records
                || valid.resolved_prefix != baseline.resolved_prefix[..depth]
                || questions(&valid.prompt)? != questions(&baseline.prompt)?[..depth]
            {
                return Err("valid truncation changes source/query identity".into());
            }
            if depth == 1 {
                continue;
            }
            let known = if depth == 2 {
                first_value
            } else {
                words(&baseline.records[baseline.resolved_prefix[1][0]])
                    [baseline.resolved_prefix[1][1]]
            };
            for variant in ["missing", "conflict"] {
                let changed = *group
                    .get(&(depth, variant))
                    .ok_or("unresolved case missing")?;
                if changed.prompt != valid.prompt
                    || changed.resolved_prefix != valid.resolved_prefix[..depth - 1]
                    || changed.blocked_at != Some(depth - 1)
                {
                    return Err("unresolved case changes an earlier selected path/query".into());
                }
                let mut expected = valid.records.clone();
                if variant == "missing" {
                    let fresh = NAMES
                        .iter()
                        .find(|name| {
                            !valid
                                .records
                                .iter()
                                .any(|r| words(r).contains(&name.as_bytes()))
                        })
                        .ok_or("unused name")?;
                    let source = valid.resolved_prefix[depth - 1][0];
                    expected[source] = replace(&expected[source], known, fresh.as_bytes())?;
                } else {
                    expected[inactive] = replace(&expected[inactive], terminal, known)?;
                }
                if changed.records != expected {
                    return Err("unresolved intervention changes unrelated source bytes".into());
                }
            }
        }
    }
    Ok(
        json!({"rows":rows.len(),"matched_families":groups.len(),"underlying_depth_families":64,"rows_per_matched_family":7,"counts":counts,
        "unique_ids":ids.len(),"unique_inputs":inputs.len(),"resolved_rows":768,"unresolved_rows":1024,
        "inference_inputs":["records","prompt"],"evaluation_only":["answer","resolved_prefix","blocked_at","expected_reason"],
        "blocked_at_indexing":"zero-based clause index","outcomes":["Completed","NoCompatibleCandidate","Ambiguous"],
        "scope":"missing or conflicting continuation in these authored records; no global absence claim","model_filtering":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_oracle_separates_terminal_from_unresolved_and_rejects_prefix_damage(
    ) -> Result<(), String> {
        let rows = corpus()?;
        assert_eq!(rows.len(), 1792);
        assert_eq!(validate(&rows)?["unresolved_rows"], 1024);
        let mut bad = rows
            .iter()
            .find(|r| r.variant == "missing")
            .ok_or("missing case")?
            .clone();
        bad.expected_reason = "Completed".into();
        bad.answer = Some(b"felix".to_vec());
        assert!(check_row(&bad).is_err());
        let mut bad = rows.clone();
        let i = bad
            .iter()
            .position(|r| r.variant == "conflict")
            .ok_or("conflict")?;
        let first = bad[i].resolved_prefix[0][0];
        bad[i].records[first] = bad[i].records[(first + 1) % 4].clone();
        assert!(validate(&bad).is_err());
        Ok(())
    }
}
