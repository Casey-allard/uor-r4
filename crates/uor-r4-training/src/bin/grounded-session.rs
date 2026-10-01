//! The grounded session (#962, D19) as an ordinary-text command line.
//!
//! ```text
//! grounded-session init out=NEW_SESSION checkpoint=SEALED_CHECKPOINT tokenizer=T.json \
//!   compiler=COMPILER.json [scope=default] [entity=user] [max_new_tokens=32] [max_turns=64] \
//!   [max_source_bytes=65536] [max_history_tokens=1048576] [max_store_records=4096] \
//!   [context=whole_turns|strict]
//! grounded-session turn session=SESSION out=NEW_SESSION text=TEXT [read=true] [write=true]
//! grounded-session chat session=SESSION out=NEW_SESSION
//! grounded-session restart session=SESSION out=NEW_SESSION [scope=SCOPE] [entity=ENTITY]
//! grounded-session evaluate session=BASE_SESSION cases=SCRIPT.json out=NEW_REPORT \
//!   [max_cases=64] [max_events=512] [max_bytes=1048576] [max_seconds=600]
//! grounded-session show session=SESSION
//! ```
//!
//! `init` binds a sealed inference checkpoint (model and store), its
//! tokenizer and a saved relation compiler (`m-world compiler-save`) or its
//! learned temporal adapter (`temporal-compiler fit`) into an
//! empty session envelope. `turn` loads an envelope in this fresh process,
//! answers one user turn and prints its outcome as JSON; `chat` answers one
//! user turn per line of standard input. Both save the continued session to
//! a new envelope; an envelope is never changed. The compiler is always
//! loaded from the envelope's own `compiler.bin`, so a session resumes only
//! with its exact saved compiler, and loading replays the whole transcript
//! against the store before any new turn.
//! `restart` begins an empty conversation over the retained exact store,
//! optionally at another caller-supplied scope/entity. It clears the prior
//! transcript and generated context, not durable records or old envelopes.
//! These identities address memory; they provide no authentication or erasure.
//!
//! This is a floating-point development path, not a D11 serving kernel, and
//! makes no capability claim.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use candle_core::Device;
use serde_json::json;
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::grounded_session_eval::{evaluate, EvalSettings, ScriptSet};
use uor_r4_training::stack_grounded_session::{
    ContextPolicy, GroundedSession, SessionLimits, SessionScope, TurnControls, TurnOutcome,
    COMPILER_FILE, TOKENIZER_FILE,
};
use uor_r4_training::temporal_compiler::GroundedCompiler;

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("grounded-session: {error}");
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

    fn text(&self, key: &str) -> Result<&str, Error> {
        Ok(self.0.get(key).ok_or_else(|| format!("missing {key}="))?)
    }

    fn path(&self, key: &str) -> Result<PathBuf, Error> {
        Ok(PathBuf::from(self.text(key)?))
    }

    fn or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.0.get(key).map_or(default, String::as_str)
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

fn run() -> Result<(), Error> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((mode, rest)) = arguments.split_first() else {
        return Err(
            "usage: grounded-session init|turn|chat|restart|evaluate|show key=value ...".into(),
        );
    };
    match mode.as_str() {
        "init" => init(&Args::parse(
            rest,
            &[
                "out",
                "checkpoint",
                "tokenizer",
                "compiler",
                "scope",
                "entity",
                "max_new_tokens",
                "max_turns",
                "max_source_bytes",
                "max_history_tokens",
                "max_store_records",
                "context",
            ],
        )?),
        "turn" => turn(&Args::parse(
            rest,
            &["session", "out", "text", "read", "write"],
        )?),
        "chat" => chat(&Args::parse(rest, &["session", "out"])?),
        "restart" => restart(&Args::parse(rest, &["session", "out", "scope", "entity"])?),
        "evaluate" => evaluate_script(&Args::parse(
            rest,
            &[
                "session",
                "cases",
                "out",
                "max_cases",
                "max_events",
                "max_bytes",
                "max_seconds",
            ],
        )?),
        "show" => show(&Args::parse(rest, &["session"])?),
        other => Err(format!("unknown mode {other}").into()),
    }
}

fn init(args: &Args) -> Result<(), Error> {
    let out = args.path("out")?;
    let tokenizer_json = fs::read(args.path("tokenizer")?)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let compiler = GroundedCompiler::from_bytes(fs::read(args.path("compiler")?)?)?;
    let scope = SessionScope {
        scope: args.or("scope", "default").as_bytes().to_vec(),
        entity: tokenizer.encode(args.or("entity", "user")),
    };
    let limits = SessionLimits {
        max_new_tokens: args.number("max_new_tokens", 32)?,
        max_turns: args.number("max_turns", 64)?,
        max_source_bytes: args.number("max_source_bytes", 1 << 16)?,
        max_history_tokens: args.number("max_history_tokens", 1 << 20)?,
        max_store_records: args.number("max_store_records", 4_096)?,
        context_policy: match args.or("context", "whole_turns") {
            "whole_turns" => ContextPolicy::WholeCompletedTurns,
            "strict" => ContextPolicy::StrictFullHistory,
            other => return Err(format!("unknown context={other}").into()),
        },
    };
    let session = GroundedSession::from_checkpoint_path(
        &args.path("checkpoint")?,
        tokenizer_json,
        compiler,
        scope,
        limits,
        &Device::Cpu,
    )?;
    session.save(&out)?;
    println!(
        "{}",
        json!({"session": out.display().to_string(),
               "compiler_sha256": session.compiler_identity().artifact_sha256})
    );
    Ok(())
}

/// Load an envelope with the compiler saved inside it.
fn open(root: &Path) -> Result<GroundedSession<GroundedCompiler>, Error> {
    let compiler = GroundedCompiler::from_bytes(fs::read(root.join(COMPILER_FILE))?)?;
    Ok(GroundedSession::load(root, compiler, &Device::Cpu)?)
}

fn outcome_json(index: usize, outcome: &TurnOutcome) -> serde_json::Value {
    json!({
        "turn": index,
        "user": outcome.source,
        "reply": outcome.reply_text,
        "action": outcome.action,
        "memory": outcome.memory,
        "recall": outcome.recall,
        "stop": outcome.stop,
        "memory_commit": outcome.memory_commit,
        "retained_from_turn": outcome.retained_from_turn,
    })
}

fn turn(args: &Args) -> Result<(), Error> {
    let out = args.path("out")?;
    let mut session = open(&args.path("session")?)?;
    let controls = TurnControls {
        read: args.number("read", true)?,
        write: args.number("write", true)?,
    };
    let outcome = session.turn_with_controls(args.text("text")?, controls)?;
    session.save(&out)?;
    println!("{}", outcome_json(session.turns().len() - 1, &outcome));
    Ok(())
}

fn chat(args: &Args) -> Result<(), Error> {
    let out = args.path("out")?;
    let mut session = open(&args.path("session")?)?;
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match session.turn(&line) {
            Ok(outcome) => println!("{}", outcome.reply_text),
            // A failed turn changes nothing; the session continues.
            Err(error) => eprintln!("grounded-session: turn refused: {error}"),
        }
    }
    session.save(&out)?;
    eprintln!(
        "grounded-session: {} turns saved to {}",
        session.turns().len(),
        out.display()
    );
    Ok(())
}

fn show(args: &Args) -> Result<(), Error> {
    let session = open(&args.path("session")?)?;
    for (index, outcome) in session.turns().iter().enumerate() {
        println!("{}", outcome_json(index, outcome));
    }
    Ok(())
}

fn restart(args: &Args) -> Result<(), Error> {
    let out = args.path("out")?;
    let mut session = open(&args.path("session")?)?;
    let mut scope = session.scope().clone();
    if let Some(value) = args.0.get("scope") {
        scope.scope = value.as_bytes().to_vec();
    }
    if let Some(value) = args.0.get("entity") {
        let tokenizer_json = fs::read(args.path("session")?.join(TOKENIZER_FILE))?;
        if sha256_hex(&tokenizer_json) != session.compiler_identity().tokenizer_sha256 {
            return Err("saved tokenizer changed after loading the session".into());
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
            .ok_or("unreadable saved tokenizer.json")?;
        scope.entity = tokenizer.encode(value);
    }
    session.start_conversation(scope)?;
    session.save(&out)?;
    println!(
        "{}",
        json!({
            "session":out.display().to_string(),
            "scope":session.scope(),
            "turns":session.turns().len(),
            "memory_commit":session.store().commit(),
            "memory_records":session.store().records(),
            "compiler_sha256":session.compiler_identity().artifact_sha256,
            "durable_memory_retained":true,
        })
    );
    Ok(())
}

/// Evaluation annotations are frozen input data, never runtime instructions.
/// The new report is claimed before cases or a model are loaded. A failed
/// attempt is retained and sealed; another invocation must choose a new root.
fn evaluate_script(args: &Args) -> Result<(), Error> {
    let out = args.path("out")?;
    let baseline = args.path("session")?;
    let cases_path = args.path("cases")?;
    let defaults = EvalSettings::default();
    let settings = EvalSettings {
        max_cases: args.number("max_cases", defaults.max_cases)?,
        max_events: args.number("max_events", defaults.max_events)?,
        max_bytes: args.number("max_bytes", defaults.max_bytes)?,
        max_seconds: args.number("max_seconds", defaults.max_seconds)?,
    };
    settings.validate()?;
    let read_limit = u64::try_from(
        settings
            .max_bytes
            .checked_add(1)
            .ok_or("input bound overflow")?,
    )?;
    report_output::claim(&out)?;
    let result: Result<(), Error> = (|| {
        let mut content = Vec::new();
        fs::File::open(&cases_path)?
            .take(read_limit)
            .read_to_end(&mut content)?;
        if content.len() > settings.max_bytes {
            return Err("case input exceeds max_bytes".into());
        }
        let cases: ScriptSet = serde_json::from_slice(&content)?;
        cases.validate(&settings)?;
        fs::write(out.join("cases.json"), &content)?;
        let mut report = evaluate(&baseline, &cases, &out, &settings)?;
        let record = report
            .as_object_mut()
            .ok_or("evaluator returned no report object")?;
        record.insert(
            "inputs".into(),
            json!({
                "cases_path": cases_path.display().to_string(),
                "cases_bytes": content.len(),
                "cases_sha256":sha256_hex(&content),
                "baseline_path":baseline.display().to_string(),
            }),
        );
        record.insert(
            "execution".into(),
            json!({
                "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),
                "executable_sha256":uor_r4_training::sha256_file(&std::env::current_exe()?)?,
            }),
        );
        fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::write(
            out.join("error.json"),
            serde_json::to_vec(&json!({"error":error.to_string()}))?,
        );
        report_output::seal(&out)?;
        report_output::verify(&out)?;
        return Err(error);
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    println!(
        "{}",
        json!({"report":out.join("report.json").display().to_string()})
    );
    Ok(())
}
