//! Deterministic generator, noise-spread analyzer, and overlap/seal verifier
//! for the 256-row M2 fresh held-out reply panel (`m2-fresh256`, issue #2849).
//!
//! Zero floating-point types or operations are used anywhere in this crate.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_tokenizer::ByteBpeTokenizer;

/// Sealed SHA-256 of the saved437 cross-state field (`archive/labs/m2-fresh-437-2026-10-10/README.md:97`).
pub const SAVED437_FIELD_SHA256: &str =
    "de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61";

/// Sealed SHA-256 of the 512-row development inputs (`archive/labs/m2-fresh-437-2026-10-10/README.md:100`).
pub const DEV512_INPUTS_SHA256: &str =
    "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";

/// Sealed SHA-256 of the opened 128-row transfer inputs (`archive/labs/m2-fresh-437-2026-10-10/README.md:105`).
pub const OPENED_FRESH128_INPUTS_SHA256: &str =
    "c5a643ff9ef86f5f16f15b420f42fe8b1921c5c8c7df5a7bbb88f7759cda5bda";

/// Pinned SHA-256 of the 4,096-token `ByteBpeTokenizer` (`tokenizer.json`).
pub const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";

/// Pinned EOS token ID in `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
pub const EOS_TOKEN_ID: u32 = 1;

/// Pinned period (`.` ) token ID in `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
pub const PERIOD_TOKEN_ID: u32 = 16;

/// Ten 2-word fresh target literals for `length2` (all 10) and `reassert` (first 8).
/// Content words are 100% disjoint from all historical curriculum/panel literals and across all 34 fresh literals.
pub const FRESH256_LEN2: [&str; 10] = [
    "basalt loggia",
    "syenite annex",
    "quartzite promenade",
    "dolomite veranda",
    "obsidian courtyard",
    "travertine pavilion",
    "feldspar gallery",
    "malachite rotunda",
    "alabaster cloister",
    "porphyry belvedere",
];

/// Four distinct 2-word initial-assertion literals overwritten in `update` histories.
pub const FRESH256_UPDATE_INIT: [&str; 4] = [
    "limestone pergola",
    "sandstone arcade",
    "chert conservatory",
    "chalk esplanade",
];

/// Ten 4-word fresh target literals for `length4` and `update`.
pub const FRESH256_LEN4: [&str; 10] = [
    "calm heron near marshland",
    "swift falcon over headland",
    "bold osprey by cliffside",
    "keen plover along dikeway",
    "lone curlew across moorland",
    "fleet swallow under viaduct",
    "stout badger within thicket",
    "brisk otter beside millrace",
    "agile marten around coppice",
    "steady kestrel above fenland",
];

/// Ten 8-word fresh target literals for `length8`.
pub const FRESH256_LEN8: [&str; 10] = [
    "the bronze kettle waits near a cobble seawall",
    "a carved violin gleams under the stone bridge",
    "the iron anchor hangs within a dry cabin",
    "a brass bell stands along the high cliff",
    "the pewter goblet sits across a tiled porch",
    "a glass prism ticks inside the old tower",
    "the steel pendulum swings outside a brick foundry",
    "a slate mirror leans around the steep ledge",
    "the nickel whistle points over a wide marsh",
    "a flint wheel turns below the arched gate",
];

/// All 74 historical curriculum and transfer literals from
/// `crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs` (64 literals)
/// and `crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs` (10 literals).
/// `verify_m2_fresh256_panel` parses those repository files directly from disk and verifies
/// that the on-disk literals match this list and have zero overlap with `m2-fresh256`.
pub const HISTORICAL_EXPOSED_LITERALS: [&str; 74] = [
    // VALUES[0..32] from crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs
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
    // FACTOR_VALUES[0..8]
    "Cerulith",
    "Mornveil",
    "Jade Anchorage",
    "Pearl Crossing",
    "autumn lantern over water",
    "spring compass beside orchard",
    "the autumn lantern shines above the peaceful inlet",
    "a spring compass rests beside the ancient orchard",
    // FRESH_VALUES[0..8]
    "Raventhorn",
    "Selquorin",
    "Umber Landing",
    "Saffron Estuary",
    "winter beacon near hills",
    "summer ribbon above fields",
    "the winter beacon shines beyond the tranquil hillside",
    "a summer ribbon drifts above the quiet pasture",
    // FRESH2_VALUES[0..8]
    "Vellumspire",
    "Duskharbor",
    "Amber Footbridge",
    "Indigo Clearing",
    "dawn lantern beside stream",
    "twilight compass beyond meadow",
    "the dawn lantern stands beside the peaceful stream",
    "a twilight compass rests beyond the open meadow",
    // FRESH3_VALUES[0..8]
    "Orielhaven",
    "Fernwatch",
    "Cobalt Causeway",
    "Rosewood Terrace",
    "moon compass beside brook",
    "sunrise banner beyond glade",
    "the moon compass rests beside the still brook",
    "a sunrise banner stands beyond the shaded glade",
    // Untouched transfer literals from crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs
    "amber willow",
    "copper cedar",
    "amber birch silver willow copper cedar harbor silver",
    "silver cedar copper birch amber willow amber harbor",
    "violet willow",
    "copper meadow silver birch",
    "amber cedar violet willow",
    "orchard silver harbor amber",
    "birch copper",
    "willow copper birch violet",
];

/// Supported current-role job queries (`geometric-native-bank-observation-panel.rs:177`).
pub const CURRENT_JOB_QUERIES: [&str; 6] = [
    "What is my current job?",
    "What job do I currently have?",
    "What is my job now?",
    "Which job do I have now?",
    "Remind me of my current job.",
    "Tell me my current job.",
];

/// Supported current-role home queries (`geometric-native-bank-observation-panel.rs:189`).
pub const CURRENT_HOME_QUERIES: [&str; 6] = [
    "Where do I currently live?",
    "Where do I live now?",
    "What is my current residence?",
    "Remind me where I currently live.",
    "Tell me where I live now.",
    "Where is my current home?",
];

/// Exact pre-token BPE encodings from the pinned 4,096-token `ByteBpeTokenizer`
/// (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`,
/// `ByteLevel(add_prefix_space=false, use_regex=true)`).
const PRETOKEN_BPE_TABLE: &[(&str, &[u32])] = &[
    (" I", &[340]),
    (" a", &[261]),
    (" above", &[642, 2156]),
    (" across", &[2318]),
    (" agile", &[559, 599]),
    (" alabaster", &[554, 621, 795, 284]),
    (" along", &[2321]),
    (" anchor", &[3984]),
    (" annex", &[423, 80, 839]),
    (" arcade", &[1419, 69, 566]),
    (" arched", &[1419, 814]),
    (" around", &[622]),
    (" badger", &[892, 1047]),
    (" basalt", &[267, 639, 421, 86]),
    (" bell", &[2046]),
    (" below", &[1255, 320]),
    (" belvedere", &[1255, 414, 1833]),
    (" beside", &[324, 520]),
    (" bold", &[408, 342]),
    (" brass", &[975, 560]),
    (" brick", &[3909, 350]),
    (" bridge", &[2701]),
    (" brisk", &[564, 1874]),
    (" bronze", &[564, 292, 1753]),
    (" by", &[860]),
    (" cabin", &[2870]),
    (" calm", &[2292]),
    (" carved", &[526, 414]),
    (" chalk", &[3355]),
    (" changed", &[2878]),
    (" chert", &[2806, 86]),
    (" cliff", &[499, 2548]),
    (" cliffside", &[499, 2548, 520]),
    (" cloister", &[788, 1636]),
    (" cobble", &[3566, 1076]),
    (" conservatory", &[1205, 2728, 3426, 91]),
    (" coppice", &[581, 330, 552]),
    (" courtyard", &[1955, 576, 471]),
    (" curlew", &[280, 456, 763]),
    (" current", &[280, 456, 275, 349]),
    (" currently", &[280, 456, 275, 349, 463]),
    (" dikeway", &[279, 1326, 479]),
    (" do", &[377]),
    (" dolomite", &[377, 78, 286, 718]),
    (" dry", &[1622]),
    (" esplanade", &[347, 85, 805, 304, 566]),
    (" falcon", &[271, 421, 69, 292]),
    (" feldspar", &[635, 342, 2453, 291]),
    (" fenland", &[271, 303, 3108]),
    (" fleet", &[2333, 309]),
    (" flint", &[608, 272, 86]),
    (" foundry", &[505, 441]),
    (" gallery", &[298, 361, 2256]),
    (" gate", &[2553]),
    (" glass", &[2537]),
    (" gleams", &[298, 305, 308, 85]),
    (" goblet", &[480, 1076, 86]),
    (" hangs", &[2576, 85]),
    (" has", &[1139]),
    (" have", &[500]),
    (" headland", &[1153, 3108]),
    (" heron", &[338, 292]),
    (" high", &[901]),
    (" home", &[582]),
    (" in", &[315]),
    (" inside", &[747]),
    (" iron", &[223, 307, 292]),
    (" is", &[435]),
    (" job", &[1541]),
    (" keen", &[723, 303]),
    (" kestrel", &[723, 353, 985]),
    (" kettle", &[402, 309, 86, 305]),
    (" leans", &[3103, 85]),
    (" ledge", &[3751, 781]),
    (" limestone", &[4065, 353, 511]),
    (" live", &[2097]),
    (" loggia", &[438, 1916, 587]),
    (" lone", &[278, 511]),
    (" malachite", &[283, 421, 593, 718]),
    (" marsh", &[1477, 1389]),
    (" marshland", &[1477, 1389, 3108]),
    (" marten", &[1477, 3553]),
    (" me", &[519]),
    (" millrace", &[3116, 453, 339]),
    (" mirror", &[2663]),
    (" moorland", &[752, 306, 3108]),
    (" my", &[627]),
    (" near", &[820]),
    (" nickel", &[290, 465, 346, 78]),
    (" now", &[960]),
    (" obsidian", &[2048, 85, 295, 2015]),
    (" of", &[372]),
    (" old", &[782]),
    (" osprey", &[332, 2453, 275, 91]),
    (" otter", &[3583]),
    (" outside", &[699]),
    (" over", &[883]),
    (" pavilion", &[1103, 88, 316, 848]),
    (" pendulum", &[281, 355, 498, 494]),
    (" pergola", &[1364, 73, 1976, 67]),
    (" pewter", &[281, 662, 442]),
    (" plover", &[317, 81, 322]),
    (" points", &[1678, 1657]),
    (" porch", &[4071]),
    (" porphyry", &[3957, 1535, 91, 441]),
    (" prism", &[615, 289, 79]),
    (" promenade", &[1361, 303, 566]),
    (" quartzite", &[949, 1527, 92, 718]),
    (" residence", &[1673, 295, 1691]),
    (" rotunda", &[3679, 2772, 67]),
    (" sandstone", &[1193, 353, 511]),
    (" seawall", &[1582, 89, 361]),
    (" sits", &[1405, 85]),
    (" slate", &[756, 834]),
    (" stands", &[1873, 85]),
    (" steady", &[1422, 907]),
    (" steel", &[1422, 418]),
    (" steep", &[1422, 612]),
    (" stone", &[1881]),
    (" stout", &[345, 410]),
    (" swallow", &[637, 361, 320]),
    (" swift", &[637, 613, 86]),
    (" swings", &[1889]),
    (" syenite", &[262, 91, 303, 718]),
    (" the", &[265]),
    (" thicket", &[2662, 309]),
    (" ticks", &[2826, 85]),
    (" tiled", &[259, 556]),
    (" to", &[268]),
    (" tower", &[1581]),
    (" travertine", &[1015, 322, 86, 819]),
    (" turns", &[1579]),
    (" under", &[775]),
    (" veranda", &[223, 322, 3336]),
    (" viaduct", &[3682, 373, 592, 86]),
    (" violin", &[3811]),
    (" waits", &[1168, 85]),
    (" wheel", &[2153]),
    (" where", &[1059]),
    (" whistle", &[2167]),
    (" wide", &[2149]),
    (" within", &[331, 272]),
    (".", &[16]),
    ("?", &[33]),
    ("I", &[43]),
    ("My", &[2485]),
    ("Remind", &[52, 970, 445]),
    ("Tell", &[54, 540]),
    ("What", &[1085]),
    ("Where", &[2812]),
    ("Which", &[759, 1788]),
    ("a", &[67]),
    ("agile", &[547, 599]),
    ("alabaster", &[421, 621, 795, 284]),
    ("basalt", &[68, 639, 421, 86]),
    ("bold", &[1321, 342]),
    ("brisk", &[1986, 1874]),
    ("calm", &[69, 2106]),
    ("chalk", &[397, 2773]),
    ("chert", &[69, 370, 86]),
    ("dolomite", &[70, 1976, 286, 718]),
    ("feldspar", &[722, 342, 2453, 291]),
    ("fleet", &[72, 305, 309]),
    ("keen", &[346, 303]),
    ("limestone", &[78, 395, 353, 511]),
    ("lone", &[78, 511]),
    ("malachite", &[79, 421, 593, 718]),
    ("obsidian", &[567, 85, 295, 2015]),
    ("porphyry", &[82, 306, 1535, 91, 441]),
    ("quartzite", &[675, 1527, 92, 718]),
    ("sandstone", &[85, 789, 353, 511]),
    ("steady", &[353, 71, 907]),
    ("stout", &[353, 410]),
    ("swift", &[1903, 613, 86]),
    ("syenite", &[1897, 303, 718]),
    ("the", &[86, 260]),
    ("travertine", &[86, 453, 322, 86, 819]),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wire {
    pub text: String,
    pub relation: String,
    pub act: String,
    pub template: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    pub id: String,
    pub stratum: String,
    pub turns: Vec<Wire>,
    pub queries: Vec<Wire>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeInputs {
    pub schema: String,
    pub cases: Vec<ProbePacket>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbePacket {
    pub id: String,
    pub segments: Vec<ProbeSegment>,
    pub query_ids: Vec<u32>,
    pub actual_prefix_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ProbeSegment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeLabels {
    pub schema: String,
    pub protocol: String,
    pub membership_only: bool,
    pub cases: Vec<ProbeLabel>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeLabel {
    pub id: String,
    pub pair_id: String,
    pub answers: FrozenAnswers,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dev512SpreadReport {
    pub total_rows: usize,
    pub complete_rows: usize,
    /// Status string copied from the saved 2026-10-10 replay audit (`development-replay-audit-summary.json`).
    pub saved_audit_status: String,
    /// Checked step count copied from the saved 2026-10-10 replay audit (`development-replay-audit-summary.json`).
    pub saved_audit_checked_steps: usize,
    pub stratum_complete: BTreeMap<String, (usize, usize)>,
    pub block_pass_counts: Vec<(String, usize, usize)>,
    pub forward_pass: usize,
    pub reverse_pass: usize,
    pub chronology_abs_diff: usize,
    pub chronology_discordant_pairs: usize,
    pub swap0_pass: usize,
    pub swap1_pass: usize,
    pub swap_abs_diff: usize,
    pub swap_discordant_pairs: usize,
    pub wording_a_pass: usize,
    pub wording_b_pass: usize,
    pub wording_abs_diff: usize,
    pub wording_discordant_pairs: usize,
    pub job_pass: usize,
    pub home_pass: usize,
    pub role_abs_diff: usize,
    pub both_roles_complete_histories: usize,
    pub role_discordant_histories: usize,
    pub even_blocks_pass_of_256: usize,
    pub odd_blocks_pass_of_256: usize,
    pub half_panel_split_abs_diff: usize,
    /// Stratified 16-block (256-row) cluster-sampling SE in hundredths of a row (e.g. 624 = 6.24 rows).
    pub cluster_se_256_hundredths_rows: u64,
    /// Stratified 16-block (256-row) cluster-sampling SE in basis points of 256 (e.g. 244 = 2.44%).
    pub cluster_se_256_bps: u64,
    /// Independent binomial SE at the 40% (103/256) bar in hundredths of a row (783 = floor(7.8384 * 100)).
    pub bar40_binomial_se_hundredths_rows: u64,
    /// Cluster-inflated SE at the 40% (103/256) bar in hundredths of a row.
    pub bar40_cluster_se_hundredths_rows: u64,
    /// Cluster-inflated SE at the 40% (103/256) bar in basis points of 256.
    pub bar40_cluster_se_bps: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fresh256Bundle {
    pub plan_json: Vec<u8>,
    pub plan_sha256: String,
    pub inputs_json: Vec<u8>,
    pub inputs_sha256: String,
    pub labels_json: Vec<u8>,
    pub labels_sha256: String,
    pub manifest_json: Vec<u8>,
    pub manifest_sha256: String,
    pub histories: Vec<History>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlapAndSealAudit {
    pub rows: usize,
    pub histories: usize,
    pub unordered_blocks: usize,
    pub ordered_banks: usize,
    pub historical_literals_parsed_from_disk: usize,
    pub exact_literal_overlaps: usize,
    pub content_word_overlaps: usize,
    pub ngram_overlaps: usize,
    pub intra_panel_content_word_duplicates: usize,
    pub min_same_stratum_edit_distance: usize,
    pub literal_role_balance: bool,
    pub literal_initial_position_balance: bool,
    pub max_query_wording_spread: usize,
    pub max_turn_tokens: usize,
    pub max_target_tokens: usize,
    pub max_total_replay_tokens: usize,
    /// History-pair-verified hits (out of 256 rows) for always emitting the first-stated literal.
    pub copy_first_stated_hits: usize,
    /// History-pair-verified hits (out of 256 rows) for always emitting the last-stated literal.
    pub copy_last_stated_hits: usize,
    /// History-pair-verified hits (out of 256 rows) for emitting the opposite role's literal.
    pub role_swap_distractor_hits: usize,
    /// History-pair-verified hits (out of 256 rows) for emitting any single constant literal.
    pub best_constant_reply_hits: usize,
    pub unscored_inputs_leak_count: usize,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Exact integer floor square root (`floor(sqrt(n))`).
pub fn isqrt_u128(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// Circle schedule enumerating every unordered edge once (`geometric_role_diversity.rs:275`).
pub fn round_robin_edges(count: usize) -> Vec<(usize, usize)> {
    if count < 2 {
        return Vec::new();
    }
    let size = count + count % 2;
    let mut order: Vec<usize> = (0..size).collect();
    let mut result = Vec::new();
    for _ in 0..size - 1 {
        for index in 0..size / 2 {
            let left = order[index];
            let right = order[size - 1 - index];
            if left < count && right < count {
                let edge = if left < right {
                    (left, right)
                } else {
                    (right, left)
                };
                result.push(edge);
            }
        }
        let last = order.pop().expect("non-empty circle");
        order.insert(1, last);
    }
    result
}

pub fn supported_wire(
    relation: &str,
    act: &str,
    value: &str,
    query_index: usize,
) -> Result<Wire, String> {
    let (template, text) = match (relation, act) {
        ("job", "assert") => (
            "My current job is {v}.".to_string(),
            format!("My current job is {value}."),
        ),
        ("job", "update") => (
            "My current job has changed to {v}.".to_string(),
            format!("My current job has changed to {value}."),
        ),
        ("home", "assert") => (
            "I currently live in {v}.".to_string(),
            format!("I currently live in {value}."),
        ),
        ("home", "update") => (
            "I now live in {v}.".to_string(),
            format!("I now live in {value}."),
        ),
        ("job", "query") => {
            let q = CURRENT_JOB_QUERIES[query_index % CURRENT_JOB_QUERIES.len()].to_string();
            (q.clone(), q)
        }
        ("home", "query") => {
            let q = CURRENT_HOME_QUERIES[query_index % CURRENT_HOME_QUERIES.len()].to_string();
            (q.clone(), q)
        }
        _ => return Err(format!("unsupported role/act ({relation}, {act})")),
    };
    Ok(Wire {
        text,
        relation: relation.to_string(),
        act: act.to_string(),
        template: Some(template),
    })
}

pub fn extract_literal(w: &Wire) -> Result<&str, String> {
    let template = w.template.as_deref().ok_or("write template absent")?;
    let (prefix, suffix) = template.split_once("{v}").ok_or("write slot absent")?;
    w.text
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(suffix))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "write text/template mismatch".to_string())
}

/// Encode text into exact `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`)
/// token IDs using the GPT-2 `ByteLevel(add_prefix_space=false, use_regex=true)` pre-token table.
pub fn encode_pinned_bpe(text: &str) -> Result<Vec<u32>, String> {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut ids = Vec::new();
    while i < bytes.len() {
        let pretoken = if bytes[i] == b'.' || bytes[i] == b'?' {
            let pt = &text[i..i + 1];
            i += 1;
            pt
        } else if bytes[i] == b' ' {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            if i == start + 1 {
                return Err(format!("standalone space in BPE input: {text:?}"));
            }
            &text[start..i]
        } else if bytes[i].is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            &text[start..i]
        } else {
            return Err(format!(
                "unsupported byte {:?} in BPE input: {text:?}",
                bytes[i] as char
            ));
        };
        let idx = PRETOKEN_BPE_TABLE
            .binary_search_by_key(&pretoken, |(k, _)| *k)
            .map_err(|_| format!("pre-token {pretoken:?} missing from PRETOKEN_BPE_TABLE"))?;
        ids.extend_from_slice(PRETOKEN_BPE_TABLE[idx].1);
    }
    Ok(ids)
}

/// Reconstruct the 512-row `dev512` (`m2-reply512`) case structure and compute the exact
/// spread and cluster-sampling standard error from the sealed `saved437` receipts in `archive/`.
pub fn analyze_saved437_dev512_spread(repo_root: &Path) -> Result<Dev512SpreadReport, String> {
    let rank177_path =
        repo_root.join("archive/labs/m2-pooled-rank-2026-10-10/reviewer-rank177.json");
    let replay_path = repo_root
        .join("archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json");

    let rank177_bytes =
        std::fs::read(&rank177_path).map_err(|e| format!("{}: {e}", rank177_path.display()))?;
    let replay_bytes =
        std::fs::read(&replay_path).map_err(|e| format!("{}: {e}", replay_path.display()))?;

    let rank177: Value =
        serde_json::from_slice(&rank177_bytes).map_err(|e| format!("rank177 json: {e}"))?;
    let replay: Value =
        serde_json::from_slice(&replay_bytes).map_err(|e| format!("replay json: {e}"))?;

    if replay["field_sha256"].as_str() != Some(SAVED437_FIELD_SHA256)
        || replay["checked_rows"].as_u64() != Some(512)
        || replay["counts"]["complete"].as_u64() != Some(437)
    {
        return Err("replay summary SHA-256 bindings do not match saved437/dev512".to_string());
    }

    let complete_arr = rank177["endpoints"]["final"]["complete_ids"]
        .as_array()
        .ok_or("endpoints.final.complete_ids absent")?;
    let mut complete_set = BTreeSet::new();
    for item in complete_arr {
        let id = item.as_str().ok_or("complete_id not string")?;
        complete_set.insert(id.to_string());
    }
    if complete_set.len() != 437 {
        return Err(format!(
            "expected 437 complete_ids, got {}",
            complete_set.len()
        ));
    }

    let strata_specs: [(&str, usize); 5] = [
        ("length2", 8),
        ("length4", 8),
        ("length8", 8),
        ("update", 4),
        ("reassert", 4),
    ];

    let mut stratum_complete = BTreeMap::new();
    let mut block_pass_counts = Vec::new();
    let mut forward_pass = 0usize;
    let mut reverse_pass = 0usize;
    let mut chronology_discordant_pairs = 0usize;
    let mut swap0_pass = 0usize;
    let mut swap1_pass = 0usize;
    let mut swap_discordant_pairs = 0usize;
    let mut wording_a_pass = 0usize;
    let mut wording_b_pass = 0usize;
    let mut wording_discordant_pairs = 0usize;
    let mut job_pass = 0usize;
    let mut home_pass = 0usize;
    let mut both_roles_complete_histories = 0usize;
    let mut role_discordant_histories = 0usize;
    let mut even_blocks_pass_of_256 = 0usize;
    let mut odd_blocks_pass_of_256 = 0usize;

    let mut global_block = 0usize;
    let mut all_ids = BTreeSet::new();

    for (stratum, num_blocks) in strata_specs {
        let mut s_pass = 0usize;
        let mut s_total = 0usize;
        for selected in 0..num_blocks {
            let q_a = global_block % 6;
            let q_b = (global_block + 3) % 6;
            let mut b_pass = 0usize;

            for swapped in [0usize, 1usize] {
                for (q_idx, q) in [q_a, q_b].into_iter().enumerate() {
                    for role in ["job", "home"] {
                        let fwd_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q}-forward-{role}"
                        );
                        let rev_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q}-reverse-{role}"
                        );
                        all_ids.insert(fwd_id.clone());
                        all_ids.insert(rev_id.clone());
                        let f_ok = complete_set.contains(&fwd_id);
                        let r_ok = complete_set.contains(&rev_id);
                        if f_ok {
                            forward_pass += 1;
                            s_pass += 1;
                            b_pass += 1;
                        }
                        if r_ok {
                            reverse_pass += 1;
                            s_pass += 1;
                            b_pass += 1;
                        }
                        s_total += 2;
                        if f_ok != r_ok {
                            chronology_discordant_pairs += 1;
                        }
                        if q_idx == 0 {
                            wording_a_pass += usize::from(f_ok) + usize::from(r_ok);
                        } else {
                            wording_b_pass += usize::from(f_ok) + usize::from(r_ok);
                        }
                    }
                    for dir in ["forward", "reverse"] {
                        let job_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q}-{dir}-job"
                        );
                        let home_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q}-{dir}-home"
                        );
                        let j_ok = complete_set.contains(&job_id);
                        let h_ok = complete_set.contains(&home_id);
                        if j_ok {
                            job_pass += 1;
                        }
                        if h_ok {
                            home_pass += 1;
                        }
                        if j_ok && h_ok {
                            both_roles_complete_histories += 1;
                        } else if j_ok != h_ok {
                            role_discordant_histories += 1;
                        }
                    }
                }
                for dir in ["forward", "reverse"] {
                    for role in ["job", "home"] {
                        let qa_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_a}-{dir}-{role}"
                        );
                        let qb_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap{swapped}-q{q_b}-{dir}-{role}"
                        );
                        if complete_set.contains(&qa_id) != complete_set.contains(&qb_id) {
                            wording_discordant_pairs += 1;
                        }
                    }
                }
            }

            for q in [q_a, q_b] {
                for dir in ["forward", "reverse"] {
                    for role in ["job", "home"] {
                        let s0_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap0-q{q}-{dir}-{role}"
                        );
                        let s1_id = format!(
                            "development-diverse-{stratum}-{selected:02}-swap1-q{q}-{dir}-{role}"
                        );
                        let s0_ok = complete_set.contains(&s0_id);
                        let s1_ok = complete_set.contains(&s1_id);
                        if s0_ok {
                            swap0_pass += 1;
                        }
                        if s1_ok {
                            swap1_pass += 1;
                        }
                        if s0_ok != s1_ok {
                            swap_discordant_pairs += 1;
                        }
                    }
                }
            }

            if selected % 2 == 0 {
                even_blocks_pass_of_256 += b_pass;
            } else {
                odd_blocks_pass_of_256 += b_pass;
            }
            block_pass_counts.push((stratum.to_string(), selected, b_pass));
            global_block += 1;
        }
        let expected_stratum = replay["strata"][stratum]["complete"]
            .as_u64()
            .ok_or_else(|| format!("replay stratum {stratum} absent"))?
            as usize;
        if s_pass != expected_stratum {
            return Err(format!(
                "stratum {stratum} count mismatch: {s_pass} vs {expected_stratum}"
            ));
        }
        stratum_complete.insert(stratum.to_string(), (s_pass, s_total));
    }

    if all_ids.len() != 512 || !complete_set.is_subset(&all_ids) {
        return Err("complete_ids is not a subset of the 512 canonical dev512 row IDs".to_string());
    }

    // Compute exact stratified 16-block (256-row) cluster-sampling variance:
    // Var(S_256) = sum_h b'_h * s_h^2, where s_h^2 = sum_i (c_{h,i} - mean_h)^2 / (B_h - 1).
    // Common denominator for Var(S_256) is 336 (112 * 3 = 336, 24 * 14 = 336).
    let mut var_num_336: u128 = 0;
    for (stratum, num_blocks) in strata_specs {
        let counts: Vec<i128> = block_pass_counts
            .iter()
            .filter(|(s, _, _)| s == stratum)
            .map(|(_, _, c)| *c as i128)
            .collect();
        let sum_c: i128 = counts.iter().sum();
        let b_h = num_blocks as i128;
        let mut sq_sum: u128 = 0;
        for c in counts {
            let diff = b_h * c - sum_c;
            sq_sum += (diff * diff) as u128;
        }
        if num_blocks == 8 {
            var_num_336 += sq_sum * 3;
        } else {
            var_num_336 += sq_sum * 14;
        }
    }
    let cluster_se_256_hundredths_rows = isqrt_u128((var_num_336 * 10_000 + 168) / 336) as u64;
    let cluster_se_256_bps = (cluster_se_256_hundredths_rows * 100 + 128) / 256;

    // Independent binomial variance at the 40% (p = 0.40) bar on 256 rows:
    // Var_ind_bar40 = 256 * 0.40 * 0.60 = 61.44 = 1536 / 25.
    // isqrt_u128(614_400) = 783 (floor of 7.8384 * 100, i.e. 7.83 rows floor / 7.84 rows rounded).
    let bar40_binomial_se_hundredths_rows = isqrt_u128(614_400) as u64;
    let var_cluster_bar40_times_10000 =
        (1536u128 * 10_000 * var_num_336 * 1024) / (25u128 * 336 * 32_775);
    let bar40_cluster_se_hundredths_rows =
        isqrt_u128(var_cluster_bar40_times_10000.max(614_400)) as u64;
    let bar40_cluster_se_bps = (bar40_cluster_se_hundredths_rows * 100 + 128) / 256;

    let saved_audit_status = replay["status"]
        .as_str()
        .ok_or("replay status absent")?
        .to_string();
    let saved_audit_checked_steps = replay["checked_steps"]
        .as_u64()
        .ok_or("replay checked_steps absent")? as usize;

    Ok(Dev512SpreadReport {
        total_rows: all_ids.len(),
        complete_rows: complete_set.len(),
        saved_audit_status,
        saved_audit_checked_steps,
        stratum_complete,
        block_pass_counts,
        forward_pass,
        reverse_pass,
        chronology_abs_diff: forward_pass.abs_diff(reverse_pass),
        chronology_discordant_pairs,
        swap0_pass,
        swap1_pass,
        swap_abs_diff: swap0_pass.abs_diff(swap1_pass),
        swap_discordant_pairs,
        wording_a_pass,
        wording_b_pass,
        wording_abs_diff: wording_a_pass.abs_diff(wording_b_pass),
        wording_discordant_pairs,
        job_pass,
        home_pass,
        role_abs_diff: job_pass.abs_diff(home_pass),
        both_roles_complete_histories,
        role_discordant_histories,
        even_blocks_pass_of_256,
        odd_blocks_pass_of_256,
        half_panel_split_abs_diff: even_blocks_pass_of_256.abs_diff(odd_blocks_pass_of_256),
        cluster_se_256_hundredths_rows,
        cluster_se_256_bps,
        bar40_binomial_se_hundredths_rows,
        bar40_cluster_se_hundredths_rows,
        bar40_cluster_se_bps,
    })
}

/// Generate the frozen 256-row held-out reply panel (`m2-fresh256`) and its sealed oracle labels.
pub fn generate_m2_fresh256_panel() -> Result<Fresh256Bundle, String> {
    let mut histories = Vec::new();
    let mut used_unordered: BTreeSet<(String, String)> = BTreeSet::new();

    let strata_specs: [(&str, &[&str], usize); 5] = [
        ("length2", &FRESH256_LEN2, 4),
        ("length4", &FRESH256_LEN4, 4),
        ("length8", &FRESH256_LEN8, 4),
        ("update", &FRESH256_LEN4, 2),
        ("reassert", &FRESH256_LEN2[..8], 2),
    ];

    for (stratum, values, target_blocks) in strata_specs {
        let mut selected = 0usize;
        for (left, right) in round_robin_edges(values.len()) {
            if selected == target_blocks {
                break;
            }
            let a = values[left];
            let b = values[right];
            let key = if a < b {
                (a.to_string(), b.to_string())
            } else {
                (b.to_string(), a.to_string())
            };
            if used_unordered.contains(&key) {
                continue;
            }
            let block = histories.len() / 8;
            for swapped in [false, true] {
                let (jv, hv) = if swapped { (b, a) } else { (a, b) };
                let mut j = supported_wire("job", "assert", jv, 0)?;
                let mut h = supported_wire("home", "assert", hv, 0)?;
                let mut after = Vec::new();
                if stratum == "update" {
                    let role = if swapped { "home" } else { "job" };
                    let init_val = FRESH256_UPDATE_INIT[selected % 4];
                    let initial = supported_wire(role, "assert", init_val, 0)?;
                    if swapped {
                        h = initial;
                    } else {
                        j = initial;
                    }
                    after.push(supported_wire(role, "update", a, 0)?);
                    after.push(if swapped { j.clone() } else { h.clone() });
                } else if stratum == "reassert" {
                    after.push(if swapped { h.clone() } else { j.clone() });
                }
                for query_index in [block % 6, (block + 3) % 6] {
                    let queries = vec![
                        supported_wire("job", "query", "", query_index)?,
                        supported_wire("home", "query", "", query_index)?,
                    ];
                    let base_id = format!(
                        "diverse-{stratum}-{selected:02}-swap{}-q{query_index}",
                        usize::from(swapped)
                    );
                    let stratum_tag = format!("{stratum}/q{query_index}");
                    let mut forward_turns = vec![j.clone(), h.clone()];
                    forward_turns.extend(after.clone());
                    let mut reverse_turns = vec![h.clone(), j.clone()];
                    reverse_turns.extend(after.iter().rev().cloned());
                    let mut variants = vec![
                        History {
                            id: format!("{base_id}-forward"),
                            stratum: stratum_tag.clone(),
                            turns: forward_turns,
                            queries: queries.clone(),
                        },
                        History {
                            id: format!("{base_id}-reverse"),
                            stratum: stratum_tag,
                            turns: reverse_turns,
                            queries,
                        },
                    ];
                    if stratum == "reassert" {
                        for h_var in &mut variants {
                            let repeated = h_var
                                .turns
                                .first()
                                .cloned()
                                .ok_or_else(|| "reassert initial absent".to_string())?;
                            *h_var
                                .turns
                                .last_mut()
                                .ok_or_else(|| "reassert last absent".to_string())? = repeated;
                        }
                    }
                    histories.extend(variants);
                }
            }
            used_unordered.insert(key);
            selected += 1;
        }
        if selected != target_blocks {
            return Err(format!("insufficient disjoint blocks for {stratum}"));
        }
    }

    if histories.len() != 128 {
        return Err(format!("expected 128 histories, got {}", histories.len()));
    }

    let plan_val = json!({
        "schema": "uor-r4.raw-natural-reader-construction-plan/1",
        "split": "fresh",
        "source_policy": "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2",
        "histories": histories,
    });
    let plan_json = serde_json::to_vec_pretty(&plan_val).map_err(|e| e.to_string())?;
    let plan_sha256 = sha256_hex(&plan_json);

    let mut runtime_cases = Vec::new();
    let mut label_cases = Vec::new();

    for h in &histories {
        let mut current_by_role: BTreeMap<&str, (usize, &Wire, String)> = BTreeMap::new();
        for (event, w) in h.turns.iter().enumerate() {
            let lit = extract_literal(w)?.to_string();
            current_by_role.insert(w.relation.as_str(), (event, w, lit));
        }
        let mut records: Vec<(usize, u32, &Wire, String)> = current_by_role
            .into_iter()
            .map(|(role, (event, w, lit))| {
                let rel_id = if role == "job" { 1u32 } else { 2u32 };
                (event, rel_id, w, lit)
            })
            .collect();
        records.sort_by_key(|r| r.0);
        if records.len() != 2 {
            return Err("history lacks both current records".to_string());
        }

        let mut segments = Vec::new();
        for (event, rel_id, w, lit) in &records {
            let cue_ids = encode_pinned_bpe(&w.text)?;
            let val_ids = encode_pinned_bpe(lit)?;
            // In StackStore::write_from (`geometric-native-bank-observation-panel.rs:948`),
            // each write event `0, 1, ...` assigns `record = (event + 1) as u64` and `commit = (event + 1) as u64`.
            let record_and_commit = (*event as u64) + 1;
            segments.push(json!({
                "kind": "Context",
                "event": event,
                "role": 1,
                "token_ids": cue_ids,
            }));
            segments.push(json!({
                "kind": "Source",
                "event": event,
                "record": record_and_commit,
                "commit": record_and_commit,
                "scope": "compiler-probe",
                "entity": [1],
                "relation": rel_id,
                "view": 0,
                "original_source_ids": val_ids,
            }));
        }

        let pair_id = format!("fresh-{}", h.id);
        let mut sorted_queries: Vec<&Wire> = h.queries.iter().collect();
        sorted_queries.sort_by_key(|q| if q.relation == "job" { 0usize } else { 1usize });
        for q in sorted_queries {
            let case_id = format!("fresh-{}-{}", h.id, q.relation);
            let query_ids = encode_pinned_bpe(&q.text)?;
            runtime_cases.push(json!({
                "id": case_id,
                "segments": segments,
                "query_ids": query_ids,
                "actual_prefix_ids": [],
            }));
            let expected_rel = if q.relation == "job" { 1u32 } else { 2u32 };
            let (_, _, _, expected_lit) = records
                .iter()
                .find(|(_, rel, _, _)| *rel == expected_rel)
                .ok_or_else(|| "expected role record absent".to_string())?;
            label_cases.push(json!({
                "id": case_id,
                "pair_id": pair_id,
                "answers": {
                    "intent": "current",
                    "accepted": [format!("{expected_lit}.")],
                }
            }));
        }
    }

    let inputs_val = json!({
        "schema": "uor-r4.native-source-bank-probe-input/1",
        "cases": runtime_cases,
    });
    let inputs_json = serde_json::to_vec_pretty(&inputs_val).map_err(|e| e.to_string())?;
    let inputs_sha256 = sha256_hex(&inputs_json);

    let labels_val = json!({
        "schema": "uor-r4.native-source-bank-labels/1",
        "protocol": "uor-r4.literal-role-dialogue/2",
        "membership_only": true,
        "cases": label_cases,
    });
    let labels_json = serde_json::to_vec_pretty(&labels_val).map_err(|e| e.to_string())?;
    let labels_sha256 = sha256_hex(&labels_json);

    let manifest_val = json!({
        "schema": "uor-r4.m2-fresh256-panel-manifest/1",
        "panel_id": "m2-fresh256",
        "milestone": "m2",
        "issue": "#2849",
        "milestone_issue": "#2030",
        "rows": 256,
        "histories": 128,
        "unordered_bank_blocks": 16,
        "ordered_role_value_banks": 32,
        "opposite_query_pairs": 128,
        "strata_rows": {
            "length2": 64,
            "length4": 64,
            "length8": 64,
            "update": 32,
            "reassert": 32
        },
        "target_bar_metric": "pairs_both_complete_distinct",
        "target_bar_pairs": 52,
        "target_bar_pairs_of": 128,
        "target_bar_rows": 104,
        "target_bar_of": 256,
        "tokenizer_sha256": PINNED_TOKENIZER_SHA256,
        "files": {
            "plan.json": {
                "bytes": plan_json.len(),
                "sha256": plan_sha256,
            },
            "inputs.json": {
                "bytes": inputs_json.len(),
                "sha256": inputs_sha256,
            },
            "labels.json": {
                "bytes": labels_json.len(),
                "sha256": labels_sha256,
            }
        },
        "excluded_historical_exposures": {
            "dev512_inputs_sha256": DEV512_INPUTS_SHA256,
            "opened_fresh128_inputs_sha256": OPENED_FRESH128_INPUTS_SHA256,
            "historical_exposed_literals": HISTORICAL_EXPOSED_LITERALS.len()
        },
        "model_predictions": "NOT_RUN"
    });
    let manifest_json = serde_json::to_vec_pretty(&manifest_val).map_err(|e| e.to_string())?;
    let manifest_sha256 = sha256_hex(&manifest_json);

    Ok(Fresh256Bundle {
        plan_json,
        plan_sha256,
        inputs_json,
        inputs_sha256,
        labels_json,
        labels_sha256,
        manifest_json,
        manifest_sha256,
        histories,
    })
}

fn is_function_word(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "a"
            | "an"
            | "above"
            | "beside"
            | "beyond"
            | "near"
            | "over"
            | "under"
            | "across"
            | "along"
            | "around"
            | "within"
            | "inside"
            | "outside"
            | "below"
            | "by"
            | "stands"
            | "grows"
            | "rests"
            | "rises"
    )
}

fn content_words(s: &str) -> BTreeSet<String> {
    s.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty() && !is_function_word(w))
        .collect()
}

fn word_ngrams(s: &str, n: usize) -> BTreeSet<Vec<String>> {
    let words: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < n {
        return BTreeSet::new();
    }
    words.windows(n).map(|w| w.to_vec()).collect()
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=bc.len()).collect();
    let mut curr = vec![0usize; bc.len() + 1];
    for (i, &ca) in ac.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in bc.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[bc.len()]
}

fn parse_quoted_strings_in_block(
    source: &str,
    header: &str,
    end_marker: &str,
) -> Result<Vec<String>, String> {
    let after_header = source
        .split_once(header)
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("header {header:?} not found"))?;
    let block = after_header
        .split_once(end_marker)
        .map(|(b, _)| b)
        .ok_or_else(|| format!("end marker {end_marker:?} after {header:?} not found"))?;
    let mut out = Vec::new();
    let mut rest = block;
    while let Some((_, after_open)) = rest.split_once('"') {
        let (lit, after_close) = after_open
            .split_once('"')
            .ok_or_else(|| format!("unclosed quote in {header:?}"))?;
        out.push(lit.to_string());
        rest = after_close;
    }
    Ok(out)
}

/// Read and parse the actual historical curriculum and observation-plan files from `repo_root`
/// (`crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs`,
/// `crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs`, and
/// `archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json`)
/// to extract the 74 historical exposed literals directly from disk.
pub fn load_historical_exposed_literals_from_repo(repo_root: &Path) -> Result<Vec<String>, String> {
    let curriculum_path =
        repo_root.join("crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs");
    let curriculum_src = std::fs::read_to_string(&curriculum_path)
        .map_err(|e| format!("{}: {e}", curriculum_path.display()))?;

    let mut extracted = Vec::new();
    for (header, expected_len) in [
        ("const VALUES: [&str; 32] = [", 32usize),
        ("const FACTOR_VALUES: [&str; 8] = [", 8),
        ("const FRESH_VALUES: [&str; 8] = [", 8),
        ("const FRESH2_VALUES: [&str; 8] = [", 8),
        ("const FRESH3_VALUES: [&str; 8] = [", 8),
    ] {
        let items = parse_quoted_strings_in_block(&curriculum_src, header, "];")?;
        if items.len() != expected_len {
            return Err(format!(
                "{header} length mismatch in curriculum.rs: expected {expected_len}, got {}",
                items.len()
            ));
        }
        extracted.extend(items);
    }

    let plan_path =
        repo_root.join("crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs");
    let plan_src =
        std::fs::read_to_string(&plan_path).map_err(|e| format!("{}: {e}", plan_path.display()))?;
    let raw_fixed = parse_quoted_strings_in_block(&plan_src, "let fixed = [", "];")?;
    let untouched: Vec<String> = raw_fixed
        .into_iter()
        .filter(|s| s != "job" && s != "home")
        .collect();
    if untouched.len() != 10 {
        return Err(format!(
            "expected 10 untouched transfer literals in geometric-native-bank-observation-plan.rs, got {}",
            untouched.len()
        ));
    }
    extracted.extend(untouched);

    let audit_path = repo_root
        .join("archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json");
    let audit_bytes =
        std::fs::read(&audit_path).map_err(|e| format!("{}: {e}", audit_path.display()))?;
    let audit_val: Value =
        serde_json::from_slice(&audit_bytes).map_err(|e| format!("audit json: {e}"))?;
    if audit_val["input_bytes_sha256"].as_str() != Some(OPENED_FRESH128_INPUTS_SHA256)
        || audit_val["tokenizer_sha256"].as_str() != Some(PINNED_TOKENIZER_SHA256)
    {
        return Err("fresh-preparation-audit-attempt1.json SHA-256 bindings differ".to_string());
    }
    let extracted_set: BTreeSet<&str> = extracted.iter().map(String::as_str).collect();
    if let Some(banks) = audit_val["frozen_episode_banks_in_literal_support"].as_array() {
        for bank in banks {
            if let Some(pairs) = bank.as_array() {
                for pair in pairs {
                    if let Some(lit) = pair.get(1).and_then(Value::as_str) {
                        if !extracted_set.contains(lit) {
                            return Err(format!(
                                "frozen_episode_banks_in_literal_support literal {lit:?} missing from extracted set"
                            ));
                        }
                    }
                }
            }
        }
    }

    if extracted.len() != HISTORICAL_EXPOSED_LITERALS.len()
        || extracted
            .iter()
            .zip(HISTORICAL_EXPOSED_LITERALS.iter())
            .any(|(a, b)| a != b)
    {
        return Err(
            "on-disk historical literals differ from HISTORICAL_EXPOSED_LITERALS".to_string(),
        );
    }
    Ok(extracted)
}

type BankBlockMap =
    BTreeMap<(String, String), BTreeMap<(String, String), Vec<(String, Vec<String>)>>>;

/// Verify structural balance, real-file overlap exclusion against all 74 historical literals on disk,
/// `ByteBpeTokenizer` (`d36d3e87…`) token and alphabet budgets, `#[serde(deny_unknown_fields)]`
/// consumer schema compatibility, history-pair question-blind controls (`< 103`), and unscored-input sealing.
pub fn verify_m2_fresh256_panel(
    repo_root: &Path,
    bundle: &Fresh256Bundle,
) -> Result<OverlapAndSealAudit, String> {
    if sha256_hex(&bundle.plan_json) != bundle.plan_sha256
        || sha256_hex(&bundle.inputs_json) != bundle.inputs_sha256
        || sha256_hex(&bundle.labels_json) != bundle.labels_sha256
        || sha256_hex(&bundle.manifest_json) != bundle.manifest_sha256
    {
        return Err("bundle SHA-256 mismatch".to_string());
    }

    if bundle.histories.len() != 128 {
        return Err("histories count != 128".to_string());
    }

    let historical_from_disk = load_historical_exposed_literals_from_repo(repo_root)?;

    // Check consumer schema compatibility under #[serde(deny_unknown_fields)].
    let parsed_inputs: ProbeInputs = serde_json::from_slice(&bundle.inputs_json)
        .map_err(|e| format!("inputs.json failed deny_unknown_fields schema: {e}"))?;
    let parsed_labels: ProbeLabels = serde_json::from_slice(&bundle.labels_json)
        .map_err(|e| format!("labels.json failed deny_unknown_fields schema: {e}"))?;
    if parsed_inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || parsed_labels.schema != "uor-r4.native-source-bank-labels/1"
        || parsed_labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !parsed_labels.membership_only
        || parsed_inputs.cases.len() != 256
        || parsed_labels.cases.len() != 256
    {
        return Err("inputs.json / labels.json consumer header or case count mismatch".to_string());
    }

    let mut banks: BankBlockMap = BTreeMap::new();
    let mut role_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut position_counts: BTreeMap<(String, usize), usize> = BTreeMap::new();
    let mut stratum_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut query_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut fresh_literals_used: BTreeSet<String> = BTreeSet::new();

    for h in &bundle.histories {
        let stratum = h
            .stratum
            .split("/q")
            .next()
            .ok_or_else(|| "stratum absent".to_string())?;
        *stratum_counts.entry(stratum.to_string()).or_default() += 1;
        if h.turns.len() < 2 || h.turns.len() > 8 || h.queries.len() != 2 {
            return Err("invalid turn or query count".to_string());
        }
        let mut current = BTreeMap::new();
        let mut chronology = Vec::new();
        for (pos, w) in h.turns.iter().enumerate() {
            let val = extract_literal(w)?.to_string();
            fresh_literals_used.insert(val.clone());
            current.insert(w.relation.clone(), val.clone());
            *role_counts
                .entry((val.clone(), w.relation.clone()))
                .or_default() += 1;
            if pos < 2 {
                *position_counts.entry((val.clone(), pos)).or_default() += 1;
            }
            chronology.push(format!("{}/{}/{val}", w.relation, w.act));
        }
        let j = current
            .get("job")
            .ok_or_else(|| "job absent".to_string())?
            .clone();
        let home = current
            .get("home")
            .ok_or_else(|| "home absent".to_string())?
            .clone();
        if j == home {
            return Err("equal role values".to_string());
        }
        let unordered = if j < home {
            (j.clone(), home.clone())
        } else {
            (home.clone(), j.clone())
        };
        let mut wording = BTreeMap::new();
        for q in &h.queries {
            wording.insert(q.relation.clone(), q.text.clone());
            *query_counts
                .entry((q.relation.clone(), q.text.clone()))
                .or_default() += 1;
        }
        let wordkey = serde_json::to_string(&wording).map_err(|e| e.to_string())?;
        banks
            .entry(unordered)
            .or_default()
            .entry((j, home))
            .or_default()
            .push((wordkey, chronology));
    }

    let expected_strata = BTreeMap::from([
        ("length2".to_string(), 32usize),
        ("length4".to_string(), 32),
        ("length8".to_string(), 32),
        ("update".to_string(), 16),
        ("reassert".to_string(), 16),
    ]);
    if stratum_counts != expected_strata {
        return Err(format!("stratum quota mismatch: {stratum_counts:?}"));
    }

    for variants in banks.values() {
        if variants.len() != 2 || variants.values().any(|v| v.len() != 4) {
            return Err("bank block lacks both role assignments and 4 variants each".to_string());
        }
    }

    let mut literal_role_balance = true;
    for ((val, role), count) in &role_counts {
        let other = if role == "job" { "home" } else { "job" };
        if role_counts.get(&(val.clone(), other.to_string())) != Some(count) {
            literal_role_balance = false;
        }
    }
    if !literal_role_balance {
        return Err("literal role imbalance".to_string());
    }

    let mut literal_initial_position_balance = true;
    for ((val, pos), count) in &position_counts {
        if position_counts.get(&(val.clone(), 1 - pos)) != Some(count) {
            literal_initial_position_balance = false;
        }
    }
    if !literal_initial_position_balance {
        return Err("literal initial position imbalance".to_string());
    }

    let mut max_query_wording_spread = 0usize;
    for role in ["job", "home"] {
        let counts: Vec<usize> = query_counts
            .iter()
            .filter(|((r, _), _)| r == role)
            .map(|(_, c)| *c)
            .collect();
        if counts.len() != 6 {
            return Err("missing query wording form".to_string());
        }
        let spread =
            counts.iter().max().copied().unwrap_or(0) - counts.iter().min().copied().unwrap_or(0);
        max_query_wording_spread = max_query_wording_spread.max(spread);
        if spread > 4 {
            return Err("query wording forms unbalanced".to_string());
        }
    }

    // Check exact, content-word, n-gram, and edit-distance disjointness against all 74 historical
    // literals parsed from disk, plus pairwise content-word disjointness across all 34 fresh literals.
    let hist_set: BTreeSet<&str> = historical_from_disk.iter().map(String::as_str).collect();
    let mut exact_literal_overlaps = 0usize;
    let mut content_word_overlaps = 0usize;
    let mut ngram_overlaps = 0usize;
    let mut intra_panel_content_word_duplicates = 0usize;
    let mut seen_fresh_content_words: BTreeSet<String> = BTreeSet::new();
    let mut min_same_stratum_edit_distance = usize::MAX;

    for fresh_lit in &fresh_literals_used {
        if hist_set.contains(fresh_lit.as_str()) {
            exact_literal_overlaps += 1;
        }
        let f_words = content_words(fresh_lit);
        for w in &f_words {
            if !seen_fresh_content_words.insert(w.clone()) {
                intra_panel_content_word_duplicates += 1;
            }
        }
        let f_bi = word_ngrams(fresh_lit, 2);
        let f_len = fresh_lit.split_whitespace().count();
        for hist_lit in &historical_from_disk {
            let h_words = content_words(hist_lit);
            if !f_words.is_disjoint(&h_words) {
                content_word_overlaps += 1;
            }
            let h_bi = word_ngrams(hist_lit, 2);
            if !f_bi.is_disjoint(&h_bi) {
                ngram_overlaps += 1;
            }
            if hist_lit.split_whitespace().count() == f_len {
                let d = levenshtein_distance(fresh_lit, hist_lit);
                min_same_stratum_edit_distance = min_same_stratum_edit_distance.min(d);
            }
        }
    }

    if exact_literal_overlaps != 0
        || content_word_overlaps != 0
        || ngram_overlaps != 0
        || intra_panel_content_word_duplicates != 0
        || min_same_stratum_edit_distance < 8
    {
        return Err(format!(
            "overlap exclusion failed: exact={exact_literal_overlaps} word={content_word_overlaps} ngram={ngram_overlaps} intra_dup={intra_panel_content_word_duplicates} min_edit={min_same_stratum_edit_distance}"
        ));
    }

    let inputs_str = String::from_utf8_lossy(&bundle.inputs_json);
    let unscored_inputs_leak_count = usize::from(inputs_str.contains("\"accepted\""));
    if unscored_inputs_leak_count != 0 {
        return Err("inputs.json leaked accepted answer strings".to_string());
    }

    let tok = load_pinned_byte_bpe_tokenizer(repo_root)?;
    for &(pretoken, expected_ids) in PRETOKEN_BPE_TABLE {
        let actual_ids = tok.encode(pretoken);
        if actual_ids.as_slice() != expected_ids {
            return Err(format!(
                "PRETOKEN_BPE_TABLE mismatch for {pretoken:?}: table={expected_ids:?} vs ByteBpeTokenizer={actual_ids:?}"
            ));
        }
    }

    let mut max_turn_tokens = 0usize;
    let mut max_target_tokens = 0usize;
    let mut max_total_replay_tokens = 0usize;
    let mut copy_first_generated = Vec::with_capacity(256);
    let mut copy_last_generated = Vec::with_capacity(256);
    let mut role_swap_generated = Vec::with_capacity(256);

    for h in &bundle.histories {
        let first_lit = extract_literal(&h.turns[0])?.to_string();
        let last_lit = extract_literal(h.turns.last().ok_or("empty turns")?)?.to_string();
        let mut current: BTreeMap<&str, (&Wire, String)> = BTreeMap::new();
        for w in &h.turns {
            let encoded = encode_pinned_bpe(&w.text)?;
            if encoded != tok.encode(&w.text) {
                return Err(format!("turn BPE mismatch on {:?}", w.text));
            }
            max_turn_tokens = max_turn_tokens.max(encoded.len());
            current.insert(w.relation.as_str(), (w, extract_literal(w)?.to_string()));
        }
        let mut replay_tokens = 0usize;
        let mut alphabet = BTreeSet::from([PERIOD_TOKEN_ID, EOS_TOKEN_ID]);
        for (w, lit) in current.values() {
            let cue_ids = encode_pinned_bpe(&w.text)?;
            let spaced_lit = format!(" {lit}");
            let emitted_ids = encode_pinned_bpe(&spaced_lit)?;
            if emitted_ids != tok.encode(&spaced_lit) {
                return Err(format!("literal BPE mismatch on {spaced_lit:?}"));
            }
            replay_tokens += cue_ids.len() + emitted_ids.len();
            alphabet.extend(emitted_ids);
        }

        let mut sorted_queries: Vec<&Wire> = h.queries.iter().collect();
        sorted_queries.sort_by_key(|q| if q.relation == "job" { 0usize } else { 1usize });
        for q in sorted_queries {
            let q_ids = encode_pinned_bpe(&q.text)?;
            if q_ids != tok.encode(&q.text) {
                return Err(format!("query BPE mismatch on {:?}", q.text));
            }
            max_turn_tokens = max_turn_tokens.max(q_ids.len());
            let (_, expected_lit) = current
                .get(q.relation.as_str())
                .ok_or_else(|| "missing role".to_string())?;
            let other_role = if q.relation == "job" { "home" } else { "job" };
            let (_, distractor_lit) = current
                .get(other_role)
                .ok_or_else(|| "missing other role".to_string())?;
            let target_str = format!(" {expected_lit}.");
            let mut target_ids = encode_pinned_bpe(&target_str)?;
            if target_ids != tok.encode(&target_str) {
                return Err(format!("target BPE mismatch on {target_str:?}"));
            }
            target_ids.push(EOS_TOKEN_ID);
            if target_ids.iter().any(|t| !alphabet.contains(t)) {
                return Err(format!("target {expected_lit:?} outside replay alphabet"));
            }
            let target_len = target_ids.len();
            max_target_tokens = max_target_tokens.max(target_len);
            max_total_replay_tokens =
                max_total_replay_tokens.max(replay_tokens + q_ids.len() + target_len);

            let mut first_ids = tok.encode(&format!("{first_lit}."));
            first_ids.push(EOS_TOKEN_ID);
            copy_first_generated.push(first_ids);

            let mut last_ids = tok.encode(&format!("{last_lit}."));
            last_ids.push(EOS_TOKEN_ID);
            copy_last_generated.push(last_ids);

            let mut swap_ids = tok.encode(&format!("{distractor_lit}."));
            swap_ids.push(EOS_TOKEN_ID);
            role_swap_generated.push(swap_ids);
        }
    }

    if max_turn_tokens > 128 || max_target_tokens > 32 || max_total_replay_tokens > 128 {
        return Err("token budget exceeded".to_string());
    }

    // Score each control through the real `ByteBpeTokenizer` + `FrozenAnswers` + `pair_id`
    // grading loop (`crates/uor-r4-core/examples/native-bank-generalization.rs:827-948`).
    let (_, copy_first_pairs) = grade_generated_replies_with_evaluator_oracle(
        &tok,
        &parsed_inputs,
        &parsed_labels,
        &copy_first_generated,
    )?;
    let (_, copy_last_pairs) = grade_generated_replies_with_evaluator_oracle(
        &tok,
        &parsed_inputs,
        &parsed_labels,
        &copy_last_generated,
    )?;
    let (_, role_swap_pairs) = grade_generated_replies_with_evaluator_oracle(
        &tok,
        &parsed_inputs,
        &parsed_labels,
        &role_swap_generated,
    )?;
    let mut best_constant_pairs = 0usize;
    for candidate_const in &fresh_literals_used {
        let mut const_ids = tok.encode(&format!("{candidate_const}."));
        const_ids.push(EOS_TOKEN_ID);
        let const_generated = vec![const_ids; 256];
        let (_, pairs_ok) = grade_generated_replies_with_evaluator_oracle(
            &tok,
            &parsed_inputs,
            &parsed_labels,
            &const_generated,
        )?;
        best_constant_pairs = best_constant_pairs.max(pairs_ok);
    }

    let copy_first_stated_hits = copy_first_pairs * 2;
    let copy_last_stated_hits = copy_last_pairs * 2;
    let role_swap_distractor_hits = role_swap_pairs * 2;
    let best_constant_reply_hits = best_constant_pairs * 2;
    if copy_first_pairs != 0
        || copy_last_pairs != 0
        || role_swap_pairs != 0
        || best_constant_pairs != 0
    {
        return Err(format!(
            "control check failed (expected 0/128 pairs < 52/128 bar): first={copy_first_pairs} last={copy_last_pairs} swap={role_swap_pairs} const={best_constant_pairs}"
        ));
    }

    Ok(OverlapAndSealAudit {
        rows: 256,
        histories: 128,
        unordered_blocks: banks.len(),
        ordered_banks: banks.len() * 2,
        historical_literals_parsed_from_disk: historical_from_disk.len(),
        exact_literal_overlaps,
        content_word_overlaps,
        ngram_overlaps,
        intra_panel_content_word_duplicates,
        min_same_stratum_edit_distance,
        literal_role_balance,
        literal_initial_position_balance,
        max_query_wording_spread,
        max_turn_tokens,
        max_target_tokens,
        max_total_replay_tokens,
        copy_first_stated_hits,
        copy_last_stated_hits,
        role_swap_distractor_hits,
        best_constant_reply_hits,
        unscored_inputs_leak_count,
    })
}

/// Load the pinned `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`)
/// from `.uor-models/research/issue-1019/tokenizer/tokenizer.json` (or parent repo `.uor-models/` when in a `.worktrees/` checkout).
pub fn load_pinned_byte_bpe_tokenizer(repo_root: &Path) -> Result<ByteBpeTokenizer, String> {
    let candidates = [
        repo_root.join(".uor-models/research/issue-1019/tokenizer/tokenizer.json"),
        repo_root.join("../../.uor-models/research/issue-1019/tokenizer/tokenizer.json"),
        repo_root.join(".uor-models/research/issue-973-group-retention/tokenizer/tokenizer.json"),
        repo_root
            .join("../../.uor-models/research/issue-973-group-retention/tokenizer/tokenizer.json"),
    ];
    let tok_path = candidates
        .iter()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            format!(
                "pinned tokenizer.json ({PINNED_TOKENIZER_SHA256}) not found under {} (.uor-models/research/issue-1019/tokenizer/tokenizer.json)",
                repo_root.display()
            )
        })?;
    let tok_bytes = std::fs::read(tok_path).map_err(|e| format!("{}: {e}", tok_path.display()))?;
    if sha256_hex(&tok_bytes) != PINNED_TOKENIZER_SHA256 {
        return Err("tokenizer.json SHA-256 differs from PINNED_TOKENIZER_SHA256".to_string());
    }
    ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .ok_or_else(|| "ByteBpeTokenizer::from_tokenizer_json_bytes failed".to_string())
}

/// Execute the exact `FrozenAnswers` + `ByteBpeTokenizer` + `pair_groups` grading loop from
/// `crates/uor-r4-core/examples/native-bank-generalization.rs:827-948` over deserialized
/// `inputs.json` and `labels.json`, returning `(complete_rows, pairs_both_complete_distinct)`.
pub fn grade_generated_replies_with_evaluator_oracle(
    tok: &ByteBpeTokenizer,
    inputs: &ProbeInputs,
    labels: &ProbeLabels,
    generated_by_case: &[Vec<u32>],
) -> Result<(usize, usize), String> {
    if inputs.cases.len() != labels.cases.len() || inputs.cases.len() != generated_by_case.len() {
        return Err("case count mismatch in evaluator oracle grading".to_string());
    }
    let mut seen = BTreeSet::new();
    let mut complete = 0usize;
    let mut pair_groups: BTreeMap<String, Vec<(String, bool, Vec<u32>)>> = BTreeMap::new();
    for ((p, l), generated_ids) in inputs
        .cases
        .iter()
        .zip(&labels.cases)
        .zip(generated_by_case)
    {
        if p.id.is_empty() || p.id != l.id || !seen.insert(p.id.clone()) {
            return Err("input/label IDs mismatched or duplicate".to_string());
        }
        l.answers.validate().map_err(|e| e.to_string())?;
        let eos = generated_ids.last().copied() == Some(EOS_TOKEN_ID);
        let output_ids = if eos {
            &generated_ids[..generated_ids.len() - 1]
        } else {
            &generated_ids[..]
        };
        let decoded_bytes = tok.decode_bytes(output_ids);
        let valid_text = std::str::from_utf8(&decoded_bytes).ok();
        let accepted = eos && valid_text.is_some_and(|text| l.answers.accepts(text));
        complete += usize::from(accepted);
        let target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err("evaluator accepted form does not roundtrip tokenizer".to_string());
        }
        if l.pair_id.is_empty() {
            return Err("empty pair metadata".to_string());
        }
        pair_groups.entry(l.pair_id.clone()).or_default().push((
            p.id.clone(),
            accepted,
            generated_ids.clone(),
        ));
    }
    let mut pairs_both_complete_distinct = 0usize;
    for arms in pair_groups.values() {
        if arms.len() != 2 {
            return Err("each evaluator pair must have two cases".to_string());
        }
        let both = arms.iter().all(|v| v.1);
        let distinct = arms[0].2 != arms[1].2;
        pairs_both_complete_distinct += usize::from(both && distinct);
    }
    Ok((complete, pairs_both_complete_distinct))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .expect("repo root")
            .to_path_buf()
    }

    #[test]
    fn dev512_spread_matches_sealed_saved437_receipts() {
        let report = analyze_saved437_dev512_spread(&repo_root()).expect("dev512 spread");
        assert_eq!(report.total_rows, 512);
        assert_eq!(report.complete_rows, 437);
        assert_eq!(report.saved_audit_status, "PASS_SAVED_EVIDENCE");
        assert_eq!(report.saved_audit_checked_steps, 7908);
        assert_eq!(report.both_roles_complete_histories, 186);
        assert_eq!(report.forward_pass + report.reverse_pass, 437);
        assert_eq!(report.swap0_pass + report.swap1_pass, 437);
        assert_eq!(report.wording_a_pass + report.wording_b_pass, 437);
        assert_eq!(report.job_pass + report.home_pass, 437);
        assert_eq!(
            report.even_blocks_pass_of_256 + report.odd_blocks_pass_of_256,
            437
        );
        assert!(report.cluster_se_256_hundredths_rows > 0);
        assert!(
            report.bar40_cluster_se_hundredths_rows >= report.bar40_binomial_se_hundredths_rows
        );
    }

    #[test]
    fn real_byte_bpe_tokenizer_and_evaluator_oracle_verify_tokens_and_controls() {
        let root = repo_root();
        let tok = load_pinned_byte_bpe_tokenizer(&root).expect("load real ByteBpeTokenizer");
        for &(pretoken, expected_ids) in PRETOKEN_BPE_TABLE {
            assert_eq!(
                tok.encode(pretoken).as_slice(),
                expected_ids,
                "PRETOKEN_BPE_TABLE mismatch on {pretoken:?}"
            );
        }
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");
        let inputs: ProbeInputs = serde_json::from_slice(&bundle.inputs_json).expect("inputs");
        let labels: ProbeLabels = serde_json::from_slice(&bundle.labels_json).expect("labels");

        // Oracle ground-truth answers score 256/256 rows and 128/128 distinct pairs.
        let oracle_generated: Vec<Vec<u32>> = labels
            .cases
            .iter()
            .map(|l| {
                let mut ids = tok.encode(&l.answers.accepted[0]);
                ids.push(EOS_TOKEN_ID);
                ids
            })
            .collect();
        let (oracle_complete, oracle_pairs) = grade_generated_replies_with_evaluator_oracle(
            &tok,
            &inputs,
            &labels,
            &oracle_generated,
        )
        .expect("grade oracle");
        assert_eq!(oracle_complete, 256);
        assert_eq!(oracle_pairs, 128);
    }

    #[test]
    fn fresh256_panel_passes_all_overlap_balance_and_seal_checks() {
        let root = repo_root();
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");
        let again = generate_m2_fresh256_panel().expect("generate fresh256 again");
        assert_eq!(bundle.plan_sha256, again.plan_sha256);
        assert_eq!(bundle.inputs_sha256, again.inputs_sha256);
        assert_eq!(bundle.labels_sha256, again.labels_sha256);
        assert_eq!(bundle.manifest_sha256, again.manifest_sha256);

        let audit = verify_m2_fresh256_panel(&root, &bundle).expect("verify fresh256");
        assert_eq!(audit.rows, 256);
        assert_eq!(audit.histories, 128);
        assert_eq!(audit.unordered_blocks, 16);
        assert_eq!(audit.ordered_banks, 32);
        assert_eq!(audit.historical_literals_parsed_from_disk, 74);
        assert_eq!(audit.exact_literal_overlaps, 0);
        assert_eq!(audit.content_word_overlaps, 0);
        assert_eq!(audit.ngram_overlaps, 0);
        assert_eq!(audit.intra_panel_content_word_duplicates, 0);
        assert!(audit.min_same_stratum_edit_distance >= 8);
        assert!(audit.literal_role_balance);
        assert!(audit.literal_initial_position_balance);
        assert_eq!(audit.copy_first_stated_hits, 0);
        assert_eq!(audit.copy_last_stated_hits, 0);
        assert_eq!(audit.role_swap_distractor_hits, 0);
        assert_eq!(audit.best_constant_reply_hits, 0);
        assert_eq!(audit.unscored_inputs_leak_count, 0);
    }

    #[test]
    fn fresh256_panel_hashes_match_registered_panel_json() {
        let root = repo_root();
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");
        let panel_path = root.join("docs/harness/panels/m2-fresh256.json");
        let panel_text = std::fs::read_to_string(&panel_path)
            .expect("read docs/harness/panels/m2-fresh256.json");
        assert!(
            panel_text.contains(&bundle.plan_sha256),
            "m2-fresh256.json missing plan_sha256 {}",
            bundle.plan_sha256
        );
        assert!(
            panel_text.contains(&bundle.inputs_sha256),
            "m2-fresh256.json missing inputs_sha256 {}",
            bundle.inputs_sha256
        );
        assert!(
            panel_text.contains(&bundle.labels_sha256),
            "m2-fresh256.json missing labels_sha256 {}",
            bundle.labels_sha256
        );
        assert!(
            panel_text.contains(&bundle.manifest_sha256),
            "m2-fresh256.json missing manifest_sha256 {}",
            bundle.manifest_sha256
        );
    }

    #[test]
    fn negative_mutations_are_rejected_by_verifier() {
        let root = repo_root();
        let bundle = generate_m2_fresh256_panel().expect("generate fresh256");

        // 1. Injecting an exposed historical literal is caught by overlap exclusion.
        let mut leaked_lit = bundle.clone();
        for h in &mut leaked_lit.histories {
            for t in &mut h.turns {
                if t.text.contains("basalt loggia") {
                    t.text = t.text.replace("basalt loggia", "azure orchard");
                }
            }
        }
        let err1 = verify_m2_fresh256_panel(&root, &leaked_lit).unwrap_err();
        assert!(err1.contains("overlap exclusion failed"), "got: {err1}");

        // 2. Breaking initial position balance is caught.
        let mut unbalanced = bundle.clone();
        unbalanced.histories[0].turns.swap(0, 1);
        let err2 = verify_m2_fresh256_panel(&root, &unbalanced).unwrap_err();
        assert!(
            err2.contains("literal initial position imbalance"),
            "got: {err2}"
        );

        // 3. Leaking `"accepted"` into unscored `inputs_json` is caught.
        let mut leaked_inputs = bundle.clone();
        let inputs_text = String::from_utf8(leaked_inputs.inputs_json.clone()).expect("utf8");
        leaked_inputs.inputs_json = inputs_text
            .replacen(
                "fresh-diverse-length2-00-swap0-q0-forward-job",
                "accepted",
                1,
            )
            .into_bytes();
        leaked_inputs.inputs_sha256 = sha256_hex(&leaked_inputs.inputs_json);
        let err3 = verify_m2_fresh256_panel(&root, &leaked_inputs).unwrap_err();
        assert!(
            err3.contains("inputs.json leaked accepted answer strings"),
            "got: {err3}"
        );

        // 4. Corrupting a SHA-256 seal is caught.
        let mut bad_seal = bundle;
        bad_seal.labels_sha256 = "0".repeat(64);
        let err4 = verify_m2_fresh256_panel(&root, &bad_seal).unwrap_err();
        assert!(err4.contains("bundle SHA-256 mismatch"), "got: {err4}");
    }
}
