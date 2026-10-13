//! Finite, exact CUDA construction controls. No optimizer application or corpus run.
use crate::{
    device::{Device, Snapshot},
    Error, Identity, Parameter, Result, Spec,
};
use std::path::Path;
fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}
// Refusal controls must observe their declared model/domain error. A driver
// fault is not evidence that malformed input was correctly refused.
fn expect_invalid<T>(result: Result<T>, expected: &str, label: &str) -> Result<()> {
    match result {
        Err(Error::Invalid(actual)) if actual == expected => Ok(()),
        Err(error @ Error::Cuda(_)) => Err(error),
        Err(error) => Err(Error::Invalid(format!(
            "{label}: expected Invalid({expected:?}), observed {error}"
        ))),
        Ok(_) => Err(Error::Invalid(format!(
            "{label}: expected Invalid({expected:?}), operation succeeded"
        ))),
    }
}
fn identity(token: u32, occurrence: u64) -> Identity {
    Identity {
        token,
        occurrence,
        record: 17,
        version: 9,
    }
}
fn fixture(vocabulary: u32, context: u32, kept: u32) -> Result<Snapshot> {
    let device = Device::new(
        Spec {
            vocabulary,
            blocks: 2,
            context,
            kept_cells: kept,
        },
        7,
        0,
    )?;
    let mut snapshot = device.snapshot()?;
    // Literal independent E8 witnesses. These are fixture bytes, not a CPU model.
    let a = [2, 2, 0, 0, 0, 0, 0, 0];
    let negative = [-2, -2, 0, 0, 0, 0, 0, 0];
    let b = [0, 0, 2, 0, 2, 0, 0, 0];
    for token in 0..vocabulary as usize {
        for block in 0..2 {
            snapshot.table[(token * 2 + block) * 8..(token * 2 + block + 1) * 8].copy_from_slice(
                if token == 1 {
                    &negative
                } else if token == 2 {
                    &b
                } else {
                    &a
                },
            );
        }
    }
    Ok(snapshot)
}
fn exact_writer() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let carried = device.carry(&[identity(0, 0), identity(1, 1)])?;
    let result = device.forward(&carried, &[0, 0])?;
    require(
        result.predictions == [0, 1],
        "literal two-token ranking differs",
    )?;
    require(
        result.losses == [0, 640]
            && result.token_rivals == [1, 1]
            && result.address_rivals == [1, 1]
            && result.admitted_cells[..2] == [0, 17]
            && result.admitted_cells[16..18] == [17, 0],
        "exact token+address margin numerators differ",
    )?;
    require(
        result.vectors()? == carried.values()?,
        "one-leaf transport differs from identity",
    )
}
fn proposals() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let input = device.carry(&[identity(1, 0)])?;
    let before = device.snapshot()?;
    let proposal = device
        .propose(&input, &[0], Parameter::VectorBlock(0))?
        .observation()?;
    require(
        proposal.candidate == 7
            && proposal.before == 640
            && proposal.after == 544
            && proposal.epoch == 0,
        "nonvacuous exact geometric codebook proposal differs",
    )?;
    let repeated = device
        .propose(&input, &[0], Parameter::VectorBlock(0))?
        .observation()?;
    require(
        repeated == proposal,
        "identical proposals changed the exact cost or deterministic winner",
    )?;
    let noop = device
        .propose(&input, &[0], Parameter::Route(0))?
        .observation()?;
    require(
        noop.candidate == 0 && noop.before == 640 && noop.after == 640,
        "identity must win a complete no-choice route tie",
    )?;
    let after = device.snapshot()?;
    require(
        before.table == after.table && before.routes == after.routes && before.epoch == after.epoch,
        "virtual proposal changed learned state",
    )
}
fn context_transport() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    // The last token's B_12*a query chooses the middle FBS child. Its carrier
    // is -a, and actual BFS transport must produce -B_12*a, not the query.
    let input = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    let result = device.forward(&input, &[0])?;
    let expected = [0, 0, -2, 0, -2, 0, 0, 0, 0, 0, -2, 0, -2, 0, 0, 0];
    require(
        result.vectors()? == expected,
        "actual FBS earlier-child transport differs",
    )?;
    let second = device.carry(&[identity(0, 0), identity(0, 1), identity(2, 2)])?;
    let changed = device.forward(&second, &[0])?;
    let expected_changed = [0, 0, 2, 0, 2, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0];
    require(
        changed.vectors()? == expected_changed,
        "selected earlier occurrence does not reach writer",
    )
}
fn isolated_windows() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    let input = device.carry(&[
        identity(0, 0),
        identity(1, 1),
        identity(2, 2),
        identity(1, 3),
        identity(0, 4),
        identity(2, 5),
    ])?;
    let first = device.forward(&input, &[0, 0])?;
    let changed = device.carry(&[
        identity(0, 0),
        identity(1, 1),
        identity(2, 2),
        identity(2, 3),
        identity(2, 4),
        identity(0, 5),
    ])?;
    let second = device.forward(&changed, &[0, 1])?;
    require(
        first.vectors()?[..16] == second.vectors()?[..16]
            && first.predictions[0] == second.predictions[0]
            && first.losses[0] == second.losses[0],
        "a different window/future row changed earlier output",
    )
}
fn collision_and_omitted_target() -> Result<()> {
    let device = Device::from_snapshot(fixture(257, 1, 1)?, 0)?;
    let ids = [identity(0, 5), identity(256, 8)];
    let input = device.carry(&ids)?;
    require(
        input.identities() == ids,
        "colliding carrier identities were conflated",
    )?;
    let result = device.forward(&input, &[0, 256])?;
    require(
        result.predictions == [0, 0]
            && result.losses == [128, 128]
            && result.token_rivals == [254, 254]
            && result.address_rivals == [0, 0],
        "collision rank tie or legal absent-address rival differs",
    )?;
    let omitted = device.forward(&input, &[1, 1])?;
    require(
        omitted.predictions == [0, 0]
            && omitted.losses == [640, 640]
            && omitted.token_rivals == [255, 255]
            && omitted.address_rivals == [1, 1],
        "omitted target changed admission or escaped objective",
    )
}
fn nonvacuity() -> Result<()> {
    let device = Device::from_snapshot(fixture(2, 1, 1)?, 0)?;
    let input = device.carry(&[identity(0, 0)])?;
    let output = device.forward(&input, &[0])?;
    require(
        output.predictions == [0]
            && output.losses == [0]
            && output.token_rivals == [0]
            && output.address_rivals == [0],
        "absent comparisons must be explicit, not labelled supervised perfection",
    )?;
    expect_invalid(
        device.propose(&input, &[0], Parameter::VectorBlock(0)),
        "discrete CUDA counterfactual refused with status 8",
        "invalid baseline proposal",
    )
}
fn artifact_and_identity() -> Result<()> {
    let original = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let input = original.carry(&[identity(1, 0)])?;
    let bytes =
        serde_json::to_vec(&original.snapshot()?).map_err(|e| Error::Invalid(e.to_string()))?;
    let restored = Device::from_snapshot(
        serde_json::from_slice(&bytes).map_err(|e| Error::Invalid(e.to_string()))?,
        0,
    )?;
    expect_invalid(
        restored.forward(&input, &[0]),
        "carried batch belongs to another model/source/geometry",
        "foreign live carrier",
    )?;
    let recarried = restored.carry(input.identities())?;
    let a = original.forward(&input, &[0])?;
    let b = restored.forward(&recarried, &[0])?;
    require(
        a.predictions == b.predictions && a.losses == b.losses && a.vectors()? == b.vectors()?,
        "artifact round-trip changed exact output",
    )?;
    let mut invalid = original.snapshot()?;
    invalid.table[0] = 3;
    expect_invalid(
        Device::from_snapshot(invalid, 0),
        "discrete CUDA snapshot validation refused with status 6",
        "non-geometric codebook",
    )?;
    let mut wrong = original.snapshot()?;
    wrong.schema = "old-float-writer".into();
    expect_invalid(
        Device::from_snapshot(wrong, 0),
        "snapshot schema/geometry/AOT kernel/compiler source binding differs",
        "historical writer artifact",
    )
}
fn inventory_bounds() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    let partial = device.carry(&[identity(0, 0)])?;
    expect_invalid(
        device.forward(&partial, &[0]),
        "forward/proposal requires 1..64 admitted nonempty row plans",
        "incomplete context",
    )?;
    expect_invalid(
        device.carry(&[identity(3, 0)]),
        "carry requires nonempty valid token identities",
        "out-of-range token",
    )?;
    let full = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    expect_invalid(
        device.propose(&full, &[0], Parameter::Route(u32::MAX)),
        "proposal parameter index exceeds its declared inventory",
        "out-of-range proposal parameter",
    )?;
    match device.propose(&full, &[0], Parameter::Gate(0)) {
        Err(Error::UnimplementedParameter { kind: 3, index: 0 }) => {}
        Err(error) => return Err(error),
        Ok(_) => {
            return Err(Error::Invalid(
                "reserved gate proposal was activated".into(),
            ))
        }
    }
    match crate::training_admission() {
        Err(Error::TrainingHeld) => Ok(()),
        Err(error) => Err(error),
        Ok(()) => Err(Error::Invalid("training hold released".into())),
    }
}

fn variable_length_prefixes() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 3)?, 0)?;
    let windows = vec![
        vec![identity(2, 0), identity(1, 1)],
        vec![identity(0, 2), identity(1, 3), identity(2, 4)],
    ];
    let input = device.carry_windows(&windows)?;
    require(
        input.valid_lengths() == [2, 3] && input.identities().len() == 5,
        "window lengths or exact logical occurrences changed",
    )?;
    let output = device.forward(&input, &[0, 0])?;
    // Short prefix [b,-a]: frontier [a,-a], query -a selects the second
    // leaf, whose actual a -> -a action carries -a -> +a. Padding b must
    // not become a third leaf/query. Full [a,-a,b] gives -b as above.
    let expected = [
        2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -2, 0, -2, 0, 0, 0, 0, 0, -2, 0, -2,
        0, 0, 0,
    ];
    require(
        output.vectors()? == expected,
        "padding changed the last valid query or became a binder leaf",
    )?;
    let alone = device.carry_windows(&[windows[0].clone()])?;
    require(
        device.forward(&alone, &[0])?.vectors()? == expected[..16],
        "a short prefix depends on another row's plan",
    )?;
    let permuted = device.carry_windows(&[windows[1].clone(), windows[0].clone()])?;
    let reordered = device.forward(&permuted, &[0, 0])?;
    let original_vectors = output.vectors()?;
    let reordered_vectors = reordered.vectors()?;
    require(
        reordered.predictions == [output.predictions[1], output.predictions[0]]
            && reordered.losses == [output.losses[1], output.losses[0]]
            && reordered.token_rivals == [output.token_rivals[1], output.token_rivals[0]]
            && reordered.address_rivals == [output.address_rivals[1], output.address_rivals[0]]
            && reordered_vectors[..16] == original_vectors[16..]
            && reordered_vectors[16..] == original_vectors[..16]
            && reordered.admitted_cells[..16] == output.admitted_cells[16..]
            && reordered.admitted_cells[16..] == output.admitted_cells[..16],
        "row permutation changed a prefix's exact writer/transport result",
    )?;
    let changed = device.carry_windows(&[
        windows[0].clone(),
        vec![identity(0, 2), identity(1, 3), identity(1, 4)],
    ])?;
    let trailing = device.forward(&changed, &[0, 0])?;
    let trailing_vectors = trailing.vectors()?;
    require(
        trailing_vectors[..16] == original_vectors[..16]
            && trailing.predictions[0] == output.predictions[0]
            && trailing.losses[0] == output.losses[0]
            && trailing.admitted_cells[..16] == output.admitted_cells[..16],
        "another row's trailing token changed the short prefix",
    )?;
    require(
        trailing_vectors[16..] != original_vectors[16..],
        "trailing-token perturbation failed to exercise a different other-row output",
    )
}

fn target_independent_admission() -> Result<()> {
    let device = Device::from_snapshot(fixture(3, 3, 2)?, 0)?;
    let input = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    let first = device.forward(&input, &[0])?;
    let second = device.forward(&input, &[1])?;
    require(
        first.vectors()? == second.vectors()?
            && first.predictions == second.predictions
            && first.admitted_cells == second.admitted_cells,
        "target participated in forward geometry/admission/prediction",
    )
}

fn token_relabelling_preserves_geometry() -> Result<()> {
    let original = Device::from_snapshot(fixture(2, 1, 2)?, 0)?;
    let mut relabelled = fixture(2, 1, 2)?;
    // Fixed artifact-label permutation only: no host score or transport.
    for coordinate in 0..16 {
        relabelled.table.swap(coordinate, 16 + coordinate);
    }
    let relabelled = Device::from_snapshot(relabelled, 0)?;
    let input = original.carry(&[identity(1, 0)])?;
    let changed = relabelled.carry(&[identity(0, 0)])?;
    let first = original.forward(&input, &[0])?;
    let second = relabelled.forward(&changed, &[1])?;
    require(
        first.predictions == [1]
            && second.predictions == [0]
            && first.losses == [640]
            && second.losses == [640]
            && first.admitted_cells == second.admitted_cells
            && first.token_rivals == second.token_rivals
            && first.address_rivals == second.address_rivals
            && first.vectors()? == second.vectors()?,
        "relabelled token IDs changed geometric routing or objective",
    )
}
pub const NAMES: &[&str] = &[
    "exact_writer",
    "virtual_proposals",
    "context_transport",
    "isolated_windows",
    "collision_and_omitted_target",
    "nonvacuity",
    "artifact_and_identity",
    "inventory_bounds",
    "variable_length_prefixes",
    "target_independent_admission",
    "token_relabelling_preserves_geometry",
];
/// Independent assertion findings are collected. Any CUDA fault hard-stops;
/// remaining controls are recorded NOT_RUN. Exclusive reporting precedes CUDA.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn() -> Result<()>; 11] = [
        exact_writer,
        proposals,
        context_transport,
        isolated_windows,
        collision_and_omitted_target,
        nonvacuity,
        artifact_and_identity,
        inventory_bounds,
        variable_length_prefixes,
        target_independent_admission,
        token_relabelling_preserves_geometry,
    ];
    let mut records = Vec::new();
    let mut failed = false;
    let mut passed = 0usize;
    let mut attempted = 0usize;
    let mut hard_stop = None;
    for (index, (name, test)) in NAMES.iter().zip(controls).enumerate() {
        let outcome = test();
        attempted += 1;
        let record = match outcome {
            Ok(()) => {
                passed += 1;
                serde_json::json!({"index":index+1,"name":name,"status":"PASS"})
            }
            Err(error @ Error::Cuda(_)) => {
                failed = true;
                let record = serde_json::json!({"index":index+1,"name":name,"status":"HARD_STOP","error":error.to_string()});
                hard_stop = Some(error);
                record
            }
            Err(e) => {
                failed = true;
                serde_json::json!({"index":index+1,"name":name,"status":"FAIL","error":e.to_string()})
            }
        };
        println!("{record}");
        records.push(record);
        if hard_stop.is_some() {
            break;
        }
    }
    for (index, name) in NAMES.iter().enumerate().skip(attempted) {
        let record = serde_json::json!({"index":index+1,"name":name,"status":"NOT_RUN","reason":"preceding CUDA fault; model/device safety not established"});
        println!("{record}");
        records.push(record);
    }
    let payload = serde_json::json!({"schema":crate::SCHEMA,"status":if hard_stop.is_some(){"ABORTED"}else if failed{"FAILED"}else{"PASS"},"registered":NAMES.len(),"attempted":attempted,"passed":passed,"failed":attempted-passed,"not_run":NAMES.len()-attempted,"controls":records,"training":"HELD","applied_updates":0});
    std::fs::write(
        report.join("controls.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    if let Some(error) = hard_stop {
        return Err(error);
    }
    require(!failed, "one or more finite construction controls failed")
}
