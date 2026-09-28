
/// Per-head mean of `|k|^2` over batch and time, for keys `(batch, heads, time,
/// head_dim)`. Multiplying a head's curvature by it gives the dimensionless
/// curvature `t`, which is invariant to rescaling that head's keys.
pub fn mean_key_sq(key: &Tensor) -> Result<Vec<f32>> {
    Ok(key.sqr()?.sum(3)?.mean((0, 2))?.to_vec1()?)
}

/// Attention statistics per layer and head (`[layer][head]`).
#[derive(Clone, Debug, Serialize)]
pub struct HeadStatistics {
    /// Median `|k|^2`: the unit of the dimensionless curvature `t`.
    pub median_key_sq: Vec<Vec<f32>>,
    /// Mean cosine between a query and its highest-weight key.
    pub top1_cosine: Vec<Vec<f32>>,
    /// Mean attention mass held by the top `fractions[i]` of the causal keys,
    /// the ceiling for any index that scores that fraction exactly.
    pub top_mass: Vec<Vec<Vec<f32>>>,
    pub fractions: Vec<f64>,
    /// Queries with fewer causal keys than this are skipped for cosine and mass.
    pub min_candidates: usize,
    /// Query rows averaged per head.
    pub rows: u64,
}

/// Accumulates [`HeadStatistics`] over batches of one model's attention.
pub struct HeadStatisticsAccumulator {
    fractions: Vec<f64>,
    min_candidates: usize,
    key_sq: Vec<Vec<Vec<f32>>>,
    cosine: Vec<Vec<f64>>,
    mass: Vec<Vec<Vec<f64>>>,
    rows: Vec<Vec<u64>>,
}

fn dot64(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

impl HeadStatisticsAccumulator {
    pub fn new(shape: &LlamaShape, fractions: &[f64], min_candidates: usize) -> Result<Self> {
        if fractions.is_empty()
            || fractions.iter().any(|f| !(*f > 0.0 && *f <= 1.0))
            || min_candidates == 0
        {
            return Err(invalid(
                "fractions must lie in (0, 1] and min_candidates must be positive",
            ));
        }
        Ok(Self {
            fractions: fractions.to_vec(),
            min_candidates,
            key_sq: vec![vec![Vec::new(); shape.heads]; shape.layers],
            cosine: vec![vec![0.0; shape.heads]; shape.layers],
            mass: vec![vec![vec![0.0; fractions.len()]; shape.heads]; shape.layers],
            rows: vec![vec![0; shape.heads]; shape.layers],
        })
    }

    /// Run `model` on one batch without a backward graph and accumulate every head.
    pub fn add(&mut self, model: &KappaLlama, ids: &[u32], batch: usize, time: usize) -> Result<()> {
        let cpu = Device::Cpu;
        let mut probe =
            |layer: usize, query: &Tensor, key: &Tensor, probability: &Tensor| -> Result<()> {
                let (b_len, h_len, t_len, width) = key.dims4()?;
                let q: Vec<f32> = query.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let k: Vec<f32> = key.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let p: Vec<f32> = probability.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let mut row = Vec::with_capacity(t_len);
                for b in 0..b_len {
                    for h in 0..h_len {
                        let base = (b * h_len + h) * t_len;
                        for t in 0..t_len {
                            let kt = &k[(base + t) * width..(base + t + 1) * width];
                            self.key_sq[layer][h].push(dot64(kt, kt) as f32);
                        }
                        for t in self.min_candidates.saturating_sub(1)..t_len {
                            let start = (base + t) * t_len;
                            let weights = &p[start..start + t + 1];
                            let mut top = 0;
                            for (i, w) in weights.iter().enumerate() {
                                if *w > weights[top] {
                                    top = i;
                                }
                            }
                            let qt = &q[(base + t) * width..(base + t + 1) * width];
                            let kt = &k[(base + top) * width..(base + top + 1) * width];
                            let norms = (dot64(qt, qt) * dot64(kt, kt)).sqrt();
                            if norms > 0.0 {
                                self.cosine[layer][h] += dot64(qt, kt) / norms;
                            }
                            row.clear();
                            row.extend_from_slice(weights);
                            row.sort_by(|a, b| b.total_cmp(a));
                            for (i, fraction) in self.fractions.iter().enumerate() {
                                let keep = ((fraction * (t + 1) as f64).ceil() as usize).clamp(1, t + 1);
                                self.mass[layer][h][i] +=
                                    row[..keep].iter().map(|w| f64::from(*w)).sum::<f64>();
                            }
                            self.rows[layer][h] += 1;
                        }
                    }
                }
                Ok(())
            };
        model.forward_with_probe(ids, batch, time, true, &mut probe)?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<HeadStatistics> {
        let rows = self.rows.iter().flatten().copied().min().unwrap_or(0);
        if rows == 0 || self.key_sq.iter().flatten().any(Vec::is_empty) {
            return Err(invalid("no query had enough causal keys for head statistics"));
        }
        let mut median_key_sq = Vec::with_capacity(self.key_sq.len());
        for layer in &mut self.key_sq {
            let mut medians = Vec::with_capacity(layer.len());
            for values in layer.iter_mut() {
                values.sort_by(f32::total_cmp);
                medians.push(values[values.len() / 2]);
            }
            median_key_sq.push(medians);
        }
        let mut top1_cosine = Vec::with_capacity(self.cosine.len());
        let mut top_mass = Vec::with_capacity(self.mass.len());
        for ((cosines, masses), counts) in self.cosine.iter().zip(&self.mass).zip(&self.rows) {
            top1_cosine.push(
                cosines
                    .iter()
                    .zip(counts)
                    .map(|(sum, n)| (sum / *n as f64) as f32)
                    .collect(),
            );
            top_mass.push(
                masses
                    .iter()
                    .zip(counts)
                    .map(|(sums, n)| sums.iter().map(|sum| (sum / *n as f64) as f32).collect())
                    .collect(),
            );
        }
        Ok(HeadStatistics {
            median_key_sq,
            top1_cosine,
            top_mass,
            fractions: self.fractions,
            min_candidates: self.min_candidates,
            rows,
        })
    }
}
