//! Fixture splits under `CARGO_TARGET_TMPDIR`; every refusal and pin of the scale-probe order.

use scale_probe_instrument::{
    assess, discordant, mcnemar_floor, ngram_key, prefix_len, top1, train_ngrams, Tables, NGRAM,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};

fn base(test: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("scale-probe")
        .join(test);
    if p.exists() {
        fs::remove_dir_all(&p).unwrap();
    }
    fs::create_dir_all(&p).unwrap();
    p
}

fn sha(path: &Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

struct Fx {
    dir: PathBuf,
    t: String,
    m: String,
}

impl Fx {
    fn spec(&self) -> String {
        format!("{}:{}:{}", self.dir.display(), self.t, self.m)
    }
    fn named(&self, name: &str) -> String {
        format!("{name}:{}", self.spec())
    }
    fn edit_manifest(&self, f: impl FnOnce(&mut Value)) {
        let p = self.dir.join("manifest.json");
        let mut v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        f(&mut v);
        fs::write(p, serde_json::to_vec(&v).unwrap()).unwrap();
    }
}

fn split(dir: &Path, tokens: &[u16], mask: &[u8], vocab: u32, manifest: bool) -> Fx {
    fs::create_dir_all(dir).unwrap();
    let tp = dir.join("tokens.u16");
    let mut w = CorpusWriter::create(&tp, vocab).unwrap();
    w.write_tokens(tokens).unwrap();
    w.finish().unwrap();
    let mp = dir.join("response_mask.u8");
    fs::write(&mp, mask).unwrap();
    let (t, m) = (sha(&tp), sha(&mp));
    if manifest {
        let v = json!({
            "source": format!("src-{}", dir.display()), "revision": "r1", "licence": "L",
            "attribution": "A", "bos_id": 0, "eos_id": 1, "tokens": tokens.len(),
            "content_bytes": 3 * tokens.len(), "tokens_sha256": t, "mask_sha256": m,
        });
        fs::write(dir.join("manifest.json"), serde_json::to_vec(&v).unwrap()).unwrap();
    }
    Fx {
        dir: dir.to_path_buf(),
        t,
        m,
    }
}

/// Documents of lengths 3, 2, 6, 1, 4 (16 tokens; starts 0, 3, 5, 11, 12).
fn root_a(dir: &Path) -> Fx {
    let t = [0, 2, 3, 0, 4, 0, 5, 6, 7, 8, 9, 0, 0, 10, 11, 12];
    let m = [0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1];
    split(dir, &t, &m, 4096, true)
}

/// Documents of lengths 4, 4.
fn root_b(dir: &Path) -> Fx {
    split(
        dir,
        &[0, 20, 21, 22, 0, 23, 24, 25],
        &[0, 1, 1, 1, 0, 1, 1, 1],
        4096,
        true,
    )
}

/// unigram 5:2 9:2 7:1 6:1; bigram 0->{5:2, 9:1}... see `tie_rule_lower_id_wins`.
fn train_fx(dir: &Path) -> Fx {
    split(
        dir,
        &[0, 5, 7, 0, 5, 6, 0, 9, 9],
        &[0, 1, 1, 0, 1, 1, 0, 1, 1],
        4096,
        true,
    )
}

const PANEL_T: [u16; 8] = [0, 5, 7, 0, 5, 6, 9, 8];
const PANEL_M: [u8; 8] = [0, 1, 1, 0, 1, 1, 1, 0];

fn panel_fx(dir: &Path) -> Fx {
    split(dir, &PANEL_T, &PANEL_M, 4096, true)
}

fn run(args: &[String]) -> Result<String, String> {
    scale_probe_instrument::run(args)
}

fn args(cmd: &str, kv: &[(&str, String)]) -> Vec<String> {
    let mut v = vec![cmd.to_string()];
    v.extend(kv.iter().map(|(k, x)| format!("{k}={x}")));
    v
}

fn refused(r: Result<String, String>, needle: &str) {
    match r {
        Ok(s) => panic!("expected a refusal containing {needle:?}, got Ok({s})"),
        Err(e) => assert!(e.contains(needle), "refusal {e:?} lacks {needle:?}"),
    }
}

fn mix(out: &Path, frac: &str, roots: &[String]) -> Result<String, String> {
    let mut kv = vec![
        ("out", out.display().to_string()),
        ("fraction", frac.to_string()),
    ];
    kv.extend(roots.iter().map(|r| ("root", r.clone())));
    run(&args("mix", &kv))
}

fn read_tokens(dir: &Path) -> Vec<u16> {
    MmapCorpusReader::open(dir.join("tokens.u16"))
        .unwrap()
        .as_slice()
        .to_vec()
}

fn rows_file(path: &Path, split: &str, pos: &[usize], tgt: &[u32], near: &[u32], wr: &[u32]) {
    let v = json!({"split": split, "memory": false, "positions": pos, "targets": tgt,
        "nearest": near, "writer": wr});
    fs::write(path, serde_json::to_vec(&v).unwrap()).unwrap();
}

fn inputs_cmd(
    cmd: &str,
    out: &Path,
    train: &Fx,
    panels: &[(&str, &Fx)],
    rows: &[(&str, &Path)],
) -> Result<String, String> {
    let mut kv = vec![("out", out.display().to_string()), ("train", train.spec())];
    kv.extend(panels.iter().map(|(n, f)| ("panel", f.named(n))));
    kv.extend(
        rows.iter()
            .map(|(n, p)| ("rows", format!("{n}:{}", p.display()))),
    );
    run(&args(cmd, &kv))
}

fn grade(
    out: &Path,
    train: &Fx,
    panels: &[(&str, &Fx)],
    rows: &[(&str, &Path)],
) -> Result<String, String> {
    inputs_cmd("grade", out, train, panels, rows)
}

/// Grades one `p` panel rows file against `train_fx`.
fn grade_rows(test: &str, pos: &[usize], tgt: &[u32], near: &[u32]) -> Result<String, String> {
    let b = base(test);
    let (tr, pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    let rp = b.join("rows.json");
    rows_file(&rp, "p", pos, tgt, near, near);
    grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)])
}

fn json_of(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

// ---- mix ----

#[test]
fn mix_prefix_rule_nesting_and_root_order() {
    let b = base("mix_prefix");
    let (ra, rb) = (root_a(&b.join("a")), root_b(&b.join("b")));
    let ta = read_tokens(&ra.dir);
    assert_eq!(
        [
            prefix_len(&ta, 1, 4),
            prefix_len(&ta, 1, 2),
            prefix_len(&ta, 1, 1)
        ],
        [5, 11, 16]
    );
    let roots = [rb.named("b"), ra.named("a")];
    let mut outs = Vec::new();
    for (frac, la, lb) in [("1/4", 5, 4), ("1/2", 11, 4), ("1/1", 16, 8)] {
        let out = b.join(frac.replace('/', "-"));
        mix(&out, frac, &roots).unwrap();
        let m = json_of(&out.join("manifest.json"));
        assert_eq!(m["roots"][0]["name"], "b");
        assert_eq!(m["roots"][1]["tokens_taken"], la);
        assert_eq!(m["roots"][0]["tokens_taken"], lb);
        assert_eq!(m["tokens"], la + lb);
        assert_eq!(m["tokens_sha256"], sha(&out.join("tokens.u16")));
        assert_eq!(m["mask_sha256"], sha(&out.join("response_mask.u8")));
        assert_eq!(
            m["roots"][1]["source"],
            json!(format!("src-{}", ra.dir.display()))
        );
        let toks = read_tokens(&out);
        let tb = read_tokens(&rb.dir);
        assert_eq!(toks, [&tb[..lb], &ta[..la]].concat());
        outs.push((lb, la));
    }
    let q = json_of(&b.join("1-4/manifest.json"));
    assert_eq!(q["roots"][1]["documents_taken"], 2);
    assert_eq!(q["roots"][1]["scored_taken"], 3);
    assert_eq!(q["documents"], 3);
    // Nesting: each root's 1/4 prefix is a prefix of its 1/2, which is a prefix of the whole.
    assert!(outs.windows(2).all(|w| {
        let (a, b) = (&w[0], &w[1]);
        a.0 <= b.0 && a.1 <= b.1
    }));
}

#[test]
fn mix_refuses_existing_out() {
    let b = base("mix_out");
    let ra = root_a(&b.join("a"));
    fs::create_dir_all(b.join("o")).unwrap();
    refused(mix(&b.join("o"), "1/2", &[ra.named("a")]), "already exists");
}

#[test]
fn mix_refuses_tokens_sha_mismatch() {
    let b = base("mix_toksha");
    let mut ra = root_a(&b.join("a"));
    ra.t = "0".repeat(64);
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a")]),
        "tokens.u16: sha256",
    );
}

#[test]
fn mix_refuses_mask_sha_mismatch() {
    let b = base("mix_masksha");
    let mut ra = root_a(&b.join("a"));
    ra.m = "0".repeat(64);
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a")]),
        "response_mask.u8: sha256",
    );
}

#[test]
fn mix_refuses_manifest_sha_disagreement() {
    let b = base("mix_mfsha");
    let ra = root_a(&b.join("a"));
    ra.edit_manifest(|v| v["mask_sha256"] = json!("ab"));
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a")]),
        "manifest.json: mask_sha256",
    );
}

#[test]
fn mix_refuses_missing_manifest() {
    let b = base("mix_nomf");
    let ra = split(&b.join("a"), &[0, 2, 3], &[0, 1, 1], 4096, false);
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a")]),
        "manifest.json: missing",
    );
}

#[test]
fn mix_refuses_count_mask_length_mismatch() {
    let b = base("mix_len");
    let ra = split(&b.join("a"), &[0, 2, 3], &[0, 1], 4096, true);
    refused(mix(&b.join("o"), "1/2", &[ra.named("a")]), "3 tokens but");
}

#[test]
fn mix_refuses_non_bos_first_token() {
    let b = base("mix_bos0");
    let ra = split(&b.join("a"), &[2, 0, 3], &[0, 0, 1], 4096, true);
    refused(mix(&b.join("o"), "1/2", &[ra.named("a")]), "tokens[0]");
}

#[test]
fn mix_refuses_manifest_bos_id() {
    let b = base("mix_bosid");
    let ra = root_a(&b.join("a"));
    ra.edit_manifest(|v| v["bos_id"] = json!(1));
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a")]),
        "bos_id 1 is not 0",
    );
}

#[test]
fn mix_refuses_duplicate_root_name() {
    let b = base("mix_dup");
    let (ra, rb) = (root_a(&b.join("a")), root_b(&b.join("b")));
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("x"), rb.named("x")]),
        "duplicate root name",
    );
}

#[test]
fn mix_refuses_vocab_difference() {
    let b = base("mix_vocab");
    let ra = root_a(&b.join("a"));
    let rc = split(&b.join("c"), &[0, 2, 3], &[0, 1, 1], 4000, true);
    refused(
        mix(&b.join("o"), "1/2", &[ra.named("a"), rc.named("c")]),
        "vocab_size 4000",
    );
}

#[test]
fn mix_refuses_bad_fraction() {
    let b = base("mix_frac");
    let ra = root_a(&b.join("a"));
    for f in ["0/4", "5/4", "1/65", "1-2"] {
        refused(mix(&b.join("o"), f, &[ra.named("a")]), "fraction");
    }
    assert!(!b.join("o").exists());
}

// ---- tables ----

#[test]
fn tie_rule_lower_id_wins() {
    assert_eq!(top1(&[0u64, 3, 3, 1]), Some(1));
    assert_eq!(top1(&[0u32, 0]), None);
    let t: Vec<u16> = vec![0, 5, 7, 0, 5, 6, 0, 9, 9];
    let tb = Tables::build(&t, &[0, 1, 1, 0, 1, 1, 0, 1, 1]).unwrap();
    assert_eq!(tb.unigram_top1, 5, "unigram tie 5:2 vs 9:2");
    assert_eq!(tb.t2(5), 6, "bigram tie after 5: 6:1 vs 7:1");
}

#[test]
fn unseen_key_falls_back_to_unigram_top1() {
    let t: Vec<u16> = vec![0, 5, 7, 0, 5, 6, 0, 9, 9];
    let tb = Tables::build(&t, &[0, 1, 1, 0, 1, 1, 0, 1, 1]).unwrap();
    assert_eq!(tb.bigram_top1[7], None);
    assert_eq!(tb.t2(7), tb.unigram_top1);
    assert_eq!(tb.t2(4095), 5);
    assert_eq!(tb.t2(9), 9);
}

#[test]
fn bos_targets_and_mask_zero_are_not_counted() {
    let tb = Tables::build(&[0, 3, 0, 4, 4], &[0, 1, 1, 0, 1]).unwrap();
    assert_eq!(tb.scored_targets, 2);
    assert_eq!(tb.unigram_top1, 3);
    assert_eq!(
        tb.bigram_top1[3], None,
        "the BOS at position 2 is no target"
    );
    assert_eq!(tb.bigram_keys_seen, 2);
}

// ---- grade ----

#[test]
fn grade_refuses_off_by_one_rows() {
    grade_rows("grade_obo_ok", &[4, 5], &[5, 6], &[5, 6]).unwrap();
    refused(
        grade_rows("grade_obo", &[5, 6], &[5, 6], &[5, 6]),
        "!= target",
    );
}

#[test]
fn grade_refuses_existing_out() {
    let b = base("grade_out");
    let (tr, pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[1], &[5], &[5], &[5]);
    fs::create_dir_all(b.join("g")).unwrap();
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "already exists",
    );
}

#[test]
fn grade_refuses_sha_mismatch() {
    let b = base("grade_sha");
    let (tr, mut pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    pa.t = tr.t.clone();
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[1], &[5], &[5], &[5]);
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "tokens.u16: sha256",
    );
}

#[test]
fn grade_refuses_train_sha_or_panel_mask_sha() {
    let b = base("grade_sha2");
    let (mut tr, pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[1], &[5], &[5], &[5]);
    let good = tr.t.clone();
    tr.t = pa.t.clone();
    refused(
        grade(&b.join("g1"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "train/tokens.u16: sha256",
    );
    tr.t = good;
    let pm = Fx {
        dir: pa.dir.clone(),
        t: pa.t.clone(),
        m: tr.m.clone(),
    };
    refused(
        grade(&b.join("g2"), &tr, &[("p", &pm)], &[("p", &rp)]),
        "panel/response_mask.u8: sha256",
    );
}

#[test]
fn grade_refuses_count_mask_length_mismatch() {
    let b = base("grade_len");
    let tr = split(&b.join("train"), &[0, 5, 7], &[0, 1], 4096, false);
    let pa = panel_fx(&b.join("panel"));
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[1], &[5], &[5], &[5]);
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "3 tokens but",
    );
}

#[test]
fn grade_refuses_split_name_mismatch() {
    let b = base("grade_name");
    let (tr, pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    let rp = b.join("rows.json");
    rows_file(&rp, "other", &[1], &[5], &[5], &[5]);
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "!= rows name p",
    );
}

#[test]
fn grade_refuses_unequal_lists() {
    refused(
        grade_rows("grade_lists", &[1, 2], &[5], &[5, 7]),
        "unequal list lengths",
    );
}

#[test]
fn grade_refuses_position_zero() {
    refused(
        grade_rows("grade_pos0", &[0], &[0], &[0]),
        "row 0: position 0",
    );
}

#[test]
fn grade_refuses_position_beyond_panel() {
    refused(
        grade_rows("grade_far", &[1, 8], &[5, 5], &[5, 5]),
        "row 1: position 8: beyond",
    );
}

#[test]
fn grade_refuses_bos_target() {
    refused(grade_rows("grade_bos", &[3], &[0], &[0]), "target is BOS");
}

#[test]
fn grade_refuses_mask_zero_row() {
    refused(grade_rows("grade_mask", &[7], &[8], &[8]), "mask is 0");
}

#[test]
fn grade_refuses_duplicate_position() {
    refused(
        grade_rows("grade_dup", &[1, 2, 1], &[5, 7, 5], &[5, 7, 5]),
        "row 2: position 1",
    );
}

#[test]
fn grade_refuses_ids_beyond_vocab() {
    refused(
        grade_rows("grade_id", &[1], &[5], &[4096]),
        "nearest id 4096",
    );
}

#[test]
fn grade_refuses_a_panel_token_beyond_vocab() {
    // Token 5000 keys the scored row at 2; T2 has no row for it.
    let b = base("grade_tok");
    let tr = train_fx(&b.join("train"));
    let pa = split(&b.join("panel"), &[0, 5000, 7], &[0, 0, 1], 8192, true);
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[2], &[7], &[7], &[7]);
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[("p", &rp)]),
        "token 1 is 5000, not below the vocabulary of 4096",
    );
}

#[test]
fn seen_refuses_existing_out() {
    let c = clean_setup("seen_out");
    let panels = [("p", &c.pa), ("q", &c.pa)];
    let rows = [("p", c.rows[0].as_path()), ("q", &c.rows[1])];
    refused(seen_cmd(&c.seen, &c.tr, &panels, &rows), "already exists");
}

#[test]
fn grade_refuses_unmatched_panel_or_rows() {
    let b = base("grade_match");
    let (tr, pa) = (train_fx(&b.join("train")), panel_fx(&b.join("panel")));
    let rp = b.join("rows.json");
    rows_file(&rp, "p", &[1], &[5], &[5], &[5]);
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &[]),
        "panel p: no rows file",
    );
    let both = [("p", rp.as_path()), ("q", rp.as_path())];
    refused(
        grade(&b.join("g"), &tr, &[("p", &pa)], &both),
        "rows q: no panel",
    );
}

// ---- band ----

#[test]
fn floor_values() {
    let got: Vec<u64> = [0, 1, 2, 4, 100]
        .iter()
        .map(|&n| mcnemar_floor(n))
        .collect();
    assert_eq!(got, [0, 2, 3, 4, 20]);
}

#[test]
fn band_ahead_behind_inside_mixed() {
    let ahead = assess([100, 100], [0, 0]);
    assert_eq!((ahead.seeds[0].band, ahead.verdict), (20, "ahead"));
    assert_eq!(assess([0, 0], [100, 100]).verdict, "behind");
    let inside = assess([1, 1], [1, 1]);
    assert_eq!(
        (inside.seeds[0].d, inside.seeds[0].band, inside.verdict),
        (0, 3, "inside")
    );
    let mixed = assess([100, 20], [0, 0]);
    assert_eq!(
        [mixed.seeds[0].position, mixed.seeds[1].position],
        ["ahead", "inside"]
    );
    assert_eq!(mixed.verdict, "mixed");
}

#[test]
fn band_reads_a_third_seed() {
    // The third seed must agree: two ahead and one inside is mixed.
    let m = assess([100, 100, 20], [0, 0, 0]);
    assert_eq!(m.seeds.len(), 3);
    assert_eq!(
        m.seeds.iter().map(|s| s.position).collect::<Vec<_>>(),
        ["ahead", "ahead", "inside"]
    );
    assert_eq!(m.verdict, "mixed");
    // The spread runs over all three seeds: d = 100, 50, 0 gives a band of 100, so all are
    // inside. The first two alone (spread 50) would read seed 1 as ahead.
    let s = assess([100, 50, 0], [0, 0, 0]);
    assert_eq!((s.spread, s.verdict), (100, "inside"));
    assert_eq!(assess([100, 50], [0, 0]).verdict, "mixed");
}

#[test]
fn band_spread_sets_the_band() {
    // Hits as LSB-first bitsets: X gets rows 0..100 at seed 1 and none at seed 2; Y gets none.
    let x1 = vec![0xffu8; 13];
    let none = vec![0u8; 13];
    let (b1, c1) = discordant(&x1, &none, 0, 100);
    let (b2, c2) = discordant(&none, &none, 0, 100);
    assert_eq!((b1, c1, b2, c2), (100, 0, 0, 0));
    let a = assess([b1, b2], [c1, c2]);
    assert_eq!(a.spread, 100);
    assert_eq!(
        [a.seeds[0].band, a.seeds[1].band],
        [100, 100],
        "floors are only 20 and 0"
    );
    assert_eq!(a.verdict, "inside");
}

// ---- pair and the end-to-end round trip ----

struct Cell {
    b: PathBuf,
    tr: Fx,
    pa: Fx,
}

fn e2e_setup(test: &str) -> Cell {
    let b = base(test);
    let root = train_fx(&b.join("root"));
    mix(&b.join("mix"), "1/1", &[root.named("r")]).unwrap();
    let m = json_of(&b.join("mix/manifest.json"));
    let tr = Fx {
        dir: b.join("mix"),
        t: m["tokens_sha256"].as_str().unwrap().into(),
        m: m["mask_sha256"].as_str().unwrap().into(),
    };
    let pa = panel_fx(&b.join("panel"));
    Cell { b, tr, pa }
}

const POS: [usize; 5] = [1, 2, 4, 5, 6];
const TGT: [u32; 5] = [5, 7, 5, 6, 9];

fn grade_cell(c: &Cell, name: &str, near: &[u32]) -> PathBuf {
    let rp = c.b.join(format!("{name}.rows.json"));
    rows_file(&rp, "p", &POS, &TGT, near, &[0, 0, 0, 0, 9]);
    let out = c.b.join(name);
    grade(&out, &c.tr, &[("p", &c.pa)], &[("p", &rp)]).unwrap();
    out
}

fn pair(out: &Path, kv: &[(&str, &Path)], table: Option<&str>) -> Result<String, String> {
    let mut v = vec![("out", out.display().to_string())];
    v.extend(kv.iter().map(|(k, p)| (*k, p.display().to_string())));
    if let Some(t) = table {
        v.push(("table", t.to_string()));
    }
    run(&args("pair", &v))
}

#[test]
fn pair_refuses_cells_with_mixed_train() {
    let c = e2e_setup("mixed_train");
    let tr2 = split(
        &c.b.join("train2"),
        &[0, 5, 7, 0, 5, 6, 0, 9, 9, 0, 5],
        &[0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1],
        4096,
        true,
    );
    let pa = Fx {
        dir: c.pa.dir.clone(),
        t: c.pa.t.clone(),
        m: c.pa.m.clone(),
    };
    let c2 = Cell {
        b: c.b.clone(),
        tr: tr2,
        pa,
    };
    let [a1, a2, a3, b1, b2, b3] =
        ["a1", "a2", "a3", "b1", "b2", "b3"].map(|n| grade_cell(&c, n, &TGT));
    let [x1, x2, x3] = ["x1", "x2", "x3"].map(|n| grade_cell(&c2, n, &TGT));
    let go = |name: &str, kv: &[(&str, &PathBuf)]| {
        let kv: Vec<(&str, &Path)> = kv.iter().map(|(k, p)| (*k, p.as_path())).collect();
        pair(&c.b.join(name), &kv, None)
    };
    let a = [("a1", &a1), ("a2", &a2)];
    let cell_a = "cell a's seeds share TRAIN";
    let cell_b = "cell b's seeds share TRAIN";
    refused(
        go(
            "a3.json",
            &[
                a[0],
                a[1],
                ("a3", &x3),
                ("b1", &b1),
                ("b2", &b2),
                ("b3", &b3),
            ],
        ),
        cell_a,
    );
    refused(
        go("b2.json", &[a[0], a[1], ("b1", &b1), ("b2", &x2)]),
        cell_b,
    );
    refused(
        go(
            "b3.json",
            &[
                a[0],
                a[1],
                ("a3", &a3),
                ("b1", &b1),
                ("b2", &b2),
                ("b3", &x3),
            ],
        ),
        cell_b,
    );
    // The two cells may train on different TRAIN: a data-size pair does.
    go(
        "bx.json",
        &[
            a[0],
            a[1],
            ("a3", &a3),
            ("b1", &x1),
            ("b2", &x2),
            ("b3", &x3),
        ],
    )
    .unwrap();
}

#[test]
fn pair_takes_a_third_seed() {
    let c = e2e_setup("three");
    let a1 = grade_cell(&c, "a1", &TGT);
    let a2 = grade_cell(&c, "a2", &TGT);
    let a3 = grade_cell(&c, "a3", &[5, 0, 0, 0, 0]);
    let b1 = grade_cell(&c, "b1", &[5, 0, 0, 0, 0]);
    let b2 = grade_cell(&c, "b2", &[5, 0, 0, 0, 0]);
    let b3 = grade_cell(&c, "b3", &[5, 0, 0, 0, 0]);
    let six = [
        ("a1", &a1),
        ("a2", &a2),
        ("a3", &a3),
        ("b1", &b1),
        ("b2", &b2),
        ("b3", &b3),
    ];
    let po = c.b.join("ab3.json");
    pair(&po, &six.map(|(k, p)| (k, p.as_path())), None).unwrap();
    let p = json_of(&po);
    let two = json!({"b": 4, "c": 0, "d": 4, "band": 4, "position": "inside"});
    let third = json!({"b": 0, "c": 0, "d": 0, "band": 4, "position": "inside"});
    assert_eq!(p["pooled"]["seeds"], json!([two, two, third]));
    assert_eq!(p["pooled"]["spread"], 4);
    assert_eq!(p["inputs_grade_sha256"]["a3"], sha(&a3.join("grade.json")));
    assert_eq!(p["inputs_grade_sha256"]["b3"], sha(&b3.join("grade.json")));
    // Table mode takes a3 alone; seed 3's (b, c) is its own grade's.
    let pt = c.b.join("a3t2.json");
    let three = [("a1", &a1), ("a2", &a2), ("a3", &a3)];
    pair(&pt, &three.map(|(k, p)| (k, p.as_path())), Some("t2")).unwrap();
    let t = json_of(&pt);
    let g3 = json_of(&a3.join("grade.json"));
    assert_eq!(t["pooled"]["seeds"][2]["b"], g3["pooled"]["vs_t2"]["b"]);
    assert_eq!(t["pooled"]["seeds"][2]["c"], g3["pooled"]["vs_t2"]["c"]);
    // A third seed is given to both cells or to neither.
    let five = |skip: &str| {
        six.iter()
            .filter(|(k, _)| *k != skip)
            .map(|(k, p)| (*k, p.as_path()))
            .collect::<Vec<_>>()
    };
    refused(
        pair(&c.b.join("r1.json"), &five("b3"), None),
        "a3= needs b3=",
    );
    refused(
        pair(&c.b.join("r2.json"), &five("a3"), None),
        "b3= needs a3=",
    );
    let tb = [("a1", &a1), ("a2", &a2), ("a3", &a3), ("b3", &b3)];
    refused(
        pair(
            &c.b.join("r3.json"),
            &tb.map(|(k, p)| (k, p.as_path())),
            Some("t2"),
        ),
        "b3= is for cell mode",
    );
}

#[test]
fn end_to_end_mix_grade_pair() {
    let c = e2e_setup("e2e");
    let a1 = grade_cell(&c, "a1", &TGT);
    let a2 = grade_cell(&c, "a2", &TGT);
    let b1 = grade_cell(&c, "b1", &[5, 0, 0, 0, 0]);
    let b2 = grade_cell(&c, "b2", &[5, 0, 0, 0, 0]);
    let g = json_of(&a1.join("grade.json"));
    assert_eq!(g["unigram_top1"], 5);
    assert_eq!(g["train"]["scored_targets"], 6);
    assert_eq!(g["train"]["bigram_keys_seen"], 3);
    let s = &g["splits"][0];
    assert_eq!(s["rows"], 5);
    assert_eq!(s["panel_scored_positions"], 5);
    assert_eq!(
        [
            &s["model_hits"],
            &s["writer_hits"],
            &s["unigram_hits"],
            &s["t2_hits"]
        ],
        [5, 1, 2, 3]
    );
    assert_eq!(s["vs_t2"], json!({"b": 2, "c": 0}));
    assert_eq!(s["vs_unigram"], json!({"b": 3, "c": 0}));
    assert_eq!([&s["content_bytes"], &s["panel_tokens"]], [24, 8]);
    let mut id = Sha256::new();
    for (p, t) in POS.iter().zip(TGT) {
        id.update((*p as u64).to_le_bytes());
        id.update(t.to_le_bytes());
    }
    assert_eq!(s["row_identity_sha256"], hex::encode(id.finalize()));
    assert_eq!(fs::read(a1.join("nearest-hits.bin")).unwrap(), [0x1f]);
    assert_eq!(fs::read(b1.join("nearest-hits.bin")).unwrap(), [0x01]);
    let po = c.b.join("ab.json");
    pair(
        &po,
        &[("a1", &a1), ("a2", &a2), ("b1", &b1), ("b2", &b2)],
        None,
    )
    .unwrap();
    let p = json_of(&po);
    let want = json!({"b": 4, "c": 0, "d": 4, "band": 4, "position": "inside"});
    assert_eq!(p["pooled"]["seeds"], json!([want, want]));
    assert_eq!(p["splits"][0]["verdict"], "inside");
    assert_eq!(p["inputs_grade_sha256"]["b2"], sha(&b2.join("grade.json")));
    let pt = c.b.join("at2.json");
    pair(&pt, &[("a1", &a1), ("a2", &a2)], Some("t2")).unwrap();
    let t = json_of(&pt);
    let want = json!({"b": 2, "c": 0, "d": 2, "band": 3, "position": "inside"});
    assert_eq!(t["pooled"]["seeds"][1], want);
    assert_eq!(t["pooled"]["spread"], 0);
}

#[test]
fn pair_refuses_existing_out() {
    let c = e2e_setup("pair_out");
    let a1 = grade_cell(&c, "a1", &TGT);
    fs::write(c.b.join("p.json"), b"{}").unwrap();
    refused(
        pair(&c.b.join("p.json"), &[("a1", &a1), ("a2", &a1)], Some("t2")),
        "already exists",
    );
}

#[test]
fn pair_refuses_hits_sha_mismatch() {
    let c = e2e_setup("pair_hits");
    let a1 = grade_cell(&c, "a1", &TGT);
    let a2 = grade_cell(&c, "a2", &TGT);
    fs::write(a2.join("nearest-hits.bin"), [0x1e]).unwrap();
    refused(
        pair(&c.b.join("p.json"), &[("a1", &a1), ("a2", &a2)], Some("t2")),
        "!= hits_sha256",
    );
}

#[test]
fn pair_refuses_row_identity_or_split_difference() {
    let c = e2e_setup("pair_id");
    let a1 = grade_cell(&c, "a1", &TGT);
    let rp = c.b.join("short.rows.json");
    rows_file(&rp, "p", &POS[..4], &TGT[..4], &TGT[..4], &TGT[..4]);
    let a2 = c.b.join("short");
    grade(&a2, &c.tr, &[("p", &c.pa)], &[("p", &rp)]).unwrap();
    refused(
        pair(&c.b.join("p.json"), &[("a1", &a1), ("a2", &a2)], Some("t2")),
        "pooled row_identity_sha256 differs from a1",
    );
    let rq = c.b.join("q.rows.json");
    rows_file(&rq, "q", &POS, &TGT, &TGT, &TGT);
    let aq = c.b.join("q");
    grade(&aq, &c.tr, &[("q", &c.pa)], &[("q", &rq)]).unwrap();
    refused(
        pair(&c.b.join("p.json"), &[("a1", &a1), ("a2", &aq)], Some("t2")),
        "pooled row_identity_sha256 differs from a1",
    );
}

#[test]
fn pair_refuses_seeds_with_different_train() {
    let c = e2e_setup("pair_train");
    let a1 = grade_cell(&c, "a1", &TGT);
    let (t, m) = (
        [0, 5, 7, 0, 5, 6, 0, 9, 9, 9],
        [0, 1, 1, 0, 1, 1, 0, 1, 1, 1],
    );
    let other = split(&c.b.join("other"), &t, &m, 4096, false);
    let rp = c.b.join("o.rows.json");
    rows_file(&rp, "p", &POS, &TGT, &TGT, &TGT);
    let a2 = c.b.join("o");
    grade(&a2, &other, &[("p", &c.pa)], &[("p", &rp)]).unwrap();
    refused(
        pair(&c.b.join("p.json"), &[("a1", &a1), ("a2", &a2)], Some("t2")),
        "share TRAIN",
    );
}

#[test]
fn pair_refuses_both_or_neither_b_and_table() {
    let c = e2e_setup("pair_mode");
    let a1 = grade_cell(&c, "a1", &TGT);
    let out = c.b.join("p.json");
    let all = [("a1", a1.as_path()), ("a2", &a1), ("b1", &a1), ("b2", &a1)];
    refused(pair(&out, &all, Some("t2")), "exactly one of");
    refused(pair(&out, &all[..2], None), "exactly one of");
    refused(pair(&out, &all[..3], None), "exactly one of");
    assert!(!out.exists());
}

#[test]
fn pair_refuses_a_bad_table_or_argument() {
    let c = e2e_setup("pair_args");
    let a1 = grade_cell(&c, "a1", &TGT);
    let out = c.b.join("p.json");
    let two = [("a1", a1.as_path()), ("a2", &a1)];
    refused(
        pair(&out, &two, Some("foo")),
        "table=foo: expected t2 or unigram",
    );
    let o = out.display().to_string();
    refused(
        run(&args("pair", &[("out", o.clone()), ("out", o.clone())])),
        "out= given more than once",
    );
    refused(
        run(&args("pair", &[("out", o), ("seed", "1".into())])),
        "unknown argument seed=",
    );
    assert!(!out.exists());
}

#[test]
fn pair_refuses_per_split_differences() {
    // The pooled identity is copied from a1, so only the split-by-split checks can refuse.
    type Edit = fn(&mut Value);
    let cases: [(Edit, &str); 3] = [
        (
            |v| v["splits"][1]["name"] = json!("r"),
            "split names/order differ from a1",
        ),
        (
            |v| v["splits"][0]["row_identity_sha256"] = json!("x"),
            "split p row_identity_sha256 differs from a1",
        ),
        (
            |v| {
                v["splits"][0]["rows"] = json!(4);
                v["splits"][1]["rows"] = json!(3);
            },
            "split p holds 4 rows, a1's 5",
        ),
    ];
    for (i, (edit, needle)) in cases.into_iter().enumerate() {
        let c = clean_setup(&format!("pair_split_{i}"));
        let [a1, a2, ..] = &c.cells;
        edit_json(&a2.join("grade.json"), edit);
        let out = c.b.join("p.json");
        refused(pair(&out, &[("a1", a1), ("a2", a2)], Some("t2")), needle);
    }
}

// ---- seen, the table bitsets and clean= pairing ----

/// Each part opened by a BOS; mask 1 except at each BOS.
fn docs(parts: &[&[u16]]) -> (Vec<u16>, Vec<u8>) {
    let mut t = Vec::new();
    for p in parts {
        t.push(0);
        t.extend_from_slice(p);
    }
    let m = t.iter().map(|&x| u8::from(x != 0)).collect();
    (t, m)
}

/// Documents at 0, 10, 15 of lengths 10, 5, 8: in-document 8-grams start at 0, 1, 2 and 15.
fn seen_train_tokens() -> (Vec<u16>, Vec<u8>) {
    docs(&[
        &[10, 11, 12, 13, 14, 15, 16, 17, 18],
        &[30, 31, 32, 33],
        &[34, 35, 36, 37, 38, 39, 40],
    ])
}

/// Documents at 0, 10, 20; 27 scored positions. Not short: 8, 9, 18, 19, 28, 29; of those
/// 8 and 9 have a TRAIN context (`[0, 10..=16]`, `[10..=17]`).
fn seen_panel(dir: &Path) -> Fx {
    let (t, m) = docs(&[
        &[10, 11, 12, 13, 14, 15, 16, 17, 18],
        &[1, 2, 3, 4, 5, 6, 7, 8, 9],
        &[30, 31, 32, 33, 34, 35, 36, 37, 50],
    ]);
    split(dir, &t, &m, 4096, true)
}

/// Split p classes: short, seen, seen, unseen, unseen. Split q: seen, unseen.
const SP_POS: [usize; 5] = [3, 8, 9, 19, 29];
const SP_TGT: [u32; 5] = [12, 17, 18, 9, 50];
const SQ_POS: [usize; 2] = [9, 19];
const SQ_TGT: [u32; 2] = [18, 9];

fn seen_cmd(
    out: &Path,
    train: &Fx,
    panels: &[(&str, &Fx)],
    rows: &[(&str, &Path)],
) -> Result<String, String> {
    inputs_cmd("seen", out, train, panels, rows)
}

fn edit_json(path: &Path, f: impl FnOnce(&mut Value)) {
    let mut v = json_of(path);
    f(&mut v);
    fs::write(path, serde_json::to_vec(&v).unwrap()).unwrap();
}

/// A miss writes nearest 0, which never equals a target (targets are never BOS).
fn near(tgt: &[u32], hits: &[u8]) -> Vec<u32> {
    tgt.iter()
        .zip(hits)
        .map(|(&t, &h)| if h == 1 { t } else { 0 })
        .collect()
}

struct Clean {
    b: PathBuf,
    tr: Fx,
    pa: Fx,
    rows: [PathBuf; 2],
    cells: [PathBuf; 4],
    seen: PathBuf,
}

/// Pooled row order p0..p4, q0, q1. Nearest hits a1 1011111, a2 0011001, b1 = b2 1110000;
/// t2 1110010, unigram (top-1 10) 0000000, excluded 1110010.
fn clean_setup(test: &str) -> Clean {
    let b = base(test);
    let (t, m) = seen_train_tokens();
    let tr = split(&b.join("train"), &t, &m, 4096, true);
    let pa = seen_panel(&b.join("panel"));
    let rows = [b.join("p.rows.json"), b.join("q.rows.json")];
    rows_file(&rows[0], "p", &SP_POS, &SP_TGT, &SP_TGT, &SP_TGT);
    rows_file(&rows[1], "q", &SQ_POS, &SQ_TGT, &SQ_TGT, &SQ_TGT);
    let seen = b.join("seen");
    let panels = [("p", &pa), ("q", &pa)];
    seen_cmd(&seen, &tr, &panels, &[("p", &rows[0]), ("q", &rows[1])]).unwrap();
    let hits: [(&str, [u8; 5], [u8; 2]); 4] = [
        ("a1", [1, 0, 1, 1, 1], [1, 1]),
        ("a2", [0, 0, 1, 1, 0], [0, 1]),
        ("b1", [1, 1, 1, 0, 0], [0, 0]),
        ("b2", [1, 1, 1, 0, 0], [0, 0]),
    ];
    let cells = hits.map(|(name, hp, hq)| {
        let (rp, rq) = (
            b.join(format!("{name}.p.json")),
            b.join(format!("{name}.q.json")),
        );
        rows_file(&rp, "p", &SP_POS, &SP_TGT, &near(&SP_TGT, &hp), &SP_TGT);
        rows_file(&rq, "q", &SQ_POS, &SQ_TGT, &near(&SQ_TGT, &hq), &SQ_TGT);
        let out = b.join(name);
        grade(&out, &tr, &panels, &[("p", &rp), ("q", &rq)]).unwrap();
        out
    });
    Clean {
        b,
        tr,
        pa,
        rows,
        cells,
        seen,
    }
}

/// A seen count object as rows, seen, short, unseen.
fn counts(v: &Value) -> [&Value; 4] {
    [&v["rows"], &v["seen"], &v["short"], &v["unseen"]]
}

/// `(b, c)` of both seeds of one assessment.
fn bc(v: &Value) -> [(u64, u64); 2] {
    [0, 1].map(|s| {
        let x = &v["seeds"][s];
        (x["b"].as_u64().unwrap(), x["c"].as_u64().unwrap())
    })
}

#[test]
fn train_ngrams_stay_inside_one_document() {
    let (t, _) = seen_train_tokens();
    let keys = train_ngrams(&t).unwrap();
    let mut want: Vec<u128> = [0usize, 1, 2, 15]
        .iter()
        .map(|&i| ngram_key(&t[i..i + NGRAM]))
        .collect();
    want.sort_unstable();
    assert_eq!(keys, want, "4 in-document runs; 16 if runs crossed a BOS");
    assert_eq!(ngram_key(&[1, 2, 4095]), (1 << 24) | (2 << 12) | 4095);
}

#[test]
fn seen_classifies_short_seen_and_unseen_rows() {
    let c = clean_setup("seen_classes");
    let s = json_of(&c.seen.join("seen.json"));
    assert_eq!(s["ngram"], 8);
    assert_eq!(s["train_ngrams_distinct"], 4);
    assert_eq!(counts(&s["splits"][0]), [5, 2, 1, 2]);
    assert_eq!(counts(&s["splits"][1]), [2, 1, 0, 1]);
    assert_eq!(counts(&s["pooled"]), [7, 3, 1, 3]);
    assert_eq!(s["splits"][0]["rows_file_sha256"], sha(&c.rows[0]));
    assert_eq!(fs::read(c.seen.join("excluded.bin")).unwrap(), [0x27]);
    assert_eq!(s["excluded_sha256"], sha(&c.seen.join("excluded.bin")));
    let g = json_of(&c.cells[0].join("grade.json"));
    assert_eq!(
        s["pooled"]["row_identity_sha256"],
        g["pooled"]["row_identity_sha256"]
    );
}

#[test]
fn seen_rows_all_takes_every_scored_position() {
    let c = clean_setup("seen_all");
    let out = c.b.join("seen-all");
    let all = Path::new("all");
    seen_cmd(&out, &c.tr, &[("p", &c.pa)], &[("p", all)]).unwrap();
    let s = json_of(&out.join("seen.json"));
    assert_eq!(counts(&s["pooled"]), [27, 2, 21, 4]);
    assert!(s["splits"][0]["rows_file_sha256"].is_null());
    // Rows 16, 17, 25, 26 (positions 18, 19, 28, 29) are unseen.
    assert_eq!(
        fs::read(out.join("excluded.bin")).unwrap(),
        [0xff, 0xff, 0xfc, 0x01]
    );
}

#[test]
fn grade_refuses_rows_all() {
    let c = clean_setup("grade_all");
    let out = c.b.join("g-all");
    let all = Path::new("all");
    refused(
        grade(&out, &c.tr, &[("p", &c.pa)], &[("p", all)]),
        "is for seen only",
    );
    assert!(!out.exists());
}

#[test]
fn grade_writes_table_bitsets() {
    let c = clean_setup("grade_bits");
    let a1 = &c.cells[0];
    let g = json_of(&a1.join("grade.json"));
    assert_eq!(g["unigram_top1"], 10);
    assert_eq!(fs::read(a1.join("nearest-hits.bin")).unwrap(), [0x7d]);
    assert_eq!(fs::read(a1.join("t2-hits.bin")).unwrap(), [0x27]);
    assert_eq!(fs::read(a1.join("unigram-hits.bin")).unwrap(), [0x00]);
    assert_eq!(g["t2_hits_file"], "t2-hits.bin");
    assert_eq!(g["t2_hits_sha256"], sha(&a1.join("t2-hits.bin")));
    assert_eq!(g["unigram_hits_file"], "unigram-hits.bin");
    assert_eq!(g["unigram_hits_sha256"], sha(&a1.join("unigram-hits.bin")));
    assert_eq!(g["pooled"]["vs_t2"], json!({"b": 3, "c": 1}));
}

#[test]
fn pair_clean_cell_mode_all_and_unseen() {
    let c = clean_setup("clean_cells");
    let [a1, a2, b1, b2] = &c.cells;
    let kv = [("a1", a1), ("a2", a2), ("b1", b1), ("b2", b2)];
    let plain = c.b.join("plain.json");
    pair(&plain, &kv.map(|(k, p)| (k, p.as_path())), None).unwrap();
    assert!(json_of(&plain).get("unseen").is_none());
    let out = c.b.join("ab.json");
    let mut kv: Vec<(&str, &Path)> = kv.iter().map(|(k, p)| (*k, p.as_path())).collect();
    kv.push(("clean", &c.seen));
    pair(&out, &kv, None).unwrap();
    let p = json_of(&out);
    assert_eq!(bc(&p["pooled"]), [(4, 1), (2, 2)]);
    assert_eq!(bc(&p["unseen"]["pooled"]), [(3, 0), (2, 0)]);
    assert_eq!(p["pooled"]["verdict"], "inside");
    assert_eq!(p["unseen"]["pooled"]["verdict"], "inside");
    let sp = |v: &Value| [bc(&v[0]), bc(&v[1])];
    assert_eq!(sp(&p["splits"]), [[(2, 1), (1, 2)], [(2, 0), (1, 0)]]);
    assert_eq!(
        sp(&p["unseen"]["splits"]),
        [[(2, 0), (1, 0)], [(1, 0), (1, 0)]]
    );
    assert_eq!(p["unseen"]["splits"][1]["name"], "q");
    assert_eq!(
        p["inputs_grade_sha256"]["clean_seen_sha256"],
        sha(&c.seen.join("seen.json"))
    );
}

#[test]
fn pair_clean_table_mode_all_and_unseen() {
    let c = clean_setup("clean_table");
    let kv = [
        ("a1", c.cells[0].as_path()),
        ("a2", &c.cells[1]),
        ("clean", &c.seen),
    ];
    for (t, all, unseen) in [
        ("t2", [(3, 1), (2, 3)], [(3, 0), (2, 0)]),
        ("unigram", [(6, 0), (3, 0)], [(3, 0), (2, 0)]),
    ] {
        let out = c.b.join(format!("{t}.json"));
        pair(&out, &kv, Some(t)).unwrap();
        let p = json_of(&out);
        assert_eq!(bc(&p["pooled"]), all, "{t}");
        assert_eq!(bc(&p["unseen"]["pooled"]), unseen, "{t}");
    }
}

#[test]
fn pair_clean_refuses_excluded_sha_mismatch() {
    let c = clean_setup("clean_sha");
    fs::write(c.seen.join("excluded.bin"), [0x26]).unwrap();
    let kv = [
        ("a1", c.cells[0].as_path()),
        ("a2", &c.cells[1]),
        ("clean", &c.seen),
    ];
    refused(
        pair(&c.b.join("p.json"), &kv, Some("t2")),
        "excluded_sha256",
    );
}

#[test]
fn pair_clean_refuses_identity_or_split_name() {
    let c = clean_setup("clean_id");
    let only_p = c.b.join("seen-p");
    seen_cmd(&only_p, &c.tr, &[("p", &c.pa)], &[("p", &c.rows[0])]).unwrap();
    let kv = |s: &Path| {
        [
            ("a1", c.cells[0].clone()),
            ("a2", c.cells[1].clone()),
            ("clean", s.into()),
        ]
    };
    let go = |s: &Path| {
        let kv = kv(s);
        let kv: Vec<(&str, &Path)> = kv.iter().map(|(k, p)| (*k, p.as_path())).collect();
        pair(&c.b.join("p.json"), &kv, Some("t2"))
    };
    refused(go(&only_p), "pooled row_identity_sha256");
    edit_json(&c.seen.join("seen.json"), |v| {
        v["splits"][1]["name"] = json!("r")
    });
    refused(go(&c.seen), "split names/order");
}

#[test]
fn pair_clean_refuses_other_train() {
    let c = clean_setup("clean_train");
    let (mut t, mut m) = seen_train_tokens();
    t.extend([0, 50, 51]);
    m.extend([0, 1, 1]);
    let tr2 = split(&c.b.join("train2"), &t, &m, 4096, true);
    let other = c.b.join("seen-other");
    let panels = [("p", &c.pa), ("q", &c.pa)];
    seen_cmd(
        &other,
        &tr2,
        &panels,
        &[("p", &c.rows[0]), ("q", &c.rows[1])],
    )
    .unwrap();
    let kv = [
        ("a1", c.cells[0].as_path()),
        ("a2", &c.cells[1]),
        ("clean", &other),
    ];
    refused(
        pair(&c.b.join("p.json"), &kv, Some("t2")),
        "train.tokens_sha256 differs from a1's grade",
    );
}

#[test]
fn pair_clean_refuses_excluded_length() {
    let c = clean_setup("clean_len");
    let ep = c.seen.join("excluded.bin");
    fs::write(&ep, [0x27, 0x00]).unwrap();
    let s = sha(&ep);
    edit_json(&c.seen.join("seen.json"), |v| {
        v["excluded_sha256"] = json!(s)
    });
    let kv = [
        ("a1", c.cells[0].as_path()),
        ("a2", &c.cells[1]),
        ("clean", &c.seen),
    ];
    refused(
        pair(&c.b.join("p.json"), &kv, Some("t2")),
        "excluded.bin length",
    );
}

#[test]
fn pair_table_refuses_older_grade() {
    let c = clean_setup("table_old");
    edit_json(&c.cells[0].join("grade.json"), |v| {
        let o = v.as_object_mut().unwrap();
        o.remove("t2_hits_file");
        o.remove("t2_hits_sha256");
    });
    let kv = [("a1", c.cells[0].as_path()), ("a2", &c.cells[1])];
    refused(pair(&c.b.join("p.json"), &kv, Some("t2")), "older grade");
}

#[test]
fn pair_table_refuses_grade_whose_counts_disagree() {
    let c = clean_setup("table_disagree");
    let kv = [("a1", c.cells[0].as_path()), ("a2", &c.cells[1])];
    edit_json(&c.cells[1].join("grade.json"), |v| {
        v["splits"][1]["vs_unigram"]["b"] = json!(0)
    });
    refused(
        pair(&c.b.join("u.json"), &kv, Some("unigram")),
        "from the hit bitsets",
    );
    edit_json(&c.cells[0].join("grade.json"), |v| {
        v["pooled"]["vs_t2"]["c"] = json!(0)
    });
    refused(
        pair(&c.b.join("t.json"), &kv, Some("t2")),
        "from the hit bitsets",
    );
}
