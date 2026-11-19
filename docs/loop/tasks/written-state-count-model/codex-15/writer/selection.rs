use super::{hash, load, save, Error, Prompt};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use uor_r4_integer::{discrete_d11::IntegerDiscreteModel, paired_role::RoleSpan};

#[derive(Deserialize)]
struct Bounds {
    start: usize,
    end: usize,
}
#[derive(Deserialize)]
struct Grade {
    stratum: String,
    prefix_sha256: String,
    requested: Option<[usize; 2]>,
    requested_name: Option<String>,
    question_mentions: Vec<(String, [usize; 2])>,
    facts: Vec<Fact>,
}
#[derive(Deserialize)]
struct Fact {
    name: String,
}

pub fn prefix_hash(ids: &[u32]) -> String {
    let mut h = Sha256::new();
    for id in ids {
        h.update(id.to_le_bytes());
    }
    format!("{:x}", h.finalize())
}

pub fn select(args: &[String]) -> Result<(), Error> {
    let [artifact, blind, arm, bounds, grades, out] = args else {
        return Err("select ARTIFACT BLIND public|bounded|lexical|no-read|oracle|wrong BOUNDS|- GRADES|- OUT".into());
    };
    let oracle = matches!(arm.as_str(), "oracle" | "wrong");
    let bounded = matches!(arm.as_str(), "bounded" | "lexical" | "oracle" | "wrong");
    if (!bounded && !matches!(arm.as_str(), "public" | "no-read"))
        || bounded != (bounds != "-")
        || oracle != (grades != "-")
    {
        return Err("invalid intervention or custody inputs".into());
    }
    let prompts: Vec<Prompt> = serde_json::from_value(load(Path::new(blind))?)?;
    let intervals: Vec<Bounds> = if bounded {
        serde_json::from_value(load(Path::new(bounds))?)?
    } else {
        Vec::new()
    };
    let labels: Vec<Grade> = if oracle {
        serde_json::from_value(load(Path::new(grades))?)?
    } else {
        Vec::new()
    };
    if (bounded && intervals.len() != prompts.len()) || (oracle && labels.len() != prompts.len()) {
        return Err("row count mismatch".into());
    }
    let model = IntegerDiscreteModel::load(Path::new(artifact))?;
    let mut rows = Vec::new();
    for (i, prompt) in prompts.iter().enumerate() {
        let ids = &prompt.prefix;
        let digest = prefix_hash(ids);
        let region = if bounded {
            intervals[i].start..intervals[i].end
        } else {
            0..ids.len()
        };
        if region.is_empty() || region.end > ids.len() {
            return Err("invalid bounds".into());
        }
        if oracle && labels[i].prefix_sha256 != digest {
            return Err("unbound oracle label".into());
        }
        if oracle && labels[i].stratum != "supported" {
            continue;
        }
        let mut roles = model.paired_roles_diagnostic(ids, region.clone(), arm == "lexical")?;
        let original = roles.words();
        let mut endpoint = None;
        if oracle {
            let g = &labels[i];
            let mut target = g.requested.ok_or("missing supported role")?;
            if arm == "wrong" {
                let rivals: Vec<_> = g
                    .question_mentions
                    .iter()
                    .filter(|(n, _)| {
                        Some(n) != g.requested_name.as_ref()
                            && g.facts.iter().take(2).any(|fact| fact.name == *n)
                    })
                    .collect();
                if rivals.len() != 1 {
                    return Err(
                        format!("row {i} does not have one alternative factual endpoint").into(),
                    );
                }
                target = rivals[0].1;
            }
            endpoint = Some(RoleSpan {
                start: target[0],
                end: target[1],
            });
        }
        let (paired, assertions) = if arm == "no-read" {
            (
                model
                    .paired_copy_decision(ids, false)?
                    .ok_or("no read policy")?,
                Vec::new(),
            )
        } else {
            model.paired_copy_witness(ids, roles, endpoint)?
        };
        if arm == "public"
            && serde_json::to_value(&paired)?
                != serde_json::to_value(
                    model
                        .paired_copy_decision(ids, true)?
                        .ok_or("no public policy")?,
                )?
        {
            return Err("full-prefix diagnostic diverged from normal dispatch".into());
        }
        if oracle {
            roles.requested = endpoint;
            if paired.role_words != Some(roles.words()) {
                return Err("oracle changed other role words".into());
            }
        }
        let candidates: Value = if arm == "no-read" {
            Value::Null
        } else if model.ordered_role_bank().is_some() {
            serde_json::to_value(model.ordered_role_trace(ids, region.clone())?)?
        } else {
            serde_json::to_value(model.frozen_role_trace(ids, region.clone(), arm == "lexical")?)?
        };
        rows.push(json!({"index":i,"prefix_sha256":digest,"region":[region.start,region.end],"original_role_words":original,"paired":paired,"assertions":assertions,"candidates":candidates}));
    }
    save(
        Path::new(out),
        &json!({"schema":"uor-r4.ordered-role.selections/1","artifact":hash(Path::new(artifact))?,"blind":hash(Path::new(blind))?,"bounds":if bounded {Some(hash(Path::new(bounds))?)}else{None},"arm":arm,"rows":rows}),
    )
}
