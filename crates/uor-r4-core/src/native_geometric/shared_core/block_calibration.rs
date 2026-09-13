//! Offline exhaustive angular-cap block fitting. Serving computation is unchanged.
use super::*;
use std::time::Instant;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BlockCalibrationConfig {
    pub(super) max_seconds: u64,
}

#[derive(Serialize, Deserialize)]
pub struct BlockCalibrationReport {
    pub parent: String,
    pub data_digest: String,
    pub before: Metrics,
    pub after: Metrics,
    pub root_branch_errors: usize,
    pub ascii_branch_errors: usize,
    pub root_positions: usize,
    pub ascii_positions: usize,
    pub nodes: usize,
    pub settings: usize,
    pub elapsed_ms: u128,
    pub improvement_tolerance: f64,
}

type Rows = Vec<([u16; LANES], bool)>;
type Tail = [[[usize; 10]; 10]; 2];
const LOSS_TOLERANCE: f64 = 1e-10;

fn branch_loss(score: i16, target: bool) -> f64 {
    let margin = f64::from(score) / 4.0;
    libm::log1p(libm::exp(if target { -margin } else { margin }))
}

fn losses(tail: &Tail, thresholds: [i16; 2], deltas: &[[f64; 18]; 2]) -> [f64; 2] {
    let base = tail[0][0][0] as f64 * branch_loss(-9, false)
        + tail[1][0][0] as f64 * branch_loss(-9, true);
    let mut result = [base; 2];
    // E[L(S)] = L(-9) + sum_k P(S >= k) * (L(k)-L(k-1)).
    for k in -8i16..=9 {
        let a = (k + thresholds[0] + 4).clamp(0, 9) as usize;
        let b = (k + thresholds[1] + 4).clamp(0, 9) as usize;
        for tag in 0..2 {
            let both = tail[tag][a][b];
            let either = tail[tag][a][0] + tail[tag][0][b] - both;
            let delta = deltas[tag][(k + 8) as usize];
            result[0] += both as f64 * delta;
            result[1] += either as f64 * delta;
        }
    }
    result
}

fn optimize(
    model: &SharedCore,
    rows: &Rows,
    depth: usize,
    current: CapBranch,
    start: Instant,
    max_seconds: u64,
) -> Result<(CapBranch, usize)> {
    let mut best = rows
        .iter()
        .map(|&(roots, y)| {
            branch_loss(
                model.cap_score(roots, current, depth, &mut Work::default()),
                y,
            )
        })
        .sum::<f64>();
    let mut chosen = current;
    let deltas: [[f64; 18]; 2] = std::array::from_fn(|tag| {
        std::array::from_fn(|i| {
            let k = i as i16 - 8;
            branch_loss(k, tag == 1) - branch_loss(k - 1, tag == 1)
        })
    });
    let lane = depth & 3;
    let scores: [Vec<Vec<i16>>; 2] = std::array::from_fn(|side| {
        (0..ROOTS)
            .map(|landmark| {
                rows.iter()
                    .map(|&(roots, _)| {
                        model.relative_score(
                            roots[(lane + side) & 3],
                            landmark as u16,
                            &mut Work::default(),
                        )
                    })
                    .collect()
            })
            .collect()
    });
    let mut settings = 0;
    for first in 0..ROOTS {
        if start.elapsed().as_secs() >= max_seconds {
            return Err(CoreError::InvalidInput(
                "block calibration time limit; no partial artifact",
            ));
        }
        for second in 0..ROOTS {
            let mut tail = [[[0usize; 10]; 10]; 2];
            for (i, &(_, y)) in rows.iter().enumerate() {
                let a = scores[0][first][i];
                let b = scores[1][second][i];
                if !(-4..=4).contains(&a) || !(-4..=4).contains(&b) {
                    return Err(CoreError::InvalidArtifact(
                        "angular rank outside block fitter domain",
                    ));
                }
                tail[usize::from(y)][(a + 4) as usize][(b + 4) as usize] += 1;
            }
            for tag in &mut tail {
                for a in (0..9).rev() {
                    for b in (0..9).rev() {
                        tag[a][b] += tag[a + 1][b] + tag[a][b + 1] - tag[a + 1][b + 1];
                    }
                }
            }
            for t0 in -5..=5 {
                for t1 in -5..=5 {
                    for (op, nll) in losses(&tail, [t0, t1], &deltas).into_iter().enumerate() {
                        settings += 1;
                        if nll < best - LOSS_TOLERANCE {
                            best = nll;
                            chosen = CapBranch {
                                landmarks: [first as u16, second as u16],
                                thresholds: [t0, t1],
                                union: op == 1,
                            };
                        }
                    }
                }
            }
        }
    }
    let actual: f64 = rows
        .iter()
        .map(|&(roots, y)| {
            branch_loss(
                model.cap_score(roots, chosen, depth, &mut Work::default()),
                y,
            )
        })
        .sum();
    if (actual - best).abs() > 1e-8 {
        return Err(CoreError::InvalidInput("block likelihood replay mismatch"));
    }
    Ok((chosen, settings))
}

impl SharedCore {
    /// Optimize each observed emission node's complete finite parameter block.
    /// Select only by likelihood; no branch-error gate enters this learner.
    pub fn calibrate_emission_blocks(
        &self,
        documents: &[Vec<u8>],
        max_seconds: u64,
    ) -> Result<(Self, BlockCalibrationReport)> {
        training::validate_data(documents)?;
        if self.artifact.calibrated.is_none() || max_seconds == 0 || max_seconds > 120 {
            return Err(CoreError::InvalidInput(
                "schema-2 emitter and block time bound required",
            ));
        }
        let start = Instant::now();
        let before = self.evaluate(documents, Intervention::Full)?;
        let mut rows = vec![Rows::new(); 512];
        let mut depths = [0usize; 512];
        for doc in documents {
            let mut session = self.session(Intervention::Full);
            for target in doc
                .iter()
                .map(|&b| u16::from(b))
                .chain(std::iter::once(EOS))
            {
                let (mut lo, mut hi, mut node, mut depth) = (0, 257, 0, 0);
                while hi - lo > 1 {
                    let mid = (lo + hi) >> 1;
                    let right = target >= mid;
                    rows[node].push((session.state_roots(), right));
                    depths[node] = depth;
                    if right {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                    node = (node << 1) + 1 + usize::from(right);
                    depth += 1;
                }
                if target != EOS {
                    session.observe(target as u8)?;
                }
            }
        }
        let mut model = self.clone();
        let (mut nodes, mut settings) = (0, 0);
        for node in 0..512 {
            if rows[node].is_empty() {
                continue;
            }
            let current = model
                .artifact
                .calibrated
                .as_ref()
                .ok_or(CoreError::InvalidArtifact("emitter"))?
                .branches[node];
            let (branch, count) = optimize(
                &model,
                &rows[node],
                depths[node],
                current,
                start,
                max_seconds,
            )?;
            model
                .artifact
                .calibrated
                .as_mut()
                .ok_or(CoreError::InvalidArtifact("emitter"))?
                .branches[node] = branch;
            nodes += 1;
            settings += count;
        }
        let mut hash = blake3::Hasher::new();
        hash.update(b"uor-r4.shared-core-block-calibration/1");
        for doc in documents {
            hash.update(&(doc.len() as u64).to_le_bytes());
            hash.update(doc);
        }
        let digest = format!("blake3:{}", hash.finalize());
        let emission = model
            .artifact
            .calibrated
            .as_mut()
            .ok_or(CoreError::InvalidArtifact("emitter"))?;
        emission.parent = self.cid.clone();
        emission.calibration_data = Some(digest.clone());
        emission.calibration_passes = 1;
        emission.block_config = Some(BlockCalibrationConfig { max_seconds });
        model.artifact.implementation = Self::implementation_digest();
        model.artifact.training_digest = None;
        model.artifact.fit_config = None;
        model.artifact.training_parent = None;
        model.refresh_identity()?;
        if start.elapsed().as_secs() >= max_seconds {
            return Err(CoreError::InvalidInput(
                "block calibration deadline before final evaluation",
            ));
        }
        let after = model.evaluate(documents, Intervention::Full)?;
        if after.mean_nll > before.mean_nll + 1e-10 {
            return Err(CoreError::InvalidInput("block calibration increased loss"));
        }
        let errors = |node: usize| -> Result<usize> {
            let branch = model
                .artifact
                .calibrated
                .as_ref()
                .ok_or(CoreError::InvalidArtifact("emitter"))?
                .branches[node];
            Ok(rows[node]
                .iter()
                .filter(|&&(roots, y)| {
                    (model.cap_score(roots, branch, depths[node], &mut Work::default()) > 0) != y
                })
                .count())
        };
        let report = BlockCalibrationReport {
            parent: self.cid.clone(),
            data_digest: digest,
            before,
            after,
            root_branch_errors: errors(0)?,
            ascii_branch_errors: errors(1)?,
            root_positions: rows[0].len(),
            ascii_positions: rows[1].len(),
            nodes,
            settings,
            elapsed_ms: start.elapsed().as_millis(),
            improvement_tolerance: LOSS_TOLERANCE,
        };
        if start.elapsed().as_secs() >= max_seconds {
            return Err(CoreError::InvalidInput(
                "block calibration deadline before artifact return",
            ));
        }
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_fit_preserves_state_parent_and_records_complete_search() {
        let legacy = SharedCore::initialized(745).unwrap();
        let model = legacy.upgrade_calibrated().unwrap();
        let bytes = model.to_bytes().unwrap();
        let documents = vec![b"a".to_vec()];
        let (fitted, report) = model.calibrate_emission_blocks(&documents, 120).unwrap();
        assert_eq!(report.settings, report.nodes * 3_484_800);
        assert_eq!(report.root_positions, 2);
        assert_eq!(report.ascii_positions, 1);
        assert!(report.after.mean_nll <= report.before.mean_nll);
        assert_eq!(report.parent, model.artifact_cid());
        assert_eq!(model.to_bytes().unwrap(), bytes);
        assert_eq!(model.artifact.parameters, fitted.artifact.parameters);
        assert_eq!(model.artifact.geometry, fitted.artifact.geometry);
        let mut a = model.session(Intervention::Full);
        let mut b = fitted.session(Intervention::Full);
        for byte in 0..=255 {
            a.observe(byte).unwrap();
            b.observe(byte).unwrap();
            assert_eq!(a.state_roots(), b.state_roots());
        }
        let wire = fitted.to_bytes().unwrap();
        let restored = SharedCore::from_bytes(&wire).unwrap();
        assert_eq!(restored.to_bytes().unwrap(), wire);
        let mut corrupt: serde_json::Value = serde_json::from_slice(&wire).unwrap();
        corrupt["calibrated"]["block_config"]["max_seconds"] = 121.into();
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&corrupt).unwrap()).is_err());
        corrupt["calibrated"]["block_config"]["max_seconds"] = 120.into();
        corrupt["calibrated"]["calibration_passes"] = 3.into();
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&corrupt).unwrap()).is_err());
        assert!(model.calibrate_emission_blocks(&documents, 0).is_err());
        assert!(legacy.calibrate_emission_blocks(&documents, 120).is_err());
    }

    #[test]
    fn block_tail_loss_matches_direct_for_all_signed_ranks_and_thresholds() {
        let mut tail = [[[0usize; 10]; 10]; 2];
        for a in 0..9 {
            for b in 0..9 {
                tail[0][a][b] = 1;
                tail[1][a][b] = 2;
            }
        }
        for tag in &mut tail {
            for a in (0..9).rev() {
                for b in (0..9).rev() {
                    tag[a][b] += tag[a + 1][b] + tag[a][b + 1] - tag[a + 1][b + 1];
                }
            }
        }
        let delta = std::array::from_fn(|tag| {
            std::array::from_fn(|i| {
                let k = i as i16 - 8;
                branch_loss(k, tag == 1) - branch_loss(k - 1, tag == 1)
            })
        });
        for t0 in -5..=5 {
            for t1 in -5..=5 {
                for (op, nll) in losses(&tail, [t0, t1], &delta).into_iter().enumerate() {
                    let mut direct = 0.0;
                    for a in -4i16..=4 {
                        for b in -4i16..=4 {
                            let score = if op == 0 {
                                (a - t0).min(b - t1)
                            } else {
                                (a - t0).max(b - t1)
                            };
                            direct += branch_loss(score, false) + 2.0 * branch_loss(score, true);
                        }
                    }
                    assert!((nll - direct).abs() < 1e-9);
                }
            }
        }
    }
}
