//! Build a synthetic in-context memory corpus: many fact / competing-statement
//! / question documents whose assistant turn is the correct answer.
//!
//! ```text
//! geometric-stack synthetic-memory out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   rows=2000 seed=7 [split=train] [vocab_size=4096]
//! ```
//!
//! Why this exists: the value-faithfulness diagnostic (#1512) found that a
//! competing turn costs the emitter about 2/10 of answer-target stability
//! (fact+question 7/10, fact+competing+question 4-5/10), and that the deficit is
//! *not* retrieval, selection or question binding. Testing whether it is
//! trainable needs a **distribution** of such bindings, and the development
//! panel is ten rows -- roughly 500 tokens, about 0.0006% of the 82.5M-token
//! chat-v0 response store, far too little to move a fitted model.
//!
//! The documents are written in the corpus format the trainer already loads
//! (schema `uor-r4-chat-corpus/v1`): `<|bos|>` then turns joined by a single
//! newline, each turn `<marker><content>` with markers `User: ` / `Assistant: `,
//! every assistant turn ending with `<|eos|>`; `tokens.u16` holds the ids and
//! `response_mask.u8` is 1 on each assistant response token and its terminating
//! EOS, 0 elsewhere. Encoding goes through the same `DialogueProtocol` and
//! `DialogueEncoder` the rest of the project uses, so role markers, separators
//! and masking are not re-implemented here.
//!
//! Both the asked relation and the competing statement are always *stated*, and
//! the answer is always the asked relation's value, so a model that learns the
//! corpus is learning to answer the asked relation rather than to prefer the
//! most recent value. Which relation is asked varies, and the competing
//! statement's relation is drawn independently.

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

/// A relation, the questions that ask it, and the answers it can state.
///
/// `asks` are question forms and `states` are statement forms; `{v}` is the
/// value. Keeping several of each stops the generator from teaching a single
/// surface pattern.
struct Relation {
    id: &'static str,
    asks: &'static [&'static str],
    states: &'static [&'static str],
    /// How the answer is phrased. A template rather than the relation id
    /// interpolated into one sentence, because the ids are not all noun phrases
    /// and `"Your brothers is seven."` is not English.
    answers: &'static [&'static str],
}

const RELATIONS: &[Relation] = &[
    Relation {
        id: "name",
        asks: &["What is my name?", "Do you know my name?"],
        states: &["My name is {v}.", "I am called {v}."],
        answers: &["Your name is {v}.", "You are called {v}."],
    },
    Relation {
        id: "cat",
        asks: &["What is my cat's name?", "What did I name my cat?"],
        states: &["My cat is named {v}.", "I named my cat {v}."],
        answers: &["Your cat is named {v}.", "Your cat's name is {v}."],
    },
    Relation {
        id: "colour",
        asks: &["What is my favorite color?", "Which color do I like best?"],
        states: &["My favorite color is {v}.", "I like {v} best."],
        answers: &["Your favorite color is {v}.", "You like {v} best."],
    },
    Relation {
        id: "job",
        asks: &["What is my job?", "What do I do for work?"],
        states: &["I work as a {v}.", "My job is {v}."],
        answers: &["Your job is {v}.", "You work as a {v}."],
    },
    Relation {
        id: "sister",
        asks: &["Where does my sister live?", "Which city is my sister in?"],
        states: &["My sister lives in {v}.", "My sister is in {v}."],
        answers: &["Your sister lives in {v}.", "Your sister is in {v}."],
    },
    Relation {
        id: "brothers",
        asks: &[
            "How many brothers do I have?",
            "How many brothers are there?",
        ],
        states: &["I have {v} brothers.", "There are {v} of us brothers."],
        answers: &["You have {v} brothers."],
    },
    Relation {
        id: "birthday",
        asks: &["When is my birthday?", "Which month is my birthday?"],
        states: &["My birthday is in {v}.", "I was born in {v}."],
        answers: &["Your birthday is in {v}.", "You were born in {v}."],
    },
    Relation {
        id: "car",
        asks: &["What color is my car?", "Which color is my car?"],
        states: &["I drive a {v} car.", "My car is {v}."],
        answers: &["Your car is {v}.", "You drive a {v} car."],
    },
    Relation {
        id: "instrument",
        asks: &[
            "What instrument am I learning?",
            "Which instrument am I learning?",
        ],
        states: &["I am learning to play the {v}.", "I am learning the {v}."],
        answers: &[
            "You are learning to play the {v}.",
            "You are learning the {v}.",
        ],
    },
    Relation {
        id: "food",
        asks: &["What is my favorite food?", "Which food do I like best?"],
        states: &["My favorite food is {v}.", "I like {v} best."],
        answers: &["Your favorite food is {v}.", "You like {v} best."],
    },
];

/// Values per relation. Deliberately includes values that are also a *distinct*
/// relation's value (for example `blue` for both colour and car), so the model
/// cannot answer by surfacing a value that only ever belongs to one relation.
const VALUES: &[(&str, &[&str])] = &[
    (
        "name",
        &["Alex", "Sam", "Jordan", "Riley", "Casey", "Morgan"],
    ),
    ("cat", &["Momo", "Piper", "Otis", "Juno", "Cleo", "Milo"]),
    (
        "colour",
        &["green", "blue", "red", "amber", "violet", "teal"],
    ),
    (
        "job",
        &["teacher", "nurse", "engineer", "baker", "pilot", "farmer"],
    ),
    (
        "sister",
        &["Tokyo", "Lisbon", "Denver", "Cairo", "Oslo", "Quito"],
    ),
    (
        "brothers",
        &["two", "three", "four", "five", "six", "seven"],
    ),
    (
        "birthday",
        &["July", "March", "October", "January", "June", "December"],
    ),
    ("car", &["blue", "silver", "black", "white", "red", "green"]),
    (
        "instrument",
        &["piano", "violin", "flute", "guitar", "cello", "harp"],
    ),
    (
        "food",
        &["pizza", "soup", "curry", "noodles", "salad", "rice"],
    ),
];

/// Relations whose value sets overlap, so a distractor can reuse the asked
/// relation's *shape* without being its answer.
const OVERLAP: &[(&str, &str)] = &[("colour", "car"), ("car", "colour")];

fn values_of(id: &str) -> Result<&'static [&'static str]> {
    VALUES
        .iter()
        .find(|(relation, _)| *relation == id)
        .map(|(_, values)| *values)
        .ok_or_else(|| format!("no values for relation {id}"))
}

/// A small deterministic generator: splitmix64, so a run is reproducible from
/// its seed alone and needs no external rng dependency.
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

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// One document's turns as owned strings: a stated asked relation, a stated
/// competing relation, the question, and the correct answer.
///
/// Owned strings rather than borrowed `Message`s so the caller can borrow them
/// for the duration of the encode call -- `Message` holds `&str`, and leaking
/// to satisfy it would be a real leak for no benefit.
struct Document {
    first: String,
    second: String,
    question: String,
    answer: String,
    wanted: String,
}

fn document(rng: &mut Rng) -> Result<Document> {
    let asked = rng.pick(RELATIONS);
    let asked_values = values_of(asked.id)?;
    let wanted = *rng.pick(asked_values);
    // A competing relation, never the asked one.
    let competing = loop {
        let candidate = rng.pick(RELATIONS);
        if candidate.id != asked.id {
            break candidate;
        }
    };
    // Half the time the competing value is drawn from the asked relation's
    // value set when the two overlap, which is the hardest case: the competing
    // statement is a plausible answer to the question.
    let competing_value = match OVERLAP
        .iter()
        .find(|(a, _)| *a == competing.id)
        .map(|(_, other)| *other)
    {
        Some(other) if other == asked.id && rng.below(2) == 0 => *rng.pick(asked_values),
        _ => *rng.pick(values_of(competing.id)?),
    };

    let asked_statement = rng.pick(asked.states).replace("{v}", wanted);
    let competing_statement = rng.pick(competing.states).replace("{v}", competing_value);
    let question = rng.pick(asked.asks).to_string();
    // The asked statement is sometimes stated first and sometimes second, so
    // recency cannot be the rule the corpus teaches.
    let (first, second) = if rng.below(2) == 0 {
        (asked_statement, competing_statement)
    } else {
        (competing_statement, asked_statement)
    };
    Ok(Document {
        first,
        second,
        question,
        answer: rng.pick(asked.answers).replace("{v}", wanted),
        wanted: wanted.to_string(),
    })
}

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<PathBuf> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing {key}="))
    };
    let number = |key: &str, default: u64| -> Result<u64> {
        match args.iter().find_map(|a| a.strip_prefix(&format!("{key}="))) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("{key}= must be a number")),
        }
    };
    let out = arg("out")?;
    let tokenizer_path = arg("tokenizer")?;
    let rows = number("rows", 2000)? as usize;
    let seed = number("seed", 7)?;
    let split = args
        .iter()
        .find_map(|a| a.strip_prefix("split="))
        .unwrap_or("train")
        .to_string();
    if rows == 0 {
        return Err("rows must be at least 1".into());
    }

    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let vocab_size = number("vocab_size", 4096)? as u32;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;

    // Claim the root exclusively before writing anything into it.
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let (bos, eos) = (protocol.bos_id, protocol.eos_id);

    let mut rng = Rng(seed);
    let mut documents: Vec<(Vec<u16>, Vec<u8>)> = Vec::with_capacity(rows);
    let mut total = 0usize;
    let mut response_tokens = 0usize;
    for _ in 0..rows {
        let doc = document(&mut rng)?;
        let messages = [
            Message {
                role: "user",
                content: &doc.first,
            },
            Message {
                role: "user",
                content: &doc.second,
            },
            Message {
                role: "user",
                content: &doc.question,
            },
            Message {
                role: "assistant",
                content: &doc.answer,
            },
        ];
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len() {
            return Err(format!(
                "{} turns were asked for but {} were emitted",
                messages.len(),
                encoded.emitted_turns
            ));
        }
        let ids: Vec<u16> = encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t).map_err(|_| "token id above u16".to_string()))
            .collect::<Result<_>>()?;
        // The mask must mark the answer, or the fit scores nothing.
        if encoded.response_mask.iter().filter(|&&m| m == 1).count() == 0 {
            return Err("a document has no masked response tokens".into());
        }
        if encoded.tokens.len() != encoded.response_mask.len() {
            return Err(format!(
                "the encoder returned {} tokens and {} mask bytes",
                encoded.tokens.len(),
                encoded.response_mask.len()
            ));
        }
        // Decode exactly the masked region -- the scored answer and its EOS --
        // and require the wanted value to appear in it. This is the check that
        // matters: if the mask does not cover the answer, the fit scores the
        // wrong tokens and the corpus is silently useless.
        let scored: Vec<u32> = ids
            .iter()
            .zip(&encoded.response_mask)
            .filter(|(_, &m)| m == 1)
            .map(|(&id, _)| id as u32)
            .collect();
        let scored_text = tokenizer.decode(&scored);
        if !scored_text
            .to_lowercase()
            .contains(&doc.wanted.to_lowercase())
        {
            return Err(format!(
                "the scored region {scored_text:?} does not contain the answer {:?}",
                doc.wanted
            ));
        }
        total += ids.len();
        response_tokens += encoded.response_mask.iter().filter(|&&m| m == 1).count();
        documents.push((ids, encoded.response_mask));
    }

    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab_size).map_err(|e| e.to_string())?;
    let mut mask: Vec<u8> = Vec::with_capacity(total);
    for (ids, row_mask) in &documents {
        writer.write_tokens(ids).map_err(|e| e.to_string())?;
        mask.extend(row_mask);
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != total {
        return Err(format!("wrote {written} tokens, expected {total}"));
    }
    // The mask must be exactly as long as the token stream: the trainer reads
    // them as parallel arrays, and a short mask would silently misalign every
    // document after the first divergence. `total` is the sum of the encoded
    // token vectors, so the two agree by construction unless the encoder's
    // `tokens` and `response_mask` are themselves different lengths, which is
    // checked here because it is the one way this can still be wrong.
    if mask.len() != total {
        return Err(format!(
            "the mask is {} bytes for {total} tokens",
            mask.len()
        ));
    }
    let mask_path = out.join("response_mask.u8");
    fs::write(&mask_path, &mask).map_err(|e| e.to_string())?;

    let tokens_sha = uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?;
    let mask_sha = uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?;
    let tokenizer_sha = uor_r4_training::sha256_file(&tokenizer_path).map_err(|e| e.to_string())?;
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": split,
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.",
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.",
        "synthetic": {
            "generator": "geometric-stack synthetic-memory",
            "why": "the value path needs a distribution of asked-relation vs competing-statement bindings; the ten-row development panel is about 500 tokens",
            "rows": rows,
            "seed": seed,
            "relations": RELATIONS.iter().map(|r| r.id).collect::<Vec<_>>(),
            "properties": [
                "the asked relation and the competing statement are always both stated",
                "the answer is always the asked relation's value",
                "the competing relation is never the asked relation",
                "the asked statement is first half the time and second half the time",
                "overlapping relations (colour/car) sometimes share a competing value"
            ]
        },
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": tokenizer_sha},
        "dialogue_protocol": protocol.schema,
        "bos_id": bos,
        "eos_id": eos,
        "rows_used": rows,
        "tokens": total,
        "response_tokens": response_tokens,
        "response_fraction": response_tokens as f64 / total as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": mask.len(),
        "tokens_sha256": tokens_sha,
        "mask_sha256": mask_sha,
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{rows} documents, {total} tokens, {response_tokens} response tokens ({:.1}%)",
        100.0 * response_tokens as f64 / total as f64
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
