//! Offline angular partition learning. Serving traverses a bounded static
//! decoder tree over signed H4 comparisons, with integer score leaves.
use super::*;
use std::{collections::HashMap, time::Instant};
const EPS: f64 = 1e-10;
const MAX_NODES: usize = 15;
type Row = ([u16; LANES], bool);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AngularTreeConfig {
    pub max_depth: u8,
    pub min_leaf: usize,
    pub max_seconds: u64,
}
impl AngularTreeConfig {
    pub(super) fn validate(self) -> Result<()> {
        if !(1..=3).contains(&self.max_depth)
            || !(4..=8192).contains(&self.min_leaf)
            || !(1..=60).contains(&self.max_seconds)
        {
            return Err(CoreError::InvalidInput(
                "angular tree depth, leaf or time bound",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum AngularNode {
    Leaf {
        score: i16,
    },
    Split {
        lane: u8,
        landmark: u16,
        threshold: i16,
        left: u8,
        right: u8,
    },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AngularEmission {
    pub(super) config: AngularTreeConfig,
    pub(super) parent: String,
    pub(super) data_digest: String,
    pub(super) branches: Vec<Option<Vec<AngularNode>>>,
}
fn cid_valid(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}
impl AngularEmission {
    pub(super) fn validate(&self) -> Result<()> {
        self.config.validate()?;
        if !cid_valid(&self.parent) || !cid_valid(&self.data_digest) || self.branches.len() != 512 {
            return Err(CoreError::InvalidArtifact(
                "angular emission provenance or branch count",
            ));
        }
        for tree in self.branches.iter().flatten() {
            validate_tree(tree, self.config.max_depth)?;
        }
        Ok(())
    }
}
fn validate_tree(nodes: &[AngularNode], max_depth: u8) -> Result<()> {
    if nodes.is_empty() || nodes.len() > MAX_NODES {
        return Err(CoreError::InvalidArtifact("angular tree node count"));
    }
    let mut depths = [None; MAX_NODES];
    depths[0] = Some(0_u8);
    for (i, node) in nodes.iter().enumerate() {
        let depth = depths[i].ok_or(CoreError::InvalidArtifact("unreachable angular node"))?;
        match *node {
            AngularNode::Leaf { score } => {
                if !(-10..=10).contains(&score) {
                    return Err(CoreError::InvalidArtifact("angular leaf score"));
                }
            }
            AngularNode::Split {
                lane,
                landmark,
                threshold,
                left,
                right,
            } => {
                if usize::from(lane) >= LANES
                    || usize::from(landmark) >= ROOTS
                    || !(-5..=5).contains(&threshold)
                    || depth >= max_depth
                    || left == right
                {
                    return Err(CoreError::InvalidArtifact("angular split fields or depth"));
                }
                for child in [left, right] {
                    let child = usize::from(child);
                    if child <= i || child >= nodes.len() || depths[child].is_some() {
                        return Err(CoreError::InvalidArtifact(
                            "angular graph must be a complete forward tree",
                        ));
                    }
                    depths[child] = Some(depth + 1);
                }
            }
        }
    }
    Ok(())
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AngularTreeReport {
    pub config: AngularTreeConfig,
    pub parent: String,
    pub data_digest: String,
    pub before: Metrics,
    pub after: Metrics,
    pub elapsed_ms: u128,
    pub occupied_branches: usize,
    pub selected_trees: usize,
    pub total_nodes: usize,
    pub split_candidates: usize,
}
fn deadline(start: Instant, seconds: u64) -> Result<()> {
    if start.elapsed().as_secs() >= seconds {
        return Err(CoreError::InvalidInput(
            "angular tree deadline; no partial artifact",
        ));
    }
    Ok(())
}
fn loss(score: i16, target: bool) -> f64 {
    let score = f64::from(score) / 4.0;
    libm::log1p(libm::exp(if target { -score } else { score }))
}
struct Builder<'a> {
    model: &'a SharedCore,
    config: AngularTreeConfig,
    start: Instant,
    leaf_cache: HashMap<[usize; 2], (i16, f64)>,
    split_candidates: usize,
}
impl Builder<'_> {
    fn leaf(&mut self, counts: [usize; 2]) -> (i16, f64) {
        *self.leaf_cache.entry(counts).or_insert_with(|| {
            let mut best = (0_i16, f64::INFINITY);
            for score in -10..=10 {
                let cost =
                    counts[0] as f64 * loss(score, false) + counts[1] as f64 * loss(score, true);
                if cost < best.1 {
                    best = (score, cost);
                }
            }
            best
        })
    }
    fn build(&mut self, rows: &[Row], depth: u8, nodes: &mut Vec<AngularNode>) -> Result<u8> {
        deadline(self.start, self.config.max_seconds)?;
        if nodes.len() >= MAX_NODES || rows.is_empty() {
            return Err(CoreError::InvalidInput("angular builder bounds"));
        }
        let positives = rows.iter().filter(|r| r.1).count();
        let counts = [rows.len() - positives, positives];
        let (score, mut best_cost) = self.leaf(counts);
        let index = nodes.len() as u8;
        nodes.push(AngularNode::Leaf { score });
        if depth == self.config.max_depth || rows.len() < self.config.min_leaf.saturating_mul(2) {
            return Ok(index);
        }
        let mut chosen = None;
        for lane in 0..LANES {
            for landmark in 0..ROOTS {
                deadline(self.start, self.config.max_seconds)?;
                let mut histogram = [[0usize; 2]; 9];
                for &(roots, target) in rows {
                    let angular = self.model.relative_score(
                        roots[lane],
                        landmark as u16,
                        &mut Work::default(),
                    );
                    if !(-4..=4).contains(&angular) {
                        return Err(CoreError::InvalidArtifact(
                            "angular rank outside split domain",
                        ));
                    }
                    histogram[(angular + 4) as usize][usize::from(target)] += 1;
                }
                for threshold in -5_i16..=5 {
                    self.split_candidates += 1;
                    let mut left = [0usize; 2];
                    for (rank, bins) in histogram.iter().enumerate() {
                        if rank as i16 - 4 <= threshold {
                            left[0] += bins[0];
                            left[1] += bins[1];
                        }
                    }
                    let right = [counts[0] - left[0], counts[1] - left[1]];
                    if left.iter().sum::<usize>() < self.config.min_leaf
                        || right.iter().sum::<usize>() < self.config.min_leaf
                    {
                        continue;
                    }
                    let cost = self.leaf(left).1 + self.leaf(right).1;
                    if cost + EPS < best_cost {
                        best_cost = cost;
                        chosen = Some((lane as u8, landmark as u16, threshold));
                    }
                }
            }
        }
        if let Some((lane, landmark, threshold)) = chosen {
            let (mut left_rows, mut right_rows) = (Vec::new(), Vec::new());
            for &row in rows {
                if self.model.relative_score(
                    row.0[usize::from(lane)],
                    landmark,
                    &mut Work::default(),
                ) > threshold
                {
                    right_rows.push(row);
                } else {
                    left_rows.push(row);
                }
            }
            let left = self.build(&left_rows, depth + 1, nodes)?;
            let right = self.build(&right_rows, depth + 1, nodes)?;
            nodes[usize::from(index)] = AngularNode::Split {
                lane,
                landmark,
                threshold,
                left,
                right,
            };
        }
        Ok(index)
    }
}
impl SharedCore {
    /// Fit a static decoder tree independently at each occupied byte branch.
    /// Construction labels choose leaves/splits; no development/control labels
    /// enter this method. Recurrence, routing, geometry and caps remain fixed.
    pub fn fit_angular_emission(
        &self,
        documents: &[Vec<u8>],
        config: AngularTreeConfig,
    ) -> Result<(Self, AngularTreeReport)> {
        config.validate()?;
        training::validate_data(documents)?;
        if self.artifact.calibrated.is_none() || self.artifact.angular_tree.is_some() {
            return Err(CoreError::InvalidInput(
                "angular fit requires a calibrated cap parent",
            ));
        }
        let start = Instant::now();
        deadline(start, config.max_seconds)?;
        let before = self.evaluate(documents, Intervention::Full)?;
        deadline(start, config.max_seconds)?;
        let mut rows: Vec<Vec<Row>> = vec![Vec::new(); 512];
        let mut depths = [0usize; 512];
        for doc in documents {
            let mut session = self.session(Intervention::Full);
            for target in doc
                .iter()
                .map(|&b| u16::from(b))
                .chain(std::iter::once(EOS))
            {
                deadline(start, config.max_seconds)?;
                let (mut lo, mut hi, mut node, mut depth) = (0_u16, 257_u16, 0, 0);
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
        let mut builder = Builder {
            model: self,
            config,
            start,
            leaf_cache: HashMap::new(),
            split_candidates: 0,
        };
        let mut branches = vec![None; 512];
        let (mut occupied_branches, mut selected_trees, mut total_nodes) = (0, 0, 0);
        let caps = self
            .artifact
            .calibrated
            .as_ref()
            .ok_or(CoreError::InvalidArtifact("cap parent"))?;
        for (node, data) in rows.iter().enumerate() {
            deadline(start, config.max_seconds)?;
            if data.is_empty() {
                continue;
            }
            occupied_branches += 1;
            if data.len() < config.min_leaf {
                continue;
            }
            let mut tree = Vec::with_capacity(MAX_NODES);
            builder.build(data, 0, &mut tree)?;
            validate_tree(&tree, config.max_depth)?;
            let mut cap_loss = 0.0;
            let mut tree_loss = 0.0;
            for &(roots, target) in data {
                cap_loss += loss(
                    self.cap_score(
                        roots,
                        caps.branches[node],
                        depths[node],
                        &mut Work::default(),
                    ),
                    target,
                );
                tree_loss += loss(
                    self.angular_tree_score(roots, &tree, &mut Work::default()),
                    target,
                );
            }
            if tree_loss + EPS < cap_loss {
                selected_trees += 1;
                total_nodes += tree.len();
                branches[node] = Some(tree);
            }
        }
        deadline(start, config.max_seconds)?;
        let mut hash = blake3::Hasher::new();
        hash.update(b"uor-r4.angular-emission-training/1");
        for doc in documents {
            hash.update(&(doc.len() as u64).to_le_bytes());
            hash.update(doc);
        }
        let data_digest = format!("blake3:{}", hash.finalize());
        let mut model = self.clone();
        model.artifact.schema = ANGULAR_TREE_SCHEMA.into();
        model.artifact.implementation = Self::implementation_digest();
        model.artifact.training_digest = None;
        model.artifact.training_parent = None;
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = None;
        model.artifact.angular_tree = Some(AngularEmission {
            config,
            parent: self.cid.clone(),
            data_digest: data_digest.clone(),
            branches,
        });
        model
            .artifact
            .angular_tree
            .as_ref()
            .ok_or(CoreError::InvalidArtifact("angular emitter"))?
            .validate()?;
        model.refresh_identity()?;
        if model
            .to_bytes()?
            .len()
            .saturating_sub(self.to_bytes()?.len())
            > 1024 * 1024
        {
            return Err(CoreError::InvalidInput(
                "angular emission serialized growth bound",
            ));
        }
        deadline(start, config.max_seconds)?;
        let after = model.evaluate(documents, Intervention::Full)?;
        deadline(start, config.max_seconds)?;
        if after.mean_nll > before.mean_nll + EPS {
            return Err(CoreError::InvalidInput(
                "angular emission replay increased NLL",
            ));
        }
        let report = AngularTreeReport {
            config,
            parent: self.cid.clone(),
            data_digest,
            before,
            after,
            elapsed_ms: start.elapsed().as_millis(),
            occupied_branches,
            selected_trees,
            total_nodes,
            split_candidates: builder.split_candidates,
        };
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> AngularTreeConfig {
        AngularTreeConfig {
            max_depth: 3,
            min_leaf: 4,
            max_seconds: 60,
        }
    }
    fn fixture() -> SharedCore {
        let mut model = SharedCore::initialized(745)
            .unwrap()
            .upgrade_calibrated()
            .unwrap();
        let mut branches = vec![None; 512];
        branches[0] = Some(vec![
            AngularNode::Split {
                lane: 0,
                landmark: model.artifact.geometry.identity,
                threshold: 0,
                left: 1,
                right: 2,
            },
            AngularNode::Leaf { score: -10 },
            AngularNode::Leaf { score: 10 },
        ]);
        model.artifact.angular_tree = Some(AngularEmission {
            config: config(),
            parent: model.cid.clone(),
            data_digest: format!("blake3:{}", "a".repeat(64)),
            branches,
        });
        model.artifact.schema = ANGULAR_TREE_SCHEMA.into();
        model.refresh_identity().unwrap();
        model
    }
    #[test]
    fn angular_split_uses_signed_geometry_and_matches_exhaustive_leaf_loss() {
        let model = SharedCore::initialized(745)
            .unwrap()
            .upgrade_calibrated()
            .unwrap();
        let id = model.artifact.geometry.identity;
        let positive = (0..120)
            .find(|&r| model.relative_score(r, id, &mut Work::default()) == 4)
            .unwrap();
        let negative = (0..120)
            .find(|&r| model.relative_score(r, id, &mut Work::default()) == -4)
            .unwrap();
        assert_ne!(positive, negative);
        let mut builder = Builder {
            model: &model,
            config: config(),
            start: Instant::now(),
            leaf_cache: HashMap::new(),
            split_candidates: 0,
        };
        let mut rows = vec![([positive; LANES], true); 4];
        rows.extend(vec![([negative; LANES], false); 4]);
        let mut tree = Vec::new();
        builder.build(&rows, 0, &mut tree).unwrap();
        validate_tree(&tree, 3).unwrap();
        assert_eq!(tree.len(), 3);
        assert_eq!(
            model.angular_tree_score([positive; LANES], &tree, &mut Work::default()),
            10
        );
        assert_eq!(
            model.angular_tree_score([negative; LANES], &tree, &mut Work::default()),
            -10
        );
        let (score, cost) = builder.leaf([4, 8]);
        let independently_summed: Vec<_> = (-10..=10)
            .map(|s| {
                (
                    s,
                    rows.iter().take(4).map(|_| loss(s, false)).sum::<f64>()
                        + rows.iter().map(|_| loss(s, true)).sum::<f64>(),
                )
            })
            .collect();
        assert!(independently_summed
            .iter()
            .all(|(_, other)| cost <= *other + EPS));
        assert!((cost - (4.0 * loss(score, false) + 8.0 * loss(score, true))).abs() < EPS);
        let mut work = Work::default();
        model.angular_tree_score([positive; LANES], &tree, &mut work);
        assert_eq!(work.products, 1);
    }
    #[test]
    fn angular_artifact_rejects_malformed_trees_and_preserves_runtime_checkpoint() {
        let model = fixture();
        let bytes = model.to_bytes().unwrap();
        let loaded = SharedCore::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.to_bytes().unwrap(), bytes);
        assert_eq!(loaded.artifact_cid(), model.artifact_cid());
        let mut a = model.session(Intervention::Full);
        for b in b"abca" {
            a.observe(*b).unwrap();
        }
        let snapshot = a.checkpoint().unwrap();
        let mut b = loaded.restore(&snapshot).unwrap();
        for byte in b"defgh" {
            assert_eq!(a.predict(), b.predict());
            a.observe(*byte).unwrap();
            b.observe(*byte).unwrap();
            assert_eq!(a.state_roots(), b.state_roots());
        }
        assert_eq!(
            model.generate(b"ab", 24, Intervention::Full).unwrap(),
            loaded.generate(b"ab", 24, Intervention::Full).unwrap()
        );
        let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for (field, value) in [
            ("left", serde_json::json!(0)),
            ("right", serde_json::json!(1)),
            ("lane", serde_json::json!(4)),
            ("landmark", serde_json::json!(120)),
            ("threshold", serde_json::json!(6)),
        ] {
            let mut bad = original.clone();
            bad["angular_tree"]["branches"][0][0][field] = value;
            assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
        let mut bad = original.clone();
        bad["angular_tree"]["branches"][0][1]["score"] = serde_json::json!(11);
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = original.clone();
        bad["angular_tree"]["config"]["max_depth"] = serde_json::json!(0);
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = original.clone();
        bad["schema"] = serde_json::json!(CALIBRATED_SCHEMA);
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = original.clone();
        bad["angular_tree"]["parent"] = serde_json::json!("blake3:bad");
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = original.clone();
        bad["angular_tree"]["branches"][0]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"kind":"leaf","score":0}));
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        let deep = vec![
            AngularNode::Split {
                lane: 0,
                landmark: 0,
                threshold: 0,
                left: 1,
                right: 2,
            },
            AngularNode::Leaf { score: 0 },
            AngularNode::Split {
                lane: 0,
                landmark: 0,
                threshold: 0,
                left: 3,
                right: 4,
            },
            AngularNode::Leaf { score: 0 },
            AngularNode::Leaf { score: 1 },
        ];
        assert!(validate_tree(&deep, 1).is_err());
        assert!(model
            .fit(
                &[b"a".to_vec()],
                FitConfig {
                    seed: 1,
                    max_proposals: 1,
                    max_seconds: 1
                }
            )
            .is_err());
        assert!(model
            .calibrate_emission_blocks(&[b"a".to_vec()], 1)
            .is_err());
        assert!(model.calibrate_emission(&[b"a".to_vec()]).is_err());
        assert!(model
            .fit_angular_emission(&[b"a".to_vec()], config())
            .is_err());
        assert!(model
            .fit_tied(
                &[b"a".to_vec()],
                TiedFitConfig {
                    indices: [1176, 1930],
                    seed: 1,
                    batches: 1,
                    batch_size: 2,
                    step_size: 1.0,
                    max_seconds: 1
                }
            )
            .is_err());
        assert!(model.upgrade_calibrated().is_err());
        // Historical schema-2 and tied artifacts retain omitted-field bytes and
        // their exact implementation pin, even though this source adds schema 3.
        let mut old = SharedCore::initialized(745)
            .unwrap()
            .upgrade_calibrated()
            .unwrap();
        old.artifact.implementation = TIED_V1_IMPLEMENTATION.into();
        for tied in [false, true] {
            if tied {
                old.artifact.tied_fit_config = Some(TiedFitConfig {
                    indices: [1176, 1930],
                    seed: 1,
                    batches: 1,
                    batch_size: 2,
                    step_size: 1.0,
                    max_seconds: 1,
                });
                old.artifact.training_digest = Some(format!("blake3:{}", "b".repeat(64)));
                old.artifact.training_parent = Some(model.cid.clone());
            }
            old.refresh_identity().unwrap();
            let wire = old.to_bytes().unwrap();
            assert!(!String::from_utf8_lossy(&wire).contains("angular_tree"));
            let restored = SharedCore::from_bytes(&wire).unwrap();
            assert_eq!(restored.to_bytes().unwrap(), wire);
            assert_eq!(restored.cid, old.cid);
            assert_eq!(
                restored.generate(b"a", 8, Intervention::Full).unwrap(),
                old.generate(b"a", 8, Intervention::Full).unwrap()
            );
        }
    }
}
