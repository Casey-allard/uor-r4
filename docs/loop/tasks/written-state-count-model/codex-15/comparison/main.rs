//! Readout over already graded frozen rows. No model or selector executes here.
mod bootstrap;

use bootstrap::{confidence, Interval};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];

const TESTS: [(&str, usize, usize); 4] = [
    ("public_identity", 0, 1),
    ("public_frozen", 0, 2),
    ("bounded_identity", 3, 4),
    ("bounded_frozen", 3, 5),
];
const INC: &str = "incidental-no-request";
const PROSE: &str = "unbound-prose";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Paths {
    blind: PathBuf,
    labels: PathBuf,
    matching: PathBuf,
    public_positional: PathBuf,
    public_identity: PathBuf,
    public_frozen: PathBuf,
    bounded_positional: PathBuf,
    bounded_identity: PathBuf,
    bounded_frozen: PathBuf,
    fixed_positional: PathBuf,
    fixed_identity: PathBuf,
    correct_oracle: PathBuf,
    #[serde(default)]
    diagnostics: BTreeMap<String, PathBuf>,
}

#[derive(Deserialize)]
struct Label {
    id: String,
    family: String,
    pair: String,
    direction: usize,
    stratum: String,
    counterpart: Option<String>,
    requested: Option<Span>,
    requested_name: Option<String>,
    expected: Option<Vec<u32>>,
    prefix_sha256: String,
    facts: Vec<Fact>,
}

#[derive(Deserialize)]
struct Fact {
    name: String,
    value_span: Span,
}

#[derive(Deserialize)]
struct GradeFile {
    schema: String,
    grades: String,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    index: usize,
    id: String,
    family: String,
    pair: String,
    direction: usize,
    stratum: String,
    counterpart: Option<String>,
    requested: Option<Span>,
    gold_requested: Option<Span>,
    endpoint_exact: bool,
    source: Option<usize>,
    source_matches_requested_value: bool,
    expected: Option<Vec<u32>>,
    generated: Vec<u32>,
    terminal_eos: bool,
    exact: Option<bool>,
    first_wrong: Option<usize>,
    expected_wrong_id: Option<u32>,
    actual_wrong_id: Option<u32>,
    first_free: Value,
    text: Value,
    acceptable_prose: Value,
}

impl Row {
    fn exact_reply(&self) -> bool {
        self.exact == Some(true) && self.terminal_eos
    }

    fn unsupported_copy(&self) -> bool {
        self.source.is_some() && (self.expected.is_none() || !self.source_matches_requested_value)
    }

    fn observation(&self) -> Value {
        json!({
            "first_wrong":self.first_wrong, "expected_wrong_id":self.expected_wrong_id,
            "actual_wrong_id":self.actual_wrong_id, "requested":self.requested,
            "endpoint_exact":self.endpoint_exact, "source":self.source,
            "correct_source":self.source_matches_requested_value,
            "terminal_eos":self.terminal_eos, "first_free":self.first_free,
            "first_id":self.generated.first(), "capped":!self.terminal_eos && self.generated.len()==16,
        })
    }

    fn validate(&self, label: &Label) -> Result<()> {
        let first_wrong = self.expected.as_ref().and_then(|ids| {
            (0..ids.len().max(self.generated.len())).find(|&i| ids.get(i) != self.generated.get(i))
        });
        let expected_wrong_id = first_wrong
            .and_then(|i| self.expected.as_ref()?.get(i))
            .copied();
        let actual_wrong_id = first_wrong.and_then(|i| self.generated.get(i)).copied();
        let exact = self.expected.as_ref().map(|_| first_wrong.is_none());
        let source_matches = label.facts.iter().any(|fact| {
            Some(&fact.name) == label.requested_name.as_ref()
                && Some(fact.value_span[0]) == self.source
        });
        if self.id != label.id
            || self.family != label.family
            || self.pair != label.pair
            || self.stratum != label.stratum
            || self.direction != label.direction
            || self.counterpart != label.counterpart
            || self.gold_requested != label.requested
            || self.expected != label.expected
            || self.endpoint_exact != (self.requested == label.requested)
            || self.terminal_eos != (self.generated.last() == Some(&1))
            || self.first_wrong != first_wrong
            || self.exact != exact
            || self.expected_wrong_id != expected_wrong_id
            || self.actual_wrong_id != actual_wrong_id
            || self.source_matches_requested_value != source_matches
            || self.generated.is_empty()
            || self.generated.len() > 16
        {
            return Err(format!(
                "unbound or inconsistent grade at {} ({})",
                self.index, self.id
            )
            .into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct Match {
    pair: String,
    family: String,
    row_left: usize,
    row_right: usize,
    left: [Option<Span>; 2],
    right: [Option<Span>; 2],
    eligible: bool,
    reasons: Vec<String>,
}

#[derive(Deserialize)]
struct FixedFile {
    schema: String,
    mode: String,
    bare_sha256: String,
    mapping_sha256: String,
    pairs: Vec<FixedPair>,
}

#[derive(Deserialize)]
struct FixedPair {
    family: String,
    pair: String,
    matched: bool,
    reasons: Vec<String>,
    left: FixedDirection,
    right: FixedDirection,
}

#[derive(Deserialize)]
struct FixedDirection {
    row: usize,
    prefix_sha256: String,
    spans: [Option<Span>; 2],
    fixed: Option<Decision>,
    global: Decision,
}

#[derive(Deserialize)]
struct Decision {
    requested: Option<Span>,
}

#[derive(Clone, Copy, Default, Serialize)]
struct Contingency {
    both_win: usize,
    positional_only: usize,
    comparator_only: usize,
    both_miss: usize,
}

impl Contingency {
    fn add(&mut self, a: bool, b: bool) {
        match (a, b) {
            (true, true) => self.both_win += 1,
            (true, false) => self.positional_only += 1,
            (false, true) => self.comparator_only += 1,
            (false, false) => self.both_miss += 1,
        }
    }
    fn delta(self) -> i32 {
        self.positional_only as i32 - self.comparator_only as i32
    }
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<(T, String)> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok((
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))?,
        format!("{:x}", Sha256::digest(&bytes)),
    ))
}

fn write(path: &Path, report: &Value) -> Result<()> {
    let mut file = BufWriter::new(File::options().create_new(true).write(true).open(path)?);
    serde_json::to_writer(&mut file, report)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

fn grades(
    path: &Path,
    labels: &[Label],
    expected_sha: &str,
    subset: bool,
) -> Result<(Vec<Row>, String)> {
    let (file, sha): (GradeFile, _) = read(path)?;
    if file.schema != "uor-r4.ordered-role.grades/1" || file.grades != expected_sha {
        return Err(format!("{} grade schema or custody SHA differs", path.display()).into());
    }
    let indices = labels
        .iter()
        .enumerate()
        .filter_map(|(i, l)| (!subset || l.stratum == "supported").then_some(i));
    let pairs: Vec<_> = indices.collect();
    if file.rows.len() != pairs.len() {
        return Err(format!("{} grade row count differs", path.display()).into());
    }
    for (row, i) in file.rows.iter().zip(pairs) {
        if row.index != i {
            return Err(format!("{} reorders or duplicates row {i}", path.display()).into());
        }
        row.validate(&labels[i])?;
    }
    Ok((file.rows, sha))
}

fn cohort(labels: &[Label], matches: &[Match]) -> Result<()> {
    let mut pairs = BTreeSet::new();
    let mut rows = BTreeSet::new();
    let mut families: BTreeMap<&str, [usize; 2]> = BTreeMap::new();
    for m in matches {
        if !pairs.insert(&m.pair) || !rows.insert(m.row_left) || !rows.insert(m.row_right) {
            return Err("duplicate sealed pair or supported row".into());
        }
        let a = labels.get(m.row_left).ok_or("left row outside labels")?;
        let b = labels.get(m.row_right).ok_or("right row outside labels")?;
        for (direction, row) in [a, b].into_iter().enumerate() {
            if row.family != m.family
                || row.pair != m.pair
                || row.direction != direction
                || row.stratum != "supported"
                || row.requested.is_none()
                || row.expected.as_ref().and_then(|ids| ids.last()) != Some(&1)
            {
                return Err("sealed pair and supported grade disagree".into());
            }
        }
        if a.requested_name == b.requested_name || m.eligible != m.reasons.is_empty() {
            return Err("invalid sealed reversal or eligibility receipts".into());
        }
        if m.eligible
            && (!m.left.contains(&a.requested)
                || !m.right.contains(&b.requested)
                || m.left.contains(&None)
                || m.right.contains(&None))
        {
            return Err("eligible mapped pair does not contain both reversal endpoints".into());
        }
        let counts = families.entry(&m.family).or_default();
        counts[0] += 1;
        counts[1] += usize::from(m.eligible);
    }
    if pairs.len() != 128
        || !families
            .keys()
            .copied()
            .eq(["held-0", "held-1", "held-2", "held-3"])
        || families
            .values()
            .any(|[all, matched]| *all != 32 || *matched < 16)
        || labels.iter().filter(|r| r.stratum == "supported").count() != rows.len()
    {
        return Err("sealed four-family cohort or matched denominator invalid".into());
    }
    Ok(())
}

fn fixed(
    path: &Path,
    mode: &str,
    labels: &[Label],
    matches: &[Match],
    blind_sha: &str,
    mapping_sha: &str,
    bounded: &[Row],
) -> Result<(Vec<[[bool; 2]; 2]>, String)> {
    let (file, sha): (FixedFile, _) = read(path)?;
    if file.schema != "uor-r4.ordered-role-fixed/1"
        || file.mode != mode
        || file.bare_sha256 != blind_sha
        || file.mapping_sha256 != mapping_sha
        || file.pairs.len() != matches.len()
    {
        return Err(format!("{} fixed cohort binding differs", path.display()).into());
    }
    let mut marks = Vec::new();
    for (mapping, ranked) in matches.iter().zip(file.pairs) {
        if mapping.pair != ranked.pair
            || mapping.family != ranked.family
            || mapping.eligible != ranked.matched
            || mapping.reasons != ranked.reasons
        {
            return Err("fixed ranks reorder or alter the sealed cohort".into());
        }
        let mut choices = [[false; 2]; 2];
        for (direction, (row, spans, recorded)) in [
            (mapping.row_left, mapping.left, ranked.left),
            (mapping.row_right, mapping.right, ranked.right),
        ]
        .into_iter()
        .enumerate()
        {
            let label = &labels[row];
            if row != recorded.row
                || spans != recorded.spans
                || recorded.prefix_sha256 != label.prefix_sha256
                || recorded.global.requested != bounded[row].requested
                || (mapping.eligible && recorded.fixed.is_none())
            {
                return Err(
                    "fixed direction is unbound or global role differs from graded bounded role"
                        .into(),
                );
            }
            let selected = recorded.fixed.and_then(|decision| decision.requested);
            if selected.is_some() && !spans.contains(&selected) {
                return Err("fixed rank nominates an outside candidate".into());
            }
            choices[0][direction] = selected == label.requested;
            choices[1][direction] = recorded.global.requested == label.requested;
        }
        if mapping.eligible && mode == "Identity" && choices[0] == [true; 2] {
            return Err(format!("matched identity switched winner on {}", mapping.pair).into());
        }
        marks.push(choices);
    }
    Ok((marks, sha))
}

#[derive(Default, Serialize)]
struct ByPair {
    literal_pair: Contingency,
    literal_direction: [Contingency; 2],
    requested_pair: Contingency,
    requested_direction: [Contingency; 2],
}

fn cohort_samples<const N: usize>(
    matches: &[Match],
    eligible_only: bool,
    delta: impl Fn(usize, &Match) -> [i32; N],
) -> Vec<Vec<[i32; N]>> {
    let mut strata = BTreeMap::new();
    for (i, mapping) in matches.iter().enumerate() {
        if !eligible_only || mapping.eligible {
            strata
                .entry(&mapping.family)
                .or_insert_with(Vec::new)
                .push((&mapping.pair, delta(i, mapping)));
        }
    }
    strata
        .into_values()
        .map(|mut strata| {
            strata.sort_unstable_by_key(|row| row.0);
            strata.into_iter().map(|(_, values)| values).collect()
        })
        .collect()
}

fn both(rows: &[Row], m: &Match, predicate: impl Fn(&Row) -> bool) -> bool {
    predicate(&rows[m.row_left]) && predicate(&rows[m.row_right])
}

fn compare(matches: &[Match], a: &[Row], b: &[Row], interval: Interval, endpoints: bool) -> Value {
    let mut total = ByPair::default();
    let mut families: BTreeMap<_, ByPair> = BTreeMap::new();
    let mut transitions = Vec::new();
    let mut new_pairs = 0;
    let mut new_endpoint_pairs = 0;
    let mut new_literal_wrong_source = Vec::new();
    let mut new_literal_free_generation = Vec::new();
    for m in matches {
        let family = families.entry(&m.family).or_default();
        let a_pair = both(a, m, Row::exact_reply);
        let b_pair = both(b, m, Row::exact_reply);
        let a_roles = both(a, m, |r| r.endpoint_exact);
        let b_roles = both(b, m, |r| r.endpoint_exact);
        if a_pair && !b_pair {
            new_pairs += 1;
            new_endpoint_pairs += usize::from(a_roles);
        }
        for summary in [&mut total, family] {
            summary.literal_pair.add(a_pair, b_pair);
            summary.requested_pair.add(a_roles, b_roles);
            for (direction, row) in [m.row_left, m.row_right].into_iter().enumerate() {
                summary.literal_direction[direction]
                    .add(a[row].exact_reply(), b[row].exact_reply());
                summary.requested_direction[direction]
                    .add(a[row].endpoint_exact, b[row].endpoint_exact);
            }
        }
        for row in [m.row_left, m.row_right] {
            let (a, b) = (&a[row], &b[row]);
            if a.exact_reply() && !b.exact_reply() {
                if a.source.is_some() && !a.source_matches_requested_value {
                    new_literal_wrong_source.push(&a.id);
                }
                if a.source.is_none() {
                    new_literal_free_generation.push(&a.id);
                }
            }
            transitions.push(json!({
                "id":a.id, "pair":m.pair, "family":m.family, "direction":a.direction,
                "comparator":b.observation(), "positional":a.observation(),
            }));
        }
    }
    let added_unsupported: Vec<_> = a
        .iter()
        .zip(b)
        .filter(|(a, b)| a.unsupported_copy() && !b.unsupported_copy())
        .map(|(a, _)| &a.id)
        .collect();
    let point = total.literal_pair.delta();
    let endpoints_ok = !endpoints || (new_pairs > 0 && 4 * new_endpoint_pairs >= 3 * new_pairs);
    let safety_ok = added_unsupported.is_empty();
    json!({
        "pass":point>=8 && interval.lower_pair_count>0 && endpoints_ok && safety_ok,
        "exact_pair_delta":point,"denominator":matches.len(),
        "confidence":interval, "total":total, "families":families,
        "newly_exact_pairs":new_pairs, "newly_exact_pairs_both_endpoints_correct":new_endpoint_pairs,
        "endpoint_gate_applies":endpoints, "endpoint_gate_pass":endpoints_ok,
        "newly_exact_rows_wrong_source":new_literal_wrong_source,
        "newly_exact_rows_free_generation":new_literal_free_generation,
        "newly_unsupported_copy_rows":added_unsupported, "no_new_unsupported_copy_pass":safety_ok,
        "first_wrong_transitions":transitions,
    })
}

fn matched_readout(matches: &[Match], a: &[[[bool; 2]; 2]], b: &[[[bool; 2]; 2]]) -> Value {
    let samples = cohort_samples(matches, true, |i, _| {
        [i32::from(a[i][0] == [true; 2]) - i32::from(b[i][0] == [true; 2])]
    });
    let [interval] = confidence(&samples);
    let mut total = [Contingency::default(); 3];
    let mut global = [Contingency::default(); 3];
    let mut families: BTreeMap<_, ([Contingency; 3], [Contingency; 3])> = BTreeMap::new();
    let mut count = 0;
    let mut detail = Vec::new();
    for (i, m) in matches.iter().enumerate().filter(|(_, m)| m.eligible) {
        count += 1;
        let (family_fixed, family_global) = families.entry(&m.family).or_default();
        for (mode, targets) in [[&mut total, family_fixed], [&mut global, family_global]]
            .into_iter()
            .enumerate()
        {
            for counts in targets {
                counts[0].add(a[i][mode] == [true; 2], b[i][mode] == [true; 2]);
                for direction in 0..2 {
                    counts[direction + 1].add(a[i][mode][direction], b[i][mode][direction]);
                }
            }
        }
        detail.push(
            json!({"pair":m.pair,"positional_fixed":a[i][0],"identity_fixed":b[i][0],
            "positional_global":a[i][1],"identity_global":b[i][1]}),
        );
    }
    let delta = total[0].delta();
    json!({
        "pass":delta>=4 && interval.lower_pair_count>0, "denominator":count,
        "exact_role_pair_delta":delta,"confidence":interval, "fixed_pair_then_directions":total,
        "global_pair_then_directions":global,"families_fixed_then_global":families,"pairs":detail,
    })
}

fn controls(a: &[Row], comparators: [&[Row]; 2], labels: &[Label]) -> Result<Value> {
    let by_id: BTreeMap<_, _> = a.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut strata = BTreeMap::new();
    let mut counterparts = Vec::new();
    let mut passed = true;
    for stratum in [PROSE, INC, "unmentioned", "conflict", "surface-reorder"] {
        let rows: Vec<_> = a.iter().filter(|r| r.stratum == stratum).collect();
        if rows.len() != 32 {
            return Err(format!("registered panel stratum {stratum} does not have 32 rows").into());
        }
        let no_request = [PROSE, INC].contains(&stratum);
        let false_roles = rows.iter().filter(|r| r.requested.is_some()).count();
        let references = comparators.map(|b| {
            rows.iter()
                .filter(|r| b[r.index].requested.is_some())
                .count()
        });
        let mut local = !no_request || references.iter().all(|&n| false_roles <= n);
        if stratum == INC {
            let mut pairs = BTreeSet::new();
            for row in &rows {
                let requested = row
                    .counterpart
                    .as_deref()
                    .and_then(|id| by_id.get(id))
                    .ok_or("missing incidental counterpart")?;
                if requested.stratum != "supported"
                    || !pairs.insert(&requested.pair)
                    || labels[row.index].requested.is_some()
                    || requested.gold_requested.is_none()
                {
                    return Err("invalid incidental contrast or duplicate counterpart pair".into());
                }
                counterparts.push(json!({"incidental":row.id,"counterpart":requested.id,
                    "incidental_false_request":row.requested.is_some(),"request_endpoint_exact":requested.endpoint_exact}));
            }
            let exact = rows
                .iter()
                .filter(|r| {
                    by_id
                        .get(r.counterpart.as_deref().unwrap_or_default())
                        .is_some_and(|r| r.endpoint_exact)
                })
                .count();
            local &= false_roles <= 8 && exact >= 24;
        }
        passed &= local;
        strata.insert(stratum, json!({
            "rows":rows.len(),"nominations":false_roles,"identity_frozen_nominations":references,
            "gate_pass":local,"role_endpoint_exact":rows.iter().filter(|r| r.endpoint_exact).count(),
            "terminal_eos":rows.iter().filter(|r| r.terminal_eos).count(),
            "copy_rows":rows.iter().filter_map(|r| r.source.map(|s| (r.id.as_str(),s))).collect::<Vec<_>>(),
            "reply_exact":rows.iter().filter(|r| r.exact_reply()).count(),
            "responses":rows.iter().map(|row| json!({"id":row.id,"observed_text":row.text,
                "authored_acceptability":row.acceptable_prose,"generated":row.generated,
                "terminal_eos":row.terminal_eos,"capped":!row.terminal_eos&&row.generated.len()==16})).collect::<Vec<_>>(),
        }));
    }
    Ok(json!({
        "pass":passed, "strata":strata,
        "incidental_counterpart_exact":counterparts.iter().filter(|r| r["request_endpoint_exact"]==true).count(),
        "incidental_counterparts":counterparts,
        "prose_relevance":"authored criteria and exact observed responses retained for independent grading",
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [seed, config, out] = args.as_slice() else {
        return Err("usage: ordered-role-comparison SEED ARMS_JSON OUT".into());
    };
    if !["1", "2"].contains(&seed.as_str()) {
        return Err("seed must be 1 or 2".into());
    }
    let seed = seed.parse::<u8>()?;
    let config = Path::new(config);
    let root = config.parent().ok_or("missing config directory")?;
    let out = Path::new(out);
    let (paths, config_sha): (Paths, _) = read(config)?;
    let (labels, label_sha): (Vec<Label>, _) = read(&root.join(paths.labels))?;
    let (matching, mapping_sha): (Vec<Match>, _) = read(&root.join(paths.matching))?;
    let (_, blind_sha): (Vec<Value>, _) = read(&root.join(paths.blind))?;
    if labels.len() != 416
        || labels.iter().map(|r| &r.id).collect::<BTreeSet<_>>().len() != labels.len()
    {
        return Err("held custody row census changes or duplicates labels".into());
    }
    cohort(&labels, &matching)?;

    // Gate before opening any selector result files. Preserve every oracle miss.
    let (oracle, oracle_sha) = grades(&root.join(paths.correct_oracle), &labels, &label_sha, true)?;
    if oracle.iter().any(|r| !r.endpoint_exact) {
        return Err("correct-role oracle grade does not force exact gold endpoints".into());
    }
    let oracle_exact = oracle.iter().filter(|r| r.exact_reply()).count();
    let oracle_report = json!({
        "sha256":oracle_sha, "exact_through_eos":oracle_exact, "denominator":oracle.len(),
        "passes_248_floor":oracle_exact>=248,
        "misses":oracle.iter().filter(|r| !r.exact_reply()).map(|r| json!({"id":r.id,
            "observation":r.observation(),"generated":r.generated,"expected":r.expected})).collect::<Vec<_>>(),
    });
    let base = json!({
        "schema":"uor-r4.ordered-role.comparison/1", "seed":seed, "config_sha256":config_sha,
        "blind_sha256":blind_sha, "held_grades_sha256":label_sha, "matching_sha256":mapping_sha,
        "correct_role_oracle":oracle_report,
    });
    if oracle_exact < 248 {
        let mut report = base;
        report["disposition"] = json!("unread_oracle_adequacy_failed");
        return write(out, &report);
    }

    let files = [
        paths.public_positional,
        paths.public_identity,
        paths.public_frozen,
        paths.bounded_positional,
        paths.bounded_identity,
        paths.bounded_frozen,
    ];
    let mut armed = Vec::new();
    let mut receipts = BTreeMap::new();
    for (name, path) in [
        "public_positional",
        "public_identity",
        "public_frozen",
        "bounded_positional",
        "bounded_identity",
        "bounded_frozen",
    ]
    .into_iter()
    .zip(files)
    {
        let (rows, sha) = grades(&root.join(path), &labels, &label_sha, false)?;
        armed.push(rows);
        receipts.insert(name, sha);
    }
    let (fixed_pos, fixed_pos_sha) = fixed(
        &root.join(paths.fixed_positional),
        "Positional",
        &labels,
        &matching,
        &blind_sha,
        &mapping_sha,
        &armed[3],
    )?;
    let (fixed_id, fixed_id_sha) = fixed(
        &root.join(paths.fixed_identity),
        "Identity",
        &labels,
        &matching,
        &blind_sha,
        &mapping_sha,
        &armed[4],
    )?;
    receipts.insert("fixed_positional", fixed_pos_sha);
    receipts.insert("fixed_identity", fixed_id_sha);

    let samples = cohort_samples(&matching, false, |_, m| {
        TESTS.map(|(_, a, b)| {
            i32::from(both(&armed[a], m, Row::exact_reply))
                - i32::from(both(&armed[b], m, Row::exact_reply))
        })
    });
    let intervals = confidence(&samples);
    let comparisons: BTreeMap<_, _> = TESTS
        .into_iter()
        .zip(intervals)
        .map(|((name, a, b), interval)| {
            (
                name,
                compare(&matching, &armed[a], &armed[b], interval, a == 0),
            )
        })
        .collect();
    let nested = matched_readout(&matching, &fixed_pos, &fixed_id);
    let public = controls(&armed[0], [&armed[1], &armed[2]], &labels)?;
    let bounded = controls(&armed[3], [&armed[4], &armed[5]], &labels)?;
    let mut diagnostic_reports = BTreeMap::new();
    for (name, path) in paths.diagnostics {
        let subset = name == "wrong";
        let (rows, sha) = grades(&root.join(path), &labels, &label_sha, subset)?;
        diagnostic_reports.insert(name, json!({
            "sha256":sha, "rows":rows.len(), "exact":rows.iter().filter(|r| r.exact_reply()).count(),
            "exact_endpoints":rows.iter().filter(|r| r.endpoint_exact).count(),
            "unsupported_copy_ids":rows.iter().filter(|r|r.unsupported_copy()).map(|r| &r.id).collect::<Vec<_>>(),
            "outcomes":rows.iter().map(|r| json!({"id":r.id,"observation":r.observation()})).collect::<Vec<_>>(),
        }));
    }

    let mut report = base;
    report["disposition"] = json!(if comparisons.values().all(|r| r["pass"] == true)
        && nested["pass"] == true
        && public["pass"] == true
        && bounded["pass"] == true
    {
        "registered_numeric_gates_pass"
    } else {
        "registered_numeric_gates_failed"
    });
    report["scope"] = json!("graded row statistics; upstream execution and qualitative prose grades require separate receipts");
    report["bootstrap"] = json!({"seed":bootstrap::SEED,"replicates":bootstrap::REPLICATES,
        "ranks":[bootstrap::LOW_RANK,bootstrap::HIGH_RANK],
        "order":"family then pair lexicographic; one set of full-panel draws shared by all four contrasts; separate reset stream for sealed matched cohort"});
    report["graded_artifacts_sha256"] = json!(receipts);
    report["comparisons"] = json!(comparisons);
    report["fixed_competitor"] = nested;
    report["public_controls"] = public;
    report["bounded_controls"] = bounded;
    report["diagnostics"] = json!(diagnostic_reports);
    report["public_to_bounded_positional"] = json!(armed[0]
        .iter()
        .zip(&armed[3])
        .filter_map(|(a, b)| (a.requested != b.requested
            || a.generated != b.generated
            || a.source != b.source)
            .then(|| json!({"id":a.id,"public":a.observation(),"bounded":b.observation()})))
        .collect::<Vec<_>>());
    write(out, &report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(index: usize, exact: bool, endpoint: bool) -> Row {
        let expected = vec![11, 1];
        let generated = vec![if exact { 11 } else { 12 }, 1];
        Row {
            index,
            id: format!("row-{index}"),
            family: "held-0".into(),
            pair: format!("pair-{}", index / 2),
            direction: index % 2,
            stratum: "supported".into(),
            counterpart: None,
            requested: endpoint.then_some([9, 10]),
            gold_requested: Some([9, 10]),
            endpoint_exact: endpoint,
            source: Some(3),
            source_matches_requested_value: true,
            expected: Some(expected),
            generated,
            terminal_eos: true,
            exact: Some(exact),
            first_wrong: (!exact).then_some(0),
            expected_wrong_id: (!exact).then_some(11),
            actual_wrong_id: (!exact).then_some(12),
            first_free: Value::Null,
            text: Value::Null,
            acceptable_prose: Value::Null,
        }
    }

    #[test]
    fn endpoint_consistency_and_unsupported_on_a_passing_gain() {
        let pairs: Vec<_> = (0..8)
            .map(|i| Match {
                pair: format!("pair-{i}"),
                family: "held-0".into(),
                row_left: i * 2,
                row_right: i * 2 + 1,
                left: [Some([9, 10]), Some([15, 16])],
                right: [Some([15, 16]), Some([9, 10])],
                eligible: true,
                reasons: vec![],
            })
            .collect();
        let mut positional: Vec<_> = (0..16).map(|i| row(i, true, i < 12)).collect();
        let baseline: Vec<_> = (0..16).map(|i| row(i, false, false)).collect();
        let bound = Interval {
            lower_pair_count: 1,
            upper_pair_count: 8,
        };
        let report = compare(&pairs, &positional, &baseline, bound, true);
        assert_eq!(report["pass"], true);
        assert_eq!(report["newly_exact_pairs_both_endpoints_correct"], 6);
        assert_eq!(
            report["first_wrong_transitions"][0]["comparator"]["actual_wrong_id"],
            12
        );
        positional[10].endpoint_exact = false;
        let report = compare(&pairs, &positional, &baseline, bound, true);
        assert_eq!(report["endpoint_gate_pass"], false);
        assert_eq!(report["pass"], false);
        assert_eq!(
            compare(&pairs, &positional, &baseline, bound, false)["pass"],
            true
        );
        positional[10].source_matches_requested_value = false;
        let report = compare(&pairs, &positional, &baseline, bound, false);
        assert_eq!(report["newly_unsupported_copy_rows"], json!(["row-10"]));
        assert_eq!(report["newly_exact_rows_wrong_source"], json!(["row-10"]));
        assert_eq!(report["pass"], false);
    }

    #[test]
    fn asymmetric_directions_cannot_make_an_exact_pair() {
        let pairs = [Match {
            pair: "pair-0".into(),
            family: "held-0".into(),
            row_left: 0,
            row_right: 1,
            left: [None; 2],
            right: [None; 2],
            eligible: false,
            reasons: vec!["example".into()],
        }];
        let positional = [row(0, true, true), row(1, false, false)];
        let identity = [row(0, false, false), row(1, true, true)];
        let report = compare(
            &pairs,
            &positional,
            &identity,
            Interval {
                lower_pair_count: 0,
                upper_pair_count: 0,
            },
            true,
        );
        assert_eq!(report["exact_pair_delta"], 0);
        assert_eq!(report["total"]["literal_pair"]["both_miss"], 1);
        assert_eq!(
            report["total"]["literal_direction"][0]["positional_only"],
            1
        );
        assert_eq!(
            report["total"]["literal_direction"][1]["comparator_only"],
            1
        );
    }
}
