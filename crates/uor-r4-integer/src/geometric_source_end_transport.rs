//! Factual-reader-bound Source-end compatibility. Retained latent states are
//! distinct from observed presence. Terminal coefficients never select a Source.
use crate::{
    geometric_context::NativeContextState,
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_cue_carrier::CueCarrierMetadata,
    geometric_potential_q4::unpack_coefficients,
    geometric_prefix_transport::{PrefixState, PrefixTransportMetadata, PrefixTransportTrace},
    geometric_source_realizer::{
        HeadTrace, NativeArtifactBinding, SourceRuntimeError, SourceRuntimeResult as Result,
    },
    h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT, TRUSTED_MATHEMATICAL_SHA256},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
fn error(s: impl Into<String>) -> SourceRuntimeError {
    SourceRuntimeError::Invalid(s.into())
}
fn sha(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
pub const SCHEMA: &str = "uor-r4.geometric-source-end-transport/1";
pub const POLICY:&str="factual-summed-raw-Copy-earliest-bank-occurrence;source-emission-view-full-end;actual-response-prefix;retained-latent;inverse(response)*end;independent-Period-Stop-q4;no-target-length-cursor;one-global-final-alias-reduction/1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceEndScoreMode {
    DirectedRelative,
    SourceEndUnary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEndAngularConfig {
    pub heads: usize,
    pub lanes_per_head: usize,
    pub mode: SourceEndScoreMode,
}
impl SourceEndAngularConfig {
    pub fn coefficient_count(self) -> Result<usize> {
        if !(1..=2).contains(&self.heads) || !(1..=4).contains(&self.lanes_per_head) {
            return Err(error("source-end dimensions out of bounds"));
        }
        Ok(self.heads * self.lanes_per_head * ROOT_COUNT)
    }
}
pub struct SourceEndAngularQ4 {
    config: SourceEndAngularConfig,
    period: Vec<u8>,
    stop: Vec<u8>,
    period_tables: Vec<Vec<[i32; ROOT_COUNT]>>,
    stop_tables: Vec<Vec<[i32; ROOT_COUNT]>>,
}
fn tables(c: SourceEndAngularConfig, b: &[u8]) -> Result<Vec<Vec<[i32; ROOT_COUNT]>>> {
    let q = unpack_coefficients(c.coefficient_count()?, b).map_err(|e| error(e.to_string()))?;
    let mut rows = q.chunks_exact(ROOT_COUNT);
    let mut result = Vec::new();
    for _ in 0..c.heads {
        let mut head = Vec::new();
        for _ in 0..c.lanes_per_head {
            let row = rows
                .next()
                .ok_or_else(|| error("source-end table absent"))?;
            let mut table = [0; ROOT_COUNT];
            for (dst, &v) in table.iter_mut().zip(row) {
                *dst = i32::from(v) << 22;
            }
            head.push(table);
        }
        result.push(head);
    }
    Ok(result)
}
impl SourceEndAngularQ4 {
    pub fn new(config: SourceEndAngularConfig, period: &[u8], stop: &[u8]) -> Result<Self> {
        Ok(Self {
            config,
            period: period.to_vec(),
            stop: stop.to_vec(),
            period_tables: tables(config, period)?,
            stop_tables: tables(config, stop)?,
        })
    }
    pub fn config(&self) -> SourceEndAngularConfig {
        self.config
    }
    pub fn period_packed_coefficients(&self) -> &[u8] {
        &self.period
    }
    pub fn stop_packed_coefficients(&self) -> &[u8] {
        &self.stop
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEndTransportMetadata {
    pub schema: String,
    pub policy: String,
    pub parent_artifact: NativeArtifactBinding,
    pub frozen_cue: CueCarrierMetadata,
    pub frozen_prefix: PrefixTransportMetadata,
    pub context: ContextQ4Config,
    pub context_packed_sha256: String,
    pub algebra_sha256: String,
    pub potential: SourceEndAngularConfig,
    pub period_packed_sha256: String,
    pub stop_packed_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEndState {
    pub source_segment_index: usize,
    pub token_ids: Vec<u32>,
    pub states: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SourceEndCosts {
    pub extra_source_encoder_tokens: usize,
    pub extra_context_coefficient_reads: usize,
    pub extra_geometry_relative_operations: usize,
    pub extra_angular_table_reads: usize,
    pub compiled_table_payload_bytes: usize,
    pub packed_source_bytes: usize,
    pub logical_sidecar_payload_bytes: usize,
    pub preparation_vec_containers: usize,
    pub extra_global_reductions: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEndTransportTrace {
    pub metadata: SourceEndTransportMetadata,
    pub sources: Vec<SourceEndState>,
    pub response: PrefixState,
    pub selected_bank_index: Option<usize>,
    pub selected_source_index: Option<usize>,
    pub factual_joint_copy_q24: Vec<i64>,
    pub angular_indices: Vec<u8>,
    pub relative_roots: Vec<u8>,
    pub period_q24: Vec<i64>,
    pub stop_q24: Vec<i64>,
    pub costs: SourceEndCosts,
}
pub struct NativeSourceEndTransport<'a> {
    context: &'a NativeContextQ4,
    geometry: &'a HistoricalH4Tables,
    potential: SourceEndAngularQ4,
    metadata: SourceEndTransportMetadata,
}
/// All raw Copy heads participate; strict comparison preserves earliest bank
/// occurrence ties. Terminal scores, labels and endpoint coefficients are absent.
pub fn factual_copy_route(
    heads: &[HeadTrace],
    candidates: usize,
) -> Result<(Option<usize>, Vec<i64>)> {
    if heads.is_empty() || heads.iter().any(|h| h.scores_q24.len() != candidates) {
        return Err(error("source-end factual Copy shape differs"));
    }
    let mut scores = vec![0i64; candidates];
    for head in heads {
        for (sum, &score) in scores.iter_mut().zip(&head.scores_q24) {
            *sum = sum
                .checked_add(score)
                .ok_or_else(|| error("source-end factual score overflow"))?;
        }
    }
    let mut best = None;
    for (index, &score) in scores.iter().enumerate() {
        if best.map(|i: usize| score > scores[i]).unwrap_or(true) {
            best = Some(index);
        }
    }
    Ok((best, scores))
}
impl<'a> NativeSourceEndTransport<'a> {
    pub fn compile(
        parent: NativeArtifactBinding,
        context: &'a NativeContextQ4,
        geometry: &'a HistoricalH4Tables,
        frozen_cue: CueCarrierMetadata,
        frozen_prefix: PrefixTransportMetadata,
        potential: SourceEndAngularQ4,
    ) -> Result<Self> {
        parent.identity.validate()?;
        let c = context.config();
        let p = potential.config();
        let algebra =
            TRUSTED_MATHEMATICAL_SHA256.ok_or_else(|| error("source-end algebra absent"))?;
        let context_sha = sha(context.packed_coefficients());
        if frozen_prefix.schema != crate::geometric_prefix_transport::SCHEMA
            || frozen_prefix.policy != crate::geometric_prefix_transport::POLICY
            || frozen_cue.context != c
            || frozen_cue.context_packed_sha256 != context_sha
            || frozen_cue.algebra_sha256 != algebra
            || frozen_prefix.parent_artifact != parent
            || frozen_cue.parent_artifact != parent
            || frozen_prefix.frozen_cue != frozen_cue
            || frozen_prefix.context != c
            || frozen_prefix.context_packed_sha256 != context_sha
            || frozen_prefix.algebra_sha256 != algebra
            || p.heads != c.heads
            || p.lanes_per_head != c.lanes_per_head
        {
            return Err(error("source-end frozen parent/cue/prefix/context differs"));
        }
        let metadata = SourceEndTransportMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            parent_artifact: parent,
            frozen_cue,
            frozen_prefix,
            context: c,
            context_packed_sha256: context_sha,
            algebra_sha256: algebra.into(),
            potential: p,
            period_packed_sha256: sha(potential.period_packed_coefficients()),
            stop_packed_sha256: sha(potential.stop_packed_coefficients()),
        };
        Ok(Self {
            context,
            geometry,
            potential,
            metadata,
        })
    }
    pub fn metadata(&self) -> &SourceEndTransportMetadata {
        &self.metadata
    }
    pub fn period_packed_coefficients(&self) -> &[u8] {
        self.potential.period_packed_coefficients()
    }
    pub fn stop_packed_coefficients(&self) -> &[u8] {
        self.potential.stop_packed_coefficients()
    }
    pub(crate) fn validate_execution(
        &self,
        parent: &NativeArtifactBinding,
        context: &NativeContextQ4,
        geometry: &HistoricalH4Tables,
        cue: &CueCarrierMetadata,
        prefix: &PrefixTransportMetadata,
    ) -> Result<()> {
        if parent != &self.metadata.parent_artifact
            || cue != &self.metadata.frozen_cue
            || prefix != &self.metadata.frozen_prefix
            || !std::ptr::eq(context, self.context)
            || !std::ptr::eq(geometry, self.geometry)
        {
            return Err(error("foreign source-end execution"));
        }
        Ok(())
    }
    pub(crate) fn prepare(
        &self,
        prefix: &PrefixTransportTrace,
        heads: &[HeadTrace],
    ) -> Result<SourceEndTransportTrace> {
        if prefix.metadata != self.metadata.frozen_prefix {
            return Err(error("source-end prefix trace differs"));
        }
        let c = self.context.config();
        let width = c.heads * c.lanes_per_head;
        let (selected_bank_index, factual_joint_copy_q24) =
            factual_copy_route(heads, prefix.candidate_source_indices.len())?;
        let selected_source_index = selected_bank_index.map(|i| prefix.candidate_source_indices[i]);
        let mut costs = SourceEndCosts {
            compiled_table_payload_bytes: self.potential.config().coefficient_count()? * 8,
            packed_source_bytes: self.period_packed_coefficients().len()
                + self.stop_packed_coefficients().len(),
            extra_global_reductions: 1,
            ..Default::default()
        };
        let mut sources = Vec::new();
        for source in &prefix.sources {
            let states = if let Some(&last) = source.token_ids.last() {
                let before = source
                    .states_before
                    .last()
                    .ok_or_else(|| error("source-end last prefix absent"))?;
                let retained = before
                    .iter()
                    .copied()
                    .map(H4Code::try_from)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| error(e.to_string()))?;
                let mut state =
                    NativeContextState::from_states(c.heads, c.lanes_per_head, &retained)
                        .map_err(|e| error(e.to_string()))?;
                let step = state
                    .step(last as usize, self.context.native(), self.geometry)
                    .map_err(|e| error(e.to_string()))?;
                costs.extra_source_encoder_tokens += 1;
                costs.extra_context_coefficient_reads += step.coefficient_reads;
                state.states().iter().map(|s| s.index()).collect()
            } else {
                vec![H4Code::IDENTITY.index(); width]
            };
            sources.push(SourceEndState {
                source_segment_index: source.source_segment_index,
                token_ids: source.token_ids.clone(),
                states,
            });
        }
        let mut angular_indices = vec![0; width];
        let mut relative_roots = vec![0; width];
        let mut period_q24 = vec![0i64; c.heads];
        let mut stop_q24 = period_q24.clone();
        if let Some(source) = selected_source_index {
            let end = sources
                .get(source)
                .ok_or_else(|| error("source-end candidate Source absent"))?;
            if end.states.len() != width || prefix.response.states.len() != width {
                return Err(error("source-end retained state shape differs"));
            }
            let mut global = 0;
            for h in 0..c.heads {
                for lane in 0..c.lanes_per_head {
                    let response = H4Code::try_from(prefix.response.states[global])
                        .map_err(|e| error(e.to_string()))?;
                    let root =
                        H4Code::try_from(end.states[global]).map_err(|e| error(e.to_string()))?;
                    let relative = self.geometry.relative(response, root).index();
                    let index = match self.potential.config.mode {
                        SourceEndScoreMode::DirectedRelative => relative,
                        SourceEndScoreMode::SourceEndUnary => root.index(),
                    };
                    angular_indices[global] = index;
                    relative_roots[global] = relative;
                    period_q24[h] = period_q24[h]
                        .checked_add(i64::from(
                            self.potential.period_tables[h][lane][usize::from(index)],
                        ))
                        .ok_or_else(|| error("source-end Period overflow"))?;
                    stop_q24[h] = stop_q24[h]
                        .checked_add(i64::from(
                            self.potential.stop_tables[h][lane][usize::from(index)],
                        ))
                        .ok_or_else(|| error("source-end Stop overflow"))?;
                    costs.extra_geometry_relative_operations += 1;
                    costs.extra_angular_table_reads += 2;
                    global += 1;
                }
            }
        }
        costs.logical_sidecar_payload_bytes = sources
            .iter()
            .map(|s| s.token_ids.len() * 4 + s.states.len() + std::mem::size_of::<usize>())
            .sum::<usize>()
            + prefix.response.token_ids.len() * 4
            + prefix.response.states.len()
            + factual_joint_copy_q24.len() * 8
            + width * 2
            + c.heads * 16;
        costs.preparation_vec_containers = 8 + sources.len() * 3;
        Ok(SourceEndTransportTrace {
            metadata: self.metadata.clone(),
            sources,
            response: prefix.response.clone(),
            selected_bank_index,
            selected_source_index,
            factual_joint_copy_q24,
            angular_indices,
            relative_roots,
            period_q24,
            stop_q24,
            costs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn head(scores: Vec<i64>) -> HeadTrace {
        HeadTrace {
            scores_q24: scores,
            weights_q31: Vec::new(),
            no_read_q24: i64::MAX,
            no_read_weight_q31: 0,
            total_weight_q31: 0,
        }
    }
    #[test]
    fn factual_route_sums_heads_preserves_earliest_tie_and_has_no_terminal_gate() -> Result<()> {
        let (index, scores) =
            factual_copy_route(&[head(vec![10, -2, 8]), head(vec![-10, 2, -8])], 3)?;
        assert_eq!(scores, vec![0, 0, 0]);
        assert_eq!(index, Some(0));
        assert_eq!(
            factual_copy_route(&[head(vec![2, 9]), head(vec![9, -1])], 2)?.0,
            Some(0)
        );
        assert_eq!(factual_copy_route(&[head(Vec::new())], 0)?.0, None);
        assert!(factual_copy_route(&[head(vec![i64::MAX]), head(vec![1])], 1).is_err());
        assert!(factual_copy_route(&[head(vec![1])], 2).is_err());
        Ok(())
    }
    #[test]
    fn source_end_signed_quarters_and_independent_tables_are_exact() -> Result<()> {
        let c = SourceEndAngularConfig {
            heads: 2,
            lanes_per_head: 4,
            mode: SourceEndScoreMode::DirectedRelative,
        };
        let mut q = vec![0; c.coefficient_count()?];
        q[0] = -7;
        q[119] = 7;
        q[120] = 1;
        let packed = crate::geometric_potential_q4::pack_coefficients(&q)
            .map_err(|e| error(e.to_string()))?;
        let zero = vec![0; packed.len()];
        let p = SourceEndAngularQ4::new(c, &packed, &zero)?;
        assert_eq!(p.period_tables[0][0][0], -7i32 << 22);
        assert_eq!(p.period_tables[0][0][119], 7i32 << 22);
        assert_eq!(p.period_tables[0][1][0], 1i32 << 22);
        assert!(p
            .stop_tables
            .iter()
            .flatten()
            .all(|row| row.iter().all(|v| *v == 0)));
        assert!(SourceEndAngularQ4::new(c, &vec![0x88; packed.len()], &zero).is_err());
        assert!(SourceEndAngularQ4::new(c, &packed[..packed.len() - 1], &zero).is_err());
        Ok(())
    }
    #[test]
    fn retained_context_constructor_preserves_exact_codes_and_checks_shape() -> Result<()> {
        let roots = [
            H4Code::try_from(0).map_err(|e| error(e.to_string()))?,
            H4Code::IDENTITY,
        ];
        let state =
            NativeContextState::from_states(1, 2, &roots).map_err(|e| error(e.to_string()))?;
        assert_eq!(state.states(), &roots);
        assert!(NativeContextState::from_states(1, 2, &roots[..1]).is_err());
        assert!(NativeContextState::from_states(3, 2, &roots).is_err());
        Ok(())
    }
}
