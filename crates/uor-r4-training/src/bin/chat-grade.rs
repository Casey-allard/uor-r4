//! The scale ladder's chat-quality instrument (#820): a fixed panel of
//! everyday requests, replies from any saved stack model, and grades from a
//! local offline teacher.
//!
//! ```text
//! chat-grade extract out=PANEL.json heldout=SPLIT_DIR tokenizer=T.json \
//!   [count=48] [max_words=24] [seed=1]
//! chat-grade grade out=NEW_REPORT_ROOT model=ROOT/model tokenizer=T.json \
//!   requests=PANEL.json[,MORE.json] [protocol=2] [max_new_tokens=64] \
//!   [grader=qwen2.5:1.5b] [ollama_url=http://127.0.0.1:11434]
//! ```
//!
//! `extract` decodes a prepared protocol-2 held-out chat split (documents from
//! BOS; the first `User:` turn up to the next newline) and keeps, by a seeded
//! shuffle, `count` single-turn requests of at most `max_words` words. The
//! held-out split was never trained on.
//!
//! `grade` answers every request greedily (`stack_dialogue::reply_panel`)
//! and asks the grader, a local Ollama model used as an offline judge only,
//! to score each reply's fluency and relevance from 1 to 5 against the
//! conversation so far. As a validity control it also grades each reply
//! against the *next* request's conversation (a derangement): relevance must
//! fall there, fluency need not. The report root is claimed before the model
//! loads and sealed at the end.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::sha256_file;
use uor_r4_training::stack_dialogue::{greedy_reply, load_requests, reply_panel, Request};
use uor_r4_training::stack_tracking::Rng;

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chat-grade: {error}");
            ExitCode::FAILURE
        }
    }
}

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
}

fn run() -> Result<(), Error> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("extract") => extract(&arguments[1..]),
        Some("grade") => grade(&arguments[1..]),
        _ => Err("usage: chat-grade extract|grade key=value...".into()),
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

/// Grades from the local Ollama judge.
struct Grader {
    url: String,
    model: String,
}

impl Grader {
    fn ask(&self, prompt: &str) -> Result<String, Error> {
        use std::io::Write;
        let body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "stream": false,
            "options": {"temperature": 0.0, "seed": 1, "num_predict": 4},
        });
        let mut child = std::process::Command::new("curl")
            .args(["-sS", "--fail", "-H", "Content-Type: application/json"])
            .args(["--data-binary", "@-"])
            .arg(format!("{}/api/chat", self.url))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("curl has no stdin")?
            .write_all(body.to_string().as_bytes())?;
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(format!("ollama failed: {}", output.status).into());
        }
        let reply: Value = serde_json::from_slice(&output.stdout)?;
        Ok(reply["message"]["content"]
            .as_str()
            .ok_or("ollama returned no message")?
            .to_owned())
    }

    fn digest(&self) -> Result<String, Error> {
        let output = std::process::Command::new("curl")
            .args(["-sS", "--fail"])
            .arg(format!("{}/api/tags", self.url))
            .output()?;
        let tags: Value = serde_json::from_slice(&output.stdout)?;
        tags["models"]
            .as_array()
            .and_then(|m| m.iter().find(|m| m["name"] == self.model.as_str()))
            .and_then(|m| m["digest"].as_str())
            .map(str::to_owned)
            .ok_or_else(|| format!("ollama has no model {}", self.model).into())
    }

    /// One yes/no judgement of the last reply of `text`, with worked
    /// examples on topics outside the panel. Returns the verdict (or `None`
    /// when the answer has neither word) and the raw answer.
    fn judge(
        &self,
        text: &str,
        question: &str,
        examples: &str,
    ) -> Result<(Option<bool>, String), Error> {
        let prompt = format!(
            "You check replies written by a small chat assistant. Answer with one word: yes or no.\n\n\
             {examples}\nNow this conversation:\n{text}\nQuestion: {question}\nAnswer:"
        );
        let answer = self.ask(&prompt)?;
        Ok((parse_yes_no(&answer), answer))
    }

    /// Fluency and relevance verdicts for the final reply of `conversation`,
    /// with the raw answers.
    fn grade(&self, conversation: &[(String, String)]) -> Result<Grades, Error> {
        let mut text = String::new();
        for (user, assistant) in conversation {
            text.push_str(&format!("User: {user}\nAssistant: {assistant}\n"));
        }
        let (fluent, fluent_raw) = self.judge(&text, FLUENT_QUESTION, FLUENT_EXAMPLES)?;
        let (relevant, relevant_raw) = self.judge(&text, RELEVANT_QUESTION, RELEVANT_EXAMPLES)?;
        Ok(Grades {
            fluent,
            relevant,
            raw: [fluent_raw, relevant_raw],
        })
    }
}

const FLUENT_QUESTION: &str =
    "Is the last Assistant reply written in clear, correct, sensible English sentences?";
const FLUENT_EXAMPLES: &str = "Example:\nUser: Do you like trains?\nAssistant: Yes, I think trains are fun to ride.\n\
     Question: Is the last Assistant reply written in clear, correct, sensible English sentences?\nAnswer: yes\n\n\
     Example:\nUser: Do you like trains?\nAssistant: The train train is of the a ride ride.\n\
     Question: Is the last Assistant reply written in clear, correct, sensible English sentences?\nAnswer: no\n";
const RELEVANT_QUESTION: &str =
    "Does the last Assistant reply respond sensibly to the user's last message?";
const RELEVANT_EXAMPLES: &str = "Example:\nUser: What color is grass?\nAssistant: Grass is usually green.\n\
     Question: Does the last Assistant reply respond sensibly to the user's last message?\nAnswer: yes\n\n\
     Example:\nUser: What color is grass?\nAssistant: I had pancakes for breakfast today.\n\
     Question: Does the last Assistant reply respond sensibly to the user's last message?\nAnswer: no\n";

/// Two yes/no verdicts and the grader's raw answers.
struct Grades {
    fluent: Option<bool>,
    relevant: Option<bool>,
    raw: [String; 2],
}

impl Grades {
    fn record(&self) -> Value {
        json!({"fluent": self.fluent, "relevant": self.relevant, "raw": self.raw})
    }
}

/// The verdict of a yes/no answer: its first word, `yes` or `no`.
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

#[derive(Default)]
struct Tally {
    replies: usize,
    fluent: usize,
    relevant: usize,
    acceptable: usize,
    unparsed: usize,
}

impl Tally {
    fn add(&mut self, grades: &Grades) {
        self.replies += 1;
        self.fluent += usize::from(grades.fluent == Some(true));
        self.relevant += usize::from(grades.relevant == Some(true));
        self.acceptable +=
            usize::from(grades.fluent == Some(true) && grades.relevant == Some(true));
        self.unparsed +=
            usize::from(grades.fluent.is_none()) + usize::from(grades.relevant.is_none());
    }

    fn record(&self) -> Value {
        json!({
            "replies": self.replies, "fluent": self.fluent, "relevant": self.relevant,
            "acceptable": self.acceptable, "unparsed_answers": self.unparsed,
            "acceptable_rule": "judged both fluent and relevant (an unparsed answer counts as no)",
        })
    }
}

fn grade(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &[
            "out",
            "model",
            "tokenizer",
            "requests",
            "protocol",
            "max_new_tokens",
            "grader",
            "ollama_url",
        ],
    )?;
    let out = PathBuf::from(args.required("out")?);
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let request_paths: Vec<PathBuf> = args
        .required("requests")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let version: u8 = args.number("protocol", 2)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 64)?;
    let grader = Grader {
        url: args
            .0
            .get("ollama_url")
            .cloned()
            .unwrap_or_else(|| "http://127.0.0.1:11434".into()),
        model: args
            .0
            .get("grader")
            .cloned()
            .unwrap_or_else(|| "qwen2.5:1.5b".into()),
    };
    report_output::claim(&out)?;
    let result = grade_into(
        &out,
        &model_dir,
        &tokenizer_path,
        &request_paths,
        version,
        max_new_tokens,
        &grader,
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

#[allow(clippy::too_many_arguments)]
fn grade_into(
    out: &Path,
    model_dir: &Path,
    tokenizer_path: &Path,
    request_paths: &[PathBuf],
    version: u8,
    max_new_tokens: usize,
    grader: &Grader,
    started: Instant,
) -> Result<(), Error> {
    let tokenizer = load_tokenizer(tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut requests: Vec<Request> = Vec::new();
    for path in request_paths {
        requests.extend(load_requests(path)?);
    }
    let model = StackModel::load(model_dir, &candle_core::Device::Cpu)?;
    let clock = Instant::now();
    let panel = reply_panel(
        &encoder,
        &protocol,
        &requests,
        model.config.context,
        max_new_tokens,
        &|ids| tokenizer.decode(ids),
        &mut |history, cap| greedy_reply(&model, history, cap, protocol.eos_id),
    )?;
    let generation_seconds = clock.elapsed().as_secs_f64();
    // Each request's conversation as (user, reply) pairs.
    let conversations: Vec<Vec<(String, String)>> = panel["rows"]
        .as_array()
        .ok_or("panel without rows")?
        .iter()
        .map(|row| {
            row["turns"]
                .as_array()
                .map(|turns| {
                    turns
                        .iter()
                        .map(|t| {
                            (
                                t["user"].as_str().unwrap_or_default().to_owned(),
                                t["reply"].as_str().unwrap_or_default().trim().to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    let (mut actual, mut control) = (Tally::default(), Tally::default());
    let mut per_category: BTreeMap<String, Tally> = BTreeMap::new();
    let mut graded_rows = Vec::new();
    let n = conversations.len();
    for (i, conversation) in conversations.iter().enumerate() {
        let grades = grader.grade(conversation)?;
        actual.add(&grades);
        per_category
            .entry(requests[i].category.clone())
            .or_default()
            .add(&grades);
        // Control: this reply as the answer to the next request's last turn.
        let mut swapped = conversations[(i + 1) % n].clone();
        let reply = conversation
            .last()
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        if let Some(last) = swapped.last_mut() {
            last.1 = reply;
        }
        let control_grades = grader.grade(&swapped)?;
        control.add(&control_grades);
        graded_rows.push(json!({
            "id": requests[i].id, "category": requests[i].category,
            "conversation": conversation.iter().map(|(u, a)| json!({"user": u, "assistant": a})).collect::<Vec<_>>(),
            "grades": grades.record(),
            "control_grades": control_grades.record(),
        }));
    }
    let report = json!({
        "schema": "uor-r4.chat-grade/1",
        "model": model_dir.display().to_string(),
        "model_sha256": sha256_file(&model_dir.join("model.safetensors")).ok(),
        "parameters": model.parameter_count(),
        "tokenizer_sha256": sha256_file(tokenizer_path)?,
        "protocol": protocol.schema,
        "requests": request_paths.iter().map(|p| json!({"path": p.display().to_string(), "sha256": sha256_file(p).ok()})).collect::<Vec<_>>(),
        "max_new_tokens": max_new_tokens,
        "decoding": "greedy",
        "grader": {
            "engine": "ollama (local)", "model": grader.model, "digest": grader.digest()?,
            "role": "offline judge only; it never serves", "temperature": 0, "seed": 1,
            "questions": [FLUENT_QUESTION, RELEVANT_QUESTION],
        },
        "actual": actual.record(),
        "control_derangement": control.record(),
        "control_rule": "each reply graded as the answer to the next request's last turn; relevance must fall for the grader to be measuring relevance",
        "per_category": per_category.iter().map(|(k, t)| (k.clone(), t.record())).collect::<BTreeMap<_, _>>(),
        "rows": graded_rows,
        "generation_seconds": generation_seconds,
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    println!(
        "actual {} | control {}",
        report["actual"], report["control_derangement"]
    );
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
        assert_eq!(
            first_user_turn("User: Hello there\nAssistant: Hi"),
            Some("Hello there".into())
        );
        assert_eq!(first_user_turn("Assistant: Hi"), None);
    }
}
