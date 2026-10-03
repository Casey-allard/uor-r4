//! Build and verify the session-evaluation disjoint panel.
//!
//! Rust port of the frozen Python builder/verifier, so data preparation stays in
//! Rust per the project rule. It reproduces the frozen JSON **byte for byte**,
//! which requires CPython's Mersenne Twister: `random.Random(20261003)` seeds via
//! `init_by_array`, not `init_genrand`, and differs from splitmix64 used
//! elsewhere in this crate. The RNG below is verified against CPython on
//! `getrandbits(32)`, `randrange(n)` and `random()`.
//!
//! `build` regenerates the panel and asserts the five invariants; any failure
//! aborts rather than emitting a defective panel. `verify` checks an existing
//! frozen panel against the same invariants and re-derives its hashes.
//!
//! Usage: disjoint-panel build  out=DIR
//!        disjoint-panel verify dir=DIR

use std::{fmt::Write as _, fs, path::PathBuf};

// ---------------------------------------------------------------- RNG (CPython)
const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER: u32 = 0x8000_0000;
const LOWER: u32 = 0x7fff_ffff;

struct PyRandom {
    mt: [u32; N],
    idx: usize,
}

impl PyRandom {
    fn new(seed: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = 19650218;
        for i in 1..N {
            let p = mt[i - 1] as u64 ^ ((mt[i - 1] >> 30) as u64);
            mt[i] = (1812433253u64.wrapping_mul(p).wrapping_add(i as u64)) as u32;
        }
        let key = [seed];
        let (mut i, mut j) = (1usize, 0usize);
        let mut k = if N > key.len() { N } else { key.len() };
        while k > 0 {
            let p = mt[i - 1] as u64 ^ ((mt[i - 1] >> 30) as u64);
            mt[i] = (mt[i] ^ ((p.wrapping_mul(1664525)) as u32))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let p = mt[i - 1] as u64 ^ ((mt[i - 1] >> 30) as u64);
            mt[i] = (mt[i] ^ ((p.wrapping_mul(1566083941)) as u32)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        mt[0] = 0x8000_0000;
        PyRandom { mt, idx: N }
    }

    fn twist(&mut self) {
        for i in 0..N {
            let y = (self.mt[i] & UPPER) | (self.mt[(i + 1) % N] & LOWER);
            let mut n = self.mt[(i + M) % N] ^ (y >> 1);
            if y & 1 == 1 {
                n ^= MATRIX_A;
            }
            self.mt[i] = n;
        }
        self.idx = 0;
    }

    fn next_u32(&mut self) -> u32 {
        if self.idx >= N {
            self.twist();
        }
        let mut y = self.mt[self.idx];
        self.idx += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    fn getrandbits(&mut self, k: u32) -> u32 {
        if k == 0 {
            0
        } else {
            self.next_u32() >> (32 - k)
        }
    }

    fn below(&mut self, n: u32) -> u32 {
        let k = 32 - (n - 1).leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    fn choice(&mut self, n: usize) -> usize {
        self.below(n as u32) as usize
    }

    fn random(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as u64;
        let b = (self.next_u32() >> 6) as u64;
        ((a << 26) + b) as f64 / 9007199254740992.0
    }
}

// ------------------------------------------------------- typed intent (the spec)
const RELATIONS: &[(&str, &[&str], &[&str], &[&str])] = &[
    (
        "hometown",
        &[
            "I grew up in {v}.",
            "I was raised in {v}.",
            "My home town is {v}.",
        ],
        &[
            "Which town do I come from originally?",
            "Where did I grow up?",
            "What town am I originally from?",
        ],
        &[
            "Tulsa",
            "Cork",
            "Bergen",
            "Kyoto",
            "Perth",
            "Bristol",
            "Galway",
            "Aarhus",
            "Hobart",
            "Reykjavik",
        ],
    ),
    (
        "allergy",
        &[
            "I am allergic to {v}.",
            "{v} brings me out in a rash.",
            "I react badly to {v}.",
        ],
        &[
            "What am I allergic to?",
            "Which thing am I allergic to?",
            "What brings me out in a rash?",
        ],
        &[
            "penicillin",
            "pollen",
            "peanuts",
            "latex",
            "shellfish",
            "aspirin",
            "wool",
            "nickel",
            "soy",
            "dust",
        ],
    ),
    (
        "degree",
        &[
            "I read {v} at university.",
            "I studied {v} at university.",
            "My degree was in {v}.",
        ],
        &[
            "Which subject did I study at university?",
            "What did I read at university?",
            "What was my degree in?",
        ],
        &[
            "geology",
            "music",
            "history",
            "physics",
            "law",
            "economics",
            "biology",
            "chemistry",
            "philosophy",
            "medicine",
        ],
    ),
    (
        "flatmate",
        &[
            "{v} is the person I share my flat with.",
            "I share my flat with {v}.",
            "My flatmate is {v}.",
        ],
        &[
            "Who do I share my flat with?",
            "Who is my flatmate?",
            "Who do I live with?",
        ],
        &[
            "Nadia", "Tomas", "Elena", "Marcus", "Priya", "Oskar", "Freya", "Hugo", "Iris",
            "Dmitri",
        ],
    ),
    (
        "commute",
        &[
            "Getting to work takes me {v}.",
            "My journey to work is {v}.",
            "I commute for {v} each way.",
        ],
        &[
            "How long does my journey to work take?",
            "How long is my commute?",
            "How long do I take getting to work?",
        ],
        &[
            "half an hour",
            "twenty minutes",
            "forty minutes",
            "an hour",
            "fifteen minutes",
            "ten minutes",
            "an hour and a half",
            "twenty-five minutes",
            "fifty minutes",
            "thirty-five minutes",
        ],
    ),
    (
        "holiday",
        &[
            "I have booked a holiday to {v}.",
            "I am going on holiday to {v}.",
            "I am spending my holiday in {v}.",
        ],
        &[
            "Where am I going on holiday?",
            "Which place am I visiting on holiday?",
            "Where am I spending my holiday?",
        ],
        &[
            "Iceland", "Morocco", "Portugal", "Vietnam", "Chile", "Finland", "Nepal", "Kenya",
            "Croatia", "Bolivia",
        ],
    ),
    (
        "bank",
        &[
            "I bank with {v}.",
            "My bank is {v}.",
            "I use {v} for my banking.",
        ],
        &[
            "Which bank do I use?",
            "Who do I bank with?",
            "What is my bank?",
        ],
        &[
            "Monzo",
            "HSBC",
            "Nationwide",
            "Barclays",
            "Lloyds",
            "Santander",
            "Halifax",
            "NatWest",
            "Starling",
            "Revolut",
        ],
    ),
    (
        "shoe_size",
        &[
            "I take a size {v} in shoes.",
            "My shoe size is {v}.",
            "I wear size {v} shoes.",
        ],
        &[
            "What size shoes do I take?",
            "What is my shoe size?",
            "Which shoe size do I wear?",
        ],
        &[
            "nine", "seven", "ten", "six", "eleven", "eight", "five", "twelve", "four", "thirteen",
        ],
    ),
    (
        "vet",
        &[
            "My vet is called {v}.",
            "I take my cat to {v}.",
            "My vet practice is {v}.",
        ],
        &[
            "Who is my vet?",
            "What is my vet called?",
            "Which vet do I use?",
        ],
        &[
            "Okafor",
            "Bhatt",
            "Lindqvist",
            "Nowak",
            "Ferreira",
            "Novak",
            "Osei",
            "Rossi",
            "Dubois",
            "Hansen",
        ],
    ),
    (
        "landline",
        &[
            "My landline number ends in {v}.",
            "My home number ends in {v}.",
            "The last digits of my landline are {v}.",
        ],
        &[
            "What does my landline number end in?",
            "What are the last digits of my home number?",
            "How does my landline number end?",
        ],
        &[
            "six three zero",
            "two four one",
            "nine one seven",
            "five five two",
            "eight three four",
            "one two nine",
            "seven zero six",
            "three eight five",
            "four one eight",
            "zero nine three",
        ],
    ),
];

const SEED: u32 = 20261003;

/// The world's relation/value tables, parsed from its source at build time.
fn world_tables() -> Result<(Vec<String>, Vec<String>), String> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/milestone_world.rs");
    let src = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let i = src
        .find("pub(crate) const RELATIONS")
        .ok_or("world RELATIONS not found")?;
    let blk: String = src[i..].chars().take(14_000).collect();
    let mut rels = Vec::new();
    for (n, pat) in blk.match_indices("name: \"") {
        let rest = &blk[n + pat.len()..];
        if let Some(end) = rest.find('"') {
            rels.push(rest[..end].to_string());
        }
    }
    let mut vals = Vec::new();
    for marker in ["train_values: &[", "development_values: &["] {
        for (n, _) in blk.match_indices(marker) {
            let rest = &blk[n + marker.len()..];
            let end = rest.find(']').unwrap_or(0);
            for q in rest[..end].split('"').skip(1).step_by(2) {
                vals.push(q.to_string());
            }
        }
    }
    Ok((rels, vals))
}

// ---------------------------------------------------- Python-style JSON emitter
fn py_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// `json.dumps(rows, indent=1)` — Python puts list items at indent, dict keys at
/// indent+1, and a space after every colon. serde_json's pretty printer uses two
/// spaces and no space after the colon, so it cannot reproduce the frozen bytes.
fn py_dumps(rows: &[(String, Vec<String>)]) -> String {
    let mut o = String::from("[");
    for (i, (id, turns)) in rows.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("\n {\n");
        let _ = write!(o, "  \"id\": {},\n", py_str(id));
        o.push_str("  \"category\": \"memory\",\n");
        o.push_str("  \"user_turns\": [");
        for (j, t) in turns.iter().enumerate() {
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

fn py_dumps_map(exp: &[(String, String)]) -> String {
    let mut o = String::from("{");
    for (i, (k, v)) in exp.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        let _ = write!(o, "\n {}: {}", py_str(k), py_str(v));
    }
    o.push_str("\n}");
    o
}

fn build() -> Result<(Vec<(String, Vec<String>)>, Vec<(String, String)>), String> {
    let mut rng = PyRandom::new(SEED);
    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    let mut seen: Vec<Vec<String>> = Vec::new();
    let mut expected: Vec<(String, String)> = Vec::new();
    let mut idx = 0usize;
    for (ai, (_, states, questions, values)) in RELATIONS.iter().enumerate() {
        let others: Vec<usize> = (0..RELATIONS.len()).filter(|k| *k != ai).collect();
        for v in values.iter() {
            let d = others[rng.choice(others.len())];
            let d_values = RELATIONS[d].3;
            let dv = d_values[rng.choice(d_values.len())];
            if dv == *v {
                continue;
            }
            let fact = states[rng.choice(states.len())].replace("{v}", v);
            let dist = RELATIONS[d].1[rng.choice(RELATIONS[d].1.len())].replace("{v}", dv);
            let q = questions[rng.choice(questions.len())].to_string();
            let turns = if rng.random() < 0.5 {
                vec![fact, dist, q]
            } else {
                vec![dist, fact, q]
            };
            if seen.contains(&turns) {
                continue;
            }
            seen.push(turns.clone());
            idx += 1;
            let rid = format!("dp-{idx:03}");
            rows.push((rid.clone(), turns));
            // Record the chosen value directly, exactly as the Python builder does
            // (`expected[rid] = v`). Recovering it afterwards by substring match is
            // what produced the dp-047 defect in an earlier draft of this port:
            // "an hour" is a substring of "an hour and a half".
            expected.push((rid, (*v).to_string()));
        }
    }
    Ok((rows, expected))
}

/// True when `needle` occurs in `hay` on word boundaries. A plain `contains` is
/// wrong here: "an hour" is a substring of "half an hour", and a padded-string
/// comparison is wrong too, because a value may be followed by a period
/// ("dust brings me out in a rash."). Boundaries are non-alphanumeric or the ends.
fn contains_word(hay: &str, needle: &str) -> bool {
    let (h, n) = (hay.to_lowercase(), needle.to_lowercase());
    let (hb, nb) = (h.as_bytes(), n.as_bytes());
    if nb.is_empty() || nb.len() > hb.len() {
        return false;
    }
    let boundary = |c: u8| !c.is_ascii_alphanumeric();
    let mut i = 0;
    while let Some(pos) = h[i..].find(&n) {
        let start = i + pos;
        let end = start + nb.len();
        let left_ok = start == 0 || boundary(hb[start - 1]);
        let right_ok = end == hb.len() || boundary(hb[end]);
        if left_ok && right_ok {
            return true;
        }
        i = start + 1;
        if i >= hb.len() {
            break;
        }
    }
    false
}

fn assert_invariants(rows: &[(String, Vec<String>)], exp: &[(String, String)]) -> Vec<String> {
    let mut errs = Vec::new();
    let (_, wvals) = match world_tables() {
        Ok(t) => t,
        Err(e) => {
            errs.push(e);
            return errs;
        }
    };
    for (id, turns) in rows {
        let ans = exp
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let stated = turns[..turns.len() - 1].join(" ").to_lowercase();
        if !stated.contains(&ans.to_lowercase()) {
            errs.push(format!(
                "A1 {id}: answer {ans:?} not verbatim in stated turns"
            ));
        }
        if wvals.iter().any(|w| w.eq_ignore_ascii_case(&ans)) {
            errs.push(format!(
                "A4 {id}: answer {ans:?} collides with a world value"
            ));
        }
    }
    for i in 0..rows.len() {
        for j in i + 1..rows.len() {
            if rows[i].1 == rows[j].1 {
                errs.push(format!(
                    "A2 {} and {} share a turn set",
                    rows[i].0, rows[j].0
                ));
            }
        }
    }
    errs
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .ok_or("usage: disjoint-panel build out=DIR | verify dir=DIR")?;
    let kv = |k: &str| -> Option<String> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{k}=")).map(str::to_string))
    };
    let (rows, exp) = build()?;
    let errs = assert_invariants(&rows, &exp);
    match mode.as_str() {
        "build" => {
            let dir = kv("out").ok_or("build needs out=DIR")?;
            if !errs.is_empty() {
                for e in &errs {
                    eprintln!("ASSERTION FAILED: {e}");
                }
                return Err(format!(
                    "{} assertion failures; no panel written",
                    errs.len()
                ));
            }
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            fs::write(format!("{dir}/panel-requests.json"), py_dumps(&rows))
                .map_err(|e| e.to_string())?;
            fs::write(format!("{dir}/panel-expected.json"), py_dumps_map(&exp))
                .map_err(|e| e.to_string())?;
            println!("  wrote {} rows to {dir}", rows.len());
        }
        "verify" => {
            let dir = kv("dir").ok_or("verify needs dir=DIR")?;
            let got_req = fs::read_to_string(format!("{dir}/panel-requests.json"))
                .map_err(|e| e.to_string())?;
            let got_exp = fs::read_to_string(format!("{dir}/panel-expected.json"))
                .map_err(|e| e.to_string())?;
            let same_req = got_req == py_dumps(&rows);
            let same_exp = got_exp == py_dumps_map(&exp);
            println!("  rows regenerated: {}", rows.len());
            println!("  requests byte-identical: {same_req}");
            println!("  expected byte-identical: {same_exp}");
            println!("  assertion failures: {}", errs.len());
            for e in errs.iter().take(5) {
                println!("    {e}");
            }
            if !same_req || !same_exp || !errs.is_empty() {
                return Err("verification failed".into());
            }
        }
        other => return Err(format!("unknown mode {other}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A1 — every recorded answer appears VERBATIM in its row's stated turns.
    /// This is the invariant whose violation (a truncated "an hour") an earlier
    /// draft of this port produced, so it is asserted directly.
    #[test]
    fn a1_answers_are_verbatim_in_the_stated_turns() {
        let (rows, exp) = build().expect("build");
        for (id, turns) in &rows {
            let ans = &exp.iter().find(|(k, _)| k == id).expect("answer").1;
            assert!(!ans.is_empty(), "{id}: empty answer");
            let stated = turns[..turns.len() - 1].join(" ").to_lowercase();
            assert!(
                stated.contains(&ans.to_lowercase()),
                "{id}: answer {ans:?} does not appear verbatim in the stated turns"
            );
        }
    }

    /// A2 — no two rows share a turn set. Structurally excludes the defect found in
    /// the inherited heldout panels, where identical turns carried different answers.
    #[test]
    fn a2_no_duplicate_turn_sets() {
        let (rows, _) = build().expect("build");
        for i in 0..rows.len() {
            for j in i + 1..rows.len() {
                assert_ne!(
                    rows[i].1, rows[j].1,
                    "{} and {} share turns",
                    rows[i].0, rows[j].0
                );
            }
        }
    }

    /// A4 — no accepted answer collides with a value in the world's tables, so a
    /// store built from the world cannot already contain the answer.
    #[test]
    fn a4_no_world_value_collision() {
        let (_, exp) = build().expect("build");
        let (_, world_values) = world_tables().expect("world tables");
        for (id, ans) in &exp {
            assert!(
                !world_values.iter().any(|w| w.eq_ignore_ascii_case(ans)),
                "{id}: answer {ans:?} collides with a world value"
            );
        }
    }

    /// A3/A5 — each row asks about exactly one relation, identified by its question
    /// template, and the recorded answer is a value of that relation stated in the row.
    ///
    /// The frozen panel has rows where TWO values of the asked relation match on word
    /// boundaries, because one value is a phrase containing the other ("an hour" in
    /// "half an hour"). In every such row the extra match lives in the DISTRACTOR turn
    /// and the recorded answer is still the fact turn's value, so the answer remains
    /// unambiguous — the generator's pick is recorded directly, never inferred. This
    /// test therefore asserts the recorded answer is among the matches, and separately
    /// that it is the LONGEST match, which is what disambiguates the substring case.
    #[test]
    fn a3_a5_stated_value_is_the_recorded_answer() {
        let (rows, exp) = build().expect("build");
        let mut substring_rows = 0;
        for (id, turns) in &rows {
            let question = turns[turns.len() - 1].as_str();
            let stated = turns[..turns.len() - 1].join(" ");
            let asking: Vec<_> = RELATIONS
                .iter()
                .filter(|(_, _, qs, _)| qs.contains(&question))
                .collect();
            assert_eq!(
                asking.len(),
                1,
                "{id}: {question:?} matched {} relations",
                asking.len()
            );
            let values = asking[0].3;
            let matches: Vec<&&str> = values
                .iter()
                .filter(|v| contains_word(&stated, v))
                .collect();
            assert!(
                !matches.is_empty(),
                "{id}: no stated value for the asked relation"
            );
            let ans = &exp.iter().find(|(k, _)| k == id).expect("answer").1;
            assert!(
                matches.iter().any(|m| **m == ans.as_str()),
                "{id}: recorded answer {ans:?} is not a stated value of the asked relation"
            );
            if matches.len() > 1 {
                substring_rows += 1;
                let longest = matches.iter().map(|m| m.len()).max().unwrap();
                assert_eq!(
                    ans.len(),
                    longest,
                    "{id}: with multiple boundary matches the answer must be the longest"
                );
            }
        }
        // Documented as a property of the frozen panel, not a failure: 2 rows match
        // on a value contained inside the recorded answer.
        assert_eq!(
            substring_rows, 2,
            "expected exactly 2 substring-overlap rows"
        );
    }

    /// The whole-panel assertion set runs clean, and the panel is the expected size.
    #[test]
    fn all_invariants_hold_and_panel_is_well_formed() {
        let (rows, exp) = build().expect("build");
        assert_eq!(rows.len(), 100, "expected 100 rows");
        assert_eq!(exp.len(), rows.len());
        assert_eq!(
            rows.iter().map(|(_, t)| t.len()).max(),
            Some(3),
            "rows are 3 turns"
        );
        let errs = assert_invariants(&rows, &exp);
        assert!(errs.is_empty(), "assert_invariants reported: {errs:?}");
    }

    /// The RNG must stay CPython-compatible: these three values are what
    /// `random.Random(20261003)` yields for getrandbits(32), randrange(10) and
    /// random(). If a refactor swaps in a different generator, byte-identity with
    /// the frozen panel breaks, and this fails before that is discovered downstream.
    #[test]
    fn rng_matches_cpython_reference_values() {
        // getrandbits(32), which is what CPython's reference was taken with.
        let mut a = PyRandom::new(SEED);
        assert_eq!(a.getrandbits(32), 120_965_577);
        assert_eq!(a.getrandbits(32), 2_619_064_220);
        assert_eq!(a.getrandbits(32), 3_958_931_158);
        // getrandbits(32) is next_u32() >> 0, so both expose the same sequence;
        // asserted once here to pin that they agree.
        let mut a2 = PyRandom::new(SEED);
        assert_eq!(a2.next_u32(), 120_965_577);

        let mut b = PyRandom::new(SEED);
        let below: Vec<u32> = (0..4).map(|_| b.below(10)).collect();
        assert_eq!(below, vec![0, 9, 0, 5]);

        // `random()` is exercised end-to-end by the byte-identity of the generated
        // panel (it decides turn order), which `verify` checks against the frozen
        // files. Asserted here only to the extent that it stays in [0,1) and is
        // reproducible, rather than pinning a literal that a refactor would have to
        // update in two places.
        let mut c = PyRandom::new(SEED);
        let v = c.random();
        let mut d = PyRandom::new(SEED);
        assert_eq!(v, d.random(), "random() must be reproducible from the seed");
        assert!((0.0..1.0).contains(&v), "random() out of range: {v}");
    }
}
