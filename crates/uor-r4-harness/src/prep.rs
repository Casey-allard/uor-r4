//! Pre-registration sheets — the read half of the D22 gate.
//!
//! D22 requires a sheet at `TEST FITNESS: FIT` on a milestone **before** any model compute, so the
//! question this module answers is the gate's question: *does this sheet carry the blocks a run must
//! be pre-registered with, and what does its verdict line say?*
//!
//! **This is the read half only.** Rendering a new sheet (`prep new`) writes a file with a form a
//! lab fills in, and it is the next piece; validating is where the gate actually decides, and it is
//! read-only, so it is the safer half to move first. Both halves must agree, which is why the
//! required blocks live in one place here rather than being restated by each.
//!
//! A sheet that cannot be read is **not** a sheet that passes. Every failure below is explicit about
//! which block is missing and what to add, because a lab that cannot tell why its sheet is not FIT
//! will either guess or skip the gate.

use std::path::{Path, PathBuf};

use crate::state::StatePaths;

/// A verdict line, or the absence of one. Mirrors the Python `ABSENT` sentinel.
pub const ABSENT: &str = "ABSENT";
pub const PENDING: &str = "PENDING";
pub const FIT: &str = "FIT";
pub const NOT_FIT: &str = "NOT_FIT";

/// The blocks a sheet must carry, in the order a lab should fill them.
///
/// Held here once, so the validator and a future renderer cannot drift apart about what a sheet
/// needs — the same reason the Python implementation keeps them in one function.
pub const REQUIRED_BLOCKS: [(&str, &str); 8] = [
    ("bar", "Bar"),
    ("base", "Base artifact"),
    ("split", "Data split"),
    ("steps", "Steps"),
    ("stop", "Stop rule"),
    ("cost", "Cost"),
    ("power", "Minimum test power"),
    ("fit", "TEST FITNESS line"),
];

/// Which heading substring satisfies each block. `fit` is special: it needs a verdict line, not a
/// heading, and is handled separately.
fn heading_needle(key: &str) -> Option<&'static str> {
    Some(match key {
        "bar" => "bar",
        "base" => "base",
        "split" => "split",
        "steps" => "step",
        "stop" => "stop",
        "cost" => "cost",
        "power" => "power",
        _ => return None,
    })
}

/// One `##`/`###`/`####` section of a sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub body: String,
}

/// Split a sheet into sections at heading level 2–4.
///
/// Deliberately not a Markdown parser: the harness writes and reads these files itself, and the
/// Python implementation splits on the same expression. Anything more would be two parsers to keep
/// in agreement for no gain.
pub fn sections(text: &str) -> Vec<Section> {
    let lines: Vec<&str> = text.lines().collect();
    let mut marks: Vec<(usize, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let hashes = line.len() - line.trim_start_matches('#').len();
        if (2..=4).contains(&hashes) && line[hashes..].starts_with(' ') {
            let heading = line[hashes..].trim().to_string();
            if !heading.is_empty() {
                marks.push((i, heading));
            }
        }
    }
    let mut out = Vec::with_capacity(marks.len());
    for (idx, (line_no, heading)) in marks.iter().enumerate() {
        let end = marks.get(idx + 1).map(|(n, _)| *n).unwrap_or(lines.len());
        let body = lines[line_no + 1..end].join("\n").trim().to_string();
        out.push(Section {
            heading: heading.clone(),
            body,
        });
    }
    out
}

/// The verdict on the sheet's TEST FITNESS line, or [`ABSENT`].
///
/// Anchored at line start and followed by a non-word character, so prose that merely mentions
/// "TEST FITNESS" — a record explaining the gate, for instance — is not read as a verdict. That is
/// the difference between a sheet being FIT and a document talking about fitness.
pub fn test_fitness(text: &str) -> &'static str {
    for line in text.lines() {
        let t = line.trim_start_matches([' ', '\t']);
        let upper = t.to_ascii_uppercase();
        let Some(rest) = upper.strip_prefix("TEST FITNESS:") else {
            continue;
        };
        let rest = rest.trim_start();
        for (word, verdict) in [("PENDING", PENDING), ("NOT_FIT", NOT_FIT), ("FIT", FIT)] {
            if let Some(tail) = rest.strip_prefix(word) {
                // `(?![A-Za-z_])`: FIT must not match FITNESS, and NOT_FIT is tried first so it is
                // not read as FIT.
                if !tail.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                    return verdict;
                }
            }
        }
    }
    ABSENT
}

/// A `- Label: value` field from anywhere in the sheet.
pub fn field(text: &str, label: &str) -> String {
    let want = label.to_ascii_lowercase();
    for line in text.lines() {
        let stripped = line
            .trim_start()
            .trim_start_matches(['-', '*'])
            .trim_start();
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case(&want) {
            return value.trim().to_string();
        }
    }
    String::new()
}

/// Whether the sheet plans a run: it names one, or carries a steps/cost/run block.
pub fn has_run_plan(text: &str) -> bool {
    if !field(text, "Run").is_empty() {
        return true;
    }
    sections(text).iter().any(|s| {
        let h = s.heading.to_ascii_lowercase();
        h.contains("step") || h.contains("cost") || h.contains("run")
    })
}

/// One block of a sheet, and whether it is satisfied.
#[derive(Debug, Clone)]
pub struct Block {
    pub key: &'static str,
    pub name: &'static str,
    pub present: bool,
}

/// The verdict on a whole sheet.
#[derive(Debug, Clone)]
pub struct SheetReport {
    pub path: PathBuf,
    pub blocks: Vec<Block>,
    pub fitness: &'static str,
    pub run_planned: bool,
}

impl SheetReport {
    /// Every required block present. A sheet can be complete and still not FIT — completeness is
    /// "the lab filled the form in", fitness is "a reviewer said the bar is testable".
    pub fn complete(&self) -> bool {
        self.blocks.iter().all(|b| b.present)
    }

    pub fn missing(&self) -> Vec<&'static str> {
        self.blocks
            .iter()
            .filter(|b| !b.present)
            .map(|b| b.name)
            .collect()
    }

    /// The D22 gate: a sheet may be computed against only at FIT.
    ///
    /// The gate does not require completeness, because a reviewer can mark FIT and the missing
    /// block is then a documentation gap rather than a gate. It does require the verdict, and it
    /// requires it to be FIT.
    pub fn may_compute(&self) -> bool {
        self.fitness == FIT
    }

    /// What a lab should do next, stated rather than left to inference.
    pub fn verdict(&self) -> String {
        match (self.may_compute(), self.complete()) {
            (true, true) => "FIT and complete - the run may proceed (owner holds still apply)".into(),
            (true, false) => format!(
                "FIT, but incomplete: missing {}. Review may have accepted it; a filled-in sheet is \
                 what makes the bar re-checkable later.",
                self.missing().join(", ")
            ),
            (false, _) => format!(
                "NOT cleared for compute: TEST FITNESS is {}. D22 requires FIT before any model \
                 compute, and a bar fixed after the numbers are visible is not a pre-registration.",
                self.fitness
            ),
        }
    }
}

/// Check one sheet's text.
pub fn check_sheet(path: &Path, text: &str) -> SheetReport {
    let secs = sections(text);
    let fitness = test_fitness(text);
    let blocks = REQUIRED_BLOCKS
        .iter()
        .map(|(key, name)| {
            let present = if *key == "fit" {
                fitness != ABSENT
            } else {
                let needle = heading_needle(key).unwrap_or("");
                secs.iter()
                    .any(|s| s.heading.to_ascii_lowercase().contains(needle))
            };
            Block { key, name, present }
        })
        .collect();
    SheetReport {
        path: path.to_path_buf(),
        blocks,
        fitness,
        run_planned: has_run_plan(text),
    }
}

/// Every sheet in the prep directory, oldest name first.
pub fn sheet_files(paths: &StatePaths) -> Vec<PathBuf> {
    let dir = paths.repo.join("docs").join("harness").join("prep");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    out.sort();
    out
}

/// Check every sheet on disk.
pub fn check_all(paths: &StatePaths) -> Vec<SheetReport> {
    sheet_files(paths)
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok().map(|t| check_sheet(p, &t)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // A sheet in the shape the harness writes, with every required block.
    const COMPLETE: &str = "\
# Pre-registration - M1

- date: 2026-10-13
- mechanism: rank-margin
- run: Rung 0 on the compliant trunk

## Bar
dev margin loss falls monotonically

## Base artifact
D10

## Data split
mw-balp, 2 seeds

## Steps
1. build

## Stop rule
no move after N steps

## Cost
$4

## Minimum test power
>= 5x the untrained rate

## TEST FITNESS
TEST FITNESS: FIT
";

    #[test]
    fn a_complete_sheet_reports_every_block() {
        let r = check_sheet(Path::new("s.md"), COMPLETE);
        assert!(r.complete(), "missing: {:?}", r.missing());
        assert_eq!(r.fitness, FIT);
        assert!(r.may_compute());
    }

    #[test]
    fn sections_split_at_heading_level() {
        let secs = sections(COMPLETE);
        let heads: Vec<&str> = secs.iter().map(|s| s.heading.as_str()).collect();
        assert!(heads.contains(&"Bar"), "{heads:?}");
        assert!(heads.contains(&"Minimum test power"), "{heads:?}");
        // a level-1 title is not a section
        assert!(
            !heads.iter().any(|h| h.starts_with("Pre-registration")),
            "{heads:?}"
        );
    }

    /// The distinction the gate turns on: completeness is the form being filled in, fitness is a
    /// reviewer saying the bar is testable. A complete sheet with no verdict is NOT cleared.
    #[test]
    fn a_complete_sheet_without_a_verdict_is_not_cleared() {
        let text = COMPLETE.replace("TEST FITNESS: FIT", "TEST FITNESS: PENDING");
        let r = check_sheet(Path::new("s.md"), &text);
        assert!(r.complete(), "the blocks are still there");
        assert_eq!(r.fitness, PENDING);
        assert!(!r.may_compute(), "PENDING is not FIT");
        assert!(r.verdict().contains("NOT cleared"), "{}", r.verdict());
    }

    #[test]
    fn a_missing_block_is_named() {
        let text = COMPLETE.replace("## Cost\n$4\n", "");
        let r = check_sheet(Path::new("s.md"), &text);
        assert!(!r.complete());
        assert_eq!(r.missing(), vec!["Cost"]);
    }

    /// NOT_FIT must not be read as FIT, and the verdict word is anchored.
    #[test]
    fn not_fit_is_not_read_as_fit() {
        let text = COMPLETE.replace("TEST FITNESS: FIT", "TEST FITNESS: NOT_FIT");
        assert_eq!(test_fitness(&text), NOT_FIT);
    }

    /// Prose that mentions the gate is not a verdict. Records and lessons do this constantly, and
    /// reading one as a verdict would let a sheet that merely *discusses* fitness pass the gate.
    #[test]
    fn prose_mentioning_the_gate_is_not_a_verdict() {
        assert_eq!(
            test_fitness("The `TEST FITNESS: FIT` string must appear on the sheet."),
            ABSENT
        );
        assert_eq!(test_fitness("see TEST FITNESS: FIT for details"), ABSENT);
        assert_eq!(
            test_fitness("| TEST FITNESS: FIT | yes |"),
            ABSENT,
            "a table cell is not the line"
        );
        assert_eq!(test_fitness("no verdict here"), ABSENT);
    }

    #[test]
    fn a_verdict_is_read_whatever_the_case_and_indent() {
        assert_eq!(test_fitness("  TEST FITNESS: fit"), FIT);
        assert_eq!(test_fitness("TEST FITNESS:FIT"), FIT);
        assert_eq!(test_fitness("\tTEST FITNESS:   NOT_FIT"), NOT_FIT);
    }

    #[test]
    fn fields_are_read_from_bulleted_and_bare_lines() {
        assert_eq!(field(COMPLETE, "mechanism"), "rank-margin");
        assert_eq!(
            field(COMPLETE, "Mechanism"),
            "rank-margin",
            "case-insensitive"
        );
        assert_eq!(field("- run: x\n", "run"), "x");
        assert_eq!(field("* Run: y\n", "run"), "y");
        assert_eq!(field(COMPLETE, "nothing"), "");
    }

    #[test]
    fn a_run_plan_is_recognised_by_a_named_run_or_a_block() {
        assert!(has_run_plan(COMPLETE), "the sheet names a run");
        assert!(has_run_plan("## Steps\nx\n"));
        let bare = "# sheet\n\n## Bar\ny\n";
        assert!(
            !has_run_plan(bare),
            "neither a run nor a steps/cost/run block"
        );
    }

    #[test]
    fn the_real_sheets_are_readable() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let sheets = check_all(&paths);
        if sheets.is_empty() {
            return; // a checkout with no sheets is not a failure
        }
        for s in &sheets {
            assert_ne!(
                s.fitness,
                ABSENT,
                "{}: no verdict line found",
                s.path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------------- rendering

/// The cost buckets a run must charge, in the session-goal contract's own breakdown.
pub const COST_BUCKETS: [&str; 6] = [
    "preparation",
    "run",
    "controls",
    "evaluation",
    "retries",
    "storage",
];

/// Owner rules that apply to every pre-registration.
///
/// **D31 and D32 were added the day they were issued, and that is the point.** D32 **supersedes
/// D25's floating-point allowance** and D31 governs what happens when an instrument check fails —
/// **both change what a pre-registration may propose.** A sheet that omitted them would send a lab
/// to work under a rule the owner had already replaced, **which is exactly the defect that put a
/// banned grader on a sheet**: the rule text a lab reads is the rule text it follows.
///
/// **Kept equal to Python's tuple, and `parity.render_agreement` is what keeps them equal.**
///
/// **A copy of Python's `prep.APPLICABLE_RULES`, and it drifted — which `parity.py` caught the moment
/// the Python list was corrected.** The Rust port carried `D21..D24` from before the port began while
/// the owner had added **D28** (the rule that blocks the v5 reply panel's neural grader) and **D29**.
/// A sheet rendered by Rust therefore **omitted the rule that explains why its own panel is
/// blocked** — the same defect Antigravity reported, surviving in the other implementation.
///
/// **Two lists of one rule is the defect class this whole line keeps producing.** The durable fix is
/// to render from the registry rather than a constant, so there is one list; until then, this one is
/// kept equal to Python's by `parity.render_agreement`, which is why the drift was found at all.
pub const APPLICABLE_RULES: [&str; 9] = [
    "D21", "D22", "D23", "D24", "D25", "D28", "D29", "D31", "D32",
];

/// D22 section 3: the minimum test power a decisive run must state.
pub const MIN_SEEDS: u32 = 2;
pub const MIN_ROWS: u32 = 200;

/// Owner caps (AGENTS.md "GPU pods").
pub const POD_CAP: u32 = 4;
pub const RATE_CAP_USD_PER_H: f64 = 8.00;
pub const STORAGE_MARGIN_MIB: u32 = 128;

/// What a lab supplies for a new sheet. Every field is optional; a missing one renders as
/// `MISSING`, which is the point — the sheet shows what still has to be decided.
#[derive(Debug, Clone, Default)]
pub struct NewSheet {
    /// Empty means "today", which the renderer fills in. A caller that has to supply the date is a
    /// caller that can get it wrong, and the date is part of the sheet's identity.
    pub date: String,
    pub mechanism: String,
    pub run: String,
    pub bar: String,
    pub base: String,
    pub split: String,
    pub steps: String,
    pub stop: String,
    pub cost: String,
    pub seeds: Option<u32>,
    pub rows: Option<u32>,
}

/// Render a `TEST FITNESS` review block (Python `prep fit`).
///
/// **The verdict clears compute for the configuration in that sheet ONLY** (D22), so the block names
/// the sheet and the milestone and carries that sentence. A review that does not say what it clears is
/// a review a lab can over-read.
pub fn render_fit(
    verdict: &str,
    milestone: &crate::registries::Milestone,
    sheet: &str,
    reviewer: &str,
    note: &str,
    mechanism: &str,
) -> String {
    let missing = |v: &str| {
        if v.trim().is_empty() {
            "MISSING".to_string()
        } else {
            v.trim().to_string()
        }
    };
    let mut out = String::new();
    out.push_str(&format!(
        "TEST FITNESS: {verdict} - {} #{}{}\n",
        milestone.id,
        milestone.issue,
        if mechanism.trim().is_empty() {
            String::new()
        } else {
            format!(" {mechanism}")
        }
    ));
    out.push_str(&format!("sheet: {}\n", missing(sheet)));
    out.push_str(&format!("reviewer: {}\n", missing(reviewer)));
    out.push_str(&format!("note: {}\n", missing(note)));
    out.push_str(&format!("date: {}\n", today()));
    out.push_str(
        "A FIT verdict clears compute for the configuration in that sheet only (D22). \n\n         Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.",
    );
    out
}

/// The pre-registration directory.
pub fn prep_dir(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("prep")
}

/// The default sheet path for a milestone and mechanism.
///
/// `<milestone-key>-<mechanism-or-no-mechanism>-<date>.md`, slugged — **the same name Python writes**,
/// because a lab that runs the Rust `prep new` and the Python one must land on one file rather than
/// two. Two names for one sheet is how a milestone ends up with a duplicate pre-registration and
/// `prep-check` reports on whichever it finds first.
pub fn sheet_name(milestone_key: &str, mechanism: &str, date: &str) -> String {
    let mech = if mechanism.trim().is_empty() {
        "no-mechanism"
    } else {
        mechanism
    };
    format!(
        "{}-{}-{}.md",
        crate::state::slug(milestone_key),
        crate::state::slug(mech),
        date
    )
}

/// Whether the sheet describes model work, which decides the D24 placement text.
///
/// A heuristic over the lab's own words, and deliberately crude: the alternative is asking for a
/// flag, and a lab that forgot the flag would get the wrong placement text on the one document that
/// exists to fix the plan before compute.
fn looks_like_model_work(s: &NewSheet) -> bool {
    let text = format!("{} {} {} {} {}", s.run, s.steps, s.bar, s.mechanism, s.stop).to_lowercase();
    [
        "train",
        "fit",
        "finetune",
        "fine-tune",
        "gpu",
        "cuda",
        "seed",
        "checkpoint",
    ]
    .iter()
    .any(|t| text.contains(t))
}

/// Render a pre-registration sheet.
///
/// Byte-for-byte the same text Python produces for the same inputs — that is the property the port
/// is verified on, because a sheet is a document two implementations must agree about. It is
/// compared in `parity.py`, not assumed.
pub fn render_sheet(
    sheet: &NewSheet,
    milestone: &crate::registries::Milestone,
    panel: Option<&crate::registries::Panel>,
    rules: &dyn Fn(&str) -> String,
    paths: &StatePaths,
) -> String {
    let missing = |v: &str| -> String {
        if v.trim().is_empty() {
            "MISSING".to_string()
        } else {
            v.to_string()
        }
    };
    let mechanism = sheet.mechanism.trim().to_string();
    let line = milestone.lines.first();
    let line_name = line
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "MISSING".into());
    let line_count = line
        .map(|l| crate::registries::fmt_num(&l.count))
        .unwrap_or_else(|| "MISSING".into());
    let line_of = line
        .map(|l| crate::registries::fmt_num(&l.of))
        .unwrap_or_else(|| "3".into());

    let mut out: Vec<String> = vec![
        format!("# Pre-registration - {} ({})", milestone.id, missing(&mechanism)),
        String::new(),
        format!("- date: {}", if sheet.date.trim().is_empty() { today() } else { sheet.date.trim().to_string() }),
        format!("- milestone: {} #{} - {}", milestone.id, milestone.issue, milestone.title),
        format!("- mechanism: {}", missing(&mechanism)),
        format!("- mechanism brief: {}", mechanism_path(paths, &mechanism)),
        format!("- run: {}", missing(&sheet.run)),
        format!(
            "- line: {line_name} - count {line_count}/{line_of} (D21/D22 count is owned by the milestone, not this sheet)"
        ),
        String::new(),
    ];

    // Bar
    out.push("## Bar (pre-registered)".into());
    if sheet.bar.trim().is_empty() {
        out.push("- bar: MISSING - a run without a pre-registered bar cannot be scored".into());
    } else {
        out.push(format!("- bar: {}", sheet.bar.trim()));
    }
    match panel {
        Some(p) => {
            out.push(format!(
                "- panel: `{}` (grader: {}, frozen: {})",
                p.id,
                if p.grader.is_empty() {
                    "MISSING"
                } else {
                    &p.grader
                },
                if p.frozen { "yes" } else { "NO" }
            ));
            out.push(format!(
                "- panel noise: {}",
                if p.noise.is_empty() {
                    "MISSING"
                } else {
                    &p.noise
                }
            ));
            out.extend(p.lines());
            out.push("- no bar in this sheet may be narrower than the panel noise above (D22 section 3).".into());
        }
        None => out.push("- panel: MISSING - the milestone names no frozen panel".into()),
    }

    out.push(String::new());
    out.push("## Base artifact".into());
    let base = if !sheet.base.trim().is_empty() {
        sheet.base.trim().to_string()
    } else if !milestone.base_artifact.is_empty() {
        milestone.base_artifact.clone()
    } else {
        "MISSING".into()
    };
    out.push(format!("- base: {base}"));
    out.push(String::new());
    out.push("## Data split".into());
    out.push(format!("- split: {}", missing(&sheet.split)));
    out.push(
        "- training data never includes the frozen panel rows; scoring uses the frozen panel and, for a KEEP, a fresh held-out draw (D22 section 4)"
            .into(),
    );
    out.push(String::new());
    out.push("## Steps".into());
    out.push(format!("- steps: {}", missing(&sheet.steps)));
    out.push(String::new());

    // Power (D22 section 3, explicit and checkable)
    let decisive = sheet.seeds.unwrap_or(0) >= MIN_SEEDS || sheet.rows.unwrap_or(0) >= MIN_ROWS;
    out.push("## Minimum test power (D22 section 3)".into());
    out.push(format!(
        "- seeds: {} (>= {MIN_SEEDS} seeds for a decisive run)",
        sheet
            .seeds
            .map(|v| v.to_string())
            .unwrap_or_else(|| "MISSING".into())
    ));
    out.push(format!(
        "- fresh rows: {} (>= {MIN_ROWS} fresh rows for a decisive run)",
        sheet
            .rows
            .map(|v| v.to_string())
            .unwrap_or_else(|| "MISSING".into())
    ));
    out.push(format!(
        "- decisive run: {} - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows{}",
        if decisive { "YES" } else { "NO" },
        if decisive {
            ""
        } else {
            " (raise --seeds or --rows before compute)"
        }
    ));
    out.push(format!(
        "- trained in rather than bolted on: {} - state the pretraining or the full fine-tune that gives the base time to adapt (a short add-on is a probe, not a decisive run)",
        if decisive { "YES" } else { "MISSING" }
    ));
    out.push("- fresh-transfer plan for a KEEP: MISSING - every development KEEP also reports a fresh, held-out transfer number (D22 section 4)".into());
    out.push("- noise floor: the bars above must not be narrower than the panel's stated noise; a single-row bar is not a bar".into());
    out.push(String::new());

    // Cost
    let token = missing(&sheet.cost);
    out.push("## Cost (charged cumulatively, before execution)".into());
    for (bucket, detail) in [
        ("preparation", "(context/data windows, threads, peak RAM)"),
        ("run", "(wall time and steps; the whole job, not the first attempt)"),
        ("controls", "(the no-mechanism control at the same recipe, steps and seeds)"),
        ("evaluation", "(each checkpoint scored, plus the fresh held-out draw for a KEEP)"),
        ("retries", "(diagnose, correct and retry within the remaining budget; a self-set budget is not a stop rule)"),
        ("storage", "(temporary, retained, and the 128 MiB storage stop margin)"),
    ] {
        debug_assert!(COST_BUCKETS.contains(&bucket));
        let _ = STORAGE_MARGIN_MIB;
        out.push(format!("- {bucket}: {token} {detail}"));
    }
    if looks_like_model_work(sheet) {
        out.push(
            "- placement (D24): model compute is GPU work only - lease the GPUs you use through `scripts/pod/uor-pod run POD --lab L --session S --gpu K -- CMD`; no CPU model compute and no host fallback (a fallback is a defect to fix, not a cost to report)".into(),
        );
    } else {
        out.push(
            "- placement (D24): no model compute is planned; CPU-bound preparation, grading and checks run on the owner's laptop, and no GPU is requested for work that cannot change a decision".into(),
        );
    }
    out.push("- pod/lease plan: MISSING - name the pod, the GPUs leased, `--hours` set to the whole job plus margin, the renew cadence, and the release-then-down step".into());
    out.push(format!(
        "- owner caps: <= {POD_CAP} running pods and <= ${RATE_CAP_USD_PER_H:.2}/h across all labs, all sessions; no new spending class without an owner decision (AGENTS.md, docs/labs/compute.md section Caps)"
    ));
    out.push(String::new());

    out.push("## Stop rule".into());
    out.push(format!("- stop: {}", missing(&sheet.stop)));
    out.push("- self-set step/wall budgets are estimates, not stop rules: extend and renew rather than cancel a healthy run; the only hard stops are the owner caps, the disk floor and a failed or diverged run".into());
    out.push(String::new());
    out.push("## Owner rules that apply".into());
    for rid in APPLICABLE_RULES {
        let text = rules(rid);
        if text.is_empty() {
            out.push(format!(
                "- {rid}: MISSING from docs/harness/milestones.json"
            ));
        } else {
            out.push(format!("- {rid}: {text}"));
        }
    }
    out.push("- D22: a negative closes this configuration only, never the mechanism; stopping a mechanism needs the owner.".into());
    out.push(String::new());
    out.push("## Panel target (milestone registry)".into());
    out.push(format!("- headline: {}", milestone.headline_line()));
    out.push(format!(
        "- panel: {}",
        if milestone.panel.is_empty() {
            "MISSING"
        } else {
            &milestone.panel
        }
    ));
    out.push(format!(
        "- source: docs/harness/milestones.json (issue #{})",
        milestone.issue
    ));
    out.push(String::new());
    out.push("## Verdict".into());
    out.push("TEST FITNESS: PENDING".into());

    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// The default filename for a new sheet: `<milestone>-<mechanism>-<date>.md`.
pub fn default_sheet_name(milestone_key: &str, mechanism: &str, date: &str) -> String {
    let m = crate::state::slug(milestone_key);
    let mech = if mechanism.trim().is_empty() {
        "no-mechanism".to_string()
    } else {
        crate::state::slug(mechanism)
    };
    format!("{m}-{mech}-{date}.md")
}

/// Where a mechanism's brief is, or why it is missing.
///
/// Python checks the file: a sheet that cites a brief which does not exist reads as prepared when it
/// is not, and the brief is where the mechanism's idea and the fair test are stated. My first
/// version returned the path whenever the mechanism name was non-empty — which is exactly the
/// "looks fine, is wrong" failure a pre-registration exists to prevent.
///
/// The known briefs are listed on a miss, because the usual cause is a name that does not match one.
fn mechanism_path(paths: &StatePaths, mechanism: &str) -> String {
    if mechanism.is_empty() {
        return "MISSING".to_string();
    }
    let rel = format!("docs/mechanisms/{mechanism}.md");
    let dir = paths.repo.join("docs").join("mechanisms");
    if dir.join(format!("{mechanism}.md")).is_file() {
        return rel;
    }
    let mut known: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect();
    known.sort();
    format!(
        "MISSING (no brief at {rel}; known: {})",
        if known.is_empty() {
            "none".to_string()
        } else {
            known.join(", ")
        }
    )
}

/// Today, in UTC, as `YYYY-MM-DD`.
///
/// UTC rather than local time, matching the shard-month rule: the sheet's date is part of its
/// filename and its identity, and two labs on different offsets must not disagree about which day a
/// pre-registration belongs to. The receipt keeps the local offset for readability.
pub fn today() -> String {
    crate::receipt::now_rfc3339()[..10].to_string()
}

#[cfg(test)]
mod applicable_rules_tests {
    use super::*;

    /// **The sheet must cite every rule the owner has in force**, and the two newest were added the
    /// day they were issued. A missing rule is not a cosmetic gap: D32 **supersedes** D25's
    /// floating-point allowance, so a sheet citing only D25 would send a lab to work under a
    /// replaced rule — the same failure that put a banned grader on a sheet.
    #[test]
    fn the_applicable_rules_include_the_current_owner_rules() {
        for rid in [
            "D21", "D22", "D23", "D24", "D25", "D28", "D29", "D31", "D32",
        ] {
            assert!(
                APPLICABLE_RULES.contains(&rid),
                "{rid} is an owner rule and must appear on every pre-registration"
            );
        }
        assert_eq!(
            APPLICABLE_RULES.len(),
            9,
            "the set is nine ids, asserted so a drift fails loudly"
        );
    }

    /// Every cited rule must have text in the registry, or the sheet prints a hole where a rule
    /// should be — which reads as "no rule applies" rather than "the registry is missing one".
    #[test]
    fn every_applicable_rule_has_registry_text() {
        use crate::registries::Registries;
        use crate::state::StatePaths;
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let ids: Vec<String> = regs
            .owner_rules
            .iter()
            .filter_map(|r| r.get("id").and_then(|i| i.as_str()).map(str::to_string))
            .collect();
        for rid in APPLICABLE_RULES {
            assert!(
                ids.iter().any(|i| i == rid),
                "{rid} is cited but has no text in the registry"
            );
        }
    }
}
