//! claude-439: the Pick read of the trunk `seed1.d11` with Identity transport on the GPU,
//! with host helpers and a laptop check against the trunk's own CPU oracle.
#![allow(dead_code)]

use crate::{fail, Failure};
use std::path::Path;
use uor_r4_discrete::kn_pick::PickRow;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

pub const BLOCKS: usize = 4;
pub const VECTOR: usize = 32;
pub const ABSENT: u32 = u32::MAX;
pub const TRUNK_SHA256: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
pub const C3_SEED: u64 = 439;
/// Windows per device call: the device's maximum. A forward call costs about 100 ms whatever
/// it holds (claude-439 probe on wick, 19 Nov), and the tape does not depend on the batch size.
pub const READ_BATCH: usize = 4096;

/// Loads the seed-1 trunk and refuses any other artifact or shape.
pub fn load_trunk(path: &Path) -> Result<IntegerDiscreteModel, Failure> {
    let model = IntegerDiscreteModel::load(path)?;
    if model.artifact_sha256() != TRUNK_SHA256
        || model.shape().blocks as usize != BLOCKS
        || model.shape().context != 192
    {
        return Err(fail("unexpected seed-1 Relation D11 source"));
    }
    Ok(model)
}

/// Bit 2d is set iff x[d] > 0, bit 2d + 1 iff x[d] < 0.
fn root_code(x: &[i32]) -> u16 {
    let mut code = 0u16;
    for (d, &v) in x.iter().enumerate() {
        if v > 0 {
            code |= 1 << (2 * d);
        } else if v < 0 {
            code |= 1 << (2 * d + 1);
        }
    }
    code
}

/// The root code of every block of every token row of `table`.
pub fn root_codes(table: &[i32]) -> Result<Vec<[u16; BLOCKS]>, Failure> {
    if table.len() % VECTOR != 0 {
        return Err(fail(
            "token table length is not a multiple of the vector width",
        ));
    }
    Ok(table
        .chunks(VECTOR)
        .map(|row| std::array::from_fn(|b| root_code(&row[b * 8..(b + 1) * 8])))
        .collect())
}

/// Row i of the result is row perm[i] of `table`, as `permute_code_rows` moves codes.
pub fn permute_table_rows(table: &[i32], perm: &[usize]) -> Result<Vec<i32>, Failure> {
    if table.len() != perm.len() * VECTOR || perm.iter().any(|&from| from >= perm.len()) {
        return Err(fail("permutation does not match the token table"));
    }
    Ok(perm
        .iter()
        .flat_map(|&from| table[from * VECTOR..(from + 1) * VECTOR].iter().copied())
        .collect())
}

/// One Pick row from block 0's five words over window `w` with held target `target`.
pub fn row_from_words(w: &[u16], target: u16, block0: &[u32; 5]) -> Result<PickRow, Failure> {
    let (pick, distance) = (block0[0], block0[1]);
    if pick == ABSENT {
        if distance != ABSENT {
            return Err(fail("absent pick with a distance"));
        }
        return Ok(PickRow {
            class: 0,
            hit: false,
            pick: None,
            distance: None,
        });
    }
    let next = pick as usize + 1;
    if next >= w.len() {
        return Err(fail("pick successor outside the window"));
    }
    Ok(PickRow {
        class: if distance == 0 { 1 } else { 2 },
        hit: w[next] == target,
        pick: Some(pick),
        distance: Some(distance),
    })
}

/// (T, window) for each non-BOS position T of `stream`, by kn_pick's window rule.
pub fn windows(stream: &[u16], context: usize) -> Result<Vec<(usize, &[u16])>, Failure> {
    if stream.first() != Some(&0) {
        return Err(fail("the stream must start with BOS"));
    }
    let mut out = Vec::new();
    let mut s = 0;
    for t in 1..stream.len() {
        if stream[t - 1] == 0 {
            s = t - 1;
        }
        if stream[t] != 0 {
            out.push((t, &stream[s.max(t.saturating_sub(context))..t]));
        }
    }
    Ok(out)
}

/// The trunk's CPU oracle row and block-0 words at every target of `stream`.
pub fn oracle_rows(
    model: &IntegerDiscreteModel,
    stream: &[u16],
    context: usize,
) -> Result<Vec<(PickRow, [u32; 5])>, Failure> {
    windows(stream, context)?
        .into_iter()
        .map(|(t, w)| {
            let prefix: Vec<u32> = w.iter().map(|&id| id as u32).collect();
            let seen = model.observe_identity_pick(&prefix)?;
            let block0 = *seen
                .block_words
                .first()
                .ok_or_else(|| fail("no block words"))?;
            Ok((row_from_words(w, stream[t], &block0)?, block0))
        })
        .collect()
}

#[cfg(feature = "cuda")]
pub use gpu::*;

#[cfg(feature = "cuda")]
mod gpu {
    use super::*;
    use std::time::Instant;
    use uor_r4_discrete::{
        device::{Device, FLOCK_TRACE_HEADER_WORDS, FLOCK_TRACE_LEVEL_WORDS},
        normalization_mechanism::NormalizationMode,
        position_mechanism::{PositionConfig, PositionMode, PositionStepTable},
        Identity, ObjectivePolicy, ReadInitialization, ReadSpec, Spec,
    };

    pub struct PickDevice {
        device: Device,
        stride: usize,
    }

    pub struct KeptWords {
        pub ordinal: usize,
        pub blocks: [[u32; 5]; BLOCKS],
    }

    pub struct DeviceTape {
        pub rows: Vec<PickRow>,
        pub words: Vec<KeptWords>,
    }

    impl PickDevice {
        /// codex's CUDA Pick device with Identity transport; `permutation` moves token rows.
        pub fn new(
            model: &IntegerDiscreteModel,
            permutation: Option<&[usize]>,
        ) -> Result<Self, Failure> {
            let shape = model.shape();
            let position = PositionConfig::new(
                PositionMode::IdentityControlV1,
                PositionStepTable::identity(shape.blocks)?,
            );
            let fresh = Device::new_with_position(
                Spec {
                    vocabulary: shape.vocabulary,
                    blocks: shape.blocks,
                    context: shape.context,
                    kept_cells: shape.kept_cells,
                },
                ReadSpec::Pick,
                ObjectivePolicy::NearestAddressMarginV1,
                ReadInitialization::ActiveLeafV1,
                NormalizationMode::ScaleMatchedSnapV1,
                Some(position),
                shape.seed,
                0,
            )?;
            let mut snapshot = fresh.snapshot()?;
            drop(fresh);
            let (table, routes, gates) = model.identity_pick_inventory()?;
            snapshot.table = match permutation {
                Some(perm) => permute_table_rows(&table, perm)?,
                None => table,
            };
            (snapshot.routes, snapshot.gates) = (routes, gates);
            snapshot.epoch = shape.epoch;
            let expected_geometry = snapshot.table.clone();
            let expected_gates = snapshot.gates.clone();
            let expected_routes = snapshot.routes.clone();
            let device = Device::from_snapshot(snapshot, 0)?;
            let returned = device.snapshot()?;
            if returned.table != expected_geometry
                || returned.gates != expected_gates
                || returned.routes != expected_routes
                || returned
                    .position
                    .as_ref()
                    .map(|config| config.mode.device_word())
                    != Some(3)
            {
                return Err(fail(
                    "CUDA Pick state readback differs from the explicit snapshot",
                ));
            }
            let stride = FLOCK_TRACE_HEADER_WORDS + shape.stages as usize * FLOCK_TRACE_LEVEL_WORDS;
            Ok(Self { device, stride })
        }

        /// The device Pick row at every target of `stream`, keeping all blocks' words where asked.
        pub fn rows(
            &self,
            stream: &[u16],
            context: usize,
            keep_words: &dyn Fn(usize) -> bool,
        ) -> Result<DeviceTape, Failure> {
            let (tape, _times) = self.rows_batched(stream, context, keep_words, READ_BATCH)?;
            Ok(tape)
        }

        /// `rows` with `batch` windows per device call, and the time spent in each part
        /// (microseconds: identities, carry, forward, trace readback, row parse).
        pub fn rows_batched(
            &self,
            stream: &[u16],
            context: usize,
            keep_words: &dyn Fn(usize) -> bool,
            batch_rows: usize,
        ) -> Result<(DeviceTape, [u128; 5]), Failure> {
            let all = windows(stream, context)?;
            let mut tape = DeviceTape {
                rows: Vec::with_capacity(all.len()),
                words: Vec::new(),
            };
            let mut times = [0u128; 5];
            for (chunk, batch) in all.chunks(batch_rows).enumerate() {
                let at = Instant::now();
                let identities: Vec<Vec<_>> = batch
                    .iter()
                    .enumerate()
                    .map(|(row, (_, w))| {
                        w.iter()
                            .enumerate()
                            .map(|(pos, &token)| Identity {
                                token: token as u32,
                                occurrence: pos as u64,
                                record: (chunk * batch_rows + row) as u64,
                                version: 0,
                            })
                            .collect()
                    })
                    .collect();
                let placeholder: Vec<u32> =
                    batch.iter().map(|(_, w)| w[w.len() - 1] as u32).collect();
                times[0] += at.elapsed().as_micros();
                let at = Instant::now();
                let carried = self.device.carry_windows(&identities)?;
                times[1] += at.elapsed().as_micros();
                let at = Instant::now();
                let forward = self.device.forward_traced(&carried, &placeholder)?;
                times[2] += at.elapsed().as_micros();
                let at = Instant::now();
                let trace = forward
                    .flock_trace()?
                    .ok_or_else(|| fail("missing Pick device trace"))?;
                times[3] += at.elapsed().as_micros();
                let at = Instant::now();
                if trace.len() != batch.len() * BLOCKS * self.stride {
                    return Err(fail("unexpected Pick trace shape"));
                }
                for (row, &(t, w)) in batch.iter().enumerate() {
                    let mut blocks = [[ABSENT; 5]; BLOCKS];
                    for (b, words) in blocks.iter_mut().enumerate() {
                        let rec = &trace[(row * BLOCKS + b) * self.stride..][..self.stride];
                        if rec[5..].iter().any(|&word| word != ABSENT) {
                            return Err(fail("unexpected nonempty Pick trace remainder"));
                        }
                        words.copy_from_slice(&rec[..5]);
                    }
                    let ordinal = chunk * batch_rows + row;
                    tape.rows.push(row_from_words(w, stream[t], &blocks[0])?);
                    if keep_words(ordinal) {
                        tape.words.push(KeptWords { ordinal, blocks });
                    }
                }
                times[4] += at.elapsed().as_micros();
            }
            Ok((tape, times))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::kn_pick::{code_pick, code_pick_rows, permute_code_rows, row_permutation};

    fn splitmix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    #[test]
    fn root_codes_set_sign_bits() {
        let mut table = vec![0i32; 2 * VECTOR];
        table[0] = 3; // token 0 block 0 d0 > 0: bit 0
        table[1] = -1; // d1 < 0: bit 3
        table[VECTOR + 3 * 8 + 7] = -2; // token 1 block 3 d7 < 0: bit 15
        table[VECTOR + 8 + 2] = 1; // token 1 block 1 d2 > 0: bit 4
        let codes = root_codes(&table).unwrap();
        assert_eq!(codes, vec![[0b1001, 0, 0, 0], [0, 1 << 4, 0, 1 << 15]]);
        assert!(root_codes(&table[1..]).is_err());
    }

    #[test]
    fn permuted_table_matches_permuted_codes() {
        let mut state = 7u64;
        let table: Vec<i32> = (0..4096 * VECTOR)
            .map(|_| (splitmix(&mut state) % 5) as i32 - 2)
            .collect();
        let moved = permute_table_rows(&table, &row_permutation(4096, C3_SEED)).unwrap();
        let codes = root_codes(&table).unwrap();
        assert_eq!(
            root_codes(&moved).unwrap(),
            permute_code_rows(&codes, C3_SEED)
        );
    }

    #[test]
    fn the_read_fills_each_device_call() {
        assert_eq!(READ_BATCH, uor_r4_discrete::corpus::MAX_BATCH_ROWS as usize);
    }

    #[test]
    fn row_from_words_maps_classes() {
        let w = [0u16, 5, 6, 5];
        let none = row_from_words(&w, 6, &[ABSENT, ABSENT, 0, 1, 0]).unwrap();
        assert_eq!(
            none,
            PickRow {
                class: 0,
                hit: false,
                pick: None,
                distance: None
            }
        );
        assert!(row_from_words(&w, 6, &[ABSENT, 2, 0, 1, 0]).is_err());
        let exact = row_from_words(&w, 6, &[1, 0, 0, 1, 0]).unwrap();
        assert_eq!(
            exact,
            PickRow {
                class: 1,
                hit: true,
                pick: Some(1),
                distance: Some(0)
            }
        );
        let near = row_from_words(&w, 7, &[1, 3, 2, 1, 0]).unwrap();
        assert_eq!(
            near,
            PickRow {
                class: 2,
                hit: false,
                pick: Some(1),
                distance: Some(3)
            }
        );
        assert!(row_from_words(&w, 6, &[3, 0, 0, 1, 0]).is_err());
    }

    #[test]
    #[ignore]
    fn host_pick_equals_the_trunk_oracle() {
        let path = std::env::var("UOR_PICK_TRUNK")
            .expect("set UOR_PICK_TRUNK to the seed1.d11 trunk (gate G2 runs this test)");
        let model = load_trunk(Path::new(&path)).unwrap();
        let (blocks, _, _) = model.identity_pick_inventory().unwrap();
        let codes = root_codes(&blocks).unwrap();
        let mut state = 439u64;
        let mut draw = |n: usize| -> Vec<u16> {
            (0..n)
                .map(|_| (2 + splitmix(&mut state) % 4094) as u16)
                .collect()
        };
        let mut body = draw(300);
        let repeat: Vec<u16> = body[20..80].to_vec();
        body[100..160].copy_from_slice(&repeat);
        let mut stream = vec![0u16];
        stream.extend(&body);
        stream.extend([1, 0]);
        stream.extend(draw(250));
        stream.push(1);
        let host = code_pick_rows(&stream, &codes, 192).unwrap();
        let oracle = oracle_rows(&model, &stream, 192).unwrap();
        assert_eq!(host.len(), oracle.len());
        let mut exact = 0;
        for (((t, w), row), (want, words)) in windows(&stream, 192)
            .unwrap()
            .into_iter()
            .zip(&host)
            .zip(&oracle)
        {
            assert_eq!(row, want, "target {t}");
            if let (Some((p, d, ties)), Some(_)) = (code_pick(&codes, w), want.pick) {
                assert_eq!(
                    [p as u32, d, ties],
                    [words[0], words[1], words[2]],
                    "target {t}"
                );
            }
            exact += usize::from(want.distance == Some(0));
        }
        assert!(exact >= 50, "only {exact} exact picks");
    }

    /// The device read gives the same tape at every batch size, and prints where it spends time.
    #[cfg(feature = "cuda")]
    #[test]
    #[ignore]
    fn device_read_is_batch_invariant() {
        let path = std::env::var("UOR_PICK_TRUNK").expect("set UOR_PICK_TRUNK");
        let model = load_trunk(Path::new(&path)).unwrap();
        let mut state = 4391u64;
        let mut stream = Vec::new();
        for r in 0..24 {
            stream.push(0u16);
            let len = 100 + (r * 37) % 500;
            let body: Vec<u16> = (0..len)
                .map(|_| (2 + splitmix(&mut state) % 4094) as u16)
                .collect();
            stream.extend(&body);
            stream.extend(&body[..len / 3]);
            stream.push(1);
        }
        let device = PickDevice::new(&model, None).unwrap();
        let keep = |k: usize| k % 3 == 0;
        let mut tapes = Vec::new();
        for batch in [256, 4096, 1000] {
            let at = std::time::Instant::now();
            let (tape, times) = device.rows_batched(&stream, 192, &keep, batch).unwrap();
            eprintln!(
                "batch {batch}: {} rows in {} ms; us identities {} carry {} forward {} trace {} parse {}",
                tape.rows.len(),
                at.elapsed().as_millis(),
                times[0],
                times[1],
                times[2],
                times[3],
                times[4]
            );
            tapes.push(tape);
        }
        for tape in &tapes[1..] {
            assert_eq!(tape.rows, tapes[0].rows);
            assert_eq!(tape.words.len(), tapes[0].words.len());
            for (a, b) in tape.words.iter().zip(&tapes[0].words) {
                assert_eq!((a.ordinal, a.blocks), (b.ordinal, b.blocks));
            }
        }
        assert!(tapes[0].rows.iter().any(|r| r.distance == Some(0)));
    }
}
