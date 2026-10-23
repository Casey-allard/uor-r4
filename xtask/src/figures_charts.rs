//! `cargo xtask figures-charts [--write] [--catalog PATH] [--out DIR]`: the
//! documentation's data charts, drawn in Rust (D27) from the research catalog.
//!
//! Four figures, all following the docs figure style guide (theme variables with
//! a dark-mode override, Okabe-Ito palette, `<title>` and `<desc>` first, no
//! literal colour outside the `<style>` block):
//!
//! * `start/status-vs-baselines.svg`: the best trained model against two simple
//!   count tables, on one scale (constants below, with their sources);
//! * `research/results-share-by-week.svg`: per ISO week, the share of catalog
//!   records that are model results;
//! * `research/timeline.svg`: catalog records per day, under the five phases;
//! * `research-lines-timeline.svg`: swimlanes of the twelve largest research
//!   lines, one dot per dated record.
//!
//! Everything is deterministic. Without `--write` the task checks that each
//! file under `--out` (default `docs/figures`) is current, so CI catches a
//! stale chart or a hand edit. The catalog defaults to `archive/catalog.jsonl`.
//! Text never leaves the viewBox: each label's width is estimated and the task
//! fails instead of writing a clipped figure.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::docs_catalog::{parse_records, Record};

type Fail = Box<dyn std::error::Error>;

const DEFAULT_CATALOG: &str = "archive/catalog.jsonl";
const DEFAULT_OUT: &str = "docs/figures";

// ------------------------------------------------------------------ constants

/// Correct first guesses on the 14,287-position DEV panel.
/// Sources: `docs/labs/k16-outcome-call-2026-10-19/README.md` (Fields table:
/// variant A top-1, seed 1 = 65, seed 2 = 463) and
/// `docs/labs/host-diagnostics-2026-10-17/README.md` (unigram table 1,226 and
/// next-word table T2 6,300, both on the full 14,287 panel). Variant R (101 and
/// 230) is omitted: it collapsed to 19-20 of 256 cells.
const PANEL_POSITIONS: u64 = 14_287;
const M1_BAR: u64 = 6_300;

/// (label, sub-label, value, is_trained_model), top to bottom.
const BASELINE_BARS: [(&str, &str, u64, bool); 4] = [
    (
        "Next-word table (T2)",
        "previous token \u{2192} most common next",
        6_300,
        false,
    ),
    (
        "Unigram table",
        "always the most common token",
        1_226,
        false,
    ),
    ("Trained model, seed 2", "variant A", 463, true),
    ("Trained model, seed 1", "variant A", 65, true),
];

/// The five phases A to E of `docs/research/history.md`: (letter, first day,
/// last day, plain name, tone). Phases B and E overlap their neighbours by a day
/// or two, as the history page states them.
const PHASES: [(&str, &str, &str, &str, &str); 5] = [
    (
        "A",
        "2026-06-10",
        "2026-08-26",
        "Compile a teacher into tables",
        "blue",
    ),
    (
        "B",
        "2026-08-25",
        "2026-09-08",
        "Learned geometric decoder",
        "orange",
    ),
    (
        "C",
        "2026-09-08",
        "2026-09-25",
        "Native learner and exact memory",
        "green",
    ),
    (
        "D",
        "2026-09-26",
        "2026-10-11",
        "Geometric stack, labs, chat and memory",
        "purple",
    ),
    (
        "E",
        "2026-10-10",
        "2026-10-31",
        "Lawful operators and the discrete substrate",
        "verm",
    ),
];

/// The span of the timeline axis.
const TIMELINE_FIRST: &str = "2026-06-10";
const TIMELINE_LAST: &str = "2026-10-31";

/// How many research lines the swimlane chart shows.
const LANES: usize = 12;

// ----------------------------------------------------------------- dates

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    days_from_civil(
        if m == 12 { y + 1 } else { y },
        if m == 12 { 1 } else { m + 1 },
        1,
    ) - days_from_civil(y, m, 1)
}

/// `YYYY-MM-DD` to days since 1970-01-01.
fn parse_ymd(s: &str) -> Result<i64, String> {
    let bad = || format!("not a YYYY-MM-DD date: {s:?}");
    let mut it = s.split('-');
    let (y, m, d) = match (it.next(), it.next(), it.next(), it.next()) {
        (Some(y), Some(m), Some(d), None) => (y, m, d),
        _ => return Err(bad()),
    };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return Err(bad());
    }
    let (y, m, d) = (
        y.parse::<i64>().map_err(|_| bad())?,
        m.parse::<i64>().map_err(|_| bad())?,
        d.parse::<i64>().map_err(|_| bad())?,
    );
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return Err(bad());
    }
    Ok(days_from_civil(y, m, d))
}

/// 0 = Monday ... 6 = Sunday.
fn weekday(days: i64) -> i64 {
    (days + 3).rem_euclid(7)
}

/// ISO 8601 (year, week) of a day.
fn iso_week(days: i64) -> (i64, i64) {
    let thursday = days - weekday(days) + 3;
    let (year, _, _) = civil_from_days(thursday);
    (year, (thursday - days_from_civil(year, 1, 1)) / 7 + 1)
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// "12 Oct".
fn day_month(days: i64) -> String {
    let (_, m, d) = civil_from_days(days);
    format!("{d} {}", MONTHS[(m - 1) as usize])
}

// --------------------------------------------------------------- catalog data

/// Outcomes that count as a model result.
fn is_result(outcome: &str) -> bool {
    matches!(
        outcome,
        "positive" | "negative" | "inconclusive" | "diagnostic-stop"
    )
}

/// Integer percentage, rounded half up.
fn pct(part: usize, whole: usize) -> usize {
    (100 * part + whole / 2).checked_div(whole).unwrap_or(0)
}

struct Dated<'a> {
    day: i64,
    rec: &'a Record,
}

/// Dated records, and how many records have no date.
fn dated(records: &[Record]) -> Result<(Vec<Dated<'_>>, usize), Fail> {
    let mut out = Vec::new();
    let mut undated = 0;
    for rec in records {
        match &rec.date {
            Some(d) => out.push(Dated {
                day: parse_ymd(d).map_err(|e| format!("{}: {e}", rec.path))?,
                rec,
            }),
            None => undated += 1,
        }
    }
    if out.is_empty() {
        return Err("the catalog has no dated record".into());
    }
    Ok((out, undated))
}

struct Week {
    monday: i64,
    results: usize,
    total: usize,
    tooling: usize,
}

/// One row per ISO week from the first to the last dated record, gaps included.
fn weekly(dated: &[Dated]) -> Vec<Week> {
    let mut by_monday: BTreeMap<i64, (usize, usize, usize)> = BTreeMap::new();
    for d in dated {
        let e = by_monday.entry(d.day - weekday(d.day)).or_default();
        e.1 += 1;
        if is_result(&d.rec.outcome) {
            e.0 += 1;
        }
        if d.rec.outcome == "tooling" {
            e.2 += 1;
        }
    }
    let (first, last) = match (by_monday.keys().next(), by_monday.keys().next_back()) {
        (Some(a), Some(b)) => (*a, *b),
        _ => return Vec::new(),
    };
    (0..=(last - first) / 7)
        .map(|i| {
            let monday = first + 7 * i;
            let (results, total, tooling) = by_monday.get(&monday).copied().unwrap_or_default();
            Week {
                monday,
                results,
                total,
                tooling,
            }
        })
        .collect()
}

// -------------------------------------------------------------------- svg

const CANVAS: f64 = 960.0;
const EDGE: f64 = 16.0;
const FONT: &str = "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif";
const TONES: [&str; 13] = [
    "ink", "body", "muted", "rule", "panel", "bg", "blue", "sky", "green", "orange", "verm",
    "purple", "pink",
];

/// Text style: size, weight 600 or not, colour tone.
#[derive(Clone, Copy)]
struct Ty {
    size: f64,
    bold: bool,
    tone: &'static str,
}

const TITLE: Ty = Ty {
    size: 20.0,
    bold: true,
    tone: "ink",
};
const SUB: Ty = Ty {
    size: 12.0,
    bold: false,
    tone: "muted",
};
const LBL: Ty = Ty {
    size: 13.0,
    bold: false,
    tone: "body",
};
const LBL_B: Ty = Ty {
    size: 13.0,
    bold: true,
    tone: "ink",
};
const SMALL: Ty = Ty {
    size: 12.0,
    bold: false,
    tone: "body",
};
const NOTE: Ty = Ty {
    size: 11.0,
    bold: false,
    tone: "muted",
};

#[derive(Clone, Copy)]
enum Anchor {
    Start,
    Middle,
    End,
}

/// Number with at most one decimal and no trailing `.0`.
fn n(v: f64) -> String {
    if v.abs() < 0.05 {
        return "0".to_string();
    }
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

/// 1234567 to "1,234,567".
fn comma(v: u64) -> String {
    let digits = v.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Rough rendered width of `s`: 0.55 x size per character, 0.6 when bold.
fn text_width(s: &str, size: f64, bold: bool) -> f64 {
    s.chars().count() as f64 * size * if bold { 0.6 } else { 0.55 }
}

struct Doc {
    name: &'static str,
    h: f64,
    body: String,
    overflow: Vec<String>,
}

impl Doc {
    fn new(name: &'static str, h: f64) -> Doc {
        Doc {
            name,
            h,
            body: String::new(),
            overflow: Vec::new(),
        }
    }

    fn text(&mut self, x: f64, y: f64, ty: Ty, anchor: Anchor, s: &str) {
        let w = text_width(s, ty.size, ty.bold);
        let (x0, x1, a) = match anchor {
            Anchor::Start => (x, x + w, "start"),
            Anchor::Middle => (x - w / 2.0, x + w / 2.0, "middle"),
            Anchor::End => (x - w, x, "end"),
        };
        if x0 < EDGE || x1 > CANVAS - EDGE {
            self.overflow.push(format!(
                "{}: \"{s}\" needs x {x0:.0}..{x1:.0}, the figure allows {EDGE}..{}",
                self.name,
                CANVAS - EDGE
            ));
        }
        let b = if ty.bold { " b" } else { "" };
        let _ = writeln!(
            self.body,
            r#"<text x="{}" y="{}" font-size="{}" class="t-{}{b}" text-anchor="{a}">{}</text>"#,
            n(x),
            n(y),
            n(ty.size),
            ty.tone,
            esc(s)
        );
    }

    fn rect(&mut self, [x, y, w, h]: [f64; 4], rx: f64, fill: &str, stroke: Option<(&str, f64)>) {
        let st = stroke
            .map(|(t, sw)| format!(r#" class="f-{fill} s-{t}" stroke-width="{}""#, n(sw)))
            .unwrap_or_else(|| format!(r#" class="f-{fill}""#));
        let _ = writeln!(
            self.body,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}"{st}/>"#,
            n(x),
            n(y),
            n(w),
            n(h),
            n(rx)
        );
    }

    /// An unfilled rectangle outline.
    fn outline(&mut self, [x, y, w, h]: [f64; 4], rx: f64, tone: &str, sw: f64) {
        let _ = writeln!(
            self.body,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="none" class="s-{tone}" stroke-width="{}"/>"#,
            n(x),
            n(y),
            n(w),
            n(h),
            n(rx),
            n(sw)
        );
    }

    /// A rectangle with partial fill opacity.
    fn tint(
        &mut self,
        [x, y, w, h]: [f64; 4],
        fill: &str,
        opacity: f64,
        stroke: Option<(&str, f64)>,
    ) {
        let st = stroke
            .map(|(t, sw)| format!(r#" s-{t}" stroke-width="{}"#, n(sw)))
            .unwrap_or_default();
        let _ = writeln!(
            self.body,
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill-opacity="{opacity}" class="f-{fill}{st}"/>"#,
            n(x),
            n(y),
            n(w),
            n(h)
        );
    }

    fn line(&mut self, [x1, y1, x2, y2]: [f64; 4], tone: &str, width: f64, dash: Option<&str>) {
        let d = dash
            .map(|d| format!(r#" stroke-dasharray="{d}""#))
            .unwrap_or_default();
        let _ = writeln!(
            self.body,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="s-{tone}" stroke-width="{}"{d}/>"#,
            n(x1),
            n(y1),
            n(x2),
            n(y2),
            n(width)
        );
    }

    fn raw(&mut self, s: &str) {
        self.body.push_str(s);
        self.body.push('\n');
    }

    /// Title and subtitle at the top.
    fn heading(&mut self, title: &str, sub: &str) {
        self.text(EDGE, 34.0, TITLE, Anchor::Start, title);
        self.text(EDGE, 54.0, SUB, Anchor::Start, sub);
    }

    fn finish(self, title: &str, desc: &str) -> Result<String, Fail> {
        if !self.overflow.is_empty() {
            return Err(format!("text does not fit:\n  {}", self.overflow.join("\n  ")).into());
        }
        let mut out = String::new();
        let _ = writeln!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-labelledby="t d">"#,
            w = CANVAS,
            h = n(self.h)
        );
        let _ = writeln!(out, r#"<title id="t">{}</title>"#, esc(title));
        let _ = writeln!(out, r#"<desc id="d">{}</desc>"#, esc(desc));
        out.push_str(&style());
        let _ = writeln!(
            out,
            r#"<rect class="bg" x="0" y="0" width="{}" height="{}"/>"#,
            CANVAS,
            n(self.h)
        );
        out.push_str(&self.body);
        out.push_str("</svg>\n");
        Ok(out)
    }
}

/// The theme: light variables, the dark override, and one class per use.
fn style() -> String {
    let mut s = String::from("<style>\n");
    s.push_str(concat!(
        "svg { --bg:#ffffff; --ink:#1f2328; --body:#2b3138; --muted:#57606a; --rule:#d0d7de; --panel:#f6f8fa;\n",
        "      --blue:#0072b2; --sky:#56b4e9; --green:#007a5e; --orange:#e69f00; --verm:#d55e00; --purple:#7a4fa3; --pink:#cc79a7; }\n",
        "@media (prefers-color-scheme: dark) { svg { --bg:#0d1117; --ink:#e6edf3; --body:#c9d1d9; --muted:#8b949e; --rule:#30363d; --panel:#161b22;\n",
        "      --blue:#58a6ff; --sky:#79c0ff; --green:#3fb950; --orange:#f0b72f; --verm:#ff7b72; --purple:#bc8cff; --pink:#f778ba; } }\n",
    ));
    let _ = writeln!(s, ".bg {{ fill: var(--bg); }}");
    let _ = writeln!(s, "text {{ font-family: {FONT}; fill: var(--ink); }}");
    let _ = writeln!(s, ".b {{ font-weight: 600; }}");
    for t in TONES {
        let _ = writeln!(
            s,
            ".t-{t} {{ fill: var(--{t}); }} .f-{t} {{ fill: var(--{t}); }} .s-{t} {{ stroke: var(--{t}); }}"
        );
    }
    s.push_str("</style>\n");
    s
}

// --------------------------------------------------------- 1. baselines chart

fn status_vs_baselines() -> Result<String, Fail> {
    let mut d = Doc::new("status-vs-baselines", 440.0);
    d.heading(
        "Correct first guesses on 14,287 development-panel positions",
        "The best trained model against two simple count tables, on one scale. Higher is better.",
    );
    // Legend.
    d.rect([EDGE, 68.0, 16.0, 12.0], 2.0, "blue", None);
    d.text(EDGE + 24.0, 79.0, LBL, Anchor::Start, "Trained model");
    d.rect([160.0, 68.0, 16.0, 12.0], 2.0, "muted", None);
    d.text(184.0, 79.0, LBL, Anchor::Start, "Count table");

    let (left, right) = (270.0, 900.0);
    let scale = (right - left) / PANEL_POSITIONS as f64;
    let x = |v: u64| left + v as f64 * scale;
    let (top, pitch, bar_h) = (124.0, 58.0, 34.0);
    let axis_y = top + pitch * 3.0 + bar_h + 16.0;

    for tick in [0, 5_000, 10_000, PANEL_POSITIONS] {
        d.line([x(tick), top - 12.0, x(tick), axis_y], "rule", 1.0, None);
        d.text(x(tick), axis_y + 18.0, NOTE, Anchor::Middle, &comma(tick));
    }
    d.text(
        (left + right) / 2.0,
        axis_y + 36.0,
        LBL,
        Anchor::Middle,
        "correct first guesses out of 14,287",
    );

    for (i, (label, sub, value, trained)) in BASELINE_BARS.iter().enumerate() {
        let y = top + pitch * i as f64;
        d.text(EDGE, y + 15.0, LBL_B, Anchor::Start, label);
        d.text(EDGE, y + 30.0, NOTE, Anchor::Start, sub);
        let w = (*value as f64 * scale).max(2.0);
        if *trained {
            d.rect([left, y, w, bar_h], 2.0, "blue", None);
        } else {
            d.tint([left, y, w, bar_h], "muted", 0.7, None);
        }
        d.text(
            left + w + 8.0,
            y + 22.0,
            LBL_B,
            Anchor::Start,
            &comma(*value),
        );
    }

    // The new M1 bar.
    let bx = x(M1_BAR);
    d.line([bx, top - 12.0, bx, axis_y], "verm", 2.0, Some("6 4"));
    d.text(
        bx + 8.0,
        top - 24.0,
        LBL,
        Anchor::Start,
        "new M1 bar: beat both tables, on each of two seeds",
    );

    let fy = axis_y + 64.0;
    d.text(
        EDGE,
        fy,
        NOTE,
        Anchor::Start,
        "Variant R (101 and 230) is omitted: it collapsed to 19 or 20 of 256 cells.",
    );
    d.text(
        EDGE,
        fy + 16.0,
        NOTE,
        Anchor::Start,
        "Source: docs/labs/k16-outcome-call-2026-10-19; docs/labs/host-diagnostics-2026-10-17.",
    );
    d.finish(
        "Trained model against count tables",
        "Horizontal bars on one scale from 0 to 14,287 development-panel positions. Next-word table 6,300, unigram table 1,226, trained model seed 2 463, trained model seed 1 65. A dashed line at 6,300 marks the new M1 bar: beat both tables on each of two seeds. The trained model is far below both tables.",
    )
}

// ------------------------------------------------------- 2. results by week

fn results_share_by_week(records: &[Record]) -> Result<String, Fail> {
    let (dated, undated) = dated(records)?;
    let weeks = weekly(&dated);
    let first_day = dated.iter().map(|d| d.day).min().unwrap_or(0);
    let last_day = dated.iter().map(|d| d.day).max().unwrap_or(0);

    let mut d = Doc::new("results-share-by-week", 548.0);
    d.heading(
        "Share of each week's records that were model results",
        "Per ISO week, from the research catalog. Results: positive, negative, inconclusive or diagnostic stop.",
    );

    let (left, right) = (56.0, 944.0);
    let (top, bottom) = (124.0, 384.0);
    let slot = (right - left) / weeks.len() as f64;
    let bar_w = (slot * 0.64).min(48.0);
    let y_of = |p: f64| bottom - (bottom - top) * p;

    // Legend.
    d.rect([EDGE, 70.0, 16.0, 12.0], 2.0, "blue", None);
    d.text(EDGE + 24.0, 81.0, LBL, Anchor::Start, "model results");
    d.rect([150.0, 70.0, 16.0, 12.0], 2.0, "rule", Some(("muted", 1.0)));
    d.text(
        174.0,
        81.0,
        LBL,
        Anchor::Start,
        "everything else (tooling, plans, process, review, ops)",
    );

    // Grid and the 50% reference.
    for p in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let y = y_of(p);
        if (p - 0.5_f64).abs() < 1e-9 {
            d.line([left, y, right, y], "ink", 1.5, Some("5 4"));
        } else {
            d.line([left, y, right, y], "rule", 1.0, None);
        }
        d.text(
            left - 8.0,
            y + 4.0,
            NOTE,
            Anchor::End,
            &format!("{}%", (p * 100.0).round() as u32),
        );
    }

    let mut busiest: Option<usize> = None; // the largest week, annotated below
    for (i, w) in weeks.iter().enumerate() {
        if busiest.is_none_or(|j| w.total > weeks[j].total) {
            busiest = Some(i);
        }
    }

    for (i, w) in weeks.iter().enumerate() {
        let cx = left + slot * (i as f64 + 0.5);
        let x0 = cx - bar_w / 2.0;
        if w.total > 0 {
            let share = w.results as f64 / w.total as f64;
            let split = y_of(share);
            d.rect(
                [x0, top, bar_w, split - top],
                0.0,
                "rule",
                Some(("muted", 1.0)),
            );
            if w.results > 0 {
                d.rect([x0, split, bar_w, bottom - split], 0.0, "blue", None);
            }
            d.text(
                cx,
                top - 8.0,
                LBL_B,
                Anchor::Middle,
                &format!("{}%", pct(w.results, w.total)),
            );
        } else {
            d.text(cx, top - 8.0, NOTE, Anchor::Middle, "none");
        }
        let (_, wk) = iso_week(w.monday);
        let partial = w.monday < first_day || w.monday + 6 > last_day;
        let star = if partial { "*" } else { "" };
        d.text(
            cx,
            bottom + 18.0,
            LBL_B,
            Anchor::Middle,
            &format!("W{wk}{star}"),
        );
        d.text(
            cx,
            bottom + 33.0,
            NOTE,
            Anchor::Middle,
            &day_month(w.monday),
        );
        d.text(
            cx,
            bottom + 48.0,
            NOTE,
            Anchor::Middle,
            &format!("n = {}", w.total),
        );
    }

    // Annotate the busiest week.
    if let Some(i) = busiest {
        let w = &weeks[i];
        let cx = left + slot * (i as f64 + 0.5);
        d.outline(
            [
                cx - slot / 2.0 + 2.0,
                100.0,
                slot - 4.0,
                bottom + 60.0 - 100.0,
            ],
            4.0,
            "verm",
            2.0,
        );
        let (_, wk) = iso_week(w.monday);
        d.text(
            right,
            92.0,
            LBL_B,
            Anchor::End,
            &format!(
                "W{wk}: {} results among {} records; {} of the records are tooling",
                w.results, w.total, w.tooling
            ),
        );
    }

    let notes = [
        "Part of the drop after 11 October was ordered by the owner (the softmax and matmul bans and the GPU-only and Rust-first rules forced a rebuild",
        "and many tool ports on 12 and 13 October), so the later weeks are an upper bound on discretionary process, not a verdict.",
        &format!(
            "Compare shares, not counts: early rows mix loose documents and lane results. * partial week: the catalog runs from {} to {}.",
            day_month(first_day),
            day_month(last_day)
        ),
        &format!(
            "Source: the research catalog, {} dated records; {} records have no date and are not shown.",
            dated.len(),
            undated
        ),
    ];
    for (k, line) in notes.iter().enumerate() {
        d.text(
            EDGE,
            bottom + 82.0 + 16.0 * k as f64,
            NOTE,
            Anchor::Start,
            line,
        );
    }

    let shares: Vec<String> = weeks
        .iter()
        .map(|w| format!("{}%", pct(w.results, w.total)))
        .collect();
    let desc = format!(
        "Stacked bars, one per ISO week from {} to {}, each split into model results and everything else, with the results share above each bar: {}. A dashed line marks 50%. The share falls sharply in the busiest week.",
        day_month(weeks.first().map_or(0, |w| w.monday)),
        day_month(last_day),
        shares.join(", ")
    );
    d.finish("Model-result share of records by week", &desc)
}

// ------------------------------------------------------------ 3. timeline

/// "Nice" y-axis step and ceiling for a maximum.
fn nice(max: usize) -> (usize, usize) {
    let step = [5, 10, 20, 25, 50, 100, 200, 500]
        .into_iter()
        .find(|s| max.div_ceil(*s) <= 5)
        .unwrap_or(1000);
    (step, max.max(1).div_ceil(step) * step)
}

fn timeline(records: &[Record]) -> Result<String, Fail> {
    let (dated, undated) = dated(records)?;
    let d0 = parse_ymd(TIMELINE_FIRST)?;
    let d1 = parse_ymd(TIMELINE_LAST)?;
    if let Some(r) = dated.iter().find(|r| !(d0..=d1).contains(&r.day)) {
        return Err(format!(
            "a catalog record dated outside the timeline window {TIMELINE_FIRST} to {TIMELINE_LAST} would be left off the chart: {}; extend TIMELINE_LAST and phase E",
            r.rec.path
        )
        .into());
    }
    let span = (d1 - d0 + 1) as f64;
    let (left, right) = (64.0, 944.0);
    let x = |day: i64| left + (day - d0) as f64 / span * (right - left);

    let mut per_day: BTreeMap<i64, usize> = BTreeMap::new();
    for r in &dated {
        if (d0..=d1).contains(&r.day) {
            *per_day.entry(r.day).or_default() += 1;
        }
    }
    let first_rec = per_day.keys().next().copied().unwrap_or(d0);
    let last_rec = per_day.keys().next_back().copied().unwrap_or(d1);
    let (peak_day, peak) = per_day
        .iter()
        .max_by_key(|(day, c)| (**c, std::cmp::Reverse(**day)))
        .map(|(a, b)| (*a, *b))
        .unwrap_or((d0, 0));
    let (step, ymax) = nice(peak);

    let mut d = Doc::new("timeline", 560.0);
    d.heading(
        "Five approaches in sequence, and the records each day left behind",
        "Catalog records per day, 10 June to 19 October 2026, under the five phases A to E.",
    );

    // Phase bands in two lanes so the overlaps stay readable.
    let mut legend = Vec::new();
    for (i, (letter, a, b, name, tone)) in PHASES.iter().enumerate() {
        let (s, e) = (parse_ymd(a)?, parse_ymd(b)?);
        let lane = if i % 2 == 0 { 76.0 } else { 100.0 };
        let (bx, bw) = (x(s), x(e + 1) - x(s));
        d.tint([bx, lane, bw, 20.0], tone, 0.28, Some((tone, 1.5)));
        d.text(bx + bw / 2.0, lane + 15.0, LBL_B, Anchor::Middle, letter);
        legend.push((*letter, *tone, *name, s, e));
    }

    // Plot area.
    let (top, bottom) = (150.0, 330.0);
    let y = |c: usize| bottom - (bottom - top) * c as f64 / ymax as f64;
    d.text(left, top - 10.0, NOTE, Anchor::Start, "records per day");
    let mut tick = 0;
    while tick <= ymax {
        d.line([left, y(tick), right, y(tick)], "rule", 1.0, None);
        d.text(
            left - 8.0,
            y(tick) + 4.0,
            NOTE,
            Anchor::End,
            &comma(tick as u64),
        );
        tick += step;
    }
    // Mondays, faintly; months, labelled.
    for day in d0..=d1 {
        if weekday(day) == 0 {
            d.line([x(day), top, x(day), bottom], "rule", 0.5, None);
        }
    }

    // The stretch before the first catalog record.
    if first_rec > d0 {
        d.tint(
            [x(d0), top, x(first_rec) - x(d0), bottom - top],
            "panel",
            1.0,
            None,
        );
        d.text(
            (x(d0) + x(first_rec)) / 2.0,
            (top + bottom) / 2.0,
            NOTE,
            Anchor::Middle,
            "GitHub items only",
        );
        d.text(
            (x(d0) + x(first_rec)) / 2.0,
            (top + bottom) / 2.0 + 14.0,
            NOTE,
            Anchor::Middle,
            "no catalog records",
        );
    }

    // The area.
    let mut path = format!("M{},{}", n(x(first_rec)), n(bottom));
    for day in first_rec..=last_rec {
        let c = per_day.get(&day).copied().unwrap_or(0);
        let _ = write!(
            path,
            " L{},{} L{},{}",
            n(x(day)),
            n(y(c)),
            n(x(day + 1)),
            n(y(c))
        );
    }
    let _ = write!(path, " L{},{} Z", n(x(last_rec + 1)), n(bottom));
    d.raw(&format!(
        r#"<path d="{path}" fill-opacity="0.35" stroke-width="1.5" stroke-linejoin="round" class="f-blue s-blue"/>"#
    ));
    d.line([left, bottom, right, bottom], "muted", 1.0, None);

    // Peak label.
    let label = format!("peak: {peak} records on {}", day_month(peak_day));
    let lw = text_width(&label, NOTE.size, false);
    let px = x(peak_day) + (x(peak_day + 1) - x(peak_day)) / 2.0;
    if px + 6.0 + lw > CANVAS - EDGE {
        d.text(px - 6.0, y(peak) - 6.0, NOTE, Anchor::End, &label);
    } else {
        d.text(px + 6.0, y(peak) - 6.0, NOTE, Anchor::Start, &label);
    }

    // Axis labels.
    d.text(x(d0), bottom + 18.0, NOTE, Anchor::Start, &day_month(d0));
    for day in d0..=d1 {
        let (_, _, dd) = civil_from_days(day);
        if dd == 1 {
            d.line([x(day), bottom, x(day), bottom + 5.0], "muted", 1.0, None);
            d.text(x(day), bottom + 18.0, LBL, Anchor::Middle, &day_month(day));
        }
    }
    d.text(x(d1 + 1), bottom + 18.0, NOTE, Anchor::End, &day_month(d1));

    // Phase key.
    let mut ky = 392.0;
    d.text(EDGE, ky - 14.0, LBL_B, Anchor::Start, "Phases");
    for (letter, tone, name, s, e) in &legend {
        d.tint([EDGE, ky - 11.0, 24.0, 16.0], tone, 0.28, Some((tone, 1.5)));
        d.text(EDGE + 12.0, ky + 1.0, SMALL, Anchor::Middle, letter);
        d.text(
            EDGE + 36.0,
            ky + 1.0,
            LBL,
            Anchor::Start,
            &format!("{name}  ({} to {})", day_month(*s), day_month(*e)),
        );
        ky += 24.0;
    }

    d.text(
        EDGE,
        ky + 20.0,
        NOTE,
        Anchor::Start,
        "Phase spans are those of the research history page; phases overlap by a day or two where one hands over to the next.",
    );
    d.text(
        EDGE,
        ky + 36.0,
        NOTE,
        Anchor::Start,
        &format!(
            "Daily counts: the research catalog, {} dated records; {} records have no date and are not shown.",
            dated.len(),
            undated
        ),
    );
    let h = ky + 56.0;
    d.h = h;
    let desc = format!(
        "A time axis from 10 June to 19 October 2026. Two rows of bands mark the five phases A to E. Below, an area chart of catalog records per day: nothing before {} (GitHub items only), then a rise to a peak of {} records on {}.",
        day_month(first_rec),
        peak,
        day_month(peak_day)
    );
    d.finish("Phases and weekly activity, June to October 2026", &desc)
}

// ----------------------------------------------------- 4. research lines

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Other,
    Negative,
    Positive,
}

fn class_of(outcome: &str) -> Class {
    match outcome {
        "positive" => Class::Positive,
        "negative" => Class::Negative,
        _ => Class::Other,
    }
}

fn research_lines_timeline(records: &[Record]) -> Result<String, Fail> {
    let (dated, undated) = dated(records)?;
    let d0 = dated.iter().map(|r| r.day).min().unwrap_or(0);
    let d1 = dated.iter().map(|r| r.day).max().unwrap_or(0);

    // The largest lines by record count (all records, dated or not).
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in records {
        *counts.entry(r.line.as_str()).or_default() += 1;
    }
    let mut order: Vec<(&str, usize)> = counts.iter().map(|(k, v)| (*k, *v)).collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let shown: Vec<(&str, usize)> = order.iter().take(LANES).copied().collect();
    let hidden_lines = order.len().saturating_sub(LANES);
    let hidden_records: usize = order.iter().skip(LANES).map(|(_, c)| c).sum();
    let undated_shown = records
        .iter()
        .filter(|r| r.date.is_none() && shown.iter().any(|(l, _)| *l == r.line))
        .count();
    let shown_records: usize = shown.iter().map(|(_, c)| c).sum();

    let (label_w, left, right) = (250.0, 262.0, 944.0);
    let span = (d1 - d0 + 1) as f64;
    let x = |day: i64| left + (day - d0) as f64 / span * (right - left);
    let (top, pitch, lane_h) = (112.0, 42.0, 38.0);
    let bottom = top + pitch * shown.len() as f64;

    let mut d = Doc::new("research-lines-timeline", bottom + 118.0);
    d.heading(
        "When each research line ran",
        &format!(
            "The {} largest research lines by record count, one dot per dated record, {} to {}.",
            shown.len(),
            day_month(d0),
            day_month(d1)
        ),
    );

    // Legend: shape and colour.
    let mut lx = EDGE;
    let ly = 84.0;
    d.raw(&format!(
        r#"<circle cx="{}" cy="{}" r="3.4" class="f-blue"/>"#,
        n(lx + 4.0),
        n(ly - 4.0)
    ));
    lx += 16.0;
    d.text(lx, ly, LBL, Anchor::Start, "result: positive");
    lx += 140.0;
    d.raw(&format!(
        r#"<path d="M{},{} l4,4 l-4,4 l-4,-4 z" class="f-verm"/>"#,
        n(lx + 4.0),
        n(ly - 8.0)
    ));
    lx += 16.0;
    d.text(lx, ly, LBL, Anchor::Start, "result: negative");
    lx += 140.0;
    d.raw(&format!(
        r#"<circle cx="{}" cy="{}" r="2.6" class="s-muted" stroke-width="1.2"/>"#,
        n(lx + 4.0),
        n(ly - 4.0)
    ));
    lx += 16.0;
    d.text(
        lx,
        ly,
        LBL,
        Anchor::Start,
        "anything else: plans, tooling, process, review, inconclusive and so on",
    );

    // Month grid under the lanes.
    d.text(x(d0), bottom + 20.0, NOTE, Anchor::Start, &day_month(d0));
    d.text(x(d1 + 1), bottom + 20.0, NOTE, Anchor::End, &day_month(d1));
    for day in d0..=d1 {
        let (_, _, dd) = civil_from_days(day);
        if dd == 1 {
            d.line(
                [x(day), top - 4.0, x(day), bottom],
                "rule",
                1.0,
                Some("2 3"),
            );
            d.line([x(day), bottom, x(day), bottom + 5.0], "muted", 1.0, None);
            d.text(x(day), bottom + 20.0, LBL, Anchor::Middle, &day_month(day));
        }
    }

    // Lanes and dots.
    let mut dots: BTreeMap<Class, String> = BTreeMap::new();
    for (i, (line, count)) in shown.iter().enumerate() {
        let y0 = top + pitch * i as f64;
        let cy = y0 + lane_h / 2.0;
        d.rect([left, y0, right - left, lane_h], 4.0, "panel", None);
        d.text(EDGE, cy + 4.0, LBL, Anchor::Start, line);
        d.text(
            label_w - 4.0,
            cy + 4.0,
            NOTE,
            Anchor::End,
            &comma(*count as u64),
        );

        // Group by day; a busy day stacks in a small grid.
        let mut by_day: BTreeMap<i64, Vec<(Class, &str)>> = BTreeMap::new();
        for r in dated.iter().filter(|r| r.rec.line == *line) {
            by_day
                .entry(r.day)
                .or_default()
                .push((class_of(&r.rec.outcome), r.rec.path.as_str()));
        }
        for (day, mut recs) in by_day {
            recs.sort();
            let cx = x(day) + (x(day + 1) - x(day)) / 2.0;
            for (k, (class, _)) in recs.iter().enumerate() {
                let px = cx + ((k / 8) % 3) as f64 * 2.4 - 2.4;
                let py = cy - 14.0 + (k % 8) as f64 * 4.0;
                let s = dots.entry(*class).or_default();
                match class {
                    Class::Positive => {
                        let _ = writeln!(s, r#"<circle cx="{}" cy="{}" r="2.2"/>"#, n(px), n(py));
                    }
                    Class::Negative => {
                        let _ = writeln!(
                            s,
                            r#"<path d="M{},{} l2.8,2.8 l-2.8,2.8 l-2.8,-2.8 z"/>"#,
                            n(px),
                            n(py - 2.8)
                        );
                    }
                    Class::Other => {
                        let _ = writeln!(s, r#"<circle cx="{}" cy="{}" r="1.6"/>"#, n(px), n(py));
                    }
                }
            }
        }
    }
    if let Some(s) = dots.get(&Class::Other) {
        let _ = writeln!(
            d.body,
            "<g class=\"s-muted\" stroke-width=\"1\" fill=\"none\">\n{s}</g>"
        );
    }
    if let Some(s) = dots.get(&Class::Negative) {
        let _ = writeln!(d.body, "<g class=\"f-verm\" fill-opacity=\"0.9\">\n{s}</g>");
    }
    if let Some(s) = dots.get(&Class::Positive) {
        let _ = writeln!(d.body, "<g class=\"f-blue\" fill-opacity=\"0.9\">\n{s}</g>");
    }

    d.text(
        EDGE,
        bottom + 46.0,
        NOTE,
        Anchor::Start,
        "Records of one line on one day stack in a small grid; a very busy day overlaps. The number at the left of each lane is the line's record count.",
    );
    d.text(
        EDGE,
        bottom + 62.0,
        NOTE,
        Anchor::Start,
        &format!(
            "Not shown: {hidden_lines} smaller lines ({hidden_records} records), and {undated_shown} undated records of the lines above. Catalog: {} records, {undated} without a date.",
            records.len()
        ),
    );
    d.text(
        EDGE,
        bottom + 78.0,
        NOTE,
        Anchor::Start,
        &format!(
            "Lines shown hold {shown_records} of the catalog's {} records.",
            records.len()
        ),
    );
    let names: Vec<&str> = shown.iter().map(|(l, _)| *l).collect();
    let desc = format!(
        "A timeline with one row per research line for the {} largest lines, {}. Each dot is one dated catalog record on its date: blue circles are positive results, vermilion diamonds negative results, hollow grey circles everything else. Notice where each line starts and ends and where records pile up.",
        shown.len(),
        names.join(", ")
    );
    d.finish("Research lines on a timeline", &desc)
}

// -------------------------------------------------------------------- task

/// Every figure: its path under the output folder and its SVG.
fn render(records: &[Record]) -> Result<Vec<(&'static str, String)>, Fail> {
    Ok(vec![
        ("start/status-vs-baselines.svg", status_vs_baselines()?),
        (
            "research/results-share-by-week.svg",
            results_share_by_week(records)?,
        ),
        ("research/timeline.svg", timeline(records)?),
        (
            "research-lines-timeline.svg",
            research_lines_timeline(records)?,
        ),
    ])
}

fn resolve(root: &Path, p: &str) -> PathBuf {
    let p = Path::new(p);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

pub fn run(root: &Path, args: &[String]) -> Result<(), Fail> {
    let mut write = false;
    let mut catalog = DEFAULT_CATALOG.to_string();
    let mut out = DEFAULT_OUT.to_string();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--write" => write = true,
            "--catalog" => catalog = it.next().ok_or("--catalog needs a path")?.clone(),
            "--out" => out = it.next().ok_or("--out needs a directory")?.clone(),
            other => {
                return Err(format!(
                    "unknown argument {other:?}; usage: figures-charts [--write] [--catalog PATH] [--out DIR]"
                )
                .into())
            }
        }
    }
    let catalog_path = resolve(root, &catalog);
    let text = std::fs::read_to_string(&catalog_path)
        .map_err(|e| format!("{}: {e}", catalog_path.display()))?;
    let records = parse_records(&text)?;
    let out_dir = resolve(root, &out);

    let figures = render(&records)?;
    let mut stale = Vec::new();
    for (rel, svg) in &figures {
        let path = out_dir.join(rel);
        if write {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, svg)?;
            println!("wrote {}", path.display());
        } else if !std::fs::read_to_string(&path)
            .is_ok_and(|c| crate::figures_math::svg_matches(&c, svg))
        {
            stale.push(*rel);
        }
    }
    if stale.is_empty() {
        if !write {
            println!(
                "figures-charts: all {} chart figures are current",
                figures.len()
            );
        }
        Ok(())
    } else {
        Err(format!(
            "stale chart figures (run `cargo xtask figures-charts --write`): {}",
            stale.join(", ")
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(date: Option<&str>, line: &str, outcome: &str, path: &str) -> String {
        let date = date.map_or("null".to_string(), |d| format!("\"{d}\""));
        format!(
            r#"{{"path":"{path}","title":"t","date":{date},"lab":"x","line":"{line}","outcome":"{outcome}","mechanisms":[],"prs":[]}}"#
        )
    }

    fn sample() -> Vec<Record> {
        let rows = [
            rec(Some("2026-07-21"), "a", "positive", "p1"),
            rec(Some("2026-07-22"), "a", "tooling", "p2"),
            rec(Some("2026-10-18"), "b", "negative", "p3"),
            rec(Some("2026-10-19"), "b", "diagnostic-stop", "p4"),
            rec(Some("2026-10-19"), "b", "plan", "p5"),
            rec(None, "b", "process", "p6"),
        ];
        parse_records(&rows.join("\n")).expect("sample parses")
    }

    #[test]
    fn a_record_outside_the_timeline_window_is_refused_not_dropped() {
        assert!(timeline(&sample()).is_ok());
        let late = [rec(Some("2026-11-01"), "b", "positive", "late")];
        let records = parse_records(&late.join("\n")).expect("parses");
        let err = timeline(&records).expect_err("a date after the window must fail");
        assert!(err.to_string().contains("late"), "{err}");
    }

    #[test]
    fn a_record_from_late_october_is_inside_the_window() {
        // Loop PR 8 archives two pages dated 2026-10-22; the window must chart them.
        let rows = [rec(Some("2026-10-22"), "harness-governance", "ops", "STATUS.md")];
        let records = parse_records(&rows.join("\n")).expect("parses");
        assert!(timeline(&records).is_ok());
    }

    #[test]
    fn iso_weeks_match_the_calendar() {
        let w = |s: &str| iso_week(parse_ymd(s).expect("date"));
        assert_eq!(w("2026-10-19"), (2026, 43));
        assert_eq!(w("2026-10-18"), (2026, 42));
        assert_eq!(w("2026-07-21"), (2026, 30));
        assert_eq!(w("2026-01-01"), (2026, 1));
        assert_eq!(w("2027-01-01"), (2026, 53));
        assert_eq!(w("2024-12-30"), (2025, 1));
        assert_eq!(w("2021-01-03"), (2020, 53));
        assert_eq!(weekday(parse_ymd("2026-10-19").expect("date")), 0);
        let day = parse_ymd("2026-02-28").expect("date");
        assert_eq!(civil_from_days(day + 1), (2026, 3, 1));
        assert!(parse_ymd("2026-02-30").is_err());
        assert!(parse_ymd("2026-1-05").is_err());
    }

    #[test]
    fn weekly_buckets_count_results_and_fill_gaps() {
        let records = sample();
        let (dated, undated) = dated(&records).expect("dated");
        assert_eq!(undated, 1);
        let weeks = weekly(&dated);
        // W30 (Mon 20 Jul) through W43 (Mon 19 Oct).
        assert_eq!(weeks.len(), 14);
        assert_eq!(
            (weeks[0].results, weeks[0].total, weeks[0].tooling),
            (1, 2, 1)
        );
        assert_eq!((weeks[12].results, weeks[12].total), (1, 1)); // W42: 18 Oct is a Sunday
        assert_eq!((weeks[13].results, weeks[13].total), (1, 2)); // W43: 19 Oct
        assert_eq!(weeks[5].total, 0);
    }

    #[test]
    fn shares_round_half_up_and_survive_empty_weeks() {
        assert_eq!(pct(34, 309), 11);
        assert_eq!(pct(4, 32), 13);
        assert_eq!(pct(0, 5), 0);
        assert_eq!(pct(0, 0), 0);
        assert_eq!(pct(5, 5), 100);
        assert!(is_result("diagnostic-stop") && !is_result("tooling") && !is_result("plan"));
    }

    #[test]
    fn numbers_format_stably() {
        assert_eq!(comma(14_287), "14,287");
        assert_eq!(comma(65), "65");
        assert_eq!(comma(1_000), "1,000");
        assert_eq!(n(3.0), "3");
        assert_eq!(n(3.26), "3.3");
        assert_eq!(n(-0.01), "0");
        assert_eq!(nice(309), (100, 400));
        assert_eq!(nice(5), (5, 5));
    }

    #[test]
    fn rendering_is_deterministic_and_themed() {
        let records = sample();
        let a = render(&records).expect("render");
        let b = render(&records).expect("render");
        assert_eq!(a.len(), 4);
        for ((pa, sa), (pb, sb)) in a.iter().zip(&b) {
            assert_eq!(pa, pb);
            assert_eq!(sa, sb, "{pa} differs between runs");
            assert!(sa.contains("prefers-color-scheme: dark"), "{pa}");
            assert!(sa.contains("<title") && sa.contains("<desc"), "{pa}");
            assert!(
                !sa.contains("<script") && !sa.contains("foreignObject"),
                "{pa}"
            );
            // No literal colour outside the style block.
            let after = sa.split("</style>").nth(1).expect("style block");
            assert!(
                !after
                    .as_bytes()
                    .windows(4)
                    .any(|w| w[0] == b'#' && w[1..].iter().all(u8::is_ascii_hexdigit)),
                "{pa} has a literal colour outside <style>"
            );
        }
    }

    #[test]
    fn a_catalog_without_dates_is_refused() {
        let rows = rec(None, "a", "plan", "p");
        let records = parse_records(&rows).expect("parses");
        assert!(results_share_by_week(&records).is_err());
    }

    #[test]
    fn overlong_text_is_an_error_not_a_clipped_figure() {
        let mut d = Doc::new("t", 100.0);
        d.text(EDGE, 20.0, LBL, Anchor::Start, &"x".repeat(200));
        assert!(d.finish("t", "d").is_err());
    }
}
