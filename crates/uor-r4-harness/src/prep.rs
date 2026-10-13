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
