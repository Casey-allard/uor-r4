//! `uor-harness loop board [--json]`: a read-only view of every registered lab (loop PR 7c).
//!
//! For each lab in `docs/harness/labs.json` it shows the vendor, client and role; its presence
//! (state, age, `doing`, `next`), live tier first and committed as the fallback, exactly as
//! `multilab presence` reads it; how many claims it holds; and its open PRs, each with number,
//! draft or ready, and short head. A PR is a lab's when its title starts with `[<lab>]` for a
//! registered lab; every other open PR is listed apart under "untagged", so an unattributed PR is
//! on screen rather than silently dropped.
//!
//! **Why GitHub sits behind a trait with one read method.** The board must never write. The trait
//! [`BoardGh`] has no write method, so no implementation (the real one or the test fake) can be
//! asked to write, and the fake keeps no calls log because there is nothing to record.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::labs::Lab;
use crate::multilab::{self, BoardEntry};
use crate::state::StatePaths;

/// Tests only: names a JSON file holding the open-PR list, in the form `gh pr list --json` prints.
pub const BOARD_GH_FAKE_ENV: &str = "UOR_LOOP_BOARD_GH_FAKE";

const USAGE: &str = "usage: uor-harness loop board [--json]";

/// One open PR, as `gh pr list --json number,title,headRefOid,isDraft` prints it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPr {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub head_ref_oid: String,
    #[serde(default)]
    pub is_draft: bool,
}

impl OpenPr {
    fn short_head(&self) -> &str {
        self.head_ref_oid
            .get(..7)
            .unwrap_or(self.head_ref_oid.as_str())
    }

    fn readiness(&self) -> &'static str {
        if self.is_draft {
            "draft"
        } else {
            "ready"
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "number": self.number,
            "title": self.title,
            "draft": self.is_draft,
            "head": self.short_head(),
        })
    }
}

/// Every GitHub read the board makes. It has no write method, so the board cannot write.
pub trait BoardGh {
    /// Every open PR in the repository.
    fn open_prs(&self) -> Result<Vec<OpenPr>, String>;
}

/// `gh` itself, run in the checkout.
pub struct RealBoardGh {
    pub root: PathBuf,
}

impl BoardGh for RealBoardGh {
    fn open_prs(&self) -> Result<Vec<OpenPr>, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--limit",
                "200",
                "--json",
                "number,title,headRefOid,isDraft",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("gh pr list printed malformed JSON: {e}"))
    }
}

/// The in-memory stand-in for `gh`, read from the file `UOR_LOOP_BOARD_GH_FAKE` names.
pub struct FakeBoardGh {
    prs: Vec<OpenPr>,
}

impl FakeBoardGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {BOARD_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        let prs = serde_json::from_str(&text).map_err(|e| {
            format!(
                "{BOARD_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })?;
        Ok(FakeBoardGh { prs })
    }
}

impl BoardGh for FakeBoardGh {
    fn open_prs(&self) -> Result<Vec<OpenPr>, String> {
        Ok(self.prs.clone())
    }
}

/// One lab's row on the board.
pub struct LabRow {
    pub name: String,
    pub lab: Lab,
    /// `None` when the lab has no presence file in either tier.
    pub presence: Option<BoardEntry>,
    pub claims: usize,
    pub prs: Vec<OpenPr>,
}

/// The whole board: labs sorted by name, PRs by number.
pub struct Board {
    pub labs: Vec<LabRow>,
    pub untagged: Vec<OpenPr>,
}

/// The registered lab whose `[<lab>]` tag starts `title`, if any.
pub fn lab_of(title: &str, labs: &BTreeMap<String, Lab>) -> Option<String> {
    labs.keys()
        .find(|name| title.starts_with(&format!("[{name}]")))
        .cloned()
}

/// Assemble the board from the registry, the state tiers and the open PRs. Reads only.
pub fn build(
    paths: &StatePaths,
    labs: &BTreeMap<String, Lab>,
    mut prs: Vec<OpenPr>,
    now: i64,
) -> Board {
    prs.sort_by_key(|p| p.number);
    let presence = multilab::board(paths, now);
    let claims = multilab::claims(paths);
    let mut by_lab: BTreeMap<String, Vec<OpenPr>> = BTreeMap::new();
    let mut untagged = Vec::new();
    for pr in prs {
        match lab_of(&pr.title, labs) {
            Some(name) => by_lab.entry(name).or_default().push(pr),
            None => untagged.push(pr),
        }
    }
    let rows = labs
        .iter()
        .map(|(name, lab)| LabRow {
            name: name.clone(),
            lab: lab.clone(),
            presence: presence
                .iter()
                .find(|e| crate::state::slug(&e.row.lab) == *name)
                .cloned(),
            claims: claims
                .iter()
                .filter(|(_, c)| crate::state::slug(&c.lab) == *name)
                .count(),
            prs: by_lab.remove(name).unwrap_or_default(),
        })
        .collect();
    Board {
        labs: rows,
        untagged,
    }
}

/// The board as text, one block per lab and the untagged PRs last.
pub fn render_text(board: &Board) -> String {
    let mut out = String::new();
    for r in &board.labs {
        out.push_str(&format!(
            "{}  {} / {}  role: {}\n",
            r.name,
            r.lab.vendor,
            r.lab.client,
            r.lab.role.as_deref().unwrap_or("-")
        ));
        match &r.presence {
            Some(e) => {
                out.push_str(&format!("  presence: {} [{}]\n", e.state(), e.source));
                out.push_str(&format!("  doing: {}\n", or_dash(&e.row.doing)));
                out.push_str(&format!("  next: {}\n", or_dash(&e.row.next)));
            }
            None => out.push_str("  presence: no check-in\n"),
        }
        out.push_str(&format!("  claims: {}\n", r.claims));
        push_prs(&mut out, &r.prs);
    }
    out.push_str("untagged\n");
    push_prs(&mut out, &board.untagged);
    out
}

fn or_dash(s: &str) -> &str {
    if s.trim().is_empty() {
        "-"
    } else {
        s
    }
}

fn push_prs(out: &mut String, prs: &[OpenPr]) {
    if prs.is_empty() {
        out.push_str("  open PRs: none\n");
    }
    for p in prs {
        out.push_str(&format!(
            "  #{} {} {} {}\n",
            p.number,
            p.readiness(),
            p.short_head(),
            p.title
        ));
    }
}

/// The board as JSON: one entry per registered lab, and the untagged PRs.
pub fn render_json(board: &Board, fake_github: bool) -> Value {
    let labs: Vec<Value> = board
        .labs
        .iter()
        .map(|r| {
            let presence = r.presence.as_ref().map(|e| {
                json!({
                    "state": if e.stale { "stale" } else { "live" },
                    "age_minutes": (e.age_minutes >= 0).then_some(e.age_minutes),
                    "source": e.source,
                    "milestone": e.row.milestone,
                    "doing": e.row.doing,
                    "next": e.row.next,
                })
            });
            json!({
                "lab": r.name,
                "vendor": r.lab.vendor,
                "client": r.lab.client,
                "role": r.lab.role,
                "presence": presence,
                "claims": r.claims,
                "prs": r.prs.iter().map(OpenPr::to_json).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "fake_github": fake_github,
        "labs": labs,
        "untagged": board.untagged.iter().map(OpenPr::to_json).collect::<Vec<_>>(),
    })
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `loop board [--json]`. Exit 0 when it rendered, 2 when it could not run.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let mut json_out = false;
    for a in args {
        match a.as_str() {
            "--json" => json_out = true,
            _ => {
                eprintln!("uor-harness: loop board: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
        }
    }
    let labs = match crate::labs::load_labs(root) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("uor-harness: loop board: {e}");
            return 2;
        }
    };
    let fake = std::env::var_os(BOARD_GH_FAKE_ENV);
    let gh: Box<dyn BoardGh> = match &fake {
        Some(p) => match FakeBoardGh::from_file(Path::new(p)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("uor-harness: loop board: {e}");
                return 2;
            }
        },
        None => Box::new(RealBoardGh {
            root: root.to_path_buf(),
        }),
    };
    let prs = match gh.open_prs() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("uor-harness: loop board: {e}");
            return 2;
        }
    };
    let paths = StatePaths::from_env(root);
    let board = build(&paths, &labs, prs, now_unix());
    if json_out {
        println!("{}", render_json(&board, fake.is_some()));
    } else {
        if let Some(p) = &fake {
            println!(
                "FAKE GitHub: {BOARD_GH_FAKE_ENV}={} (tests only)",
                p.to_string_lossy()
            );
        }
        print!("{}", render_text(&board));
    }
    0
}
