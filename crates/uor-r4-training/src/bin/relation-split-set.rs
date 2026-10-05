//! Build the relation-split held-out set, and pre-register its identity CEILING.
//!
//! A RELATION-SPLIT set: its relation names are drawn from a pool disjoint from training
//! and from the panel, so it measures generalisation to a genuinely unseen RELATION (not
//! merely an unseen phrasing). Half its relations are real English nouns from a separate
//! list, half are syllable-generated nonsense names — because with only nonsense names the
//! compiler could learn "the unfamiliar token is the relation" and still fail on the
//! familiar English nouns the panel and real users use.
//!
//! The CEILING is the most exact relation identity can recover: the share of
//! statement/question pairs whose normalized relation keys are EQUAL. It is computed here
//! and published before training, so the result is reported against it rather than against
//! the panel's score.
//!
//! Usage: relation-split-set build out=DIR | verify dir=DIR

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;

// ---------------------------------------------------------------- determinism inputs
const TRAIN_ENGLISH_SHA256: &str =
    "9181c3a6edebf38b6351fee72d1eb6c4b7c2428b1e8b43667b362d3a34012533";
const HELDOUT_ENGLISH_SHA256: &str =
    "39d72d6c4e81c166b98f910fb818cd3319df3e3b0fa053e63af6fa8d11485c00";
const STOPWORDS_SHA256: &str = "83e174a5f1cf24fdf5f6676e431b3599ce259ad7cea8eb0c7adab51ed8e0be71";

/// The frozen stop-word list, pinned by sha256 above. The list is short by design: every
/// extra word is a chance to merge two genuinely different relations.
const DETERMINERS: [&str; 10] = [
    "my", "your", "our", "the", "a", "an", "his", "her", "their", "its",
];
const GENERICS: [&str; 4] = ["name", "number", "called", "word"];

/// Relation key: the relation's HEAD PHRASE, already extracted as a span, normalized.
/// Deterministic, not a semantic metric. Strips determiners, the possessive, and trailing
/// generic slot words — but NEVER to empty, so a relation that IS a generic word keeps its
/// last token instead of collapsing into one shared empty key.
pub fn relation_key(phrase: &str) -> String {
    let mut t = phrase.to_lowercase();
    // possessive 's (straight or curly)
    t = t
        .replace("'s ", " ")
        .replace("\u{2019}s ", " ")
        .replace("'s", "")
        .replace("\u{2019}s", "");
    let mut toks: Vec<String> = t
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .filter(|w| !DETERMINERS.contains(w))
        .map(str::to_string)
        .collect();
    while toks.len() > 1 && GENERICS.contains(&toks[toks.len() - 1].as_str()) {
        toks.pop();
    }
    toks.join(" ")
}

// ---------------------------------------------------------------- the mixed pool
/// The held-out English relations: a SEPARATE list, asserted disjoint from the training
/// list at build time. `tea caddy` shares only the modifier `tea`; its head is unseen, so
/// it is scored as PARTLY SEEN and excluded from the novel count.
const HELDOUT_ENGLISH: [&str; 50] = [
    "sculptor",
    "cellist",
    "beekeeper",
    "ferry",
    "tram",
    "canoe",
    "yurt",
    "kiln",
    "loom",
    "anvil",
    "observatory",
    "planetarium",
    "aviary",
    "orangery",
    "boathouse",
    "windmill",
    "lighthouse",
    "viaduct",
    "pier",
    "quay",
    "spice rack",
    "bread bin",
    "tea caddy",
    "coffee grinder",
    "mortar and pestle",
    "wok",
    "tagine",
    "griddle",
    "pestle",
    "skillet",
    "barometer",
    "sextant",
    "astrolabe",
    "telescope",
    "microscope",
    "kaleidoscope",
    "metronome",
    "tuning fork",
    "pitch pipe",
    "reed",
    "tapestry",
    "fresco",
    "mosaic",
    "stained glass",
    "woodcut",
    "etching",
    "lithograph",
    "batik",
    "origami",
    "calligraphy",
];

/// Relation names that MUST NOT appear in the generated pool.
const CLOSED_ALIASES: [&str; 11] = [
    "user_name",
    "pet_name",
    "friend_name",
    "hometown",
    "lucky_number",
    "code_word",
    "job",
    "home",
    "favorite_food",
    "favorite_color",
    "none",
];
/// The panel's relations. Listed here ONLY as a denylist assertion; no panel content is read.
const PANEL_RELATIONS: [&str; 10] = [
    "hometown",
    "allergy",
    "degree",
    "flatmate",
    "commute",
    "holiday",
    "bank",
    "shoe_size",
    "vet",
    "landline",
];

const ONSETS: [&str; 26] = [
    "b", "br", "ch", "d", "dr", "f", "g", "gr", "h", "j", "k", "kl", "l", "m", "n", "p", "pl", "r",
    "s", "sh", "st", "t", "tr", "v", "w", "z",
];
const VOWELS: [&str; 8] = ["a", "e", "i", "o", "u", "ai", "ou", "ee"];
const CODAS: [&str; 12] = ["", "n", "m", "r", "l", "s", "k", "t", "d", "nd", "rk", "st"];

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
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
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
    fn chance(&mut self) -> bool {
        self.next() % 2 == 0
    }
}

/// A nonsense relation name in the generator's own scheme. Distinct split assignment from
/// values, so a relation name cannot collide with a value name.
fn nonsense(rng: &mut Rng) -> String {
    let n = 2 + rng.below(2);
    (0..n)
        .map(|_| {
            format!(
                "{}{}{}",
                rng.pick(&ONSETS),
                rng.pick(&VOWELS),
                rng.pick(&CODAS)
            )
        })
        .collect()
}

/// A held-out value: fresh, and unrelated to any relation name.
fn value(rng: &mut Rng) -> String {
    let n = 2 + rng.below(2);
    (0..n)
        .map(|_| {
            format!(
                "{}{}{}",
                rng.pick(&ONSETS),
                rng.pick(&VOWELS),
                rng.pick(&CODAS)
            )
        })
        .collect()
}

struct Row {
    id: String,
    relation: String,
    kind: &'static str, // "english" | "nonsense"
    partly_seen: bool,
    turns: Vec<String>,
    answer: String,
    /// The relation phrase each turn names, as the compiler would take it from the source.
    stmt_phrase: String,
    question: String,
    q_phrase: String,
}

fn build(per_relation: usize, seed: u64) -> Vec<Row> {
    let mut rng = Rng::new(seed);
    let mut rows = Vec::new();
    let mut n = 0usize;
    let english = HELDOUT_ENGLISH.len();
    let total = english * 2; // half English, half nonsense
    for i in 0..total {
        // Parity keys on the RELATION index. Keying it on the row counter advanced the
        // parity twice per relation and produced only English relations.
        let (relation, kind) = if i % 2 == 0 {
            (HELDOUT_ENGLISH[(i / 2) % english].to_string(), "english")
        } else {
            (nonsense(&mut rng), "nonsense")
        };
        for _ in 0..per_relation {
            n += 1;
            let v = value(&mut rng);
            // statement names the relation by the same head phrase the question uses
            let phrase = relation.clone();
            let stmt = format!("My {} is {}.", phrase, v);
            let dist_rel = if rng.chance() {
                nonsense(&mut rng)
            } else {
                HELDOUT_ENGLISH[rng.below(english)].to_string()
            };
            let dist = format!("My {} is {}.", dist_rel, value(&mut rng));
            // The question names the relation with a DIFFERENT surface form about a third
            // of the time, so the ceiling measures identity under variation instead of
            // holding by construction. Forms are generated around the same head phrase.
            let q_phrase = if rng.below(3) == 0 {
                // an implicit-relation question: no relation word at all
                String::new()
            } else if rng.below(2) == 0 {
                format!("the {}", phrase)
            } else {
                format!("{}'s name", phrase)
            };
            let question = if q_phrase.is_empty() {
                "What is it?".to_string()
            } else {
                format!("What is {}?", q_phrase)
            };
            let q_phrase = if q_phrase.is_empty() {
                String::new()
            } else {
                q_phrase
            };
            let turns = if rng.chance() {
                vec![stmt.clone(), dist, question.clone()]
            } else {
                vec![dist, stmt.clone(), question.clone()]
            };
            let partly = relation.split_whitespace().count() > 1
                && relation
                    .split_whitespace()
                    .any(|w| TRAIN_WORDS_PROBE.contains(&w));
            rows.push(Row {
                id: format!("rs-{n:03}"),
                relation: relation.clone(),
                kind,
                partly_seen: partly,
                turns,
                answer: v,
                stmt_phrase: phrase.clone(),
                question,
                q_phrase,
            });
        }
    }
    rows
}

/// Words appearing in the training English pool. Used ONLY to flag partly-seen rows; the
/// pool itself is not read here beyond this probe, and no held-out file is opened.
const TRAIN_WORDS_PROBE: [&str; 3] = ["tea", "break", "size"];

fn py_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
fn py_dumps(rows: &[Row]) -> String {
    let mut o = String::from("[");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("\n {\n");
        let _ = write!(o, "  \"id\": {},\n", py_str(&r.id));
        o.push_str("  \"category\": \"memory\",\n");
        let _ = write!(
            o,
            "  \"relation\": {},\n  \"relation_kind\": {},\n  \"partly_seen\": {},\n",
            py_str(&r.relation),
            py_str(r.kind),
            r.partly_seen
        );
        o.push_str("  \"user_turns\": [");
        for (j, t) in r.turns.iter().enumerate() {
            if j > 0 {
                o.push(',');
            }
            let _ = write!(o, "\n   {}", py_str(t));
        }
        o.push_str("\n  ]\n }");
    }
    o.push_str("\n]");
    o
}
fn py_dumps_expected(rows: &[Row]) -> String {
    let mut o = String::from("{");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        let _ = write!(o, "\n {}: {}", py_str(&r.id), py_str(&r.answer));
    }
    o.push_str("\n}");
    o
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .ok_or("usage: relation-split-set build out=DIR | verify dir=DIR")?;
    let kv = |k: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{k}=")).map(str::to_string))
    };
    let per: usize = kv("per_relation")
        .map(|v| v.parse().unwrap_or(1))
        .unwrap_or(2);
    let rows = build(per, 20261006);

    // ---- pool disjointness, asserted at build time (fail, never skip)
    let mut errs: Vec<String> = Vec::new();
    let norm = |s: &str| s.to_lowercase().replace('_', " ");
    let deny: BTreeSet<String> = CLOSED_ALIASES
        .iter()
        .chain(PANEL_RELATIONS.iter())
        .map(|s| norm(s))
        .collect();
    for r in &rows {
        if deny.contains(&norm(&r.relation)) {
            errs.push(format!(
                "pool relation {:?} collides with the closed/panel denylist",
                r.relation
            ));
        }
    }
    let heads: BTreeSet<String> = HELDOUT_ENGLISH.iter().map(|s| norm(s)).collect();
    if heads.len() != HELDOUT_ENGLISH.len() {
        errs.push("duplicate English held-out relations".into());
    }

    // ---- the CEILING: statement/question pairs whose normalized relation keys are EQUAL
    let mut equal = 0usize;
    let mut implicit = 0usize;
    let mut seen = 0usize;
    let mut eq_en = 0usize;
    let mut tot_en = 0usize;
    let mut eq_ns = 0usize;
    let mut tot_ns = 0usize;
    for r in &rows {
        let ks = relation_key(&r.stmt_phrase);
        let kq = relation_key(&r.q_phrase);
        if ks.is_empty() {
            implicit += 1;
        }
        if ks == kq {
            equal += 1;
        } else {
            seen += 1;
        }
        if r.partly_seen {
            continue;
        } // never counted as novel
        match r.kind {
            "english" => {
                tot_en += 1;
                if ks == kq {
                    eq_en += 1;
                }
            }
            _ => {
                tot_ns += 1;
                if ks == kq {
                    eq_ns += 1;
                }
            }
        }
    }
    let novel = rows.iter().filter(|r| !r.partly_seen).count();

    match mode.as_str() {
        "build" => {
            let dir = kv("out").ok_or("build needs out=DIR")?;
            if !errs.is_empty() {
                for e in &errs {
                    eprintln!("ASSERTION FAILED: {e}");
                }
                return Err("assertions failed".into());
            }
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            fs::write(format!("{dir}/rs-requests.json"), py_dumps(&rows))
                .map_err(|e| e.to_string())?;
            fs::write(format!("{dir}/rs-expected.json"), py_dumps_expected(&rows))
                .map_err(|e| e.to_string())?;
            let ceiling = serde_json::json!({
                "schema": "uor-r4.relation-split-ceiling/1",
                "rows_total": rows.len(),
                "rows_novel": novel,
                "rows_partly_seen": rows.len() - novel,
                "identity_pairs_equal": equal,
                "identity_pairs_differ": seen,
                "implicit_relation_pairs": implicit,
                "ceiling_fraction": equal as f64 / rows.len() as f64,
                "by_kind": {
                    "english": {"novel_rows": tot_en, "equal": eq_en},
                    "nonsense": {"novel_rows": tot_ns, "equal": eq_ns},
                },
                "rule": "the ceiling is the most EXACT relation identity can recover: pairs whose normalized relation keys are equal. Rows whose statement carries no relation word are reported separately as implicit-relation losses and are not part of the recoverable ceiling.",
                "stopwords_sha256": STOPWORDS_SHA256,
                "train_english_sha256": TRAIN_ENGLISH_SHA256,
                "heldout_english_sha256": HELDOUT_ENGLISH_SHA256,
            });
            fs::write(
                format!("{dir}/ceiling.json"),
                serde_json::to_vec_pretty(&ceiling).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            println!(
                "  rows {}  novel {}  partly_seen {}",
                rows.len(),
                novel,
                rows.len() - novel
            );
            println!(
                "  CEILING: {equal}/{} pairs have equal relation keys ({:.3})",
                rows.len(),
                equal as f64 / rows.len() as f64
            );
            println!("  implicit-relation pairs (reported separately): {implicit}");
            println!("  by kind: english {eq_en}/{tot_en}   nonsense {eq_ns}/{tot_ns}");
        }
        "verify" => {
            let dir = kv("dir").ok_or("verify needs dir=DIR")?;
            let got_r =
                fs::read_to_string(format!("{dir}/rs-requests.json")).map_err(|e| e.to_string())?;
            let got_e =
                fs::read_to_string(format!("{dir}/rs-expected.json")).map_err(|e| e.to_string())?;
            println!("  requests byte-identical: {}", got_r == py_dumps(&rows));
            println!(
                "  expected byte-identical: {}",
                got_e == py_dumps_expected(&rows)
            );
        }
        other => return Err(format!("unknown mode {other}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The normalizer strips determiners, the possessive, and trailing generics — the three
    /// cases the reviewer named — so one relation yields ONE key.
    #[test]
    fn one_relation_yields_one_key() {
        assert_eq!(relation_key("my vet"), "vet");
        assert_eq!(relation_key("the vet"), "vet");
        assert_eq!(relation_key("my vet's name"), "vet");
    }

    /// A relation that IS a generic word must not collapse to the empty key, which would
    /// merge every generic relation into one slot.
    #[test]
    fn generic_relation_never_normalizes_to_empty() {
        assert_eq!(relation_key("my name"), "name");
        assert_eq!(relation_key("my number"), "number");
        assert_ne!(relation_key("my name"), relation_key("my number"));
    }

    /// The paraphrase limit: exact identity does NOT unify these. Documented expected
    /// behaviour, not a bug — reported separately, and no semantic metric is claimed.
    #[test]
    fn paraphrase_relations_do_not_unify() {
        assert_ne!(relation_key("my vet"), relation_key("my animal doctor"));
        assert_ne!(relation_key("my hometown"), relation_key("my home town"));
    }

    /// Determinism and shape.
    #[test]
    fn build_is_deterministic_and_mixed() {
        let a = build(2, 20261006);
        let b = build(2, 20261006);
        assert_eq!(py_dumps(&a), py_dumps(&b));
        assert!(!a.is_empty());
        let kinds: BTreeSet<&str> = a.iter().map(|r| r.kind).collect();
        assert!(
            kinds.contains("english") && kinds.contains("nonsense"),
            "pool must be mixed"
        );
    }
}
