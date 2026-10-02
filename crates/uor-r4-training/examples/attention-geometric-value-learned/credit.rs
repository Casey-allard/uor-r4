//! Offline, frozen-teacher credit allocation. Nothing here enters prediction.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AuxiliaryCredit {
    Legacy,
    QueryRead,
    UniformMatchedMass,
}

pub(super) fn loss(
    logits: &Tensor,
    targets: &[u32],
    weights: &[f32],
    normalizer: Option<f64>,
) -> Result<Tensor> {
    let mass = weights.iter().map(|&x| f64::from(x)).sum::<f64>();
    if weights.iter().any(|x| !x.is_finite() || *x < 0.)
        || !mass.is_finite()
        || normalizer.is_some_and(|n| !n.is_finite() || n <= 0.)
    {
        return Err(invalid("invalid auxiliary mass/normalizer"));
    }
    if mass == 0. {
        // Candle simplifies affine(0, 0) to a detached constant. Subtraction
        // keeps the actual logits in the graph with an exactly zero adjoint.
        return Ok((logits - logits)?.sum_all()?);
    }
    let mean = logits_cross_entropy(logits, targets, Some(weights))?;
    Ok(match normalizer {
        Some(n) => mean.affine(mass / n, 0.)?,
        None => mean,
    })
}

pub(super) fn apply(
    labels: &Targets,
    trace: &NativeReadTrace,
    lengths: &[usize],
    queries: &[usize],
    policy: AuxiliaryCredit,
) -> Result<(Targets, Value, Vec<u8>)> {
    if policy == AuxiliaryCredit::Legacy {
        return Ok((
            labels.clone(),
            json!({"policy":policy,"objective":"legacy-global-position-mean/1"}),
            vec![],
        ));
    }
    let (batch, time) = (trace.batch, trace.time);
    let n = batch
        .checked_mul(time)
        .and_then(|n| n.checked_mul(16))
        .ok_or_else(|| invalid("credit shape overflow"))?;
    if batch == 0
        || trace.heads != 2
        || trace.value_width != 16
        || time == 0
        || time > 128
        || lengths.len() != batch
        || queries.len() != batch
        || trace.rows.len() != batch * 2 * time
        || labels.roots.len() != n
        || labels.categories.len() != n
        || labels.root_weights.len() != n
        || labels.category_weights.len() != n
        || labels.normalizer.is_some()
        || labels
            .root_weights
            .iter()
            .chain(&labels.category_weights)
            .any(|x| *x != 0. && *x != 1.)
    {
        return Err(invalid("credit shape or Boolean teacher mask differs"));
    }
    let mut out = labels.clone();
    out.root_weights.fill(0.);
    out.category_weights.fill(0.);
    out.normalizer = Some((batch * 16) as f64);
    let mut groups = Vec::new();
    let mut heads = Vec::new();
    let mut raw = Vec::new();
    let mut exact_mass_by_head = [[0f64; 2]; 2];
    for b in 0..batch {
        let query = queries[b];
        if lengths[b] == 0 || lengths[b] > time || query + 1 != lengths[b] {
            return Err(invalid("credit query must be the final actual position"));
        }
        for h in 0..2 {
            // The observer validates the row's integer sum and causal support.
            let (_, denominator) = trace.source_fraction(b, h, query, &[])?;
            let row = &trace.rows[(b * 2 + h) * time + query];
            for x in [
                b as u64,
                h as u64,
                query as u64,
                denominator,
                row.no_read_weight_q31,
            ] {
                raw.extend(x.to_le_bytes());
            }
            raw.extend(row.max_score_q24.to_le_bytes());
            for &w in &row.occurrence_weights_q31 {
                raw.extend(w.to_le_bytes());
            }
            heads.push(json!({"batch":b,"head":h,"query":query,"no_read_numerator":row.no_read_weight_q31,"denominator":denominator,"read_mass":1.-row.no_read_weight_q31 as f64/denominator as f64}));
            for lane in 0..4 {
                for atom in 0..2 {
                    let mut family = Vec::new();
                    for (kind, mask) in [&labels.root_weights, &labels.category_weights]
                        .into_iter()
                        .enumerate()
                    {
                        let mut eligible = Vec::new();
                        let mut numerator = 0u64;
                        for t in 0..=query {
                            let at = (((b * time + t) * 2 + h) * 4 + lane) * 2 + atom;
                            if mask[at] != 0. {
                                eligible.push((at, row.occurrence_weights_q31[t]));
                                numerator = numerator
                                    .checked_add(row.occurrence_weights_q31[t])
                                    .ok_or_else(|| invalid("eligible credit numerator overflow"))?;
                            }
                        }
                        let count = eligible.len();
                        let mass = numerator as f64 / denominator as f64;
                        let uniform = if count == 0 { 0. } else { mass / count as f64 };
                        let (mut query_sum, mut uniform_sum) = (0f64, 0f64);
                        for &(at, w) in &eligible {
                            let q = (w as f64 / denominator as f64) as f32;
                            let u = uniform as f32;
                            query_sum += f64::from(q);
                            uniform_sum += f64::from(u);
                            let selected = if policy == AuxiliaryCredit::QueryRead {
                                q
                            } else {
                                u
                            };
                            if kind == 0 {
                                out.root_weights[at] = selected;
                            } else {
                                out.category_weights[at] = selected;
                            }
                        }
                        let tolerance = 2. * f64::from(f32::EPSILON) * mass;
                        if (query_sum - uniform_sum).abs() > tolerance
                            || (query_sum - mass).abs() > tolerance
                            || (uniform_sum - mass).abs() > tolerance
                        {
                            return Err(invalid(
                                "F32 allocation changed eligible mass beyond rounding bound",
                            ));
                        }
                        exact_mass_by_head[h][kind] += mass;
                        family.push(json!({"numerator":numerator,"eligible":count,"exact_mass":mass,"query_f32_mass":query_sum,"uniform_f32_mass":uniform_sum}));
                    }
                    groups.push(json!({"batch":b,"head":h,"lane":lane,"atom":atom,"root":family[0],"category":family[1]}));
                }
            }
        }
    }
    let metadata = json!({"policy":policy,"objective":"matched-teacher-eligible-mass-fixed-example-head-primitive-denominator/1","denominator":batch*16,"heads":heads,"groups":groups,"root_category_eligible_mass_by_head":exact_mass_by_head,"raw_query_format":"LEu64 b,h,query,total,no_read;LEi64 max_score;LEu64 occurrence[query+1], in B,H order","raw_query_sha256":sha256_bytes(&raw),"target_format":"B,T,H,lane,atom: root:u8,category:u8,root_weight:LEf32,category_weight:LEf32","weight_conversion":"exact-u64-fraction-to-F64-to-F32;uniform exact eligible mass divided by teacher eligible count;no current prediction masks"});
    Ok((out, metadata, raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_training::geometric_read_native::NativeReadRow;

    #[test]
    fn query_credit_matches_group_mass_and_preserves_null_zero_padding() -> Result<()> {
        let (b, t) = (2, 3);
        let n = b * t * 16;
        let mut labels = Targets {
            roots: vec![1; n],
            categories: vec![1; n],
            root_weights: vec![1.; n],
            category_weights: vec![1.; n],
            normalizer: None,
        };
        // B1's final two positions are padding. B0,H0,atom0 at t1 is absent;
        // atom1 at t0 is PRESENT_ZERO (category supervised, root excluded).
        for at in 4 * 16..n {
            labels.root_weights[at] = 0.;
            labels.category_weights[at] = 0.;
        }
        labels.root_weights[16] = 0.;
        labels.category_weights[16] = 0.;
        labels.root_weights[1] = 0.;
        labels.categories[1] = 0;
        let mut rows = Vec::new();
        for batch in 0..b {
            for head in 0..2 {
                for q in 0..t {
                    let weights = if batch == 0 && head == 0 && q == 2 {
                        vec![2, 3, 5]
                    } else if batch == 1 {
                        vec![0; q + 1]
                    } else {
                        vec![1; q + 1]
                    };
                    let null = if batch == 0 && head == 0 { 10 } else { 100 };
                    rows.push(NativeReadRow {
                        output_q16: vec![0; 16],
                        total_weight_q31: weights.iter().sum::<u64>() + null,
                        occurrence_weights_q31: weights,
                        no_read_weight_q31: null,
                        max_score_q24: 0,
                    });
                }
            }
        }
        let trace = NativeReadTrace {
            batch: b,
            heads: 2,
            time: t,
            value_width: 16,
            no_read_q24: vec![],
            values_q16: vec![],
            rows,
        };
        let (q, qm, raw) = apply(
            &labels,
            &trace,
            &[3, 1],
            &[2, 0],
            AuxiliaryCredit::QueryRead,
        )?;
        let (u, um, raw_u) = apply(
            &labels,
            &trace,
            &[3, 1],
            &[2, 0],
            AuxiliaryCredit::UniformMatchedMass,
        )?;
        assert_eq!(raw, raw_u);
        assert_eq!(
            qm["root_category_eligible_mass_by_head"],
            um["root_category_eligible_mass_by_head"]
        );
        assert_eq!(q.root_weights[0], 0.1);
        assert_eq!(q.root_weights[16], 0.);
        assert_eq!(q.root_weights[32], 0.25);
        assert_eq!(u.root_weights[0], 0.175);
        assert_eq!(u.root_weights[32], 0.175);
        assert_eq!(q.root_weights[1], 0.);
        assert_eq!(q.category_weights[1], 0.1);
        assert!(q.root_weights[3 * 16..].iter().all(|x| *x == 0.));
        for family in 0..2 {
            let (qw, uw, classes) = if family == 0 {
                (&q.root_weights, &u.root_weights, 120)
            } else {
                (&q.category_weights, &u.category_weights, 32)
            };
            let logits = Var::zeros((n, classes), candle_core::DType::F32, &Device::Cpu)?;
            let lq = loss(logits.as_tensor(), &vec![1; n], qw, q.normalizer)?;
            let lu = loss(logits.as_tensor(), &vec![1; n], uw, u.normalizer)?;
            assert!((lq.to_scalar::<f32>()? - lu.to_scalar::<f32>()?).abs() < 1e-6);
            assert!(
                lq.backward()?
                    .get(logits.as_tensor())
                    .ok_or_else(|| invalid("credit gradient absent"))?
                    .abs()?
                    .sum_all()?
                    .to_scalar::<f32>()?
                    > 0.
            );
        }
        assert_eq!(q.bytes().len(), n * 10);
        assert_eq!(
            f32::from_le_bytes(
                q.bytes()[2..6]
                    .try_into()
                    .map_err(|_| invalid("weight bytes"))?
            ),
            0.1
        );
        let logits = Var::zeros((n, 120), candle_core::DType::F32, &Device::Cpu)?;
        let zero = loss(logits.as_tensor(), &vec![1; n], &vec![0.; n], Some(32.))?;
        assert_eq!(zero.to_scalar::<f32>()?, 0.);
        assert_eq!(
            zero.backward()?
                .get(logits.as_tensor())
                .ok_or_else(|| invalid("connected zero absent"))?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?,
            0.
        );
        assert!(apply(
            &labels,
            &trace,
            &[3, 1],
            &[1, 0],
            AuxiliaryCredit::QueryRead
        )
        .is_err());
        Ok(())
    }
}
