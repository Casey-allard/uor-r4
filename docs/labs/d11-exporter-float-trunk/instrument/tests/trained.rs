//! Opt-in host parity on a trained artifact (claude-128): the host twin equals
//! the Rust integer session at every position of a readout's DEV windows and
//! of its single tokens. A finite host check of the port before a pod run; the
//! GPU qualification is `d11-fbs-eval run ... --verify` on the pod.
//!
//! UOR_FBS_ARTIFACT=<artifact.uorlut> UOR_FBS_READOUT=<readout.json> \
//!   cargo test --target <host> --test trained -- --ignored --nocapture

use d11_fbs_evaluator::{first_difference, host_sequences, session_sequences};
use uor_r4_integer::stack::IntegerStackModel;

#[test]
#[ignore = "needs UOR_FBS_ARTIFACT and UOR_FBS_READOUT"]
fn host_twin_equals_the_session_on_a_trained_artifact() {
    let (Ok(artifact), Ok(readout)) = (
        std::env::var("UOR_FBS_ARTIFACT"),
        std::env::var("UOR_FBS_READOUT"),
    ) else {
        panic!("set UOR_FBS_ARTIFACT and UOR_FBS_READOUT");
    };
    let model =
        IntegerStackModel::parse(&std::fs::read(&artifact).expect("artifact")).expect("parse");
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&readout).expect("readout")).expect("json");
    let windows: Vec<Vec<u32>> = value["sequences"]
        .as_array()
        .expect("sequences")
        .iter()
        .map(|s| serde_json::from_value(s["tokens"].clone()).expect("tokens"))
        .collect();
    let vocab = model.shape().vocab as u32;
    let singles: Vec<Vec<u32>> = (0..vocab).map(|t| vec![t]).collect();
    for (name, sequences) in [("single tokens", singles), ("DEV windows", windows)] {
        let host = host_sequences(&model, &sequences).expect("host twin");
        let rust = session_sequences(&model, &sequences).expect("Rust session");
        eprintln!(
            "{name}: {} positions, first difference {:?}",
            host.sequences * host.len,
            first_difference(&host, &rust)
        );
        assert_eq!(host, rust, "{name}");
    }
}
