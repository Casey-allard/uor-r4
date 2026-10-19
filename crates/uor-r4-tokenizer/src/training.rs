//! Deterministic, bounded byte-level BPE data preparation. No model runtime.
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap};

use serde_json::{json, Value};

use crate::{bytes_to_unicode, for_each_pre_token, parse_pre_tokenizer, ByteBpeTokenizer, Piece};

const SPECIALS: [&str; 3] = ["<|bos|>", "<|eos|>", "<|unk|>"];
const BASE_SIZE: usize = 259;
type Pair = (u32, u32);

/// Hard ingestion limits. A failed addition must be discarded with its trainer.
#[derive(Clone, Copy, Debug)]
pub struct TrainingLimits {
    pub max_input_bytes: usize,
    pub max_unique_pretokens: usize,
    pub max_pretoken_bytes: usize,
}

impl Default for TrainingLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_unique_pretokens: 1_000_000,
            max_pretoken_bytes: 16 * 1024,
        }
    }
}

/// Add UTF-8 documents, then consume the trainer to produce a tokenizer.json.
/// IDs 0/1/2 are BOS/EOS/UNK; all bytes are available even if unseen in TRAIN.
/// The requested size is an upper bound: exhausted pairs never get padded.
pub struct BpeTrainer {
    target_size: usize,
    min_frequency: u64,
    pre_tokenizer: Value,
    seed: ByteBpeTokenizer,
    words: BTreeMap<String, u64>,
    limits: TrainingLimits,
    input_bytes: usize,
    failed: bool,
}

impl BpeTrainer {
    pub fn new(
        target_size: usize,
        min_frequency: u64,
        pre_tokenizer: Value,
    ) -> Result<Self, String> {
        if !(BASE_SIZE..=65_536).contains(&target_size) {
            return Err("vocab size must be in 259..=65536 (u16 IDs)".into());
        }
        if min_frequency == 0 {
            return Err("minimum frequency must be positive".into());
        }
        // Canonicalize only the policy the shared engine actually implements.
        // Refuse ignored ByteLevel options rather than silently approximating.
        validate_policy(&pre_tokenizer)?;
        let config = parse_pre_tokenizer(Some(&pre_tokenizer))
            .ok_or("expected ByteLevel, optionally after Digits")?;
        let byte_level = json!({"type":"ByteLevel", "add_prefix_space":config.add_prefix_space,
            "trim_offsets":true,"use_regex":true});
        let pre_tokenizer = match config.digits_individual {
            Some(individual) => json!({"type":"Sequence","pretokenizers":[
                {"type":"Digits","individual_digits":individual},byte_level]}),
            None => byte_level,
        };
        let seed = ByteBpeTokenizer::from_tokenizer_json_bytes(
            &serde_json::to_vec(&tokenizer_json(&initial_tokens(), &[], &pre_tokenizer))
                .map_err(|error| error.to_string())?,
        )
        .ok_or("internal base tokenizer is invalid")?;
        Ok(Self {
            target_size,
            min_frequency,
            pre_tokenizer,
            seed,
            words: BTreeMap::new(),
            limits: TrainingLimits::default(),
            input_bytes: 0,
            failed: false,
        })
    }

    /// Set limits before adding documents. Zero limits are rejected.
    pub fn with_limits(mut self, limits: TrainingLimits) -> Result<Self, String> {
        if self.input_bytes != 0 || self.failed {
            return Err("set limits before adding input".into());
        }
        if limits.max_input_bytes == 0
            || limits.max_unique_pretokens == 0
            || limits.max_pretoken_bytes == 0
        {
            return Err("training limits must be positive".into());
        }
        self.limits = limits;
        Ok(self)
    }

    /// File boundaries and atomic specials never contribute merge pairs.
    pub fn add_text(&mut self, text: &str) -> Result<(), String> {
        if self.failed {
            return Err("trainer was invalidated by a prior input error".into());
        }
        self.failed = true;
        self.input_bytes = self
            .input_bytes
            .checked_add(text.len())
            .ok_or("input byte count overflow")?;
        if self.input_bytes > self.limits.max_input_bytes {
            return Err("input byte limit exceeded".into());
        }
        let mut error = None;
        for piece in self.seed.split_on_added_tokens(text) {
            if let Piece::Text(segment) = piece {
                for_each_pre_token(
                    segment,
                    self.seed.add_prefix_space,
                    self.seed.digits_individual,
                    |word| {
                        if error.is_some() {
                            return;
                        }
                        if word.len() > self.limits.max_pretoken_bytes {
                            error = Some("pre-token byte limit exceeded".to_string());
                            return;
                        }
                        if !self.words.contains_key(word)
                            && self.words.len() >= self.limits.max_unique_pretokens
                        {
                            error = Some("unique pre-token limit exceeded".to_string());
                            return;
                        }
                        let count = self.words.entry(word.to_owned()).or_default();
                        match count.checked_add(1) {
                            Some(next) => *count = next,
                            None => error = Some("pre-token frequency overflow".to_string()),
                        }
                    },
                );
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        self.failed = false;
        Ok(())
    }

    pub fn finish(self) -> Result<Value, String> {
        if self.failed {
            return Err("trainer was invalidated by an input error".into());
        }
        if self.words.is_empty() {
            return Err("no ordinary text to train on".into());
        }
        let alphabet = bytes_to_unicode();
        let mut tokens = initial_tokens();
        let mut token_ids: HashMap<String, u32> = tokens
            .iter()
            .enumerate()
            .map(|(id, token)| (token.clone(), id as u32))
            .collect();
        let mut words: Vec<Word> = self
            .words
            .into_iter()
            .map(|(text, weight)| Word {
                symbols: text
                    .bytes()
                    .map(|byte| token_ids[&alphabet[byte as usize].to_string()])
                    .collect(),
                weight,
            })
            .collect();
        let mut index = HashMap::<Pair, PairStats>::new();
        for (id, word) in words.iter().enumerate() {
            add_word(&mut index, id, word)?;
        }
        let mut heap = make_heap(&index, &tokens);
        let mut merges = Vec::new();
        while tokens.len() < self.target_size {
            let Some(candidate) = heap.pop() else {
                break;
            };
            if index.get(&candidate.pair).map(|stats| stats.count) != Some(candidate.count) {
                continue;
            }
            if candidate.count < self.min_frequency {
                break;
            }
            let (left, right) = candidate.pair;
            let merged = format!("{}{}", tokens[left as usize], tokens[right as usize]);
            let merged_id = match token_ids.get(&merged) {
                Some(&id) => id,
                None => {
                    let id = tokens.len() as u32;
                    token_ids.insert(merged.clone(), id);
                    tokens.push(merged);
                    id
                }
            };
            merges.push([
                tokens[left as usize].clone(),
                tokens[right as usize].clone(),
            ]);
            let affected: Vec<_> = index[&candidate.pair].words.iter().copied().collect();
            let mut changed_pairs = BTreeSet::new();
            for id in affected {
                let word = &mut words[id];
                for (pair, count) in pair_counts(word)? {
                    let stats = index.get_mut(&pair).ok_or("missing indexed pair")?;
                    stats.count = stats
                        .count
                        .checked_sub(count)
                        .ok_or("pair count underflow")?;
                    stats.words.remove(&id);
                    changed_pairs.insert(pair);
                }
                let mut output = Vec::with_capacity(word.symbols.len());
                let mut i = 0;
                while i < word.symbols.len() {
                    if i + 1 < word.symbols.len()
                        && (word.symbols[i], word.symbols[i + 1]) == candidate.pair
                    {
                        output.push(merged_id);
                        i += 2;
                    } else {
                        output.push(word.symbols[i]);
                        i += 1;
                    }
                }
                word.symbols = output;
                for pair in pair_counts(word)?.keys() {
                    changed_pairs.insert(*pair);
                }
                add_word(&mut index, id, word)?;
            }
            for pair in changed_pairs {
                if index[&pair].count == 0 {
                    index.remove(&pair);
                } else {
                    heap.push(candidate_for(pair, index[&pair].count, &tokens));
                }
            }
            // Bound lazy stale entries; rebuilding does not affect tie order.
            if heap.len() > index.len().saturating_mul(4).max(1024) {
                heap = make_heap(&index, &tokens);
            }
        }
        Ok(tokenizer_json(&tokens, &merges, &self.pre_tokenizer))
    }
}

fn validate_policy(policy: &Value) -> Result<(), String> {
    let steps: Vec<&Value> = if policy["type"] == "Sequence" {
        let object = policy.as_object().ok_or("Sequence must be an object")?;
        if object
            .keys()
            .any(|key| key != "type" && key != "pretokenizers")
        {
            return Err("unsupported Sequence option".into());
        }
        let steps = policy["pretokenizers"]
            .as_array()
            .ok_or("Sequence requires pretokenizers")?;
        if steps.len() != 2 || steps[0]["type"] != "Digits" || steps[1]["type"] != "ByteLevel" {
            return Err("Sequence must contain exactly Digits then ByteLevel".into());
        }
        steps.iter().collect()
    } else {
        vec![policy]
    };
    for step in steps {
        let object = step.as_object().ok_or("pre-tokenizer must be an object")?;
        let allowed: &[&str] = match step["type"].as_str() {
            Some("ByteLevel") => &["type", "add_prefix_space", "trim_offsets", "use_regex"],
            Some("Digits") => &["type", "individual_digits"],
            _ => return Err("only ByteLevel and Digits are supported".into()),
        };
        if object.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err("unsupported pre-tokenizer option".into());
        }
        for (key, value) in object {
            if key != "type" && !value.is_boolean() {
                return Err("pre-tokenizer options must be boolean".into());
            }
        }
        if step["type"] == "ByteLevel" && step.get("use_regex") == Some(&Value::Bool(false)) {
            return Err("ByteLevel use_regex=false is unsupported".into());
        }
    }
    Ok(())
}

fn initial_tokens() -> Vec<String> {
    SPECIALS
        .iter()
        .map(|token| (*token).to_owned())
        .chain(bytes_to_unicode().into_iter().map(|ch| ch.to_string()))
        .collect()
}

fn tokenizer_json(tokens: &[String], merges: &[[String; 2]], pre_tokenizer: &Value) -> Value {
    let vocab: BTreeMap<_, _> = tokens
        .iter()
        .enumerate()
        .map(|(id, token)| (token, id))
        .collect();
    let specials: Vec<_> = SPECIALS.iter().enumerate().map(|(id, content)| json!({
        "id":id,"content":content,"single_word":false,"lstrip":false,"rstrip":false,"normalized":false,"special":true
    })).collect();
    json!({"version":"1.0","truncation":null,"padding":null,"added_tokens":specials,
        "normalizer":null,"pre_tokenizer":pre_tokenizer,"post_processor":null,
        "decoder":{"type":"ByteLevel","add_prefix_space":true,"trim_offsets":true,"use_regex":true},
        "model":{"type":"BPE","dropout":null,"unk_token":"<|unk|>","continuing_subword_prefix":"",
            "end_of_word_suffix":"","fuse_unk":false,"byte_fallback":false,"ignore_merges":false,
            "vocab":vocab,"merges":merges}})
}

struct Word {
    symbols: Vec<u32>,
    weight: u64,
}
#[derive(Default)]
struct PairStats {
    count: u64,
    words: BTreeSet<usize>,
}

fn pair_counts(word: &Word) -> Result<BTreeMap<Pair, u64>, String> {
    let mut counts = BTreeMap::<Pair, u64>::new();
    for pair in word.symbols.windows(2) {
        let count = counts.entry((pair[0], pair[1])).or_default();
        *count = count
            .checked_add(word.weight)
            .ok_or("pair occurrence overflow")?;
    }
    Ok(counts)
}
fn add_word(index: &mut HashMap<Pair, PairStats>, id: usize, word: &Word) -> Result<(), String> {
    for (pair, count) in pair_counts(word)? {
        let stats = index.entry(pair).or_default();
        stats.count = stats
            .count
            .checked_add(count)
            .ok_or("pair frequency overflow")?;
        stats.words.insert(id);
    }
    Ok(())
}
#[derive(Eq, PartialEq)]
struct Candidate {
    count: u64,
    key: Reverse<(String, String)>,
    pair: Pair,
}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count
            .cmp(&other.count)
            .then_with(|| self.key.cmp(&other.key))
            .then_with(|| self.pair.cmp(&other.pair))
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
fn candidate_for(pair: Pair, count: u64, tokens: &[String]) -> Candidate {
    Candidate {
        count,
        key: Reverse((
            tokens[pair.0 as usize].clone(),
            tokens[pair.1 as usize].clone(),
        )),
        pair,
    }
}
fn make_heap(index: &HashMap<Pair, PairStats>, tokens: &[String]) -> BinaryHeap<Candidate> {
    index
        .iter()
        .map(|(&pair, stats)| candidate_for(pair, stats.count, tokens))
        .collect()
}
