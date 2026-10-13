//! One learned vector table for token input, addressed output and carried tokens.
//! Model arithmetic is CUDA-only. Product addresses are identity metadata, not
//! a semantic distance inferred from token numbers. Geometry scores displacement
//! in learned vectors; fixed SpiralCore atlas ports admit a bounded candidate set.
use crate::{invalid, sha256_bytes, Result};
#[cfg(test)]
use candle_core::IndexOp;
use candle_core::{DType, Tensor, TensorId};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "uor-r4.shared-native-vector-writer/1";
pub const CELLS: usize = 256;
pub const MAX_PER_CELL: usize = 32;
pub const DEFAULT_KEPT_CELLS: usize = 8;
/// Includes the pinned rung0 batch16 * context384 = 6144 positions.
/// Candidate cells and tokens per cell retain their independent bounds.
pub const MAX_QUERY_ROWS: usize = 8192;
// At most 1 GiB for each materialized F32 candidate-vector intermediate.
// Retained forward/backward graphs require a separately admitted GPU budget.
pub const MAX_CANDIDATE_VECTOR_ELEMENTS: usize = 256 * 1024 * 1024;
const ABSENT_SCORE: f32 = -1.0e30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterSpec {
    pub vocabulary: usize,
    pub width: usize,
    pub kept_cells: usize,
}
impl WriterSpec {
    pub fn validate(&self) -> Result<()> {
        if !(CELLS..=CELLS * MAX_PER_CELL).contains(&self.vocabulary)
            || self.width == 0
            || self.width % 8 != 0
            || self.width > 1024
            || !(1..=16).contains(&self.kept_cells)
        {
            return Err(invalid("native vector writer needs 256..8192 identities, whole 8D blocks <=1024, and 1..16 product cells"));
        }
        Ok(())
    }
    pub fn binding(&self) -> Result<serde_json::Value> {
        self.validate()?;
        let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
        Ok(serde_json::json!({"schema":SCHEMA, "spec":self,
            "source_sha256":sha256_bytes(include_bytes!("native_vector_writer.rs")),
            "geometry":plan.fingerprint,
            "identity_partition":"token-id-modulo-256; quotient is collision slot",
            "query_ports":"first-and-last-whole-8D-block; alias at width8",
            "score":"negative-mean-L1-displacement; raw",
            "address_credit":"target-cell-versus-admitted-cell-margin; surrogate"}))
    }
}

#[derive(Clone)]
pub struct NativeVectorWriter {
    table: Tensor,
    spec: WriterSpec,
    roots: Tensor,
    slots: usize,
    ids: Tensor,
    valid: Tensor,
}
/// IDs and masks accompany bounded raw scores. No omitted target is inserted.
pub struct VectorCandidates {
    token_ids: Tensor,
    valid: Tensor,
    raw_scores: Tensor,
    cell_ids: Tensor,
    address_scores: Tensor,
    query: Tensor,
    table_id: TensorId,
}
pub struct VectorMargin {
    pub token_margin: Tensor,
    pub address_margin: Tensor,
}
/// Per-position losses remain on device so a rate counts positions, not an
/// aggregate loss. Token credit and address-surrogate credit stay distinct.
pub struct VectorMarginRows {
    pub token_margin: Tensor,
    pub address_margin: Tensor,
}
/// CUDA scalar losses/counts and host shape metadata only. No score vector is
/// downloaded for ranking, objective evaluation or finiteness arithmetic.
pub struct VectorMetrics {
    pub rows: usize,
    pub token_margin: Tensor,
    pub address_margin: Tensor,
    pub total_margin: Tensor,
    pub top1_count: Tensor,
    pub recall_at_k_count: Tensor,
    pub admitted_target_count: Tensor,
    /// Token-only zero-violation count, as declared by the rung0 token bar.
    pub unviolated_count: Tensor,
    pub token_unviolated_count: Tensor,
    /// Both token and address components are zero; separate from the token bar.
    pub total_unviolated_count: Tensor,
}
impl VectorMargin {
    pub fn total(&self) -> Result<Tensor> {
        Ok(self.token_margin.add(&self.address_margin)?)
    }
}
impl VectorCandidates {
    pub fn token_ids(&self) -> &Tensor {
        &self.token_ids
    }
    pub fn valid(&self) -> &Tensor {
        &self.valid
    }
    pub fn raw_scores(&self) -> &Tensor {
        &self.raw_scores
    }
    pub fn cell_ids(&self) -> &Tensor {
        &self.cell_ids
    }
    pub fn address_scores(&self) -> &Tensor {
        &self.address_scores
    }
    /// Ranked kept token list for recall@K: bounded candidates only, smallest
    /// exact token identity on a tie, no target-aware insertion.
    pub fn top_token_ids(&self, k: usize) -> Result<Tensor> {
        if k == 0
            || k > self.raw_scores.dim(1)?
            || self
                .valid
                .to_dtype(DType::U32)?
                .sum(1)?
                .ge(k as u32)?
                .min_all()?
                .to_scalar::<u8>()?
                == 0
        {
            return Err(invalid(
                "requested token shortlist exceeds admitted identities",
            ));
        }
        let mut scores = self.raw_scores.detach();
        let mut result = Vec::new();
        for _ in 0..k {
            let tied = scores
                .broadcast_eq(&scores.max_keepdim(1)?)?
                .mul(&self.valid)?;
            let id = tied
                .where_cond(
                    &self.token_ids,
                    &Tensor::full(u32::MAX, self.token_ids.shape(), self.token_ids.device())?,
                )?
                .min_keepdim(1)?;
            scores = self.token_ids.broadcast_eq(&id)?.where_cond(
                &Tensor::full(ABSENT_SCORE, scores.shape(), scores.device())?,
                &scores,
            )?;
            result.push(id);
        }
        Ok(Tensor::cat(&result, 1)?)
    }
}
/// Exact occurrence/version identity stays separate from its geometric vector.
#[derive(Clone)]
pub struct CarriedVectorToken {
    token_id: u32,
    occurrence: u64,
    version: u64,
    vector: Tensor,
    table_id: TensorId,
}
impl CarriedVectorToken {
    pub fn token_id(&self) -> u32 {
        self.token_id
    }
    pub fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn vector(&self) -> &Tensor {
        &self.vector
    }
}
impl NativeVectorWriter {
    pub fn new(table: &Tensor, spec: WriterSpec) -> Result<Self> {
        // Refuse before any model arithmetic, including validation reductions.
        if !table.device().is_cuda() {
            return Err(invalid(
                "native vector writer requires CUDA; no host fallback",
            ));
        }
        spec.validate()?;
        if table.dtype() != DType::F32 || table.dims() != [spec.vocabulary, spec.width] {
            return Err(invalid("shared vector table shape/dtype differs"));
        }
        let g = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
        let roots = Tensor::from_vec(
            g.geometry
                .roots
                .iter()
                .flatten()
                .map(|v| f32::from(*v))
                .collect::<Vec<_>>(),
            (16, 8),
            table.device(),
        )?;
        let slots = spec.vocabulary.div_ceil(CELLS);
        let mut ids = Vec::with_capacity(CELLS * slots);
        let mut valid = Vec::with_capacity(CELLS * slots);
        for cell in 0..CELLS {
            for slot in 0..slots {
                let id = cell + CELLS * slot;
                ids.push(if id < spec.vocabulary { id as u32 } else { 0 });
                valid.push(u8::from(id < spec.vocabulary));
            }
        }
        Ok(Self {
            table: table.clone(),
            roots,
            slots,
            ids: Tensor::from_vec(ids, (CELLS, slots), table.device())?,
            valid: Tensor::from_vec(valid, (CELLS, slots), table.device())?,
            spec,
        })
    }
    pub fn spec(&self) -> &WriterSpec {
        &self.spec
    }
    pub fn table(&self) -> &Tensor {
        &self.table
    }
    pub fn table_id(&self) -> TensorId {
        self.table.id()
    }
    fn check(&self, value: &Tensor) -> Result<()> {
        if !value.device().is_cuda()
            || !value.device().same_device(self.table.device())
            || value.dtype() != DType::F32
        {
            return Err(invalid(
                "native vector arithmetic needs same-device CUDA F32",
            ));
        }
        // Only a validation status crosses to the host; no scores or gradients.
        if value.abs()?.le(1.0e6f32)?.min_all()?.to_scalar::<u8>()? == 0 {
            return Err(invalid(
                "native vector nonfinite or outside declared arithmetic bound",
            ));
        }
        Ok(())
    }
    pub fn encode(&self, ids: &[u32]) -> Result<Tensor> {
        if ids.is_empty() || ids.iter().any(|&id| id as usize >= self.spec.vocabulary) {
            return Err(invalid("native vector token inventory invalid"));
        }
        let at = Tensor::from_vec(ids.to_vec(), ids.len(), self.table.device())?;
        let vectors = self.table.index_select(&at, 0)?;
        self.check(&vectors)?;
        Ok(vectors)
    }
    pub fn carry(
        &self,
        token_id: u32,
        occurrence: u64,
        version: u64,
    ) -> Result<CarriedVectorToken> {
        Ok(CarriedVectorToken {
            token_id,
            occurrence,
            version,
            vector: self.encode(&[token_id])?,
            table_id: self.table.id(),
        })
    }
    pub fn read_carried(&self, token: &CarriedVectorToken) -> Result<Tensor> {
        if token.table_id != self.table.id() || token.token_id as usize >= self.spec.vocabulary {
            return Err(invalid(
                "carried token belongs to another shared parameter allocation",
            ));
        }
        self.check(&token.vector)?;
        Ok(token.vector.clone())
    }
    fn address_scores(&self, query: &Tensor) -> Result<Tensor> {
        let rows = query.dim(0)?;
        let port = |start| -> Result<Tensor> {
            Ok(query
                .narrow(1, start, 8)?
                .unsqueeze(1)?
                .broadcast_sub(&self.roots.unsqueeze(0)?)?
                .abs()?
                .mean(2)?
                .neg()?)
        };
        let a = port(0)?;
        let b = port(self.spec.width - 8)?;
        Ok(a.unsqueeze(2)?
            .broadcast_add(&b.unsqueeze(1)?)?
            .reshape((rows, CELLS))?)
    }
    pub fn candidates(&self, query: &Tensor) -> Result<VectorCandidates> {
        if query.dims().len() != 2
            || query.dim(0)? == 0
            || query.dim(0)? > MAX_QUERY_ROWS
            || query.dim(1)? != self.spec.width
        {
            return Err(invalid(
                "native writer query needs 1..8192 rows of complete vector blocks",
            ));
        }
        let candidate_elements = query
            .dim(0)?
            .checked_mul(self.spec.kept_cells * self.slots)
            .and_then(|n| n.checked_mul(self.spec.width))
            .ok_or_else(|| invalid("native candidate-vector inventory overflow"))?;
        if candidate_elements > MAX_CANDIDATE_VECTOR_ELEMENTS {
            return Err(invalid("native candidate-vector intermediate exceeds 1 GiB F32 bound; use explicitly weighted row chunks"));
        }
        self.check(query)?;
        let query = query.to_dtype(DType::F32)?;
        let rows = query.dim(0)?;
        let address_scores = self.address_scores(&query)?;
        let addresses = Tensor::from_vec(
            (0..CELLS as u32).collect::<Vec<_>>(),
            CELLS,
            self.table.device(),
        )?
        .unsqueeze(0)?
        .broadcast_as((rows, CELLS))?;
        let mut left = address_scores.detach();
        let mut cells = Vec::new();
        for _ in 0..self.spec.kept_cells {
            // Exact score tie -> smallest cell identity, never a score epsilon.
            let best = left.max_keepdim(1)?;
            let tied = left.broadcast_eq(&best)?;
            let absent = Tensor::full(u32::MAX, (rows, CELLS), self.table.device())?;
            let cell = tied.where_cond(&addresses, &absent)?.min_keepdim(1)?;
            left = addresses.broadcast_eq(&cell)?.where_cond(
                &Tensor::full(ABSENT_SCORE, (rows, CELLS), self.table.device())?,
                &left,
            )?;
            cells.push(cell);
        }
        let cell_ids = Tensor::cat(&cells, 1)?;
        let flat = cell_ids.flatten_all()?;
        let count = self.spec.kept_cells * self.slots;
        let token_ids = self.ids.index_select(&flat, 0)?.reshape((rows, count))?;
        let valid = self.valid.index_select(&flat, 0)?.reshape((rows, count))?;
        let vectors = self
            .table
            .index_select(&token_ids.flatten_all()?, 0)?
            .reshape((rows, count, self.spec.width))?;
        self.check(&vectors)?;
        let scores = query
            .unsqueeze(1)?
            .broadcast_sub(&vectors)?
            .abs()?
            .mean(2)?
            .neg()?;
        let raw_scores = valid.where_cond(
            &scores,
            &Tensor::full(ABSENT_SCORE, (rows, count), self.table.device())?,
        )?;
        Ok(VectorCandidates {
            token_ids,
            valid,
            raw_scores,
            cell_ids,
            address_scores,
            query,
            table_id: self.table.id(),
        })
    }
    pub fn write(&self, query: &Tensor) -> Result<Tensor> {
        let c = self.candidates(query)?;
        let best = c.raw_scores.max_keepdim(1)?;
        let tied = c.raw_scores.broadcast_eq(&best)?.mul(&c.valid)?;
        Ok(tied
            .where_cond(
                &c.token_ids,
                &Tensor::full(u32::MAX, c.token_ids.shape(), self.table.device())?,
            )?
            .min(1)?)
    }
    /// Native loss explicitly scores an omitted target but never changes admission.
    /// Admission margin supplies a labelled auxiliary surrogate, not hard-topK's derivative.
    pub fn margin_rows(
        &self,
        c: &VectorCandidates,
        targets: &[u32],
        rivals: usize,
    ) -> Result<VectorMarginRows> {
        let rows = c.query.dim(0)?;
        if c.table_id != self.table.id() || targets.len() != rows || !(1..=8).contains(&rivals) {
            return Err(invalid(
                "native vector objective binding/target/rival inventory differs",
            ));
        }
        let target_vectors = self.encode(targets)?;
        let target_score = c
            .query
            .sub(&target_vectors)?
            .abs()?
            .mean(1)?
            .neg()?
            .unsqueeze(1)?;
        let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), self.table.device())?;
        let available = c.token_ids.broadcast_ne(&target_ids)?.mul(&c.valid)?;
        if available
            .to_dtype(DType::U32)?
            .sum(1)?
            .ge(rivals as u32)?
            .min_all()?
            .to_scalar::<u8>()?
            == 0
        {
            return Err(invalid(
                "declared rivals exceed admitted non-target identities",
            ));
        }
        // For V>=256 each cell contains at least one real token; exclude self.
        let mut selected = available.where_cond(
            &c.raw_scores,
            &Tensor::full(ABSENT_SCORE, c.raw_scores.shape(), self.table.device())?,
        )?;
        let mut losses = Vec::new();
        let indices = Tensor::from_vec(
            (0..selected.dim(1)? as u32).collect::<Vec<_>>(),
            selected.dim(1)?,
            self.table.device(),
        )?
        .unsqueeze(0)?
        .broadcast_as(selected.shape())?;
        for _ in 0..rivals {
            let tied = selected
                .broadcast_eq(&selected.max_keepdim(1)?)?
                .mul(&available)?;
            let id = tied
                .where_cond(
                    &c.token_ids,
                    &Tensor::full(u32::MAX, c.token_ids.shape(), self.table.device())?,
                )?
                .min_keepdim(1)?;
            let at = c
                .token_ids
                .broadcast_eq(&id)?
                .mul(&c.valid)?
                .to_dtype(DType::U32)?
                .argmax_keepdim(1)?;
            let rival = selected.gather(&at, 1)?;
            let hinge = rival
                .broadcast_sub(&target_score)?
                .affine(1., 1.)?
                .maximum(&Tensor::zeros((rows, 1), DType::F32, self.table.device())?)?;
            losses.push(hinge);
            selected = indices.broadcast_eq(&at)?.where_cond(
                &Tensor::full(ABSENT_SCORE, selected.shape(), self.table.device())?,
                &selected,
            )?;
        }
        let target_cells = Tensor::from_vec(
            targets
                .iter()
                .map(|id| id % CELLS as u32)
                .collect::<Vec<_>>(),
            (rows, 1),
            self.table.device(),
        )?;
        let target_address = c.address_scores.gather(&target_cells, 1)?;
        let admitted = c.address_scores.gather(&c.cell_ids, 1)?;
        let other = c.cell_ids.broadcast_ne(&target_cells)?;
        let address_rival = other
            .where_cond(
                &admitted,
                &Tensor::full(ABSENT_SCORE, admitted.shape(), self.table.device())?,
            )?
            .max_keepdim(1)?;
        let address_loss = address_rival
            .sub(&target_address)?
            .affine(1., 1.)?
            .maximum(&Tensor::zeros((rows, 1), DType::F32, self.table.device())?)?;
        Ok(VectorMarginRows {
            token_margin: Tensor::cat(&losses, 1)?.mean(1)?,
            address_margin: address_loss.squeeze(1)?,
        })
    }
    pub fn margin_components(
        &self,
        c: &VectorCandidates,
        targets: &[u32],
        rivals: usize,
    ) -> Result<VectorMargin> {
        let rows = self.margin_rows(c, targets, rivals)?;
        Ok(VectorMargin {
            token_margin: rows.token_margin.mean_all()?,
            address_margin: rows.address_margin.mean_all()?,
        })
    }
    /// Measures bounded raw output and the exact objective components. An
    /// omitted target is scored by the objective but never inserted into the
    /// kept list, so recall and admission can remain zero even with low loss.
    pub fn metrics(
        &self,
        c: &VectorCandidates,
        targets: &[u32],
        rivals: usize,
        k: usize,
    ) -> Result<VectorMetrics> {
        let margins = self.margin_rows(c, targets, rivals)?;
        let rows = targets.len();
        let targets = Tensor::from_vec(targets.to_vec(), (rows, 1), self.table.device())?;
        let ranked = c.top_token_ids(k)?;
        let top1_count = ranked
            .narrow(1, 0, 1)?
            .eq(&targets)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let recall_at_k_count = ranked
            .broadcast_eq(&targets)?
            .to_dtype(DType::U32)?
            .sum(1)?
            .gt(0u32)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let admitted_target_count = c
            .token_ids
            .broadcast_eq(&targets)?
            .mul(&c.valid)?
            .to_dtype(DType::U32)?
            .sum(1)?
            .gt(0u32)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let token_zero = margins.token_margin.eq(0f32)?;
        let token_unviolated_count = token_zero.to_dtype(DType::U32)?.sum_all()?;
        let total_unviolated_count = token_zero
            .mul(&margins.address_margin.eq(0f32)?)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let token_margin = margins.token_margin.mean_all()?;
        let address_margin = margins.address_margin.mean_all()?;
        Ok(VectorMetrics {
            rows,
            total_margin: token_margin.add(&address_margin)?,
            token_margin,
            address_margin,
            top1_count,
            recall_at_k_count,
            admitted_target_count,
            unviolated_count: token_unviolated_count.clone(),
            token_unviolated_count,
            total_unviolated_count,
        })
    }
    pub fn margin(&self, c: &VectorCandidates, targets: &[u32], rivals: usize) -> Result<Tensor> {
        self.margin_components(c, targets, rivals)?.total()
    }
}

impl NativeVectorWriter {
    /// Full-V compatibility allocation, with no additional vector comparisons
    /// or parameter scan. The native margin never consumes omitted sentinels.
    pub fn compatibility_scores(&self, query: &Tensor) -> Result<Tensor> {
        let c = self.candidates(query)?;
        let rows = query.dim(0)?;
        let zero = Tensor::zeros(c.raw_scores.shape(), DType::F32, self.table.device())?;
        let values = c.valid.where_cond(&c.raw_scores, &zero)?;
        let base = Tensor::zeros(
            (rows, self.spec.vocabulary),
            DType::F32,
            self.table.device(),
        )?;
        let scores = base.scatter_add(&c.token_ids, &values, 1)?;
        let present = base
            .scatter_add(&c.token_ids, &c.valid.to_dtype(DType::F32)?, 1)?
            .gt(0f32)?;
        Ok(present.where_cond(
            &scores,
            &Tensor::full(ABSENT_SCORE, scores.shape(), self.table.device())?,
        )?)
    }
}

/// The compiler/store/emitter path borrows the very same learned parameter
/// tensor as the stack. It cannot rebuild another prototype table or silently
/// reinterpret an old packed/probability artifact. Exact store control is host
/// metadata; encode, carrier transport, selection and learning stay on CUDA.
#[derive(Clone)]
pub struct NativeVectorChannel {
    writer: NativeVectorWriter,
}
impl NativeVectorChannel {
    pub fn new(writer: NativeVectorWriter) -> Self {
        Self { writer }
    }
    pub fn writer(&self) -> &NativeVectorWriter {
        &self.writer
    }
    pub fn compile_tokens(
        &self,
        ids: &[u32],
        first_occurrence: u64,
        version: u64,
    ) -> Result<Vec<CarriedVectorToken>> {
        let vectors = self.writer.encode(ids)?;
        ids.iter()
            .enumerate()
            .map(|(i, &token_id)| {
                Ok(CarriedVectorToken {
                    token_id,
                    occurrence: first_occurrence
                        .checked_add(i as u64)
                        .ok_or_else(|| invalid("vector occurrence overflow"))?,
                    version,
                    vector: vectors.narrow(0, i, 1)?,
                    table_id: self.writer.table_id(),
                })
            })
            .collect()
    }
    pub fn store_read(
        &self,
        token: &CarriedVectorToken,
        occurrence: u64,
        version: u64,
    ) -> Result<Tensor> {
        if token.occurrence != occurrence || token.version != version {
            return Err(invalid("vector occurrence/version witness differs"));
        }
        self.writer.read_carried(token)
    }
    pub fn transport(&self, token: &CarriedVectorToken, queries: &Tensor) -> Result<Tensor> {
        let carrier = self.writer.read_carried(token)?;
        crate::spiralcore_fbs::ProjectionPlan::new(self.writer.spec.width, self.writer.spec.width)?
            .forward(queries, &carrier)
    }
    pub fn emit(&self, vector: &Tensor) -> Result<Tensor> {
        self.writer.write(vector)
    }
    pub fn loss(&self, vector: &Tensor, targets: &[u32], rivals: usize) -> Result<Tensor> {
        self.writer
            .margin(&self.writer.candidates(vector)?, targets, rivals)
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    #[test]
    fn binding_and_cpu_refusal() -> Result<()> {
        let spec = WriterSpec {
            vocabulary: 4096,
            width: 8,
            kept_cells: 8,
        };
        let binding = spec.binding()?;
        assert_eq!(binding["schema"], SCHEMA);
        assert_eq!(binding, spec.binding()?);
        assert!(WriterSpec {
            width: 7,
            ..spec.clone()
        }
        .validate()
        .is_err());
        assert!(WriterSpec {
            vocabulary: 8193,
            ..spec.clone()
        }
        .validate()
        .is_err());
        let table = Tensor::zeros((4096, 8), DType::F32, &candle_core::Device::Cpu)?;
        assert!(NativeVectorWriter::new(&table, spec).is_err());
        Ok(())
    }
}
#[cfg(all(test, feature = "cuda"))]
mod cuda_tests {
    use super::*;
    use candle_core::{Device, Var};
    fn fixture(vocab: usize, width: usize, kept: usize) -> Result<(Var, NativeVectorWriter)> {
        let device = Device::new_cuda(0)?; // Missing CUDA is a failure, not a skipped PASS.
                                           // Fixed input constants, not a host-side forward/reference calculation.
        let table = Var::from_tensor(&Tensor::zeros((vocab, width), DType::F32, &device)?)?;
        let writer = NativeVectorWriter::new(
            table.as_tensor(),
            WriterSpec {
                vocabulary: vocab,
                width,
                kept_cells: kept,
            },
        )?;
        Ok((table, writer))
    }
    #[test]
    fn ties_identity_and_shared_channel() -> Result<()> {
        let (_, w) = fixture(257, 8, 8)?;
        let q = Tensor::zeros((1, 8), DType::F32, w.table.device())?;
        let c = w.candidates(&q)?;
        assert_eq!(
            c.cell_ids.to_vec2::<u32>()?[0],
            vec![0, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(w.write(&q)?.to_vec1::<u32>()?, vec![0]);
        assert_eq!(
            c.top_token_ids(8)?.to_vec2::<u32>()?[0],
            vec![0, 1, 2, 3, 4, 5, 6, 7]
        );
        // Repeated identities at collision slots survive; padding never emits.
        assert_eq!(c.token_ids.to_vec2::<u32>()?[0][1], 256);
        assert_eq!(c.valid.to_vec2::<u8>()?[0][3], 0);
        let channel = NativeVectorChannel::new(w.clone());
        assert_eq!(channel.writer.table_id(), w.table_id());
        let tokens = channel.compile_tokens(&[256, 0], 42, 9)?;
        assert_eq!(tokens[0].token_id, 256);
        assert_eq!(tokens[1].occurrence, 43);
        assert_eq!(
            channel.store_read(&tokens[0], 42, 9)?.to_vec2::<f32>()?,
            vec![vec![0.; 8]]
        );
        assert!(channel.store_read(&tokens[0], 42, 8).is_err());
        assert_eq!(
            channel
                .emit(&channel.store_read(&tokens[0], 42, 9)?)?
                .to_vec1::<u32>()?,
            w.write(&q)?.to_vec1::<u32>()?
        );
        let (_, other) = fixture(257, 8, 8)?;
        assert!(other.read_carried(&tokens[0]).is_err());
        Ok(())
    }
    #[test]
    fn local_distance_adjoints_and_omitted_target() -> Result<()> {
        let (table, w) = fixture(512, 16, 8)?;
        let q = Var::from_tensor(&Tensor::full(0.5f32, (1, 16), w.table.device())?)?;
        let c = w.candidates(q.as_tensor())?;
        // Fixed analytic expectation: displacement .5 in each coordinate.
        for value in c.raw_scores.to_vec2::<f32>()?[0].iter() {
            assert_eq!(*value, -0.5);
        }
        let grads = c.raw_scores.i((0, 0))?.backward()?;
        assert_eq!(
            grads
                .get(q.as_tensor())
                .ok_or_else(|| invalid("query credit absent"))?
                .to_vec2::<f32>()?,
            vec![vec![-0.0625; 16]]
        );
        let tg = grads
            .get(table.as_tensor())
            .ok_or_else(|| invalid("table credit absent"))?
            .to_vec2::<f32>()?;
        let first = c.token_ids.to_vec2::<u32>()?[0][0] as usize;
        assert_eq!(tg[first], vec![0.0625; 16]);
        assert_eq!(tg.iter().filter(|r| r.iter().any(|x| *x != 0.)).count(), 1);
        let admitted = c.token_ids.to_vec2::<u32>()?[0].clone();
        let target = (0..512u32)
            .find(|id| !admitted.contains(id))
            .ok_or_else(|| invalid("fixture needs omitted target"))?;
        let before = c.token_ids.to_vec2::<u32>()?;
        let loss = w.margin(&c, &[target], 1)?;
        assert!(loss.to_scalar::<f32>()?.is_finite());
        let gradients = loss.backward()?;
        let address = w.margin_components(&c, &[target], 1)?.address_margin;
        let address_grads = address.backward()?;
        assert!(
            address_grads
                .get(q.as_tensor())
                .ok_or_else(|| invalid("address credit absent"))?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        assert_eq!(
            w.compatibility_scores(q.as_tensor())?
                .i((0, first))?
                .to_scalar::<f32>()?,
            -0.5
        );
        assert_eq!(
            w.compatibility_scores(q.as_tensor())?
                .i((0, target as usize))?
                .to_scalar::<f32>()?,
            ABSENT_SCORE
        );
        assert!(
            gradients
                .get(table.as_tensor())
                .ok_or_else(|| invalid("omitted target credit absent"))?
                .i(target as usize)?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        assert_eq!(c.token_ids.to_vec2::<u32>()?, before); // No target-injected inference.
        assert_eq!(
            NativeVectorChannel::new(w.clone())
                .loss(q.as_tensor(), &[target], 1)?
                .to_scalar::<f32>()?,
            loss.to_scalar::<f32>()?
        );
        Ok(())
    }
    #[test]
    fn numerical_and_device_boundaries() -> Result<()> {
        let (_, w) = fixture(256, 8, 8)?;
        let dev = w.table.device();
        assert!(w.candidates(&Tensor::full(f32::NAN, (1, 8), dev)?).is_err());
        assert!(w
            .candidates(&Tensor::zeros((1, 7), DType::F32, dev)?)
            .is_err());
        assert!(w
            .candidates(&Tensor::zeros((1, 8), DType::F32, &Device::Cpu)?)
            .is_err());
        assert!(w.encode(&[256]).is_err());
        assert!(w.encode(&[]).is_err());
        Ok(())
    }

    #[test]
    fn device_metrics_keep_token_and_address_bars_distinct() -> Result<()> {
        let (_, w) = fixture(257, 8, 8)?;
        let q = Tensor::zeros((4, 8), DType::F32, w.table.device())?;
        let candidates = w.candidates(&q)?;
        let metrics = w.metrics(&candidates, &[0, 7, 8, 256], 1, 8)?;
        assert_eq!(metrics.rows, 4);
        assert_eq!(metrics.token_margin.to_scalar::<f32>()?, 1.);
        assert_eq!(metrics.address_margin.to_scalar::<f32>()?, 1.);
        assert_eq!(metrics.total_margin.to_scalar::<f32>()?, 2.);
        // Zero query: all address and token scores tie. Admission is cells0..7,
        // shortlist0..7; collision256 is admitted but outside recall@8.
        assert_eq!(metrics.top1_count.to_scalar::<u32>()?, 1);
        assert_eq!(metrics.recall_at_k_count.to_scalar::<u32>()?, 2);
        assert_eq!(metrics.admitted_target_count.to_scalar::<u32>()?, 3);
        assert_eq!(metrics.unviolated_count.to_scalar::<u32>()?, 0);
        assert_eq!(metrics.total_unviolated_count.to_scalar::<u32>()?, 0);

        let dev = w.table.device();
        let table = Tensor::cat(
            &[
                Tensor::zeros((1, 8), DType::F32, dev)?,
                Tensor::full(2f32, (255, 8), dev)?,
            ],
            0,
        )?;
        let separated = NativeVectorWriter::new(
            &table,
            WriterSpec {
                vocabulary: 256,
                width: 8,
                kept_cells: 8,
            },
        )?;
        let query = Tensor::zeros((1, 8), DType::F32, dev)?;
        let metrics = separated.metrics(&separated.candidates(&query)?, &[0], 1, 8)?;
        assert_eq!(metrics.token_margin.to_scalar::<f32>()?, 0.);
        assert_eq!(metrics.address_margin.to_scalar::<f32>()?, 1.);
        assert_eq!(metrics.token_unviolated_count.to_scalar::<u32>()?, 1);
        assert_eq!(metrics.unviolated_count.to_scalar::<u32>()?, 1);
        assert_eq!(metrics.total_unviolated_count.to_scalar::<u32>()?, 0);

        let (_, singleton) = fixture(256, 8, 1)?;
        let singleton_query = Tensor::zeros((1, 8), DType::F32, singleton.table.device())?;
        let only = singleton.candidates(&singleton_query)?;
        assert!(singleton.margin(&only, &[0], 1).is_err());
        assert!(singleton.metrics(&only, &[0], 1, 1).is_err());
        assert!(only.top_token_ids(2).is_err());
        Ok(())
    }

    #[test]
    fn rung0_full_batch_writer_forward_backward_without_updates() -> Result<()> {
        // Exact pinned head shape, independently of a whole-trunk fixture.
        // 6144*128*256 F32 values = 768MiB per materialized vector intermediate;
        // declared retained-graph budget is separate from this 1GiB guard.
        let (table, writer) = fixture(4096, 256, 8)?;
        let dev = writer.table.device();
        let rows = 16 * 384;
        let query = Var::from_tensor(&Tensor::full(0.5f32, (rows, 256), dev)?)?;
        let candidates = writer.candidates(query.as_tensor())?;
        assert_eq!(candidates.raw_scores.dims(), [rows, 128]);
        assert_eq!(
            candidates
                .raw_scores
                .affine(1., 0.5)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?,
            0.
        );
        let metrics = writer.metrics(&candidates, &vec![0; rows], 8, 8)?;
        assert_eq!(metrics.rows, rows);
        assert_eq!(metrics.token_margin.to_scalar::<f32>()?, 1.);
        let gradients = metrics.total_margin.backward()?;
        for parameter in [query.as_tensor(), table.as_tensor()] {
            let gradient = gradients
                .get(parameter)
                .ok_or_else(|| invalid("full-shape writer parameter gradient absent"))?;
            assert_eq!(gradient.dims(), parameter.dims());
            assert_eq!(
                gradient.abs()?.le(1.0e6f32)?.min_all()?.to_scalar::<u8>()?,
                1
            );
        }
        // No optimizer exists in this test. GPU checks verify unchanged values.
        assert_eq!(table.abs()?.max_all()?.to_scalar::<f32>()?, 0.);
        assert_eq!(query.eq(0.5f32)?.min_all()?.to_scalar::<u8>()?, 1);
        assert!(writer
            .candidates(&Tensor::zeros((MAX_QUERY_ROWS + 1, 256), DType::F32, dev)?)
            .is_err());
        Ok(())
    }

    #[test]
    fn actual_stack_store_channel_and_artifact_identity() -> Result<()> {
        use crate::geometric_stack::{QuaternionWeights, StackArch, StackConfig, StackModel};
        use crate::whole_path_learning::device::{
            carry_native_store_read, native_vector_channel, DeviceVectorStoreRead,
        };
        let dev = Device::new_cuda(0)?;
        let mut config = StackConfig::transformer_control(3);
        config.arch = StackArch::Geometric;
        // Parameter preparation uses the same admitted geometric recurrence as
        // the native trunk; the historical initializer is excluded under D29.
        config.rec_gates = crate::geometric_stack::RecGates::Geometric;
        config.gate = crate::geometric_stack::Gate::Hamilton;
        config.pattern = "r".into();
        config.width = 32;
        config.heads = 4;
        config.mlp_hidden = 64;
        config.vocab_size = 257;
        config.context = 4;
        let mut model = StackModel::new(config, &dev)?; // Parameter preparation only.
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(8)?;
        assert!(model
            .add_pointer(crate::geometric_stack::PointerConfig::new(128), 0)
            .is_err());
        model.config.pointer = Some(crate::geometric_stack::PointerConfig::new(128));
        assert!(model.native_vector_writer().is_err());
        model.config.pointer = None;
        let writer = model.native_vector_writer()?;
        let channel = native_vector_channel(&model)?;
        assert!(model.target_nll(&[0], &[0], 1, 1).is_err());
        assert!(model.next_scores(&[0]).is_err());
        assert!(model.next_scores_with_pointer(&[0]).is_err());
        assert!(model.last_pointer_selection(&[0]).is_err());
        assert_eq!(writer.table_id(), channel.writer().table_id());
        let vectors = model.embed(&[0, 1], 1, 2)?.reshape((2, 32))?;
        assert_eq!(
            vectors.to_vec2::<f32>()?,
            writer.encode(&[0, 1])?.to_vec2::<f32>()?
        );
        assert_eq!(
            model.head(&vectors)?.to_vec2::<f32>()?,
            writer.compatibility_scores(&vectors)?.to_vec2::<f32>()?
        );
        assert_eq!(
            channel.emit(&vectors)?.to_vec1::<u32>()?,
            writer.write(&vectors)?.to_vec1::<u32>()?
        );
        let stored = crate::stack_store::StoreRead::Found(crate::stack_store::StoreValue {
            tokens: vec![7, 7],
            record: 42,
            commit: 9,
            update: crate::stack_store::Update::Correct,
            conflict: true,
        });
        let DeviceVectorStoreRead::Found(carried) = carry_native_store_read(&channel, &stored)?
        else {
            return Err(invalid("found record lost"));
        };
        assert_eq!((carried.record, carried.commit), (42, 9));
        assert!(carried.conflict);
        assert_eq!(carried.update, crate::stack_store::Update::Correct);
        assert_eq!(
            carried
                .tokens
                .iter()
                .map(|t| t.occurrence)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        let plan = crate::spiralcore_fbs::ProjectionPlan::new(32, 32)?;
        let queries = Tensor::zeros(&plan.query_shape(), DType::F32, &dev)?;
        let routed = channel.transport(&carried.tokens[0], &queries)?;
        assert_eq!(
            routed.to_vec2::<f32>()?,
            plan.forward(&queries, carried.tokens[0].vector())?
                .to_vec2::<f32>()?
        );
        assert_eq!(
            channel.emit(&routed)?.to_vec1::<u32>()?,
            writer.write(&routed)?.to_vec1::<u32>()?
        );
        assert!(matches!(
            carry_native_store_read(&channel, &crate::stack_store::StoreRead::Evicted)?,
            DeviceVectorStoreRead::Evicted
        ));
        let root = std::path::PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .ok_or_else(|| invalid("owned artifact directory required"))?,
        )
        .join(format!(
            "vector-artifact-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        std::fs::create_dir(&root)?;
        model.save(&root)?;
        let restored = StackModel::load(&root, &dev)?;
        assert_eq!(restored.native_vector_writer()?.spec(), writer.spec());
        assert_eq!(
            restored
                .native_vector_writer()?
                .encode(&[0, 1])?
                .to_vec2::<f32>()?,
            vectors.to_vec2::<f32>()?
        );
        let file = root.join("config.json");
        let original = std::fs::read(&file)?;
        let mut saved: serde_json::Value = serde_json::from_slice(&original)?;
        saved
            .as_object_mut()
            .ok_or_else(|| invalid("config object absent"))?
            .remove("native_vector_head");
        std::fs::write(&file, serde_json::to_vec(&saved)?)?;
        assert!(StackModel::load(&root, &dev).is_err());
        std::fs::write(&file, original)?;
        Ok(())
    }
}
