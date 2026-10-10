//! Structural controls for known-literal prospective role/bank composition panels.
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
fn literal(w: &Value) -> Result<String> {
    let text = w["text"]
        .as_str()
        .ok_or_else(|| invalid("diversity text absent"))?;
    let template = w["template"]
        .as_str()
        .ok_or_else(|| invalid("diversity template absent"))?;
    let (pre, post) = template
        .split_once("{v}")
        .ok_or_else(|| invalid("diversity slot absent"))?;
    if post.contains("{v}") {
        return Err(invalid("diversity multiple slots").into());
    }
    Ok(text
        .strip_prefix(pre)
        .and_then(|s| s.strip_suffix(post))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("diversity literal mismatch"))?
        .to_owned())
}
/// Checks counts, whole-bank family blocks and lexical role/position balance.
/// No model scores, geometric addresses or answer labels enter this control.
pub(super) fn validate(plan: &Value) -> Result<Value> {
    let split = plan["split"]
        .as_str()
        .ok_or_else(|| invalid("diversity split absent"))?;
    let expected = match split {
        "development" => 256,
        "fresh" => 64,
        _ => return Err(invalid("diversity split differs").into()),
    };
    let histories = plan["histories"]
        .as_array()
        .ok_or_else(|| invalid("diversity histories absent"))?;
    if histories.len() != expected {
        return Err(invalid("diversity fixed count differs").into());
    }
    let mut banks =
        BTreeMap::<(String, String), BTreeMap<(String, String), Vec<(String, Vec<String>)>>>::new();
    let mut role_counts = BTreeMap::<(String, String), usize>::new();
    let mut position_counts = BTreeMap::<(String, usize), usize>::new();
    let mut stratum_counts = BTreeMap::<String, usize>::new();
    let mut query_counts = BTreeMap::<(String, String), usize>::new();
    for h in histories {
        let turns = h["turns"]
            .as_array()
            .ok_or_else(|| invalid("diversity turns absent"))?;
        let queries = h["queries"]
            .as_array()
            .ok_or_else(|| invalid("diversity queries absent"))?;
        let stratum = h["stratum"]
            .as_str()
            .and_then(|s| s.split("/q").next())
            .ok_or_else(|| invalid("diversity stratum absent"))?;
        *stratum_counts.entry(stratum.to_owned()).or_default() += 1;
        if turns.len() < 2 || turns.len() > 8 {
            return Err(invalid("diversity turn bound differs").into());
        }
        if queries.len() != 2 {
            return Err(invalid("diversity requires both queries").into());
        }
        let mut current = BTreeMap::new();
        let mut chronology = Vec::new();
        for (position, w) in turns.iter().enumerate() {
            let role = w["relation"]
                .as_str()
                .filter(|r| matches!(*r, "job" | "home"))
                .ok_or_else(|| invalid("diversity role differs"))?;
            let value = literal(w)?;
            current.insert(role, value.clone());
            *role_counts
                .entry((value.clone(), role.to_owned()))
                .or_default() += 1;
            if position < 2 {
                *position_counts
                    .entry((value.clone(), position))
                    .or_default() += 1;
            }
            chronology.push(format!(
                "{role}/{}/{value}",
                w["act"]
                    .as_str()
                    .ok_or_else(|| invalid("diversity action absent"))?
            ));
        }
        let j = current
            .get("job")
            .ok_or_else(|| invalid("diversity job absent"))?
            .clone();
        let home = current
            .get("home")
            .ok_or_else(|| invalid("diversity home absent"))?
            .clone();
        if j == home {
            return Err(invalid("diversity equal role values").into());
        }
        let unordered = if j < home {
            (j.clone(), home.clone())
        } else {
            (home.clone(), j.clone())
        };
        let mut wording = BTreeMap::new();
        for q in queries {
            let role = q["relation"]
                .as_str()
                .ok_or_else(|| invalid("diversity query role absent"))?;
            let text = q["text"]
                .as_str()
                .ok_or_else(|| invalid("diversity query text absent"))?;
            if wording.insert(role, text).is_some() {
                return Err(invalid("diversity duplicate query role").into());
            }
            *query_counts
                .entry((role.to_owned(), text.to_owned()))
                .or_default() += 1;
        }
        if wording.keys().copied().collect::<BTreeSet<_>>() != BTreeSet::from(["job", "home"]) {
            return Err(invalid("diversity query roles differ").into());
        }
        let wordkey = serde_json::to_string(&wording)?;
        banks
            .entry(unordered)
            .or_default()
            .entry((j, home))
            .or_default()
            .push((wordkey, chronology));
    }
    let multiplier = if split == "development" { 4 } else { 1 };
    if stratum_counts
        != BTreeMap::from([
            ("length2".to_owned(), 16 * multiplier),
            ("length4".to_owned(), 16 * multiplier),
            ("length8".to_owned(), 16 * multiplier),
            ("update".to_owned(), 8 * multiplier),
            ("reassert".to_owned(), 8 * multiplier),
        ])
    {
        return Err(invalid("diversity declared stratum quotas differ").into());
    }
    for variants in banks.values() {
        if variants.len() != 2 || variants.values().any(|v| v.len() != 4) {
            return Err(invalid(
                "diversity requires both role assignments and four history variants each",
            )
            .into());
        }
        let mut role_wordsets = Vec::new();
        for histories in variants.values() {
            let mut groups = BTreeMap::<&str, Vec<&Vec<String>>>::new();
            for (q, t) in histories {
                groups.entry(q).or_default().push(t);
            }
            if groups.len() != 2 || groups.values().any(|v| v.len() != 2 || v[0][0] == v[1][0]) {
                return Err(invalid("diversity bank lacks crossed wording/order").into());
            }
            role_wordsets.push(groups.keys().copied().collect::<BTreeSet<_>>());
        }
        if role_wordsets[0] != role_wordsets[1] {
            return Err(invalid("diversity wording correlated with role assignment").into());
        }
    }
    for ((value, role), count) in &role_counts {
        let other = if role == "job" { "home" } else { "job" };
        if role_counts.get(&(value.clone(), other.to_owned())) != Some(count) {
            return Err(invalid("diversity literal role imbalance").into());
        }
    }
    for ((value, position), count) in &position_counts {
        if position_counts.get(&(value.clone(), 1 - position)) != Some(count) {
            return Err(invalid("diversity literal position imbalance").into());
        }
    }
    for role in ["job", "home"] {
        let counts = query_counts
            .iter()
            .filter(|((r, _), _)| r == role)
            .map(|(_, n)| *n)
            .collect::<Vec<_>>();
        if counts.len() != 6
            || counts.iter().max().copied().unwrap_or(0) - counts.iter().min().copied().unwrap_or(0)
                > 4
        {
            return Err(invalid("diversity six familiar query forms unbalanced").into());
        }
    }
    Ok(
        json!({"shared_literal_vertex_caps":{"length2":10,"length4":10,"length8":10,"update":10,"reassert":8},"candidate_order":"deterministic round-robin circle schedule over sorted capped shared exact donor literals; public exclusions only", "histories":histories.len(),"rows":histories.len()*2,"unordered_literal_bank_blocks":banks.len(),"ordered_role_value_banks":banks.len()*2,"wording_pairs_per_bank":2,"role_assignments_per_block":2,"chronology_orders_per_wording":2,"query_counts":query_counts.iter().map(|((role,text),count)|json!({"role":role,"text":text,"histories":count})).collect::<Vec<_>>(),"literal_role_balance":true,"literal_initial_position_balance":true,"scope":"known retained full literals; heldout bank combinations with familiar wording; no unseen-word/literal/paraphrase claim; no model or geometry selection"}),
    )
}

/// Ordered role/value identity ignores query wording and chronological order.
pub(super) fn bank_keys(plan: &Value) -> Result<BTreeSet<(String, String)>> {
    let mut keys = BTreeSet::new();
    for h in plan["histories"]
        .as_array()
        .ok_or_else(|| invalid("bank histories absent"))?
    {
        let mut current = BTreeMap::new();
        for w in h["turns"]
            .as_array()
            .ok_or_else(|| invalid("bank turns absent"))?
        {
            let role = w["relation"]
                .as_str()
                .ok_or_else(|| invalid("bank role absent"))?;
            current.insert(role, literal(w)?);
        }
        keys.insert((
            current
                .get("job")
                .ok_or_else(|| invalid("bank job absent"))?
                .clone(),
            current
                .get("home")
                .ok_or_else(|| invalid("bank home absent"))?
                .clone(),
        ));
    }
    Ok(keys)
}

/// Assertion context plus both current source records for every query.
pub(super) fn validate_all_sources(packet: &Value) -> Result<()> {
    let segments = packet["segments"]
        .as_array()
        .ok_or_else(|| invalid("diversity runtime segments absent"))?;
    if segments.len() != 4
        || segments[0]["kind"] != "Context"
        || segments[1]["kind"] != "Source"
        || segments[2]["kind"] != "Context"
        || segments[3]["kind"] != "Source"
    {
        return Err(invalid("diversity all-bank packet lost context/source candidate").into());
    }
    let relations = segments
        .iter()
        .filter(|s| s["kind"] == "Source")
        .filter_map(|s| s["relation"].as_u64())
        .collect::<BTreeSet<_>>();
    if relations != BTreeSet::from([1, 2]) {
        return Err(invalid("diversity all-bank packet filtered query role").into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allbank_retains_distractor_for_both_query_roles() -> Result<()> {
        let mut p = json!({"segments":[{"kind":"Context"},{"kind":"Source","relation":1},{"kind":"Context"},{"kind":"Source","relation":2}],"query_ids":[10]});
        validate_all_sources(&p)?;
        p["query_ids"] = json!([20]);
        validate_all_sources(&p)?;
        p["segments"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture segments absent"))?
            .truncate(2);
        assert!(validate_all_sources(&p).is_err());
        Ok(())
    }
}

/// Circle schedule enumerates every unordered edge once, spreading vertex degrees
/// within each round instead of creating a lexicographic first-literal hub.
pub(super) fn round_robin_edges(count: usize) -> Vec<(usize, usize)> {
    if count < 2 {
        return Vec::new();
    }
    let size = count + count % 2;
    let mut vertices = (0..size).collect::<Vec<_>>();
    let mut edges = Vec::new();
    for _ in 0..size - 1 {
        for i in 0..size / 2 {
            let (a, b) = (vertices[i], vertices[size - 1 - i]);
            if a < count && b < count {
                edges.push((a.min(b), a.max(b)));
            }
        }
        vertices[1..].rotate_right(1);
    }
    edges
}
/// Prospective evaluation holds out combinations, not entire full literals.
/// Reject an insufficient schedule explicitly; never redraw for model scores.
pub(super) fn validate_literal_coverage(development: &Value, fresh: &Value) -> Result<()> {
    fn values(plan: &Value) -> Result<BTreeSet<String>> {
        let mut values = BTreeSet::new();
        for h in plan["histories"]
            .as_array()
            .ok_or_else(|| invalid("coverage histories absent"))?
        {
            for w in h["turns"]
                .as_array()
                .ok_or_else(|| invalid("coverage turns absent"))?
            {
                values.insert(literal(w)?);
            }
        }
        Ok(values)
    }
    let training = values(development)?;
    let evaluation = values(fresh)?;
    if !evaluation.is_subset(&training) {
        return Err(invalid("diversity insufficient training full-literal coverage; fresh would confound unseen literals with new combinations; no redraw").into());
    }
    Ok(())
}
#[cfg(test)]
mod schedule_tests {
    use super::*;
    #[test]
    fn round_robin_spreads_first_round_and_exhausts_unique_edges() {
        for n in [8usize, 10] {
            let edges = round_robin_edges(n);
            assert_eq!(edges.len(), n * (n - 1) / 2);
            assert_eq!(
                edges.iter().copied().collect::<BTreeSet<_>>().len(),
                edges.len()
            );
            let mut degree = vec![0; n];
            for (a, b) in &edges[..n / 2] {
                degree[*a] += 1;
                degree[*b] += 1;
            }
            assert!(degree.iter().all(|d| *d == 1));
            for (a, b) in &edges[n / 2..n] {
                degree[*a] += 1;
                degree[*b] += 1;
            }
            assert!(degree.iter().all(|d| *d == 2));
        }
        assert_eq!(round_robin_edges(5).len(), 10);
    }
    #[test]
    fn fresh_full_literal_must_have_training_witness() -> Result<()> {
        let training = json!({"histories":[{"turns":[{"text":"Old amber willow.","template":"Old {v}."},{"text":"Old copper cedar.","template":"Old {v}."}]}]});
        let fresh = json!({"histories":[{"turns":[{"text":"New copper cedar.","template":"New {v}."},{"text":"New amber willow.","template":"New {v}."}]}]});
        validate_literal_coverage(&training, &fresh)?;
        let mut missing = fresh.clone();
        missing["histories"][0]["turns"][0]["text"] = json!("New previously absent.");
        assert!(validate_literal_coverage(&training, &missing).is_err());
        Ok(())
    }
}
