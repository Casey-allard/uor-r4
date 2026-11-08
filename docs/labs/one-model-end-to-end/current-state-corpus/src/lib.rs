//! Deterministic TRAIN generator of "current-state" chat episodes for two
//! relations, job and home (claude-324).
//!
//! Each document is one user message, which states the job and the home (open,
//! multi-word values), may update either, and then asks for the CURRENT value
//! of one, followed by one assistant reply that is exactly the bare current
//! value plus a period, `{v}.`, then EOS. That is the only reply form the
//! `m2-fresh256` scoring panel accepts. The user message is the
//! `one-user-turn/v1` shape `served-reply-panel` renders: statements and the
//! question joined by single spaces.
//!
//! Two shapes, alternating by row (exactly 50/50):
//! - `panel-shape` (even rows): one statement per relation, as the served
//!   panel prompt keeps only each relation's latest statement;
//! - `update-history` (odd rows): one to three statements per relation, at
//!   least three in all, the first an assertion and the rest updates, every
//!   statement kept and interleaved uniformly at random while each relation
//!   keeps its own order.
//!
//! The output is a `uor-r4-chat-corpus/v1` store (`tokens.u16`,
//! `response_mask.u8`, `manifest.json`) that `mix-chat-corpus` accepts.

use m2_fresh_heldout_panel_instrument as panel;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, CORPUS_HEADER_SIZE};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, EncodedDialogue, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

pub const CORPUS_SCHEMA: &str = "uor-r4-chat-corpus/v1";
pub const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
pub const MAX_TOKENS: usize = 1024;
pub const SHAPE_PANEL: &str = "panel-shape";
pub const SHAPE_UPDATES: &str = "update-history";
/// Two, like the panel's shortest literals: a one-word value of one token gives a reply of 3
/// scored targets (value, '.', EOS), under the trainer's 4-target window minimum, so those rows
/// would be dropped silently (2,545 of 40,000 at seed 324 with a minimum of 1).
pub const MIN_VALUE_WORDS: usize = 2;
pub const MAX_VALUE_WORDS: usize = 8;
const TOKENIZER_REL: &str = ".uor-models/research/issue-1019/tokenizer/tokenizer.json";

/// Value vocabulary: lowercase food, household, music, place, animal and
/// adjective words; no function words, none equal to a content word of the
/// panel's literals (pinned by `value_words_are_disjoint_from_panel_literals`).
pub const VALUE_WORDS: &[&str] = &[
    // food
    "apple", "banana", "cherry", "grape", "lemon", "lime", "mango", "melon", "peach", "pear",
    "plum", "apricot", "fig", "olive", "tomato", "potato", "carrot", "onion", "garlic", "pepper",
    "cabbage", "lettuce", "spinach", "celery", "radish", "turnip", "pumpkin", "squash",
    "zucchini", "cucumber", "broccoli", "cauliflower", "bean", "lentil", "chickpea", "rice",
    "noodle", "pasta", "bread", "bagel", "muffin", "pancake", "waffle", "biscuit", "cookie",
    "cake", "pie", "tart", "donut", "pretzel", "cracker", "cheese", "butter", "yogurt", "cream",
    "honey", "jam", "syrup", "sugar", "salt", "cinnamon", "nutmeg", "ginger", "vanilla", "cocoa",
    "coffee", "tea", "juice", "soup", "stew", "salad", "sandwich", "taco", "burrito", "pizza",
    "dumpling", "sushi", "curry", "omelette", "toast", "cereal", "oatmeal", "popcorn", "walnut",
    "almond", "peanut", "cashew", "pistachio", "hazelnut", "coconut", "pineapple", "papaya",
    "kiwi", "raspberry", "strawberry", "blueberry", "blackberry", "cranberry",
    // household
    "spoon", "fork", "knife", "plate", "bowl", "cup", "mug", "saucer", "teapot", "skillet",
    "ladle", "whisk", "spatula", "colander", "blender", "toaster", "oven", "fridge", "freezer",
    "sink", "faucet", "towel", "blanket", "pillow", "mattress", "sofa", "couch", "armchair",
    "stool", "table", "desk", "shelf", "drawer", "cupboard", "closet", "wardrobe", "curtain",
    "carpet", "rug", "lamp", "candle", "clock", "radio", "television", "phone", "laptop",
    "keyboard", "printer", "stapler", "pencil", "eraser", "notebook", "envelope", "scissors",
    "ruler", "broom", "mop", "bucket", "sponge", "soap", "shampoo", "toothbrush", "comb",
    "hairbrush", "umbrella", "basket", "hamper", "hanger", "vase", "teacup", "napkin",
    "tablecloth", "doormat",
    // music
    "piano", "guitar", "cello", "viola", "harp", "flute", "oboe", "clarinet", "bassoon",
    "trumpet", "trombone", "tuba", "saxophone", "drum", "tambourine", "cymbal", "xylophone",
    "accordion", "harmonica", "banjo", "ukulele", "mandolin", "bagpipe", "organ", "melody",
    "harmony", "rhythm", "chord", "tempo", "ballad", "anthem", "lullaby", "sonata", "symphony",
    "concerto", "opera", "chorus", "duet", "trio", "quartet", "waltz", "tango", "polka", "jazz",
    "reggae", "orchestra", "metronome",
    // places
    "library", "museum", "theater", "cinema", "bakery", "pharmacy", "hospital", "school",
    "college", "stadium", "airport", "station", "garage", "warehouse", "factory", "office",
    "studio", "apartment", "hotel", "motel", "restaurant", "cafe", "diner", "pub", "market",
    "supermarket", "mall", "chapel", "castle", "palace", "fortress", "lighthouse", "windmill",
    "barn", "farmhouse", "cottage", "bungalow", "mansion", "village", "suburb", "downtown",
    "island", "peninsula", "desert", "jungle", "prairie", "tundra", "volcano", "glacier",
    "lagoon", "beach", "coast", "lake", "pond", "waterfall", "mountain", "zoo", "aquarium",
    "playground",
    // animals
    "dog", "cat", "horse", "cow", "pig", "sheep", "goat", "chicken", "duck", "goose", "turkey",
    "rabbit", "hamster", "mouse", "squirrel", "chipmunk", "beaver", "raccoon", "skunk", "fox",
    "wolf", "bear", "deer", "moose", "elk", "bison", "camel", "llama", "alpaca", "zebra",
    "giraffe", "elephant", "rhino", "hippo", "lion", "tiger", "leopard", "cheetah", "panther",
    "jaguar", "monkey", "gorilla", "lemur", "koala", "kangaroo", "wombat", "platypus",
    "penguin", "parrot", "pigeon", "sparrow", "robin", "crow", "raven", "owl", "eagle",
    "flamingo", "pelican", "swan", "peacock", "turtle", "tortoise", "lizard", "iguana", "snake",
    "frog", "toad", "salmon", "trout", "tuna", "shark", "whale", "dolphin", "octopus", "squid",
    "crab", "lobster", "shrimp", "jellyfish", "starfish", "snail", "butterfly", "bee", "ant",
    "beetle", "spider",
    // adjectives
    "red", "orange", "yellow", "purple", "pink", "brown", "black", "white", "gray", "golden",
    "scarlet", "teal", "turquoise", "happy", "sleepy", "hungry", "noisy", "tiny", "huge",
    "little", "big", "tall", "round", "soft", "cold", "sunny", "rainy", "windy", "snowy",
    "cloudy", "foggy", "shiny", "fuzzy", "fluffy", "smooth", "rough", "crispy", "crunchy",
    "juicy", "sweet", "sour", "salty", "spicy", "bitter", "fresh", "frozen", "clever", "lucky", "friendly", "funny", "silly",
    "brave", "proud", "shy", "busy", "lazy", "polite", "cheerful", "jolly", "merry",
];

/// Panel write templates (`m2-fresh-heldout-panel` `supported_wire`) and the
/// curriculum's TRAIN phrasings (`milestone_world.rs`; never `development`).
const JOB_ASSERT: [&str; 4] = [
    "My current job is {v}.",
    "I work as a {v}.",
    "My job is {v}.",
    "I am a {v} by trade.",
];
const JOB_UPDATE: [&str; 3] = [
    "My current job has changed to {v}.",
    "Actually, I work as a {v} now.",
    "I changed jobs. Now I'm a {v}.",
];
const HOME_ASSERT: [&str; 4] = [
    "I currently live in {v}.",
    "I live in {v}.",
    "My home is in {v}.",
    "I moved to {v}.",
];
const HOME_UPDATE: [&str; 3] = [
    "I now live in {v}.",
    "Actually, I live in {v} now.",
    "I just moved to {v}.",
];
const JOB_TRAIN_QUERIES: [&str; 3] = [
    "What is my job?",
    "What do I do for work?",
    "What do I work as?",
];
const HOME_TRAIN_QUERIES: [&str; 3] = [
    "Where do I live?",
    "What city do I live in?",
    "Where is my home?",
];

/// splitmix64, seeded directly from `seed`.
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// An index below `n` (`n` > 0); modulo bias is below 2^-50 for the
    /// small `n` used here.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub relation: &'static str,
    pub update: bool,
    pub value: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Episode {
    pub shape: &'static str,
    pub statements: Vec<Statement>,
    pub query_relation: &'static str,
    pub question: String,
}

impl Episode {
    /// The `one-user-turn/v1` user message: statements, then the question,
    /// joined by single spaces.
    pub fn user_message(&self) -> String {
        let mut parts: Vec<&str> = self.statements.iter().map(|s| s.text.as_str()).collect();
        parts.push(&self.question);
        parts.join(" ")
    }

    /// The asked relation's value in its last statement.
    pub fn current_value(&self) -> &str {
        &self
            .statements
            .iter()
            .rev()
            .find(|s| s.relation == self.query_relation)
            .expect("every episode states both relations")
            .value
    }

    /// The bare reply: `{v}.`.
    pub fn reply(&self) -> String {
        format!("{}.", self.current_value())
    }
}

fn fresh_value(rng: &mut SplitMix64, used: &mut Vec<String>) -> String {
    loop {
        let len = MIN_VALUE_WORDS + rng.below(MAX_VALUE_WORDS - MIN_VALUE_WORDS + 1);
        let words: Vec<&str> = (0..len)
            .map(|_| VALUE_WORDS[rng.below(VALUE_WORDS.len())])
            .collect();
        let value = words.join(" ");
        if !used.contains(&value) {
            used.push(value.clone());
            return value;
        }
    }
}

fn statement(
    rng: &mut SplitMix64,
    relation: &'static str,
    update: bool,
    used: &mut Vec<String>,
) -> Statement {
    let templates: &[&str] = match (relation, update) {
        ("job", false) => &JOB_ASSERT,
        ("job", true) => &JOB_UPDATE,
        ("home", false) => &HOME_ASSERT,
        _ => &HOME_UPDATE,
    };
    let value = fresh_value(rng, used);
    let template = templates[rng.below(templates.len())];
    Statement {
        relation,
        update,
        text: template.replace("{v}", &value),
        value,
    }
}

/// One episode; `row` parity picks the shape.
pub fn episode(rng: &mut SplitMix64, row: usize) -> Episode {
    let mut used = Vec::new();
    let (shape, statements) = if row % 2 == 0 {
        let job_update = rng.below(2) == 1;
        let job = statement(rng, "job", job_update, &mut used);
        let home_update = rng.below(2) == 1;
        let home = statement(rng, "home", home_update, &mut used);
        let pair = if rng.below(2) == 0 {
            vec![job, home]
        } else {
            vec![home, job]
        };
        (SHAPE_PANEL, pair)
    } else {
        let (job_count, home_count) = loop {
            let j = 1 + rng.below(3);
            let h = 1 + rng.below(3);
            if j + h >= 3 {
                break (j, h);
            }
        };
        let mut job = Vec::new();
        for i in 0..job_count {
            job.push(statement(rng, "job", i > 0, &mut used));
        }
        let mut home = Vec::new();
        for i in 0..home_count {
            home.push(statement(rng, "home", i > 0, &mut used));
        }
        // Uniform over interleavings: take from a relation with probability
        // proportional to its remaining statements.
        let (mut job, mut home) = (job.into_iter(), home.into_iter());
        let (mut left_job, mut left_home) = (job_count, home_count);
        let mut merged = Vec::new();
        while left_job + left_home > 0 {
            if rng.below(left_job + left_home) < left_job {
                merged.push(job.next().expect("job statement left"));
                left_job -= 1;
            } else {
                merged.push(home.next().expect("home statement left"));
                left_home -= 1;
            }
        }
        (SHAPE_UPDATES, merged)
    };
    let (query_relation, queries): (&'static str, Vec<&str>) = if rng.below(2) == 0 {
        (
            "job",
            panel::CURRENT_JOB_QUERIES
                .iter()
                .chain(JOB_TRAIN_QUERIES.iter())
                .copied()
                .collect(),
        )
    } else {
        (
            "home",
            panel::CURRENT_HOME_QUERIES
                .iter()
                .chain(HOME_TRAIN_QUERIES.iter())
                .copied()
                .collect(),
        )
    };
    let question = queries[rng.below(queries.len())].to_string();
    Episode {
        shape,
        statements,
        query_relation,
        question,
    }
}

/// The first `rows` episodes from `seed`.
pub fn episodes(rows: usize, seed: u64) -> Vec<Episode> {
    let mut rng = SplitMix64::new(seed);
    (0..rows).map(|row| episode(&mut rng, row)).collect()
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

pub struct PinnedTokenizer {
    pub tokenizer: ByteBpeTokenizer,
    pub path: PathBuf,
    pub sha256: String,
    pub protocol: DialogueProtocol,
}

/// The pinned tokenizer, in the checkout or (from a `.worktrees/` checkout)
/// the main checkout; refused unless its SHA-256 is the panel's pin.
pub fn load_tokenizer() -> Result<PinnedTokenizer, String> {
    let root = repo_root();
    let candidates = [root.join(TOKENIZER_REL), root.join("../..").join(TOKENIZER_REL)];
    let found = candidates
        .iter()
        .find(|p| p.is_file())
        .ok_or_else(|| format!("{TOKENIZER_REL} not found from {}", root.display()))?;
    let path = found
        .canonicalize()
        .map_err(|e| format!("{}: {e}", found.display()))?;
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let sha256 = panel::sha256_hex(&bytes);
    if sha256 != panel::PINNED_TOKENIZER_SHA256 {
        return Err(format!("{}: SHA-256 {sha256} is not the pin", path.display()));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or("unreadable tokenizer.json")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    Ok(PinnedTokenizer {
        tokenizer,
        path,
        sha256,
        protocol,
    })
}

/// Every served user message of `m2-fresh256`, in two forms: as
/// `served-reply-panel` renders it from the inputs (Context segments decoded,
/// trimmed, sorted by event, joined by spaces, then the decoded question), and
/// as the full `History.turns` join followed by each query.
pub fn panel_user_messages(tokenizer: &ByteBpeTokenizer) -> Result<BTreeSet<String>, String> {
    let bundle = panel::generate_m2_fresh256_panel()?;
    let inputs: panel::ProbeInputs =
        serde_json::from_slice(&bundle.inputs_json).map_err(|e| e.to_string())?;
    let mut out = BTreeSet::new();
    for case in &inputs.cases {
        let mut contexts: Vec<(u64, String)> = case
            .segments
            .iter()
            .filter_map(|segment| match segment {
                panel::ProbeSegment::Context {
                    event, token_ids, ..
                } => Some((*event, tokenizer.decode(token_ids).trim().to_owned())),
                _ => None,
            })
            .collect();
        contexts.sort_by_key(|context| context.0);
        let mut parts: Vec<String> = contexts.into_iter().map(|context| context.1).collect();
        parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
        out.insert(parts.join(" "));
    }
    for history in &bundle.histories {
        for query in &history.queries {
            let mut parts: Vec<&str> = history.turns.iter().map(|w| w.text.as_str()).collect();
            parts.push(&query.text);
            out.insert(parts.join(" "));
        }
    }
    if inputs.cases.is_empty() || bundle.histories.is_empty() {
        return Err("the panel produced no cases".into());
    }
    Ok(out)
}

/// The two-turn document for `episode`, refused over `MAX_TOKENS`.
pub fn encode_episode(
    encoder: &DialogueEncoder<'_>,
    episode: &Episode,
) -> Result<EncodedDialogue, String> {
    let user = episode.user_message();
    let reply = episode.reply();
    let doc = encoder.encode_document(&[
        Message {
            role: "user",
            content: &user,
        },
        Message {
            role: "assistant",
            content: &reply,
        },
    ]);
    if doc.emitted_turns != 2 {
        return Err(format!("{} turns emitted, expected 2", doc.emitted_turns));
    }
    if doc.tokens.len() > MAX_TOKENS {
        return Err(format!(
            "a document of {} tokens exceeds max_tokens {MAX_TOKENS}",
            doc.tokens.len()
        ));
    }
    Ok(doc)
}

/// The served prompt is a token prefix of the document; the mask is 0 over it
/// and 1 from its end through the final EOS; the reply decodes to `{v}.`.
pub fn verify_document(
    pinned: &PinnedTokenizer,
    encoder: &DialogueEncoder<'_>,
    episode: &Episode,
    doc: &EncodedDialogue,
) -> Result<(), String> {
    let user = episode.user_message();
    let prefix = encoder
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: &user,
        }])
        .tokens;
    let p = prefix.len();
    if doc.tokens.len() <= p + 1 || doc.tokens[..p] != prefix[..] {
        return Err("the served prompt is not a token prefix of the document".into());
    }
    if doc.response_mask.len() != doc.tokens.len()
        || doc.response_mask[..p].iter().any(|&m| m != 0)
        || doc.response_mask[p..].iter().any(|&m| m != 1)
    {
        return Err("the mask is not 0 over the prompt and 1 over the reply".into());
    }
    if doc.tokens.last() != Some(&pinned.protocol.eos_id) {
        return Err("the document does not end at EOS".into());
    }
    let decoded = pinned.tokenizer.decode(&doc.tokens[p..doc.tokens.len() - 1]);
    let reply = pinned.protocol.reply_text(&decoded);
    if reply != episode.reply() {
        return Err(format!("reply {reply:?} is not {:?}", episode.reply()));
    }
    Ok(())
}

pub struct Corpus {
    pub tokens: Vec<u16>,
    pub mask: Vec<u8>,
    pub panel_rows: usize,
    pub update_rows: usize,
    pub max_document_tokens: usize,
    pub special_token_occurrences: usize,
}

/// Per-document counts of scored (mask 1) tokens, documents split at BOS (token 0). The
/// trainer keeps a window only with at least 4 scored targets (`build_document_windows`), so
/// `under_4` is the number of rows it will drop.
pub fn response_census(tokens: &[u16], mask: &[u8]) -> Value {
    let mut counts: Vec<usize> = Vec::new();
    for (t, m) in tokens.iter().zip(mask) {
        if *t == 0 {
            counts.push(0);
        }
        if let Some(c) = counts.last_mut() {
            *c += usize::from(*m == 1);
        }
    }
    let mut hist = BTreeMap::new();
    for c in &counts {
        *hist.entry(c.to_string()).or_insert(0usize) += 1;
    }
    json!({
        "rows": counts.len(),
        "min": counts.iter().min(),
        "max": counts.iter().max(),
        "under_4": counts.iter().filter(|&&c| c < 4).count(),
        "histogram": hist,
    })
}

/// Generate, encode and verify `rows` documents; refuse any user message
/// byte-equal to one in `panel_messages`.
pub fn build(
    pinned: &PinnedTokenizer,
    rows: usize,
    seed: u64,
    panel_messages: &BTreeSet<String>,
) -> Result<Corpus, String> {
    let encoder = pinned
        .protocol
        .bind(&pinned.tokenizer)
        .map_err(|e| e.to_string())?;
    let mut corpus = Corpus {
        tokens: Vec::new(),
        mask: Vec::new(),
        panel_rows: 0,
        update_rows: 0,
        max_document_tokens: 0,
        special_token_occurrences: 0,
    };
    for (row, episode) in episodes(rows, seed).iter().enumerate() {
        if panel_messages.contains(&episode.user_message()) {
            return Err(format!("row {row}: user message equals a panel user message"));
        }
        let doc = encode_episode(&encoder, episode)?;
        verify_document(pinned, &encoder, episode, &doc).map_err(|e| format!("row {row}: {e}"))?;
        for &id in &doc.tokens {
            corpus
                .tokens
                .push(u16::try_from(id).map_err(|_| format!("token id {id} exceeds u16"))?);
        }
        corpus.mask.extend_from_slice(&doc.response_mask);
        corpus.max_document_tokens = corpus.max_document_tokens.max(doc.tokens.len());
        corpus.special_token_occurrences += doc.special_token_occurrences;
        if episode.shape == SHAPE_PANEL {
            corpus.panel_rows += 1;
        } else {
            corpus.update_rows += 1;
        }
    }
    if corpus.special_token_occurrences != 0 {
        return Err("a special token surface occurs in literal text".into());
    }
    Ok(corpus)
}

/// Write the store into a NEW directory `out` (an existing one is refused)
/// and return its manifest.
pub fn write_store(
    pinned: &PinnedTokenizer,
    out: &Path,
    rows: usize,
    seed: u64,
) -> Result<Value, String> {
    if rows == 0 {
        return Err("rows must be positive".into());
    }
    let panel_messages = panel_user_messages(&pinned.tokenizer)?;
    let corpus = build(pinned, rows, seed, &panel_messages)?;
    fs::create_dir(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, pinned.tokenizer.vocab_size() as u32)
        .map_err(|e| e.to_string())?;
    writer
        .write_tokens(&corpus.tokens)
        .map_err(|e| e.to_string())?;
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != corpus.tokens.len() {
        return Err(format!("wrote {written} tokens, expected {}", corpus.tokens.len()));
    }
    let mask_path = out.join("response_mask.u8");
    fs::write(&mask_path, &corpus.mask).map_err(|e| e.to_string())?;
    let tokens_bytes = fs::read(&tokens_path).map_err(|e| e.to_string())?;
    if tokens_bytes.len() != CORPUS_HEADER_SIZE + 2 * corpus.tokens.len() {
        return Err("tokens.u16 size differs from header plus two bytes per token".into());
    }
    let words_joined = VALUE_WORDS.join("\n");
    let protocol = &pinned.protocol;
    let manifest = json!({
        "schema": CORPUS_SCHEMA,
        "mask_schema": MASK_SCHEMA,
        "split": "train",
        "generator": "docs/labs/one-model-end-to-end/current-state-corpus",
        "crate_version": env!("CARGO_PKG_VERSION"),
        "rows": rows,
        "rows_used": rows,
        "rows_total": rows,
        "documents": rows,
        "bos_id": protocol.bos_id,
        "source": "UOR-R4 generator docs/labs/one-model-end-to-end/current-state-corpus (claude-324)",
        "revision": format!("current-state-corpus {} seed {seed}", env!("CARGO_PKG_VERSION")),
        "licence": "generated by the project; no third-party text",
        "attribution": "UOR-R4 (claude-324). Templates from the m2-fresh256 panel's supported_wire and milestone_world.rs train phrasings.",
        "seed": seed,
        "rng": "splitmix64(seed)",
        "turn_structure": "one-user-turn/v1",
        "shapes": {
            SHAPE_PANEL: {
                "rows": corpus.panel_rows,
                "share": format!("{}/{rows}", corpus.panel_rows),
                "rule": "even rows: one statement per relation (assert or update template), in random order",
            },
            SHAPE_UPDATES: {
                "rows": corpus.update_rows,
                "share": format!("{}/{rows}", corpus.update_rows),
                "rule": "odd rows: 1 to 3 statements per relation, at least 3 in all, first an assertion and the rest updates, uniformly interleaved keeping each relation's order",
            },
        },
        "values": {
            "words": VALUE_WORDS.len(),
            "words_sha256": panel::sha256_hex(words_joined.as_bytes()),
            "min_words": MIN_VALUE_WORDS,
            "max_words": MAX_VALUE_WORDS,
            "distinct_within_episode": true,
        },
        "reply_rule": "the assistant reply is exactly the asked relation's current value (its last statement) followed by '.', then <|eos|>",
        "template_rule": "<|bos|>, then 'User:' and the user content (literal-role-dialogue/2: the content carries the space after the marker), '\\n', 'Assistant:' and the reply content, then <|eos|>; the user content is the statements and then the question joined by single spaces",
        "mask_rule": "1 = each token of the assistant reply content and its terminating <|eos|>; 0 = <|bos|>, role markers, the separator and the user content",
        "panel_overlap": {
            "panel": "m2-fresh256",
            "panel_user_messages": panel_messages.len(),
            "byte_equal_user_messages": 0,
        },
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": corpus.special_token_occurrences,
        },
        "dialogue_protocol": protocol.schema,
        "tokenizer": {
            "path": pinned.path.display().to_string(),
            "sha256": pinned.sha256,
            "tokenizer_cid": protocol.tokenizer_cid,
            "vocab_size": pinned.tokenizer.vocab_size(),
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "unk_id": protocol.unk_id,
        },
        "max_tokens": MAX_TOKENS,
        "max_document_tokens": corpus.max_document_tokens,
        "response_tokens_per_row": response_census(&corpus.tokens, &corpus.mask),
        "tokens": corpus.tokens.len(),
        "tokens_bytes": tokens_bytes.len(),
        "mask_bytes": corpus.mask.len(),
        "response_tokens": corpus.mask.iter().filter(|&&m| m == 1).count(),
        "tokens_sha256": panel::sha256_hex(&tokens_bytes),
        "mask_sha256": panel::sha256_hex(&corpus.mask),
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                    .to_ascii_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    #[test]
    fn value_words_are_disjoint_from_panel_literals() {
        let mut literals: Vec<String> = panel::FRESH256_LEN2
            .iter()
            .chain(panel::FRESH256_UPDATE_INIT.iter())
            .chain(panel::FRESH256_LEN4.iter())
            .chain(panel::FRESH256_LEN8.iter())
            .chain(panel::HISTORICAL_EXPOSED_LITERALS.iter())
            .map(|s| s.to_string())
            .collect();
        let from_disk = panel::load_historical_exposed_literals_from_repo(&repo_root())
            .expect("historical literals parse from the repository");
        assert_eq!(from_disk.len(), panel::HISTORICAL_EXPOSED_LITERALS.len());
        literals.extend(from_disk);
        let panel_words: BTreeSet<String> = literals.iter().flat_map(|l| words(l)).collect();
        let mut clashes = Vec::new();
        let mut seen = BTreeSet::new();
        for &w in VALUE_WORDS {
            assert!(seen.insert(w), "{w} repeats");
            assert!(
                !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase()),
                "{w} is not a lowercase word"
            );
            // The word, and its regular singular and plural forms (-s, -es, -y/-ies), both ways.
            let mut forms = vec![w.to_string(), format!("{w}s"), format!("{w}es")];
            for suffix in ["s", "es"] {
                if let Some(stem) = w.strip_suffix(suffix) {
                    forms.push(stem.to_string());
                }
            }
            if let Some(stem) = w.strip_suffix("ies") {
                forms.push(format!("{stem}y"));
            }
            if let Some(stem) = w.strip_suffix('y') {
                forms.push(format!("{stem}ies"));
            }
            if forms.iter().any(|f| panel_words.contains(f)) {
                clashes.push(w);
            }
        }
        assert!(clashes.is_empty(), "value words in panel literals: {clashes:?}");
        for function_word in ["the", "a", "an", "of", "in", "on", "by", "to", "and", "or", "is"] {
            assert!(!seen.contains(function_word), "function word {function_word}");
        }
    }

    #[test]
    fn tokenizer_is_the_pinned_one() {
        let pinned = load_tokenizer().unwrap();
        assert_eq!(
            pinned.sha256,
            "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
        );
        assert_eq!(pinned.protocol.schema, "uor-r4.literal-role-dialogue/2");
    }

    #[test]
    fn no_user_message_equals_a_panel_user_message() {
        let pinned = load_tokenizer().unwrap();
        let panel_messages = panel_user_messages(&pinned.tokenizer).unwrap();
        // 256 served prompts plus the distinct history-turn joins; pinned so a change to the
        // panel or to this rebuild of its `render` is seen.
        assert_eq!(panel_messages.len(), 320, "{} panel messages", panel_messages.len());
        assert!(panel_messages.iter().any(|m| m.contains("basalt loggia")));
        for episode in episodes(5000, 324) {
            assert!(!panel_messages.contains(&episode.user_message()));
        }
        assert!(build(&pinned, 5000, 324, &panel_messages).is_ok());
    }

    #[test]
    fn documents_have_bare_reply_form_and_mask() {
        let pinned = load_tokenizer().unwrap();
        let encoder = pinned.protocol.bind(&pinned.tokenizer).unwrap();
        let all = episodes(4000, 7);
        let mut shapes = (0, 0);
        for episode in &all {
            let mut last_job = None;
            let mut last_home = None;
            for s in &episode.statements {
                match s.relation {
                    "job" => last_job = Some(s.value.as_str()),
                    _ => last_home = Some(s.value.as_str()),
                }
            }
            let expected = if episode.query_relation == "job" {
                last_job
            } else {
                last_home
            };
            assert_eq!(Some(episode.current_value()), expected);
            assert_eq!(episode.reply(), format!("{}.", expected.unwrap()));
            let values: BTreeSet<&str> =
                episode.statements.iter().map(|s| s.value.as_str()).collect();
            assert_eq!(values.len(), episode.statements.len(), "values repeat");
            for s in &episode.statements {
                let n = s.value.split(' ').count();
                assert!((MIN_VALUE_WORDS..=MAX_VALUE_WORDS).contains(&n));
            }
            if episode.shape == SHAPE_PANEL {
                shapes.0 += 1;
                assert_eq!(episode.statements.len(), 2);
            } else {
                shapes.1 += 1;
                assert!(episode.statements.len() >= 3);
                for relation in ["job", "home"] {
                    let own: Vec<&Statement> = episode
                        .statements
                        .iter()
                        .filter(|s| s.relation == relation)
                        .collect();
                    assert!(!own.is_empty() && !own[0].update);
                    assert!(own[1..].iter().all(|s| s.update));
                }
            }
            let doc = encode_episode(&encoder, episode).unwrap();
            verify_document(&pinned, &encoder, episode, &doc).unwrap();
        }
        assert_eq!(shapes, (2000, 2000));
    }

    #[test]
    fn same_seed_gives_identical_bytes() {
        let pinned = load_tokenizer().unwrap();
        let none = BTreeSet::new();
        let a = build(&pinned, 3000, 11, &none).unwrap();
        let b = build(&pinned, 3000, 11, &none).unwrap();
        assert_eq!(a.tokens, b.tokens);
        assert_eq!(a.mask, b.mask);
        let c = build(&pinned, 3000, 12, &none).unwrap();
        assert_ne!(a.tokens, c.tokens);
    }

    #[test]
    fn store_reads_back_with_every_mix_field() {
        let pinned = load_tokenizer().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("cs");
        let manifest = write_store(&pinned, &out, 2000, 5).unwrap();
        assert!(write_store(&pinned, &out, 2000, 5).is_err(), "an existing dir is refused");
        let on_disk: Value =
            serde_json::from_slice(&fs::read(out.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(on_disk, manifest);

        let reader = MmapCorpusReader::open(out.join("tokens.u16")).unwrap();
        let expected = build(&pinned, 2000, 5, &BTreeSet::new()).unwrap();
        assert_eq!(reader.as_slice(), &expected.tokens[..]);
        assert_eq!(reader.vocab_size() as usize, pinned.tokenizer.vocab_size());
        let mask = fs::read(out.join("response_mask.u8")).unwrap();
        assert_eq!(mask, expected.mask);
        let tokens_bytes = fs::read(out.join("tokens.u16")).unwrap();

        assert_eq!(manifest["schema"], "uor-r4-chat-corpus/v1");
        assert_eq!(manifest["mask_schema"], "uor-r4-response-mask/u8/v1");
        assert_eq!(manifest["dialogue_protocol"], "uor-r4.literal-role-dialogue/2");
        let t = &manifest["tokenizer"];
        assert_eq!(t["sha256"], panel::PINNED_TOKENIZER_SHA256);
        assert_eq!(t["tokenizer_cid"], pinned.protocol.tokenizer_cid.as_str());
        assert_eq!(t["vocab_size"].as_u64(), Some(reader.vocab_size() as u64));
        assert_eq!(t["bos_id"].as_u64(), Some(pinned.protocol.bos_id as u64));
        assert_eq!(t["eos_id"].as_u64(), Some(pinned.protocol.eos_id as u64));
        assert_eq!(t["unk_id"].as_u64(), Some(pinned.protocol.unk_id as u64));
        assert_eq!(manifest["max_tokens"].as_u64(), Some(1024));
        assert_eq!(manifest["tokens"].as_u64(), Some(reader.total_tokens() as u64));
        assert_eq!(
            manifest["tokens_bytes"].as_u64(),
            Some(fs::metadata(out.join("tokens.u16")).unwrap().len())
        );
        assert_eq!(
            manifest["tokens_bytes"].as_u64(),
            Some((CORPUS_HEADER_SIZE + 2 * reader.total_tokens()) as u64)
        );
        assert_eq!(manifest["mask_bytes"].as_u64(), Some(mask.len() as u64));
        assert_eq!(
            manifest["response_tokens"].as_u64(),
            Some(mask.iter().filter(|&&m| m == 1).count() as u64)
        );
        assert_eq!(manifest["tokens_sha256"], panel::sha256_hex(&tokens_bytes).as_str());
        assert_eq!(manifest["mask_sha256"], panel::sha256_hex(&mask).as_str());
        // The scale-probe mix (docs/labs/scale-probe/instrument/src/lib.rs, load_split and mix)
        // reads a top-level bos_id of 0 and copies these four provenance fields from a root.
        assert_eq!(manifest["bos_id"].as_u64(), Some(0));
        // Every row survives the trainer's 4-scored-target window minimum.
        assert_eq!(manifest["response_tokens_per_row"]["under_4"].as_u64(), Some(0));
        for key in ["source", "revision", "licence", "attribution"] {
            assert!(manifest[key].is_string(), "manifest lacks {key}");
        }
        assert_eq!(manifest["rows_used"].as_u64(), Some(2000));
        assert_eq!(manifest["rows_total"].as_u64(), Some(2000));
        for key in [
            "rows_dropped_no_messages",
            "rows_dropped_empty",
            "rows_dropped_no_response",
            "rows_dropped_oversized",
            "special_token_occurrences",
        ] {
            assert_eq!(manifest["drops"][key].as_u64(), Some(0), "{key}");
        }
        assert!(manifest["template_rule"].is_string());
        assert!(manifest["mask_rule"].is_string());
        assert_eq!(manifest["shapes"][SHAPE_PANEL]["rows"].as_u64(), Some(1000));
        assert_eq!(manifest["shapes"][SHAPE_UPDATES]["rows"].as_u64(), Some(1000));
    }
}
