//! End-to-end: `generate`, resume, a torn `.part` shard, `seal` and `verify` against a
//! mock OpenAI-compatible server on the loopback address, through the functions `main` calls.

use m2_donor_corpus::{cmd_generate, cmd_seal, cmd_verify, Args};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const BASE_SEED: u64 = 77;

fn reply(body: &Value) -> Value {
    let msgs = body["messages"].as_array().unwrap();
    let last = msgs.last().unwrap()["content"].as_str().unwrap();
    let seed = body["seed"].as_u64().unwrap_or(0);
    let mut answer = format!("ans[{}] s{seed}: {last}", msgs.len());
    if last.contains("MARK") {
        answer.push_str(" <think>");
    }
    let finish = if last.contains("LEN") {
        "length"
    } else {
        "stop"
    };
    let mut message = json!({"role": "assistant", "content": answer});
    if !last.contains("NORS") {
        let field = if seed.is_multiple_of(2) {
            "reasoning_content"
        } else {
            "reasoning"
        };
        message[field] = json!(format!("why {last}"));
    }
    json!({"choices": [{"index": 0, "finish_reason": finish, "message": message}],
        "usage": {"completion_tokens": 10 + last.len(), "prompt_tokens": 5}})
}

/// What a mock server saw and how it answers: `status(n, body)` gives the HTTP status for
/// the n-th POST (200 answers with `reply`).
struct Mock {
    status: fn(usize, &Value) -> u16,
    count: AtomicUsize,
    seen: Mutex<Vec<(String, Value)>>,
}

fn ok_always(_: usize, _: &Value) -> u16 {
    200
}

fn serve(stream: TcpStream, mock: Arc<Mock>) {
    let mut r = BufReader::new(stream.try_clone().unwrap());
    let mut w = stream;
    loop {
        let mut len = 0usize;
        let mut head = String::new();
        loop {
            let mut line = String::new();
            if r.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
            if head.is_empty() {
                assert!(line.starts_with("POST /v1/chat/completions"), "{line}");
            }
            let l = line.to_ascii_lowercase();
            if let Some(v) = l.strip_prefix("content-length:") {
                len = v.trim().parse().unwrap();
            }
            head.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let n = mock.count.fetch_add(1, Ordering::SeqCst);
        let code = (mock.status)(n, &body);
        let out = if code == 200 {
            reply(&body).to_string()
        } else {
            "{}".to_string()
        };
        mock.seen.lock().unwrap().push((head, body));
        let proto = concat!("HTTP/1", ".", "1");
        let msg = format!(
            "{proto} {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{out}",
            out.len()
        );
        if w.write_all(msg.as_bytes()).is_err() {
            return;
        }
    }
}

fn mock_with(status: fn(usize, &Value) -> u16) -> (String, Arc<Mock>) {
    let mock = Arc::new(Mock {
        status,
        count: AtomicUsize::new(0),
        seen: Mutex::new(vec![]),
    });
    let l = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let addr = l.local_addr().unwrap();
    let m = mock.clone();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let m = m.clone();
            std::thread::spawn(move || serve(s, m));
        }
    });
    (format!("http://{addr}"), mock)
}

fn mock_server() -> String {
    let (url, _) = mock_with(ok_always);
    url
}

/// A fresh run root with a bank of `n` prompts.
fn setup(name: &str, n: usize) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let bank = root.join("bank.jsonl");
    fs::write(
        &bank,
        (0..n).map(|i| bank_line(i) + "\n").collect::<String>(),
    )
    .unwrap();
    (root.join("run"), bank, root)
}

fn gen_args(server: &str, bank: &Path, dir: &Path, extra: &[(&str, String)]) -> Args {
    let mut a = args(&[
        ("server", server.to_string()),
        ("prompts", bank.display().to_string()),
        ("out", dir.display().to_string()),
        ("budget", "1000000".into()),
        ("instrument-commit", "testcommit".into()),
        ("parallel", "1".into()),
    ]);
    a.extend(args(extra));
    a
}

const KEY: &str = "sk-test-0123456789abcdef";

/// Every file under `dir`, recursively.
fn all_files(dir: &Path) -> Vec<PathBuf> {
    let mut v = vec![];
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            v.extend(all_files(&p));
        } else {
            v.push(p);
        }
    }
    v
}

/// `openai` mode: no seed and no template kwargs in any body, the bearer header on every
/// request, the extra body merged, and the key in no file the run wrote.
#[test]
fn openai_mode_body_auth_and_no_key_on_disk() {
    let (dir, bank, root) = setup("e2e-openai", 6);
    let key_file = root.join("key.txt");
    fs::write(&key_file, format!("{KEY}\n")).unwrap();
    let extra = root.join("extra.json");
    fs::write(&extra, r#"{"thinking": {"type": "enabled"}}"#).unwrap();
    let (server, mock) = mock_with(ok_always);
    let seg = cmd_generate(&gen_args(
        &server,
        &bank,
        &dir,
        &[
            ("api", "openai".into()),
            ("api-key-file", key_file.display().to_string()),
            ("extra-body", extra.display().to_string()),
        ],
    ))
    .unwrap();
    assert_eq!(seg["stopped_by"], "exhausted");
    assert_eq!(
        (&seg["api"], &seg["auth"], &seg["base_seed"]),
        (&json!("openai"), &json!("file"), &Value::Null)
    );
    assert_eq!(seg["effort"], Value::Null);
    // The key file ends with a newline; the header carries the key without it.
    let auth = format!("authorization: bearer {}\r\n", KEY.to_ascii_lowercase());
    {
        let seen = mock.seen.lock().unwrap();
        assert!(seen.len() >= 6);
        for (head, body) in seen.iter() {
            let o = body.as_object().unwrap();
            assert!(!o.contains_key("seed") && !o.contains_key("chat_template_kwargs"));
            assert_eq!(body["thinking"], json!({"type": "enabled"}));
            assert!(head.to_ascii_lowercase().contains(&auth), "{head}");
        }
        mock.count.store(0, Ordering::SeqCst);
    }
    let (kept, _, _) = census(&dir);
    assert!(!kept.is_empty());
    for p in all_files(&dir) {
        let b = fs::read(&p).unwrap();
        assert!(
            !b.windows(KEY.len()).any(|w| w == KEY.as_bytes()),
            "the key is in {}",
            p.display()
        );
    }
    // The env form names the variable and is trimmed too; vllm mode records its seed.
    std::env::set_var("DONOR_TEST_KEY", format!(" {KEY}\n"));
    mock.seen.lock().unwrap().clear();
    let dir2 = root.join("run-env");
    let seg = cmd_generate(&gen_args(
        &server,
        &bank,
        &dir2,
        &[("api-key-env", "DONOR_TEST_KEY".into())],
    ))
    .unwrap();
    assert_eq!(
        (&seg["api"], &seg["auth"]),
        (&json!("vllm"), &json!("env:DONOR_TEST_KEY"))
    );
    assert_eq!(seg["base_seed"], 20261116);
    assert_eq!(seg["effort"], 50);
    let seen = mock.seen.lock().unwrap();
    assert!(!seen.is_empty());
    for (head, body) in seen.iter() {
        assert!(body["seed"].is_u64() && body["chat_template_kwargs"].is_object());
        assert!(head.to_ascii_lowercase().contains(&auth), "{head}");
    }
    drop(seen);
    fs::remove_dir_all(&root).unwrap();
}

/// Bad options are refused before any request, and no error carries the key.
#[test]
fn option_errors_are_refused() {
    let (dir, bank, root) = setup("e2e-options", 2);
    let (server, mock) = mock_with(ok_always);
    let bad_key = root.join("bad.txt");
    fs::write(&bad_key, "sk-has space\n").unwrap();
    std::env::set_var("DONOR_TEST_EMPTY", "");
    for extra in [
        vec![("api", "openai".to_string()), ("seed", "5".into())],
        vec![("api", "grpc".into())],
        vec![
            ("api-key-env", "DONOR_TEST_EMPTY".into()),
            ("api-key-file", bad_key.display().to_string()),
        ],
        vec![("api-key-env", "DONOR_TEST_EMPTY".into())],
        vec![("api-key-env", "DONOR_TEST_UNSET_XYZ".into())],
        vec![("api-key-file", bad_key.display().to_string())],
    ] {
        let e = cmd_generate(&gen_args(&server, &bank, &dir, &extra)).unwrap_err();
        assert!(!e.contains("sk-has"), "{e}");
    }
    assert_eq!(mock.count.load(Ordering::SeqCst), 0);
    fs::remove_dir_all(&root).unwrap();
}

/// 401 stops the segment as `refused` and drops nothing.
#[test]
fn unauthorized_stops_refused_without_dropping() {
    let (dir, bank, root) = setup("e2e-401", 4);
    let (server, mock) = mock_with(|_, _| 401);
    let seg = cmd_generate(&gen_args(&server, &bank, &dir, &[])).unwrap();
    assert_eq!(seg["stopped_by"], "refused");
    let (kept, dropped, _) = census(&dir);
    assert!(
        kept.is_empty() && dropped.is_empty(),
        "{kept:?} {dropped:?}"
    );
    assert_eq!(mock.count.load(Ordering::SeqCst), 1);
    fs::remove_dir_all(&root).unwrap();
}

/// A 429 is backed off and retried; the record is kept.
#[test]
fn rate_limit_then_ok_keeps_the_record() {
    let (dir, bank, root) = setup("e2e-429", 1);
    let (server, mock) = mock_with(|n, _| if n == 0 { 429 } else { 200 });
    let seg = cmd_generate(&gen_args(&server, &bank, &dir, &[])).unwrap();
    assert_eq!(seg["stopped_by"], "exhausted");
    let (kept, dropped, _) = census(&dir);
    assert_eq!((kept, dropped), (vec!["q00".to_string()], vec![]));
    // q00 is two-turn: one 429 then two answers.
    assert_eq!(mock.count.load(Ordering::SeqCst), 3);
    fs::remove_dir_all(&root).unwrap();
}

/// A 429 that outlasts the server wait leaves the prompt undone.
#[test]
fn rate_limit_past_the_wait_abandons() {
    let (dir, bank, root) = setup("e2e-429-wait", 2);
    let (server, _) = mock_with(|_, _| 429);
    let seg = cmd_generate(&gen_args(
        &server,
        &bank,
        &dir,
        &[("server-wait-seconds", "1".into())],
    ))
    .unwrap();
    assert_eq!(seg["stopped_by"], "server");
    let (kept, dropped, _) = census(&dir);
    assert!(kept.is_empty() && dropped.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

/// 32 consecutive 400s stop the segment as `refused` instead of dropping the whole bank.
#[test]
fn consecutive_bad_requests_stop_refused() {
    let (dir, bank, root) = setup("e2e-400", 40);
    let (server, _) = mock_with(|_, _| 400);
    let seg = cmd_generate(&gen_args(&server, &bank, &dir, &[])).unwrap();
    assert_eq!(seg["stopped_by"], "refused");
    let (kept, dropped, _) = census(&dir);
    assert_eq!((kept.len(), dropped.len()), (0, 32));
    fs::remove_dir_all(&root).unwrap();
}

fn bank_line(i: usize) -> String {
    let mut t0 = format!("p{i} first");
    let two = i.is_multiple_of(3);
    if i % 7 == 3 {
        t0.push_str(" LEN");
    }
    if i % 5 == 4 {
        t0.push_str(" NORS");
    }
    let mut msgs = vec![json!({"role": "user", "content": t0})];
    if two {
        msgs.push(json!({"role": "user", "content": format!("p{i} second")}));
    }
    if i % 11 == 5 {
        let last = msgs.last_mut().unwrap();
        last["content"] = json!(format!("{} MARK", last["content"].as_str().unwrap()));
    }
    let kind = if two { "two-turn" } else { "single" };
    // Spaced and role-first on purpose: records must copy these bytes exactly.
    let raw: Vec<String> = msgs
        .iter()
        .map(|m| format!("{{\"role\": \"user\", \"content\": {}}}", m["content"]))
        .collect();
    format!(
        "{{\"id\": \"q{i:02}\", \"kind\": \"{kind}\", \"source\": \"src{}\", \"messages\": [{}]}}",
        i % 2,
        raw.join(", ")
    )
}

fn args(pairs: &[(&str, String)]) -> Args {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

fn lines(p: &Path) -> Vec<Value> {
    fs::read_to_string(p)
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn files(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            n.starts_with(prefix) && n.ends_with(".jsonl")
        })
        .collect();
    v.sort();
    v
}

/// Ids in shards (checked against counts lines) and in dropped.jsonl.
fn census(dir: &Path) -> (Vec<String>, Vec<String>, u64) {
    let (mut kept, mut kept_tokens) = (vec![], 0u64);
    let shards = files(dir, "shard-");
    let counts = files(dir, "counts-");
    assert_eq!(shards.len(), counts.len());
    for (s, c) in shards.iter().zip(&counts) {
        let (sl, cl) = (lines(s), lines(c));
        assert_eq!(sl.len(), cl.len());
        for (r, k) in sl.iter().zip(&cl) {
            assert_eq!(r["id"], k["id"]);
            kept.push(r["id"].as_str().unwrap().to_string());
            kept_tokens += k["kept_tokens"].as_u64().unwrap();
        }
    }
    let dropped = lines(&dir.join("dropped.jsonl"))
        .iter()
        .map(|d| d["id"].as_str().unwrap().to_string())
        .collect();
    (kept, dropped, kept_tokens)
}

/// A server that stays down stops the segment with every prompt left undone, never dropped.
#[test]
fn server_down_stops_without_dropping() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("e2e-server-down");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let dir = root.join("run");
    let bank = root.join("bank.jsonl");
    fs::write(
        &bank,
        (0..4).map(|i| bank_line(i) + "\n").collect::<String>(),
    )
    .unwrap();
    // A port that was bound and released: nothing listens on it.
    let addr = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap();
    let seg = cmd_generate(&args(&[
        ("server", format!("http://{addr}")),
        ("prompts", bank.display().to_string()),
        ("out", dir.display().to_string()),
        ("budget", "1000000".into()),
        ("instrument-commit", "testcommit".into()),
        ("parallel", "2".into()),
        ("server-wait-seconds", "1".into()),
    ]))
    .unwrap();
    assert_eq!(seg["stopped_by"], "server");
    let (kept, dropped, _) = census(&dir);
    assert!(kept.is_empty(), "{kept:?}");
    assert!(dropped.is_empty(), "a down server dropped {dropped:?}");
}

#[test]
fn generate_resume_seal_verify_end_to_end() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("e2e-donor-corpus");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let dir = root.join("run");
    let bank = root.join("bank.jsonl");
    let bank_text: String = (0..40).map(|i| bank_line(i) + "\n").collect();
    fs::write(&bank, &bank_text).unwrap();
    let server = mock_server();
    let gen = |limit: usize| {
        cmd_generate(&args(&[
            ("server", server.clone()),
            ("prompts", bank.display().to_string()),
            ("out", dir.display().to_string()),
            ("budget", "1000000".into()),
            ("instrument-commit", "testcommit".into()),
            ("parallel", "4".into()),
            ("shard-records", "8".into()),
            ("seed", BASE_SEED.to_string()),
            ("limit", limit.to_string()),
        ]))
        .unwrap()
    };

    // Segment one: the first 20 prompts.
    let seg = gen(20);
    assert_eq!(seg["stopped_by"], "exhausted");
    let (kept, dropped, kept_tokens) = census(&dir);
    assert_eq!(kept.len() + dropped.len(), 20);
    let reasons: HashSet<String> = lines(&dir.join("dropped.jsonl"))
        .iter()
        .map(|d| d["reason"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(reasons, ["length", "markup"].map(String::from).into());
    for id in ["q03", "q05", "q10", "q16", "q17"] {
        assert!(dropped.contains(&id.to_string()), "{id} not dropped");
    }
    let metrics: Value =
        serde_json::from_slice(&fs::read(dir.join("metrics.json")).unwrap()).unwrap();
    assert_eq!(metrics["records"], kept.len());
    assert_eq!(metrics["kept_tokens"], kept_tokens);
    assert!(metrics["empty_reasoning_turns"].as_u64().unwrap() > 0);
    assert!(files(&dir, "shard-").len() >= 2);

    // Records copy bank bytes and carry both turns with the right seeds.
    let bank_lines: Vec<&str> = bank_text.lines().collect();
    let all: Vec<Value> = files(&dir, "shard-")
        .iter()
        .flat_map(|p| lines(p))
        .collect();
    let raw_shards: String = files(&dir, "shard-")
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    let q00 = all.iter().find(|r| r["id"] == "q00").unwrap();
    let t1 = q00["turns"][1]["answer"].as_str().unwrap();
    assert!(
        t1.starts_with(&format!("ans[3] s{}:", BASE_SEED + 1)),
        "{t1}"
    );
    let q12 = all.iter().find(|r| r["id"] == "q12").unwrap();
    let want = format!("ans[1] s{}: p12 first", BASE_SEED + 24);
    assert_eq!(q12["turns"][0]["answer"], want.as_str());
    assert!(raw_shards
        .contains(&bank_lines[12][bank_lines[12].find('[').unwrap()..bank_lines[12].len() - 1]));

    // Segment two resumes: nothing repeats and the totals add up.
    gen(40);
    let (kept, dropped, kept_tokens) = census(&dir);
    let ids: HashSet<&String> = kept.iter().chain(&dropped).collect();
    assert_eq!((kept.len() + dropped.len(), ids.len()), (40, 40));
    let metrics: Value =
        serde_json::from_slice(&fs::read(dir.join("metrics.json")).unwrap()).unwrap();
    assert_eq!(metrics["kept_tokens"], kept_tokens);
    assert_eq!(metrics["records"], kept.len());
    assert_eq!(lines(&dir.join("segments.jsonl")).len(), 2);

    // A torn `.part` pair: the last shard reopened, a torn record and an extra counts line.
    let last_shard = files(&dir, "shard-").pop().unwrap();
    let last_counts = files(&dir, "counts-").pop().unwrap();
    let (s_bytes, c_bytes) = (
        fs::read(&last_shard).unwrap(),
        fs::read(&last_counts).unwrap(),
    );
    let part = |p: &Path| PathBuf::from(format!("{}.part", p.display()));
    fs::rename(&last_shard, part(&last_shard)).unwrap();
    fs::rename(&last_counts, part(&last_counts)).unwrap();
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(part(&last_shard))
        .unwrap();
    f.write_all(b"{\"id\":\"q99\",\"ki").unwrap();
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(part(&last_counts))
        .unwrap();
    f.write_all(b"{\"id\":\"q99\",\"kept_tokens\":1,\"generated_tokens\":1,\"prompt_tokens\":1}\n")
        .unwrap();
    for _ in 0..2 {
        let seg = gen(40);
        assert_eq!(seg["records_at_end"], kept.len());
        assert!(!part(&last_shard).exists() && !part(&last_counts).exists());
        assert_eq!(fs::read(&last_shard).unwrap(), s_bytes);
        assert_eq!(fs::read(&last_counts).unwrap(), c_bytes);
    }

    // Seal and verify, then corrupt one shard byte.
    for (k, v) in [
        ("model", "{\"name\": \"m\"}"),
        ("machine", "{\"gpus\": 8}"),
        ("engine", "{\"vllm\": \"x\"}"),
    ] {
        fs::write(root.join(format!("{k}.json")), v).unwrap();
    }
    let seal_args: Vec<(&str, String)> = vec![
        ("dir", dir.display().to_string()),
        ("prompts", bank.display().to_string()),
        ("model", root.join("model.json").display().to_string()),
        ("machine", root.join("machine.json").display().to_string()),
        ("engine", root.join("engine.json").display().to_string()),
    ];
    cmd_seal(&args(&seal_args)).unwrap();
    let ver = args(&seal_args[..2]);
    let v = cmd_verify(&ver).unwrap();
    assert_eq!(v["verified"], true);
    assert_eq!(v["records"], kept.len());
    assert_eq!(v["kept_tokens"], kept_tokens);
    let first = files(&dir, "shard-").remove(0);
    let mut b = fs::read(&first).unwrap();
    b[20] ^= 1;
    fs::write(&first, &b).unwrap();
    assert!(cmd_verify(&ver).is_err());
    fs::remove_dir_all(&root).unwrap();
}
