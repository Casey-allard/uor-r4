//! `paraphrase-gen` — teacher paraphrase data for the relation channel.
//!
//! Owner ruling, 2026-10-01 04:14 UTC: local SmolLM2-Instruct is used **offline as a
//! data source only, never serving**. It generates wordings of each relation's
//! statements and questions, **seeded with training phrasings only**. Any generated
//! wording that matches a development template, or shares a 4-word sequence with one,
//! is screened out, so the development cells stay held out.
//!
//! ```text
//! paraphrase-gen out=NEW_ROOT model=SMOLM2_DIR [conversations=2000] [items=200]
//!   [per_item=4] [seed=1] [temperature=0.8] [top_p=0.95] [max_tokens=48] [threads=1]
//! ```
//!
//! Steps: (1) draw relation user turns from the **training-phrasing** cells as seeds;
//! (2) draw the **development-phrasing** cells' user turns as the held-out screen;
//! (3) sample `per_item` wordings per seed with the teacher; (4) screen exact matches
//! and shared 4-word sequences; (5) write `paraphrase.jsonl` + `manifest.json` and
//! seal the root. The teacher is the flat-limit checkpoint itself (score=dot,
//! trainable=scalars, no learned curvature applied).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::kappa_llama::{load_checkpoint, KappaLlama, ScoreKind, Trainable};
use uor_r4_training::milestone_world::Split;
use uor_r4_training::milestone_world_v2::{relation_names, Cell, MWorld2, Mix};
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

const SYSTEM: &str = "You are a helpful AI assistant named SmolLM, trained by Hugging Face";
const FLAT_LOG_EPS: f32 = -12.0;

struct Args(std::collections::BTreeMap<String, String>);

impl Args {
    fn parse() -> Result<Self> {
        let mut pairs = std::collections::BTreeMap::new();
        for argument in std::env::args().skip(1) {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid("arguments are key=value"))?;
            pairs.insert(key.to_string(), value.to_string());
        }
        Ok(Self(pairs))
    }
    fn required(&self, key: &str) -> Result<&str> {
        self.0
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }
    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| invalid(format!("{key} is not a number"))),
        }
    }
    fn text(&self, key: &str, default: &str) -> String {
        self.0
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_owned())
    }
}

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

/// Lowercase alphanumeric word tokens, for the n-gram guard.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

fn strip_prompt_labels(text: &str) -> String {
    let mut rest = text.trim().to_owned();
    loop {
        rest = rest
            .trim_matches(|c: char| c == '"' || c == '\u{201c}' || c == '\u{201d}' || c == '\'')
            .trim()
            .to_owned();
        let lowered = rest.to_lowercase();
        let label = ["rewrite:", "rewritten:", "paraphrase:", "paraphrased:"]
            .iter()
            .find(|label| lowered.starts_with(**label));
        match label {
            Some(label) => rest = rest[label.len()..].to_owned(),
            None => return rest,
        }
    }
}

/// Every digit run and capitalized non-initial token of the seed must survive the
/// rewrite, and no new digit run or new capitalized multi-letter token may appear
/// (invented values); otherwise the wording cannot supervise the same relation.
fn preserves_values(seed: &str, wording: &str) -> bool {
    fn value_tokens(text: &str) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut digits = BTreeSet::new();
        let mut capitals = BTreeSet::new();
        for (index, raw) in text
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .enumerate()
        {
            if raw.chars().all(|c| c.is_ascii_digit()) {
                digits.insert(raw.to_lowercase());
            } else if index > 0
                && raw.len() > 1
                && raw.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            {
                capitals.insert(raw.to_lowercase());
            }
        }
        (digits, capitals)
    }
    let (seed_digits, seed_caps) = value_tokens(seed);
    let (wording_digits, wording_caps) = value_tokens(wording);
    seed_digits.is_subset(&wording_digits)
        && wording_digits.is_subset(&seed_digits)
        && seed_caps.is_subset(&wording_caps)
        && wording_caps.is_subset(&seed_caps)
}

/// A rewrite must not ask about someone else: reject a newly introduced
/// second-person address that drops the seed's first-person reference.
fn flips_person(seed: &str, wording: &str) -> bool {
    fn count(text: &str, markers: &[&str]) -> usize {
        words(text)
            .iter()
            .filter(|word| markers.contains(&word.as_str()))
            .count()
    }
    const FIRST: [&str; 5] = ["i", "me", "my", "mine", "myself"];
    const SECOND: [&str; 4] = ["you", "your", "yours", "yourself"];
    count(wording, &SECOND) > count(seed, &SECOND) && count(wording, &FIRST) < count(seed, &FIRST)
}

fn ngram_set(text: &str, n: usize) -> BTreeSet<Vec<String>> {
    let w = words(text);
    if w.len() < n {
        return BTreeSet::new();
    }
    w.windows(n).map(|window| window.to_vec()).collect()
}

/// Maps a typed intent to the relation and act the E3 compiler assigns.
fn relation_act(intent: &str) -> Option<(String, &'static str)> {
    for name in relation_names() {
        if let Some(act) = intent
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('_'))
        {
            let act = match act {
                "assert" => "assert",
                "update" => "update",
                "query" | "absent" => "query",
                _ => continue,
            };
            return Some((name.to_owned(), act));
        }
    }
    None
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    let out = PathBuf::from(args.required("out")?);
    let model_dir = PathBuf::from(args.required("model")?);
    let conversations: usize = args.number("conversations", 2_000)?;
    let items: usize = args.number("items", 200)?;
    let per_item: usize = args.number("per_item", 4)?;
    let seed: u64 = args.number("seed", 1)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_p: f64 = args.number("top_p", 0.95)?;
    let max_tokens: usize = args.number("max_tokens", 48)?;

    // Claim before any model work; never reuse a root.
    report_output::claim(&out)?;

    let device = Device::Cpu;
    let tokenizer = HfBpeTokenizer::from_dir(&model_dir)
        .map_err(|error| invalid(format!("tokenizer: {error}")))?;
    let count = |text: &str| tokenizer.encode(text).len();

    // (1)+(2) seeds from training phrasings; held-out screen from development phrasings.
    let mut world = MWorld2::new(&count, Mix::default())?;
    let mut rng = Rng::new(seed);
    let mut seeds: Vec<(String, String)> = Vec::new();
    let mut seen_texts: BTreeSet<String> = BTreeSet::new();
    let train_cells = [
        Cell::new(Split::Train, Split::Train),
        Cell::new(Split::Train, Split::Development),
    ];
    for cell in train_cells {
        for _ in 0..conversations {
            let conversation = world.conversation_in(&mut rng, cell)?;
            for turn in &conversation.turns {
                if relation_act(&turn.intent).is_some() && !turn.user.trim().is_empty() {
                    let text = turn.user.trim().to_owned();
                    if seen_texts.insert(words(&text).join(" ")) {
                        seeds.push((turn.intent.clone(), text));
                    }
                }
            }
        }
    }
    seeds.truncate(items);

    let dev_cells = [
        Cell::new(Split::Development, Split::Development),
        Cell::new(Split::Development, Split::Train),
    ];
    let mut held_out_texts: Vec<String> = Vec::new();
    let mut held_out_ngrams: BTreeSet<Vec<String>> = BTreeSet::new();
    for cell in dev_cells {
        for _ in 0..conversations {
            let conversation = world.conversation_in(&mut rng, cell)?;
            for turn in &conversation.turns {
                if turn.user.trim().is_empty() {
                    continue;
                }
                held_out_ngrams.extend(ngram_set(&turn.user, 4));
                held_out_texts.push(words(&turn.user).join(" "));
            }
        }
    }
    let held_out_texts: BTreeSet<String> = held_out_texts.into_iter().collect();

    // (3) teacher sampling: the flat-limit checkpoint itself.
    let checkpoint = load_checkpoint(&model_dir, &device)?;
    let weights_sha = sha256_file(&model_dir.join("model.safetensors"))?;
    let mut model = KappaLlama::new(
        checkpoint,
        ScoreKind::Dot,
        FLAT_LOG_EPS,
        Trainable::Scalars,
        &device,
    )?;
    let stop = tokenizer.encode("<|im_end|>");
    let mut sampler = Rng::new(seed ^ 0x9E37_79B9_7F4A_7C15);

    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut accepted_seen: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let (mut accepted, mut rejected_exact, mut rejected_ngram) = (0usize, 0usize, 0usize);
    for (intent, seed_text) in &seeds {
        for draw in 0..per_item {
            let ask = format!(
                "Paraphrase the example sentence below, keeping every name, number and code word \
                 exactly as written. Do not answer it and do not reply to it; rewrite it in \
                 different words. Reply with the rewrite only.\n\nExample sentence: \"{seed_text}\"\nRewrite:"
            );
            let chat = format!(
                "<|im_start|>system\n{SYSTEM}<|im_end|>\n<|im_start|>user\n{ask}<|im_end|>\n<|im_start|>assistant\n"
            );
            let mut ids = tokenizer.encode(&chat);
            let prompt_len = ids.len();
            for _ in 0..max_tokens {
                let logits = model.forward(&ids, 1, ids.len(), true)?;
                let last = logits.narrow(1, ids.len() - 1, 1)?.flatten_all()?;
                let values = last.to_vec1::<f32>()?;
                let next = sample(&values, temperature, top_p, &mut sampler)?;
                ids.push(next);
                if stop.len() == 1 && next == stop[0] {
                    break;
                }
            }
            // Trim the stop token (and anything after it) from the decoded wording.
            let generated: Vec<u32> = ids[prompt_len..].to_vec();
            let generated = match stop.first() {
                Some(stop_id) => match generated.iter().position(|token| token == stop_id) {
                    Some(position) => generated[..position].to_vec(),
                    None => generated,
                },
                None => generated,
            };
            let wording = strip_prompt_labels(&tokenizer.decode(&generated));
            let normalized = words(&wording).join(" ");
            let rejection = if normalized.is_empty() {
                Some("empty")
            } else if normalized == words(seed_text).join(" ") {
                Some("copied_seed")
            } else if seed_text.trim_end().ends_with('?') != wording.trim_end().ends_with('?') {
                Some("kind_changed")
            } else if flips_person(seed_text, &wording) {
                Some("person_flipped")
            } else if held_out_texts.contains(&normalized) {
                Some("development_template_exact")
            } else if !ngram_set(&wording, 4).is_disjoint(&held_out_ngrams) {
                Some("development_ngram4_overlap")
            } else if !preserves_values(seed_text, &wording) {
                Some("value_changed")
            } else if accepted_seen
                .get(seed_text)
                .is_some_and(|wordings| wordings.contains(&normalized))
            {
                Some("duplicate_wording")
            } else {
                None
            };
            match rejection {
                None => {
                    accepted_seen
                        .entry(seed_text.clone())
                        .or_default()
                        .insert(normalized.clone());
                    accepted += 1;
                }
                Some("development_template_exact") => rejected_exact += 1,
                Some("development_ngram4_overlap") => rejected_ngram += 1,
                Some(_) => {}
            }
            rows.push(serde_json::json!({
                "intent": intent,
                "seed": seed_text,
                "draw": draw,
                "wording": wording,
                "accepted": rejection.is_none(),
                "reject_reason": rejection,
            }));
        }
    }

    // (4)+(5) write the artifact and seal.
    let mut jsonl = String::new();
    for row in &rows {
        jsonl.push_str(&serde_json::to_string(row)?);
        jsonl.push('\n');
    }
    std::fs::write(out.join("paraphrase.jsonl"), jsonl)?;
    let mut paraphrases = String::new();
    let (mut consumer_rows, mut unmapped, mut duplicates) = (0usize, 0usize, 0usize);
    let mut consumer_seen: BTreeSet<(String, &'static str, String)> = BTreeSet::new();
    for row in &rows {
        if row["accepted"].as_bool() != Some(true) {
            continue;
        }
        let Some((relation, act)) = relation_act(row["intent"].as_str().unwrap_or_default()) else {
            unmapped += 1;
            continue;
        };
        let wording = row["wording"].as_str().unwrap_or_default();
        if !consumer_seen.insert((relation.clone(), act, words(wording).join(" "))) {
            duplicates += 1;
            continue;
        }
        paraphrases.push_str(&serde_json::to_string(&serde_json::json!({
            "relation": relation,
            "act": act,
            "text": wording,
        }))?);
        paraphrases.push('\n');
        consumer_rows += 1;
    }
    std::fs::write(out.join("paraphrases.jsonl"), paraphrases)?;
    let manifest = serde_json::json!({
        "schema": "uor-r4.paraphrase-gen/1",
        "ruling": "owner 2026-10-01T04:14Z: teacher paraphrase data for the relation channel",
        "teacher": {
            "path": model_dir.display().to_string(),
            "model_sha256": weights_sha,
            "role": "offline data source only; never serving",
            "path_used": "flat-limit checkpoint (dot, trainable=scalars)",
        },
        "seed_plans": {
            "cells": ["train_phrasing_train_value", "train_phrasing_dev_value"],
            "conversations": conversations,
            "items": seeds.len(),
            "per_item": per_item,
            "seed": seed,
        },
        "screen_plans": {
            "cells": ["dev_phrasing_dev_value", "dev_phrasing_train_value"],
            "held_out_user_turns": held_out_texts.len(),
            "guard": "reject exact normalized match, or any shared 4-word sequence",
        },
        "sampling": { "temperature": temperature, "top_p": top_p, "max_tokens": max_tokens },
        "consumer": {
            "file": "paraphrases.jsonl",
            "schema": "relation, act, text for relation_compiler::paraphrase_examples",
            "rows": consumer_rows,
            "skipped_unmapped": unmapped,
            "skipped_duplicate": duplicates,
        },
        "counts": {
            "rows": rows.len(),
            "accepted": accepted,
            "rejected_exact": rejected_exact,
            "rejected_ngram4": rejected_ngram,
        },
        "provenance": { "executable_sha256": sha256_file(&std::env::current_exe()?)? },
    });
    std::fs::write(
        out.join("paraphrase-manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    report_output::seal(&out)?;
    let unlisted = report_output::verify(&out)?;
    if !unlisted.is_empty() {
        return Err(invalid(format!("unlisted files: {unlisted:?}")));
    }
    println!(
        "{} rows ({} accepted, {} exact-rejected, {} ngram-rejected) from {} seeds; {} consumer rows",
        rows.len(),
        accepted,
        rejected_exact,
        rejected_ngram,
        seeds.len(),
        consumer_rows
    );
    Ok(())
}

/// Temperature + nucleus sampling over one logits row.
fn sample(logits: &[f32], temperature: f64, top_p: f64, rng: &mut Rng) -> Result<u32> {
    if logits.is_empty() || !temperature.is_finite() || temperature <= 0.0 {
        return Err(invalid("sampler needs logits and a positive temperature"));
    }
    if !top_p.is_finite() || top_p <= 0.0 || top_p > 1.0 {
        return Err(invalid("sampler needs top_p in (0, 1]"));
    }
    let mut ranked: Vec<(u32, f64)> = logits
        .iter()
        .enumerate()
        .map(|(index, value)| (index as u32, f64::from(*value) / temperature))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let maximum = ranked[0].1;
    let weights: Vec<(u32, f64)> = ranked
        .into_iter()
        .map(|(index, score)| (index, (score - maximum).exp()))
        .collect();
    let mass: f64 = weights.iter().map(|(_, weight)| weight).sum();
    let mut total = 0.0f64;
    let mut probs: Vec<(u32, f64)> = Vec::with_capacity(weights.len());
    for (index, weight) in weights {
        total += weight;
        probs.push((index, weight));
        if total >= top_p * mass {
            break;
        }
    }
    let threshold = (rng.next_u64() as f64 / u64::MAX as f64) * total;
    let mut running = 0.0f64;
    for (index, weight) in &probs {
        running += weight;
        if running >= threshold {
            return Ok(*index);
        }
    }
    Ok(probs.last().expect("nonempty nucleus").0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nucleus_keeps_both_leading_tokens_and_validates_top_p() -> Result<()> {
        let logits = [2.0f32, 1.9, 1.8, -12.0];
        let mut rng = Rng::new(11);
        let (mut first, mut second) = (false, false);
        for _ in 0..64 {
            let index = sample(&logits, 1.0, 0.9, &mut rng)?;
            first |= index == 0;
            second |= index == 1;
        }
        assert!(first && second);
        assert!(sample(&logits, 1.0, 0.0, &mut rng).is_err());
        assert!(sample(&logits, 1.0, 1.5, &mut rng).is_err());
        Ok(())
    }

    #[test]
    fn person_flip_needs_a_new_second_person_and_a_lost_first_person() {
        assert!(flips_person("Where do I live?", "Where do you live?"));
        assert!(flips_person(
            "What's my friend called?",
            "What is the name of your friend?"
        ));
        assert!(!flips_person(
            "Do you remember my lucky number?",
            "Can you recall my number?"
        ));
        assert!(!flips_person(
            "What's my friend called?",
            "What is the name of my friend?"
        ));
    }

    #[test]
    fn prompt_labels_are_stripped_from_wordings() {
        assert_eq!(
            strip_prompt_labels("\"Rewrite: I am from Dulseestfield.\""),
            "I am from Dulseestfield."
        );
        assert_eq!(
            strip_prompt_labels("Paraphrased: Where do I live?"),
            "Where do I live?"
        );
        assert_eq!(
            strip_prompt_labels("  What is my name?  "),
            "What is my name?"
        );
    }

    #[test]
    fn intents_map_to_the_compilers_relation_acts() {
        assert_eq!(
            relation_act("friend_name_absent"),
            Some(("friend_name".to_owned(), "query"))
        );
        assert_eq!(
            relation_act("home_assert"),
            Some(("home".to_owned(), "assert"))
        );
        assert_eq!(
            relation_act("user_name_update"),
            Some(("user_name".to_owned(), "update"))
        );
        assert_eq!(relation_act("none"), None);
    }
}
