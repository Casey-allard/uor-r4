// Finite check using the actual parent and a loaded D35 carrier with no tokenizer bound.
use std::{env, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{PairedRoleDecision, RequestedReplayArm},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [parent, carrier] = args.as_slice() else {
        return Err("expected parent and D35 carrier".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(carrier))?;
    assert!(model.pair_rank_bindings().is_some());
    let roles = PairedRoleDecision {
        requested: None,
        excluded: None,
        requested_score: 0,
        excluded_score: 0,
        requested_runner_up: 0,
        excluded_runner_up: 0,
    };
    for result in [
        model.paired_copy_decision(&[0, 1], true).map(|_| ()),
        model.role_choice(&[0, 1], true).map(|_| ()),
        model
            .requested_role_replay(&[0, 1], RequestedReplayArm::FullRepeated)
            .map(|_| ()),
        model
            .paired_copy_selected_witness(&[0, 1], roles, &[])
            .map(|_| ()),
    ] {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("tokenizer-bound full-detached"));
    }
    assert!(model.role_choice(&[0, 1], false)?.unwrap().source.is_none());
    let old = IntegerDiscreteModel::load(Path::new(parent))?;
    assert!(old.pair_rank_bindings().is_none());
    assert!(old.role_choice(&[0, 1], false)?.unwrap().source.is_none());
    Ok(())
}
