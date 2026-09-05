//! Small joint open-development exercise for learned typed-value selection.
//!
//! Fit receives raw prompt/response bytes only. Task and counterfactual labels
//! are probe diagnostics and never enter inference or provide source positions.
//! Generated code is saved unchanged; this probe does not compile or execute it.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::{
    Control, Model, ResponseEntryFitConfig, ValueCompletionFitConfig, ValueExample, ValueFitConfig,
};

type ProbeResult<T> = Result<T, Box<dyn Error>>;
#[path = "native_geometric_value_probe/wording.rs"]
mod wording;
const SOURCE_SCHEMA: &str = "uor-r4.native-typed-value-source/1";
const LEXEME_SOURCE_SCHEMA: &str = "uor-r4.native-typed-value-source/2";
const WORD_COPY_SOURCE_SCHEMA: &str = "uor-r4.native-typed-value-source/3";
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const COMPLETION_OUTPUT_BYTES: usize = 20 * 1024 * 1024;
const PRESERVATION_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
static OUTPUT_BYTES_REMAINING: AtomicUsize = AtomicUsize::new(usize::MAX);

#[derive(Debug, Serialize)]
struct Options {
    mode: String,
    model: PathBuf,
    source: Option<PathBuf>,
    output_dir: PathBuf,
    fit_worlds: usize,
    development_worlds: usize,
    epochs: usize,
    learning_rate: f64,
    max_features: usize,
    generated_tokens: usize,
    max_seconds: u64,
    lexeme_cues: bool,
    controls: String,
    completion_max_positions: usize,
}

impl Options {
    fn parse() -> ProbeResult<Self> {
        let mut arguments = std::env::args().skip(1).peekable();
        let mode = if arguments.peek().is_some_and(|arg| !arg.starts_with('-')) {
            arguments.next().ok_or("missing mode")?
        } else {
            "fit".into()
        };
        let mut result = Self {
            mode,
            model: PathBuf::new(),
            source: None,
            output_dir: PathBuf::new(),
            fit_worlds: 16,
            development_worlds: 4,
            epochs: 24,
            learning_rate: 0.1,
            max_features: 65_536,
            generated_tokens: 32,
            max_seconds: 120,
            lexeme_cues: false,
            controls: "all".into(),
            completion_max_positions: 4096,
        };
        while let Some(flag) = arguments.next() {
            if flag == "--help" || flag == "-h" {
                println!(
                    "native_geometric_value_probe [prepare|prepare-copy|prepare-facts|prepare-wording|fit|completion|entry|copy|copy-completed|copy-composed|copy-binding|copy-binding-plain|evaluate] --output-dir NEW_DIRECTORY\n\
                     prepare-copy: --source SOURCE_V2 --lexeme-cues true\n\
                     copy: --model ENTRY_MODEL --source SOURCE_V3 --lexeme-cues true --generated-tokens 64\n\
                     copy-completed: same source/parent, suffix frame starts after the observed copied word\n\
                     fit: --model BASELINE [--source source.json]\n\
                     completion: --model TYPED_MODEL --source SOURCE_V2 --lexeme-cues true\n\
                     entry: --model COMPLETION_MODEL --source SOURCE_V2 --lexeme-cues true --generated-tokens 64\n\
                     evaluate: --model VALUE_MODEL --source source.json\n\
                     evaluate --controls full: only Full, including the existing binding pairs\n\
                     --fit-worlds EVEN_NUMBER --development-worlds EVEN_NUMBER\n\
                     --epochs N --learning-rate F --max-features N\n\
                     --completion-max-positions N (default 4096, completion or entry fitting)\n\
                     --generated-tokens N --max-seconds N\n\
                     --lexeme-cues true|false (default false; true appends 64 fit\n\
                     name swaps at default world count and uses source schema /2)\n\
                     Completion output is capped at 20 MiB; Full-only preservation at 8 MiB.\n\
                     Defaults: 128 fit and 32 development cases across prose/Rust;\n\
                     paired worlds alter one numeric literal. Full byte exactness,\n\
                     leading numeral correctness and no-write behavior stay separate.\n\
                     The elapsed limit is checked between bounded operations; fitting\n\
                     and artifact serialization may overrun. Use an external monitor."
                );
                std::process::exit(0);
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--model" => result.model = value.into(),
                "--source" => result.source = Some(value.into()),
                "--output-dir" => result.output_dir = value.into(),
                "--fit-worlds" => result.fit_worlds = value.parse()?,
                "--development-worlds" => result.development_worlds = value.parse()?,
                "--epochs" => result.epochs = value.parse()?,
                "--learning-rate" => result.learning_rate = value.parse()?,
                "--max-features" => result.max_features = value.parse()?,
                "--generated-tokens" => result.generated_tokens = value.parse()?,
                "--max-seconds" => result.max_seconds = value.parse()?,
                "--lexeme-cues" => result.lexeme_cues = value.parse()?,
                "--controls" => result.controls = value,
                "--completion-max-positions" | "--entry-max-positions" => {
                    result.completion_max_positions = value.parse()?
                }
                _ => return Err(format!("unknown option {flag}").into()),
            }
        }
        if ![
            "prepare",
            "prepare-copy",
            "prepare-facts",
            "fit",
            "completion",
            "entry",
            "copy",
            "copy-completed",
            "copy-composed",
            "copy-binding",
            "copy-binding-plain",
            "evaluate",
        ]
        .contains(&result.mode.as_str())
            || result.output_dir.as_os_str().is_empty()
            || (!["prepare", "prepare-copy", "prepare-facts"].contains(&result.mode.as_str())
                && result.model.as_os_str().is_empty())
            || ([
                "prepare-copy",
                "prepare-facts",
                "completion",
                "entry",
                "copy",
                "copy-completed",
                "copy-composed",
                "copy-binding",
                "copy-binding-plain",
                "evaluate",
            ]
            .contains(&result.mode.as_str())
                && result.source.is_none())
            || ([
                "prepare-copy",
                "prepare-facts",
                "completion",
                "entry",
                "copy",
                "copy-completed",
                "copy-composed",
                "copy-binding",
                "copy-binding-plain",
            ]
            .contains(&result.mode.as_str())
                && !result.lexeme_cues)
            || ([
                "completion",
                "entry",
                "copy",
                "copy-completed",
                "copy-composed",
                "copy-binding",
                "copy-binding-plain",
            ]
            .contains(&result.mode.as_str())
                && result.epochs > 64)
            || !(1..=4096).contains(&result.completion_max_positions)
            || !["all", "full"].contains(&result.controls.as_str())
            || (result.controls == "full" && result.mode != "evaluate")
            || !(2..=128).contains(&result.fit_worlds)
            || !(2..=32).contains(&result.development_worlds)
            || !result.fit_worlds.is_multiple_of(2)
            || !result.development_worlds.is_multiple_of(2)
            || !(1..=128).contains(&result.epochs)
            || !result.learning_rate.is_finite()
            || !(0.0..=1.0).contains(&result.learning_rate)
            || result.learning_rate == 0.0
            || !(1..=262_144).contains(&result.max_features)
            || !(1..=256).contains(&result.generated_tokens)
            || result.max_seconds == 0
        {
            return Err("unsupported options; see --help".into());
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    family: String,
    task: String,
    world: usize,
    pair_id: String,
    variant: usize,
    prompt: String,
    response: String,
}

impl Case {
    fn example(&self) -> ValueExample {
        ValueExample {
            id: self.id.clone(),
            prompt: self.prompt.clone(),
            response: self.response.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema: String,
    scope: String,
    tokenization_law: String,
    fit: Vec<Case>,
    development: Vec<Case>,
}

#[derive(Debug, Serialize)]
struct BindingPair {
    pair_id: String,
    /// Identical query bytes follow both versions of the supplied context.
    query: String,
    original: Case,
    swapped: Case,
}

#[derive(Debug, Serialize)]
struct BindingSource {
    schema: String,
    scope: String,
    pairs: Vec<BindingPair>,
}

// A source intervention, not a model-side parsing rule. All four pairs use
// authored constants and preserve the exact numeric-byte sequence and query.
// Only the identity attached to source values changes. A fixed numeric rank
// therefore cannot return both correct answers within a pair.
fn binding_source() -> ProbeResult<BindingSource> {
    let (value, delta, earlier, irrelevant) = (73_i64, 4_i64, 31_i64, 301_i64);
    let mut pairs = Vec::new();
    for family in ["prose", "rust"] {
        for task in ["copy_current", "add"] {
            let (context, swapped_context, query, response, swapped_response) =
                match (family, task) {
                    ("prose", "copy_current") => (
                        format!("User: suri had {earlier} coins. tavi has {irrelevant} coins.\nUser: Update: suri now has {value} coins.\n"),
                        format!("User: tavi had {earlier} coins. suri has {irrelevant} coins.\nUser: Update: tavi now has {value} coins.\n"),
                        "User: How many coins does suri have now?\nAssistant:".to_owned(),
                        format!("{value}.\n"),
                        format!("{irrelevant}.\n"),
                    ),
                    ("prose", "add") => (
                        format!("User: suri has {value} coins. orin has {delta} coins. tavi has {irrelevant} coins.\n"),
                        format!("User: suri has {value} coins. tavi has {delta} coins. orin has {irrelevant} coins.\n"),
                        "User: What is the sum of suri's and orin's coins?\nAssistant:".to_owned(),
                        format!("{}.\n", value + delta),
                        format!("{}.\n", value + irrelevant),
                    ),
                    ("rust", "copy_current") => (
                        format!("fn main() {{\n    let mut suri = {earlier};\n    let tavi = {irrelevant};\n    suri = {value};\n    "),
                        format!("fn main() {{\n    let mut tavi = {earlier};\n    let suri = {irrelevant};\n    tavi = {value};\n    "),
                        "assert_eq!(suri,".to_owned(),
                        format!("{value});\n}}\n"),
                        format!("{irrelevant});\n}}\n"),
                    ),
                    _ => (
                        format!("fn main() {{\n    let suri = {value};\n    let orin = {delta};\n    let tavi = {irrelevant};\n    "),
                        format!("fn main() {{\n    let suri = {value};\n    let tavi = {delta};\n    let orin = {irrelevant};\n    "),
                        "assert_eq!(suri + orin,".to_owned(),
                        format!("{});\n}}\n", value + delta),
                        format!("{});\n}}\n", value + irrelevant),
                    ),
                };
            let pair_id = format!("typed-value/binding/{family}/{task}");
            let original = Case {
                id: format!("{pair_id}/original"),
                family: family.into(),
                task: task.into(),
                world: 100_000,
                pair_id: pair_id.clone(),
                variant: 0,
                prompt: format!("{context}{query}"),
                response,
            };
            let swapped = Case {
                id: format!("{pair_id}/source_names_swapped"),
                variant: 1,
                prompt: format!("{swapped_context}{query}"),
                response: swapped_response,
                ..original.clone()
            };
            if raw_digit_runs(original.prompt.as_bytes())
                != raw_digit_runs(swapped.prompt.as_bytes())
                || !original.prompt.ends_with(&query)
                || !swapped.prompt.ends_with(&query)
                || original.response == swapped.response
            {
                return Err(
                    "binding intervention changed numeric bytes/query or failed to change answer"
                        .into(),
                );
            }
            pairs.push(BindingPair {
                pair_id,
                query,
                original,
                swapped,
            });
        }
    }
    Ok(BindingSource {
        schema: "uor-r4.native-typed-value-binding-control-source/1".into(),
        scope: "Four authored source-name swaps, eight Full-only runs, frozen before fitting. Original numeric literals, their order and query bytes remain identical within each pair; source entity names change and expected answers change. The originals duplicate four default primary development prompts deliberately. Neither originals nor swaps are added to fit. These controls distinguish fixed numeric-occurrence rank selection from sensitivity to source identity in this tiny scope; success alone does not establish general identity binding or reasoning.".into(),
        pairs,
    })
}

/// Exact runs of ASCII digit bytes, including any digits embedded in source
/// types. This validates a text intervention; it is not the model's lexer.
fn raw_digit_runs(bytes: &[u8]) -> Vec<&[u8]> {
    let mut runs = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_digit() {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            runs.push(&bytes[start..cursor]);
        } else {
            cursor += 1;
        }
    }
    runs
}

fn make_cases(split: &str, worlds: usize) -> Vec<Case> {
    let mut cases = Vec::with_capacity(worlds * 8);
    let development = split == "development";
    for local_world in 0..worlds {
        let world = if development { 100_000 } else { 0 } + local_world;
        let pair = local_world / 2;
        let variant = local_world % 2;
        let names = if development {
            ["suri", "orin", "tavi"]
        } else if pair.is_multiple_of(2) {
            ["ada", "ben", "cyra"]
        } else {
            ["dara", "eli", "finn"]
        };
        let [first, second, distractor] = names;
        let magnitude = if development { 73 } else { 11 } + pair as i64 * 5;
        let original = if pair % 3 == 1 { -magnitude } else { magnitude };
        let value = original + variant as i64 * 3;
        let delta = 4 + pair as i64 % 5;
        let earlier = 31 + pair as i64;
        let reassigned = 181 + pair as i64;
        let irrelevant = 301 + pair as i64;
        for family in ["prose", "rust"] {
            for task in [
                "copy_current",
                "add",
                "dependency_before_update",
                "no_write",
            ] {
                let (prompt, response) = if family == "prose" {
                    match task {
                        "copy_current" => (
                            format!("User: {first} had {earlier} coins. {distractor} has {irrelevant} coins.\nUser: Update: {first} now has {value} coins.\nUser: How many coins does {first} have now?\nAssistant:"),
                            format!("{value}.\n"),
                        ),
                        "add" => (
                            format!("User: {first} has {value} coins. {second} has {delta} coins. {distractor} has {irrelevant} coins.\nUser: What is the sum of {first}'s and {second}'s coins?\nAssistant:"),
                            format!("{}.\n", value + delta),
                        ),
                        "dependency_before_update" => (
                            format!("User: {first} has {value} coins. {second}'s recorded count is {first}'s count plus {delta}.\nUser: {first} now has {reassigned} coins. {distractor} has {irrelevant} coins.\nUser: What was {second}'s recorded count before the update?\nAssistant:"),
                            format!("{}.\n", value + delta),
                        ),
                        _ => (
                            format!("User: {first} has {value} coins. {distractor} has {irrelevant} coins.\nUser: What city does {second} live in?\nAssistant:"),
                            " Unknown. The conversation does not give a city.\n".into(),
                        ),
                    }
                } else {
                    match task {
                        "copy_current" => (
                            format!("fn main() {{\n    let mut {first} = {earlier};\n    let {distractor} = {irrelevant};\n    {first} = {value};\n    assert_eq!({first},"),
                            format!("{value});\n}}\n"),
                        ),
                        "add" => (
                            format!("fn main() {{\n    let {first} = {value};\n    let {second} = {delta};\n    let {distractor} = {irrelevant};\n    assert_eq!({first} + {second},"),
                            format!("{});\n}}\n", value + delta),
                        ),
                        "dependency_before_update" => (
                            format!("fn main() {{\n    let mut {first} = {value};\n    let {second} = {first} + {delta};\n    {first} = {reassigned};\n    let {distractor} = {irrelevant};\n    assert_eq!({second},"),
                            format!("{});\n}}\n", value + delta),
                        ),
                        _ => (
                            format!("// {first} has {value} coins; {distractor} has {irrelevant}.\n// Return the input unchanged.\nfn identity(value: i32) -> i32 {{\n    "),
                            "value\n}\n".into(),
                        ),
                    }
                };
                cases.push(Case {
                    id: format!("typed-value/{split}/{family}/{task}/{world}"),
                    family: family.into(),
                    task: task.into(),
                    world,
                    pair_id: format!("typed-value/{split}/{family}/{task}/pair-{pair}"),
                    variant,
                    prompt,
                    response,
                });
            }
        }
    }
    cases
}

/// Additional raw fit examples only. Each appended row has a corresponding
/// original row in make_cases("fit", worlds). Context names exchange roles;
/// numeric source order and the entire query stay fixed. The expected answer
/// comes from authored world constants, never from a model-side source index.
fn make_fit_name_swaps(worlds: usize) -> ProbeResult<Vec<Case>> {
    let mut swaps = Vec::with_capacity(worlds * 4);
    for original in make_cases("fit", worlds)
        .into_iter()
        .filter(|case| ["copy_current", "add"].contains(&case.task.as_str()))
    {
        let pair = original.world / 2;
        let [first, second, distractor] = if pair.is_multiple_of(2) {
            ["ada", "ben", "cyra"]
        } else {
            ["dara", "eli", "finn"]
        };
        let magnitude = 11 + pair as i64 * 5;
        let initial = if pair % 3 == 1 { -magnitude } else { magnitude };
        let value = initial + original.variant as i64 * 3;
        let irrelevant = 301 + pair as i64;
        let (left, expected) = if original.task == "copy_current" {
            (first, irrelevant)
        } else {
            (second, value + irrelevant)
        };
        let query_start = original
            .prompt
            .rfind(if original.family == "prose" {
                "User: "
            } else {
                "assert_eq!"
            })
            .ok_or("authored fit case lacks query boundary")?;
        let (context, query) = original.prompt.split_at(query_start);
        // All six authored fit names are distinct literal strings absent from
        // ordinary words in these contexts. The temporary marker is never saved.
        let swapped_context = context
            .replace(left, "\0")
            .replace(distractor, left)
            .replace('\0', distractor);
        let swapped = Case {
            id: format!("{}/source_names_swapped", original.id),
            pair_id: format!("{}/source_names_swapped", original.pair_id),
            prompt: format!("{swapped_context}{query}"),
            response: if original.family == "prose" {
                format!("{expected}.\n")
            } else {
                format!("{expected});\n}}\n")
            },
            ..original.clone()
        };
        if raw_digit_runs(original.prompt.as_bytes()) != raw_digit_runs(swapped.prompt.as_bytes())
            || !swapped.prompt.ends_with(query)
            || original.response == swapped.response
            || original.prompt == swapped.prompt
        {
            return Err("fit name swap changed numeric bytes/query or not the answer".into());
        }
        swaps.push(swapped);
    }
    Ok(swaps)
}

fn source(options: &Options) -> ProbeResult<Source> {
    let mut result = if let Some(path) = &options.source {
        if fs::metadata(path)?.len() > MAX_SOURCE_BYTES {
            return Err("value source exceeds 4 MiB".into());
        }
        serde_json::from_slice(&fs::read(path)?)?
    } else {
        Source {
            schema: SOURCE_SCHEMA.into(),
            scope: "Small finite synthetic open development with shared templates, different names/values and disjoint world/document IDs. Counterfactual pairs change one relevant input literal. Both families share one fitted artifact. Task and pair labels are evaluator-only. The dependency examples test response-time original-occurrence binding; they do not demonstrate eager statement execution, arbitrary Rust interpretation, retention beyond eviction or alpha.".into(),
            tokenization_law: "Numeric responses begin with exact signed decimal bytes, without an implicit leading space. Their punctuation and closing code are ordinary generated output. Leading-numeral accuracy and complete byte equality are separate from canonical lexical-token accuracy. No suffix or answer template is appended to generated output.".into(),
            fit: make_cases("fit", options.fit_worlds),
            development: make_cases("development", options.development_worlds),
        }
    };
    if options.source.is_none() && options.lexeme_cues {
        result.schema = LEXEME_SOURCE_SCHEMA.into();
        result.scope.push_str(" Source /2 appends context-only copy/add name swaps to the unchanged /1 fit cases, with fixed numeric source order and query bytes but changed correct operands. Whole-word cues are an explicit model variant. The unchanged primary development cases and four binding controls have already informed this design and are reused OPEN development feedback, not sealed or final held-out evaluation.");
        result.fit.extend(make_fit_name_swaps(options.fit_worlds)?);
    }
    if options.mode == "prepare-copy" {
        append_word_copy_cases(&mut result)?;
    }
    if options.mode == "prepare-facts" {
        append_fact_cases(&mut result)?;
    }
    validate_source(&result)?;
    if (result.schema != SOURCE_SCHEMA) != options.lexeme_cues {
        return Err(
            "source schema must match --lexeme-cues; prepare a new /2 source with true".into(),
        );
    }
    if [
        "copy",
        "copy-completed",
        "copy-composed",
        "copy-binding",
        "copy-binding-plain",
    ]
    .contains(&options.mode.as_str())
        && result.schema != WORD_COPY_SOURCE_SCHEMA
    {
        return Err("copy fitting requires the prepared /3 source".into());
    }
    Ok(result)
}

// These are authored construction and OPEN development cases. Inference sees
// their prompt bytes only; neither task labels nor expected source positions
// enter the model. The original /2 cases remain byte-for-byte unchanged.
fn append_word_copy_cases(source: &mut Source) -> ProbeResult<()> {
    if source.schema != LEXEME_SOURCE_SCHEMA {
        return Err("prepare-copy requires an unextended /2 source".into());
    }
    let names = [
        "item", "datum", "entry", "token", "scalar", "number", "sample", "point",
    ];
    for context in 0..4 {
        for (index, name) in names.iter().enumerate() {
            source.fit.push(word_copy_case("fit", context, index, name));
        }
    }
    for (pair, names) in [
        ["input", "count"],
        ["payload", "argument"],
        ["amount", "operand"],
        ["buffer", "element"],
        ["left", "right"],
        ["value_a", "value_b"],
    ]
    .iter()
    .enumerate()
    {
        let context = [0, 0, 1, 2, 3, 1][pair];
        for (variant, name) in names.iter().enumerate() {
            let mut case = word_copy_case("development", context, pair * 2 + variant, name);
            case.pair_id = format!("word-copy/development/pair-{pair}");
            source.development.push(case);
        }
    }
    for (variant, city) in ["Oslo", "Lima"].iter().enumerate() {
        source.development.push(Case {
            id: format!("word-copy/development/city-{variant}"),
            family: "prose".into(), task: "city_transfer".into(), world: 300_000 + variant,
            pair_id: "word-copy/development/city".into(), variant,
            prompt: format!("User: suri has 73 coins. tavi has 301 coins.\nUser: orin lives in {city}.\nUser: What city does orin live in?\nAssistant:"),
            response: format!(" {city}.\n"),
        });
    }
    source.schema = WORD_COPY_SOURCE_SCHEMA.into();
    source.scope.push_str(" Source /3 preserves every /2 fit and development case and appends 32 construction parameter-name cases, 12 OPEN parameter-transfer cases, and the two already observed city failures. Parameter variants change retained rank, add a misleading comment, or duplicate the spelling; the target alone does not identify which equal-spelling occurrence was selected. Construction names differ from transfer names. These are development feedback, not final held-out qualification. The city responses begin with a space and remain a boundary diagnostic for first-position copying.");
    Ok(())
}

fn word_copy_case(split: &str, context: usize, index: usize, name: &str) -> Case {
    let development = split == "development";
    let numeric = if development {
        "// suri has 73 coins; tavi has 301.\n"
    } else {
        "// ada has 11 coins; cyra has 301.\n"
    };
    let comment = match context {
        1 => "// the result is unchanged\n    ".to_owned(),
        2 => "// input is mentioned again\n    ".to_owned(),
        3 => format!("// {name} is the supplied argument\n    "),
        _ => String::new(),
    };
    Case {
        id: format!("word-copy/{split}/context-{context}/name-{index}"),
        family: "rust".into(), task: "parameter_transfer".into(),
        world: if development { 200_000 + index } else { 2_000 + index },
        pair_id: format!("word-copy/{split}/context-{context}/pair-{}", index / 2),
        variant: index % 2,
        prompt: format!("{numeric}// Return the input unchanged.\nfn identity({name}: i32) -> i32 {{\n    {comment}"),
        response: format!("{name}\n}}\n"),
    }
}

fn validate_source(source: &Source) -> ProbeResult<()> {
    if ![SOURCE_SCHEMA, LEXEME_SOURCE_SCHEMA, WORD_COPY_SOURCE_SCHEMA]
        .contains(&source.schema.as_str())
        || source.fit.is_empty()
        || source.development.is_empty()
    {
        return Err("invalid value source schema or empty split".into());
    }
    let mut ids = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    let mut documents = BTreeSet::new();
    let fit_worlds: BTreeSet<_> = source.fit.iter().map(|case| case.world).collect();
    if source
        .development
        .iter()
        .any(|case| fit_worlds.contains(&case.world))
    {
        return Err("fit/development world overlap".into());
    }
    let mut pairs: BTreeMap<&str, Vec<&Case>> = BTreeMap::new();
    for case in source.fit.iter().chain(&source.development) {
        if case.prompt.is_empty()
            || case.response.is_empty()
            || !["prose", "rust"].contains(&case.family.as_str())
            || !ids.insert(&case.id)
            || !prompts.insert(&case.prompt)
            || !documents.insert(format!("{}{}", case.prompt, case.response))
        {
            return Err("empty, duplicated or unsupported source case".into());
        }
        pairs.entry(&case.pair_id).or_default().push(case);
    }
    for pair in pairs.values() {
        if pair.len() != 2 || pair[0].variant == pair[1].variant {
            return Err("each counterfactual pair must have two distinct variants".into());
        }
    }
    Ok(())
}

/// Compare decimal spellings, not token IDs or parsed arithmetic answers. No
/// leading whitespace is discarded. Reject an incomplete sign or an identifier
/// or fractional continuation that happens to begin with the expected digits.
fn leading_numeral(bytes: &[u8]) -> Option<&[u8]> {
    let start = usize::from(bytes.first() == Some(&b'-'));
    let mut end = start;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if end == start
        || bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        || (bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(u8::is_ascii_digit))
    {
        return None;
    }
    Some(&bytes[..end])
}

#[derive(Default, Serialize)]
struct Metrics {
    cases: usize,
    exact_responses: usize,
    numeric_cases: usize,
    correct_leading_numerals: usize,
    nonnumeric_cases: usize,
    nonnumeric_cases_with_leading_numerals: usize,
    generated_tokens: usize,
}

fn write_new(path: &Path, bytes: &[u8]) -> ProbeResult<()> {
    OUTPUT_BYTES_REMAINING
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
            remaining.checked_sub(bytes.len())
        })
        .map_err(|_| {
            format!(
                "output allowance exhausted before writing {}; retained partial files remain",
                path.display()
            )
        })?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn write_json(path: &Path, value: &impl Serialize) -> ProbeResult<()> {
    write_new(path, &serde_json::to_vec_pretty(value)?)
}

fn within_limit(start: Instant, options: &Options) -> ProbeResult<()> {
    if start.elapsed().as_secs_f64() >= options.max_seconds as f64 {
        Err("monitored model-work elapsed limit reached; retained partial files remain".into())
    } else {
        Ok(())
    }
}

fn reject_training_overlap(model: &Model, cases: &[Case]) -> ProbeResult<()> {
    for case in cases {
        let pair = serde_json::to_vec(&(&case.prompt, &case.response))?;
        let pair_cid = format!("blake3:{}", blake3::hash(&pair).to_hex());
        let whole_cid = format!(
            "blake3:{}",
            blake3::hash(format!("{}{}", case.prompt, case.response).as_bytes()).to_hex()
        );
        if model
            .construction()
            .iter()
            .chain(model.readout_training())
            .chain(model.memory_read_training())
            .chain(model.value_training())
            .chain(model.value_completion_training())
            .chain(model.response_entry_training())
            .chain(model.word_copy_training())
            .any(|known| {
                known.id == case.id || known.text_cid == pair_cid || known.text_cid == whole_cid
            })
        {
            return Err(format!(
                "development case overlaps actual artifact training: {}",
                case.id
            )
            .into());
        }
    }
    Ok(())
}

fn evaluate(
    model: &Model,
    source: &Source,
    options: &Options,
    start: Instant,
) -> ProbeResult<Value> {
    reject_training_overlap(model, &source.development)?;
    let mut arms = Vec::new();
    let completion = model.value_completion_version().is_some();
    let entry = model.response_entry_version().is_some();
    let copy = !model.word_copy_training().is_empty();
    let mut controls = if options.controls == "full" {
        vec![Control::Full]
    } else if copy {
        vec![
            Control::Full,
            Control::WordCopyDisabled,
            Control::WordCopyGeometryDisabled,
            Control::WordCopyDispatchDisabled,
        ]
    } else if entry {
        vec![
            Control::Full,
            Control::ResponseEntryDisabled,
            Control::ResponseEntryGeometryDisabled,
        ]
    } else if completion {
        vec![
            Control::Full,
            Control::ValueCompletionDisabled,
            Control::ValueCompletionGeometryDisabled,
            Control::ValuesDisabled,
        ]
    } else {
        vec![
            Control::Full,
            Control::ValuesDisabled,
            Control::H4Disabled,
            Control::ZetaDisabled,
        ]
    };
    if options.lexeme_cues && options.controls == "all" && !completion {
        // Suppress kind-16 features in this fitted artifact while retaining its
        // state. This measures feature sensitivity, not a separately fitted
        // matched baseline.
        controls.push(Control::ValueLexemesDisabled);
    }
    for control in controls {
        let mut metrics: BTreeMap<String, Metrics> = BTreeMap::new();
        let mut rows = Vec::new();
        let mut pairs: BTreeMap<&str, Vec<(bool, Option<Vec<u8>>)>> = BTreeMap::new();
        for (index, case) in source.development.iter().enumerate() {
            within_limit(start, options)?;
            let generation_start = Instant::now();
            let generation = model.generate(&case.prompt, options.generated_tokens, control)?;
            let generation_elapsed_us = generation_start.elapsed().as_micros();
            let expected = leading_numeral(case.response.as_bytes());
            let actual = leading_numeral(&generation.bytes);
            let numeral_correct = expected.map(|value| actual == Some(value));
            let exact = generation.bytes == case.response.as_bytes();
            for key in [
                case.family.clone(),
                format!("{}/{}", case.family, case.task),
            ] {
                let row = metrics.entry(key).or_default();
                row.cases += 1;
                row.exact_responses += usize::from(exact);
                row.generated_tokens += generation.token_ids.len();
                if let Some(correct) = numeral_correct {
                    row.numeric_cases += 1;
                    row.correct_leading_numerals += usize::from(correct);
                } else {
                    row.nonnumeric_cases += 1;
                    row.nonnumeric_cases_with_leading_numerals += usize::from(actual.is_some());
                }
            }
            if let Some(correct) = numeral_correct {
                pairs
                    .entry(&case.pair_id)
                    .or_default()
                    .push((correct, actual.map(<[u8]>::to_vec)));
            }
            let generated_source = if control == Control::Full && case.family == "rust" {
                let path = options.output_dir.join(format!("rust-case-{index}.rs"));
                let mut bytes = case.prompt.as_bytes().to_vec();
                bytes.extend_from_slice(&generation.bytes);
                write_new(&path, &bytes)?;
                Some(
                    json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string(),"bytes":bytes.len(),"compilation":"NOT_RUN","execution":"NOT_RUN"}),
                )
            } else {
                None
            };
            rows.push(json!({
                "case":case,
                "expected_leading_numeral":expected.map(String::from_utf8_lossy),
                "actual_leading_numeral":actual.map(String::from_utf8_lossy),
                "leading_numeral_correct":numeral_correct,
                "exact_response":exact,
                "generated_source":generated_source,
                "generation":generation,
                "generation_elapsed_us":generation_elapsed_us,
            }));
        }
        let pair_rows: Vec<_> = pairs.into_iter().map(|(id, values)| json!({
            "pair_id":id,
            "cases":values.len(),
            "both_leading_numerals_correct":values.len()==2 && values.iter().all(|value| value.0),
            "generated_leading_numeral_changed":values.len()==2 && values[0].1.is_some() && values[1].1.is_some() && values[0].1!=values[1].1,
        })).collect();
        let mut arm = json!({"control":control,"metrics":metrics,"numeric_counterfactual_pairs":pair_rows,"cases":rows});
        if control == Control::ValueLexemesDisabled {
            arm["scope"] = json!("Within-artifact lexical-feature sensitivity: kind-16 query-position/source-word match features disabled while retaining the same fitted artifact and state. This is not a separately fitted matched baseline.");
        }
        if control == Control::ValueCompletionDisabled {
            arm["scope"] = json!("Within-artifact completion-feature suppression. Typed operand selection and numeral emission remain enabled. This is not a separately fitted matched baseline.");
        }
        if control == Control::ValueCompletionGeometryDisabled {
            arm["scope"] = json!("Within-artifact suppression of completion H4/orientation/phase feature contributions. The underlying typed-value and ordinary model geometry remain enabled. This measures feature sensitivity, not separately fitted matched training or general geometric advantage.");
        }
        if control == Control::ResponseEntryDisabled {
            arm["scope"] = json!("Within-artifact suppression of response-entry offers while retaining ordinary, typed and numeral-completion components. State remains bounded; this is not a separately fitted baseline.");
        }
        if control == Control::ResponseEntryGeometryDisabled {
            arm["scope"] = json!("Within-artifact suppression of only response-entry H4/orientation/phase score features. Other components remain enabled. Candidate support and complete work must be checked separately; this is not a refitted geometry comparator.");
        }
        if control == Control::WordCopyDisabled {
            arm["scope"] = json!("Suppress only the retained-word copy extension. The inherited lexical entry, numeric path and ordinary model remain enabled; this is a same-artifact intervention.");
        }
        if control == Control::WordCopyGeometryDisabled {
            arm["scope"] = json!("Suppress H4/orientation/phase score features in both the copy selector and its learned suffix. Candidate support, lexical copy features and inherited entry remain enabled; this measures combined feature sensitivity, not selector-only attribution or a separately fitted geometry advantage.");
        }
        // Completion reports retain the complete generation objects once in
        // report.json to keep the model, exact outputs and traces within the
        // configured output allowance. Historical typed-only files stay intact.
        if !completion && !entry {
            write_json(
                &options.output_dir.join(format!("arm-{}.json", arms.len())),
                &arm,
            )?;
        }
        arms.push(arm);
    }
    Ok(json!(arms))
}

fn evaluate_binding(
    model: &Model,
    source: &BindingSource,
    options: &Options,
    start: Instant,
) -> ProbeResult<Value> {
    // Check every original and swapped raw pair against the loaded artifact's
    // actual construction, readout, memory and value receipts before any
    // control generation. The supplied source.fit is not artifact authority.
    for pair in &source.pairs {
        reject_training_overlap(model, std::slice::from_ref(&pair.original))?;
        reject_training_overlap(model, std::slice::from_ref(&pair.swapped))?;
    }
    let mut pairs = Vec::new();
    for (pair_index, pair) in source.pairs.iter().enumerate() {
        let mut rows = Vec::new();
        let mut correct = 0;
        let mut exact = 0;
        let mut emitted_numerals = Vec::new();
        for (label, case) in [
            ("original", &pair.original),
            ("source_names_swapped", &pair.swapped),
        ] {
            within_limit(start, options)?;
            let generation =
                model.generate(&case.prompt, options.generated_tokens, Control::Full)?;
            let expected =
                leading_numeral(case.response.as_bytes()).ok_or("binding target lacks numeral")?;
            let actual = leading_numeral(&generation.bytes);
            let numeral_correct = actual == Some(expected);
            let response_exact = generation.bytes == case.response.as_bytes();
            correct += usize::from(numeral_correct);
            exact += usize::from(response_exact);
            emitted_numerals.push(actual.map(<[u8]>::to_vec));
            let generated_source = if case.family == "rust" {
                let path = options
                    .output_dir
                    .join(format!("binding-{pair_index}-{label}.rs"));
                let mut bytes = case.prompt.as_bytes().to_vec();
                bytes.extend_from_slice(&generation.bytes);
                write_new(&path, &bytes)?;
                Some(
                    json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string(),"bytes":bytes.len(),"compilation":"NOT_RUN","execution":"NOT_RUN"}),
                )
            } else {
                None
            };
            rows.push(json!({"intervention":label,"case":case,
                "expected_leading_numeral":String::from_utf8_lossy(expected),
                "actual_leading_numeral":actual.map(String::from_utf8_lossy),
                "leading_numeral_correct":numeral_correct,"exact_response":response_exact,
                "generated_source":generated_source,"generation":generation}));
        }
        pairs.push(json!({"pair_id":pair.pair_id,"control":"full",
            "numeric_byte_runs_equal":raw_digit_runs(pair.original.prompt.as_bytes())==raw_digit_runs(pair.swapped.prompt.as_bytes()),
            "query_bytes_equal":pair.original.prompt.ends_with(&pair.query)&&pair.swapped.prompt.ends_with(&pair.query),
            "both_leading_numerals_correct":correct==2,"both_responses_exact":exact==2,
            "generated_leading_numeral_changed":emitted_numerals.iter().all(Option::is_some)&&emitted_numerals[0]!=emitted_numerals[1],
            "cases":rows}));
    }
    let report = json!({"scope":source.scope,"primary_metrics_include_these_runs":false,"fit_includes_these_runs":false,"runs":8,"pairs":pairs});
    write_json(
        &options.output_dir.join("binding-controls-report.json"),
        &report,
    )?;
    Ok(report)
}

fn main() -> ProbeResult<()> {
    if std::env::args().nth(1).as_deref() == Some("prepare-wording") {
        return wording::prepare();
    }
    let options = Options::parse()?;
    let output_limit = if [
        "completion",
        "entry",
        "copy",
        "copy-completed",
        "copy-composed",
        "copy-binding",
        "copy-binding-plain",
    ]
    .contains(&options.mode.as_str())
    {
        Some(COMPLETION_OUTPUT_BYTES)
    } else if options.controls == "full" {
        Some(PRESERVATION_OUTPUT_BYTES)
    } else {
        None
    };
    OUTPUT_BYTES_REMAINING.store(output_limit.unwrap_or(usize::MAX), Ordering::Relaxed);
    let start = Instant::now();
    let source = source(&options)?;
    if let Some(parent) = options
        .output_dir
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&options.output_dir)?;
    let source_bytes = serde_json::to_vec_pretty(&source)?;
    write_new(&options.output_dir.join("source.json"), &source_bytes)?;
    write_json(&options.output_dir.join("options.json"), &options)?;
    let mut binding_source = binding_source()?;
    if options.lexeme_cues {
        binding_source.scope.push_str(" These exact four pairs informed the /2 complete-lexeme-cue and fit-name-swap revision. They are reused OPEN development feedback; they remain absent from fit and do not constitute final held-out evaluation.");
    }
    if binding_source
        .pairs
        .iter()
        .flat_map(|pair| [&pair.original, &pair.swapped])
        .any(|case| source.fit.iter().any(|fit| fit.prompt == case.prompt))
    {
        return Err("binding control prompt overlaps the supplied fit source".into());
    }
    let binding_bytes = serde_json::to_vec_pretty(&binding_source)?;
    write_new(
        &options.output_dir.join("binding-controls-source.json"),
        &binding_bytes,
    )?;
    if ["prepare", "prepare-copy", "prepare-facts"].contains(&options.mode.as_str()) {
        println!(
            "{}",
            json!({"status":"prepared","fit_cases":source.fit.len(),"development_cases":source.development.len(),"source_blake3":blake3::hash(&source_bytes).to_hex().to_string(),"elapsed_ms":start.elapsed().as_millis()})
        );
        return Ok(());
    }
    within_limit(start, &options)?;
    let baseline = Model::from_bytes(&fs::read(&options.model)?)?;
    let input_artifact = baseline.artifact_cid().to_owned();
    let (model, fit_report) = if [
        "fit",
        "completion",
        "entry",
        "copy",
        "copy-completed",
        "copy-composed",
        "copy-binding",
        "copy-binding-plain",
    ]
    .contains(&options.mode.as_str())
    {
        let examples: Vec<_> = source.fit.iter().map(Case::example).collect();
        let (fitted, report) = if [
            "copy",
            "copy-completed",
            "copy-composed",
            "copy-binding",
            "copy-binding-plain",
        ]
        .contains(&options.mode.as_str())
        {
            let config = ResponseEntryFitConfig {
                epochs: options.epochs,
                learning_rate: options.learning_rate,
                max_positions: options.completion_max_positions,
            };
            let (fitted, report) =
                if matches!(options.mode.as_str(), "copy-binding" | "copy-binding-plain") {
                    baseline.fit_response_entry_copy_binding(
                        &examples,
                        config,
                        options.mode == "copy-binding-plain",
                    )?
                } else if options.mode == "copy-composed" {
                    baseline.fit_response_entry_copy_composed(&examples, config)?
                } else if options.mode == "copy-completed" {
                    baseline.fit_response_entry_copy_completed_word(&examples, config)?
                } else {
                    baseline.fit_response_entry_copy(&examples, config)?
                };
            write_json(&options.output_dir.join("fit-report.json"), &report)?;
            (fitted, serde_json::to_value(report)?)
        } else if options.mode == "entry" {
            let (fitted, report) = baseline.fit_response_entry(
                &examples,
                ResponseEntryFitConfig {
                    epochs: options.epochs,
                    learning_rate: options.learning_rate,
                    max_positions: options.completion_max_positions,
                },
            )?;
            write_json(&options.output_dir.join("fit-report.json"), &report)?;
            (fitted, serde_json::to_value(report)?)
        } else if options.mode == "completion" {
            let (fitted, report) = baseline.fit_value_completion(
                &examples,
                ValueCompletionFitConfig {
                    epochs: options.epochs,
                    learning_rate: options.learning_rate,
                    max_positions: options.completion_max_positions,
                },
            )?;
            write_json(&options.output_dir.join("fit-report.json"), &report)?;
            (fitted, serde_json::to_value(report)?)
        } else {
            let config = ValueFitConfig {
                epochs: options.epochs,
                learning_rate: options.learning_rate,
                max_features: options.max_features,
            };
            let (fitted, report) = if options.lexeme_cues {
                baseline.fit_values_with_lexeme_cues(&examples, config)?
            } else {
                baseline.fit_values(&examples, config)?
            };
            write_json(&options.output_dir.join("fit-report.json"), &report)?;
            (fitted, serde_json::to_value(report)?)
        };
        write_new(&options.output_dir.join("model.json"), &fitted.to_bytes()?)?;
        // Evaluate the serialized/reloaded artifact, not only the trainer's
        // in-memory object. Parent monitoring charges this loading/serialization.
        let reloaded = Model::from_bytes(&fs::read(options.output_dir.join("model.json"))?)?;
        (reloaded, Some(report))
    } else {
        (baseline, None)
    };
    within_limit(start, &options)?;
    let arms = evaluate(&model, &source, &options, start)?;
    let binding_controls = evaluate_binding(&model, &binding_source, &options, start)?;
    let mut report = json!({
        "schema":if !model.word_copy_training().is_empty() { "uor-r4.native-response-entry-copy-probe/1" } else if model.response_entry_version().is_some() { "uor-r4.native-response-entry-probe/1" } else if model.value_completion_version().is_some() { "uor-r4.native-value-completion-probe/1" } else if options.lexeme_cues { "uor-r4.native-typed-value-probe/2" } else { "uor-r4.native-typed-value-probe/1" },
        "status":"completed",
        "scope":source.scope,
        "tokenization_law":source.tokenization_law,
        "input_artifact_cid":input_artifact,
        "artifact_cid":model.artifact_cid(),
        "source_blake3":blake3::hash(&source_bytes).to_hex().to_string(),
        "source_schema":source.schema,
        "lexeme_cues":options.lexeme_cues,
        "development_reused_after_design_feedback":options.lexeme_cues,
        "fit_cases":source.fit.len(),
        "development_cases":source.development.len(),
        "fit_report":fit_report,
        "arms":arms,
        "binding_controls_source_blake3":blake3::hash(&binding_bytes).to_hex().to_string(),
        "binding_controls":binding_controls,
        "resources":{"elapsed_ms":start.elapsed().as_millis(),"max_seconds":options.max_seconds,"threads":1,"rss":"NOT_MEASURED: use parent process monitor","time_boundary":"Checked between bounded operations, not a hard interruption of fitting/generation/serialization. All model work belongs to the inherited cumulative ledger."},
        "compilation":"NOT_RUN: exact generated Rust saved for separate inspected-source assessment",
        "execution":"NOT_RUN",
    });
    if let Some(version) = model.response_entry_version() {
        report["response_entry_operator_version"] = json!(version);
        report["response_entry_scope"] = json!("Same raw /2 source and reused OPEN development. Nonnumeric response entry learns canonical token IDs and EOS after actual model-selected NoWrite. Continuation follows only a quantized selected Enter that was observed; no target creates a hidden anchor. Whole responses over 32 canonical-token/EOS positions are skipped, not truncated. Numeric typed/completion behavior remains the upstream path. Same-artifact entry and entry-geometry controls retain all other components.");
    }
    if let Some(version) = model.value_completion_version() {
        report["completion_operator_version"] = json!(version);
        report["completion_scope"] = json!("Same /2 raw source and unchanged OPEN development targets. Fitting follows actual upstream numeral rollouts, then supervises ordinary suffix bytes and EOS; no generated suffix or target template is appended. Completion traces describe the learned step transitions. Primary and binding controls remain reused design feedback, not final held-out evaluation.");
    }
    if !model.word_copy_training().is_empty() {
        report["response_entry_scope"] = json!("Source /3 OPEN development. The /2 copy extension competes against inherited lexical entry and ordinary Base on actual model-selected NoWrite. It selects a retained occurrence through learned bounded scalar features, emits its exact bytes after matching observation, and learns a suffix only after a quantized selected complete copy. Parent entry and all upstream parameters remain fixed. Equal-spelling matches are latent alternatives; target byte equality does not establish occurrence-role binding. Numeric preservation, copy selection, suffix completion and compiled behavior are reported separately.");
        report["resources"]["rust_layout"] = json!({
            "session_struct_bytes": std::mem::size_of::<uor_r4_core::native_geometric::Session>(),
            "work_struct_bytes": std::mem::size_of::<uor_r4_core::native_geometric::Work>(),
            "word_copy_work_struct_bytes": std::mem::size_of::<uor_r4_core::native_geometric::WordCopyWork>(),
            "word_copy_decision_struct_bytes": std::mem::size_of::<uor_r4_core::native_geometric::WordCopyDecision>(),
            "scope": "Current compiled Rust layouts, including optional and transient slots; excludes heap buffers and immutable model tables. Generation state separately reports the copy state layout. No predecessor total Session layout measurement or exact stack high-water measurement is available."
        });
    }
    if options.controls != "all" {
        report["controls_selection"] = json!(options.controls);
    }
    if options.mode == "copy-composed" {
        report["response_entry_scope"] = json!("Composed /2 extension: first response word after at most one learned lexical prefix token; exact source/query equality plus relative H4/phase features select a retained occurrence. No numeric-source requirement. NoCopy lexical continuation learns after an actually selected first transition. Forced interior copy bytes dispatch before ordinary scoring with score1 marker; observation/memory updates remain.64new construction and16fresh cases augment the unchanged source; no template or target buffer enters inference.");
    }
    if matches!(options.mode.as_str(), "copy-binding" | "copy-binding-plain") {
        report["response_entry_scope"] = json!({
            "operator": "Composed copy with the existing candidate binding-mask/preceding-word address supplied to lexical entry as one sparse feature per retained occurrence. At most32entry features,16lexical candidates and16retained words. Source payloads and copy selection remain unchanged. No candidate rank or answer bytes enter the new entry feature; repeated features retain multiplicity.",
            "copy_geometry_disabled_during_fit_and_serving": options.mode == "copy-binding-plain",
            "control_scope": "Matched parent, source, dose, caps and nongeometric features. This flag removes H4/orientation/zeta features from the copy extension only; inherited ordinary/memory/typed paths remain. Reserved cases are evaluated separately after design selection."
        });
    }
    if let Some(limit) = output_limit {
        report["resources"]["output_bytes_limit"] = json!(limit);
        report["resources"]["output_bytes_before_report"] =
            json!(limit - OUTPUT_BYTES_REMAINING.load(Ordering::Relaxed));
    }
    write_json(&options.output_dir.join("report.json"), &report)?;
    println!(
        "{}",
        json!({"status":"completed","artifact_cid":model.artifact_cid(),"output_dir":options.output_dir,"elapsed_ms":start.elapsed().as_millis(),"metrics":report["arms"].as_array().map(|arms|arms.iter().map(|arm|json!({"control":arm["control"],"metrics":arm["metrics"]})).collect::<Vec<_>>())})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_copy_source_preserves_prior_cases_and_separates_new_names() {
        let mut source = Source {
            schema: LEXEME_SOURCE_SCHEMA.into(),
            scope: String::new(),
            tokenization_law: String::new(),
            fit: make_cases("fit", 16),
            development: make_cases("development", 4),
        };
        source.fit.extend(make_fit_name_swaps(16).unwrap());
        let original_fit = serde_json::to_vec(&source.fit).unwrap();
        let original_development = serde_json::to_vec(&source.development).unwrap();
        append_word_copy_cases(&mut source).unwrap();
        validate_source(&source).unwrap();
        assert_eq!(source.fit.len(), 224);
        assert_eq!(source.development.len(), 46);
        assert_eq!(
            original_fit,
            serde_json::to_vec(&source.fit[..192]).unwrap()
        );
        assert_eq!(
            original_development,
            serde_json::to_vec(&source.development[..32]).unwrap()
        );
        let fit_names: BTreeSet<_> = source.fit[192..]
            .iter()
            .map(|case| case.response.lines().next().unwrap())
            .collect();
        for case in &source.development[32..44] {
            assert!(!fit_names.contains(case.response.lines().next().unwrap()));
        }
        assert!(append_word_copy_cases(&mut source).is_err());
    }

    #[test]
    fn generated_splits_and_counterfactuals_are_distinct() {
        let source = Source {
            schema: SOURCE_SCHEMA.into(),
            scope: String::new(),
            tokenization_law: String::new(),
            fit: make_cases("fit", 16),
            development: make_cases("development", 4),
        };
        validate_source(&source).unwrap();
        assert_eq!(source.fit.len(), 128);
        assert_eq!(source.development.len(), 32);
        for pair in source.development.chunks(16) {
            for index in 0..8 {
                assert_eq!(pair[index].pair_id, pair[index + 8].pair_id);
                assert_ne!(pair[index].prompt, pair[index + 8].prompt);
                if pair[index].task != "no_write" {
                    assert_ne!(pair[index].response, pair[index + 8].response);
                }
            }
        }
    }

    #[test]
    fn leading_numeral_does_not_credit_longer_or_implicit_spellings() {
        assert_eq!(leading_numeral(b"17);\n}"), Some(&b"17"[..]));
        assert_eq!(leading_numeral(b"-17.\n"), Some(&b"-17"[..]));
        assert_ne!(leading_numeral(b"170);"), Some(&b"17"[..]));
        for invalid in [
            b" 17".as_slice(),
            b"-",
            b"17name",
            b"17_",
            b"17.2",
            b"Unknown",
        ] {
            assert_eq!(leading_numeral(invalid), None);
        }
    }

    #[test]
    fn lexeme_source_adds_fit_name_swaps_without_changing_primary_cases() {
        let original_fit = make_cases("fit", 16);
        let development = make_cases("development", 4);
        let swaps = make_fit_name_swaps(16).unwrap();
        assert_eq!(swaps.len(), 64);
        for swapped in &swaps {
            let original_id = swapped.id.strip_suffix("/source_names_swapped").unwrap();
            let original = original_fit
                .iter()
                .find(|case| case.id == original_id)
                .unwrap();
            assert_eq!(
                raw_digit_runs(original.prompt.as_bytes()),
                raw_digit_runs(swapped.prompt.as_bytes())
            );
            let query_start = original
                .prompt
                .rfind(if original.family == "prose" {
                    "User: "
                } else {
                    "assert_eq!"
                })
                .unwrap();
            assert!(swapped.prompt.ends_with(&original.prompt[query_start..]));
            assert_ne!(original.response, swapped.response);
            assert!(["copy_current", "add"].contains(&swapped.task.as_str()));
            assert!(!swapped.prompt.contains("suri"));
            assert!(!swapped.prompt.contains("orin"));
            assert!(!swapped.prompt.contains("tavi"));
        }
        let mut fit = original_fit.clone();
        fit.extend(swaps);
        let source = Source {
            schema: LEXEME_SOURCE_SCHEMA.into(),
            scope: String::new(),
            tokenization_law: String::new(),
            fit,
            development,
        };
        validate_source(&source).unwrap();
        assert_eq!(source.fit.len(), 192);
        assert_eq!(source.development.len(), 32);
        assert_eq!(
            serde_json::to_vec(&source.fit[..128]).unwrap(),
            serde_json::to_vec(&original_fit).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&source.development).unwrap(),
            serde_json::to_vec(&make_cases("development", 4)).unwrap()
        );
    }

    #[test]
    fn binding_controls_preserve_numeric_order_and_query_but_change_source_roles() {
        let source = binding_source().unwrap();
        let primary = make_cases("development", 2);
        assert_eq!(source.pairs.len(), 4);
        for pair in &source.pairs {
            assert_eq!(
                raw_digit_runs(pair.original.prompt.as_bytes()),
                raw_digit_runs(pair.swapped.prompt.as_bytes())
            );
            assert!(pair.original.prompt.ends_with(&pair.query));
            assert!(pair.swapped.prompt.ends_with(&pair.query));
            assert_ne!(pair.original.response, pair.swapped.response);
            let original = primary
                .iter()
                .find(|case| {
                    case.world == 100_000
                        && case.family == pair.original.family
                        && case.task == pair.original.task
                })
                .unwrap();
            assert_eq!(pair.original.prompt, original.prompt);
            assert_eq!(pair.original.response, original.response);
        }
    }
}

/// Authored before fitting; labels remain probe-only. All prompts fit the
/// sixteen retained-word bound, including both entities in update cases.
fn append_fact_cases(source: &mut Source) -> ProbeResult<()> {
    if source.schema != WORD_COPY_SOURCE_SCHEMA {
        return Err("prepare-facts requires /3".into());
    }
    let names = ["ada", "bea", "cyra", "dara", "elin", "faye", "gita", "hana"];
    let places = [
        "Rome", "Paris", "Dover", "York", "Bath", "Perth", "Cairo", "Tokyo",
    ];
    for world in 0..8 {
        for task in 0..4 {
            for variant in 0..2 {
                source.fit.push(fact_case(
                    "fit",
                    world,
                    task,
                    variant,
                    names[world % 8],
                    names[(world + 1) % 8],
                    places[(world + variant) % 8],
                    places[(world + variant + 2) % 8],
                ));
            }
        }
    }
    for task in 0..4 {
        for world in 0..2 {
            for variant in 0..2 {
                let names = [["mira", "theo"], ["nora", "kian"]][world];
                let cities = [["Oslo", "Lima"], ["Bern", "Pune"]][world];
                source.development.push(fact_case(
                    "fresh",
                    world,
                    task,
                    variant,
                    names[0],
                    names[1],
                    cities[variant],
                    cities[1 - variant],
                ));
            }
        }
    }
    source.scope.push_str(" Fact composition:64additional construction cases,16fresh cases fixed before fit (four each simple,distractor,update,unsupported), no numeric decoys. Fresh names and values are absent from added construction. Existing46development remain open. All sixteen fresh results must be reported, without tuning on them.");
    validate_source(source)
}
fn fact_case(
    split: &str,
    world: usize,
    task: usize,
    variant: usize,
    a: &str,
    b: &str,
    x: &str,
    y: &str,
) -> Case {
    let style = world % 2;
    let (verb, query) = if style == 0 {
        ("lives", format!("Where is {a}?"))
    } else {
        ("stays", format!("Where does {a} stay?"))
    };
    let (facts, answer) = match task {
        0 => (format!("{a} {verb} in {x}."), x),
        1 if variant == 0 => (format!("{a} {verb} in {x}. {b} {verb} in {y}."), x),
        1 => (format!("{b} {verb} in {y}. {a} {verb} in {x}."), x),
        2 => (format!("{a} in {x}. {b} in Bath. {a} now in {y}."), y),
        _ => (format!("{b} {verb} in {x}."), "Unknown"),
    };
    Case {
        id: format!("fact/{split}/{world}/{task}/{variant}"),
        family: "prose".into(),
        task: [
            "fact_simple",
            "fact_distractor",
            "fact_update",
            "fact_unsupported",
        ][task]
            .into(),
        world: if split == "fit" {
            400000 + world
        } else {
            500000 + world
        },
        pair_id: format!("fact/{split}/{world}/{task}"),
        variant,
        prompt: format!("{facts} {query} Answer:"),
        response: format!(" {answer}.\n"),
    }
}
