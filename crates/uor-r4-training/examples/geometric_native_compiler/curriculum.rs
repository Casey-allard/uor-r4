//! Prospectively prepared compiler supervision. Templates/labels never enter
//! learned feature extraction. This module constructs data and reference controls.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use uor_r4_training::{
    relation_compiler::{word_spans, Example, NONE},
    sha256_bytes,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}
const DONOR: &str = include_str!("paraphrase-donor-job-home.jsonl");
pub struct StoreEpisode {
    pub id: String,
    pub turns: Vec<Example>,
    pub query_expected: Vec<Option<String>>,
}
pub struct Curriculum {
    pub training: Vec<Example>,
    pub development: Vec<Example>,
    pub fresh: Vec<Example>,
    pub known_phrasing_new_values: Vec<Example>,
    pub new_phrasing_known_values: Vec<Example>,
    pub repeated_values: Vec<Example>,
    pub development_episodes: Vec<StoreEpisode>,
    pub fresh_episodes: Vec<StoreEpisode>,
    pub reference_templates: Vec<Example>,
    pub manifest: Value,
}
#[derive(Clone)]
struct Frame {
    relation: String,
    act: &'static str,
    text: String,
    provenance: Value,
}
fn act(value: &str) -> Result<&'static str> {
    match value {
        "assert" => Ok("assert"),
        "update" => Ok("update"),
        "query" => Ok("query"),
        "none" => Ok(NONE),
        _ => Err(invalid("invalid curriculum act").into()),
    }
}
fn exclusion(text: &str) -> Option<&'static str> {
    match text {
        "Here are 10 different ways to rewrite the sentence \"I just moved to {v}\":" => {
            Some("meta-rewrite instruction does not correct personal home")
        }
        "I drive a {v} now." => Some("vehicle statement does not identify occupation"),
        "I am now {v}." => Some("unspecified relation"),
        "What is my place of employment?" => Some("employer/location question is not occupation"),
        "What activity am I engaged in?" => Some("activity question does not identify occupation"),
        "What role do I fill in my life?" => Some("life role question is not occupation"),
        "What do you think I do for a living?" => {
            Some("asks conjecture rather than saved current occupation")
        }
        "I am a {v} by choice." | "I am a {v} by nature." | "I am a {v}." => {
            Some("bare personal category without explicit occupation")
        }
        "I've switched to a {v}." => Some("unspecified correction relation"),
        _ => None,
    }
}
fn donor_frames() -> Result<(Vec<Frame>, Value)> {
    let mut groups = BTreeMap::<(String, String, String), Vec<Value>>::new();
    let mut raw = 0;
    for line in DONOR.lines() {
        let row: Value = serde_json::from_str(line)?;
        let relation = row["relation"]
            .as_str()
            .ok_or_else(|| invalid("donor relation absent"))?;
        let a = row["act"]
            .as_str()
            .ok_or_else(|| invalid("donor act absent"))?;
        let text = row["text"]
            .as_str()
            .ok_or_else(|| invalid("donor text absent"))?;
        groups
            .entry((relation.into(), a.into(), text.into()))
            .or_default()
            .push(row);
        raw += 1;
    }
    if raw != 153 || groups.len() != 132 {
        return Err(invalid("pinned donor cardinality mismatch").into());
    }
    let mut frames = Vec::new();
    let mut exclusions = Vec::new();
    for ((relation, a, text), sources) in groups {
        if let Some(reason) = exclusion(&text) {
            exclusions.push(json!({"text":text,"reason":reason,"sources":sources}));
            continue;
        }
        let a = act(&a)?;
        if (a == "query" && text.contains("{v}"))
            || (a != "query" && text.matches("{v}").count() != 1)
        {
            return Err(invalid("donor value-slot contract mismatch").into());
        }
        frames.push(Frame {
            relation,
            act: a,
            text,
            provenance: json!({"kind":"retained-paraphrase","sources":sources}),
        });
    }
    let manifest = json!({"fixture_sha256":sha256_bytes(DONOR.as_bytes()),"raw_rows":raw,"distinct_label_text_frames":132,"duplicate_rows":21,"semantic_eligible_distinct_frames":frames.len(),"semantic_exclusions":exclusions,"eligibility_scope":"explicit conservative exclusions;remaining labels retain donor's current-value/assert-update convention;not independent semantic language proof"});
    Ok((frames, manifest))
}
fn cell(frames: &[Frame], relation: &str, a: &str) -> Vec<Frame> {
    frames
        .iter()
        .filter(|f| f.relation == relation && f.act == a)
        .cloned()
        .collect()
}
fn authored(relation: &str, a: &'static str, text: &str, tag: &str) -> Frame {
    Frame {
        relation: relation.into(),
        act: a,
        text: text.into(),
        provenance: json!({"kind":"Rust-authored","group":tag}),
    }
}
fn fill(frame: &Frame, value: &str) -> Example {
    Example {
        text: frame.text.replace("{v}", value),
        relation: frame.relation.clone(),
        act: frame.act,
        template: Some(frame.text.clone()),
    }
}
const VALUES: [&str; 32] = [
    "mariner",
    "carpenter",
    "botanist",
    "cartographer",
    "sculptor",
    "astronomer",
    "chemist",
    "surveyor",
    "azure orchard",
    "copper oasis",
    "mossy plateau",
    "velvet garden",
    "ochre canyon",
    "crimson inlet",
    "ivory forest",
    "violet hillside",
    "quiet cedar beside harbor",
    "bright willow beyond meadow",
    "small maple near river",
    "gentle oak above valley",
    "silver pine beside canyon",
    "warm birch beyond orchard",
    "green elm near plateau",
    "blue ash above garden",
    "the quiet cedar stands beside a blue harbor",
    "a bright willow grows beyond the silver meadow",
    "the small maple rests beside a warm river",
    "a gentle oak rises above the green valley",
    "the silver pine stands beside an ochre canyon",
    "a warm birch grows beyond the violet orchard",
    "the green elm rests beside an ivory plateau",
    "a blue ash rises above the crimson garden",
];
const FACTOR_VALUES: [&str; 8] = [
    "Cerulith",
    "Mornveil",
    "Jade Anchorage",
    "Pearl Crossing",
    "autumn lantern over water",
    "spring compass beside orchard",
    "the autumn lantern shines above the peaceful inlet",
    "a spring compass rests beside the ancient orchard",
];
const FRESH_VALUES: [&str; 8] = [
    "Raventhorn",
    "Selquorin",
    "Umber Landing",
    "Saffron Estuary",
    "winter beacon near hills",
    "summer ribbon above fields",
    "the winter beacon shines beyond the tranquil hillside",
    "a summer ribbon drifts above the quiet pasture",
];
const FRESH2_VALUES: [&str; 8] = [
    "Vellumspire",
    "Duskharbor",
    "Amber Footbridge",
    "Indigo Clearing",
    "dawn lantern beside stream",
    "twilight compass beyond meadow",
    "the dawn lantern stands beside the peaceful stream",
    "a twilight compass rests beyond the open meadow",
];
const FRESH2_NONE: [&str; 8] = [
    "A theatre program printed {v}.",
    "Would an allegory mention {v}?",
    "The novelist gave a character these words: My profession is {v}.",
    "An exhibit quotes the sentence I live in {v}.",
    "A copy editor underlined {v} in a manuscript.",
    "An imaginary travel journal describes {v}.",
    "Could a folk tale contain {v}?",
    "The reading group examined imagery about {v}.",
];
fn fresh2_writes(relation: &str, a: &'static str) -> Vec<Frame> {
    let frames = match (relation, a) {
        ("job", "assert") => [
            "The profession I want recorded is {v}.",
            "For my occupation entry, remember {v}.",
        ],
        ("job", "update") => [
            "Replace my saved occupation with {v}.",
            "Please revise my professional role to {v}.",
        ],
        ("home", "assert") => [
            "The residence I want recorded is {v}.",
            "For my home location entry, remember {v}.",
        ],
        _ => [
            "Replace my saved residence with {v}.",
            "Please revise my home location to {v}.",
        ],
    };
    frames
        .iter()
        .map(|text| authored(relation, a, text, "fresh-crossed-2"))
        .collect()
}
fn fresh2_questions(relation: &str) -> Vec<Frame> {
    let frames = if relation == "job" {
        [
            "What does my saved occupation entry currently say?",
            "Read the professional role I asked you to record.",
            "Can you recover my present profession from memory?",
            "Which recorded occupation is associated with my name?",
        ]
    } else {
        [
            "What does my saved residence entry currently say?",
            "Read the home location I asked you to record.",
            "Can you recover my present residence from memory?",
            "Which recorded home location is associated with my name?",
        ]
    };
    frames
        .iter()
        .map(|text| authored(relation, "query", text, "fresh-crossed-2"))
        .collect()
}
const FRESH3_VALUES: [&str; 8] = [
    "Orielhaven",
    "Fernwatch",
    "Cobalt Causeway",
    "Rosewood Terrace",
    "moon compass beside brook",
    "sunrise banner beyond glade",
    "the moon compass rests beside the still brook",
    "a sunrise banner stands beyond the shaded glade",
];
const FRESH3_NONE: [&str; 8] = [
    "A stage backdrop depicts {v}.",
    "Might a fable refer to {v}?",
    "A playwright wrote this fictional line: My profession is {v}.",
    "The museum labels a quotation: I live in {v}.",
    "A proofreader circled {v} on a draft.",
    "The invented diary contains the phrase {v}.",
    "Does an old ballad describe {v}?",
    "Students discussed the symbolism of {v}.",
];
fn fresh3_writes(relation: &str, a: &'static str) -> Vec<Frame> {
    let frames = match (relation, a) {
        ("job", "assert") => [
            "Make a note of my occupation: {v}.",
            "My own professional designation is {v}.",
        ],
        ("job", "update") => [
            "Amend the occupation you remember for me to {v}.",
            "My professional designation has changed to {v}.",
        ],
        ("home", "assert") => [
            "Make a note of my residence: {v}.",
            "My own residential location is {v}.",
        ],
        _ => [
            "Amend the residence you remember for me to {v}.",
            "My residential location has changed to {v}.",
        ],
    };
    frames
        .iter()
        .map(|text| authored(relation, a, text, "fresh-crossed-3"))
        .collect()
}
fn fresh3_questions(relation: &str) -> Vec<Frame> {
    let frames = if relation == "job" {
        [
            "Tell me the occupation you currently remember for me.",
            "What professional designation did I last give you?",
            "Retrieve my recorded profession, please.",
            "Which occupation did you retain about me?",
        ]
    } else {
        [
            "Tell me the residence you currently remember for me.",
            "What residential location did I last give you?",
            "Retrieve my recorded residence, please.",
            "Which home location did you retain about me?",
        ]
    };
    frames
        .iter()
        .map(|text| authored(relation, "query", text, "fresh-crossed-3"))
        .collect()
}
const TRAIN_NONE: [&str; 16] = [
    "A story mentioned {v}.",
    "Someone painted a picture of {v}.",
    "What makes {v} interesting?",
    "Could a novel describe {v}?",
    "The discussion included {v}.",
    "I heard the phrase {v} in fiction.",
    "A song contains the words {v}.",
    "We compared metaphors about {v}.",
    "The example in the textbook is {v}.",
    "Why might {v} appear in a poem?",
    "People debate the phrase {v}.",
    "A sketch was titled {v}.",
    "Quoted statement: My profession is {v}.",
    "This is a quotation, not my address: I live in {v}.",
    "Here are ways to rewrite the sentence I just moved to {v}.",
    "Do stories use the phrase {v}?",
];
const DEV_NONE: [&str; 8] = [
    "The fictional character discussed {v}.",
    "Can a painting symbolize {v}?",
    "An essay used {v} as an example.",
    "A poet wrote about {v}.",
    "A grammar exercise contains the sentence My job is {v}.",
    "I am quoting somebody else saying I dwell in {v}.",
    "What literary effect could {v} have?",
    "We reviewed a book titled {v}.",
];
const FRESH_NONE: [&str; 8] = [
    "A museum label describes {v}.",
    "Does the expression {v} sound lyrical?",
    "The script gives an actor the line My profession is {v}.",
    "A quotation in the archive reads I live in {v}.",
    "An editor revised a sentence containing {v}.",
    "A fictional diary mentions {v}.",
    "Would a legend feature {v}?",
    "We discussed the symbolism of {v}.",
];
fn authored_questions(relation: &str, split: &str) -> Vec<Frame> {
    let frames = if relation == "job" {
        if split == "development" {
            vec![
                "Please retrieve the occupation I recorded.",
                "Which occupation did I give you?",
                "Return my saved professional role.",
                "What occupation is currently recorded for me?",
                "Please recall the profession in my record.",
                "Which professional role have I stated?",
                "Give the current occupation from my saved details.",
                "What profession did I tell you to remember?",
                "Please report my stored job title.",
                "Which occupation is saved for me?",
                "Return the profession I supplied earlier.",
                "What is the job entry in my saved details?",
                "Can you retrieve my recorded occupation?",
                "Please show my remembered line of work.",
                "Which profession appears in my record?",
                "What saved occupation belongs to me?",
            ]
        } else {
            vec![
                "Read back the occupation from my personal record.",
                "Give me the remembered profession associated with me.",
                "Retrieve the job value I previously recorded.",
                "Tell me the occupation entry you have saved for me.",
            ]
        }
    } else if split == "development" {
        vec![
            "Please retrieve the residence I recorded.",
            "Which residence did I give you?",
            "Return my saved place of residence.",
            "What residence is currently recorded for me?",
            "Please recall the home in my record.",
            "Which home location have I stated?",
            "Give the current residence from my saved details.",
            "What residence did I tell you to remember?",
            "Please report my stored home location.",
            "Which residence is saved for me?",
            "Return the home location I supplied earlier.",
            "What is the home entry in my saved details?",
            "Can you retrieve my recorded residence?",
            "Please show my remembered place of residence.",
            "Which residence appears in my record?",
            "What saved home location belongs to me?",
        ]
    } else {
        vec![
            "Read back the residence from my personal record.",
            "Give me the remembered home location associated with me.",
            "Retrieve the home value I previously recorded.",
            "Tell me the residence entry you have saved for me.",
        ]
    };
    frames
        .into_iter()
        .map(|s| authored(relation, "query", s, split))
        .collect()
}
fn validate_row(row: &Example) -> Result<()> {
    let words = word_spans(&row.text);
    if words.is_empty() || words.len() > 64 {
        return Err(invalid("curriculum word count invalid").into());
    }
    if row.act == "assert" || row.act == "update" {
        let (start, end) = row
            .slot_span()
            .ok_or_else(|| invalid("write lacks exact value span"))?;
        let inside: Vec<_> = words
            .iter()
            .filter(|s| s.start >= start && s.end <= end)
            .collect();
        if inside.is_empty()
            || inside.len() > 8
            || inside.first().map(|s| s.start) != Some(start)
            || inside.last().map(|s| s.end) != Some(end)
        {
            return Err(invalid("write span not exactly representable within eight words").into());
        }
    }
    Ok(())
}
fn validate_panels(panels: &[(&str, &[Example])]) -> Result<Value> {
    let mut labels = BTreeMap::<String, (String, &'static str, Option<(usize, usize)>)>::new();
    let mut split_sources = Vec::new();
    for (name, rows) in panels {
        let mut sources = BTreeSet::new();
        for row in *rows {
            validate_row(row)?;
            let label = (
                row.relation.clone(),
                row.act,
                if row.act == "assert" || row.act == "update" {
                    row.slot_span()
                } else {
                    None
                },
            );
            if let Some(old) = labels.insert(row.text.clone(), label.clone()) {
                if old != label {
                    return Err(
                        invalid("identical source has conflicting supervised labels").into(),
                    );
                }
            }
            sources.insert(row.text.clone());
        }
        split_sources.push((*name, sources));
    }
    for i in 0..panels.len() {
        for j in i + 1..panels.len() {
            if !split_sources[i].1.is_disjoint(&split_sources[j].1) {
                return Err(invalid("curriculum panel source overlap").into());
            }
        }
    }
    for i in [1usize, 2] {
        if split_sources[i].1.len() != panels[i].1.len() {
            return Err(invalid("development or fresh source duplicate").into());
        }
    }
    Ok(json!(split_sources.iter().map(|(name,set)|json!({"panel":name,"visits":panels.iter().find(|p|p.0==*name).map(|p|p.1.len()),"distinct_sources":set.len()})).collect::<Vec<_>>()))
}
fn episode(
    id: &str,
    writes: &BTreeMap<(String, String), Vec<Frame>>,
    queries: &BTreeMap<String, Vec<Frame>>,
    values: &[&str],
    reverse: bool,
) -> Result<StoreEpisode> {
    let get = |relation: &str, a: &str, index: usize| -> Result<&Frame> {
        writes
            .get(&(relation.into(), a.into()))
            .and_then(|r| r.get(index))
            .ok_or_else(|| invalid("episode frame missing").into())
    };
    let query = |relation: &str| -> Result<&Frame> {
        queries
            .get(relation)
            .and_then(|r| r.first())
            .ok_or_else(|| invalid("episode query missing").into())
    };
    let mut turns = vec![
        fill(get("job", "assert", 0)?, values[0]),
        fill(get("home", "assert", 0)?, values[1]),
    ];
    if reverse {
        turns.reverse();
    }
    let mut expected = vec![None, None];
    turns.push(fill(query("job")?, ""));
    expected.push(Some(values[0].into()));
    turns.push(fill(query("home")?, ""));
    expected.push(Some(values[1].into()));
    turns.push(fill(get("job", "update", 1)?, values[2]));
    expected.push(None);
    turns.push(fill(get("home", "assert", 1)?, values[1]));
    expected.push(None);
    turns.push(fill(query("job")?, ""));
    expected.push(Some(values[2].into()));
    turns.push(fill(query("home")?, ""));
    expected.push(Some(values[1].into()));
    Ok(StoreEpisode {
        id: id.into(),
        turns,
        query_expected: expected,
    })
}
pub fn build() -> Result<Curriculum> {
    build_profile("crossed-1")
}
/// Crossed-2 and crossed-3 change only prospective fresh rows and store episodes.
/// Training, development and opened factors remain exactly the retained inputs.
pub fn build_profile(profile: &str) -> Result<Curriculum> {
    if !matches!(profile, "crossed-1" | "crossed-2" | "crossed-3") {
        return Err(invalid("unknown crossed curriculum profile").into());
    }
    let curriculum = build_inner(profile)?;
    if profile != "crossed-1" {
        for old_profile in if profile == "crossed-3" {
            vec!["crossed-1", "crossed-2"]
        } else {
            vec!["crossed-1"]
        } {
            let old = build_inner(old_profile)?;
            let exposed_rows: Vec<&Example> = old
                .training
                .iter()
                .chain(&old.development)
                .chain(&old.fresh)
                .chain(&old.known_phrasing_new_values)
                .chain(&old.new_phrasing_known_values)
                .chain(&old.repeated_values)
                .chain(old.development_episodes.iter().flat_map(|e| e.turns.iter()))
                .chain(old.fresh_episodes.iter().flat_map(|e| e.turns.iter()))
                .collect();
            let exposed_sources: BTreeSet<&str> =
                exposed_rows.iter().map(|r| r.text.as_str()).collect();
            let exposed_values: BTreeSet<&str> = exposed_rows
                .iter()
                .filter(|r| matches!(r.act, "assert" | "update"))
                .filter_map(|r| r.slot_value())
                .collect();
            for value in if profile == "crossed-3" {
                FRESH3_VALUES
            } else {
                FRESH2_VALUES
            } {
                if exposed_values.contains(value) {
                    return Err(invalid("fresh literal was previously exposed").into());
                }
            }
            for row in curriculum.fresh.iter().chain(
                curriculum
                    .fresh_episodes
                    .iter()
                    .flat_map(|e| e.turns.iter()),
            ) {
                if exposed_sources.contains(row.text.as_str()) {
                    return Err(invalid("fresh source was previously exposed").into());
                }
            }
        }
    }
    Ok(curriculum)
}
fn build_inner(profile: &str) -> Result<Curriculum> {
    let fresh_values = if profile == "crossed-3" {
        &FRESH3_VALUES
    } else if profile == "crossed-2" {
        &FRESH2_VALUES
    } else {
        &FRESH_VALUES
    };
    let fresh_none = if profile == "crossed-3" {
        &FRESH3_NONE
    } else if profile == "crossed-2" {
        &FRESH2_NONE
    } else {
        &FRESH_NONE
    };
    let (frames, donor) = donor_frames()?;
    let training_literals: BTreeSet<_> = VALUES.iter().copied().collect();
    let factor_literals: BTreeSet<_> = FACTOR_VALUES.iter().copied().collect();
    let fresh_literals: BTreeSet<_> = fresh_values.iter().copied().collect();
    if training_literals.len() != 32
        || factor_literals.len() != 8
        || fresh_literals.len() != 8
        || !training_literals.is_disjoint(&factor_literals)
        || !training_literals.is_disjoint(&fresh_literals)
        || !factor_literals.is_disjoint(&fresh_literals)
    {
        return Err(invalid("curriculum value pools must be unique and disjoint").into());
    }
    for (i, value) in VALUES.iter().enumerate() {
        if word_spans(value).len() != [1, 2, 4, 8][i / 8] {
            return Err(invalid("training value length stratum mismatch").into());
        }
    }
    for values in [&FACTOR_VALUES, fresh_values] {
        for (i, value) in values.iter().enumerate() {
            if word_spans(value).len() != [1, 2, 4, 8][i / 2] {
                return Err(invalid("held-out value length stratum mismatch").into());
            }
        }
    }
    let mut training = Vec::new();
    let mut development = Vec::new();
    let mut fresh = Vec::new();
    let mut known_phrasing_new_values = Vec::new();
    let mut new_phrasing_known_values = Vec::new();
    let mut repeated_values = Vec::new();
    let mut reference_templates = Vec::new();
    let mut partitions = Vec::new();
    let mut dev_write = BTreeMap::new();
    let mut fresh_write = BTreeMap::new();
    let mut dev_query = BTreeMap::new();
    let mut fresh_query = BTreeMap::new();
    for relation in ["job", "home"] {
        for a in ["assert", "update"] {
            let candidates = cell(&frames, relation, a);
            if candidates.len() < 12 {
                return Err(invalid("insufficient eligible write frames").into());
            }
            let train = &candidates[..8];
            let wording = &candidates[8..10];
            let new = if profile == "crossed-3" {
                fresh3_writes(relation, a)
            } else if profile == "crossed-2" {
                fresh2_writes(relation, a)
            } else {
                candidates[10..12].to_vec()
            };
            partitions.push(json!({"relation":relation,"act":a,"training_frames":train.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>(),"factor_frames":wording.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>(),"fresh_frames":new.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>() }));
            for (f, frame) in train.iter().enumerate() {
                for j in 0..8 {
                    training.push(fill(frame, VALUES[(4 * f + j) % 32]));
                }
                for offset in [8, 17] {
                    development.push(fill(frame, VALUES[(4 * f + offset) % 32]));
                }
                for value in [FACTOR_VALUES[f], FACTOR_VALUES[(f + 4) % 8]] {
                    known_phrasing_new_values.push(fill(frame, value));
                }
            }
            for (f, frame) in wording.iter().enumerate() {
                for j in 0..8 {
                    new_phrasing_known_values.push(fill(frame, VALUES[(f * 16 + j * 2) % 32]));
                }
            }
            for (f, frame) in new.iter().enumerate() {
                for j in 0..4 {
                    fresh.push(fill(frame, fresh_values[f * 4 + j]));
                }
            }
            for (f, frame) in train.iter().take(2).enumerate() {
                for j in 0..2 {
                    let value = VALUES[(f * 8 + j) % 32];
                    repeated_values.push(fill(frame, &format!("{value} {value}")));
                }
            }
            dev_write.insert((relation.into(), a.into()), train[..2].to_vec());
            fresh_write.insert((relation.into(), a.into()), new.to_vec());
            reference_templates
                .extend(train.iter().chain(wording).chain(&new).map(|f| fill(f, "")));
        }
        let questions = cell(&frames, relation, "query");
        if questions.len() < 16 {
            return Err(invalid("insufficient eligible query frames").into());
        }
        let train = &questions[..16];
        for frame in train {
            for _ in 0..4 {
                training.push(fill(frame, ""));
            }
        }
        let mut dev = questions[16..].iter().take(16).cloned().collect::<Vec<_>>();
        for frame in authored_questions(relation, "development") {
            if dev.len() == 16 {
                break;
            }
            if !questions.iter().any(|f| f.text == frame.text)
                && !dev.iter().any(|f| f.text == frame.text)
            {
                dev.push(frame);
            }
        }
        if dev.len() != 16 {
            return Err(invalid("development question diversity inadequate").into());
        }
        let new = if profile == "crossed-3" {
            fresh3_questions(relation)
        } else if profile == "crossed-2" {
            fresh2_questions(relation)
        } else {
            authored_questions(relation, "fresh")
        };
        development.extend(dev.iter().map(|f| fill(f, "")));
        fresh.extend(new.iter().map(|f| fill(f, "")));
        dev_query.insert(relation.into(), dev.clone());
        fresh_query.insert(relation.into(), new.clone());
        partitions.push(json!({"relation":relation,"act":"query","training_frames":train.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>(),"development_frames":dev.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>(),"fresh_frames":new.iter().map(|f|json!({"text":f.text,"provenance":f.provenance})).collect::<Vec<_>>() }));
        reference_templates.extend(train.iter().chain(&dev).chain(&new).map(|f| fill(f, "")));
    }
    for (f, text) in TRAIN_NONE.iter().enumerate() {
        let frame = authored(NONE, NONE, text, "training-negative");
        for j in 0..8 {
            training.push(fill(&frame, VALUES[(2 * f + j) % 32]));
        }
        reference_templates.push(fill(&frame, ""));
    }
    for (f, text) in DEV_NONE.iter().enumerate() {
        let frame = authored(NONE, NONE, text, "development-negative");
        for j in 0..4 {
            development.push(fill(&frame, VALUES[(4 * f + j) % 32]));
        }
        reference_templates.push(fill(&frame, ""));
    }
    for (f, text) in fresh_none.iter().enumerate() {
        let frame = authored(NONE, NONE, text, "fresh-negative");
        fresh.push(fill(&frame, fresh_values[f]));
        reference_templates.push(fill(&frame, ""));
    }
    if (
        training.len(),
        development.len(),
        fresh.len(),
        known_phrasing_new_values.len(),
        new_phrasing_known_values.len(),
    ) != (512, 128, 48, 64, 64)
    {
        return Err(invalid("curriculum count mismatch").into());
    }
    let mut counts = BTreeMap::<(String, String, String), usize>::new();
    for row in &training {
        if row.act == "assert" || row.act == "update" {
            let value = row
                .slot_value()
                .ok_or_else(|| invalid("training value unavailable"))?;
            *counts
                .entry((row.relation.clone(), row.act.into(), value.into()))
                .or_default() += 1;
        }
    }
    for relation in ["job", "home"] {
        for a in ["assert", "update"] {
            for value in VALUES {
                if counts.get(&(relation.into(), a.into(), value.into())) != Some(&2) {
                    return Err(invalid("role-act-value balance invalid").into());
                }
            }
        }
    }
    // Each cell uses the same sixteen held-out conjunction values once each;
    // these source/value edges are absent from that cell's training frame.
    let mut development_value_sets = Vec::new();
    for relation in ["job", "home"] {
        for a in ["assert", "update"] {
            let mut degrees = BTreeMap::<String, usize>::new();
            for row in development
                .iter()
                .filter(|r| r.relation == relation && r.act == a)
            {
                let value = row
                    .slot_value()
                    .ok_or_else(|| invalid("development write value missing"))?;
                *degrees.entry(value.into()).or_default() += 1;
                if training.iter().any(|t| t.text == row.text) {
                    return Err(invalid("development conjunction edge present in training").into());
                }
            }
            if degrees.len() != 16 || degrees.values().any(|degree| *degree != 1) {
                return Err(invalid("development conjunction value degree invalid").into());
            }
            development_value_sets.push(degrees.into_keys().collect::<BTreeSet<_>>());
        }
    }
    if development_value_sets
        .windows(2)
        .any(|pair| pair[0] != pair[1])
    {
        return Err(invalid("development conjunction value sets differ by role/act").into());
    }
    let panels = validate_panels(&[
        ("training", &training),
        ("development", &development),
        ("fresh", &fresh),
        ("known_phrasing_new_values", &known_phrasing_new_values),
        ("new_phrasing_known_values", &new_phrasing_known_values),
        ("repeated_values", &repeated_values),
    ])?;
    let development_episodes = vec![
        episode(
            "development-order-forward",
            &dev_write,
            &dev_query,
            &VALUES,
            false,
        )?,
        episode(
            "development-order-reversed",
            &dev_write,
            &dev_query,
            &VALUES,
            true,
        )?,
        episode(
            "development-four-eight-word-values",
            &dev_write,
            &dev_query,
            &[VALUES[16], VALUES[24], VALUES[17]],
            false,
        )?,
    ];
    let fresh_episodes = vec![
        episode(
            "fresh-order-forward",
            &fresh_write,
            &fresh_query,
            fresh_values,
            false,
        )?,
        episode(
            "fresh-order-reversed",
            &fresh_write,
            &fresh_query,
            fresh_values,
            true,
        )?,
        episode(
            "fresh-four-eight-word-values",
            &fresh_write,
            &fresh_query,
            &[fresh_values[4], fresh_values[6], fresh_values[5]],
            false,
        )?,
    ];
    for episode in development_episodes.iter().chain(&fresh_episodes) {
        if episode.turns.len() != episode.query_expected.len() {
            return Err(invalid("episode scorer alignment mismatch").into());
        }
        for (turn, expected) in episode.turns.iter().zip(&episode.query_expected) {
            validate_row(turn)?;
            if (turn.act == "query") != expected.is_some() {
                return Err(invalid("episode query expectation mismatch").into());
            }
        }
    }
    let manifest = json!({"schema":"uor-r4.compiler-crossed-curriculum/1","profile":profile,"fresh_exposed_exclusion":"crossed-2 excludes crossed-1;crossed-3 excludes crossed-1+crossed-2 exact source/literal overlap across training/development/factors/fresh and original store episode turns;driver additionally admits legacy/local exclusion;no claim of every encoder corpus exclusion","donor":donor,"partitions":partitions,"negative_frame_provenance":{"kind":"Rust-authored","training":TRAIN_NONE,"development":DEV_NONE,"fresh":fresh_none},"training_values":VALUES,"factor_values":FACTOR_VALUES,"fresh_values":fresh_values,"training_word_length_strata":[1,2,4,8],"factor_value_word_lengths":FACTOR_VALUES.iter().map(|v|word_spans(v).len()).collect::<Vec<_>>(),"fresh_value_word_lengths":fresh_values.iter().map(|v|word_spans(v).len()).collect::<Vec<_>>(),"training_schedule":"four cells8frames*8rotatingvalues;32values degree2percell;32distinctqueries4visits;16noneframes8rotatingvalues","selection":"128 distinct development source rows only;64 missing conjunction writes+32unseen questions+32unseen prose","panels":panels,"training_value_role_act_count":2,"development_conjunction_offsets":[8,17],"development_value_role_act_degree":1,"development_distinct_shared_conjunction_values":16,"runtime_templates":"labels/referenceinstrument only;never native features","exclusion_scope":"source-level train/dev/fresh disjoint;driver additionally checks exposed legacy/local panels","native_token_admission":"driver must verify all originaltext BPE <=128, not inferred from words","store_episodes":"predicted actions only;natural assertions/query;correct job and same-value home reassertion;both short-value orderings plus four/eight-word episode;expected values scorer only"});
    Ok(Curriculum {
        training,
        development,
        fresh,
        known_phrasing_new_values,
        new_phrasing_known_values,
        repeated_values,
        development_episodes,
        fresh_episodes,
        reference_templates,
        manifest,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crossed3_retains_fixed_inputs_and_excludes_both_exposed_profiles() -> Result<()> {
        let new = build_profile("crossed-3")?;
        let identity = |rows: &[Example]| {
            rows.iter()
                .map(|r| {
                    (
                        r.text.clone(),
                        r.relation.clone(),
                        r.act,
                        r.template.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        for profile in ["crossed-1", "crossed-2"] {
            let old = build_profile(profile)?;
            for (a, b) in [
                (&old.training, &new.training),
                (&old.development, &new.development),
                (
                    &old.known_phrasing_new_values,
                    &new.known_phrasing_new_values,
                ),
                (
                    &old.new_phrasing_known_values,
                    &new.new_phrasing_known_values,
                ),
                (&old.repeated_values, &new.repeated_values),
            ] {
                assert_eq!(identity(a), identity(b));
            }
            let exposed: BTreeSet<_> = old
                .fresh
                .iter()
                .chain(old.fresh_episodes.iter().flat_map(|e| e.turns.iter()))
                .map(|r| r.text.as_str())
                .collect();
            assert!(new
                .fresh
                .iter()
                .chain(new.fresh_episodes.iter().flat_map(|e| e.turns.iter()))
                .all(|r| !exposed.contains(r.text.as_str())));
            let templates: BTreeSet<_> = old
                .reference_templates
                .iter()
                .filter_map(|r| r.template.as_deref())
                .collect();
            assert!(new
                .fresh
                .iter()
                .filter_map(|r| r.template.as_deref())
                .all(|t| !templates.contains(t)));
        }
        assert_eq!(new.fresh.len(), 48);
        assert_eq!(new.fresh_episodes.len(), 3);
        Ok(())
    }
    #[test]
    fn crossed2_changes_only_unexposed_fresh_rows_and_episodes() -> Result<()> {
        let old = build()?;
        let new = build_profile("crossed-2")?;
        let identity = |rows: &[Example]| {
            rows.iter()
                .map(|r| {
                    (
                        r.text.clone(),
                        r.relation.clone(),
                        r.act,
                        r.template.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        for (before, after) in [
            (&old.training, &new.training),
            (&old.development, &new.development),
            (
                &old.known_phrasing_new_values,
                &new.known_phrasing_new_values,
            ),
            (
                &old.new_phrasing_known_values,
                &new.new_phrasing_known_values,
            ),
            (&old.repeated_values, &new.repeated_values),
        ] {
            assert_eq!(identity(before), identity(after));
        }
        let exposed: BTreeSet<_> = old
            .training
            .iter()
            .chain(&old.development)
            .chain(&old.fresh)
            .chain(&old.known_phrasing_new_values)
            .chain(&old.new_phrasing_known_values)
            .chain(&old.repeated_values)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(new.fresh.len(), 48);
        assert!(new.fresh.iter().all(|r| !exposed.contains(r.text.as_str())));
        assert_eq!(new.manifest["profile"], "crossed-2");
        assert_eq!(new.fresh_episodes.len(), 3);
        for (before, after) in old
            .development_episodes
            .iter()
            .zip(&new.development_episodes)
        {
            assert_eq!(identity(&before.turns), identity(&after.turns));
            assert_eq!(before.query_expected, after.query_expected);
        }
        let old_templates: BTreeSet<_> = old
            .reference_templates
            .iter()
            .filter_map(|r| r.template.as_deref())
            .collect();
        assert!(new.fresh.iter().all(|r| r
            .template
            .as_deref()
            .is_some_and(|t| !old_templates.contains(t))));
        assert!(build_profile("unknown").is_err());
        Ok(())
    }
    #[test]
    fn crossed_curriculum_distinct_balanced_and_eligible() -> Result<()> {
        let c = build()?;
        assert_eq!(
            (c.training.len(), c.development.len(), c.fresh.len()),
            (512, 128, 48)
        );
        let counts = |rows: &[Example]| {
            let mut n = BTreeMap::new();
            for r in rows {
                *n.entry(r.act).or_insert(0) += 1;
            }
            n
        };
        assert!(counts(&c.training).values().all(|n| *n == 128));
        assert!(counts(&c.development).values().all(|n| *n == 32));
        assert_eq!(
            c.development
                .iter()
                .map(|r| &r.text)
                .collect::<BTreeSet<_>>()
                .len(),
            128
        );
        assert_eq!(
            c.fresh
                .iter()
                .map(|r| &r.text)
                .collect::<BTreeSet<_>>()
                .len(),
            48
        );
        assert_eq!(c.manifest["donor"]["raw_rows"], 153);
        assert_eq!(c.manifest["donor"]["distinct_label_text_frames"], 132);
        Ok(())
    }
    #[test]
    fn episode_expectations_match_typed_labels_and_original_spans() -> Result<()> {
        let c = build()?;
        for e in c.development_episodes.iter().chain(&c.fresh_episodes) {
            assert_eq!(e.turns.len(), e.query_expected.len());
            for (t, expected) in e.turns.iter().zip(&e.query_expected) {
                validate_row(t)?;
                assert_eq!(t.act == "query", expected.is_some());
            }
        }
        Ok(())
    }
}
