//! `panel score` — the D22 acceptance-panel check (D30 port of `prep.score_panel`).
//!
//! # The decision, and why AT NOISE exists
//!
//! `PASS` / `FAIL` / `AT NOISE`. **`AT NOISE` means the movement is inside the panel's stated noise
//! band, so it must not be reported as a real difference** — the measured value is still reported. A
//! bar may state its own noise, which then beats the panel's; a noise that cannot be parsed is `None`
//! rather than zero, because **a band of zero would turn every movement into a real one.**
//!
//! A forbid bar (target == current) is the exception the port copies rather than smooths: any movement
//! is a violation, not a difference to measure, **so noise does not excuse it.**

use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

use crate::registries::{Bar, Panel};

/// `util.MISSING` — the word the harness prints where a field is absent.
pub const MISSING: &str = "MISSING";

/// `registries.find_panel`: an exact id first, then the containment match the registry uses.
///
/// The second pass is deliberately loose — `v5-reply` and `m2-reply512` have both been used for the
/// same idea — and **it is why an unknown panel reports every known id rather than guessing.**
pub fn find_panel<'a>(panels: &'a [Panel], key: &str) -> Option<&'a Panel> {
    panels.iter().find(|p| p.id == key).or_else(|| {
        panels
            .iter()
            .find(|p| key.contains(&p.id) || p.id.contains(key))
    })
}

/// A JSON value as a number, the way Python's `float()` reads one.
fn num_of(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// `_noise_band`: the numeric half-width of a stated noise, or `None` when it is unstated.
///
/// Accepts the shapes the repo writes: `+-3/40`, `±3 rows`, `noise 0.01`.
pub fn noise_band(noise: &str) -> Option<f64> {
    if noise.is_empty() {
        return None;
    }
    static PLUS_MINUS: OnceLock<Regex> = OnceLock::new();
    let rx = PLUS_MINUS.get_or_init(|| {
        Regex::new(r"(?:\+/-|±|\+-)\s*([0-9]*\.?[0-9]+)").expect("a valid pattern")
    });
    if let Some(c) = rx.captures(noise) {
        return c.get(1).and_then(|m| m.as_str().parse::<f64>().ok());
    }
    static WORD: OnceLock<Regex> = OnceLock::new();
    let rx2 = WORD.get_or_init(|| {
        Regex::new(r"\bnoise\b[^0-9\-]*([0-9]*\.?[0-9]+)").expect("a valid pattern")
    });
    rx2.captures(noise)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
}

/// `_direction`: `higher` or `lower` — which way is better for this bar.
///
/// An undeclared direction falls back to comparing target against current, because **an undeclared
/// direction that defaulted to `higher` would let a regression read as a gain.**
pub fn bar_direction(bar: &Bar) -> String {
    let raw = bar.direction.trim().to_lowercase();
    if ["open", "higher", "up", "floor", "at least", ">="]
        .iter()
        .any(|d| *d == raw)
    {
        return "higher".to_string();
    }
    if [
        "ceiling",
        "lower",
        "down",
        "cap",
        "at most",
        "<=",
        "must be 0",
    ]
    .iter()
    .any(|d| *d == raw)
    {
        return "lower".to_string();
    }
    match (num_of(&bar.current), num_of(&bar.target)) {
        (Some(c), Some(t)) if t >= c => "higher".to_string(),
        (Some(_), Some(_)) => "lower".to_string(),
        _ => "higher".to_string(),
    }
}

/// The result of scoring one value against one frozen bar.
///
/// **The field order is Python's dict order**, because `--json` is compared byte for byte.
#[derive(Debug, Clone, PartialEq)]
pub struct BarScore {
    pub panel: String,
    pub bar: String,
    pub value: f64,
    pub current: Value,
    pub target: Value,
    pub direction: String,
    pub noise: String,
    pub noise_source: String,
    pub noise_band: Option<f64>,
    pub improvement: Option<f64>,
    pub needed: Option<f64>,
    pub outcome: String,
    pub reason: String,
    pub label: Option<String>,
}

/// `classify`: PASS / FAIL / AT NOISE for one measured value against one frozen bar.
pub fn classify(panel: &Panel, bar: &Bar, value: f64) -> BarScore {
    let direction = bar_direction(bar);
    let bar_noise = bar.noise.trim().to_string();
    let noise_text = if bar_noise.is_empty() {
        panel.noise.clone()
    } else {
        bar_noise.clone()
    };
    let band = noise_band(&noise_text);
    // Python's `bar.get("id", "?")`: absent is `?`, present-but-empty stays empty.
    let bar_id = if bar.id.is_empty() {
        "?".to_string()
    } else {
        bar.id.clone()
    };
    let mut out = BarScore {
        panel: panel.id.clone(),
        bar: bar_id.clone(),
        value,
        current: bar.current.clone(),
        target: bar.target.clone(),
        direction: direction.clone(),
        noise: if noise_text.is_empty() {
            "unstated".to_string()
        } else {
            noise_text.clone()
        },
        noise_source: if !bar_noise.is_empty() {
            format!("bar {bar_id}")
        } else if !panel.noise.is_empty() {
            "panel".to_string()
        } else {
            "unstated".to_string()
        },
        noise_band: band,
        improvement: None,
        needed: None,
        outcome: String::new(),
        reason: String::new(),
        label: None,
    };
    let (Some(cur), Some(tgt)) = (num_of(&bar.current), num_of(&bar.target)) else {
        out.outcome = "FAIL".to_string();
        out.reason = "the bar has no numeric current/target, so nothing can be scored against it"
            .to_string();
        return out;
    };
    let met = if direction == "higher" {
        value >= tgt
    } else {
        value <= tgt
    };
    let improvement = if direction == "higher" {
        value - cur
    } else {
        cur - value
    };
    let needed = if direction == "higher" {
        tgt - cur
    } else {
        cur - tgt
    };
    out.improvement = Some(improvement);
    out.needed = Some(needed);
    if let Some(band) = band {
        if needed != 0.0 && improvement.abs() <= band && improvement != 0.0 {
            out.outcome = "AT NOISE".to_string();
            out.reason = format!(
                "moved {} (bar needs {}); the stated noise is {noise_text}, so this is not a real difference",
                py_sign_g(improvement),
                py_sign_g(needed)
            );
            return out;
        }
        if needed != 0.0 && improvement == 0.0 {
            out.outcome = "AT NOISE".to_string();
            out.reason = format!(
                "no movement against a bar that needs {}; stated noise {noise_text}",
                py_sign_g(needed)
            );
            return out;
        }
    }
    if met {
        out.outcome = "PASS".to_string();
        out.reason = format!(
            "{} meets the frozen target {} (from {})",
            py_g(value),
            py_g(tgt),
            py_g(cur)
        );
    } else if needed == 0.0 {
        // A forbid bar (target == current, "must be 0"): any movement is a violation, not a
        // difference to measure, so noise does not excuse it.
        out.outcome = "FAIL".to_string();
        out.reason = format!(
            "{} violates a bar that must stay at {} (D23/D24 forbid rule); this is not a noisy improvement",
            py_g(value),
            py_g(tgt)
        );
    } else {
        out.outcome = "FAIL".to_string();
        out.reason = format!(
            "{} does not reach the frozen target {} (from {}); needs {}",
            py_g(value),
            py_g(tgt),
            py_g(cur),
            py_sign_g(needed)
        );
    }
    // **D22 section 3, and the line this port missed by reading only half the function.** With no
    // stated noise band there is nothing to compare a movement against, so a movement that is not
    // zero and a gap that is not zero cannot be called a real difference — and the reason says so.
    // The comparison against Python is what showed the missing sentence.
    if band.is_none() && improvement != 0.0 && needed != 0.0 {
        out.reason.push_str(
            " - noise is unstated for this bar, so this cannot be called a real difference (D22 section 3)",
        );
    }
    out
}

/// Python's `%g`: six significant digits, trailing zeros stripped, exponent form outside `1e-4..1e6`.
///
/// Written out because **a score line is compared character for character**: `37` must not print as
/// `37.0`, and `{value:g}` is what Python prints.
pub fn py_g(v: f64) -> String {
    if v.is_nan() {
        return "nan".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    if v == 0.0 {
        return if v.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    let abs = v.abs();
    let mut exp = abs.log10().floor() as i32;
    // `log10` can land either side of a power of ten; correct rather than trust it.
    if abs >= 10f64.powi(exp + 1) {
        exp += 1;
    }
    if abs < 10f64.powi(exp) {
        exp -= 1;
    }
    if !(-4..6).contains(&exp) {
        let mantissa = abs / 10f64.powi(exp);
        let mut m = format!("{mantissa:.5}");
        if m.contains('.') {
            m = m.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        let sign = if v < 0.0 { "-" } else { "" };
        format!(
            "{sign}{m}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    } else {
        let decimals = (5 - exp).max(0) as usize;
        let mut s = format!("{v:.decimals$}");
        if s.contains('.') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        s
    }
}

/// Python's `%+g`: the same, with an explicit sign — `+3`, `-0.5`.
pub fn py_sign_g(v: f64) -> String {
    if v < 0.0 || (v == 0.0 && v.is_sign_negative()) {
        format!("-{}", py_g(v.abs()))
    } else {
        format!("+{}", py_g(v.abs()))
    }
}

/// Python's `str()` for a JSON value, which is how a bar's raw current/target prints.
fn py_str_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// ``panel score``, as Python prints it.
pub fn render_score(panel: &Panel, bar: &Bar, s: &BarScore, value: f64) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}  {} / {} = {}\n",
        s.outcome,
        panel.id,
        s.bar,
        py_g(value)
    ));
    out.push_str(&format!("  {}\n", s.reason));
    // `.rstrip()` in Python: the unit is appended after a space, and a bar with no unit must not
    // leave a trailing blank.
    let bar_line = format!(
        "  bar {}: {} -> {} {}",
        bar.label,
        py_str_of(&bar.current),
        py_str_of(&bar.target),
        bar.unit
    );
    out.push_str(&format!("{}\n", bar_line.trim_end()));
    out.push_str(&format!("  noise used ({}): {}\n", s.noise_source, s.noise));
    out.push_str(&format!(
        "  grader: {}  source: {}\n",
        if panel.grader.is_empty() {
            MISSING
        } else {
            &panel.grader
        },
        panel.source.as_deref().unwrap_or(MISSING)
    ));
    if s.outcome == "AT NOISE" {
        out.push_str("  AT NOISE must never be reported as a real difference.\n");
    }
    out
}

/// A float as Python's `json.dumps` writes it, including the non-finite literals.
fn json_number(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    serde_json::Number::from_f64(f)
        .map(|n| n.to_string())
        .unwrap_or_else(|| f.to_string())
}

fn json_string(s: &str) -> String {
    Value::String(s.to_string()).to_string()
}

/// `panel score --json`: `json.dumps(result, indent=2, ensure_ascii=False)`.
///
/// Rendered by hand because the key order is Python's dict order and **one pair of keys is
/// conditional**: the non-numeric bar path returns before `improvement` and `needed` exist, so a
/// renderer that always emitted them would invent two fields the registry never produced.
pub fn score_json(s: &BarScore) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("  \"panel\": {}", json_string(&s.panel)));
    lines.push(format!("  \"bar\": {}", json_string(&s.bar)));
    lines.push(format!("  \"value\": {}", json_number(s.value)));
    lines.push(format!(
        "  \"current\": {}",
        serde_json::to_string(&s.current).unwrap_or_else(|_| "null".into())
    ));
    lines.push(format!(
        "  \"target\": {}",
        serde_json::to_string(&s.target).unwrap_or_else(|_| "null".into())
    ));
    lines.push(format!("  \"direction\": {}", json_string(&s.direction)));
    lines.push(format!("  \"noise\": {}", json_string(&s.noise)));
    lines.push(format!(
        "  \"noise_source\": {}",
        json_string(&s.noise_source)
    ));
    lines.push(format!(
        "  \"noise_band\": {}",
        match s.noise_band {
            Some(b) => json_number(b),
            None => "null".to_string(),
        }
    ));
    if let (Some(i), Some(n)) = (s.improvement, s.needed) {
        lines.push(format!("  \"improvement\": {}", json_number(i)));
        lines.push(format!("  \"needed\": {}", json_number(n)));
    }
    lines.push(format!("  \"outcome\": {}", json_string(&s.outcome)));
    lines.push(format!("  \"reason\": {}", json_string(&s.reason)));
    if let Some(label) = &s.label {
        lines.push(format!("  \"label\": {}", json_string(label)));
    }
    format!("{{\n{}\n}}", lines.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registries::{Bar, Panel};
    use serde_json::json;

    fn bar(id: &str, current: Value, target: Value, direction: &str, noise: &str) -> Bar {
        Bar {
            id: id.to_string(),
            label: "a bar".to_string(),
            current,
            target,
            unit: "rows".to_string(),
            direction: direction.to_string(),
            noise: noise.to_string(),
            ..Bar::default()
        }
    }

    fn panel(noise: &str) -> Panel {
        Panel {
            id: "p".to_string(),
            noise: noise.to_string(),
            grader: "g".to_string(),
            source: Some("s".to_string()),
            ..Panel::default()
        }
    }

    /// **A noise that cannot be read is `None`, not zero.** A band of zero would turn every movement
    /// into a real difference, which is the one thing the band exists to prevent.
    #[test]
    fn a_noise_band_is_read_from_the_shapes_the_repo_writes() {
        assert_eq!(noise_band("+-3/40"), Some(3.0));
        assert_eq!(noise_band("±3 rows"), Some(3.0));
        assert_eq!(noise_band("noise 0.01"), Some(0.01));
        assert_eq!(noise_band(""), None);
        assert_eq!(
            noise_band("unstated. No panel sigma is stated for this panel."),
            None
        );
    }

    #[test]
    fn direction_is_declared_or_derived_never_assumed_up() {
        assert_eq!(
            bar_direction(&bar("b", json!(0), json!(10), "open", "")),
            "higher"
        );
        assert_eq!(
            bar_direction(&bar("b", json!(0), json!(10), "must be 0", "")),
            "lower"
        );
        // Undeclared: derived from current vs target. A default of `higher` here would make a
        // regression read as a gain.
        assert_eq!(
            bar_direction(&bar("b", json!(10), json!(20), "", "")),
            "higher"
        );
        assert_eq!(
            bar_direction(&bar("b", json!(20), json!(10), "", "")),
            "lower"
        );
        // No numbers at all still answers, because a bar has to be scorable.
        assert_eq!(
            bar_direction(&bar("b", Value::Null, Value::Null, "", "")),
            "higher"
        );
    }

    #[test]
    fn a_pass_and_a_fail_carry_pythons_numbers() {
        let p = panel("+-5 rows");
        let b = bar("rows", json!(100), json!(150), "higher", "");
        let hit = classify(&p, &b, 150.0);
        assert_eq!(hit.outcome, "PASS");
        assert_eq!(hit.reason, "150 meets the frozen target 150 (from 100)");
        assert_eq!(hit.noise_source, "panel");
        let miss = classify(&p, &b, 120.0);
        assert_eq!(miss.outcome, "FAIL");
        assert_eq!(
            miss.reason,
            "120 does not reach the frozen target 150 (from 100); needs +50"
        );
    }

    /// The D22 section 3 sentence, which the first version of this port did not carry: with NO stated
    /// band there is nothing to compare a movement against.
    #[test]
    fn an_unstated_band_says_the_difference_cannot_be_called_real() {
        let p = panel("");
        let b = bar("rows", json!(100), json!(150), "higher", "");
        let s = classify(&p, &b, 120.0);
        assert_eq!(s.noise, "unstated");
        assert_eq!(s.noise_source, "unstated");
        assert!(s.reason.ends_with(
            " - noise is unstated for this bar, so this cannot be called a real difference (D22 section 3)"
        ),
        "{}",
        s.reason);
        // No movement, or no gap, and the sentence does not apply.
        let same = classify(&p, &b, 100.0);
        assert!(!same.reason.contains("D22 section 3"), "{}", same.reason);
    }

    #[test]
    fn a_movement_inside_the_band_is_at_noise() {
        let p = panel("+-5 rows");
        let b = bar("rows", json!(100), json!(150), "higher", "");
        let s = classify(&p, &b, 103.0);
        assert_eq!(s.outcome, "AT NOISE");
        assert!(
            s.reason.starts_with("moved +3 (bar needs +50);"),
            "{}",
            s.reason
        );
        let still = classify(&p, &b, 100.0);
        assert_eq!(still.outcome, "AT NOISE");
        assert!(still.reason.contains("no movement"), "{}", still.reason);
    }

    /// A bar's own noise beats the panel's, and a forbid bar is not excused by noise.
    #[test]
    fn a_bars_own_noise_wins_and_a_forbid_bar_is_not_excused() {
        let p = panel("+-50 rows");
        let own = bar("rows", json!(100), json!(150), "higher", "+-2 rows");
        // A movement of 1 is inside the BAR's ±2 band, where the panel's ±50 would have made it look
        // like nothing at all: the narrower band is the one that decides.
        let s = classify(&p, &own, 101.0);
        assert_eq!(s.noise_source, "bar rows");
        assert_eq!(s.noise_band, Some(2.0));
        assert_eq!(s.outcome, "AT NOISE");
        // 3 is outside the bar's own band, so it is scored rather than excused.
        assert_eq!(classify(&p, &own, 103.0).outcome, "FAIL");
        let forbid = bar("violations", json!(0), json!(0), "must be 0", "");
        let f = classify(&p, &forbid, 3.0);
        assert_eq!(f.outcome, "FAIL");
        assert!(f.reason.contains("forbid rule"), "{}", f.reason);
        assert_eq!(f.direction, "lower");
    }

    #[test]
    fn a_bar_with_no_numbers_fails_rather_than_guesses() {
        let p = panel("");
        let b = bar("rows", Value::Null, Value::Null, "higher", "");
        let s = classify(&p, &b, 1.0);
        assert_eq!(s.outcome, "FAIL");
        assert_eq!(
            s.reason,
            "the bar has no numeric current/target, so nothing can be scored against it"
        );
        // The improvement/needed keys are absent, and `score_json` must not invent them.
        assert_eq!(s.improvement, None);
        assert_eq!(s.needed, None);
        assert!(!score_json(&s).contains("improvement"));
    }

    #[test]
    fn percent_g_matches_python() {
        assert_eq!(py_g(37.0), "37");
        assert_eq!(py_g(37.5), "37.5");
        assert_eq!(py_g(0.0), "0");
        assert_eq!(py_g(-181.0), "-181");
        assert_eq!(py_g(1_000_000.0), "1e+06");
        assert_eq!(py_g(0.0001), "0.0001");
        assert_eq!(py_g(0.00001), "1e-05");
        assert_eq!(py_sign_g(3.0), "+3");
        assert_eq!(py_sign_g(-0.5), "-0.5");
        assert_eq!(py_sign_g(0.0), "+0");
    }

    #[test]
    fn the_json_is_pythons_dict_order() {
        let p = panel("+-5 rows");
        let b = bar("rows", json!(100), json!(150), "higher", "");
        let mut s = classify(&p, &b, 150.0);
        s.label = Some("seed-1".to_string());
        let text = score_json(&s);
        let keys: Vec<String> = text
            .lines()
            .filter(|l| l.trim_start().starts_with('"'))
            .filter_map(|l| l.trim().split('"').nth(1).map(str::to_string))
            .collect();
        assert_eq!(
            keys,
            vec![
                "panel",
                "bar",
                "value",
                "current",
                "target",
                "direction",
                "noise",
                "noise_source",
                "noise_band",
                "improvement",
                "needed",
                "outcome",
                "reason",
                "label",
            ]
        );
        assert!(text.contains("  \"value\": 150.0"), "{text}");
        assert!(text.contains("  \"improvement\": 50.0"), "{text}");
        assert!(text.starts_with("{\n  \"panel\""), "{text}");
    }

    #[test]
    fn the_score_line_is_what_python_prints() {
        let p = panel("+-5 rows");
        let b = bar("rows", json!(100), json!(150), "higher", "");
        let s = classify(&p, &b, 150.0);
        let text = render_score(&p, &b, &s, 150.0);
        assert_eq!(
            text,
            concat!(
                "PASS  p / rows = 150\n",
                "  150 meets the frozen target 150 (from 100)\n",
                "  bar a bar: 100 -> 150 rows\n",
                "  noise used (panel): +-5 rows\n",
                "  grader: g  source: s\n"
            )
        );
        // The AT NOISE warning is a line of its own, and only for that outcome.
        let at = classify(&p, &b, 103.0);
        assert!(render_score(&p, &b, &at, 103.0)
            .ends_with("  AT NOISE must never be reported as a real difference.\n"));
    }

    /// `find_panel` falls back to containment, so an unknown key reports every known id.
    #[test]
    fn a_panel_is_found_by_id_then_by_containment() {
        let a = Panel {
            id: "m2-reply512".to_string(),
            ..Panel::default()
        };
        let b = Panel {
            id: "v5-reply".to_string(),
            ..Panel::default()
        };
        let panels = vec![a, b];
        assert_eq!(
            find_panel(&panels, "v5-reply").map(|p| p.id.clone()),
            Some("v5-reply".to_string())
        );
        assert_eq!(
            find_panel(&panels, "reply").map(|p| p.id.clone()),
            Some("m2-reply512".to_string())
        );
        assert!(find_panel(&panels, "nope").is_none());
    }
}
