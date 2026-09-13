//! Frozen construction-family data for a matched context-support intervention.
//! Only offline training/probe data; these substitutions are never serving logic.
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{collections::BTreeSet, fs, io::Write, path::Path};

pub(super) type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ORIGINALS: [&str; 12] = [
    "The small boat crossed the lake. The water was calm.\n",
    "A red bird rested on a branch. Then the bird flew away.\n",
    "The garden was dry. Rain made the soil wet.\n",
    "Ada lives in Lima. Ada now lives in Oslo. Ada lives in Oslo.\n",
    "Ben owns a blue cup. Cora owns a red book. Ben owns the cup.\n",
    "User: Hello.\nAssistant: Hello. How are you?\nUser: Well.\n",
    "User: Where is the book?\nAssistant: The book is on the table.\n",
    "fn add(a: i32, b: i32) -> i32 { a + b }\n",
    "fn first() -> i32 { 1 }\nfn second() -> i32 { 2 }\n",
    "let x = 2;\nlet y = x + 3;\nassert_eq!(y, 5);\n",
    "The cup is blue. The book is red. The small book is on the table.\n",
    "User: Tell me about the lake.\nAssistant: The lake is calm and blue.\n",
];

// Versions 1..3 train; version 4 recombines construction-family elements and is
// frozen as an assessment-only conditional probe before either fit. No opened
// development document is inspected by this preparation function.
fn rule(index: usize, version: usize) -> RunResult<Vec<(&'static str, &'static str)>> {
    let pairs: &[(&str, &str)] = match (index, version) {
        (0, 1) => &[
            ("small", "green"),
            ("boat", "ship"),
            ("lake", "pond"),
            ("water", "river"),
            ("calm", "warm"),
        ],
        (0, 2) => &[
            ("small", "great"),
            ("boat", "raft"),
            ("lake", "pool"),
            ("water", "ocean"),
            ("calm", "cool"),
        ],
        (0, 3) => &[
            ("small", "white"),
            ("boat", "buoy"),
            ("lake", "cove"),
            ("water", "coast"),
            ("calm", "dark"),
        ],
        (0, 4) => &[
            ("small", "green"),
            ("boat", "raft"),
            ("lake", "cove"),
            ("water", "river"),
            ("calm", "cool"),
        ],
        (1, 1) => &[("red", "tan"), ("bird", "moth"), ("branch", "flower")],
        (1, 2) => &[("red", "shy"), ("bird", "dove"), ("branch", "window")],
        (1, 3) => &[("red", "big"), ("bird", "wren"), ("branch", "rafter")],
        (1, 4) => &[("bird", "dove"), ("branch", "flower")],
        (2, 1) => &[("garden", "forest"), ("Rain", "Mist"), ("soil", "path")],
        (2, 2) => &[
            ("garden", "meadow"),
            ("Rain", "Snow"),
            ("soil", "yard"),
            ("wet", "icy"),
        ],
        (2, 3) => &[("garden", "jungle"), ("Rain", "Hail"), ("soil", "road")],
        (2, 4) => &[
            ("garden", "forest"),
            ("Rain", "Snow"),
            ("soil", "yard"),
            ("wet", "icy"),
        ],
        (3, 1) => &[("Ada", "Eve"), ("Lima", "Rome"), ("Oslo", "Bath")],
        (3, 2) => &[("Ada", "Max"), ("Lima", "Bonn"), ("Oslo", "Cork")],
        (3, 3) => &[("Ada", "Zoe"), ("Lima", "Leon"), ("Oslo", "Lyon")],
        (3, 4) => &[("Ada", "Eve"), ("Lima", "Bonn"), ("Oslo", "Lyon")],
        (4, 1) => &[
            ("Ben", "Tom"),
            ("blue", "gold"),
            ("cup", "box"),
            ("Cora", "Nora"),
            ("red", "tan"),
            ("book", "bowl"),
        ],
        (4, 2) => &[
            ("Ben", "Sam"),
            ("blue", "pink"),
            ("cup", "hat"),
            ("Cora", "Lena"),
            ("book", "lamp"),
        ],
        (4, 3) => &[
            ("Ben", "Leo"),
            ("blue", "gray"),
            ("cup", "bag"),
            ("Cora", "Mina"),
            ("red", "big"),
            ("book", "vase"),
        ],
        (4, 4) => &[
            ("Ben", "Tom"),
            ("blue", "pink"),
            ("cup", "bag"),
            ("Cora", "Nora"),
            ("red", "tan"),
            ("book", "lamp"),
        ],
        (5, 1) => &[("Hello", "Howdy"), ("Well", "Fine")],
        (5, 2) => &[("Well", "Good")],
        (5, 3) => &[("Hello", "Howdy"), ("Well", "Okay")],
        (5, 4) => &[("Well", "Fine")],
        (6, 1) => &[("book", "bowl"), ("table", "chair")],
        (6, 2) => &[("book", "lamp"), ("table", "shelf")],
        (6, 3) => &[("book", "vase"), ("table", "floor")],
        (6, 4) => &[("book", "bowl"), ("table", "shelf")],
        (7, 1) => &[("add", "sum"), ("a", "x"), ("b", "y")],
        (7, 2) => &[("add", "mix"), ("a", "p"), ("b", "q")],
        (7, 3) => &[("add", "duo"), ("a", "u"), ("b", "v")],
        (7, 4) => &[("add", "sum"), ("a", "p"), ("b", "q")],
        (8, 1) => &[
            ("first", "start"),
            ("second", "finish"),
            ("1", "3"),
            ("2", "4"),
        ],
        (8, 2) => &[
            ("first", "third"),
            ("second", "fourth"),
            ("1", "5"),
            ("2", "6"),
        ],
        (8, 3) => &[
            ("first", "prior"),
            ("second", "latter"),
            ("1", "7"),
            ("2", "8"),
        ],
        (8, 4) => &[("1", "3"), ("2", "8")],
        (9, 1) => &[("x", "a"), ("y", "b"), ("2", "1"), ("3", "4")],
        (9, 2) => &[("x", "p"), ("y", "q"), ("2", "3"), ("3", "2")],
        (9, 3) => &[("x", "m"), ("y", "n"), ("2", "4"), ("3", "4"), ("5", "8")],
        (9, 4) => &[("x", "a"), ("y", "b"), ("3", "4"), ("5", "6")],
        (10, 1) => &[
            ("cup", "box"),
            ("blue", "gold"),
            ("book", "bowl"),
            ("red", "tan"),
            ("small", "green"),
            ("table", "chair"),
        ],
        (10, 2) => &[
            ("cup", "hat"),
            ("blue", "pink"),
            ("book", "lamp"),
            ("small", "great"),
            ("table", "shelf"),
        ],
        (10, 3) => &[
            ("cup", "bag"),
            ("blue", "gray"),
            ("book", "vase"),
            ("red", "big"),
            ("small", "white"),
            ("table", "floor"),
        ],
        (10, 4) => &[
            ("cup", "box"),
            ("blue", "pink"),
            ("book", "lamp"),
            ("red", "tan"),
            ("small", "green"),
            ("table", "floor"),
        ],
        (11, 1) => &[("lake", "pond"), ("calm", "warm"), ("blue", "gold")],
        (11, 2) => &[("lake", "pool"), ("calm", "cool"), ("blue", "gray")],
        (11, 3) => &[("lake", "cove"), ("calm", "deep"), ("blue", "aqua")],
        (11, 4) => &[("lake", "pond"), ("calm", "cool")],
        _ => return Err("unknown context transformation".into()),
    };
    Ok(pairs.to_vec())
}

// Simultaneous substitutions of complete ASCII word tokens. In particular, the
// numeric token 2 never modifies i32, and 2->3 plus 3->2 cannot cascade.
fn substitute(text: &str, replacements: &[(&str, &str)]) -> RunResult<String> {
    if !text.is_ascii() || replacements.iter().any(|(a, b)| a.len() != b.len()) {
        return Err("ASCII length-preserving substitutions required".into());
    }
    let mut seen = vec![false; replacements.len()];
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_' {
            let begin = at;
            while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                at += 1;
            }
            let word = &text[begin..at];
            if let Some((index, (_, replacement))) = replacements
                .iter()
                .enumerate()
                .find(|(_, (from, _))| *from == word)
            {
                out.push_str(replacement);
                seen[index] = true;
            } else {
                out.push_str(word);
            }
        } else {
            out.push(char::from(bytes[at]));
            at += 1;
        }
    }
    if seen.iter().any(|hit| !hit) || out.len() != text.len() {
        return Err("unapplied rule or changed byte exposure".into());
    }
    Ok(out)
}

pub(super) fn prepare(original: &Value) -> RunResult<Value> {
    let supplied: Vec<String> = serde_json::from_value(original["training"].clone())?;
    if supplied != ORIGINALS || supplied.iter().map(|s| s.len() + 1).sum::<usize>() != 672 {
        return Err("original construction corpus differs from frozen source".into());
    }
    let (mut varied, mut repeated, mut probe, mut variants, mut probe_rules) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut unique = BTreeSet::new();
    for (index, text) in supplied.iter().enumerate() {
        for version in 0..4 {
            let changes = if version == 0 {
                Vec::new()
            } else {
                rule(index, version)?
            };
            let transformed = substitute(text, &changes)?;
            if !unique.insert(transformed.clone()) {
                return Err("duplicate varied context".into());
            }
            variants.push(json!({"original_index":index,"version":version,"rule":changes,"text":transformed,"byte_count":transformed.len(),"positions_with_eos":transformed.len()+1}));
            varied.push(transformed);
            repeated.push(text.clone());
        }
        let changes = rule(index, 4)?;
        let text = substitute(text, &changes)?;
        probe_rules.push(json!({"original_index":index,"version":4,"rule":changes,"text":text,"byte_count":text.len()}));
        probe.push(text);
    }
    let probe_unique: BTreeSet<_> = probe.iter().collect();
    if probe_unique.len() != 12
        || probe.iter().any(|s| unique.contains(s))
        || varied
            .iter()
            .zip(&repeated)
            .any(|(a, b)| a.len() != b.len())
    {
        return Err("probe overlap, duplication, or unmatched exposure".into());
    }
    Ok(json!({
        "schema":"uor-r4.matched-context-data/1", "varied":varied,"repeated":repeated,"probe":probe,
        "prompts":["The green raft ","Eve now lives in Lyon. Eve lives in ","User: Where is the bowl?\nAssistant:","fn sum(p: i32, q: i32) -> i32 { "],
        "variants":variants,"probe_rules":probe_rules,
        "constraints":{"original_documents":12,"documents_per_arm":48,"byte_eos_positions_per_arm":2688,"probe_documents":12,"probe_positions":672,"varied_unique_documents":48,"repeated_unique_documents":12,"order":"original index, then version 0..3; matched copy at each position","length_preserving":true,"probe_exact_text_disjoint_from_both_arms":true,"scope":"Authored construction-family conditional probes; not independent general-language or executable-generated-code qualification.","opened_development_used_to_author":false,"runtime_templates":false,"fit_selection_uses_probe":false,"source_rules":"simultaneous complete ASCII word substitution, fixed before fitting"}
    }))
}

#[test]
fn context_data_equal_exposure_unique_variants_and_rust_arithmetic() -> RunResult<()> {
    let original = json!({"training":ORIGINALS});
    let data = prepare(&original)?;
    assert_eq!(data, prepare(&original)?);
    for key in ["varied", "repeated"] {
        let texts: Vec<String> = serde_json::from_value(data[key].clone())?;
        assert_eq!(texts.len(), 48);
        assert_eq!(texts.iter().map(|s| s.len() + 1).sum::<usize>(), 2688);
    }
    // Check the changed numerical assertion, including the simultaneous 2/3
    // swap. This validates prepared authored programs, not generated behavior.
    for version in 0..=4 {
        let changes = if version == 0 {
            Vec::new()
        } else {
            rule(9, version)?
        };
        let program = substitute(ORIGINALS[9], &changes)?;
        let numbers: Vec<i32> = program
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;
        assert_eq!(numbers.len(), 3);
        assert_eq!(numbers[0] + numbers[1], numbers[2]);
        let add = if version == 0 {
            ORIGINALS[7].to_string()
        } else {
            substitute(ORIGINALS[7], &rule(7, version)?)?
        };
        assert_eq!(add.matches("i32").count(), 3);
        assert!(add.contains(" + "));
    }
    assert_eq!(
        substitute("2 + 3; i32", &[("2", "3"), ("3", "2")])?,
        "3 + 2; i32"
    );
    let mut changed = original;
    changed["training"][0] = json!("different");
    assert!(prepare(&changed).is_err());
    Ok(())
}

fn write(out: &Path, name: &str, value: &Value) -> RunResult<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

#[test]
#[ignore = "parent runs once with exclusive saved data output and metered preparation"]
fn context_data_prepare_saved() -> RunResult<()> {
    let corpus = fs::canonicalize(std::env::var("UOR_CONTEXT_CORPUS")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_CONTEXT_DATA_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    if out.starts_with(&corpus) {
        return Err("output beneath sealed corpus".into());
    }
    crate::report_output::claim(&out)?;
    let result: RunResult<()> = (|| {
        crate::report_output::verify(&corpus)?;
        let bytes = fs::read(corpus.join("design.json"))?;
        let original: Value = serde_json::from_slice(&bytes)?;
        let dataset = prepare(&original)?;
        write(&out, "dataset.json", &dataset)?;
        let saved = fs::read(out.join("dataset.json"))?;
        write(
            &out,
            "source.json",
            &json!({"schema":"uor-r4.matched-context-data-source/1","corpus_root":corpus,"corpus_file":"design.json","corpus_sha256":format!("{:x}",sha2::Sha256::digest(&bytes)),"preparation_source_blake3":format!("blake3:{}",blake3::hash(include_bytes!("context_data.rs")).to_hex()),"dataset_blake3":format!("blake3:{}",blake3::hash(&saved).to_hex()),"model_calls":0,"scope":"Data frozen before the paired fits; original opened panel neither accessed nor used in preparation."}),
        )?;
        crate::report_output::verify(&corpus)?;
        Ok(())
    })();
    if let Err(error) = &result {
        write(
            &out,
            "failure.json",
            &json!({"status":"INCOMPLETE","error":error.to_string()}),
        )?;
    }
    crate::report_output::seal(&out)?;
    crate::report_output::verify(&out)?;
    result
}
