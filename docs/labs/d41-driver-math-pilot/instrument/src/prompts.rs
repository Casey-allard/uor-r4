//! The pilot's prompt set: built deterministically from a seed and the templates
//! below, so the set is fixed and hashed before any prompt reaches the donor.
//!
//! Three kinds, drawn per position at 45 / 45 / 10 before duplicates are skipped (small talk has only 30 texts, and repeats are skipped, so the kept mix leans to `grounded`; `prompts` prints it):
//! - `grounded`: one to three facts about the user, then a question about one of
//!   them, or (about one time in five) about a fact the user never gave;
//! - `open`: a simple question or a two-sentence story request about an everyday thing;
//! - `small-talk`: greetings and short social turns.
//!
//! All are single-turn. A prompt is dropped when its text, or any of its sentences,
//! equals a string held in `data/panels/` (see [`PanelGuard`]).

use std::collections::HashSet;

/// The system message every request carries.
pub const SYSTEM: &str = "You are a friendly helper talking with a child. Answer in one to three \
short sentences, using simple words. If the user asks about themselves and has not told you the \
answer, say that they have not told you.";

/// splitmix64: a small, fixed, seedable generator, so the set is reproducible.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A value in `0..n` (`n > 0`).
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    pub fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.below(xs.len())]
    }
}

const NAMES: &[&str] = &[
    "Ava", "Ben", "Chloe", "Dan", "Ella", "Finn", "Grace", "Hugo", "Isla", "Jack", "Kira", "Leo",
    "Maya", "Noah", "Olive", "Pablo", "Quinn", "Rosa", "Sam", "Tara", "Umar", "Vera", "Will",
    "Yara", "Zoe", "Amir", "Bella", "Carlos", "Daisy", "Eli", "Fatima", "Gus", "Hana", "Ivan",
    "Jade", "Kofi", "Lena", "Milo", "Nora", "Omar",
];
const ANIMALS: &[&str] = &[
    "dog",
    "cat",
    "rabbit",
    "hamster",
    "goldfish",
    "parrot",
    "tortoise",
    "pony",
    "guinea pig",
    "budgie",
    "lizard",
    "puppy",
    "kitten",
    "duck",
    "hen",
];
const PLACES: &[&str] = &[
    "Leeds", "Cork", "Perth", "Lyon", "Porto", "Oslo", "Austin", "Denver", "Halifax", "Nairobi",
    "Accra", "Pune", "Osaka", "Hobart", "Malmo", "Gdansk", "Bergen", "Quito", "Lima", "Tucson",
    "Galway", "Dundee", "Bruges", "Graz", "Turku",
];
const COLOURS: &[&str] = &[
    "red", "blue", "green", "yellow", "purple", "orange", "pink", "black", "white", "silver",
];
const FOODS: &[&str] = &[
    "pizza",
    "pasta",
    "rice",
    "soup",
    "pancakes",
    "apples",
    "bananas",
    "carrots",
    "cheese",
    "noodles",
    "toast",
    "strawberries",
    "curry",
    "tacos",
    "porridge",
];
const RELATIVES: &[&str] = &[
    "brother",
    "sister",
    "cousin",
    "aunt",
    "uncle",
    "grandma",
    "grandpa",
    "best friend",
    "neighbour",
];
const JOBS: &[&str] = &[
    "teacher",
    "nurse",
    "baker",
    "farmer",
    "pilot",
    "doctor",
    "builder",
    "painter",
    "driver",
    "vet",
    "chef",
    "librarian",
    "plumber",
    "gardener",
    "dentist",
];
const THINGS: &[&str] = &[
    "bike",
    "bag",
    "hat",
    "coat",
    "kite",
    "ball",
    "scarf",
    "door",
    "lunch box",
    "umbrella",
];

/// One fact the user can give, and the questions that ask for it.
struct Fact {
    slot: &'static str,
    sentence: String,
    questions: Vec<String>,
}

fn fact(rng: &mut Rng, slot: &'static str) -> Fact {
    match slot {
        "pet" => {
            let a = rng.pick(ANIMALS);
            let n = rng.pick(NAMES);
            Fact {
                slot,
                sentence: format!("I have a {a} called {n}."),
                questions: vec![
                    format!("What is my {a} called?"),
                    format!("What is my {a}'s name?"),
                ],
            }
        }
        "home" => Fact {
            slot,
            sentence: format!("I live in {}.", rng.pick(PLACES)),
            questions: vec!["Where do I live?".into(), "Which town do I live in?".into()],
        },
        "colour" => Fact {
            slot,
            sentence: format!("My favourite colour is {}.", rng.pick(COLOURS)),
            questions: vec![
                "What is my favourite colour?".into(),
                "Which colour do I like best?".into(),
            ],
        },
        "food" => Fact {
            slot,
            sentence: format!("My favourite food is {}.", rng.pick(FOODS)),
            questions: vec![
                "What is my favourite food?".into(),
                "What food do I like best?".into(),
            ],
        },
        "name" => Fact {
            slot,
            sentence: format!("My name is {}.", rng.pick(NAMES)),
            questions: vec!["What is my name?".into(), "Do you remember my name?".into()],
        },
        "age" => Fact {
            slot,
            sentence: format!("I am {} years old.", 3 + rng.below(10)),
            questions: vec!["How old am I?".into(), "What age am I?".into()],
        },
        "relative" => {
            let r = rng.pick(RELATIVES);
            let n = rng.pick(NAMES);
            Fact {
                slot,
                sentence: format!("My {r} is called {n}."),
                questions: vec![
                    format!("What is my {r} called?"),
                    format!("What is my {r}'s name?"),
                ],
            }
        }
        "job" => {
            let r = rng.pick(RELATIVES);
            let j = rng.pick(JOBS);
            Fact {
                slot,
                sentence: format!("My {r} works as a {j}."),
                questions: vec![
                    format!("What does my {r} do?"),
                    format!("What is my {r}'s job?"),
                ],
            }
        }
        _ => {
            let t = rng.pick(THINGS);
            let c = rng.pick(COLOURS);
            Fact {
                slot,
                sentence: format!("My {t} is {c}."),
                questions: vec![format!("What colour is my {t}?")],
            }
        }
    }
}

const SLOTS: &[&str] = &[
    "pet", "home", "colour", "food", "name", "age", "relative", "job", "thing",
];

/// Questions about the user that no fact template answers ("not told" rows).
const UNTOLD: &[&str] = &[
    "What is my middle name?",
    "What is my teacher called?",
    "What did I eat for breakfast?",
    "What is my favourite song?",
    "How many cousins do I have?",
    "What is my street called?",
    "What did I dream about last night?",
    "What is my best friend's birthday?",
];

fn grounded(rng: &mut Rng) -> Vec<String> {
    let k = 1 + rng.below(3);
    let mut slots: Vec<&'static str> = Vec::new();
    while slots.len() < k {
        let s = SLOTS[rng.below(SLOTS.len())];
        if !slots.contains(&s) {
            slots.push(s);
        }
    }
    let facts: Vec<Fact> = slots.iter().map(|s| fact(rng, s)).collect();
    let mut parts: Vec<String> = facts.iter().map(|f| f.sentence.clone()).collect();
    if rng.below(5) == 0 {
        parts.push(rng.pick(UNTOLD).to_string());
    } else {
        let f = &facts[rng.below(facts.len())];
        let _ = f.slot;
        parts.push(f.questions[rng.below(f.questions.len())].clone());
    }
    parts
}

const NOUNS: &[&str] = &[
    "cat",
    "dog",
    "cow",
    "horse",
    "bee",
    "frog",
    "owl",
    "fish",
    "bird",
    "whale",
    "elephant",
    "lion",
    "penguin",
    "butterfly",
    "snail",
    "tree",
    "flower",
    "apple",
    "banana",
    "carrot",
    "bread",
    "milk",
    "egg",
    "honey",
    "rain",
    "snow",
    "wind",
    "cloud",
    "rainbow",
    "sun",
    "moon",
    "star",
    "river",
    "sea",
    "mountain",
    "forest",
    "beach",
    "boat",
    "train",
    "bus",
    "bike",
    "plane",
    "car",
    "house",
    "school",
    "park",
    "library",
    "farm",
    "garden",
    "book",
    "pencil",
    "clock",
    "ball",
    "kite",
    "drum",
    "candle",
    "shoe",
    "bridge",
    "castle",
    "volcano",
];
const FRAMES: &[&str] = &[
    "What is a {x}?",
    "Tell me one fact about a {x}.",
    "What colour is a {x}?",
    "Where can I see a {x}?",
    "Why do people like a {x}?",
    "Is a {x} big or small?",
    "Write a two-sentence story about a {x}.",
    "What sound does a {x} make?",
    "Can you describe a {x} in simple words?",
    "What is a {x} for?",
    "How is a {x} different from a {y}?",
    "Which is bigger, a {x} or a {y}?",
    "Write a short story about a {x} and a {y}.",
    "What do a {x} and a {y} have in common?",
];

fn open(rng: &mut Rng) -> Vec<String> {
    let f = rng.pick(FRAMES);
    let x = rng.pick(NOUNS);
    let mut y = rng.pick(NOUNS);
    while y == x {
        y = rng.pick(NOUNS);
    }
    vec![f.replace("{x}", x).replace("{y}", y)]
}

const SMALL_TALK: &[&str] = &[
    "Hi!",
    "Hello there.",
    "Good morning!",
    "Good night.",
    "How are you today?",
    "Thank you!",
    "Thanks for your help.",
    "Can you help me?",
    "Tell me a joke.",
    "I am bored.",
    "I feel sad today.",
    "I am happy today!",
    "What should I play today?",
    "Can we be friends?",
    "Goodbye!",
    "What is your name?",
    "Do you like stories?",
    "I finished my homework.",
    "It is raining here.",
    "I lost my tooth today!",
    "I am hungry.",
    "Can you sing a song?",
    "What can you do?",
    "I had a bad dream.",
    "My friend is not talking to me.",
    "I got a new puppy!",
    "Sorry, I made a mistake.",
    "Is it okay to be scared of the dark?",
    "I can count to one hundred!",
    "Say something funny.",
];

/// Panel strings that must never reach the donor: every JSON string value held in the
/// panel files, compared exactly after trimming.
pub struct PanelGuard {
    strings: HashSet<String>,
}

impl PanelGuard {
    pub fn new() -> Self {
        PanelGuard {
            strings: HashSet::new(),
        }
    }
    fn insert(&mut self, s: &str) {
        let n = normalize(s);
        if !n.is_empty() {
            self.strings.insert(n);
        }
    }
    /// Adds every string value of a JSON document (recursively).
    pub fn add_json(&mut self, v: &serde_json::Value) {
        match v {
            serde_json::Value::String(s) => self.insert(s),
            serde_json::Value::Array(xs) => xs.iter().for_each(|x| self.add_json(x)),
            serde_json::Value::Object(m) => m.values().for_each(|x| self.add_json(x)),
            _ => {}
        }
    }
    /// Adds every line and every tab-separated field of a text file (TSV, TXT), skipping
    /// `#` comment lines.
    pub fn add_tsv(&mut self, text: &str) {
        for line in text.lines() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            self.insert(line);
            for field in line.split('\t') {
                self.insert(field);
            }
        }
    }
    /// Adds a panel file by its extension: `.json` as one document, `.jsonl` line by line
    /// (each line as JSON, and as text when it does not parse), `.tsv` and `.txt` as text.
    /// Returns false for an extension it does not read.
    pub fn add_file(&mut self, ext: &str, text: &str) -> bool {
        match ext {
            "json" => match serde_json::from_str::<serde_json::Value>(text) {
                Ok(v) => self.add_json(&v),
                Err(_) => self.add_tsv(text),
            },
            "jsonl" => {
                for line in text.lines() {
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(v) => self.add_json(&v),
                        Err(_) => self.add_tsv(line),
                    }
                }
            }
            "tsv" | "txt" | "md" => self.add_tsv(text),
            _ => return false,
        }
        true
    }
    pub fn len(&self) -> usize {
        self.strings.len()
    }
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
    /// True when the joined text or any part equals a panel string, after [`normalize`].
    pub fn hits(&self, parts: &[String]) -> bool {
        let joined = parts.join(" ");
        self.strings.contains(&normalize(&joined))
            || parts.iter().any(|p| self.strings.contains(&normalize(p)))
    }
}

/// The form both sides are compared in: lower case, letters and digits only, words split
/// by single spaces. So case, spacing and punctuation never let a panel text through.
pub fn normalize(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Default for PanelGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// One prompt of the set.
#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub id: String,
    pub kind: &'static str,
    pub user: String,
}

/// The counts a build reports.
#[derive(Debug, Default, PartialEq)]
pub struct BuildStats {
    pub duplicates_skipped: usize,
    pub panel_hits_dropped: usize,
}

/// Builds `count` distinct prompts from `seed`. Kinds are drawn per position at 45 / 45 / 10 before duplicates are skipped,
/// so every prefix of the set holds about the same mix.
pub fn build(seed: u64, count: usize, guard: &PanelGuard) -> (Vec<Prompt>, BuildStats) {
    let mut rng = Rng::new(seed);
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(count);
    let mut stats = BuildStats::default();
    let mut small_left: Vec<&str> = SMALL_TALK.to_vec();
    let mut tries = 0usize;
    while out.len() < count {
        tries += 1;
        if tries > count * 200 {
            break;
        }
        let roll = rng.below(100);
        let (kind, parts): (&'static str, Vec<String>) = if roll < 10 && !small_left.is_empty() {
            let i = rng.below(small_left.len());
            ("small-talk", vec![small_left.swap_remove(i).to_string()])
        } else if roll < 55 {
            ("grounded", grounded(&mut rng))
        } else {
            ("open", open(&mut rng))
        };
        let user = parts.join(" ");
        if seen.contains(&user) {
            stats.duplicates_skipped += 1;
            continue;
        }
        if guard.hits(&parts) {
            stats.panel_hits_dropped += 1;
            seen.insert(user);
            continue;
        }
        seen.insert(user.clone());
        out.push(Prompt {
            id: format!("p{:05}", out.len()),
            kind,
            user,
        });
    }
    (out, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_set_is_deterministic_and_distinct() {
        let g = PanelGuard::new();
        let (a, _) = build(7, 3000, &g);
        let (b, _) = build(7, 3000, &g);
        assert_eq!(a, b);
        assert_eq!(a.len(), 3000);
        let uniq: HashSet<&str> = a.iter().map(|p| p.user.as_str()).collect();
        assert_eq!(uniq.len(), a.len());
        let (c, _) = build(8, 3000, &g);
        assert_ne!(a, c);
    }

    #[test]
    fn every_prefix_holds_all_three_kinds() {
        let (a, _) = build(7, 6000, &PanelGuard::new());
        let head = &a[..500];
        for k in ["grounded", "open", "small-talk"] {
            assert!(
                head.iter().any(|p| p.kind == k),
                "{k} missing from the first 500"
            );
        }
        let grounded = head.iter().filter(|p| p.kind == "grounded").count();
        assert!(
            (150..=300).contains(&grounded),
            "grounded {grounded} of 500"
        );
    }

    #[test]
    fn a_panel_string_never_reaches_the_set() {
        let (a, _) = build(7, 2000, &PanelGuard::new());
        // Put one whole prompt and one sentence of another into the guard.
        let whole = a[3].user.clone();
        let sentence = a.iter().find(|p| p.kind == "grounded").map(|p| {
            p.user
                .split_inclusive('.')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        });
        let mut g = PanelGuard::new();
        g.add_json(&serde_json::json!({"rows": [{"user_turns": [whole.clone()]}]}));
        if let Some(s) = &sentence {
            g.add_tsv(&format!("x\t{s}\n"));
        }
        let (b, stats) = build(7, 2000, &g);
        assert!(stats.panel_hits_dropped >= 1);
        assert!(b.iter().all(|p| p.user != whole));
        if let Some(s) = sentence {
            assert!(b.iter().all(|p| !p
                .user
                .split_inclusive(['.', '?', '!'])
                .any(|x| x.trim() == s)));
        }
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;

    #[test]
    fn a_txt_panel_line_in_another_case_and_spacing_is_still_dropped() {
        let (a, _) = build(7, 2000, &PanelGuard::new());
        let target = a.iter().find(|p| p.kind == "open").map(|p| p.user.clone());
        let target = target.unwrap_or_default();
        assert!(!target.is_empty());
        // The panel holds it only in a .txt file, shouted, with odd spacing and punctuation.
        let shouted = format!(
            "# comment line\n  {}!!  \n",
            target.to_uppercase().replace(' ', "   ")
        );
        let mut g = PanelGuard::new();
        assert!(g.add_file("txt", &shouted));
        assert!(!g.add_file("bin", "x"));
        let (b, stats) = build(7, 2000, &g);
        assert!(stats.panel_hits_dropped >= 1);
        assert!(b.iter().all(|p| normalize(&p.user) != normalize(&target)));
    }

    #[test]
    fn normalize_ignores_case_spacing_and_punctuation() {
        assert_eq!(normalize("  What is   a CAT? "), "what is a cat");
        assert_eq!(normalize("My dog's name."), "my dog s name");
        assert_eq!(normalize("?!"), "");
    }
}
