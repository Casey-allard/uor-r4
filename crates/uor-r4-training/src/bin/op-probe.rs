//! Record what the OP MODEL actually emitted for a panel's question turns.
//!
//! The compiled action alone cannot show this. Policy `unless_query` returns the TABLE's
//! Unresolved when it discards a non-statement op, so a turn whose op read
//! `Op: query sculptor` and a turn whose op read nonsense are INDISTINGUISHABLE in the
//! final action. Both are `Unresolved { "the heads name no relation" }`.
//!
//! That distinction decides whether the `unless_query` precedence bug is worth fixing: if
//! the op model already names the right relation and the policy throws it away, the fix is
//! a precedence correction. If the op model names nothing (or names a closed label), the
//! precedence is not the binding constraint and the fix would only convert Unresolved into
//! wrong-relation queries.
//!
//! Usage: op-probe compiler=DIR trunk=DIR tokenizer=FILE panel=FILE [limit=N]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_training::relation_compiler::Trunk;
use uor_r4_training::relation_compiler::{
    derived_relation_id, relation_phrase, OpPolicy, OpProbe, SavedCompiler,
};

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kv = |k: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{k}=")).map(str::to_string))
    };
    let compiler_dir = PathBuf::from(kv("compiler").ok_or("need compiler=DIR")?);
    let trunk_dir = PathBuf::from(kv("trunk").ok_or("need trunk=DIR")?);
    let tokenizer_path = kv("tokenizer").ok_or("need tokenizer=FILE")?;
    let panel_path = kv("panel").ok_or("need panel=FILE")?;
    let limit: usize = kv("limit")
        .map(|v| v.parse().unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);

    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| format!("tokenizer: {e}"))?;
    let bytes =
        fs::read(compiler_dir.join("compiler.json")).map_err(|e| format!("compiler: {e}"))?;
    let compiler = SavedCompiler::load(
        bytes,
        Some(Trunk::load(&trunk_dir, &tokenizer_json, &Device::Cpu).map_err(|e| format!("{e}"))?),
    )
    .map_err(|e| format!("load: {e}"))?
    .with_op_policy(OpPolicy::UnlessQuery)
    .map_err(|e| format!("policy: {e}"))?;

    let panel: serde_json::Value =
        serde_json::from_slice(&fs::read(&panel_path).map_err(|e| format!("panel: {e}"))?)
            .map_err(|e| format!("panel json: {e}"))?;
    let rows = panel.as_array().ok_or("panel is not an array")?;

    // mode=phrase validates the DETERMINISTIC extractor against the panel's own relation
    // field: does the phrase taken from the turn's words name the row's relation, and do
    // the statement and the question derive the SAME address?
    if kv("mode").as_deref() == Some("phrase") {
        let (mut exact, mut same_addr, mut declined, mut wrong) = (0usize, 0usize, 0usize, 0usize);
        let mut examples: Vec<(String, String, String)> = Vec::new();
        for row in rows.iter() {
            let rel = row["relation"].as_str().unwrap_or_default();
            let Some(turns) = row["user_turns"].as_array() else {
                continue;
            };
            let Some(last) = turns.last().and_then(|t| t.as_str()) else {
                continue;
            };
            let q = relation_phrase(last);
            // the statement turn is the one that is not a question
            let stmt = turns
                .iter()
                .filter_map(|t| t.as_str())
                .find(|t| !t.trim_end().ends_with('?'))
                .and_then(relation_phrase);
            match &q {
                None => declined += 1,
                Some(p) => {
                    if *p == rel {
                        exact += 1;
                    } else {
                        wrong += 1;
                        if examples.len() < 5 {
                            examples.push((last.to_string(), p.clone(), rel.to_string()));
                        }
                    }
                }
            }
            if let (Some(a), Some(b)) = (&stmt, &q) {
                if derived_relation_id(a) == derived_relation_id(b) {
                    same_addr += 1;
                }
            }
        }
        println!("  phrase validation over {} rows", rows.len());
        println!("    question phrase == row relation : {exact}");
        println!("    question phrase != row relation : {wrong}");
        println!("    declined (no relation word)     : {declined}");
        println!("    statement and question SAME address : {same_addr}");
        if !examples.is_empty() {
            println!("\n    mismatches:");
            for (t, got, want) in &examples {
                println!("      {t:?} -> {got:?}  (row relation {want:?})");
            }
        }
        return Ok(());
    }

    // question form -> (count, op text -> count)
    let mut by_form: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut total = 0usize;
    for row in rows.iter().take(limit) {
        let Some(turns) = row["user_turns"].as_array() else {
            continue;
        };
        let Some(last) = turns.last().and_then(|t| t.as_str()) else {
            continue;
        };
        let form = if last.ends_with("'s name?") {
            "What is <REL>'s name?"
        } else if last == "What is it?" {
            "What is it?"
        } else if last.starts_with("What is the ") {
            "What is the <REL>?"
        } else {
            "other"
        };
        let op = match compiler.op_probe(last) {
            Ok(OpProbe::Text(t)) => t.trim().to_string(),
            Ok(OpProbe::Refused(r)) => format!("<refused: {r}>"),
            Err(e) => format!("<error: {e}>"),
        };
        *by_form
            .entry(form.to_string())
            .or_default()
            .entry(op)
            .or_insert(0) += 1;
        total += 1;
    }

    println!("  probed {total} question turns\n");
    for (form, ops) in &by_form {
        let n: usize = ops.values().sum();
        println!("  === {form}  ({n} turns) ===");
        let mut sorted: Vec<(&String, &usize)> = ops.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (op, c) in sorted.iter().take(8) {
            println!("    {c:>4}  {op:?}");
        }
        if sorted.len() > 8 {
            println!("    ... {} more distinct ops", sorted.len() - 8);
        }
        println!();
    }
    Ok(())
}
