#![cfg(feature = "cuda")]

use uor_r4_discrete::native_vector::q31_hamilton_rotate;
use uor_r4_discrete::native_vector::Q31HamiltonModel;
use uor_r4_discrete::native_vector::{Q31FirstPickRead, Q31ReadOutput, Q31ReadTrainStep};
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

const ONE: i64 = 1 << 31;
const IDENTITY: [i64; 4] = [ONE, 0, 0, 0];

fn engine_or_skip() -> uor_r4_discrete::Result<Option<NativeVectorEngine>> {
    match NativeVectorEngine::new(0) {
        Ok(engine) => Ok(Some(engine)),
        Err(error) if std::env::var("UOR_REQUIRE_CUDA").as_deref() != Ok("1") => {
            println!("SKIPPED (no CUDA device): {error}");
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn admitted(&mut self) -> i64 {
        (self.next() % ((1 << 32) + 1)) as i64 - ONE
    }

    fn small(&mut self) -> i64 {
        (self.next() % (1 << 21)) as i64 - (1 << 20)
    }
}

fn read_parity(
    engine: &NativeVectorEngine,
    read: &Q31FirstPickRead,
    stream: &[i64],
    ids: &[u32],
    table: &[i64],
    batch: usize,
    time: usize,
) -> uor_r4_discrete::Result<Q31ReadOutput> {
    let cuda = engine.first_pick_read(read, stream, ids, table, batch, time)?;
    let oracle =
        NativeVectorEngine::first_pick_read_reference(read, stream, ids, table, batch, time)?;
    assert_eq!(cuda.picks, oracle.picks, "first-pick read picks differ");
    for (index, (gpu, cpu)) in cuda.out.iter().zip(&oracle.out).enumerate() {
        assert_eq!(gpu, cpu, "first-pick read out differs at word {index}");
    }
    assert_eq!(cuda, oracle);
    Ok(cuda)
}

#[test]
fn cuda_first_pick_read_matches_the_oracle_on_a_seeded_fixture() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    let (width, batch, time, vocab) = (256, 4, 64, 50);
    let mut rng = XorShift(0x3c6e_f372_fe94_f82b);
    let gates = |rng: &mut XorShift| -> Vec<[i64; 4]> {
        (0..width / 4)
            .map(|_| {
                [
                    rng.admitted(),
                    rng.admitted(),
                    rng.admitted(),
                    rng.admitted(),
                ]
            })
            .collect()
    };
    let query_gates = gates(&mut rng);
    let key_gates = gates(&mut rng);
    let weights: Vec<i64> = (0..width).map(|_| rng.admitted()).collect();
    let table: Vec<i64> = (0..vocab * width).map(|_| rng.admitted()).collect();
    let stream: Vec<i64> = (0..batch * time * width).map(|_| rng.admitted()).collect();
    let ids: Vec<u32> = (0..batch * time)
        .map(|_| (rng.next() % vocab as u64) as u32)
        .collect();
    for gain_shift in [0, 4] {
        let read = Q31FirstPickRead::new(
            query_gates.clone(),
            key_gates.clone(),
            weights.clone(),
            gain_shift,
        )?;
        let output = read_parity(&engine, &read, &stream, &ids, &table, batch, time)?;
        assert_eq!(output.picks.len(), batch * time);
        assert!((0..batch).all(|b| output.picks[b * time] == 0));
        assert!(output.out.iter().any(|&word| word == ONE || word == -ONE));
    }
    Ok(())
}

#[test]
fn cuda_first_pick_read_matches_ties_saturation_and_refusals() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    let (width, batch, time) = (16, 2, 12);
    let read = Q31FirstPickRead::new(
        vec![IDENTITY; width / 4],
        vec![IDENTITY; width / 4],
        vec![ONE; width],
        0,
    )?;
    // Three row patterns repeat, so later rows tie exactly between earlier
    // copies; the pattern words sit near +-2^31, so a full table row saturates.
    let patterns = [
        vec![ONE - 5; width],
        vec![-ONE + 5; width],
        (0..width as i64).map(|lane| lane * 1000 - 7000).collect(),
    ];
    let stream: Vec<i64> = (0..batch * time)
        .flat_map(|row| patterns[row % 3].clone())
        .collect();
    let table = [vec![ONE; width], vec![-ONE; width], vec![17; width]].concat();
    let ids: Vec<u32> = (0..batch * time).map(|row| (row % 3) as u32).collect();
    let output = read_parity(&engine, &read, &stream, &ids, &table, batch, time)?;
    // Row 9 matches rows 0, 3 and 6 exactly; the tie goes to row 6.
    assert_eq!(output.picks[9], 7);
    assert_eq!(output.picks[time + 9], 7);
    assert!(output.out.iter().any(|&word| word == ONE));
    assert!(output.out.iter().any(|&word| word == -ONE));

    let bad_ids: Vec<u32> = ids.iter().map(|&id| id + 1).collect();
    assert!(read.apply(&stream, &bad_ids, &table, batch, time).is_err());
    assert!(engine
        .first_pick_read(&read, &stream, &bad_ids, &table, batch, time)
        .is_err());
    Ok(())
}

#[test]
fn cuda_first_pick_read_tracks_a_one_lsb_key_gate_change() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    let (width, batch, time, vocab) = (16, 2, 8, 5);
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    let weights: Vec<i64> = (0..width).map(|_| rng.admitted()).collect();
    let table: Vec<i64> = (0..vocab * width).map(|_| rng.small()).collect();
    let ids: Vec<u32> = (0..batch * time)
        .map(|_| (rng.next() % vocab as u64) as u32)
        .collect();
    let mut stream: Vec<i64> = (0..batch * time * width).map(|_| rng.small()).collect();
    // Rows 2 and 4 sit 1000 above and below row 6's lane 0 and tie exactly.
    let near = 3i64 << 29;
    stream[6 * width] = near;
    let query_row = stream[6 * width..7 * width].to_vec();
    for (row, lane0) in [(2, near + 1000), (4, near - 1000)] {
        stream[row * width..(row + 1) * width].copy_from_slice(&query_row);
        stream[row * width] = lane0;
    }
    let base_read = Q31FirstPickRead::new(
        vec![IDENTITY; width / 4],
        vec![IDENTITY; width / 4],
        weights.clone(),
        0,
    )?;
    let mut key_gates = vec![IDENTITY; width / 4];
    key_gates[0][0] = ONE - 1;
    let nudged_read = Q31FirstPickRead::new(vec![IDENTITY; width / 4], key_gates, weights, 0)?;
    let base = read_parity(&engine, &base_read, &stream, &ids, &table, batch, time)?;
    let nudged = read_parity(&engine, &nudged_read, &stream, &ids, &table, batch, time)?;
    assert_eq!(base.picks[6], 5, "the tie goes to row 4");
    assert_eq!(nudged.picks[6], 3, "row 2 is now strictly closer");
    assert_ne!(nudged, base);
    Ok(())
}

// ---- claude-392: the read's gate update, device against host ----

/// The served residual's scale at the read (claude-390): about 2^18.
const S: i64 = 1 << 18;

fn identity_read(width: usize) -> Q31FirstPickRead {
    Q31FirstPickRead::new(
        vec![IDENTITY; width / 4],
        vec![IDENTITY; width / 4],
        vec![0; width],
        4,
    )
    .unwrap()
}

fn row(lane0: i64, lane1: i64) -> Vec<i64> {
    let mut row = vec![0; 16];
    row[0] = lane0;
    row[1] = lane1;
    row
}

/// claude-391's fixture: row 2's nearest earlier key is row 0's; its target is row 1.
fn fixture(scale: i64) -> Vec<i64> {
    [row(scale, 0), row(0, scale), row(3 * scale / 4, scale / 2)].concat()
}

/// One step on the device and through the host rule; every field must agree.
#[allow(clippy::too_many_arguments)]
fn train_parity(
    engine: &NativeVectorEngine,
    read: &Q31FirstPickRead,
    stream: &[i64],
    ids: &[u32],
    table: &[i64],
    batch: usize,
    time: usize,
    targets: &[u32],
    margin: u64,
    shift: u32,
) -> uor_r4_discrete::Result<Q31ReadTrainStep> {
    let cuda = engine.read_train_pick_step(
        read, stream, ids, table, batch, time, targets, margin, shift,
    )?;
    let host = NativeVectorEngine::read_train_pick_step_reference(
        read, stream, ids, table, batch, time, targets, margin, shift,
    )?;
    assert_eq!(cuda.rivals, host.rivals, "rivals differ");
    assert_eq!(
        (cuda.loss_before, cuda.loss_after, cuda.accepted),
        (host.loss_before, host.loss_after, host.accepted),
        "losses or accept differ"
    );
    assert_eq!(
        cuda.read.query_gates(),
        host.read.query_gates(),
        "query gates differ"
    );
    assert_eq!(
        cuda.read.key_gates(),
        host.read.key_gates(),
        "key gates differ"
    );
    assert_eq!(cuda, host);
    Ok(cuda)
}

/// Chain steps until one is rejected, checking parity at every step.
fn train_chain(
    engine: &NativeVectorEngine,
    stream: &[i64],
    targets: &[u32],
    shift: u32,
) -> uor_r4_discrete::Result<(Q31FirstPickRead, usize)> {
    let time = targets.len();
    let ids: Vec<u32> = (0..time as u32).collect();
    let table = vec![0; time * 16];
    let mut read = identity_read(16);
    for steps in 0..64 {
        let step = train_parity(
            engine, &read, stream, &ids, &table, 1, time, targets, 0, shift,
        )?;
        if !step.accepted {
            return Ok((read, steps));
        }
        read = step.read;
    }
    panic!("training did not settle in 64 steps");
}

#[test]
fn cuda_read_update_matches_the_host_on_claude_391_fixtures() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    // A wrong pick becomes right in 4 steps at shift 4, and 49 at shift 8.
    let (read, steps) = train_chain(&engine, &fixture(S), &[0, 0, 2], 4)?;
    assert_eq!(steps, 4);
    assert_eq!(
        engine
            .first_pick_read(&read, &fixture(S), &[0, 1, 2], &[0; 48], 1, 3)?
            .picks,
        vec![0, 1, 2]
    );
    let (_, steps) = train_chain(&engine, &fixture(S), &[0, 0, 2], 8)?;
    assert_eq!(steps, 49);
    // The step count does not depend on the stream's scale.
    for scale in [1 << 10, 1 << 28] {
        let (_, steps) = train_chain(&engine, &fixture(scale), &[0, 0, 2], 4)?;
        assert_eq!(steps, 4, "scale {scale}");
    }
    // A margin-0 tie with a later rival is a loss of 1, and trains away.
    let tie = [row(S, 0), row(0, S), row(S / 2, S / 2)].concat();
    let step = train_parity(
        &engine,
        &identity_read(16),
        &tie,
        &[0, 1, 2],
        &[0; 48],
        1,
        3,
        &[0, 0, 1],
        0,
        4,
    )?;
    assert_eq!(step.loss_before, 1);
    train_chain(&engine, &tie, &[0, 0, 1], 4)?;
    // An exact tie sends the rival to the latest row.
    let latest = [row(S, 0), row(S, 0), row(0, S), row(3 * S / 4, S / 2)].concat();
    let step = train_parity(
        &engine,
        &identity_read(16),
        &latest,
        &[0, 1, 2, 3],
        &[0; 64],
        1,
        4,
        &[0, 0, 0, 3],
        0,
        4,
    )?;
    assert_eq!(step.rivals, vec![0, 0, 0, 2]);
    // A margin widens a right pick that sits too close.
    let step = train_parity(
        &engine,
        &identity_read(16),
        &fixture(S),
        &[0, 1, 2],
        &[0; 48],
        1,
        3,
        &[0, 0, 1],
        S as u64,
        4,
    )?;
    assert!(step.accepted && step.loss_before == S as u128 / 2);
    // Two swapped windows overshoot at shift 0, and the candidate is rejected.
    let swapped = [fixture(S), row(0, S), row(S, 0), row(S / 2, S / 2)].concat();
    let step = train_parity(
        &engine,
        &identity_read(16),
        &swapped,
        &[0, 1, 2, 0, 1, 2],
        &[0; 48],
        2,
        3,
        &[0, 0, 2, 0, 0, 2],
        0,
        0,
    )?;
    assert_eq!((step.loss_before, step.loss_after), (131_072, 393_216));
    assert!(!step.accepted);
    Ok(())
}

#[test]
fn cuda_read_update_matches_the_host_on_seeded_batches() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    let mut rng = XorShift(0xa54f_f53a_5f1d_36f1);
    let (batch, time, vocab) = (4, 32, 40);
    let mut accepted = 0;
    let mut rejected = 0;
    for width in [64, 256] {
        for full_range in [false, true] {
            let word = |rng: &mut XorShift| {
                if full_range {
                    rng.admitted()
                } else {
                    (rng.next() % (1 << 19)) as i64 - S
                }
            };
            let quaternions = |rng: &mut XorShift| -> Vec<[i64; 4]> {
                (0..width / 4)
                    .map(|_| {
                        [
                            rng.admitted(),
                            rng.admitted(),
                            rng.admitted(),
                            rng.admitted(),
                        ]
                    })
                    .collect()
            };
            let stream: Vec<i64> = (0..batch * time * width).map(|_| word(&mut rng)).collect();
            let table: Vec<i64> = (0..vocab * width).map(|_| rng.small()).collect();
            let ids: Vec<u32> = (0..batch * time)
                .map(|_| (rng.next() % vocab as u64) as u32)
                .collect();
            let targets: Vec<u32> = (0..batch * time)
                .map(|row| {
                    let t = (row % time) as u64;
                    if t == 0 || rng.next().is_multiple_of(4) {
                        0
                    } else {
                        (1 + rng.next() % t) as u32
                    }
                })
                .collect();
            let start = Q31FirstPickRead::new(
                quaternions(&mut rng),
                quaternions(&mut rng),
                (0..width).map(|_| rng.small()).collect(),
                4,
            )?;
            // `u64::MAX` puts every row's hinge above 2^64, into the high loss word.
            for margin in [0, 1 << 20, u64::MAX] {
                for shift in [0, 4, 31] {
                    let mut read = start.clone();
                    for _ in 0..3 {
                        let step = train_parity(
                            &engine, &read, &stream, &ids, &table, batch, time, &targets, margin,
                            shift,
                        )?;
                        if step.accepted {
                            accepted += 1;
                        } else {
                            rejected += 1;
                        }
                        read = step.read;
                    }
                }
            }
        }
    }
    // Both branches of the accept rule were compared.
    assert!(
        accepted > 0 && rejected > 0,
        "accepted {accepted}, rejected {rejected}"
    );
    Ok(())
}

#[test]
fn cuda_read_update_refuses_what_the_host_refuses() -> uor_r4_discrete::Result<()> {
    let Some(engine) = engine_or_skip()? else {
        return Ok(());
    };
    let (stream, ids, table) = (fixture(S), [0, 1, 2], [0; 48]);
    let read = identity_read(16);
    // (name, stream, targets, step shift, the rule the refusal names)
    type Case<'a> = (&'a str, &'a [i64], &'a [u32], u32, &'a str);
    let cases: [Case; 5] = [
        (
            "short targets",
            &stream,
            &[0, 0],
            4,
            "needs batch * time targets",
        ),
        (
            "target on row 0",
            &stream,
            &[1, 0, 0],
            4,
            "an earlier row of its window",
        ),
        (
            "target on itself",
            &stream,
            &[0, 0, 3],
            4,
            "an earlier row of its window",
        ),
        (
            "shift 32",
            &stream,
            &[0, 0, 2],
            32,
            "step shift must be in 0..=31",
        ),
        (
            "short stream",
            &stream[16..],
            &[0, 0, 2],
            4,
            "batch * time * width",
        ),
    ];
    for (name, words, targets, shift, rule) in cases {
        let cuda = engine
            .read_train_pick_step(&read, words, &ids, &table, 1, 3, targets, 0, shift)
            .expect_err(name)
            .to_string();
        let host = NativeVectorEngine::read_train_pick_step_reference(
            &read, words, &ids, &table, 1, 3, targets, 0, shift,
        )
        .expect_err(name)
        .to_string();
        assert!(
            cuda.contains(rule),
            "{name}: device {cuda:?} lacks {rule:?}"
        );
        assert!(host.contains(rule), "{name}: host {host:?} lacks {rule:?}");
    }
    // An empty batch is no step on both sides.
    let empty = train_parity(&engine, &read, &[], &[], &table, 0, 3, &[], 0, 4)?;
    assert!(!empty.accepted && empty.rivals.is_empty());
    Ok(())
}
