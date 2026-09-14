//! Authored contiguous answer spans over retained relative-language grammar.
//! The raw grammar oracle is independent of learned routing and span features.
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
    pub answer: Vec<u8>,
    pub expected_path: Vec<[usize; 2]>,
    pub expected_span: [usize; 3],
    pub expected_bounds: [usize; 2],
}
const NAMES: [&str; 8] = [
    "ruby", "felix", "clara", "dylan", "alice", "bruno", "helen", "oscar",
];
const VARIANTS: [&str; 3] = ["baseline", "active", "inactive"];
fn intervals(bytes: &[u8]) -> Vec<[usize; 2]> {
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
        } else if let Some(first) = start.take() {
            out.push([first, i]);
        }
    }
    out
}
fn words(bytes: &[u8]) -> Vec<&[u8]> {
    intervals(bytes)
        .into_iter()
        .map(|[a, b]| &bytes[a..b])
        .collect()
}
fn questions(prompt: &[u8]) -> Result<Vec<&[u8]>, String> {
    let result: Vec<_> = prompt
        .split_inclusive(|&b| b == b'?')
        .map(|q| q.strip_prefix(b" ").unwrap_or(q))
        .collect();
    if prompt.len() > 256
        || result.is_empty()
        || result.len() > 3
        || result.iter().any(|q| q.last() != Some(&b'?'))
    {
        return Err("invalid sequential question protocol".into());
    }
    Ok(result)
}
fn replace(record: &[u8], first: usize, last: usize, value: &[u8]) -> Result<Vec<u8>, String> {
    let spans = intervals(record);
    if first >= last || last > spans.len() {
        return Err("replacement span outside record".into());
    }
    let mut out = record[..spans[first][0]].to_vec();
    out.extend(value);
    out.extend(&record[spans[last - 1][1]..]);
    Ok(out)
}
fn phrase(name: &[u8], length: usize, changed: bool, alternative: &[u8]) -> Vec<u8> {
    match length {
        1 => {
            if changed {
                alternative.to_vec()
            } else {
                name.to_vec()
            }
        }
        2 => [name, if changed { b"cedar" } else { b"amber" }].join(&b' '),
        _ => [name, b"amber", if changed { b"cedar" } else { b"birch" }].join(&b' '),
    }
}
#[derive(Clone)]
struct Fact {
    subject: [usize; 2],
    object: [usize; 2],
    verb: usize,
}
fn fact(record: &[u8], styled: bool) -> Result<Fact, String> {
    let r = words(record);
    let aux = if styled {
        b"can".as_slice()
    } else {
        b"did".as_slice()
    };
    let positions: Vec<_> = r
        .iter()
        .enumerate()
        .filter(|(_, w)| **w == aux)
        .map(|(i, _)| i)
        .collect();
    if positions.len() != 1 || record.last() != Some(&b'.') {
        return Err("fact auxiliary or punctuation".into());
    }
    let subject = usize::from(styled);
    let auxiliary = positions[0];
    let verb = auxiliary + 1 + usize::from(styled);
    if auxiliary <= subject
        || verb + 1 >= r.len()
        || (styled
            && (r.first() != Some(&b"today".as_slice())
                || r.get(auxiliary + 1) != Some(&b"quietly".as_slice())))
        || !matches!(r[verb], b"guide" | b"trust")
    {
        return Err("fact outside declared phrase grammar".into());
    }
    Ok(Fact {
        subject: [subject, auxiliary],
        object: [verb + 1, r.len()],
        verb,
    })
}
#[derive(Clone)]
struct Resolved {
    source: usize,
    span: [usize; 2],
    bounds: [usize; 2],
    value: Vec<u8>,
    subject: bool,
}
fn resolve(
    records: &[Vec<u8>; 4],
    query: &[u8],
    previous: Option<&[u8]>,
) -> Result<Resolved, String> {
    let mut q = words(query);
    let styled = query.starts_with(b"which person ");
    let prefix: &[&[u8]] = if styled {
        &[b"which", b"person"]
    } else {
        &[b"who"]
    };
    let auxiliary = if styled {
        b"can".as_slice()
    } else {
        b"did".as_slice()
    };
    let lead = prefix.len() + 1;
    if q.len() != lead + 2 + usize::from(styled)
        || &q[..prefix.len()] != prefix
        || q[prefix.len()] != auxiliary
    {
        return Err("question outside retained construction".into());
    }
    let refs: Vec<_> = q
        .iter()
        .enumerate()
        .filter(|(_, w)| **w == b"they" || **w == b"them")
        .map(|(i, _)| i)
        .collect();
    match previous {
        Some(value) if refs.len() == 1 && words(value).len() == 1 => q[refs[0]] = value,
        None if refs.is_empty() => (),
        _ => return Err("invalid single-word intermediate reference".into()),
    }
    let mut answers = Vec::new();
    for (source, record) in records.iter().enumerate() {
        let parsed = fact(record, styled)?;
        let r = words(record);
        let si = lead + usize::from(styled);
        let oi = lead + 1 + usize::from(styled);
        let subject_query = (!styled || q[lead] == b"quietly")
            && r[parsed.verb] == q[si]
            && r[parsed.object[0]..parsed.object[1]] == [q[si + 1]];
        let object_query = (!styled || q[lead + 1] == b"quietly")
            && r[parsed.verb] == q[oi]
            && r[parsed.subject[0]..parsed.subject[1]] == [q[lead]];
        for (matched, span, subject) in [
            (subject_query, parsed.subject, true),
            (object_query, parsed.object, false),
        ] {
            if matched {
                let positions = intervals(record);
                let bounds = [positions[span[0]][0], positions[span[1] - 1][1]];
                answers.push(Resolved {
                    source,
                    span,
                    bounds,
                    value: record[bounds[0]..bounds[1]].to_vec(),
                    subject,
                });
            }
        }
    }
    if answers.len() != 1 {
        return Err("raw grammar answer absent or ambiguous".into());
    }
    answers
        .pop()
        .ok_or_else(|| "raw grammar answer disappeared".into())
}
fn oracle(records: &[Vec<u8>; 4], prompt: &[u8]) -> Result<Vec<Resolved>, String> {
    let mut resolved: Vec<Resolved> = Vec::new();
    for query in questions(prompt)? {
        let next = resolve(records, query, resolved.last().map(|r| r.value.as_slice()))?;
        resolved.push(next);
    }
    Ok(resolved)
}
fn authored() -> Result<Vec<Example>, String> {
    let mut rows = Vec::with_capacity(2304);
    for base in depth_data::corpus()?
        .into_iter()
        .filter(|r| r.variant == "baseline")
    {
        let mut matched = None;
        for offset in 0..8 {
            for role in 0..2 {
                for style in 0..2 {
                    if base.family == format!("depth-n{offset}-v{role}-w{role}-s{style}") {
                        matched = Some((offset, role, style));
                    }
                }
            }
        }
        let Some((offset, role, style)) = matched else {
            continue;
        };
        let rotation: usize = base
            .id
            .strip_prefix(&format!("{}-r", base.family))
            .and_then(|s| s.strip_suffix("-baseline"))
            .ok_or("depth baseline rotation")?
            .parse()
            .map_err(|_| "invalid rotation")?;
        let alternate = NAMES
            .iter()
            .find(|name| {
                !base
                    .records
                    .iter()
                    .any(|r| words(r).contains(&name.as_bytes()))
            })
            .ok_or("unused existing name")?
            .as_bytes();
        let all_questions = questions(&base.prompt)?;
        for length in 1..=3 {
            let family = format!("span-n{offset}-v{role}-s{style}-l{length}");
            for depth in 1..=if offset < 4 { 1 } else { 3 } {
                let prompt = all_questions[..depth].join(&b' ');
                let [source, word] = base.expected_path[depth - 1];
                let original_words = words(&base.records[source]);
                let value = original_words[word];
                let baseline_phrase = phrase(value, length, false, alternate);
                for variant in VARIANTS {
                    let mut records = base.records.clone();
                    records[source] = replace(
                        &records[source],
                        word,
                        word + 1,
                        &phrase(value, length, variant == "active", alternate),
                    )?;
                    if variant == "inactive" {
                        let inactive = (3 + rotation) % 4;
                        let parsed = fact(&records[inactive], style == 1)?;
                        let range = if role == 1 {
                            parsed.subject
                        } else {
                            parsed.object
                        };
                        let inactive_words = words(&records[inactive]);
                        let changed = phrase(inactive_words[range[0]], length, true, alternate);
                        records[inactive] =
                            replace(&records[inactive], range[0], range[1], &changed)?;
                    }
                    let resolved = oracle(&records, &prompt)?;
                    let final_read = resolved.last().ok_or("empty resolved path")?;
                    let answer = if variant == "active" {
                        phrase(value, length, true, alternate)
                    } else {
                        baseline_phrase.clone()
                    };
                    if final_read.value != answer {
                        return Err(
                            "authored final phrase disagrees with independent grammar".into()
                        );
                    }
                    rows.push(Example {
                        id: format!("{family}-r{rotation}-d{depth}-{variant}"),
                        family: family.clone(),
                        depth,
                        variant: variant.into(),
                        records,
                        prompt: prompt.clone(),
                        answer,
                        expected_path: base.expected_path[..depth].to_vec(),
                        expected_span: [source, word, word + length],
                        expected_bounds: final_read.bounds,
                    });
                }
            }
        }
    }
    Ok(rows)
}
pub fn split(rows: &[Example]) -> (Vec<Example>, Vec<Example>) {
    rows.iter()
        .cloned()
        .partition(|r| (0..4).any(|n| r.family.starts_with(&format!("span-n{n}-"))))
}
/// Preserve the earlier direct training distribution without consuming any of
/// its development targets. Its own raw grammar validator remains authoritative.
pub fn training_with_retained(new_training: &[Example]) -> Result<Vec<Example>, String> {
    if new_training.len() != 576 || new_training.iter().any(|r| r.depth != 1) {
        return Err("new direct training cohort differs".into());
    }
    let (retained, _) = crate::native_geometric::relative_language::data::corpus()?;
    if retained.len() != 2048 {
        return Err("retained direct training count differs".into());
    }
    let mut rows = new_training.to_vec();
    for e in retained {
        let positions = intervals(&e.records[e.expected_source]);
        let bounds = *positions
            .get(e.expected_word)
            .ok_or("retained answer occurrence outside source")?;
        if e.records[e.expected_source][bounds[0]..bounds[1]] != e.answer {
            return Err("retained direct answer differs from exact occurrence".into());
        }
        rows.push(Example {
            id: format!("retained-{}", e.id),
            family: format!("retained-{}", e.family),
            depth: 1,
            variant: "retained_training".into(),
            records: e.records,
            prompt: e.question,
            answer: e.answer,
            expected_path: vec![[e.expected_source, e.expected_word]],
            expected_span: [e.expected_source, e.expected_word, e.expected_word + 1],
            expected_bounds: bounds,
        });
    }
    Ok(rows)
}
fn check_row(row: &Example) -> Result<(usize, bool, usize), String> {
    let qs = questions(&row.prompt)?;
    if qs.len() != row.depth || row.answer.len() > 50 || row.answer.is_empty() {
        return Err("declared depth or answer window differs".into());
    }
    for bytes in row
        .records
        .iter()
        .map(Vec::as_slice)
        .chain(qs.iter().copied())
    {
        let ws = words(bytes);
        if bytes.len() > 128
            || ws.is_empty()
            || ws.len() > 16
            || ws
                .iter()
                .any(|w| w.len() > 16 || !w.iter().all(u8::is_ascii_lowercase))
        {
            return Err("retained lexical window exceeded".into());
        }
    }
    let result = oracle(&row.records, &row.prompt)?;
    let final_read = result.last().ok_or("empty resolved path")?;
    let path: Vec<_> = result.iter().map(|r| [r.source, r.span[0]]).collect();
    let length = final_read.span[1] - final_read.span[0];
    if !(1..=3).contains(&length)
        || row.expected_path != path
        || row.answer != final_read.value
        || row.expected_span != [final_read.source, final_read.span[0], final_read.span[1]]
        || row.expected_bounds != final_read.bounds
        || result[..result.len() - 1]
            .iter()
            .any(|r| r.span[1] - r.span[0] != 1)
        || result.iter().any(|r| r.subject != final_read.subject)
    {
        return Err(format!(
            "{}: typed span, boundary, answer or intermediate differs",
            row.id
        ));
    }
    Ok((
        usize::from(row.prompt.starts_with(b"which person ")),
        final_read.subject,
        length,
    ))
}
pub fn corpus() -> Result<Vec<Example>, String> {
    let rows = authored()?;
    validate(&rows)?;
    Ok(rows)
}
pub fn validate(rows: &[Example]) -> Result<Value, String> {
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut groups: BTreeMap<&str, BTreeMap<(usize, usize, &str), &Example>> = BTreeMap::new();
    let mut styles = [0usize; 2];
    let mut roles = [0usize; 2];
    let mut lengths = [0usize; 3];
    let mut depths = [0usize; 3];
    let mut variants: BTreeMap<&str, usize> = BTreeMap::new();
    for row in rows {
        let (style, subject, length) = check_row(row)?;
        let suffix = row
            .id
            .strip_prefix(&format!("{}-r", row.family))
            .ok_or("row/family identity mismatch")?;
        let (rotation, tail) = suffix.split_once("-d").ok_or("rotation/depth missing")?;
        let rotation: usize = rotation.parse().map_err(|_| "rotation invalid")?;
        let (depth, variant) = tail.split_once('-').ok_or("depth/variant missing")?;
        let depth: usize = depth.parse().map_err(|_| "depth invalid")?;
        if rotation >= 4
            || depth != row.depth
            || variant != row.variant
            || !VARIANTS.contains(&variant)
            || !ids.insert(&row.id)
            || !inputs.insert((&row.records, &row.prompt))
        {
            return Err("duplicate input or malformed corpus identity".into());
        }
        if groups
            .entry(&row.family)
            .or_default()
            .insert((depth, rotation, variant), row)
            .is_some()
        {
            return Err("duplicate matched family cell".into());
        }
        styles[style] += 1;
        roles[usize::from(subject)] += 1;
        lengths[length - 1] += 1;
        depths[depth - 1] += 1;
        *variants.entry(variant).or_default() += 1;
    }
    let (train, dev) = split(rows);
    if rows.len() != 2304
        || groups.len() != 96
        || train.len() != 576
        || dev.len() != 1728
        || train.iter().any(|r| r.depth != 1)
        || styles != [1152; 2]
        || roles != [1152; 2]
        || lengths != [768; 3]
        || depths != [1152, 576, 576]
        || VARIANTS.iter().any(|v| variants.get(v) != Some(&768))
    {
        return Err("span corpus counts or balance differ".into());
    }
    for (family, cells) in &groups {
        let is_train = (0..4).any(|n| family.starts_with(&format!("span-n{n}-")));
        let max_depth = if is_train { 1 } else { 3 };
        if cells.len() != max_depth * 12 {
            return Err("incomplete matched family".into());
        }
        for depth in 1..=max_depth {
            let reference = cells
                .get(&(depth, 0, "baseline"))
                .ok_or("missing unrotated baseline")?;
            for rotation in 0..4 {
                let get = |variant| {
                    cells
                        .get(&(depth, rotation, variant))
                        .copied()
                        .ok_or("missing variant")
                };
                let base = get("baseline")?;
                let active = get("active")?;
                let inactive = get("inactive")?;
                if base.prompt != reference.prompt
                    || base.answer != reference.answer
                    || (0..4).any(|s| base.records[(s + rotation) % 4] != reference.records[s])
                    || active.prompt != base.prompt
                    || inactive.prompt != base.prompt
                    || active.expected_path != base.expected_path
                    || inactive.expected_path != base.expected_path
                    || inactive.answer != base.answer
                    || active.answer == base.answer
                {
                    return Err("rotation or matched intervention identity differs".into());
                }
                let [source, first, last] = base.expected_span;
                let mut expected = base.records.clone();
                expected[source] = replace(&expected[source], first, last, &active.answer)?;
                if active.records != expected {
                    return Err("active intervention modified unrelated bytes".into());
                }
                let base_words = words(&base.answer);
                let active_words = words(&active.answer);
                if active_words.len() != base_words.len()
                    || (base_words.len() > 1
                        && (base_words[..base_words.len() - 1]
                            != active_words[..active_words.len() - 1]
                            || base_words.last() == active_words.last()))
                {
                    return Err(
                        "active multiword intervention must change only final component".into(),
                    );
                }
                let unused = (3 + rotation) % 4;
                if base.expected_path.iter().any(|p| p[0] == unused) {
                    return Err("inactive source is active".into());
                }
                let styled = base.prompt.starts_with(b"which person ");
                let parsed = fact(&base.records[unused], styled)?;
                let subject = oracle(&base.records, &base.prompt)?
                    .last()
                    .ok_or("empty path")?
                    .subject;
                let before = if subject {
                    parsed.subject
                } else {
                    parsed.object
                };
                let changed = fact(&inactive.records[unused], styled)?;
                let after = if subject {
                    changed.subject
                } else {
                    changed.object
                };
                let spans = intervals(&inactive.records[unused]);
                let value = &inactive.records[unused][spans[after[0]][0]..spans[after[1] - 1][1]];
                let mut expected = base.records.clone();
                expected[unused] = replace(&expected[unused], before[0], before[1], value)?;
                if inactive.records != expected
                    || inactive.records == base.records
                    || words(value).len() != base_words.len()
                {
                    return Err(
                        "inactive intervention modified unrelated bytes or phrase length".into(),
                    );
                }
            }
        }
    }
    Ok(
        json!({"examples":rows.len(),"families":groups.len(),"training_examples":train.len(),"development_examples":dev.len(),
        "styles":styles,"answer_roles":roles,"answer_word_lengths":lengths,"depths":depths,"variants":variants,
        "source_rotations":4,"name_offsets":8,"split":"whole families; offsets 0-3 direct training only; offsets 4-7 depths 1-3 development",
        "name_vocabulary_overlap":true,"phrase_construction":"authored composed names: name / name amber / name amber birch; active change final component",
        "inference_inputs":["records","prompt"],"training_targets":["answer bytes","EOS"],
        "evaluation_only":["expected_path","expected_span","expected_bounds","depth","variant","family"],
        "model_filtering":false,"final_holdout":"NOT_RUN","natural_prose_claim":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn span_oracle_rejects_wrong_boundaries_and_unrelated_source_edits() -> Result<(), String> {
        let rows = corpus()?;
        let (train, dev) = split(&rows);
        assert_eq!((train.len(), dev.len()), (576, 1728));
        let families: BTreeSet<_> = train.iter().map(|r| &r.family).collect();
        assert!(dev.iter().all(|r| !families.contains(&r.family)));
        let multi = rows
            .iter()
            .find(|r| words(&r.answer).len() == 3)
            .ok_or("missing multiword row")?;
        let mut corrupt = multi.clone();
        corrupt.expected_span[2] -= 1;
        assert!(check_row(&corrupt).is_err());
        let mut corrupt = multi.clone();
        corrupt.expected_bounds[1] += 1;
        assert!(check_row(&corrupt).is_err());
        let mut corrupted = rows.clone();
        let active = corrupted
            .iter_mut()
            .find(|r| r.variant == "active" && r.depth == 1)
            .ok_or("missing active row")?;
        let other = (active.expected_span[0] + 1) % 4;
        active.records[other].insert(0, b' ');
        assert!(validate(&corrupted).is_err());
        Ok(())
    }
}
