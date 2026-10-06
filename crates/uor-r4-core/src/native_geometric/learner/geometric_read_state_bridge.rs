//! Native selected-source -> retained-state bridge. Selection belongs to caller.
#![forbid(unsafe_code)]
use super::integrated_attention::geometry::{AlgebraReadCounts, FiniteAlgebra, GeometryError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    h4_tables::{H4Code, HistoricalH4Tables, TRUSTED_MATHEMATICAL_SHA256},
};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
pub const MAX_LANES: usize = 8;
pub const ROOTS: usize = 120;
pub const SCORE_SHIFT: u32 = 20;
pub const BIAS_BYTES_PER_LANE: usize = 60;
pub const RELATIVE_BYTES_PER_LANE: usize = 7680;
pub const SCHEMA: &str = "uor-r4.native-geometric-read-state-bridge/1";
pub const POLICY:&str="signed-historical-H4;d=inverse(query)*selected-source;action=bias[a]+table[a,d];strict-q4[-7,7];identity-first-zero-ties;post=query*action;shift20-Q24-action-energy;no-selected-occurrence-gradient/1";
const FRAME: &[u8] =
    include_bytes!("../../../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
#[derive(Clone, Debug)]
pub enum BridgeError {
    Shape(&'static str),
    Binding(&'static str),
    Coefficient(usize, i8),
    Geometry(GeometryError),
    Artifact(String),
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for BridgeError {}
impl From<GeometryError> for BridgeError {
    fn from(e: GeometryError) -> Self {
        Self::Geometry(e)
    }
}
pub type Result<T> = std::result::Result<T, BridgeError>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeMetadata {
    pub schema: String,
    pub policy: String,
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
    bias: Vec<u8>,
    relative: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    metadata: BridgeMetadata,
    payload: Payload,
}
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct BridgeReadCounts {
    pub algebra: AlgebraReadCounts,
    pub coefficient_reads: u64,
    pub scores: u64,
}
#[derive(Clone, Debug)]
pub struct NativeGeometricReadStateBridge {
    artifact: Artifact,
    score_len: usize,
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn algebra() -> Result<&'static FiniteAlgebra> {
    static A: std::sync::OnceLock<Result<FiniteAlgebra>> = std::sync::OnceLock::new();
    A.get_or_init(|| {
        let t = HistoricalH4Tables::from_bytes(FRAME)
            .map_err(|e| BridgeError::Artifact(e.to_string()))?;
        let a = FiniteAlgebra::from_historical_h4(&t)?;
        for c in 0..120 {
            if a.compose(1, c)? != c || a.compose(c, 1)? != c {
                return Err(BridgeError::Binding("historical identity must be code1"));
            }
        }
        Ok(a)
    })
    .as_ref()
    .map_err(Clone::clone)
}
fn coefficient(bytes: &[u8], index: usize) -> i8 {
    let n = (bytes[index >> 1] >> if index & 1 == 0 { 0 } else { 4 }) & 15;
    if n < 8 {
        n as i8
    } else {
        n as i8 - 16
    }
}
impl NativeGeometricReadStateBridge {
    pub fn compile(
        binding: &SourceActionBinding,
        lanes: usize,
        packed_bias: &[u8],
        packed_relative: &[u8],
    ) -> Result<Self> {
        if !(1..=MAX_LANES).contains(&lanes)
            || packed_bias.len() != lanes * BIAS_BYTES_PER_LANE
            || packed_relative.len() != lanes * RELATIVE_BYTES_PER_LANE
        {
            return Err(BridgeError::Shape("bridge L1..8 or packed lengths differ"));
        }
        let payload = Payload {
            bias: packed_bias.to_vec(),
            relative: packed_relative.to_vec(),
        };
        let metadata = BridgeMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            lanes,
            score_shift: SCORE_SHIFT,
            tokenizer_sha256: binding.tokenizer_sha256().into(),
            dialogue_protocol: binding.protocol().clone(),
            h4_mathematical_sha256: TRUSTED_MATHEMATICAL_SHA256
                .ok_or(BridgeError::Binding("missing H4 trust anchor"))?
                .into(),
            algebra_digest: *algebra()?.digest(),
            payload_sha256: hash(
                &serde_json::to_vec(&payload).map_err(|e| BridgeError::Artifact(e.to_string()))?,
            ),
        };
        let m = Self {
            artifact: Artifact { metadata, payload },
            score_len: lanes * ROOTS,
        };
        m.validate(binding)?;
        Ok(m)
    }
    pub fn zeroed(binding: &SourceActionBinding, lanes: usize) -> Result<Self> {
        if !(1..=MAX_LANES).contains(&lanes) {
            return Err(BridgeError::Shape("bridge lanes outside1..8"));
        }
        Self::compile(
            binding,
            lanes,
            &vec![0; lanes * BIAS_BYTES_PER_LANE],
            &vec![0; lanes * RELATIVE_BYTES_PER_LANE],
        )
    }
    fn validate(&self, b: &SourceActionBinding) -> Result<()> {
        let m = &self.artifact.metadata;
        let p = &self.artifact.payload;
        if m.schema != SCHEMA
            || m.policy != POLICY
            || m.score_shift != SCORE_SHIFT
            || m.tokenizer_sha256 != b.tokenizer_sha256()
            || m.dialogue_protocol != *b.protocol()
            || m.algebra_digest != *algebra()?.digest()
            || Some(m.h4_mathematical_sha256.as_str()) != TRUSTED_MATHEMATICAL_SHA256
        {
            return Err(BridgeError::Binding(
                "bridge format/tokenizer/H4 binding differs",
            ));
        }
        if !(1..=MAX_LANES).contains(&m.lanes)
            || p.bias.len() != m.lanes * BIAS_BYTES_PER_LANE
            || p.relative.len() != m.lanes * RELATIVE_BYTES_PER_LANE
        {
            return Err(BridgeError::Shape("bridge artifact lengths differ"));
        }
        for (i, &v) in p.bias.iter().chain(&p.relative).enumerate() {
            for odd in [false, true] {
                let n = if odd { v >> 4 } else { v & 15 };
                if n == 8 {
                    return Err(BridgeError::Coefficient(i, -8));
                }
            }
        }
        for row in p.relative.chunks_exact(64) {
            if row[60..].iter().any(|&v| v != 0) {
                return Err(BridgeError::Shape("relative padding120..128 must be zero"));
            }
        }
        if m.payload_sha256
            != hash(&serde_json::to_vec(p).map_err(|e| BridgeError::Artifact(e.to_string()))?)
        {
            return Err(BridgeError::Binding("bridge payload hash differs"));
        }
        Ok(())
    }
    pub fn from_bytes(bytes: &[u8], binding: &SourceActionBinding) -> Result<Self> {
        if bytes.len() > 512 * 1024 {
            return Err(BridgeError::Shape("bridge artifact exceeds512KiB"));
        }
        let artifact: Artifact =
            serde_json::from_slice(bytes).map_err(|e| BridgeError::Artifact(e.to_string()))?;
        let m = Self {
            score_len: artifact
                .metadata
                .lanes
                .checked_mul(ROOTS)
                .ok_or(BridgeError::Shape("score count overflow"))?,
            artifact,
        };
        m.validate(binding)?;
        Ok(m)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.artifact).map_err(|e| BridgeError::Artifact(e.to_string()))
    }
    pub fn metadata(&self) -> &BridgeMetadata {
        &self.artifact.metadata
    }
    pub fn lanes(&self) -> usize {
        self.artifact.metadata.lanes
    }
    pub fn packed_bias(&self) -> &[u8] {
        &self.artifact.payload.bias
    }
    pub fn packed_relative(&self) -> &[u8] {
        &self.artifact.payload.relative
    }
    pub fn algebra(&self) -> Result<&'static FiniteAlgebra> {
        algebra()
    }
    pub fn coefficient_bias(&self, lane: usize, action: usize) -> Result<i8> {
        if lane >= self.lanes() || action >= ROOTS {
            return Err(BridgeError::Shape("bias index outside bridge"));
        }
        Ok(coefficient(
            &self.artifact.payload.bias[lane * 60..(lane + 1) * 60],
            action,
        ))
    }
    pub fn coefficient_relative(&self, lane: usize, action: usize, d: usize) -> Result<i8> {
        if lane >= self.lanes() || action >= ROOTS || d >= ROOTS {
            return Err(BridgeError::Shape("relative index outside bridge"));
        }
        Ok(coefficient(
            &self.artifact.payload.relative[lane * 7680..(lane + 1) * 7680],
            (action << 7) + d,
        ))
    }
    /// Success path: preallocated slices, shifts/additions/tables only. One call
    /// emits L*120 scores and L poststates/actions; token selection is external.
    pub fn apply_into(
        &self,
        query: &[H4Code],
        selected_source: &[H4Code],
        post: &mut [H4Code],
        actions: &mut [H4Code],
        scores: &mut [i64],
        counts: &mut BridgeReadCounts,
    ) -> Result<()> {
        if query.len() != self.lanes()
            || selected_source.len() != self.lanes()
            || post.len() != self.lanes()
            || actions.len() != self.lanes()
            || scores.len() != self.score_len
        {
            return Err(BridgeError::Shape("bridge input/output shapes differ"));
        }
        let a = algebra()?;
        let mut score_rest = scores;
        let mut bias_rest = self.artifact.payload.bias.as_slice();
        let mut table_rest = self.artifact.payload.relative.as_slice();
        for (((&q, &k), out), chosen) in query.iter().zip(selected_source).zip(post).zip(actions) {
            let (row, rest) = score_rest.split_at_mut(ROOTS);
            score_rest = rest;
            let (bias, rest) = bias_rest.split_at(BIAS_BYTES_PER_LANE);
            bias_rest = rest;
            let (table, rest) = table_rest.split_at(RELATIVE_BYTES_PER_LANE);
            table_rest = rest;
            let inv = a.inverse_counted(q.index(), &mut counts.algebra)?;
            let d = usize::from(a.compose_counted(inv, k.index(), &mut counts.algebra)?);
            for (action, score) in row.iter_mut().enumerate() {
                *score = (i64::from(coefficient(bias, action))
                    + i64::from(coefficient(table, (action << 7) + d)))
                    << SCORE_SHIFT;
            }
            let mut best = 1usize;
            for action in 0..120 {
                if row[action] > row[best] {
                    best = action;
                }
            }
            let code =
                H4Code::try_from(best as u8).map_err(|e| BridgeError::Artifact(e.to_string()))?;
            *chosen = code;
            *out = H4Code::try_from(a.compose_counted(
                q.index(),
                code.index(),
                &mut counts.algebra,
            )?)
            .map_err(|e| BridgeError::Artifact(e.to_string()))?;
            counts.coefficient_reads = counts
                .coefficient_reads
                .checked_add(240)
                .ok_or(BridgeError::Shape("read counter overflow"))?;
            counts.scores = counts
                .scores
                .checked_add(120)
                .ok_or(BridgeError::Shape("score counter overflow"))?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    #[test]
    fn identity_roundtrip_and_shapes() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let b = SourceActionBinding::new(TOK.as_bytes())?;
        let m = NativeGeometricReadStateBridge::zeroed(&b, 2)?;
        let m = NativeGeometricReadStateBridge::from_bytes(&m.to_bytes()?, &b)?;
        let q = [H4Code::try_from(0)?, H4Code::try_from(119)?];
        let k = [H4Code::IDENTITY; 2];
        let mut p = k;
        let mut ac = k;
        let mut s = [0; 240];
        let mut c = BridgeReadCounts::default();
        m.apply_into(&q, &k, &mut p, &mut ac, &mut s, &mut c)?;
        assert_eq!(p, q);
        assert_eq!(ac, k);
        assert!(s.iter().all(|&v| v == 0));
        assert_eq!(c.coefficient_reads, 480);
        assert!(m
            .apply_into(&q[..1], &k, &mut p, &mut ac, &mut s, &mut c)
            .is_err());
        Ok(())
    }
    #[test]
    fn directed_relation_nonidentity_and_padding_rejection(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let b = SourceActionBinding::new(TOK.as_bytes())?;
        let a = algebra()?;
        let key = (0..120)
            .find(|&c| a.inverse(c).is_ok_and(|i| i != c))
            .ok_or("nonselfinverse root absent")?;
        let mut t = vec![0; 7680];
        let action = 2usize;
        let index = (action << 7) + usize::from(key);
        t[index >> 1] |= 7 << if index & 1 == 0 { 0 } else { 4 };
        let m = NativeGeometricReadStateBridge::compile(&b, 1, &[0; 60], &t)?;
        let mut p = [H4Code::IDENTITY];
        let mut ac = p;
        let mut scores = [0; 120];
        m.apply_into(
            &[H4Code::IDENTITY],
            &[H4Code::try_from(key)?],
            &mut p,
            &mut ac,
            &mut scores,
            &mut BridgeReadCounts::default(),
        )?;
        assert_eq!(ac[0].index(), 2);
        assert_eq!(p[0].index(), 2);
        m.apply_into(
            &[H4Code::try_from(key)?],
            &[H4Code::IDENTITY],
            &mut p,
            &mut ac,
            &mut scores,
            &mut BridgeReadCounts::default(),
        )?;
        assert_eq!(ac[0], H4Code::IDENTITY);
        t[63] = 1;
        assert!(NativeGeometricReadStateBridge::compile(&b, 1, &[0; 60], &t).is_err());
        assert!(
            NativeGeometricReadStateBridge::compile(&b, 1, &[0x88; 60], &vec![0; 7680]).is_err()
        );
        Ok(())
    }
}
