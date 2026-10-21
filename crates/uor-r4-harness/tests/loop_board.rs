//! `uor-harness loop board` pinned against a temporary git repository, GitHub replaced by the
//! test-only `UOR_LOOP_BOARD_GH_FAKE` file and the live tier by a scratch `UOR_HARNESS_LIVE`. Every
//! child process has every `GIT_*` variable removed, and the repository sits inside its temp dir.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const LABS: &str = r#"{"labs":{
  "antigravity":{"vendor":"Google","client":"Antigravity","headless":"agy -p","open_pr_limit":3},
  "claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3},
  "codex":{"vendor":"OpenAI","client":"Codex","headless":"codex exec","open_pr_limit":3},
  "deepseek":{"vendor":"DeepSeek","client":"DeepSeek harness","headless":"dsh","open_pr_limit":3},
  "kimi":{"vendor":"Moonshot AI","client":"Kimi Code","headless":"kimi -p","open_pr_limit":3,"role":"pm"}}}"#;

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_BOARD_GH_FAKE")
        .env_remove("UOR_HARNESS_LIVE")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
    live: PathBuf,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let root = base.join("repo");
        let live = base.join("live");
        assert!(root.starts_with(&base) && live.starts_with(&base));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { tmp, root, live };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.invalid"]);
        r.git(&["config", "user.name", "t"]);
        r.write("docs/harness/labs.json", LABS);
        r.write("docs/harness/claims.json", "{}\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        r
    }

    fn git(&self, args: &[&str]) {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        assert!(
            p.starts_with(&self.root),
            "write stays inside the temp repo"
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn write_live(&self, rel: &str, text: &str) {
        let p = self.live.join(rel);
        assert!(
            p.starts_with(&self.live),
            "write stays inside the scratch live tier"
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn fake_path(&self) -> PathBuf {
        self.tmp
            .path()
            .canonicalize()
            .unwrap()
            .join("board-fake.json")
    }

    /// `loop board` with `prs` as the fake open-PR list.
    fn board(&self, prs: &Value, json_out: bool) -> Output {
        std::fs::write(self.fake_path(), prs.to_string()).unwrap();
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        cmd.arg("--repo").arg(&self.root).args(["loop", "board"]);
        if json_out {
            cmd.arg("--json");
        }
        cmd.env("UOR_LOOP_BOARD_GH_FAKE", self.fake_path())
            .env("UOR_HARNESS_LIVE", &self.live)
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }
}

fn prs() -> Value {
    json!([
        {"number": 30, "title": "[kimi] plan", "headRefOid": "cccccccccccc", "isDraft": false},
        {"number": 12, "title": "[claude] loop board", "headRefOid": "aaaaaaaaaaaa", "isDraft": true},
        {"number": 20, "title": "fix a typo", "headRefOid": "bbbbbbbbbbbb", "isDraft": false},
        {"number": 15, "title": "[claude] loop queue", "headRefOid": "dddddddddddd", "isDraft": false}
    ])
}

fn stdout(o: &Output) -> String {
    assert!(o.status.success(), "loop board failed: {o:?}");
    String::from_utf8(o.stdout.clone()).unwrap()
}

/// The text block for `lab`: from its header line to the next unindented line.
fn block(text: &str, lab: &str) -> String {
    let mut lines = text
        .lines()
        .skip_while(|l| !l.starts_with(&format!("{lab} ")) && *l != lab);
    let mut out = vec![lines.next().expect("lab block present").to_string()];
    out.extend(lines.take_while(|l| l.starts_with(' ')).map(String::from));
    out.join("\n")
}

#[test]
fn five_labs_render_with_vendor_client_and_role() {
    let r = Repo::new();
    let text = stdout(&r.board(&json!([]), false));
    assert!(
        text.contains("FAKE GitHub: UOR_LOOP_BOARD_GH_FAKE="),
        "{text}"
    );
    for (lab, vendor, client) in [
        ("antigravity", "Google", "Antigravity"),
        ("claude", "Anthropic", "Claude Code"),
        ("codex", "OpenAI", "Codex"),
        ("deepseek", "DeepSeek", "DeepSeek harness"),
        ("kimi", "Moonshot AI", "Kimi Code"),
    ] {
        let b = block(&text, lab);
        assert!(b.contains(&format!("{vendor} / {client}")), "{b}");
    }
    assert!(block(&text, "kimi").contains("role: pm"), "{text}");
    assert!(block(&text, "claude").contains("role: -"), "{text}");
}

#[test]
fn tagged_prs_land_under_their_lab_and_untagged_apart() {
    let r = Repo::new();
    let text = stdout(&r.board(&prs(), false));
    let claude = block(&text, "claude");
    assert!(
        claude.contains("#12 draft aaaaaaa [claude] loop board"),
        "{claude}"
    );
    assert!(
        claude.contains("#15 ready ddddddd [claude] loop queue"),
        "{claude}"
    );
    assert!(
        claude.find("#12").unwrap() < claude.find("#15").unwrap(),
        "{claude}"
    );
    assert!(!claude.contains("#20"), "{claude}");
    assert!(
        block(&text, "kimi").contains("#30 ready ccccccc [kimi] plan"),
        "{text}"
    );
    let untagged = block(&text, "untagged");
    assert!(
        untagged.contains("#20 ready bbbbbbb fix a typo"),
        "{untagged}"
    );
    assert!(
        !untagged.contains("#12") && !untagged.contains("#30"),
        "{untagged}"
    );
    assert!(block(&text, "codex").contains("open PRs: none"), "{text}");
}

#[test]
fn presence_shows_state_doing_next_and_absence_shows_no_check_in() {
    let r = Repo::new();
    r.write_live(
        "presence/claude.json",
        r#"{"lab":"claude","updated":"2026-01-01T00:00:00Z","milestone":"m1","doing":"build board","next":"run gates"}"#,
    );
    let text = stdout(&r.board(&json!([]), false));
    let claude = block(&text, "claude");
    assert!(claude.contains("presence: STALE ("), "{claude}");
    assert!(claude.contains("[live]"), "{claude}");
    assert!(claude.contains("doing: build board"), "{claude}");
    assert!(claude.contains("next: run gates"), "{claude}");
    assert!(
        block(&text, "codex").contains("presence: no check-in"),
        "{text}"
    );
}

#[test]
fn claims_are_counted_per_lab_from_the_live_tier() {
    let r = Repo::new();
    r.write_live(
        "claims.json",
        r#"{"claude/a":{"lab":"claude","head":"x","claimed_at":"t","note":""},
            "claude/b":{"lab":"claude","head":"x","claimed_at":"t","note":""},
            "codex/c":{"lab":"codex","head":"x","claimed_at":"t","note":""}}"#,
    );
    let text = stdout(&r.board(&json!([]), false));
    assert!(block(&text, "claude").contains("claims: 2"), "{text}");
    assert!(block(&text, "codex").contains("claims: 1"), "{text}");
    assert!(block(&text, "kimi").contains("claims: 0"), "{text}");
}

#[test]
fn json_has_one_entry_per_lab() {
    let r = Repo::new();
    let out: Value = serde_json::from_str(&stdout(&r.board(&prs(), true))).unwrap();
    let labs = out["labs"].as_array().unwrap();
    let names: Vec<&str> = labs.iter().map(|l| l["lab"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        ["antigravity", "claude", "codex", "deepseek", "kimi"]
    );
    assert_eq!(out["fake_github"], json!(true));
    assert_eq!(labs[4]["role"], json!("pm"));
    assert_eq!(labs[1]["presence"], Value::Null);
    let nums: Vec<u64> = labs[1]["prs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["number"].as_u64().unwrap())
        .collect();
    assert_eq!(nums, [12, 15]);
    assert_eq!(labs[1]["prs"][0]["draft"], json!(true));
    assert_eq!(out["untagged"][0]["number"], json!(20));
}

/// The trait has one read method and nothing else: this complete implementation would stop
/// compiling if a write method were added without a default.
struct OnlyReads;
impl uor_r4_harness::loopboard::BoardGh for OnlyReads {
    fn open_prs(&self) -> Result<Vec<uor_r4_harness::loopboard::OpenPr>, String> {
        Ok(Vec::new())
    }
}

#[test]
fn the_board_makes_no_github_write() {
    let _ = uor_r4_harness::loopboard::BoardGh::open_prs(&OnlyReads);
    let r = Repo::new();
    r.write_live("claims/claude-1.json", "{\"lab\": \"claude\"}\n");
    let snapshot = |dir: &std::path::Path| {
        let mut files = std::collections::BTreeMap::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    files.insert(p.clone(), None);
                    stack.push(p);
                } else {
                    files.insert(p.clone(), Some(std::fs::read(&p).unwrap()));
                }
            }
        }
        files
    };
    let before = snapshot(&r.live);
    assert!(!before.is_empty());
    stdout(&r.board(&prs(), false));
    assert_eq!(snapshot(&r.live), before, "the live tier is unchanged");
    let st = clean(&mut Command::new("git"))
        .args(["status", "--porcelain"])
        .current_dir(&r.root)
        .output()
        .unwrap();
    assert!(st.stdout.is_empty(), "the checkout stays clean: {st:?}");
}

#[test]
fn an_unreadable_registry_exits_2() {
    let r = Repo::new();
    r.write("docs/harness/labs.json", "not json");
    let o = r.board(&json!([]), false);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
}
