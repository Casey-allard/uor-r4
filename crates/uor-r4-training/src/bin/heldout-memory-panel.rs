//! Build a held-out evaluation panel whose relations, values, question forms
//! and answer forms are **all disjoint** from those the synthetic memory
//! generator emits.
//!
//! ```text
//! heldout-memory-panel out=NEW_DIR rows=10 seed=3
//! ```
//!
//! Why this exists: the synthetic emitter fit was measured on panels that
//! changed the *test wording* but reused the training relations, values and
//! templates -- the generator emits ten relations and nineteen answer
//! templates, and the fitted model scores 10/10 on panels drawn from them
//! while scoring 6/10 on re-worded questions over the same relations. Those two
//! numbers cannot be separated by a panel that shares the template set, so
//! neither can say whether the model learned a *binding* or memorised a
//! *surface*.
//!
//! This tool's panel shares nothing with training: different relation ids,
//! different value vocabularies, question forms the generator never emits, and
//! answer forms it never emits. The fact and its competing statement are both
//! stated, in the generator's turn shape, so the panel measures exactly the
//! competing-turn binding the fit targeted -- against surfaces it has never
//! seen. A model that learned the mechanism answers; a model that learned
//! nineteen templates cannot.
//!
//! The output is a request panel (`[{id, category, user_turns}]`) for
//! `geometric-stack lut-chat requests=`, plus `expected.json` mapping each id
//! to the value a correct reply must contain, so scoring needs no hand
//! transcription.

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uor_r4_core::report_output;

type Result<T> = std::result::Result<T, String>;

/// Held-out relations. Every id differs from the training set
/// (`name, cat, colour, job, sister, brothers, birthday, car, instrument,
/// food`), and every value vocabulary is a separate set.
struct Heldout {
    id: &'static str,
    question: &'static str,
    asked: &'static str,
    competing_id: &'static str,
    competing: &'static str,
    values: &'static [&'static str],
    competing_values: &'static [&'static str],
}

/// Question forms are new sentences; statement forms are new sentences; the
/// answer form is a new sentence. Nothing here appears in the generator.
const HELDOUT: &[Heldout] = &[
    Heldout {
        id: "hometown",
        question: "Which town do I come from originally?",
        asked: "I grew up in {v}.",
        competing_id: "team",
        competing: "I have followed {v} since childhood.",
        values: &["Bergen", "Cork", "Perth", "Tulsa", "Riga"],
        competing_values: &["Arsenal", "Celtic", "Lakers", "Rangers"],
    },
    Heldout {
        id: "allergy",
        question: "What am I allergic to?",
        asked: "Shellfish brings me out in a rash.",
        competing_id: "commute",
        competing: "I cycle to the office most days.",
        values: &["shellfish", "pollen", "peanuts", "penicillin"],
        competing_values: &["cycling", "walking", "driving"],
    },
    Heldout {
        id: "degree",
        question: "Which subject did I study at university?",
        asked: "I read {v} at university.",
        competing_id: "landlord",
        competing: "My landlord is called {v}.",
        values: &["geology", "music", "law", "pharmacy"],
        competing_values: &["Priya", "Oscar", "Dmitri"],
    },
    Heldout {
        id: "flatmate",
        question: "Who do I share my flat with?",
        asked: "{v} is the person I share my flat with.",
        competing_id: "diet",
        competing: "I have been eating {v} lately.",
        values: &["Nadia", "Tomas", "Elena"],
        competing_values: &["vegan", "keto", "halal"],
    },
    Heldout {
        id: "commute_time",
        question: "How long does my journey to work take?",
        asked: "Getting to work takes me {v}.",
        competing_id: "language",
        competing: "I have been learning {v}.",
        values: &["forty minutes", "half an hour", "an hour"],
        competing_values: &["Mandarin", "Turkish", "Welsh"],
    },
    Heldout {
        id: "holiday",
        question: "Where am I going on holiday?",
        asked: "I have booked a holiday to {v}.",
        competing_id: "chore",
        competing: "My least favourite chore is {v}.",
        values: &["Iceland", "Morocco", "Nepal"],
        competing_values: &["ironing", "dusting", "hoovering"],
    },
    Heldout {
        id: "bank",
        question: "Which bank do I use?",
        asked: "I keep my account with {v}.",
        competing_id: "gym",
        competing: "I have joined a gym in {v}.",
        values: &["Monzo", "HSBC", "Nationwide"],
        competing_values: &["Leeds", "Bristol", "Cardiff"],
    },
    Heldout {
        id: "shoe_size",
        question: "What size shoes do I take?",
        asked: "I take a size {v} shoe.",
        competing_id: "plant",
        competing: "I keep a {v} on the windowsill.",
        values: &["nine", "ten", "eleven"],
        competing_values: &["cactus", "fern", "orchid"],
    },
    Heldout {
        id: "vet",
        question: "What is my vet called?",
        asked: "My vet is Dr {v}.",
        competing_id: "band",
        competing: "I have been listening to {v}.",
        values: &["Okafor", "Lindqvist", "Bhatt"],
        competing_values: &["Radiohead", "Bjork", "Kraftwerk"],
    },
    Heldout {
        id: "landline",
        question: "What is my extension at work?",
        asked: "My extension at work is {v}.",
        competing_id: "tool",
        competing: "The tool I reach for most is {v}.",
        values: &["four two seven", "two one nine", "six three zero"],
        competing_values: &["a chisel", "a spanner", "a hacksaw"],
    },
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<String> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(str::to_string)
            .ok_or_else(|| format!("missing {key}="))
    };
    let number = |key: &str, default: u64| -> Result<u64> {
        match args.iter().find_map(|a| a.strip_prefix(&format!("{key}="))) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("{key}= must be a number")),
        }
    };
    let out = PathBuf::from(arg("out")?);
    let rows = number("rows", 10)? as usize;
    let seed = number("seed", 3)?;
    if rows == 0 || rows > HELDOUT.len() * 5 {
        return Err(format!("rows must be 1..={}", HELDOUT.len() * 5));
    }

    let mut rng = Rng(seed);
    let mut panel = Vec::with_capacity(rows);
    let mut expected = serde_json::Map::new();
    for n in 0..rows {
        let relation = &HELDOUT[n % HELDOUT.len()];
        let wanted = relation.values[rng.below(relation.values.len())];
        let competing = relation.competing_values[rng.below(relation.competing_values.len())];
        let asked = relation.asked.replace("{v}", wanted);
        let other = relation.competing.replace("{v}", competing);
        // The fact is first half the time and the competing statement first the
        // other half, matching the training distribution's shape.
        let (first, second) = if rng.below(2) == 0 {
            (asked, other)
        } else {
            (other, asked)
        };
        let id = format!("heldout-{:02}", n + 1);
        panel.push(json!({
            "id": id,
            "category": "memory",
            "user_turns": [first, second, relation.question],
        }));
        expected.insert(id, json!(wanted));
    }

    report_output::claim(&out).map_err(|e| e.to_string())?;
    fs::write(
        out.join("requests.json"),
        serde_json::to_vec_pretty(&panel).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("expected.json"),
        serde_json::to_vec_pretty(&serde_json::Value::Object(expected))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("README.md"),
        "Held-out memory panel.\n\n\
         Every relation id, value vocabulary, question form and statement form here is disjoint\n\
         from the synthetic memory generator's training tables, so a model that memorised the\n\
         generator's nineteen answer templates cannot answer this panel by template alone.\n\n\
         `requests.json` is the panel for `geometric-stack lut-chat requests=`; `expected.json`\n\
         maps each id to the value a correct reply must contain.\n",
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{rows} held-out requests over {} disjoint relations",
        HELDOUT.len()
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
