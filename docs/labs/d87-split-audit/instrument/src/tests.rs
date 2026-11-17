//! Fixture tests on small synthetic shards.

use super::*;
use std::collections::BTreeSet;

/// The split side of a prompt (destructured, not `.1`, which the D32 lexer reads as a float).
fn side(sp: &Splitter, first_user: &str) -> Split {
    let (_, split) = sp.split_of(first_user);
    split
}

/// The split key of a prompt.
fn key(sp: &Splitter, first_user: &str) -> [u8; 32] {
    let (digest, _) = sp.split_of(first_user);
    digest
}

/// GPT-2's byte-to-character table, as the tokenizer uses it.
fn byte_char(b: u8) -> char {
    let printable = |x: u8| (33..=126).contains(&x) || (161..=172).contains(&x) || x >= 174;
    if printable(b) {
        return b as char;
    }
    let n = (0u8..=255).filter(|&x| !printable(x) && x < b).count() as u32;
    char::from_u32(256 + n).unwrap()
}

/// A byte-level tokenizer with no merges: one token per text byte.
fn coder() -> Coder {
    coder_missing(&[])
}

/// The same tokenizer without the bytes in `missing`, which it then drops (claude-420).
fn coder_missing(missing: &[u8]) -> Coder {
    let mut vocab = serde_json::Map::new();
    for (i, name) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert(name.to_string(), json!(i));
    }
    for b in (0u16..256).filter(|&b| !missing.contains(&(b as u8))) {
        let id = vocab.len();
        vocab.insert(byte_char(b as u8).to_string(), json!(id));
    }
    let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate()
        .map(|(i, c)| json!({"id": i, "content": c, "special": true})).collect();
    let j = json!({"added_tokens": added,
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true, "use_regex": true},
        "model": {"type": "BPE", "vocab": vocab, "merges": []}});
    Coder::new(&serde_json::to_vec(&j).unwrap()).unwrap()
}

fn rec(id: &str, users: &[&str], turns: &[(&str, &str)]) -> String {
    let kind = if users.len() == 1 { "single" } else { "two-turn" };
    let msgs: Vec<Value> = users.iter().map(|u| json!({"role": "user", "content": u})).collect();
    let ts: Vec<Value> = turns.iter().map(|(r, a)| json!({"reasoning": r, "answer": a})).collect();
    json!({"id": id, "kind": kind, "source": "fx", "messages": msgs, "turns": ts}).to_string()
}

struct Dir(PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        // A counter too: two tests may build the same fixture at once.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("d87-split-test-{}-{name}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        Dir(d)
    }
    fn shard(&self, name: &str, lines: &[String]) -> PathBuf {
        let p = self.0.join(name);
        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
        if name.ends_with(".zst") {
            std::fs::write(&p, zstd::encode_all(text.as_bytes(), 3).unwrap()).unwrap();
        } else {
            std::fs::write(&p, text).unwrap();
        }
        p
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn kept(p: &Plan) -> BTreeMap<String, Split> {
    SPLITS.iter().flat_map(|&s| p.manifest(s).lines().map(move |i| (i.to_string(), s)).collect::<Vec<_>>()).collect()
}

#[test]
fn alternatives_of_one_prompt_stay_together() {
    let d = Dir::new("alt");
    let sp = Splitter::new(Config { dev_tune: (1, 4), dev_test: (1, 4), ..Config::default() }).unwrap();
    let (mut a, mut b) = (vec![], vec![]);
    for i in 0..200 {
        let prompt = format!("Prompt number {i} about things");
        for k in 0..3 {
            let line = rec(&format!("p{i}-a{k}"), &[&prompt], &[("think", &format!("alternative {k} answer for prompt {i}"))]);
            if k == 1 { b.push(line) } else { a.push(line) }
        }
        // The same first message, recased and repunctuated, as a two-turn record.
        b.push(rec(&format!("p{i}-t2"), &[&format!("PROMPT number {i}, about things!"), "More?"],
            &[("r", &format!("two-turn first answer {i}")), ("r", &format!("two-turn second answer {i}"))]));
    }
    let p = plan(&sp, &[d.shard("s0.jsonl.zst", &a), d.shard("s1.jsonl", &b)], 3).unwrap();
    let k = kept(&p);
    assert_eq!(k.len(), 800);
    let mut used = BTreeSet::new();
    for i in 0..200 {
        let s = k[&format!("p{i}-a0")];
        for id in [format!("p{i}-a1"), format!("p{i}-a2"), format!("p{i}-t2")] {
            assert_eq!(k[&id], s, "{id} left its prompt's split");
        }
        used.insert(s);
    }
    assert_eq!(used.len(), 3, "every split holds prompts at 1/4 + 1/4");
    let s = p.summary();
    let prompts: u64 = SPLITS.iter().map(|x| s["splits"][x.name()]["prompts"].as_u64().unwrap()).sum();
    assert_eq!((s["prompts"].as_u64(), prompts), (Some(200), 200));
}

#[test]
fn split_is_deterministic() {
    let d = Dir::new("det");
    let sp = Splitter::new(Config::default()).unwrap();
    let lines: Vec<String> = (0..300).map(|i| rec(&format!("r{i}"), &[&format!("question {i}")], &[("", &format!("reply {i}"))])).collect();
    let (x, y) = (d.shard("a.jsonl.zst", &lines[..150]), d.shard("b.jsonl.zst", &lines[150..]));
    let read = |out: &Path| -> Vec<u8> {
        let mut all = vec![];
        for f in ["train.ids", "dev-tune.ids", "dev-test.ids", "split-summary.json"] {
            all.extend(std::fs::read(out.join(f)).unwrap());
        }
        all
    };
    let (o1, o2) = (d.0.join("out1"), d.0.join("out2"));
    plan(&sp, &[x.clone(), y.clone()], 1).unwrap().write(&o1).unwrap();
    plan(&sp, &[y, x], 4).unwrap().write(&o2).unwrap();
    let (h1, h2) = (sha(&[&read(&o1)]), sha(&[&read(&o2)]));
    assert_eq!(h1, h2);
    // A different key moves prompts: the hash really depends on the assignment.
    let other = Splitter::new(Config { key: "other".into(), ..Config::default() }).unwrap();
    let moved = (0..300).filter(|i| side(&sp, &format!("question {i}")) != side(&other, &format!("question {i}"))).count();
    assert!(moved > 0);
}

#[test]
fn planted_duplicates_are_removed() {
    let d = Dir::new("dup");
    let sp = Splitter::new(Config::default()).unwrap();
    let long = "The capital of France is Paris, a city on the Seine.";
    let r1 = rec("r1", &["Where is Paris?"], &[("", long)]);
    let a = vec![r1.clone(), rec("r2", &["Is it?"], &[("", "yes")]), rec("r3", &["Was it?"], &[("", "yes")])];
    let b = vec![
        r1,                                                                  // exact copy
        rec("r1", &["Where is Paris?"], &[("", "Another answer entirely, long enough to count.")]), // repeated id
        rec("r4", &["Name France's capital."], &[("hm", "the CAPITAL of france is paris -- a city on the seine")]), // near
        rec("r5", &["Something else"], &[("", "A different and sufficiently long answer text.")]),
    ];
    let p = plan(&sp, &[d.shard("b.jsonl", &b), d.shard("a.jsonl", &a)], 2).unwrap();
    let k = kept(&p);
    assert_eq!(k.keys().cloned().collect::<Vec<_>>(), ["r1", "r2", "r3", "r5"]);
    let s = p.summary();
    assert_eq!(s["dropped"], json!({"exact": 1, "id": 1, "near": 1}));
    assert_eq!((s["lines"].as_u64(), s["records"].as_u64()), (Some(7), Some(4)));
}

/// A prompt text `"{stem} {n}{end}"` that lands in `want`.
fn prompt_in(sp: &Splitter, stem: &str, end: char, want: Split) -> String {
    (0..).map(|n| format!("{stem} {n}{end}")).find(|t| side(sp, t) == want).unwrap()
}

fn census(plant: bool) -> (Value, u64) {
    let d = Dir::new(if plant { "plant" } else { "clean" });
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let dev_user = prompt_in(&sp, "devq", '?', Split::DevTest);
    let dev_answer = if plant { "xhQWERTYUIOPLKJzv dev tail" } else { "xhQWERTYUIOPLKXzv dev tail" };
    let t1 = prompt_in(&sp, "trn one", '.', Split::Train);
    let t2 = prompt_in(&sp, "trn two", '.', Split::Train);
    let lines = vec![
        rec("dev", &[&dev_user], &[("", dev_answer)]),
        rec("t1", &[&t1], &[("", "moQWERTYUIOPLKJnp moQWERTYUIOPLKJnp train tail")]),
        rec("t2", &[&t2], &[("", "moQWERTYUIOPLKJns other words")]),
    ];
    let p = plan(&sp, &[d.shard("s.jsonl.zst", &lines)], 2).unwrap();
    assert_eq!(kept(&p)["dev"], Split::DevTest);
    // BOS + "\nUser:" + " " prompt + "\nAssistant:" + " " answer + EOS, one token per byte.
    let len = 1 + 6 + 1 + dev_user.len() + 11 + 1 + dev_answer.len() + 1;
    (audit(&p, &coder(), 2, 12).unwrap(), (len - (GRAM - 1)) as u64)
}

#[test]
fn planted_train_dev_gram_counts_once() {
    let (a, positions) = census(true);
    let t = &a["overlap"]["dev-test"];
    assert_eq!(t["positions"].as_u64(), Some(positions));
    assert_eq!(t["overlapped"].as_u64(), Some(1), "{a:#}");
    assert_eq!(t["overlap_by_last_token_segment"]["answer"][0], 1);
    assert_eq!(a["overlap"]["dev-tune"]["positions"].as_u64(), Some(0));
    assert_eq!(a["splits"]["train"]["records"].as_u64(), Some(2));
}

#[test]
fn dev_with_no_overlap_reports_zero() {
    let (a, positions) = census(false);
    assert_eq!(a["overlap"]["dev-test"]["overlap"], json!([0, positions]));
    assert_eq!(a["gate"]["verdict"], "PASS");
}

#[test]
fn packing_is_exact() {
    let base: Vec<u32> = (0..13).map(|i| 4095 - i).collect();
    let mut other = base.clone();
    other[12] = 4094;
    assert_ne!(pack(&base), pack(&other));
    let w = pack(&base);
    let back: Vec<u32> = (0..13).map(|i| (w[i / 5] >> (12 * (i % 5)) & 0xfff) as u32).collect();
    assert_eq!(back, base);
    assert_eq!(unpack(&w), base);
}

#[test]
#[should_panic(expected = "does not fit")]
fn packing_refuses_an_id_past_twelve_bits() {
    let mut ids = vec![1u32; 13];
    ids[3] = 4096;
    let _ = pack(&ids);
}

/// `len` letters from a fixed stride walk over `alphabet`.
fn letters(alphabet: &[u8], stride: usize, start: usize, len: usize) -> String {
    (0..len).map(|i| alphabet[(start + i * stride) % alphabet.len()] as char).collect()
}

/// Two DEV-test records: `da` holds a 50-token answer span that TRAIN holds, `db` a 49-token
/// one. The brackets around each span differ between DEV and TRAIN, so no span is longer.
fn span_fixture() -> (Value, Extras) {
    let d = Dir::new("span");
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let s50 = letters(b"abcdefghijklmnopqrstuvwxyz", 7, 3, 50);
    let s49 = letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 49);
    let lines = vec![
        rec("da", &[&prompt_in(&sp, "deva", '?', Split::DevTest)], &[("", &format!("<{s50}>"))]),
        rec("db", &[&prompt_in(&sp, "devb", '?', Split::DevTest)], &[("", &format!("({s49})"))]),
        rec("ta", &[&prompt_in(&sp, "trna", '.', Split::Train)], &[("", &format!("train [{s50}] side"))]),
        rec("tb", &[&prompt_in(&sp, "trnb", '.', Split::Train)], &[(&format!("{{{s49}}}"), "ok")]),
    ];
    let p = plan(&sp, &[d.shard("s.jsonl", &lines)], 2).unwrap();
    audit_full(&p, &coder(), 2, 12).unwrap()
}

#[test]
fn planted_50_token_span_contaminates_exactly_one_record() {
    let (a, x) = span_fixture();
    let by = &a["contamination"]["by_span"];
    assert_eq!(a["contamination"]["dev_test_records"], 2);
    assert_eq!(by["50"]["contaminated"], 1, "{a:#}");
    assert_eq!(x.contaminated[1], ["da"]);
    assert_eq!(a["gate"]["contaminated"], json!([1, 2]));
    assert_eq!(a["gate"]["verdict"], "FAIL");
    assert_eq!((by["100"]["contaminated"].as_u64(), by["100"]["records_with_a_span"].as_u64()), (Some(0), Some(0)));
}

#[test]
fn a_49_token_span_is_not_contamination_at_50() {
    let (a, x) = span_fixture();
    assert!(!x.contaminated[1].contains(&"db".to_string()));
    // The 49-token span is real: at n = 32 both records hold one.
    assert_eq!(x.contaminated[0], ["da", "db"]);
    assert_eq!(a["contamination"]["by_span"]["32"]["contaminated"], 2);
    // Both answers are mostly TRAIN 13-grams.
    assert_eq!(a["answer_13gram_records"]["records"], json!([2, 2]));
}

#[test]
fn novel_targets_leave_out_the_planted_gram() {
    let (clean, _) = census(false);
    let (plant, _) = census(true);
    let (c, p) = (&clean["novel_targets"]["all"], &plant["novel_targets"]["all"]);
    assert_eq!(c["novel"], c["with_13_context"]);
    assert_eq!(p["novel"].as_u64().unwrap() + 1, p["with_13_context"].as_u64().unwrap());
    let t = p["targets"].as_u64().unwrap();
    assert_eq!(p["with_13_context"].as_u64().unwrap() + p["short_context"].as_u64().unwrap(), t);
    assert_eq!(plant["answer_13gram_records"]["records"], json!([0, 1]));
}

#[test]
fn template_key_merges_numbers() {
    assert_eq!(template_key("What is 12+7?"), "what is # #");
    assert_eq!(template_key("What is 12+7?"), template_key("what is 3+5 ?"));
    let tpl = Splitter::new(Config { group: Group::Template, ..Config::default() }).unwrap();
    let prm = Splitter::new(Config::default()).unwrap();
    assert_eq!(key(&tpl, "What is 12+7?"), key(&tpl, "what is 3+5 ?"));
    assert_ne!(key(&prm, "What is 12+7?"), key(&prm, "what is 3+5 ?"));
    // The group survives the summary; the prompt group writes no field.
    assert_eq!(Splitter::from_config_json(&tpl.config_json()).unwrap().cfg, tpl.cfg);
    assert!(prm.config_json().get("group").is_none());
}

#[test]
fn template_resplit_is_deterministic() {
    let d = Dir::new("tpl");
    let sp = Splitter::new(Config { group: Group::Template, ..Config::default() }).unwrap();
    let words = ["a", "b", "c"];
    let lines: Vec<String> = (0..120)
        .map(|i| rec(&format!("q{i}"), &[&format!("What is {i}+{}?", i * 3 + 1)], &[("", &format!("it is {}", i * 4 + 1))]))
        .chain((0..120).map(|i| rec(&format!("w{i}"), &[&format!("word {} question", words[i % 3].repeat(i + 1))], &[("", "fine")])))
        .collect();
    let (x, y) = (d.shard("a.jsonl", &lines[..100]), d.shard("b.jsonl", &lines[100..]));
    let (o1, o2) = (d.0.join("o1"), d.0.join("o2"));
    let t1 = plan(&sp, &[x.clone(), y.clone()], 1).unwrap().write(&o1).unwrap();
    let t2 = plan(&sp, &[y.clone(), x.clone()], 3).unwrap().write(&o2).unwrap();
    assert_eq!(t1, t2);
    for f in ["train.ids", "dev-tune.ids", "dev-test.ids"] {
        assert_eq!(std::fs::read(o1.join(f)).unwrap(), std::fs::read(o2.join(f)).unwrap());
    }
    let k = kept(&plan(&sp, &[x, y], 2).unwrap());
    let s0 = k["q0"];
    assert!((0..120).all(|i| k[&format!("q{i}")] == s0), "one template, one split");
    assert_eq!(serde_json::from_str::<Value>(&t1).unwrap()["prompts"], 121);
}

/// The span classification fixture (claude-414), three DEV-test records over four shards:
/// `dc` holds span A, which one TRAIN record holds; `ds` holds B, which three TRAIN records in
/// three shards hold; `dr` holds C, which one TRAIN record holds three times.
fn class_fixture(threads: usize) -> (Value, Vec<TopSpan>) {
    let d = Dir::new("class");
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let a = letters(b"abcdefghijklmnopqrstuvwxyz", 7, 3, 50);
    let b = letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 50);
    let c = letters(b"0123456789", 3, 1, 50);
    let dev = |id: &str, stem: &str, s: &str| rec(id, &[&prompt_in(&sp, stem, '?', Split::DevTest)], &[("", &format!("<{s}>"))]);
    let trn = |id: &str, stem: &str, ans: String| rec(id, &[&prompt_in(&sp, stem, '.', Split::Train)], &[("", &ans)]);
    let s0 = vec![
        dev("dc", "devc", &a),
        dev("ds", "devs", &b),
        dev("dr", "devr", &c),
        trn("ta", "trna", format!("train a [{a}] side")),
    ];
    let s1 = vec![trn("tb1", "trnb one", format!("first [{b}] one"))];
    let s2 = vec![trn("tb2", "trnb two", format!("second [{b}] two")), trn("tr", "trnr", format!("[{c}] and [{c}] and [{c}]"))];
    let s3 = vec![trn("tb3", "trnb three", format!("third [{b}] three"))];
    let shards = [d.shard("s0.jsonl", &s0), d.shard("s1.jsonl", &s1), d.shard("s2.jsonl", &s2), d.shard("s3.jsonl", &s3)];
    let p = plan(&sp, &shards, threads).unwrap();
    assert_eq!(kept(&p).len(), 8, "no fixture record is a duplicate");
    let (_, x) = audit_classify(&p, &coder(), threads, 12, 10).unwrap();
    x.classify.unwrap()
}

/// The top-list entry whose tokens are `text`, one token per byte (id = byte + 3).
fn top_of<'a>(top: &'a [TopSpan], text: &str) -> &'a TopSpan {
    let ids: Vec<u32> = text.bytes().map(|b| b as u32 + 3).collect();
    top.iter().find(|t| t.tokens == ids).unwrap_or_else(|| panic!("{text} is not a shared span"))
}

#[test]
fn a_span_in_one_train_record_is_copied() {
    let (v, top) = class_fixture(2);
    let a = top_of(&top, &letters(b"abcdefghijklmnopqrstuvwxyz", 7, 3, 50));
    assert_eq!((a.train_records, a.copied, a.dev_records), (1, true, 1));
    assert_eq!(v["contaminated"], 3, "{v:#}");
    assert_eq!(v["copied_contaminated"], 2);
    assert_eq!(v["copied_share"], json!([2, 3]));
    assert_eq!(v["bar"]["verdict"], "SPAN-DEDUP");
}

#[test]
fn a_span_in_three_train_records_is_stock() {
    let (v, top) = class_fixture(2);
    let b = top_of(&top, &letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 50));
    assert_eq!((b.train_records, b.copied), (3, false));
    assert_eq!(v["stock_only"], 1);
    assert_eq!(v["shared_spans_by_class"], json!({"copied": 2, "stock": 1}));
    assert_eq!(v["most_train_records_of_a_span"], 3);
}

#[test]
fn a_span_repeated_in_one_train_record_counts_once() {
    let (v, top) = class_fixture(2);
    let c = top_of(&top, &letters(b"0123456789", 3, 1, 50));
    assert_eq!((c.train_records, c.copied), (1, true), "{v:#}");
    let want = json!({"1": 2, "2": 0, "3-9": 1, "10-99": 0, ">=100": 0});
    assert_eq!(v["span_multiplicity_histogram"], want);
    assert_eq!(v["record_least_multiplicity_histogram"], want);
}

/// A DEV-test record holding a copied span (one TRAIN record) and a stock span (three) is
/// copied-contaminated, and counts in the record histogram under its least multiplicity.
#[test]
fn a_record_with_a_copied_and_a_stock_span_is_copied() {
    let d = Dir::new("class-mixed");
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let a = letters(b"abcdefghijklmnopqrstuvwxyz", 7, 3, 50);
    let b = letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 50);
    let trn = |id: &str, stem: &str, ans: String| rec(id, &[&prompt_in(&sp, stem, '.', Split::Train)], &[("", &ans)]);
    let dm = rec("dm", &[&prompt_in(&sp, "devm", '?', Split::DevTest)], &[("", &format!("<{a}> then <{b}>"))]);
    let s0 = vec![dm, trn("ta", "trna", format!("train a [{a}] side")), trn("tb1", "trnb one", format!("first [{b}] one"))];
    let s1 = vec![trn("tb2", "trnb two", format!("second [{b}] two")), trn("tb3", "trnb three", format!("third [{b}] three"))];
    let shards = [d.shard("s0.jsonl", &s0), d.shard("s1.jsonl", &s1)];
    let p = plan(&sp, &shards, 2).unwrap();
    let (_, x) = audit_classify(&p, &coder(), 2, 12, 10).unwrap();
    let (v, _) = x.classify.unwrap();
    assert_eq!((&v["contaminated"], &v["copied_contaminated"]), (&json!(1), &json!(1)), "{v:#}");
    let want = json!({"1": 1, "2": 0, "3-9": 0, "10-99": 0, ">=100": 0});
    assert_eq!(v["record_least_multiplicity_histogram"], want);
}

#[test]
fn multiplicity_does_not_depend_on_thread_count() {
    let one = class_fixture(1);
    for threads in [2, 3, 4] {
        assert_eq!(class_fixture(threads), one, "threads = {threads}");
    }
    let (_, top) = one;
    let mut m: Vec<u32> = top.iter().map(|t| t.train_records).collect();
    m.sort_unstable();
    assert_eq!(m, [1, 1, 3]);
}

/// DEV-test `dd` holds a 100-token answer span; TRAIN `ta` holds all of it, `tb` only its first
/// 60 tokens, `tc` none of it (claude-416). With `second`, TRAIN `td` in the other shard also
/// holds all of it, inside different surrounding text.
fn drop_fixture(threads: usize, drop: &[&str], second: bool) -> Result<(Value, Extras), String> {
    let d = Dir::new("drop");
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let s = letters(b"abcdefghijklmnopqrstuvwxyz0123456789", 7, 3, 100);
    let trn = |id: &str, stem: &str, ans: String| rec(id, &[&prompt_in(&sp, stem, '.', Split::Train)], &[("", &ans)]);
    let s0 = vec![
        rec("dd", &[&prompt_in(&sp, "devd", '?', Split::DevTest)], &[("", &format!("<{s}>"))]),
        trn("ta", "trna", format!("train a [{s}] side")),
    ];
    let mut s1 = vec![trn("tb", "trnb", format!("train b [{}] side", &s[..60])), trn("tc", "trnc", "nothing shared here".into())];
    if second {
        let pad = letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 300);
        s1.push(trn("td", "trnd", format!("{pad} ({s}) {pad}")));
    }
    let p = plan(&sp, &[d.shard("s0.jsonl", &s0), d.shard("s1.jsonl", &s1)], threads).unwrap();
    assert_eq!(kept(&p).len(), 4 + usize::from(second), "no fixture record is a duplicate");
    let opts = AuditOptions { drop: drop.iter().map(|s| s.to_string()).collect(), find_drop: Some(DROP_SPAN), ..AuditOptions::default() };
    audit_opts(&p, &coder(), threads, 12, &opts)
}

#[test]
fn a_train_record_holding_a_100_token_span_is_listed_for_drop() {
    let (a, x) = drop_fixture(2, &[], false).unwrap();
    assert_eq!(x.drop_found, ["ta"], "{a:#}");
    assert_eq!(a["train_drop"]["found"]["records"], 1);
    assert_eq!(a["contamination"]["by_span"]["100"]["contaminated"], 1);
    assert_eq!(a["splits"]["train"]["records"], 3);
}

#[test]
fn the_drop_list_removes_the_100_token_contamination_only() {
    let (base, _) = drop_fixture(2, &[], false).unwrap();
    let (a, x) = drop_fixture(2, &["ta"], false).unwrap();
    let by = &a["contamination"]["by_span"];
    assert_eq!((&by["100"]["contaminated"], &by["50"]["contaminated"]), (&json!(0), &json!(1)), "{a:#}");
    assert!(x.drop_found.is_empty());
    assert_eq!(a["splits"]["train"]["records"], 2);
    assert_eq!(a["train_drop"]["applied"]["records"], 1);
    let dropped = a["train_drop"]["applied"]["tokens"].as_u64().unwrap();
    let before = base["splits"]["train"]["tokens"].as_u64().unwrap();
    assert_eq!(a["splits"]["train"]["tokens"].as_u64().unwrap() + dropped, before);
    assert_eq!(a["train_drop"]["applied_token_share"], json!([dropped, before]));
}

#[test]
fn a_drop_id_that_is_not_a_kept_train_record_is_refused() {
    for bad in [&["dd"][..], &["nope"], &["ta", "nope"]] {
        let Err(err) = drop_fixture(2, bad, false) else { panic!("{bad:?} was accepted") };
        assert!(err.contains("drop ids are kept TRAIN records"), "{bad:?}: {err}");
    }
}

#[test]
fn the_drop_list_does_not_depend_on_thread_count() {
    let (one, x1) = drop_fixture(1, &[], false).unwrap();
    for threads in [2, 3, 4] {
        let (a, x) = drop_fixture(threads, &[], false).unwrap();
        assert_eq!((a, x.drop_found), (one.clone(), x1.drop_found.clone()), "threads = {threads}");
    }
}

/// `holds` has no early skip: a second record holding a span another record already marked is
/// still listed, at every thread count (review LOW-1 on PR 4051).
#[test]
fn every_holder_of_a_shared_span_is_listed_for_drop() {
    for threads in [1, 2, 3, 4] {
        let (a, x) = drop_fixture(threads, &[], true).unwrap();
        assert_eq!(x.drop_found, ["ta", "td"], "threads = {threads}: {a:#}");
    }
}

/// DEV-test "dc" holds a 60-token answer span shared with `holders` TRAIN records (copied at 1
/// or 2); DEV-test "ds" holds one shared with three TRAIN records (stock) (claude-417).
fn copied_fixture(threads: usize, holders: usize, drop: &[&str], drop_dev: &[&str]) -> Result<(Value, Extras), String> {
    let ids = |l: &[&str]| l.iter().map(|s| s.to_string()).collect();
    let opts = AuditOptions { drop: ids(drop), drop_dev_test: ids(drop_dev), find_copied: drop.is_empty(), ..AuditOptions::default() };
    copied_audit(threads, holders, &opts)
}

/// The audit of `copied_fixture`'s records under any options.
fn copied_audit(threads: usize, holders: usize, opts: &AuditOptions) -> Result<(Value, Extras), String> {
    let d = Dir::new("copied");
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let c = letters(b"abcdefghijklmnopqrstuvwxyz0123456789", 7, 3, 60);
    let st = letters(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ", 11, 5, 60);
    let trn = |id: &str, ans: String| rec(id, &[&prompt_in(&sp, &format!("trn{id}"), '.', Split::Train)], &[("", &ans)]);
    let mut s0 = vec![
        rec("dc", &[&prompt_in(&sp, "devc", '?', Split::DevTest)], &[("", &format!("<{c}>"))]),
        rec("ds", &[&prompt_in(&sp, "devs", '?', Split::DevTest)], &[("", &format!("<{st}>"))]),
        trn("tz", "nothing shared here".into()),
    ];
    for h in 1..=holders {
        s0.push(trn(&format!("h{h}"), format!("train h{h} [{c}] side")));
    }
    let s1: Vec<String> = (1..=3).map(|k| trn(&format!("k{k}"), format!("train k{k} [{st}] side"))).collect();
    let p = plan(&sp, &[d.shard("s0.jsonl", &s0), d.shard("s1.jsonl", &s1)], threads).unwrap();
    assert_eq!(kept(&p).len(), 6 + holders, "no fixture record is a duplicate");
    audit_opts(&p, &coder(), threads, 12, opts)
}

#[test]
fn every_holder_of_a_copied_span_is_listed() {
    for (holders, want) in [(1, &["h1"][..]), (2, &["h1", "h2"])] {
        let (a, x) = copied_fixture(2, holders, &[], &[]).unwrap();
        assert_eq!(x.copied_found, want, "{a:#}");
        assert_eq!(a["train_drop"]["copied"]["records"], holders);
        assert_eq!(x.residue, ["dc", "ds"]);
    }
}

#[test]
fn no_holder_of_a_stock_span_is_listed() {
    let (a, x) = copied_fixture(2, 1, &[], &[]).unwrap();
    assert!(x.copied_found.iter().all(|id| !id.starts_with('k')), "{:?}: {a:#}", x.copied_found);
}

#[test]
fn both_lists_applied_leave_no_50_token_contamination() {
    let (base, x) = copied_fixture(2, 2, &[], &[]).unwrap();
    let drop: Vec<&str> = x.copied_found.iter().map(String::as_str).collect();
    let (_, y) = copied_fixture(2, 2, &drop, &[]).unwrap();
    assert_eq!(y.residue, ["ds"], "only the stock record stays contaminated");
    let residue: Vec<&str> = y.residue.iter().map(String::as_str).collect();
    let (a, z) = copied_fixture(2, 2, &drop, &residue).unwrap();
    assert_eq!(a["contamination"]["by_span"]["50"]["contaminated"], 0, "{a:#}");
    assert!(z.residue.is_empty());
    let before = base["splits"]["dev-test"]["records"].as_u64().unwrap();
    assert_eq!(a["splits"]["dev-test"]["records"].as_u64().unwrap() + residue.len() as u64, before);
    assert_eq!(a["contamination"]["dev_test_records"].as_u64().unwrap() + residue.len() as u64, before);
    assert_eq!(a["dev_test_drop"]["applied"]["records"], residue.len());
}

#[test]
fn a_dev_test_drop_id_that_is_not_a_kept_dev_test_record_is_refused() {
    for bad in [&["h1"][..], &["nope"], &["ds", "nope"]] {
        let Err(err) = copied_fixture(2, 1, &[], bad) else { panic!("{bad:?} was accepted") };
        assert!(err.contains("DEV-test drop ids are kept DEV-test records"), "{bad:?}: {err}");
    }
}

/// Multiplicity is counted over split v1, so the copied finder refuses a TRAIN drop list
/// (review L1 on PR 4053).
#[test]
fn the_copied_finder_refuses_a_train_drop_list() {
    let (_, x) = copied_fixture(2, 2, &[], &[]).unwrap();
    let drop: HashSet<String> = x.copied_found.into_iter().collect();
    let opts = AuditOptions { drop, find_copied: true, ..AuditOptions::default() };
    let Err(err) = copied_audit(2, 2, &opts) else { panic!("find_copied with a drop list was accepted") };
    assert!(err.contains("takes no TRAIN drop list"), "{err}");
}

#[test]
fn the_copied_and_residue_lists_do_not_depend_on_thread_count() {
    let (one, x1) = copied_fixture(1, 2, &[], &[]).unwrap();
    for threads in [2, 3, 4] {
        let (a, x) = copied_fixture(threads, 2, &[], &[]).unwrap();
        assert_eq!((a, x.copied_found, x.residue), (one.clone(), x1.copied_found.clone(), x1.residue.clone()), "threads = {threads}");
    }
}

// claude-420: the sealed per-split streams.

const SHARED: &str = "moQWERTYUIOPLKJnp";
const CLASS_NAMES: [&str; 3] = ["seen", "novel", "short_context"];

/// Three shards with every split present.
/// - TRAIN `t1` and DEV-test `x1` share the 19-token run " moQWERTYUIOPLKJnp ", so seven DEV-test
///   windows are in TRAIN.
/// - DEV-tune `u2` shares the 18-token run " moQWERTYUIOPLKJnp", so six are.
/// - TRAIN `t2` holds 中, whose first byte 0xe4 the stream coder cannot emit.
///
/// No DEV record has reasoning, every TRAIN prompt ends in '.', and no DEV answer starts with a
/// TRAIN answer's first letter (w, s, t). So no frame text makes another shared 13-gram, such as
/// "\nAssistant: t".
fn stream_shards(d: &Dir, sp: &Splitter) -> Vec<PathBuf> {
    let q = |stem: &str, end: char, s: Split| prompt_in(sp, stem, end, s);
    let (train, tune, test) = (Split::Train, Split::DevTune, Split::DevTest);
    vec![
        d.shard("a.jsonl", &[
            rec("t1", &[&q("ta", '.', train)], &[("why", &format!("train {SHARED} one"))]),
            rec("u1", &[&q("ua", '?', tune)], &[("", "one tuned answer")]),
            rec("x1", &[&q("xa", '?', test)], &[("", &format!("dev {SHARED} tail"))]),
        ]),
        d.shard("b.jsonl.zst", &[
            rec("t2", &[&q("tb", '.', train)], &[("", "second 中 record")]),
            rec("x2", &[&q("xb", '?', test)], &[("", "fresh words only here")]),
        ]),
        d.shard("c.jsonl", &[
            rec("t3", &[&q("tc", '.', train)], &[("", "third")]),
            rec("u2", &[&q("ub", '?', tune)], &[("", SHARED)]),
        ]),
    ]
}

fn stream_splitter() -> Splitter {
    Splitter::new(Config { dev_tune: (1, 3), dev_test: (1, 3), ..Config::default() }).unwrap()
}

fn stream_coder() -> Coder {
    coder_missing(&[0xe4])
}

/// The plan and `write_streams`' summaries of the stream fixture, written to `<d>/out`.
fn stream_run(d: &Dir, threads: usize) -> (Plan, Vec<Value>) {
    let sp = stream_splitter();
    let p = plan(&sp, &stream_shards(d, &sp), threads).unwrap();
    let v = streams::write_streams(&p, &stream_coder(), threads, 12, &d.0.join("out"), "fixture").unwrap();
    (p, v)
}

/// One split as claude-403's `Ingest` writer makes it from the same records in plan order.
fn ingest_reference(p: &Plan, c: &Coder, s: Split) -> [Vec<u8>; 3] {
    let mut ing = Ingest::new(Cursor::new(vec![]), vec![], vec![]).unwrap();
    for ((path, _), fate) in p.shards.iter().zip(&p.fate) {
        let mut i = 0usize;
        read_shard(path, &mut |line| {
            let f = fate[i];
            i += 1;
            if f == Fate::Keep(s) {
                ing.add(c, &parse_record(line)?)?;
            }
            Ok(())
        })
        .unwrap();
    }
    let (t, m, g, _) = ing.finish().unwrap();
    [t.into_inner(), m, g]
}

fn read_out(out: &Path, s: Split, ext: &str) -> Vec<u8> {
    std::fs::read(out.join(format!("{}.{ext}", s.name()))).unwrap()
}

#[test]
fn streams_equal_the_ingest_writer() {
    let d = Dir::new("streams-ref");
    let (p, v) = stream_run(&d, 2);
    let out = d.0.join("out");
    for (s, records) in SPLITS.iter().zip([3, 2, 2]) {
        let w = &v[s.ix()];
        assert_eq!(w["records"], records, "{}", s.name());
        let want = ingest_reference(&p, &stream_coder(), *s);
        for (k, ext) in ["tokens.u16", "mask.u8", "segment.u8"].iter().enumerate() {
            let got = read_out(&out, *s, ext);
            assert_eq!(got, want[k], "{} {ext}", s.name());
            assert_eq!(w["files"][k]["sha256"], hex(&sha(&[&got])), "{} {ext}", s.name());
        }
        assert_eq!(w["tokens"].as_u64().unwrap() * 2 + UORT_HEADER as u64, want[0].len() as u64);
    }
    assert!(!out.join("parts").exists());
    let table = std::fs::read(out.join("id-bytes.u8")).unwrap();
    assert_eq!((table.len(), &table[..3]), (4096, &[0u8, 0, 0][..]));
}

#[test]
fn streams_do_not_depend_on_shard_order_or_threads() {
    let d = Dir::new("streams-order");
    let (sp, c) = (stream_splitter(), stream_coder());
    let mut shards = stream_shards(&d, &sp);
    let p1 = plan(&sp, &shards, 1).unwrap();
    shards.reverse();
    let p3 = plan(&sp, &shards, 3).unwrap();
    let (o1, o3) = (d.0.join("o1"), d.0.join("o3"));
    streams::write_streams(&p1, &c, 1, 12, &o1, "fixture").unwrap();
    streams::write_streams(&p3, &c, 3, 12, &o3, "fixture").unwrap();
    let names = |o: &Path| {
        let mut n: Vec<_> = std::fs::read_dir(o).unwrap().map(|e| e.unwrap().file_name()).collect();
        n.sort();
        n
    };
    // id-bytes, four TRAIN files, five per DEV split.
    assert_eq!(names(&o1).len(), 15);
    assert_eq!(names(&o1), names(&o3));
    for n in names(&o1) {
        assert_eq!(std::fs::read(o1.join(&n)).unwrap(), std::fs::read(o3.join(&n)).unwrap(), "{n:?}");
    }
}

#[test]
fn novel_classes_match_the_audit_and_the_hand_count() {
    let d = Dir::new("streams-novel");
    let (p, v) = stream_run(&d, 2);
    let a = audit(&p, &stream_coder(), 2, 12).unwrap();
    let (n, all, resp) = (&v[Split::DevTest.ix()]["novel"]["classes"], &a["novel_targets"]["all"], &a["novel_targets"]["response_mask"]);
    assert_eq!(n["seen"]["targets"], 7, "{n:#}");
    assert_eq!(n["novel"]["targets"], all["novel"]);
    assert_eq!(n["short_context"]["targets"], all["short_context"]);
    assert_eq!(n["novel"]["response_targets"], resp["novel"]);
    let sum: u64 = CLASS_NAMES.iter().map(|k| n[k]["targets"].as_u64().unwrap()).sum();
    assert_eq!(json!(sum), all["targets"]);
    assert_eq!(v[Split::DevTest.ix()]["targets"], all["targets"]);
    assert_eq!(v[Split::DevTune.ix()]["novel"]["classes"]["seen"]["targets"], 6);
    assert_eq!(v[Split::Train.ix()]["novel"], json!(null));
    // By hand: per record, BOS is 3, the next 12 positions are 2, and the rest are 0 or 1.
    let out = d.0.join("out");
    for s in [Split::DevTune, Split::DevTest] {
        let nv = read_out(&out, s, "novel.u8");
        let toks = read_out(&out, s, "tokens.u16");
        let ids: Vec<u16> = toks[UORT_HEADER..].chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
        assert_eq!(ids.len(), nv.len());
        let starts: Vec<usize> = (0..ids.len()).filter(|&j| ids[j] == 0).collect();
        assert_eq!(starts.len(), 2);
        for (j, &k) in nv.iter().enumerate() {
            let e = j - starts.iter().rev().find(|&&b| b <= j).unwrap();
            match e {
                0 => assert_eq!(k, streams::FRAME, "{} position {j}", s.name()),
                e if e < GRAM => assert_eq!(k, streams::SHORT, "{} position {j}", s.name()),
                _ => assert!(k == streams::SEEN || k == streams::NOVEL, "{} position {j}", s.name()),
            }
        }
    }
}

#[test]
fn target_bytes_are_text_bytes_less_the_dropped() {
    let d = Dir::new("streams-bytes");
    let (_, v) = stream_run(&d, 2);
    let u = |w: &Value, k: &str| w[k].as_u64().unwrap();
    let t = &v[Split::Train.ix()];
    // 中 is e4 b8 ad: e4 is dropped, b8 and ad are one-byte tokens.
    assert_eq!(u(t, "dropped_bytes"), 1);
    assert_eq!(u(t, "target_bytes") + 1, u(t, "text_bytes"));
    let x = &v[Split::DevTest.ix()];
    assert_eq!((u(x, "dropped_bytes"), u(x, "target_bytes")), (0, u(x, "text_bytes")));
    // One token per byte, so the targets are the bytes plus one EOS per record.
    assert_eq!(u(x, "targets"), u(x, "target_bytes") + 2);
    // Response: " dev moQWERTYUIOPLKJnp tail" (27) and " fresh words only here" (22), plus each EOS.
    assert_eq!((u(x, "response_bytes"), u(x, "response_targets")), (49, 51));
}

#[test]
fn a_shard_inside_out_or_a_stale_parts_folder_is_refused() {
    let d = Dir::new("streams-refuse");
    let (sp, c) = (stream_splitter(), stream_coder());
    let p = plan(&sp, &stream_shards(&d, &sp), 2).unwrap();
    // The shards sit in `d`, so an --out of `d` itself, or of `d/.` , holds them.
    for out in [d.0.clone(), d.0.join(".")] {
        let e = streams::write_streams(&p, &c, 2, 12, &out, "fixture").unwrap_err();
        assert!(e.contains("is inside --out"), "{e}");
    }
    let out = d.0.join("out");
    std::fs::create_dir_all(out.join("parts")).unwrap();
    let e = streams::write_streams(&p, &c, 2, 12, &out, "fixture").unwrap_err();
    assert!(e.contains("parts exists"), "{e}");
}

#[test]
fn a_wrong_byte_table_is_refused() {
    let c = stream_coder();
    let t = tokenize(&c, &rec("r", &["hi there"], &[("", "an answer")])).unwrap();
    let mut table = streams::id_bytes(&c).unwrap();
    let text: u64 = t.seg_bytes.iter().sum();
    assert_eq!(streams::target_bytes(&t, &table), Ok(text));
    table[t.tokens[3] as usize] += 1;
    let e = streams::target_bytes(&t, &table).unwrap_err();
    assert!(e.contains("decode to"), "{e}");
}
