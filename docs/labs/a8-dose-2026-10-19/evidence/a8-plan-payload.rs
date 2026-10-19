#![forbid(unsafe_code)]
//! Typed plan metadata only: parse an AttentionPlan and print the crate's own
//! review_payload_sha256. Never opens plan FilePins, creates no CUDA context.
use uor_r4_discrete::controller::attention_learning::{review_payload_sha256, AttentionPlan};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: a8-plan-payload PLAN_JSON")?;
    let plan: AttentionPlan = serde_json::from_slice(&std::fs::read(path)?)?;
    println!("{}", serde_json::json!({"scope": plan.scope, "reviewed_plan_payload_sha256": review_payload_sha256(&plan)?, "model_operations": 0}));
    Ok(())
}
