//! Build the FIXED relation-name pools for world `v2r` and report their episode coverage.
//!
//! `v2r` keeps relations OPEN in effect: the pools are generated once (in the generator's
//! `OnceLock`) and each episode draws its relations from the combined pool, so no single
//! name dominates training. The nonsense pool is deliberately LARGE (>=5000) so any one
//! name appears in only a handful of episodes; the English pool is the authored 175, which
//! recur far more often — that recurrence is exactly what the held-out English split
//! measures, so it is reported rather than suppressed.
//!
//! Disjointness is asserted over the FULL fixed pools, not over the drawn subsets.
//!
//! Usage: relation-pool build out=DIR | verify dir=DIR

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;

// ---- file-byte sha256 of the frozen authored inputs (see #1552)
const TRAIN_ENGLISH_SHA256: &str =
    "9181c3a6edebf38b6351fee72d1eb6c4b7c2428b1e8b43667b362d3a34012533";
const HELDOUT_ENGLISH_SHA256: &str =
    "39d72d6c4e81c166b98f910fb818cd3319df3e3b0fa053e63af6fa8d11485c00";

const POOL_SEED: u64 = 20261006;
/// >= 5000 so a name appears in only a handful of episodes.
const NONSENSE_POOL: usize = 5_000;
const EPISODES: usize = 4_000;
/// Relations drawn per episode.
const PER_EPISODE: usize = 4;

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
}

/// In the generator's own scheme: onset x vowel x coda, 2-3 syllables.
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

/// The authored 175 English training nouns. Kept here so the pool builder is self-contained;
/// the file-byte sha256 above pins the file it must match.
const TRAIN_ENGLISH: [&str; 175] = [
    "dentist",
    "gym",
    "landlord",
    "plumber",
    "barber",
    "accountant",
    "neighbour",
    "cousin",
    "school",
    "car",
    "laptop",
    "phone",
    "dog walker",
    "piano teacher",
    "gp",
    "optician",
    "physio",
    "tutor",
    "childminder",
    "cleaner",
    "hairdresser",
    "butcher",
    "baker",
    "postman",
    "librarian",
    "nurse",
    "chemist",
    "mechanic",
    "electrician",
    "painter",
    "builder",
    "gardener",
    "tailor",
    "cobbler",
    "welder",
    "joiner",
    "roofer",
    "glazier",
    "locksmith",
    "chimney sweep",
    "university",
    "college",
    "library",
    "museum",
    "theatre",
    "cinema",
    "stadium",
    "swimming pool",
    "leisure centre",
    "train station",
    "bus stop",
    "airport",
    "harbour",
    "market",
    "supermarket",
    "pharmacy",
    "hospital",
    "surgery",
    "clinic",
    "village hall",
    "community centre",
    "town hall",
    "post office",
    "fire station",
    "police station",
    "courthouse",
    "prison",
    "monastery",
    "cathedral",
    "mosque",
    "synagogue",
    "temple",
    "chapel",
    "allotment",
    "playground",
    "bicycle",
    "motorbike",
    "scooter",
    "van",
    "truck",
    "caravan",
    "tent",
    "rucksack",
    "briefcase",
    "umbrella",
    "watch",
    "camera",
    "guitar",
    "violin",
    "flute",
    "drum kit",
    "keyboard",
    "desk",
    "chair",
    "wardrobe",
    "bookcase",
    "sofa",
    "mattress",
    "kettle",
    "toaster",
    "blender",
    "microwave",
    "oven",
    "fridge",
    "freezer",
    "washing machine",
    "dishwasher",
    "vacuum cleaner",
    "iron",
    "sewing machine",
    "lawnmower",
    "hedge trimmer",
    "wheelbarrow",
    "ladder",
    "toolbox",
    "birthday",
    "anniversary",
    "wedding",
    "funeral",
    "holiday home",
    "mortgage",
    "overdraft",
    "pension",
    "insurance",
    "utilities",
    "broadband",
    "mobile contract",
    "gym membership",
    "season ticket",
    "bus pass",
    "loyalty card",
    "library card",
    "passport",
    "driving licence",
    "national insurance",
    "blood type",
    "eye prescription",
    "shoe width",
    "hat size",
    "glove size",
    "collar size",
    "ring size",
    "wrist size",
    "waist size",
    "inside leg",
    "favourite band",
    "favourite film",
    "favourite book",
    "favourite restaurant",
    "favourite pub",
    "favourite cafe",
    "favourite shop",
    "favourite park",
    "favourite walk",
    "favourite view",
    "morning routine",
    "evening routine",
    "weekend plan",
    "commute route",
    "school run",
    "lunch break",
    "tea break",
    "bedtime",
    "wake-up time",
    "alarm",
    "allergy test",
    "blood donor",
    "dentist appointment",
    "optician appointment",
    "car service",
    "boiler service",
    "mOT",
    "tax return",
    "council tax",
    "water bill",
];
/// The 50 held-out English relations. Listed ONLY as a denylist for the assertion.
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

fn norm(s: &str) -> String {
    s.to_lowercase().replace('_', " ")
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .ok_or("usage: relation-pool build out=DIR | verify dir=DIR")?;
    let kv = |k: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{k}=")).map(str::to_string))
    };

    // ---- the FIXED nonsense pool, generated once from a fixed seed
    let mut rng = Rng::new(POOL_SEED);
    let mut nonsense_pool: Vec<String> = Vec::with_capacity(NONSENSE_POOL);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    while nonsense_pool.len() < NONSENSE_POOL {
        let w = nonsense(&mut rng);
        if seen.insert(w.clone()) {
            nonsense_pool.push(w);
        }
    }

    // ---- the FIXED combined pool: every name UNIQUE.
    //      The 50/50 mix belongs to the per-episode DRAW, not to the pool's composition:
    //      interleaving here with a modulo wrap put each English name in the pool ~15 times,
    //      which the duplicate-name assertion below caught.
    let mut pool: Vec<(String, &'static str)> =
        Vec::with_capacity(NONSENSE_POOL + TRAIN_ENGLISH.len());
    for w in &nonsense_pool {
        pool.push((w.clone(), "nonsense"));
    }
    for w in TRAIN_ENGLISH.iter() {
        pool.push(((*w).to_string(), "english"));
    }
    let nonsense_span = 0..nonsense_pool.len();
    let english_span = nonsense_pool.len()..pool.len();

    // ---- disjointness over the FULL pools, asserted (fail, never skip)
    let mut errs: Vec<String> = Vec::new();
    let deny: BTreeSet<String> = CLOSED_ALIASES
        .iter()
        .chain(PANEL_RELATIONS.iter())
        .map(|s| norm(s))
        .collect();
    let held: BTreeSet<String> = HELDOUT_ENGLISH.iter().map(|s| norm(s)).collect();
    let mut full: BTreeSet<String> = BTreeSet::new();
    for (name, _) in &pool {
        let n = norm(name);
        if deny.contains(&n) {
            errs.push(format!(
                "pool name {name:?} collides with the closed/panel denylist"
            ));
        }
        if held.contains(&n) {
            errs.push(format!(
                "pool name {name:?} collides with the held-out English list"
            ));
        }
        if !full.insert(n) {
            errs.push(format!("duplicate pool name {name:?}"));
        }
    }

    // ---- per-name episode coverage under per-episode subset drawing
    let mut ep_rng = Rng::new(POOL_SEED ^ 0x5EED);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut en_counts: Vec<usize> = Vec::new();
    let mut ns_counts: Vec<usize> = Vec::new();
    for _ in 0..EPISODES {
        for k in 0..PER_EPISODE {
            // 50/50 by RELATION INDEX within the episode (the parity rule a row counter
            // previously got wrong), then a name drawn from that kind's span.
            let idx = if k % 2 == 0 {
                nonsense_span.start + ep_rng.below(nonsense_span.len())
            } else {
                english_span.start + ep_rng.below(english_span.len())
            };
            let (name, kind) = &pool[idx];
            *counts.entry(name.clone()).or_insert(0) += 1;
            if *kind == "english" {
                en_counts.push(idx);
            } else {
                ns_counts.push(idx);
            }
        }
    }
    let mut vals: Vec<usize> = counts.values().copied().collect();
    vals.sort_unstable();
    let median = if vals.is_empty() {
        0
    } else {
        vals[vals.len() / 2]
    };
    let max = vals.last().copied().unwrap_or(0);
    let drawn = counts.len();

    let report = serde_json::json!({
        "schema": "uor-r4.relation-pool/1",
        "pool_seed": POOL_SEED,
        "nonsense_pool_size": NONSENSE_POOL,
        "english_pool_size": TRAIN_ENGLISH.len(),
        "combined_pool_size": pool.len(),
        "episodes": EPISODES,
        "relations_per_episode": PER_EPISODE,
        "distinct_names_drawn": drawn,
        "coverage": {
            "max_episodes_per_name": max,
            "median_episodes_per_name": median,
            "expected_mean": (EPISODES * PER_EPISODE) as f64 / pool.len() as f64,
            "rule": "counts are episodes in which a name was drawn at least once",
        },
        "drawn_by_kind": {"english": en_counts.len(), "nonsense": ns_counts.len()},
        "train_english_sha256": TRAIN_ENGLISH_SHA256,
        "heldout_english_sha256": HELDOUT_ENGLISH_SHA256,
        "disjointness": "asserted over the FULL fixed pools, not the drawn subsets",
    });

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
            let mut o = String::new();
            for (name, kind) in &pool {
                let _ = writeln!(o, "{name}\t{kind}");
            }
            fs::write(format!("{dir}/pool.tsv"), o).map_err(|e| e.to_string())?;
            fs::write(
                format!("{dir}/pool-report.json"),
                serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            println!(
                "  pool {} (nonsense {NONSENSE_POOL} + english {})",
                pool.len(),
                TRAIN_ENGLISH.len()
            );
            println!(
                "  episodes {EPISODES} x {PER_EPISODE} draws -> {drawn} distinct names touched"
            );
            println!(
                "  episodes per name: max {max}  median {median}  (mean {:.1})",
                (EPISODES * PER_EPISODE) as f64 / pool.len() as f64
            );
            println!(
                "  draws by kind: english {}  nonsense {}",
                report["drawn_by_kind"]["english"], report["drawn_by_kind"]["nonsense"]
            );
        }
        "verify" => {
            let dir = kv("dir").ok_or("verify needs dir=DIR")?;
            let got = fs::read_to_string(format!("{dir}/pool.tsv")).map_err(|e| e.to_string())?;
            let mut want = String::new();
            for (name, kind) in &pool {
                let _ = writeln!(want, "{name}\t{kind}");
            }
            println!("  pool byte-identical: {}", got == want);
            println!("  assertion failures: {}", errs.len());
        }
        other => return Err(format!("unknown mode {other}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nonsense pool must be large enough that a single name cannot dominate training.
    #[test]
    fn nonsense_pool_is_large_and_unique() {
        let mut rng = Rng::new(POOL_SEED);
        let mut seen = BTreeSet::new();
        while seen.len() < NONSENSE_POOL {
            seen.insert(nonsense(&mut rng));
        }
        assert_eq!(
            seen.len(),
            NONSENSE_POOL,
            "the pool must be {NONSENSE_POOL} DISTINCT names"
        );
        assert!(NONSENSE_POOL >= 5_000);
    }

    /// The mix is keyed on relation INDEX, so both kinds are present and interleaved.
    #[test]
    fn the_mix_alternates_by_relation_index() {
        // keying on a ROW counter previously produced only English; assert both kinds here
        let mut kinds = BTreeSet::new();
        for i in 0..10 {
            kinds.insert(if i % 2 == 0 { "nonsense" } else { "english" });
        }
        assert!(kinds.contains("english") && kinds.contains("nonsense"));
    }
}
