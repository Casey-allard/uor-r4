use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ARMS: [&str; 6] = [
    "frozen-legacy",
    "frozen-whole",
    "fitted-legacy",
    "fitted-whole",
    "augmented-legacy",
    "augmented-whole",
];
const NATIVE: [&str; 3] = ["ordinary", "person", "assertions"];
const NIL: u64 = u32::MAX as u64;
const STOPS: [u64; 4] = [16, 1, 0, 201];

fn check(ok: bool, context: &str) -> Result<()> {
    if !ok {
        return Err(context.to_owned().into());
    }
    Ok(())
}

fn num(value: &Value) -> Result<u64> {
    Ok(value.as_u64().ok_or("expected integer")?)
}
fn string(value: &Value) -> Result<&str> {
    Ok(value.as_str().ok_or("expected string")?)
}
fn array(value: &Value) -> Result<&[Value]> {
    Ok(value.as_array().ok_or("expected array")?)
}
fn words(value: &Value) -> Result<Vec<u64>> {
    array(value)?.iter().map(num).collect()
}

fn span(value: &Value, length: usize) -> Result<[usize; 2]> {
    match words(value)?.as_slice() {
        [a, b] if a < b && *b <= length as u64 => Ok([*a as usize, *b as usize]),
        _ => Err(format!("invalid span {value}").into()),
    }
}

fn jsonl(path: &Path) -> Result<BTreeMap<u64, Value>> {
    let mut entries = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let value: Value = serde_json::from_str(line)?;
        let id = num(&value["row"])?;
        check(entries.insert(id, value).is_none(), "duplicate sealed row")?;
    }
    check(!entries.is_empty(), "empty seal")?;
    Ok(entries)
}

fn first(expected: &[u64], actual: &[u64]) -> Value {
    (0..expected.len().max(actual.len()))
        .find(|&i| expected.get(i) != actual.get(i))
        .map(|i| json!({"offset":i,"expected":expected.get(i),"actual":actual.get(i)}))
        .unwrap_or(Value::Null)
}

fn copy_tape(ids: &[u64], source: u64) -> Result<Vec<u64>> {
    let suffix = ids.get(source as usize..).ok_or("source outside prompt")?;
    let fence = suffix
        .iter()
        .position(|id| STOPS.contains(id))
        .ok_or("copy source has no stop")?;
    check((1..=7).contains(&fence), "source outside writer copy cap")?;
    Ok([&suffix[..fence], &[1]].concat())
}

struct Key {
    ids: Vec<u64>,
    gold: Value,
    requested: Value,
    heads: Vec<([usize; 2], [usize; 2])>,
    target: Option<Vec<u64>>,
}

fn read_panel(panel: &Path) -> Result<BTreeMap<u64, Key>> {
    let blind = jsonl(&panel.join("held.blind.jsonl"))?;
    let gold = jsonl(&panel.join("held.gold.jsonl"))?;
    let people = jsonl(&panel.join("held.person.jsonl"))?;
    check(
        blind.keys().eq(gold.keys()) && blind.keys().eq(people.keys()),
        "sealed keys differ",
    )?;
    let mut output = BTreeMap::new();
    for (row, gold) in gold {
        let ids = words(&blind[&row]["ids"])?;
        check(
            ids.first() == Some(&0) && ids.iter().all(|&id| id < 4096),
            "invalid prompt tape",
        )?;
        let bytes: Vec<_> = ids
            .iter()
            .flat_map(|&id| (id as u32).to_le_bytes())
            .collect();
        check(
            gold["prefix_sha256"] == format!("{:x}", Sha256::digest(bytes)),
            "prefix digest differs",
        )?;
        let requested = people[&row]["requested"].clone();
        let query = span(&requested, ids.len())?;
        check(gold["requested"] == requested, "requested seal differs")?;
        let mut heads = Vec::new();
        let facts = array(&gold["assertions"])?;
        for index in words(&gold["selected"])? {
            let fact = facts
                .get(index as usize)
                .ok_or("selected assertion outside list")?;
            let head = span(&fact["person"], ids.len())?;
            let value = span(&fact["value"], ids.len())?;
            check(
                !heads.iter().any(|(h, _)| *h == head)
                    && head[1] <= value[0]
                    && value[1] < query[0]
                    && ids[head[0]..head[1]] == ids[query[0]..query[1]]
                    && STOPS.contains(&ids[value[1]]),
                "selected gold assertion is not a unique causal span",
            )?;
            heads.push((head, value));
        }
        check(
            gold["selected_value_spans"] == json!(heads.iter().map(|(_, v)| v).collect::<Vec<_>>()),
            "selected value spans disagree",
        )?;
        let target = (!gold["expected_tape"].is_null())
            .then(|| words(&gold["expected_tape"]))
            .transpose()?;
        check(
            matches!(
                (string(&gold["stratum"])?, target.is_some(), heads.len()),
                ("matched", true, 1)
                    | ("agreement", true, 2)
                    | ("conflict", false, 2)
                    | ("absent", false, 0)
            ),
            "invalid stratum or denominator",
        )?;
        if let Some(target) = &target {
            for &(_, [start, end]) in &heads {
                check(
                    copy_tape(&ids, start as u64)? == *target
                        && ids[start..end] == target[..target.len() - 1],
                    "target is not original value IDs through EOS",
                )?;
            }
        } else if heads.len() == 2 {
            check(
                copy_tape(&ids, heads[0].1[0] as u64)? != copy_tape(&ids, heads[1].1[0] as u64)?,
                "conflicting oracle tapes agree",
            )?;
        }
        output.insert(
            row,
            Key {
                ids,
                gold,
                requested,
                heads,
                target,
            },
        );
    }
    Ok(output)
}

fn candidate_reading(c: &[u64]) -> Result<&'static str> {
    let margin = c[8] as u32 as i32;
    Ok(match c[4] {
        1 if c[3] >= 8 && c[5..].iter().all(|&word| word == NIL) => "copy_cap",
        2 if c[3] < 8 && c[8..] == [NIL, NIL] => match (c[5], c[6], c[7]) {
            (NIL, NIL, _) => "unseen_both",
            (NIL, _, _) => "unseen_positive",
            (_, NIL, _) => "unseen_negative",
            (_, _, 65535) => "unfitted_radius",
            (positive, _, radius) if positive > radius => "positive_radius_reject",
            _ => return Err("seen candidate marked unseen".into()),
        },
        3 if c[3] < 8
            && c[5] != NIL
            && c[6] != NIL
            && c[7] != 65535
            && c[5] <= c[7]
            && margin as i64 == c[6] as i64 - c[5] as i64
            && c[9] == c[2].min(3) + c[1] - c[0] =>
        {
            if margin > 0 {
                "positive_margin"
            } else {
                "nonpositive"
            }
        }
        _ => return Err("invalid candidate trace words".into()),
    })
}

fn replay_onset(candidates: &[Vec<u64>]) -> (u64, Option<u64>) {
    if candidates.iter().any(|c| c[4] == 2) {
        return (2, None);
    }
    let ranked = |a: &[u64], b: &[u64]| {
        let signed = |word: u64| word as u32 as i32 as i64;
        (signed(a[8]) * b[9] as i64).cmp(&(signed(b[8]) * a[9] as i64))
    };
    let Some(best) = candidates
        .iter()
        .filter(|c| c[4] == 3)
        .max_by(|a, b| ranked(a, b))
    else {
        return (1, None);
    };
    if best[8] as u32 as i32 <= 0 {
        return (4, None);
    }
    if candidates
        .iter()
        .filter(|c| c[4] == 3 && ranked(c, best).is_eq())
        .count()
        > 1
    {
        return (3, None);
    }
    (0, Some(best[0]))
}

fn visits(arm: &str, lane: &str, key: &Key, stages: &Value) -> Result<Vec<Value>> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for stage in array(stages)? {
        let head = span(&stage["head"], key.ids.len())?;
        let query = span(&stage["actual_query"], key.ids.len())?;
        check(
            seen.insert(head) && head[1] < query[0] && stage["query"] == key.requested,
            "invalid assertion visit",
        )?;
        check(
            lane == "ordinary" || stage["actual_query"] == key.requested,
            "supplied query differs",
        )?;
        let fence = stage["fence"].as_u64().map(|f| f as usize);
        if let Some(end) = fence {
            check(
                (head[1]..query[0]).contains(&end) && STOPS.contains(&key.ids[end]),
                "invalid assertion fence",
            )?;
        }
        let status = num(&stage["onset"]["status"])?;
        let selected = stage["onset"]["source"].as_u64();
        check(
            status <= 4 && (status == 0) == selected.is_some(),
            "invalid onset result",
        )?;
        let gold = key.heads.iter().find(|(h, _)| *h == head).map(|(_, v)| *v);
        let mut candidates = Vec::new();
        let mut raw_words = Vec::new();
        let mut previous_end = head[1] as u64 + 1;
        for candidate in array(&stage["candidate_words"])? {
            let c = words(candidate)?;
            let end = fence.ok_or("candidate without original fence")? as u64;
            check(
                c.len() == 10
                    && c.iter().all(|&w| w <= NIL)
                    && c[0] >= previous_end
                    && c[0] < c[1]
                    && c[1] <= end
                    && c[2] == c[0] - head[1] as u64
                    && c[3] == end - c[0]
                    && (!arm.ends_with("-legacy") || c[2] <= 3),
                "candidate source coordinates differ",
            )?;
            previous_end = c[1];
            candidates.push(json!({"source":c[0],"words":c,"reading":candidate_reading(&c)?}));
            raw_words.push(c);
        }
        check(
            replay_onset(&raw_words) == (status, selected),
            "onset status or winner differs from candidate score replay",
        )?;
        let gold_reading = gold.map(|[source, _]| {
            candidates
                .iter()
                .find(|c| c["source"] == source)
                .map(|c| c["reading"].clone())
                .unwrap_or(json!("ineligible"))
        });
        let wrong = selected
            .zip(gold)
            .and_then(|(s, [g, _])| match s.cmp(&(g as u64)) {
                std::cmp::Ordering::Less => Some("premature"),
                std::cmp::Ordering::Greater => Some("late_suffix"),
                std::cmp::Ordering::Equal => None,
            });
        result.push(
            json!({"head":head,"fence":fence,"status":status,"onset_source":selected,
            "gold":gold,"gold_reading":gold_reading,"wrong_onset":wrong,"candidates":candidates,
            "rival_unseen_veto":status == 2 && gold_reading.as_ref().is_some_and(|r| r == "positive_margin" || r == "nonpositive"),
            "continuation":stage["continuation"],"completion":stage["completion"]}),
        );
    }
    if lane == "assertions" {
        check(
            seen == key.heads.iter().map(|(h, _)| *h).collect(),
            "oracle assertion visit denominator differs",
        )?;
    }
    Ok(result)
}

type Outcomes = BTreeMap<(u64, String), Value>;

fn analyze_arm(arm: &str, root: &Path, keys: &BTreeMap<u64, Key>) -> Result<Outcomes> {
    let root = root.join(arm);
    let rows: Vec<Value> = serde_json::from_slice(&fs::read(root.join("rows.json"))?)?;
    let visit_rows: Vec<Value> = serde_json::from_slice(&fs::read(root.join("visits.json"))?)?;
    let mut accounted = BTreeSet::new();
    let mut result = BTreeMap::new();
    for record in rows {
        let id = num(&record["row"])?;
        let lane = string(&record["lane"])?;
        let context = format!("{arm}/{id}/{lane}");
        let key = keys.get(&id).ok_or("unknown grader row")?;
        let stages = array(&record["stages"])?;
        for stage in stages {
            let index = num(&stage["visit"])? as usize;
            check(
                accounted.insert(index)
                    && visit_rows.get(index) == Some(&json!({"row":id,"lane":lane,"stage":stage})),
                &format!("{context}: visit ledger mismatch"),
            )?;
        }
        let person = &record["person"];
        let source = record["source"].as_u64();
        let matching = person == &key.requested;
        let exact_source = key.target.is_some()
            && source.is_some_and(|s| key.heads.iter().any(|(_, v)| v[0] as u64 == s));
        let tape = words(&record["tape"])?;
        let exact = key.target.as_ref().map(|target| *target == tape);
        let delta = key
            .target
            .as_ref()
            .map(|target| first(target, &tape))
            .unwrap_or(Value::Null);
        check(
            record["stratum"] == key.gold["stratum"]
                && record["expected"] == key.gold["expected_tape"]
                && record["value_spans"] == key.gold["selected_value_spans"]
                && record["exact"] == json!(exact)
                && record["source_correct"] == exact_source
                && record["person_correct"] == matching
                && record["first"] == delta,
            &format!("{context}: grader fields contradict sealed key or raw tape"),
        )?;
        check(
            !tape.is_empty()
                && tape.len() <= 16
                && tape.iter().all(|&id| id < 4096)
                && !tape[..tape.len() - 1].contains(&1),
            &format!("{context}: invalid output tape"),
        )?;
        if !person.is_null() {
            span(person, key.ids.len())?;
        }
        if let Some(source) = source {
            check(
                copy_tape(&key.ids, source)? == tape,
                &format!("{context}: generated tape differs from original source and EOS"),
            )?;
        }
        let localization = if NATIVE.contains(&lane) {
            check(
                record["decision"].is_object()
                    && record["decision"]["choice"]["source"] == record["source"],
                &format!("{context}: native decision source differs"),
            )?;
            for stage in stages {
                check(
                    &stage["actual_query"] == person,
                    &format!("{context}: trace uses a different query"),
                )?;
            }
            check(
                lane == "ordinary" || matching,
                &format!("{context}: incorrect oracle query"),
            )?;
            visits(arm, lane, key, &record["stages"]).map_err(|e| format!("{context}: {e}"))?
        } else {
            check(
                stages.is_empty() && record["decision"].is_null(),
                &format!("{context}: forced lane in native ledger"),
            )?;
            match lane {
                "gold_value" => check(
                    matching
                        && exact == Some(true)
                        && source == key.heads.first().map(|(_, v)| v[0] as u64),
                    &format!("{context}: invalid gold writer oracle"),
                )?,
                "opposing" => {
                    let source = source.ok_or("opposing lane has no source")?;
                    let allowed = key.gold["wrong_source"].as_u64().map_or_else(
                        || {
                            key.gold["stratum"] == "agreement"
                                && key.heads.first().is_some_and(|(_, v)| {
                                    (v[0] as u64 + 1..v[1] as u64).contains(&source)
                                })
                        },
                        |wrong| source == wrong,
                    );
                    check(
                        matching && exact == Some(false) && allowed,
                        &format!("{context}: invalid opposing writer oracle"),
                    )?;
                }
                "disabled" => check(
                    person.is_null() && source.is_none(),
                    &format!("{context}: disabled writer copied"),
                )?,
                _ => return Err(format!("unknown lane {lane}").into()),
            }
            Vec::new()
        };
        let issues: Vec<_> = [
            (lane == "ordinary" && !matching, "query_endpoint"),
            (
                NATIVE.contains(&lane)
                    && key.target.is_some()
                    && matching
                    && localization.iter().all(|v| v["gold"].is_null()),
                "gold_assertion_missing",
            ),
            (
                localization
                    .iter()
                    .any(|v| v["gold_reading"] == "ineligible"),
                "gold_ineligible",
            ),
            (
                localization.iter().any(|v| v["status"] == 2),
                "assertion_unseen_veto",
            ),
            (
                localization.iter().any(|v| v["wrong_onset"] == "premature"),
                "premature_onset",
            ),
            (
                localization
                    .iter()
                    .any(|v| v["wrong_onset"] == "late_suffix"),
                "late_suffix",
            ),
            (
                key.target.is_some() && source.is_some() && !exact_source,
                "wrong_released_source",
            ),
            (
                key.target.is_none() && source.is_some(),
                "unsupported_release",
            ),
        ]
        .into_iter()
        .filter_map(|(set, name)| set.then_some(name))
        .collect();
        let item = json!({"person":person,"person_correct":matching,"source":source,"source_correct":exact_source,
            "exact":exact,"first":delta,"tape":tape,"eos":tape.last()==Some(&1),"issues":issues,"visits":localization});
        check(
            result.insert((id, lane.to_owned()), item).is_none(),
            &format!("{context}: duplicate lane row"),
        )?;
    }
    check(
        accounted.len() == visit_rows.len(),
        &format!("{arm}: unreferenced visits"),
    )?;
    for (&id, key) in keys {
        for lane in NATIVE.into_iter().chain(["disabled"]) {
            check(
                result.contains_key(&(id, lane.to_owned())),
                &format!("{arm}/{id}: missing {lane}"),
            )?;
        }
        for lane in ["gold_value", "opposing"] {
            let needed = lane == "gold_value" && key.target.is_some()
                || lane == "opposing" && !key.gold["wrong_source"].is_null();
            check(
                !needed || result.contains_key(&(id, lane.to_owned())),
                &format!("{arm}/{id}: missing {lane}"),
            )?;
        }
    }
    Ok(result)
}

const PAIRS: [(&str, &str); 11] = [
    ("frozen-legacy", "frozen-whole"),
    ("fitted-legacy", "fitted-whole"),
    ("augmented-legacy", "augmented-whole"),
    ("frozen-legacy", "fitted-legacy"),
    ("frozen-whole", "fitted-whole"),
    ("frozen-legacy", "augmented-legacy"),
    ("frozen-whole", "augmented-whole"),
    ("frozen-legacy", "fitted-whole"),
    ("frozen-legacy", "augmented-whole"),
    ("fitted-legacy", "augmented-legacy"),
    ("fitted-whole", "augmented-whole"),
];

fn is_new_late_exact(before: &Value, after: &Value) -> bool {
    if after["source_correct"] != true
        || after["exact"] != true
        || after["eos"] != true
        || before["source_correct"] == true && before["exact"] == true
    {
        return false;
    }
    let Some(source) = after["source"].as_u64() else {
        return false;
    };
    after["visits"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|visit| {
            visit["onset_source"] == source
                && visit["gold"][0] == source
                && visit["head"][1]
                    .as_u64()
                    .and_then(|end| source.checked_sub(end))
                    .is_some_and(|pieces| pieces > 3)
        })
}

fn summary(ids: &[u64], arms: &BTreeMap<&str, Outcomes>) -> Value {
    let counts: Vec<_> = arms.iter().map(|(&arm,values)| {
        let lanes: BTreeSet<_> = values.keys().map(|(_,lane)|lane).collect();
        let report: Vec<_> = lanes.into_iter().map(|lane| {
            let rows: Vec<_> = ids.iter().filter_map(|id|values.get(&(*id,lane.clone()))).collect();
            let mut issue_counts = BTreeMap::new();
            let mut gold_counts = BTreeMap::new();
            for v in &rows {
                for issue in v["issues"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                    *issue_counts.entry(issue).or_insert(0) += 1;
                }
                for gold in v["visits"].as_array().into_iter().flatten().filter_map(|v|v["gold_reading"].as_str()) {
                    *gold_counts.entry(gold).or_insert(0) += 1;
                }
            }
            json!({"lane":lane,"n":rows.len(),"graded":rows.iter().filter(|r|r["exact"].is_boolean()).count(),
                "exact":rows.iter().filter(|r|r["exact"]==true).count(),
                "gold_source":rows.iter().filter(|r|r["source_correct"]==true).count(),
                "eos":rows.iter().filter(|r|r["eos"]==true).count(),
                "issues":issue_counts,"gold_candidate_visits":gold_counts})
        }).collect();
        let interventions: Vec<_> = [
            ("ordinary", "person"),
            ("person", "assertions"),
            ("assertions", "gold_value"),
            ("disabled", "ordinary"),
        ].into_iter().map(|(before,after)| {
            let mut gains = Vec::new();
            let mut losses = Vec::new();
            let mut n = 0;
            for &row in ids {
                let a = values.get(&(row,before.to_owned())).and_then(|v| v["exact"].as_bool());
                let b = values.get(&(row,after.to_owned())).and_then(|v| v["exact"].as_bool());
                if let (Some(a),Some(b)) = (a,b) {
                    n += 1;
                    if !a && b { gains.push(row); }
                    if a && !b { losses.push(row); }
                }
            }
            json!({"before":before,"after":after,"graded":n,"gains":gains,"losses":losses})
        }).collect();
        json!({"arm":arm,"lanes":report,"interventions":interventions})
    }).collect();
    let comparisons: Vec<_> = PAIRS
        .into_iter()
        .map(|(before, after)| {
            let lanes: Vec<_> = NATIVE
                .into_iter()
                .map(|lane| {
                    let mut gains = Vec::new();
                    let mut losses = Vec::new();
                    let mut new_wrong = Vec::new();
                    let mut new_late = Vec::new();
                    let mut both = 0;
                    for &id in ids {
                        let mapkey = (id, lane.to_owned());
                        let a = &arms[before][&mapkey];
                        let b = &arms[after][&mapkey];
                        match (a["exact"].as_bool(), b["exact"].as_bool()) {
                            (Some(false), Some(true)) => gains.push(id),
                            (Some(true), Some(false)) => losses.push(id),
                            (Some(true), Some(true)) => both += 1,
                            _ => {}
                        }
                        let wrong =
                            |v: &Value| !v["source"].is_null() && v["source_correct"] == false;
                        if !wrong(a) && wrong(b) {
                            new_wrong.push(id);
                        }
                        let late = is_new_late_exact(a, b);
                        if late {
                            new_late.push(id);
                        }
                    }
                    json!({"lane":lane,"gains":gains,"losses":losses,"unchanged_exact":both,
                "new_wrong_or_unsupported_releases":new_wrong,"new_correct_late_sources":new_late})
                })
                .collect();
            json!({"before":before,"after":after,"lanes":lanes})
        })
        .collect();
    json!({"n":ids.len(),"arms":counts,"paired":comparisons})
}

fn run(panel: &Path, grade: &Path, out: &Path) -> Result<()> {
    let keys = read_panel(panel)?;
    let mut texts = BTreeMap::new();
    for line in fs::read_to_string(panel.join("held.blind.tsv"))?.lines() {
        let (row, text) = line.split_once('\t').ok_or("invalid blind prompt TSV")?;
        check(
            !text.is_empty() && texts.insert(row.parse::<u64>()?, text.to_owned()).is_none(),
            "duplicate or empty blind prompt",
        )?;
    }
    check(
        texts.keys().eq(keys.keys()),
        "blind prompt text denominator differs",
    )?;
    let mut arms = BTreeMap::new();
    for arm in ARMS {
        arms.insert(arm, analyze_arm(arm, grade, &keys)?);
    }
    let expected = arms[ARMS[0]].keys();
    for arm in ARMS {
        check(
            arms[arm].keys().eq(expected.clone()),
            &format!("{arm}: cross-arm denominators differ"),
        )?;
    }
    let all: Vec<_> = keys.keys().copied().collect();
    let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut details = Vec::new();
    for (&id, key) in &keys {
        let stratum = string(&key.gold["stratum"])?;
        groups
            .entry(format!("stratum/{stratum}"))
            .or_default()
            .push(id);
        if stratum == "matched" {
            let family = string(&key.gold["family"])?;
            let connector = match num(&key.gold["connector_arm"])? {
                0 => "early",
                1 => "late",
                _ => return Err("invalid connector arm".into()),
            };
            groups
                .entry(format!("family/{family}"))
                .or_default()
                .push(id);
            groups
                .entry(format!("connector/{connector}"))
                .or_default()
                .push(id);
            groups
                .entry(format!("family/{family}/{connector}"))
                .or_default()
                .push(id);
        }
        let outcomes: BTreeMap<_, BTreeMap<_, _>> = arms
            .iter()
            .map(|(&arm, items)| {
                (
                    arm,
                    items
                        .iter()
                        .filter(|((row, _), _)| *row == id)
                        .map(|((_, lane), value)| (lane, value))
                        .collect(),
                )
            })
            .collect();
        details.push(json!({"row":id,"text":texts[&id],"stratum":stratum,"family":key.gold["family"],
            "connector_arm":key.gold["connector_arm"],"object_arm":key.gold["object_arm"],
            "requested":key.requested,"gold":key.gold["selected_value_spans"],"expected_value":key.gold["expected_value"],
            "expected":key.target,"outcomes":outcomes}));
    }
    let groups: BTreeMap<_, _> = groups
        .iter()
        .map(|(name, ids)| (name, summary(ids, &arms)))
        .collect();
    let mut file = File::create_new(out)?;
    serde_json::to_writer_pretty(
        &mut file,
        &json!({"schema":"uor-r4.d28-paired-analysis/1",
        "overall":summary(&all,&arms),"groups":groups,"rows":details}),
    )?;
    writeln!(file)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [panel, grade, out] = args.as_slice() else {
        return Err("usage: d28-onset-analysis PANEL GRADE_ROOT NEW_REPORT_JSON".into());
    };
    run(Path::new(panel), Path::new(grade), Path::new(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn newly_reachable_uses_actual_connector_and_eos() {
        let absent = json!({"source_correct":false});
        let mut outcome = json!({"connector_arm":1,"source_correct":true,"exact":true,"eos":true,
            "source":6,"visits":[{"onset_source":6,"gold":[6,7],"head":[3,4]}]});
        assert!(!is_new_late_exact(&absent, &outcome));
        outcome["visits"][0]["head"] = json!([1, 2]);
        assert!(is_new_late_exact(&absent, &outcome));
        outcome["eos"] = json!(false);
        assert!(!is_new_late_exact(&absent, &outcome));
    }

    fn save(path: &Path, value: &Value) -> Result<()> {
        fs::write(path, serde_json::to_vec(value)?)?;
        Ok(())
    }

    #[test]
    fn planted_pair_and_corrupt_receipts() -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("d28-analysis-{}-{timestamp}", std::process::id()));
        fs::create_dir(&root)?;
        let ids = [0_u32, 10, 20, 30, 40, 16, 10, 50];
        let sha = format!(
            "{:x}",
            Sha256::digest(
                ids.iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>()
            )
        );
        save(&root.join("held.blind.jsonl"), &json!({"row":1,"ids":ids}))?;
        save(
            &root.join("held.person.jsonl"),
            &json!({"row":1,"requested":[6,7]}),
        )?;
        save(
            &root.join("held.gold.jsonl"),
            &json!({"row":1,"stratum":"matched","family":"plant",
            "requested":[6,7],"assertions":[{"person":[1,2],"value":[3,5]}],"selected":[0],
            "selected_value_spans":[[3,5]],"expected_tape":[30,40,1],"prefix_sha256":sha,
            "connector_arm":0,"object_arm":0,"wrong_source":4}),
        )?;
        fs::write(root.join("held.blind.tsv"), "1\tPlanted prompt.\n")?;
        for arm in ARMS {
            let path = root.join(arm);
            fs::create_dir(&path)?;
            let winning = arm.starts_with("fitted");
            let mut visits = Vec::new();
            let mut rows = Vec::new();
            for lane in NATIVE
                .into_iter()
                .chain(["gold_value", "opposing", "disabled"])
            {
                let mut stages = Vec::new();
                let person = (lane != "disabled").then_some([6, 7]);
                let (source, tape): (Option<u64>, Vec<u64>) = match lane {
                    "gold_value" => (Some(3), vec![30, 40, 1]),
                    "opposing" => (Some(4), vec![40, 1]),
                    "disabled" => (None, vec![99, 1]),
                    _ if winning => (Some(3), vec![30, 40, 1]),
                    _ => (None, vec![99, 1]),
                };
                let decision = if NATIVE.contains(&lane) {
                    let candidate = if winning {
                        vec![3, 4, 1, 2, 3, 5, 15, 10, 10, 2]
                    } else {
                        vec![3, 4, 1, 2, 2, 5, NIL, 10, NIL, NIL]
                    };
                    let stage = json!({"visit":visits.len(),"head":[1,2],"actual_query":[6,7],"query":[6,7],
                        "fence":5,"onset":{"status":if winning { 0 } else { 2 },"source":source},"candidate_words":[candidate]});
                    visits.push(json!({"row":1,"lane":lane,"stage":stage}));
                    stages.push(stage);
                    json!({"choice":{"source":source}})
                } else {
                    Value::Null
                };
                rows.push(json!({"row":1,"stratum":"matched","lane":lane,"person":person,"person_correct":person.is_some(),
                    "source":source,"source_correct":source==Some(3),"value_spans":[[3,5]],"expected":[30,40,1],"tape":tape,
                    "first":first(&[30,40,1],&tape),"exact":tape==[30,40,1],"decision":decision,"stages":stages}));
            }
            save(&path.join("visits.json"), &json!(visits))?;
            save(&path.join("rows.json"), &json!(rows))?;
        }
        let output = root.join("result.json");
        run(&root, &root, &output)?;
        let result: Value = serde_json::from_slice(&fs::read(output)?)?;
        assert_eq!(result["overall"]["n"], 1);
        assert_eq!(
            result["overall"]["paired"][3]["lanes"][0]["gains"],
            json!([1])
        );
        let path = root.join("frozen-legacy/rows.json");
        let mut rows: Value = serde_json::from_slice(&fs::read(&path)?)?;
        rows[0]["first"]["actual"] = json!(98);
        save(&path, &rows)?;
        let failure = run(&root, &root, &root.join("bad-result.json"))
            .expect_err("tampered tape mismatch passed");
        assert!(failure
            .to_string()
            .contains("contradict sealed key or raw tape"));
        rows[0]["first"]["actual"] = json!(99);
        rows[0]["stages"][0]["visit"] = json!(1);
        save(&path, &rows)?;
        assert!(run(&root, &root, &root.join("bad-visits.json"))
            .expect_err("bad visit passed")
            .to_string()
            .contains("visit ledger mismatch"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
