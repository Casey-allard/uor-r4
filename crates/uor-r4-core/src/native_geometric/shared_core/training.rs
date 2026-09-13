//! Host-only objective and bounded root-coordinate proposals. Every proposal
//! recomputes the discrete recurrent forward path; no cached teacher states,
//! target-dependent runtime decisions or soft-forward quality are substituted.
use super::*;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitConfig {
    pub seed: u64,
    pub max_proposals: usize,
    pub max_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub positions: usize,
    pub correct: usize,
    pub mean_nll: f64,
    pub work: Work,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitReport {
    pub config: FitConfig,
    pub before: Metrics,
    pub after: Metrics,
    pub proposals: usize,
    pub accepted: usize,
    pub parameter_families: [String; 9],
    /// Counts use the schema-specific names above.
    pub proposals_by_family: [usize; 9],
    pub accepted_by_family: [usize; 9],
    pub elapsed_ms: u128,
    pub time_limit_reached: bool,
    pub training_digest: String,
}

pub(super) fn angular(geometry: &Geometry) -> Result<Vec<i16>> {
    let mut values: Vec<_> = geometry
        .anchors
        .rows
        .iter()
        .map(|row| row.root_scaled_zphi[0])
        .collect();
    values.sort_by(|a, b| super::super::training::exact_sign([a[0] - b[0], a[1] - b[1]]).cmp(&0));
    values.dedup();
    let zero = values
        .iter()
        .position(|&x| x == [0, 0])
        .ok_or(CoreError::InvalidArtifact("missing exact angular zero"))? as i16;
    geometry
        .anchors
        .rows
        .iter()
        .map(|row| {
            let rank = values
                .iter()
                .position(|&x| x == row.root_scaled_zphi[0])
                .ok_or(CoreError::InvalidArtifact("missing exact angular rank"))?
                as i16;
            Ok(rank - zero)
        })
        .collect()
}

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

pub(super) fn validate_data(documents: &[Vec<u8>]) -> Result<()> {
    if documents.is_empty()
        || documents.len() > 64
        || documents.iter().any(|d| d.is_empty() || d.len() > 512)
        || documents.iter().map(|d| d.len() + 1).sum::<usize>() > 8192
    {
        return Err(CoreError::InvalidInput(
            "1..64 nonempty documents; 512 bytes each; 8192 positions",
        ));
    }
    Ok(())
}

impl SharedCore {
    /// Compile canonical finite geometry and initialize learned roots offline.
    /// Byte identity is exact and complete; prime magnitude never ranks meaning.
    pub fn initialized(seed: u64) -> Result<Self> {
        if seed == 0 {
            return Err(CoreError::InvalidInput("nonzero seed required"));
        }
        let geometry = super::super::training::geometry(258, CONTEXT)
            .map_err(|e| CoreError::Host(e.to_string()))?;
        let ranks = angular(&geometry)?;
        let mut rng = seed;
        let mut parameters = vec![geometry.identity; PARAMETERS];
        for p in &mut parameters[EMBED..TRANSITION] {
            *p = (next(&mut rng) % 120) as u16;
        }
        for root in 0..ROOTS {
            parameters[QUERY + root] = root as u16;
            parameters[KEY + root] = root as u16;
        }
        for p in &mut parameters[OUTPUT..OUTPUT_MIX] {
            *p = (next(&mut rng) % 120) as u16;
        }
        let mut model = Self {
            artifact: Artifact {
                schema: SCHEMA.into(),
                implementation: Self::implementation_digest(),
                geometry,
                angular: ranks,
                parameters,
                seed,
                training_digest: None,
                fit_config: None,
                tied_fit_config: None,
                training_parent: None,
                calibrated: None,
                angular_tree: None,
            },
            cid: String::new(),
        };
        model.refresh_identity()?;
        Ok(model)
    }

    /// Teacher forcing observes only earlier bytes. The target is used solely
    /// by this host loss along the same branch-score function used by predict.
    pub fn evaluate(&self, documents: &[Vec<u8>], control: Intervention) -> Result<Metrics> {
        validate_data(documents)?;
        let (mut positions, mut correct, mut nll) = (0, 0, 0.0);
        let mut work = Work::default();
        for document in documents {
            let mut session = self.session(control);
            for target in document
                .iter()
                .map(|&b| u16::from(b))
                .chain(std::iter::once(EOS))
            {
                correct += usize::from(session.predict() == target);
                nll += session.target_nll(target);
                positions += 1;
                if target != EOS {
                    session.observe(target as u8)?;
                }
            }
            let w = session.work;
            work.products += w.products;
            work.table_reads += w.table_reads;
            work.candidates += w.candidates;
            work.output_decisions += w.output_decisions;
        }
        Ok(Metrics {
            positions,
            correct,
            mean_nll: nll / positions as f64,
            work,
        })
    }

    /// One joint objective over all parameter families. Coordinate proposals
    /// are a bounded feasibility experiment, not an efficient general trainer.
    /// The caller's parent is never mutated; every accepted move is evaluated
    /// against all supplied construction positions under the hard forward path.
    pub fn fit(&self, documents: &[Vec<u8>], config: FitConfig) -> Result<(Self, FitReport)> {
        if self.artifact.angular_tree.is_some() {
            return Err(CoreError::InvalidInput(
                "angular emission requires its own training provenance",
            ));
        }
        validate_data(documents)?;
        if config.seed == 0
            || config.max_seconds == 0
            || config.max_seconds > 120
            || config.max_proposals == 0
            || config.max_proposals > 100000
        {
            return Err(CoreError::InvalidInput("fit bounds"));
        }
        let start = Instant::now();
        let before = self.evaluate(documents, Intervention::Full)?;
        let mut model = self.clone();
        model.artifact.implementation = Self::implementation_digest();
        let mut best = before.clone();
        let mut rng = config.seed;
        let mut proposed = [0; 9];
        let mut accepted = [0; 9];
        let mut proposals = 0;
        let offsets = [
            EMBED, TRANSITION, QUERY, KEY, READ, PHASE, OUTPUT, OUTPUT_MIX, NULL, PARAMETERS,
        ];
        while proposals < config.max_proposals && start.elapsed().as_secs() < config.max_seconds {
            let family = proposals % 9;
            let index =
                offsets[family] + next(&mut rng) as usize % (offsets[family + 1] - offsets[family]);
            let old = model.artifact.parameters[index];
            let replacement = (next(&mut rng) % 120) as u16;
            let mut old_branch = None;
            if let Some(c) = model
                .artifact
                .calibrated
                .as_mut()
                .filter(|_| family == 6 || family == 7)
            {
                let node = next(&mut rng) as usize % 512;
                old_branch = Some((node, c.branches[node]));
                if family == 6 {
                    c.branches[node].landmarks[next(&mut rng) as usize & 1] = replacement;
                } else {
                    match next(&mut rng) % 3 {
                        0 => c.branches[node].thresholds[0] = (next(&mut rng) % 11) as i16 - 5,
                        1 => c.branches[node].thresholds[1] = (next(&mut rng) % 11) as i16 - 5,
                        _ => c.branches[node].union = !c.branches[node].union,
                    }
                }
            } else {
                model.artifact.parameters[index] = replacement;
            }
            let scored = model.evaluate(documents, Intervention::Full)?;
            proposals += 1;
            proposed[family] += 1;
            if scored.mean_nll < best.mean_nll {
                best = scored;
                accepted[family] += 1;
            } else {
                if let Some((node, branch)) = old_branch {
                    if let Some(c) = &mut model.artifact.calibrated {
                        c.branches[node] = branch;
                    }
                } else {
                    model.artifact.parameters[index] = old;
                }
            }
        }
        let mut hash = blake3::Hasher::new();
        hash.update(b"uor-r4.shared-core-training/1");
        for document in documents {
            hash.update(&(document.len() as u64).to_le_bytes());
            hash.update(document);
        }
        let digest = format!("blake3:{}", hash.finalize());
        model.artifact.training_digest = Some(digest.clone());
        model.artifact.fit_config = Some(config);
        model.artifact.tied_fit_config = None;
        model.artifact.training_parent = Some(self.cid.clone());
        model.refresh_identity()?;
        let report = FitReport {
            config,
            before,
            after: best,
            proposals,
            accepted: accepted.iter().sum(),
            parameter_families: if model.artifact.calibrated.is_some() {
                [
                    "embedding",
                    "transition",
                    "query",
                    "key",
                    "read",
                    "phase",
                    "cap_landmarks",
                    "cap_thresholds_union",
                    "null",
                ]
            } else {
                [
                    "embedding",
                    "transition",
                    "query",
                    "key",
                    "read",
                    "phase",
                    "emission",
                    "output_mix",
                    "null",
                ]
            }
            .map(str::to_owned),
            proposals_by_family: proposed,
            accepted_by_family: accepted,
            elapsed_ms: start.elapsed().as_millis(),
            time_limit_reached: start.elapsed().as_secs() >= config.max_seconds,
            training_digest: digest,
        };
        Ok((model, report))
    }

    pub fn generate(
        &self,
        prompt: &[u8],
        limit: usize,
        control: Intervention,
    ) -> Result<(Vec<u8>, bool, Work)> {
        if prompt.len() > 512 || limit > 96 || limit == 0 {
            return Err(CoreError::InvalidInput("generation bounds"));
        }
        let mut session = self.session(control);
        for &byte in prompt {
            session.observe(byte)?;
        }
        let mut output = Vec::with_capacity(limit);
        for _ in 0..limit {
            let token = session.predict();
            if token == EOS {
                return Ok((output, true, session.work()));
            }
            output.push(token as u8);
            session.observe(token as u8)?;
        }
        Ok((output, false, session.work()))
    }
}

impl CoreSession<'_> {
    fn target_nll(&mut self, target: u16) -> f64 {
        let (mut lo, mut hi, mut node, mut depth) = (0_u16, EOS + 1, 0, 0);
        let mut loss = 0.0;
        while hi - lo > 1 {
            let mid = (lo + hi) >> 1;
            let right = target >= mid;
            let score = f64::from(self.branch_score(node, depth)) / 4.0;
            let signed = if right { score } else { -score };
            loss += libm::log1p(libm::exp(-signed));
            if right {
                lo = mid;
            } else {
                hi = mid;
            }
            node = (node << 1) + 1 + usize::from(right);
            depth += 1;
        }
        loss
    }
}
