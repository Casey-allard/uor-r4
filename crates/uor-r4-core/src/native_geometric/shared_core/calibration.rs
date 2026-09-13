//! Host-only calibration of the new branch class on fixed recurrent states.
//! This records a separate learned checkpoint before any joint state update.
use super::*;

#[derive(Serialize, Deserialize)]
pub struct CalibrationReport {
    pub parent: String,
    pub data_digest: String,
    pub passes: usize,
    pub before: Metrics,
    pub after: Metrics,
    pub root_branch_errors: usize,
    pub ascii_branch_errors: usize,
    pub root_positions: usize,
    pub ascii_positions: usize,
}

impl SharedCore {
    pub fn upgrade_calibrated(&self) -> Result<Self> {
        if self.artifact.angular_tree.is_some() {
            return Err(CoreError::InvalidInput(
                "angular emission cannot be upgraded as a cap",
            ));
        }
        if self.artifact.calibrated.is_some() {
            return Err(CoreError::InvalidInput("already calibrated schema"));
        }
        let mut model = self.clone();
        model.artifact.schema = CALIBRATED_SCHEMA.into();
        model.artifact.implementation = Self::implementation_digest();
        model.artifact.calibrated = Some(CalibratedEmission {
            branches: (0..512)
                .map(|node| CapBranch {
                    landmarks: [self.artifact.parameters[OUTPUT + node]; 2],
                    thresholds: [0; 2],
                    union: false,
                })
                .collect(),
            parent: self.cid.clone(),
            calibration_data: None,
            calibration_passes: 0,
            block_config: None,
        });
        model.artifact.training_digest = None;
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = None;
        model.artifact.training_parent = None;
        model.refresh_identity()?;
        Ok(model)
    }

    /// Exactly three deterministic coordinate passes with recurrence fixed.
    /// A failed gate is failure at this dose, not a representation impossibility.
    pub fn calibrate_emission(&self, documents: &[Vec<u8>]) -> Result<(Self, CalibrationReport)> {
        if self.artifact.angular_tree.is_some() {
            return Err(CoreError::InvalidInput(
                "angular emission cannot be calibrated as a cap",
            ));
        }
        training::validate_data(documents)?;
        if self.artifact.calibrated.is_none() {
            return Err(CoreError::InvalidInput("calibrated schema required"));
        }
        let before = self.evaluate(documents, Intervention::Full)?;
        let mut rows = vec![Vec::<([u16; 4], bool)>::new(); 512];
        let mut depths = [0usize; 512];
        for doc in documents {
            let mut session = self.session(Intervention::Full);
            for target in doc
                .iter()
                .map(|&b| u16::from(b))
                .chain(std::iter::once(EOS))
            {
                let (mut lo, mut hi, mut node, mut depth) = (0u16, 257u16, 0usize, 0usize);
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
        for _ in 0..3 {
            for node in 0..512 {
                if rows[node].is_empty() {
                    continue;
                }
                let mut branch = model
                    .artifact
                    .calibrated
                    .as_ref()
                    .ok_or(CoreError::InvalidArtifact("emitter"))?
                    .branches[node];
                let objective = |b: CapBranch| -> f64 {
                    rows[node]
                        .iter()
                        .map(|&(roots, right)| {
                            let score = f64::from(model.cap_score(
                                roots,
                                b,
                                depths[node],
                                &mut Work::default(),
                            )) / 4.0;
                            libm::log1p(libm::exp(if right { -score } else { score }))
                        })
                        .sum()
                };
                for field in 0..5 {
                    let mut best = objective(branch);
                    let mut chosen = branch;
                    let count = if field < 2 {
                        120
                    } else if field < 4 {
                        11
                    } else {
                        2
                    };
                    for value in 0..count {
                        let mut candidate = branch;
                        match field {
                            0 | 1 => candidate.landmarks[field] = value as u16,
                            2 | 3 => candidate.thresholds[field - 2] = value as i16 - 5,
                            _ => candidate.union = value != 0,
                        }
                        let score = objective(candidate);
                        if score < best {
                            best = score;
                            chosen = candidate;
                        }
                    }
                    branch = chosen;
                }
                model
                    .artifact
                    .calibrated
                    .as_mut()
                    .ok_or(CoreError::InvalidArtifact("emitter"))?
                    .branches[node] = branch;
            }
        }
        let mut hash = blake3::Hasher::new();
        hash.update(b"uor-r4.shared-core-calibration/1");
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
        emission.calibration_passes = 3;
        emission.block_config = None;
        model.artifact.implementation = Self::implementation_digest();
        model.artifact.training_digest = None;
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = None;
        model.artifact.training_parent = None;
        model.refresh_identity()?;
        let after = model.evaluate(documents, Intervention::Full)?;
        let errors = |node: usize| -> Result<usize> {
            let branch = model
                .artifact
                .calibrated
                .as_ref()
                .ok_or(CoreError::InvalidArtifact("emitter"))?
                .branches[node];
            Ok(rows[node]
                .iter()
                .filter(|&&(roots, right)| {
                    (model.cap_score(roots, branch, depths[node], &mut Work::default()) > 0)
                        != right
                })
                .count())
        };
        let report = CalibrationReport {
            parent: self.cid.clone(),
            data_digest: digest,
            passes: 3,
            before,
            after,
            root_branch_errors: errors(0)?,
            ascii_branch_errors: errors(1)?,
            root_positions: rows[0].len(),
            ascii_positions: rows[1].len(),
        };
        Ok((model, report))
    }
}
