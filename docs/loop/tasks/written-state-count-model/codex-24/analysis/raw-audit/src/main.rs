use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Feature = [i64; 32];

#[derive(Deserialize)]
struct Mention {
    prompt_id: String,
    person: String,
    family: String,
    ambiguous: bool,
    polarity_pair: Option<String>,
}

#[derive(Deserialize)]
struct Prompt {
    prompt_id: String,
    text: String,
}

fn require(ok: bool, why: impl FnOnce() -> String) -> Result<()> {
    if !ok {
        return Err(why().into());
    }
    Ok(())
}

fn read_json_lines<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?)
}

fn words<const N: usize>(line: &str) -> Result<[i64; N]> {
    line.split_ascii_whitespace()
        .map(str::parse)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| format!("expected {N} integer words").into())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn prediction_audit(raw: &Path, features: &[Feature]) -> Result<(Vec<i64>, Value)> {
    let bank = fs::read(raw.join("role.bank"))?;
    require(bank.len() == 800 && &bank[..8] == b"D24ROLE1", || {
        "incorrect native bank header or length".into()
    })?;
    let mut state = [[0i64; 33]; 3];
    for (word, bytes) in state.iter_mut().flatten().zip(bank[8..].chunks_exact(8)) {
        *word = i64::from_le_bytes(bytes.try_into()?);
    }
    require(
        state.iter().all(|role| (1..=1_000_000).contains(&role[0])),
        || "invalid bank counts".into(),
    )?;
    let counts = state.map(|record| i128::from(record[0]));
    let predictions = fs::read_to_string(raw.join("predictions.tsv"))?;
    let mut labels = Vec::new();
    for (row, line) in predictions.lines().enumerate() {
        let got = words::<4>(line)?;
        let point = features.get(row).ok_or("extra GPU prediction")?;
        let numerators: [i128; 3] = std::array::from_fn(|role| {
            point
                .iter()
                .zip(&state[role][1..])
                .map(|(&coordinate, &sum)| {
                    (counts[role] * i128::from(coordinate) - i128::from(sum)).abs()
                })
                .sum()
        });
        let winners: Vec<_> = (0..3)
            .filter(|&a| (0..3).all(|b| numerators[a] * counts[b] <= numerators[b] * counts[a]))
            .collect();
        let role = if winners.len() == 1 {
            winners[0] as i64
        } else {
            -1
        };
        require(
            got[0] == role && got[1..].iter().copied().map(i128::from).eq(numerators),
            || {
                format!(
                    "raw rational prediction mismatch at row {}: {}",
                    row + 1,
                    raw.display()
                )
            },
        )?;
        labels.push(role);
    }
    require(labels.len() == features.len(), || {
        "missing GPU prediction".into()
    })?;
    let receipt = json!({
        "bank_sha256": digest(&bank), "prediction_sha256": digest(predictions.as_bytes()),
        "training_counts": counts.map(|n| n as i64),
        "prediction_words_checked": 4 * labels.len(),
        "ties": labels.iter().filter(|&&label| label == -1).count()
    });
    Ok((labels, receipt))
}

#[derive(Default)]
struct Grade {
    prompts: usize,
    exact: usize,
    all_roles: usize,
    positive: usize,
    positive_exact: usize,
    false_positive_empty: usize,
    confusion: [[usize; 4]; 3],
}

impl Grade {
    fn write(&mut self, rows: &[usize], truth: &[usize], predictions: &[i64]) {
        let positive = rows.iter().any(|&row| truth[row] == 1);
        let exact = rows
            .iter()
            .all(|&row| (truth[row] == 1) == (predictions[row] == 1));
        self.prompts += 1;
        self.exact += usize::from(exact);
        self.all_roles += usize::from(
            rows.iter()
                .all(|&row| predictions[row] == truth[row] as i64),
        );
        self.positive += usize::from(positive);
        self.positive_exact += usize::from(positive && exact);
        self.false_positive_empty += usize::from(!positive && !exact);
        for &row in rows {
            let predicted = if predictions[row] < 0 {
                3
            } else {
                predictions[row] as usize
            };
            self.confusion[truth[row]][predicted] += 1;
        }
    }

    fn json(&self) -> Value {
        json!({
            "prompts": self.prompts, "exact_requested_set": self.exact,
            "all_mention_roles": self.all_roles, "gold_positive": self.positive,
            "positive_exact_requested_set": self.positive_exact,
            "false_positive_on_gold_empty": self.false_positive_empty,
            "confusion": self.confusion
        })
    }
}

fn representation_ceiling(features: &[Feature], truth: &[usize], known: &[usize]) -> Value {
    let mut classes = BTreeMap::<Feature, [usize; 3]>::new();
    for &row in known {
        classes.entry(features[row]).or_default()[truth[row]] += 1;
    }
    let mut conflicts = [0; 3];
    let mut request_conflicts = [0; 3];
    let mut class_ceiling = 0;
    let mut request_ceiling = 0;
    for counts in classes.values() {
        let classes_present = counts.iter().filter(|&&n| n > 0).count();
        let binary_conflict = counts[1] > 0 && counts[0] + counts[2] > 0;
        for role in 0..3 {
            if classes_present > 1 {
                conflicts[role] += counts[role];
            }
            if binary_conflict {
                request_conflicts[role] += counts[role];
            }
        }
        class_ceiling += counts.iter().max().unwrap_or(&0);
        request_ceiling += counts[1].max(counts[0] + counts[2]);
    }
    json!({
        "candidate_rows": known.len(), "distinct_vectors": classes.len(),
        "forced_conflict_rows_by_gold_class": conflicts,
        "requested_binary_conflict_rows_by_gold_class": request_conflicts,
        "any_deterministic_vector_only_class_correct_at_most": class_ceiling,
        "any_deterministic_vector_only_request_bit_correct_at_most": request_ceiling
    })
}

fn exact_set_ceiling(
    features: &[Feature],
    truth: &[usize],
    prompts: &BTreeMap<&str, Vec<usize>>,
) -> Result<usize> {
    let mut vector_ids = BTreeMap::new();
    let mut requirements = Vec::new();
    for rows in prompts.values() {
        let mut must_request = 0usize;
        let mut must_skip = 0usize;
        for &row in rows {
            let next = vector_ids.len();
            let index = *vector_ids.entry(features[row]).or_insert(next);
            require(index < 16, || {
                "too many distinct vectors for exact enumeration".into()
            })?;
            let bits = if truth[row] == 1 {
                &mut must_request
            } else {
                &mut must_skip
            };
            *bits |= 1 << index;
        }
        requirements.push((must_request, must_skip));
    }
    // Enumerate every vector-only request mask. Emit the ceiling, never a mask.
    Ok((0usize..1 << vector_ids.len())
        .map(|mask| {
            requirements
                .iter()
                .filter(|&&(yes, no)| mask & yes == yes && mask & no == 0)
                .count()
        })
        .max()
        .unwrap_or(0))
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [root] = args.as_slice() else {
        return Err("usage: d24-raw-audit CLAUSE_ROLE_24_ROOT".into());
    };
    let root = Path::new(root);
    let panel = root.join("panel");
    let mentions: Vec<Mention> = read_json_lines(&panel.join("held.rows.jsonl"))?;
    let text: BTreeMap<_, _> = read_json_lines::<Prompt>(&panel.join("held.prompts.jsonl"))?
        .into_iter()
        .map(|row| (row.prompt_id, row.text))
        .collect();
    let truth = fs::read_to_string(panel.join("held.labels.txt"))?
        .lines()
        .map(str::parse)
        .collect::<std::result::Result<Vec<usize>, _>>()?;
    require(
        truth.len() == mentions.len() && truth.iter().all(|&label| label < 3),
        || "invalid held label tape".into(),
    )?;
    let mut prompts = BTreeMap::<&str, Vec<usize>>::new();
    let mut polarity = BTreeMap::<(&str, &str), Vec<usize>>::new();
    for (row, mention) in mentions
        .iter()
        .enumerate()
        .filter(|(_, mention)| !mention.ambiguous)
    {
        prompts.entry(&mention.prompt_id).or_default().push(row);
        if let Some(pair) = &mention.polarity_pair {
            polarity
                .entry((pair, &mention.person))
                .or_default()
                .push(row);
        }
    }
    require(
        prompts
            .values()
            .all(|rows| rows.len() == 2 && mentions[rows[0]].person != mentions[rows[1]].person),
        || "invalid prompt groups".into(),
    )?;
    require(
        polarity
            .values()
            .all(|rows| rows.len() == 2 && truth[rows[0]] != truth[rows[1]]),
        || "invalid within-person polarity pairs".into(),
    )?;
    let known: Vec<_> = prompts.values().flatten().copied().collect();
    let slots = fs::read_to_string(panel.join("held.slots.txt"))?
        .lines()
        .map(words::<24>)
        .collect::<Result<Vec<_>>>()?;
    require(slots.len() == mentions.len(), || {
        "wrong slot tape length".into()
    })?;
    let mut report = json!({
        "role_order": ["unspecified", "requested", "excluded"],
        "confusion_prediction_order": ["unspecified", "requested", "excluded", "tie"],
        "unresolved_prompts_excluded_from_grade": (mentions.len() - known.len()) / 2
    });
    for arm in ["identity", "positional"] {
        let feature_path = format!("held.{arm}.txt");
        let raw_features = fs::read_to_string(panel.join(&feature_path))?;
        require(
            raw_features == fs::read_to_string(root.join("panel-seed2").join(&feature_path))?,
            || "artifact paths have different coordinates; audit them separately".into(),
        )?;
        let features = raw_features
            .lines()
            .map(words::<32>)
            .collect::<Result<Vec<_>>>()?;
        require(features.len() == mentions.len(), || {
            "wrong feature tape length".into()
        })?;
        let mut arm_report = json!({
            "held_feature_sha256": digest(raw_features.as_bytes()),
            "ceiling": representation_ceiling(&features, &truth, &known),
            "exact_requested_sets_ceiling": exact_set_ceiling(&features, &truth, &prompts)?
        });
        let mut previous = None;
        for seed in ["seed1", "seed2"] {
            let result = root.join("result").join(seed).join(arm);
            let raw = result.join("primary");
            require(
                raw_features.as_bytes() == fs::read(raw.join("held.features"))?,
                || format!("GPU held coordinates differ from the sealed Rust tape: {seed}/{arm}"),
            )?;
            for file in ["role.bank", "held.features", "predictions.tsv"] {
                require(
                    fs::read(raw.join(file))? == fs::read(result.join("cold").join(file))?,
                    || format!("cold artifact differs: {seed}/{arm}/{file}"),
                )?;
            }
            let (predictions, receipt) = prediction_audit(&raw, &features)?;
            if let Some(prior) = &previous {
                require(prior == &receipt, || {
                    "identical projected inputs produced different GPU artifacts".into()
                })?;
            }
            previous = Some(receipt.clone());
            arm_report[seed] = receipt;
            let mut totals = Grade::default();
            let mut strata = BTreeMap::<&str, Grade>::new();
            for rows in prompts.values() {
                let family = mentions[rows[0]].family.as_str();
                let cue = if matches!(family, "near" | "right") {
                    "changed_cue"
                } else {
                    "shared_cue"
                };
                totals.write(rows, &truth, &predictions);
                for key in [family, cue] {
                    strata
                        .entry(key)
                        .or_default()
                        .write(rows, &truth, &predictions);
                }
            }
            let mut pair_stats = BTreeMap::<&str, [usize; 6]>::new();
            let mut examples = Vec::new();
            for rows in polarity.values() {
                let [a, b] = [rows[0], rows[1]];
                let family = mentions[a].family.as_str();
                let same_bag = {
                    let (mut left, mut right) = (slots[a], slots[b]);
                    left.sort_unstable();
                    right.sort_unstable();
                    left == right
                };
                let same_feature = features[a] == features[b];
                let both_correct =
                    predictions[a] == truth[a] as i64 && predictions[b] == truth[b] as i64;
                let summary = pair_stats.entry(family).or_default();
                summary[0] += 1;
                summary[1] += usize::from(same_bag);
                summary[2] += usize::from(same_feature);
                summary[3] += usize::from(both_correct);
                summary[4] += usize::from(same_bag && !same_feature);
                summary[5] += usize::from(same_bag && both_correct);
                if family == "root_order" && same_bag && examples.is_empty() {
                    examples.push(json!({
                        "person": mentions[a].person, "texts": [&text[&mentions[a].prompt_id], &text[&mentions[b].prompt_id]],
                        "true_roles": [truth[a], truth[b]],
                        "predicted_roles": [predictions[a], predictions[b]],
                        "same_feature": same_feature
                    }));
                }
            }
            arm_report["grade"] = totals.json();
            arm_report["strata"] = json!(strata
                .into_iter()
                .map(|(name, grade)| (name, grade.json()))
                .collect::<BTreeMap<_, _>>());
            arm_report["same_person_polarity_columns"] = json!([
                "pairs",
                "same_root_multiset",
                "same_vector",
                "both_roles_correct",
                "same_roots_distinct_vectors",
                "same_roots_both_roles_correct"
            ]);
            arm_report["same_person_polarity_by_family"] = json!(pair_stats);
            arm_report["example"] = json!(examples);
        }
        report[arm] = arm_report;
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
