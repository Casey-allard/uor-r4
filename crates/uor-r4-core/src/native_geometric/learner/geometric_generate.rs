//! Native ordinary-token scores from learned prototypes and shared finite-H4
//! unary/ordered-pair factors. Training/normalization are separate owners.
//! Numerical calls allocate nothing and do not invoke a floating table builder.
#![forbid(unsafe_code)]
use super::integrated_attention::geometry::{
    AlgebraReadCounts, EnergyReadCounts, EnergyTables, FiniteAlgebra, GeometryError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    h4_tables::{H4Code, HistoricalH4Tables, TRUSTED_MATHEMATICAL_SHA256},
};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
pub const MAX_VOCAB: usize = 4096;
pub const MAX_LANES: usize = 8;
pub const MAX_PAIRS: usize = 4;
pub const SCORE_SHIFT: u32 = 20;
pub const MAX_PAYLOAD_BYTES: usize = 512 * 1024;
pub const SCHEMA: &str = "uor-r4.native-geometric-generate/1";
pub const POLICY:&str="trusted-historical-signed-H4;directed-inverse-state-times-token-prototype;shared-full120-unary+declared-ordered-pairs;strict-q4[-7,7];score-code-shift20-Q24;all-bound-token-IDs;no-normalization/1";
const ALGEBRA_BYTES: &[u8] =
    include_bytes!("../../../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
#[derive(Clone, Debug)]
pub enum GenerateError {
    Shape(&'static str),
    Binding(&'static str),
    Coefficient { index: usize, value: i8 },
    Geometry(GeometryError),
    Artifact(String),
}
impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for GenerateError {}
impl From<GeometryError> for GenerateError {
    fn from(e: GeometryError) -> Self {
        Self::Geometry(e)
    }
}
pub type Result<T> = std::result::Result<T, GenerateError>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateMetadata {
    pub schema: String,
    pub policy: String,
    pub vocab_size: usize,
    pub lanes: usize,
    pub score_shift: u32,
    pub tokenizer_sha256: String,
    pub dialogue_protocol: DialogueProtocol,
    pub h4_mathematical_sha256: String,
    pub algebra_digest: [u8; 32],
    pub payload_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    algebra: FiniteAlgebra,
    prototypes: Vec<u8>,
    biases: Vec<u8>,
    energy: EnergyTables,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    metadata: GenerateMetadata,
    payload: Payload,
}
#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct GenerateReadCounts {
    pub algebra: AlgebraReadCounts,
    pub energy: EnergyReadCounts,
    pub prototype_reads: u64,
    pub bias_reads: u64,
    pub scores: u64,
}
#[derive(Clone, Debug)]
pub struct NativeGeometricGenerate {
    artifact: Artifact,
    // Admission-only index construction: numerical scoring reads this table
    // rather than multiplying a runtime token ID by a runtime lane count.
    prototype_offsets: Vec<usize>,
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn trusted() -> Result<HistoricalH4Tables> {
    HistoricalH4Tables::from_bytes(ALGEBRA_BYTES)
        .map_err(|e| GenerateError::Artifact(e.to_string()))
}
// Canonical frame admission is immutable; do not repeat its cubic group-law
// validation for every offline parameter snapshot. Learned payload admission
// and source/tokenizer binding are still checked independently each time.
fn trusted_algebra() -> Result<&'static FiniteAlgebra> {
    static TABLE: std::sync::OnceLock<Result<FiniteAlgebra>> = std::sync::OnceLock::new();
    TABLE
        .get_or_init(|| Ok(FiniteAlgebra::from_historical_h4(&trusted()?)?))
        .as_ref()
        .map_err(Clone::clone)
}
fn payload_hash(p: &Payload) -> Result<String> {
    Ok(hash(
        &serde_json::to_vec(p).map_err(|e| GenerateError::Artifact(e.to_string()))?,
    ))
}
fn decode(b: u8, odd: bool) -> i8 {
    let n = (b >> if odd { 4 } else { 0 }) & 15;
    if n < 8 {
        n as i8
    } else {
        n as i8 - 16
    }
}
impl NativeGeometricGenerate {
    /// All token IDs0..binding.vocab_size are retained, including EOS. The
    /// emission pool separately owns EOS/Stop legality; no hidden mask here.
    pub fn compile(
        binding: &SourceActionBinding,
        lanes: usize,
        prototypes: &[u8],
        packed_biases: &[u8],
        energy: EnergyTables,
    ) -> Result<Self> {
        let vocab = binding.vocab_size();
        if !(1..=MAX_VOCAB).contains(&vocab)
            || !(1..=MAX_LANES).contains(&lanes)
            || prototypes.len() != vocab * lanes
            || packed_biases.len() != vocab.div_ceil(2)
            || usize::from(energy.lanes()) != lanes
            || energy.edges().len() > MAX_PAIRS
        {
            return Err(GenerateError::Shape(
                "Generate constructor configuration/payload bounds differ",
            ));
        }
        let algebra = trusted_algebra()?.clone();
        let payload = Payload {
            algebra,
            prototypes: prototypes.to_vec(),
            biases: packed_biases.to_vec(),
            energy,
        };
        let mathematical = TRUSTED_MATHEMATICAL_SHA256.ok_or(GenerateError::Binding(
            "historical trust anchor unavailable",
        ))?;
        let metadata = GenerateMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            vocab_size: binding.vocab_size(),
            lanes,
            score_shift: SCORE_SHIFT,
            tokenizer_sha256: binding.tokenizer_sha256().into(),
            dialogue_protocol: binding.protocol().clone(),
            h4_mathematical_sha256: mathematical.into(),
            algebra_digest: *payload.algebra.digest(),
            payload_sha256: payload_hash(&payload)?,
        };
        let model = Self {
            prototype_offsets: (0..metadata.vocab_size)
                .map(|token| token * lanes)
                .collect(),
            artifact: Artifact { metadata, payload },
        };
        model.validate(binding)?;
        Ok(model)
    }
    pub fn from_bytes(bytes: &[u8], binding: &SourceActionBinding) -> Result<Self> {
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(GenerateError::Shape("Generate artifact size cap"));
        }
        let artifact: Artifact =
            serde_json::from_slice(bytes).map_err(|e| GenerateError::Artifact(e.to_string()))?;
        let model = Self {
            prototype_offsets: (0..artifact.metadata.vocab_size.min(MAX_VOCAB))
                .map(|token| token * artifact.metadata.lanes.min(MAX_LANES))
                .collect(),
            artifact,
        };
        model.validate(binding)?;
        Ok(model)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&self.artifact)
            .map_err(|e| GenerateError::Artifact(e.to_string()))?;
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(GenerateError::Shape(
                "Generate serialized artifact size cap",
            ));
        }
        Ok(bytes)
    }
    fn validate(&self, binding: &SourceActionBinding) -> Result<()> {
        let m = &self.artifact.metadata;
        let p = &self.artifact.payload;
        if m.schema != SCHEMA
            || m.policy != POLICY
            || m.score_shift != SCORE_SHIFT
            || m.vocab_size == 0
            || m.vocab_size > MAX_VOCAB
            || m.vocab_size != binding.vocab_size()
            || !(1..=MAX_LANES).contains(&m.lanes)
            || usize::from(p.energy.lanes()) != m.lanes
            || p.energy.edges().len() > MAX_PAIRS
        {
            return Err(GenerateError::Shape("Generate configuration differs"));
        }
        if m.tokenizer_sha256 != binding.tokenizer_sha256()
            || m.dialogue_protocol != *binding.protocol()
            || m.h4_mathematical_sha256
                != TRUSTED_MATHEMATICAL_SHA256.ok_or(GenerateError::Binding(
                    "historical trust anchor unavailable",
                ))?
        {
            return Err(GenerateError::Binding(
                "Generate tokenizer/protocol/H4 frame differs",
            ));
        }
        // Keep token-ID indexing stable even for sparse tokenizer domains.
        // The bound vocabulary reducer admits legal IDs; holes have no action
        // and cannot acquire probability merely by having a prototype row.
        if p.prototypes.len() != m.vocab_size * m.lanes
            || p.prototypes.iter().any(|&q| q >= 120)
            || p.biases.len() != m.vocab_size.div_ceil(2)
        {
            return Err(GenerateError::Shape(
                "Generate code/bias payload shape differs",
            ));
        }
        if m.vocab_size & 1 != 0 && p.biases.last().is_some_and(|b| b & 240 != 0) {
            return Err(GenerateError::Binding("Generate nonzero bias padding"));
        }
        for token in 0..m.vocab_size {
            self.token_bias(token)?;
        }
        p.energy.validate()?;
        for lane in 0..m.lanes {
            for root in 0..120 {
                let w = p.energy.get_unary(lane as u8, root)?;
                if w == -8 {
                    return Err(GenerateError::Coefficient {
                        index: lane * 120 + root as usize,
                        value: w,
                    });
                }
            }
        }
        for edge in 0..p.energy.edges().len() {
            for a in 0..120 {
                for b in 0..120 {
                    let w = p.energy.get_pair(edge, a, b)?;
                    if w == -8 {
                        return Err(GenerateError::Coefficient {
                            index: edge * 14400 + usize::from(a) * 120 + usize::from(b),
                            value: w,
                        });
                    }
                }
            }
        }
        let expected = trusted_algebra()?;
        if &p.algebra != expected || m.algebra_digest != *expected.digest() {
            return Err(GenerateError::Binding(
                "Generate algebra bytes/order differ from trusted context frame",
            ));
        }
        if m.payload_sha256 != payload_hash(p)? {
            return Err(GenerateError::Binding("Generate payload digest differs"));
        }
        Ok(())
    }
    pub fn metadata(&self) -> &GenerateMetadata {
        &self.artifact.metadata
    }
    pub fn vocab_size(&self) -> usize {
        self.metadata().vocab_size
    }
    pub fn lanes(&self) -> usize {
        self.metadata().lanes
    }
    pub fn prototypes(&self) -> &[u8] {
        &self.artifact.payload.prototypes
    }
    pub fn packed_biases(&self) -> &[u8] {
        &self.artifact.payload.biases
    }
    pub fn energy(&self) -> &EnergyTables {
        &self.artifact.payload.energy
    }
    pub fn algebra(&self) -> &FiniteAlgebra {
        &self.artifact.payload.algebra
    }
    pub fn token_bias(&self, token: usize) -> Result<i8> {
        if token >= self.vocab_size() {
            return Err(GenerateError::Shape("Generate token out of domain"));
        }
        let value = decode(self.artifact.payload.biases[token >> 1], token & 1 != 0);
        if value == -8 {
            return Err(GenerateError::Coefficient {
                index: token,
                value,
            });
        }
        Ok(value)
    }
    fn check_state(&self, state: &[H4Code]) -> Result<()> {
        if state.len() != self.lanes() {
            return Err(GenerateError::Shape(
                "Generate retained state lane count differs",
            ));
        }
        Ok(())
    }
    fn frame(&self, state: &[H4Code], counts: &mut GenerateReadCounts) -> Result<[u8; MAX_LANES]> {
        let mut frame = [0u8; MAX_LANES];
        for (lane, code) in state.iter().enumerate() {
            frame[lane] = self
                .algebra()
                .inverse_counted(code.index(), &mut counts.algebra)?;
        }
        Ok(frame)
    }
    fn relative_tuple(
        &self,
        inverse_state: &[u8],
        token: usize,
        override_code: Option<(usize, u8)>,
        counts: &mut GenerateReadCounts,
    ) -> Result<[u8; MAX_LANES]> {
        if token >= self.vocab_size() {
            return Err(GenerateError::Shape("Generate token out of domain"));
        }
        let mut relative = [0u8; MAX_LANES];
        for lane in 0..self.lanes() {
            let code = if let Some((l, c)) = override_code.filter(|(l, _)| *l == lane) {
                let _ = l;
                c
            } else {
                counts.prototype_reads = counts
                    .prototype_reads
                    .checked_add(1)
                    .ok_or(GeometryError::CounterOverflow)?;
                self.prototypes()[self.prototype_offsets[token] + lane]
            };
            relative[lane] =
                self.algebra()
                    .compose_counted(inverse_state[lane], code, &mut counts.algebra)?;
        }
        Ok(relative)
    }
    /// Offline logical learning-coordinate incidence, in unary-lane then declared
    /// edge order. Keys exclude bias/prototype choices and packed-table padding.
    /// Invalid shape/token/state leaves the output unchanged.
    pub fn factor_incidence_into(
        &self,
        state: &[H4Code],
        token: usize,
        out: &mut [u32],
        counts: &mut GenerateReadCounts,
    ) -> Result<()> {
        self.check_state(state)?;
        if out.len() != self.lanes() + self.energy().edges().len() {
            return Err(GenerateError::Shape(
                "Generate factor incidence shape differs",
            ));
        }
        let frame = self.frame(state, counts)?;
        let relative = self.relative_tuple(&frame[..self.lanes()], token, None, counts)?;
        let mut keys = [0u32; MAX_LANES + MAX_PAIRS];
        for lane in 0..self.lanes() {
            keys[lane] = (lane * 120 + usize::from(relative[lane])) as u32;
        }
        for (index, edge) in self.energy().edges().iter().enumerate() {
            keys[self.lanes() + index] = (self.lanes() * 120
                + index * 14_400
                + usize::from(relative[usize::from(edge.left)]) * 120
                + usize::from(relative[usize::from(edge.right)]))
                as u32;
        }
        out.copy_from_slice(&keys[..out.len()]);
        Ok(())
    }
    fn score(
        &self,
        inverse_state: &[u8],
        token: usize,
        override_code: Option<(usize, u8)>,
        counts: &mut GenerateReadCounts,
    ) -> Result<i64> {
        let relative = self.relative_tuple(inverse_state, token, override_code, counts)?;
        let z = self
            .energy()
            .score(&relative[..self.lanes()], &mut counts.energy)?
            + i32::from(self.token_bias(token)?);
        counts.bias_reads = counts
            .bias_reads
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        counts.scores = counts
            .scores
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        Ok(i64::from(z) << SCORE_SHIFT)
    }
    pub fn score_into(
        &self,
        state: &[H4Code],
        out: &mut [i64],
        counts: &mut GenerateReadCounts,
    ) -> Result<()> {
        self.check_state(state)?;
        if out.len() != self.vocab_size() {
            return Err(GenerateError::Shape(
                "Generate output vocabulary length differs",
            ));
        }
        let frame = self.frame(state, counts)?;
        for (token, z) in out.iter_mut().enumerate() {
            *z = self.score(&frame[..self.lanes()], token, None, counts)?;
        }
        Ok(())
    }
    pub fn state_conditional_scores_into(
        &self,
        state: &[H4Code],
        lane: usize,
        token: usize,
        out: &mut [i64; 120],
        counts: &mut GenerateReadCounts,
    ) -> Result<()> {
        self.check_state(state)?;
        if lane >= self.lanes() {
            return Err(GenerateError::Shape(
                "Generate conditional state lane differs",
            ));
        }
        let mut changed = self.frame(state, counts)?;
        for (code, z) in out.iter_mut().enumerate() {
            changed[lane] = self
                .algebra()
                .inverse_counted(code as u8, &mut counts.algebra)?;
            *z = self.score(&changed[..self.lanes()], token, None, counts)?;
        }
        Ok(())
    }
    pub fn code_conditional_scores_into(
        &self,
        state: &[H4Code],
        lane: usize,
        token: usize,
        out: &mut [i64; 120],
        counts: &mut GenerateReadCounts,
    ) -> Result<()> {
        self.check_state(state)?;
        if lane >= self.lanes() {
            return Err(GenerateError::Shape(
                "Generate conditional code lane differs",
            ));
        }
        let frame = self.frame(state, counts)?;
        for (code, z) in out.iter_mut().enumerate() {
            *z = self.score(
                &frame[..self.lanes()],
                token,
                Some((lane, code as u8)),
                counts,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::integrated_attention::geometry::LanePair;
    use super::*;
    const TOKENIZER:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn binding() -> Result<SourceActionBinding> {
        SourceActionBinding::new(TOKENIZER).map_err(|e| GenerateError::Artifact(e.to_string()))
    }
    fn codes(values: &[u8]) -> Result<Vec<H4Code>> {
        values
            .iter()
            .map(|&c| H4Code::try_from(c).map_err(|e| GenerateError::Artifact(e.to_string())))
            .collect()
    }
    fn model() -> Result<NativeGeometricGenerate> {
        let mut energy = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])?;
        for c in 0..120 {
            energy.set_unary(0, c, (c % 15) as i8 - 7)?;
            energy.set_unary(1, c, ((c + 3) % 15) as i8 - 7)?;
        }
        for a in 0..120 {
            for b in 0..120 {
                energy.set_pair(
                    0,
                    a,
                    b,
                    ((usize::from(a) * 5 + usize::from(b) * 7) % 15) as i8 - 7,
                )?;
            }
        }
        NativeGeometricGenerate::compile(
            &binding()?,
            2,
            &[1, 1, 0, 1, 3, 5, 5, 3, 31, 57, 31, 57],
            &[0x71, 0x20, 0x1f],
            energy,
        )
    }
    #[test]
    fn native_generate_preserves_sparse_token_id_domain() -> Result<()> {
        let mut tokenizer: serde_json::Value = serde_json::from_slice(TOKENIZER)
            .map_err(|e| GenerateError::Artifact(e.to_string()))?;
        tokenizer["added_tokens"]
            .as_array_mut()
            .ok_or(GenerateError::Shape("test tokenizer added tokens"))?
            .push(serde_json::json!({"id":8,"content":"extra"}));
        let bytes =
            serde_json::to_vec(&tokenizer).map_err(|e| GenerateError::Artifact(e.to_string()))?;
        let binding =
            SourceActionBinding::new(&bytes).map_err(|e| GenerateError::Artifact(e.to_string()))?;
        assert_eq!(binding.vocab_size(), 9);
        assert!(!binding.admits_token(6));
        assert!(binding.admits_token(8));
        let model = NativeGeometricGenerate::compile(
            &binding,
            1,
            &[1; 9],
            &[0; 5],
            EnergyTables::zeroed(1, vec![])?,
        )?;
        let restored = NativeGeometricGenerate::from_bytes(&model.to_bytes()?, &binding)?;
        assert_eq!(restored.vocab_size(), 9);
        Ok(())
    }
    #[test]
    fn native_generate_exact_frame_reload_and_directed_scores() -> Result<()> {
        let h4 = trusted()?;
        let exact = FiniteAlgebra::from_historical_h4(&h4)?;
        let old = FiniteAlgebra::new_2i()?;
        assert_eq!(exact, old);
        assert_eq!(exact.identity(), 1);
        for a in 0..120 {
            let ca = H4Code::try_from(a).map_err(|e| GenerateError::Artifact(e.to_string()))?;
            assert_eq!(exact.inverse(a)?, h4.inverse(ca).index());
            for b in 0..120 {
                let cb = H4Code::try_from(b).map_err(|e| GenerateError::Artifact(e.to_string()))?;
                assert_eq!(exact.compose(a, b)?, h4.compose(ca, cb).index());
            }
        }
        let m = model()?;
        let bytes = m.to_bytes()?;
        let restored = NativeGeometricGenerate::from_bytes(&bytes, &binding()?)?;
        assert_eq!(restored.to_bytes()?, bytes);
        let state = codes(&[3, 5])?;
        let mut before = [0i64; 6];
        let mut after = [0i64; 6];
        let mut counts = GenerateReadCounts::default();
        m.score_into(&state, &mut before, &mut counts)?;
        restored.score_into(&state, &mut after, &mut GenerateReadCounts::default())?;
        assert_eq!(before, after);
        assert_eq!(counts.algebra.inverse_reads, 2);
        assert_eq!(counts.algebra.product_reads, 12);
        assert_eq!(counts.energy.coefficients, 18);
        assert_eq!(counts.prototype_reads, 12);
        assert_eq!(counts.bias_reads, 6);
        assert_eq!(counts.scores, 6);
        for token in 0..6 {
            let mut rel = [0u8; 2];
            for lane in 0..2 {
                rel[lane] = h4
                    .relative(
                        state[lane],
                        H4Code::try_from(m.prototypes()[token * 2 + lane])
                            .map_err(|e| GenerateError::Artifact(e.to_string()))?,
                    )
                    .index();
            }
            let expected = m.energy().score(&rel, &mut EnergyReadCounts::default())?
                + i32::from(m.token_bias(token)?);
            assert_eq!(before[token], i64::from(expected) << 20);
            assert!(before[token].abs() <= 91i64 << 20);
        }
        assert_ne!(
            h4.relative(
                H4Code::try_from(3).map_err(|e| GenerateError::Artifact(e.to_string()))?,
                H4Code::try_from(5).map_err(|e| GenerateError::Artifact(e.to_string()))?
            )
            .index(),
            h4.relative(
                H4Code::try_from(5).map_err(|e| GenerateError::Artifact(e.to_string()))?,
                H4Code::try_from(3).map_err(|e| GenerateError::Artifact(e.to_string()))?
            )
            .index()
        );
        // Duplicate learned prototypes remain independent exact token entries;
        // bias changes their scores without merging their identities.
        assert_eq!(&m.prototypes()[8..10], &m.prototypes()[10..12]);
        assert_ne!(before[4], before[5]);
        Ok(())
    }
    #[test]
    fn native_generate_factor_incidence_matches_recompiled_shared_edits() -> Result<()> {
        let b = binding()?;
        // Declared edge order differs from lexicographic order.
        let edges = vec![
            LanePair { left: 1, right: 3 },
            LanePair { left: 0, right: 2 },
        ];
        let prototypes = vec![
            3, 5, 7, 9, 31, 57, 11, 13, 3, 5, 7, 9, 17, 19, 23, 29, 41, 43, 47, 53, 59, 61, 67, 71,
        ];
        let base = NativeGeometricGenerate::compile(
            &b,
            4,
            &prototypes,
            &[0; 3],
            EnergyTables::zeroed(4, edges)?,
        )?;
        let state = codes(&[31, 57, 79, 83])?;
        let mut incidence = vec![[0u32; 6]; 6];
        for (token, keys) in incidence.iter_mut().enumerate() {
            base.factor_incidence_into(&state, token, keys, &mut GenerateReadCounts::default())?;
            for lane in 0..4 {
                let relative = base.algebra().compose(
                    base.algebra().inverse(state[lane].index())?,
                    prototypes[token * 4 + lane],
                )?;
                assert_eq!(keys[lane], (lane * 120 + usize::from(relative)) as u32);
            }
        }
        assert_eq!(incidence[0], incidence[2]);
        let mut original = [0i64; 6];
        base.score_into(&state, &mut original, &mut GenerateReadCounts::default())?;
        let keys = [incidence[0][0], incidence[0][4], incidence[1][5]];
        let mut energy = base.energy().clone();
        for &key in &keys {
            if key < 480 {
                energy.set_unary((key / 120) as u8, (key % 120) as u8, 1)?;
            } else {
                let pair = key - 480;
                energy.set_pair(
                    (pair / 14_400) as usize,
                    ((pair % 14_400) / 120) as u8,
                    (pair % 120) as u8,
                    1,
                )?;
            }
        }
        let candidate = NativeGeometricGenerate::compile(&b, 4, &prototypes, &[0; 3], energy)?;
        let reloaded = NativeGeometricGenerate::from_bytes(&candidate.to_bytes()?, &b)?;
        let mut actual = [0i64; 6];
        reloaded.score_into(&state, &mut actual, &mut GenerateReadCounts::default())?;
        for token in 0..6 {
            let affected = keys
                .iter()
                .filter(|key| incidence[token].contains(key))
                .count();
            assert_eq!(
                actual[token] - original[token],
                (affected as i64) << SCORE_SHIFT
            );
        }
        let mut unchanged = [u32::MAX; 6];
        assert!(base
            .factor_incidence_into(
                &state,
                6,
                &mut unchanged,
                &mut GenerateReadCounts::default()
            )
            .is_err());
        assert_eq!(unchanged, [u32::MAX; 6]);
        assert!(base
            .factor_incidence_into(
                &state,
                0,
                &mut unchanged[..5],
                &mut GenerateReadCounts::default()
            )
            .is_err());
        assert_eq!(unchanged, [u32::MAX; 6]);
        Ok(())
    }
    #[test]
    fn native_generate_conditionals_reproduce_every_factual_replacement() -> Result<()> {
        let m = model()?;
        let state = codes(&[31, 57])?;
        for lane in 0..2 {
            for token in 0..6 {
                let mut counter = [0i64; 120];
                m.state_conditional_scores_into(
                    &state,
                    lane,
                    token,
                    &mut counter,
                    &mut GenerateReadCounts::default(),
                )?;
                let mut prototype = [0i64; 120];
                m.code_conditional_scores_into(
                    &state,
                    lane,
                    token,
                    &mut prototype,
                    &mut GenerateReadCounts::default(),
                )?;
                for code in 0..120 {
                    let mut changed = state.clone();
                    changed[lane] = H4Code::try_from(code as u8)
                        .map_err(|e| GenerateError::Artifact(e.to_string()))?;
                    let mut scores = [0i64; 6];
                    m.score_into(&changed, &mut scores, &mut GenerateReadCounts::default())?;
                    assert_eq!(counter[code], scores[token]);
                    // Same admitted shape; only this valid finite code changes
                    // in the independent factual numerical fixture. Avoid
                    // repeating cubic artifact admission for all1440 cases.
                    let mut other = m.clone();
                    other.artifact.payload.prototypes[token * 2 + lane] = code as u8;
                    other.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
                    assert_eq!(prototype[code], scores[token]);
                }
            }
        }
        Ok(())
    }
    #[test]
    fn native_generate_rejects_forged_frame_binding_and_minus8() -> Result<()> {
        let m = model()?;
        let b = binding()?;
        let mut artifact = m.artifact.clone();
        artifact.payload.algebra = FiniteAlgebra::new_c120()?;
        artifact.metadata.algebra_digest = *artifact.payload.algebra.digest();
        artifact.metadata.payload_sha256 = payload_hash(&artifact.payload)?;
        let bytes =
            serde_json::to_vec(&artifact).map_err(|e| GenerateError::Artifact(e.to_string()))?;
        assert!(NativeGeometricGenerate::from_bytes(&bytes, &b).is_err());
        let mut artifact = m.artifact.clone();
        artifact.metadata.tokenizer_sha256 = "00".repeat(32);
        let bytes =
            serde_json::to_vec(&artifact).map_err(|e| GenerateError::Artifact(e.to_string()))?;
        assert!(NativeGeometricGenerate::from_bytes(&bytes, &b).is_err());
        let mut energy = m.energy().clone();
        energy.set_unary(0, 1, -8)?;
        assert!(
            NativeGeometricGenerate::compile(&b, 2, m.prototypes(), m.packed_biases(), energy)
                .is_err()
        );
        let mut energy = m.energy().clone();
        energy.set_pair(0, 1, 1, -8)?;
        assert!(
            NativeGeometricGenerate::compile(&b, 2, m.prototypes(), m.packed_biases(), energy)
                .is_err()
        );
        assert!(NativeGeometricGenerate::compile(
            &b,
            2,
            m.prototypes(),
            &[0x08, 0, 0],
            m.energy().clone()
        )
        .is_err());
        assert!(NativeGeometricGenerate::from_bytes(&vec![0; MAX_PAYLOAD_BYTES + 1], &b).is_err());
        assert!(m
            .score_into(
                &codes(&[1])?,
                &mut [0; 6],
                &mut GenerateReadCounts::default()
            )
            .is_err());
        assert!(m
            .score_into(
                &codes(&[1, 1])?,
                &mut [0; 5],
                &mut GenerateReadCounts::default()
            )
            .is_err());
        assert!(
            NativeGeometricGenerate::compile(&b, usize::MAX, &[], &[], m.energy().clone()).is_err()
        );
        Ok(())
    }
}
