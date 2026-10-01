//! A reviewed derivative of raw teacher paraphrases (#1552). The label audits
//! found inherited-label errors in the raw batches, so a traceable review
//! keeps, relabels or drops each raw row; raw files are never changed.
//!
//! Raw files are named `p1`, `p2`, … in order, and a row is `pN:LINE`
//! (one-based). A decision line is tab-separated: `id`, `decision` (`keep`,
//! `relabel`, `drop` or `duplicate`), the new `relation` and `act` (relabel
//! only), a `reason`, and the row's exact `text`. [`apply`] refuses unless:
//! - every raw row has exactly one decision whose text equals the raw text;
//! - a row is `duplicate` exactly when an earlier row has identical text, and
//!   its reason names that first row;
//! - every kept or relabelled row satisfies the slot rule (one `{v}` in a
//!   statement, none in a query) and a relabel changes a label.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::milestone_world_v2::relation_names;
use crate::{invalid, Result};

const SLOT: &str = "{v}";
/// The acts a paraphrase can carry.
pub const ACTS: [&str; 3] = ["assert", "update", "query"];

/// A raw paraphrase row.
#[derive(Clone, Debug)]
pub struct RawRow {
    /// `pN:LINE`.
    pub id: String,
    pub relation: String,
    pub act: String,
    pub text: String,
    pub source_template: Value,
    /// The digest of the raw file holding the row.
    pub file_sha256: String,
}

/// The rows of a raw paraphrase file named `name` (`p1`, …).
pub fn raw_rows(name: &str, content: &str, file_sha256: &str) -> Result<Vec<RawRow>> {
    content
        .lines()
        .enumerate()
        .map(|(line, text)| {
            let row: Value = serde_json::from_str(text)?;
            let id = format!("{name}:{}", line + 1);
            let field = |key: &str| -> Result<String> {
                row[key]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid(format!("{id} has no {key}")))
            };
            Ok(RawRow {
                relation: field("relation")?,
                act: field("act")?,
                text: field("text")?,
                source_template: row["source_template"].clone(),
                file_sha256: file_sha256.to_owned(),
                id,
            })
        })
        .collect()
}

/// One review decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub kind: String,
    pub relation: String,
    pub act: String,
    pub reason: String,
    pub text: String,
}

/// The decisions of a decision file, by row id.
pub fn parse_decisions(tsv: &str) -> Result<BTreeMap<String, Decision>> {
    let mut decisions = BTreeMap::new();
    for (line, entry) in tsv.lines().enumerate() {
        let fields: Vec<&str> = entry.split('\t').collect();
        let [id, kind, relation, act, reason, text] = fields[..] else {
            return Err(invalid(format!(
                "decision line {} needs 6 tab-separated fields",
                line + 1
            )));
        };
        let decision = Decision {
            kind: kind.to_owned(),
            relation: relation.to_owned(),
            act: act.to_owned(),
            reason: reason.to_owned(),
            text: text.to_owned(),
        };
        if decisions.insert(id.to_owned(), decision).is_some() {
            return Err(invalid(format!("{id} has two decisions")));
        }
    }
    Ok(decisions)
}

/// The derivative: its rows (JSON lines that load wherever raw paraphrases
/// do) and its tallies.
#[derive(Debug)]
pub struct Review {
    pub rows: Vec<Value>,
    pub decisions: BTreeMap<String, usize>,
    pub relabels: BTreeMap<String, usize>,
    pub by_label: BTreeMap<String, usize>,
}

/// Whether labels and text satisfy the slot rule.
fn check_labels(id: &str, relation: &str, act: &str, text: &str) -> Result<()> {
    if !relation_names().contains(&relation) {
        return Err(invalid(format!("{id}: unknown relation {relation}")));
    }
    if !ACTS.contains(&act) {
        return Err(invalid(format!("{id}: unknown act {act}")));
    }
    let slots = text.matches(SLOT).count();
    let wanted = usize::from(act != "query");
    if slots != wanted {
        return Err(invalid(format!(
            "{id}: a {act} must hold {wanted} slot(s), found {slots}"
        )));
    }
    Ok(())
}

/// Apply `decisions` to `raws` (in file then line order).
pub fn apply(raws: &[RawRow], decisions: &BTreeMap<String, Decision>) -> Result<Review> {
    let ids: BTreeSet<&str> = raws.iter().map(|raw| raw.id.as_str()).collect();
    if let Some(stray) = decisions.keys().find(|id| !ids.contains(id.as_str())) {
        return Err(invalid(format!("{stray} names no raw row")));
    }
    let mut first: BTreeMap<&str, &str> = BTreeMap::new();
    let mut review = Review {
        rows: Vec::new(),
        decisions: BTreeMap::new(),
        relabels: BTreeMap::new(),
        by_label: BTreeMap::new(),
    };
    for raw in raws {
        let id = raw.id.as_str();
        let decision = decisions
            .get(id)
            .ok_or_else(|| invalid(format!("{id} has no decision")))?;
        if decision.text != raw.text {
            return Err(invalid(format!(
                "{id}: the decision's text differs from the raw row"
            )));
        }
        if decision.reason.trim().is_empty() {
            return Err(invalid(format!("{id}: a decision needs a reason")));
        }
        let labels = match (
            first.get(raw.text.as_str()).copied(),
            decision.kind.as_str(),
        ) {
            (Some(earlier), "duplicate")
                if decision.reason == format!("duplicate of {earlier}") =>
            {
                None
            }
            (Some(earlier), "duplicate") => {
                return Err(invalid(format!(
                    "{id}: duplicates {earlier}, not as recorded"
                )))
            }
            (Some(earlier), _) => {
                return Err(invalid(format!(
                    "{id}: identical to {earlier}, so it must be a duplicate"
                )))
            }
            (None, "duplicate") => {
                return Err(invalid(format!(
                    "{id}: marked duplicate but its text is new"
                )))
            }
            (None, "keep") => Some((raw.relation.as_str(), raw.act.as_str())),
            (None, "relabel") => {
                if (decision.relation.as_str(), decision.act.as_str())
                    == (raw.relation.as_str(), raw.act.as_str())
                {
                    return Err(invalid(format!("{id}: a relabel must change a label")));
                }
                *review
                    .relabels
                    .entry(format!(
                        "{}/{} -> {}/{}",
                        raw.relation, raw.act, decision.relation, decision.act
                    ))
                    .or_default() += 1;
                Some((decision.relation.as_str(), decision.act.as_str()))
            }
            (None, "drop") => None,
            (_, other) => return Err(invalid(format!("{id}: unknown decision {other}"))),
        };
        if let Some((relation, act)) = labels {
            check_labels(id, relation, act, &raw.text)?;
            *review
                .by_label
                .entry(format!("{relation}/{act}"))
                .or_default() += 1;
            review.rows.push(json!({
                "relation": relation,
                "act": act,
                "text": raw.text,
                "source_template": raw.source_template,
                "source": id,
                "source_sha256": raw.file_sha256,
                "review": if decision.reason == "-" { Value::Null } else { json!(decision.reason) },
            }));
        }
        first.entry(raw.text.as_str()).or_insert(id);
        *review.decisions.entry(decision.kind.clone()).or_default() += 1;
    }
    Ok(review)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raws() -> Result<Vec<RawRow>> {
        let p1 = concat!(
            r#"{"relation":"user_name","act":"assert","text":"I am called {v}.","source_template":"My name is {v}."}"#,
            "\n",
            r#"{"relation":"user_name","act":"update","text":"I'm referred to as {v}.","source_template":"Actually, my name is {v}."}"#,
            "\n",
            r#"{"relation":"hometown","act":"update","text":"Sorry, I was born in {v}.","source_template":"Sorry, I grew up in {v}."}"#,
            "\n",
        );
        let p2 = concat!(
            r#"{"relation":"user_name","act":"assert","text":"I am called {v}.","source_template":"My name is {v}."}"#,
            "\n",
            r#"{"relation":"user_name","act":"query","text":"What's my name?","source_template":"What is my name?"}"#,
            "\n",
        );
        let mut rows = raw_rows("p1", p1, &"a".repeat(64))?;
        rows.extend(raw_rows("p2", p2, &"b".repeat(64))?);
        Ok(rows)
    }

    const DECISIONS: &str = concat!(
        "p1:1\tkeep\t\t\t-\tI am called {v}.\n",
        "p1:2\trelabel\tuser_name\tassert\tno correction cue\tI'm referred to as {v}.\n",
        "p1:3\tdrop\t\t\tmeaning: birthplace\tSorry, I was born in {v}.\n",
        "p2:1\tduplicate\t\t\tduplicate of p1:1\tI am called {v}.\n",
        "p2:2\tkeep\t\t\t-\tWhat's my name?\n",
    );

    #[test]
    fn a_review_keeps_relabels_and_drops_traceably() -> Result<()> {
        let review = apply(&raws()?, &parse_decisions(DECISIONS)?)?;
        assert_eq!(review.rows.len(), 3);
        assert_eq!(review.rows[1]["act"], "assert");
        assert_eq!(review.rows[1]["source"], "p1:2");
        assert_eq!(review.rows[1]["review"], "no correction cue");
        assert_eq!(review.rows[0]["review"], Value::Null);
        assert_eq!(review.decisions["duplicate"], 1);
        assert_eq!(review.relabels["user_name/update -> user_name/assert"], 1);
        assert_eq!(review.by_label["user_name/assert"], 2);
        Ok(())
    }

    #[test]
    fn a_review_refuses_inconsistent_decisions() -> Result<()> {
        let raws = raws()?;
        let refuse = |tsv: &str| apply(&raws, &parse_decisions(tsv)?).map(|_| ());
        // A missed duplicate, a false duplicate, a wrong first row.
        assert!(refuse(&DECISIONS.replace(
            "p2:1\tduplicate\t\t\tduplicate of p1:1",
            "p2:1\tkeep\t\t\t-"
        ))
        .is_err());
        assert!(refuse(&DECISIONS.replace(
            "p2:2\tkeep\t\t\t-",
            "p2:2\tduplicate\t\t\tduplicate of p1:1"
        ))
        .is_err());
        assert!(refuse(&DECISIONS.replace("duplicate of p1:1", "duplicate of p1:2")).is_err());
        // Text that is not the raw row's, a slot in a query, a no-op relabel.
        assert!(refuse(&DECISIONS.replace("\tWhat's my name?", "\tWhat is my name?")).is_err());
        assert!(refuse(
            &DECISIONS.replace("relabel\tuser_name\tassert", "relabel\tuser_name\tquery")
        )
        .is_err());
        assert!(refuse(
            &DECISIONS.replace("relabel\tuser_name\tassert", "relabel\tuser_name\tupdate")
        )
        .is_err());
        // A missing row and an unknown decision.
        assert!(refuse(
            DECISIONS
                .lines()
                .take(4)
                .collect::<Vec<_>>()
                .join("\n")
                .as_str()
        )
        .is_err());
        assert!(refuse(&DECISIONS.replace("p1:3\tdrop", "p1:3\tskip")).is_err());
        assert!(parse_decisions("p1:1\tkeep\n").is_err());
        Ok(())
    }
}
