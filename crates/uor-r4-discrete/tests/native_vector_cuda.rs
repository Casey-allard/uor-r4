#![cfg(feature = "cuda")]

use uor_r4_discrete::native_vector::q31_hamilton_rotate;
use uor_r4_discrete::native_vector::Q31HamiltonModel;
use uor_r4_discrete::native_vector_device::NativeVectorEngine;

fn values(seed: u64, count: usize) -> Vec<[i64; 4]> {
    let mut out = vec![
        [i64::from(i32::MIN); 4],
        [1 << 31, -(1 << 31), 1, -1],
        [1 << 30, 1 << 29, -(1 << 28), 7],
        [0, 1 << 31, 0, -(1 << 31)],
    ];
    let mut seed = seed;
    out.extend((0..count.saturating_sub(out.len())).map(|_| {
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as i32
        };
        [
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
        ]
    }));
    out
}

#[test]
fn cuda_and_rust_return_identical_q31_transport_words() -> uor_r4_discrete::Result<()> {
    let engine = match NativeVectorEngine::new(0) {
        Ok(engine) => engine,
        Err(error) if std::env::var("UOR_REQUIRE_CUDA").as_deref() != Ok("1") => {
            println!("SKIPPED (no CUDA device): {error}");
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let valid = [[0, 1 << 31, 0, 0]];
    for invalid in [[[(1 << 31) + 1, 0, 0, 0]], [[-((1 << 31) + 1), 0, 0, 0]]] {
        assert!(engine.hamilton_rotate(&invalid, &valid).is_err());
        assert!(engine.hamilton_rotate(&valid, &invalid).is_err());
    }
    let gates = values(0x6a09_e667_f3bc_c909, 4096);
    let held = values(0xbb67_ae85_84ca_a73b, 4096);
    let cuda = engine.hamilton_rotate(&gates, &held)?;
    for i in 0..gates.len() {
        assert_eq!(
            cuda[i],
            q31_hamilton_rotate(gates[i], held[i])?,
            "native Q31 Hamilton parity differs at item {i}"
        );
    }
    assert_eq!(
        NativeVectorEngine::hamilton_rotate_reference(gates[0], held[0])?,
        cuda[0]
    );
    let unit = [[1 << 31, 0, 0, 0]];
    let half = [[1 << 30, 0, 0, 0]];
    let unit_cuda = engine.hamilton_rotate(&unit, &half)?;
    let planted_shift = ((u64::from(1_u32 << 30)) << 31) as i64 >> 30;
    assert_ne!(
        unit_cuda[0][0], planted_shift,
        "a planted Q30/Q31 shift change was not detected"
    );
    let one = 1 << 31;
    let axes = engine.hamilton_rotate(
        &[[0, one, 0, 0], [0, 0, one, 0]],
        &[[0, 0, one, 0], [0, one, 0, 0]],
    )?;
    assert_eq!(axes, [[0, 0, 0, one], [0, 0, 0, -one]]);
    Ok(())
}

#[test]
fn a_one_bit_scale_change_is_detected() {
    let gate = [1 << 31, 0, 0, 0];
    let value = [1 << 30, 0, 0, 0];
    let expected = q31_hamilton_rotate(gate, value).unwrap();
    let wrong_shift = ((value[0].unsigned_abs() << 31) as i64) >> 30;
    assert_ne!(expected[0], wrong_shift);
}

#[test]
fn cuda_training_steps_match_the_rust_oracle_and_lower_loss() -> uor_r4_discrete::Result<()> {
    let engine = match NativeVectorEngine::new(0) {
        Ok(engine) => engine,
        Err(error) if std::env::var("UOR_REQUIRE_CUDA").as_deref() != Ok("1") => {
            println!("SKIPPED (no CUDA device): {error}");
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let gates = values(0x510e_527f_ade6_82d1, 4096);
    let held = values(0x9b05_688c_2b3e_6c1f, 4096);
    let targets = values(0x1f83_d9ab_fb41_bd6b, 4096);
    let gpu = engine.hamilton_train_step(&gates, &held, &targets, 8)?;
    for index in 0..gates.len() {
        let cpu = NativeVectorEngine::hamilton_train_step_reference(
            gates[index],
            held[index],
            targets[index],
            8,
        )?;
        assert_eq!(gpu[index], cpu, "training step differs at item {index}");
    }

    let one = 1 << 31;
    let target = [1_518_500_249, 1_518_500_249, 0, 0];
    let mut gate = [one, 0, 0, 0];
    let initial =
        NativeVectorEngine::hamilton_train_step_reference(gate, [one, 0, 0, 0], target, 8)?
            .loss_before;
    for _ in 0..64 {
        let step = engine.hamilton_train_step(&[gate], &[[one, 0, 0, 0]], &[target], 8)?;
        gate = step[0].accepted_gate;
    }
    let final_step = engine.hamilton_train_step(&[gate], &[[one, 0, 0, 0]], &[target], 8)?;
    assert!(
        final_step[0].loss_before < initial,
        "initial {initial}, final {}",
        final_step[0].loss_before
    );
    Ok(())
}

#[test]
fn cuda_and_rust_shared_model_steps_keep_the_same_gate_and_epoch() -> uor_r4_discrete::Result<()> {
    let engine = match NativeVectorEngine::new(0) {
        Ok(engine) => engine,
        Err(error) if std::env::var("UOR_REQUIRE_CUDA").as_deref() != Ok("1") => {
            println!("SKIPPED (no CUDA device): {error}");
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let one = 1 << 31;
    let target = [1_518_500_249, 1_518_500_249, 0, 0];
    let values = vec![[one, 0, 0, 0]; 8];
    let targets = vec![target; 8];
    let mut cuda_model = Q31HamiltonModel::new([one, 0, 0, 0], 8)?;
    let mut expected = Q31HamiltonModel::new([one, 0, 0, 0], 8)?;
    let first_cuda = engine.train_model_step(&mut cuda_model, &values, &targets)?;
    let first_rust =
        NativeVectorEngine::train_model_step_reference(&mut expected, &values, &targets)?;
    assert_eq!(first_cuda, first_rust);
    let initial_loss = first_cuda.loss_before;
    for step in 1..64 {
        let cuda = engine.train_model_step(&mut cuda_model, &values, &targets)?;
        let rust =
            NativeVectorEngine::train_model_step_reference(&mut expected, &values, &targets)?;
        assert_eq!(cuda, rust, "shared model step differs after step {step}");
        assert_eq!(cuda_model.gate(), cuda.gate_after);
        assert_eq!(cuda_model.epoch(), step + 1);
    }
    let mut probe = cuda_model;
    let final_loss =
        NativeVectorEngine::train_model_step_reference(&mut probe, &values, &targets)?.loss_before;
    assert!(
        final_loss < initial_loss,
        "initial {initial_loss}, final {final_loss}"
    );
    assert_eq!(
        Q31HamiltonModel::from_bytes(cuda_model.to_bytes())?,
        cuda_model
    );
    Ok(())
}
