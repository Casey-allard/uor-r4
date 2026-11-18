//! A grade-free CE rollout. The selector handoff is content-addressed and this
//! instrument reuses the #4093 writer, including every per-step differential.
use super::{rollout, Bare, BarePrompt, HISTORIC_CUBIN};
use crate::copy_rollout::{contents, parse, replay, Arm};
use crate::role_swap::RELEASED;
use m1_d11_artifact_scorer_instrument::{artifact::ArtifactTables, device};
use serde::Deserialize;
use serde_json::{json, Value};
use uor_r4_discrete::corpus::sha256;

const MANIFEST: &str =
    include_str!("../../../../loop/tasks/written-state-count-model/codex-8/writer/INPUTS.json");
const EOS_CUBIN: &str = "8a73fffb5edfe3f5b36b891b37a96ae5683aacde49d08ae69caab92a7cab9b85";

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Identity {
    model: String,
    bare: String,
    picks: String,
}

#[derive(Deserialize)]
struct Handoff {
    arm: String,
    schema: String,
    artifact_sha256: String,
    bare_sha256: String,
    rows: Vec<Selection>,
}
#[derive(Deserialize)]
struct Selection {
    prefix_sha256: String,
    continuation: uor_r4_integer::continuation_selector::Continuation,
}

pub fn run(model_path: &str, bare_path: &str, picks_path: &str) -> Result<Value, String> {
    let model = contents(model_path)?;
    let bare = contents(bare_path)?;
    let picks = contents(picks_path)?;
    let identity = Identity {
        model: sha256(&model),
        bare: sha256(&bare),
        picks: sha256(&picks),
    };
    if !RELEASED.contains(&identity.model.as_str())
        || !parse::<Vec<Identity>>(MANIFEST.as_bytes())?.contains(&identity)
        || device::cubin_sha256() != HISTORIC_CUBIN
        || sha256(device::COMPLETION_CUBIN) != EOS_CUBIN
    {
        return Err("unsealed selector handoff, model or CE cubin".into());
    }
    let inputs: Vec<Bare> = parse(&bare)?;
    let picks: Handoff = parse(&picks)?;
    if picks.schema != "uor-r4.entity-pair-chain/1"
        || !matches!(picks.arm.as_str(), "baseline" | "repair" | "oracle")
        || picks.artifact_sha256 != identity.model
        || picks.bare_sha256 != identity.bare
        || inputs.is_empty()
        || inputs.len() != picks.rows.len()
    {
        return Err("inconsistent blind CE writer handoff".into());
    }
    let (tables, model) = ArtifactTables::from_bytes(&model)?;
    let gpu = device::DeviceScorer::new(0, &tables)?;
    let mut rows = Vec::with_capacity(inputs.len());
    let mut transitions = 0;
    for (items, decisions) in inputs.chunks(32).zip(picks.rows.chunks(32)) {
        let mut prompts = Vec::with_capacity(items.len());
        for (item, decision) in items.iter().zip(decisions) {
            let ids = &item.prefix;
            let bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
            let source = decision.continuation.source;
            if decision.prefix_sha256 != sha256(&bytes)
                || source.is_some_and(|pos| pos as usize >= ids.len())
            {
                return Err("different bare prefix or source beyond prompt".into());
            }
            prompts.push(BarePrompt {
                hash: &decision.prefix_sha256,
                prefix: ids,
            });
        }
        let mut runs: Vec<_> = prompts
            .iter()
            .zip(decisions)
            .map(|(p, d)| rollout(p, Arm::ContinuationEos, d.continuation.source))
            .collect();
        transitions += replay(&mut runs, &model, &gpu)?;
        for (r, d) in runs.into_iter().zip(decisions) {
            rows.push(json!({
                "prefix_sha256": d.prefix_sha256,
                "source": r.source,
                "generated": &r.prefix[r.case.prefix.len()..],
                "first_free": r.first_free,
                "steps": r.steps
            }));
        }
    }
    Ok(
        json!({"schema":"uor-r4.entity-writer/1", "model":identity.model,
        "bare":identity.bare, "arm":picks.arm, "picks":identity.picks, "writer_cubin":HISTORIC_CUBIN,
        "completion_cubin":EOS_CUBIN, "transitions":transitions, "rows":rows}),
    )
}
