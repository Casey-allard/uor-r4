//! `uor-review v1` records, setup-PR status, the D59 switch and the review gate (loop PR 6),
//! through the pure functions in `uor_r4_harness::loopreview`.

use std::collections::BTreeMap;
use uor_r4_harness::loopreview::*;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_HEAD: &str = "fedcba9876543210fedcba9876543210fedcba98";
const PR: u64 = 2926;
const LOOP_PAGE: &str = include_str!("../../../docs/loop/README.md");

fn vendors() -> BTreeMap<String, String> {
    [
        ("claude", "anthropic"),
        ("codex", "openai"),
        ("kimi", "moonshot"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

fn rows() -> Vec<String> {
    build_rows(LOOP_PAGE)
}

fn record(lab: &str, verdict: Verdict) -> Record {
    Record {
        lab: lab.to_string(),
        pr: PR,
        head: HEAD.to_string(),
        model: "unknown".to_string(),
        launch: "manual".to_string(),
        verdict,
        order: "docs/loop/tasks/infra-loop-harness/claude-18/ORDER.md".to_string(),
        fresh_context: false,
        done_when: DoneWhen::Host("cargo test -p uor-r4-harness: ok".to_string()),
    }
}

fn body(lab: &str, verdict: Verdict) -> String {
    render(&record(lab, verdict))
}

/// The record with `context: fresh`, as D59 asks of claude on a setup PR.
fn fresh(lab: &str, verdict: Verdict) -> String {
    render(&Record {
        fresh_context: true,
        ..record(lab, verdict)
    })
}

const ORDER_PATH: &str = "docs/loop/tasks/infra-loop-harness/claude-18/ORDER.md";

/// A gate run over `comments` for a PR built by `order_lab` with item `item`.
struct Case<'a> {
    order_lab: &'a str,
    item: &'a str,
    setup_row: Option<&'a str>,
    d59: bool,
    doc_only: bool,
}

const CODE_PR: Case = Case {
    order_lab: "claude",
    item: "infra-other",
    setup_row: None,
    d59: true,
    doc_only: false,
};

const SETUP_PR: Case = Case {
    order_lab: "claude",
    item: "infra-loop-pr06-review-records",
    setup_row: Some("6"),
    d59: true,
    doc_only: false,
};

fn run(c: &Case, comments: &[String]) -> GateOutcome {
    let (rows, vendors) = (rows(), vendors());
    gate(&GateInput {
        pr: PR,
        head: HEAD,
        order_path: ORDER_PATH,
        order_lab: c.order_lab,
        order_item: c.item,
        setup_row: c.setup_row,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: c.d59,
        doc_only: c.doc_only,
        comments,
    })
}

fn has(out: &GateOutcome, text: &str) -> bool {
    out.details.iter().any(|d| d.contains(text))
}

#[test]
fn planted_fault_an_untagged_record_is_refused() {
    let tagged = body("codex", Verdict::Approve);
    let untagged = tagged.replacen("[codex] uor-review v1", "uor-review v1", 1);
    match parse(&untagged) {
        Parsed::Malformed(why) => assert!(why.contains("untagged"), "{why}"),
        other => panic!("expected Malformed, got {other:?}"),
    }
    let out = run(&CODE_PR, &[untagged]);
    assert!(!out.pass, "{:?}", out.details);
    assert!(
        has(&out, "refused, malformed record: untagged"),
        "{:?}",
        out.details
    );

    let mismatched = tagged.replacen("[codex] uor-review v1", "[kimi] uor-review v1", 1);
    match parse(&mismatched) {
        Parsed::Malformed(why) => assert!(why.contains("differs from the lab"), "{why}"),
        other => panic!("expected Malformed, got {other:?}"),
    }
    let out = run(&CODE_PR, &[mismatched]);
    assert!(!out.pass);
    assert!(has(&out, "the tag [kimi] differs"), "{:?}", out.details);

    // The same record, tagged, counts: the refusal is the tag alone.
    assert!(run(&CODE_PR, &[tagged]).pass);
}

#[test]
fn planted_fault_a_same_lab_record_without_fresh_context_is_refused() {
    let out = run(&CODE_PR, &[body("claude", Verdict::Approve)]);
    assert!(!out.pass);
    assert!(!out.setup_pr);
    assert!(
        has(&out, "needs `context: fresh` (D66)"),
        "{:?}",
        out.details
    );
    assert!(run(&CODE_PR, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn another_lab_of_the_order_labs_vendor_is_still_refused_d66() {
    // D66 counts the ORDER lab's own fresh record, not every lab of its vendor.
    let rows = rows();
    let mut vendors = vendors();
    vendors.insert("anthro".to_string(), "anthropic".to_string());
    let comments = [fresh("anthro", Verdict::Approve)];
    let out = gate(&GateInput {
        pr: PR,
        head: HEAD,
        order_path: ORDER_PATH,
        order_lab: CODE_PR.order_lab,
        order_item: CODE_PR.item,
        setup_row: CODE_PR.setup_row,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: CODE_PR.d59,
        doc_only: CODE_PR.doc_only,
        comments: &comments,
    });
    assert!(!out.pass);
    assert!(
        has(
            &out,
            "is neither the ORDER lab claude nor of another vendor"
        ),
        "{:?}",
        out.details
    );
}

#[test]
fn planted_fault_a_non_claude_record_on_a_setup_pr_is_refused() {
    let out = run(&SETUP_PR, &[body("codex", Verdict::Approve)]);
    assert!(out.setup_pr);
    assert!(!out.pass);
    assert!(
        has(
            &out,
            "setup PR under D59 counts only claude records, not codex"
        ),
        "{:?}",
        out.details
    );
    let out = run(
        &SETUP_PR,
        &[
            body("codex", Verdict::Approve),
            fresh("claude", Verdict::Approve),
        ],
    );
    assert!(out.pass, "{:?}", out.details);
    assert!(has(&out, "[claude] APPROVE"), "{:?}", out.details);
    // A later CHANGES from another lab does not count, so it does not block.
    let out = run(
        &SETUP_PR,
        &[
            fresh("claude", Verdict::Approve),
            body("codex", Verdict::Changes),
        ],
    );
    assert!(out.pass, "{:?}", out.details);
    assert!(has(&out, "[codex] CHANGES"), "{:?}", out.details);
}

#[test]
fn a_loop_item_without_a_setup_row_counts_no_record() {
    let c = Case {
        item: "infra-loop-pr06-review-records",
        setup_row: None,
        ..CODE_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
    assert!(has(&out, "needs a valid `Setup row:`"), "{:?}", out.details);
    assert!(!run(&c, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn a_setup_row_not_in_the_table_is_not_a_setup_pr() {
    let c = Case {
        setup_row: Some("99"),
        ..SETUP_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
    assert!(has(&out, "needs a valid `Setup row:`"), "{:?}", out.details);
    assert!(!run(&c, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn the_item_must_be_the_harness_item_or_the_row_s_own() {
    let rows = rows();
    assert!(!is_setup_pr(Some("6"), &rows, "infra-loop-pr07b-session"));
    assert!(is_setup_pr(Some("6"), &rows, "infra-loop-harness"));
    assert!(is_setup_pr(
        Some("6"),
        &rows,
        "infra-loop-pr06-review-records"
    ));
    assert!(!is_setup_pr(None, &rows, "infra-loop-harness"));
    let c = Case {
        item: "infra-loop-pr07b-session",
        ..SETUP_PR
    };
    let out = run(&c, &[body("claude", Verdict::Approve)]);
    assert!(!out.setup_pr);
    assert!(!out.pass);
}

#[test]
fn a_fresh_same_lab_record_counts_on_any_pr_d66() {
    let doc = Case {
        doc_only: true,
        ..CODE_PR
    };
    let fresh = render(&Record {
        fresh_context: true,
        ..record("claude", Verdict::Approve)
    });
    assert!(fresh.contains("\ncontext: fresh\n"));
    let out = run(&doc, std::slice::from_ref(&fresh));
    assert!(out.pass, "{:?}", out.details);

    let out = run(&doc, &[body("claude", Verdict::Approve)]);
    assert!(!out.pass);
    assert!(has(&out, "needs `context: fresh`"), "{:?}", out.details);

    // D66: the review is part of the unit, so the same fresh record counts on a code PR too.
    let out = run(&CODE_PR, &[fresh]);
    assert!(out.pass, "{:?}", out.details);
    assert!(!out.setup_pr);

    let out = run(&CODE_PR, &[body("claude", Verdict::Approve)]);
    assert!(!out.pass);
    assert!(
        has(&out, "needs `context: fresh` (D66)"),
        "{:?}",
        out.details
    );
}

#[test]
fn the_last_counted_record_decides() {
    let approve = body("codex", Verdict::Approve);
    let changes = body("codex", Verdict::Changes);
    let out = run(&CODE_PR, &[approve.clone(), changes.clone()]);
    assert!(!out.pass);
    assert!(
        has(&out, "fail: a counted CHANGES from codex stands"),
        "{:?}",
        out.details
    );
    assert!(run(&CODE_PR, &[changes, approve]).pass);

    let stale = render(&Record {
        head: OTHER_HEAD.to_string(),
        ..record("codex", Verdict::Approve)
    });
    let out = run(&CODE_PR, &[stale]);
    assert!(!out.pass);
    assert!(has(&out, "not the current head"), "{:?}", out.details);

    let elsewhere = render(&Record {
        pr: PR + 1,
        ..record("codex", Verdict::Approve)
    });
    let out = run(&CODE_PR, &[elsewhere]);
    assert!(!out.pass);
    assert!(has(&out, "not PR 2926"), "{:?}", out.details);

    let out = run(&CODE_PR, &["LGTM".to_string()]);
    assert!(!out.pass);
    assert!(has(&out, "fail: no counted record"), "{:?}", out.details);
}

/// `standing_changes` over the same input as [`run`].
fn standing(c: &Case, comments: &[String]) -> Vec<String> {
    let (rows, vendors) = (rows(), vendors());
    standing_changes(&GateInput {
        pr: PR,
        head: HEAD,
        order_path: ORDER_PATH,
        order_lab: c.order_lab,
        order_item: c.item,
        setup_row: c.setup_row,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: c.d59,
        doc_only: c.doc_only,
        comments,
    })
}

/// Planted fault: another lab's later APPROVE at the same head clears a counted CHANGES. Owner,
/// 22 Oct 2026 17:51Z ("No: CHANGES blocks"): it stands until a new head or the same lab's own
/// later APPROVE.
#[test]
fn planted_fault_another_lab_s_approve_does_not_clear_a_counted_changes() {
    let codex_changes = body("codex", Verdict::Changes);
    let kimi_approve = body("kimi", Verdict::Approve);
    let codex_approve = body("codex", Verdict::Approve);

    let comments = [codex_changes.clone(), kimi_approve.clone()];
    let out = run(&CODE_PR, &comments);
    assert!(
        !out.pass,
        "another lab's APPROVE cleared a counted CHANGES: {:?}",
        out.details
    );
    assert!(
        has(&out, "fail: a counted CHANGES from codex stands"),
        "{:?}",
        out.details
    );
    assert_eq!(standing(&CODE_PR, &comments), ["codex"]);

    // The same lab's own later APPROVE clears it.
    let comments = [codex_changes, kimi_approve.clone(), codex_approve];
    assert!(run(&CODE_PR, &comments).pass);
    assert!(standing(&CODE_PR, &comments).is_empty());

    // A new head clears it: a record counts only at the current head.
    let stale = render(&Record {
        head: OTHER_HEAD.to_string(),
        ..record("codex", Verdict::Changes)
    });
    assert!(run(&CODE_PR, &[stale, kimi_approve]).pass);
}

/// `last_counted_approve` over the same input as [`run`].
fn approve_at(c: &Case, comments: &[String]) -> Option<usize> {
    let (rows, vendors) = (rows(), vendors());
    last_counted_approve(&GateInput {
        pr: PR,
        head: HEAD,
        order_path: ORDER_PATH,
        order_lab: c.order_lab,
        order_item: c.item,
        setup_row: c.setup_row,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: c.d59,
        doc_only: c.doc_only,
        comments,
    })
}

/// A loop item whose ORDER names no valid setup row counts no record, so a claude APPROVE gives
/// no standing approval.
#[test]
fn last_counted_approve_is_none_for_a_loop_item_without_a_valid_setup_row() {
    let approve = fresh("claude", Verdict::Approve);
    for setup_row in [None, Some("99")] {
        let c = Case {
            item: "infra-loop-pr06-review-records",
            setup_row,
            ..SETUP_PR
        };
        assert_eq!(approve_at(&c, std::slice::from_ref(&approve)), None);
    }
    assert_eq!(
        approve_at(&SETUP_PR, std::slice::from_ref(&approve)),
        Some(0)
    );
}

/// The index of the last counted record when it is an APPROVE; records that do not count and
/// plain comments are skipped; a last counted CHANGES or no counted record gives `None`.
#[test]
fn last_counted_approve_finds_the_standing_approval() {
    let approve = body("codex", Verdict::Approve);
    let changes = body("codex", Verdict::Changes);
    let note = "LGTM".to_string();
    assert_eq!(
        approve_at(&CODE_PR, std::slice::from_ref(&approve)),
        Some(0)
    );
    assert_eq!(
        approve_at(
            &CODE_PR,
            &[changes.clone(), note.clone(), approve.clone(), note.clone()]
        ),
        Some(2)
    );
    assert_eq!(
        approve_at(&CODE_PR, &[approve.clone(), changes.clone()]),
        None
    );
    assert_eq!(approve_at(&CODE_PR, std::slice::from_ref(&note)), None);
    // The ORDER lab's CHANGES without `context: fresh` does not count, so the APPROVE before it
    // stands; with `context: fresh` it counts (D66) and blocks.
    let same = body("claude", Verdict::Changes);
    assert_eq!(approve_at(&CODE_PR, &[approve.clone(), same]), Some(0));
    let same_fresh = fresh("claude", Verdict::Changes);
    assert_eq!(approve_at(&CODE_PR, &[approve, same_fresh]), None);
    // On a setup PR under D59, a later kimi CHANGES does not count; a later claude CHANGES does.
    let claude_ok = fresh("claude", Verdict::Approve);
    let kimi_no = fresh("kimi", Verdict::Changes);
    let claude_no = fresh("claude", Verdict::Changes);
    assert_eq!(
        approve_at(&SETUP_PR, &[claude_ok.clone(), kimi_no]),
        Some(0)
    );
    assert_eq!(approve_at(&SETUP_PR, &[claude_ok, claude_no]), None);
}

#[test]
fn the_switch_ends_d59_only_when_listed_and_ended() {
    let unlisted = r#"{"schema":1,"paths":[{"path":"CHARTER.md"}]}"#;
    let listed = r#"{"schema":1,"paths":[{"path":"CHARTER.md"},{"path":"docs/harness/d59.json"}]}"#;
    let ended = r#"{"schema":1,"d59":"ended","owner_words":"x","time":"2026-10-21T00:00:00Z"}"#;
    assert!(d59_in_force(unlisted, Some(ended)));
    assert!(!d59_in_force(listed, Some(ended)));
    assert!(d59_in_force(listed, Some("{not json")));
    assert!(d59_in_force(listed, None));
    assert!(d59_in_force(listed, Some(r#"{"d59":"in force"}"#)));
    // The owner ended D59 on 2026-10-22 ("End D59 now"); the repository's own listing and switch say so.
    let real = include_str!("../../../docs/harness/owner-paths.json");
    let real_switch = include_str!("../../../docs/harness/d59.json");
    assert!(!d59_in_force(real, Some(real_switch)));
    assert!(!d59_in_force(real, Some(ended)));

    let off = Case {
        d59: d59_in_force(listed, Some(ended)),
        ..SETUP_PR
    };
    let out = run(&off, &[body("claude", Verdict::Approve)]);
    assert!(out.setup_pr);
    assert!(!out.pass);
    assert!(
        has(&out, "needs `context: fresh` (D66)"),
        "{:?}",
        out.details
    );
    assert!(run(&off, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn rows_prefixes_round_trip_and_setup_row() {
    let rows = rows();
    for r in ["A", "1", "6", "7a", "7b", "7c", "12"] {
        assert!(rows.iter().any(|x| x == r), "{r} missing from {rows:?}");
    }
    assert_eq!(row_item_prefix("6").as_deref(), Some("infra-loop-pr06-"));
    assert_eq!(row_item_prefix("7a").as_deref(), Some("infra-loop-pr07a-"));
    assert_eq!(row_item_prefix("12").as_deref(), Some("infra-loop-pr12-"));
    assert_eq!(row_item_prefix("A"), None);
    assert_eq!(row_item_prefix("1"), None);

    let mut r = record("kimi", Verdict::Changes);
    r.fresh_context = true;
    r.done_when = DoneWhen::Gpu("ab".repeat(32));
    assert_eq!(parse(&render(&r)), Parsed::Record(r.clone()));
    let plain = record("codex", Verdict::Approve);
    let with_findings = format!("{}\nFindings: none.\nverdict: CHANGES\n", render(&plain));
    assert_eq!(parse(&with_findings), Parsed::Record(plain));

    assert_eq!(
        setup_row("Lab: claude. Setup row: 7b. Base: x").as_deref(),
        Some("7b")
    );
    assert_eq!(setup_row("Lab: claude."), None);
}

#[test]
fn malformed_fields_are_named() {
    let good = body("codex", Verdict::Approve);
    let cases = [
        (good.replace("pr: 2926\n", ""), "missing field \"pr\""),
        (
            good.replace("pr: 2926\n", "pr: 2926\npr: 2926\n"),
            "duplicated field",
        ),
        (good.replace(HEAD, &HEAD.to_uppercase()), "bad head"),
        (good.replace("pr: 2926", "pr: #2926"), "bad pr"),
        (
            good.replace("verdict: APPROVE", "verdict: LGTM"),
            "bad verdict",
        ),
        (
            good.replace("done-when: host: ", "done-when: maybe: "),
            "bad done-when",
        ),
        (
            good.replace("model: unknown", "model: unknown\nmood: good"),
            "unknown field",
        ),
    ];
    for (text, want) in cases {
        match parse(&text) {
            Parsed::Malformed(why) => assert!(why.contains(want), "{want}: {why}"),
            other => panic!("{want}: got {other:?}"),
        }
    }
    assert_eq!(parse("Looks good to me"), Parsed::NotARecord);
}

/// `uor-harness loop review` against a temporary repository with a bare `origin`, GitHub replaced
/// by the test-only `UOR_LOOP_REVIEW_GH_FAKE` file. Every child process has every `GIT_*`
/// variable removed, and each repository sits inside its temp directory.
mod cli {
    use super::{parse, Parsed, Verdict, HEAD, LOOP_PAGE};
    use serde_json::{json, Value};
    use std::path::PathBuf;
    use std::process::{Command, Output};

    const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
    const N: u64 = 2930;
    const ORDER: &str = "docs/loop/tasks/infra-loop-harness/claude-18/ORDER.md";
    const SETUP_ORDER: &str =
        "Lab: claude\nItem: [i](../../../roadmap/items/infra-loop-harness.md)\nSetup row: 6\n";
    const CODE_ORDER: &str = "Lab: claude\nItem: [i](../../../roadmap/items/infra-other.md)\n";
    const CODE_FILE: &str = "crates/uor-r4-harness/src/x.rs";

    fn clean(cmd: &mut Command) -> &mut Command {
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(k);
            }
        }
        cmd.env_remove("UOR_LAB")
            .env_remove("UOR_LOOP_MODEL")
            .env_remove("UOR_LOOP_LAUNCH")
            .env_remove("UOR_LOOP_REVIEW_GH_FAKE")
    }

    struct Repo {
        tmp: tempfile::TempDir,
        root: PathBuf,
    }

    impl Repo {
        fn new() -> Repo {
            let tmp = tempfile::tempdir().unwrap();
            let base = tmp.path().canonicalize().unwrap();
            let root = base.join("repo");
            assert!(root.starts_with(&base));
            std::fs::create_dir_all(&root).unwrap();
            let r = Repo { tmp, root };
            r.git(&["init", "-q", "-b", "main"]);
            r.git(&["config", "user.email", "t@example.invalid"]);
            r.git(&["config", "user.name", "t"]);
            r.write(".gitignore", "/local/\n");
            r.write("docs/loop/README.md", LOOP_PAGE);
            r.write(
                "docs/harness/labs.json",
                r#"{"labs":{
                  "claude":{"vendor":"Anthropic","client":"b","headless":"c","open_pr_limit":3},
                  "codex":{"vendor":"OpenAI","client":"b","headless":"c","open_pr_limit":3}}}"#,
            );
            r.write(
                "docs/harness/owner-paths.json",
                r#"{"paths":[{"path":"CHARTER.md","why":"x"}]}"#,
            );
            r.git(&["add", "-A"]);
            r.git(&["commit", "-q", "-m", "init"]);
            let remote = base.join("remote.git");
            let st = clean(&mut Command::new("git"))
                .args(["init", "-q", "--bare", "-b", "main"])
                .arg(&remote)
                .output()
                .unwrap();
            assert!(st.status.success(), "{st:?}");
            r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
            r.git(&["push", "-q", "origin", "main"]);
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
            assert!(p.starts_with(&self.root));
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }

        fn fake_path(&self) -> PathBuf {
            self.tmp.path().join("review-fake.json")
        }

        /// The comment bodies `post_comment` logged.
        fn posts(&self) -> Vec<String> {
            let calls = PathBuf::from(format!("{}.calls.jsonl", self.fake_path().display()));
            std::fs::read_to_string(calls)
                .unwrap_or_default()
                .lines()
                .map(|l| {
                    let v: Value = serde_json::from_str(l).unwrap();
                    v["body"].as_str().unwrap().to_string()
                })
                .collect()
        }

        /// `loop review <args>` as `lab` (none when empty) with `fake` as GitHub.
        fn review(&self, lab: &str, fake: &Value, args: &[&str]) -> Output {
            std::fs::write(self.fake_path(), fake.to_string()).unwrap();
            let mut cmd = Command::new(BIN);
            clean(&mut cmd);
            cmd.arg("--repo")
                .arg(&self.root)
                .args(["loop", "review"])
                .args(args)
                .env("UOR_LOOP_REVIEW_GH_FAKE", self.fake_path())
                .current_dir(&self.root);
            if !lab.is_empty() {
                cmd.env("UOR_LAB", lab);
            }
            cmd.output().unwrap()
        }
    }

    fn fake(order: &str, files: &[&str], comments: &[String]) -> Value {
        let mut all: Vec<&str> = vec![ORDER];
        all.extend_from_slice(files);
        json!({
            "pr": {"headRefOid": HEAD, "files": all},
            "files": {ORDER: order},
            "comments": comments,
        })
    }

    fn text(o: &Output) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        )
    }

    const APPROVE_HOST: [&str; 6] = [
        "--pr",
        "2930",
        "--verdict",
        "APPROVE",
        "--host",
        "cargo test: ok",
    ];

    const FRESH_APPROVE_HOST: [&str; 8] = [
        "--pr",
        "2930",
        "--verdict",
        "APPROVE",
        "--host",
        "cargo test: ok",
        "--context",
        "fresh",
    ];

    #[test]
    fn a_claude_approve_on_a_setup_pr_is_posted_as_a_record_at_the_head() {
        let r = Repo::new();
        let o = r.review(
            "claude",
            &fake(SETUP_ORDER, &[CODE_FILE], &[]),
            &FRESH_APPROVE_HOST,
        );
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        assert!(text(&o).contains(&format!("review: posted APPROVE on PR #{N} at {HEAD}")));
        let posts = r.posts();
        assert_eq!(posts.len(), 1);
        let Parsed::Record(rec) = parse(&posts[0]) else {
            panic!("not a record: {}", posts[0]);
        };
        assert_eq!(
            (rec.head.as_str(), rec.pr, rec.lab.as_str()),
            (HEAD, N, "claude")
        );
        assert_eq!(rec.verdict, Verdict::Approve);
        assert_eq!(rec.order, ORDER);
        assert_eq!(
            (rec.model.as_str(), rec.launch.as_str()),
            ("unknown", "manual")
        );
    }

    #[test]
    fn a_codex_approve_on_a_setup_pr_is_refused_under_d59() {
        let r = Repo::new();
        let o = r.review(
            "codex",
            &fake(SETUP_ORDER, &[CODE_FILE], &[]),
            &APPROVE_HOST,
        );
        assert_eq!(o.status.code(), Some(1), "{}", text(&o));
        assert!(text(&o).contains("D59"), "{}", text(&o));
        assert!(r.posts().is_empty());
    }

    #[test]
    fn a_same_lab_approve_without_fresh_context_on_a_code_pr_is_refused() {
        let r = Repo::new();
        let o = r.review(
            "claude",
            &fake(CODE_ORDER, &[CODE_FILE], &[]),
            &APPROVE_HOST,
        );
        assert_eq!(o.status.code(), Some(1), "{}", text(&o));
        assert!(
            text(&o).contains("needs `context: fresh` (D66)"),
            "{}",
            text(&o)
        );
        assert!(r.posts().is_empty());
    }

    #[test]
    fn a_fresh_context_self_review_counts_on_any_pr_d66() {
        let mut args = APPROVE_HOST.to_vec();
        args.extend(["--context", "fresh"]);
        let r = Repo::new();
        let o = r.review("claude", &fake(CODE_ORDER, &["docs/x.md"], &[]), &args);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        assert_eq!(r.posts().len(), 1);
        // D66: the review is part of the unit, so it is posted on a code PR too.
        let r = Repo::new();
        let o = r.review("claude", &fake(CODE_ORDER, &[CODE_FILE], &[]), &args);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        assert_eq!(r.posts().len(), 1);
    }

    #[test]
    fn dry_run_prints_the_record_and_posts_nothing() {
        let r = Repo::new();
        let mut args = FRESH_APPROVE_HOST.to_vec();
        args.push("--dry-run");
        let o = r.review("claude", &fake(SETUP_ORDER, &[CODE_FILE], &[]), &args);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(out.contains("[claude] uor-review v1\n"), "{out}");
        assert!(out.contains(&format!("head: {HEAD}\n")), "{out}");
        assert!(r.posts().is_empty());
    }

    #[test]
    fn a_claude_record_on_a_setup_pr_without_fresh_context_is_refused() {
        let r = Repo::new();
        let o = r.review(
            "claude",
            &fake(SETUP_ORDER, &[CODE_FILE], &[]),
            &APPROVE_HOST,
        );
        assert_eq!(o.status.code(), Some(1), "{}", text(&o));
        assert!(text(&o).contains("needs `context: fresh`"), "{}", text(&o));
        assert!(r.posts().is_empty());
        let o = r.review(
            "claude",
            &fake(SETUP_ORDER, &[CODE_FILE], &[]),
            &FRESH_APPROVE_HOST,
        );
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        assert_eq!(r.posts().len(), 1);
        assert!(r.posts()[0].contains("\ncontext: fresh\n"));
    }

    fn record_body(lab: &str) -> String {
        format!(
            "[{lab}] uor-review v1\npr: {N}\nhead: {HEAD}\nlab: {lab}\nmodel: m\nlaunch: manual\nverdict: APPROVE\norder: {ORDER}\ncontext: fresh\ndone-when: host: ok\n"
        )
    }

    #[test]
    fn check_passes_on_a_counted_claude_approve_and_fails_on_codex_alone() {
        let r = Repo::new();
        let f = fake(SETUP_ORDER, &[CODE_FILE], &[record_body("claude")]);
        let o = r.review("codex", &f, &["--check", "--pr", "2930"]);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o));
        assert!(text(&o).contains(": counted"), "{}", text(&o));
        let f = fake(SETUP_ORDER, &[CODE_FILE], &[record_body("codex")]);
        let o = r.review("claude", &f, &["--check", "--pr", "2930"]);
        assert_eq!(o.status.code(), Some(1), "{}", text(&o));
        assert!(text(&o).contains("under D59"), "{}", text(&o));
        assert!(r.posts().is_empty());
    }

    #[test]
    fn no_lab_no_order_or_a_failed_post_cannot_run() {
        let r = Repo::new();
        let o = r.review("", &fake(SETUP_ORDER, &[CODE_FILE], &[]), &APPROVE_HOST);
        assert_eq!(o.status.code(), Some(2), "{}", text(&o));
        let no_order = json!({"pr": {"headRefOid": HEAD, "files": [CODE_FILE]}});
        let o = r.review("claude", &no_order, &APPROVE_HOST);
        assert_eq!(o.status.code(), Some(2), "{}", text(&o));
        assert!(text(&o).contains("adds no ORDER"), "{}", text(&o));
        let mut failing = fake(SETUP_ORDER, &[CODE_FILE], &[]);
        failing["failPost"] = json!(true);
        let o = r.review("claude", &failing, &FRESH_APPROVE_HOST);
        assert_eq!(o.status.code(), Some(2), "{}", text(&o));
        assert!(r.posts().is_empty());
    }
}

#[test]
fn a_claude_record_on_a_setup_pr_under_d59_needs_fresh_context() {
    let out = run(&SETUP_PR, &[body("claude", Verdict::Approve)]);
    assert!(out.setup_pr);
    assert!(!out.pass);
    assert!(
        has(&out, "on a setup PR under D59 needs `context: fresh`"),
        "{:?}",
        out.details
    );
    assert!(run(&SETUP_PR, &[fresh("claude", Verdict::Approve)]).pass);
    // Without D59, the ORDER lab's own fresh record counts by D66; another lab of its vendor
    // still does not count.
    let off = Case {
        d59: false,
        ..SETUP_PR
    };
    assert!(run(&off, &[fresh("claude", Verdict::Approve)]).pass);
}

#[test]
fn a_loop_item_needs_a_valid_setup_row_and_others_are_ordinary() {
    for row in [None, Some("99")] {
        let c = Case {
            item: "infra-loop-pr07-x",
            setup_row: row,
            ..CODE_PR
        };
        let out = run(&c, &[body("codex", Verdict::Approve)]);
        assert!(!out.pass, "{row:?}: {:?}", out.details);
        assert!(
            has(&out, "a loop setup item's ORDER needs a valid `Setup row:`"),
            "{:?}",
            out.details
        );
        let (rows, vendors) = (rows(), vendors());
        let input = GateInput {
            pr: PR,
            head: HEAD,
            order_path: ORDER_PATH,
            order_lab: "claude",
            order_item: "infra-loop-pr07-x",
            setup_row: row,
            rows: &rows,
            vendors: &vendors,
            d59_in_force: true,
            doc_only: false,
            comments: &[],
        };
        assert!(!may_review(&input, "claude"));
        assert!(!may_review(&input, "codex"));
    }
    // A non-loop item with no row is an ordinary PR: the vendor rule.
    assert!(run(&CODE_PR, &[body("codex", Verdict::Approve)]).pass);
}

#[test]
fn setup_row_reads_only_a_row_token() {
    let order = "Lab: claude. Item: [x](../x.md) (#2909). Setup row: 6. Base: abc";
    assert_eq!(setup_row(order).as_deref(), Some("6"));
    assert_eq!(setup_row("Setup row:\tA\n").as_deref(), Some("A"));
    assert_eq!(setup_row("Setup row: 12").as_deref(), Some("12"));
    assert_eq!(setup_row("Setup row: 7b next").as_deref(), Some("7b"));
    assert_eq!(setup_row("no `Setup row:` line"), None);
    assert_eq!(setup_row("Setup row: **6**"), None);
    assert_eq!(setup_row("Setup row: 6x5"), None);
    assert_eq!(setup_row("Setup row: B."), None);
}

#[test]
fn a_record_marker_below_the_first_line_is_malformed() {
    let late = format!("Review below.\n{}", body("codex", Verdict::Approve));
    match parse(&late) {
        Parsed::Malformed(why) => assert!(why.contains("must start the comment"), "{why}"),
        other => panic!("expected Malformed, got {other:?}"),
    }
    let out = run(&CODE_PR, &[late]);
    assert!(!out.pass);
    assert!(has(&out, "must start the comment"), "{:?}", out.details);
}

#[test]
fn a_record_for_another_order_does_not_count() {
    let other = render(&Record {
        order: "docs/loop/tasks/infra-other/codex-3/ORDER.md".to_string(),
        ..record("codex", Verdict::Approve)
    });
    let out = run(&CODE_PR, &[other]);
    assert!(!out.pass);
    assert!(
        has(&out, &format!("is not this PR's ORDER {ORDER_PATH}")),
        "{:?}",
        out.details
    );
}

#[test]
fn doc_only_includes_generated_paths() {
    let c = Case {
        doc_only: doc_only(["Cargo.lock", "docs/x.md", ORDER_PATH]),
        ..CODE_PR
    };
    assert!(c.doc_only);
    assert!(doc_only(["docs/roadmap/README.md", "docs/y.md"]));
    assert!(!doc_only(["Cargo.lock", "crates/x/src/lib.rs"]));
    assert!(run(&c, &[fresh("claude", Verdict::Approve)]).pass);
}

// ---- the steady-state render PR (branch `kimi/render`, no ORDER) ----

const RENDER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn render_record(verdict: Verdict) -> Record {
    Record {
        head: RENDER_HEAD.to_string(),
        order: RENDER_ORDER.to_string(),
        ..record("claude", verdict)
    }
}

fn render_vendors() -> BTreeMap<String, String> {
    [
        ("claude", "anthropic"),
        ("deepseek", "deepseek"),
        ("kimi", "moonshot"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

#[test]
fn render_pr_without_order_needs_the_render_branch_and_views_only() {
    let views = vec![
        "STATE.md".to_string(),
        "docs/harness/LESSONS.md".to_string(),
    ];
    assert!(render_pr_without_order(RENDER_BRANCH, &views));
    assert!(!render_pr_without_order("kimi/some-branch", &views));
    assert!(!render_pr_without_order(RENDER_BRANCH, &[]));
    assert!(!render_pr_without_order(
        RENDER_BRANCH,
        &["crates/uor-r4-harness/src/lib.rs".to_string()]
    ));
    assert!(!render_pr_without_order(
        RENDER_BRANCH,
        &["STATE.md".to_string(), "README.md".to_string()]
    ));
    assert!(render_pr_without_order(
        RENDER_BRANCH,
        &["AGENTS.md".to_string()]
    ));
    assert!(!render_pr_without_order(
        RENDER_BRANCH,
        &["docs/roadmap/README.md".to_string()]
    ));
}

#[test]
fn compare_rendered_views_catches_a_hand_edited_view() {
    let expected = vec![
        ("STATE.md".to_string(), "stamp\n# STATE\n".to_string()),
        ("HANDOFF.md".to_string(), "stamp\n# HANDOFF\n".to_string()),
    ];
    let mut actual: BTreeMap<String, String> = expected.iter().cloned().collect();
    assert_eq!(compare_rendered_views(&expected, &actual), Ok(()));
    actual.insert(
        "STATE.md".to_string(),
        "stamp\n# STATE hand-edited\n".to_string(),
    );
    let err = compare_rendered_views(&expected, &actual).expect_err("hand edit must fail");
    assert!(
        err.contains("STATE.md") && err.contains("hand-edited"),
        "{err}"
    );
}

#[test]
fn a_render_record_counts_with_the_sentinel_order_but_not_on_an_ordinary_pr() {
    let body = render(&render_record(Verdict::Approve));
    let parsed_order = match parse(&body) {
        Parsed::Record(r) => r.order,
        _ => panic!("the render record must parse"),
    };
    assert_eq!(parsed_order, RENDER_ORDER);
    let comments = [body];
    let vendors = render_vendors();
    let render_input = GateInput {
        pr: PR,
        head: RENDER_HEAD,
        order_path: RENDER_ORDER,
        order_lab: "kimi",
        order_item: "",
        setup_row: None,
        rows: &[],
        vendors: &vendors,
        d59_in_force: false,
        doc_only: true,
        comments: &comments,
    };
    assert!(
        gate(&render_input).pass,
        "a render record with the sentinel must count"
    );
    let ordinary = GateInput {
        order_path: ORDER_PATH,
        ..render_input
    };
    assert!(
        !gate(&ordinary).pass,
        "the sentinel must not count on an ordinary PR"
    );
}

#[test]
fn a_follow_up_loop_item_is_ordinary_but_a_build_plan_item_still_needs_its_setup_row() {
    assert!(!is_setup_item("infra-loop-render-review"));
    assert!(is_setup_item("infra-loop-harness"));
    assert!(is_setup_item("infra-loop-pr08-migration"));
    let vendors = render_vendors();
    let body = render(&Record {
        lab: "deepseek".to_string(),
        head: HEAD.to_string(),
        order: ORDER_PATH.to_string(),
        ..record("claude", Verdict::Approve)
    });
    let comments = [body];
    let rows = build_rows(LOOP_PAGE);
    let ordinary = GateInput {
        pr: PR,
        head: HEAD,
        order_path: ORDER_PATH,
        order_lab: "claude",
        order_item: "infra-loop-render-review",
        setup_row: None,
        rows: &rows,
        vendors: &vendors,
        d59_in_force: false,
        doc_only: false,
        comments: &comments,
    };
    assert!(
        gate(&ordinary).pass,
        "a follow-up infra-loop item is ordinary and needs no Setup row"
    );
    let build_plan = GateInput {
        order_item: "infra-loop-pr08-migration",
        ..ordinary
    };
    let outcome = gate(&build_plan);
    assert!(
        !outcome.pass,
        "a build-plan item without a Setup row must not count"
    );
    assert!(
        outcome.details.iter().any(|d| d.contains("Setup row")),
        "the refusal names the Setup row: {:?}",
        outcome.details
    );
}

#[test]
fn default_render_at_is_the_merge_base_on_the_render_branch() {
    fn run(dir: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
    let dir = std::env::temp_dir().join(format!(
        "uor-looprender-default-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    run(&dir, &["init", "-q", "-b", "main"]);
    run(&dir, &["remote", "add", "origin", dir.to_str().unwrap()]);
    run(&dir, &["config", "uploadpack.allowAnySHA1InWant", "true"]);
    run(
        &dir,
        &["config", "uploadpack.allowReachableSHA1InWant", "true"],
    );
    std::fs::write(dir.join("a"), "a").unwrap();
    run(&dir, &["add", "."]);
    run(
        &dir,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "a",
        ],
    );
    let a = run(&dir, &["rev-parse", "HEAD"]);
    run(&dir, &["update-ref", "refs/remotes/origin/main", &a]);
    run(&dir, &["checkout", "-q", "-b", "kimi/render"]);
    std::fs::write(dir.join("b"), "b").unwrap();
    run(&dir, &["add", "."]);
    run(
        &dir,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "b",
        ],
    );
    assert_eq!(
        uor_r4_harness::looprender::default_at(&dir, "kimi/render").unwrap(),
        a,
        "the render branch stamps the merge base"
    );
    assert_eq!(
        uor_r4_harness::looprender::default_at(&dir, "feature").unwrap(),
        "HEAD",
        "other branches keep HEAD"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn render_sentinel_requires_a_known_same_repository_head() {
    let views = vec![
        "STATE.md".to_string(),
        "docs/harness/LESSONS.md".to_string(),
    ];
    assert!(render_sentinel(RENDER_BRANCH, &views, Some(false)));
    assert!(
        !render_sentinel(RENDER_BRANCH, &views, Some(true)),
        "a fork must not take the sentinel path"
    );
    assert!(
        !render_sentinel(RENDER_BRANCH, &views, None),
        "an unknown repository must not take the sentinel path"
    );
    assert!(!render_sentinel("kimi/other", &views, Some(false)));
    assert!(!render_sentinel(
        RENDER_BRANCH,
        &["crates/uor-r4-harness/src/lib.rs".to_string()],
        Some(false)
    ));
}

/// Offline fixture: a temp git repo (base commit, `kimi/render` head), a fake gh file and an
/// injected renderer drive `loopreview::cli_with_renderer`; exit 1 means the render gate passed
/// and only the record was missing, exit 2 means the gate refused. The cases pin the planted
/// faults: removing `render_gate` from `context()` or disabling the view/block/stamp checks
/// turns a refusal into exit 1 and fails the assertions.
#[test]
fn render_gate_fixture_refuses_tampered_branch_only_forks_and_unknown() {
    use std::path::{Path, PathBuf};
    use uor_r4_harness::looprender::AtCommit;
    use uor_r4_harness::loopreview::{cli_with_renderer, RenderSource, REVIEW_GH_FAKE_ENV};

    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());

    struct TestRender {
        views: std::collections::BTreeMap<String, Vec<(String, String)>>,
        blocks: std::collections::BTreeMap<String, String>,
    }
    impl RenderSource for TestRender {
        fn render_all(&self, at: &AtCommit) -> Result<Vec<(String, String)>, String> {
            self.views
                .get(&at.sha)
                .cloned()
                .ok_or_else(|| format!("no fake render at {}", at.sha))
        }
        fn render_top_ten(&self, at: &AtCommit) -> Result<String, String> {
            self.blocks
                .get(&at.sha)
                .cloned()
                .ok_or_else(|| format!("no fake block at {}", at.sha))
        }
    }

    fn git_at(dir: &Path, args: &[&str]) -> String {
        let mut cmd = std::process::Command::new("git");
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(k);
            }
        }
        let out = cmd
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
    fn commit(dir: &Path) -> String {
        git_at(dir, &["add", "."]);
        git_at(
            dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "-m",
                "c",
            ],
        );
        git_at(dir, &["rev-parse", "HEAD"])
    }

    // case: 0 clean, 1 hand-edited view, 2 fork, 3 AGENTS outside block, 4 altered block, 5 wrong stamp.
    fn setup(tag: &str, case: u8) -> (PathBuf, String, TestRender) {
        let dir = std::env::temp_dir().join(format!(
            "uor-render-gate-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        git_at(&dir, &["init", "-q", "-b", "main"]);
        git_at(&dir, &["remote", "add", "origin", dir.to_str().unwrap()]);
        git_at(&dir, &["config", "uploadpack.allowAnySHA1InWant", "true"]);
        git_at(
            &dir,
            &["config", "uploadpack.allowReachableSHA1InWant", "true"],
        );
        std::fs::write(
            dir.join("AGENTS.md"),
            "top\n<!-- loop:lessons-top-ten:start -->\nold\n<!-- loop:lessons-top-ten:end -->\ntail\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("docs/loop")).unwrap();
        std::fs::create_dir_all(dir.join("docs/harness")).unwrap();
        std::fs::write(
            dir.join("docs/loop/README.md"),
            "## Build plan\n\n| PR | What it builds |\n|---|---|\n| 6 | harness |\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("docs/harness/labs.json"),
            r#"{"schema":1,"labs":{"antigravity":{"vendor":"Google","client":"A","headless":"a -p","open_pr_limit":3},"claude":{"vendor":"Anthropic","client":"C","headless":"c -p","open_pr_limit":3},"codex":{"vendor":"OpenAI","client":"X","headless":"x exec","open_pr_limit":3},"deepseek":{"vendor":"DeepSeek","client":"D","headless":"d","open_pr_limit":3},"kimi":{"vendor":"Moonshot AI","client":"K","headless":"k -p","open_pr_limit":3,"role":"pm"}}}"#,
        )
        .unwrap();
        std::fs::write(dir.join("docs/harness/owner-paths.json"), r#"{"paths":[]}"#).unwrap();
        for p in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
            let path = dir.join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "base\n").unwrap();
        }
        let a = commit(&dir);
        git_at(&dir, &["checkout", "-q", "-b", "kimi/render"]);
        let base_stamp = uor_r4_harness::looprender::stamp(&a);
        let mut views = std::collections::BTreeMap::new();
        let mut blocks = std::collections::BTreeMap::new();
        let mut per = Vec::new();
        for p in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
            let text = format!("{base_stamp}\n# {p}\n");
            per.push((p.to_string(), text.clone()));
            std::fs::write(dir.join(p), &text).unwrap();
        }
        views.insert(a.clone(), per.clone());
        blocks.insert(a.clone(), format!("\n{base_stamp}\n- one\n"));
        let mut block = format!("\n{base_stamp}\n- one\n");
        if case == 4 {
            block = format!("\n{base_stamp}\n- altered\n");
        }
        let mut agents = format!(
            "top\n<!-- loop:lessons-top-ten:start -->{block}<!-- loop:lessons-top-ten:end -->\ntail\n"
        );
        if case == 3 {
            agents = format!(
                "top\n<!-- loop:lessons-top-ten:start -->{block}<!-- loop:lessons-top-ten:end -->\ntail\nextra\n"
            );
        }
        std::fs::write(dir.join("AGENTS.md"), agents).unwrap();
        let head = commit(&dir);
        let mut head_sha = head.clone();
        if case == 5 {
            // Commit views whose stamp is the branch head, and a correct base-stamped AGENTS block.
            let hstamp = uor_r4_harness::looprender::stamp(&head);
            for p in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
                std::fs::write(dir.join(p), format!("{hstamp}\n# {p}\n")).unwrap();
            }
            head_sha = commit(&dir);
        }
        if case == 1 {
            std::fs::write(
                dir.join("STATE.md"),
                format!("{base_stamp}\n# STATE hand-edited\n"),
            )
            .unwrap();
            head_sha = commit(&dir);
        }
        if case == 5 {
            // The canned renderer must serve the commit the committed views actually stamp, so
            // that disabling ONLY the views' stamp check still passes the render comparison and
            // turns the refusal into exit 1 - which the test asserts against.
            let text = std::fs::read_to_string(dir.join("STATE.md")).unwrap();
            let stamp_sha = text
                .lines()
                .next()
                .and_then(uor_r4_harness::looprender::parse_stamp)
                .expect("committed view is stamped")
                .to_string();
            let mut served = Vec::new();
            for p in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
                served.push((p.to_string(), std::fs::read_to_string(dir.join(p)).unwrap()));
            }
            views.insert(stamp_sha, served);
        }
        git_at(&dir, &["update-ref", "refs/remotes/origin/main", &a]);
        let cross = case == 2;
        let gh = serde_json::json!({
            "pr": {
                "headRefOid": head_sha,
                "headRefName": "kimi/render",
                "isCrossRepository": cross,
                "files": ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md", "AGENTS.md"],
            },
            "files": {},
            "comments": [],
        });
        std::fs::write(dir.join("gh.json"), gh.to_string()).unwrap();
        (dir, head_sha, TestRender { views, blocks })
    }

    for (case, want) in [(0u8, 1), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2)] {
        let (dir, _head, render) = setup(&format!("c{case}"), case);
        std::env::set_var(REVIEW_GH_FAKE_ENV, dir.join("gh.json"));
        let args: Vec<String> = ["--check", "--pr", "7"]
            .into_iter()
            .map(String::from)
            .collect();
        let got = cli_with_renderer(&dir, "deepseek", &args, &render);
        std::env::remove_var(REVIEW_GH_FAKE_ENV);
        assert_eq!(got, want, "case {case}: expected exit {want}, got {got}");
        std::fs::remove_dir_all(&dir).ok();
    }
}

/// The queue mirror must ask for a known same-repository head. The built binary is run with a
/// fake GitHub file whose `is_cross_repository` is true/false; the `review-record` detail differs
/// if `ReviewContext::of` ever passes `Some(false)` instead of the real flag.
#[test]
fn queue_mirror_requires_a_known_same_repository_head() {
    use std::path::Path;
    fn git_at(dir: &Path, args: &[&str]) -> String {
        let mut cmd = std::process::Command::new("git");
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(k);
            }
        }
        let out = cmd
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
    fn commit(dir: &Path) -> String {
        git_at(dir, &["add", "."]);
        git_at(
            dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "-m",
                "c",
            ],
        );
        git_at(dir, &["rev-parse", "HEAD"])
    }
    let dir = std::env::temp_dir().join(format!(
        "uor-mirror-cross-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(dir.join("docs/loop")).unwrap();
    std::fs::create_dir_all(dir.join("docs/harness")).unwrap();
    std::fs::write(
        dir.join("docs/loop/README.md"),
        "## Build plan\n\n| PR | What it builds |\n|---|---|\n| 6 | harness |\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("docs/harness/labs.json"),
        r#"{"schema":1,"labs":{"antigravity":{"vendor":"Google","client":"A","headless":"a","open_pr_limit":3},"claude":{"vendor":"Anthropic","client":"C","headless":"c","open_pr_limit":3},"codex":{"vendor":"OpenAI","client":"X","headless":"x","open_pr_limit":3},"deepseek":{"vendor":"DeepSeek","client":"D","headless":"d","open_pr_limit":3},"kimi":{"vendor":"Moonshot AI","client":"K","headless":"k","open_pr_limit":3}}}"#,
    )
    .unwrap();
    std::fs::write(dir.join("docs/harness/owner-paths.json"), r#"{"paths":[]}"#).unwrap();
    for view in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
        let path = dir.join(view);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "base\n").unwrap();
    }
    std::fs::write(
        dir.join("AGENTS.md"),
        "<!-- loop:lessons-top-ten:start -->\n<!-- loop:lessons-top-ten:end -->\n",
    )
    .unwrap();
    git_at(&dir, &["init", "-q", "-b", "main"]);
    let a = commit(&dir);
    git_at(&dir, &["checkout", "-q", "-b", "kimi/render"]);
    for view in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
        std::fs::write(
            dir.join(view),
            format!("{}\n# \n", uor_r4_harness::looprender::stamp(&a)),
        )
        .unwrap();
    }
    commit(&dir);
    git_at(&dir, &["update-ref", "refs/remotes/origin/main", &a]);
    for cross in [true, false] {
        let fake = dir.join(format!("gh-{cross}.json"));
        std::fs::write(
            &fake,
            format!("{{\"is_cross_repository\":{cross},\"comments\":[]}}"),
        )
        .unwrap();
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_uor-harness"))
            .args(["loop", "pr-check", "--pr", "7", "--json"])
            .current_dir(&dir)
            .env("UOR_LAB", "deepseek")
            .env("UOR_PRCHECK_GH_FAKE", &fake)
            .output()
            .expect("binary runs");
        let text = String::from_utf8_lossy(&out.stdout);
        let err_text = String::from_utf8_lossy(&out.stderr);
        let v: serde_json::Value = serde_json::from_str(text.trim()).unwrap_or_else(|e| {
            panic!("pr-check --json printed {text:?} (stderr {err_text:?}): {e}");
        });
        let details = v["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["rule"] == "review-record")
            .map(|r| format!("{}{}", r["summary"], r["details"]))
            .unwrap_or_default();
        if cross {
            assert!(
                details.contains("cannot check the review record"),
                "fork must leave the sentinel untouched: {details}"
            );
        } else {
            assert!(
                details.contains("no counted record"),
                "same-repo render PR reaches the counted-record gate: {details}"
            );
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}
