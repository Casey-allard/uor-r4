//! The scale ladder's chat-quality instrument (#820): a fixed panel of
//! everyday requests, replies from any saved stack model, and grades from a
//! local offline teacher.
//!
//! ```text
//! chat-grade extract out=PANEL.json heldout=SPLIT_DIR tokenizer=T.json \
//!   [count=48] [max_words=24] [seed=1]
//! chat-grade grade out=NEW_REPORT_ROOT model=ROOT/model tokenizer=T.json \
//!   requests=PANEL.json[,MORE.json] [protocol=2] [max_new_tokens=64] \
//!   [grader=qwen2.5:1.5b] [ollama_url=http://127.0.0.1:11434] [device=cpu|cuda|metal]
//! chat-grade reply out=NEW_REPORT_ROOT model=ROOT/model tokenizer=T.json \
//!   requests=PANEL.json[,MORE.json] [protocol=2] [max_new_tokens=64] \
//!   [copy_oracle=span checks=CHECKS.tsv] [device=cpu|cuda|metal]
//! chat-grade grade-replies out=NEW_REPORT_ROOT replies=REPLIES.json \
//!   [grader=qwen2.5:1.5b] [ollama_url=http://127.0.0.1:11434] [ill_posed=IDS.txt]
//! chat-grade check requests=PANEL.json[,MORE.json] tokenizer=T.json \
//!   [protocol=2] [context=384] [max_new_tokens=64] [exclude=IDS.txt]
//! chat-grade filter out=NEW_PANEL.json requests=PANEL.json[,MORE.json] \
//!   (exclude=IDS.txt | only=IDS.txt)
//! chat-grade tiers report=REPORT.json [ill_posed=IDS.txt] [out=NEW_REPORT_ROOT]
//! chat-grade compare a=REPORT.json b=REPORT.json [tier=all|C|K-clean|...] \
//!   [ill_posed=IDS.txt] [checks=CHECKS.tsv] [out=NEW_REPORT_ROOT]
//! chat-grade leak requests=PANEL.json[,MORE.json] reference=FILE[,FILE] \
//!   [strict=FILE[,FILE]] [whole_only=FILE[,FILE]] [corpora=DIR[,DIR] tokenizer=T.json] \
//!   [n=6] [strict_n=4] [corpus_n=8] [out=NEW_REPORT_ROOT]
//! chat-grade ill-posed requests=PANEL.json[,MORE.json] [expect=IDS.txt] \
//!   [out_tsv=NEW.tsv out_ids=NEW_IDS.txt out_clean=NEW_CLEAN.txt]
//! ```
//!
//! `grade` and `reply` also take `exclude=IDS.txt` (drop those request ids
//! before answering); `grade`/`grade-replies` take `ill_posed=IDS.txt`,
//! `checks=CHECKS.tsv` and `constants=default|none|FILE`; `check` and `tiers`
//! take `checks=`.
//!
//! **Tiers.** Every graded row belongs to one evaluation tier, from its id and
//! category alone: `C` (the conversational panel, ids `conv-*`), `K-clean` and
//! `K-ill-posed` (category `heldout_first_turn`, split by the ill-posed id list:
//! requests that, read alone, refer to material they do not contain),
//! `stretch` (category `stretch_heldout`) and `everyday` (the everyday-32
//! categories). Any other category is an error. The ill-posed list defaults to
//! `data/panels/heldout-ill-posed-v3-ids.txt` (written by `ill-posed`, one
//! text-only rule, [`MISSING_MATERIAL_RULE`]), embedded in the executable;
//! `ill_posed=` overrides it. Graded reports append `per_tier` (actual,
//! controls and paired-against-control per tier, and per category within it)
//! and a `headline`: per tier, the categories where the model beats that
//! category's best constant reply ([`BEST_CONSTANT_RULE`]). A tier total is
//! never the headline. `tiers` recomputes that from an existing report's rows
//! without regrading; `compare` pairs two graded reports' rows by id within
//! each tier and gives the two-sided exact McNemar for acceptable, fluent and
//! relevant, per tier and per category.
//!
//! **Row checks.** The grader judges only whether a reply is fluent and
//! responds to the user's *last* message, so it cannot tell a recalled fact
//! from an invented one. A row may carry a frozen deterministic check
//! (`data/panels/conversational-v2-checks.tsv`, `-v3-checks.tsv` and
//! `-v4-checks.tsv`, embedded together; their ids are disjoint; `checks=`
//! overrides): `any` (the reply
//! contains one of the listed words or phrases), `abstain` (the reply says it
//! does not know or cannot), `question` (the reply asks a question), `exact`
//! (panel v3/v4 memory: the reply names the expected value, no forbidden value
//! and no word of the distractor's key -- a wrong value, both values, the
//! value bound to the wrong key or no value fails) or `abstain_exact` (panel
//! v3/v4 unknowable: an abstention that neither agrees, guesses, asserts after
//! "but" nor names a made-up specific; [`abstention_fault`]). A
//! checked row is acceptable only when it is fluent, relevant and passes its
//! check.
//!
//! **Controls.** Each reply is also graded against the next row's
//! conversation (the derangement: relevance must fall), and each row is
//! graded with every fixed constant reply in place of the model's (by default
//! three generic replies, `DEFAULT_CONSTANTS`): a category whose constant
//! control passes as often as the model measures nothing about the model.
//! The check-only controls (no grader) apply each row's check to its own last
//! user turn and to all its user turns joined, so a check that an echo passes
//! is visible; memory rows also get the first- and last-stated value (copy),
//! an authored wrong-binding reply (`conversational-v3-swaps.tsv`,
//! `-v4-swaps.tsv`) and each expected spelling as a bare reply, and
//! `abstain_exact` rows three fixed adversarial abstentions.
//!
//! `check` runs the panel's context/turn check against a tokenizer with no
//! model, validates the row checks against the panel and reports the
//! check-only and constant pass counts; `filter` writes a new panel file and
//! never edits its inputs; `leak` compares every user turn with reference
//! texts (string literals of Rust sources, string values of JSON, lines of
//! other files) and with the user lines of prepared corpora.
//!
//! `extract` decodes a prepared protocol-2 held-out chat split (documents from
//! BOS; the first `User:` turn up to the next newline) and keeps, by a seeded
//! shuffle, `count` single-turn requests of at most `max_words` words. The
//! held-out split was never trained on.
//!
//! `grade` answers every request greedily (`stack_dialogue::reply_panel`)
//! and asks the grader, a local Ollama model used as an offline judge only,
//! two yes/no questions about each reply against the conversation so far:
//! is it fluent, and does it respond sensibly to the user's last message.
//! The controls above are graded the same way. The report root is claimed
//! before the model loads and sealed at the end.
//!
//! `reply` writes the same greedy replies without a grader (`replies.json`:
//! the `reply_panel` record with each reply's generated ids, seconds and ids
//! per second), so a served integer artifact's replies (`geometric-stack
//! lut-chat`) can be compared with the float model's id for id.
//! `cycle_repeats=N` (`reply`; integer 2..=64, default 3) is how many times a
//! terminal cycle of 1..4 ids repeats before the reply stops; 3 is the
//! historical rule, and a larger N lets a code fence (the backtick id three
//! times) through. It is recorded in `replies.json` only when not 3.
//!
//! `device=` (`grade` and `reply`; default `cpu`, so a command without it is
//! unchanged) is where the stack model generates its replies; the grader and
//! everything else stay where they were. `cuda` needs a `--features cuda`
//! build (ordinal 0 of CUDA_VISIBLE_DEVICES), `metal` a `--features metal`
//! one; there is no fallback and TF32 is off. The device is opened before
//! the report root is claimed and recorded in the report. Greedy replies can
//! differ from the CPU's where float reduction order changes an argmax.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::sha256_file;

type Error = Box<dyn std::error::Error>;

/// One request of a reply panel: user turns answered in order, the format of
/// the retained study's request files.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    id: String,
    category: String,
    user_turns: Vec<String>,
}

const MAX_REQUESTS: usize = 128;
const MAX_USER_TURNS: usize = 8;
const MAX_NEW_TOKENS: usize = 128;

/// Read a request panel: 1 to [`MAX_REQUESTS`] requests with distinct ids,
/// each with 1 to [`MAX_USER_TURNS`] nonblank user turns.
fn load_requests(path: &Path) -> Result<Vec<Request>, Error> {
    let requests: Vec<Request> = serde_json::from_slice(&fs::read(path)?)?;
    let ids: BTreeSet<&str> = requests.iter().map(|r| r.id.as_str()).collect();
    if requests.is_empty()
        || requests.len() > MAX_REQUESTS
        || ids.len() != requests.len()
        || requests.iter().any(|r| {
            r.user_turns.is_empty()
                || r.user_turns.len() > MAX_USER_TURNS
                || r.user_turns.iter().any(|t| t.trim().is_empty())
        })
    {
        return Err(format!(
            "a request panel needs 1 to {MAX_REQUESTS} requests with distinct ids, each with 1 to \
             {MAX_USER_TURNS} nonblank user turns"
        )
        .into());
    }
    Ok(requests)
}

/// Check that the whole panel can be answered before anything is generated:
/// a positive cap of at most [`MAX_NEW_TOKENS`], every user turn one protocol
/// turn with no special tokens, and every request's history within `context`
/// even if each reply reaches the cap and is closed with EOS by the caller.
fn check_panel(
    encoder: &DialogueEncoder<'_>,
    requests: &[Request],
    context: usize,
    max_new_tokens: usize,
) -> Result<(), Error> {
    if max_new_tokens == 0 || max_new_tokens > MAX_NEW_TOKENS {
        return Err(format!("max_new_tokens must be 1 to {MAX_NEW_TOKENS}").into());
    }
    for request in requests {
        let mut longest = 1usize;
        for (turn, user) in request.user_turns.iter().enumerate() {
            let prefix = encoder.encode_user_prefix(user, turn != 0);
            if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                return Err(format!(
                    "request {}: turn {} is not one plain user turn",
                    request.id,
                    turn + 1
                )
                .into());
            }
            longest += prefix.tokens.len() + max_new_tokens;
            if longest > context {
                return Err(format!(
                    "request {}: with every reply at {max_new_tokens} ids the history reaches \
                     {longest} of {context} positions by turn {}",
                    request.id,
                    turn + 1
                )
                .into());
            }
            if turn + 1 < request.user_turns.len() {
                longest += 1;
            }
        }
    }
    Ok(())
}

/// SplitMix64: deterministic initialisation and data.
#[derive(Clone, Copy, Debug)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x6A09_E667_F3BC_C909)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound.max(1) as u64) as usize
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chat-grade: {error}");
            ExitCode::FAILURE
        }
    }
}

#[allow(dead_code)]
struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], keys: &[&str]) -> Result<Self, Error> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got {argument}"))?;
            if !keys.contains(&key) || pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("unknown or repeated argument {key}").into());
            }
        }
        Ok(Self(pairs))
    }

    #[allow(dead_code)]
    fn optional(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn required(&self, key: &str) -> Result<String, Error> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| format!("missing {key}=").into())
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, Error> {
        match self.0.get(key) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("invalid {key}={v}").into()),
        }
    }

    /// `device=cpu|cuda|metal` (default cpu): its name and the opened device.
    #[allow(dead_code)]
    fn device(&self) -> Result<(&'static str, Device), Error> {
        let name = device_name(self.0.get("device").map(String::as_str))?;
        let device = match name {
            "cpu" => Device::Cpu,
            "metal" => {
                #[cfg(feature = "metal")]
                {
                    Device::new_metal(0)?
                }
                #[cfg(not(feature = "metal"))]
                {
                    return Err("Metal requested without compiled Metal feature".into());
                }
            }
            "cuda" => {
                #[cfg(feature = "cuda")]
                {
                    Device::new_cuda(0)?
                }
                #[cfg(not(feature = "cuda"))]
                {
                    return Err("CUDA requested without compiled CUDA feature".into());
                }
            }
            _ => unreachable!(),
        };
        Ok((name, device))
    }
}

/// The name of `device=`, default `cpu`; anything else is refused.
#[allow(dead_code)]
fn device_name(given: Option<&str>) -> Result<&'static str, Error> {
    match given {
        None | Some("cpu") => Ok("cpu"),
        Some("cuda") => Ok("cuda"),
        Some("metal") => Ok("metal"),
        Some(other) => {
            Err(format!("unknown device={other} (cpu, cuda or metal; no implicit fallback)").into())
        }
    }
}

/// The report record of where the replies were generated.
#[allow(dead_code)]
fn device_record(name: &str) -> Value {
    json!({
        "device": name,
        "tf32": false,
        "cuda_visible_devices": std::env::var("CUDA_VISIBLE_DEVICES").ok(),
    })
}

fn run() -> Result<(), Error> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("extract") => extract(&arguments[1..]),
        Some("grade") => grade(&arguments[1..]),
        Some("reply") => reply(&arguments[1..]),
        Some("grade-replies") => grade_replies(&arguments[1..]),
        Some("check") => check(&arguments[1..]),
        Some("filter") => filter(&arguments[1..]),
        Some("tiers") => tiers(&arguments[1..]),
        Some("compare") => compare(&arguments[1..]),
        Some("leak") => leak(&arguments[1..]),
        Some("ill-posed") => ill_posed_command(&arguments[1..]),
        _ => Err(
            "usage: chat-grade extract|grade|reply|grade-replies|check|filter|tiers|compare|leak|\
             ill-posed key=value..."
                .into(),
        ),
    }
}

fn load_tokenizer(path: &Path) -> Result<ByteBpeTokenizer, Error> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
        .ok_or_else(|| format!("{} is not a supported tokenizer.json", path.display()).into())
}

/// The first user turn of a decoded protocol-2 document, if it has one.
fn first_user_turn(text: &str) -> Option<String> {
    let start = text.find("User:")? + "User:".len();
    let rest = &text[start..];
    let line = rest.split('\n').next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

fn extract(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(
        arguments,
        &["out", "heldout", "tokenizer", "count", "max_words", "seed"],
    )?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    let heldout = PathBuf::from(args.required("heldout")?);
    let tokenizer = load_tokenizer(Path::new(&args.required("tokenizer")?))?;
    let (count, max_words, seed): (usize, usize, u64) = (
        args.number("count", 48)?,
        args.number("max_words", 24)?,
        args.number("seed", 1)?,
    );
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
    let reader = MmapCorpusReader::open(heldout.join("tokens.u16"))?;
    let tokens = reader.as_slice();
    let mut candidates: Vec<String> = Vec::new();
    let mut start = None;
    for (i, &t) in tokens.iter().enumerate().chain([(tokens.len(), &0u16)]) {
        let boundary = i == tokens.len() || u32::from(t) == protocol.bos_id;
        if boundary {
            if let Some(s) = start {
                let ids: Vec<u32> = tokens[s..i]
                    .iter()
                    .map(|&id| u32::from(id))
                    .filter(|&id| id != protocol.eos_id)
                    .collect();
                if let Some(turn) = first_user_turn(&tokenizer.decode(&ids)) {
                    let words = turn.split_whitespace().count();
                    if (3..=max_words).contains(&words) && !candidates.contains(&turn) {
                        candidates.push(turn);
                    }
                }
            }
            start = (i < tokens.len()).then_some(i + 1);
        }
    }
    let available = candidates.len();
    let mut rng = Rng::new(seed);
    for i in (1..candidates.len()).rev() {
        candidates.swap(i, rng.below(i + 1));
    }
    candidates.truncate(count);
    let requests: Vec<Request> = candidates
        .into_iter()
        .enumerate()
        .map(|(i, turn)| Request {
            id: format!("heldout-{i:03}"),
            category: "heldout_first_turn".into(),
            user_turns: vec![turn],
        })
        .collect();
    fs::write(&out, serde_json::to_vec_pretty(&requests)?)?;
    println!(
        "{} requests from {available} eligible first turns; heldout tokens sha256 {}",
        requests.len(),
        sha256_file(&heldout.join("tokens.u16"))?
    );
    Ok(())
}

/// Default reason for panel rows that have no exact deterministic check under D28.
const BLOCKED_REASON: &str = "open_ended_no_exact_anchor";

/// Refuse any neural grader argument (`grader=` or `ollama_url=`) under D28.
fn refuse_neural_grader(args: &Args) -> Result<(), Error> {
    if args.0.contains_key("grader") || args.0.contains_key("ollama_url") {
        return Err(
            "D28: neural graders (grader= / ollama_url= / qwen2.5) are banned; score replies \
             deterministically with `chat-grade grade-replies` and exact row checks"
                .into(),
        );
    }
    Ok(())
}

/// Verdicts of a graded row. Under D28 deterministic grading (`deterministic = true`),
/// `fluent` and `relevant` are `None` and acceptability is determined strictly by the row's
/// deterministic check (`PASS` -> acceptable, `FAIL` or `BLOCKED` -> not acceptable).
struct Grades {
    deterministic: bool,
    fluent: Option<bool>,
    relevant: Option<bool>,
    #[allow(dead_code)]
    raw: [String; 2],
}

/// The verdict of a yes/no answer: its first word, `yes` or `no`.
#[allow(dead_code)]
fn parse_yes_no(answer: &str) -> Option<bool> {
    let first: String = answer
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase();
    match first.as_str() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

impl Grades {
    fn acceptable(&self) -> bool {
        self.fluent == Some(true) && self.relevant == Some(true)
    }
}

/// Paired verdicts of each request's actual reply and its derangement
/// control: the discordant counts and the two-sided exact McNemar test.
#[derive(Default)]
struct Paired {
    /// Actual yes, control no.
    actual_only: u64,
    /// Control yes, actual no.
    control_only: u64,
    both: u64,
    neither: u64,
}

impl Paired {
    fn add(&mut self, actual: bool, control: bool) {
        match (actual, control) {
            (true, false) => self.actual_only += 1,
            (false, true) => self.control_only += 1,
            (true, true) => self.both += 1,
            (false, false) => self.neither += 1,
        }
    }

    fn record(&self) -> Value {
        let p = mcnemar_exact(self.actual_only, self.control_only);
        json!({
            "actual_only": self.actual_only, "control_only": self.control_only,
            "both": self.both, "neither": self.neither,
            "mcnemar_exact_p": p,
            "discriminates": self.actual_only > self.control_only && p < 0.05,
        })
    }
}

/// Two-sided exact McNemar p-value for discordant counts `b` and `c`: twice
/// the binomial(b + c, 1/2) tail at min(b, c), capped at 1.
fn mcnemar_exact(b: u64, c: u64) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    // log C(n, i) - n log 2, summed in probability space.
    let mut log_choose = 0f64;
    let mut tail = 0f64;
    for i in 0..=k {
        if i > 0 {
            log_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (log_choose - n as f64 * std::f64::consts::LN_2).exp();
    }
    (2.0 * tail).min(1.0)
}

/// A row's acceptability: under deterministic D28 grading (`grades.deterministic`),
/// a row is acceptable iff it has a frozen deterministic check and passes it (`check == Some(true)`);
/// blocked (`None`) rows never pass. For legacy pre-D28 reports with fluent/relevant verdicts,
/// preserves the historical rule.
fn acceptable(grades: &Grades, check: Option<bool>) -> bool {
    if grades.deterministic {
        check == Some(true)
    } else {
        grades.acceptable() && check != Some(false)
    }
}

const ACCEPTABLE_RULE: &str = "under D28 deterministic grading, a row is PASS (acceptable) iff it \
     carries a frozen deterministic check and passes it; unchecked open-ended rows are BLOCKED \
     (open_ended_no_exact_anchor) and never counted as PASS";

#[derive(Default)]
struct Tally {
    replies: usize,
    fluent: usize,
    relevant: usize,
    fluent_and_relevant: usize,
    acceptable: usize,
    unparsed: usize,
    checked: usize,
    check_pass: usize,
}

impl Tally {
    fn add(&mut self, grades: &Grades, check: Option<bool>) {
        self.replies += 1;
        self.fluent += usize::from(grades.fluent == Some(true));
        self.relevant += usize::from(grades.relevant == Some(true));
        self.fluent_and_relevant += usize::from(grades.acceptable());
        self.acceptable += usize::from(acceptable(grades, check));
        if !grades.deterministic {
            self.unparsed +=
                usize::from(grades.fluent.is_none()) + usize::from(grades.relevant.is_none());
        }
        self.checked += usize::from(check.is_some());
        self.check_pass += usize::from(check == Some(true));
    }

    fn record(&self) -> Value {
        let check_fail = self.checked.saturating_sub(self.check_pass);
        let blocked_rows = self.replies.saturating_sub(self.checked);
        json!({
            "replies": self.replies, "fluent": self.fluent, "relevant": self.relevant,
            "fluent_and_relevant": self.fluent_and_relevant,
            "acceptable": self.acceptable, "unparsed_answers": self.unparsed,
            "checked_rows": self.checked, "check_pass": self.check_pass,
            "check_fail": check_fail, "blocked_rows": blocked_rows,
            "blocked_reason": (blocked_rows > 0).then_some(BLOCKED_REASON),
            "acceptable_rule": ACCEPTABLE_RULE,
        })
    }
}

/// Check passes of a reply with no grader (the check-only controls).
#[derive(Default)]
struct CheckTally {
    checked: usize,
    pass: usize,
}

impl CheckTally {
    fn add(&mut self, check: Option<bool>) {
        self.checked += usize::from(check.is_some());
        self.pass += usize::from(check == Some(true));
    }

    fn record(&self) -> Value {
        json!({"checked_rows": self.checked, "check_pass": self.pass})
    }
}

/// The three fixed replies graded in place of the model's on every row, unless
/// `constants=` replaces them. They are generic: an "I don't know" with a
/// clarifying question, a friendly acknowledgement, and a TinyStories opening.
const DEFAULT_CONSTANTS: [&str; 3] = [
    "I'm not sure. Can you tell me more about what you mean?",
    "That sounds nice! Thank you for telling me.",
    "Once upon a time, there was a little girl named Lily. She liked to play outside.",
];

/// The constant replies of a grading run and where they came from.
struct Constants {
    replies: Vec<String>,
    source: Value,
}

/// `constants=default` (or absent), `constants=none`, or a file with one reply
/// per line (blank lines and `#` comments ignored).
fn constants(args: &Args) -> Result<Constants, Error> {
    match args.0.get("constants").map(String::as_str) {
        None | Some("default") => Ok(Constants {
            replies: DEFAULT_CONSTANTS.iter().map(|s| (*s).to_owned()).collect(),
            source: json!("default"),
        }),
        Some("none") => Ok(Constants {
            replies: Vec::new(),
            source: json!("none"),
        }),
        Some(path) => {
            let bytes = fs::read(path)?;
            let replies: Vec<String> = std::str::from_utf8(&bytes)?
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(str::to_owned)
                .collect();
            if replies.is_empty() {
                return Err(format!("{path} holds no constant replies").into());
            }
            Ok(Constants {
                replies,
                source: json!({"path": path, "sha256": uor_r4_training::sha256_bytes(&bytes)}),
            })
        }
    }
}

impl Constants {
    fn record(&self) -> Value {
        json!({"source": self.source, "replies": self.replies})
    }
}

// ---------------------------------------------------------------------------
// Row checks.

/// Words of a text: lowercased, split at anything that is not a letter, digit
/// or apostrophe (a curly apostrophe counts as one), apostrophes trimmed from
/// the ends of each word.
fn words(text: &str) -> Vec<String> {
    text.replace('\u{2019}', "'")
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .map(|w| w.trim_matches('\''))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether `phrase` (as words) occurs as consecutive words of `text`.
fn contains_phrase(text: &[String], phrase: &[String]) -> bool {
    !phrase.is_empty() && text.windows(phrase.len()).any(|w| w == phrase)
}

/// Compact, sign-preserving text normalization for algebraic/numeric checks (`exact_text`):
/// lowercased, unicode minus/dashes normalized to `-`, superscript `²`/`³` normalized to
/// `^2`/`^3`, `√` normalized to `sqrt`, and all whitespace removed.
fn compact_math_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.to_lowercase().chars() {
        if c.is_whitespace() {
            continue;
        }
        match c {
            '\u{2212}' | '\u{2013}' | '\u{2014}' => out.push('-'),
            '\u{00b2}' => out.push_str("^2"),
            '\u{00b3}' => out.push_str("^3"),
            '\u{221a}' => out.push_str("sqrt"),
            _ => out.push(c),
        }
    }
    out
}

/// The frozen phrases an `abstain` check accepts: the reply says it does not
/// know, cannot, or was not told.
const ABSTAIN_PHRASES: &[&str] = &[
    "don't know",
    "dont know",
    "do not know",
    "didn't tell",
    "did not tell",
    "haven't told",
    "have not told",
    "not sure",
    "no way to know",
    "no way of knowing",
    "can't know",
    "cannot know",
    "can't",
    "cant",
    "cannot",
    "can not",
    "not able",
    "unable",
    "no idea",
    "i wish i could",
    "impossible",
];

/// Additional legitimate abstention phrases accepted in `v2` (`F5`) so polite
/// knowledge-boundary phrasings ("I'm sorry, I don't have that information.")
/// pass without loosening the fabricated-specific or answer-class checks.
const ABSTAIN_PHRASES_V2_EXTRA: &[&str] = &[
    "don't have that information",
    "dont have that information",
    "do not have that information",
    "don't have that",
    "dont have that",
    "do not have that",
    "don't have access",
    "dont have access",
    "do not have access",
    "haven't been told",
    "have not been told",
    "wasn't told",
    "was not told",
    "weren't told",
    "were not told",
    "not provided",
    "not specified",
    "not mentioned",
];

#[derive(Clone, Debug, PartialEq)]
enum CheckKind {
    /// The reply contains at least one of these words or phrases.
    Any(Vec<Vec<String>>),
    /// The reply contains one of [`ABSTAIN_PHRASES`].
    Abstain,
    /// The reply contains a question mark.
    Question,
    /// Exact recall (panel v3 memory rows): the reply contains one of the
    /// expected value's spellings and none of the row's forbidden values (the
    /// distractor values stated in the conversation and, for a closed class
    /// such as colours, days or small numbers, the class's other members). A
    /// wrong value, a hedge naming both values, or no value fails.
    Exact(Vec<Vec<String>>),
    /// [`CheckKind::Exact`] without the recall precondition, so a single-turn
    /// row can carry the same three components: the reply contains one of the
    /// expected spellings, none of the row's forbidden distractors, and none of
    /// the distractor's key words. Where `exact` asks "did the model recall the
    /// value stated in an earlier turn", this asks "does the reply contain the
    /// required content and not the wrong one" — presence *and* discrimination
    /// on a row whose request states neither.
    ReplyExact(Vec<Vec<String>>),
    /// Sign- and punctuation-preserving single-turn exact check (`v2` `F7` for
    /// algebraic/numeric rows such as `heldout-122` and `heldout-125`): matches
    /// [`compact_math_text`] substrings so `+`, `-`, `/`, `.` and `(` / `)` are
    /// preserved rather than stripped by [`words`].
    ExactText {
        terms: Vec<String>,
        forbid: Vec<String>,
    },
    /// A clarification of an ill-posed request: the reply asks a question (or,
    /// in `v2`, makes a polite imperative request) **about the material the
    /// request refers to but does not contain**, named by a row-specific phrase
    /// (`which city`, `what text`, `which words`). Asking a generic question is
    /// not enough — that would make a canned greeting pass every ill-posed row —
    /// and naming the missing material is the whole behaviour.
    Clarify(Vec<Vec<String>>),
    /// Abstention without a fabricated specific or agreement (panel v3
    /// unknowable rows): see [`abstention_fault`]; additionally none of the
    /// row's forbidden words (the answer class of the question, e.g. colours
    /// for "what colour is my shirt", or words that perform an impossible
    /// action, e.g. "here's a hug").
    AbstainExact,
    /// `v2` (`F5`) bare-salutation check (`Hi Dr. X,`): the user sent only a
    /// greeting/salutation with no body. A reply that greets/offers help or
    /// asks what the user needs, without fabricating specifics (`names a
    /// specific`) and without any forbidden continuation word, passes.
    Salutation,
    /// `v2` (`F6`) exact whole-reply choice (`heldout-092`: "Answer with one of
    /// the following options"): the reply's entire word sequence matches one of
    /// the listed options and contains no extra words.
    OneOf(Vec<Vec<String>>),
    /// `v2` (`F6`) exact word-count format check (`heldout-173`: "Create a story
    /// using only six words"): the reply has exactly `n` words, none of the
    /// row's `forbid` words, and at least one of the row's `keys` (if any).
    WordCount(usize),
}

impl CheckKind {
    fn name(&self) -> &'static str {
        match self {
            CheckKind::Any(_) => "any",
            CheckKind::Abstain => "abstain",
            CheckKind::Question => "question",
            CheckKind::Exact(_) => "exact",
            CheckKind::ReplyExact(_) => "reply_exact",
            CheckKind::ExactText { .. } => "exact_text",
            CheckKind::Clarify(_) => "clarify",
            CheckKind::AbstainExact => "abstain_exact",
            CheckKind::Salutation => "salutation",
            CheckKind::OneOf(_) => "one_of",
            CheckKind::WordCount(6) => "word_count_6",
            CheckKind::WordCount(_) => "word_count",
        }
    }
}

/// How a multi-turn row's last turn depends on the earlier ones.
#[derive(Clone, Copy, Debug, PartialEq)]
enum History {
    /// A single-turn row.
    None,
    /// An `any` term is stated in an earlier user turn, not in the last.
    Recall,
    /// The answer follows from earlier turns but is not stated in them (a
    /// count, a sum); no term is in the last turn.
    Derived,
    /// The last turn names nothing it refers to; the earlier turn sets the
    /// topic (`any` terms are topic words not in the last turn).
    Topic,
}

#[derive(Clone, Debug, PartialEq)]
struct RowCheck {
    kind: CheckKind,
    history: History,
    /// Words or phrases whose presence fails the check (`exact`, `reply_exact`,
    /// `clarify`, `abstain_exact`, `salutation` and `word_count`; empty
    /// otherwise). For `exact` these are values.
    forbid: Vec<Vec<String>>,
    /// `exact` / `reply_exact` / `clarify`: words of the distractor's *key*
    /// (the goldfish when the turtle is asked): a reply naming one binds the
    /// asked value to the wrong key ("The goldfish is Shelby") and fails.
    /// For `word_count`, `keys` holds the required story-anchor words.
    keys: Vec<Vec<String>>,
}

/// Words capitalised anywhere in a sentence that name nothing: the forms of
/// "I" and "OK".
const NOT_SPECIFIC: [&str; 6] = ["i", "i'm", "i'll", "i've", "i'd", "ok"];

/// The frozen words a sentence of an `abstain_exact` reply may begin with
/// when capitalised (pronouns, determiners, question words, auxiliaries,
/// conjunctions, interjections and a few sentence adverbs). Any other
/// capitalised sentence-initial word the user did not write counts as a
/// made-up name ("I'm not sure. Sam is the one." fails).
#[rustfmt::skip]
const SENTENCE_OPENERS: &[&str] = &[
    "a", "about", "actually", "after", "all", "also", "am", "an", "and", "any", "are", "as",
    "ask", "aw", "aww", "because", "but", "can", "could", "did", "do", "does", "even", "every",
    "for", "good", "great", "has", "have", "he", "hello", "her", "hey", "hi", "him", "his", "hm",
    "hmm", "how", "if", "in", "is", "it", "it's", "its", "just", "let", "let's", "maybe", "me",
    "my", "nice", "no", "nobody", "not", "now", "oh", "okay", "on", "one", "only", "oops", "or",
    "our", "perhaps", "please", "really", "sadly", "she", "should", "so", "some", "sometimes",
    "sorry", "still", "sure", "thank", "thanks", "that", "that's", "the", "their", "them",
    "then", "there", "there's", "these", "they", "they're", "this", "those", "though", "to",
    "try", "uh", "um", "unfortunately", "was", "we", "we're", "well", "what", "what's", "when",
    "where", "which", "who", "why", "will", "with", "without", "would", "wow", "yeah", "yes",
    "you", "you're", "your",
];

/// Additional sentence openers accepted in `v2` (`F5`) so legitimate sentence-initial
/// words ("Don't know, sorry.", "Nope, I don't know.", "Honestly, I don't know.") are
/// not misclassified as fabricated proper nouns. Kept in a separate slice so the
/// novelty scripts' parser on `SENTENCE_OPENERS` sees the exact `v1` block.
const SENTENCE_OPENERS_V2_EXTRA: &[&str] = &[
    "alas",
    "apologies",
    "don't",
    "dont",
    "forgive",
    "frankly",
    "honestly",
    "i'm",
    "im",
    "nope",
    "pardon",
    "regrettably",
    "truthfully",
    "uh-oh",
];

/// After one of these words an inability phrase is eagerness, not inability
/// ("I can't wait to play!").
const NOT_INABILITY_AFTER: [&str; 1] = ["wait"];

/// Phrases that agree to, perform or cheer on the request: an `abstain_exact`
/// reply containing one fails ("Yes! I can't wait to play with you!"). "sure"
/// counts too unless the word before it is "not".
const AFFIRM_PHRASES: &[&str] = &[
    "yes",
    "yeah",
    "yep",
    "yay",
    "of course",
    "let's",
    "lets",
    "i'd love to",
    "i would love to",
    "here you go",
    "there you go",
    "here it is",
    "no problem",
    "sounds fun",
    "sounds good",
    "nothing is impossible",
    "i can do that",
    "i can do it",
];

/// Phrases that mark a guess. A sentence of an `abstain_exact` reply that
/// contains one fails unless it is a question or invites the user to supply
/// the answer ([`INVITE_PHRASES`]): "Maybe you could ask your mom." passes,
/// "Maybe they are under the bed." fails.
const GUESS_MARKERS: &[&str] = &[
    "maybe",
    "probably",
    "perhaps",
    "i think",
    "i guess",
    "i bet",
    "my guess",
    "might be",
    "must be",
    "could be",
    "likely",
    "i believe",
];

/// Phrases that hand the question back to the user.
const INVITE_PHRASES: &[&str] = &["tell me", "ask", "let me know", "show me"];

/// Additional invitation/suggestion phrases accepted in `v2` (`F5`) so helpful
/// follow-ups after "but" ("I don't know, but you could check your coat.") and
/// polite imperative clarifications ("Please share the text you want edited.")
/// pass without allowing fabricated assertions ("I don't know, but they are under
/// the bed." still fails).
const INVITE_PHRASES_V2_EXTRA: &[&str] = &[
    "you could",
    "you can",
    "you might",
    "you may",
    "check",
    "look",
    "provide",
    "share",
    "give me",
    "send",
    "paste",
    "specify",
    "clarify",
];

/// Greeting and help-offer phrases accepted by `CheckKind::Salutation` (`v2` `F5`).
const SALUTATION_WORDS_V2: &[&str] = &[
    "hello",
    "hi",
    "hey",
    "greetings",
    "welcome",
    "good morning",
    "good afternoon",
    "good evening",
    "how can i help",
    "how may i help",
    "what can i help",
    "what would you like",
    "please share",
    "please provide",
    "go ahead",
    "tell me",
];

/// Whether `reply` contains one of [`ABSTAIN_PHRASES`] (plus [`ABSTAIN_PHRASES_V2_EXTRA`] in `v2`).
fn abstains_mode(reply: &[String], v2: bool) -> bool {
    has_any(reply, ABSTAIN_PHRASES) || (v2 && has_any(reply, ABSTAIN_PHRASES_V2_EXTRA))
}

/// Whether `reply` contains one of [`ABSTAIN_PHRASES`] (plus [`ABSTAIN_PHRASES_V2_EXTRA`] in `v2`)
/// not followed by a word of [`NOT_INABILITY_AFTER`] ("can't wait" is not an abstention).
fn abstains_strictly_mode(reply: &[String], v2: bool) -> bool {
    let check_slice = |phrases: &[&str]| {
        phrases.iter().any(|p| {
            let phrase = words(p);
            reply.windows(phrase.len()).enumerate().any(|(i, w)| {
                w == phrase.as_slice()
                    && !reply
                        .get(i + phrase.len())
                        .is_some_and(|next| NOT_INABILITY_AFTER.contains(&next.as_str()))
            })
        })
    };
    check_slice(ABSTAIN_PHRASES) || (v2 && check_slice(ABSTAIN_PHRASES_V2_EXTRA))
}

fn invites_mode(text: &[String], v2: bool) -> bool {
    has_any(text, INVITE_PHRASES) || (v2 && has_any(text, INVITE_PHRASES_V2_EXTRA))
}

/// Whether one of `phrases` occurs in `text` (as words).
fn has_any(text: &[String], phrases: &[&str]) -> bool {
    phrases.iter().any(|p| contains_phrase(text, &words(p)))
}

/// The sentences of a reply (split after `.`, `!`, `?` and newlines), each as
/// words with whether it ends in `?`.
fn reply_sentences(reply: &str) -> Vec<(Vec<String>, bool)> {
    let mut out = Vec::new();
    let mut current = String::new();
    for c in reply.chars() {
        if matches!(c, '.' | '!' | '?' | '\n') {
            let w = words(&current);
            if !w.is_empty() {
                out.push((w, c == '?'));
            }
            current.clear();
        } else {
            current.push(c);
        }
    }
    let w = words(&current);
    if !w.is_empty() {
        out.push((w, false));
    }
    out
}

/// Why an `abstain_exact` reply is not a clean abstention, or `None` when it
/// is one. A clean abstention contains an abstain phrase that is not "can't
/// wait"; no agreement ([`AFFIRM_PHRASES`], or "sure" not after "not"); no
/// guess ([`GUESS_MARKERS`]) in a sentence that is neither a question nor an
/// invitation; in each sentence, nothing after "but" unless that clause
/// abstains again, invites the user or is a question ("I don't know, but they
/// are under the bed." fails); and no made-up specific
/// ([`fabricated_specifics_mode`]). A plain assertion outside these patterns and
/// outside the row's answer-class list still passes.
fn abstention_fault_mode(users: &[&str], reply: &str, v2: bool) -> Option<&'static str> {
    let reply_words = words(reply);
    if !abstains_strictly_mode(&reply_words, v2) {
        return Some("no abstain phrase");
    }
    let sure_agrees = reply_words
        .iter()
        .enumerate()
        .any(|(i, w)| w == "sure" && (i == 0 || reply_words[i - 1] != "not"));
    if sure_agrees || has_any(&reply_words, AFFIRM_PHRASES) {
        return Some("agrees");
    }
    for (sentence, question) in reply_sentences(reply) {
        let invites = question || invites_mode(&sentence, v2);
        if !invites && has_any(&sentence, GUESS_MARKERS) {
            return Some("guesses");
        }
        if let Some(at) = sentence.iter().position(|w| w == "but") {
            let rest = &sentence[at + 1..];
            if !(question || abstains_strictly_mode(rest, v2) || invites_mode(rest, v2)) {
                return Some("asserts after but");
            }
        }
    }
    if !fabricated_specifics_mode(users, reply, v2).is_empty() {
        return Some("names a specific");
    }
    None
}

#[allow(dead_code)]
fn abstention_fault(users: &[&str], reply: &str) -> Option<&'static str> {
    abstention_fault_mode(users, reply, false)
}

/// The specifics a reply asserts that its user turns did not supply: digits,
/// and capitalised words other than a form of "I" or "OK" that are inside a
/// sentence, or begin one (the start of the reply or after `.`, `!`, `?`, a
/// newline, `:` or an opening quote) without being one of
/// [`SENTENCE_OPENERS`] (plus [`SENTENCE_OPENERS_V2_EXTRA`] in `v2`). Words of
/// the user turns (compared lowercased) are exempt. An empty result means the
/// reply names no made-up name or number.
fn fabricated_specifics_mode(users: &[&str], reply: &str, v2: bool) -> Vec<String> {
    let user_words: BTreeSet<String> = users.iter().flat_map(|u| words(u)).collect();
    let text = reply.replace('\u{2019}', "'");
    let mut out = Vec::new();
    let mut sentence_start = true;
    let mut current = String::new();
    let is_opener = |lower: &str| {
        SENTENCE_OPENERS.contains(&lower) || (v2 && SENTENCE_OPENERS_V2_EXTRA.contains(&lower))
    };
    let flush = |word: &mut String, start: &mut bool, out: &mut Vec<String>| {
        let trimmed = word.trim_matches('\'');
        if !trimmed.is_empty() {
            let lower = trimmed.to_lowercase();
            let digit = trimmed.chars().any(|c| c.is_ascii_digit());
            let capital = trimmed.chars().next().is_some_and(char::is_uppercase);
            let told = user_words.contains(&lower);
            let named = capital
                && !NOT_SPECIFIC.contains(&lower.as_str())
                && (!*start || !is_opener(&lower));
            if !told && (digit || named) {
                out.push(trimmed.to_owned());
            }
            *start = false;
        }
        word.clear();
    };
    for c in text.chars() {
        if c.is_alphanumeric() || c == '\'' {
            current.push(c);
            continue;
        }
        flush(&mut current, &mut sentence_start, &mut out);
        if matches!(c, '.' | '!' | '?' | '\n' | '"' | '\u{201c}' | ':') {
            sentence_start = true;
        }
    }
    flush(&mut current, &mut sentence_start, &mut out);
    out
}

#[allow(dead_code)]
fn fabricated_specifics(users: &[&str], reply: &str) -> Vec<String> {
    fabricated_specifics_mode(users, reply, false)
}

impl RowCheck {
    /// Whether `reply` passes under `v1` (`v2 = false`) or `v2` (`v2 = true`).
    fn passes_mode(&self, users: &[&str], reply: &str, v2: bool) -> bool {
        let reply_words = words(reply);
        let forbidden = self.forbid.iter().any(|t| contains_phrase(&reply_words, t));
        match &self.kind {
            CheckKind::Any(terms) => terms.iter().any(|t| contains_phrase(&reply_words, t)),
            CheckKind::Abstain => abstains_mode(&reply_words, v2),
            CheckKind::Question => reply.contains('?'),
            CheckKind::Exact(terms) | CheckKind::ReplyExact(terms) => {
                !forbidden
                    && !self.keys.iter().any(|t| contains_phrase(&reply_words, t))
                    && terms.iter().any(|t| contains_phrase(&reply_words, t))
            }
            CheckKind::ExactText { terms, forbid } => {
                let compact = compact_math_text(reply);
                !forbid.iter().any(|f| compact.contains(f.as_str()))
                    && terms.iter().any(|t| compact.contains(t.as_str()))
            }
            CheckKind::Clarify(terms) => {
                let asks_or_invites =
                    reply.contains('?') || (v2 && invites_mode(&reply_words, true));
                asks_or_invites
                    && !forbidden
                    && !self.keys.iter().any(|t| contains_phrase(&reply_words, t))
                    && terms.iter().any(|t| contains_phrase(&reply_words, t))
            }
            CheckKind::AbstainExact => {
                !forbidden && abstention_fault_mode(users, reply, v2).is_none()
            }
            CheckKind::Salutation => {
                !reply_words.is_empty()
                    && !forbidden
                    && (reply.contains('?') || has_any(&reply_words, SALUTATION_WORDS_V2))
                    && fabricated_specifics_mode(users, reply, true).is_empty()
            }
            CheckKind::OneOf(choices) => choices.iter().any(|c| &reply_words == c),
            CheckKind::WordCount(n) => {
                reply_words.len() == *n
                    && !forbidden
                    && (self.keys.is_empty()
                        || self.keys.iter().any(|t| contains_phrase(&reply_words, t)))
            }
        }
    }

    /// Whether `reply` passes under the default (`v1`) rule.
    fn passes(&self, users: &[&str], reply: &str) -> bool {
        self.passes_mode(users, reply, false)
    }

    /// Why a reply failed this deterministic check.
    fn fail_reason_mode(&self, users: &[&str], reply: &str, v2: bool) -> &'static str {
        let reply_words = words(reply);
        let forbidden = self.forbid.iter().any(|t| contains_phrase(&reply_words, t));
        let has_key = self.keys.iter().any(|t| contains_phrase(&reply_words, t));
        match &self.kind {
            CheckKind::Any(_) => "missing_required_term",
            CheckKind::Abstain => "no_abstain_phrase",
            CheckKind::Question => "missing_question_mark",
            CheckKind::Exact(_) | CheckKind::ReplyExact(_) => {
                if forbidden {
                    "contains_forbidden_term"
                } else if has_key {
                    "contains_distractor_key"
                } else {
                    "missing_expected_value"
                }
            }
            CheckKind::ExactText { forbid, .. } => {
                let compact = compact_math_text(reply);
                if forbid.iter().any(|f| compact.contains(f.as_str())) {
                    "contains_forbidden_term"
                } else {
                    "missing_expected_value"
                }
            }
            CheckKind::Clarify(_) => {
                let asks_or_invites =
                    reply.contains('?') || (v2 && invites_mode(&reply_words, true));
                if !asks_or_invites {
                    "missing_question_mark"
                } else if forbidden {
                    "contains_forbidden_term"
                } else if has_key {
                    "contains_distractor_key"
                } else {
                    "missing_clarify_phrase"
                }
            }
            CheckKind::AbstainExact => {
                if forbidden {
                    "contains_forbidden_term"
                } else {
                    abstention_fault_mode(users, reply, v2).unwrap_or("failed_abstain_exact")
                }
            }
            CheckKind::Salutation => {
                if forbidden {
                    "contains_forbidden_term"
                } else if !fabricated_specifics_mode(users, reply, true).is_empty() {
                    "names a specific"
                } else {
                    "missing_salutation_or_question"
                }
            }
            CheckKind::OneOf(_) => "not_exact_one_of_choice",
            CheckKind::WordCount(_) => {
                if forbidden {
                    "contains_forbidden_term"
                } else if !self.keys.is_empty() && !has_key {
                    "missing_story_anchor"
                } else {
                    "wrong_word_count"
                }
            }
        }
    }
}

/// Deterministic per-row classification (`PASS`, `FAIL`, or `BLOCKED`).
struct RowVerdict {
    status: &'static str,
    check_kind: Option<&'static str>,
    check_pass: Option<bool>,
    reason: &'static str,
}

impl RowVerdict {
    fn record(&self) -> Value {
        json!({
            "status": self.status,
            "check_kind": self.check_kind,
            "check_pass": self.check_pass,
            "reason": self.reason,
            "fluent": Value::Null,
            "relevant": Value::Null,
            "raw": [],
        })
    }
}

/// The row checks of a run and where they came from.
struct Checks {
    rows: BTreeMap<String, RowCheck>,
    source: Value,
}

impl Checks {
    fn is_v2(&self) -> bool {
        self.source.get("version").and_then(Value::as_str) == Some("v2")
    }

    /// The check result of `reply` on row `id` (whose user turns are
    /// `users`), or `None` for an unchecked (`BLOCKED`) row.
    fn of(&self, id: &str, users: &[&str], reply: &str) -> Option<bool> {
        let v2 = self.is_v2();
        self.rows.get(id).map(|c| c.passes_mode(users, reply, v2))
    }

    /// Deterministic verdict (`PASS`, `FAIL`, or `BLOCKED`) for `reply` on row `id`.
    fn verdict(&self, id: &str, users: &[&str], reply: &str) -> RowVerdict {
        let v2 = self.is_v2();
        match self.rows.get(id) {
            None => RowVerdict {
                status: "BLOCKED",
                check_kind: None,
                check_pass: None,
                reason: BLOCKED_REASON,
            },
            Some(check) => {
                if check.passes_mode(users, reply, v2) {
                    RowVerdict {
                        status: "PASS",
                        check_kind: Some(check.kind.name()),
                        check_pass: Some(true),
                        reason: "passed_deterministic_check",
                    }
                } else {
                    RowVerdict {
                        status: "FAIL",
                        check_kind: Some(check.kind.name()),
                        check_pass: Some(false),
                        reason: check.fail_reason_mode(users, reply, v2),
                    }
                }
            }
        }
    }
}

/// A checks file: tab-separated `id kind history terms [forbid [keys]]` per line
/// (`#` comments and blank lines ignored). `kind` is `any`, `abstain`,
/// `question`, `exact`, `reply_exact`, `exact_text`, `clarify`, `abstain_exact`,
/// `salutation`, `one_of` or `word_count_6`; `history` is `none`, `recall`,
/// `derived` or `topic`; `terms` is `|`-separated words or phrases for `any`,
/// `exact`, `reply_exact`, `exact_text`, `clarify` and `one_of` and `-`
/// otherwise; `forbid` (default `-`) is `|`-separated words or phrases that fail
/// an `exact`, `reply_exact`, `exact_text`, `clarify`, `abstain_exact`,
/// `salutation` or `word_count_6` check and must be `-` for the other kinds;
/// `keys` (default `-`) is `|`-separated words of the distractor's key (or story
/// anchors for `word_count_6`).
fn parse_checks(text: &str) -> Result<BTreeMap<String, RowCheck>, Error> {
    let mut rows = BTreeMap::new();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let (id, kind, history, terms, forbid, keys) = match fields[..] {
            [id, kind, history, terms] => (id, kind, history, terms, "-", "-"),
            [id, kind, history, terms, forbid] => (id, kind, history, terms, forbid, "-"),
            [id, kind, history, terms, forbid, keys] => (id, kind, history, terms, forbid, keys),
            _ => {
                return Err(format!(
                    "checks line {}: expected 4 to 6 tab-separated fields",
                    number + 1
                )
                .into())
            }
        };
        let term_list = |terms: &str| -> Result<Vec<Vec<String>>, Error> {
            let terms: Vec<Vec<String>> = terms.split('|').map(words).collect();
            if terms.iter().any(Vec::is_empty) {
                return Err(format!("checks line {}: an empty term", number + 1).into());
            }
            Ok(terms)
        };
        let compact_list = |terms: &str| -> Result<Vec<String>, Error> {
            let terms: Vec<String> = terms.split('|').map(compact_math_text).collect();
            if terms.iter().any(String::is_empty) {
                return Err(format!("checks line {}: an empty exact_text term", number + 1).into());
            }
            Ok(terms)
        };
        let history = match history {
            "none" => History::None,
            "recall" => History::Recall,
            "derived" => History::Derived,
            "topic" => History::Topic,
            other => return Err(format!("checks line {}: history {other}", number + 1).into()),
        };
        let kind = match (kind, terms) {
            ("any", terms) => CheckKind::Any(term_list(terms)?),
            ("exact", terms) if terms != "-" => CheckKind::Exact(term_list(terms)?),
            ("reply_exact", terms) if terms != "-" => CheckKind::ReplyExact(term_list(terms)?),
            ("exact_text", terms) if terms != "-" => {
                if forbid == "-" {
                    return Err(format!(
                        "checks line {}: an exact_text check needs a forbidden (distractor) value",
                        number + 1
                    )
                    .into());
                }
                let terms_c = compact_list(terms)?;
                let forbid_c = compact_list(forbid)?;
                if let Some(t) = terms_c.iter().find(|t| forbid_c.contains(t)) {
                    return Err(format!(
                        "checks line {}: '{t}' is both expected and forbidden",
                        number + 1
                    )
                    .into());
                }
                CheckKind::ExactText {
                    terms: terms_c,
                    forbid: forbid_c,
                }
            }
            ("clarify", terms) if terms != "-" => CheckKind::Clarify(term_list(terms)?),
            ("one_of", terms) if terms != "-" => CheckKind::OneOf(term_list(terms)?),
            ("abstain", "-") => CheckKind::Abstain,
            ("abstain_exact", "-") => CheckKind::AbstainExact,
            ("salutation", "-") => CheckKind::Salutation,
            ("word_count_6", "-") => CheckKind::WordCount(6),
            ("question", "-") => CheckKind::Question,
            _ => {
                return Err(
                    format!("checks line {}: kind {kind} with terms {terms}", number + 1).into(),
                )
            }
        };
        let forbid = match (&kind, forbid) {
            (_, "-") | (CheckKind::ExactText { .. }, _) => Vec::new(),
            (
                CheckKind::Exact(_)
                | CheckKind::ReplyExact(_)
                | CheckKind::AbstainExact
                | CheckKind::Clarify(_)
                | CheckKind::Salutation
                | CheckKind::WordCount(_),
                forbid,
            ) => term_list(forbid)?,
            _ => {
                return Err(format!(
                    "checks line {}: this kind takes no forbidden terms",
                    number + 1
                )
                .into())
            }
        };
        let keys = match (&kind, keys) {
            (_, "-") => Vec::new(),
            (
                CheckKind::Exact(_)
                | CheckKind::ReplyExact(_)
                | CheckKind::Clarify(_)
                | CheckKind::WordCount(_),
                keys,
            ) => term_list(keys)?,
            _ => {
                return Err(format!(
                    "checks line {}: only exact, reply_exact, clarify and word_count take keys",
                    number + 1
                )
                .into())
            }
        };
        if let CheckKind::Exact(terms) | CheckKind::ReplyExact(terms) = &kind {
            if let Some(t) = terms.iter().find(|t| keys.contains(t)) {
                return Err(format!(
                    "checks line {}: '{}' is both expected and a distractor key",
                    number + 1,
                    t.join(" ")
                )
                .into());
            }
            if forbid.is_empty() {
                return Err(format!(
                    "checks line {}: an exact check needs a forbidden (distractor) value",
                    number + 1
                )
                .into());
            }
            if let Some(t) = terms.iter().find(|t| forbid.contains(t)) {
                return Err(format!(
                    "checks line {}: '{}' is both expected and forbidden",
                    number + 1,
                    t.join(" ")
                )
                .into());
            }
        }
        if let CheckKind::Clarify(terms) = &kind {
            // A clarify row needs no distractor (asking about the missing material
            // is the whole behaviour), but a term that is also forbidden or a key
            // would make the check unsatisfiable, so both are refused.
            if let Some(t) = terms
                .iter()
                .find(|t| forbid.contains(t) || keys.contains(t))
            {
                return Err(format!(
                    "checks line {}: '{}' is both a clarify term and excluded",
                    number + 1,
                    t.join(" ")
                )
                .into());
            }
        }
        if rows
            .insert(
                id.to_owned(),
                RowCheck {
                    kind,
                    history,
                    forbid,
                    keys,
                },
            )
            .is_some()
        {
            return Err(format!("checks repeat {id}").into());
        }
    }
    Ok(rows)
}

/// The frozen deterministic row checks of the conversational and open reply
/// panels (`v1`), embedded so a graded report is bound to them unless `checks=`
/// overrides. The panels' ids are disjoint (`conv-*` in v2, `conv-v3-*` in v3,
/// `conv-v4-*` in v4, `conv-v5-*` in v5, and `talk-*`/`ask-*`/`do-*`/`follow-*`/`heldout-*`
/// in `reply-panel-checks.tsv`), so the default is their union.
const EMBEDDED_CHECKS: [(&str, &str); 5] = [
    (
        "data/panels/conversational-v2-checks.tsv",
        include_str!("../../../../data/panels/conversational-v2-checks.tsv"),
    ),
    (
        "data/panels/conversational-v3-checks.tsv",
        include_str!("../../../../data/panels/conversational-v3-checks.tsv"),
    ),
    (
        "data/panels/conversational-v4-checks.tsv",
        include_str!("../../../../data/panels/conversational-v4-checks.tsv"),
    ),
    (
        "data/panels/conversational-v5-checks.tsv",
        include_str!("../../../../data/panels/conversational-v5-checks.tsv"),
    ),
    (
        "data/panels/reply-panel-checks.tsv",
        include_str!("../../../../data/panels/reply-panel-checks.tsv"),
    ),
];

/// The `v2` (`F5`/`F6`) check set: pairs the frozen conversational-v2/v3/v4/v5
/// checks with `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (96
/// checked rows on the 232-row open panel) and enables the widened `v2`
/// positive-control rules.
const EMBEDDED_CHECKS_V2: [(&str, &str); 5] = [
    (
        "data/panels/conversational-v2-checks.tsv",
        include_str!("../../../../data/panels/conversational-v2-checks.tsv"),
    ),
    (
        "data/panels/conversational-v3-checks.tsv",
        include_str!("../../../../data/panels/conversational-v3-checks.tsv"),
    ),
    (
        "data/panels/conversational-v4-checks.tsv",
        include_str!("../../../../data/panels/conversational-v4-checks.tsv"),
    ),
    (
        "data/panels/conversational-v5-checks.tsv",
        include_str!("../../../../data/panels/conversational-v5-checks.tsv"),
    ),
    (
        "crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv",
        include_str!("../../fixtures/reply-panel-checks-v2.tsv"),
    ),
];

fn embedded_checks_from(files: &[(&str, &str)], v2: bool) -> Result<Checks, Error> {
    let mut rows = BTreeMap::new();
    let mut sources = Vec::new();
    for &(path, text) in files {
        let file = parse_checks(text)?;
        sources.push(json!({
            "embedded": path,
            "sha256": uor_r4_training::sha256_bytes(text.as_bytes()),
            "rows": file.len(),
        }));
        for (id, check) in file {
            if rows.insert(id.clone(), check).is_some() {
                return Err(format!("embedded checks repeat {id}").into());
            }
        }
    }
    Ok(Checks {
        source: json!({
            "version": if v2 { "v2" } else { "v1" },
            "embedded": sources,
            "rows": rows.len(),
        }),
        rows,
    })
}

/// The union of the default (`v1`) embedded checks files; an id in two files is an error.
fn embedded_checks() -> Result<Checks, Error> {
    embedded_checks_from(&EMBEDDED_CHECKS, false)
}

/// The union of the `v2` (`F5`/`F6`) embedded checks files.
fn embedded_checks_v2() -> Result<Checks, Error> {
    embedded_checks_from(&EMBEDDED_CHECKS_V2, true)
}

/// `checks=` if given (`checks=none` for no checks, `checks=v1` for default
/// embedded v1, `checks=v2` for embedded v2 with widened `F5`/`F6` rules, or a
/// file path), else the default embedded `v1` files.
fn load_checks(args: &Args) -> Result<Checks, Error> {
    match args.0.get("checks").map(String::as_str) {
        Some("none") => Ok(Checks {
            rows: BTreeMap::new(),
            source: json!("none"),
        }),
        None | Some("v1") => embedded_checks(),
        Some("v2") => embedded_checks_v2(),
        Some(path) => {
            let bytes = fs::read(path)?;
            let rows = parse_checks(std::str::from_utf8(&bytes)?)?;
            let v2 = path.contains("v2");
            Ok(Checks {
                source: json!({
                    "version": if v2 { "v2" } else { "v1" },
                    "path": path,
                    "sha256": uor_r4_training::sha256_bytes(&bytes),
                    "rows": rows.len(),
                }),
                rows,
            })
        }
    }
}

const CHECK_RULE: &str = "words are lowercased and split at anything not a letter, digit or \
     apostrophe; a phrase matches consecutive words. any = the reply contains one listed word or \
     phrase; abstain = the reply contains one of the abstain phrases; question = the reply \
     contains '?'; exact = the reply contains one spelling of the expected value and none of the \
     forbidden values (the conversation's distractor values and, for a closed class, its other \
     members) and none of the distractor's key words: a wrong value, both values, the value \
     bound to the distractor's key, or no value fails; reply_exact = exact without the multi-turn \
     recall precondition; exact_text (v2) = sign-preserving substring match on lowercased \
     whitespace-stripped text (with unicode minus/superscripts normalized) containing at least one \
     expected term and no forbidden term; clarify = the reply asks (or in v2 politely requests via \
     an invitation phrase) about the missing material named by one of the row's clarify phrases, \
     and contains none of the forbidden/key words; abstain_exact = the reply contains an abstain \
     phrase not followed by 'wait', no agreement phrase (or 'sure' not after 'not'), no guess \
     marker in a sentence that is neither a question nor contains an invitation phrase, nothing \
     after 'but' in a sentence unless that clause abstains, invites or is a question, none of the \
     row's forbidden words, no digit, and no capitalised word that is not a word of the user \
     turns, I or OK, or a listed sentence opener at the start of a sentence; salutation (v2) = the \
     reply greets/offers help or asks a question without fabricated specifics or forbidden words; \
     one_of (v2) = the reply's full word sequence matches one listed option; word_count_6 (v2) = \
     format-only check that the reply has exactly 6 words and no forbidden words; rows with no \
     deterministic check are classified BLOCKED (open_ended_no_exact_anchor)";

fn checks_record(checks: &Checks) -> Value {
    let v2 = checks.is_v2();
    json!({
        "version": if v2 { "v2" } else { "v1" },
        "source": checks.source,
        "rule": CHECK_RULE,
        "blocked_reason": BLOCKED_REASON,
        "abstain_phrases": ABSTAIN_PHRASES,
        "abstain_phrases_v2_extra": v2.then_some(ABSTAIN_PHRASES_V2_EXTRA),
        "abstain_exact": {
            "not_inability_after": NOT_INABILITY_AFTER,
            "affirm_phrases": AFFIRM_PHRASES,
            "guess_markers": GUESS_MARKERS,
            "invite_phrases": INVITE_PHRASES,
            "invite_phrases_v2_extra": v2.then_some(INVITE_PHRASES_V2_EXTRA),
            "sentence_openers": SENTENCE_OPENERS,
            "sentence_openers_v2_extra": v2.then_some(SENTENCE_OPENERS_V2_EXTRA),
        },
    })
}

/// The first of `terms` that occurs in one of `texts` (as words), if any.
fn any_in(terms: &[Vec<String>], texts: &[Vec<String>]) -> Option<String> {
    terms
        .iter()
        .find(|t| texts.iter().any(|e| contains_phrase(e, t)))
        .map(|t| t.join(" "))
}

/// Validate checks against loaded requests: a multi-turn row needs a check
/// whose history is not `none` and a single-turn row history `none`; no `any`
/// or `exact` term may occur in the last user turn; a `recall` row needs a
/// term in an earlier user turn. An `exact` row must be a multi-turn recall
/// row whose forbidden values are absent from the last turn and at least one
/// of which (a distractor) is stated in an earlier turn, so the row can be
/// answered neither from its last turn nor by copying its whole history; its
/// distractor keys, if any, likewise absent from the last turn and one of
/// them stated in an earlier turn. An
/// `abstain_exact` row's forbidden words must not occur in its user turns.
/// Returns the ids of checks with no loaded request.
fn validate_checks(checks: &Checks, requests: &[Request]) -> Result<Vec<String>, Error> {
    let mut seen = BTreeSet::new();
    for request in requests {
        let Some(check) = checks.rows.get(&request.id) else {
            if request.id.starts_with("conv-") && request.user_turns.len() > 1 {
                return Err(format!("multi-turn row {} has no check", request.id).into());
            }
            continue;
        };
        seen.insert(request.id.as_str());
        let multi = request.user_turns.len() > 1;
        if multi == (check.history == History::None) {
            return Err(format!(
                "row {} has {} user turns but history {:?}",
                request.id,
                request.user_turns.len(),
                check.history
            )
            .into());
        }
        let turns: Vec<Vec<String>> = request.user_turns.iter().map(|t| words(t)).collect();
        let (earlier, last) = turns.split_at(turns.len().saturating_sub(1));
        match &check.kind {
            CheckKind::Any(terms) | CheckKind::Exact(terms) => {
                if let Some(t) = any_in(terms, last) {
                    return Err(format!(
                        "row {}: check term '{t}' is in the last user turn",
                        request.id
                    )
                    .into());
                }
                if check.history == History::Recall && any_in(terms, earlier).is_none() {
                    return Err(format!(
                        "recall row {}: no check term is in an earlier user turn",
                        request.id
                    )
                    .into());
                }
            }
            CheckKind::AbstainExact | CheckKind::Salutation | CheckKind::WordCount(_) => {
                if let Some(t) = any_in(&check.forbid, &turns) {
                    return Err(format!(
                        "row {}: forbidden word '{t}' is in a user turn",
                        request.id
                    )
                    .into());
                }
            }
            CheckKind::OneOf(_) => {}
            CheckKind::ExactText { terms, forbid } => {
                let compact_turns: Vec<String> = request
                    .user_turns
                    .iter()
                    .map(|t| compact_math_text(t))
                    .collect();
                if let Some(last_compact) = compact_turns.last() {
                    if let Some(t) = terms.iter().find(|t| last_compact.contains(t.as_str())) {
                        return Err(format!(
                            "row {}: exact_text term '{t}' is in the last user turn",
                            request.id
                        )
                        .into());
                    }
                }
                if let Some(f) = forbid
                    .iter()
                    .find(|f| compact_turns.iter().any(|u| u.contains(f.as_str())))
                {
                    return Err(format!(
                        "row {}: exact_text forbidden term '{f}' is in a user turn",
                        request.id
                    )
                    .into());
                }
            }
            CheckKind::ReplyExact(terms) => {
                // The whole point of this kind: no recall precondition, so it
                // validates on a single-turn row. The terms must still be the
                // model's own work (not answerable by echoing the request), and
                // neither the forbidden distractors nor their key words may be
                // words the user wrote — a distractor the user supplied is not
                // a distractor.
                if let Some(t) = any_in(terms, last) {
                    return Err(format!(
                        "row {}: check term '{t}' is in the last user turn",
                        request.id
                    )
                    .into());
                }
                if let Some(t) = any_in(&check.forbid, &turns) {
                    return Err(format!(
                        "row {}: reply_exact forbidden word '{t}' is in a user turn",
                        request.id
                    )
                    .into());
                }
                if let Some(t) = any_in(&check.keys, &turns) {
                    return Err(format!(
                        "row {}: reply_exact distractor key '{t}' is in a user turn",
                        request.id
                    )
                    .into());
                }
            }
            CheckKind::Clarify(terms) => {
                // Same rule as `any` for the terms — a phrase the request already
                // contains is not the model's own work — plus the same distractor
                // exclusions as `reply_exact`.
                if let Some(t) = any_in(terms, last) {
                    return Err(format!(
                        "row {}: clarify term '{t}' is in the last user turn",
                        request.id
                    )
                    .into());
                }
                if let Some(t) = any_in(&check.forbid, &turns) {
                    return Err(format!(
                        "row {}: clarify forbidden word '{t}' is in a user turn",
                        request.id
                    )
                    .into());
                }
                if let Some(t) = any_in(&check.keys, &turns) {
                    return Err(
                        format!("row {}: clarify key '{t}' is in a user turn", request.id).into(),
                    );
                }
            }
            CheckKind::Abstain | CheckKind::Question => {
                if check.history == History::Recall || check.history == History::Topic {
                    return Err(format!(
                        "row {}: {:?} needs an any or exact check",
                        request.id, check.history
                    )
                    .into());
                }
            }
        }
        if matches!(check.kind, CheckKind::Exact(_)) {
            if check.history != History::Recall {
                return Err(format!("exact row {} must be a recall row", request.id).into());
            }
            if let Some(t) = any_in(&check.forbid, last) {
                return Err(format!(
                    "row {}: forbidden value '{t}' is in the last user turn",
                    request.id
                )
                .into());
            }
            if any_in(&check.forbid, earlier).is_none() {
                return Err(format!(
                    "exact row {}: no distractor value is stated in an earlier turn",
                    request.id
                )
                .into());
            }
            if let Some(t) = any_in(&check.keys, last) {
                return Err(format!(
                    "row {}: distractor key '{t}' is in the last user turn",
                    request.id
                )
                .into());
            }
            if !check.keys.is_empty() && any_in(&check.keys, earlier).is_none() {
                return Err(format!(
                    "exact row {}: no distractor key is stated in an earlier turn",
                    request.id
                )
                .into());
            }
        }
    }
    Ok(checks
        .rows
        .keys()
        .filter(|id| !seen.contains(id.as_str()))
        .cloned()
        .collect())
}

/// Everything a deterministic grading run is judged with, besides the replies.
struct Judging<'a> {
    ill: &'a IdList,
    checks: &'a Checks,
    constants: &'a Constants,
}

impl Judging<'_> {
    fn grader_record(&self) -> Value {
        json!({
            "engine": "deterministic (chat-grade exact checks)",
            "model": "deterministic-exact-checks/1",
            "digest": self.checks.source,
            "role": "D28 deterministic exact-check grader; zero neural models or external services",
            "blocked_reason": BLOCKED_REASON,
        })
    }

    /// Grade the panel deterministically and return the report fields for `grade-replies`.
    fn judge(&self, panel: &Value) -> Result<serde_json::Map<String, Value>, Error> {
        let rows = judge_panel(panel, self.checks, &self.constants.replies)?;
        let overall = summarize(&rows, self.checks, &self.constants.replies)?;
        let per_tier = tier_summary(&rows, &self.ill.ids, self.checks, &self.constants.replies)?;
        let requests: Vec<Request> = panel_conversations(panel)?
            .into_iter()
            .map(|(id, category, turns)| Request {
                id,
                category,
                user_turns: turns.into_iter().map(|(u, _)| u).collect(),
            })
            .collect();
        let per_cat_check_only =
            check_only_controls(&requests, self.checks, &self.constants.replies)?;
        let mut fields = serde_json::Map::new();
        fields.insert("grader".into(), self.grader_record());
        for key in [
            "actual",
            "control_derangement",
            "paired_against_control",
            "control_constants",
            "best_constant",
            "check_only_controls",
            "best_control_or_copy",
        ] {
            fields.insert(key.into(), overall[key].clone());
        }
        fields.insert("control_rule".into(), json!(CONTROL_RULE));
        let per_category: BTreeMap<String, Value> = overall["per_category"]
            .as_object()
            .ok_or("summary without per_category")?
            .iter()
            .map(|(k, v)| (k.clone(), v["actual"].clone()))
            .collect();
        fields.insert("per_category".into(), json!(per_category));
        fields.insert(
            "per_category_controls".into(),
            overall["per_category"].clone(),
        );
        fields.insert(
            "per_category_check_only_controls".into(),
            per_cat_check_only,
        );
        fields.insert("rows".into(), Value::Array(rows));
        fields.insert("headline".into(), headline_of(&per_tier));
        fields.insert("per_tier".into(), per_tier);
        fields.insert("tiers".into(), tiers_record(self.ill));
        fields.insert("checks".into(), checks_record(self.checks));
        fields.insert("constants".into(), self.constants.record());
        Ok(fields)
    }
}

/// The report headline: per tier, the categories where the model beats that
/// category's best constant (see [`BEST_CONSTANT_RULE`]) and where it beats
/// `max(best_constant, copy_first_stated, copy_last_stated)` (`F3`/`F4`).
fn headline_of(per_tier: &Value) -> Value {
    let extract = |field: &str| -> serde_json::Map<String, Value> {
        per_tier
            .as_object()
            .map(|tiers| {
                tiers
                    .iter()
                    .map(|(tier, record)| {
                        let headline = &record["headline"];
                        let value = match headline.get(field) {
                            Some(beats) => beats.clone(),
                            None => headline.clone(),
                        };
                        (tier.clone(), value)
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    json!({
        "categories_model_beats_best_constant": extract("categories_model_beats_best_constant"),
        "categories_model_beats_best_control_or_copy": extract("categories_model_beats_best_control_or_copy"),
        "rule": BEST_CONSTANT_RULE,
        "detail": "per_tier.<tier>.headline.per_category",
    })
}

const CONTROL_RULE: &str = "derangement: each reply checked against the next request's \
     deterministic check; constants: each row checked with every constant reply in \
     place of the model's; a category the model does not beat its constants on measures nothing \
     about the model. check-only controls: each row's check applied to its last user turn \
     (echo_last) and to all its user turns joined (echo_history), with no neural grader";

fn grade(_arguments: &[String]) -> Result<(), Error> {
    Err(
        "D28/D29: `chat-grade grade` (combined historical CPU model generation + Ollama neural \
         grading) is retired; score recorded replies deterministically with `chat-grade \
         grade-replies`"
            .into(),
    )
}

/// The window span of a value's last occurrence in `history`, by decoded text
/// (decoding is per-id concatenation, so token `j` covers exactly the
/// characters between the `j`-th and `j + 1`-th prefix decodings). The match
/// is ASCII-case-insensitive: the frozen checks' terms are lowercase words
/// while the conversations state the values capitalised. The last occurrence
/// over every alternative spelling wins; a value that is not stated in the
/// window has no span. This is the copy-span oracle's source map: it reads
/// the frozen check's expected value and the row's own history, nothing else.
#[allow(dead_code)]
fn find_value_span(
    history: &[u32],
    decode: &dyn Fn(&[u32]) -> String,
    alternatives: &[String],
) -> Option<(usize, usize)> {
    let mut boundaries = Vec::with_capacity(history.len() + 1);
    boundaries.push(0usize);
    let mut text = String::new();
    for &id in history {
        text.push_str(&decode(&[id]));
        boundaries.push(text.len());
    }
    // The rightmost occurrence of any alternative spelling, as a byte range.
    let mut best: Option<(usize, usize)> = None;
    for alternative in alternatives {
        let needle = alternative.trim();
        if needle.is_empty() || needle.len() > text.len() {
            continue;
        }
        for (at, _) in text
            .as_bytes()
            .windows(needle.len())
            .enumerate()
            .filter(|(_, window)| window.eq_ignore_ascii_case(needle.as_bytes()))
        {
            if best.is_none_or(|(best_at, _)| at >= best_at) {
                best = Some((at, needle.len()));
            }
        }
    }
    let (hit, needle_len) = best?;
    let start = boundaries.iter().rposition(|&b| b <= hit)?;
    let end = boundaries.iter().position(|&b| b >= hit + needle_len)?;
    (start < end).then_some((start, end))
}

/// `reply`: retired in `chat-grade` under D24/D29 (historical CPU/Candle `StackModel` is quarantined).
fn reply(_arguments: &[String]) -> Result<(), Error> {
    Err(
        "D24/D29: historical CPU/Candle StackModel reply generation in `chat-grade reply` is \
         retired; generate replies on the admitted native GPU path and score them deterministically \
         with `chat-grade grade-replies`"
            .into(),
    )
}

/// Each panel row's id, category and conversation as (user, reply) pairs,
/// from a `reply_panel` record (`rows[].turns[].{user, reply}`).
fn panel_conversations(
    panel: &Value,
) -> Result<Vec<(String, String, Vec<(String, String)>)>, Error> {
    let rows = panel["rows"].as_array().ok_or("panel without rows")?;
    rows.iter()
        .map(|row| {
            let field = |key: &str| -> Result<String, Error> {
                Ok(row[key]
                    .as_str()
                    .ok_or_else(|| format!("a panel row without {key}"))?
                    .to_owned())
            };
            let turns = row["turns"]
                .as_array()
                .ok_or("a panel row without turns")?
                .iter()
                .map(|t| -> Result<(String, String), Error> {
                    Ok((
                        t["user"].as_str().ok_or("a turn without user")?.to_owned(),
                        t["reply"]
                            .as_str()
                            .ok_or("a turn without reply")?
                            .trim()
                            .to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            if turns.is_empty() {
                return Err("a panel row with no turns".into());
            }
            Ok((field("id")?, field("category")?, turns))
        })
        .collect()
}

/// Grade every conversation of a `reply_panel` record deterministically under D28,
/// along with its derangement control (the reply checked against the next request's
/// conversation) and each constant reply in place of the model's. Returns the
/// graded rows in panel order; [`summarize`] turns them into tallies.
fn judge_panel(panel: &Value, checks: &Checks, constants: &[String]) -> Result<Vec<Value>, Error> {
    let conversations = panel_conversations(panel)?;
    let n = conversations.len();
    let mut graded_rows = Vec::new();
    for (i, (id, category, conversation)) in conversations.iter().enumerate() {
        let users: Vec<&str> = conversation.iter().map(|(u, _)| u.as_str()).collect();
        let reply = conversation.last().map(|(_, r)| r.as_str()).unwrap_or("");
        let verdict = checks.verdict(id, &users, reply);
        let (next_id, _, next_conv) = &conversations[(i + 1) % n];
        let next_users: Vec<&str> = next_conv.iter().map(|(u, _)| u.as_str()).collect();
        let control_verdict = checks.verdict(next_id, &next_users, reply);
        let constant_grades: Vec<Value> = constants
            .iter()
            .map(|c| checks.verdict(id, &users, c).record())
            .collect();
        graded_rows.push(json!({
            "id": id,
            "category": category,
            "status": verdict.status,
            "check_kind": verdict.check_kind,
            "check_pass": verdict.check_pass,
            "reason": verdict.reason,
            "conversation": conversation.iter().map(|(u, a)| json!({"user": u, "assistant": a})).collect::<Vec<_>>(),
            "grades": verdict.record(),
            "control_grades": control_verdict.record(),
            "constant_grades": constant_grades,
        }));
    }
    Ok(graded_rows)
}

/// `grade-replies`: deterministically grade replies another tool already produced --
/// a `replies.json` (`panel`) or a `geometric-stack lut-chat` `chat.json` (`record`).
/// Any neural `grader=` or `ollama_url=` argument is refused under D28.
fn grade_replies(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &[
            "out",
            "replies",
            "grader",
            "ollama_url",
            "ill_posed",
            "checks",
            "constants",
        ],
    )?;
    refuse_neural_grader(&args)?;
    let out = PathBuf::from(args.required("out")?);
    let ill = ill_posed(&args)?;
    let checks = load_checks(&args)?;
    let constants = constants(&args)?;
    let replies_path = PathBuf::from(args.required("replies")?);
    let judging = Judging {
        ill: &ill,
        checks: &checks,
        constants: &constants,
    };
    let bytes = fs::read(&replies_path)?;
    let source: Value = serde_json::from_slice(&bytes)?;
    let panel = if source.get("panel").is_some() {
        &source["panel"]
    } else if source.get("record").is_some() {
        &source["record"]
    } else {
        return Err("replies must hold a reply_panel record under `panel` or `record`".into());
    };
    report_output::claim(&out)?;
    let result = (|| -> Result<(), Error> {
        let judged = judging.judge(panel)?;
        let mut report = json!({
            "schema": "uor-r4.chat-grade/1",
            "replies": {
                "path": replies_path.display().to_string(),
                "sha256": uor_r4_training::sha256_bytes(&bytes),
                "schema": source["schema"],
                "model": source.get("model").or_else(|| source.get("artifact")).cloned(),
                "engine": source.get("engine").cloned(),
            },
            "decoding": "as recorded in the replies file",
        });
        let object = report.as_object_mut().ok_or("report is not an object")?;
        object.extend(judged);
        object.insert(
            "executable_sha256".into(),
            json!(sha256_file(&std::env::current_exe()?)?),
        );
        object.insert(
            "wall_seconds".into(),
            json!(started.elapsed().as_secs_f64()),
        );
        fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
        println!(
            "actual {} | control {}",
            report["actual"], report["control_derangement"]
        );
        Ok(())
    })();
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

/// The ill-posed heldout-200 ids (tier `K-ill-posed`), embedded so a graded
/// report's tiers are bound to the executable unless `ill_posed=` overrides.
const EMBEDDED_ILL_POSED: &str =
    include_str!("../../../../data/panels/heldout-ill-posed-v3-ids.txt");
const EMBEDDED_ILL_POSED_PATH: &str = "data/panels/heldout-ill-posed-v3-ids.txt";

const TIER_RULE: &str = "tier from id and category only: C = ids conv-* with a tier C category; \
     K-clean / K-ill-posed = category heldout_first_turn, split by the ill-posed id list; stretch = \
     category stretch_heldout; everyday = the everyday-32 categories; any other category is an \
     error. A row's controls are graded on its own reply's row, so per-tier control and paired \
     counts use the same rows as the tier's actual";

/// The categories of tier C (ids `conv-*`) and of everyday-32.
const TIER_C_CATEGORIES: [&str; 6] = [
    "smalltalk_feelings",
    "multi_turn_memory",
    "self_contained_instruction",
    "clarify_or_on_topic",
    "story_continuation",
    "unknowable_or_impossible",
];
const EVERYDAY_CATEGORIES: [&str; 4] = [
    "smalltalk",
    "simple_question",
    "simple_instruction",
    "follow_up",
];

/// The tier names `tier_of` can return.
const TIERS: [&str; 5] = ["C", "K-clean", "K-ill-posed", "stretch", "everyday"];

/// An id list and where it came from (path or embedded, with its sha256).
struct IdList {
    ids: BTreeSet<String>,
    source: Value,
}

/// An id list file: one id per line; blank lines and `#` comments ignored; a
/// repeated id is an error.
fn parse_id_list(text: &str) -> Result<BTreeSet<String>, Error> {
    let mut ids = BTreeSet::new();
    for line in text.lines() {
        let id = line.trim();
        if id.is_empty() || id.starts_with('#') {
            continue;
        }
        if !ids.insert(id.to_owned()) {
            return Err(format!("id list repeats {id}").into());
        }
    }
    Ok(ids)
}

fn load_id_list(path: &Path) -> Result<IdList, Error> {
    let bytes = fs::read(path)?;
    let ids = parse_id_list(std::str::from_utf8(&bytes)?)?;
    Ok(IdList {
        source: json!({
            "path": path.display().to_string(),
            "sha256": uor_r4_training::sha256_bytes(&bytes),
            "ids": ids.len(),
        }),
        ids,
    })
}

fn optional_id_list(args: &Args, key: &str) -> Result<Option<IdList>, Error> {
    args.0
        .get(key)
        .map(|path| load_id_list(Path::new(path)))
        .transpose()
}

/// `ill_posed=` if given, else the embedded list.
fn ill_posed(args: &Args) -> Result<IdList, Error> {
    if let Some(list) = optional_id_list(args, "ill_posed")? {
        return Ok(list);
    }
    let ids = parse_id_list(EMBEDDED_ILL_POSED)?;
    Ok(IdList {
        source: json!({
            "embedded": EMBEDDED_ILL_POSED_PATH,
            "sha256": uor_r4_training::sha256_bytes(EMBEDDED_ILL_POSED.as_bytes()),
            "ids": ids.len(),
        }),
        ids,
    })
}

fn tiers_record(ill: &IdList) -> Value {
    json!({"rule": TIER_RULE, "ill_posed": ill.source})
}

/// The evaluation tier of a panel row (see [`TIER_RULE`]); an unknown
/// category, or a `conv-*` id outside the tier C categories, is an error.
fn tier_of(id: &str, category: &str, ill: &BTreeSet<String>) -> Result<&'static str, Error> {
    if id.starts_with("conv-") {
        if TIER_C_CATEGORIES.contains(&category) {
            return Ok("C");
        }
        return Err(format!("row {id}: {category} is not a tier C category").into());
    }
    match category {
        "heldout_first_turn" if ill.contains(id) => Ok("K-ill-posed"),
        "heldout_first_turn" => Ok("K-clean"),
        "stretch_heldout" => Ok("stretch"),
        c if EVERYDAY_CATEGORIES.contains(&c) => Ok("everyday"),
        _ => Err(format!("row {id}: unknown category {category}").into()),
    }
}

/// Keep only (`keep = true`) or drop (`keep = false`) the listed request ids.
/// Returns the remaining requests and how many listed ids were present.
fn filter_requests(
    requests: Vec<Request>,
    ids: &BTreeSet<String>,
    keep: bool,
) -> (Vec<Request>, usize) {
    let matched = requests.iter().filter(|r| ids.contains(&r.id)).count();
    let kept = requests
        .into_iter()
        .filter(|r| ids.contains(&r.id) == keep)
        .collect();
    (kept, matched)
}

/// Load panel files (each within `load_requests`' limits), reject an id that
/// repeats across files, and drop `exclude`'s ids. Returns the requests and
/// the number of excluded ids that were present.
fn load_panels(
    paths: &[PathBuf],
    exclude: Option<&IdList>,
) -> Result<(Vec<Request>, usize), Error> {
    let mut requests: Vec<Request> = Vec::new();
    for path in paths {
        requests.extend(load_requests(path)?);
    }
    let distinct: BTreeSet<&str> = requests.iter().map(|r| r.id.as_str()).collect();
    if distinct.len() != requests.len() {
        return Err("a request id repeats across the panel files".into());
    }
    let (requests, excluded) = match exclude {
        Some(list) => filter_requests(requests, &list.ids, false),
        None => (requests, 0),
    };
    if requests.is_empty() {
        return Err("no requests remain after the exclusion".into());
    }
    Ok((requests, excluded))
}

fn split_paths(value: &str) -> Vec<PathBuf> {
    value.split(',').map(PathBuf::from).collect()
}

/// `check`: the panel's turn and context check (`check_panel`) against a
/// tokenizer, with no model; prints the worst-case history.
fn check(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(
        arguments,
        &[
            "requests",
            "tokenizer",
            "protocol",
            "context",
            "max_new_tokens",
            "exclude",
            "checks",
            "constants",
        ],
    )?;
    let paths = split_paths(&args.required("requests")?);
    let exclude = optional_id_list(&args, "exclude")?;
    let checks = load_checks(&args)?;
    let constants = constants(&args)?;
    let tokenizer = load_tokenizer(Path::new(&args.required("tokenizer")?))?;
    let protocol =
        DialogueProtocol::literal_roles_version(&tokenizer, args.number("protocol", 2)?)?;
    let encoder = protocol.bind(&tokenizer)?;
    let (context, max_new_tokens): (usize, usize) = (
        args.number("context", 384)?,
        args.number("max_new_tokens", 64)?,
    );
    let (requests, excluded) = load_panels(&paths, exclude.as_ref())?;
    check_panel(&encoder, &requests, context, max_new_tokens)?;
    let unmatched_checks = validate_checks(&checks, &requests)?;
    let check_only = check_only_controls(&requests, &checks, &constants.replies)?;
    let mut worst = (0usize, String::new());
    let mut categories: BTreeMap<&str, usize> = BTreeMap::new();
    let mut history: BTreeMap<String, usize> = BTreeMap::new();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for request in &requests {
        *categories.entry(request.category.as_str()).or_default() += 1;
        if let Some(c) = checks.rows.get(&request.id) {
            *history
                .entry(format!("{:?}", c.history).to_lowercase())
                .or_default() += 1;
            *kinds
                .entry(format!("{}/{}", request.category, c.kind.name()))
                .or_default() += 1;
        }
        let mut longest = 1usize;
        for (turn, user) in request.user_turns.iter().enumerate() {
            longest += encoder.encode_user_prefix(user, turn != 0).tokens.len() + max_new_tokens;
            if turn + 1 < request.user_turns.len() {
                longest += 1;
            }
        }
        if longest > worst.0 {
            worst = (longest, request.id.clone());
        }
    }
    println!(
        "{}",
        json!({
            "requests": requests.len(), "excluded": excluded,
            "multi_turn": requests.iter().filter(|r| r.user_turns.len() > 1).count(),
            "categories": categories, "context": context, "max_new_tokens": max_new_tokens,
            "worst_case_history": {"positions": worst.0, "id": worst.1},
            "check_panel": "pass",
            "checks": checks.source,
            "checked_rows_by_history": history,
            "checked_rows_by_category_and_kind": kinds,
            "checks_without_loaded_request": unmatched_checks,
            "check_only_controls": check_only,
        })
    );
    Ok(())
}

/// The copy controls of an `exact` row: the candidate value (an expected or
/// forbidden term) stated first and the one stated last in the earlier user
/// turns, as replies. A model that copies a stated value of the right type
/// without binding it to the asked key passes about half the rows; `None`
/// for any other row.
fn copy_replies(check: &RowCheck, users: &[&str]) -> Option<[String; 2]> {
    let CheckKind::Exact(expected) = &check.kind else {
        return None;
    };
    let earlier = &users[..users.len().saturating_sub(1)];
    let mut found: Vec<((usize, usize), &Vec<String>)> = Vec::new();
    for (t, turn) in earlier.iter().enumerate() {
        let turn = words(turn);
        for term in expected.iter().chain(&check.forbid) {
            for (w, window) in turn.windows(term.len()).enumerate() {
                if window == term.as_slice() {
                    found.push(((t, w), term));
                }
            }
        }
    }
    let first = found.iter().min_by_key(|(at, _)| *at)?.1.join(" ");
    let last = found.iter().max_by_key(|(at, _)| *at)?.1.join(" ");
    Some([first, last])
}

/// Check-only tallies of the copy controls over checked rows, plus row-by-row
/// paired McNemar counts against the model (`F4`).
#[derive(Default)]
struct CopyTally {
    first: CheckTally,
    last: CheckTally,
    first_paired: Paired,
    last_paired: Paired,
}

impl CopyTally {
    fn add(&mut self, checks: &Checks, id: &str, users: &[&str]) {
        self.add_with_actual(checks, id, users, None);
    }

    fn add_with_actual(&mut self, checks: &Checks, id: &str, users: &[&str], actual: Option<bool>) {
        let replies = checks
            .rows
            .get(id)
            .and_then(|c| copy_replies(c, users).map(|r| (c, r)));
        match replies {
            Some((check, [first, last])) => {
                let v2 = checks.is_v2();
                let first_pass = check.passes_mode(users, &first, v2);
                let last_pass = check.passes_mode(users, &last, v2);
                self.first.add(Some(first_pass));
                self.last.add(Some(last_pass));
                if let Some(model_ok) = actual {
                    self.first_paired.add(model_ok, first_pass);
                    self.last_paired.add(model_ok, last_pass);
                }
            }
            None => {
                self.first.add(None);
                self.last.add(None);
            }
        }
    }

    fn record(&self) -> Value {
        let has_paired = (self.first_paired.actual_only
            + self.first_paired.control_only
            + self.first_paired.both
            + self.first_paired.neither)
            > 0;
        let mut out = json!({
            "copy_first_stated": self.first.record(),
            "copy_last_stated": self.last.record(),
        });
        if has_paired {
            out["paired_first_stated"] = self.first_paired.record();
            out["paired_last_stated"] = self.last_paired.record();
        }
        out
    }
}

/// The binding-swap replies of the conversational-v3 and -v4 memory rows
/// (`id reply` per line): each names a stated value bound to the wrong key, so
/// each must fail its row's `exact` check. The files' ids are disjoint.
const EMBEDDED_SWAPS: [(&str, &str); 3] = [
    (
        "data/panels/conversational-v3-swaps.tsv",
        include_str!("../../../../data/panels/conversational-v3-swaps.tsv"),
    ),
    (
        "data/panels/conversational-v4-swaps.tsv",
        include_str!("../../../../data/panels/conversational-v4-swaps.tsv"),
    ),
    (
        "data/panels/conversational-v5-swaps.tsv",
        include_str!("../../../../data/panels/conversational-v5-swaps.tsv"),
    ),
];

/// The binding-swap replies by row id (the union of the embedded files).
fn swap_replies() -> Result<BTreeMap<String, String>, Error> {
    let mut out = BTreeMap::new();
    for (path, text) in EMBEDDED_SWAPS {
        for (number, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((id, reply)) = line.split_once('\t') else {
                return Err(format!("{path} line {}: expected id and reply", number + 1).into());
            };
            if out.insert(id.to_owned(), reply.to_owned()).is_some() {
                return Err(format!("swaps repeat {id}").into());
            }
        }
    }
    Ok(out)
}

/// Fixed replies that look like abstentions but agree, name a made-up
/// specific or assert an answer after "but". Applied to every
/// `abstain_exact` row; each must fail every row.
const ADVERSARIAL_ABSTENTIONS: [&str; 3] = [
    "Yes! I can't wait to play with you!",
    "I'm not sure. Sam is the one.",
    "I don't know, but they are under the bed.",
];

/// Per category: how many checked rows each check-only reply passes -- the
/// last user turn, all user turns joined, the first- and last-stated
/// candidate values of an exact row, the binding-swap reply of a memory row,
/// the expected value of an exact row (a row passes when each of its expected
/// spellings, as a bare reply, passes; every exact row should), the
/// adversarial abstentions on `abstain_exact` rows, and each constant reply.
/// No grader.
fn check_only_controls(
    requests: &[Request],
    checks: &Checks,
    constants: &[String],
) -> Result<Value, Error> {
    #[derive(Default)]
    struct Row {
        echo_last: CheckTally,
        echo_history: CheckTally,
        copy: CopyTally,
        binding_swap: CheckTally,
        expected_value: CheckTally,
        adversarial: Vec<CheckTally>,
        constants: Vec<CheckTally>,
    }
    let swaps = swap_replies()?;
    let mut per_category: BTreeMap<&str, Row> = BTreeMap::new();
    for request in requests {
        let entry = per_category.entry(request.category.as_str()).or_default();
        entry
            .constants
            .resize_with(constants.len(), CheckTally::default);
        let users: Vec<&str> = request.user_turns.iter().map(String::as_str).collect();
        let last = users.last().copied().unwrap_or("");
        entry.echo_last.add(checks.of(&request.id, &users, last));
        entry
            .echo_history
            .add(checks.of(&request.id, &users, &users.join(" ")));
        entry.copy.add(checks, &request.id, &users);
        entry.binding_swap.add(
            swaps
                .get(&request.id)
                .and_then(|swap| checks.of(&request.id, &users, swap)),
        );
        entry
            .expected_value
            .add(checks.rows.get(&request.id).and_then(|c| {
                let CheckKind::Exact(terms) = &c.kind else {
                    return None;
                };
                Some(
                    terms
                        .iter()
                        .all(|t| c.passes(&users, &format!("{}.", t.join(" ")))),
                )
            }));
        entry
            .adversarial
            .resize_with(ADVERSARIAL_ABSTENTIONS.len(), CheckTally::default);
        let abstain_exact = checks
            .rows
            .get(&request.id)
            .is_some_and(|c| c.kind == CheckKind::AbstainExact);
        for (tally, reply) in entry.adversarial.iter_mut().zip(ADVERSARIAL_ABSTENTIONS) {
            tally.add(
                abstain_exact
                    .then(|| checks.of(&request.id, &users, reply))
                    .flatten(),
            );
        }
        for (tally, constant) in entry.constants.iter_mut().zip(constants) {
            tally.add(checks.of(&request.id, &users, constant));
        }
    }
    Ok(per_category
        .iter()
        .map(|(k, r)| {
            (
                (*k).to_owned(),
                json!({
                    "echo_last": r.echo_last.record(),
                    "echo_history": r.echo_history.record(),
                    "copy": r.copy.record(),
                    "binding_swap": r.binding_swap.record(),
                    "expected_value": r.expected_value.record(),
                    "adversarial_abstentions": ADVERSARIAL_ABSTENTIONS
                        .iter()
                        .zip(&r.adversarial)
                        .map(|(reply, t)| json!({"reply": reply, "result": t.record()}))
                        .collect::<Vec<_>>(),
                    "constants": r.constants.iter().map(CheckTally::record).collect::<Vec<_>>(),
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>()
        .into())
}

/// `filter`: write a new panel file holding the requests of `requests` minus
/// `exclude` (or only `only`). The inputs are never modified.
fn filter(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["out", "requests", "exclude", "only"])?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    let paths = split_paths(&args.required("requests")?);
    let (list, keep) = match (
        optional_id_list(&args, "exclude")?,
        optional_id_list(&args, "only")?,
    ) {
        (Some(list), None) => (list, false),
        (None, Some(list)) => (list, true),
        _ => return Err("give exactly one of exclude= or only=".into()),
    };
    let (requests, matched) = filter_requests(load_panels(&paths, None)?.0, &list.ids, keep);
    if requests.is_empty() {
        return Err("the filter leaves no requests".into());
    }
    fs::write(&out, serde_json::to_vec_pretty(&requests)?)?;
    println!(
        "{} requests written ({} of {} listed ids present); {} sha256 {}",
        requests.len(),
        matched,
        list.ids.len(),
        out.display(),
        sha256_file(&out)?
    );
    Ok(())
}

/// The graded rows of a `judge_panel` record or a report.
fn judged_rows(record: &Value) -> Result<&[Value], Error> {
    Ok(record["rows"]
        .as_array()
        .ok_or("a graded record without rows")?)
}

/// The fluent/relevant verdicts of a graded row's `grades` or `control_grades`.
fn row_grades(record: &Value) -> Result<Grades, Error> {
    let verdict = |key: &str| -> Result<Option<bool>, Error> {
        match record.get(key) {
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(Value::Null) => Ok(None),
            _ => Err(format!("a graded row without a {key} verdict").into()),
        }
    };
    Ok(Grades {
        fluent: verdict("fluent")?,
        relevant: verdict("relevant")?,
        deterministic: record.get("status").is_some(),
        raw: [String::new(), String::new()],
    })
}

fn row_field<'a>(row: &'a Value, key: &str) -> Result<&'a str, Error> {
    row[key]
        .as_str()
        .ok_or_else(|| format!("a graded row without {key}").into())
}

/// A graded row as the summaries read it.
struct RowView<'a> {
    id: &'a str,
    category: &'a str,
    users: Vec<&'a str>,
    reply: &'a str,
    grades: Grades,
    control: Grades,
    constants: Vec<Grades>,
}

/// The graded rows of a report as [`RowView`]s, each with exactly
/// `constants` constant grades (none in a report graded without constants).
fn row_views(rows: &[Value], constants: usize) -> Result<Vec<RowView<'_>>, Error> {
    rows.iter()
        .map(|row| {
            let id = row_field(row, "id")?;
            let reply = row["conversation"]
                .as_array()
                .and_then(|c| c.last())
                .and_then(|t| t["assistant"].as_str())
                .ok_or_else(|| format!("row {id} without a last reply"))?;
            let constant_grades = match row.get("constant_grades") {
                None => Vec::new(),
                Some(Value::Array(grades)) => {
                    grades.iter().map(row_grades).collect::<Result<_, _>>()?
                }
                Some(_) => return Err(format!("row {id}: constant_grades is not a list").into()),
            };
            if constant_grades.len() != constants {
                return Err(format!(
                    "row {id} has {} constant grades, the report {constants} constant replies",
                    constant_grades.len()
                )
                .into());
            }
            Ok(RowView {
                id,
                category: row_field(row, "category")?,
                users: row_users(row)?,
                reply,
                grades: row_grades(&row["grades"])?,
                control: row_grades(&row["control_grades"])?,
                constants: constant_grades,
            })
        })
        .collect()
}

/// Actual, derangement, constant and check-only tallies over a set of rows.
#[derive(Default)]
struct Accumulator {
    actual: Tally,
    control: Tally,
    acceptable: Paired,
    relevant: Paired,
    /// Per constant reply: its tally, acceptable paired against actual, and
    /// check pass paired against actual (checked rows only).
    constants: Vec<ConstantControl>,
    /// Check pass of the actual replies on checked rows, paired with the
    /// check-only controls below.
    echo_last: CheckTally,
    echo_history: CheckTally,
    copy: CopyTally,
}

/// One constant reply's grades over a set of rows, paired with the actual
/// replies on the same rows.
#[derive(Default)]
struct ConstantControl {
    tally: Tally,
    /// `actual_only` = the model acceptable and the constant not.
    acceptable: Paired,
    /// The same for the check alone (checked rows only; no grader).
    check: Paired,
}

const BEST_CONSTANT_RULE: &str = "per category, the constant reply with the most acceptable rows \
     (ties to the lower index) is that category's best constant; the model's acceptable rows are \
     paired with it on the same rows (two-sided exact McNemar on the discordant rows). A category \
     is a headline only when the model is acceptable on more discordant rows than its best \
     constant with p < 0.05. Choosing the best constant after grading favours the constant, so the \
     test is conservative for the model. A tier total is never a headline";

impl Accumulator {
    /// Add `view`; `next` is the row whose conversation its derangement
    /// control was graded on (so the control is checked with `next`'s check).
    fn add(&mut self, view: &RowView, next: &RowView, checks: &Checks, constants: &[String]) {
        let check = checks.of(view.id, &view.users, view.reply);
        let control_check = checks.of(next.id, &next.users, view.reply);
        let ok = acceptable(&view.grades, check);
        self.actual.add(&view.grades, check);
        self.control.add(&view.control, control_check);
        self.acceptable
            .add(ok, acceptable(&view.control, control_check));
        self.relevant.add(
            view.grades.relevant == Some(true),
            view.control.relevant == Some(true),
        );
        self.constants
            .resize_with(view.constants.len(), Default::default);
        for (control, (grades, text)) in self
            .constants
            .iter_mut()
            .zip(view.constants.iter().zip(constants))
        {
            let constant_check = checks.of(view.id, &view.users, text);
            control.tally.add(grades, constant_check);
            control
                .acceptable
                .add(ok, acceptable(grades, constant_check));
            if let (Some(model), Some(constant)) = (check, constant_check) {
                control.check.add(model, constant);
            }
        }
        self.echo_last.add(checks.of(
            view.id,
            &view.users,
            view.users.last().copied().unwrap_or(""),
        ));
        self.echo_history
            .add(checks.of(view.id, &view.users, &view.users.join(" ")));
        self.copy
            .add_with_actual(checks, view.id, &view.users, Some(ok));
    }

    /// The best constant of these rows (most acceptable, ties to the lower
    /// index) and the model paired against it; `None` without constants.
    fn best_constant(&self, constants: &[String]) -> Option<Value> {
        let (index, best) = self
            .constants
            .iter()
            .enumerate()
            .rev()
            .max_by_key(|(_, c)| c.tally.acceptable)?;
        let paired = &best.acceptable;
        let p = mcnemar_exact(paired.actual_only, paired.control_only);
        let check_rows =
            best.check.actual_only + best.check.control_only + best.check.both + best.check.neither;
        Some(json!({
            "index": index,
            "reply": constants.get(index),
            "rows": self.actual.replies,
            "model_acceptable": self.actual.acceptable,
            "constant_acceptable": best.tally.acceptable,
            "model_minus_constant": self.actual.acceptable as i64 - best.tally.acceptable as i64,
            "model_only": paired.actual_only,
            "constant_only": paired.control_only,
            "mcnemar_exact_p": p,
            "model_beats_constant": paired.actual_only > paired.control_only && p < 0.05,
            "check_pass": (check_rows > 0).then(|| json!({
                "checked_rows": check_rows,
                "model": best.check.actual_only + best.check.both,
                "constant": best.check.control_only + best.check.both,
                "model_only": best.check.actual_only,
                "constant_only": best.check.control_only,
                "mcnemar_exact_p": mcnemar_exact(best.check.actual_only, best.check.control_only),
            })),
        }))
    }

    /// The strongest baseline across both constant replies and (for `exact`
    /// memory rows) `copy_first_stated` and `copy_last_stated` (`F3`/`F4`).
    fn best_control_or_copy(&self, constants: &[String]) -> Option<Value> {
        struct Candidate<'a> {
            kind: &'static str,
            label: String,
            acceptable: usize,
            paired: &'a Paired,
        }
        let mut candidates: Vec<Candidate<'_>> = self
            .constants
            .iter()
            .enumerate()
            .map(|(i, c)| Candidate {
                kind: "constant",
                label: constants.get(i).cloned().unwrap_or_default(),
                acceptable: c.tally.acceptable,
                paired: &c.acceptable,
            })
            .collect();
        if self.copy.first.checked > 0 {
            candidates.push(Candidate {
                kind: "copy_first_stated",
                label: "copy_first_stated".to_owned(),
                acceptable: self.copy.first.pass,
                paired: &self.copy.first_paired,
            });
            candidates.push(Candidate {
                kind: "copy_last_stated",
                label: "copy_last_stated".to_owned(),
                acceptable: self.copy.last.pass,
                paired: &self.copy.last_paired,
            });
        }
        let best = candidates
            .iter()
            .rev()
            .max_by_key(|c| (c.acceptable, c.paired.both))?;
        let p = mcnemar_exact(best.paired.actual_only, best.paired.control_only);
        Some(json!({
            "kind": best.kind,
            "control": best.label,
            "rows": self.actual.replies,
            "model_acceptable": self.actual.acceptable,
            "control_acceptable": best.acceptable,
            "model_minus_control": self.actual.acceptable as i64 - best.acceptable as i64,
            "model_only": best.paired.actual_only,
            "control_only": best.paired.control_only,
            "both": best.paired.both,
            "neither": best.paired.neither,
            "mcnemar_exact_p": p,
            "model_beats_best_control_or_copy": best.paired.actual_only > best.paired.control_only && p < 0.05,
        }))
    }

    fn record(&self, constants: &[String]) -> Value {
        json!({
            "actual": self.actual.record(),
            "control_derangement": self.control.record(),
            "paired_against_control": {
                "acceptable": self.acceptable.record(),
                "relevant": self.relevant.record(),
                "rule": "a reading counts only if the actual replies beat the derangement control \
                         on the same requests with a two-sided exact McNemar p < 0.05",
            },
            "control_constants": self.constants.iter().zip(constants).map(|(c, text)| json!({
                "reply": text,
                "grades": c.tally.record(),
                "paired_acceptable": c.acceptable.record(),
                "paired_check": c.check.record(),
            })).collect::<Vec<_>>(),
            "best_constant": self.best_constant(constants),
            "best_control_or_copy": self.best_control_or_copy(constants),
            "check_only_controls": {
                "echo_last": self.echo_last.record(),
                "echo_history": self.echo_history.record(),
                "copy": self.copy.record(),
            },
        })
    }
}

/// An accumulator over a group of rows and over each category in it.
#[derive(Default)]
struct Group {
    all: Accumulator,
    per_category: BTreeMap<String, Accumulator>,
}

impl Group {
    fn record(&self, constants: &[String]) -> Value {
        let mut record = self.all.record(constants);
        let per_category: serde_json::Map<String, Value> = self
            .per_category
            .iter()
            .map(|(k, a)| (k.clone(), a.record(constants)))
            .collect();
        record["per_category"] = Value::Object(per_category);
        record["headline"] = self.headline(constants);
        record
    }

    /// Per category: the model against that category's best constant, and the
    /// categories where the model beats it (plus `best_control_or_copy` under
    /// `F3`/`F4`). Never a total.
    fn headline(&self, constants: &[String]) -> Value {
        if constants.is_empty() {
            return json!({"unavailable": "graded without constant controls", "rule": BEST_CONSTANT_RULE});
        }
        let mut categories = serde_json::Map::new();
        let mut categories_control_or_copy = serde_json::Map::new();
        let mut beats = Vec::new();
        let mut beats_control_or_copy = Vec::new();
        for (name, accumulator) in &self.per_category {
            let Some(best) = accumulator.best_constant(constants) else {
                continue;
            };
            if best["model_beats_constant"] == true {
                beats.push(name.clone());
            }
            categories.insert(name.clone(), best);
            if let Some(best_cc) = accumulator.best_control_or_copy(constants) {
                if best_cc["model_beats_best_control_or_copy"] == true {
                    beats_control_or_copy.push(name.clone());
                }
                categories_control_or_copy.insert(name.clone(), best_cc);
            }
        }
        json!({
            "rule": BEST_CONSTANT_RULE,
            "per_category": categories,
            "per_category_vs_control_or_copy": categories_control_or_copy,
            "categories_model_beats_best_constant": beats,
            "categories_model_beats_best_control_or_copy": beats_control_or_copy,
        })
    }
}

/// Group graded rows by `key` and accumulate each group (and each category
/// within it). A row's derangement neighbour is the next row in panel order.
fn grouped(
    rows: &[Value],
    checks: &Checks,
    constants: &[String],
    key: impl Fn(&RowView) -> Result<&'static str, Error>,
) -> Result<BTreeMap<&'static str, Group>, Error> {
    let views = row_views(rows, constants.len())?;
    let mut groups: BTreeMap<&'static str, Group> = BTreeMap::new();
    for (i, view) in views.iter().enumerate() {
        let next = &views[(i + 1) % views.len()];
        let group = groups.entry(key(view)?).or_default();
        group.all.add(view, next, checks, constants);
        group
            .per_category
            .entry(view.category.to_owned())
            .or_default()
            .add(view, next, checks, constants);
    }
    Ok(groups)
}

/// The summary of all graded rows, with per-category accumulators.
fn summarize(rows: &[Value], checks: &Checks, constants: &[String]) -> Result<Value, Error> {
    let groups = grouped(rows, checks, constants, |_| Ok("all"))?;
    groups
        .get("all")
        .map(|g| g.record(constants))
        .ok_or_else(|| "no graded rows".into())
}

/// Per tier: the actual, derangement and constant tallies, the paired
/// against-control counts, the check-only controls and the same per category.
fn tier_summary(
    rows: &[Value],
    ill: &BTreeSet<String>,
    checks: &Checks,
    constants: &[String],
) -> Result<Value, Error> {
    let groups = grouped(rows, checks, constants, |v| tier_of(v.id, v.category, ill))?;
    Ok(Value::Object(
        groups
            .iter()
            .map(|(k, g)| ((*k).to_owned(), g.record(constants)))
            .collect(),
    ))
}

/// The constant replies a graded report was graded with (none in a report
/// from before constants existed).
fn report_constants(report: &Value) -> Result<Vec<String>, Error> {
    match report.get("constants") {
        None => Ok(Vec::new()),
        Some(c) => c["replies"]
            .as_array()
            .ok_or("report constants without replies")?
            .iter()
            .map(|r| {
                r.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "a constant reply is not a string".into())
            })
            .collect(),
    }
}

fn read_report(path: &Path) -> Result<(Value, String), Error> {
    let bytes = fs::read(path)?;
    let report: Value = serde_json::from_slice(&bytes)?;
    if report["schema"] != "uor-r4.chat-grade/1" {
        return Err(format!("{} is not a chat-grade report", path.display()).into());
    }
    Ok((report, uor_r4_training::sha256_bytes(&bytes)))
}

/// Write `name` into a newly claimed report root, then seal and verify it. A
/// failed write returns before sealing, so a sealed root always holds the
/// record.
fn write_sealed(out: &Path, name: &str, record: &Value) -> Result<(), Error> {
    report_output::claim(out)?;
    fs::write(out.join(name), serde_json::to_vec_pretty(record)?)?;
    report_output::seal(out)?;
    report_output::verify(out)?;
    Ok(())
}

/// `tiers`: the per-tier summary of an existing graded report, without
/// regrading.
fn tiers(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["report", "ill_posed", "checks", "out"])?;
    let path = PathBuf::from(args.required("report")?);
    let ill = ill_posed(&args)?;
    let checks = load_checks(&args)?;
    let (report, sha256) = read_report(&path)?;
    let constants = report_constants(&report)?;
    let per_tier = tier_summary(judged_rows(&report)?, &ill.ids, &checks, &constants)?;
    let record = json!({
        "schema": "uor-r4.chat-grade-tiers/1",
        "report": {"path": path.display().to_string(), "sha256": sha256, "grader": report["grader"]},
        "tiers": tiers_record(&ill),
        "checks": checks_record(&checks),
        "constants": constants,
        "headline": headline_of(&per_tier),
        "per_tier": per_tier,
    });
    if let Some(out) = args.0.get("out") {
        write_sealed(Path::new(out), "tiers.json", &record)?;
    }
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

/// Paired verdicts of report `a` against report `b` on the same requests
/// (`Paired::actual_only` is `a` only, `control_only` is `b` only).
#[derive(Default)]
struct Compared {
    acceptable: Paired,
    fluent: Paired,
    relevant: Paired,
    /// Check passes, on checked rows only.
    check: Paired,
    /// Per category: acceptable and check pass, paired.
    per_category: BTreeMap<String, (Paired, Paired)>,
}

impl Compared {
    /// Add one paired row: each side's grades and check result (one check
    /// applies to both sides, so both results are `None` or both `Some`).
    fn add(&mut self, category: &str, a: (&Grades, Option<bool>), b: (&Grades, Option<bool>)) {
        let (ok_a, ok_b) = (acceptable(a.0, a.1), acceptable(b.0, b.1));
        self.acceptable.add(ok_a, ok_b);
        self.fluent
            .add(a.0.fluent == Some(true), b.0.fluent == Some(true));
        self.relevant
            .add(a.0.relevant == Some(true), b.0.relevant == Some(true));
        let entry = self.per_category.entry(category.to_owned()).or_default();
        entry.0.add(ok_a, ok_b);
        if let (Some(x), Some(y)) = (a.1, b.1) {
            self.check.add(x, y);
            entry.1.add(x, y);
        }
    }

    fn rows(&self) -> u64 {
        let p = &self.acceptable;
        p.actual_only + p.control_only + p.both + p.neither
    }

    fn record(&self) -> Value {
        let pair = |p: &Paired| {
            json!({
                "rows": p.actual_only + p.control_only + p.both + p.neither,
                "a": p.actual_only + p.both, "b": p.control_only + p.both,
                "a_only": p.actual_only, "b_only": p.control_only,
                "both": p.both, "neither": p.neither,
                "mcnemar_exact_p": mcnemar_exact(p.actual_only, p.control_only),
            })
        };
        let per_category: BTreeMap<String, Value> = self
            .per_category
            .iter()
            .map(|(k, (acceptable, check))| {
                let mut record = pair(acceptable);
                record["check_pass"] = pair(check);
                (k.clone(), record)
            })
            .collect();
        json!({
            "acceptable": pair(&self.acceptable),
            "fluent": pair(&self.fluent),
            "relevant": pair(&self.relevant),
            "check_pass": pair(&self.check),
            "per_category_acceptable": per_category,
            "per_category_rule": "per category, a against b paired by row: acceptable (and \
                                  check pass on checked rows) with the two-sided exact McNemar; \
                                  read categories, not the tier total",
        })
    }
}

/// The user turns of a graded row (its conversation's `user` fields).
fn row_users(row: &Value) -> Result<Vec<&str>, Error> {
    row["conversation"]
        .as_array()
        .ok_or("a graded row without conversation")?
        .iter()
        .map(|t| {
            t["user"]
                .as_str()
                .ok_or_else(|| "a turn without user".into())
        })
        .collect()
}

fn index_rows(rows: &[Value]) -> Result<BTreeMap<&str, &Value>, Error> {
    let mut index = BTreeMap::new();
    for row in rows {
        let id = row_field(row, "id")?;
        if index.insert(id, row).is_some() {
            return Err(format!("a report repeats row {id}").into());
        }
    }
    Ok(index)
}

/// Pair the rows of two graded reports by id within each tier (or only
/// `tier`): an id in both must carry the same category and user turns. Rows in
/// only one report are counted as unpaired and left out of the test.
fn compare_rows(
    a: &[Value],
    b: &[Value],
    ill: &BTreeSet<String>,
    checks: &Checks,
    tier: Option<&str>,
) -> Result<Value, Error> {
    let (a, b) = (index_rows(a)?, index_rows(b)?);
    let mut tiers: BTreeMap<&'static str, Compared> = BTreeMap::new();
    let mut all = Compared::default();
    let mut unpaired: BTreeMap<&'static str, [u64; 2]> = BTreeMap::new();
    let last_reply = |row: &Value| -> Result<String, Error> {
        Ok(row["conversation"]
            .as_array()
            .and_then(|c| c.last())
            .and_then(|t| t["assistant"].as_str())
            .ok_or("a graded row without a last reply")?
            .to_owned())
    };
    for (side, rows, other) in [(0usize, &a, &b), (1, &b, &a)] {
        for (id, row) in rows {
            let category = row_field(row, "category")?;
            let name = tier_of(id, category, ill)?;
            if tier.is_some_and(|t| t != name) {
                continue;
            }
            let Some(other_row) = other.get(id) else {
                unpaired.entry(name).or_default()[side] += 1;
                continue;
            };
            if side == 1 {
                continue;
            }
            if row_field(other_row, "category")? != category
                || row_users(row)? != row_users(other_row)?
            {
                return Err(format!(
                    "row {id} has a different category or user turns in the two reports"
                )
                .into());
            }
            let (ga, gb) = (
                row_grades(&row["grades"])?,
                row_grades(&other_row["grades"])?,
            );
            let users = row_users(row)?;
            let (ca, cb) = (
                checks.of(id, &users, &last_reply(row)?),
                checks.of(id, &users, &last_reply(other_row)?),
            );
            tiers
                .entry(name)
                .or_default()
                .add(category, (&ga, ca), (&gb, cb));
            all.add(category, (&ga, ca), (&gb, cb));
        }
    }
    if all.rows() == 0 {
        return Err("the two reports share no rows in the selected tier".into());
    }
    let per_tier: BTreeMap<String, Value> = tiers
        .iter()
        .map(|(k, c)| (k.to_string(), c.record()))
        .collect();
    let unpaired: BTreeMap<String, Value> = unpaired
        .iter()
        .map(|(k, [x, y])| (k.to_string(), json!({"a_rows": x, "b_rows": y})))
        .collect();
    Ok(json!({
        "per_tier": per_tier,
        "all_selected": all.record(),
        "unpaired": unpaired,
        "rule": "rows paired by id within a tier; a_only = acceptable (or fluent, relevant, \
                 check pass) in a but not in b; acceptable includes the row's frozen check; \
                 two-sided exact McNemar on the discordant counts; rows present in only one \
                 report are excluded and counted under unpaired",
    }))
}

/// `compare`: paired McNemar of two graded reports, per tier.
fn compare(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["a", "b", "tier", "ill_posed", "checks", "out"])?;
    let path_a = PathBuf::from(args.required("a")?);
    let path_b = PathBuf::from(args.required("b")?);
    let ill = ill_posed(&args)?;
    let checks = load_checks(&args)?;
    let tier = args
        .0
        .get("tier")
        .map(String::as_str)
        .filter(|t| *t != "all");
    if let Some(t) = tier {
        if !TIERS.contains(&t) {
            return Err(format!("unknown tier {t}; one of all, {}", TIERS.join(", ")).into());
        }
    }
    let (a, sha_a) = read_report(&path_a)?;
    let (b, sha_b) = read_report(&path_b)?;
    let compared = compare_rows(judged_rows(&a)?, judged_rows(&b)?, &ill.ids, &checks, tier)?;
    let same_grader = a["grader"]["model"] == b["grader"]["model"]
        && a["grader"]["digest"] == b["grader"]["digest"];
    let source = |r: &Value| r.get("model").or_else(|| r.get("replies")).cloned();
    let record = json!({
        "schema": "uor-r4.chat-grade-compare/1",
        "a": {"path": path_a.display().to_string(), "sha256": sha_a, "grader": a["grader"], "source": source(&a)},
        "b": {"path": path_b.display().to_string(), "sha256": sha_b, "grader": b["grader"], "source": source(&b)},
        "same_grader": same_grader,
        "tier": tier.unwrap_or("all"),
        "tiers": tiers_record(&ill),
        "checks": checks_record(&checks),
        "compared": compared,
    });
    if !same_grader {
        eprintln!("chat-grade: warning: the two reports were graded by different graders");
    }
    if let Some(out) = args.0.get("out") {
        write_sealed(Path::new(out), "compare.json", &record)?;
    }
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tier K: the text-only missing-material rule.

/// The single criterion of tier K-ill-posed, read from the request text alone
/// and never from a model's reply or grade.
const MISSING_MATERIAL_RULE: &str = "a heldout request is ill-posed when its text refers to \
     material that the text does not contain. The text shows this by one of: open_end (it ends \
     with ':' or ',', so the material it introduces is absent -- the panel keeps only a turn's \
     first line); slot (an unfilled [placeholder] that is neither defined in the text nor a label \
     after ':'); constraints_only (every sentence constrains 'your response' to a query the text \
     does not hold); deictic (a pointer such as 'the following', 'below', 'this essay', 'the \
     passage', 'the recipe', 'your suggestion', or a clause-final 'this' in a one-sentence \
     request, with the material not included); own_material ('I wrote', 'here is', ... with the \
     material not included); variable (a bare x, y or n with no equation). Material counts as \
     included when the text holds a quoted span of at least 3 words, or at least 25 words or 3 \
     sentences after its first ':'. A fragment that refers to nothing missing (a greeting, a \
     headline) is clean under this rule";

const CONSTRAINT_OPENERS: [&str; 17] = [
    "your response",
    "your answer",
    "the response",
    "your entire response",
    "paragraphs are",
    "there should be",
    "do not include",
    "include keywords",
    "include the keywords",
    "the keywords are",
    "answer with",
    "use the markdown",
    "highlight at least",
    "wrap your",
    "finish your response",
    "in your response",
    "your reply",
];
const MATERIAL_NOUNS: [&str; 35] = [
    "text",
    "passage",
    "essay",
    "code",
    "snippet",
    "paragraph",
    "paragraphs",
    "article",
    "argument",
    "bio",
    "poem",
    "introduction",
    "description",
    "draft",
    "document",
    "data",
    "figures",
    "table",
    "recipe",
    "email",
    "letter",
    "query",
    "question",
    "message",
    "report",
    "excerpt",
    "story",
    "list",
    "sentence",
    "sentences",
    "function",
    "program",
    "script",
    "equation",
    "problem",
];
const MATERIAL_DETERMINERS: [&str; 9] = [
    "this",
    "these",
    "the following",
    "the above",
    "the attached",
    "the given",
    "following",
    "below",
    "above",
];
/// Definite references that need an antecedent a single turn cannot have.
const DEFINITE_REFERENCES: [(&str, &[&str]); 3] = [
    (
        "the",
        &[
            "text",
            "passage",
            "essay",
            "code",
            "snippet",
            "recipe",
            "region",
            "protagonist",
            "argument",
            "article",
            "poem",
            "bio",
            "draft",
        ],
    ),
    (
        "your",
        &["suggestion", "suggestions", "supplies", "edit", "draft"],
    ),
    ("this", &["particular"]),
];
const MATERIAL_POINTERS: [&str; 4] = ["below", "above", "as follows", "the following"];
const OWN_MATERIAL: [&str; 7] = [
    "i wrote",
    "i've written",
    "i'd written",
    "i have written",
    "here's",
    "here is",
    "i've got",
];
const AFTER_CLAUSE_FINAL_THIS: [&str; 6] = ["but", "and", "so", "please", "for", "to"];

/// The sentences of a text: split after `.`, `!` or `?` followed by
/// whitespace; pieces without words are dropped.
fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let chars: Vec<(usize, char)> = text.trim().char_indices().collect();
    let trimmed = text.trim();
    for (i, &(at, c)) in chars.iter().enumerate() {
        let next_space = chars.get(i + 1).is_some_and(|(_, n)| n.is_whitespace());
        if matches!(c, '.' | '!' | '?') && next_space {
            out.push(&trimmed[start..at + c.len_utf8()]);
            start = at + c.len_utf8();
        }
    }
    out.push(&trimmed[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|s| !words(s).is_empty())
        .collect()
}

/// Whether the text includes material: a quoted span (straight or curly
/// quotes) of at least 3 words, or at least 25 words or 3 sentences after its
/// first colon.
fn material_included(text: &str) -> bool {
    for (open, close) in [('"', '"'), ('\u{201c}', '\u{201d}')] {
        let mut rest = text;
        while let Some(i) = rest.find(open) {
            let after = &rest[i + open.len_utf8()..];
            let Some(j) = after.find(close) else {
                break;
            };
            if words(&after[..j]).len() >= 3 {
                return true;
            }
            rest = &after[j + close.len_utf8()..];
        }
    }
    text.split_once(':')
        .is_some_and(|(_, after)| words(after).len() >= 25 || sentences(after).len() >= 3)
}

/// The unfilled `[slot]` placeholders of a text: a bracketed lowercase name
/// that is not a label right after ':' and is not defined (`[x] are`,
/// `[x] is`, `[x]:`) elsewhere in the text.
fn unfilled_slot(text: &str) -> Option<String> {
    let mut rest = text;
    let mut offset = 0;
    while let Some(i) = rest.find('[') {
        let after = &rest[i + 1..];
        let Some(j) = after.find(']') else {
            break;
        };
        let name = &after[..j];
        let start = offset + i;
        offset = start + 1;
        rest = &text[offset..];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            continue;
        }
        let slot = format!("[{name}]");
        let label = text[..start].trim_end().ends_with(':');
        let defined = text.match_indices(&slot).any(|(at, _)| {
            let tail = text[at + slot.len()..].trim_start();
            tail.starts_with("are") || tail.starts_with("is") || tail.starts_with(':')
        });
        if !label && !defined {
            return Some(slot);
        }
    }
    None
}

/// The missing-material reason of a request text ([`MISSING_MATERIAL_RULE`]):
/// the family and the cue that matched, or `None` for a clean request.
fn missing_material(text: &str) -> Option<(&'static str, String)> {
    let trimmed = text.trim_end();
    if let Some(end) = trimmed.chars().last().filter(|c| matches!(c, ':' | ',')) {
        return Some(("open_end", end.to_string()));
    }
    if let Some(slot) = unfilled_slot(text) {
        return Some(("slot", slot));
    }
    let all = sentences(text);
    if !all.is_empty()
        && all.iter().all(|s| {
            let joined = words(s).join(" ");
            s.trim_start().starts_with('[')
                || CONSTRAINT_OPENERS.iter().any(|o| joined.starts_with(o))
        })
    {
        return Some(("constraints_only", all[0].to_owned()));
    }
    let text_words = words(text);
    let has = |phrase: &[String]| contains_phrase(&text_words, phrase);
    if !material_included(text) {
        for determiner in MATERIAL_DETERMINERS {
            for noun in MATERIAL_NOUNS {
                let phrase = words(&format!("{determiner} {noun}"));
                if has(&phrase) {
                    return Some(("deictic", phrase.join(" ")));
                }
            }
        }
        for (determiner, nouns) in DEFINITE_REFERENCES {
            for noun in nouns {
                let phrase = words(&format!("{determiner} {noun}"));
                if has(&phrase) {
                    return Some(("deictic", phrase.join(" ")));
                }
            }
        }
        for pointer in MATERIAL_POINTERS {
            if has(&words(pointer)) {
                return Some(("deictic", pointer.to_owned()));
            }
        }
        if all.len() == 1 {
            for (i, word) in text_words.iter().enumerate() {
                let next = text_words.get(i + 1).map(String::as_str);
                if word == "this" && next.is_none_or(|n| AFTER_CLAUSE_FINAL_THIS.contains(&n)) {
                    let cue = next.map_or_else(|| "this".to_owned(), |n| format!("this {n}"));
                    return Some(("deictic", cue));
                }
            }
        }
        for own in OWN_MATERIAL {
            if has(&words(own)) {
                return Some(("own_material", own.to_owned()));
            }
        }
    }
    let bare_variable = text
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|t| matches!(t, "x" | "y" | "n"));
    if bare_variable
        && !text
            .chars()
            .any(|c| c == '=' || c == '^' || c.is_ascii_digit())
    {
        return Some(("variable", "x/y/n".to_owned()));
    }
    None
}

/// `ill-posed`: apply [`missing_material`] to every request of the panels
/// (single-turn only); print the counts, compare with `expect=` (an id list;
/// any difference is an error) and write the reasons, ill-posed ids and clean
/// ids to new files when asked.
fn ill_posed_command(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(
        arguments,
        &["requests", "expect", "out_tsv", "out_ids", "out_clean"],
    )?;
    let paths = split_paths(&args.required("requests")?);
    let requests = load_panels(&paths, None)?.0;
    let outputs: Vec<PathBuf> = ["out_tsv", "out_ids", "out_clean"]
        .iter()
        .filter_map(|k| args.0.get(*k).map(PathBuf::from))
        .collect();
    if let Some(existing) = outputs.iter().find(|p| p.exists()) {
        return Err(format!("{} exists", existing.display()).into());
    }
    let mut tsv = format!(
        "# id\treason\tcue -- one text-only criterion ({}) over {} requests\n",
        "chat-grade ill-posed; rule in data/panels/README.md",
        requests.len()
    );
    let (mut ill, mut clean) = (BTreeSet::new(), BTreeSet::new());
    let mut families: BTreeMap<&str, usize> = BTreeMap::new();
    for request in &requests {
        let [turn] = &request.user_turns[..] else {
            return Err(format!("{} is not a single-turn request", request.id).into());
        };
        match missing_material(turn) {
            Some((family, cue)) => {
                *families.entry(family).or_default() += 1;
                tsv.push_str(&format!("{}\t{family}\t{cue}\n", request.id));
                ill.insert(request.id.clone());
            }
            None => {
                clean.insert(request.id.clone());
            }
        }
    }
    let list = |ids: &BTreeSet<String>| ids.iter().map(|i| format!("{i}\n")).collect::<String>();
    let mismatch = match args.0.get("expect") {
        Some(path) => {
            let expected = load_id_list(Path::new(path))?.ids;
            let missing: Vec<&String> = expected.difference(&ill).collect();
            let extra: Vec<&String> = ill.difference(&expected).collect();
            Some(json!({"expected_not_found": missing, "found_not_expected": extra}))
        }
        None => None,
    };
    for (key, text) in [
        ("out_tsv", tsv.clone()),
        ("out_ids", list(&ill)),
        ("out_clean", list(&clean)),
    ] {
        if let Some(path) = args.0.get(key) {
            fs::write(path, &text)?;
        }
    }
    println!(
        "{}",
        json!({
            "requests": requests.len(), "ill_posed": ill.len(), "clean": clean.len(),
            "families": families, "rule": MISSING_MATERIAL_RULE, "expect": mismatch,
            "panels": paths.iter().map(|p| json!({"path": p.display().to_string(), "sha256": sha256_file(p).ok()})).collect::<Vec<_>>(),
        })
    );
    if let Some(m) = &mismatch {
        if m["expected_not_found"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
            || m["found_not_expected"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            return Err("the rule's ill-posed ids differ from expect=".into());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Leakage: panel turns against reference texts and training corpora.

/// A reference text as words, `None` marking a `{placeholder}`.
type Pattern = Vec<Option<String>>;

fn pattern(text: &str) -> Pattern {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        out.extend(words(&rest[..open]).into_iter().map(Some));
        out.push(None);
        rest = &rest[open + close + 1..];
    }
    out.extend(words(rest).into_iter().map(Some));
    out
}

/// A template (a reference text with a placeholder) counts as a whole-turn
/// match only with at least this many literal words, so `{x} is {y}.` or
/// `List {n} {x}.` (which fit almost any short sentence) do not.
const MIN_TEMPLATE_LITERALS: usize = 3;

/// Whether `turn` matches `pattern` whole, each placeholder standing for one
/// to four words.
fn template_match(pattern: &[Option<String>], turn: &[String]) -> bool {
    match pattern.split_first() {
        None => turn.is_empty(),
        Some((Some(word), rest)) => turn.first() == Some(word) && template_match(rest, &turn[1..]),
        Some((None, rest)) => (1..=4.min(turn.len())).any(|k| template_match(rest, &turn[k..])),
    }
}

/// The longest run of consecutive words shared by `pattern` and `turn` (a
/// placeholder matches nothing).
fn longest_shared_run(pattern: &[Option<String>], turn: &[String]) -> usize {
    let mut best = 0;
    let mut previous = vec![0usize; turn.len() + 1];
    for word in pattern {
        let mut current = vec![0usize; turn.len() + 1];
        if let Some(word) = word {
            for (j, t) in turn.iter().enumerate() {
                if t == word {
                    current[j + 1] = previous[j] + 1;
                    best = best.max(current[j + 1]);
                }
            }
        }
        previous = current;
    }
    best
}

/// The string literals of a Rust source (line comments skipped; `\n` and `\t`
/// read as spaces; a line-continuation backslash joins lines).
fn rust_string_literals(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '\'' {
            // A char literal such as '"' or '\"' is skipped; a lifetime is not one.
            if chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') {
                i += 4;
            } else if chars.get(i + 2) == Some(&'\'') {
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }
        let identifier_before = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
        if c == 'r' && !identifier_before && matches!(chars.get(i + 1), Some('"') | Some('#')) {
            let mut j = i + 1;
            let mut hashes = 0;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let start = j + 1;
                let mut k = start;
                while k < chars.len()
                    && !(chars[k] == '"' && (0..hashes).all(|h| chars.get(k + 1 + h) == Some(&'#')))
                {
                    k += 1;
                }
                out.push(chars[start..k.min(chars.len())].iter().collect());
                i = k + 1 + hashes;
                continue;
            }
        }
        if c == '"' {
            let mut text = String::new();
            let mut k = i + 1;
            while k < chars.len() && chars[k] != '"' {
                if chars[k] == '\\' {
                    match chars.get(k + 1) {
                        Some('n') | Some('t') => text.push(' '),
                        Some('\n') => {
                            k += 2;
                            while k < chars.len() && chars[k].is_whitespace() {
                                k += 1;
                            }
                            continue;
                        }
                        Some(&escaped) => text.push(escaped),
                        None => {}
                    }
                    k += 2;
                    continue;
                }
                text.push(chars[k]);
                k += 1;
            }
            out.push(text);
            i = k + 1;
            continue;
        }
        i += 1;
    }
    out
}

/// Every string value in a JSON document (keys excluded).
fn json_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| json_strings(v, out)),
        Value::Object(map) => map.values().for_each(|v| json_strings(v, out)),
        _ => {}
    }
}

/// The reference texts of a file: string literals of `.rs`, string values of
/// `.json`, lines (and tab-separated fields) of anything else.
fn reference_texts(path: &Path) -> Result<Vec<String>, Error> {
    let text = fs::read_to_string(path)?;
    Ok(match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => rust_string_literals(&text),
        Some("json") => {
            let mut out = Vec::new();
            json_strings(&serde_json::from_str(&text)?, &mut out);
            out
        }
        _ => text
            .lines()
            .flat_map(|l| l.split('\t'))
            .map(str::to_owned)
            .collect(),
    })
}

/// A corpus line without a leading role label (`User:`, `Assistant:`, a
/// speaker name): up to three words before a colon in the first 24 bytes.
fn strip_role(line: &str) -> &str {
    match line.find(':') {
        Some(colon)
            if colon <= 24
                && !line[..colon].trim().is_empty()
                && line[..colon].split_whitespace().count() <= 3
                && line[..colon]
                    .chars()
                    .all(|c| c.is_alphabetic() || c == ' ' || c == '_') =>
        {
            line[colon + 1..].trim()
        }
        _ => line.trim(),
    }
}

/// What one panel turn shares with the references and corpora.
#[derive(Default)]
struct TurnLeak {
    exact_references: Vec<Value>,
    longest_reference_run: (usize, Value),
    /// The first reference sharing at least its file's run threshold.
    run_leak: Option<Value>,
    corpus_exact: (u64, Option<String>),
    corpus_ngram: (u64, Option<String>),
}

/// The default shared-run threshold of `strict=` references (the M-world
/// sources): a panel turn sharing four consecutive words with an M-world
/// template or phrasing leaks.
const STRICT_LEAK_N: usize = 4;

/// What a panel turn shares with one reference text: whether it matches the
/// text whole (a template's placeholders standing for one to four words, and
/// a template counting only with [`MIN_TEMPLATE_LITERALS`] literal words) and
/// the longest run of consecutive words they share.
fn reference_overlap(pattern: &[Option<String>], turn: &[String]) -> (bool, usize) {
    let literals = pattern.iter().filter(|w| w.is_some()).count();
    let placeholders = pattern.len() - literals;
    let whole = literals > 0
        && (placeholders == 0 || literals >= MIN_TEMPLATE_LITERALS)
        && template_match(pattern, turn);
    (whole, longest_shared_run(pattern, turn))
}

/// `leak`: every user turn of a panel against reference texts (whole-turn or
/// template matches and the longest shared word run) and the lines of prepared
/// corpora (whole-turn matches and shared `corpus_n`-grams). A turn leaks when
/// it matches a reference or corpus line whole, shares at least `n`
/// consecutive words with a `reference=` text or at least `strict_n`
/// (default 4) with a `strict=` text (the M-world sources); corpus n-gram hits
/// are reported.
fn leak(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &[
            "requests",
            "reference",
            "strict",
            "strict_n",
            "whole_only",
            "corpora",
            "tokenizer",
            "n",
            "corpus_n",
            "out",
        ],
    )?;
    let requests = load_panels(&split_paths(&args.required("requests")?), None)?.0;
    let n: usize = args.number("n", 6)?;
    let corpus_n: usize = args.number("corpus_n", 8)?;
    let strict_n: usize = args.number("strict_n", STRICT_LEAK_N)?;
    if n == 0 || corpus_n == 0 || strict_n == 0 {
        return Err("n, strict_n and corpus_n must be positive".into());
    }
    let turns: Vec<Vec<Vec<String>>> = requests
        .iter()
        .map(|r| r.user_turns.iter().map(|t| words(t)).collect())
        .collect();
    let mut leaks: Vec<Vec<TurnLeak>> = turns
        .iter()
        .map(|r| r.iter().map(|_| TurnLeak::default()).collect())
        .collect();
    let mut references = Vec::new();
    let reference_paths = split_paths(&args.required("reference")?);
    let optional_paths = |key: &str| args.0.get(key).map_or_else(Vec::new, |p| split_paths(p));
    let (strict_paths, whole_only_paths) = (optional_paths("strict"), optional_paths("whole_only"));
    for (path, threshold) in reference_paths
        .iter()
        .map(|p| (p, Some(n)))
        .chain(strict_paths.iter().map(|p| (p, Some(strict_n))))
        .chain(whole_only_paths.iter().map(|p| (p, None)))
    {
        let texts = reference_texts(path)?;
        let file = path.display().to_string();
        let mut used = 0usize;
        for text in &texts {
            let pattern = pattern(text);
            if pattern.iter().all(Option::is_none) {
                continue;
            }
            used += 1;
            for (r, row) in turns.iter().enumerate() {
                for (t, turn) in row.iter().enumerate() {
                    let leak = &mut leaks[r][t];
                    let (whole, run) = reference_overlap(&pattern, turn);
                    if whole {
                        leak.exact_references
                            .push(json!({"file": file, "text": text}));
                    }
                    let Some(threshold) = threshold else {
                        continue;
                    };
                    if run > leak.longest_reference_run.0 {
                        leak.longest_reference_run = (run, json!({"file": file, "text": text}));
                    }
                    if run >= threshold && leak.run_leak.is_none() {
                        leak.run_leak = Some(json!({
                            "file": file, "text": text, "words": run, "threshold": threshold,
                        }));
                    }
                }
            }
        }
        references.push(json!({
            "path": file, "sha256": sha256_file(path)?, "texts": used,
            "use": match threshold {
                Some(k) => format!("whole-turn, template and shared runs of {k} words"),
                None => "whole-turn and template only".to_owned(),
            },
        }));
    }
    let mut corpora = Vec::new();
    if let Some(dirs) = args.0.get("corpora") {
        let tokenizer = load_tokenizer(Path::new(&args.required("tokenizer")?))?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
        let mut whole: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
        let mut grams: std::collections::HashMap<String, Vec<(usize, usize)>> =
            std::collections::HashMap::new();
        for (r, row) in turns.iter().enumerate() {
            for (t, turn) in row.iter().enumerate() {
                whole.entry(turn.join(" ")).or_default().push((r, t));
                for window in turn.windows(corpus_n) {
                    let entry = grams.entry(window.join(" ")).or_default();
                    if !entry.contains(&(r, t)) {
                        entry.push((r, t));
                    }
                }
            }
        }
        for dir in split_paths(dirs) {
            let reader = MmapCorpusReader::open(dir.join("tokens.u16"))?;
            let tokens = reader.as_slice();
            let (mut documents, mut lines) = (0u64, 0u64);
            let mut start = 0usize;
            for i in 0..=tokens.len() {
                let boundary = i == tokens.len() || {
                    let id = u32::from(tokens[i]);
                    id == protocol.bos_id || id == protocol.eos_id
                };
                if !boundary {
                    continue;
                }
                if i > start {
                    documents += 1;
                    let ids: Vec<u32> = tokens[start..i].iter().map(|&id| u32::from(id)).collect();
                    for line in tokenizer.decode(&ids).lines() {
                        let text = strip_role(line);
                        let line_words = words(text);
                        if line_words.is_empty() {
                            continue;
                        }
                        lines += 1;
                        if let Some(hits) = whole.get(&line_words.join(" ")) {
                            for &(r, t) in hits {
                                let c = &mut leaks[r][t].corpus_exact;
                                c.0 += 1;
                                c.1.get_or_insert_with(|| format!("{}: {text}", dir.display()));
                            }
                        }
                        for window in line_words.windows(corpus_n) {
                            if let Some(hits) = grams.get(&window.join(" ")) {
                                for &(r, t) in hits {
                                    let c = &mut leaks[r][t].corpus_ngram;
                                    c.0 += 1;
                                    c.1.get_or_insert_with(|| {
                                        format!(
                                            "{}: {}",
                                            dir.display(),
                                            text.chars().take(240).collect::<String>()
                                        )
                                    });
                                }
                            }
                        }
                    }
                }
                start = i + 1;
            }
            corpora.push(json!({
                "path": dir.display().to_string(),
                "tokens_sha256": sha256_file(&dir.join("tokens.u16"))?,
                "documents": documents, "lines": lines,
            }));
        }
    }
    let mut rows = Vec::new();
    let mut leaking = Vec::new();
    let mut corpus_ngram_rows = Vec::new();
    for (r, request) in requests.iter().enumerate() {
        let mut row_leaks = false;
        let mut row_ngram = false;
        let turn_records: Vec<Value> = request
            .user_turns
            .iter()
            .zip(&leaks[r])
            .map(|(turn, leak)| {
                let blocking = !leak.exact_references.is_empty()
                    || leak.corpus_exact.0 > 0
                    || leak.run_leak.is_some();
                row_leaks |= blocking;
                row_ngram |= leak.corpus_ngram.0 > 0;
                json!({
                    "turn": turn,
                    "leaks": blocking,
                    "exact_references": leak.exact_references,
                    "longest_reference_run": {"words": leak.longest_reference_run.0, "reference": leak.longest_reference_run.1},
                    "run_leak": leak.run_leak,
                    "corpus_exact": {"lines": leak.corpus_exact.0, "example": leak.corpus_exact.1},
                    "corpus_ngram": {"lines": leak.corpus_ngram.0, "example": leak.corpus_ngram.1},
                })
            })
            .collect();
        if row_leaks {
            leaking.push(request.id.clone());
        }
        if row_ngram {
            corpus_ngram_rows.push(request.id.clone());
        }
        rows.push(json!({"id": request.id, "category": request.category, "turns": turn_records}));
    }
    let record = json!({
        "schema": "uor-r4.chat-grade-leak/1",
        "rule": format!(
            "a turn leaks when its words (lowercased, split at anything not a letter, digit or \
             apostrophe) equal a reference text's or a corpus line's (role label stripped), a \
             {{placeholder}} standing for 1 to 4 words in a template of at least \
             {MIN_TEMPLATE_LITERALS} literal words, or when it shares {n} or more consecutive \
             words with a reference= text or {strict_n} or more with a strict= text (whole_only= \
             texts take part in whole-turn and template matches only); corpus lines sharing \
             {corpus_n} consecutive words are reported, not counted as leaks"
        ),
        "n": n, "strict_n": strict_n, "corpus_n": corpus_n,
        "requests": requests.len(),
        "references": references,
        "corpora": corpora,
        "leaking_rows": leaking,
        "corpus_ngram_rows": corpus_ngram_rows,
        "rows": rows,
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    if let Some(out) = args.0.get("out") {
        write_sealed(Path::new(out), "leak.json", &record)?;
    }
    println!(
        "{}",
        json!({
            "requests": requests.len(), "leaking_rows": record["leaking_rows"],
            "corpus_ngram_rows": record["corpus_ngram_rows"], "corpora": record["corpora"],
        })
    );
    if !leaking.is_empty() {
        return Err(format!("{} rows leak", leaking.len()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_and_turns_parse() {
        assert_eq!(parse_yes_no("Yes."), Some(true));
        assert_eq!(parse_yes_no(" no, it does not"), Some(false));
        assert_eq!(parse_yes_no("Maybe"), None);
        // Exact McNemar: 8 vs 0 discordant gives 2 / 2^8; balanced gives 1.
        assert!((mcnemar_exact(8, 0) - 2.0 / 256.0).abs() < 1e-12);
        assert!((mcnemar_exact(3, 3) - 1.0).abs() < 1e-12);
        assert!((mcnemar_exact(0, 0) - 1.0).abs() < 1e-12);
        assert!(mcnemar_exact(10, 2) < 0.05 && mcnemar_exact(6, 2) > 0.05);
        assert_eq!(
            first_user_turn("User: Hello there\nAssistant: Hi"),
            Some("Hello there".into())
        );
        assert_eq!(first_user_turn("Assistant: Hi"), None);
    }

    /// `device=` defaults to the CPU, takes three names and never falls back.
    #[test]
    fn the_device_defaults_to_the_cpu_and_never_falls_back() {
        assert_eq!(device_name(None).expect("default"), "cpu");
        assert_eq!(device_name(Some("cpu")).expect("cpu"), "cpu");
        assert_eq!(device_name(Some("cuda")).expect("cuda"), "cuda");
        assert_eq!(device_name(Some("metal")).expect("metal"), "metal");
        for bad in ["gpu", "CUDA", "", "cuda:1"] {
            assert!(device_name(Some(bad)).is_err(), "{bad}");
        }
        let keys = ["out", "device"];
        let args = |given: &[&str]| {
            Args::parse(
                &given.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
                &keys,
            )
            .expect("arguments")
        };
        let (name, device) = args(&["out=r"]).device().expect("cpu by default");
        assert_eq!(name, "cpu");
        assert!(device.is_cpu());
        assert!(args(&["out=r", "device=gpu"]).device().is_err());
        #[cfg(not(feature = "cuda"))]
        assert!(args(&["out=r", "device=cuda"]).device().is_err());
        assert_eq!(device_record("cpu")["tf32"], json!(false));
        // A mode's key list without `device` refuses it.
        assert!(Args::parse(&["device=cpu".to_owned()], &["out"]).is_err());
    }

    fn request(id: &str, category: &str) -> Request {
        Request {
            id: id.into(),
            category: category.into(),
            user_turns: vec![format!("turn of {id}")],
        }
    }

    #[test]
    fn the_oracle_span_is_the_values_last_occurrence() {
        // Token-per-word decode: a token's characters are exactly its word, so
        // the span arithmetic is checked by hand.
        let words = [
            "<s>", "my ", "pet ", "is ", "Bart", ", ", "the ", "cat ", "is ", "Britain", " ",
            "and ", "Britain", " wins",
        ];
        let history: Vec<u32> = (0..words.len() as u32).collect();
        let decode = |ids: &[u32]| {
            ids.iter()
                .map(|&i| words[i as usize])
                .collect::<Vec<_>>()
                .join("")
        };
        // "Britain" is stated twice; the oracle reads the last (token 12).
        assert_eq!(
            find_value_span(&history, &decode, &["Britain".to_string()]),
            Some((12, 13))
        );
        // The frozen checks' terms are lowercase words while the conversation
        // states the value capitalised: the match is case-insensitive.
        assert_eq!(
            find_value_span(&history, &decode, &["britain".to_string()]),
            Some((12, 13))
        );
        // A value whose characters cross a token boundary covers both tokens.
        assert_eq!(
            find_value_span(&history, &decode, &["the cat".to_string()]),
            Some((6, 8))
        );
        // A value that is never stated has no span, so its row runs plain.
        assert_eq!(
            find_value_span(&history, &decode, &["Shelby".to_string()]),
            None
        );
        assert_eq!(
            find_value_span(&history, &decode, &["  ".to_string()]),
            None
        );
    }

    /// A graded row with reply `reply`: `[fluent, relevant]` for the reply,
    /// its derangement control and each constant reply.
    fn graded(
        id: &str,
        category: &str,
        users: &[&str],
        reply: &str,
        actual: [bool; 2],
        control: [bool; 2],
        constants: &[[bool; 2]],
    ) -> Value {
        let mut conversation: Vec<Value> = users
            .iter()
            .map(|u| json!({"user": u, "assistant": "earlier reply"}))
            .collect();
        if let Some(last) = conversation.last_mut() {
            last["assistant"] = json!(reply);
        }
        let grade = |g: [bool; 2]| json!({"fluent": g[0], "relevant": g[1], "raw": ["", ""]});
        json!({
            "id": id, "category": category,
            "conversation": conversation,
            "grades": grade(actual),
            "control_grades": grade(control),
            "constant_grades": constants.iter().map(|g| grade(*g)).collect::<Vec<_>>(),
        })
    }

    fn row(id: &str, category: &str, actual: [bool; 2], control: [bool; 2]) -> Value {
        graded(
            id,
            category,
            &[&format!("turn of {id}")],
            "a reply",
            actual,
            control,
            &[],
        )
    }

    impl Checks {
        /// The check of a reply on a row, with no user turns.
        fn of_reply(&self, id: &str, reply: &str) -> Option<bool> {
            self.of(id, &[], reply)
        }
    }

    fn no_checks() -> Checks {
        Checks {
            rows: BTreeMap::new(),
            source: json!("none"),
        }
    }

    #[test]
    fn id_lists_filter_and_tiers() {
        let ids = parse_id_list("# comment\nheldout-002\n\n  heldout-005  \n").unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("heldout-005"));
        assert!(parse_id_list("a\na\n").is_err());
        let embedded = parse_id_list(EMBEDDED_ILL_POSED).unwrap();
        assert_eq!(embedded.len(), 67);
        // The embedded list is the rule's output, and the clean list is its
        // complement over the 200 rows.
        let clean = parse_id_list(include_str!(
            "../../../../data/panels/heldout-clean-v3-ids.txt"
        ))
        .unwrap();
        assert_eq!(clean.len(), 133);
        assert!(clean.is_disjoint(&embedded));
        let reasons = include_str!("../../../../data/panels/heldout-ill-posed-v3.tsv");
        let listed: BTreeSet<String> = reasons
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| l.split('\t').next().unwrap().to_owned())
            .collect();
        assert_eq!(listed, embedded);
        assert!(embedded.contains("heldout-008") && !embedded.contains("heldout-000"));
        // One criterion: twins in form share a tier.
        for id in [
            "heldout-002",
            "heldout-117",
            "heldout-144",
            "heldout-030",
            "heldout-128",
        ] {
            assert!(embedded.contains(id), "{id}");
        }

        let panel = vec![
            request("talk-01", "smalltalk"),
            request("heldout-002", "heldout_first_turn"),
            request("heldout-003", "heldout_first_turn"),
            request("heldout-005", "heldout_first_turn"),
        ];
        let (kept, matched) = filter_requests(panel.clone(), &ids, false);
        assert_eq!(matched, 2);
        let kept: Vec<&str> = kept.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(kept, ["talk-01", "heldout-003"]);
        let (only, _) = filter_requests(panel, &ids, true);
        assert_eq!(only.len(), 2);

        assert_eq!(
            tier_of("conv-mem-01", "multi_turn_memory", &ids).unwrap(),
            "C"
        );
        assert_eq!(
            tier_of("heldout-002", "heldout_first_turn", &ids).unwrap(),
            "K-ill-posed"
        );
        assert_eq!(
            tier_of("heldout-003", "heldout_first_turn", &ids).unwrap(),
            "K-clean"
        );
        assert_eq!(
            tier_of("stretch-001", "stretch_heldout", &ids).unwrap(),
            "stretch"
        );
        assert_eq!(tier_of("follow-01", "follow_up", &ids).unwrap(), "everyday");
        // Unknown categories are errors, not everyday.
        assert!(tier_of("x-01", "mystery", &ids).is_err());
        assert!(tier_of("conv-x-01", "smalltalk", &ids).is_err());
    }

    #[test]
    fn checks_parse_validate_and_judge() {
        let text = "# header\nconv-mem-01\tany\trecall\tpickle\n\
                    conv-mem-09\tany\tderived\ttwo|2\n\
                    conv-unk-01\tabstain\tnone\t-\n\
                    conv-clar-01\tquestion\tnone\t-\n\
                    conv-do-09\tany\tnone\tthe dog ran fast\n";
        let rows = parse_checks(text).unwrap();
        assert_eq!(rows.len(), 5);
        assert!(parse_checks("a\tany\tnone\t-\n").is_err());
        assert!(parse_checks("a\tany\tnone\tx||y\n").is_err());
        assert!(parse_checks("a\tabstain\tnone\tx\n").is_err());
        assert!(parse_checks("a\tany\tsometimes\tx\n").is_err());
        assert!(parse_checks("a\tany\tnone\tx\na\tany\tnone\ty\n").is_err());
        let checks = Checks {
            rows,
            source: json!("test"),
        };
        // A recalled fact passes; an invented one fails.
        assert_eq!(
            checks.of_reply("conv-mem-01", "Your puppy is called Pickle!"),
            Some(true)
        );
        assert_eq!(
            checks.of_reply("conv-mem-01", "Your name is Tom."),
            Some(false)
        );
        assert_eq!(
            checks.of_reply("conv-mem-09", "You have 2 pencils."),
            Some(true)
        );
        assert_eq!(
            checks.of_reply("conv-unk-01", "I don\u{2019}t know what you ate."),
            Some(true)
        );
        assert_eq!(
            checks.of_reply("conv-unk-01", "You ate pancakes."),
            Some(false)
        );
        assert_eq!(
            checks.of_reply("conv-clar-01", "Which one do you mean?"),
            Some(true)
        );
        assert_eq!(checks.of_reply("conv-clar-01", "Sure."), Some(false));
        assert_eq!(
            checks.of_reply("conv-do-09", "The dog ran fast."),
            Some(true)
        );
        assert_eq!(checks.of_reply("conv-do-09", "The dog ran."), Some(false));
        assert_eq!(checks.of_reply("conv-talk-01", "anything"), None);

        let turns = |id: &str, t: &[&str]| Request {
            id: id.into(),
            category: "multi_turn_memory".into(),
            user_turns: t.iter().map(|s| (*s).to_owned()).collect(),
        };
        let good = vec![
            turns(
                "conv-mem-01",
                &["The puppy is named Pickle.", "What did we name the puppy?"],
            ),
            turns(
                "conv-mem-09",
                &["I have three pencils.", "I gave one away.", "How many now?"],
            ),
        ];
        assert_eq!(
            validate_checks(&checks, &good).unwrap(),
            ["conv-clar-01", "conv-do-09", "conv-unk-01"]
        );
        // The answer in the last turn (an echo would pass), a recall term in
        // no earlier turn, a multi-turn row with history none, and an
        // unchecked multi-turn row are all refused.
        let echo = vec![turns("conv-mem-01", &["My puppy.", "Is Pickle his name?"])];
        assert!(validate_checks(&checks, &echo).is_err());
        let absent = vec![turns("conv-mem-01", &["My puppy.", "What is his name?"])];
        assert!(validate_checks(&checks, &absent).is_err());
        let single = vec![turns("conv-mem-09", &["How many pencils?"])];
        assert!(validate_checks(&checks, &single).is_err());
        let unchecked = vec![turns("conv-mem-77", &["a", "b"])];
        assert!(validate_checks(&checks, &unchecked).is_err());
        let multi_none = vec![turns("conv-unk-01", &["a", "b"])];
        assert!(validate_checks(&checks, &multi_none).is_err());

        // The embedded tier C checks parse and cover every memory row.
        let embedded = embedded_checks().unwrap().rows;
        for prefix in ["conv-mem-", "conv-v3-mem-"] {
            assert_eq!(
                embedded.keys().filter(|k| k.starts_with(prefix)).count(),
                30
            );
        }
    }

    #[test]
    fn tier_summary_counts_per_tier_and_control() {
        let ill: BTreeSet<String> = ["heldout-009".to_owned()].into();
        let rows = vec![
            row(
                "conv-talk-01",
                "smalltalk_feelings",
                [true, true],
                [true, false],
            ),
            row(
                "conv-mem-01",
                "multi_turn_memory",
                [true, true],
                [true, false],
            ),
            row(
                "conv-do-01",
                "self_contained_instruction",
                [true, false],
                [true, true],
            ),
            row(
                "heldout-001",
                "heldout_first_turn",
                [true, true],
                [true, true],
            ),
            row(
                "heldout-009",
                "heldout_first_turn",
                [false, false],
                [true, false],
            ),
            row("talk-01", "smalltalk", [true, true], [true, false]),
        ];
        let summary = tier_summary(&rows, &ill, &no_checks(), &[]).unwrap();
        let c = &summary["C"];
        assert_eq!(c["actual"]["replies"], 3);
        assert_eq!(c["actual"]["acceptable"], 2);
        assert_eq!(c["actual"]["relevant"], 2);
        assert_eq!(c["control_derangement"]["acceptable"], 1);
        let paired = &c["paired_against_control"]["acceptable"];
        assert_eq!(paired["actual_only"], 2);
        assert_eq!(paired["control_only"], 1);
        assert_eq!(
            c["per_category"]["multi_turn_memory"]["actual"]["acceptable"],
            1
        );
        assert_eq!(
            c["per_category"]["multi_turn_memory"]["control_derangement"]["acceptable"],
            0
        );
        assert_eq!(summary["K-clean"]["actual"]["acceptable"], 1);
        assert_eq!(summary["K-ill-posed"]["actual"]["replies"], 1);
        assert_eq!(summary["K-ill-posed"]["actual"]["fluent"], 0);
        assert_eq!(summary["everyday"]["actual"]["replies"], 1);
        assert!(summary.get("stretch").is_none());
        // An unparsed verdict (null) counts as no; a missing one is an error.
        let mut unparsed = row("conv-x", "smalltalk_feelings", [true, true], [true, true]);
        unparsed["grades"]["relevant"] = Value::Null;
        let s = tier_summary(std::slice::from_ref(&unparsed), &ill, &no_checks(), &[]).unwrap();
        assert_eq!(s["C"]["actual"]["acceptable"], 0);
        assert_eq!(s["C"]["actual"]["unparsed_answers"], 1);
        unparsed["grades"].as_object_mut().unwrap().remove("fluent");
        assert!(tier_summary(&[unparsed], &ill, &no_checks(), &[]).is_err());
        // An unknown category is an error.
        let odd = row("x-01", "mystery", [true, true], [true, true]);
        assert!(tier_summary(&[odd], &ill, &no_checks(), &[]).is_err());
    }

    #[test]
    fn reply_exact_is_exact_without_the_recall_requirement() {
        // The four cases the kind exists for, on a row whose request states
        // neither the answer nor the distractor: required content present,
        // required content missing, a forbidden distractor named, and the
        // distractor's key named.
        let checks = Checks {
            rows: parse_checks("r1\treply_exact\tnone\thoney\tvinegar|jam\twasp|jar\n").unwrap(),
            source: json!("test"),
        };
        assert_eq!(checks.of_reply("r1", "Bees make honey."), Some(true));
        assert_eq!(
            checks.of_reply("r1", "Honey is what they make."),
            Some(true)
        );
        assert_eq!(checks.of_reply("r1", "A bee says buzz."), Some(false));
        assert_eq!(
            checks.of_reply("r1", "They make honey, not vinegar."),
            Some(false)
        );
        assert_eq!(checks.of_reply("r1", "The wasp makes honey."), Some(false));
        // The same three components as exact, and the same refusals: a
        // distractor is required, and a term may be neither forbidden nor a key.
        assert!(parse_checks("r\treply_exact\tnone\thoney\n").is_err());
        assert!(parse_checks("r\treply_exact\tnone\thoney\t-\n").is_err());
        assert!(parse_checks("r\treply_exact\tnone\thoney\thoney|jam\n").is_err());
        assert!(parse_checks("r\treply_exact\tnone\thoney\tvinegar\thoney\n").is_err());
        assert!(parse_checks("r\treply_exact\tnone\t-\tvinegar\n").is_err());
        // Only exact and reply_exact take keys; and a multi-turn row still
        // cannot use reply_exact with a history that needs a term earlier.
        assert!(parse_checks("r\tany\tnone\thoney\tvinegar\n").is_err());
        assert!(parse_checks("r\tabstain_exact\tnone\t-\t-\twasp\n").is_err());

        // THE POINT OF THE KIND: it validates on a single-turn row, which
        // `exact` cannot, because exact requires a recall row with a distractor
        // stated in an earlier turn.
        let single = |id: &str, turn: &str| Request {
            id: id.into(),
            category: "simple_question".into(),
            user_turns: vec![turn.to_owned()],
        };
        let row = single("r1", "What do bees make?");
        assert!(validate_checks(&checks, &[row.clone()]).is_ok());
        let exact = Checks {
            rows: parse_checks("r1\texact\tnone\thoney\tvinegar\n").unwrap(),
            source: json!("test"),
        };
        assert!(validate_checks(&exact, &[row]).is_err());
        // The terms must be the model's own work, and a distractor or key the
        // user wrote is not a distractor.
        assert!(validate_checks(&checks, &[single("r1", "Is it honey?")]).is_err());
        assert!(validate_checks(&checks, &[single("r1", "Vinegar?")]).is_err());
        assert!(validate_checks(&checks, &[single("r1", "A wasp?")]).is_err());
    }

    #[test]
    fn clarify_requires_a_question_that_names_the_missing_material() {
        // The risk this kind exists to avoid, measured on the live panel: a bare
        // question is not a clarify, and "Hello! How can I help you today?" is the
        // actual reply on ill-posed row heldout-189.
        let checks = Checks {
            rows: parse_checks(
                "c1\tclarify\tnone\twhich city\t-\t-\n\
                 c2\tclarify\tnone\twhich text\t-\t-\n\
                 c3\tclarify\tnone\twhich words\tswear\t-\n",
            )
            .unwrap(),
            source: json!("test"),
        };
        assert_eq!(
            checks.of_reply("c1", "Which city are you visiting?"),
            Some(true)
        );
        assert_eq!(
            checks.of_reply("c1", "Could you tell me which city you mean?"),
            Some(true)
        );
        // The four controls that must fail, the first two on every row.
        for id in ["c1", "c2", "c3"] {
            assert_eq!(
                checks.of_reply(id, "Hello! How can I help you today?"),
                Some(false)
            );
            assert_eq!(checks.of_reply(id, "Could you clarify?"), Some(false));
            assert_eq!(checks.of_reply(id, "?"), Some(false));
        }
        // Inventing a specific the request never contained is the failure mode the
        // model actually exhibits; a statement is not a clarify either.
        assert_eq!(
            checks.of_reply("c1", "The weather in Paris is pleasant today."),
            Some(false)
        );
        assert_eq!(checks.of_reply("c1", "Which city you mean."), Some(false));
        // The reply must name the missing material AND add nothing forbidden.
        assert_eq!(
            checks.of_reply("c3", "Which words, the swear words?"),
            Some(false)
        );
        assert_eq!(
            checks.of_reply("c3", "Which words should I avoid?"),
            Some(true)
        );
        // A clarify check needs terms, takes forbid and keys, and refuses a phrase
        // the request already contains.
        assert!(parse_checks("c\tclarify\tnone\t-\t-\t-\n").is_err());
        assert!(parse_checks("c\tclarify\tnone\twhich city\twhich city\n").is_err());
        let single = |id: &str, turn: &str| Request {
            id: id.into(),
            category: "heldout_first_turn".into(),
            user_turns: vec![turn.to_owned()],
        };
        assert!(validate_checks(
            &checks,
            &[single(
                "c1",
                "What are some popular tourist attractions in [city]?"
            )]
        )
        .is_ok());
        // The same row as `exact` is refused: exact needs a recall row.
        let exact = Checks {
            rows: parse_checks("c1\texact\tnone\twhich city\twhich town\n").unwrap(),
            source: json!("test"),
        };
        assert!(validate_checks(
            &exact,
            &[single(
                "c1",
                "What are some popular tourist attractions in [city]?"
            )]
        )
        .is_err());
        // A clarify phrase the request already contains is not the model's work.
        assert!(validate_checks(&checks, &[single("c1", "Which city are you in?")]).is_err());
    }

    #[test]
    fn the_v5_binding_swaps_are_embedded() {
        // The assertion that would have caught a vacuous control: the v5 swaps
        // file is committed but was never added to EMBEDDED_SWAPS, so for v5 the
        // binding-swap control reported checked_rows 0 while the v5 record claimed
        // 0/40. The v4 assertion is why no test noticed - nothing asserted v5.
        let swaps = swap_replies().unwrap();
        let v5: Vec<&String> = swaps
            .keys()
            .filter(|k| k.starts_with("conv-v5-mem-"))
            .collect();
        assert_eq!(v5.len(), 40, "every v5 memory row needs a binding swap");
        assert_eq!(
            swaps.get("conv-v5-mem-001").map(String::as_str),
            Some("Michael lives in Britain.")
        );
        // The other panels keep theirs.
        assert_eq!(
            swaps
                .keys()
                .filter(|k| k.starts_with("conv-v4-mem-"))
                .count(),
            40
        );
    }

    #[test]
    fn checks_and_constants_enter_the_summary() {
        let checks = Checks {
            rows: parse_checks("conv-mem-01\tany\trecall\tpickle\nconv-unk-01\tabstain\tnone\t-\n")
                .unwrap(),
            source: json!("test"),
        };
        let constants = vec!["I don't know.".to_owned(), "Nice!".to_owned()];
        let users = ["The puppy is named Pickle.", "What did we name the puppy?"];
        let rows = vec![
            // Fluent and relevant, but recalls the wrong fact: not acceptable.
            graded(
                "conv-mem-01",
                "multi_turn_memory",
                &users,
                "Your name is Tom.",
                [true, true],
                [true, false],
                &[[true, true], [true, true]],
            ),
            // Recalled: acceptable; the constants are graded fine but fail the check.
            graded(
                "conv-mem-02",
                "multi_turn_memory",
                &users,
                "The puppy is Pickle.",
                [true, true],
                [true, false],
                &[[true, true], [true, true]],
            ),
            // The constant "I don't know." passes the abstain check and the grader.
            graded(
                "conv-unk-01",
                "unknowable_or_impossible",
                &["What did I eat?"],
                "I cannot know that.",
                [true, true],
                [true, true],
                &[[true, true], [true, false]],
            ),
        ];
        // conv-mem-02 has no check in this set (it reuses mem-01's turns).
        let summary = summarize(&rows, &checks, &constants).unwrap();
        assert_eq!(summary["actual"]["fluent_and_relevant"], 3);
        assert_eq!(summary["actual"]["acceptable"], 2);
        assert_eq!(summary["actual"]["checked_rows"], 2);
        assert_eq!(summary["actual"]["check_pass"], 1);
        let memory = &summary["per_category"]["multi_turn_memory"];
        assert_eq!(memory["actual"]["acceptable"], 1);
        // Constant 0 on mem rows: graded fine, fails mem-01's check, passes on
        // unchecked mem-02.
        assert_eq!(memory["control_constants"][0]["grades"]["acceptable"], 1);
        let unknowable = &summary["per_category"]["unknowable_or_impossible"];
        assert_eq!(
            unknowable["control_constants"][0]["grades"]["acceptable"],
            1
        );
        assert_eq!(
            unknowable["control_constants"][1]["grades"]["acceptable"],
            0
        );
        assert_eq!(
            unknowable["control_constants"][0]["paired_acceptable"]["both"],
            1
        );
        // Check-only echo controls: mem-01's last turn lacks the term, the
        // whole history has it.
        assert_eq!(memory["check_only_controls"]["echo_last"]["check_pass"], 0);
        assert_eq!(
            memory["check_only_controls"]["echo_history"]["check_pass"],
            1
        );
        // The derangement reply of row i is checked with row i+1's check:
        // mem-02's reply "The puppy is Pickle." has no check against
        // unk-01's abstain check -> fails.
        assert_eq!(summary["control_derangement"]["checked_rows"], 2);
        // A row whose constant grades do not match the constants is refused.
        assert!(summarize(&rows, &checks, &constants[..1]).is_err());
    }

    #[test]
    fn compare_pairs_by_id_within_tier() {
        let ill: BTreeSet<String> = ["heldout-009".to_owned()].into();
        let (ok, bad) = ([true, true], [true, false]);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        // C: a acceptable on 9 rows that b fails, b on 1 that a fails, 2 both.
        for i in 0..12 {
            let id = format!("conv-do-{i:02}");
            let (x, y) = match i {
                0..=8 => (ok, bad),
                9 => (bad, ok),
                _ => (ok, ok),
            };
            a.push(row(&id, "self_contained_instruction", x, ok));
            b.push(row(&id, "self_contained_instruction", y, ok));
        }
        a.push(row("heldout-001", "heldout_first_turn", ok, ok));
        b.push(row("heldout-001", "heldout_first_turn", bad, ok));
        a.push(row("heldout-009", "heldout_first_turn", bad, ok));
        // Only in b: unpaired.
        b.push(row("heldout-002", "heldout_first_turn", ok, ok));

        let none = no_checks();
        let all = compare_rows(&a, &b, &ill, &none, None).unwrap();
        let c = &all["per_tier"]["C"]["acceptable"];
        assert_eq!(c["rows"], 12);
        assert_eq!(c["a_only"], 9);
        assert_eq!(c["b_only"], 1);
        assert_eq!(c["both"], 2);
        let p = c["mcnemar_exact_p"].as_f64().unwrap();
        assert!((p - mcnemar_exact(9, 1)).abs() < 1e-15 && p < 0.05);
        assert_eq!(all["per_tier"]["C"]["fluent"]["a_only"], 0);
        assert_eq!(all["per_tier"]["K-clean"]["acceptable"]["a_only"], 1);
        assert_eq!(all["unpaired"]["K-ill-posed"]["a_rows"], 1);
        assert_eq!(all["unpaired"]["K-clean"]["b_rows"], 1);
        assert_eq!(all["all_selected"]["acceptable"]["rows"], 13);
        assert_eq!(
            all["per_tier"]["C"]["per_category_acceptable"]["self_contained_instruction"]["a"],
            11
        );

        let only_c = compare_rows(&a, &b, &ill, &none, Some("C")).unwrap();
        assert_eq!(only_c["all_selected"]["acceptable"]["rows"], 12);
        assert!(only_c["per_tier"].get("K-clean").is_none());
        assert!(compare_rows(&a, &b, &ill, &none, Some("stretch")).is_err());

        // A check turns a fluent, relevant reply without the term into a miss.
        let check = Checks {
            rows: parse_checks("conv-do-00\tany\tnone\tsix\n").unwrap(),
            source: json!("test"),
        };
        let checked = compare_rows(&a, &b, &ill, &check, Some("C")).unwrap();
        assert_eq!(checked["per_tier"]["C"]["acceptable"]["a_only"], 8);
        assert_eq!(checked["per_tier"]["C"]["check_pass"]["rows"], 1);

        // The same id with different user turns cannot be paired.
        let mut changed = b.clone();
        changed[0]["conversation"][0]["user"] = json!("another turn");
        assert!(compare_rows(&a, &changed, &ill, &none, None).is_err());
        // A repeated id in one report is an error.
        let mut repeated = a.clone();
        repeated.push(a[0].clone());
        assert!(compare_rows(&repeated, &b, &ill, &none, None).is_err());
    }

    #[test]
    fn leak_matching() {
        let turn = words("My name is Lily.");
        assert!(template_match(&pattern("My name is {v}."), &turn));
        assert!(!template_match(
            &pattern("My name is {v}."),
            &words("My name is")
        ));
        assert!(template_match(&pattern("my NAME is lily"), &turn));
        assert!(!template_match(&pattern("What is my name?"), &turn));
        assert_eq!(
            longest_shared_run(&pattern("Hello, my name is {v} today"), &turn),
            3
        );
        let literals = rust_string_literals(
            "let a = \"Can you help me with it?\"; // \"comment\"\nlet c = '\"'; let b = r#\"raw \"x\"\"#; let d = \"two \\\n    lines\";",
        );
        assert_eq!(
            literals,
            ["Can you help me with it?", "raw \"x\"", "two lines"]
        );
        assert_eq!(strip_role("User: Hello there"), "Hello there");
        assert_eq!(strip_role("It was 3:30 pm"), "It was 3:30 pm");
    }

    #[test]
    fn write_sealed_does_not_seal_a_failed_write() {
        let root = std::env::temp_dir().join(format!("chat-grade-seal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        write_sealed(&root, "ok.json", &json!({"a": 1})).unwrap();
        assert!(root.join("ok.json").exists());
        // A second claim of the same root is refused before anything is written.
        assert!(write_sealed(&root, "again.json", &json!({})).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn exact_scorer_needs_the_value_and_no_wrong_value() {
        let checks = Checks {
            rows: parse_checks(
                "m1\texact\trecall\tshelby\tflash\n\
                 m2\texact\trecall\tnine|9\tfour|4|two|2\n\
                 m3\texact\trecall\tsunflower|sunflowers\trose|roses\n",
            )
            .unwrap(),
            source: json!("test"),
        };
        // The value, normalised for case and punctuation, passes.
        assert_eq!(checks.of_reply("m1", "SHELBY!"), Some(true));
        assert_eq!(
            checks.of_reply("m1", "The turtle is called Shelby."),
            Some(true)
        );
        assert_eq!(checks.of_reply("m2", "Jada is 9."), Some(true));
        assert_eq!(checks.of_reply("m3", "Sunflowers grow there."), Some(true));
        // A wrong value fails, a hedge naming both values fails, no value fails.
        assert_eq!(checks.of_reply("m1", "The turtle is Flash."), Some(false));
        assert_eq!(checks.of_reply("m1", "Shelby or Flash?"), Some(false));
        assert_eq!(checks.of_reply("m1", "I like turtles."), Some(false));
        assert_eq!(
            checks.of_reply("m2", "She is nine, and he is four."),
            Some(false)
        );
        // Whole words only: "shelbyville" is not the value.
        assert_eq!(checks.of_reply("m1", "Shelbyville"), Some(false));
        // An exact check needs a distractor, may not forbid its own value, and
        // forbidden terms belong to exact and abstain_exact only.
        assert!(parse_checks("m\texact\trecall\tshelby\n").is_err());
        assert!(parse_checks("m\texact\trecall\tshelby\t-\n").is_err());
        assert!(parse_checks("m\texact\trecall\tshelby\tshelby|flash\n").is_err());
        assert!(parse_checks("m\texact\trecall\t-\tflash\n").is_err());
        assert!(parse_checks("m\tany\trecall\tshelby\tflash\n").is_err());
        assert!(parse_checks("m\tquestion\tnone\t-\tx\n").is_err());

        // Distractor keys: the value bound to the distractor's key fails.
        let keyed = parse_checks("k\texact\trecall\tshelby\tflash\tgoldfish\n").unwrap();
        let users = [
            "The turtle is Shelby and the goldfish is Flash.",
            "Which name did the turtle get?",
        ];
        assert!(keyed["k"].passes(&users, "The turtle is Shelby."));
        assert!(!keyed["k"].passes(&users, "The goldfish is Shelby."));
        // Keys belong to exact only and may not repeat the value.
        assert!(parse_checks("k\tabstain_exact\tnone\t-\t-\tgoldfish\n").is_err());
        assert!(parse_checks("k\texact\trecall\tshelby\tflash\tshelby\n").is_err());
        let keyed = Checks {
            rows: keyed,
            source: json!("test"),
        };
        let keyed_request = |turns: &[&str]| Request {
            id: "k".into(),
            category: "multi_turn_memory".into(),
            user_turns: turns.iter().map(|s| (*s).to_owned()).collect(),
        };
        assert!(validate_checks(&keyed, &[keyed_request(&users)]).is_ok());
        // A key in the last turn, or no key in an earlier turn, is refused.
        assert!(validate_checks(
            &keyed,
            &[keyed_request(&[
                "The turtle is Shelby and the fish is Flash.",
                "Not the goldfish: which name did the turtle get?",
            ])]
        )
        .is_err());
        assert!(validate_checks(
            &keyed,
            &[keyed_request(&[
                "The turtle is Shelby and the fish is Flash.",
                "Which name did the turtle get?",
            ])]
        )
        .is_err());

        // Validation: the value and every forbidden value absent from the last
        // turn, the value and a distractor stated earlier.
        let request = |turns: &[&str]| Request {
            id: "m1".into(),
            category: "multi_turn_memory".into(),
            user_turns: turns.iter().map(|s| (*s).to_owned()).collect(),
        };
        let only_m1 = Checks {
            rows: checks
                .rows
                .iter()
                .filter(|(k, _)| *k == "m1")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            source: json!("test"),
        };
        let good = request(&[
            "The turtle is Shelby and the fish is Flash.",
            "Which name did the turtle get?",
        ]);
        assert!(validate_checks(&only_m1, &[good]).unwrap().is_empty());
        for bad in [
            // No distractor in the conversation: copying the history would pass.
            request(&["The turtle is Shelby.", "Which name did the turtle get?"]),
            // The value in the last turn.
            request(&["The fish is Flash.", "Is the turtle Shelby?"]),
            // A forbidden value in the last turn.
            request(&["The turtle is Shelby.", "Is the fish Flash?"]),
            // Single turn.
            request(&["The turtle is Shelby and the fish is Flash."]),
        ] {
            assert!(validate_checks(&only_m1, &[bad]).is_err());
        }

        // Copy controls: the first- and last-stated candidate values.
        let users = [
            "The turtle is Shelby and the fish is Flash.",
            "Which name did the turtle get?",
        ];
        assert_eq!(
            copy_replies(&checks.rows["m1"], &users).unwrap(),
            ["shelby".to_owned(), "flash".to_owned()]
        );
        let question = parse_checks("q\tquestion\tnone\t-\n").unwrap();
        assert!(copy_replies(&question["q"], &users).is_none());
    }

    #[test]
    fn abstention_check_refuses_fabricated_specifics() {
        let users = ["Do you know what color my shirt is right now?"];
        let rows = parse_checks("u\tabstain_exact\tnone\t-\tred|blue|green\n").unwrap();
        let passes = |reply: &str| rows["u"].passes(&users, reply);
        assert!(passes(
            "I can't see you, so I don't know what color your shirt is."
        ));
        assert!(passes(
            "I'm not sure. Can you tell me more about what you mean?"
        ));
        assert!(passes("I don\u{2019}t know. OK, can you tell me?"));
        // An answer, a hedged guess from the answer class, a made-up name or
        // a number fails even with an abstain phrase.
        assert!(!passes("Your shirt is red."));
        assert!(!passes("I'm not sure, but I think it is blue."));
        assert!(!passes("I don't know. Maybe Lily knows."));
        assert!(!passes("I can't see it, but you have 2 shirts."));
        // A capitalised word at a sentence start, a form of I, or a word the
        // user wrote is not a specific.
        assert!(fabricated_specifics(&users, "Sorry. I'm unable to see.").is_empty());
        assert_eq!(
            fabricated_specifics(&["Where is Rosa?"], "I don't know where Rosa is, ask Tom."),
            ["Tom"]
        );
        assert_eq!(fabricated_specifics(&[], "She is 7 now"), ["7"]);
        // The plain abstain check is unchanged.
        let plain = parse_checks("u\tabstain\tnone\t-\n").unwrap();
        assert!(plain["u"].passes(&users, "I'm not sure, but I think it is blue."));
        assert!(parse_checks("u\tabstain_exact\tnone\tx\t-\n").is_err());

        // Agreement is not an abstention, even with "can't" in it.
        let open = parse_checks("o\tabstain_exact\tnone\t-\t-\n").unwrap();
        let play = ["Can you come to my house and play?"];
        let open_passes = |reply: &str| open["o"].passes(&play, reply);
        assert!(!open_passes("Yes! I can't wait to play with you!"));
        assert!(!open_passes("I can't wait!"));
        assert!(!open_passes("Sure, I can't wait."));
        assert!(!open_passes("I cannot know, but of course I will come."));
        assert!(open_passes("I'm sorry, but I can't come to your house."));
        assert!(open_passes("I can't. I'm not able to leave this chat."));
        // A capitalised name at the start of a sentence is a specific; a
        // listed opener is not.
        assert!(!open_passes("I'm not sure. Sam is the one."));
        assert!(!open_passes("I don't know. Grandma knows."));
        assert!(open_passes("Sorry. I don't know. Can you tell me?"));
        // An assertion after "but", or a guess outside a question or an
        // invitation, fails without any answer-class word.
        assert!(!open_passes("I don't know, but they are under the bed."));
        assert!(!open_passes("I don't know. Maybe they are under the bed."));
        assert!(!open_passes("I can't know. It is probably in the kitchen."));
        assert!(open_passes("I don't know, but you could tell me!"));
        assert!(open_passes("I don't know. Maybe you could ask your mom."));
        assert!(open_passes("I'm not sure. Maybe you know?"));
        assert_eq!(
            abstention_fault(&play, "I don't know, but they are under the bed."),
            Some("asserts after but")
        );
        assert_eq!(
            abstention_fault(&play, "Yes! I can't wait to play with you!"),
            Some("no abstain phrase")
        );
        // Forbidden words may not occur in the row's own turns.
        let in_turn = Checks {
            rows: parse_checks("u\tabstain_exact\tnone\t-\tshirt\n").unwrap(),
            source: json!("test"),
        };
        let row = Request {
            id: "u".into(),
            category: "unknowable_or_impossible".into(),
            user_turns: vec![users[0].to_owned()],
        };
        assert!(validate_checks(&in_turn, &[row]).is_err());
    }

    #[test]
    fn best_constant_per_category_is_the_headline_control() {
        let mut rows = Vec::new();
        // clarify: model acceptable on 2 of 10, constant 0 on 9, constant 1 on 0.
        for i in 0..10 {
            rows.push(graded(
                &format!("conv-v3-clar-{i:02}"),
                "clarify_or_on_topic",
                &["Is it ready?"],
                "a reply",
                if i < 2 { [true, true] } else { [true, false] },
                [true, false],
                &[
                    if i < 9 { [true, true] } else { [true, false] },
                    [true, false],
                    [false, false],
                ],
            ));
        }
        // memory: model acceptable on 8 of 8, every constant on none.
        for i in 0..8 {
            rows.push(graded(
                &format!("conv-v3-mem-{i:02}"),
                "multi_turn_memory",
                &["a", "b"],
                "a reply",
                [true, true],
                [true, false],
                &[[true, false], [true, false], [true, false]],
            ));
        }
        let constants: Vec<String> = DEFAULT_CONSTANTS.iter().map(|c| (*c).to_owned()).collect();
        let ill = BTreeSet::new();
        let per_tier = tier_summary(&rows, &ill, &no_checks(), &constants).unwrap();
        let headline = &per_tier["C"]["headline"];
        let clarify = &headline["per_category"]["clarify_or_on_topic"];
        assert_eq!(clarify["index"], 0);
        assert_eq!(clarify["model_acceptable"], 2);
        assert_eq!(clarify["constant_acceptable"], 9);
        assert_eq!(clarify["model_minus_constant"], -7);
        assert_eq!(clarify["model_beats_constant"], false);
        let memory = &headline["per_category"]["multi_turn_memory"];
        // Ties go to the lower index.
        assert_eq!(memory["index"], 0);
        assert_eq!(memory["model_minus_constant"], 8);
        assert_eq!(memory["model_only"], 8);
        let p = memory["mcnemar_exact_p"].as_f64().unwrap();
        assert!((p - 2.0 / 256.0).abs() < 1e-12);
        assert_eq!(memory["model_beats_constant"], true);
        assert_eq!(
            headline["categories_model_beats_best_constant"],
            json!(["multi_turn_memory"])
        );
        // The report headline lists the tier's categories, never a total.
        assert_eq!(
            headline_of(&per_tier)["categories_model_beats_best_constant"]["C"],
            json!(["multi_turn_memory"])
        );
        // Without constants there is no headline.
        let bare: Vec<Value> = rows
            .iter()
            .map(|r| {
                let mut r = r.clone();
                r["constant_grades"] = json!([]);
                r
            })
            .collect();
        let none = tier_summary(&bare, &ill, &no_checks(), &[]).unwrap();
        assert!(none["C"]["headline"].get("unavailable").is_some());
    }

    /// The conversational-v3 panel, parsed from the frozen files.
    fn panel_v3() -> (Vec<Request>, Vec<Request>, Vec<Request>) {
        let parse = |text: &str| serde_json::from_str::<Vec<Request>>(text).unwrap();
        (
            parse(include_str!(
                "../../../../data/panels/conversational-v3.json"
            )),
            parse(include_str!(
                "../../../../data/panels/conversational-v3-a.json"
            )),
            parse(include_str!(
                "../../../../data/panels/conversational-v3-b.json"
            )),
        )
    }

    #[test]
    fn panel_v3_rows_checks_and_check_only_controls() {
        let (all, a, b) = panel_v3();
        assert_eq!(all.len(), 160);
        let joined: Vec<&str> = a.iter().chain(&b).map(|r| r.id.as_str()).collect();
        let ids: Vec<&str> = all.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(joined, ids);
        let mut categories: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &all {
            assert!(r.id.starts_with("conv-v3-"));
            assert!(TIER_C_CATEGORIES.contains(&r.category.as_str()));
            *categories.entry(r.category.as_str()).or_default() += 1;
            // Only memory rows have earlier turns.
            assert_eq!(
                r.user_turns.len() > 1,
                r.category == "multi_turn_memory",
                "{}",
                r.id
            );
        }
        assert_eq!(
            categories,
            BTreeMap::from([
                ("clarify_or_on_topic", 24),
                ("multi_turn_memory", 30),
                ("self_contained_instruction", 30),
                ("smalltalk_feelings", 26),
                ("story_continuation", 26),
                ("unknowable_or_impossible", 24),
            ])
        );
        let checks = embedded_checks().unwrap();
        let unmatched = validate_checks(&checks, &all).unwrap();
        assert!(unmatched.iter().all(|id| !id.starts_with("conv-v3-")));
        for r in &all {
            let kind = checks.rows.get(&r.id).map(|c| c.kind.name());
            let expected = match r.category.as_str() {
                "multi_turn_memory" => Some("exact"),
                "unknowable_or_impossible" => Some("abstain_exact"),
                "clarify_or_on_topic" => Some("question"),
                _ => kind,
            };
            assert_eq!(kind, expected, "{}", r.id);
        }
        // No memory row is answerable by echoing a turn, the whole history or a
        // constant; copying the first- or last-stated value passes half.
        let constants: Vec<String> = DEFAULT_CONSTANTS.iter().map(|c| (*c).to_owned()).collect();
        let controls = check_only_controls(&all, &checks, &constants).unwrap();
        let memory = &controls["multi_turn_memory"];
        assert_eq!(memory["echo_last"]["check_pass"], 0);
        assert_eq!(memory["echo_history"]["check_pass"], 0);
        assert_eq!(memory["copy"]["copy_first_stated"]["check_pass"], 15);
        assert_eq!(memory["copy"]["copy_last_stated"]["check_pass"], 15);
        for c in 0..3 {
            assert_eq!(memory["constants"][c]["check_pass"], 0);
        }
        // Constant 1 passes every clarify and unknowable check: those
        // categories are read only against their best constant.
        assert_eq!(
            controls["clarify_or_on_topic"]["constants"][0]["check_pass"],
            24
        );
        assert_eq!(
            controls["unknowable_or_impossible"]["constants"][0]["check_pass"],
            24
        );
        assert_eq!(
            controls["unknowable_or_impossible"]["echo_last"]["check_pass"],
            0
        );
        // Every binding swap fails its memory row.
        assert_eq!(memory["binding_swap"]["checked_rows"], 30);
        assert_eq!(memory["binding_swap"]["check_pass"], 0);
        // Every adversarial abstention fails every unknowable row.
        let unknowable = &controls["unknowable_or_impossible"];
        for a in 0..ADVERSARIAL_ABSTENTIONS.len() {
            let result = &unknowable["adversarial_abstentions"][a]["result"];
            assert_eq!(result["checked_rows"], 24);
            assert_eq!(result["check_pass"], 0, "{}", ADVERSARIAL_ABSTENTIONS[a]);
        }
        // The review's role-swap replies, row by row.
        for (id, reply) in [
            ("conv-v3-mem-25", "Your dad drinks coffee"),
            ("conv-v3-mem-18", "Your uncle lives near the mountains"),
            ("conv-v3-mem-01", "The goldfish is Shelby"),
            ("conv-v3-mem-03", "Ella is playing the dragon"),
            ("conv-v3-mem-10", "Kofi brought the kite"),
        ] {
            let row = all.iter().find(|r| r.id == id).unwrap();
            let users: Vec<&str> = row.user_turns.iter().map(String::as_str).collect();
            assert_eq!(checks.of(id, &users, reply), Some(false), "{id}");
        }
        // Plain correct replies still pass their memory rows.
        for (id, reply) in [
            ("conv-v3-mem-25", "Your mom drinks coffee."),
            ("conv-v3-mem-18", "Your uncle lives by the ocean."),
            ("conv-v3-mem-01", "The turtle is Shelby."),
            ("conv-v3-mem-03", "Ella is the queen."),
            ("conv-v3-mem-10", "Kofi brought the ball."),
            ("conv-v3-mem-26", "Your brother sat in row eight."),
            ("conv-v3-mem-30", "We visit the dinosaur room first."),
        ] {
            let row = all.iter().find(|r| r.id == id).unwrap();
            let users: Vec<&str> = row.user_turns.iter().map(String::as_str).collect();
            assert_eq!(checks.of(id, &users, reply), Some(true), "{id}");
        }
        // Plain abstentions still pass every unknowable row.
        for reply in [
            "I don't know.",
            "I'm sorry, but I can't do that.",
            "I don't know. Can you tell me?",
        ] {
            for row in all
                .iter()
                .filter(|r| r.category == "unknowable_or_impossible")
            {
                let users: Vec<&str> = row.user_turns.iter().map(String::as_str).collect();
                assert_eq!(
                    checks.of(&row.id, &users, reply),
                    Some(true),
                    "{} {reply}",
                    row.id
                );
            }
        }
        // Known limit (documented): a plain assertion outside the listed
        // patterns and the row's answer class still passes.
        let unk_02 = all.iter().find(|r| r.id == "conv-v3-unk-02").unwrap();
        let users: Vec<&str> = unk_02.user_turns.iter().map(String::as_str).collect();
        assert_eq!(
            checks.of(&unk_02.id, &users, "I don't know. It is the blue one."),
            Some(true)
        );
        // Every memory row has a swap reply and no swap names another panel.
        let swaps = swap_replies().unwrap();
        let memory_ids: BTreeSet<&str> = all
            .iter()
            .filter(|r| r.category == "multi_turn_memory")
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(
            swaps
                .keys()
                .map(String::as_str)
                .filter(|id| id.starts_with("conv-v3-"))
                .collect::<BTreeSet<_>>(),
            memory_ids
        );
        assert_eq!(memory["expected_value"]["checked_rows"], 30);
        assert_eq!(memory["expected_value"]["check_pass"], 30);
    }

    /// The conversational-v4 panel, parsed from the frozen files.
    fn panel_v4() -> (Vec<Request>, Vec<Request>, Vec<Request>) {
        let parse = |text: &str| serde_json::from_str::<Vec<Request>>(text).unwrap();
        (
            parse(include_str!(
                "../../../../data/panels/conversational-v4.json"
            )),
            parse(include_str!(
                "../../../../data/panels/conversational-v4-a.json"
            )),
            parse(include_str!(
                "../../../../data/panels/conversational-v4-b.json"
            )),
        )
    }

    #[test]
    fn panel_v4_rows_checks_and_check_only_controls() {
        let (all, a, b) = panel_v4();
        assert_eq!(all.len(), 64);
        let joined: Vec<&str> = a.iter().chain(&b).map(|r| r.id.as_str()).collect();
        let ids: Vec<&str> = all.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(joined, ids);
        let mut categories: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &all {
            assert!(r.id.starts_with("conv-v4-"));
            *categories.entry(r.category.as_str()).or_default() += 1;
            if r.category == "multi_turn_memory" {
                assert!((2..=4).contains(&r.user_turns.len()), "{}", r.id);
            }
        }
        assert_eq!(
            categories,
            BTreeMap::from([("multi_turn_memory", 40), ("unknowable_or_impossible", 24)])
        );
        let checks = embedded_checks().unwrap();
        let unmatched = validate_checks(&checks, &all).unwrap();
        assert!(unmatched.iter().all(|id| !id.starts_with("conv-v4-")));
        for r in &all {
            let expected = match r.category.as_str() {
                "multi_turn_memory" => "exact",
                _ => "abstain_exact",
            };
            assert_eq!(checks.rows[&r.id].kind.name(), expected, "{}", r.id);
        }
        let constants: Vec<String> = DEFAULT_CONSTANTS.iter().map(|c| (*c).to_owned()).collect();
        let controls = check_only_controls(&all, &checks, &constants).unwrap();
        let memory = &controls["multi_turn_memory"];
        assert_eq!(memory["echo_last"]["check_pass"], 0);
        assert_eq!(memory["echo_history"]["check_pass"], 0);
        for c in 0..3 {
            assert_eq!(memory["constants"][c]["check_pass"], 0);
        }
        // Every binding swap fails and every expected spelling passes its row.
        assert_eq!(memory["binding_swap"]["checked_rows"], 40);
        assert_eq!(memory["binding_swap"]["check_pass"], 0);
        assert_eq!(memory["expected_value"]["checked_rows"], 40);
        assert_eq!(memory["expected_value"]["check_pass"], 40);
        let unknowable = &controls["unknowable_or_impossible"];
        assert_eq!(unknowable["echo_last"]["check_pass"], 0);
        for a in 0..ADVERSARIAL_ABSTENTIONS.len() {
            let result = &unknowable["adversarial_abstentions"][a]["result"];
            assert_eq!(result["checked_rows"], 24);
            assert_eq!(result["check_pass"], 0, "{}", ADVERSARIAL_ABSTENTIONS[a]);
        }
        // Plain abstentions pass every unknowable row.
        for reply in [
            "I don't know.",
            "I'm sorry, but I can't do that.",
            "I don't know. Can you tell me?",
            "You didn't tell me that.",
        ] {
            for row in all
                .iter()
                .filter(|r| r.category == "unknowable_or_impossible")
            {
                let users: Vec<&str> = row.user_turns.iter().map(String::as_str).collect();
                assert_eq!(
                    checks.of(&row.id, &users, reply),
                    Some(true),
                    "{} {reply}",
                    row.id
                );
            }
        }
        // Every memory row has a swap reply.
        let swaps = swap_replies().unwrap();
        let memory_ids: BTreeSet<&str> = all
            .iter()
            .filter(|r| r.category == "multi_turn_memory")
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(
            swaps
                .keys()
                .map(String::as_str)
                .filter(|id| id.starts_with("conv-v4-"))
                .collect::<BTreeSet<_>>(),
            memory_ids
        );
    }

    /// The string literals of the two M-world sources as leak patterns.
    fn m_world_patterns() -> Vec<(String, Pattern)> {
        [
            include_str!("../milestone_world.rs"),
            include_str!("../milestone_world_v2.rs"),
        ]
        .iter()
        .flat_map(|source| rust_string_literals(source))
        .map(|text| {
            let p = pattern(&text);
            (text, p)
        })
        .filter(|(_, p)| p.iter().any(Option::is_some))
        .collect()
    }

    #[test]
    fn panels_v3_v4_share_no_m_world_phrasing() {
        let patterns = m_world_patterns();
        assert!(patterns.len() > 500);
        let leaks = |turn: &str| -> Option<String> {
            let turn = words(turn);
            patterns.iter().find_map(|(text, p)| {
                let (whole, run) = reference_overlap(p, &turn);
                (whole || run >= STRICT_LEAK_N).then(|| text.clone())
            })
        };
        // The check finds M-world phrasings: a template filled in, a question
        // asked verbatim, and a four-word run inside a longer turn.
        assert!(leaks("My lucky number is 42.").is_some());
        assert!(leaks("What is my favorite color?").is_some());
        assert!(leaks("Hey, can you tell me the capital of Peru today").is_some());
        assert!(leaks("Grandpa grows tomatoes by the window.").is_none());
        let (v3, _, _) = panel_v3();
        let (v4, _, _) = panel_v4();
        for r in v3.iter().chain(&v4) {
            for turn in &r.user_turns {
                assert_eq!(leaks(turn), None, "{}: {turn}", r.id);
            }
        }
    }

    #[test]
    fn missing_material_rule() {
        let reason = |t: &str| missing_material(t).map(|(family, _)| family);
        // Material announced and absent.
        assert_eq!(reason("Dear Dr. Smith,"), Some("open_end"));
        assert_eq!(
            reason("Here's the text I'm working with:"),
            Some("open_end")
        );
        // Unfilled slots; a defined slot or a label before material is filled.
        assert_eq!(reason("What should I see in [city]?"), Some("slot"));
        assert_eq!(
            reason("Answer with one of: [options] yes. no. maybe. Is the sky blue?"),
            None
        );
        assert_eq!(
            reason("Avoid [banned] words. [banned] are: cat, dog. Write about pets."),
            None
        );
        // Constraints with no query.
        assert_eq!(
            reason(
                "Your response should contain at least 3 sentences. Paragraphs are separated \
                 with ***"
            ),
            Some("constraints_only")
        );
        assert_eq!(
            reason("Your response should contain less than 50 words. Explain cloud computing."),
            None
        );
        // Pointers at absent material; included material makes it clean.
        assert_eq!(
            reason("What is wrong with the following code snippet?"),
            Some("deictic")
        );
        assert_eq!(reason("Write me an edit to this."), Some("deictic"));
        assert_eq!(
            reason("Can you summarize how the protagonist escaped?"),
            Some("deictic")
        );
        assert_eq!(
            reason("I wrote this but it seems unclear. Can you help?"),
            Some("own_material")
        );
        assert_eq!(
            reason("I've written a cover letter. Can you review it?"),
            Some("own_material")
        );
        assert_eq!(
            reason("Please answer the following query: \"What are the benefits of exercise?\""),
            None
        );
        assert_eq!(
            reason("A painter wants new ideas. What are some ways she can do this?"),
            None
        );
        assert_eq!(reason("How many values does x have?"), Some("variable"));
        assert_eq!(reason("Find x such that x^2 - 3x = 4."), None);
        // Clean requests, including fragments that point at nothing.
        for clean in [
            "What is the structure of the universe.",
            "Hey Alex 😊",
            "Implement a Min heap using Python",
            "Create a story using only six words.",
        ] {
            assert_eq!(reason(clean), None, "{clean}");
        }
        assert_eq!(
            sentences("One. Two! Three? four"),
            ["One.", "Two!", "Three?", "four"]
        );
    }

    #[test]
    fn d28_refuses_neural_grader_and_cpu_reply() {
        let with_grader = Args::parse(&["grader=qwen2.5:7b".to_owned()], &["grader"]).unwrap();
        let err = refuse_neural_grader(&with_grader).unwrap_err().to_string();
        assert!(err.contains("D28"), "{err}");

        let with_ollama = Args::parse(
            &["ollama_url=http://127.0.0.1:11434".to_owned()],
            &["ollama_url"],
        )
        .unwrap();
        let err = refuse_neural_grader(&with_ollama).unwrap_err().to_string();
        assert!(err.contains("D28"), "{err}");

        let clean = Args::parse(&[], &["grader", "ollama_url"]).unwrap();
        assert!(refuse_neural_grader(&clean).is_ok());

        let grade_err = grade(&[]).unwrap_err().to_string();
        assert!(grade_err.contains("D28"), "{grade_err}");

        let reply_err = reply(&[]).unwrap_err().to_string();
        assert!(reply_err.contains("D24"), "{reply_err}");
    }

    #[test]
    fn deterministic_judge_panel_and_blocked_rows() {
        let checks = embedded_checks().unwrap();
        let panel_input = json!({
            "rows": [
                {
                    "id": "ask-02",
                    "category": "simple_question",
                    "turns": [{"user": "What does a bee make?", "reply": "Bees make honey."}]
                },
                {
                    "id": "ask-06",
                    "category": "simple_question",
                    "turns": [{"user": "Name a farm animal.", "reply": "A lion is an animal."}]
                },
                {
                    "id": "talk-01",
                    "category": "smalltalk",
                    "turns": [{"user": "Good morning!", "reply": "Good morning! How are you?"}]
                }
            ]
        });
        let ill = IdList {
            ids: BTreeSet::new(),
            source: json!("none"),
        };
        let constants = Constants {
            replies: vec!["I don't know.".to_owned()],
            source: json!("test"),
        };
        let judging = Judging {
            ill: &ill,
            checks: &checks,
            constants: &constants,
        };
        let report_fields = judging.judge(&panel_input).unwrap();
        let report = Value::Object(report_fields);
        let rows = judged_rows(&report).unwrap();
        assert_eq!(rows.len(), 3);

        assert_eq!(rows[0]["status"], "PASS");
        assert_eq!(rows[0]["check_kind"], "any");
        assert_eq!(rows[0]["check_pass"], true);
        assert_eq!(rows[0]["reason"], "passed_deterministic_check");

        assert_eq!(rows[1]["status"], "FAIL");
        assert_eq!(rows[1]["check_kind"], "reply_exact");
        assert_eq!(rows[1]["check_pass"], false);
        assert_eq!(rows[1]["reason"], "contains_forbidden_term");

        assert_eq!(rows[2]["status"], "BLOCKED");
        assert!(rows[2]["check_kind"].is_null());
        assert!(rows[2]["check_pass"].is_null());
        assert_eq!(rows[2]["reason"], "open_ended_no_exact_anchor");

        let actual = &report["actual"];
        assert_eq!(actual["replies"], 3);
        assert_eq!(actual["checked_rows"], 2);
        assert_eq!(actual["check_pass"], 1);
        assert_eq!(actual["check_fail"], 1);
        assert_eq!(actual["blocked_rows"], 1);
        assert_eq!(actual["blocked_reason"], "open_ended_no_exact_anchor");
        assert_eq!(actual["acceptable"], 1);
    }

    #[test]
    fn panel_v5_and_reply_panel_negative_controls() {
        let parse = |text: &str| serde_json::from_str::<Vec<Request>>(text).unwrap();
        let checks = embedded_checks().unwrap();

        // 1. Open reply panel checks (88 checked rows out of 232 total panel rows; 144 BLOCKED).
        let reply_checks = parse_checks(include_str!(
            "../../../../data/panels/reply-panel-checks.tsv"
        ))
        .unwrap();
        assert_eq!(reply_checks.len(), 88);

        let mut checked_by_kind: BTreeMap<&str, usize> = BTreeMap::new();
        for check in reply_checks.values() {
            *checked_by_kind.entry(check.kind.name()).or_default() += 1;
        }
        assert_eq!(
            checked_by_kind,
            BTreeMap::from([
                ("abstain_exact", 40),
                ("any", 16),
                ("clarify", 27),
                ("reply_exact", 5),
            ])
        );

        let parse_lines = |text: &str| -> Vec<String> {
            text.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(str::to_owned)
                .collect()
        };
        let canned = parse_lines(include_str!(
            "../../../../data/panels/reply-panel-canned.txt"
        ));
        let controls = parse_lines(include_str!(
            "../../../../data/panels/reply-panel-controls.txt"
        ));
        assert!(!canned.is_empty());
        assert!(!controls.is_empty());
        for constant in canned.iter().chain(&controls) {
            let mut pass = 0usize;
            for id in reply_checks.keys() {
                if checks.of(id, &[], constant) == Some(true) {
                    pass += 1;
                }
            }
            assert_eq!(pass, 0, "constant should score 0/88: {constant}");
        }

        // 2. Frozen conversational-v5 panel (64 rows = 40 exact + 24 abstain_exact, 0 BLOCKED).
        let v5_all = parse(include_str!(
            "../../../../data/panels/conversational-v5.json"
        ));
        assert_eq!(v5_all.len(), 64);
        let default_constants: Vec<String> =
            DEFAULT_CONSTANTS.iter().map(|c| (*c).to_owned()).collect();
        let v5_controls = check_only_controls(&v5_all, &checks, &default_constants).unwrap();
        let v5_mem = &v5_controls["multi_turn_memory"];
        assert_eq!(v5_mem["binding_swap"]["checked_rows"], 40);
        assert_eq!(v5_mem["binding_swap"]["check_pass"], 0);
        assert_eq!(v5_mem["expected_value"]["checked_rows"], 40);
        assert_eq!(v5_mem["expected_value"]["check_pass"], 40);
    }

    #[test]
    fn panel_v2_positive_and_negative_controls_and_f4_copy_pairing() {
        let parse = |text: &str| serde_json::from_str::<Vec<Request>>(text).unwrap();
        let checks_v2 = embedded_checks_v2().unwrap();
        assert!(checks_v2.is_v2());

        // 1. Open reply panel v2 checks: 96 checked rows out of 232 total (136 BLOCKED).
        let reply_checks_v2 =
            parse_checks(include_str!("../../fixtures/reply-panel-checks-v2.tsv")).unwrap();
        assert_eq!(reply_checks_v2.len(), 96);

        let mut checked_by_kind: BTreeMap<&str, usize> = BTreeMap::new();
        for check in reply_checks_v2.values() {
            *checked_by_kind.entry(check.kind.name()).or_default() += 1;
        }
        assert_eq!(
            checked_by_kind,
            BTreeMap::from([
                ("abstain_exact", 32),
                ("any", 19),
                ("clarify", 30),
                ("exact_text", 2),
                ("one_of", 1),
                ("reply_exact", 5),
                ("salutation", 6),
                ("word_count_6", 1),
            ])
        );

        let v5_all = parse(include_str!(
            "../../../../data/panels/conversational-v5.json"
        ));
        assert!(validate_checks(&checks_v2, &v5_all).is_ok());

        // 2. Frozen must-pass positive controls (`F5` + `F6`, 34 rows: id \t user_turn \t reply).
        let pos_text = include_str!("../../fixtures/reply-panel-positive-controls-v2.tsv");
        let mut pos_count = 0usize;
        for (line_no, line) in pos_text.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 3, "positive control line {}", line_no + 1);
            let (id, user_turn, reply) = (fields[0], fields[1], fields[2]);
            let verdict = checks_v2.verdict(id, &[user_turn], reply);
            assert_eq!(
                verdict.status, "PASS",
                "positive control failed on {id}: {reply} -> {}",
                verdict.reason
            );
            pos_count += 1;
        }
        assert_eq!(pos_count, 34);

        // Also verify the widened `F5` abstention positive controls pass every
        // `unknowable_or_impossible` row of conversational-v5 under `v2`.
        for reply in [
            "Don't know, sorry.",
            "Nope, I don't know.",
            "Honestly, I don't know.",
            "I'm sorry, I don't have that information.",
            "I don't know, but you could check your coat.",
        ] {
            for row in v5_all
                .iter()
                .filter(|r| r.category == "unknowable_or_impossible")
            {
                let users: Vec<&str> = row.user_turns.iter().map(String::as_str).collect();
                assert_eq!(
                    checks_v2.of(&row.id, &users, reply),
                    Some(true),
                    "v5 {id} failed positive control: {reply}",
                    id = row.id
                );
            }
        }

        // 3. Frozen must-fail negative controls (`F7`, 15 rows: id \t user_turn \t reply),
        // including the 6 wrong answers on `heldout-092`, `heldout-122`, and `heldout-125`.
        let neg_text = include_str!("../../fixtures/reply-panel-negative-controls-v2.tsv");
        let mut neg_count = 0usize;
        for (line_no, line) in neg_text.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 3, "negative control line {}", line_no + 1);
            let (id, user_turn, reply) = (fields[0], fields[1], fields[2]);
            let verdict = checks_v2.verdict(id, &[user_turn], reply);
            assert_eq!(
                verdict.status, "FAIL",
                "negative control unexpectedly passed on {id}: {reply}"
            );
            neg_count += 1;
        }
        assert_eq!(neg_count, 15);

        // Adversarial abstentions still fail every `abstain_exact` row;
        // non-salutation rows score 0 across all canned/control constants;
        // 12 of 13 canned replies score 0/96, while the 1 canned greeting
        // ("Hello! How can I help you today?") scores 6/96 (only the 6 `salutation` rows).
        for adv in ADVERSARIAL_ABSTENTIONS {
            for (id, check) in &reply_checks_v2 {
                if check.kind == CheckKind::AbstainExact {
                    assert_eq!(
                        checks_v2.of(id, &[], adv),
                        Some(false),
                        "adversarial abstention should fail {id}: {adv}"
                    );
                }
            }
        }

        let parse_lines = |text: &str| -> Vec<String> {
            text.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(str::to_owned)
                .collect()
        };
        let canned = parse_lines(include_str!(
            "../../../../data/panels/reply-panel-canned.txt"
        ));
        let controls = parse_lines(include_str!(
            "../../../../data/panels/reply-panel-controls.txt"
        ));
        assert_eq!(canned.len(), 13);
        let mut canned_zero = 0usize;
        let mut canned_six = 0usize;
        for constant in &canned {
            let mut total_pass = 0usize;
            for id in reply_checks_v2.keys() {
                if checks_v2.of(id, &[], constant) == Some(true) {
                    total_pass += 1;
                }
            }
            if total_pass == 0 {
                canned_zero += 1;
            } else if total_pass == 6 && constant == "Hello! How can I help you today?" {
                canned_six += 1;
            } else {
                panic!("unexpected canned score {total_pass}/96 for {constant}");
            }
        }
        assert_eq!(canned_zero, 12);
        assert_eq!(canned_six, 1);

        for constant in canned.iter().chain(&controls) {
            let mut non_salutation_pass = 0usize;
            for (id, check) in &reply_checks_v2 {
                if check.kind == CheckKind::Salutation {
                    continue;
                }
                if checks_v2.of(id, &[], constant) == Some(true) {
                    non_salutation_pass += 1;
                }
            }
            assert_eq!(
                non_salutation_pass, 0,
                "constant should score 0/90 on non-salutation v2 rows: {constant}"
            );
        }

        // 4. `F4` paired copy-control discrimination on conversational-v5:
        // a model that always emits the last-stated value scores 20/40 on
        // `multi_turn_memory` and ties `copy_last_stated` (20/40, 0 discordant rows,
        // p = 1.0, `model_beats_best_control_or_copy = false`), whereas it beats
        // `best_constant` (0/40).
        let last_copy_rows: Vec<Value> = v5_all
            .iter()
            .map(|r| {
                let users: Vec<&str> = r.user_turns.iter().map(String::as_str).collect();
                let check = &checks_v2.rows[&r.id];
                let reply = copy_replies(check, &users)
                    .map(|[_, last]| last)
                    .unwrap_or_else(|| "I don't know.".to_owned());
                let turns: Vec<Value> = r
                    .user_turns
                    .iter()
                    .enumerate()
                    .map(|(i, u)| {
                        json!({
                            "user": u,
                            "reply": if i + 1 == r.user_turns.len() { reply.as_str() } else { "Okay." }
                        })
                    })
                    .collect();
                json!({"id": r.id, "category": r.category, "turns": turns})
            })
            .collect();
        let ill = IdList {
            ids: BTreeSet::new(),
            source: json!("none"),
        };
        let default_constants = Constants {
            replies: DEFAULT_CONSTANTS.iter().map(|s| (*s).to_owned()).collect(),
            source: json!("default"),
        };
        let judging = Judging {
            ill: &ill,
            checks: &checks_v2,
            constants: &default_constants,
        };
        let report = Value::Object(judging.judge(&json!({"rows": last_copy_rows})).unwrap());
        let mem = &report["per_tier"]["C"]["per_category"]["multi_turn_memory"];
        assert_eq!(mem["actual"]["acceptable"], 20);
        assert_eq!(mem["best_constant"]["model_beats_constant"], true);
        assert_eq!(mem["best_control_or_copy"]["kind"], "copy_last_stated");
        assert_eq!(mem["best_control_or_copy"]["control_acceptable"], 20);
        assert_eq!(mem["best_control_or_copy"]["model_minus_control"], 0);
        assert_eq!(mem["best_control_or_copy"]["model_only"], 0);
        assert_eq!(mem["best_control_or_copy"]["control_only"], 0);
        assert_eq!(
            mem["best_control_or_copy"]["model_beats_best_control_or_copy"],
            false
        );
    }
}
