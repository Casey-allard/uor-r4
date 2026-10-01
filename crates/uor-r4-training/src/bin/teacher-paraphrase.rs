//! Teacher paraphrases of M-world v2's relation templates, for the relation
//! compiler (E3 of the log-sieve design; owner ruling on #1552).
//!
//! ```text
//! teacher-paraphrase out=NEW_REPORT_ROOT teacher=SMOLLM2_INSTRUCT_DIR world_tokenizer=T.json \
//!   [per_template=8] [max_new=160] [temperature=0.8] [top_k=40] [seed=1] [draws=3000] \
//!   [probe=false]
//! ```
//!
//! A local SmolLM2-Instruct checkpoint is an **offline data source only**; it
//! never serves. For every relation template the world draws with *training*
//! phrasings (relation, act, template with its `{v}` slot), the teacher is
//! asked for `per_template` other wordings, sampled with a fixed seed. Each
//! line is cleaned, must keep the slot exactly when the template has one, and
//! is **screened out** if it equals a development template or shares a
//! four-word sequence with one, so the development cells stay held out. The
//! kept wordings go to `paraphrases.jsonl`; the report root is claimed first
//! and sealed at the end. `probe=true` generates for one template only and
//! reports the speed.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_model_source::{HuggingFaceLlamaOracle, State, TeacherExecutionConfig};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::{normalized, Split};
use uor_r4_training::milestone_world_v2::{Cell, MWorld2, Mix};
use uor_r4_training::relation_compiler::{label, NONE};
use uor_r4_training::sha256_file;
use uor_r4_training::stack_tracking::Rng;

type Error = Box<dyn std::error::Error>;

const IM_START: u32 = 1;
const IM_END: u32 = 2;
const SLOT: &str = "{v}";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("teacher-paraphrase: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse() -> Result<Self, Error> {
        const KEYS: [&str; 12] = [
            "out",
            "teacher",
            "world_tokenizer",
            "per_template",
            "max_new",
            "temperature",
            "top_k",
            "seed",
            "draws",
            "probe",
            "workers",
            "probe_template",
        ];
        let mut pairs = BTreeMap::new();
        for argument in std::env::args().skip(1) {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got {argument}"))?;
            if !KEYS.contains(&key) || pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("unknown or repeated argument {key}").into());
            }
        }
        Ok(Self(pairs))
    }

    fn path(&self, key: &str) -> Result<PathBuf, Error> {
        Ok(PathBuf::from(
            self.0.get(key).ok_or_else(|| format!("missing {key}="))?,
        ))
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, Error> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| format!("invalid {key}={value}").into()),
        }
    }
}

/// A relation template of the world: its relation, act and text.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Template {
    relation: String,
    act: &'static str,
    text: String,
}

/// The relation templates the world draws in `phrasing`, over every value split.
fn templates(
    count: &dyn Fn(&str) -> usize,
    phrasing: Split,
    draws: usize,
    seed: u64,
) -> Result<BTreeSet<Template>, Error> {
    let mix = Mix {
        mqar: 0.0,
        copy: 0.0,
        relation: 1.0,
        other: 0.0,
        ..Mix::default()
    };
    let mut found = BTreeSet::new();
    for value in [Split::Train, Split::Development] {
        let mut world = MWorld2::new(count, mix)?;
        let mut rng = Rng::new(seed);
        for _ in 0..draws {
            for turn in world
                .conversation_in(&mut rng, Cell::new(phrasing, value))?
                .turns
            {
                let (relation, act) = label(&turn);
                if relation == NONE {
                    continue;
                }
                if let Some(text) = turn.tag.template {
                    found.insert(Template {
                        relation,
                        act,
                        text,
                    });
                }
            }
        }
    }
    Ok(found)
}

/// The words of a template with the slot as one word.
fn slot_words(text: &str) -> Vec<String> {
    normalized(&text.replace(SLOT, " slotvalue "))
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

fn four_grams(text: &str) -> BTreeSet<Vec<String>> {
    let words = slot_words(text);
    words.windows(4).map(<[String]>::to_vec).collect()
}

/// The worked example shown before each request, on a topic that is no
/// M-world relation (a car), so it teaches the format without suggesting
/// any development wording.
fn example(act: &str) -> (&'static str, &'static [&'static str]) {
    match act {
        "assert" => (
            "My car is a {v}.",
            &[
                "I drive a {v}.",
                "The car I own is a {v}.",
                "I have a {v} as my car.",
                "{v} is the car I drive.",
            ],
        ),
        "update" => (
            "Actually, my car is a {v} now.",
            &[
                "I changed cars, it's a {v} now.",
                "Correction: I drive a {v} these days.",
                "These days my car is a {v}.",
                "I switched to a {v}.",
            ],
        ),
        _ => (
            "What car do I drive?",
            &[
                "Which car do I have?",
                "What kind of car do I own?",
                "What's my car again?",
                "Do you know what car I drive?",
            ],
        ),
    }
}

/// The request, split where the template enters: a prefix shared by every
/// template of one act and slot shape (its teacher state is computed once),
/// and the template's own suffix.
fn prompt(template: &Template, per_template: usize) -> (String, String) {
    let (sample, rewrites) = example(template.act);
    let slot_rule = if template.text.contains(SLOT) {
        format!(" Keep {SLOT} exactly as written, once in each line.")
    } else {
        String::new()
    };
    (
        format!(
            "Rewrite the sentence in {per_template} different ways that mean the same thing, in \
             everyday English.{slot_rule} Write one per line.\n\nSentence: {sample}\nRewrites:\n\
             {}\n\nSentence: ",
            rewrites.join("\n")
        ),
        format!("{}\nRewrites:", template.text),
    )
}

/// SmolLM2-Instruct's chat format up to the request's prefix: its default
/// system turn, then the user turn's opening.
fn prefix_ids(tokenizer: &ByteBpeTokenizer, prefix: &str) -> Vec<u32> {
    let mut ids = vec![IM_START];
    ids.extend(
        tokenizer
            .encode("system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face"),
    );
    ids.push(IM_END);
    ids.extend(tokenizer.encode("\n"));
    ids.push(IM_START);
    ids.extend(tokenizer.encode("user\n"));
    ids.extend(tokenizer.encode(prefix));
    ids
}

/// The rest of the user turn, then the assistant's opening.
fn suffix_ids(tokenizer: &ByteBpeTokenizer, suffix: &str) -> Vec<u32> {
    let mut ids = tokenizer.encode(suffix);
    ids.push(IM_END);
    ids.extend(tokenizer.encode("\n"));
    ids.push(IM_START);
    ids.extend(tokenizer.encode("assistant\n"));
    ids
}

/// A teacher state after a shared prefix, with its logits and length.
#[derive(Clone)]
struct Prefix {
    state: State,
    logits: Vec<f32>,
    len: usize,
}

/// Room left after a prefix for a template's suffix and the reply.
const SUFFIX_ROOM: usize = 96;

fn run_prefix(
    oracle: &HuggingFaceLlamaOracle,
    ids: &[u32],
    max_new: usize,
) -> Result<Prefix, Error> {
    let mut state = oracle
        .new_state_bounded(ids.len() + SUFFIX_ROOM + max_new)
        .map_err(|e| format!("teacher state: {e:?}"))?;
    let mut logits = vec![0f32; oracle.cfg().vocab];
    for (position, &id) in ids.iter().enumerate() {
        oracle
            .step_state(&mut state, id as usize, position, &mut logits)
            .map_err(|e| format!("teacher step: {e:?}"))?;
    }
    Ok(Prefix {
        state,
        logits,
        len: ids.len(),
    })
}

/// Whether a reply has finished its first block of rewrites.
fn finished(text: &str) -> bool {
    let body = text.trim_start();
    body.contains("\n\n") || body.contains("\nSentence") || body.contains("\nRewrite")
}

/// The first block of a reply's lines: up to a blank line, or a line that
/// opens a new example.
fn first_block(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    for line in text.trim_start().lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Sentence") || trimmed.starts_with("Rewrite") {
            break;
        }
        lines.push(line);
    }
    lines
}

struct Sampler(u64);

impl Sampler {
    fn uniform(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Sample from the `top_k` largest logits at `temperature`.
    fn sample(&mut self, logits: &[f32], temperature: f64, top_k: usize) -> u32 {
        let mut order: Vec<usize> = (0..logits.len()).collect();
        order.sort_by(|&a, &b| logits[b].total_cmp(&logits[a]).then(a.cmp(&b)));
        order.truncate(top_k.max(1));
        let top = f64::from(logits[order[0]]);
        let weights: Vec<f64> = order
            .iter()
            .map(|&i| ((f64::from(logits[i]) - top) / temperature.max(1e-6)).exp())
            .collect();
        let total: f64 = weights.iter().sum();
        let mut u = self.uniform() * total;
        for (&i, w) in order.iter().zip(&weights) {
            if u < *w {
                return i as u32;
            }
            u -= w;
        }
        order[order.len() - 1] as u32
    }
}

/// The teacher's reply after `prefix` and `suffix`: sampled ids up to
/// `max_new`, `<|im_end|>`, or the end of the first block of rewrites.
/// Returns the reply and the teacher steps it took (suffix and reply).
#[allow(clippy::too_many_arguments)]
fn generate(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &ByteBpeTokenizer,
    prefix: &Prefix,
    suffix: &[u32],
    max_new: usize,
    temperature: f64,
    top_k: usize,
    sampler: &mut Sampler,
) -> Result<(Vec<u32>, usize), Error> {
    if suffix.len() + max_new > SUFFIX_ROOM + max_new {
        return Err("a template's suffix does not fit the cached prefix's room".into());
    }
    let mut state = prefix.state.clone();
    let mut logits = prefix.logits.clone();
    let mut position = prefix.len;
    for &id in suffix {
        oracle
            .step_state(&mut state, id as usize, position, &mut logits)
            .map_err(|e| format!("teacher step: {e:?}"))?;
        position += 1;
    }
    let mut out = Vec::new();
    for _ in 0..max_new {
        let next = sampler.sample(&logits, temperature, top_k);
        if next == IM_END {
            break;
        }
        out.push(next);
        if finished(&tokenizer.decode(&out)) {
            break;
        }
        oracle
            .step_state(&mut state, next as usize, position, &mut logits)
            .map_err(|e| format!("teacher step: {e:?}"))?;
        position += 1;
    }
    Ok((out, position - prefix.len))
}

/// Clean one generated line: strip numbering, bullets and quotes.
fn clean(line: &str) -> String {
    let mut text = line.trim();
    let numbered = text
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i + 1);
    if let Some(end) = numbered {
        if let Some(rest) = text[end..].strip_prefix(['.', ')']) {
            text = rest.trim();
        }
    }
    text = text.trim_start_matches(['-', '*', '•']).trim();
    text.trim_matches(['"', '\u{201c}', '\u{201d}', '\''])
        .trim()
        .to_owned()
}

fn run() -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse()?;
    let out = args.path("out")?;
    let teacher_dir = args.path("teacher")?;
    let world_tokenizer_path = args.path("world_tokenizer")?;
    let per_template: usize = args.number("per_template", 8)?;
    let max_new: usize = args.number("max_new", 160)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_k: usize = args.number("top_k", 40)?;
    let seed: u64 = args.number("seed", 1)?;
    let draws: usize = args.number("draws", 3_000)?;
    let probe: bool = args.number("probe", false)?;
    let workers: usize = args.number("workers", 4)?;
    let probe_template: usize = args.number("probe_template", 0)?;
    if per_template == 0 || max_new == 0 || draws == 0 || top_k == 0 || temperature <= 0.0 {
        return Err("per_template, max_new, draws, top_k and temperature must be positive".into());
    }
    let workers = NonZeroUsize::new(workers).ok_or("workers must be positive")?;
    report_output::claim(&out)?;
    let result = generate_all(
        &out,
        &teacher_dir,
        &world_tokenizer_path,
        workers,
        probe_template,
        (
            per_template,
            max_new,
            temperature,
            top_k,
            seed,
            draws,
            probe,
        ),
        started,
    );
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

#[allow(clippy::type_complexity)]
fn generate_all(
    out: &Path,
    teacher_dir: &Path,
    world_tokenizer_path: &Path,
    workers: NonZeroUsize,
    probe_template: usize,
    (per_template, max_new, temperature, top_k, seed, draws, probe): (
        usize,
        usize,
        f64,
        usize,
        u64,
        usize,
        bool,
    ),
    started: Instant,
) -> Result<(), Error> {
    let world_tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(world_tokenizer_path)?)
            .ok_or("world_tokenizer is not a supported tokenizer.json")?;
    let count = |text: &str| world_tokenizer.encode(text).len();
    let train = templates(&count, Split::Train, draws, seed)?;
    let development = templates(&count, Split::Development, draws, seed)?;
    let dev_texts: BTreeSet<String> = development
        .iter()
        .map(|t| slot_words(&t.text).join(" "))
        .collect();
    let dev_grams: BTreeSet<Vec<String>> = development
        .iter()
        .flat_map(|t| four_grams(&t.text))
        .collect();
    let train_texts: BTreeSet<String> = train
        .iter()
        .map(|t| slot_words(&t.text).join(" "))
        .collect();
    println!(
        "{} training and {} development relation templates",
        train.len(),
        development.len()
    );
    let tokenizer_path = teacher_dir.join("tokenizer.json");
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or("the teacher's tokenizer.json is not supported")?;
    let loading = Instant::now();
    let oracle = HuggingFaceLlamaOracle::load_with_execution(
        teacher_dir,
        TeacherExecutionConfig::fixed_workers(workers),
    )
    .map_err(|e| format!("teacher load: {e:?}"))?;
    let load_seconds = loading.elapsed().as_secs_f64();
    let mut sampler = Sampler(seed);
    let mut kept = Vec::new();
    let mut tallies: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let (mut generated_tokens, mut generation_seconds) = (0usize, 0f64);
    let todo: Vec<&Template> = if probe {
        train.iter().skip(probe_template).take(1).collect()
    } else {
        train.iter().collect()
    };
    // Teacher states after each distinct shared prefix, computed once.
    let mut prefixes: BTreeMap<String, Prefix> = BTreeMap::new();
    for template in todo {
        let (prefix_text, suffix_text) = prompt(template, per_template);
        let clock = Instant::now();
        if !prefixes.contains_key(&prefix_text) {
            let ids = prefix_ids(&tokenizer, &prefix_text);
            generated_tokens += ids.len();
            prefixes.insert(prefix_text.clone(), run_prefix(&oracle, &ids, max_new)?);
        }
        let prefix = &prefixes[&prefix_text];
        let suffix = suffix_ids(&tokenizer, &suffix_text);
        let (reply, steps) = generate(
            &oracle,
            &tokenizer,
            prefix,
            &suffix,
            max_new,
            temperature,
            top_k,
            &mut sampler,
        )?;
        generation_seconds += clock.elapsed().as_secs_f64();
        generated_tokens += steps;
        let text = tokenizer.decode(&reply);
        let key = format!("{}/{}", template.relation, template.act);
        let tally = tallies.entry(key).or_default();
        let mut seen = BTreeSet::new();
        for line in first_block(&text) {
            let line = clean(line);
            if line.is_empty() {
                continue;
            }
            tally.0 += 1;
            let has_slot = template.text.contains(SLOT);
            if line.matches(SLOT).count() != usize::from(has_slot) {
                continue;
            }
            let words = slot_words(&line).join(" ");
            if words.is_empty() || train_texts.contains(&words) || !seen.insert(words.clone()) {
                continue;
            }
            if dev_texts.contains(&words) || four_grams(&line).iter().any(|g| dev_grams.contains(g))
            {
                tally.1 += 1;
                continue;
            }
            tally.2 += 1;
            kept.push(json!({
                "relation": template.relation,
                "act": template.act,
                "text": line,
                "source_template": template.text,
            }));
        }
        tally.3 += 1;
        if probe {
            println!("template: {}\nreply:\n{text}", template.text);
        }
    }
    let mut jsonl = String::new();
    for row in &kept {
        jsonl.push_str(&serde_json::to_string(row)?);
        jsonl.push('\n');
    }
    fs::write(out.join("paraphrases.jsonl"), jsonl)?;
    let rate = generated_tokens as f64 / generation_seconds.max(1e-9);
    println!(
        "kept {} paraphrases; {generated_tokens} teacher tokens in {generation_seconds:.0} s ({rate:.1}/s); load {load_seconds:.0} s",
        kept.len()
    );
    let report = json!({
        "schema": "uor-r4.teacher-paraphrase/1",
        "role": "offline teacher as a data source only; it never serves",
        "teacher": {
            "path": teacher_dir.display().to_string(),
            "config_sha256": sha256_file(&teacher_dir.join("config.json"))?,
            "model_sha256": sha256_file(&teacher_dir.join("model.safetensors"))?,
            "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        },
        "world_tokenizer_sha256": sha256_file(world_tokenizer_path)?,
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "settings": {
            "per_template": per_template, "max_new": max_new, "temperature": temperature,
            "top_k": top_k, "seed": seed, "draws": draws, "probe": probe, "workers": workers.get(),
        },
        "templates": {"training": train.len(), "development_screen": development.len()},
        "screen": "a line must keep {v} exactly when its template has it; it is dropped if it equals a training template, repeats within its reply, equals a development template or shares a four-word sequence with one (words lowercased, punctuation removed, {v} as one word)",
        "per_relation_act": tallies
            .iter()
            .map(|(key, (lines, screened, kept, prompts))| (key.clone(), json!({
                "prompts": prompts, "lines": lines, "screened_development": screened, "kept": kept,
            })))
            .collect::<BTreeMap<_, Value>>(),
        "kept": kept.len(),
        "teacher_tokens": generated_tokens,
        "generation_seconds": generation_seconds,
        "load_seconds": load_seconds,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
