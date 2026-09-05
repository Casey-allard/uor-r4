//! Optional, finite controlled experiment for context retention and fact updates.
//! All predictions and continuations come from the native model. Expected colors
//! are evaluation labels only. This is neither an alpha claim nor a CI gate.
//!
//! Example: cargo run -p uor-r4-core --example native_geometric_memory_probe --
//!   --context 128 --output-dir /tmp/r4-memory-context-128

use std::collections::BTreeSet;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::{
    Config, Control, Document, MemoryReadFitConfig, Model, ReadoutFitConfig, Trainer, BOS,
};

type ProbeResult<T> = Result<T, Box<dyn Error>>;
const OBJECTS: [&str; 6] = ["orb", "cube", "key", "coin", "ring", "gem"];
const COLORS: [&str; 4] = ["red", "blue", "green", "gold"];
const FILLER_UNITS: [usize; 4] = [0, 4, 16, 64];
const WORLD_COUNT: usize = 2880;

#[derive(Debug, Clone, Serialize)]
struct Options {
    context: usize,
    construction_documents: usize,
    readout_documents: usize,
    development_worlds: usize,
    fit_positions: usize,
    fit_epochs: usize,
    generated_tokens: usize,
    max_seconds: u64,
    family: String,
    memory_read: bool,
    word_cues: bool,
    query_context: bool,
    model: Option<PathBuf>,
    source: Option<PathBuf>,
    compile_rust: bool,
    compiler_cases: usize,
    query_tokens: Option<usize>,
    source_offsets: Option<usize>,
    postings_per_address: Option<usize>,
    memory_candidates: Option<usize>,
    output_dir: PathBuf,
}

impl Options {
    fn parse() -> ProbeResult<Self> {
        let mut options = Self {
            context: 128,
            construction_documents: 256,
            readout_documents: 64,
            development_worlds: 12,
            fit_positions: 4096,
            fit_epochs: 8,
            generated_tokens: 6,
            max_seconds: 300,
            family: "prose".into(),
            memory_read: false,
            word_cues: false,
            query_context: false,
            model: None,
            source: None,
            compile_rust: false,
            compiler_cases: 8,
            query_tokens: None,
            source_offsets: None,
            postings_per_address: None,
            memory_candidates: None,
            output_dir: PathBuf::new(),
        };
        let mut arguments = std::env::args().skip(1);
        while let Some(flag) = arguments.next() {
            if flag == "--help" || flag == "-h" {
                println!("Optional native memory experiment (not a mandatory gate).\n\
                    --context 1..4096 --construction-documents 1..1000\n\
                    --readout-documents 1..128 --development-worlds 1..32\n\
                    --fit-positions 1..16384 --fit-epochs 1..64\n\
                    --generated-tokens 1..4096 --max-seconds N --output-dir PATH\n\
                    --family prose|rust|mixed --memory-read (optional learned read)\n\
                    --word-cues (optional leading-whitespace word equivalence)\n\
                    --query-context (optional learned query-context memory features)\n\
                    --model PATH --source PATH (together: evaluate only, no fitting)\n\
                    --compile-rust (compile exact Rust continuations; never execute)\n\
                    --compiler-cases 1..512 (default 8 Rust cases per artifact, Full only)\n\
                    --query-tokens 1..32 --source-offsets 1..16\n\
                    --postings-per-address 1..8 --memory-candidates 1..256\n\
                    Mixed uses the document/world counts once per family on one model.\n\
                    Repeat with the same options except context/output-dir for a window comparison.\n\
                    The output directory must not already exist.");
                std::process::exit(0);
            }
            if flag == "--memory-read" {
                options.memory_read = true;
                continue;
            }
            if flag == "--word-cues" {
                options.word_cues = true;
                continue;
            }
            if flag == "--compile-rust" {
                options.compile_rust = true;
                continue;
            }
            if flag == "--query-context" {
                options.query_context = true;
                continue;
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--context" => options.context = value.parse()?,
                "--construction-documents" => options.construction_documents = value.parse()?,
                "--readout-documents" => options.readout_documents = value.parse()?,
                "--development-worlds" => options.development_worlds = value.parse()?,
                "--fit-positions" => options.fit_positions = value.parse()?,
                "--fit-epochs" => options.fit_epochs = value.parse()?,
                "--generated-tokens" => options.generated_tokens = value.parse()?,
                "--max-seconds" => options.max_seconds = value.parse()?,
                "--output-dir" => options.output_dir = value.into(),
                "--family" => options.family = value,
                "--model" => options.model = Some(value.into()),
                "--source" => options.source = Some(value.into()),
                "--compiler-cases" => options.compiler_cases = value.parse()?,
                "--query-tokens" => options.query_tokens = Some(value.parse()?),
                "--source-offsets" => options.source_offsets = Some(value.parse()?),
                "--postings-per-address" => options.postings_per_address = Some(value.parse()?),
                "--memory-candidates" => options.memory_candidates = Some(value.parse()?),
                _ => return Err(format!("unknown option {flag}").into()),
            }
        }
        if options.model.is_some() != options.source.is_some() {
            return Err("--model and --source must be supplied together for evaluation".into());
        }
        if !(1..=4096).contains(&options.context)
            || !(1..=1000).contains(&options.construction_documents)
            || !(1..=128).contains(&options.readout_documents)
            || !(1..=32).contains(&options.development_worlds)
            || !(1..=16384).contains(&options.fit_positions)
            || !(1..=64).contains(&options.fit_epochs)
            || !(1..=4096).contains(&options.generated_tokens)
            || !(1..=512).contains(&options.compiler_cases)
            || options.max_seconds == 0
            || (options.word_cues && !options.memory_read)
            || (options.query_context && !options.memory_read)
            || !["prose", "rust", "mixed"].contains(&options.family.as_str())
            || options.query_tokens.is_some_and(|v| !(1..=32).contains(&v))
            || options
                .source_offsets
                .is_some_and(|v| !(1..=16).contains(&v))
            || options
                .postings_per_address
                .is_some_and(|v| !(1..=8).contains(&v))
            || options
                .memory_candidates
                .is_some_and(|v| !(1..=256).contains(&v))
        {
            return Err("options exceed this experiment's supported bounds; see --help".into());
        }
        let fit_overrides = options.query_tokens.is_some()
            || options.source_offsets.is_some()
            || options.postings_per_address.is_some()
            || options.memory_candidates.is_some();
        if (fit_overrides && !options.memory_read)
            || (options.model.is_some()
                && (fit_overrides
                    || options.memory_read
                    || options.word_cues
                    || options.query_context))
        {
            return Err("memory fit options require --memory-read and cannot change a loaded artifact in evaluation-only mode".into());
        }
        if options.output_dir.as_os_str().is_empty() {
            options.output_dir = std::env::temp_dir().join(format!(
                "r4-native-memory-{}-{}",
                options.context,
                std::process::id()
            ));
        }
        Ok(options)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    world: usize,
    filler_units: usize,
    counterfactual: bool,
    query_kind: String,
    prompt: String,
    expected_color: String,
    answer_color_start_byte: usize,
    #[serde(default, skip_serializing_if = "is_false")]
    rust: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

impl Case {
    fn document(&self) -> Document {
        Document {
            id: self.id.clone(),
            text: if self.rust {
                format!("{}{}\");\n}}\n", self.prompt, self.expected_color)
            } else {
                format!("{} {}.\n", self.prompt, self.expected_color)
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Source {
    schema: String,
    construction: Vec<Case>,
    readout_fit: Vec<Case>,
    development: Vec<Case>,
    split_policy: String,
}

fn color_permutations() -> Vec<[usize; 4]> {
    let mut permutations = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let tuple = [a, b, c, d];
                    if tuple.iter().collect::<BTreeSet<_>>().len() == 4 {
                        permutations.push(tuple);
                    }
                }
            }
        }
    }
    permutations
}

fn fact(text: &mut String, object: &str, color: &str, update: bool) -> usize {
    if update {
        text.push_str("Now ");
    }
    text.push_str("the ");
    text.push_str(object);
    text.push_str(" is ");
    let start = text.len();
    text.push_str(color);
    text.push_str(". ");
    start
}

fn rust_fact(text: &mut String, object: &str, color: &str, update: bool) -> usize {
    if !update {
        text.push_str("let mut ");
    }
    text.push_str(object);
    text.push_str(" = \"");
    let start = text.len();
    text.push_str(color);
    text.push_str("\";\n");
    start
}

fn make_case(
    world: usize,
    filler_units: usize,
    counterfactual: bool,
    split: &str,
    permutations: &[[usize; 4]],
    rust: bool,
) -> Case {
    let first = world % 6;
    let other_rank = (world / 6) % 5;
    let second = if other_rank >= first {
        other_rank + 1
    } else {
        other_rank
    };
    let colors = permutations[(world / 30) % 24];
    let query_updated = (world / 720).is_multiple_of(2);
    let reverse_facts = !(world / 1440).is_multiple_of(2);
    let updated_color = COLORS[if counterfactual && query_updated {
        colors[3]
    } else {
        colors[1]
    }];
    let retained_color = COLORS[if counterfactual && !query_updated {
        colors[3]
    } else {
        colors[2]
    }];
    let mut prompt = if rust {
        String::from("fn main() {\n")
    } else {
        String::new()
    };
    let fact = if rust { rust_fact } else { fact };
    let retained_start = if reverse_facts {
        let start = fact(&mut prompt, OBJECTS[second], retained_color, false);
        fact(&mut prompt, OBJECTS[first], COLORS[colors[0]], false);
        start
    } else {
        fact(&mut prompt, OBJECTS[first], COLORS[colors[0]], false);
        fact(&mut prompt, OBJECTS[second], retained_color, false)
    };
    let updated_start = fact(&mut prompt, OBJECTS[first], updated_color, true);
    for _ in 0..filler_units {
        prompt.push_str(if rust { "let _ = 0;\n" } else { "wind moves. " });
    }
    let (query_object, expected_color, answer_color_start_byte) = if query_updated {
        (OBJECTS[first], updated_color, updated_start)
    } else {
        (OBJECTS[second], retained_color, retained_start)
    };
    if rust {
        prompt.push_str("assert_eq!(");
        prompt.push_str(query_object);
        prompt.push_str(", \"");
    } else {
        prompt.push_str("Question: What color is the ");
        prompt.push_str(query_object);
        // Stop at the punctuation boundary. The native codec owns leading spaces
        // with the following lexical piece; a trailing space would be a different
        // standalone prompt token from the training target " blue", for example.
        prompt.push_str("? Answer:");
    }
    Case {
        id: format!("{split}/world-{world}/filler-{filler_units}/swap-{counterfactual}"),
        world,
        filler_units,
        counterfactual,
        query_kind: if query_updated {
            "updated_fact"
        } else {
            "retained_fact"
        }
        .into(),
        prompt,
        expected_color: expected_color.into(),
        answer_color_start_byte,
        rust,
    }
}

fn sources(options: &Options) -> ProbeResult<Source> {
    if options.family == "mixed" {
        let mut combined = Source {
            schema: "uor-r4.native-controlled-memory-source/1".into(),
            construction: Vec::new(),
            readout_fit: Vec::new(),
            development: Vec::new(),
            split_policy: "Both existing finite prose/Rust families train and evaluate one artifact. Counts apply per family; IDs are family-prefixed and exact text/world splits remain disjoint between training and development within each family. Shared grammar and lexicon remain open development, not general conversation/coding or final held-out assessment.".into(),
        };
        for family in ["prose", "rust"] {
            let mut single = options.clone();
            single.family = family.into();
            let source = sources(&single)?;
            for (destination, cases) in [
                (&mut combined.construction, source.construction),
                (&mut combined.readout_fit, source.readout_fit),
                (&mut combined.development, source.development),
            ] {
                destination.extend(cases.into_iter().map(|mut case| {
                    case.id = format!("{family}/{}", case.id);
                    case
                }));
            }
        }
        return Ok(combined);
    }
    let permutations = color_permutations();
    let mut source = Source {
        schema: "uor-r4.native-controlled-memory-source/1".into(),
        construction: Vec::new(),
        readout_fit: Vec::new(),
        development: Vec::new(),
        split_policy: "Deterministic disjoint world IDs and exact document bytes. Count/readout fit are training; development is open controlled evaluation, not final held-out assessment. All splits share the finite grammar and lexicon. No independent natural-language or coding claim.".into(),
    };
    let mut texts = BTreeSet::new();
    let mut cursor = 0;
    for (split, count) in [
        ("construction", options.construction_documents),
        ("readout-fit", options.readout_documents),
    ] {
        let output = if split == "construction" {
            &mut source.construction
        } else {
            &mut source.readout_fit
        };
        while output.len() < count && cursor < WORLD_COUNT {
            let world = ordered_world(cursor);
            cursor += 1;
            let case = make_case(
                world,
                FILLER_UNITS[(output.len() / 4) % FILLER_UNITS.len()],
                false,
                split,
                &permutations,
                options.family == "rust",
            );
            if texts.insert(case.document().text) {
                output.push(case);
            }
        }
        if output.len() != count {
            return Err("insufficient distinct construction/readout worlds".into());
        }
    }
    let mut worlds = 0;
    while worlds < options.development_worlds && cursor < WORLD_COUNT {
        let world = ordered_world(cursor);
        cursor += 1;
        // Balance the two query roles even when a colliding family is skipped.
        if (world / 720) % 2 != worlds % 2 {
            continue;
        }
        let mut family = Vec::new();
        for filler in FILLER_UNITS {
            for counterfactual in [false, true] {
                family.push(make_case(
                    world,
                    filler,
                    counterfactual,
                    "development",
                    &permutations,
                    options.family == "rust",
                ));
            }
        }
        if family
            .iter()
            .any(|case| texts.contains(&case.document().text))
        {
            continue;
        }
        for case in family {
            if !texts.insert(case.document().text) {
                return Err("duplicate document inside a development family".into());
            }
            source.development.push(case);
        }
        worlds += 1;
    }
    if worlds != options.development_worlds {
        return Err("insufficient byte-disjoint development worlds".into());
    }
    Ok(source)
}

fn ordered_world(cursor: usize) -> usize {
    // Four-way blocks cover both query roles and initial-fact orders. The
    // coprime permutation of the remaining 720 cases is deterministic. Filler
    // length is shared inside a block, so it cannot predict the query role.
    ((cursor / 4 * 37) % 720) + (cursor % 2) * 720 + ((cursor / 2) % 2) * 1440
}

fn write_new(path: &Path, bytes: &[u8]) -> ProbeResult<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn budget(options: &Options, started: Instant, phase: &str) -> ProbeResult<()> {
    if started.elapsed().as_secs() >= options.max_seconds {
        return Err(format!("configured soft wall budget exhausted before {phase}").into());
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: usize) -> ProbeResult<Vec<u8>> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > maximum as u64 {
        return Err(format!(
            "{} exceeds the {} byte input bound",
            path.display(),
            maximum
        )
        .into());
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err("input grew beyond its byte bound while reading".into());
    }
    Ok(bytes)
}

fn validate_development(source: &Source, model: &Model) -> ProbeResult<()> {
    if source.schema != "uor-r4.native-controlled-memory-source/1"
        || source.development.is_empty()
        || source.development.len() > 512
    {
        return Err("unsupported source schema or development case count".into());
    }
    let mut ids = BTreeSet::new();
    let mut texts = BTreeSet::new();
    for case in source
        .construction
        .iter()
        .chain(&source.readout_fit)
        .chain(&source.development)
    {
        if case.id.trim().is_empty()
            || !ids.insert(&case.id)
            || case.prompt.len() > 256 * 1024
            || !case.prompt.is_char_boundary(case.answer_color_start_byte)
            || !texts.insert(case.document().text)
        {
            return Err(
                "source has repeated identities/text, an oversized prompt or invalid answer offset"
                    .into(),
            );
        }
    }
    for case in &source.development {
        let document = case.document();
        let cid = format!("blake3:{}", blake3::hash(document.text.as_bytes()));
        if model
            .construction()
            .iter()
            .chain(model.readout_training())
            .chain(model.memory_read_training())
            .any(|known| known.id == case.id || known.text_cid == cid)
        {
            return Err(format!(
                "development case {} overlaps this artifact's training",
                case.id
            )
            .into());
        }
    }
    Ok(())
}

// Host-side assessment only. Compile the exact generated bytes, without any
// target value, closing syntax, extraction or repair being inserted. No binary
// is produced or executed. Compiler diagnostics are evidence, never a response.
fn compile_rust_continuation(
    options: &Options,
    started: Instant,
    name: &str,
    case: &Case,
    continuation: &[u8],
) -> ProbeResult<Value> {
    let directory = options.output_dir.join("rust-compilation").join(name);
    fs::create_dir_all(&directory)?;
    let source_path = directory.join("generated.rs");
    let mut source = case.prompt.as_bytes().to_vec();
    source.extend_from_slice(continuation);
    if source.len() > 2 * 1024 * 1024 {
        return Err("generated Rust source exceeds 2 MiB".into());
    }
    write_new(&source_path, &source)?;
    let stdout_path = directory.join("stdout.txt");
    let stderr_path = directory.join("stderr.txt");
    let stdout = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stdout_path)?;
    let stderr = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stderr_path)?;
    let limit = Duration::from_secs(10)
        .min(Duration::from_secs(options.max_seconds).saturating_sub(started.elapsed()));
    if limit.is_zero() {
        return Ok(
            json!({"status":"not_run","reason":"budget_limit","compiled":null,"executed":false,
            "source":source_path,"source_blake3":blake3::hash(&source).to_hex().to_string()}),
        );
    }
    let user_home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME is required to locate the configured Rust toolchain")?;
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home.join(".cargo"));
    let rustup_home = std::env::var_os("RUSTUP_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home.join(".rustup"));
    let compiler = cargo_home.join("bin/rustc");
    let compiler_path = std::env::join_paths([
        cargo_home.join("bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ])?;
    let compiler_started = Instant::now();
    let mut child = match Command::new(&compiler)
        .env_clear()
        .env("HOME", &user_home)
        .env("PATH", compiler_path)
        .env("CARGO_HOME", &cargo_home)
        .env("RUSTUP_HOME", &rustup_home)
        .arg("--edition=2021")
        .arg("--crate-name=native_probe")
        .arg("--crate-type=bin")
        .arg("--emit=metadata")
        .arg(&source_path)
        .arg("-o")
        .arg(directory.join("generated.rmeta"))
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return Ok(json!({"status":"compiler_unavailable","compiled":null,
            "executed":false,"error":error.to_string(),"source":source_path}))
        }
    };
    let mut stop = "completed";
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        }
        let diagnostic_limit = [&stdout_path, &stderr_path]
            .iter()
            .any(|path| fs::metadata(path).is_ok_and(|m| m.len() > 1024 * 1024));
        if diagnostic_limit || compiler_started.elapsed() >= limit {
            stop = if diagnostic_limit {
                "diagnostic_limit"
            } else {
                "time_limit"
            };
            let _ = child.kill();
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut diagnostics = Vec::new();
    fs::File::open(&stderr_path)?
        .take(1024 * 1024)
        .read_to_end(&mut diagnostics)?;
    let compiled = (stop == "completed").then_some(status.success());
    Ok(json!({"status":stop,"compiled":compiled,
        "exit_code":status.code(),"executed":false,"source":source_path,
        "source_bytes":source.len(),"source_blake3":blake3::hash(&source).to_hex().to_string(),
        "stdout":stdout_path,"stderr":stderr_path,"diagnostics":String::from_utf8_lossy(&diagnostics),
        "diagnostics_truncated":fs::metadata(&stderr_path)?.len() > diagnostics.len() as u64,
        "elapsed_ms":compiler_started.elapsed().as_millis(),"time_limit_ms":limit.as_millis(),
        "compiler":compiler,"environment":"cleared; explicit HOME, PATH, CARGO_HOME and RUSTUP_HOME only",
        "command":"rustc --edition=2021 --crate-name=native_probe --crate-type=bin --emit=metadata generated.rs -o generated.rmeta",
        "scope":"Exact prompt plus generated continuation; binary-crate metadata checks include main requirements without linking, execution or a semantic correctness claim."}))
}

fn evaluate_case(
    model: &Model,
    case: &Case,
    control: Control,
    options: &Options,
    started: Instant,
    run_name: &str,
    compile_selected: bool,
) -> ProbeResult<Value> {
    let prompt_tokens = model.encode(&case.prompt)?;
    let answer_prefix = if case.rust { "" } else { " " };
    let answer_tokens = model.encode(&format!("{answer_prefix}{}", case.expected_color))?;
    if answer_tokens.len() != 1 {
        return Err("controlled answer color is not one native lexical token".into());
    }
    let full_document_tokens = model.encode(&case.document().text)?;
    if !full_document_tokens.starts_with(&prompt_tokens)
        || full_document_tokens.get(prompt_tokens.len()) != answer_tokens.first()
    {
        return Err("query prefix and supervised answer are not native-codec aligned".into());
    }
    let color_position = 1 + model
        .encode(case.prompt[..case.answer_color_start_byte].trim_end())?
        .len();
    let oldest_retained = (1 + prompt_tokens.len()).saturating_sub(model.config().context_tokens);
    let mut session = model.session(control)?;
    session.observe(model, BOS)?;
    for &token in &prompt_tokens {
        session.observe(model, token)?;
    }
    let prediction = session.predict(model)?;
    let target_in_shortlist = session
        .candidates()
        .iter()
        .any(|candidate| candidate.token == answer_tokens[0]);
    let predicted_bytes = model.decode(&[prediction.token])?;
    let continuation = model.generate(&case.prompt, options.generated_tokens, control)?;
    let compilation = if compile_selected {
        Some(compile_rust_continuation(
            options,
            started,
            run_name,
            case,
            &continuation.bytes,
        )?)
    } else if case.rust {
        Some(json!({"status":"not_run","compiled":null,"executed":false,
            "reason":if !options.compile_rust { "not_requested" } else if control != Control::Full { "full_control_only" } else { "compiler_case_budget" }}))
    } else {
        None
    };
    let answer_clause = format!(
        "{answer_prefix}{}{}",
        case.expected_color,
        if case.rust { "\"" } else { "." }
    );
    Ok(json!({
        "id":case.id,"world":case.world,"filler_units":case.filler_units,
        "family":if case.rust { "rust" } else { "prose" },
        "counterfactual":case.counterfactual,"query_kind":case.query_kind,
        "expected_color":case.expected_color,"expected_token":answer_tokens[0],
        "prompt_tokens":prompt_tokens.len(),"answer_color_position_including_bos":color_position,
        "answer_color_token_retained":color_position >= oldest_retained,
        "tokens_after_answer_color":prompt_tokens.len().saturating_sub(color_position),
        "prediction":prediction,"predicted_bytes":predicted_bytes,
        "predicted_text":String::from_utf8_lossy(&predicted_bytes),
        "query_correct":prediction.token == answer_tokens[0],
        "target_in_shortlist":target_in_shortlist,
        "answer_clause_prefix_correct":continuation.bytes.starts_with(answer_clause.as_bytes()),
        "query_state":session.state(),"query_work":session.work,"continuation":continuation,
        "rust_compilation":compilation
    }))
}

fn summarize(cases: &[Value], filler: usize, query_kind: &str, family: Option<&str>) -> Value {
    let selected: Vec<_> = cases
        .iter()
        .filter(|case| {
            case["filler_units"].as_u64() == Some(filler as u64)
                && case["query_kind"] == query_kind
                && family.is_none_or(|name| case["family"] == name)
        })
        .collect();
    let correct = selected
        .iter()
        .filter(|case| case["query_correct"] == true)
        .count();
    let coverage = selected
        .iter()
        .filter(|case| case["target_in_shortlist"] == true)
        .count();
    let retained = selected
        .iter()
        .filter(|case| case["answer_color_token_retained"] == true)
        .count();
    let clause = selected
        .iter()
        .filter(|case| case["answer_clause_prefix_correct"] == true)
        .count();
    let mut pairs = 0;
    let mut both_correct = 0;
    let mut changed = 0;
    for pair in selected.chunks_exact(2) {
        pairs += 1;
        both_correct +=
            usize::from(pair[0]["query_correct"] == true && pair[1]["query_correct"] == true);
        changed += usize::from(pair[0]["prediction"]["token"] != pair[1]["prediction"]["token"]);
    }
    json!({"family":family,"filler_units":filler,"query_kind":query_kind,"cases":selected.len(),
        "query_correct":correct,"query_accuracy": if selected.is_empty() { None } else { Some(correct as f64 / selected.len() as f64) },
        "target_in_shortlist":coverage,"answer_color_token_retained":retained,
        "answer_clause_prefix_correct":clause,"counterfactual_pairs":pairs,
        "pairs_both_correct":both_correct,"pairs_prediction_changed":changed})
}

fn evaluate_models(
    options: &Options,
    source: &Source,
    started: Instant,
    models: Vec<(&str, &Model)>,
) -> ProbeResult<Vec<Value>> {
    let mut arms = Vec::new();
    for (name, model) in models {
        validate_development(source, model)?;
        let mut compiler_cases = 0;
        let controls = if model.memory_read_version().is_some() {
            vec![
                Control::Full,
                Control::MemoryDisabled,
                Control::GeometryDisabled,
                Control::ZetaDisabled,
                Control::H4Disabled,
            ]
        } else {
            vec![Control::Full, Control::GeometryDisabled]
        };
        for control in controls {
            let mut cases = Vec::new();
            for (case_index, case) in source.development.iter().enumerate() {
                budget(options, started, "next open-development case")?;
                let compile_selected = options.compile_rust
                    && case.rust
                    && control == Control::Full
                    && compiler_cases < options.compiler_cases;
                if compile_selected {
                    compiler_cases += 1;
                }
                cases.push(evaluate_case(
                    model,
                    case,
                    control,
                    options,
                    started,
                    &format!("{name}-{control:?}-{case_index}"),
                    compile_selected,
                )?);
            }
            let mut summaries = Vec::new();
            for filler in FILLER_UNITS {
                for query_kind in ["updated_fact", "retained_fact"] {
                    summaries.push(summarize(&cases, filler, query_kind, None));
                }
            }
            let mut family_summaries = Vec::new();
            for family in ["prose", "rust"] {
                if !cases.iter().any(|case| case["family"] == family) {
                    continue;
                }
                for filler in FILLER_UNITS {
                    for query_kind in ["updated_fact", "retained_fact"] {
                        family_summaries.push(summarize(&cases, filler, query_kind, Some(family)));
                    }
                }
            }
            let rust_cases: Vec<_> = cases
                .iter()
                .filter_map(|case| {
                    case.get("rust_compilation")
                        .filter(|value| !value.is_null())
                })
                .collect();
            let compile_summary = json!({"requested":options.compile_rust,"cases":rust_cases.len(),
                "selection":"First configured number of Rust development cases per artifact, Full control only",
                "case_budget":options.compiler_cases,
                "not_run":rust_cases.iter().filter(|value| value["status"] == "not_run").count(),
                "compiled":rust_cases.iter().filter(|value| value["compiled"] == true).count(),
                "compiler_unavailable":rust_cases.iter().filter(|value| value["status"] == "compiler_unavailable").count(),
                "limited":rust_cases.iter().filter(|value| value["status"] == "time_limit" || value["status"] == "diagnostic_limit").count(),
                "executed":false,"scope":"Compilation of exact model continuations, separate from query/clause accuracy; no semantic execution."});
            arms.push(
                json!({"readout":name,"control":control,"artifact_cid":model.artifact_cid(),
                "uor_model_address":model.uor_model_address(),"rust_compilation":compile_summary,"summary":summaries,"family_summary":family_summaries,"cases":cases}),
            );
        }
    }
    Ok(arms)
}

fn execute_loaded(
    options: &Options,
    source: &Source,
    started: Instant,
    path: &Path,
) -> ProbeResult<Value> {
    budget(options, started, "artifact loading")?;
    let bytes = read_bounded(path, 256 * 1024 * 1024)?;
    let model = Model::from_bytes(&bytes)?;
    let arms = evaluate_models(options, source, started, vec![("loaded_artifact", &model)])?;
    Ok(
        json!({"schema":"uor-r4.native-controlled-memory-probe/1","status":"completed",
        "mode":"evaluation_only","scope":"Preserved development cases and an existing artifact; no fitting, answer insertion or code execution. Compilation, if requested, consumes the exact prompt plus generated bytes.",
        "options":options,"model_config":model.config(),"model_bytes":bytes.len(),
        "artifact_cid":model.artifact_cid(),"memory_read_version":model.memory_read_version(),
        "memory_read_config":model.memory_read_config(),"source_split_policy":source.split_policy,
        "development_overlap_check":"IDs and exact document bytes checked against construction, readout-fit and memory-fit receipts before prediction",
        "resources":{"wall_ms":started.elapsed().as_millis(),"max_seconds":options.max_seconds,
            "fitting":"NOT_RUN: evaluation-only mode","evaluation_threads":1,
            "compiler":"Optional metadata-only rustc subprocesses; at most 10 seconds each and within remaining soft wall budget; no binary execution",
            "peak_rss":"NOT_MEASURED; use an external process monitor if required"},"arms":arms}),
    )
}

fn execute(options: &Options, source: &Source, started: Instant) -> ProbeResult<Value> {
    let construction: Vec<_> = source.construction.iter().map(Case::document).collect();
    let readout: Vec<_> = source.readout_fit.iter().map(Case::document).collect();
    let config = Config {
        context_tokens: options.context,
        candidate_limit: 32,
        max_lexical_pieces: 512,
        max_rows: 65_536,
        max_associations: 500_000,
        ..Config::default()
    };
    budget(options, started, "model construction")?;
    let mut trainer = Trainer::new(config, &construction)?;
    for document in &construction {
        budget(options, started, "next construction document")?;
        trainer.train_documents(std::slice::from_ref(document))?;
    }
    let fixed = trainer.compile()?;
    let fixed_bytes = fixed.to_bytes()?;
    write_new(&options.output_dir.join("fixed-model.json"), &fixed_bytes)?;
    budget(options, started, "construction-only readout fitting")?;
    let (learned, fit_report) = fixed.fit_readout(
        &readout,
        ReadoutFitConfig {
            max_positions: options.fit_positions,
            epochs: options.fit_epochs,
            max_queries: 64,
        },
    )?;
    let learned_bytes = learned.to_bytes()?;
    write_new(
        &options.output_dir.join("learned-model.json"),
        &learned_bytes,
    )?;
    // Exercise both serialized artifacts; evaluated instances are actual reloads.
    let fixed = Model::from_bytes(&fixed_bytes)?;
    let learned = Model::from_bytes(&learned_bytes)?;
    let mut memory_fit_report = None;
    let mut memory_bytes_len = 0;
    let memory = if options.memory_read {
        budget(options, started, "construction-only memory-read fitting")?;
        let defaults = MemoryReadFitConfig::default();
        let config = MemoryReadFitConfig {
            max_positions: options.fit_positions,
            epochs: options.fit_epochs,
            query_tokens: options.query_tokens.unwrap_or(defaults.query_tokens),
            source_offsets: options.source_offsets.unwrap_or(defaults.source_offsets),
            postings_per_address: options
                .postings_per_address
                .unwrap_or(defaults.postings_per_address),
            candidate_limit: options
                .memory_candidates
                .unwrap_or(defaults.candidate_limit),
            ..defaults
        };
        let (model, report) = if options.query_context {
            learned.fit_memory_read_with_query_context(&readout, config, options.word_cues)
        } else if options.word_cues {
            learned.fit_memory_read_with_word_cues(&readout, config)
        } else {
            learned.fit_memory_read(&readout, config)
        }?;
        let bytes = model.to_bytes()?;
        memory_bytes_len = bytes.len();
        write_new(&options.output_dir.join("memory-model.json"), &bytes)?;
        memory_fit_report = Some(report);
        Some(Model::from_bytes(&bytes)?)
    } else {
        None
    };
    let mut models = vec![("fixed_readout", &fixed), ("learned_readout", &learned)];
    if let Some(model) = &memory {
        models.push(("learned_memory_read", model));
    }
    let arms = evaluate_models(options, source, started, models)?;
    Ok(json!({
        "schema":"uor-r4.native-controlled-memory-probe/1","status":"completed",
        "scope":"Finite supplied-value retention/update in prose or Rust variable/assertion syntax, with matched filler and counterfactual pairs. This evaluates the actual native predictor and generated continuations. Open development only; no general coding, broad language, long-term memory, geometric-advantage or alpha claim. Rust syntax is training/evaluation data only, never a model parser or answer template.",
        "options":options,"training":trainer.progress(),"readout_fit":fit_report,"memory_read_fit":memory_fit_report,
        "readout_query_targets_sampled":null,
        "readout_query_targets_status":"NOT_MEASURED: readout_fit reports the actual uniform per-document sampler totals; it does not expose which sampled positions are answer-query targets. No concatenated-prefix estimate is substituted.",
        "source_split_policy":source.split_policy,
        "controls":"GeometryDisabled removes admitted geometric readout channels and their candidate postings while retaining the state-update kernel. MemoryDisabled isolates the optional learned memory-read operator. The learned lexical gate may differ from the fixed baseline; compare full/off within each artifact. Consult the model operator definition for relative memory geometry controls.",
        "window_comparison":"Use the same source hash and options except context/output-dir. Each context constructs and fits a separate artifact, changing both training state and the inference window. Report physical retained-color availability separately from correct prediction and paired answer changes. A larger window retains information but does not guarantee learning to use it.",
        "resources":{"wall_ms":started.elapsed().as_millis(),"max_seconds":options.max_seconds,
            "fixed_model_bytes":fixed_bytes.len(),"learned_model_bytes":learned_bytes.len(),
            "memory_model_bytes":memory_bytes_len,
            "model_training_epochs":1,"fitting_threads":1,"evaluation_threads":1,
            "peak_rss":"NOT_MEASURED; use an external process resource monitor if required",
            "budget_boundary":"Soft wall checks between documents and development cases; finite readout fit, compile and reload may overrun. No automatic retries or additional runs."},
        "arms":arms
    }))
}

fn main() -> ProbeResult<()> {
    let options = Options::parse()?;
    let started = Instant::now();
    fs::create_dir(&options.output_dir)?;
    let (source, source_bytes) = if let Some(path) = &options.source {
        let bytes = read_bounded(path, 16 * 1024 * 1024)?;
        (serde_json::from_slice::<Source>(&bytes)?, bytes)
    } else {
        let source = sources(&options)?;
        let bytes = serde_json::to_vec_pretty(&source)?;
        (source, bytes)
    };
    write_new(&options.output_dir.join("source.json"), &source_bytes)?;
    let outcome = if let Some(path) = &options.model {
        execute_loaded(&options, &source, started, path)
    } else {
        execute(&options, &source, started)
    };
    let succeeded = outcome.is_ok();
    let mut report = outcome.unwrap_or_else(|error| json!({
        "schema":"uor-r4.native-controlled-memory-probe/1","status":"incomplete",
        "error":error.to_string(),"options":options,"wall_ms":started.elapsed().as_millis(),
        "interpretation":"Execution/resource result, not a model-quality verdict. Existing output files are preserved."
    }));
    report["source_blake3"] = json!(blake3::hash(&source_bytes).to_hex().to_string());
    report["source_bytes"] = json!(source_bytes.len());
    write_new(
        &options.output_dir.join("report.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if succeeded {
        Ok(())
    } else {
        Err("controlled memory experiment incomplete; inspect report.json".into())
    }
}
