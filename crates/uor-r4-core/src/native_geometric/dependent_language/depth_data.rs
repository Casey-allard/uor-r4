//! Independent authored three-read transfer data; no model fitting or filtering.
//! Raw records and prompt are the only runtime inputs. Paths and intermediate
//! values belong exclusively to the typed evaluation oracle.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub variant: String,
    pub records: [Vec<u8>; 4],
    pub prompt: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_path: Vec<[usize; 2]>,
    pub intermediates: Vec<Vec<u8>>,
}
const NAMES: [&str; 8] = [
    "ruby", "felix", "clara", "dylan", "alice", "bruno", "helen", "oscar",
];
const VARIANTS: [&str; 5] = ["baseline", "first", "middle", "terminal", "inactive"];
struct Style {
    auxiliary: &'static str,
    adverb: &'static str,
    before: &'static str,
    prefix: &'static str,
}
// These are the retained relative-language constructions 0 and 1, unchanged.
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
fn render(parts: &[&str], punctuation: u8) -> Vec<u8> {
    let mut bytes = parts
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
        .into_bytes();
    bytes.push(punctuation);
    bytes
}
fn fact(style: &Style, known: &str, verb: &str, answer: &str, subject_answer: bool) -> Vec<u8> {
    let (subject, object) = if subject_answer {
        (answer, known)
    } else {
        (known, answer)
    };
    render(
        &[
            style.before,
            subject,
            style.auxiliary,
            style.adverb,
            verb,
            object,
        ],
        b'.',
    )
}
fn question(style: &Style, known: &str, verb: &str, subject_answer: bool) -> Vec<u8> {
    if subject_answer {
        render(
            &[style.prefix, style.auxiliary, style.adverb, verb, known],
            b'?',
        )
    } else {
        render(
            &[style.prefix, style.auxiliary, known, style.adverb, verb],
            b'?',
        )
    }
}
fn authored() -> Vec<Example> {
    let mut rows = Vec::with_capacity(1280);
    for offset in 0..NAMES.len() {
        let [a, b, c, d, e, f] = std::array::from_fn(|i| NAMES[(offset + i) % NAMES.len()]);
        for first_subject in [false, true] {
            for repeated_subject in [false, true] {
                for (style_index, style) in STYLES.iter().enumerate() {
                    let family = format!(
                        "depth-n{offset}-v{}-w{}-s{style_index}",
                        usize::from(first_subject),
                        usize::from(repeated_subject)
                    );
                    let mut prompt = question(style, a, "guide", first_subject);
                    let reference = if repeated_subject { "them" } else { "they" };
                    for _ in 0..2 {
                        prompt.push(b' ');
                        prompt.extend(question(style, reference, "trust", repeated_subject));
                    }
                    let subject_word = usize::from(!style.before.is_empty());
                    let object_word = subject_word + 3 + usize::from(!style.adverb.is_empty());
                    let first_word = if first_subject {
                        subject_word
                    } else {
                        object_word
                    };
                    let repeated_word = if repeated_subject {
                        subject_word
                    } else {
                        object_word
                    };
                    for rotation in 0..4 {
                        for variant in VARIANTS {
                            let first_value = if variant == "first" { c } else { b };
                            let middle_value = if variant == "middle" { d } else { c };
                            let terminal_value = if variant == "terminal" { f } else { d };
                            let tail_value = if variant == "inactive" { f } else { e };
                            let logical = [
                                fact(style, a, "guide", first_value, first_subject),
                                fact(style, b, "trust", middle_value, repeated_subject),
                                fact(style, c, "trust", terminal_value, repeated_subject),
                                fact(style, d, "trust", tail_value, repeated_subject),
                            ];
                            let records = std::array::from_fn(|slot| {
                                logical[(slot + 4 - rotation) % 4].clone()
                            });
                            let (logical_path, intermediates, answer) = match variant {
                                "first" => (
                                    [0, 2, 3],
                                    vec![c.as_bytes().to_vec(), d.as_bytes().to_vec()],
                                    e,
                                ),
                                "middle" => (
                                    [0, 1, 3],
                                    vec![b.as_bytes().to_vec(), d.as_bytes().to_vec()],
                                    e,
                                ),
                                "terminal" => (
                                    [0, 1, 2],
                                    vec![b.as_bytes().to_vec(), c.as_bytes().to_vec()],
                                    f,
                                ),
                                _ => (
                                    [0, 1, 2],
                                    vec![b.as_bytes().to_vec(), c.as_bytes().to_vec()],
                                    d,
                                ),
                            };
                            rows.push(Example {
                                id: format!("{family}-r{rotation}-{variant}"),
                                family: family.clone(),
                                variant: variant.into(),
                                records,
                                prompt: prompt.clone(),
                                answer: answer.as_bytes().to_vec(),
                                expected_path: logical_path
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, source)| {
                                        [
                                            (source + rotation) % 4,
                                            if i == 0 { first_word } else { repeated_word },
                                        ]
                                    })
                                    .collect(),
                                intermediates,
                            });
                        }
                    }
                }
            }
        }
    }
    rows
}
fn words(bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .split(|byte| !byte.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .collect()
}
fn questions(prompt: &[u8]) -> Result<Vec<&[u8]>, String> {
    if prompt.len() > 256 || prompt.last() != Some(&b'?') {
        return Err("invalid three-question prompt boundary".into());
    }
    let parts: Vec<_> = prompt
        .split_inclusive(|&byte| byte == b'?')
        .map(|part| part.strip_prefix(b" ").unwrap_or(part))
        .collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || part.last() != Some(&b'?'))
    {
        return Err("expected exactly three authored questions".into());
    }
    Ok(parts)
}
type Resolved = (usize, usize, Vec<u8>, bool);
/// Independent typed grammar check: resolve each question against all raw facts.
/// It does not use the authoring path formula or any model/geometry operation.
fn resolve(
    style: &Style,
    records: &[Vec<u8>; 4],
    query: &[u8],
    previous: Option<&[u8]>,
) -> Result<Resolved, String> {
    let mut q = words(query);
    let prefix: Vec<_> = style.prefix.split_whitespace().map(str::as_bytes).collect();
    let lead = prefix.len() + 1;
    if q.len() != lead + 2 + usize::from(!style.adverb.is_empty())
        || q[..prefix.len()] != prefix
        || q[prefix.len()] != style.auxiliary.as_bytes()
    {
        return Err("question differs from its known construction".into());
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
        _ => return Err("invalid sequential reference occurrence".into()),
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
            return Err("fact differs from its known construction".into());
        }
        let si = lead + usize::from(!style.adverb.is_empty());
        let oi = lead + 1 + usize::from(!style.adverb.is_empty());
        if (style.adverb.is_empty() || q[lead] == style.adverb.as_bytes())
            && r[verb] == q[si]
            && r[object] == q[si + 1]
        {
            matches.push((source, subject, r[subject].to_vec(), true));
        }
        if (style.adverb.is_empty() || q[lead + 1] == style.adverb.as_bytes())
            && r[subject] == q[lead]
            && r[verb] == q[oi]
        {
            matches.push((source, object, r[object].to_vec(), false));
        }
    }
    if matches.len() != 1 {
        return Err("typed query has absent or ambiguous answer".into());
    }
    matches
        .pop()
        .ok_or_else(|| "typed answer disappeared".into())
}
fn check_row(row: &Example) -> Result<(usize, [bool; 2]), String> {
    let queries = questions(&row.prompt)?;
    let style_index = if queries[0].starts_with(b"who ") {
        0
    } else if queries[0].starts_with(b"which person ") {
        1
    } else {
        return Err("unknown question construction".into());
    };
    let style = &STYLES[style_index];
    for bytes in row
        .records
        .iter()
        .map(Vec::as_slice)
        .chain(queries.iter().copied())
    {
        let tokens = words(bytes);
        if bytes.len() > 128
            || tokens.is_empty()
            || tokens.len() > 16
            || tokens
                .iter()
                .any(|word| word.len() > 16 || !word.iter().all(u8::is_ascii_lowercase))
        {
            return Err(format!(
                "{}: lexical window exceeds retained limits",
                row.id
            ));
        }
    }
    let first = resolve(style, &row.records, queries[0], None)?;
    let middle = resolve(style, &row.records, queries[1], Some(&first.2))?;
    let last = resolve(style, &row.records, queries[2], Some(&middle.2))?;
    if row.expected_path != vec![[first.0, first.1], [middle.0, middle.1], [last.0, last.1]]
        || row.intermediates != vec![first.2.clone(), middle.2.clone()]
        || row.answer != last.2
        || middle.3 != last.3
        || first.2 == middle.2
        || middle.2 == last.2
        || first.2 == last.2
    {
        return Err(format!(
            "{}: typed three-read path or values differ",
            row.id
        ));
    }
    if words(&row.prompt).iter().any(|word| {
        *word == row.answer.as_slice()
            || row
                .intermediates
                .iter()
                .any(|value| *word == value.as_slice())
    }) {
        return Err(format!(
            "{}: prompt supplies an intermediate or answer",
            row.id
        ));
    }
    Ok((style_index, [first.3, middle.3]))
}
fn replace(record: &[u8], word: usize, value: &[u8]) -> Result<Vec<u8>, String> {
    let mut tokens = words(record);
    *tokens
        .get_mut(word)
        .ok_or("intervention word outside record")? = value;
    let mut out = tokens.join(&b' ');
    out.push(b'.');
    Ok(out)
}
pub fn corpus() -> Result<Vec<Example>, String> {
    let rows = authored();
    validate(&rows)?;
    Ok(rows)
}
pub fn validate(rows: &[Example]) -> Result<Value, String> {
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut families: BTreeMap<&str, BTreeMap<usize, BTreeMap<&str, &Example>>> = BTreeMap::new();
    let mut styles = [0usize; 2];
    let mut roles = [0usize; 4];
    let mut source_counts = [[0usize; 4]; 3];
    let mut variant_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut record_lengths = BTreeSet::new();
    let mut question_lengths = BTreeSet::new();
    for row in rows {
        let (style, role) = check_row(row)?;
        let suffix = row
            .id
            .strip_prefix(&format!("{}-r", row.family))
            .ok_or("row ID/family mismatch")?;
        let (rotation, variant) = suffix.split_once('-').ok_or("missing rotation/variant")?;
        let rotation: usize = rotation.parse().map_err(|_| "invalid source rotation")?;
        if rotation >= 4
            || variant != row.variant
            || !VARIANTS.contains(&variant)
            || !ids.insert(&row.id)
            || !inputs.insert((&row.records, &row.prompt))
        {
            return Err(format!(
                "{}: duplicated input or malformed row identity",
                row.id
            ));
        }
        if families
            .entry(&row.family)
            .or_default()
            .entry(rotation)
            .or_default()
            .insert(&row.variant, row)
            .is_some()
        {
            return Err("duplicate intervention within family rotation".into());
        }
        styles[style] += 1;
        roles[usize::from(role[0]) * 2 + usize::from(role[1])] += 1;
        for (i, &[source, _]) in row.expected_path.iter().enumerate() {
            source_counts[i][source] += 1;
        }
        *variant_counts.entry(&row.variant).or_default() += 1;
        record_lengths.extend(row.records.iter().map(|record| words(record).len()));
        question_lengths.extend(
            questions(&row.prompt)?
                .iter()
                .map(|query| words(query).len()),
        );
    }
    if rows.len() != 1280
        || families.len() != 64
        || styles != [640, 640]
        || roles != [320; 4]
        || source_counts.iter().flatten().any(|&n| n != 320)
        || VARIANTS
            .iter()
            .any(|variant| variant_counts.get(variant) != Some(&256))
    {
        return Err(
            "incorrect corpus counts or unbalanced constructions, roles, sources or variants"
                .into(),
        );
    }
    for (family, rotations) in &families {
        if rotations.len() != 4 {
            return Err(format!("{family}: missing source rotation"));
        }
        let reference = rotations
            .get(&0)
            .and_then(|rows| rows.get("baseline"))
            .ok_or("missing rotation-zero baseline")?;
        for (&rotation, variants) in rotations {
            if variants.len() != 5 {
                return Err(format!("{family}: missing intervention variant"));
            }
            let get = |name: &str| {
                variants
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("{family}: missing {name}"))
            };
            let base = get("baseline")?;
            let first = get("first")?;
            let middle = get("middle")?;
            let terminal = get("terminal")?;
            let inactive = get("inactive")?;
            if variants.values().any(|row| row.prompt != base.prompt)
                || base.prompt != reference.prompt
                || base.answer != reference.answer
                || base.intermediates != reference.intermediates
                || (0..4).any(|slot| base.records[(slot + rotation) % 4] != reference.records[slot])
            {
                return Err(format!(
                    "{family}: rotation or intervention changes the question/context identity"
                ));
            }
            let [p0, p1, p2] = <[[usize; 2]; 3]>::try_from(base.expected_path.as_slice())
                .map_err(|_| "baseline depth")?;
            let unused = (0..4)
                .find(|source| !base.expected_path.iter().any(|p| p[0] == *source))
                .ok_or("missing inactive source")?;
            for (changed, source, word, value) in [
                (first, p0[0], p0[1], base.intermediates[1].as_slice()),
                (middle, p1[0], p1[1], base.answer.as_slice()),
                (terminal, p2[0], p2[1], terminal.answer.as_slice()),
                (inactive, unused, p2[1], terminal.answer.as_slice()),
            ] {
                let mut expected = base.records.clone();
                expected[source] = replace(&expected[source], word, value)?;
                if changed.records != expected {
                    return Err(format!("{family}: intervention modified unrelated bytes"));
                }
            }
            if first.answer == base.answer
                || middle.answer == base.answer
                || terminal.answer == base.answer
                || inactive.answer != base.answer
                || first.answer != middle.answer
                || first.expected_path[1][0] != p2[0]
                || first.expected_path[2][0] != unused
                || middle.expected_path[1] != p1
                || middle.expected_path[2][0] != unused
                || terminal.expected_path != base.expected_path
                || inactive.expected_path != base.expected_path
            {
                return Err(format!("{family}: causal path/value intervention mismatch"));
            }
        }
    }
    Ok(
        json!({"examples":rows.len(),"families":families.len(),"rows_per_family":20,
        "variants":variant_counts,"source_rotations":4,"name_rotations":8,"styles":styles,
        "style_descriptions":["did / who / no adjuncts","can quietly / which person / source prefix today"],
        "role_combinations":roles,"source_counts_by_read":source_counts,"record_word_lengths":record_lengths,"question_word_lengths":question_lengths,
        "unique_ids":ids.len(),"unique_raw_inputs":inputs.len(),"depth":3,
        "inference_inputs":["records","prompt"],"evaluation_only":["expected_path","intermediates","answer"],
        "training":"NOT_RUN","model_filtering":false,"syntax_scope":"existing constructions with three sequential reference questions; repeated relation orientation shared",
        "reference_scope":"each reference denotes the most recently selected value in the declared sequential-question protocol; not general discourse coreference"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn depth_transfer_oracle_rejects_stale_intermediate_and_unrelated_intervention(
    ) -> Result<(), String> {
        let rows = corpus()?;
        assert_eq!(rows.len(), 1280);
        assert_eq!(validate(&rows)?["families"], 64);
        let mut corrupt = rows[0].clone();
        corrupt.intermediates[1] = corrupt.intermediates[0].clone();
        assert!(check_row(&corrupt).is_err());
        let mut corrupt = rows.clone();
        let source = corrupt[1].expected_path[1][0];
        corrupt[1].records[source] = corrupt[0].records[corrupt[0].expected_path[0][0]].clone();
        assert!(validate(&corrupt).is_err());
        Ok(())
    }
}
