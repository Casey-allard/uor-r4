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
    let mut vocab = serde_json::Map::new();
    for (i, name) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert(name.to_string(), json!(i));
    }
    for b in 0u16..256 {
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
