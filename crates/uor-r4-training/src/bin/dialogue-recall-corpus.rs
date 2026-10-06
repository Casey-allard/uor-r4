//! Step 6: a natural-dialogue recall curriculum (#820).
//!
//! ```text
//! dialogue-recall-corpus generate out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   [panels=data/panels[,DIR...]] [seed=1] (dialogues=N | token_budget=N) \
//!   [dev_seed=1000003] [dev_dialogues=300] [protocol=2] [context=384] \
//!   [samples=200] [train_on=answers|all] [generator=v2|v1] [source_commit=SHA] \
//!   [binding_labels=0|1]
//! dialogue-recall-corpus leak out=NEW_REPORT_ROOT store=STORE_DIR tokenizer=TOKENIZER.json \
//!   panels=DIR[,DIR...]
//! ```
//!
//! `generate` writes multi-turn user/assistant dialogues in plain conversational
//! English whose last assistant turns can only be right by retrieving a value
//! stated earlier in the same conversation. It writes two prepared dialogue
//! splits in the format `dialogue-train` and `mix-chat-corpus` read
//! (`uor-r4-chat-corpus/v1`: `tokens.u16`, `response_mask.u8`, `manifest.json`):
//! `OUT/train/` (from `seed`, until `dialogues` or `token_budget`) and a small
//! monitoring split `OUT/dev/` (from `dev_seed`). Beside them: `OUT/leak.json`,
//! the leak report against the frozen panels, and `OUT/generator.json`, the
//! counts per category, seeds, source commit and SHA-256 of every file. The
//! root is claimed before anything is written and sealed at the end.
//!
//! `generator=v2` (the default) mixes the v1 dialogues (share 0.46) with six
//! more question families, each drawn equally often: `rule` (one activity
//! allowed and another not, asked which is or is not allowed, sometimes as a
//! choice between the two), `implicit_update` (a plan changes without
//! "actually": "we voted again and chose ...", "it moved to ...", "now it's
//! ..."; the latest value wins and the first plan can be asked for),
//! `self_fact` (the user's own fact beside another person's: the assistant
//! answers the user's stated fact and abstains only for a relation or person
//! never stated), `attribute` (two objects of one kind told apart by colour,
//! material or size, with different locations or owners), `count` (counts per
//! container or per person, as numerals and number words) and `order`
//! (sequences with first/second/last, after/before, day parts and times).
//! Each family keeps the binding pressure, abstains for a never-stated key
//! about one time in ten, and is reported per family in `generator.json`.
//! `generator=v1` draws exactly the v1 stream (byte-identical stores).
//!
//! `train_on=answers` (the default) sets the response mask to 1 only on the
//! assistant turns that answer a question (content and EOS). Acknowledgements,
//! suggestions and distractor replies keep their tokens in the context with
//! mask 0: `dialogue-train` samples a response per mask-1 run
//! (`dialogue_episodes::EpisodeIndex`) and puts loss only on the sampled
//! run, so they are neither sampled nor trained. Such stores need the index
//! that admits an unmasked EOS closing a context-only assistant turn (added
//! with this generator); an older `dialogue-train` refuses them.
//! `train_on=all` masks every assistant turn. `generator.json` reports reply
//! and reply-token counts per kind, trained and context-only.
//! `binding_labels=1` (Step 7d, default 0) also writes, beside each split's
//! store, `binding_labels.jsonl` (per answer with an expected value stated
//! earlier: the history positions of that value, of the question's forbidden
//! values, and the answer positions that predict the value's tokens) and
//! `binding_labels.json` (counts and the SHA-256 of the token payload the
//! positions refer to). The stores are byte-identical either way;
//! `dialogue-train read_binding_supervision=` reads the sidecar.
//! `panels=` takes a comma-separated list of directories; all of them feed the
//! filters and the leak report.
//!
//! What a dialogue contains (one seeded draw per dialogue, deterministic):
//!
//! - Facts stated in user turns (and, in `assistant_stated`, chosen by the
//!   assistant and accepted by the user), usually two keys of the same type in
//!   one sentence ("the turtle is Shelby and the goldfish is Flash"), so the
//!   typical error is the other key's value.
//! - 0 to 6 distractor exchanges (small talk and unrelated questions with
//!   ordinary replies), an optional fact turn from a second relation, and then
//!   one to three questions. Every question's assistant reply is a scored
//!   response; under `train_on=all` so are the acknowledgements and
//!   distractor replies. The distractor bank holds about 450 exchanges.
//! - Question categories: `binding` (ask one of several same-type keys),
//!   `update` (a later turn corrects one value; the latest value wins, and the
//!   unchanged key keeps its value), `reverse` (who/which key holds a value),
//!   `assistant_stated`, `cross_relation` (one key in each of two relations) and
//!   `abstain` (a key that was never given a value: the reply says so and names
//!   no stated value; about 15% of questions).
//! - Answers vary: the relation's own sentence, a bare value, "You said ...",
//!   "You changed it to ...", "I suggested ...".
//!
//! Disjointness from the frozen panels (`panels=`, default `data/panels`):
//! every `*.json` (all strings except `id` and `category`) and `*.tsv` (every
//! field after the id) is read. The value pools exclude every content word of
//! every panel string and check field, so no recall value shares a word stem
//! with a panel. The values of the panels' recall rows (the `terms` and
//! `forbid` columns of rows whose history is `recall`, including the closed
//! classes of colours, weekdays and the numbers two to twelve) appear nowhere in
//! the generated text. No generated turn equals a panel string (words compared
//! lowercased, punctuation ignored) and no generated turn shares a word 6-gram
//! with one; a draw that would is redrawn and counted. Relation key words
//! (turtle, Grandpa, ...) may coincide with panel words: at least `0.6` of the
//! dialogues use only keys that share no content word with any panel, and the
//! report lists every shared key and frame word with its count.
//!
//! `leak.json` is computed from the written token stores, decoded back to
//! text, not from the generator's own records (except the value and key word
//! lists, which only the records carry). `leak` re-runs that decoded check on
//! any prepared store against any panel directories (for example the local
//! ladder panels).
//!
//! Scope: this is training data. It is not an evaluation, and the development
//! split shares the generator's pools and templates; the frozen panels remain
//! the held-out measurement.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::stack_dialogue::{episode_contract_for, DialogueSplit};
use uor_r4_training::stack_tracking::Rng;

type Result<T> = std::result::Result<T, String>;

const SCHEMA: &str = "uor-r4.dialogue-recall-corpus/1";
const BINDING_LABELS_SCHEMA: &str = "uor-r4.read-binding-labels/1";
const LEAK_SCHEMA: &str = "uor-r4.dialogue-recall-leak/1";
/// Word n-gram length of the panel overlap check.
const NGRAM: usize = 6;
/// Share of dialogues whose keys share no content word with any panel.
const STRICT_KEY_SHARE: f64 = 0.6;
/// Primary question category shares (the follow-ups below add binding and
/// abstention questions; the realised per-question shares are reported).
const PRIMARY_SHARES: [(Category, f64); 6] = [
    (Category::Binding, 0.38),
    (Category::Update, 0.16),
    (Category::Reverse, 0.12),
    (Category::AssistantStated, 0.09),
    (Category::CrossRelation, 0.12),
    (Category::Abstain, 0.13),
];
/// A follow-up question is an abstention with this probability.
const FOLLOWUP_ABSTAIN: f64 = 0.15;
const MAX_DISTRACTORS: usize = 6;

// ---------------------------------------------------------------------------
// Text normalisation shared by the panel reader, the generator and the checks.

#[rustfmt::skip]
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "if", "then", "so", "of", "to", "in", "on", "at", "by",
    "for", "with", "from", "up", "down", "out", "over", "under", "about", "into", "onto", "off",
    "as", "is", "are", "was", "were", "be", "been", "being", "am", "do", "does", "did", "done",
    "have", "has", "had", "i", "me", "my", "mine", "we", "us", "our", "ours", "you", "your",
    "yours", "he", "him", "his", "she", "her", "hers", "it", "its", "they", "them", "their",
    "theirs", "this", "that", "these", "those", "what", "which", "who", "whom", "whose", "where",
    "when", "why", "how", "there", "here", "not", "no", "yes", "all", "any", "some", "each",
    "every", "both", "too", "very", "just", "also", "can", "could", "will", "would", "shall",
    "should", "may", "might", "must", "let", "lets", "oh", "okay", "ok", "please", "than", "now",
    "again", "still", "only", "own", "same", "other", "another", "such", "more", "most", "much",
    "many", "few", "one", "get", "got", "go", "going", "went", "said", "say", "tell", "told",
    "know", "dont", "im", "ive", "youre", "thats", "whats", "didnt", "doesnt", "isnt", "wasnt",
    "cant", "wont", "ill", "id", "theyre", "hasnt", "havent", "theres", "heres", "whos", "its",
    "after", "before", "while", "because", "like", "really", "well", "way", "yet", "ever", "never",
    "something", "anything", "nothing", "please", "thanks", "thank", "hi", "hello", "hey",
];

/// Lowercased words: letters and digits; a possessive "'s" is removed and
/// other apostrophes are dropped inside a word; every other character is a
/// separator.
fn words(text: &str) -> Vec<String> {
    fn finish(current: &mut String, out: &mut Vec<String>) {
        let mut word = std::mem::take(current);
        if word.ends_with("'s") {
            word.truncate(word.len() - 2);
        }
        let word: String = word.chars().filter(|&c| c != '\'').collect();
        if !word.is_empty() {
            out.push(word);
        }
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\'' || ch == '\u{2019}' {
            if !current.is_empty() {
                current.push('\'');
            }
        } else if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else {
            finish(&mut current, &mut out);
        }
    }
    finish(&mut current, &mut out);
    out
}

/// A light plural/possessive stem, applied to both sides of every comparison.
fn stem(word: &str) -> String {
    let n = word.len();
    if !word.is_ascii() {
        return word.to_owned();
    }
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..n - 3]);
    }
    if n > 4
        && ["ches", "shes", "sses", "xes", "zes", "oes"]
            .iter()
            .any(|s| word.ends_with(s))
    {
        return word[..n - 2].to_owned();
    }
    if n > 3
        && word.ends_with('s')
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is")
    {
        return word[..n - 1].to_owned();
    }
    word.to_owned()
}

fn is_stop(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

/// Stems of the content (non-stopword, longer than one character) words.
fn content_stems(text: &str) -> BTreeSet<String> {
    words(text)
        .into_iter()
        .filter(|w| w.chars().count() > 1 && !is_stop(w))
        .map(|w| stem(&w))
        .collect()
}

fn normalized(text: &str) -> String {
    words(text).join(" ")
}

fn grams(text: &str) -> Vec<String> {
    let w = words(text);
    if w.len() < NGRAM {
        return Vec::new();
    }
    w.windows(NGRAM).map(|g| g.join(" ")).collect()
}

fn cap(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Panels.

struct Panel {
    /// (path, sha256) of every file read.
    files: Vec<(String, String)>,
    /// Normalised whole strings.
    turns: HashSet<String>,
    /// Word 6-grams of every string.
    grams: HashSet<String>,
    /// Content stems of every string and check field.
    content: BTreeSet<String>,
    /// Content stems of the recall rows' `terms` and `forbid` columns: these
    /// appear nowhere in the generated text.
    strict: BTreeSet<String>,
    /// Content stems of the recall rows' `keys` column.
    keys: BTreeSet<String>,
    strings: usize,
}

impl Panel {
    fn add_string(&mut self, text: &str) {
        let norm = normalized(text);
        if norm.is_empty() {
            return;
        }
        self.strings += 1;
        self.turns.insert(norm);
        for g in grams(text) {
            self.grams.insert(g);
        }
        self.content.extend(content_stems(text));
    }

    fn load(dirs: &[PathBuf]) -> Result<Self> {
        let mut panel = Panel {
            files: Vec::new(),
            turns: HashSet::new(),
            grams: HashSet::new(),
            content: BTreeSet::new(),
            strict: BTreeSet::new(),
            keys: BTreeSet::new(),
            strings: 0,
        };
        for dir in dirs {
            let mut entries: Vec<PathBuf> = fs::read_dir(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    matches!(
                        p.extension().and_then(|x| x.to_str()),
                        Some("json") | Some("tsv")
                    )
                })
                .collect();
            entries.sort();
            for path in entries {
                let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let sha = uor_r4_training::sha256_file(&path).map_err(|e| e.to_string())?;
                panel.files.push((path.display().to_string(), sha));
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_owned();
                if name.ends_with(".json") {
                    let value: Value = serde_json::from_slice(&bytes)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    let mut strings = Vec::new();
                    collect_strings(&value, &mut strings);
                    for s in strings {
                        panel.add_string(&s);
                    }
                } else {
                    let text =
                        String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                    let checks = name.contains("checks");
                    for line in text.lines() {
                        if line.starts_with('#') || line.trim().is_empty() {
                            continue;
                        }
                        let fields: Vec<&str> = line.split('\t').collect();
                        for (column, field) in fields.iter().enumerate().skip(1) {
                            if checks && column < 3 {
                                continue; // kind, history
                            }
                            for phrase in field.split('|') {
                                if phrase.trim() != "-" {
                                    panel.add_string(phrase);
                                }
                            }
                        }
                        if checks && fields.get(2).map(|h| h.trim()) == Some("recall") {
                            for column in [3usize, 4] {
                                if let Some(field) = fields.get(column) {
                                    for phrase in field.split('|') {
                                        panel.strict.extend(content_stems(phrase));
                                    }
                                }
                            }
                            if let Some(field) = fields.get(5) {
                                for phrase in field.split('|') {
                                    panel.keys.extend(content_stems(phrase));
                                }
                            }
                        }
                    }
                }
            }
        }
        if panel.files.is_empty() {
            return Err("no panel files (*.json, *.tsv) were found under panels=".into());
        }
        Ok(panel)
    }

    /// Whether a text shares a stem with the panels' content words.
    fn shares_content(&self, text: &str) -> bool {
        content_stems(text).iter().any(|s| self.content.contains(s))
    }

    fn has_strict(&self, text: &str) -> bool {
        content_stems(text).iter().any(|s| self.strict.contains(s))
    }

    /// Whether a generated turn leaks: equal to a panel string, sharing a
    /// 6-gram with one, or holding a recall-row value word.
    fn turn_leak(&self, text: &str) -> Option<&'static str> {
        if self.turns.contains(&normalized(text)) {
            return Some("panel_turn");
        }
        if grams(text).iter().any(|g| self.grams.contains(g)) {
            return Some("panel_6gram");
        }
        if self.has_strict(text) {
            return Some("panel_recall_value_word");
        }
        None
    }
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| collect_strings(v, out)),
        Value::Object(map) => {
            for (key, v) in map {
                if key != "id" && key != "category" {
                    collect_strings(v, out);
                }
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Pools.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Vc {
    PetName,
    PersonName,
    Food,
    Dish,
    Drink,
    Object,
    Age,
    Job,
    Place,
    Street,
    Instrument,
    Spot,
    Color,
    Time,
    Date,
    Plant,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kc {
    Pets,
    People,
    Roles,
    Items,
    Paintables,
    Appointments,
    Events,
    Beds,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Art {
    None,
    Indef,
    The,
}

#[rustfmt::skip]
const PERSON_NAMES: &[&str] = &[
    "Aiden", "Amara", "Anika", "Arjun", "Astrid", "Bastian", "Beatrix", "Bodhi", "Calla", "Cedric",
    "Celeste", "Dario", "Delphine", "Desmond", "Elio", "Elodie", "Emeka", "Esme", "Ezra", "Farah",
    "Felix", "Fiona", "Gideon", "Greta", "Hamish", "Hana", "Idris", "Ines", "Isak", "Jago",
    "Kaito", "Keziah", "Lars", "Leilani", "Linus", "Lorcan", "Mabel", "Marisol", "Matteo",
    "Mireille", "Nadia", "Nico", "Niamh", "Odette", "Oren", "Orla", "Otis", "Paloma", "Quentin",
    "Rafferty", "Rhea", "Rowan", "Sabine", "Selma", "Silas", "Soraya", "Stellan", "Tamsin",
    "Teodor", "Thea", "Tobias", "Ulla", "Valentina", "Vikram", "Wren", "Xavier", "Yara", "Yusuf",
    "Zelda", "Zubair", "Anouk", "Bram", "Casimir", "Dagny", "Emrys", "Fenna", "Gulnara", "Henrik",
    "Ilse", "Joaquin", "Kalinda", "Leopold", "Maelis", "Nuno", "Oksana", "Pilar", "Ragnar",
    "Signe", "Tariq", "Ugo", "Vesna", "Wilhelmina", "Ximena", "Yoshiro", "Zainab", "Akosua",
    "Bilal", "Chiara", "Dmitri", "Eamon", "Freya", "Gwendolyn", "Hugo", "Imogen", "Jovan",
    "Katarina", "Lucius", "Magnus", "Nell", "Osric", "Petra", "Rosamund", "Sven", "Tova",
    "Umberto", "Viggo", "Winifred", "Yannick", "Zora", "Ayesha", "Benedikt", "Clementine",
    "Dorian", "Elspeth", "Florian", "Ingrid", "Jasper", "Kenji", "Lavinia", "Marguerite",
    "Nikolai", "Ottilie", "Rufus", "Saoirse", "Thaddeus", "Ursula", "Vivienne", "Wolfgang",
    "Adaeze", "Bjorn", "Cosima", "Dov", "Eilidh", "Faisal", "Gaspard", "Halima", "Ignatius",
    "Jessamy", "Kwame", "Liesel", "Mateus", "Nkechi", "Obadiah", "Priyanka", "Radu", "Sunniva",
    "Thandiwe", "Uriel", "Vashti", "Wendell", "Xiomara", "Yevgenia", "Zephyr",
];

#[rustfmt::skip]
const PET_NAMES: &[&str] = &[
    "Noodle", "Sprocket", "Bramble", "Pudding", "Clover", "Marbles", "Gizmo", "Tofu", "Nugget",
    "Ziggy", "Mochi", "Paprika", "Domino", "Truffle", "Bandit", "Comet", "Fizz", "Gumdrop",
    "Hopscotch", "Jellybean", "Lentil", "Nimbus", "Pistachio", "Rascal", "Saffron", "Tinsel",
    "Velvet", "Yeti", "Zigzag", "Acorn", "Button", "Cinnamon", "Dumpling", "Fudge", "Inky",
    "Kipper", "Licorice", "Maple", "Nutmeg", "Rocket", "Sesame", "Toffee", "Crumpet", "Sprout",
    "Scout", "Tango", "Rumble", "Bubbles", "Cricket", "Figaro", "Goose", "Hobbes", "Igloo", "Jinx",
    "Marshmallow", "Nacho", "Quill", "Ripple", "Thimble", "Waldo", "Yoyo", "Biscotti", "Doodle",
    "Echo", "Fable", "Gnocchi", "Jalapeno", "Kazoo", "Lollipop", "Meatball", "Napkin", "Pretzel",
    "Quasar", "Raisin", "Squiggle", "Turnip", "Wobble", "Zucchini",
];

#[rustfmt::skip]
const FOODS: &[&str] = &[
    "figs", "leeks", "kale", "apricots", "radishes", "quinces", "lentils", "chestnuts", "cherries",
    "papayas", "turnips", "parsnips", "artichokes", "asparagus", "beets", "cabbages", "celery",
    "cucumbers", "eggplants", "fennel", "garlic", "grapefruit", "guavas", "kiwis", "lemons",
    "limes", "lychees", "melons", "mushrooms", "nectarines", "okra", "olives", "onions", "peaches",
    "pineapples", "pomegranates", "potatoes", "pumpkins", "raspberries", "rhubarb", "shallots",
    "spinach", "squash", "strawberries", "tangerines", "walnuts", "almonds", "hazelnuts", "pecans",
    "cashews", "peanuts", "prunes", "raisins", "blueberries", "blackberries", "cranberries",
    "gooseberries", "clementines", "persimmons", "plantains", "yams", "zucchini", "bagels",
    "croissants", "pretzels", "scones", "tortillas", "cheddar", "brie", "mozzarella", "feta",
    "salami", "sardines", "salmon", "shrimp", "tofu", "honey", "maple syrup", "oat milk", "rice",
    "couscous", "quinoa", "noodles", "samosas", "hummus", "sauerkraut", "kimchi", "granola",
    "pistachios", "dried mangoes", "sweet potatoes", "brussels sprouts", "watercress", "kohlrabi",
    "endives", "jackfruit", "dragon fruit",
];

#[rustfmt::skip]
const DISHES: &[&str] = &[
    "lasagna", "paella", "ramen", "risotto", "curry", "tacos", "sushi", "goulash", "moussaka",
    "pierogi", "biryani", "gnocchi", "chowder", "enchiladas", "jollof rice", "pad thai",
    "shakshuka", "borscht", "gumbo", "tagine", "ratatouille", "pho", "burritos", "quesadillas",
    "crepes", "dumpling soup", "fish stew", "mac and cheese", "chili", "kebabs", "falafel",
    "pozole", "laksa", "bibimbap", "carbonara", "pesto pasta", "fried rice", "lentil soup",
    "stuffed peppers", "shepherd's pie", "fajitas", "katsu curry", "mapo tofu", "spanakopita",
    "jambalaya", "ceviche", "lamb stew", "minestrone", "dal", "tamales",
];

#[rustfmt::skip]
const DRINKS: &[&str] = &[
    "lemonade", "cocoa", "espresso", "cider", "kefir", "sparkling water", "ginger ale", "chai",
    "horchata", "kombucha", "root beer", "cold brew", "matcha", "hibiscus cooler",
    "elderflower soda", "cream soda", "mango lassi", "ayran", "agua fresca", "cappuccino",
    "cortado", "mocha", "lemon soda", "iced latte", "barley water", "malted milk", "tonic water",
    "vanilla shake", "grape soda", "cherry cola",
];

#[rustfmt::skip]
const OBJECTS: &[&str] = &[
    "lantern", "umbrella", "telescope", "kettle", "compass", "harmonica", "jigsaw puzzle",
    "sketchbook", "fountain pen", "skateboard", "hammock", "wristwatch", "camera", "globe",
    "chess set", "flashlight", "candle", "blanket", "mug", "wallet", "tripod", "microscope",
    "tambourine", "snow globe", "picture frame", "toolbox", "thermos", "cookbook", "houseplant",
    "bird feeder", "sleeping bag", "yoga mat", "pair of binoculars", "magnifying glass",
    "music box", "pocket knife", "water pistol", "paint set", "robot kit", "stopwatch",
    "calculator", "beanie", "pair of mittens", "puzzle box", "hourglass", "easel", "lamp", "rug",
    "vase", "clock", "kaleidoscope", "terrarium", "pinball game", "record player", "headlamp",
    "pencil case", "jewelry box", "rain poncho", "phone stand", "desk fan",
];

#[rustfmt::skip]
const JOBS: &[&str] = &[
    "architect", "pharmacist", "carpenter", "paramedic", "librarian", "welder", "pilot", "florist",
    "plumber", "electrician", "dentist", "nurse", "accountant", "journalist", "translator",
    "beekeeper", "mechanic", "veterinarian", "surveyor", "cartographer", "locksmith", "baker",
    "tailor", "potter", "sculptor", "geologist", "botanist", "chemist", "firefighter", "zookeeper",
    "lifeguard", "barista", "park ranger", "glassblower", "jeweler", "upholsterer", "optician",
    "physiotherapist", "radiologist", "lawyer", "economist", "actuary", "statistician",
    "game designer", "animator", "sound engineer", "stage manager", "choreographer", "midwife",
    "social worker", "auctioneer", "blacksmith", "cobbler", "sommelier", "astronomer",
    "marine biologist", "archivist", "curator", "data analyst", "court reporter",
];

#[rustfmt::skip]
const PLACES: &[&str] = &[
    "Lisbon", "Oslo", "Nairobi", "Kyoto", "Montreal", "Lima", "Reykjavik", "Hanoi", "Tbilisi",
    "Marrakesh", "Edinburgh", "Valparaiso", "Cusco", "Dubrovnik", "Krakow", "Seville", "Porto",
    "Bergen", "Tallinn", "Riga", "Vilnius", "Ljubljana", "Zagreb", "Sofia", "Bucharest",
    "Budapest", "Vienna", "Prague", "Salzburg", "Munich", "Hamburg", "Antwerp", "Ghent", "Bruges",
    "Lyon", "Marseille", "Bordeaux", "Naples", "Florence", "Bologna", "Palermo", "Athens",
    "Thessaloniki", "Izmir", "Beirut", "Amman", "Muscat", "Doha", "Mumbai", "Jaipur", "Kathmandu",
    "Colombo", "Dhaka", "Chiang Mai", "Penang", "Manila", "Cebu", "Jakarta", "Perth", "Hobart",
    "Auckland", "Wellington", "Quebec", "Halifax", "Vancouver", "Seattle", "Denver", "Austin",
    "Memphis", "Savannah", "Boston", "Chicago", "Toronto", "Havana", "Bogota", "Quito",
    "Montevideo", "Asuncion", "La Paz", "Accra", "Dakar", "Lagos", "Kigali", "Addis Ababa",
    "Zanzibar", "Cape Town", "Windhoek", "Casablanca", "Tunis", "Cairo", "Seoul", "Busan",
    "Taipei", "Osaka", "Sapporo", "Ulaanbaatar", "Almaty", "Tashkent", "Baku", "Yerevan",
];

const TOWN_PREFIXES: &[&str] = &[
    "Ash", "Bram", "Cold", "Elder", "Fern", "Glen", "Kings", "Lark", "Mill", "North", "Oak",
    "Pine", "Raven", "Stone", "Thorn", "Willow", "Briar", "Dun", "Hart", "Marl", "Pen", "Sel",
    "Tarn", "Wick", "Holm", "Lang", "Cray", "Bel", "Gal", "Ross",
];
const TOWN_SUFFIXES: &[&str] = &[
    "ford", "brook", "field", "wick", "haven", "mere", "dale", "stead", "ton", "bury", "combe",
    "holt", "ridge", "worth", "moor",
];
const STREET_WORDS: &[&str] = &[
    "Hawthorn", "Juniper", "Linden", "Alder", "Larch", "Sycamore", "Magnolia", "Heron", "Kestrel",
    "Plover", "Osprey", "Curlew", "Mulberry", "Chestnut", "Quarry", "Tanner", "Cooper", "Wheeler",
    "Fletcher", "Orchard", "Meadow", "Harbor", "Lantern", "Granary", "Foundry", "Weaver", "Cobble",
    "Saddler", "Thistle", "Bracken", "Sorrel", "Yarrow", "Elm", "Poplar", "Hazel", "Laurel",
    "Cedar", "Birch", "Aspen", "Rowan",
];
const STREET_TYPES: &[&str] = &[
    "Street", "Lane", "Road", "Avenue", "Close", "Terrace", "Crescent", "Drive",
];

#[rustfmt::skip]
const INSTRUMENTS: &[&str] = &[
    "cello", "oboe", "bassoon", "clarinet", "trombone", "banjo", "mandolin", "harp", "accordion",
    "bagpipes", "viola", "flute", "xylophone", "marimba", "tuba", "sitar", "ukulele", "saxophone",
    "double bass", "harpsichord", "bongos", "steel drum", "theremin", "trumpet", "violin",
    "french horn", "glockenspiel", "lute", "dulcimer", "euphonium",
];

#[rustfmt::skip]
const SPOTS: &[&str] = &[
    "in the hallway closet", "in the garden shed", "under the stairs", "in the attic",
    "in the cellar", "behind the radiator", "in the sewing tin", "on the windowsill",
    "in the toolbox", "inside the piano bench", "in the mailbox", "on top of the fridge",
    "in the linen cupboard", "in the medicine cabinet", "on the mantelpiece", "in the shoebox",
    "in the glove box", "under the doormat", "in the umbrella stand", "in the garage",
    "behind the bookcase", "in the laundry room", "in the spice rack", "in the filing cabinet",
    "under the sink", "in the wardrobe", "in the junk tray", "in the craft cupboard",
    "inside the vase", "in the tool chest", "in the camping crate", "in the guest room",
    "in the pantry", "in the bathroom cabinet", "behind the sofa cushions", "in the hall cupboard",
    "inside the old suitcase", "in the recycling bin", "on the porch step", "in the sideboard",
];

#[rustfmt::skip]
const COLORS: &[&str] = &[
    "teal", "maroon", "crimson", "lavender", "beige", "turquoise", "magenta", "olive", "navy",
    "amber", "coral", "ivory", "indigo", "scarlet", "mint", "peach", "mustard", "burgundy",
    "cobalt", "emerald", "ochre", "rust", "sage", "charcoal", "mauve", "lilac", "tan", "cyan",
    "khaki", "cream", "copper", "bronze", "periwinkle", "aquamarine", "vermilion", "chartreuse",
    "taupe", "fuchsia", "sapphire", "jade",
];

#[rustfmt::skip]
const PLANTS: &[&str] = &[
    "tulips", "basil", "ferns", "peonies", "daffodils", "marigolds", "rosemary", "thyme",
    "zinnias", "dahlias", "hostas", "lilies", "irises", "snapdragons", "foxgloves", "begonias",
    "geraniums", "chives", "strawberries", "pansies", "petunias", "orchids", "succulents",
    "bamboo", "hydrangeas", "lupins", "poppies", "asters", "oregano", "parsley", "dill",
    "cilantro", "lettuce", "radishes", "kale", "squash", "pumpkins", "heather", "crocuses",
    "camellias",
];

#[rustfmt::skip]
const MONTHS: &[&str] = &[
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

#[rustfmt::skip]
const PEOPLE_KEYS: &[&str] = &[
    "Grandpa", "Grandma", "Mom", "Dad", "my sister", "my brother", "my cousin", "my neighbor",
    "my roommate", "my boss", "my best friend", "my nephew", "my niece", "my godmother",
    "my stepdad", "my husband", "my wife", "my partner", "my son", "my daughter", "my grandson",
    "my granddaughter", "our babysitter", "my coworker", "my mentor", "my landlord",
    "my mother-in-law", "my stepsister", "my godfather", "my flatmate",
];

#[rustfmt::skip]
const ROLE_KEYS: &[&str] = &[
    "our new landlord", "the plumber", "my boss", "the coach", "my dentist", "the babysitter",
    "our neighbor", "the intern", "my piano teacher", "the tour guide", "the vet", "the mechanic",
    "my lab partner", "the head chef", "the bus driver", "my pen pal", "the lifeguard",
    "my driving instructor", "the pharmacist", "the night nurse", "the wedding planner",
    "the electrician", "the photographer", "the librarian", "the swim coach", "the dog walker",
    "the gardener", "my study buddy",
];

#[rustfmt::skip]
const PET_KEYS: &[&str] = &[
    "the turtle", "the goldfish", "the hamster", "the parrot", "the kitten", "the puppy",
    "the lizard", "the ferret", "the guinea pig", "the canary", "the gecko", "the pony",
    "the tortoise", "the cockatiel", "the iguana", "the snake", "the chinchilla", "the hedgehog",
    "the duckling", "the budgie", "the axolotl", "the goat", "the lamb", "the piglet",
    "the donkey", "the alpaca", "the hen", "the gerbil", "the cockatoo", "the newt",
];

#[rustfmt::skip]
const ITEM_KEYS: &[&str] = &[
    "the spare key", "my passport", "the phone charger", "the tape measure", "the flashlight",
    "the remote", "my wallet", "the stapler", "the screwdriver", "the extra lightbulb",
    "the parcel", "the receipt", "the library card", "the hammer", "the measuring jug",
    "the first aid kit", "the gift card", "the spare battery", "my sketchbook", "the padlock",
    "the hair dryer", "the bike pump", "the thermometer", "my headphones", "the glue gun",
];

#[rustfmt::skip]
const PAINTABLE_KEYS: &[&str] = &[
    "the fence", "the porch", "the garage door", "the front door", "the canoe", "the shed",
    "the mailbox", "the bathroom", "the hallway", "the bedroom", "the bookcase", "the dresser",
    "the garden bench", "the treehouse", "the bike", "the boat", "the van", "the gate",
    "the playroom", "the kitchen", "the shutters", "the bird house", "the attic", "the study",
];

#[rustfmt::skip]
const APPOINTMENT_KEYS: &[&str] = &[
    "my dentist appointment", "the yoga class", "the team meeting", "my haircut",
    "the pottery class", "my eye exam", "the vet appointment", "choir practice",
    "the climbing session", "my job interview", "the parent meeting", "the plumber visit",
    "the conference call", "the piano recital", "the chess club", "the swim lesson",
    "my physio session", "the carpool pickup", "the movie", "the bake sale", "the fire drill",
    "the flu shot", "the guitar lesson", "the budget review", "the volunteer shift",
];

#[rustfmt::skip]
const EVENT_KEYS: &[&str] = &[
    "the wedding", "the school play", "the family reunion", "the garage sale", "the science fair",
    "the marathon", "the housewarming", "the recital", "the graduation", "the camping trip",
    "the art show", "the open house", "the charity walk", "the street festival",
    "the retirement party", "the anniversary trip", "the move", "the surgery", "the job fair",
    "the cousins' visit", "the quiz night", "the regatta",
];

#[rustfmt::skip]
const BED_KEYS: &[&str] = &[
    "the front bed", "the window box", "the back corner", "the raised bed", "the big planter",
    "the herb patch", "the strip by the fence", "the shady corner", "the rockery",
    "the side border", "the greenhouse", "the wheelbarrow", "the balcony box",
    "the courtyard planter", "the trough", "the allotment",
];

// ---------------------------------------------------------------------------
// Frames: a relation between a key class and a value class, with templates.
//
// Placeholders: {k} {K} user-voice key (capitalised), {kp} {Kp} its possessive;
// {ka} {KA} assistant-voice key, {kap} {KAP} its possessive; {v} {V} value;
// {av} {AV} value with its article; {old} {aold} superseded value. Pair
// templates number the two keys and values 1 and 2.

struct Frame {
    id: &'static str,
    keys: Kc,
    values: Vc,
    art: Art,
    /// "You said {av}." reads naturally for this value class.
    said: bool,
    pair: &'static [&'static str],
    single: &'static [&'static str],
    update: &'static [&'static str],
    ask: &'static [&'static str],
    answer: &'static [&'static str],
    reverse_ask: &'static [&'static str],
    reverse_answer: &'static [&'static str],
    suggest_request: &'static [&'static str],
    suggest_reply: &'static [&'static str],
}

const FRAMES: &[Frame] = &[
    Frame {
        id: "pet_names",
        keys: Kc::Pets,
        values: Vc::PetName,
        art: Art::None,
        said: true,
        pair: &[
            "We have a couple of new pets: {k1} is called {v1} and {k2} is called {v2}.",
            "Our new pets finally have names. {K1} is {v1}, and {k2} is {v2}.",
            "We brought home {k1} and {k2} this week, and we named them {v1} and {v2}, in that order.",
            "The kids named {k1} {v1} and {k2} {v2}.",
        ],
        single: &["{K} is called {v}.", "We named {k} {v}.", "Oh, and {k} is named {v}."],
        update: &[
            "Actually, we renamed {k}. It's {v} now, not {old}.",
            "Small change: {k} is called {v} now instead of {old}.",
            "We changed our minds about {k} and went with {v}.",
        ],
        ask: &[
            "What's {k} called?",
            "What name did we give {k}?",
            "What did we name {k}?",
            "Do you remember what {k} is called?",
        ],
        answer: &["{KA} is called {v}.", "{KA} is named {v}.", "You named {ka} {v}."],
        reverse_ask: &["Which pet is called {v}?", "Which of our pets is {v}?"],
        reverse_answer: &["{KA} is called {v}.", "{V} is {ka}."],
        suggest_request: &[
            "Can you help me name {k1} and {k2}?",
            "We need names for {k1} and {k2}. Any ideas?",
        ],
        suggest_reply: &[
            "How about {v1} for {ka1} and {v2} for {ka2}?",
            "You could call {ka1} {v1} and {ka2} {v2}.",
        ],
    },
    Frame {
        id: "purchases",
        keys: Kc::People,
        values: Vc::Food,
        art: Art::None,
        said: true,
        pair: &[
            "At the market, {k1} bought {v1} and {k2} bought {v2}.",
            "We went shopping today. {K1} picked up {v1}, and {k2} got {v2}.",
            "{K1} came home with {v1}, while {k2} bought {v2}.",
        ],
        single: &["{K} bought {v}.", "{K} picked up {v} at the store."],
        update: &[
            "Wait, I got that wrong. {K} actually bought {v}, not {old}.",
            "Correction: {k} bought {v}, not {old}.",
        ],
        ask: &[
            "What did {k} buy?",
            "What did {k} pick up at the store?",
            "What did {k} come home with?",
        ],
        answer: &["{KA} bought {v}.", "{KA} picked up {v}."],
        reverse_ask: &["Who bought {v}?", "Who picked up {v}?"],
        reverse_answer: &["{KA} bought {v}.", "{KA} did."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "favorite_dishes",
        keys: Kc::People,
        values: Vc::Dish,
        art: Art::None,
        said: true,
        pair: &[
            "{Kp1} favorite dish is {v1}, but {kp2} favorite is {v2}.",
            "{K1} loves {v1}, and {k2} would eat {v2} every night if they could.",
            "When we eat out, {k1} always wants {v1} and {k2} always wants {v2}.",
        ],
        single: &["{Kp} favorite dish is {v}.", "{K} really loves {v}."],
        update: &[
            "Actually, {kp} new favorite is {v}. {K} is tired of {old}.",
            "I was wrong before. {Kp} favorite dish is {v}, not {old}.",
        ],
        ask: &[
            "What's {kp} favorite dish?",
            "What does {k} love to eat?",
            "Which dish does {k} like best?",
        ],
        answer: &["{KAP} favorite dish is {v}.", "{KA} loves {v}."],
        reverse_ask: &["Who loves {v}?", "Whose favorite dish is {v}?"],
        reverse_answer: &["{KA} loves {v}.", "That's {kap} favorite."],
        suggest_request: &[
            "What should I cook for {k1} and {k2}?",
            "I'm cooking for {k1} and {k2}. What should I make?",
        ],
        suggest_reply: &[
            "Maybe {v1} for {ka1} and {v2} for {ka2}.",
            "You could make {v1} for {ka1} and {v2} for {ka2}.",
        ],
    },
    Frame {
        id: "drink_orders",
        keys: Kc::People,
        values: Vc::Drink,
        art: Art::None,
        said: true,
        pair: &[
            "At the café, {k1} ordered {v1} and {k2} had {v2}.",
            "We stopped for drinks. {K1} got {v1}, and {k2} ordered {v2}.",
            "{K1} asked for {v1}, and {k2} went with {v2}.",
        ],
        single: &["{K} ordered {v}.", "{K} had {v} at the café."],
        update: &["Oops, I mixed that up. {K} ordered {v}, not {old}."],
        ask: &[
            "What did {k} order?",
            "What did {k} get to drink?",
            "Which drink did {k} have?",
        ],
        answer: &["{KA} ordered {v}.", "{KA} had {v}."],
        reverse_ask: &["Who ordered {v}?", "Who had {v}?"],
        reverse_answer: &["{KA} ordered {v}.", "{KA} did."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "gifts",
        keys: Kc::People,
        values: Vc::Object,
        art: Art::Indef,
        said: true,
        pair: &[
            "For the holidays I got {k1} {av1} and {k2} {av2}.",
            "I finished my gift shopping: {av1} for {k1} and {av2} for {k2}.",
            "I wrapped the presents today. {K1} is getting {av1}, and {k2} is getting {av2}.",
        ],
        single: &["I got {k} {av}.", "{K} is getting {av} from me."],
        update: &[
            "Change of plans: I'm giving {k} {av} instead of {aold}.",
            "I exchanged {kp} gift. {K} is getting {av} now, not {aold}.",
        ],
        ask: &[
            "What did I get {k}?",
            "What gift is {k} getting?",
            "What am I giving {k}?",
        ],
        answer: &["You got {ka} {av}.", "{KA} is getting {av}."],
        reverse_ask: &["Who is getting {av}?", "Who did I get {av} for?"],
        reverse_answer: &["{KA} is getting {av}.", "That one is for {ka}."],
        suggest_request: &[
            "What should I get {k1} and {k2}?",
            "Any gift ideas for {k1} and {k2}?",
        ],
        suggest_reply: &[
            "Maybe {av1} for {ka1} and {av2} for {ka2}.",
            "How about {av1} for {ka1} and {av2} for {ka2}?",
        ],
    },
    Frame {
        id: "ages",
        keys: Kc::People,
        values: Vc::Age,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} just turned {v1}, and {k2} is {v2}.",
            "{K1} is {v1} years old and {k2} is {v2}.",
            "We had birthdays this month: {k1} turned {v1} and {k2} turned {v2}.",
        ],
        single: &["{K} is {v} years old.", "{K} just turned {v}."],
        update: &["Sorry, I said that wrong. {K} is {v}, not {old}."],
        ask: &[
            "How old is {k}?",
            "What age did {k} just turn?",
            "How old did I say {k} is?",
        ],
        answer: &["{KA} is {v}.", "{KA} is {v} years old."],
        reverse_ask: &["Who is {v} years old?", "Who just turned {v}?"],
        reverse_answer: &["{KA} is {v}.", "{KA} is {v} years old."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "jobs",
        keys: Kc::People,
        values: Vc::Job,
        art: Art::Indef,
        said: false,
        pair: &[
            "{K1} works as {av1}, and {k2} is {av2}.",
            "{K1} just started a job as {av1}. {K2} has been {av2} for years.",
            "In our family, {k1} is {av1} and {k2} is {av2}.",
        ],
        single: &["{K} works as {av}.", "{K} is {av}."],
        update: &["Actually, {k} switched careers and is {av} now, not {aold}."],
        ask: &[
            "What does {k} do for work?",
            "What is {kp} job?",
            "What does {k} do for a living?",
        ],
        answer: &["{KA} is {av}.", "{KA} works as {av}."],
        reverse_ask: &["Who works as {av}?", "Which of them is {av}?"],
        reverse_answer: &["{KA} is {av}.", "{KA} works as {av}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "travel",
        keys: Kc::People,
        values: Vc::Place,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is flying to {v1} next month, and {k2} is heading to {v2}.",
            "Big travel plans this year: {k1} is going to {v1} and {k2} is off to {v2}.",
            "{K1} booked a trip to {v1}, while {k2} chose {v2}.",
        ],
        single: &["{K} is going to {v} next month.", "{K} booked a trip to {v}."],
        update: &["{Kp} plans changed. {K} is going to {v} now instead of {old}."],
        ask: &[
            "Where is {k} going?",
            "Where is {k} traveling to?",
            "Which place is {k} visiting?",
        ],
        answer: &["{KA} is going to {v}.", "{KA} is traveling to {v}."],
        reverse_ask: &["Who is going to {v}?", "Who booked the trip to {v}?"],
        reverse_answer: &["{KA} is going to {v}.", "{KA} is."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "addresses",
        keys: Kc::People,
        values: Vc::Street,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} lives on {v1}, and {k2} lives on {v2}.",
            "{K1} just moved to {v1}. {K2} is still on {v2}.",
            "If you need to visit, {k1} is on {v1} and {k2} is on {v2}.",
        ],
        single: &["{K} lives on {v}.", "{K} moved to {v}."],
        update: &["{K} moved last week. The new place is on {v}, not {old}."],
        ask: &[
            "What's {kp} address?",
            "Where does {k} live?",
            "Where is {kp} place?",
        ],
        answer: &["{KA} lives on {v}.", "{KA} is on {v}."],
        reverse_ask: &["Who lives on {v}?"],
        reverse_answer: &["{KA} lives on {v}.", "{KA} does."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "instruments",
        keys: Kc::People,
        values: Vc::Instrument,
        art: Art::The,
        said: true,
        pair: &[
            "{K1} plays the {v1}, and {k2} plays the {v2}.",
            "In the band, {k1} is on the {v1} and {k2} is on the {v2}.",
            "{K1} practices the {v1} every evening, while {k2} practices the {v2}.",
        ],
        single: &["{K} plays the {v}."],
        update: &["I got that mixed up. {K} plays the {v}, not the {old}."],
        ask: &[
            "Which instrument does {k} play?",
            "What instrument does {k} play?",
            "What does {k} practice every evening?",
        ],
        answer: &["{KA} plays the {v}."],
        reverse_ask: &["Who plays the {v}?"],
        reverse_answer: &["{KA} plays the {v}.", "{KA} does."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "role_names",
        keys: Kc::Roles,
        values: Vc::PersonName,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is called {v1}, and {k2} is called {v2}.",
            "We finally met everyone. {Kp1} name is {v1}, and {kp2} name is {v2}.",
            "Quick update: {k1} is {v1} and {k2} is {v2}.",
        ],
        single: &["{Kp} name is {v}.", "{K} is called {v}."],
        update: &["I misheard before. {Kp} name is {v}, not {old}."],
        ask: &[
            "What's {kp} name?",
            "What is {k} called?",
            "Do you remember {kp} name?",
        ],
        answer: &["{KAP} name is {v}.", "{KA} is called {v}."],
        reverse_ask: &["Who is {v}?", "Which one is {v}?"],
        reverse_answer: &["{V} is {ka}.", "{KA} is called {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "storage",
        keys: Kc::Items,
        values: Vc::Spot,
        art: Art::None,
        said: false,
        pair: &[
            "I put {k1} {v1} and {k2} {v2}.",
            "Before I forget: {k1} is {v1}, and {k2} is {v2}.",
            "I tidied up. {K1} is now {v1}, and {k2} is {v2}.",
        ],
        single: &["I put {k} {v}.", "{K} is {v}."],
        update: &["Actually, I moved {k}. It's {v} now, not {old}."],
        ask: &[
            "Where did I put {k}?",
            "Where is {k}?",
            "Where did I leave {k}?",
        ],
        answer: &["You put {ka} {v}.", "{KA} is {v}."],
        reverse_ask: &["What did I put {v}?", "What's {v}?"],
        reverse_answer: &["You put {ka} {v}.", "{KA} is {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "paint_colors",
        keys: Kc::Paintables,
        values: Vc::Color,
        art: Art::None,
        said: true,
        pair: &[
            "We painted {k1} {v1} and {k2} {v2}.",
            "The painters finished. {K1} is {v1} now, and {k2} is {v2}.",
            "We picked colors at last: {v1} for {k1} and {v2} for {k2}.",
        ],
        single: &["We painted {k} {v}.", "{K} is {v} now."],
        update: &["We repainted {k}. It's {v} now, not {old}."],
        ask: &[
            "What color did we paint {k}?",
            "What color is {k} now?",
            "Which color did we pick for {k}?",
        ],
        answer: &["You painted {ka} {v}.", "{KA} is {v}."],
        reverse_ask: &["What did we paint {v}?", "Which one is {v}?"],
        reverse_answer: &["You painted {ka} {v}.", "{KA} is {v}."],
        suggest_request: &["What colors should we paint {k1} and {k2}?"],
        suggest_reply: &[
            "Try {v1} for {ka1} and {v2} for {ka2}.",
            "I'd go with {v1} for {ka1} and {v2} for {ka2}.",
        ],
    },
    Frame {
        id: "appointment_times",
        keys: Kc::Appointments,
        values: Vc::Time,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is at {v1}, and {k2} is at {v2}.",
            "My calendar is busy: {k1} at {v1} and {k2} at {v2}.",
            "Don't let me forget: {k1} starts at {v1}, and {k2} starts at {v2}.",
        ],
        single: &["{K} is at {v}.", "{K} starts at {v}."],
        update: &["{K} got moved. It's at {v} now, not {old}."],
        ask: &[
            "What time is {k}?",
            "When does {k} start?",
            "What time did I say {k} is?",
        ],
        answer: &["{KA} is at {v}.", "{KA} starts at {v}."],
        reverse_ask: &["What's happening at {v}?", "What do I have at {v}?"],
        reverse_answer: &["{KA} is at {v}.", "You have {ka} at {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "event_dates",
        keys: Kc::Events,
        values: Vc::Date,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is on {v1}, and {k2} is on {v2}.",
            "We set the dates: {k1} on {v1} and {k2} on {v2}.",
            "Mark your calendar: {k1} is {v1} and {k2} is {v2}.",
        ],
        single: &["{K} is on {v}."],
        update: &["{K} was rescheduled to {v}. It's no longer {old}."],
        ask: &["When is {k}?", "What date is {k}?", "Which date did we set for {k}?"],
        answer: &["{KA} is on {v}."],
        reverse_ask: &["What's happening on {v}?"],
        reverse_answer: &["{KA} is on {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "garden_beds",
        keys: Kc::Beds,
        values: Vc::Plant,
        art: Art::None,
        said: true,
        pair: &[
            "We planted {v1} in {k1} and {v2} in {k2}.",
            "The garden is coming along: {v1} in {k1} and {v2} in {k2}.",
            "This spring {k1} got {v1}, and {k2} got {v2}.",
        ],
        single: &["We planted {v} in {k}."],
        update: &["We dug up {k} and planted {v} there instead of {old}."],
        ask: &[
            "What did we plant in {k}?",
            "What's growing in {k}?",
            "What went into {k}?",
        ],
        answer: &["You planted {v} in {ka}.", "{KA} has {v}."],
        reverse_ask: &["Where did we plant {v}?", "Which spot has {v}?"],
        reverse_answer: &["You planted {v} in {ka}.", "{KA} has {v}."],
        suggest_request: &["What should we plant in {k1} and {k2}?"],
        suggest_reply: &["Try {v1} in {ka1} and {v2} in {ka2}."],
    },
];

/// Distractor exchanges: a user turn and its possible replies.
const FILLERS: &[(&str, &[&str])] = &[
    ("It has been raining all morning here.", &["Rainy mornings can be cozy. I hope you can stay dry.", "That sounds gloomy. Maybe it will clear up later."]),
    ("I just got back from a long walk.", &["That sounds refreshing. Did you enjoy the fresh air?", "Nice! Walking is a great way to clear your head."]),
    ("I'm feeling a little tired today.", &["I'm sorry to hear that. Maybe a short rest would help.", "That happens. Try to take it easy if you can."]),
    ("What is a good way to relax after work?", &["A quiet walk, some music, or a warm shower can help you unwind."]),
    ("I think I need a haircut soon.", &["A fresh haircut can feel great. Do you have a style in mind?"]),
    ("What's the difference between weather and climate?", &["Weather is what happens day to day, while climate is the pattern over many years."]),
    ("Do you like music?", &["I enjoy talking about music. What kind do you like?"]),
    ("I'm trying to drink more water.", &["That's a healthy habit. Keeping a bottle nearby can help."]),
    ("The traffic was terrible this morning.", &["That sounds frustrating. I hope the rest of your day goes more smoothly."]),
    ("How do plants make their own sugar?", &["Plants use sunlight, water and air to make sugar in their leaves. This is called photosynthesis."]),
    ("I can't decide what to watch tonight.", &["Maybe pick something light and funny if you want to relax."]),
    ("Thanks for helping me earlier.", &["You're welcome. I'm glad I could help.", "Anytime! I'm happy to help."]),
    ("Why is the sky dark at night?", &["At night our side of the Earth faces away from the sun, so there is no sunlight to light up the sky."]),
    ("I started learning to knit.", &["That's a nice hobby. Start with something simple, like a dishcloth."]),
    ("It's so windy outside today.", &["Windy days can be wild. Hold on to anything light if you go out."]),
    ("I have a headache.", &["I'm sorry. Drinking some water and resting your eyes might help."]),
    ("Tell me something cheerful.", &["Here is something cheerful: somewhere right now, someone is laughing at a joke they just heard."]),
    ("Do you ever get bored?", &["I don't get bored the way people do, but I enjoy a good conversation."]),
    ("How many minutes are in an hour?", &["There are sixty minutes in an hour."]),
    ("What's another word for happy?", &["You could say glad, cheerful or joyful."]),
    ("I'm nervous about my presentation.", &["That's normal. Practicing out loud a few times can really help."]),
    ("We had a power cut last night.", &["That sounds inconvenient. I hope everything is working again."]),
    ("Can you suggest a word game?", &["Try taking turns naming words that start with the last letter of the previous word."]),
    ("What is the capital of Canada?", &["The capital of Canada is Ottawa."]),
    ("My phone battery keeps dying.", &["Try lowering the screen brightness and closing apps you aren't using."]),
    ("Is it better to study in the morning or at night?", &["It depends on the person. Try both and notice when you focus best."]),
    ("What should I do if I can't sleep?", &["Try a calm routine: dim lights, no screens, and slow breathing."]),
    ("The birds outside are so loud this morning.", &["It sounds like they are having a busy morning."]),
    ("I just made a big pot of soup.", &["That sounds warming. Soup is perfect for a chilly day."]),
    ("How do you spell necessary?", &["It is spelled n-e-c-e-s-s-a-r-y."]),
    ("I'm proud of myself for finishing that project.", &["You should be proud. Finishing a project takes effort."]),
    ("It's finally warm enough to open the windows.", &["That's a nice feeling. Enjoy the fresh breeze."]),
    ("Why do some trees drop their leaves?", &["Many trees shed their leaves in the cold season to save water and energy until spring."]),
    ("Can you help me feel less stressed?", &["Let's take a slow breath together. Breathe in, hold it for a moment, and let it out gently."]),
    ("I'm thinking about learning a new language.", &["That's exciting. Which language are you curious about?"]),
    ("What is the opposite of ancient?", &["The opposite of ancient is modern."]),
    ("How far away is the moon?", &["The moon is about 384,000 kilometers from Earth."]),
    ("I'm excited for the weekend.", &["Weekends are great. Do you have anything fun planned?"]),
    ("What's a polite way to say no?", &["You could say, 'Thank you for asking, but I won't be able to this time.'"]),
    ("Do you know any tongue twisters?", &["Try this one: she sells seashells by the seashore."]),
    ("I feel like the week is going by slowly.", &["Some weeks really drag. Hang in there."]),
    ("What makes thunder?", &["Thunder is the sound of air expanding very fast when lightning heats it."]),
    ("I organized my closet today.", &["Nice work. An organized closet makes mornings easier."]),
    ("How do I stay focused?", &["Work in short blocks, put your phone away, and take small breaks."]),
    ("I just learned how to whistle.", &["That's fun! Whistling takes some practice."]),
    ("It's really hot in here.", &["Opening a window or drinking something cool might help."]),
    ("What does curious mean?", &["Curious means wanting to know or learn about something."]),
    ("I watched the sunset from the hill.", &["That sounds peaceful. Sunsets from a hill are special."]),
    ("Let's talk about something else for a second.", &["Sure. What would you like to talk about?"]),
    ("That reminds me of something funny.", &["I'd love to hear it."]),
    ("What's the tallest mountain in the world?", &["Mount Everest is the tallest mountain above sea level."]),
    ("Do penguins live at the North Pole?", &["No, wild penguins live in the Southern Hemisphere, not at the North Pole."]),
    ("I'm making a shopping list.", &["Good idea. A list makes shopping quicker."]),
    ("How are things with you?", &["Things are fine on my end. How about you?"]),
    ("I finally fixed the squeaky hinge.", &["That must be a relief. Squeaky hinges are so annoying."]),
    ("What causes a rainbow?", &["Sunlight bends and splits inside raindrops, which spreads it into bands of color."]),
    ("I'm going to try a new recipe tonight.", &["Have fun with it! Trying new recipes is a good way to learn."]),
    ("My neighbor plays loud music late at night.", &["That sounds frustrating. A friendly chat might help."]),
    ("How long does it take to boil an egg?", &["For a soft egg, a few minutes; for a firm one, a little longer."]),
    ("I need a short break.", &["Good idea. Stretch, breathe and come back when you're ready."]),
    ("Is a tomato a fruit or a vegetable?", &["Botanically it is a fruit, but cooks usually treat it as a vegetable."]),
    ("What's your favorite season?", &["I don't have one, but many people love autumn for its cool air."]),
    ("I almost missed the bus this morning.", &["Glad you made it! Mornings can be hectic."]),
    ("Can you explain what gravity is?", &["Gravity is the pull that objects with mass have on each other. It keeps us on the ground."]),
    ("I'm a bit bored right now.", &["Maybe try a puzzle, a short walk, or a new song."]),
];

#[rustfmt::skip]
// More distractor exchanges: families with one user frame and fitting replies,
// and single question/answer pairs. `filler_bank` joins them with `FILLERS`.

#[rustfmt::skip]
const ACTIVITIES: &[&str] = &[
    "going hiking", "trying pottery", "visiting a museum", "baking bread", "going camping",
    "learning to juggle", "painting a mural", "going to a concert", "cleaning out the garage",
    "planting seeds", "going for a bike ride", "watching a documentary", "trying a new café",
    "writing letters to old friends", "doing a jigsaw puzzle", "visiting the zoo", "going fishing",
    "building a birdhouse", "practicing yoga", "going ice skating", "having a picnic",
    "sorting old photos", "learning some magic tricks", "stargazing", "rearranging the furniture",
    "going bowling", "visiting an aquarium", "making candles", "going to the library",
    "taking a cooking class", "volunteering at the animal shelter", "repotting my plants",
    "going kayaking", "watching the sunrise", "playing board games", "making a scrapbook",
    "going roller skating", "walking along the river", "learning to knit", "visiting a castle",
];
const ACTIVITY_USERS: &[&str] = &[
    "I'm thinking about {x} this weekend.",
    "I might try {x} later this month.",
];
const ACTIVITY_REPLIES: &[&str] = &[
    "That sounds like a nice way to spend some time.",
    "Nice plan! I hope you enjoy it.",
    "That sounds fun. Have a great time.",
];

#[rustfmt::skip]
const CHORES: &[&str] = &[
    "doing the laundry", "washing the dishes", "vacuuming the living room", "mopping the floors",
    "ironing my shirts", "cleaning the windows", "raking the leaves", "mowing the lawn",
    "folding the towels", "dusting the shelves", "scrubbing the bathtub", "taking out the recycling",
    "organizing the pantry", "paying the bills", "answering my emails", "sorting the mail",
    "wiping down the counters", "changing the bed sheets", "watering the plants",
    "cleaning out the fridge", "sweeping the porch", "decluttering the hallway", "washing the car",
    "defrosting the freezer", "fixing a leaky faucet", "replacing a light bulb", "packing for my trip",
    "unpacking the groceries", "sharpening the knives", "cleaning the oven", "tidying my desk",
    "filing my paperwork", "shoveling the snow", "polishing my shoes", "oiling the squeaky door",
];
const CHORE_REPLIES: &[&str] = &[
    "Nice work! It feels good to get that done.",
    "Well done. That's one less thing to worry about.",
    "Great job. You deserve a break now.",
];

#[rustfmt::skip]
const GOOD_FEELINGS: &[&str] = &[
    "cheerful", "relaxed", "energetic", "grateful", "hopeful", "calm", "content", "inspired",
    "optimistic", "excited", "refreshed", "confident", "peaceful", "motivated", "upbeat",
];
const GOOD_FEELING_REPLIES: &[&str] = &[
    "That's great to hear! What made you feel that way?",
    "I'm glad you're feeling {x}. Enjoy it.",
];
#[rustfmt::skip]
const BAD_FEELINGS: &[&str] = &[
    "anxious", "stressed", "lonely", "overwhelmed", "sad", "frustrated", "restless", "gloomy",
    "worried", "homesick", "grumpy", "discouraged", "drained", "nervous", "sleepy",
];
const BAD_FEELING_REPLIES: &[&str] = &[
    "I'm sorry you're feeling {x}. Do you want to talk about it?",
    "That sounds hard. Be gentle with yourself today.",
];

#[rustfmt::skip]
const OPPOSITES: &[(&str, &str)] = &[
    ("hot", "cold"), ("early", "late"), ("empty", "full"), ("quiet", "loud"), ("rough", "smooth"),
    ("heavy", "light"), ("wide", "narrow"), ("deep", "shallow"), ("generous", "selfish"),
    ("tall", "short"), ("fast", "slow"), ("wet", "dry"), ("hard", "soft"), ("dark", "bright"),
    ("clean", "dirty"), ("near", "far"), ("open", "closed"), ("push", "pull"), ("buy", "sell"),
    ("give", "take"), ("arrive", "depart"), ("begin", "finish"), ("accept", "refuse"),
    ("increase", "decrease"), ("include", "exclude"), ("inside", "outside"), ("above", "below"),
    ("borrow", "lend"), ("cheap", "expensive"), ("polite", "rude"), ("strong", "weak"),
    ("rich", "poor"), ("sharp", "blunt"), ("thick", "thin"), ("sweet", "sour"), ("tight", "loose"),
    ("noisy", "silent"), ("private", "public"), ("maximum", "minimum"), ("entrance", "exit"),
    ("success", "failure"), ("victory", "defeat"), ("question", "answer"), ("asleep", "awake"),
    ("absent", "present"), ("frequent", "rare"), ("careful", "careless"), ("true", "false"),
    ("wild", "tame"), ("visible", "invisible"),
];

#[rustfmt::skip]
const SYNONYMS: &[(&str, &str, &str)] = &[
    ("big", "large", "huge"), ("small", "tiny", "little"), ("smart", "clever", "bright"),
    ("angry", "cross", "furious"), ("tired", "sleepy", "weary"), ("funny", "amusing", "hilarious"),
    ("quick", "fast", "speedy"), ("hard", "difficult", "tough"), ("easy", "simple", "effortless"),
    ("begin", "start", "commence"), ("end", "finish", "conclude"), ("brave", "bold", "courageous"),
    ("kind", "gentle", "caring"), ("shy", "timid", "bashful"), ("calm", "peaceful", "serene"),
    ("strange", "odd", "unusual"), ("rich", "wealthy", "prosperous"), ("loud", "noisy", "booming"),
    ("quiet", "silent", "hushed"), ("old", "ancient", "aged"), ("new", "fresh", "modern"),
    ("cold", "chilly", "freezing"), ("hot", "warm", "boiling"), ("wet", "damp", "soaked"),
    ("dirty", "messy", "grimy"), ("clean", "spotless", "tidy"), ("sad", "unhappy", "gloomy"),
    ("scared", "afraid", "frightened"), ("important", "vital", "essential"),
    ("help", "assist", "support"), ("look", "glance", "gaze"), ("walk", "stroll", "wander"),
    ("talk", "chat", "speak"), ("laugh", "giggle", "chuckle"), ("cry", "weep", "sob"),
    ("jump", "leap", "bound"), ("think", "ponder", "consider"), ("build", "construct", "assemble"),
    ("fix", "repair", "mend"), ("tasty", "flavorful", "savory"),
];

#[rustfmt::skip]
const SPELL_WORDS: &[&str] = &[
    "rhythm", "separate", "calendar", "definitely", "February", "library", "restaurant",
    "government", "environment", "temperature", "vegetable", "accommodate", "occasion",
    "embarrass", "knowledge", "island", "receipt", "column", "schedule", "mischievous",
    "pronunciation", "conscience", "tomorrow", "through", "enough", "believe", "weird",
    "colleague", "guarantee", "foreign", "height", "liaison", "millennium", "parallel",
    "privilege", "recommend", "rhyme", "souvenir", "tongue", "yacht",
];

#[rustfmt::skip]
const CAPITALS: &[(&str, &str)] = &[
    ("Japan", "Tokyo"), ("France", "Paris"), ("Spain", "Madrid"), ("Italy", "Rome"),
    ("Germany", "Berlin"), ("Australia", "Canberra"), ("Brazil", "Brasilia"),
    ("Argentina", "Buenos Aires"), ("India", "New Delhi"), ("China", "Beijing"),
    ("Russia", "Moscow"), ("Ireland", "Dublin"), ("Sweden", "Stockholm"), ("Finland", "Helsinki"),
    ("Denmark", "Copenhagen"), ("Poland", "Warsaw"), ("the Netherlands", "Amsterdam"),
    ("Belgium", "Brussels"), ("Switzerland", "Bern"), ("Turkey", "Ankara"), ("Thailand", "Bangkok"),
    ("Malaysia", "Kuala Lumpur"), ("Pakistan", "Islamabad"), ("Iran", "Tehran"), ("Iraq", "Baghdad"),
    ("Saudi Arabia", "Riyadh"), ("Morocco", "Rabat"), ("Nigeria", "Abuja"), ("Chile", "Santiago"),
    ("Venezuela", "Caracas"), ("Jamaica", "Kingston"), ("Ukraine", "Kyiv"), ("Uganda", "Kampala"),
    ("Tanzania", "Dodoma"), ("Zambia", "Lusaka"), ("Zimbabwe", "Harare"), ("Laos", "Vientiane"),
    ("Cambodia", "Phnom Penh"), ("Fiji", "Suva"), ("Samoa", "Apia"), ("Mali", "Bamako"),
    ("Niger", "Niamey"), ("Sudan", "Khartoum"), ("Algeria", "Algiers"), ("Syria", "Damascus"),
    ("Afghanistan", "Kabul"), ("Kazakhstan", "Astana"), ("Costa Rica", "San Jose"),
    ("Honduras", "Tegucigalpa"), ("Serbia", "Belgrade"), ("Albania", "Tirana"), ("Belarus", "Minsk"),
    ("Cyprus", "Nicosia"), ("Malta", "Valletta"), ("Bahrain", "Manama"), ("Botswana", "Gaborone"),
    ("Madagascar", "Antananarivo"), ("Mozambique", "Maputo"), ("Angola", "Luanda"),
];

#[rustfmt::skip]
const QA: &[(&str, &str)] = &[
    ("What do bees make?", "Bees make honey and wax."),
    ("Why do cats purr?", "Cats often purr when they are content, and sometimes to soothe themselves."),
    ("What is the largest ocean?", "The Pacific Ocean is the largest ocean on Earth."),
    ("How do birds fly?", "Birds flap their wings to push air down and back, which lifts them and moves them forward."),
    ("What is the biggest planet?", "Jupiter is the biggest planet in our solar system."),
    ("What is the closest star to Earth?", "The sun is the closest star to Earth."),
    ("Why is the ocean salty?", "Rivers carry tiny amounts of minerals into the sea, and they build up over time."),
    ("What do caterpillars turn into?", "Caterpillars turn into butterflies or moths."),
    ("How do fish breathe?", "Fish use their gills to take oxygen from the water."),
    ("What is the hottest planet?", "Venus is the hottest planet because its thick clouds trap heat."),
    ("Why do we have seasons?", "Earth is tilted, so different parts get more or less sunlight through the year."),
    ("What makes popcorn pop?", "Water inside each kernel turns to steam and bursts the shell."),
    ("What is the fastest land animal?", "The cheetah is the fastest land animal."),
    ("What is the longest river?", "The Nile and the Amazon are usually named as the longest rivers."),
    ("What is a group of crows called?", "A group of crows is called a murder."),
    ("How do volcanoes form?", "Volcanoes form where magma from deep underground pushes up through the crust."),
    ("What is the smallest bone in the body?", "The smallest bone is the stapes, inside the ear."),
    ("What do koalas eat?", "Koalas eat mostly eucalyptus leaves."),
    ("Why do zebras have stripes?", "Scientists think the stripes may help keep biting flies away."),
    ("Why do stars twinkle?", "Starlight bends as it passes through moving air, which makes stars seem to twinkle."),
    ("How do plants drink water?", "Plants pull water up from their roots through tiny tubes in their stems."),
    ("What is a glacier?", "A glacier is a huge, slow-moving river of ice."),
    ("What causes wind?", "Wind is air moving from areas of high pressure to areas of low pressure."),
    ("Why do we yawn?", "No one is completely sure, but yawning may help cool the brain or signal tiredness."),
    ("What is the tallest animal?", "The giraffe is the tallest animal."),
    ("Why do dogs wag their tails?", "Dogs often wag their tails when they are excited or friendly."),
    ("What is fog?", "Fog is a cloud that forms close to the ground."),
    ("Why does ice float?", "Ice is less dense than liquid water, so it floats."),
    ("What is the deepest part of the ocean?", "The Mariana Trench is the deepest known part of the ocean."),
    ("What do frogs eat?", "Most frogs eat insects, worms and other small creatures."),
    ("What is the largest desert?", "Antarctica is the largest desert, and the Sahara is the largest hot desert."),
    ("How fast does light travel?", "Light travels about 300,000 kilometers per second."),
    ("At what temperature does water boil?", "At sea level, water boils at 100 degrees Celsius."),
    ("At what temperature does water freeze?", "Water freezes at 0 degrees Celsius."),
    ("What is a hurricane?", "A hurricane is a powerful storm with strong winds that forms over warm ocean water."),
    ("How do magnets work?", "Magnets pull on certain metals, like iron, because of a force called magnetism."),
    ("What is the hardest natural material?", "Diamond is the hardest natural material."),
    ("What do whales eat?", "Many whales eat tiny animals called krill."),
    ("Why do camels have humps?", "A camel's hump stores fat, which it can use for energy."),
    ("How can I sleep better?", "Keep a regular bedtime, avoid screens late at night, and keep your room cool and dark."),
    ("How do I make friends in a new town?", "Join a club or class you enjoy, and say yes to small invitations."),
    ("How can I save money on groceries?", "Plan meals for the week, shop with a list, and compare prices per unit."),
    ("How do I remember names better?", "Repeat the name when you hear it and link it to something about the person."),
    ("How can I be more productive?", "Pick your most important task first and work on it before checking messages."),
    ("How do I start journaling?", "Write a few lines each night about what happened and how you felt."),
    ("How do I keep my plants healthy?", "Give them the right light, water only when the soil is dry, and check for pests."),
    ("How can I stop procrastinating?", "Break the task into tiny steps and start with one you can finish in a few minutes."),
    ("How do I calm down before an exam?", "Breathe slowly, remind yourself you have prepared, and focus on one question at a time."),
    ("How can I drink less soda?", "Swap one soda a day for plain or fizzy water."),
    ("How do I get better at drawing?", "Draw a little every day and copy simple shapes before trying complex scenes."),
    ("How can I be a better listener?", "Let the other person finish, ask follow-up questions, and avoid planning your reply while they talk."),
    ("How do I stay warm in winter?", "Wear layers, keep your hands and feet covered, and drink warm drinks."),
    ("How can I wake up earlier?", "Move your alarm a little earlier each week and get sunlight soon after waking."),
    ("How do I write a thank-you note?", "Name the gift or kindness, say how it helped you, and end with a warm wish."),
    ("How can I improve my handwriting?", "Slow down, hold the pen loosely and practice a few lines each day."),
    ("How can I reduce stress at work?", "Take short breaks, set clear priorities and talk to someone you trust."),
    ("How do I stop my glasses from fogging up?", "Make sure your mask fits snugly over your nose, or try an anti-fog wipe."),
    ("How can I make my room feel cozier?", "Add soft lighting, a few cushions and something you love to look at."),
    ("How can I learn to cook?", "Start with a few simple recipes you enjoy and cook them until they feel easy."),
    ("How do I fix a squeaky chair?", "Tighten the screws and add a drop of oil where the parts rub."),
    ("How can I keep my phone from distracting me?", "Turn off notifications and keep the phone in another room while you work."),
    ("How can I make a long drive less boring?", "Listen to podcasts or audiobooks and plan short stops to stretch."),
    ("How do I care for a cast iron pan?", "Dry it right after washing and rub in a thin layer of oil."),
    ("How can I spend less time on screens?", "Set daily limits and plan offline activities you enjoy."),
    ("How do I keep cut flowers fresh?", "Trim the stems, change the water every couple of days and keep them out of direct sun."),
    ("How can I meditate?", "Sit comfortably, close your eyes and gently bring your attention back to your breath whenever it wanders."),
    ("How do I pack light for a trip?", "Choose clothes that mix and match, and roll them instead of folding."),
    ("How can I be more patient?", "Notice when you feel rushed, take a breath, and remind yourself that most things can wait a moment."),
    ("Good morning!", "Good morning! How are you today?"),
    ("Good evening.", "Good evening! How was your day?"),
    ("I'm back.", "Welcome back! What would you like to talk about?"),
    ("That's interesting.", "I think so too."),
    ("Ha, that's funny.", "I'm glad it made you smile."),
    ("I'm not sure what to do next.", "Let's think it through together. What are your options?"),
    ("Can I ask you something?", "Of course. What's on your mind?"),
    ("I had a strange dream last night.", "Dreams can be so odd. What happened in it?"),
    ("I think I'm catching a cold.", "I'm sorry. Rest, fluids and warm drinks might help."),
    ("I just finished a great novel.", "Nice! What did you like about it?"),
    ("I'm learning to play guitar.", "That's great. Practice a little each day and it will come."),
    ("It's my favorite time of year.", "What do you love most about it?"),
    ("The kids are finally asleep.", "That must be a relief. Time to relax a little."),
    ("I got a compliment today.", "That's lovely. Compliments can really brighten a day."),
    ("I'm trying to eat healthier.", "Good for you. Small changes add up."),
    ("I'm so hungry.", "Maybe it's time for a snack or an early meal."),
    ("I moved my desk next to the window.", "Natural light can make a big difference."),
    ("My back hurts from sitting too long.", "Try standing up and stretching every half hour."),
    ("The internet is so slow today.", "That's frustrating. Restarting the router sometimes helps."),
    ("I'm going to call an old friend later.", "That's nice. Catching up can be really uplifting."),
    ("Do you have any hobbies?", "I don't have hobbies myself, but I love hearing about yours."),
    ("Tell me a joke.", "Why did the scarecrow win an award? Because he was outstanding in his field."),
    ("Tell me another joke.", "Why don't eggs tell jokes? They'd crack each other up."),
    ("I'm tired of the rain.", "Rainy days can drag. Maybe a warm drink and a good film would help."),
    ("I passed my driving test!", "Congratulations! That's a big achievement."),
    ("I got a new job.", "Congratulations! When do you start?"),
    ("See you later.", "See you! Take care."),
    ("Thanks, that was helpful.", "You're welcome. I'm glad it helped."),
    ("I'm learning to sew.", "That's a useful skill. Start with simple straight seams."),
    ("I just signed up for a pottery workshop.", "How fun! Working with clay can be very relaxing."),
];

/// Every distractor exchange: `FILLERS` and the families above, the first of
/// any repeated user turn kept.
fn filler_bank() -> Vec<(String, Vec<String>)> {
    let owned = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut bank: Vec<(String, Vec<String>)> = FILLERS
        .iter()
        .map(|(user, replies)| (user.to_string(), owned(replies)))
        .collect();
    let with = |template: &str, x: &str| template.replace("{x}", x);
    for (i, activity) in ACTIVITIES.iter().enumerate() {
        let user = with(ACTIVITY_USERS[i % ACTIVITY_USERS.len()], activity);
        bank.push((cap(&user), owned(ACTIVITY_REPLIES)));
    }
    for chore in CHORES {
        bank.push((format!("I just finished {chore}."), owned(CHORE_REPLIES)));
    }
    for (feelings, replies) in [
        (GOOD_FEELINGS, GOOD_FEELING_REPLIES),
        (BAD_FEELINGS, BAD_FEELING_REPLIES),
    ] {
        for feeling in feelings {
            bank.push((
                format!("I'm feeling {feeling} today."),
                replies.iter().map(|r| with(r, feeling)).collect(),
            ));
        }
    }
    for (word, opposite) in OPPOSITES {
        bank.push((
            format!("What is the opposite of {word}?"),
            vec![format!("The opposite of {word} is {opposite}.")],
        ));
    }
    for (word, a, b) in SYNONYMS {
        bank.push((
            format!("What's another word for {word}?"),
            vec![format!("You could say {a} or {b}.")],
        ));
    }
    for word in SPELL_WORDS {
        let letters: Vec<String> = word.to_lowercase().chars().map(String::from).collect();
        bank.push((
            format!("How do you spell {word}?"),
            vec![format!("It is spelled {}.", letters.join("-"))],
        ));
    }
    for (country, capital) in CAPITALS {
        bank.push((
            format!("What is the capital of {country}?"),
            vec![format!("The capital of {country} is {capital}.")],
        ));
    }
    for (user, reply) in QA {
        bank.push((user.to_string(), vec![reply.to_string()]));
    }
    let mut seen = BTreeSet::new();
    bank.retain(|(user, _)| seen.insert(normalized(user)));
    bank
}

const ACKS: &[&str] = &[
    "Got it.",
    "Good to know.",
    "Thanks for telling me.",
    "Okay, I'll keep that in mind.",
    "Noted!",
    "That sounds like fun.",
    "Nice!",
    "Sounds good.",
    "How exciting!",
    "Okay, thanks.",
    "That's nice to hear.",
    "I'll remember that.",
];
const UPDATE_ACKS: &[&str] = &[
    "Okay, thanks for the update.",
    "Got it, I've noted the change.",
    "Thanks for correcting that.",
    "Okay, noted.",
];
const ACCEPTS: &[&str] = &[
    "Perfect, let's go with those.",
    "I like those. We'll use them.",
    "Great ideas, thank you! Let's do that.",
    "Those work for me.",
];
const ACCEPT_ACKS: &[&str] = &[
    "Great, I'm glad you like them.",
    "Happy to help.",
    "Good choice.",
];
const QUESTION_PREFIXES: &[&str] = &[
    "Quick question: ",
    "Remind me, ",
    "Hey, ",
    "Okay, so ",
    "One more thing: ",
    "Sorry, I forgot. ",
];
const SUGGEST_ASKS: &[&str] = &[
    "What did you suggest for {k}?",
    "Remind me what you picked for {k}.",
    "Which one did you suggest for {k}?",
];
const ABSTAIN_ANSWERS: &[&str] = &[
    "You haven't told me that.",
    "You haven't told me that yet.",
    "I don't know. You didn't mention {ka}.",
    "I'm not sure. You never told me about {ka}.",
    "You haven't said anything about {ka} yet.",
    "I don't know that one. You haven't mentioned {ka}.",
];
const NO_FACT_ABSTAIN_ANSWERS: &[&str] = &[
    "You haven't told me that.",
    "I don't know. You haven't told me that.",
    "I'm not sure. You haven't mentioned it.",
];

// ---------------------------------------------------------------------------
// The world: filtered pools and templates.

#[derive(Clone, Debug)]
struct Key {
    user: String,
    asst: String,
    panel_shared: bool,
}

fn key_from(user: &str, panel: &Panel) -> Key {
    let asst = if let Some(rest) = user.strip_prefix("my ") {
        format!("your {rest}")
    } else if let Some(rest) = user.strip_prefix("our ") {
        format!("your {rest}")
    } else {
        match user {
            "Mom" => "your mom".to_owned(),
            "Dad" => "your dad".to_owned(),
            "Grandpa" => "your grandpa".to_owned(),
            "Grandma" => "your grandma".to_owned(),
            other => other.to_owned(),
        }
    };
    Key {
        panel_shared: panel.shares_content(user),
        user: user.to_owned(),
        asst,
    }
}

struct World {
    panel: Panel,
    values: BTreeMap<Vc, Vec<String>>,
    keys: BTreeMap<Kc, Vec<Key>>,
    person_names: Vec<String>,
    fillers: Vec<(String, Vec<String>)>,
    /// Filtered templates, per frame: one list per template field.
    frames: Vec<FrameT>,
    dropped: BTreeMap<String, Vec<String>>,
    /// Pools of the v2 families (their drops are reported only under v2).
    v2: V2Pools,
    /// Acknowledgement and acceptance turns that are not panel strings.
    acks: Acks,
}

/// The fixed short turns, minus any that equal a panel string or share a
/// 6-gram with one ("Got it." is a conversational-v4 check phrase).
struct Acks {
    plain: Vec<&'static str>,
    update: Vec<&'static str>,
    accepts: Vec<&'static str>,
    accept_acks: Vec<&'static str>,
}

struct FrameT {
    spec: &'static Frame,
    pair: Vec<&'static str>,
    single: Vec<&'static str>,
    update: Vec<&'static str>,
    ask: Vec<&'static str>,
    answer: Vec<&'static str>,
    reverse_ask: Vec<&'static str>,
    reverse_answer: Vec<&'static str>,
    suggest_request: Vec<&'static str>,
    suggest_reply: Vec<&'static str>,
}

fn template_literal(template: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for ch in template.chars() {
        match ch {
            '{' => {
                depth += 1;
                out.push(' ');
            }
            '}' => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

impl World {
    fn new(panel: Panel) -> Result<Self> {
        let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut discard = |kind: &str, item: &str| {
            dropped
                .entry(kind.to_owned())
                .or_default()
                .push(item.to_owned());
        };
        // Values: no content stem in common with any panel string or field.
        let mut values: BTreeMap<Vc, Vec<String>> = BTreeMap::new();
        let mut admit = |class: Vc, raw: Vec<String>, values: &mut BTreeMap<Vc, Vec<String>>| {
            let mut kept = Vec::new();
            for item in raw {
                if content_stems(&item).is_empty()
                    || panel.shares_content(&item)
                    || panel.has_strict(&item)
                {
                    discard("value", &item);
                } else if !kept.contains(&item) {
                    kept.push(item);
                }
            }
            values.insert(class, kept);
        };
        let owned = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        admit(Vc::PersonName, owned(PERSON_NAMES), &mut values);
        let mut pets = owned(PET_NAMES);
        pets.extend(owned(PERSON_NAMES));
        admit(Vc::PetName, pets, &mut values);
        admit(Vc::Food, owned(FOODS), &mut values);
        admit(Vc::Dish, owned(DISHES), &mut values);
        admit(Vc::Drink, owned(DRINKS), &mut values);
        admit(Vc::Object, owned(OBJECTS), &mut values);
        admit(
            Vc::Age,
            (13..=95).map(|n| n.to_string()).collect(),
            &mut values,
        );
        admit(Vc::Job, owned(JOBS), &mut values);
        let mut places = owned(PLACES);
        for p in TOWN_PREFIXES {
            for s in TOWN_SUFFIXES {
                places.push(format!("{p}{s}"));
            }
        }
        admit(Vc::Place, places, &mut values);
        let mut streets = Vec::new();
        for w in STREET_WORDS {
            for t in STREET_TYPES {
                streets.push(format!("{w} {t}"));
            }
        }
        admit(Vc::Street, streets, &mut values);
        admit(Vc::Instrument, owned(INSTRUMENTS), &mut values);
        admit(Vc::Spot, owned(SPOTS), &mut values);
        admit(Vc::Color, owned(COLORS), &mut values);
        let mut times = Vec::new();
        for h in 13..=22 {
            for m in [15, 20, 25, 30, 35, 40, 45, 50, 55] {
                times.push(format!("{h}:{m}"));
            }
        }
        admit(Vc::Time, times, &mut values);
        let mut dates = Vec::new();
        for month in MONTHS {
            for day in 13..=31u32 {
                let suffix = match day {
                    21 | 31 => "st",
                    22 => "nd",
                    23 => "rd",
                    _ => "th",
                };
                dates.push(format!("{month} {day}{suffix}"));
            }
        }
        admit(Vc::Date, dates, &mut values);
        admit(Vc::Plant, owned(PLANTS), &mut values);
        for (class, pool) in &values {
            if pool.len() < 8 {
                return Err(format!(
                    "value pool {class:?} has {} values after the panel filter; at least 8 are needed",
                    pool.len()
                ));
            }
        }
        let person_names = values.get(&Vc::PersonName).cloned().unwrap_or_default();

        // Keys: a key holding a recall-row value word is dropped; one sharing
        // other panel words is kept and flagged.
        let mut keys: BTreeMap<Kc, Vec<Key>> = BTreeMap::new();
        for (class, list) in [
            (Kc::Pets, PET_KEYS),
            (Kc::People, PEOPLE_KEYS),
            (Kc::Roles, ROLE_KEYS),
            (Kc::Items, ITEM_KEYS),
            (Kc::Paintables, PAINTABLE_KEYS),
            (Kc::Appointments, APPOINTMENT_KEYS),
            (Kc::Events, EVENT_KEYS),
            (Kc::Beds, BED_KEYS),
        ] {
            let mut kept = Vec::new();
            for user in list {
                if panel.has_strict(user) {
                    discard("key", user);
                } else {
                    kept.push(key_from(user, &panel));
                }
            }
            if kept.iter().filter(|k| !k.panel_shared).count() < 4 {
                return Err(format!(
                    "key class {class:?} has fewer than 4 keys disjoint from the panels"
                ));
            }
            keys.insert(class, kept);
        }

        let mut fillers = Vec::new();
        for (user, replies) in filler_bank() {
            if panel.turn_leak(&user).is_some() {
                discard("filler", &user);
                continue;
            }
            let kept: Vec<String> = replies
                .iter()
                .filter(|r| {
                    let ok = panel.turn_leak(r).is_none();
                    if !ok {
                        discard("filler_reply", r);
                    }
                    ok
                })
                .cloned()
                .collect();
            if kept.is_empty() {
                discard("filler", &user);
            } else {
                fillers.push((user, kept));
            }
        }
        if fillers.len() < 20 {
            return Err(format!(
                "only {} distractor exchanges survive the panel filter",
                fillers.len()
            ));
        }

        let mut frames = Vec::new();
        for spec in FRAMES {
            let mut keep = |field: &str, list: &'static [&'static str]| -> Vec<&'static str> {
                list.iter()
                    .copied()
                    .filter(|t| {
                        let literal = template_literal(t);
                        let ok = !panel.has_strict(&literal)
                            && !grams(&literal).iter().any(|g| panel.grams.contains(g));
                        if !ok {
                            discard(&format!("template:{}:{field}", spec.id), t);
                        }
                        ok
                    })
                    .collect()
            };
            let frame = FrameT {
                spec,
                pair: keep("pair", spec.pair),
                single: keep("single", spec.single),
                update: keep("update", spec.update),
                ask: keep("ask", spec.ask),
                answer: keep("answer", spec.answer),
                reverse_ask: keep("reverse_ask", spec.reverse_ask),
                reverse_answer: keep("reverse_answer", spec.reverse_answer),
                suggest_request: keep("suggest_request", spec.suggest_request),
                suggest_reply: keep("suggest_reply", spec.suggest_reply),
            };
            for (field, list) in [
                ("pair", &frame.pair),
                ("single", &frame.single),
                ("update", &frame.update),
                ("ask", &frame.ask),
                ("answer", &frame.answer),
                ("reverse_ask", &frame.reverse_ask),
                ("reverse_answer", &frame.reverse_answer),
            ] {
                if list.is_empty() {
                    return Err(format!("frame {} has no {field} template left", spec.id));
                }
            }
            frames.push(frame);
        }
        let mut short = |list: &[&'static str]| -> Result<Vec<&'static str>> {
            let kept: Vec<&'static str> = list
                .iter()
                .copied()
                .filter(|t| {
                    let ok = panel.turn_leak(t).is_none();
                    if !ok {
                        discard("acknowledgement", t);
                    }
                    ok
                })
                .collect();
            if kept.is_empty() {
                return Err("every acknowledgement of a list is a panel string".into());
            }
            Ok(kept)
        };
        let acks = Acks {
            plain: short(ACKS)?,
            update: short(UPDATE_ACKS)?,
            accepts: short(ACCEPTS)?,
            accept_acks: short(ACCEPT_ACKS)?,
        };
        let colors = values.get(&Vc::Color).cloned().unwrap_or_default();
        let v2 = V2Pools::new(&panel, &colors)?;
        Ok(Self {
            panel,
            values,
            keys,
            person_names,
            fillers,
            frames,
            dropped,
            v2,
            acks,
        })
    }
}

// ---------------------------------------------------------------------------
// Dialogues.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Category {
    Binding,
    Update,
    Reverse,
    AssistantStated,
    CrossRelation,
    Abstain,
    Rule,
    ImplicitUpdate,
    SelfFact,
    Attribute,
    Count,
    Order,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Update => "update",
            Self::Reverse => "reverse",
            Self::AssistantStated => "assistant_stated",
            Self::CrossRelation => "cross_relation",
            Self::Abstain => "abstain",
            Self::Rule => "rule",
            Self::ImplicitUpdate => "implicit_update",
            Self::SelfFact => "self_fact",
            Self::Attribute => "attribute",
            Self::Count => "count",
            Self::Order => "order",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    User,
    Assistant,
}

#[derive(Clone, Debug)]
struct Turn {
    role: Role,
    text: String,
    /// A distractor exchange's turn.
    filler: bool,
    /// What an assistant turn is: `answer`, `acknowledgement`, `suggestion`
    /// or `distractor` (`user` for user turns).
    kind: &'static str,
}

#[derive(Clone, Debug)]
struct Fact {
    frame: usize,
    key: Key,
    value: String,
    old: Option<String>,
    by_assistant: bool,
}

#[derive(Clone, Debug)]
struct Question {
    /// Index of the assistant turn that answers it.
    turn: usize,
    category: Category,
    followup: bool,
    /// Detail: `latest`/`unchanged` for updates, `same_relation`,
    /// `other_relation` or `no_facts` for abstentions.
    detail: &'static str,
    /// Words the answer must contain (all content words); `None` for an
    /// abstention.
    expect: Option<String>,
    /// Stated values the answer must not name.
    forbid: Vec<String>,
    /// Other keys the answer must not name.
    forbid_keys: Vec<String>,
    /// The never-stated key a v2 abstention asks about (`None` in v1).
    unstated_key: Option<String>,
}

#[derive(Clone, Debug)]
struct Dialogue {
    category: Category,
    frames: Vec<&'static str>,
    strict_keys: bool,
    turns: Vec<Turn>,
    questions: Vec<Question>,
    facts: Vec<Fact>,
    keys: Vec<Key>,
}

impl Dialogue {
    fn record(&self, id: &str) -> Value {
        json!({
            "id": id,
            "category": self.category.name(),
            "frames": self.frames,
            "strict_keys": self.strict_keys,
            "messages": self.turns.iter().map(|t| json!({
                "role": match t.role { Role::User => "user", Role::Assistant => "assistant" },
                "content": t.text,
            })).collect::<Vec<_>>(),
            "questions": self.questions.iter().map(|q| json!({
                "turn": q.turn,
                "category": q.category.name(),
                "followup": q.followup,
                "detail": q.detail,
                "expect": q.expect,
                "forbid": q.forbid,
                "forbid_keys": q.forbid_keys,
            })).collect::<Vec<_>>(),
        })
    }
}

fn chance(rng: &mut Rng, p: f64) -> bool {
    ((rng.next_u64() >> 11) as f64) / ((1u64 << 53) as f64) < p
}

fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> Result<&'a T> {
    if items.is_empty() {
        return Err("picked from an empty list".into());
    }
    Ok(&items[rng.below(items.len())])
}

fn article(value: &str, art: Art) -> String {
    match art {
        Art::None => value.to_owned(),
        Art::The => format!("the {value}"),
        Art::Indef => {
            let vowel = value
                .chars()
                .next()
                .is_some_and(|c| "aeiouAEIOU".contains(c));
            if vowel {
                format!("an {value}")
            } else {
                format!("a {value}")
            }
        }
    }
}

/// Fill `{name}` placeholders from `slots`; an unknown one is an error.
fn fill(template: &str, slots: &BTreeMap<String, String>) -> Result<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("unclosed placeholder in {template:?}"))?;
        let name = &after[..end];
        let value = slots
            .get(name)
            .ok_or_else(|| format!("no slot {name} for {template:?}"))?;
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn possessive(text: &str) -> String {
    format!("{text}'s")
}

fn key_slots(slots: &mut BTreeMap<String, String>, suffix: &str, key: &Key) {
    let mut put = |name: &str, value: String| {
        slots.insert(format!("{name}{suffix}"), value);
    };
    put("k", key.user.clone());
    put("K", cap(&key.user));
    put("kp", possessive(&key.user));
    put("Kp", cap(&possessive(&key.user)));
    put("ka", key.asst.clone());
    put("KA", cap(&key.asst));
    put("kap", possessive(&key.asst));
    put("KAP", cap(&possessive(&key.asst)));
}

fn value_slots(slots: &mut BTreeMap<String, String>, suffix: &str, value: &str, art: Art) {
    let with = article(value, art);
    slots.insert(format!("v{suffix}"), value.to_owned());
    slots.insert(format!("V{suffix}"), cap(value));
    slots.insert(format!("av{suffix}"), with.clone());
    slots.insert(format!("AV{suffix}"), cap(&with));
}

fn fact_slots(fact: &Fact, art: Art) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    key_slots(&mut slots, "", &fact.key);
    value_slots(&mut slots, "", &fact.value, art);
    if let Some(old) = &fact.old {
        slots.insert("old".into(), old.clone());
        slots.insert("aold".into(), article(old, art));
    }
    slots
}

fn pair_slots(a: &Fact, b: &Fact, art: Art) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    key_slots(&mut slots, "1", &a.key);
    key_slots(&mut slots, "2", &b.key);
    value_slots(&mut slots, "1", &a.value, art);
    value_slots(&mut slots, "2", &b.value, art);
    slots
}

/// Draw state: stems already used by keys and values in this dialogue.
struct Draw<'w> {
    world: &'w World,
    taken: BTreeSet<String>,
    strict: bool,
}

impl Draw<'_> {
    fn fresh(&self, text: &str) -> bool {
        let stems = content_stems(text);
        !stems.is_empty() && stems.iter().all(|s| !self.taken.contains(s))
    }

    fn take(&mut self, text: &str) {
        self.taken.extend(content_stems(text));
    }

    fn key(&mut self, rng: &mut Rng, class: Kc) -> Result<Option<Key>> {
        let world = self.world;
        let pool = world
            .keys
            .get(&class)
            .ok_or_else(|| format!("no keys for {class:?}"))?;
        for _ in 0..40 {
            let key = if class == Kc::People && chance(rng, 0.5) {
                let name = pick(rng, &world.person_names)?;
                Key {
                    user: name.clone(),
                    asst: name.clone(),
                    panel_shared: false,
                }
            } else {
                pick(rng, pool)?.clone()
            };
            if (self.strict && key.panel_shared) || !self.fresh(&key.user) || !self.fresh(&key.asst)
            {
                continue;
            }
            self.take(&key.user);
            self.take(&key.asst);
            return Ok(Some(key));
        }
        Ok(None)
    }

    fn value(&mut self, rng: &mut Rng, class: Vc) -> Result<Option<String>> {
        let world = self.world;
        for _ in 0..40 {
            let candidate = match class {
                Vc::PetName | Vc::PersonName if chance(rng, 0.5) => synthetic_name(rng)?,
                _ => pick(
                    rng,
                    world
                        .values
                        .get(&class)
                        .ok_or_else(|| format!("no values for {class:?}"))?,
                )?
                .clone(),
            };
            let panel = &world.panel;
            if panel.shares_content(&candidate)
                || panel.has_strict(&candidate)
                || !self.fresh(&candidate)
            {
                continue;
            }
            self.take(&candidate);
            return Ok(Some(candidate));
        }
        Ok(None)
    }
}

fn synthetic_name(rng: &mut Rng) -> Result<String> {
    const ONSETS: &[&str] = &[
        "b", "d", "f", "g", "k", "l", "m", "n", "p", "r", "s", "t", "v", "z", "br", "dr", "kl",
        "st", "tr", "sh", "th", "ch", "j", "w",
    ];
    const VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ai", "ei", "ou", "ia"];
    const CODAS: &[&str] = &["", "", "", "n", "l", "r", "s", "m", "k", "th"];
    let syllables = 2 + usize::from(chance(rng, 0.25));
    let mut name = String::new();
    for i in 0..syllables {
        if i > 0 || chance(rng, 0.8) {
            name.push_str(pick(rng, ONSETS)?);
        }
        name.push_str(pick(rng, VOWELS)?);
        if i + 1 == syllables || chance(rng, 0.3) {
            name.push_str(pick(rng, CODAS)?);
        }
    }
    Ok(cap(&name))
}

/// Outcome of one draw: a dialogue or the reason it was redrawn.
enum Drawn {
    Ok(Dialogue),
    Redraw(&'static str),
}

fn pick_category(rng: &mut Rng) -> Category {
    let total: f64 = PRIMARY_SHARES.iter().map(|(_, s)| s).sum();
    let mut x = ((rng.next_u64() >> 11) as f64) / ((1u64 << 53) as f64) * total;
    for (category, share) in PRIMARY_SHARES {
        if x < share {
            return category;
        }
        x -= share;
    }
    Category::Binding
}

struct Builder<'w> {
    draw: Draw<'w>,
    turns: Vec<Turn>,
    facts: Vec<Fact>,
    keys: Vec<Key>,
    frames: Vec<&'static str>,
    questions: Vec<Question>,
    used_fillers: BTreeSet<usize>,
}

impl<'w> Builder<'w> {
    fn world(&self) -> &'w World {
        self.draw.world
    }

    fn user(&mut self, text: String) {
        self.turns.push(Turn {
            role: Role::User,
            text,
            filler: false,
            kind: "user",
        });
    }

    fn assistant(&mut self, text: String, kind: &'static str) {
        self.turns.push(Turn {
            role: Role::Assistant,
            text,
            filler: false,
            kind,
        });
    }

    fn filler(&mut self, rng: &mut Rng) -> Result<()> {
        let fillers = &self.world().fillers;
        for _ in 0..20 {
            let index = rng.below(fillers.len());
            if self.used_fillers.contains(&index) {
                continue;
            }
            self.used_fillers.insert(index);
            let (user, replies) = &fillers[index];
            let reply = pick(rng, replies)?.clone();
            self.turns.push(Turn {
                role: Role::User,
                text: user.clone(),
                filler: true,
                kind: "user",
            });
            self.turns.push(Turn {
                role: Role::Assistant,
                text: reply,
                filler: true,
                kind: "distractor",
            });
            return Ok(());
        }
        Ok(())
    }

    fn fillers(&mut self, rng: &mut Rng, count: usize) -> Result<()> {
        for _ in 0..count {
            self.filler(rng)?;
        }
        Ok(())
    }

    fn note_frame(&mut self, frame: usize) {
        let id = self.world().frames[frame].spec.id;
        if !self.frames.contains(&id) {
            self.frames.push(id);
        }
    }

    /// Draw `n` facts of `frame` (keys and values stem-disjoint from the rest).
    fn new_facts(&mut self, rng: &mut Rng, frame: usize, n: usize) -> Result<Option<Vec<Fact>>> {
        let spec = self.world().frames[frame].spec;
        let mut facts = Vec::new();
        for _ in 0..n {
            let Some(key) = self.draw.key(rng, spec.keys)? else {
                return Ok(None);
            };
            let Some(value) = self.draw.value(rng, spec.values)? else {
                return Ok(None);
            };
            facts.push(Fact {
                frame,
                key,
                value,
                old: None,
                by_assistant: false,
            });
        }
        Ok(Some(facts))
    }

    fn ack(&mut self, rng: &mut Rng, facts: &[Fact]) -> Result<()> {
        // Sometimes the assistant restates the facts, so values also appear in
        // assistant turns.
        if chance(rng, 0.15) {
            let mut parts = vec!["Got it.".to_owned()];
            for fact in facts {
                let frame = &self.world().frames[fact.frame];
                let template = pick(rng, &frame.answer)?;
                parts.push(fill(template, &fact_slots(fact, frame.spec.art))?);
            }
            self.assistant(parts.join(" "), "acknowledgement");
        } else {
            self.assistant(
                pick(rng, &self.world().acks.plain)?.to_string(),
                "acknowledgement",
            );
        }
        Ok(())
    }

    /// A user turn stating `facts` (one or two of one frame).
    fn state(&mut self, rng: &mut Rng, facts: &[Fact]) -> Result<()> {
        let frame = &self.world().frames[facts[0].frame];
        let text = match facts {
            [a, b] => fill(pick(rng, &frame.pair)?, &pair_slots(a, b, frame.spec.art))?,
            [a] => fill(pick(rng, &frame.single)?, &fact_slots(a, frame.spec.art))?,
            _ => return Err("a statement holds one or two facts".into()),
        };
        self.user(text);
        self.ack(rng, facts)?;
        self.note_frame(facts[0].frame);
        for fact in facts {
            self.keys.push(fact.key.clone());
        }
        self.facts.extend(facts.iter().cloned());
        Ok(())
    }

    fn question_text(&self, rng: &mut Rng, text: String) -> Result<String> {
        let starts_wh = ["What", "Where", "Who", "Which", "When", "How"]
            .iter()
            .any(|w| text.starts_with(w));
        if starts_wh && chance(rng, 0.3) {
            let prefix = pick(rng, QUESTION_PREFIXES)?;
            if prefix.ends_with(". ") {
                Ok(format!("{prefix}{text}"))
            } else {
                Ok(format!("{prefix}{}", lower_first(&text)))
            }
        } else {
            Ok(text)
        }
    }

    /// Ask about fact `index` and answer it.
    fn ask_fact(
        &mut self,
        rng: &mut Rng,
        index: usize,
        category: Category,
        followup: bool,
        detail: &'static str,
    ) -> Result<()> {
        let fact = self.facts[index].clone();
        let frame = &self.world().frames[fact.frame];
        let art = frame.spec.art;
        let slots = fact_slots(&fact, art);
        let reverse = category == Category::Reverse;
        let ask = if reverse {
            pick(rng, &frame.reverse_ask)?
        } else if fact.by_assistant && chance(rng, 0.6) {
            pick(rng, SUGGEST_ASKS)?
        } else {
            pick(rng, &frame.ask)?
        };
        let question = self.question_text(rng, fill(ask, &slots)?)?;
        let answer_template: &str = if reverse {
            if chance(rng, 0.7) {
                *pick(rng, &frame.reverse_answer)?
            } else {
                "{KA}."
            }
        } else {
            let r = rng.below(100);
            if r < 55 {
                *pick(rng, &frame.answer)?
            } else if r < 75 {
                "{AV}."
            } else if fact.old.is_some() {
                *pick(rng, &["You changed it to {av}.", "It's {av} now."])?
            } else if fact.by_assistant {
                "I suggested {av} for {ka}."
            } else if frame.spec.said {
                *pick(rng, &["You said {av}.", "You told me {av}."])?
            } else {
                *pick(rng, &frame.answer)?
            }
        };
        let answer = fill(answer_template, &slots)?;
        let mut forbid: Vec<String> = self
            .facts
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, f)| f.value.clone())
            .collect();
        forbid.extend(self.facts.iter().filter_map(|f| f.old.clone()));
        if reverse {
            // The asked value is in the question; the other values stay forbidden.
            forbid.retain(|v| v != &fact.value);
        }
        let forbid_keys: Vec<String> = self
            .keys
            .iter()
            .filter(|k| k.user != fact.key.user)
            .map(|k| k.asst.clone())
            .collect();
        let expect = if reverse {
            fact.key.asst.clone()
        } else {
            fact.value.clone()
        };
        self.user(question);
        self.assistant(answer, "answer");
        self.questions.push(Question {
            turn: self.turns.len() - 1,
            category,
            followup,
            detail,
            expect: Some(expect),
            forbid,
            forbid_keys,
            unstated_key: None,
        });
        Ok(())
    }

    /// Ask about an unstated key and abstain.
    fn ask_unknown(
        &mut self,
        rng: &mut Rng,
        frame: usize,
        followup: bool,
        detail: &'static str,
    ) -> Result<bool> {
        let spec = self.world().frames[frame].spec;
        let Some(key) = self.draw.key(rng, spec.keys)? else {
            return Ok(false);
        };
        let ftemplates = &self.world().frames[frame];
        let mut slots = BTreeMap::new();
        key_slots(&mut slots, "", &key);
        let asked = fill(pick(rng, &ftemplates.ask)?, &slots)?;
        let question = self.question_text(rng, asked)?;
        let answers = if detail == "no_facts" {
            NO_FACT_ABSTAIN_ANSWERS
        } else {
            ABSTAIN_ANSWERS
        };
        let answer = fill(pick(rng, answers)?, &slots)?;
        let mut forbid: Vec<String> = self.facts.iter().map(|f| f.value.clone()).collect();
        forbid.extend(self.facts.iter().filter_map(|f| f.old.clone()));
        let forbid_keys = self.keys.iter().map(|k| k.asst.clone()).collect::<Vec<_>>();
        self.note_frame(frame);
        self.keys.push(key);
        self.user(question);
        self.assistant(answer, "answer");
        self.questions.push(Question {
            turn: self.turns.len() - 1,
            category: Category::Abstain,
            followup,
            detail,
            expect: None,
            forbid,
            forbid_keys,
            unstated_key: None,
        });
        Ok(true)
    }

    fn other_frame(&self, rng: &mut Rng, not: &[usize], need_suggest: bool) -> Result<usize> {
        let frames = &self.world().frames;
        let candidates: Vec<usize> = (0..frames.len())
            .filter(|i| !not.contains(i))
            .filter(|i| {
                !need_suggest
                    || (!frames[*i].suggest_request.is_empty()
                        && !frames[*i].suggest_reply.is_empty())
            })
            .collect();
        Ok(*pick(rng, &candidates)?)
    }
}

/// State one or two facts of the secondary relation, if there is one.
fn state_secondary(b: &mut Builder<'_>, rng: &mut Rng, secondary: Option<usize>) -> Result<bool> {
    if let Some(frame) = secondary {
        let n = 1 + usize::from(chance(rng, 0.5));
        let Some(facts) = b.new_facts(rng, frame, n)? else {
            return Ok(false);
        };
        b.state(rng, &facts)?;
    }
    Ok(true)
}

fn draw_dialogue(world: &World, rng: &mut Rng) -> Result<Drawn> {
    let category = pick_category(rng);
    let strict = chance(rng, STRICT_KEY_SHARE);
    let mut b = Builder {
        draw: Draw {
            world,
            taken: BTreeSet::new(),
            strict,
        },
        turns: Vec::new(),
        facts: Vec::new(),
        keys: Vec::new(),
        frames: Vec::new(),
        questions: Vec::new(),
        used_fillers: BTreeSet::new(),
    };
    let distractors = rng.below(MAX_DISTRACTORS + 1);
    let pre = usize::from(distractors > 0 && chance(rng, 0.25));
    let mut remaining = distractors - pre;
    b.fillers(rng, pre)?;

    let primary = b.other_frame(rng, &[], category == Category::AssistantStated)?;
    let no_facts = category == Category::Abstain && chance(rng, 0.25);
    // A second relation's facts, as context the questions must ignore.
    let secondary = if !no_facts && category != Category::CrossRelation && chance(rng, 0.35) {
        Some(b.other_frame(rng, &[primary], false)?)
    } else {
        None
    };
    let secondary_first = chance(rng, 0.5);
    if secondary_first && !state_secondary(&mut b, rng, secondary)? {
        return Ok(Drawn::Redraw("key_or_value_pool"));
    }

    let mut primary_facts: Vec<usize> = Vec::new();
    let mut updated: Option<usize> = None;
    match category {
        Category::CrossRelation => {
            let other = b.other_frame(rng, &[primary], false)?;
            let (Some(a), Some(c)) = (b.new_facts(rng, primary, 1)?, b.new_facts(rng, other, 1)?)
            else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let fa = &world.frames[primary];
            let fc = &world.frames[other];
            let text = format!(
                "{} {}",
                fill(pick(rng, &fa.single)?, &fact_slots(&a[0], fa.spec.art))?,
                fill(pick(rng, &fc.single)?, &fact_slots(&c[0], fc.spec.art))?
            );
            b.user(text);
            let both = [a[0].clone(), c[0].clone()];
            b.ack(rng, &both)?;
            b.note_frame(primary);
            b.note_frame(other);
            for fact in both {
                b.keys.push(fact.key.clone());
                primary_facts.push(b.facts.len());
                b.facts.push(fact);
            }
        }
        Category::AssistantStated => {
            let Some(mut facts) = b.new_facts(rng, primary, 2)? else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let frame = &world.frames[primary];
            let slots = pair_slots(&facts[0], &facts[1], frame.spec.art);
            b.user(fill(pick(rng, &frame.suggest_request)?, &slots)?);
            b.assistant(
                fill(pick(rng, &frame.suggest_reply)?, &slots)?,
                "suggestion",
            );
            b.user(pick(rng, &world.acks.accepts)?.to_string());
            b.assistant(
                pick(rng, &world.acks.accept_acks)?.to_string(),
                "acknowledgement",
            );
            b.note_frame(primary);
            for fact in &mut facts {
                fact.by_assistant = true;
                b.keys.push(fact.key.clone());
                primary_facts.push(b.facts.len());
                b.facts.push(fact.clone());
            }
        }
        _ if no_facts => {}
        _ => {
            // Binding, update, reverse and same-relation abstention: two keys
            // of one relation in one turn; an update sometimes states a single
            // key (the latest value still wins).
            let single = category == Category::Update && chance(rng, 0.4);
            let n = if single { 1 } else { 2 };
            let Some(facts) = b.new_facts(rng, primary, n)? else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let start = b.facts.len();
            b.state(rng, &facts)?;
            primary_facts.extend(start..b.facts.len());
            if category == Category::Binding && chance(rng, 0.3) {
                if let Some(third) = b.new_facts(rng, primary, 1)? {
                    let start = b.facts.len();
                    b.state(rng, &third)?;
                    primary_facts.extend(start..b.facts.len());
                }
            }
        }
    }
    if !secondary_first && !state_secondary(&mut b, rng, secondary)? {
        return Ok(Drawn::Redraw("key_or_value_pool"));
    }

    if category == Category::Update {
        let before = rng.below(remaining.min(2) + 1);
        b.fillers(rng, before)?;
        remaining -= before;
        let index = *pick(rng, &primary_facts)?;
        let frame_index = b.facts[index].frame;
        let spec = world.frames[frame_index].spec;
        let Some(new_value) = b.draw.value(rng, spec.values)? else {
            return Ok(Drawn::Redraw("key_or_value_pool"));
        };
        let fact = &mut b.facts[index];
        fact.old = Some(std::mem::replace(&mut fact.value, new_value));
        let slots = fact_slots(&b.facts[index], spec.art);
        let template = pick(rng, &world.frames[frame_index].update)?;
        b.user(fill(template, &slots)?);
        b.assistant(
            pick(rng, &world.acks.update)?.to_string(),
            "acknowledgement",
        );
        updated = Some(index);
    }

    let followups = if no_facts {
        0
    } else {
        let r = rng.below(100);
        if r < 65 {
            0
        } else if r < 90 {
            1
        } else {
            2
        }
    };
    let between = if followups > 0 && remaining > 0 && chance(rng, 0.5) {
        1
    } else {
        0
    };
    b.fillers(rng, remaining - between)?;

    // The first question.
    let mut asked: BTreeSet<usize> = BTreeSet::new();
    match category {
        Category::Abstain => {
            let (frame, detail) = if no_facts {
                (b.other_frame(rng, &[], false)?, "no_facts")
            } else if chance(rng, 0.65) {
                (primary, "same_relation")
            } else {
                let mut not = vec![primary];
                if let Some(s) = secondary {
                    not.push(s);
                }
                (b.other_frame(rng, &not, false)?, "other_relation")
            };
            if !b.ask_unknown(rng, frame, false, detail)? {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            }
        }
        Category::Update => {
            let target = updated.ok_or("an update without an updated fact")?;
            let others: Vec<usize> = primary_facts
                .iter()
                .copied()
                .filter(|i| *i != target)
                .collect();
            let (index, detail) = if !others.is_empty() && chance(rng, 0.3) {
                (*pick(rng, &others)?, "unchanged")
            } else {
                (target, "latest")
            };
            b.ask_fact(rng, index, Category::Update, false, detail)?;
            asked.insert(index);
        }
        _ => {
            let index = *pick(rng, &primary_facts)?;
            b.ask_fact(rng, index, category, false, "")?;
            asked.insert(index);
        }
    }

    for f in 0..followups {
        if f == 0 && between > 0 {
            b.fillers(rng, between)?;
        }
        let open: Vec<usize> = (0..b.facts.len()).filter(|i| !asked.contains(i)).collect();
        if open.is_empty() && !chance(rng, 0.3) {
            break;
        }
        if open.is_empty() || chance(rng, FOLLOWUP_ABSTAIN) {
            let frame = b.facts.first().map(|f| f.frame).unwrap_or(primary);
            if !b.ask_unknown(rng, frame, true, "same_relation")? {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            }
        } else {
            let index = *pick(rng, &open)?;
            let category = if Some(index) == updated {
                Category::Update
            } else {
                Category::Binding
            };
            let detail = if category == Category::Update {
                "latest"
            } else {
                ""
            };
            b.ask_fact(rng, index, category, true, detail)?;
            asked.insert(index);
        }
    }

    let dialogue = Dialogue {
        category,
        frames: b.frames,
        strict_keys: strict,
        turns: b.turns,
        questions: b.questions,
        facts: b.facts,
        keys: b.keys,
    };
    finalize(world, dialogue)
}

/// A distractor or acknowledgement turn naming a key or value of the
/// dialogue, or an abstention key that appears before its question.
fn collision(d: &Dialogue) -> Option<&'static str> {
    let mut stems = BTreeSet::new();
    for fact in &d.facts {
        stems.extend(content_stems(&fact.value));
        if let Some(old) = &fact.old {
            stems.extend(content_stems(old));
        }
    }
    for key in &d.keys {
        stems.extend(content_stems(&key.user));
        stems.extend(content_stems(&key.asst));
    }
    for turn in d.turns.iter().filter(|t| t.filler) {
        if content_stems(&turn.text).iter().any(|s| stems.contains(s)) {
            return Some("distractor_names_a_key_or_value");
        }
    }
    for q in d.questions.iter().filter(|q| q.expect.is_none()) {
        if let Some(key) = &q.unstated_key {
            let stated: BTreeSet<String> = d
                .facts
                .iter()
                .flat_map(|f| content_stems(&f.key.user))
                .collect();
            let fresh: Vec<String> = content_stems(key)
                .into_iter()
                .filter(|s| !stated.contains(s))
                .collect();
            for turn in &d.turns[..q.turn - 1] {
                let t = content_stems(&turn.text);
                if fresh.iter().any(|s| t.contains(s)) {
                    return Some("abstention_key_mentioned_earlier");
                }
            }
            continue;
        }
        // The abstention key is the last key pushed for it; it must not appear
        // in any turn before its question.
        let question_turn = q.turn - 1;
        let asked = content_stems(&d.turns[question_turn].text);
        let stated: BTreeSet<String> = d
            .facts
            .iter()
            .flat_map(|f| content_stems(&f.key.user))
            .collect();
        let fresh: Vec<&String> = asked.iter().filter(|s| !stated.contains(*s)).collect();
        for turn in &d.turns[..question_turn] {
            let t = content_stems(&turn.text);
            if fresh.iter().any(|s| t.contains(*s) && !is_generic(s)) {
                return Some("abstention_key_mentioned_earlier");
            }
        }
    }
    None
}

/// Question words that are not part of a key ("called", "name", ...).
fn is_generic(stem: &str) -> bool {
    const GENERIC: &[&str] = &[
        "called",
        "name",
        "named",
        "buy",
        "pick",
        "store",
        "come",
        "home",
        "favorite",
        "dish",
        "love",
        "eat",
        "best",
        "order",
        "drink",
        "gift",
        "getting",
        "giving",
        "old",
        "age",
        "turn",
        "work",
        "job",
        "living",
        "traveling",
        "place",
        "visiting",
        "street",
        "live",
        "instrument",
        "play",
        "practice",
        "evening",
        "put",
        "leave",
        "color",
        "paint",
        "time",
        "start",
        "date",
        "set",
        "plant",
        "growing",
        "went",
        "remember",
        "quick",
        "question",
        "remind",
        "sorry",
        "forgot",
        "thing",
        "give",
        "wait",
        "year",
        "years",
        "spring",
        "new",
    ];
    GENERIC.contains(&stem)
}

/// The semantic contract of every question; an error here is a generator bug.
fn check_dialogue(d: &Dialogue) -> Result<()> {
    if d.turns.is_empty() || d.questions.is_empty() {
        return Err("a dialogue without turns or questions".into());
    }
    for (i, turn) in d.turns.iter().enumerate() {
        let expected = if i % 2 == 0 {
            Role::User
        } else {
            Role::Assistant
        };
        if turn.role != expected {
            return Err(format!("turn {i} breaks user/assistant alternation"));
        }
        if turn.text.contains('\n') || turn.text.contains("<|") {
            return Err(format!("turn {i} holds a newline or special-token text"));
        }
    }
    for q in &d.questions {
        let answer = content_stems(&d.turns[q.turn].text);
        let asked = content_stems(&d.turns[q.turn - 1].text);
        if let Some(expect) = &q.expect {
            let want = content_stems(expect);
            if want.is_empty() || !want.iter().all(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} lacks {expect:?}",
                    d.turns[q.turn].text
                ));
            }
            if q.category != Category::Reverse
                && !q.detail.ends_with("choice")
                && want.iter().any(|s| asked.contains(s))
            {
                return Err(format!(
                    "question {:?} names its own answer",
                    d.turns[q.turn - 1].text
                ));
            }
        }
        for value in &q.forbid {
            if content_stems(value).iter().any(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} names the distractor {value:?}",
                    d.turns[q.turn].text
                ));
            }
        }
        for key in &q.forbid_keys {
            if content_stems(key).iter().any(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} names the other key {key:?}",
                    d.turns[q.turn].text
                ));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Generator v2: six more question families (`generator=v2`, the default).
//
// `generator=v1` draws exactly the v1 dialogues. Under v2 each draw is a v1
// dialogue with probability `V1_SHARE` and otherwise one of the families
// below, chosen uniformly: rules with negation, implicit updates, first-person
// facts beside another person's, attribute-disambiguated objects, counts per
// container or person, and temporal order.

/// Share of v1 dialogues under `generator=v2`.
const V1_SHARE: f64 = 0.46;
const FAMILIES: [Category; 6] = [
    Category::Rule,
    Category::ImplicitUpdate,
    Category::SelfFact,
    Category::Attribute,
    Category::Count,
    Category::Order,
];

#[rustfmt::skip]
const RULE_SETTINGS: &[&str] = &[
    "at the library", "in the museum", "at the pool", "in the park", "on the bus",
    "in the classroom", "at the campsite", "in the gym", "in the garden", "on the ferry",
    "at the hostel", "at the cinema", "in the art studio", "at the ice rink", "in the hospital",
    "at the stadium", "on the trail", "at the aquarium", "in the dormitory", "at the zoo",
    "in the kitchen", "at the market", "on the playground", "at the skate rink", "in the lab",
    "at the wedding", "on the plane", "at the concert hall", "in the waiting room",
    "at the bowling alley", "in the greenhouse", "at the marina", "in the chapel", "at the spa",
];

#[rustfmt::skip]
const RULE_ACTS: &[(&str, &str)] = &[
    ("diving", "dive"), ("whistling", "whistle"), ("skateboarding", "skateboard"),
    ("juggling", "juggle"), ("snacking", "snack"), ("singing", "sing"), ("napping", "nap"),
    ("sketching", "sketch"), ("drumming", "drum"), ("sledding", "sled"), ("climbing", "climb"),
    ("picnicking", "picnic"), ("cycling", "cycle"), ("rollerblading", "rollerblade"),
    ("barbecuing", "barbecue"), ("camping", "camp"), ("paddling", "paddle"),
    ("skipping", "skip"), ("dancing", "dance"), ("chanting", "chant"), ("texting", "text"),
    ("clapping", "clap"), ("feeding the ducks", "feed the ducks"),
    ("flying drones", "fly drones"), ("taking photos", "take photos"),
    ("shouting", "shout"), ("splashing", "splash"), ("somersaulting", "somersault"),
    ("humming", "hum"), ("knitting", "knit"), ("sunbathing", "sunbathe"),
    ("tobogganing", "toboggan"), ("birdwatching", "birdwatch"), ("stretching", "stretch"),
    ("meditating", "meditate"), ("snorkeling", "snorkel"), ("wrestling", "wrestle"),
    ("vaping", "vape"), ("skydiving", "skydive"), ("fencing", "fence"), ("yodeling", "yodel"),
    ("doodling", "doodle"), ("gardening", "garden"), ("gossiping", "gossip"),
];

#[rustfmt::skip]
const RULE_STATES: &[&str] = &[
    "{S}, {a} is allowed but {x} is not.",
    "{S}, {a} is fine, but {x} isn't allowed.",
    "{X} is banned {s}, though {a} is okay.",
    "They told us {a} is permitted {s}, but {x} is not.",
    "{S}, we can {ab} but we can't {xb}.",
    "{S}, you're allowed to {ab}, but you're not allowed to {xb}.",
    "The rule {s} is simple: {a} yes, {x} no.",
    "{S}, {x} is off limits, but {a} is welcome.",
];
const RULE_ASK_ALLOWED: &[&str] = &[
    "What is allowed {s}?",
    "What are we allowed to do {s}?",
    "What can we do {s}?",
    "Which activity is fine {s}?",
];
const RULE_ASK_BANNED: &[&str] = &[
    "What is not allowed {s}?",
    "What can't we do {s}?",
    "What is banned {s}?",
    "Which activity is off limits {s}?",
];
const RULE_ASK_ALLOWED_CHOICE: &[&str] = &[
    "{S}, is {p} or {q} allowed?",
    "{S}, which one is okay, {p} or {q}?",
];
const RULE_ASK_BANNED_CHOICE: &[&str] = &[
    "{S}, which is not allowed, {p} or {q}?",
    "{S}, is {p} or {q} banned?",
];
const RULE_ANSWER_ALLOWED: &[&str] = &[
    "{A} is allowed.",
    "{A} is fine {s}.",
    "{A}.",
    "You can go ahead with {a}.",
];
const RULE_ANSWER_BANNED: &[&str] = &[
    "{X} is not allowed.",
    "{X} isn't allowed {s}.",
    "{X} is off limits.",
    "{X}.",
];
const RULE_ABSTAIN: &[&str] = &[
    "You haven't told me the rules {s}.",
    "I don't know. You didn't mention the rules {s}.",
];

/// Implicit-update plan kinds: keys, the value class and the templates.
struct PlanKind {
    keys: &'static [&'static str],
    values: Vc,
    initial: &'static [&'static str],
    change: &'static [&'static str],
    change_named: &'static [&'static str],
    ask_latest: &'static [&'static str],
    answer_latest: &'static [&'static str],
    ask_first: &'static [&'static str],
    answer_first: &'static [&'static str],
}

const PLAN_KINDS: &[PlanKind] = &[
    PlanKind {
        keys: &[
            "our weekend trip",
            "the class outing",
            "the team retreat",
            "the family holiday",
            "the field trip",
            "the reunion",
            "the honeymoon",
            "the ski weekend",
            "the hiking trip",
            "the road trip",
            "the spring getaway",
            "the work conference",
        ],
        values: Vc::Place,
        initial: &[
            "We're planning {k} to {v}.",
            "{K} is going to be in {v}.",
            "For {k}, we picked {v}.",
        ],
        change: &[
            "The plan changed, we're going to {v} instead.",
            "We voted again and chose {v}.",
            "It moved to {v}.",
            "Now it's {v}.",
            "Everyone preferred {v}, so we switched.",
            "Scratch that, we're heading to {v}.",
            "New plan: {v}.",
        ],
        change_named: &[
            "The plan for {k} changed, we're going to {v} instead.",
            "We voted again on {k} and chose {v}.",
            "{K} moved to {v}.",
            "Now {k} is in {v}.",
            "New plan for {k}: {v}.",
        ],
        ask_latest: &[
            "Where are we going for {k}?",
            "Where is {k} now?",
            "What's the final plan for {k}?",
            "Where did we end up for {k}?",
        ],
        answer_latest: &[
            "You're going to {v} now.",
            "It's {v} now.",
            "{V}.",
            "{KA} is in {v} now.",
        ],
        ask_first: &[
            "What was the first plan for {k}?",
            "Where did we pick at first for {k}?",
            "What was the original plan for {k}?",
        ],
        answer_first: &[
            "The first plan was {v}.",
            "Originally it was {v}.",
            "{V}, before the change.",
        ],
    },
    PlanKind {
        keys: &[
            "the potluck dish",
            "the party menu",
            "the main course",
            "the bake-off entry",
            "the holiday meal",
            "the team dinner dish",
            "the fundraiser menu",
            "the farewell meal",
        ],
        values: Vc::Dish,
        initial: &[
            "For {k}, we picked {v}.",
            "We decided on {v} for {k}.",
            "{K} is going to be {v}.",
        ],
        change: &[
            "The plan changed, we're making {v} instead.",
            "We voted again and chose {v}.",
            "Now it's {v}.",
            "Everyone wanted {v}, so we switched.",
            "New plan: {v}.",
        ],
        change_named: &[
            "The plan for {k} changed, we're making {v} instead.",
            "We voted again on {k} and chose {v}.",
            "Now {k} is {v}.",
            "New plan for {k}: {v}.",
        ],
        ask_latest: &[
            "What are we having for {k}?",
            "What's {k} now?",
            "What did we settle on for {k}?",
        ],
        answer_latest: &[
            "It's {v} now.",
            "{V}.",
            "You settled on {v}.",
            "{KA} is {v} now.",
        ],
        ask_first: &[
            "What did we pick first for {k}?",
            "What was the original plan for {k}?",
            "What was the first choice for {k}?",
        ],
        answer_first: &[
            "The first plan was {v}.",
            "Originally it was {v}.",
            "{V}, before the change.",
        ],
    },
    PlanKind {
        keys: &[
            "the rehearsal",
            "the landlord meeting",
            "the yoga class",
            "the team call",
            "the open house",
            "the parent evening",
            "the dress fitting",
            "the tasting",
            "the sound check",
            "the quiz night",
        ],
        values: Vc::Time,
        initial: &[
            "{K} is at {v}.",
            "We set {k} for {v}.",
            "{K} starts at {v}.",
        ],
        change: &[
            "It moved to {v}.",
            "Now it's at {v}.",
            "They pushed it to {v}.",
            "The plan changed, it's at {v} instead.",
            "New time: {v}.",
        ],
        change_named: &[
            "{K} moved to {v}.",
            "Now {k} is at {v}.",
            "They pushed {k} to {v}.",
            "New time for {k}: {v}.",
        ],
        ask_latest: &[
            "What time is {k} now?",
            "When is {k}?",
            "What's the latest time for {k}?",
        ],
        answer_latest: &["It's at {v} now.", "{V}.", "{KA} is at {v} now."],
        ask_first: &[
            "When was {k} originally?",
            "What was the first time for {k}?",
        ],
        answer_first: &[
            "Originally it was at {v}.",
            "It was first at {v}.",
            "{V}, before the change.",
        ],
    },
];

/// First-person relations: the user's own fact beside another person's.
struct SelfRel {
    id: &'static str,
    values: Vc,
    art: Art,
    me: &'static str,
    other: &'static str,
    ask_me: &'static [&'static str],
    ask_other: &'static [&'static str],
    answer_me: &'static [&'static str],
    answer_other: &'static [&'static str],
}

const SELF_RELS: &[SelfRel] = &[
    SelfRel {
        id: "making",
        values: Vc::Dish,
        art: Art::None,
        me: "I'm making {v}.",
        other: "{K} is making {v}.",
        ask_me: &["What did I say I'm making?", "What dish am I making?"],
        ask_other: &["What is {k} making?", "What's {k} cooking?"],
        answer_me: &["You're making {v}.", "{V}.", "You said {v}."],
        answer_other: &["{KA} is making {v}.", "{V}."],
    },
    SelfRel {
        id: "bringing",
        values: Vc::Food,
        art: Art::None,
        me: "I'm bringing {v} to the picnic.",
        other: "{K} is bringing {v}.",
        ask_me: &[
            "What did I say I'm bringing to the picnic?",
            "What did I say I'd bring?",
        ],
        ask_other: &[
            "What is {k} bringing?",
            "What's {k} bringing to the picnic?",
        ],
        answer_me: &["You're bringing {v}.", "{V}.", "You said {v}."],
        answer_other: &["{KA} is bringing {v}.", "{V}."],
    },
    SelfRel {
        id: "learning",
        values: Vc::Instrument,
        art: Art::The,
        me: "I'm learning the {v}.",
        other: "{K} is learning the {v}.",
        ask_me: &[
            "Which instrument am I learning?",
            "What am I learning to play?",
        ],
        ask_other: &["What is {k} learning?", "Which instrument is {k} learning?"],
        answer_me: &["You're learning the {v}.", "{AV}.", "You said the {v}."],
        answer_other: &["{KA} is learning the {v}.", "{AV}."],
    },
    SelfRel {
        id: "flying",
        values: Vc::Place,
        art: Art::None,
        me: "I'm flying to {v} next month.",
        other: "{K} is flying to {v}.",
        ask_me: &[
            "Where am I flying?",
            "Where did I say I'm going next month?",
        ],
        ask_other: &["Where is {k} flying?", "Where is {k} going?"],
        answer_me: &["You're flying to {v}.", "{V}.", "You said {v}."],
        answer_other: &["{KA} is flying to {v}.", "{V}."],
    },
    SelfRel {
        id: "planting",
        values: Vc::Plant,
        art: Art::None,
        me: "I'm planting {v} this weekend.",
        other: "{K} is planting {v}.",
        ask_me: &["What am I planting?", "What did I say I'd plant?"],
        ask_other: &["What is {k} planting?"],
        answer_me: &["You're planting {v}.", "{V}.", "You said {v}."],
        answer_other: &["{KA} is planting {v}.", "{V}."],
    },
    SelfRel {
        id: "ordering",
        values: Vc::Drink,
        art: Art::None,
        me: "I'll have {v}.",
        other: "{K} wants {v}.",
        ask_me: &["What did I order?", "What am I having?"],
        ask_other: &["What does {k} want?", "What did {k} order?"],
        answer_me: &["You ordered {v}.", "{V}.", "You're having {v}."],
        answer_other: &["{KA} wants {v}.", "{V}."],
    },
    SelfRel {
        id: "favorite_color",
        values: Vc::Color,
        art: Art::None,
        me: "My favorite color is {v}.",
        other: "{Kp} favorite color is {v}.",
        ask_me: &[
            "What's my favorite color?",
            "Which color did I say I like best?",
        ],
        ask_other: &["What's {kp} favorite color?"],
        answer_me: &["Your favorite color is {v}.", "{V}.", "You said {v}."],
        answer_other: &["{KAP} favorite color is {v}.", "{V}."],
    },
    SelfRel {
        id: "job",
        values: Vc::Job,
        art: Art::Indef,
        me: "I work as {av}.",
        other: "{K} works as {av}.",
        ask_me: &["What do I do for work?", "What's my job?"],
        ask_other: &["What does {k} do for work?", "What is {kp} job?"],
        answer_me: &["You work as {av}.", "{AV}.", "You're {av}."],
        answer_other: &["{KA} works as {av}.", "{AV}."],
    },
    SelfRel {
        id: "age",
        values: Vc::Age,
        art: Art::None,
        me: "I just turned {v}.",
        other: "{K} just turned {v}.",
        ask_me: &["How old did I say I am?", "What age did I just turn?"],
        ask_other: &["How old is {k}?"],
        answer_me: &["You're {v}.", "You just turned {v}.", "{V}."],
        answer_other: &["{KA} is {v}.", "{KA} just turned {v}."],
    },
];
const SELF_ABSTAIN: &[&str] = &[
    "You haven't told me that.",
    "You haven't mentioned that yet.",
    "I don't know. You didn't tell me that about yourself.",
];

#[rustfmt::skip]
const ATTR_OBJECTS: &[(&str, &str)] = &[
    ("mug", "mugs"), ("notebook", "notebooks"), ("umbrella", "umbrellas"), ("candle", "candles"),
    ("vase", "vases"), ("ring", "rings"), ("bracelet", "bracelets"), ("button", "buttons"),
    ("spoon", "spoons"), ("jar", "jars"), ("lantern", "lanterns"), ("bottle", "bottles"),
    ("pen", "pens"), ("brush", "brushes"), ("ribbon", "ribbons"), ("figurine", "figurines"),
    ("badge", "badges"), ("purse", "purses"), ("glove", "gloves"), ("comb", "combs"),
    ("teapot", "teapots"), ("cushion", "cushions"), ("bucket", "buckets"), ("stool", "stools"),
    ("bead", "beads"), ("shell", "shells"), ("stone", "stones"), ("whistle", "whistles"),
    ("towel", "towels"), ("bowl", "bowls"), ("thimble", "thimbles"), ("ladle", "ladles"),
    ("compass", "compasses"), ("pendant", "pendants"), ("tin", "tins"), ("crate", "crates"),
];
const ATTR_MATERIALS: &[&str] = &[
    "wooden", "glass", "ceramic", "brass", "wicker", "leather", "plastic", "cotton", "woolen",
    "clay", "bamboo", "felt", "silk", "canvas", "crystal", "paper", "rubber", "steel", "pewter",
];
const ATTR_SIZES: &[&str] = &[
    "tiny",
    "small",
    "large",
    "oversized",
    "miniature",
    "chunky",
    "slim",
    "giant",
];

const ATTR_LOC_STATES: &[&str] = &[
    "I put the {a1} {o} {l1} and the {a2} {o} {l2}.",
    "The {a1} {o} is {l1}, and the {a2} one is {l2}.",
    "We have a couple of {os}: the {a1} one is {l1} and the {a2} one is {l2}.",
    "The {a2} {o} went {l2}, while the {a1} {o} is {l1}.",
];
const ATTR_LOC_THIRD: &[&str] = &["Oh, and the {a} {o} is {l}.", "I keep the {a} {o} {l}."];
const ATTR_LOC_ASK: &[&str] = &[
    "Where is the {a} {o}?",
    "Where did I put the {a} {o}?",
    "Where can I find the {a} {o}?",
];
const ATTR_LOC_ANSWER: &[&str] = &["The {a} {o} is {l}.", "You put the {a} {o} {l}.", "{L}."];
const ATTR_OWN_STATES: &[&str] = &[
    "The {a1} {o} belongs to {n1}, and the {a2} {o} is {n2}'s.",
    "{N1} owns the {a1} {o}, and {n2} owns the {a2} one.",
    "The {a1} {o} is {n1}'s and the {a2} one belongs to {n2}.",
];
const ATTR_OWN_THIRD: &[&str] = &[
    "Oh, and the {a} {o} is {n}'s.",
    "The {a} {o} belongs to {n}.",
];
const ATTR_OWN_ASK: &[&str] = &[
    "Whose is the {a} {o}?",
    "Who owns the {a} {o}?",
    "Who does the {a} {o} belong to?",
];
const ATTR_OWN_ANSWER: &[&str] = &["The {a} {o} belongs to {n}.", "It's {n}'s.", "{N}."];
const ATTR_REVERSE_LOC: &[&str] = &["Which {o} is {l}?"];
const ATTR_REVERSE_OWN: &[&str] = &["Which {o} is {n}'s?", "Which {o} belongs to {n}?"];
const ATTR_REVERSE_ANSWER: &[&str] = &["The {a} one.", "The {a} {o}.", "It's the {a} one."];
const ATTR_ABSTAIN: &[&str] = &[
    "You didn't mention {aa} {o}.",
    "You haven't told me about {aa} {o}.",
    "I don't know. You only told me about the {a1} and {a2} {os}.",
];

#[rustfmt::skip]
const COUNT_ITEMS: &[&str] = &[
    "stamps", "marbles", "stickers", "pencils", "coins", "shells", "buttons", "beads",
    "postcards", "candles", "balloons", "cupcakes", "cookies", "acorns", "seashells",
    "paper cranes", "tickets", "envelopes", "batteries", "spoons", "nails", "screws",
    "napkins", "cards", "ribbons", "pine cones", "dumplings", "pretzels", "erasers", "magnets",
    "keychains", "badges", "dominoes", "figurines", "bookmarks", "sachets",
];
const COUNT_CONTAINERS: &[(&str, &str)] = &[
    ("plate", "on"),
    ("shelf", "on"),
    ("tray", "on"),
    ("jar", "in"),
    ("box", "in"),
    ("bowl", "in"),
    ("bag", "in"),
    ("table", "on"),
    ("bench", "on"),
    ("windowsill", "on"),
    ("tin", "in"),
    ("crate", "in"),
    ("bin", "in"),
    ("cart", "in"),
    ("desk", "on"),
    ("envelope", "in"),
    ("folder", "in"),
    ("platter", "on"),
];
#[rustfmt::skip]
const COUNT_WORDS: &[&str] = &[
    "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "thirty", "forty", "fifty",
    "sixty", "seventy", "eighty", "ninety", "twenty-one", "thirty-one", "forty-one", "fifty-one",
];
const COUNT_STATES: &[&str] = &[
    "There are {v1} {it} {p1} {c1} and {v2} {p2} {c2}.",
    "I counted {v1} {it} {p1} {c1}, and {v2} {p2} {c2}.",
    "{C1} has {v1} {it}, and {c2} has {v2}.",
    "We put {v1} {it} {p1} {c1} and {v2} {p2} {c2}.",
];
const COUNT_THIRD: &[&str] = &["There are also {v} {p} {c}.", "Oh, and {c} has {v}."];
const COUNT_ASK: &[&str] = &[
    "How many {it} are {p} {c}?",
    "How many {it} did I count {p} {c}?",
    "What's the count {p} {c}?",
];
const COUNT_ANSWER: &[&str] = &[
    "There are {v} {it} {p} {c}.",
    "{V}.",
    "{C} has {v}.",
    "{V} {it}.",
];
const COUNT_PERSON_STATES: &[&str] = &[
    "{K1} has {v1} {it}, and {k2} has {v2}.",
    "{K1} collected {v1} {it} and {k2} collected {v2}.",
    "Between them, {k1} has {v1} {it} and {k2} has {v2}.",
];
const COUNT_PERSON_THIRD: &[&str] = &["{K} has {v}.", "Oh, and {k} has {v}."];
const COUNT_PERSON_ASK: &[&str] = &["How many {it} does {k} have?", "What's {kp} count?"];
const COUNT_PERSON_ANSWER: &[&str] = &["{KA} has {v} {it}.", "{V}.", "{KAP} count is {v}."];

#[rustfmt::skip]
const ORDER_EVENTS: &[&str] = &[
    "pottery", "fencing", "calligraphy", "origami", "kickboxing", "archery", "aerobics",
    "pilates", "karaoke", "trivia", "orienteering", "bouldering", "badminton", "croquet",
    "snorkeling", "taekwondo", "beekeeping", "woodworking", "glassblowing", "embroidery",
    "chemistry tutoring", "physiotherapy", "acupuncture", "carpentry", "a haircut", "the vet",
    "the optician", "the pharmacy", "the bakery", "the tailor", "the locksmith",
    "the post office", "the dry cleaner", "the barber", "the florist", "the notary",
    "the hardware store", "the laundromat", "the dentist", "volleyball", "salsa",
    "the chiropractor", "a podcast recording", "a photo shoot", "the recycling depot",
];
const ORDER_PLAIN_3: &[&str] = &[
    "Tomorrow starts with {e1}, after that {e2}, and then {e3}.",
    "First I have {e1}, later {e2}, and last {e3}.",
    "My plan for tomorrow: first {e1}, then {e2}, and finally {e3}.",
    "I'll do {e1} first, then {e2}, and {e3} at the end.",
];
const ORDER_PLAIN_4: &[&str] = &[
    "Tomorrow starts with {e1}, then {e2}, after that {e3}, and finally {e4}.",
    "First {e1}, then {e2}, then {e3}, and last {e4}.",
];
/// Day-part templates and the phrase that names each part in a question.
const ORDER_PARTS: &[(&str, [&str; 3])] = &[
    (
        "This morning I have {e1}, after lunch {e2}, and in the evening {e3}.",
        ["this morning", "after lunch", "in the evening"],
    ),
    (
        "In the morning there's {e1}, in the afternoon {e2}, and at night {e3}.",
        ["in the morning", "in the afternoon", "at night"],
    ),
];
const ORDER_TIMES: &[&str] = &[
    "At {t1} I have {e1}, at {t2} {e2}, and at {t3} {e3}.",
    "{e1} is at {t1}, {e2} at {t2}, and {e3} at {t3}.",
];
const ORDER_ASK_RANK: [&[&str]; 4] = [
    &[
        "What's first?",
        "What do I have first?",
        "What comes first?",
    ],
    &["What comes second?", "What's the second thing?"],
    &["What's third?", "What comes third?"],
    &[
        "What's last?",
        "What do I end with?",
        "What's the final thing?",
    ],
];
const ORDER_ANSWER_RANK: [&[&str]; 4] = [
    &["First is {e}.", "{E} comes first.", "{E}."],
    &["Second is {e}.", "{E} comes second.", "{E}."],
    &["Third is {e}.", "{E} comes third.", "{E}."],
    &["Last is {e}.", "You end with {e}.", "{E}."],
];
const ORDER_ASK_AFTER: &[&str] = &["What comes after {p}?", "What do I have after {p}?"];
const ORDER_ANSWER_AFTER: &[&str] = &["Next is {e}.", "{E} comes next.", "{E}."];
const ORDER_ASK_BEFORE: &[&str] = &[
    "What's right before {p}?",
    "What do I have just before {p}?",
];
const ORDER_ANSWER_BEFORE: &[&str] = &["Right before that is {e}.", "{E}."];
const ORDER_ASK_PART: &[&str] = &["What do I have {t}?", "What's on {t}?"];
const ORDER_ANSWER_PART: &[&str] = &["{T}, you have {e}.", "{E}."];
const ORDER_ASK_TIME: &[&str] = &["What's at {t}?", "What do I have at {t}?"];
const ORDER_ANSWER_TIME: &[&str] = &["At {t} you have {e}.", "{E}."];
const ORDER_ABSTAIN: &[&str] = &[
    "You didn't mention anything at {t}.",
    "You haven't told me about anything at {t}.",
];

/// Filtered pools of the v2 families. Key-type pools carry whether the entry
/// shares a content word with a panel (used only outside strict-key draws).
struct V2Pools {
    rule_settings: Vec<(String, bool)>,
    rule_acts: Vec<(String, String)>,
    plan_keys: Vec<Vec<(String, bool)>>,
    objects: Vec<(String, String, bool)>,
    attributes: Vec<Vec<(String, bool)>>,
    containers: Vec<(String, String, bool)>,
    count_items: Vec<(String, bool)>,
    count_values: Vec<String>,
    order_events: Vec<String>,
    dropped: BTreeMap<String, Vec<String>>,
}

impl V2Pools {
    fn new(panel: &Panel, colors: &[String]) -> Result<Self> {
        let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut keys = |kind: &str, list: &[&str]| -> Vec<(String, bool)> {
            let mut kept = Vec::new();
            for item in list {
                if panel.has_strict(item) {
                    dropped
                        .entry(kind.to_owned())
                        .or_default()
                        .push(item.to_string());
                } else {
                    kept.push((item.to_string(), panel.shares_content(item)));
                }
            }
            kept
        };
        let rule_settings = keys("rule_setting", RULE_SETTINGS);
        let plan_keys: Vec<Vec<(String, bool)>> = PLAN_KINDS
            .iter()
            .map(|k| keys("plan_key", k.keys))
            .collect();
        let materials = keys("attribute", ATTR_MATERIALS);
        let sizes = keys("attribute", ATTR_SIZES);
        let count_items = keys("count_item", COUNT_ITEMS);
        let attributes = vec![
            colors
                .iter()
                .map(|c| (c.clone(), false))
                .collect::<Vec<_>>(),
            materials,
            sizes,
        ];
        let mut objects = Vec::new();
        for (one, many) in ATTR_OBJECTS {
            if panel.has_strict(one) || panel.has_strict(many) {
                dropped
                    .entry("object".into())
                    .or_default()
                    .push(one.to_string());
            } else {
                objects.push((one.to_string(), many.to_string(), panel.shares_content(one)));
            }
        }
        let mut containers = Vec::new();
        for (noun, prep) in COUNT_CONTAINERS {
            if panel.has_strict(noun) {
                dropped
                    .entry("container".into())
                    .or_default()
                    .push(noun.to_string());
            } else {
                containers.push((
                    noun.to_string(),
                    prep.to_string(),
                    panel.shares_content(noun),
                ));
            }
        }
        let mut value_ok = |kind: &str, item: &str| -> bool {
            let ok = !content_stems(item).is_empty()
                && !panel.shares_content(item)
                && !panel.has_strict(item);
            if !ok {
                dropped
                    .entry(kind.to_owned())
                    .or_default()
                    .push(item.to_owned());
            }
            ok
        };
        let rule_acts: Vec<(String, String)> = RULE_ACTS
            .iter()
            .filter(|(g, b)| value_ok("rule_activity", &format!("{g} {b}")))
            .map(|(g, b)| (g.to_string(), b.to_string()))
            .collect();
        let mut count_values: Vec<String> = (13..=60).map(|n| n.to_string()).collect();
        count_values.extend(COUNT_WORDS.iter().map(|w| w.to_string()));
        count_values.retain(|v| value_ok("count", v));
        let order_events: Vec<String> = ORDER_EVENTS
            .iter()
            .filter(|e| value_ok("order_event", e))
            .map(|e| e.to_string())
            .collect();
        let pools = Self {
            rule_settings,
            rule_acts,
            plan_keys,
            objects,
            attributes,
            containers,
            count_items,
            count_values,
            order_events,
            dropped,
        };
        for (name, n) in [
            ("rule settings", pools.rule_settings.len()),
            ("rule activities", pools.rule_acts.len()),
            ("objects", pools.objects.len()),
            ("containers", pools.containers.len()),
            ("count items", pools.count_items.len()),
            ("count values", pools.count_values.len()),
            ("order events", pools.order_events.len()),
        ] {
            if n < 8 {
                return Err(format!(
                    "v2 pool {name} has {n} entries after the panel filter"
                ));
            }
        }
        Ok(pools)
    }

    fn sizes(&self) -> Value {
        json!({
            "rule_settings": self.rule_settings.len(),
            "rule_activities": self.rule_acts.len(),
            "plan_keys": self.plan_keys.iter().map(Vec::len).collect::<Vec<_>>(),
            "objects": self.objects.len(),
            "attributes_color_material_size": self.attributes.iter().map(Vec::len).collect::<Vec<_>>(),
            "containers": self.containers.len(),
            "count_items": self.count_items.len(),
            "count_values": self.count_values.len(),
            "order_events": self.order_events.len(),
        })
    }
}

impl Draw<'_> {
    /// A fresh key-type entry, honouring strict-key draws.
    fn pick_key_entry(&mut self, rng: &mut Rng, pool: &[(String, bool)]) -> Result<Option<String>> {
        for _ in 0..40 {
            let (item, shared) = pick(rng, pool)?;
            if (self.strict && *shared) || !self.fresh(item) {
                continue;
            }
            let item = item.clone();
            self.take(&item);
            return Ok(Some(item));
        }
        Ok(None)
    }

    /// A fresh value from an already filtered pool.
    fn pick_value_entry(&mut self, rng: &mut Rng, pool: &[String]) -> Result<Option<String>> {
        for _ in 0..40 {
            let item = pick(rng, pool)?;
            if !self.fresh(item) {
                continue;
            }
            let item = item.clone();
            self.take(&item);
            return Ok(Some(item));
        }
        Ok(None)
    }
}

/// A candidate question of a v2 family, already rendered.
struct QSpec {
    /// Questions of one dialogue use distinct slots.
    slot: usize,
    weight: f64,
    question: String,
    answer: String,
    category: Category,
    detail: &'static str,
    expect: Option<String>,
    forbid: Vec<String>,
    forbid_keys: Vec<String>,
    /// For an abstention: the never-stated key it asks about.
    unstated: Option<String>,
}

/// The words of `other` whose stems are not among `asked`'s: what tells the
/// two keys apart ("maroon" for "the maroon mug" against "the teal mug").
fn distinguishing(other: &str, asked: &str) -> String {
    let asked = content_stems(asked);
    words(other)
        .into_iter()
        .filter(|w| w.chars().count() > 1 && !is_stop(w) && !asked.contains(&stem(w)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Template slots from `name => value` pairs (any `ToString` value).
macro_rules! slots {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        $( map.insert(String::from($k), ($v).to_string()); )*
        map
    }};
}

fn fact(key: &str, asst: &str, value: &str, old: Option<String>) -> Fact {
    Fact {
        frame: usize::MAX,
        key: Key {
            user: key.to_owned(),
            asst: asst.to_owned(),
            panel_shared: false,
        },
        value: value.to_owned(),
        old,
        by_assistant: false,
    }
}

impl Builder<'_> {
    fn ack_plain(&mut self, rng: &mut Rng) -> Result<()> {
        self.assistant(
            pick(rng, &self.world().acks.plain)?.to_string(),
            "acknowledgement",
        );
        Ok(())
    }

    fn push_key(&mut self, user: &str, asst: &str) {
        self.keys.push(Key {
            user: user.to_owned(),
            asst: asst.to_owned(),
            panel_shared: self.draw.world.panel.shares_content(user),
        });
    }

    /// A People key, sometimes "my friend NAME".
    fn person(&mut self, rng: &mut Rng) -> Result<Option<Key>> {
        if chance(rng, 0.4) {
            let names = &self.draw.world.person_names;
            for _ in 0..40 {
                let name = pick(rng, names)?.clone();
                if self.draw.fresh(&name) && !self.draw.taken.contains("friend") {
                    self.draw.take(&name);
                    self.draw.take("friend");
                    return Ok(Some(Key {
                        user: format!("my friend {name}"),
                        asst: format!("your friend {name}"),
                        panel_shared: false,
                    }));
                }
            }
        }
        self.draw.key(rng, Kc::People)
    }
}

fn fam_rule(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let pools = &b.world().v2;
    let n = 1 + usize::from(chance(rng, 0.45));
    let mut rules: Vec<(String, (String, String), (String, String))> = Vec::new();
    for _ in 0..n {
        let Some(setting) = b.draw.pick_key_entry(rng, &pools.rule_settings)? else {
            return Ok(None);
        };
        let mut acts = Vec::new();
        for _ in 0..2 {
            let mut found = None;
            for _ in 0..40 {
                let (g, base) = pick(rng, &pools.rule_acts)?;
                if b.draw.fresh(g) && b.draw.fresh(base) {
                    b.draw.take(g);
                    b.draw.take(base);
                    found = Some((g.clone(), base.clone()));
                    break;
                }
            }
            let Some(act) = found else {
                return Ok(None);
            };
            acts.push(act);
        }
        let banned = acts.pop().ok_or("two activities")?;
        let allowed = acts.pop().ok_or("two activities")?;
        rules.push((setting, allowed, banned));
    }
    let mut sentences = Vec::new();
    for (s, (a, ab), (x, xb)) in &rules {
        let slots = slots! {"s" => s, "S" => &cap(s), "a" => a, "A" => &cap(a), "ab" => ab, "x" => x, "X" => &cap(x), "xb" => xb};
        sentences.push(fill(pick(rng, RULE_STATES)?, &slots)?);
        b.push_key(s, s);
        b.facts.push(fact(s, s, a, None));
        b.facts.push(fact(s, s, x, None));
    }
    if sentences.len() == 2 && chance(rng, 0.5) {
        b.user(sentences.join(" "));
        b.ack_plain(rng)?;
    } else {
        for s in sentences {
            b.user(s);
            b.ack_plain(rng)?;
        }
    }
    b.frames.push("rule");
    let mut specs = Vec::new();
    for (i, (s, (a, ab), (x, xb))) in rules.iter().enumerate() {
        let mut others: Vec<String> = Vec::new();
        let mut other_keys = Vec::new();
        for (j, (s2, (a2, ab2), (x2, xb2))) in rules.iter().enumerate() {
            if j != i {
                others.extend([a2.clone(), ab2.clone(), x2.clone(), xb2.clone()]);
                other_keys.push(distinguishing(s2, s));
            }
        }
        let (p, q) = if chance(rng, 0.5) { (a, x) } else { (x, a) };
        let slots = slots! {"s" => s, "S" => &cap(s), "a" => a, "A" => &cap(a), "x" => x, "X" => &cap(x), "p" => p, "q" => q};
        for (allowed, choice) in [(true, false), (false, false), (true, true), (false, true)] {
            let asks = match (allowed, choice) {
                (true, false) => RULE_ASK_ALLOWED,
                (false, false) => RULE_ASK_BANNED,
                (true, true) => RULE_ASK_ALLOWED_CHOICE,
                (false, true) => RULE_ASK_BANNED_CHOICE,
            };
            let answers = if allowed {
                RULE_ANSWER_ALLOWED
            } else {
                RULE_ANSWER_BANNED
            };
            let (expect, wrong, wrong_base) = if allowed { (a, x, xb) } else { (x, a, ab) };
            let mut forbid = vec![wrong.clone(), wrong_base.clone()];
            forbid.extend(others.iter().cloned());
            specs.push(QSpec {
                slot: i * 2 + usize::from(!allowed),
                weight: if choice { 0.12 } else { 0.33 },
                question: fill(pick(rng, asks)?, &slots)?,
                answer: fill(pick(rng, answers)?, &slots)?,
                category: Category::Rule,
                detail: match (allowed, choice) {
                    (true, false) => "allowed",
                    (false, false) => "not_allowed",
                    (true, true) => "allowed_choice",
                    (false, true) => "not_allowed_choice",
                },
                expect: Some(expect.clone()),
                forbid,
                forbid_keys: other_keys.clone(),
                unstated: None,
            });
        }
    }
    if let Some(s3) = b.draw.pick_key_entry(rng, &pools.rule_settings)? {
        let slots = slots! {"s" => &s3, "S" => &cap(&s3)};
        let forbid = rules
            .iter()
            .flat_map(|(_, (a, ab), (x, xb))| [a.clone(), ab.clone(), x.clone(), xb.clone()])
            .collect();
        specs.push(QSpec {
            slot: 90,
            weight: 0.13,
            question: fill(pick(rng, RULE_ASK_ALLOWED)?, &slots)?,
            answer: fill(pick(rng, RULE_ABSTAIN)?, &slots)?,
            category: Category::Abstain,
            detail: "rule_unstated_setting",
            expect: None,
            forbid,
            forbid_keys: rules
                .iter()
                .map(|(s, _, _)| distinguishing(s, &s3))
                .collect(),
            unstated: Some(s3.clone()),
        });
        b.push_key(&s3, &s3);
    }
    Ok(Some(specs))
}

fn fam_implicit(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let kind_index = {
        let r = rng.below(100);
        if r < 45 {
            0
        } else if r < 75 {
            1
        } else {
            2
        }
    };
    let kind = &PLAN_KINDS[kind_index];
    let key_pool = &b.world().v2.plan_keys[kind_index];
    let n = 1 + usize::from(chance(rng, 0.35));
    let mut plans: Vec<(String, Vec<String>)> = Vec::new();
    for _ in 0..n {
        let Some(key) = b.draw.pick_key_entry(rng, key_pool)? else {
            return Ok(None);
        };
        let Some(v) = b.draw.value(rng, kind.values)? else {
            return Ok(None);
        };
        plans.push((key, vec![v]));
    }
    let plan_forms = |k: &str| {
        let ka = if let Some(rest) = k.strip_prefix("our ") {
            format!("your {rest}")
        } else {
            k.to_owned()
        };
        (k.to_owned(), ka)
    };
    let mut initial = Vec::new();
    for (k, values) in &plans {
        let (k, ka) = plan_forms(k);
        let slots = slots! {"k" => &k, "K" => &cap(&k), "ka" => &ka, "v" => &values[0]};
        initial.push(fill(pick(rng, kind.initial)?, &slots)?);
    }
    if initial.len() == 2 && chance(rng, 0.6) {
        b.user(initial.join(" "));
        b.ack_plain(rng)?;
    } else {
        for s in initial {
            b.user(s);
            b.ack_plain(rng)?;
        }
    }
    if chance(rng, 0.4) {
        b.filler(rng)?;
    }
    let target = rng.below(n);
    let changes = 1 + usize::from(chance(rng, 0.3));
    for _ in 0..changes {
        let Some(v) = b.draw.value(rng, kind.values)? else {
            return Ok(None);
        };
        let (k, _) = plan_forms(&plans[target].0);
        let slots = slots! {"k" => &k, "K" => &cap(&k), "v" => &v};
        let template = if n == 1 && chance(rng, 0.75) {
            pick(rng, kind.change)?
        } else {
            pick(rng, kind.change_named)?
        };
        b.user(fill(template, &slots)?);
        if chance(rng, 0.5) {
            b.assistant(
                pick(rng, &b.world().acks.update)?.to_string(),
                "acknowledgement",
            );
        } else {
            b.ack_plain(rng)?;
        }
        plans[target].1.push(v);
    }
    b.frames.push("implicit_update");
    let all_values: Vec<String> = plans.iter().flat_map(|(_, v)| v.clone()).collect();
    for (k, values) in &plans {
        let (k, ka) = plan_forms(k);
        b.push_key(&k, &ka);
        let latest = values.last().ok_or("a plan without values")?;
        let old = (values.len() > 1).then(|| values[0].clone());
        b.facts.push(fact(&k, &ka, latest, old));
    }
    let mut specs = Vec::new();
    for (i, (k, values)) in plans.iter().enumerate() {
        let (k, ka) = plan_forms(k);
        let other_keys: Vec<String> = plans
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, (k2, _))| distinguishing(k2, &k))
            .collect();
        let forbid_except = |keep: &str| -> Vec<String> {
            all_values
                .iter()
                .filter(|v| v.as_str() != keep)
                .cloned()
                .collect()
        };
        let latest = values.last().ok_or("a plan without values")?;
        let slots = slots! {"k" => &k, "K" => &cap(&k), "ka" => &ka, "KA" => &cap(&ka), "v" => latest, "V" => &cap(latest)};
        let changed = i == target;
        specs.push(QSpec {
            slot: i * 2,
            weight: if changed { 0.55 } else { 0.15 },
            question: fill(pick(rng, kind.ask_latest)?, &slots)?,
            answer: fill(pick(rng, kind.answer_latest)?, &slots)?,
            category: Category::ImplicitUpdate,
            detail: if changed { "latest" } else { "unchanged" },
            expect: Some(latest.clone()),
            forbid: forbid_except(latest),
            forbid_keys: other_keys.clone(),
            unstated: None,
        });
        if changed {
            let first = &values[0];
            let slots =
                slots! {"k" => &k, "K" => &cap(&k), "ka" => &ka, "v" => first, "V" => &cap(first)};
            specs.push(QSpec {
                slot: i * 2 + 1,
                weight: 0.25,
                question: fill(pick(rng, kind.ask_first)?, &slots)?,
                answer: fill(pick(rng, kind.answer_first)?, &slots)?,
                category: Category::ImplicitUpdate,
                detail: "first",
                expect: Some(first.clone()),
                forbid: forbid_except(first),
                forbid_keys: other_keys,
                unstated: None,
            });
        }
    }
    if let Some(k3) = b.draw.pick_key_entry(rng, key_pool)? {
        let (k3, ka3) = plan_forms(&k3);
        let slots = slots! {"k" => &k3, "K" => &cap(&k3), "ka" => &ka3};
        specs.push(QSpec {
            slot: 90,
            weight: 0.1,
            question: fill(pick(rng, kind.ask_latest)?, &slots)?,
            answer: fill(pick(rng, ABSTAIN_ANSWERS)?, &slots)?,
            category: Category::Abstain,
            detail: "plan_unstated",
            expect: None,
            forbid: all_values.clone(),
            forbid_keys: plans.iter().map(|(k, _)| distinguishing(k, &k3)).collect(),
            unstated: Some(k3.clone()),
        });
        b.push_key(&k3, &ka3);
    }
    Ok(Some(specs))
}

fn self_slots(rel: &SelfRel, key: Option<&Key>, value: &str) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    if let Some(key) = key {
        key_slots(&mut slots, "", key);
    }
    value_slots(&mut slots, "", value, rel.art);
    slots
}

fn fam_self(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let r = rng.below(SELF_RELS.len());
    let rel = &SELF_RELS[r];
    let Some(other) = b.person(rng)? else {
        return Ok(None);
    };
    let (Some(mine), Some(theirs)) = (
        b.draw.value(rng, rel.values)?,
        b.draw.value(rng, rel.values)?,
    ) else {
        return Ok(None);
    };
    // Sometimes a second first-person fact of another relation.
    let second = if chance(rng, 0.35) {
        let r2 = (r + 1 + rng.below(SELF_RELS.len() - 1)) % SELF_RELS.len();
        match b.draw.value(rng, SELF_RELS[r2].values)? {
            Some(v) => Some((r2, v)),
            None => return Ok(None),
        }
    } else {
        None
    };
    let me = fill(rel.me, &self_slots(rel, None, &mine))?;
    let joined = chance(rng, 0.6);
    let other_template = if joined {
        rel.other.replace("{K}", "{k}").replace("{Kp}", "{kp}")
    } else {
        rel.other.to_owned()
    };
    let them = fill(&other_template, &self_slots(rel, Some(&other), &theirs))?;
    if joined {
        let me_clause = me.trim_end_matches('.');
        if chance(rng, 0.7) {
            b.user(format!("{me_clause}, and {them}"));
        } else {
            b.user(format!(
                "{}, and {}",
                cap(them.trim_end_matches('.')),
                lower_first_unless_i(&me)
            ));
        }
        b.ack_plain(rng)?;
    } else if chance(rng, 0.5) {
        b.user(me.clone());
        b.ack_plain(rng)?;
        b.user(cap(&them));
        b.ack_plain(rng)?;
    } else {
        b.user(format!("{} {}", cap(&them), me));
        b.ack_plain(rng)?;
    }
    if let Some((r2, v2)) = &second {
        b.user(fill(
            SELF_RELS[*r2].me,
            &self_slots(&SELF_RELS[*r2], None, v2),
        )?);
        b.ack_plain(rng)?;
    }
    b.frames.push("self_fact");
    b.frames.push(rel.id);
    b.keys.push(other.clone());
    b.facts.push(fact("I", "you", &mine, None));
    b.facts.push(fact(&other.user, &other.asst, &theirs, None));
    let mut stated_values = vec![mine.clone(), theirs.clone()];
    if let Some((_, v2)) = &second {
        b.facts.push(fact("I", "you", v2, None));
        stated_values.push(v2.clone());
    }
    let except = |keep: &str| -> Vec<String> {
        stated_values
            .iter()
            .filter(|v| v.as_str() != keep)
            .cloned()
            .collect()
    };
    let other_words = vec![distinguishing(&other.asst, "you")];
    let mut specs = Vec::new();
    let s = self_slots(rel, None, &mine);
    specs.push(QSpec {
        slot: 0,
        weight: 0.5,
        question: fill(pick(rng, rel.ask_me)?, &s)?,
        answer: fill(pick(rng, rel.answer_me)?, &s)?,
        category: Category::SelfFact,
        detail: "self",
        expect: Some(mine.clone()),
        forbid: except(&mine),
        forbid_keys: other_words.clone(),
        unstated: None,
    });
    let s = self_slots(rel, Some(&other), &theirs);
    specs.push(QSpec {
        slot: 1,
        weight: 0.3,
        question: fill(pick(rng, rel.ask_other)?, &s)?,
        answer: fill(pick(rng, rel.answer_other)?, &s)?,
        category: Category::SelfFact,
        detail: "other",
        expect: Some(theirs.clone()),
        forbid: except(&theirs),
        forbid_keys: vec![],
        unstated: None,
    });
    if let Some((r2, v2)) = &second {
        let rel2 = &SELF_RELS[*r2];
        let s = self_slots(rel2, None, v2);
        specs.push(QSpec {
            slot: 2,
            weight: 0.2,
            question: fill(pick(rng, rel2.ask_me)?, &s)?,
            answer: fill(pick(rng, rel2.answer_me)?, &s)?,
            category: Category::SelfFact,
            detail: "self",
            expect: Some(v2.clone()),
            forbid: except(v2),
            forbid_keys: other_words.clone(),
            unstated: None,
        });
    }
    // Abstentions only for keys that were truly never stated: the user's own
    // fact of an unstated relation (rarely), or a third person.
    let stated_rels: Vec<usize> = std::iter::once(r)
        .chain(second.iter().map(|(r2, _)| *r2))
        .collect();
    let unstated: Vec<usize> = (0..SELF_RELS.len())
        .filter(|i| !stated_rels.contains(i))
        .collect();
    let r3 = *pick(rng, &unstated)?;
    let s = self_slots(&SELF_RELS[r3], None, "x");
    specs.push(QSpec {
        slot: 3,
        weight: 0.04,
        question: fill(pick(rng, SELF_RELS[r3].ask_me)?, &s)?,
        answer: pick(rng, SELF_ABSTAIN)?.to_string(),
        category: Category::Abstain,
        detail: "self_unstated_relation",
        expect: None,
        forbid: stated_values.clone(),
        forbid_keys: other_words.clone(),
        unstated: Some(String::new()),
    });
    if let Some(third) = b.draw.key(rng, Kc::People)? {
        let s = self_slots(rel, Some(&third), "x");
        specs.push(QSpec {
            slot: 4,
            weight: 0.09,
            question: fill(pick(rng, rel.ask_other)?, &s)?,
            answer: fill(pick(rng, ABSTAIN_ANSWERS)?, &s)?,
            category: Category::Abstain,
            detail: "other_unstated_person",
            expect: None,
            forbid: stated_values.clone(),
            forbid_keys: vec![distinguishing(&other.asst, &third.asst)],
            unstated: Some(third.user.clone()),
        });
        b.keys.push(third);
    }
    Ok(Some(specs))
}

/// `text` with its first letter lowered unless it starts with the pronoun I.
fn lower_first_unless_i(text: &str) -> String {
    if text.starts_with("I ") || text.starts_with("I'") {
        text.to_owned()
    } else {
        lower_first(text)
    }
}

fn fam_attribute(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let pools = &b.world().v2;
    let mut object = None;
    for _ in 0..40 {
        let (one, many, shared) = pick(rng, &pools.objects)?;
        if (b.draw.strict && *shared) || !b.draw.fresh(one) {
            continue;
        }
        b.draw.take(one);
        b.draw.take(many);
        object = Some((one.clone(), many.clone()));
        break;
    }
    let Some((o, os)) = object else {
        return Ok(None);
    };
    let dimension = rng.below(pools.attributes.len());
    let n = 2 + usize::from(chance(rng, 0.25));
    let mut attrs = Vec::new();
    for _ in 0..n + 1 {
        let Some(a) = b.draw.pick_key_entry(rng, &pools.attributes[dimension])? else {
            return Ok(None);
        };
        attrs.push(a);
    }
    let unstated = attrs.pop().ok_or("an unstated attribute")?;
    let owner = chance(rng, 0.4);
    let mut values = Vec::new();
    for _ in 0..n {
        let v = if owner {
            b.draw.value(rng, Vc::PersonName)?
        } else {
            b.draw.value(rng, Vc::Spot)?
        };
        let Some(v) = v else {
            return Ok(None);
        };
        values.push(v);
    }
    let pair = slots! {"o" => &o, "os" => &os, "a1" => &attrs[0], "a2" => &attrs[1], "l1" => &values[0], "l2" => &values[1], "n1" => &values[0], "N1" => &cap(&values[0]), "n2" => &values[1]};
    let states = if owner {
        ATTR_OWN_STATES
    } else {
        ATTR_LOC_STATES
    };
    b.user(fill(pick(rng, states)?, &pair)?);
    b.ack_plain(rng)?;
    if n == 3 {
        let slots = slots! {"o" => &o, "a" => &attrs[2], "l" => &values[2], "n" => &values[2]};
        let third = if owner {
            ATTR_OWN_THIRD
        } else {
            ATTR_LOC_THIRD
        };
        b.user(fill(pick(rng, third)?, &slots)?);
        b.ack_plain(rng)?;
    }
    b.frames.push("attribute");
    let keys: Vec<String> = attrs.iter().map(|a| format!("the {a} {o}")).collect();
    for (k, v) in keys.iter().zip(&values) {
        b.push_key(k, k);
        b.facts.push(fact(k, k, v, None));
    }
    let mut specs = Vec::new();
    for i in 0..n {
        let others: Vec<String> = (0..n)
            .filter(|j| *j != i)
            .map(|j| values[j].clone())
            .collect();
        let other_attrs: Vec<String> = (0..n)
            .filter(|j| *j != i)
            .map(|j| attrs[j].clone())
            .collect();
        let slots = slots! {"o" => &o, "a" => &attrs[i], "l" => &values[i], "L" => &cap(&values[i]), "n" => &values[i], "N" => &cap(&values[i])};
        let (asks, answers) = if owner {
            (ATTR_OWN_ASK, ATTR_OWN_ANSWER)
        } else {
            (ATTR_LOC_ASK, ATTR_LOC_ANSWER)
        };
        specs.push(QSpec {
            slot: i,
            weight: 0.8 / n as f64,
            question: fill(pick(rng, asks)?, &slots)?,
            answer: fill(pick(rng, answers)?, &slots)?,
            category: Category::Attribute,
            detail: if owner { "owner" } else { "location" },
            expect: Some(values[i].clone()),
            forbid: others.clone(),
            forbid_keys: other_attrs.clone(),
            unstated: None,
        });
        let reverse = if owner {
            ATTR_REVERSE_OWN
        } else {
            ATTR_REVERSE_LOC
        };
        specs.push(QSpec {
            slot: 10 + i,
            weight: 0.12 / n as f64,
            question: fill(pick(rng, reverse)?, &slots)?,
            answer: fill(pick(rng, ATTR_REVERSE_ANSWER)?, &slots)?,
            category: Category::Attribute,
            detail: "reverse",
            expect: Some(attrs[i].clone()),
            forbid: others,
            forbid_keys: other_attrs,
            unstated: None,
        });
    }
    let slots = slots! {"o" => &o, "os" => &os, "a" => &unstated, "aa" => &article(&unstated, Art::Indef), "a1" => &attrs[0], "a2" => &attrs[1]};
    let asks = if owner { ATTR_OWN_ASK } else { ATTR_LOC_ASK };
    let answers: &[&str] = if n == 2 {
        ATTR_ABSTAIN
    } else {
        &ATTR_ABSTAIN[..2]
    };
    specs.push(QSpec {
        slot: 90,
        weight: 0.1,
        question: fill(pick(rng, asks)?, &slots)?,
        answer: fill(pick(rng, answers)?, &slots)?,
        category: Category::Abstain,
        detail: "attribute_unstated",
        expect: None,
        forbid: values.clone(),
        forbid_keys: vec![],
        unstated: Some(format!("the {unstated} {o}")),
    });
    let k = format!("the {unstated} {o}");
    b.push_key(&k, &k);
    Ok(Some(specs))
}

fn fam_count(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let pools = &b.world().v2;
    let Some(items) = b.draw.pick_key_entry(rng, &pools.count_items)? else {
        return Ok(None);
    };
    let n = 2 + usize::from(chance(rng, 0.25));
    let mut counts = Vec::new();
    for _ in 0..n {
        let Some(v) = b.draw.pick_value_entry(rng, &pools.count_values)? else {
            return Ok(None);
        };
        counts.push(v);
    }
    b.frames.push("count");
    let by_person = chance(rng, 0.4);
    let mut specs = Vec::new();
    if by_person {
        let mut people = Vec::new();
        for _ in 0..n + 1 {
            let Some(p) = b.person(rng)? else {
                return Ok(None);
            };
            people.push(p);
        }
        let unstated = people.pop().ok_or("an unstated person")?;
        let mut slots = slots! {"it" => &items, "v1" => &counts[0], "v2" => &counts[1]};
        key_slots(&mut slots, "1", &people[0]);
        key_slots(&mut slots, "2", &people[1]);
        b.user(fill(pick(rng, COUNT_PERSON_STATES)?, &slots)?);
        b.ack_plain(rng)?;
        if n == 3 {
            let mut slots = slots! {"v" => &counts[2]};
            key_slots(&mut slots, "", &people[2]);
            b.user(fill(pick(rng, COUNT_PERSON_THIRD)?, &slots)?);
            b.ack_plain(rng)?;
        }
        for (p, v) in people.iter().zip(&counts) {
            b.keys.push(p.clone());
            b.facts.push(fact(&p.user, &p.asst, v, None));
        }
        for i in 0..n {
            let mut slots = slots! {"it" => &items, "v" => &counts[i], "V" => &cap(&counts[i])};
            key_slots(&mut slots, "", &people[i]);
            specs.push(QSpec {
                slot: i,
                weight: 0.9 / n as f64,
                question: fill(pick(rng, COUNT_PERSON_ASK)?, &slots)?,
                answer: fill(pick(rng, COUNT_PERSON_ANSWER)?, &slots)?,
                category: Category::Count,
                detail: "per_person",
                expect: Some(counts[i].clone()),
                forbid: (0..n)
                    .filter(|j| *j != i)
                    .map(|j| counts[j].clone())
                    .collect(),
                forbid_keys: (0..n)
                    .filter(|j| *j != i)
                    .map(|j| distinguishing(&people[j].asst, &people[i].asst))
                    .collect(),
                unstated: None,
            });
        }
        let mut slots = slots! {"it" => &items};
        key_slots(&mut slots, "", &unstated);
        specs.push(QSpec {
            slot: 90,
            weight: 0.1,
            question: fill(pick(rng, COUNT_PERSON_ASK)?, &slots)?,
            answer: fill(pick(rng, ABSTAIN_ANSWERS)?, &slots)?,
            category: Category::Abstain,
            detail: "count_unstated",
            expect: None,
            forbid: counts.clone(),
            forbid_keys: people
                .iter()
                .map(|p| distinguishing(&p.asst, &unstated.asst))
                .collect(),
            unstated: Some(unstated.user.clone()),
        });
        b.keys.push(unstated);
        return Ok(Some(specs));
    }
    // Containers: the same noun told apart by an attribute, or different nouns.
    let same_noun = chance(rng, 0.6);
    let mut containers: Vec<(String, String)> = Vec::new();
    let mut shared_noun: Option<(String, String)> = None;
    for _ in 0..n + 1 {
        let entry = if same_noun {
            if shared_noun.is_none() {
                let mut found = None;
                for _ in 0..40 {
                    let (noun, prep, shared) = pick(rng, &pools.containers)?;
                    if (b.draw.strict && *shared) || !b.draw.fresh(noun) {
                        continue;
                    }
                    b.draw.take(noun);
                    found = Some((noun.clone(), prep.clone()));
                    break;
                }
                shared_noun = found;
            }
            let Some((noun, prep)) = shared_noun.clone() else {
                return Ok(None);
            };
            let dim = &pools.attributes[0];
            let Some(a) = b.draw.pick_key_entry(rng, dim)? else {
                return Ok(None);
            };
            (format!("the {a} {noun}"), prep)
        } else {
            let mut found = None;
            for _ in 0..40 {
                let (noun, prep, shared) = pick(rng, &pools.containers)?;
                if (b.draw.strict && *shared) || !b.draw.fresh(noun) {
                    continue;
                }
                b.draw.take(noun);
                found = Some((format!("the {noun}"), prep.clone()));
                break;
            }
            let Some(entry) = found else {
                return Ok(None);
            };
            entry
        };
        containers.push(entry);
    }
    let (unstated, unstated_prep) = containers.pop().ok_or("an unstated container")?;
    let slots = slots! {"it" => &items, "v1" => &counts[0], "v2" => &counts[1], "c1" => &containers[0].0, "C1" => &cap(&containers[0].0), "p1" => &containers[0].1, "c2" => &containers[1].0, "p2" => &containers[1].1};
    b.user(fill(pick(rng, COUNT_STATES)?, &slots)?);
    b.ack_plain(rng)?;
    if n == 3 {
        let slots = slots! {"v" => &counts[2], "c" => &containers[2].0, "p" => &containers[2].1};
        b.user(fill(pick(rng, COUNT_THIRD)?, &slots)?);
        b.ack_plain(rng)?;
    }
    for ((c, _), v) in containers.iter().zip(&counts) {
        b.push_key(c, c);
        b.facts.push(fact(c, c, v, None));
    }
    for i in 0..n {
        let (c, p) = &containers[i];
        let slots = slots! {"it" => &items, "v" => &counts[i], "V" => &cap(&counts[i]), "c" => c, "C" => &cap(c), "p" => p};
        specs.push(QSpec {
            slot: i,
            weight: 0.9 / n as f64,
            question: fill(pick(rng, COUNT_ASK)?, &slots)?,
            answer: fill(pick(rng, COUNT_ANSWER)?, &slots)?,
            category: Category::Count,
            detail: if same_noun {
                "per_container_attribute"
            } else {
                "per_container"
            },
            expect: Some(counts[i].clone()),
            forbid: (0..n)
                .filter(|j| *j != i)
                .map(|j| counts[j].clone())
                .collect(),
            forbid_keys: (0..n)
                .filter(|j| *j != i)
                .map(|j| distinguishing(&containers[j].0, c))
                .collect(),
            unstated: None,
        });
    }
    let slots = slots! {"it" => &items, "c" => &unstated, "ka" => &unstated, "p" => &unstated_prep};
    specs.push(QSpec {
        slot: 90,
        weight: 0.1,
        question: fill(pick(rng, COUNT_ASK)?, &slots)?,
        answer: fill(pick(rng, ABSTAIN_ANSWERS)?, &slots)?,
        category: Category::Abstain,
        detail: "count_unstated",
        expect: None,
        forbid: counts.clone(),
        forbid_keys: containers
            .iter()
            .map(|(c, _)| distinguishing(c, &unstated))
            .collect(),
        unstated: Some(unstated.clone()),
    });
    b.push_key(&unstated, &unstated);
    Ok(Some(specs))
}

fn fam_order(b: &mut Builder<'_>, rng: &mut Rng) -> Result<Option<Vec<QSpec>>> {
    let pools = &b.world().v2;
    let variant = rng.below(4); // 0-1 plain, 2 day parts, 3 times
    let n = if variant <= 1 && chance(rng, 0.3) {
        4
    } else {
        3
    };
    let mut events = Vec::new();
    for _ in 0..n {
        let Some(e) = b.draw.pick_value_entry(rng, &pools.order_events)? else {
            return Ok(None);
        };
        events.push(e);
    }
    let mut anchors: Vec<String> = Vec::new();
    let mut slots = BTreeMap::new();
    for (i, e) in events.iter().enumerate() {
        slots.insert(format!("e{}", i + 1), e.clone());
    }
    let template = match variant {
        2 => {
            let (template, parts) = pick(rng, ORDER_PARTS)?;
            anchors = parts.iter().map(|p| p.to_string()).collect();
            template.to_string()
        }
        3 => {
            let mut times = Vec::new();
            for _ in 0..4 {
                let Some(t) = b.draw.value(rng, Vc::Time)? else {
                    return Ok(None);
                };
                times.push(t);
            }
            let spare = times.pop().ok_or("a spare time")?;
            times.sort();
            anchors = times;
            anchors.push(spare);
            pick(rng, ORDER_TIMES)?.to_string()
        }
        _ if n == 4 => pick(rng, ORDER_PLAIN_4)?.to_string(),
        _ => pick(rng, ORDER_PLAIN_3)?.to_string(),
    };
    for (i, t) in anchors.iter().take(3).enumerate() {
        slots.insert(format!("t{}", i + 1), t.clone());
    }
    b.user(cap(&fill(&template, &slots)?));
    b.ack_plain(rng)?;
    b.frames.push("order");
    for (i, e) in events.iter().enumerate() {
        let key = anchors.get(i).cloned().unwrap_or_default();
        if !key.is_empty() {
            b.push_key(&key, &key);
        }
        b.facts.push(fact(&key, &key, e, None));
    }
    let except = |i: usize| -> Vec<String> {
        events
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, e)| e.clone())
            .collect()
    };
    let mut specs = Vec::new();
    for i in 0..n {
        let e = &events[i];
        let rank = if i == n - 1 { 3 } else { i };
        let s = slots! {"e" => e, "E" => &cap(e)};
        specs.push(QSpec {
            slot: i,
            weight: 0.35 / n as f64,
            question: fill(pick(rng, ORDER_ASK_RANK[rank])?, &s)?,
            answer: fill(pick(rng, ORDER_ANSWER_RANK[rank])?, &s)?,
            category: Category::Order,
            detail: ["first", "second", "third", "last"][rank],
            expect: Some(e.clone()),
            forbid: except(i),
            forbid_keys: vec![],
            unstated: None,
        });
        if i > 0 {
            let s = slots! {"e" => e, "E" => &cap(e), "p" => &events[i - 1]};
            specs.push(QSpec {
                slot: 10 + i,
                weight: 0.2 / (n - 1) as f64,
                question: fill(pick(rng, ORDER_ASK_AFTER)?, &s)?,
                answer: fill(pick(rng, ORDER_ANSWER_AFTER)?, &s)?,
                category: Category::Order,
                detail: "after",
                expect: Some(e.clone()),
                forbid: except(i),
                forbid_keys: vec![],
                unstated: None,
            });
        }
        if i + 1 < n {
            let s = slots! {"e" => e, "E" => &cap(e), "p" => &events[i + 1]};
            specs.push(QSpec {
                slot: 20 + i,
                weight: 0.15 / (n - 1) as f64,
                question: fill(pick(rng, ORDER_ASK_BEFORE)?, &s)?,
                answer: fill(pick(rng, ORDER_ANSWER_BEFORE)?, &s)?,
                category: Category::Order,
                detail: "before",
                expect: Some(e.clone()),
                forbid: except(i),
                forbid_keys: vec![],
                unstated: None,
            });
        }
        if variant >= 2 && i < 3 {
            let t = &anchors[i];
            let s = slots! {"e" => e, "E" => &cap(e), "t" => t, "T" => &cap(t)};
            let (asks, answers, detail) = if variant == 2 {
                (ORDER_ASK_PART, ORDER_ANSWER_PART, "day_part")
            } else {
                (ORDER_ASK_TIME, ORDER_ANSWER_TIME, "at_time")
            };
            specs.push(QSpec {
                slot: 30 + i,
                weight: 0.3 / n as f64,
                question: fill(pick(rng, asks)?, &s)?,
                answer: fill(pick(rng, answers)?, &s)?,
                category: Category::Order,
                detail,
                expect: Some(e.clone()),
                forbid: except(i),
                forbid_keys: (0..3)
                    .filter(|j| *j != i)
                    .map(|j| distinguishing(&anchors[j], t))
                    .collect(),
                unstated: None,
            });
        }
    }
    if variant == 3 {
        let t = &anchors[3];
        let s = slots! {"t" => t};
        specs.push(QSpec {
            slot: 90,
            weight: 0.12,
            question: fill(pick(rng, ORDER_ASK_TIME)?, &s)?,
            answer: fill(pick(rng, ORDER_ABSTAIN)?, &s)?,
            category: Category::Abstain,
            detail: "order_unstated_time",
            expect: None,
            forbid: events.clone(),
            forbid_keys: anchors[..3].iter().map(|a| distinguishing(a, t)).collect(),
            unstated: Some(t.clone()),
        });
    }
    Ok(Some(specs))
}

/// One v2-family dialogue: optional distractors, the family's statements,
/// more distractors, then one to three of its questions.
fn draw_family(world: &World, rng: &mut Rng, family: Category) -> Result<Drawn> {
    let strict = chance(rng, STRICT_KEY_SHARE);
    let mut b = Builder {
        draw: Draw {
            world,
            taken: BTreeSet::new(),
            strict,
        },
        turns: Vec::new(),
        facts: Vec::new(),
        keys: Vec::new(),
        frames: Vec::new(),
        questions: Vec::new(),
        used_fillers: BTreeSet::new(),
    };
    let distractors = rng.below(MAX_DISTRACTORS + 1);
    let pre = usize::from(distractors > 0 && chance(rng, 0.25));
    b.fillers(rng, pre)?;
    let specs = match family {
        Category::Rule => fam_rule(&mut b, rng)?,
        Category::ImplicitUpdate => fam_implicit(&mut b, rng)?,
        Category::SelfFact => fam_self(&mut b, rng)?,
        Category::Attribute => fam_attribute(&mut b, rng)?,
        Category::Count => fam_count(&mut b, rng)?,
        Category::Order => fam_order(&mut b, rng)?,
        _ => return Err("not a v2 family".into()),
    };
    let Some(specs) = specs else {
        return Ok(Drawn::Redraw("key_or_value_pool"));
    };
    let remaining = distractors - pre;
    let questions = {
        let r = rng.below(100);
        if r < 60 {
            1
        } else if r < 88 {
            2
        } else {
            3
        }
    };
    let between = usize::from(questions > 1 && remaining > 0 && chance(rng, 0.5));
    b.fillers(rng, remaining - between)?;
    let mut used = BTreeSet::new();
    let mut abstained = false;
    for i in 0..questions {
        let open: Vec<usize> = (0..specs.len())
            .filter(|&j| {
                !used.contains(&specs[j].slot) && !(abstained && specs[j].expect.is_none())
            })
            .collect();
        if open.is_empty() {
            break;
        }
        let total: f64 = open.iter().map(|&j| specs[j].weight).sum();
        let mut x = ((rng.next_u64() >> 11) as f64) / ((1u64 << 53) as f64) * total;
        let mut chosen = open[open.len() - 1];
        for &j in &open {
            if x < specs[j].weight {
                chosen = j;
                break;
            }
            x -= specs[j].weight;
        }
        if i == 1 && between > 0 {
            b.fillers(rng, between)?;
        }
        let spec = &specs[chosen];
        used.insert(spec.slot);
        abstained |= spec.expect.is_none();
        let question = b.question_text(rng, spec.question.clone())?;
        b.user(question);
        b.assistant(spec.answer.clone(), "answer");
        b.questions.push(Question {
            turn: b.turns.len() - 1,
            category: spec.category,
            followup: i > 0,
            detail: spec.detail,
            expect: spec.expect.clone(),
            forbid: spec.forbid.clone(),
            forbid_keys: spec
                .forbid_keys
                .iter()
                .filter(|k| !k.is_empty())
                .cloned()
                .collect(),
            unstated_key: spec.unstated.clone(),
        });
    }
    let dialogue = Dialogue {
        category: family,
        frames: b.frames,
        strict_keys: strict,
        turns: b.turns,
        questions: b.questions,
        facts: b.facts,
        keys: b.keys,
    };
    finalize(world, dialogue)
}

/// A `generator=v2` draw: a v1 dialogue or one of the six families.
fn draw_v2(world: &World, rng: &mut Rng) -> Result<Drawn> {
    if chance(rng, V1_SHARE) {
        return draw_dialogue(world, rng);
    }
    let family = FAMILIES[rng.below(FAMILIES.len())];
    draw_family(world, rng, family)
}

/// The checks every drawn dialogue passes: redraw on a leak or collision,
/// then the semantic contract must hold.
fn finalize(world: &World, dialogue: Dialogue) -> Result<Drawn> {
    if let Some(reason) = collision(&dialogue) {
        return Ok(Drawn::Redraw(reason));
    }
    for turn in &dialogue.turns {
        if let Some(reason) = world.panel.turn_leak(&turn.text) {
            return Ok(Drawn::Redraw(reason));
        }
    }
    // A key or value word that coincides with an answer template's own words
    // ("the school play" against "plays the viola") would make a correct
    // answer look like it names another key: redraw such a dialogue.
    for q in &dialogue.questions {
        let answer = content_stems(&dialogue.turns[q.turn].text);
        if q.forbid
            .iter()
            .chain(&q.forbid_keys)
            .any(|other| content_stems(other).iter().any(|s| answer.contains(s)))
        {
            return Ok(Drawn::Redraw("key_or_value_word_in_answer_template"));
        }
    }
    check_dialogue(&dialogue)?;
    Ok(Drawn::Ok(dialogue))
}

// ---------------------------------------------------------------------------
// Splits.

#[derive(Default)]
struct Tally {
    dialogues: usize,
    turns: usize,
    questions: BTreeMap<String, usize>,
    question_details: BTreeMap<String, usize>,
    primary: BTreeMap<String, usize>,
    frames: BTreeMap<String, usize>,
    redraws: BTreeMap<String, usize>,
    strict_key_dialogues: usize,
    distractor_exchanges: BTreeMap<usize, usize>,
    questions_per_dialogue: BTreeMap<usize, usize>,
    /// Value stems shared with the panels (must stay empty).
    value_stems_shared: BTreeSet<String>,
    /// Key words shared with the panels, with dialogue counts.
    key_words_shared: BTreeMap<String, usize>,
    dialogues_with_panel_key: usize,
    tokens: usize,
    response_tokens: usize,
    recall_response_tokens: usize,
    /// (replies, reply tokens incl. EOS) per reply kind, trained (mask 1).
    trained_replies: BTreeMap<String, (usize, usize)>,
    /// Dialogues per family (`v1` or a v2 family).
    families: BTreeMap<String, usize>,
    /// Questions per family and category.
    family_questions: BTreeMap<String, BTreeMap<String, usize>>,
    /// The same for context-only replies (mask 0).
    context_replies: BTreeMap<String, (usize, usize)>,
    max_tokens: usize,
}

impl Tally {
    fn replies_json(&self) -> Value {
        let table = |m: &BTreeMap<String, (usize, usize)>| {
            m.iter()
                .map(|(k, (n, t))| (k.clone(), json!({"replies": n, "reply_tokens": t})))
                .collect::<BTreeMap<_, _>>()
        };
        let sum = |m: &BTreeMap<String, (usize, usize)>, answers: bool| {
            m.iter()
                .filter(|(k, _)| k.starts_with("answer:") == answers)
                .fold((0, 0), |a, (_, (n, t))| (a.0 + n, a.1 + t))
        };
        let (answers, answer_tokens) = sum(&self.trained_replies, true);
        let (others, other_tokens) = sum(&self.trained_replies, false);
        let trained = answers + others;
        json!({
            "trained": table(&self.trained_replies),
            "context_only": table(&self.context_replies),
            "trained_replies": trained,
            "trained_reply_tokens": answer_tokens + other_tokens,
            "answer_reply_share_of_trained_replies": answers as f64 / trained.max(1) as f64,
            "answer_token_share_of_trained_reply_tokens":
                answer_tokens as f64 / (answer_tokens + other_tokens).max(1) as f64,
            "note": "dialogue-train samples trained replies (mask-1 runs) uniformly, so the answer reply share is the expected share of this source's sampled episodes that are question answers",
        })
    }

    fn add(&mut self, d: &Dialogue, panel: &Panel) {
        self.dialogues += 1;
        self.turns += d.turns.len();
        *self.primary.entry(d.category.name().into()).or_default() += 1;
        let family = if FAMILIES.contains(&d.category) {
            d.category.name()
        } else {
            "v1"
        };
        *self.families.entry(family.into()).or_default() += 1;
        for q in &d.questions {
            *self
                .family_questions
                .entry(family.into())
                .or_default()
                .entry(q.category.name().into())
                .or_default() += 1;
        }
        for q in &d.questions {
            *self.questions.entry(q.category.name().into()).or_default() += 1;
            let detail = format!(
                "{}{}{}",
                q.category.name(),
                if q.detail.is_empty() { "" } else { ":" },
                q.detail
            );
            *self.question_details.entry(detail).or_default() += 1;
            if q.followup {
                *self.question_details.entry("followup".into()).or_default() += 1;
            }
        }
        for f in &d.frames {
            *self.frames.entry((*f).into()).or_default() += 1;
        }
        self.strict_key_dialogues += usize::from(d.strict_keys);
        let fillers = d.turns.iter().filter(|t| t.filler).count() / 2;
        *self.distractor_exchanges.entry(fillers).or_default() += 1;
        *self
            .questions_per_dialogue
            .entry(d.questions.len())
            .or_default() += 1;
        for fact in &d.facts {
            for v in std::iter::once(&fact.value).chain(fact.old.iter()) {
                for s in content_stems(v) {
                    if panel.content.contains(&s) {
                        self.value_stems_shared.insert(s);
                    }
                }
            }
        }
        let mut shared_here = false;
        for key in &d.keys {
            for s in content_stems(&key.user) {
                if panel.content.contains(&s) {
                    *self.key_words_shared.entry(s).or_default() += 1;
                    shared_here = true;
                }
            }
        }
        self.dialogues_with_panel_key += usize::from(shared_here);
    }

    fn json(&self) -> Value {
        let total_q: usize = self.questions.values().sum();
        json!({
            "dialogues": self.dialogues,
            "turns": self.turns,
            "questions": total_q,
            "questions_per_category": self.questions,
            "question_share_per_category": self.questions.iter()
                .map(|(k, n)| (k.clone(), *n as f64 / total_q.max(1) as f64))
                .collect::<BTreeMap<_, _>>(),
            "questions_per_detail": self.question_details,
            "dialogues_per_primary_category": self.primary,
            "dialogues_per_relation": self.frames,
            "distractor_exchanges_histogram": self.distractor_exchanges,
            "questions_per_dialogue_histogram": self.questions_per_dialogue,
            "strict_key_dialogues": self.strict_key_dialogues,
            "redraws": self.redraws,
            "tokens": self.tokens,
            "response_tokens": self.response_tokens,
            "recall_answer_tokens": self.recall_response_tokens,
            "replies": self.replies_json(),
            "dialogues_per_family": self.families.keys().any(|f| f != "v1").then_some(&self.families),
            "questions_per_family": self.families.keys().any(|f| f != "v1").then_some(&self.family_questions),
            "tokens_per_dialogue": self.tokens as f64 / self.dialogues.max(1) as f64,
            "max_dialogue_tokens": self.max_tokens,
        })
    }
}

struct SplitSpec<'a> {
    name: &'static str,
    label: &'static str,
    seed: u64,
    dialogues: usize,
    token_budget: usize,
    samples: usize,
    dir: &'a Path,
}

struct Encoding<'t> {
    tokenizer: &'t ByteBpeTokenizer,
    protocol: DialogueProtocol,
    version: u8,
    context: usize,
    vocab: u32,
    /// Mask only the question answers (`true`, `train_on=answers`) or every
    /// assistant turn (`false`, `train_on=all`).
    answers_only: bool,
    /// Draw with the six v2 families (`generator=v2`) or v1 alone.
    generator_v2: bool,
    /// Write the read-binding supervision sidecar (`binding_labels=1`).
    binding_labels: bool,
}

/// One question's read-binding label: `(answer start, bound, competing,
/// queries)`, positions local to the dialogue's tokens.
type BindingLabel = (usize, Vec<usize>, Vec<usize>, Vec<usize>);

/// Step 7d's read-binding labels of one dialogue: for each question with an
/// expected value that occurs before its answer and inside the answer, its
/// [`BindingLabel`]. `bound` holds the history positions (before the answer)
/// of the expected value's word phrase, `competing` those of the question's
/// forbidden values (minus `bound`), and `queries` the input positions whose
/// next token is a token of the expected value inside the scored answer.
/// Words follow `binding_probe`'s rule, the matching the binding probe uses.
fn dialogue_binding_labels(
    tokenizer: &ByteBpeTokenizer,
    tokens: &[u32],
    dialogue: &Dialogue,
    runs: &[(usize, usize, usize)],
    counts: &mut BTreeMap<String, usize>,
) -> Result<Vec<BindingLabel>> {
    use uor_r4_training::binding_probe::{phrase_positions, token_byte_ranges, words};
    let pieces: Vec<Vec<u8>> = tokens
        .iter()
        .map(|&t| tokenizer.decode_bytes(&[t]))
        .collect();
    let ranges = token_byte_ranges(&pieces);
    let text = String::from_utf8_lossy(&pieces.concat()).into_owned();
    let mut out = Vec::new();
    for q in &dialogue.questions {
        let Some(expect) = &q.expect else {
            *counts.entry("abstain_no_value".into()).or_default() += 1;
            continue;
        };
        let &(_, start, end) = runs
            .iter()
            .find(|(turn, _, _)| *turn == q.turn)
            .ok_or("a question answer is not an assistant turn")?;
        let phrase = words(expect);
        let bound: BTreeSet<usize> = phrase_positions(&text, &ranges, 0..start, &phrase);
        let answer = phrase_positions(&text, &ranges, start..end, &phrase);
        if bound.is_empty() {
            *counts.entry("no_history_occurrence".into()).or_default() += 1;
            continue;
        }
        if answer.is_empty() {
            *counts.entry("no_answer_occurrence".into()).or_default() += 1;
            continue;
        }
        let mut competing = BTreeSet::new();
        for value in &q.forbid {
            competing.extend(phrase_positions(&text, &ranges, 0..start, &words(value)));
        }
        let competing: Vec<usize> = competing.difference(&bound).copied().collect();
        if competing.is_empty() {
            *counts
                .entry("labelled_without_competing".into())
                .or_default() += 1;
        }
        *counts.entry("labelled".into()).or_default() += 1;
        *counts.entry("rows".into()).or_default() += answer.len();
        out.push((
            start,
            bound.into_iter().collect(),
            competing,
            answer.into_iter().map(|a| a - 1).collect(),
        ));
    }
    Ok(out)
}

/// Apply `train_on` to an encoded document's response mask: under
/// `answers_only`, every assistant turn that does not answer a question keeps
/// its tokens (context) but loses its mask, EOS included. Returns the
/// per-assistant-turn runs of the original mask, in turn order.
fn apply_train_on(
    dialogue: &Dialogue,
    mask: &mut [u8],
    answers_only: bool,
) -> Result<Vec<(usize, usize, usize)>> {
    let runs = mask_runs(mask);
    let assistant_turns: Vec<usize> = (0..dialogue.turns.len())
        .filter(|i| dialogue.turns[*i].role == Role::Assistant)
        .collect();
    if runs.len() != assistant_turns.len() {
        return Err(format!(
            "{} masked runs for {} assistant turns",
            runs.len(),
            assistant_turns.len()
        ));
    }
    let answers: BTreeSet<usize> = dialogue.questions.iter().map(|q| q.turn).collect();
    let mut out = Vec::with_capacity(runs.len());
    for (&turn, &(start, end)) in assistant_turns.iter().zip(&runs) {
        if answers_only && !answers.contains(&turn) {
            mask[start..end].fill(0);
        }
        out.push((turn, start, end));
    }
    Ok(out)
}

/// Generate dialogues until the count or budget; stream tokens and mask; write
/// the samples file and the split's manifest. Returns the tally.
fn write_split(world: &World, enc: &Encoding, spec: &SplitSpec) -> Result<Tally> {
    fs::create_dir_all(spec.dir).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = enc
        .protocol
        .bind(enc.tokenizer)
        .map_err(|e| e.to_string())?;
    let tokens_path = spec.dir.join("tokens.u16");
    let mask_path = spec.dir.join("response_mask.u8");
    let samples_name = if spec.name == "dev" {
        "dialogues.jsonl"
    } else {
        "samples.jsonl"
    };
    let samples_path = spec.dir.join(samples_name);
    let mut writer = CorpusWriter::create(&tokens_path, enc.vocab).map_err(|e| e.to_string())?;
    let mut mask_out = BufWriter::new(fs::File::create(&mask_path).map_err(|e| e.to_string())?);
    let mut samples_out =
        BufWriter::new(fs::File::create(&samples_path).map_err(|e| e.to_string())?);
    let mut rng = Rng::new(spec.seed);
    let mut tally = Tally::default();
    // Step 7d: the read-binding sidecar, its counts and the SHA-256 of the
    // split's token payload (little-endian u16, no header) it is bound to.
    let mut labels_out = if enc.binding_labels {
        Some(BufWriter::new(
            fs::File::create(spec.dir.join("binding_labels.jsonl")).map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let mut label_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut payload = <sha2::Sha256 as sha2::Digest>::new();
    let mut redraws_total = 0usize;
    loop {
        if spec.dialogues > 0 && tally.dialogues >= spec.dialogues {
            break;
        }
        let drawn = if enc.generator_v2 {
            draw_v2(world, &mut rng)?
        } else {
            draw_dialogue(world, &mut rng)?
        };
        let dialogue = match drawn {
            Drawn::Ok(d) => d,
            Drawn::Redraw(reason) => {
                *tally.redraws.entry(reason.into()).or_default() += 1;
                redraws_total += 1;
                if redraws_total > 1000 + 50 * tally.dialogues {
                    return Err(format!("too many redraws: {:?}", tally.redraws));
                }
                continue;
            }
        };
        let messages: Vec<Message<'_>> = dialogue
            .turns
            .iter()
            .map(|t| Message {
                role: match t.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                content: &t.text,
            })
            .collect();
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len()
            || encoded.special_token_occurrences != 0
            || encoded.tokens.len() != encoded.response_mask.len()
        {
            return Err(format!(
                "the encoder emitted {} of {} turns with {} special-token occurrences",
                encoded.emitted_turns,
                messages.len(),
                encoded.special_token_occurrences
            ));
        }
        if encoded.tokens.len() > enc.context {
            *tally.redraws.entry("over_context".into()).or_default() += 1;
            redraws_total += 1;
            continue;
        }
        if spec.token_budget > 0 && tally.tokens + encoded.tokens.len() > spec.token_budget {
            break;
        }
        let ids: Vec<u16> = encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t).map_err(|_| "token id above u16".to_string()))
            .collect::<Result<_>>()?;
        // The masked region of each question's reply must decode to text that
        // holds the expected value: the trainer scores exactly those tokens.
        let mut mask = encoded.response_mask.clone();
        let runs = apply_train_on(&dialogue, &mut mask, enc.answers_only)?;
        let category_of: BTreeMap<usize, Category> = dialogue
            .questions
            .iter()
            .map(|q| (q.turn, q.category))
            .collect();
        for &(turn, start, end) in &runs {
            let kind = match category_of.get(&turn) {
                Some(c) => format!("answer:{}", c.name()),
                None => dialogue.turns[turn].kind.to_owned(),
            };
            let trained = mask[start] == 1;
            let entry = if trained {
                tally.trained_replies.entry(kind).or_default()
            } else {
                tally.context_replies.entry(kind).or_default()
            };
            entry.0 += 1;
            entry.1 += end - start;
        }
        for q in &dialogue.questions {
            let &(_, start, end) = runs
                .iter()
                .find(|(turn, _, _)| *turn == q.turn)
                .ok_or("a question answer is not an assistant turn")?;
            if mask[start..end].iter().any(|&m| m != 1) {
                return Err("a question answer is not fully masked".into());
            }
            tally.recall_response_tokens += end - start;
            if let Some(expect) = &q.expect {
                let scored: Vec<u32> = encoded.tokens[start..end].to_vec();
                let text = enc.tokenizer.decode(&scored);
                let got = content_stems(&text);
                if !content_stems(expect).iter().all(|s| got.contains(s)) {
                    return Err(format!(
                        "the scored reply {text:?} does not hold {expect:?}"
                    ));
                }
            }
        }
        if let Some(out) = labels_out.as_mut() {
            let offset = tally.tokens;
            for (start, bound, competing, queries) in dialogue_binding_labels(
                enc.tokenizer,
                &encoded.tokens,
                &dialogue,
                &runs,
                &mut label_counts,
            )? {
                let shift = |v: Vec<usize>| v.into_iter().map(|p| p + offset).collect::<Vec<_>>();
                serde_json::to_writer(
                    &mut *out,
                    &json!({"r": start + offset, "b": shift(bound), "c": shift(competing), "q": shift(queries)}),
                )
                .map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            for id in &ids {
                sha2::Digest::update(&mut payload, id.to_le_bytes());
            }
        }
        writer.write_tokens(&ids).map_err(|e| e.to_string())?;
        mask_out.write_all(&mask).map_err(|e| e.to_string())?;
        tally.tokens += ids.len();
        tally.response_tokens += mask.iter().filter(|&&m| m == 1).count();
        tally.max_tokens = tally.max_tokens.max(ids.len());
        if tally.dialogues < spec.samples {
            let id = format!("{}-{:07}", spec.name, tally.dialogues);
            serde_json::to_writer(&mut samples_out, &dialogue.record(&id))
                .map_err(|e| e.to_string())?;
            samples_out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        tally.add(&dialogue, &world.panel);
    }
    if tally.dialogues == 0 {
        return Err(format!("the {} split is empty", spec.name));
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != tally.tokens {
        return Err(format!("wrote {written} tokens, expected {}", tally.tokens));
    }
    mask_out.flush().map_err(|e| e.to_string())?;
    samples_out.flush().map_err(|e| e.to_string())?;
    if let Some(mut out) = labels_out.take() {
        out.flush().map_err(|e| e.to_string())?;
        let digest = sha2::Digest::finalize(payload);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        let summary = json!({
            "schema": BINDING_LABELS_SCHEMA,
            "split": spec.name,
            "tokens": tally.tokens,
            "tokens_payload_sha256": hex,
            "counts": label_counts,
            "positions": "absolute token positions of this split's tokens.u16 payload; r = the answer's first response token, b = bound (expected value) history positions, c = competing (forbidden values) history positions, q = input positions whose next token is a token of the expected value inside the answer",
            "rule": "per question with an expected value: the expected value's word phrase (binding_probe word rule) in the tokens before the answer (bound) and inside the answer (queries = those positions - 1); forbidden values' phrases before the answer, minus bound (competing); questions without a history or an answer occurrence get no label",
        });
        fs::write(
            spec.dir.join("binding_labels.json"),
            serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    drop(mask_out);
    drop(samples_out);

    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": spec.name,
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.",
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.",
        "dialogue_protocol": enc.protocol.schema,
        "train_on": if enc.answers_only { "answers" } else { "all" },
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0
        },
        "files": [{
            "label": spec.label,
            "tokens": tally.tokens,
            "response_tokens": tally.response_tokens,
            "special_token_occurrences": 0,
            "rows_used": tally.dialogues,
            "rows_total": tally.dialogues
        }],
        "max_tokens": enc.context,
        "rows_used": tally.dialogues,
        "tokens": tally.tokens,
        "response_tokens": tally.response_tokens,
        "response_fraction": tally.response_tokens as f64 / tally.tokens.max(1) as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": fs::metadata(&mask_path).map_err(|e| e.to_string())?.len(),
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        "bos_id": enc.protocol.bos_id,
        "eos_id": enc.protocol.eos_id,
        "synthetic": {
            "generator": "dialogue-recall-corpus generate",
            "schema": SCHEMA,
            "seed": spec.seed,
            "counts": tally.json(),
        },
    });
    fs::write(
        spec.dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(tally)
}

/// (start, end) of every run of mask 1.
fn mask_runs(mask: &[u8]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, &m) in mask.iter().enumerate() {
        match (m, start) {
            (1, None) => start = Some(i),
            (0, Some(s)) => {
                runs.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        runs.push((s, mask.len()));
    }
    runs
}

// ---------------------------------------------------------------------------
// The decoded-store leak check.

#[derive(Default)]
struct StoreLeak {
    documents: usize,
    turns: usize,
    panel_turn_matches: usize,
    shared_6grams: usize,
    recall_value_word_hits: usize,
    examples: Vec<Value>,
    frame_words_shared: BTreeMap<String, usize>,
}

impl StoreLeak {
    fn json(&self) -> Value {
        let mut shared: Vec<(&String, &usize)> = self.frame_words_shared.iter().collect();
        shared.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        json!({
            "documents": self.documents,
            "turns_checked": self.turns,
            "panel_turn_matches": self.panel_turn_matches,
            "shared_6grams": self.shared_6grams,
            "recall_value_word_hits": self.recall_value_word_hits,
            "examples": self.examples,
            "frame_words_shared_with_panels_distinct": self.frame_words_shared.len(),
            "frame_words_shared_with_panels_top": shared.iter().take(80)
                .map(|(w, n)| json!([w, n])).collect::<Vec<_>>(),
            "pass": self.panel_turn_matches == 0 && self.shared_6grams == 0
                && self.recall_value_word_hits == 0,
        })
    }
}

/// Decode a prepared store document by document and check every turn.
fn store_leak(store: &Path, tokenizer: &ByteBpeTokenizer, panel: &Panel) -> Result<StoreLeak> {
    let reader = MmapCorpusReader::open(store.join("tokens.u16")).map_err(|e| e.to_string())?;
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer).map_err(|e| e.to_string())?;
    let (bos, eos) = (protocol.bos_id, protocol.eos_id);
    let ids = reader.as_slice();
    let mut leak = StoreLeak::default();
    let mut start = 0usize;
    for end in 1..=ids.len() {
        if end < ids.len() && u32::from(ids[end]) != bos {
            continue;
        }
        let doc: Vec<u32> = ids[start..end]
            .iter()
            .map(|&id| u32::from(id))
            .filter(|&id| id != bos && id != eos)
            .collect();
        start = end;
        leak.documents += 1;
        let text = tokenizer.decode(&doc);
        for line in text.split('\n') {
            let content = ["User:", "Assistant:", "System:"]
                .iter()
                .find_map(|m| line.trim_start().strip_prefix(m))
                .unwrap_or(line)
                .trim();
            if content.is_empty() {
                continue;
            }
            leak.turns += 1;
            let mut hit = None;
            if panel.turns.contains(&normalized(content)) {
                leak.panel_turn_matches += 1;
                hit = Some("panel_turn");
            }
            let shared: Vec<String> = grams(content)
                .into_iter()
                .filter(|g| panel.grams.contains(g))
                .collect();
            if !shared.is_empty() {
                leak.shared_6grams += shared.len();
                hit = Some("panel_6gram");
            }
            let stems = content_stems(content);
            if stems.iter().any(|s| panel.strict.contains(s)) {
                leak.recall_value_word_hits += 1;
                hit = Some("panel_recall_value_word");
            }
            for s in stems {
                if panel.content.contains(&s) {
                    *leak.frame_words_shared.entry(s).or_default() += 1;
                }
            }
            if let Some(kind) = hit {
                let example = json!({"kind": kind, "turn": content});
                if leak.examples.len() < 40 && !leak.examples.contains(&example) {
                    leak.examples.push(example);
                }
            }
        }
    }
    Ok(leak)
}

// ---------------------------------------------------------------------------
// Driver.

fn arg<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.iter()
        .find_map(|a| a.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

fn number(args: &[String], key: &str, default: u64) -> Result<u64> {
    match arg(args, key) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| format!("{key}= must be a whole number")),
    }
}

fn panel_dirs(args: &[String]) -> Vec<PathBuf> {
    arg(args, "panels")
        .unwrap_or("data/panels")
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

fn load_tokenizer(path: &str) -> Result<ByteBpeTokenizer> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| "unreadable tokenizer.json".into())
}

fn source_commit(args: &[String]) -> Value {
    if let Some(commit) = arg(args, "source_commit") {
        return json!({"commit": commit, "from": "source_commit="});
    }
    if let Some(commit) = option_env!("UOR_BUILD_SOURCE_COMMIT") {
        return json!({"commit": commit, "from": "UOR_BUILD_SOURCE_COMMIT"});
    }
    let git = |a: &[&str]| {
        std::process::Command::new("git")
            .args(a)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    };
    match git(&["rev-parse", "HEAD"]) {
        Some(commit) => json!({
            "commit": commit,
            "from": "git rev-parse HEAD at run time",
            "dirty_including_untracked": git(&["status", "--porcelain"]).map(|s| !s.is_empty()),
        }),
        None => json!({"commit": "UNAVAILABLE", "from": "none"}),
    }
}

fn generate(args: &[String]) -> Result<()> {
    let out = PathBuf::from(arg(args, "out").ok_or("missing out=")?);
    let tokenizer_path = arg(args, "tokenizer")
        .ok_or("missing tokenizer=")?
        .to_owned();
    let seed = number(args, "seed", 1)?;
    let dialogues = number(args, "dialogues", 0)? as usize;
    let token_budget = number(args, "token_budget", 0)? as usize;
    if (dialogues == 0) == (token_budget == 0) {
        return Err("give exactly one of dialogues=N or token_budget=N".into());
    }
    let dev_seed = number(args, "dev_seed", 1_000_003)?;
    if dev_seed == seed {
        return Err("dev_seed must differ from seed".into());
    }
    let dev_dialogues = number(args, "dev_dialogues", 300)? as usize;
    let version = u8::try_from(number(args, "protocol", 2)?).map_err(|_| "protocol= is 1 or 2")?;
    if !(1..=2).contains(&version) {
        return Err("protocol= is 1 or 2".into());
    }
    let context = number(args, "context", 384)? as usize;
    if context < 256 {
        return Err("context= must be at least 256".into());
    }
    let samples = number(args, "samples", 200)? as usize;
    let generator_v2 = match arg(args, "generator").unwrap_or("v2") {
        "v2" => true,
        "v1" => false,
        other => return Err(format!("unknown generator={other}: v1 or v2")),
    };
    let train_on = arg(args, "train_on").unwrap_or("answers").to_owned();
    let answers_only = match train_on.as_str() {
        "answers" => true,
        "all" => false,
        other => return Err(format!("unknown train_on={other}: answers or all")),
    };
    let binding_labels = match arg(args, "binding_labels").unwrap_or("0") {
        "1" | "true" => true,
        "0" | "false" => false,
        other => return Err(format!("binding_labels={other}: 0 or 1")),
    };
    let dirs = panel_dirs(args);
    // Validate the inputs before claiming the root.
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let panel = Panel::load(&dirs)?;
    let world = World::new(panel)?;
    let protocol =
        DialogueProtocol::literal_roles_version(&tokenizer, version).map_err(|e| e.to_string())?;
    let vocab = u32::try_from(tokenizer.vocab_size()).map_err(|_| "vocabulary above u32")?;

    report_output::claim(&out).map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let enc = Encoding {
        tokenizer: &tokenizer,
        protocol,
        version,
        context,
        vocab,
        answers_only,
        generator_v2,
        binding_labels,
    };
    let train = write_split(
        &world,
        &enc,
        &SplitSpec {
            name: "train",
            label: "dialogue-recall",
            seed,
            dialogues,
            token_budget,
            samples,
            dir: &out.join("train"),
        },
    )?;
    let dev = write_split(
        &world,
        &enc,
        &SplitSpec {
            name: "dev",
            label: "dialogue-recall-dev",
            seed: dev_seed,
            dialogues: dev_dialogues,
            token_budget: 0,
            samples: usize::MAX,
            dir: &out.join("dev"),
        },
    )?;

    // Each split must load and index exactly as `dialogue-train` will read it.
    let mut populations = serde_json::Map::new();
    for name in ["train", "dev"] {
        let dir = out.join(name);
        let (_, contract) = episode_contract_for(&tokenizer, vocab as usize, context, enc.version)
            .map_err(|e| e.to_string())?;
        let split = DialogueSplit::load(
            &dir.join("tokens.u16"),
            &dir.join("response_mask.u8"),
            &dir.join("manifest.json"),
        )
        .map_err(|e| e.to_string())?;
        let index = split.index(contract).map_err(|e| e.to_string())?;
        let p = index.population();
        populations.insert(
            name.into(),
            json!({
                "documents": p.documents,
                "response_runs": p.response_runs,
                "eligible_responses": p.eligible_responses,
                "excluded_over_context": p.excluded_over_context,
                "context": context,
            }),
        );
    }

    // The leak report: decoded from the written stores.
    let train_leak = store_leak(&out.join("train"), &tokenizer, &world.panel)?;
    let dev_leak = store_leak(&out.join("dev"), &tokenizer, &world.panel)?;
    let structured = |t: &Tally| {
        json!({
            "recall_value_stems_shared_with_panels": t.value_stems_shared,
            "key_words_shared_with_panels": t.key_words_shared,
            "dialogues_with_a_panel_key_word": t.dialogues_with_panel_key,
            "strict_key_dialogues": t.strict_key_dialogues,
            "strict_key_share": t.strict_key_dialogues as f64 / t.dialogues.max(1) as f64,
        })
    };
    let pass = train_leak.panel_turn_matches == 0
        && train_leak.shared_6grams == 0
        && train_leak.recall_value_word_hits == 0
        && dev_leak.panel_turn_matches == 0
        && dev_leak.shared_6grams == 0
        && dev_leak.recall_value_word_hits == 0
        && train.value_stems_shared.is_empty()
        && dev.value_stems_shared.is_empty();
    let leak = json!({
        "schema": LEAK_SCHEMA,
        "pass": pass,
        "method": {
            "store": "each document of tokens.u16 decoded with the tokenizer, split into turns at the protocol's newline separator, role markers removed",
            "panel_turn_match": "a turn whose lowercased words (punctuation ignored) equal a panel string's",
            "shared_6grams": "word 6-grams of a turn found among the 6-grams of any panel string",
            "recall_value_word": "a content-word stem of a panel recall row's terms or forbid column (values, distractors and their closed classes)",
            "recall_value_stems_shared": "from the generator's records: stems of every stated value (and superseded value) found among the content stems of all panel strings and check fields",
            "frame_words": "content stems of generated turns that also occur in a panel; allowed and listed",
        },
        "panels": world.panel.files.iter().map(|(p, s)| json!({"path": p, "sha256": s})).collect::<Vec<_>>(),
        "panel_strings": world.panel.strings,
        "panel_6grams": world.panel.grams.len(),
        "panel_content_stems": world.panel.content.len(),
        "panel_recall_value_stems": world.panel.strict,
        "panel_recall_key_stems": world.panel.keys,
        "dropped_by_panel_filter": world.dropped,
        "dropped_by_panel_filter_v2": generator_v2.then(|| &world.v2.dropped),
        "train": {"decoded": train_leak.json(), "records": structured(&train)},
        "dev": {"decoded": dev_leak.json(), "records": structured(&dev)},
    });
    fs::write(
        out.join("leak.json"),
        serde_json::to_vec_pretty(&leak).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !pass {
        return Err(format!(
            "the leak check failed; see {}",
            out.join("leak.json").display()
        ));
    }

    let mut files = serde_json::Map::new();
    for rel in [
        "train/tokens.u16",
        "train/response_mask.u8",
        "train/manifest.json",
        "train/samples.jsonl",
        "dev/tokens.u16",
        "dev/response_mask.u8",
        "dev/manifest.json",
        "dev/dialogues.jsonl",
        "leak.json",
        "train/binding_labels.jsonl",
        "train/binding_labels.json",
        "dev/binding_labels.jsonl",
        "dev/binding_labels.json",
    ] {
        if rel.contains("binding_labels") && !binding_labels {
            continue;
        }
        let path = out.join(rel);
        files.insert(
            rel.into(),
            json!({
                "sha256": uor_r4_training::sha256_file(&path).map_err(|e| e.to_string())?,
                "bytes": fs::metadata(&path).map_err(|e| e.to_string())?.len(),
            }),
        );
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let generator = json!({
        "schema": SCHEMA,
        "argv": args,
        "source": source_commit(args),
        "executable": {
            "path": executable.display().to_string(),
            "sha256": uor_r4_training::sha256_file(&executable).ok(),
        },
        "tokenizer": {
            "path": tokenizer_path,
            "sha256": uor_r4_training::sha256_file(Path::new(&tokenizer_path)).map_err(|e| e.to_string())?,
        },
        "dialogue_protocol": enc.protocol.schema,
        "context": context,
        "generator": if generator_v2 { "v2" } else { "v1" },
        "v2": generator_v2.then(|| json!({
            "v1_share": V1_SHARE,
            "families": FAMILIES.iter().map(|c| c.name()).collect::<Vec<_>>(),
            "family_share_each": (1.0 - V1_SHARE) / FAMILIES.len() as f64,
            "pool_sizes": world.v2.sizes(),
        })),
        "train_on": train_on,
        "train_on_rule": if answers_only {
            "answers: response_mask is 1 only on the assistant turns that answer a recall or abstention question (content and EOS); acknowledgements, suggestions and distractor replies stay in the context with mask 0, so dialogue-train neither samples nor trains them"
        } else {
            "all: response_mask is 1 on every assistant turn"
        },
        "seed": seed,
        "dev_seed": dev_seed,
        "requested": {"dialogues": dialogues, "token_budget": token_budget, "dev_dialogues": dev_dialogues},
        "train": train.json(),
        "dev": dev.json(),
        "populations": populations,
        "leak_pass": pass,
        "relations": FRAMES.iter().map(|f| f.id).collect::<Vec<_>>(),
        "distractor_exchanges": world.fillers.len(),
        "distractor_replies": world.fillers.iter().map(|(_, r)| r.len()).sum::<usize>(),
        "pool_sizes": world.values.iter().map(|(k, v)| (format!("{k:?}"), v.len())).collect::<BTreeMap<_, _>>(),
        "primary_category_shares": PRIMARY_SHARES.iter().map(|(c, s)| (c.name(), *s)).collect::<BTreeMap<_, _>>(),
        "followup_abstain_probability": FOLLOWUP_ABSTAIN,
        "strict_key_share_target": STRICT_KEY_SHARE,
        "files": files,
        "seconds": started.elapsed().as_secs_f64(),
        "scope": "training data for in-context recall in natural dialogue; not an evaluation. The dev split shares pools and templates with train and is for monitoring only.",
    });
    fs::write(
        out.join("generator.json"),
        serde_json::to_vec_pretty(&generator).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    report_output::seal(&out).map_err(|e| e.to_string())?;
    report_output::verify(&out).map_err(|e| e.to_string())?;
    println!(
        "train: {} dialogues, {} tokens ({} recall-answer tokens); dev: {} dialogues, {} tokens; leak pass",
        train.dialogues, train.tokens, train.recall_response_tokens, dev.dialogues, dev.tokens
    );
    Ok(())
}

fn leak_mode(args: &[String]) -> Result<()> {
    let out = PathBuf::from(arg(args, "out").ok_or("missing out=")?);
    let store = PathBuf::from(arg(args, "store").ok_or("missing store=")?);
    let tokenizer_path = arg(args, "tokenizer").ok_or("missing tokenizer=")?;
    let tokenizer = load_tokenizer(tokenizer_path)?;
    let panel = Panel::load(&panel_dirs(args))?;
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let leak = store_leak(&store, &tokenizer, &panel)?;
    let report = json!({
        "schema": LEAK_SCHEMA,
        "store": store.display().to_string(),
        "tokens_sha256": uor_r4_training::sha256_file(&store.join("tokens.u16")).map_err(|e| e.to_string())?,
        "panels": panel.files.iter().map(|(p, s)| json!({"path": p, "sha256": s})).collect::<Vec<_>>(),
        "panel_strings": panel.strings,
        "panel_6grams": panel.grams.len(),
        "decoded": leak.json(),
    });
    fs::write(
        out.join("leak.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    report_output::seal(&out).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report["decoded"]).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn run(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("generate") => generate(&args[1..]),
        Some("leak") => leak_mode(&args[1..]),
        _ => Err(
            "usage: dialogue-recall-corpus generate|leak key=value ... (see the module docs)"
                .into(),
        ),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/panels");
        let panel = Panel::load(&[dir]).expect("the frozen panels load");
        World::new(panel).expect("the world builds")
    }

    fn draw_many(world: &World, seed: u64, n: usize) -> Vec<Dialogue> {
        let mut rng = Rng::new(seed);
        let mut out = Vec::new();
        while out.len() < n {
            match draw_dialogue(world, &mut rng).expect("no generator error") {
                Drawn::Ok(d) => out.push(d),
                Drawn::Redraw(_) => {}
            }
        }
        out
    }

    fn jsonl(dialogues: &[Dialogue]) -> String {
        dialogues
            .iter()
            .enumerate()
            .map(|(i, d)| d.record(&i.to_string()).to_string() + "\n")
            .collect()
    }

    #[test]
    fn same_seed_gives_identical_bytes_and_another_seed_differs() {
        let w = world();
        let a = jsonl(&draw_many(&w, 7, 400));
        let b = jsonl(&draw_many(&w, 7, 400));
        let c = jsonl(&draw_many(&w, 8, 400));
        assert_eq!(a, b);
        assert_ne!(a, c);
        // A second world from the same panels gives the same bytes too.
        let w2 = world();
        assert_eq!(a, jsonl(&draw_many(&w2, 7, 400)));
    }

    #[test]
    fn answers_name_the_bound_value_and_never_a_distractor() {
        let w = world();
        let dialogues = draw_many(&w, 11, 3000);
        let mut with_distractor = 0;
        for d in &dialogues {
            check_dialogue(d).expect("every question passes its own check");
            for q in d.questions.iter().filter(|q| q.expect.is_some()) {
                let answer = content_stems(&d.turns[q.turn].text);
                let expect = q.expect.as_deref().unwrap_or_default();
                assert!(content_stems(expect).iter().all(|s| answer.contains(s)));
                for v in &q.forbid {
                    assert!(content_stems(v).iter().all(|s| !answer.contains(s)));
                }
                if !q.forbid.is_empty() {
                    with_distractor += 1;
                    // Each distractor is stated before the question.
                    let earlier: BTreeSet<String> = d.turns[..q.turn - 1]
                        .iter()
                        .flat_map(|t| content_stems(&t.text))
                        .collect();
                    for v in &q.forbid {
                        assert!(content_stems(v).iter().all(|s| earlier.contains(s)), "{v}");
                    }
                }
            }
        }
        assert!(with_distractor > 2000, "{with_distractor}");
    }

    #[test]
    fn binding_rows_state_two_same_type_keys_in_one_turn() {
        let w = world();
        let mut seen = 0;
        for d in draw_many(&w, 13, 2000)
            .iter()
            .filter(|d| d.category == Category::Binding)
        {
            let q = &d.questions[0];
            let expect = q.expect.as_deref().unwrap_or_default();
            // The turn stating the asked value also states a same-relation distractor.
            let turn = d
                .turns
                .iter()
                .take(q.turn - 1)
                .find(|t| {
                    t.role == Role::User
                        && content_stems(&t.text).is_superset(&content_stems(expect))
                })
                .expect("the value is stated in a user turn");
            if q.forbid
                .iter()
                .any(|v| content_stems(&turn.text).is_superset(&content_stems(v)))
            {
                seen += 1;
            }
        }
        assert!(seen > 200, "{seen}");
    }

    #[test]
    fn the_latest_value_wins_after_an_update() {
        let w = world();
        let mut latest = 0;
        let mut unchanged = 0;
        for d in draw_many(&w, 17, 4000) {
            for q in d
                .questions
                .iter()
                .filter(|q| q.category == Category::Update)
            {
                let expect = q.expect.clone().unwrap_or_default();
                let fact = d
                    .facts
                    .iter()
                    .find(|f| f.value == expect)
                    .expect("the expected value is a current fact");
                if q.detail == "latest" {
                    latest += 1;
                    let old = fact
                        .old
                        .clone()
                        .expect("an updated fact keeps its old value");
                    assert!(q.forbid.contains(&old));
                    // The old value is stated, then the new value, in that order.
                    let first = |v: &str| {
                        d.turns
                            .iter()
                            .position(|t| content_stems(&t.text).is_superset(&content_stems(v)))
                            .unwrap_or(usize::MAX)
                    };
                    assert!(first(&old) < first(&expect));
                    assert!(first(&expect) < q.turn);
                } else {
                    unchanged += 1;
                    assert!(fact.old.is_none());
                    assert!(d
                        .facts
                        .iter()
                        .any(|f| f.old.as_ref().is_some_and(|o| q.forbid.contains(o))));
                }
            }
        }
        assert!(latest > 300 && unchanged > 50, "{latest} {unchanged}");
    }

    #[test]
    fn abstentions_name_no_stated_value_and_ask_an_unstated_key() {
        let w = world();
        let dialogues = draw_many(&w, 19, 4000);
        let mut abstain = 0;
        let mut questions = 0;
        for d in &dialogues {
            for q in &d.questions {
                questions += 1;
                if q.expect.is_some() {
                    continue;
                }
                abstain += 1;
                let answer = content_stems(&d.turns[q.turn].text);
                for fact in &d.facts {
                    for v in std::iter::once(&fact.value).chain(fact.old.iter()) {
                        assert!(content_stems(v).iter().all(|s| !answer.contains(s)));
                    }
                }
                assert!(
                    d.turns[q.turn].text.contains("haven't")
                        || d.turns[q.turn].text.contains("didn't")
                        || d.turns[q.turn].text.contains("never")
                );
            }
        }
        let share = abstain as f64 / questions as f64;
        assert!((0.11..=0.20).contains(&share), "abstention share {share}");
    }

    #[test]
    fn no_panel_turn_no_shared_6gram_and_no_panel_value_word() {
        let w = world();
        let dialogues = draw_many(&w, 23, 5000);
        for d in &dialogues {
            for t in &d.turns {
                assert!(!w.panel.turns.contains(&normalized(&t.text)), "{}", t.text);
                for g in grams(&t.text) {
                    assert!(!w.panel.grams.contains(&g), "{g}");
                }
                assert!(!w.panel.has_strict(&t.text), "{}", t.text);
            }
            for fact in &d.facts {
                assert!(!w.panel.shares_content(&fact.value), "{}", fact.value);
            }
        }
        // The panel's own memory example values are all excluded.
        for word in [
            "shelby", "flash", "plums", "pears", "grandpa", "turtle", "saturday",
        ] {
            assert!(w.panel.content.contains(&stem(word)), "{word}");
        }
        for word in [
            "shelby", "flash", "plum", "pear", "saturday", "blue", "nine",
        ] {
            assert!(w.panel.strict.contains(&stem(word)), "{word}");
        }
        // Strict-key dialogues use no panel key word; at least half are strict.
        let strict = dialogues.iter().filter(|d| d.strict_keys).count();
        assert!(strict * 2 > dialogues.len());
        for d in dialogues.iter().filter(|d| d.strict_keys) {
            for k in &d.keys {
                assert!(!w.panel.shares_content(&k.user), "{}", k.user);
            }
        }
    }

    #[test]
    fn the_leak_check_detects_an_injected_panel_turn() {
        let w = world();
        assert_eq!(
            w.panel.turn_leak("Which name did we give the turtle?"),
            Some("panel_turn")
        );
        assert_eq!(
            w.panel.turn_leak(
                "Yesterday we got two new pets this week. The turtle is Shelby and the goldfish is Flash."
            ),
            Some("panel_6gram")
        );
        assert_eq!(
            w.panel.turn_leak("The cat is named Shelby."),
            Some("panel_recall_value_word")
        );
        assert_eq!(w.panel.turn_leak("The ferret is named Quill."), None);
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2
    /// (GPT-2's byte alphabet), as `stack_dialogue`'s tests build it.
    fn byte_tokenizer() -> ByteBpeTokenizer {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, surface) in specials.iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let ch = if printable.contains(&byte) {
                char::from_u32(byte)
            } else {
                extra += 1;
                char::from_u32(255 + extra)
            }
            .expect("a valid char");
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .expect("the byte tokenizer parses")
    }

    /// Write a split with `write_split` and return the decoded text of every
    /// reply `dialogue-train` would sample from it (its episode index).
    fn sampled_replies(world: &World, answers_only: bool, seed: u64, n: usize) -> Vec<String> {
        let tokenizer = byte_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let vocab = u32::try_from(tokenizer.vocab_size()).expect("vocab");
        let context = 4096;
        let enc = Encoding {
            tokenizer: &tokenizer,
            protocol,
            version: 2,
            context,
            vocab,
            answers_only,
            generator_v2: false,
            binding_labels: false,
        };
        let dir = std::env::temp_dir().join(format!(
            "dialogue-recall-test-{}-{answers_only}-{seed}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let spec = SplitSpec {
            name: "train",
            label: "dialogue-recall",
            seed,
            dialogues: n,
            token_budget: 0,
            samples: 0,
            dir: &dir,
        };
        write_split(world, &enc, &spec).expect("the split is written");
        let (_, contract) =
            episode_contract_for(&tokenizer, vocab as usize, context, 2).expect("contract");
        let split = DialogueSplit::load(
            &dir.join("tokens.u16"),
            &dir.join("response_mask.u8"),
            &dir.join("manifest.json"),
        )
        .expect("the split loads as dialogue-train loads it");
        let index = split.index(contract).expect("the split indexes");
        let reader = MmapCorpusReader::open(dir.join("tokens.u16")).expect("tokens");
        let ids = reader.as_slice();
        let eos = enc.protocol.eos_id;
        let replies: Vec<String> = index
            .episodes()
            .iter()
            .map(|span| {
                let text: Vec<u32> = ids[span.response_start..span.response_end]
                    .iter()
                    .map(|&t| u32::from(t))
                    .filter(|&t| t != eos)
                    .collect();
                tokenizer.decode(&text).trim().to_owned()
            })
            .collect();
        // Every sampled id is one of these episodes, and each episode trains
        // only its own reply tokens.
        let sampled = index.sample_ids(seed, 0, 64).expect("sample");
        assert!(sampled.iter().all(|&id| id < replies.len()));
        let _ = fs::remove_dir_all(&dir);
        replies
    }

    /// Step 7d: the binding sidecar leaves the store byte-identical, binds the
    /// token payload, and every label points at the expected value: bound
    /// positions before the answer decode to the value's words, queries are
    /// scored answer positions whose next token belongs to the value.
    #[test]
    fn binding_labels_point_at_the_expected_value_and_leave_the_store_unchanged() {
        use uor_r4_training::binding_probe::words;
        let w = world();
        let tokenizer = byte_tokenizer();
        let vocab = u32::try_from(tokenizer.vocab_size()).expect("vocab");
        let base =
            std::env::temp_dir().join(format!("dialogue-recall-labels-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let mut stores = Vec::new();
        for (generator_v2, labels) in [(true, false), (true, true), (false, true)] {
            let enc = Encoding {
                tokenizer: &tokenizer,
                protocol: DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol"),
                version: 2,
                context: 4096,
                vocab,
                answers_only: true,
                generator_v2,
                binding_labels: labels,
            };
            let dir = base.join(format!("{generator_v2}-{labels}"));
            write_split(
                &w,
                &enc,
                &SplitSpec {
                    name: "train",
                    label: "dialogue-recall",
                    seed: 41,
                    dialogues: 120,
                    token_budget: 0,
                    samples: 0,
                    dir: &dir,
                },
            )
            .expect("the split is written");
            stores.push(fs::read(dir.join("tokens.u16")).expect("tokens"));
            if !labels {
                assert!(!dir.join("binding_labels.jsonl").exists());
                continue;
            }
            let summary: Value = serde_json::from_slice(
                &fs::read(dir.join("binding_labels.json")).expect("summary"),
            )
            .expect("json");
            let reader = MmapCorpusReader::open(dir.join("tokens.u16")).expect("tokens");
            let ids: Vec<u32> = reader.as_slice().iter().map(|&t| u32::from(t)).collect();
            let mask = fs::read(dir.join("response_mask.u8")).expect("mask");
            let mut payload = <sha2::Sha256 as sha2::Digest>::new();
            for &t in reader.as_slice() {
                sha2::Digest::update(&mut payload, t.to_le_bytes());
            }
            let hex: String = sha2::Digest::finalize(payload)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            assert_eq!(summary["tokens_payload_sha256"], hex.as_str());
            assert_eq!(summary["tokens"], ids.len());
            let lines = fs::read_to_string(dir.join("binding_labels.jsonl")).expect("labels");
            let mut labelled = 0usize;
            for line in lines.lines() {
                let label: Value = serde_json::from_str(line).expect("label");
                let list = |k: &str| -> Vec<usize> {
                    label[k]
                        .as_array()
                        .expect("list")
                        .iter()
                        .map(|v| v.as_u64().expect("position") as usize)
                        .collect()
                };
                let r = label["r"].as_u64().expect("r") as usize;
                let (b, c, q) = (list("b"), list("c"), list("q"));
                assert!(
                    mask[r] == 1 && mask[r - 1] == 0,
                    "r starts a scored response"
                );
                assert!(!b.is_empty() && !q.is_empty());
                assert!(b.iter().chain(&c).all(|&p| p < r));
                assert!(c.iter().all(|p| !b.contains(p)));
                assert!(q.iter().all(|&p| p + 1 >= r && mask[p + 1] == 1));
                // The bound tokens decode to text holding a word of each
                // answered value token.
                let runs_text = |positions: &[usize]| -> String {
                    let mut parts: Vec<Vec<u32>> = Vec::new();
                    for (i, &p) in positions.iter().enumerate() {
                        if i == 0 || positions[i - 1] + 1 != p {
                            parts.push(Vec::new());
                        }
                        if let Some(part) = parts.last_mut() {
                            part.push(ids[p]);
                        }
                    }
                    parts
                        .iter()
                        .map(|part| tokenizer.decode(part))
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                let bound_text = runs_text(&b);
                let next: Vec<usize> = q.iter().map(|&p| p + 1).collect();
                let answer_text = runs_text(&next);
                assert!(
                    words(&answer_text)
                        .iter()
                        .all(|w| words(&bound_text).contains(w)),
                    "{answer_text:?} not in {bound_text:?}"
                );
                labelled += 1;
            }
            assert_eq!(summary["counts"]["labelled"], labelled);
            assert!(labelled > 60, "only {labelled} labelled answers");
        }
        assert_eq!(stores[0], stores[1], "the sidecar changed the store");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn train_on_answers_leaves_exactly_the_answer_turns_sampleable() {
        let w = world();
        let n = 150;
        let dialogues = draw_many(&w, 29, n);
        let answers: Vec<String> = dialogues
            .iter()
            .flat_map(|d| d.questions.iter().map(|q| d.turns[q.turn].text.clone()))
            .collect();
        let every_reply: Vec<String> = dialogues
            .iter()
            .flat_map(|d| {
                d.turns
                    .iter()
                    .filter(|t| t.role == Role::Assistant)
                    .map(|t| t.text.clone())
            })
            .collect();
        assert!(every_reply.len() > 2 * answers.len());
        assert_eq!(sampled_replies(&w, true, 29, n), answers);
        assert_eq!(sampled_replies(&w, false, 29, n), every_reply);
    }

    #[test]
    fn the_distractor_bank_is_large_and_clean() {
        let w = world();
        assert!(w.fillers.len() >= 400, "{}", w.fillers.len());
        for (user, replies) in &w.fillers {
            assert!(w.panel.turn_leak(user).is_none(), "{user}");
            for reply in replies {
                assert!(w.panel.turn_leak(reply).is_none(), "{reply}");
            }
        }
    }

    fn family_dialogues(w: &World, family: Category, seed: u64, n: usize) -> Vec<Dialogue> {
        let mut rng = Rng::new(seed);
        let mut out = Vec::new();
        let mut tries = 0;
        while out.len() < n {
            tries += 1;
            assert!(tries < 50 * n, "too many redraws for {family:?}");
            match draw_family(w, &mut rng, family).expect("no generator error") {
                Drawn::Ok(d) => out.push(d),
                Drawn::Redraw(_) => {}
            }
        }
        out
    }

    /// Clauses of the user statement turns before the first question, each as
    /// a set of stems.
    fn statement_clauses(d: &Dialogue) -> Vec<BTreeSet<String>> {
        let first_question = d.questions.iter().map(|q| q.turn - 1).min().unwrap_or(0);
        let mut out = Vec::new();
        for t in d.turns[..first_question]
            .iter()
            .filter(|t| t.role == Role::User && !t.filler)
        {
            let mut text = format!(" {} ", t.text);
            // Keep a value such as "mac and cheese" in one clause.
            for f in &d.facts {
                if f.value.contains(" and ") {
                    text = text.replace(&f.value, &f.value.replace(" and ", " "));
                }
            }
            for sep in [
                ". ", ", ", ": ", "; ", " and ", " but ", " while ", " though ", " then ",
            ] {
                text = text.replace(sep, "|");
            }
            out.extend(
                text.split('|')
                    .map(|c| {
                        words(c)
                            .iter()
                            .map(|w| stem(w))
                            .collect::<BTreeSet<String>>()
                    })
                    .filter(|c| !c.is_empty()),
            );
        }
        out
    }

    fn clause_with<'a>(clauses: &'a [BTreeSet<String>], text: &str) -> &'a BTreeSet<String> {
        let want = content_stems(text);
        clauses
            .iter()
            .find(|c| want.iter().all(|s| c.contains(s)))
            .unwrap_or_else(|| panic!("no clause holds {text:?}: {clauses:?}"))
    }

    fn statement_text(d: &Dialogue) -> String {
        let first_question = d.questions.iter().map(|q| q.turn - 1).min().unwrap_or(0);
        d.turns[..first_question]
            .iter()
            .filter(|t| t.role == Role::User && !t.filler)
            .map(|t| t.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    #[test]
    fn rules_answer_the_allowed_or_banned_activity() {
        let w = world();
        let base: BTreeMap<&str, &str> = RULE_ACTS.iter().copied().collect();
        let negation: BTreeSet<String> =
            ["not", "isnt", "cant", "banned", "forbidden", "limits", "no"]
                .iter()
                .map(|s| stem(s))
                .collect();
        let mut seen = [0usize; 2];
        for d in family_dialogues(&w, Category::Rule, 31, 600) {
            let clauses = statement_clauses(&d);
            for q in d.questions.iter().filter(|q| q.category == Category::Rule) {
                let gerund = q.expect.clone().unwrap_or_default();
                let clause = clauses
                    .iter()
                    .find(|c| {
                        let g = content_stems(&gerund);
                        let b = content_stems(base.get(gerund.as_str()).copied().unwrap_or(""));
                        g.iter().all(|s| c.contains(s))
                            || (!b.is_empty() && b.iter().all(|s| c.contains(s)))
                    })
                    .unwrap_or_else(|| panic!("{gerund} not stated"));
                let negated = clause.iter().any(|s| negation.contains(s));
                let allowed = q.detail.starts_with("allowed");
                assert_eq!(negated, !allowed, "{gerund}: {clause:?} {}", q.detail);
                seen[usize::from(allowed)] += 1;
            }
        }
        assert!(seen[0] > 200 && seen[1] > 200, "{seen:?}");
    }

    #[test]
    fn implicit_updates_keep_the_latest_value_and_can_recall_the_first() {
        let w = world();
        let first_turn = |d: &Dialogue, v: &str| {
            let want = content_stems(v);
            d.turns
                .iter()
                .position(|t| want.iter().all(|s| content_stems(&t.text).contains(s)))
                .unwrap_or(usize::MAX)
        };
        let mut counts = BTreeMap::new();
        for d in family_dialogues(&w, Category::ImplicitUpdate, 37, 600) {
            for q in d
                .questions
                .iter()
                .filter(|q| q.category == Category::ImplicitUpdate)
            {
                let expect = q.expect.clone().unwrap_or_default();
                *counts.entry(q.detail).or_insert(0) += 1;
                match q.detail {
                    "latest" => {
                        let fact = d
                            .facts
                            .iter()
                            .find(|f| f.value == expect)
                            .expect("a current value");
                        let old = fact
                            .old
                            .clone()
                            .expect("a changed plan keeps its first value");
                        assert!(q.forbid.contains(&old));
                        assert!(first_turn(&d, &old) < first_turn(&d, &expect));
                        // No user statement is "actually" or "correction" phrased.
                        assert!(!statement_text(&d).contains("actually"));
                    }
                    "first" => {
                        let fact = d
                            .facts
                            .iter()
                            .find(|f| f.old.as_deref() == Some(expect.as_str()))
                            .expect("the first value of a changed plan");
                        assert!(q.forbid.contains(&fact.value));
                        assert!(first_turn(&d, &expect) < first_turn(&d, &fact.value));
                    }
                    "unchanged" => {
                        let fact = d.facts.iter().find(|f| f.value == expect).expect("a value");
                        assert!(fact.old.is_none());
                    }
                    other => panic!("unknown detail {other}"),
                }
            }
        }
        assert!(
            counts.get("latest").copied().unwrap_or(0) > 200,
            "{counts:?}"
        );
        assert!(counts.get("first").copied().unwrap_or(0) > 80, "{counts:?}");
    }

    #[test]
    fn self_facts_are_answered_and_bound_to_the_right_person() {
        let w = world();
        let (mut me, mut other, mut abstain) = (0, 0, 0);
        for d in family_dialogues(&w, Category::SelfFact, 41, 800) {
            let clauses = statement_clauses(&d);
            let person = d
                .facts
                .iter()
                .find(|f| f.key.user != "I")
                .expect("another person's fact");
            let person_words = content_stems(&person.key.user);
            for q in &d.questions {
                let Some(expect) = q.expect.clone() else {
                    abstain += 1;
                    continue;
                };
                let clause = clause_with(&clauses, &expect);
                if q.detail == "self" {
                    me += 1;
                    assert!(
                        person_words.iter().all(|s| !clause.contains(s)),
                        "{clause:?}"
                    );
                    // The user's own stated fact is answered, never abstained.
                    assert!(!d.turns[q.turn].text.contains("haven't"));
                } else {
                    other += 1;
                    assert!(
                        person_words.iter().all(|s| clause.contains(s)),
                        "{clause:?}"
                    );
                }
            }
        }
        assert!(
            me > other && other > abstain / 2 && abstain * 4 < me + other,
            "{me} {other} {abstain}"
        );
    }

    #[test]
    fn attribute_objects_are_told_apart_by_their_attribute() {
        let w = world();
        let mut seen = 0;
        for d in family_dialogues(&w, Category::Attribute, 43, 600) {
            let clauses = statement_clauses(&d);
            let attribute = |key: &str| words(key).get(1).cloned().unwrap_or_default();
            for q in d
                .questions
                .iter()
                .filter(|q| q.category == Category::Attribute)
            {
                let expect = q.expect.clone().unwrap_or_default();
                let (fact, value) = if q.detail == "reverse" {
                    let f = d
                        .facts
                        .iter()
                        .find(|f| attribute(&f.key.user) == expect.to_lowercase())
                        .expect("the attribute's object");
                    assert!(content_stems(&d.turns[q.turn - 1].text)
                        .is_superset(&content_stems(&f.value)));
                    (f, f.value.clone())
                } else {
                    (
                        d.facts.iter().find(|f| f.value == expect).expect("a value"),
                        expect.clone(),
                    )
                };
                let clause = clause_with(&clauses, &value);
                assert!(
                    clause.contains(&stem(&attribute(&fact.key.user))),
                    "{clause:?}"
                );
                for other in d.facts.iter().filter(|f| f.key.user != fact.key.user) {
                    assert!(
                        !clause.contains(&stem(&attribute(&other.key.user))),
                        "{clause:?}"
                    );
                }
                seen += 1;
            }
        }
        assert!(seen > 400, "{seen}");
    }

    #[test]
    fn counts_belong_to_their_container_or_person() {
        let w = world();
        let mut seen = 0;
        for d in family_dialogues(&w, Category::Count, 47, 600) {
            let clauses = statement_clauses(&d);
            for q in d.questions.iter().filter(|q| q.category == Category::Count) {
                let expect = q.expect.clone().unwrap_or_default();
                let fact = d.facts.iter().find(|f| f.value == expect).expect("a count");
                let clause = clause_with(&clauses, &expect);
                for other in d.facts.iter().filter(|f| f.key.user != fact.key.user) {
                    let mine = content_stems(&distinguishing(&fact.key.user, &other.key.user));
                    let theirs = content_stems(&distinguishing(&other.key.user, &fact.key.user));
                    assert!(
                        mine.iter().all(|s| clause.contains(s)),
                        "{clause:?} {}",
                        fact.key.user
                    );
                    assert!(theirs.iter().all(|s| !clause.contains(s)), "{clause:?}");
                }
                seen += 1;
            }
        }
        assert!(seen > 400, "{seen}");
    }

    #[test]
    fn temporal_order_answers_follow_the_stated_sequence() {
        let w = world();
        let mut seen = BTreeMap::new();
        for d in family_dialogues(&w, Category::Order, 53, 800) {
            let text = statement_text(&d);
            let mut events: Vec<(usize, String)> = d
                .facts
                .iter()
                .map(|f| {
                    (
                        text.find(&f.value.to_lowercase()).expect("stated"),
                        f.value.clone(),
                    )
                })
                .collect();
            events.sort();
            let order: Vec<String> = events.into_iter().map(|(_, e)| e).collect();
            let rank = |e: &str| order.iter().position(|x| x == e).expect("an event");
            for q in d.questions.iter().filter(|q| q.category == Category::Order) {
                let expect = q.expect.clone().unwrap_or_default();
                let r = rank(&expect);
                let question = d.turns[q.turn - 1].text.to_lowercase();
                match q.detail {
                    "first" => assert_eq!(r, 0),
                    "second" => assert_eq!(r, 1),
                    "third" => assert_eq!(r, 2),
                    "last" => assert_eq!(r, order.len() - 1),
                    "after" => assert!(question.contains(&order[r - 1].to_lowercase())),
                    "before" => assert!(question.contains(&order[r + 1].to_lowercase())),
                    "at_time" | "day_part" => {
                        let fact = d
                            .facts
                            .iter()
                            .find(|f| f.value == expect)
                            .expect("an event");
                        assert!(question.contains(&fact.key.user.to_lowercase()));
                    }
                    other => panic!("unknown detail {other}"),
                }
                *seen.entry(q.detail).or_insert(0) += 1;
            }
        }
        assert!(seen.len() >= 7, "{seen:?}");
    }

    #[test]
    fn v2_draws_are_deterministic_clean_and_mixed() {
        let w = world();
        let draw = |seed: u64, n: usize| {
            let mut rng = Rng::new(seed);
            let mut out = Vec::new();
            while out.len() < n {
                if let Drawn::Ok(d) = draw_v2(&w, &mut rng).expect("no generator error") {
                    out.push(d);
                }
            }
            out
        };
        let a = draw(59, 3000);
        assert_eq!(jsonl(&a), jsonl(&draw(59, 3000)));
        let mut families = BTreeMap::new();
        for d in &a {
            check_dialogue(d).expect("the contract holds");
            *families.entry(d.category.name()).or_insert(0) += 1;
            for t in &d.turns {
                assert!(!w.panel.turns.contains(&normalized(&t.text)), "{}", t.text);
                assert!(
                    grams(&t.text).iter().all(|g| !w.panel.grams.contains(g)),
                    "{}",
                    t.text
                );
                assert!(!w.panel.has_strict(&t.text), "{}", t.text);
            }
            for f in &d.facts {
                assert!(!w.panel.shares_content(&f.value), "{}", f.value);
            }
        }
        for family in FAMILIES {
            assert!(
                families.get(family.name()).copied().unwrap_or(0) > 150,
                "{families:?}"
            );
        }
    }

    #[test]
    fn placeholders_fill_or_fail() {
        let mut slots = BTreeMap::new();
        slots.insert("k".to_owned(), "the gecko".to_owned());
        assert_eq!(
            fill("Where is {k}?", &slots).as_deref(),
            Ok("Where is the gecko?")
        );
        assert!(fill("Where is {v}?", &slots).is_err());
        assert_eq!(article("umbrella", Art::Indef), "an umbrella");
        assert_eq!(article("lantern", Art::Indef), "a lantern");
        assert_eq!(stem("tomatoes"), "tomato");
        assert_eq!(stem("goldfish's".replace('\'', "").as_str()), "goldfish");
    }
}
