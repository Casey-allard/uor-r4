use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fmt::Write as _,
    fs,
    path::Path,
};

mod fit;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ARMS: [&str; 3] = ["baseline", "calibrated", "shuffled"];
const NATIVE: [&str; 3] = ["ordinary", "person", "assertions"];
const NIL: u32 = u32::MAX;
const STOPS: [u32; 4] = [16, 1, 0, 201];
type Span = [u32; 2];

fn require(condition: bool, context: impl Into<String>) -> Result<()> {
    if !condition {
        return Err(context.into().into());
    }
    Ok(())
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn words(bytes: &[u8]) -> Result<Vec<u32>> {
    require(bytes.len().is_multiple_of(4), "unaligned integer tape")?;
    Ok(bytes
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
        .collect())
}

#[derive(Deserialize)]
struct Blind {
    row: u32,
    ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Fact {
    person: Span,
    value: Span,
}

#[derive(Deserialize)]
struct Local {
    start: u32,
    ids: Vec<u32>,
    gold_offset: u32,
}

#[derive(Deserialize)]
struct RoleTargets {
    excluded: Option<Span>,
    unrequested: Option<Span>,
}

#[derive(Deserialize)]
struct Key {
    row: u32,
    stratum: String,
    family: String,
    connector_arm: Option<u8>,
    object_arm: Option<u8>,
    role_factors: Option<[u8; 3]>,
    role_targets: RoleTargets,
    matched_pair: Option<String>,
    context_arm: Option<u8>,
    local_pair_tape: Option<Local>,
    expected_value: Option<String>,
    expected_tape: Option<Vec<u32>>,
    prefix_sha256: String,
    assertions: Vec<Fact>,
    selected: Vec<usize>,
    selected_value_spans: Vec<Span>,
    requested: Span,
}

struct Prompt {
    key: Key,
    ids: Vec<u32>,
}

impl Prompt {
    fn heads(&self) -> Vec<Span> {
        self.key
            .selected
            .iter()
            .map(|&i| self.key.assertions[i].person)
            .collect()
    }

    fn gold_at(&self, head: Span, source: u32) -> bool {
        self.key.selected.iter().any(|&index| {
            let fact = &self.key.assertions[index];
            fact.person == head && fact.value[0] == source
        })
    }

    fn copy_from(&self, source: u32) -> Result<Vec<u32>> {
        let tail = self
            .ids
            .get(source as usize..)
            .ok_or("source out of bounds")?;
        let fence = tail
            .iter()
            .position(|id| STOPS.contains(id))
            .ok_or("source missing stop")?;
        require(
            (1..8).contains(&fence),
            "source outside original writer cap",
        )?;
        Ok([&tail[..fence], &[1]].concat())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct First {
    offset: usize,
    expected: Option<u32>,
    actual: Option<u32>,
}

fn first_difference(expected: &[u32], actual: &[u32]) -> Option<First> {
    (0..expected.len().max(actual.len()))
        .find(|&i| expected.get(i) != actual.get(i))
        .map(|offset| First {
            offset,
            expected: expected.get(offset).copied(),
            actual: actual.get(offset).copied(),
        })
}

#[derive(Deserialize)]
struct Decision {
    role_words: Option<[u32; 8]>,
}

#[derive(Deserialize)]
struct Stage {
    visit: usize,
    head: Span,
    actual_query: Span,
    onset: Onset,
    candidate_words: Vec<[u32; 10]>,
}

#[derive(Deserialize)]
struct Onset {
    status: u32,
    source: Option<u32>,
}

#[derive(Deserialize)]
struct Grade {
    row: u32,
    stratum: String,
    lane: String,
    person: Option<Span>,
    person_correct: bool,
    source: Option<u32>,
    source_correct: bool,
    value_spans: Vec<Span>,
    expected: Option<Vec<u32>>,
    exact: Option<bool>,
    tape: Vec<u32>,
    first: Option<First>,
    decision: Option<Decision>,
    stages: Vec<Stage>,
}

impl Grade {
    fn matched(&self) -> bool {
        self.exact == Some(true)
    }

    fn excluded(&self) -> Option<Span> {
        let [_, _, x, y, ..] = self.decision.as_ref()?.role_words?;
        (x != NIL).then_some([x, y])
    }

    fn eos(&self) -> bool {
        self.tape.last() == Some(&1)
    }

    fn exact_heads(&self, prompt: &Prompt) -> bool {
        self.stages
            .iter()
            .map(|stage| stage.head)
            .eq(prompt.heads())
    }

    fn first_gate(&self, prompt: &Prompt) -> &'static str {
        if prompt.key.expected_tape.is_none() {
            return if self.source.is_some() {
                "unsupported_write"
            } else {
                "unsupported_withheld"
            };
        }
        if self.matched() {
            "exact"
        } else if !self.person_correct {
            "requested_entity"
        } else if !self.exact_heads(prompt) {
            "assertion_heads"
        } else if !self.source_correct {
            "source_admission"
        } else {
            "exact_writer"
        }
    }

    fn late_exact(&self, prompt: &Prompt) -> bool {
        self.matched()
            && self.source.is_some_and(|source| {
                self.stages.iter().any(|stage| {
                    stage.onset.source == Some(source)
                        && source.saturating_sub(stage.head[1]) > 3
                        && prompt.gold_at(stage.head, source)
                })
            })
    }
}

struct Arm {
    grades: BTreeMap<(u32, String), Grade>,
    packet: Vec<u32>,
    summary_hash: String,
    packet_hash: String,
}

impl Arm {
    fn grade(&self, prompt: &Prompt, lane: &str) -> &Grade {
        &self.grades[&(prompt.key.row, lane.into())]
    }

    fn slot(&self, stage: &Stage, index: usize) -> [u32; 10] {
        let offset = 4 + 1926 * stage.visit + 6 + 10 * index;
        self.packet[offset..offset + 10].try_into().unwrap()
    }
}

fn panel(path: &Path) -> Result<Vec<Prompt>> {
    let blind = jsonl::<Blind>(&path.join("held.blind.jsonl"))?;
    let keys = jsonl::<Key>(&path.join("held.gold.jsonl"))?;
    require(
        blind.len() == 49 && keys.len() == 49,
        "expected 49 registered prompts",
    )?;
    let mut seen = BTreeSet::new();
    blind
        .into_iter()
        .zip(keys)
        .map(|(blind, key)| {
            require(
                blind.row == key.row && seen.insert(key.row),
                "duplicate or crossed prompt row",
            )?;
            let ids = blind.ids;
            require(
                ids.first() == Some(&0) && ids.iter().all(|&id| id < 4096),
                "invalid prompt ID tape",
            )?;
            let bytes: Vec<_> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
            require(sha(&bytes) == key.prefix_sha256, "prompt tape hash differs")?;
            require(
                key.requested[0] < key.requested[1] && key.requested[1] as usize <= ids.len(),
                "bad requested span",
            )?;
            require(
                key.selected.iter().all(|&i| i < key.assertions.len()),
                "bad selected assertion index",
            )?;
            let selected: Vec<_> = key
                .selected
                .iter()
                .map(|&i| key.assertions[i].value)
                .collect();
            require(
                selected == key.selected_value_spans,
                "bad selected value span seal",
            )?;
            let p = Prompt { key, ids };
            for &i in &p.key.selected {
                let fact = &p.key.assertions[i];
                let [start, end] = fact.person.map(|a| a as usize);
                let [q0, q1] = p.key.requested.map(|a| a as usize);
                require(
                    start < end
                        && end <= fact.value[0] as usize
                        && fact.value[0] < fact.value[1]
                        && (fact.value[1] as usize) < q0
                        && p.ids[start..end] == p.ids[q0..q1]
                        && STOPS.contains(&p.ids[fact.value[1] as usize]),
                    format!("noncausal or crossed expected head {}", p.key.row),
                )?;
                if let Some(target) = &p.key.expected_tape {
                    require(
                        p.copy_from(fact.value[0])? == *target,
                        "expected tape differs from original copy",
                    )?;
                }
            }
            Ok(p)
        })
        .collect()
}

fn read_arm(path: &Path, prompts: &[Prompt]) -> Result<Arm> {
    let index: BTreeMap<_, _> = prompts.iter().map(|p| (p.key.row, p)).collect();
    let raw = fs::read(path.join("rows.json"))?;
    let packet_bytes = fs::read(path.join("input.bin"))?;
    let packet = words(&packet_bytes)?;
    require(
        packet.len() >= 4
            && packet[0] == 0x49383244
            && packet[2] == 1926
            && packet[3] == 0
            && packet.len() == 4 + packet[1] as usize * 1926,
        "invalid native candidate packet",
    )?;
    let mut grades = BTreeMap::new();
    let mut visits = BTreeSet::new();
    for grade in serde_json::from_slice::<Vec<Grade>>(&raw)? {
        let p = index.get(&grade.row).ok_or("grade outside roster")?;
        require(
            grade.stratum == p.key.stratum
                && grade.expected == p.key.expected_tape
                && grade.person_correct == (grade.person == Some(p.key.requested))
                && grade.value_spans == p.key.selected_value_spans,
            format!("crossed grade row {}", grade.row),
        )?;
        require(
            grade.tape.len() <= 16
                && !grade.tape.is_empty()
                && grade.tape.iter().all(|&id| id < 4096)
                && !grade.tape[..grade.tape.len() - 1].contains(&1),
            format!("invalid emitted tape {}", grade.row),
        )?;
        require(
            grade.exact == grade.expected.as_ref().map(|t| *t == grade.tape)
                && grade.first
                    == grade
                        .expected
                        .as_ref()
                        .and_then(|t| first_difference(t, &grade.tape))
                && grade.source_correct
                    == (p.key.expected_tape.is_some()
                        && grade
                            .source
                            .is_some_and(|s| p.key.selected_value_spans.iter().any(|v| v[0] == s))),
            format!("inconsistent grade on {}", grade.row),
        )?;
        if let Some(source) = grade.source {
            require(
                p.copy_from(source)? == grade.tape,
                format!(
                    "released source does not emit its exact original tape on {}",
                    grade.row
                ),
            )?;
        }
        for stage in &grade.stages {
            require(
                stage.visit < packet[1] as usize
                    && visits.insert(stage.visit)
                    && grade.person == Some(stage.actual_query),
                "lost or reused candidate visit",
            )?;
            let offset = 4 + 1926 * stage.visit;
            let header = &packet[offset..offset + 6];
            require(
                header[0] as usize == p.ids.len()
                    && header[1] + 1 == stage.head[1]
                    && header[2] == stage.actual_query[0]
                    && header[5] as usize == stage.candidate_words.len(),
                "stage not joined to original packet",
            )?;
            require(
                (stage.onset.status == 0) == stage.onset.source.is_some(),
                "native source/status disagree",
            )?;
            for (i, candidate) in stage.candidate_words.iter().enumerate() {
                let original = &packet[offset + 6 + 10 * i..offset + 16 + 10 * i];
                require(
                    candidate[..4] == original[..4]
                        && candidate[0] < candidate[1]
                        && candidate[1] <= header[3],
                    "candidate moved in trace",
                )?;
                if candidate[4] == 4 {
                    require(
                        candidate[8] != 65535
                            && candidate[6] <= candidate[8]
                            && candidate[6] < candidate[5],
                        "invalid signed clearance",
                    )?;
                }
            }
        }
        let id = (grade.row, grade.lane.clone());
        require(grades.insert(id, grade).is_none(), "duplicate grade")?;
    }
    require(visits.len() == packet[1] as usize, "missing native visit")?;
    let arm = Arm {
        grades,
        packet,
        summary_hash: sha(&raw),
        packet_hash: sha(&packet_bytes),
    };
    for p in prompts {
        for lane in NATIVE.into_iter().chain(["disabled"]) {
            require(
                arm.grades.contains_key(&(p.key.row, lane.into())),
                "missing nominal or control lane",
            )?;
        }
        require(
            arm.grade(p, "disabled").source.is_none(),
            "disabled writer released a source",
        )?;
        if p.key.expected_tape.is_some() {
            require(
                arm.grades
                    .get(&(p.key.row, "gold_value".into()))
                    .is_some_and(Grade::matched),
                format!("missing or failed forced gold on {}", p.key.row),
            )?;
        }
        if let Some(opposing) = arm.grades.get(&(p.key.row, "opposing".into())) {
            require(
                opposing.source.is_some() && opposing.exact == Some(false),
                "opposing source control failed",
            )?;
        }
    }
    Ok(arm)
}

#[derive(Default, Serialize)]
struct Metrics {
    rows: usize,
    supported: usize,
    role: usize,
    heads: usize,
    exact: usize,
    late_exact: usize,
    source: usize,
    wrong_source: usize,
    unsupported: usize,
    eos: usize,
    first_actual: BTreeMap<String, usize>,
    gates: BTreeMap<&'static str, usize>,
}

impl Metrics {
    fn add(&mut self, p: &Prompt, g: &Grade) {
        self.rows += 1;
        self.supported += usize::from(g.expected.is_some());
        self.role += usize::from(g.person_correct);
        self.heads += usize::from(g.exact_heads(p));
        self.exact += usize::from(g.matched());
        self.late_exact += usize::from(g.late_exact(p));
        self.source += usize::from(g.source_correct);
        self.wrong_source +=
            usize::from(g.expected.is_some() && g.source.is_some() && !g.source_correct);
        self.unsupported += usize::from(g.expected.is_none() && g.source.is_some());
        self.eos += usize::from(g.eos());
        if let Some(first) = &g.first {
            *self
                .first_actual
                .entry(
                    first
                        .actual
                        .map_or_else(|| "end".into(), |id| id.to_string()),
                )
                .or_default() += 1;
        }
        *self.gates.entry(g.first_gate(p)).or_default() += 1;
    }
}

#[derive(Default, Serialize)]
struct Paired {
    recovered: Vec<u32>,
    lost: Vec<u32>,
    exact_both: Vec<u32>,
    missed_both: Vec<u32>,
    new_wrong_source: Vec<u32>,
    changed_to_wrong_source: Vec<u32>,
    new_unsupported: Vec<u32>,
}
impl Paired {
    fn add(&mut self, a: &Grade, b: &Grade) {
        if a.expected.is_some() {
            match (a.matched(), b.matched()) {
                (false, true) => &mut self.recovered,
                (true, false) => &mut self.lost,
                (true, true) => &mut self.exact_both,
                (false, false) => &mut self.missed_both,
            }
            .push(a.row);
            let wrong = |g: &Grade| g.source.is_some() && !g.source_correct;
            if !wrong(a) && wrong(b) {
                self.new_wrong_source.push(a.row);
            }
            if wrong(b) && a.source != b.source {
                self.changed_to_wrong_source.push(a.row);
            }
        } else if a.source.is_none() && b.source.is_some() {
            self.new_unsupported.push(a.row);
        }
    }
}

fn signature(slot: [u32; 10]) -> fit::Signature {
    (slot[2].min(3) | ((slot[1] - slot[0]) << 8), slot[4])
}

fn same_local(stage: &Stage, anchor: u32) -> Vec<[u32; 10]> {
    stage
        .candidate_words
        .iter()
        .map(|candidate| {
            let mut shifted = *candidate;
            shifted[0] -= anchor;
            shifted[1] -= anchor;
            shifted
        })
        .collect()
}

fn boundary(p: &[Prompt], arms: &[Arm; 3]) -> Result<Vec<Value>> {
    let mut pairs: BTreeMap<&str, Vec<&Prompt>> = BTreeMap::new();
    for row in p {
        if let Some(pair) = &row.key.matched_pair {
            pairs.entry(pair).or_default().push(row);
        }
    }
    require(pairs.len() == 2, "wrong context-pair count")?;
    let mut output = Vec::new();
    for (pair, endpoints) in pairs {
        let [a, b] = endpoints.as_slice() else {
            return Err("context-pair count is not two".into());
        };
        let x = a
            .key
            .local_pair_tape
            .as_ref()
            .ok_or("missing local witness")?;
        let y = b
            .key
            .local_pair_tape
            .as_ref()
            .ok_or("missing local witness")?;
        require(
            a.key.context_arm == Some(0)
                && b.key.context_arm == Some(1)
                && x.ids == y.ids
                && a.ids[x.start as usize..] == x.ids
                && b.ids[y.start as usize..] == y.ids
                && a.ids[..x.start as usize] != b.ids[..y.start as usize]
                && x.gold_offset != y.gold_offset
                && x.start + x.gold_offset == a.key.selected_value_spans[0][0]
                && y.start + y.gold_offset == b.key.selected_value_spans[0][0],
            "context-pair token witness differs",
        )?;
        let mut readout = BTreeMap::new();
        for (label, arm) in ARMS.into_iter().zip(arms) {
            let (a1, b1) = (arm.grade(a, "assertions"), arm.grade(b, "assertions"));
            require(
                a1.stages.len() == 1 && b1.stages.len() == 1,
                "context pair has multiple supplied heads",
            )?;
            let local_scores_equal =
                same_local(&a1.stages[0], x.start) == same_local(&b1.stages[0], y.start);
            let source = [
                a1.source.map(|s| s - x.start),
                b1.source.map(|s| s - y.start),
            ];
            let ordinary_a = arm.grade(a, "ordinary");
            let ordinary_b = arm.grade(b, "ordinary");
            readout.insert(
                label,
                json!({
                    "same_local_scores":local_scores_equal,
                    "supplied_local_sources":source,
                    "supplied_exact":[a1.matched(),b1.matched()],
                    "ordinary_sources":[ordinary_a.source,ordinary_b.source],
                    "ordinary_exact":[ordinary_a.matched(),ordinary_b.matched()],
                    "ordinary_eos":[ordinary_a.eos(),ordinary_b.eos()],
                    "ordinary_first":[ordinary_a.first,ordinary_b.first]
                }),
            );
        }
        output.push(
            json!({"pair":pair,"rows":[a.key.row,b.key.row],"local_ids":x.ids,
            "local_starts":[x.start,y.start],"gold_local_offsets":[x.gold_offset,y.gold_offset],
            "gold_values":[a.key.expected_value,b.key.expected_value],"arms":readout}),
        );
    }
    Ok(output)
}

#[derive(Default)]
struct Signed {
    clearances: Vec<Value>,
    early: Vec<Value>,
    reasons: BTreeMap<String, usize>,
}

fn sign_audit(p: &[Prompt], arms: &[Arm; 3], fit: &fit::Fit) -> Result<Signed> {
    let mut audit = Signed::default();
    require(
        arms.iter().all(|arm| arm.packet == arms[0].packet),
        "candidate/fact/role input packet changes across signed arms",
    )?;
    for prompt in p {
        for lane in NATIVE {
            let base = arms[0].grade(prompt, lane);
            for (arm_index, label) in ARMS.into_iter().enumerate().skip(1) {
                let arm = &arms[arm_index];
                let g = arm.grade(prompt, lane);
                require(
                    base.person == g.person
                        && base.excluded() == g.excluded()
                        && base.stages.len() == g.stages.len(),
                    "intervention moved the role/fact route",
                )?;
                for (orig, next) in base.stages.iter().zip(&g.stages) {
                    require(
                        orig.head == next.head
                            && orig.actual_query == next.actual_query
                            && orig.candidate_words.len() == next.candidate_words.len(),
                        "intervention moved visits",
                    )?;
                    for (i, (before, after)) in orig
                        .candidate_words
                        .iter()
                        .zip(&next.candidate_words)
                        .enumerate()
                    {
                        require(
                            before[..4] == after[..4] && before[5..8] == after[5..8],
                            "positive candidate state changed",
                        )?;
                        require(
                            if before[4] != 2 {
                                before == after
                            } else {
                                matches!(after[4], 2 | 4)
                            },
                            "already admitted or inaccessible contender changed",
                        )?;
                        if before[4] != 2 {
                            continue;
                        }
                        let sig = signature(arm.slot(next, i));
                        let support = fit.strata.get(&sig);
                        let expected = support.map_or(65535, |s| {
                            if arm_index == 1 {
                                s.calibrated
                            } else {
                                s.shuffled
                            }
                        });
                        let should_clear =
                            expected != 65535 && after[6] <= expected && after[6] < after[5];
                        require(
                            after[8] == expected && (after[4] == 4) == should_clear,
                            "clearance disagrees with fitted sign/radius",
                        )?;
                        let reason = if should_clear {
                            "cleared"
                        } else if after[5] == NIL && after[6] == NIL {
                            "both_classes_missing"
                        } else if expected == 65535 {
                            "cross_family_negative_radius_missing"
                        } else if after[6] > expected {
                            "negative_outside_radius"
                        } else {
                            "negative_not_strictly_nearer"
                        };
                        *audit
                            .reasons
                            .entry(format!("{label}/{lane}/{reason}"))
                            .or_default() += 1;
                        if !should_clear {
                            continue;
                        }
                        let is_gold = prompt.gold_at(next.head, after[0])
                            && prompt.key.expected_tape.is_some();
                        let item = json!({"row":prompt.key.row,"arm":label,"lane":lane,"fact":next.head,
                            "source":after[0],"word_end":after[1],"connector_pieces":after[2],
                            "shape":sig.0,"signature":sig.1,"positive_distance":(after[5]!=NIL).then_some(after[5]),
                            "negative_distance":after[6],"positive_radius":after[7],"negative_radius":after[8],
                            "negative_train_families":support.map(|s|&s.negative_families),
                            "gold_cleared":is_gold,"baseline_source":base.source,"actual_source":g.source,
                            "baseline_exact":base.exact,"actual_exact":g.exact});
                        if is_gold && after[2] <= 3 {
                            audit.early.push(item.clone());
                        }
                        audit.clearances.push(item);
                    }
                }
            }
        }
    }
    Ok(audit)
}

fn display(x: impl Serialize) -> String {
    serde_json::to_string(&x).unwrap()
}

fn role_matrix(prompts: &[Prompt], arms: &[Arm; 3]) -> Value {
    let mut grouped = BTreeMap::<String, [usize; 6]>::new();
    let mut distractor = BTreeMap::<String, [usize; 2]>::new();
    for p in prompts
        .iter()
        .filter(|p| p.key.stratum == "role" || p.key.stratum == "role_direct")
    {
        for (label, arm) in ARMS.into_iter().zip(arms) {
            let g = arm.grade(p, "ordinary");
            let person = arm.grade(p, "person");
            let values = [
                1,
                usize::from(g.person_correct),
                usize::from(g.excluded() == p.key.role_targets.excluded),
                usize::from(g.person_correct && g.excluded() == p.key.role_targets.excluded),
                usize::from(g.matched()),
                usize::from(person.matched()),
            ];
            let mut keys = vec![
                format!("{label}/{}", p.key.stratum),
                format!("{label}/{}/{}", p.key.stratum, p.key.family),
            ];
            if let Some([who, fact, query]) = p.key.role_factors {
                for (name, value) in [
                    ("requested_person", who),
                    ("fact_order", fact),
                    ("query_clause_order", query),
                ] {
                    keys.push(format!("{label}/{}/{name}/{value}", p.key.family));
                }
                keys.push(format!(
                    "{label}/{}/factors/{who}{fact}{query}",
                    p.key.family
                ));
            }
            if let Some(unrequested) = p.key.role_targets.unrequested {
                let selected =
                    usize::from(g.person == Some(unrequested) || g.excluded() == Some(unrequested));
                let [rows, hits] = distractor
                    .entry(format!("{label}/{}", p.key.family))
                    .or_default();
                *rows += 1;
                *hits += selected;
            }
            for key in keys {
                for (field, value) in grouped.entry(key).or_default().iter_mut().zip(values) {
                    *field += value;
                }
            }
        }
    }
    json!({"columns":["rows","requested","excluded","joint","ordinary_exact","supplied_person_exact"],"counts":grouped,
        "unrequested_in_either_role": {"columns":["rows","selected"], "counts":distractor}})
}

fn render(prompt: &[Prompt], arms: &[Arm; 3], fit: &fit::Fit, path: &Path) -> Result<()> {
    let mut totals = BTreeMap::<String, Metrics>::new();
    let mut pairs = BTreeMap::<String, Paired>::new();
    let pairings = [(0, 1), (0, 2), (2, 1)];
    let signed = sign_audit(prompt, arms, fit)?;
    let boundary = boundary(prompt, arms)?;
    let roles = role_matrix(prompt, arms);
    let mut tsv = String::from("row\tstratum\tfamily\tfactor_requested_fact_clause\tarm\tlane\trequested\texcluded\texpected_requested\texpected_excluded\tactual_heads\texpected_heads\tsource\tgold_spans\texpected_first_id\tactual_first_id\tfirst_mismatch\teos\texact\tfirst_gate\n");
    for p in prompt {
        for lane in NATIVE
            .into_iter()
            .chain(["gold_value", "opposing", "disabled"])
        {
            for (label, arm) in ARMS.into_iter().zip(arms) {
                let Some(g) = arm.grades.get(&(p.key.row, lane.into())) else {
                    continue;
                };
                totals
                    .entry(format!("{label}/{lane}/all"))
                    .or_default()
                    .add(p, g);
                totals
                    .entry(format!("{label}/{lane}/{}", p.key.stratum))
                    .or_default()
                    .add(p, g);
                if let Some(connector) = p.key.connector_arm {
                    totals
                        .entry(format!("{label}/{lane}/connector/{connector}"))
                        .or_default()
                        .add(p, g);
                }
                if let Some(object) = p.key.object_arm {
                    totals
                        .entry(format!("{label}/{lane}/object/{object}"))
                        .or_default()
                        .add(p, g);
                }
                totals
                    .entry(format!("{label}/{lane}/{}/{}", p.key.stratum, p.key.family))
                    .or_default()
                    .add(p, g);
                writeln!(
                    tsv, "{}\t{}\t{}\t{}\t{label}\t{lane}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    p.key.row, p.key.stratum, p.key.family, display(p.key.role_factors),
                    display(g.person), display(g.excluded()), display(p.key.requested), display(p.key.role_targets.excluded),
                    display(g.stages.iter().map(|s|s.head).collect::<Vec<_>>()), display(p.heads()),
                    display(g.source), display(&g.value_spans), display(g.expected.as_ref().and_then(|t|t.first())),
                    display(g.tape[0]), display(&g.first), g.eos(), display(g.exact), g.first_gate(p),
                )?;
            }
            for (a, b) in pairings {
                let [Some(x), Some(y)] =
                    [a, b].map(|i| arms[i].grades.get(&(p.key.row, lane.into())))
                else {
                    continue;
                };
                let prefix = format!("{}-to-{}/{lane}", ARMS[a], ARMS[b]);
                pairs.entry(format!("{prefix}/all")).or_default().add(x, y);
                pairs
                    .entry(format!("{prefix}/{}", p.key.stratum))
                    .or_default()
                    .add(x, y);
                if let Some(connector) = p.key.connector_arm {
                    pairs
                        .entry(format!("{prefix}/connector/{connector}"))
                        .or_default()
                        .add(x, y);
                }
                if let Some(object) = p.key.object_arm {
                    pairs
                        .entry(format!("{prefix}/object/{object}"))
                        .or_default()
                        .add(x, y);
                }
                pairs
                    .entry(format!("{prefix}/{}/{}", p.key.stratum, p.key.family))
                    .or_default()
                    .add(x, y);
            }
        }
    }
    let available = fit
        .strata
        .values()
        .filter(|s| s.calibrated != 65535)
        .count();
    let changed = fit
        .strata
        .values()
        .filter(|s| s.calibrated != s.shuffled)
        .count();
    let primary = &pairs["baseline-to-calibrated/ordinary/all"];
    let control = &pairs["shuffled-to-calibrated/ordinary/all"];
    let mut md = String::from("# [codex] D29 paired original-address readout\n\n");
    writeln!(md,
        "Calibration recovered {} ordinary exact replies and lost {} against D28 fitted-whole. It introduced {} wrong supported source selections and {} unsupported factual writes. Relative to the shuffle, it recovered {} and lost {}. The true negative table had {available} cross-family-supported signatures out of {}, and the sorted-key shuffle changed {changed}.\n",
        primary.recovered.len(), primary.lost.len(), primary.new_wrong_source.len(), primary.new_unsupported.len(),
        control.recovered.len(), control.lost.len(), fit.strata.len(),
    )?;
    if changed == 0 {
        writeln!(
            md,
            "The radius rotation made no change and is uninformative.\n"
        )?;
    }
    if !primary.changed_to_wrong_source.is_empty() && control.recovered.len() <= control.lost.len()
    {
        writeln!(md, "The registered advancement bar failed. The calibrated arm released new wrong sources and its exact-reply count did not exceed the shuffled control.\n")?;
    }
    writeln!(md,"| Carrier | Lane | Requested entity | Original assertion heads | Exact through EOS | Correct source | Wrong source | Unsupported write | EOS |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")?;
    for label in ARMS {
        for lane in NATIVE {
            let k = format!("{label}/{lane}/all");
            let m = &totals[&k];
            writeln!(
                md,
                "| {label} | {lane} | {}/{} | {}/{} | {}/{} | {}/{} | {} | {} | {}/{} |",
                m.role,
                m.rows,
                m.heads,
                m.rows,
                m.exact,
                m.supported,
                m.source,
                m.supported,
                m.wrong_source,
                m.unsupported,
                m.eos,
                m.rows
            )?;
        }
    }
    writeln!(md,"\nThe assertion-head column compares both original token endpoints and order. A wrong source means a released original address outside the supported value starts. An unsupported write means an original source was released for a conflict or absent target.\n")?;
    for label in ARMS {
        let forced = &totals[&format!("{label}/gold_value/all")];
        let competing = &totals[&format!("{label}/opposing/all")];
        let disabled = &totals[&format!("{label}/disabled/all")];
        let ordinary = &totals[&format!("{label}/ordinary/all")];
        writeln!(md,"- {label}. Forced gold matched {}/{} through EOS, forced competing matched {}/{}, and disabled matched {}/{}. The ordinary lane emitted {} exact replies from original starts beyond three connector pieces.",
            forced.exact, forced.supported, competing.exact, competing.supported, disabled.exact,
            disabled.supported, ordinary.late_exact)?;
    }
    writeln!(md, "\n## Paired changes and early values\n")?;
    for (a, b) in pairings {
        for lane in NATIVE {
            let key = format!("{}-to-{}/{lane}/all", ARMS[a], ARMS[b]);
            let r = &pairs[&key];
            writeln!(md,"- {} to {}, {lane}. Recovered `{}` and lost `{}`. New wrong source `{}` and new unsupported write `{}`.",
                ARMS[a],ARMS[b],display(&r.recovered),display(&r.lost),display(&r.new_wrong_source),display(&r.new_unsupported))?;
        }
    }
    let ordinary_early: Vec<_> = signed
        .early
        .iter()
        .filter(|r| r["lane"] == "ordinary" && r["arm"] == "calibrated")
        .collect();
    writeln!(md,"\nStage 4 cleared {} genuine early gold candidates in the ordinary calibrated lane. These original candidates were unresolved in D28. Their row, fact addresses, source addresses, signed distances, radii and downstream outcomes are retained in the structured readout.\n",ordinary_early.len())?;
    for e in ordinary_early {
        writeln!(md,"- Row {} cleared original source {} under fact span `{}` with negative distance {} and radius {}. Baseline and calibrated released sources were `{}` and `{}`.",
            e["row"],e["source"],e["fact"],e["negative_distance"],e["negative_radius"],e["baseline_source"],e["actual_source"])?;
    }
    for row in [29000, 29010, 29108, 29109] {
        let p = prompt
            .iter()
            .find(|p| p.key.row == row)
            .ok_or("missing registered example row")?;
        let base = arms[0].grade(p, "ordinary");
        let calibrated = arms[1].grade(p, "ordinary");
        writeln!(md,"- Row {row} actual token IDs changed from `{}` to `{}` through stop. The first mismatch changed from `{}` to `{}`.",
            display(&base.tape), display(&calibrated.tape), display(&base.first), display(&calibrated.first))?;
    }
    writeln!(md, "\n## Identical-local boundary pairs\n")?;
    for entry in &boundary {
        writeln!(md,"- {} rows `{}` have identical original IDs from the fact head onward and different preceding words. Gold local sources are `{}` for `{}`. Baseline, calibrated and shuffled supplied-head local sources are `{}`, `{}` and `{}`.",
            entry["pair"].as_str().unwrap(),entry["rows"],entry["gold_local_offsets"],entry["gold_values"],
            entry["arms"]["baseline"]["supplied_local_sources"],entry["arms"]["calibrated"]["supplied_local_sources"],entry["arms"]["shuffled"]["supplied_local_sources"])?;
        if ARMS
            .into_iter()
            .all(|arm| entry["arms"][arm]["same_local_scores"] == true)
        {
            writeln!(
                md,
                "  Each arm gives the two members identical supplied-head candidate scores."
            )?;
        }
    }
    writeln!(md, "\n## Role order and next IDs\n")?;
    for label in ARMS {
        for group in ["role", "role_direct"] {
            let k = format!("{label}/{group}");
            let m = &roles["counts"][k];
            writeln!(md,"- {label}, {group}. Requested {}/{}, excluded {}/{}, joint {}/{}. Exact ordinary {} and supplied person {}.",
                m[1],m[0],m[2],m[0],m[3],m[0],m[4],m[5])?;
        }
    }
    writeln!(md,"\nThe factor matrix keeps requested-person choice, fact order and request/exclusion clause order for both role families. The per-row TSV gives each original pair of entity endpoints, assertion heads, released source, first actual ID, first mismatching ID and terminal EOS. The historical D21 role result provides context. This comparison uses the fixed D28 fitted-whole parent and registered D29 prompts. [D21 historical role result](../../codex-21/REPORT.md)\n\n[Per-row original addresses and reply tape diagnostics](paired-rows.tsv) · [Complete paired and signed evidence](paired-summary.json)")?;
    let summary = json!({
        "rows":prompt.len(),"supported":prompt.iter().filter(|p|p.key.expected_tape.is_some()).count(),
        "input_hashes":ARMS.into_iter().zip(arms).map(|(name,arm)|(name,json!({"grade_sha256":arm.summary_hash,"packet_sha256":arm.packet_hash}))).collect::<BTreeMap<_,_>>(),
        "train_calibration_sha256":fit.sha256,
        "training_signatures":fit.strata.values().collect::<Vec<_>>(),
        "available_negative_signatures":available,"changed_shuffle_signatures":changed,
        "metrics":totals,"paired":pairs,"boundary_pairs":boundary,"role_factors":roles,
        "signed_clearances":signed.clearances,"early_gold_clearances":signed.early,
        "unresolved_sign_reasons":signed.reasons,
    });
    fs::create_dir(path)?;
    fs::write(
        path.join("paired-summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    fs::write(path.join("paired-rows.tsv"), tsv)?;
    fs::write(path.join("PAIRED.md"), md)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [panel_path, grades, train, calibrated, shuffled, output] = args.as_slice() else {
        return Err("usage: d29-paired-analysis PANEL GRADES TRAIN.packet CALIBRATED.raw SHUFFLED.raw NEW_OUTPUT".into());
    };
    let prompts = panel(Path::new(panel_path))?;
    let fit = fit::inspect(Path::new(train), Path::new(calibrated), Path::new(shuffled))?;
    let arms: [Arm; 3] = ARMS
        .into_iter()
        .map(|arm| read_arm(&Path::new(grades).join(arm), &prompts))
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| "missing arm")?;
    render(&prompts, &arms, &fit, Path::new(output))
}
