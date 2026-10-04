//! One finite H4 update from an actual joint-Copy K2 value, or matched causal
//! role/surface input. No Q16 mixture is projected to a root. Constructor table
//! expansion and trace reporting allocate; numerical selection uses table reads.
use crate::{
    geometric_context_q4::ContextQ4Config,
    geometric_occurrence_read::{PreparedOccurrenceContext, QuerySnapshot},
    geometric_source_actions::{ActionTrace, SourceAction},
    geometric_source_realizer::{
        NativeArtifactBinding, ObservedCode, OccurrenceIdentity, SourceRuntimeError,
        SourceRuntimeResult as Result,
    },
    geometric_value::{ValuePacket, ValueState},
    geometric_value_producer::{NativeValueProducer, ProducedValues},
    geometric_value_q4::{self, NativeValueQ4, ValueQ4Config},
    h4_tables::{H4Code, ROOT_COUNT},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

pub const SCHEMA: &str = "uor-r4.geometric-read-feedback/1";
pub const POLICY: &str = "fixed-two-pre-emission-reads;highest-joint-Copy-score-smallest-offset;terminal-mass-retained;K2-atoms-no-projection;right-H4-update;identity-first1-0-2..119;shared-controller-snapshot;q4[-7,7]-quarter/1";
pub const COEFFICIENTS_PER_ACTION: usize = 83;
const ALGEBRA: &[u8] = include_bytes!("../fixtures/historical-h4-tables-v1.bin");
fn error(message: impl Into<String>) -> SourceRuntimeError {
    SourceRuntimeError::Invalid(message.into())
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn digest_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackInputMode {
    JointCopy,
    RoleSurface,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackMetadata {
    pub schema: String,
    pub policy: String,
    pub parent_artifact: NativeArtifactBinding,
    pub context: ContextQ4Config,
    pub value: ValueQ4Config,
    pub algebra_sha256: String,
    pub value_payload_sha256: String,
    pub bridge_payload_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QuerySnapshotReport {
    pub frame_binding_sha256: String,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub last_token_id: u32,
    pub states: Vec<u8>,
    pub raw_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<ObservedCode>,
}
impl From<&QuerySnapshot<'_, '_>> for QuerySnapshotReport {
    fn from(s: &QuerySnapshot<'_, '_>) -> Self {
        Self {
            frame_binding_sha256: hex::encode(s.frame_binding()),
            heads: s.heads(),
            lanes_per_head: s.lanes_per_head(),
            last_token_id: s.last_token(),
            states: s.states().iter().map(|r| r.index()).collect(),
            raw_roots: s.raw_roots().iter().map(|r| r.index()).collect(),
            categories: s.categories().to_vec(),
            codes: s.codes().iter().copied().map(ObservedCode::from).collect(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeedbackAtom {
    pub state: &'static str,
    pub root: u8,
    pub radius_bin: u8,
}
impl From<ValuePacket> for FeedbackAtom {
    fn from(p: ValuePacket) -> Self {
        Self {
            state: match p.state() {
                ValueState::Absent => "Absent",
                ValueState::PresentZero => "PresentZero",
                ValueState::PresentNonzero => "PresentNonzero",
            },
            root: p.root().index(),
            radius_bin: p.radius_bin(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeedbackValueReport {
    pub heads: usize,
    pub occurrence_valid: bool,
    pub packets: Vec<Vec<FeedbackAtom>>,
    /// Diagnostic decoder sum only; never converted to one H4 state/action.
    pub summed_coordinates_q16: Vec<i32>,
    pub raw_root_choices: Vec<u8>,
    pub categories: Vec<u8>,
    pub coefficient_reads: usize,
}
impl From<&ProducedValues> for FeedbackValueReport {
    fn from(v: &ProducedValues) -> Self {
        let lanes = v.heads << 2;
        Self {
            heads: v.heads,
            occurrence_valid: v.occurrence_valid,
            packets: v.packets[..lanes]
                .iter()
                .map(|p| p.iter().copied().map(FeedbackAtom::from).collect())
                .collect(),
            summed_coordinates_q16: v.values_q16[..v.heads << 4].to_vec(),
            raw_root_choices: v.root_choices[..lanes << 1].to_vec(),
            categories: v.categories[..lanes << 1].to_vec(),
            coefficient_reads: v.coefficient_reads,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeedbackTrace {
    pub policy: &'static str,
    pub input_mode: FeedbackInputMode,
    pub selected_occurrence: Option<OccurrenceIdentity>,
    pub selected_joint_copy_score_q24: Option<i64>,
    pub selected_joint_copy_weight_q31: Option<u64>,
    pub provisional_period_weight_q31: u64,
    pub provisional_stop_weight_q31: u64,
    pub provisional_total_weight_q31: u64,
    pub provisional_emitted_token_id: u32,
    pub value: FeedbackValueReport,
    pub actions: Vec<u8>,
    pub before: QuerySnapshotReport,
    pub after: QuerySnapshotReport,
    pub bridge_table_reads: usize,
    pub observation_coefficient_reads: usize,
}

/// Expanded [lane, padded120 classes][1024] rows have power-of-two indexing.
/// Per action: bias1, own4, neighbor4, atom0root4/category33,
/// atom1root4/category33. Packet categories:0Absent,1PresentZero,2+radius.
#[derive(Clone, Copy)]
struct BridgeLaneLayout {
    row_base: usize,
    neighbor: usize,
    value_lane: usize,
    _padding: usize,
}
pub struct NativeReadFeedback {
    metadata: FeedbackMetadata,
    value_packed: Vec<u8>,
    bridge_packed: Vec<u8>,
    producer: NativeValueProducer,
    rows: Vec<[i32; 1024]>,
    layouts: Vec<BridgeLaneLayout>,
}
impl NativeReadFeedback {
    pub fn value_config(context: ContextQ4Config) -> ValueQ4Config {
        ValueQ4Config {
            vocab_size: context.vocab_size,
            heads: context.heads,
            latent_lanes_per_head: context.lanes_per_head,
        }
    }
    pub fn bridge_coefficient_count(context: ContextQ4Config) -> Result<usize> {
        context.validate().map_err(|e| error(e.to_string()))?;
        Ok(context.heads * context.lanes_per_head * ROOT_COUNT * COEFFICIENTS_PER_ACTION)
    }
    pub fn compile(
        parent_artifact: NativeArtifactBinding,
        context: ContextQ4Config,
        value_packed: &[u8],
        bridge_packed: &[u8],
    ) -> Result<Self> {
        let metadata = FeedbackMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            parent_artifact,
            context,
            value: Self::value_config(context),
            algebra_sha256: sha(ALGEBRA),
            value_payload_sha256: sha(value_packed),
            bridge_payload_sha256: sha(bridge_packed),
        };
        Self::new(metadata, value_packed, bridge_packed)
    }
    pub fn new(
        metadata: FeedbackMetadata,
        value_packed: &[u8],
        bridge_packed: &[u8],
    ) -> Result<Self> {
        let p = &metadata.parent_artifact;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.algebra_sha256 != sha(ALGEBRA)
            || metadata.value != Self::value_config(metadata.context)
            || metadata.value_payload_sha256 != sha(value_packed)
            || metadata.bridge_payload_sha256 != sha(bridge_packed)
            || [
                &p.metadata_sha256,
                &p.identity.tokenizer_sha256,
                &p.identity.parent_checkpoint_manifest_sha256,
                &p.identity.parent_model_sha256,
                &p.identity.parent_config_sha256,
            ]
            .iter()
            .any(|s| !digest_valid(s))
            || H4Code::IDENTITY.index() != 1
        {
            return Err(error(
                "feedback schema/config/algebra/payload/parent binding differs",
            ));
        }
        let coefficients = geometric_value_q4::unpack_coefficients(
            Self::bridge_coefficient_count(metadata.context)?,
            bridge_packed,
        )
        .map_err(|e| error(e.to_string()))?;
        let producer = NativeValueQ4::new(metadata.value, value_packed)
            .map_err(|e| error(e.to_string()))?
            .into_native()
            .map_err(|e| error(e.to_string()))?;
        let lanes = metadata.context.heads * metadata.context.lanes_per_head;
        let mut rows = vec![[0; 1024]; lanes << 7];
        let basis = geometric_value_q4::canonical_basis_q25();
        for lane in 0..lanes {
            for class in 0..ROOT_COUNT {
                let at = (lane * ROOT_COUNT + class) * COEFFICIENTS_PER_ACTION;
                let q = &coefficients[at..at + COEFFICIENTS_PER_ACTION];
                let row = &mut rows[(lane << 7) + class];
                row[768] = i32::from(q[0]) << 22;
                for (offset, coefficients) in [(0, 1), (128, 5), (256, 9), (512, 46)] {
                    for (root, coordinates) in basis.iter().enumerate() {
                        row[offset + root] =
                            basis_score(&q[coefficients..coefficients + 4], coordinates);
                    }
                }
                for c in 0..33 {
                    row[384 + c] = i32::from(q[13 + c]) << 22;
                    row[640 + c] = i32::from(q[50 + c]) << 22;
                }
            }
        }
        let layouts = (0..lanes)
            .map(|lane| {
                let head = lane / metadata.context.lanes_per_head;
                let local = lane % metadata.context.lanes_per_head;
                BridgeLaneLayout {
                    row_base: lane << 7,
                    neighbor: head * metadata.context.lanes_per_head
                        + (local + 1) % metadata.context.lanes_per_head,
                    value_lane: (head << 2) + local,
                    _padding: 0,
                }
            })
            .collect();
        Ok(Self {
            metadata,
            value_packed: value_packed.to_vec(),
            bridge_packed: bridge_packed.to_vec(),
            producer,
            rows,
            layouts,
        })
    }
    pub fn metadata(&self) -> &FeedbackMetadata {
        &self.metadata
    }
    pub fn value_packed(&self) -> &[u8] {
        &self.value_packed
    }
    pub fn bridge_packed(&self) -> &[u8] {
        &self.bridge_packed
    }
    /// Logical payload costs, not peak RSS, allocation counts or energy.
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"value_packed_bytes":self.value_packed.len(),"bridge_packed_bytes":self.bridge_packed.len(),
            "value_expanded_table_bytes":self.producer.stats().stored_bytes,
            "bridge_expanded_table_bytes":self.rows.len()*std::mem::size_of::<[i32;1024]>(),
            "bridge_layout_bytes":self.layouts.len()*std::mem::size_of::<BridgeLaneLayout>(),
            "bridge_actions_per_lane":ROOT_COUNT,"bridge_coefficients_per_action":COEFFICIENTS_PER_ACTION,
            "scope":"bounded prototype; dense finite-choice tables and producer token access retained; excludes codec/metadata/allocator/trace overhead"})
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        fs::write(
            path.join("metadata.json"),
            serde_json::to_vec_pretty(&self.metadata)?,
        )?;
        fs::write(path.join("value-q4.bin"), &self.value_packed)?;
        fs::write(path.join("bridge-q4.bin"), &self.bridge_packed)?;
        Ok(())
    }
    /// Caller-supplied metadata digest, never derived from the candidate folder.
    pub fn load(path: &Path, expected_metadata_sha256: &str) -> Result<Self> {
        if !digest_valid(expected_metadata_sha256) {
            return Err(error("invalid feedback trusted digest"));
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(error("feedback inventory contains nonregular file"));
            }
            names.push(entry.file_name());
        }
        names.sort();
        if names != ["bridge-q4.bin", "metadata.json", "value-q4.bin"].map(std::ffi::OsString::from)
        {
            return Err(error("feedback inventory differs"));
        }
        let metadata_bytes = read_capped(&path.join("metadata.json"), 16384)?;
        if sha(&metadata_bytes) != expected_metadata_sha256 {
            return Err(error("feedback trusted metadata differs"));
        }
        let metadata: FeedbackMetadata = serde_json::from_slice(&metadata_bytes)?;
        let value_len = metadata
            .value
            .coefficient_count()
            .map_err(|e| error(e.to_string()))?
            .div_ceil(2);
        let bridge_len = Self::bridge_coefficient_count(metadata.context)?.div_ceil(2);
        let value = read_capped(&path.join("value-q4.bin"), value_len)?;
        let bridge = read_capped(&path.join("bridge-q4.bin"), bridge_len)?;
        Self::new(metadata, &value, &bridge)
    }
    pub(crate) fn apply<'p, 'a>(
        &self,
        parent: &NativeArtifactBinding,
        prepared: &'p PreparedOccurrenceContext<'a>,
        original: &QuerySnapshot<'p, 'a>,
        stage1: &ActionTrace,
        mode: FeedbackInputMode,
    ) -> Result<(QuerySnapshot<'p, 'a>, FeedbackTrace)> {
        if parent != &self.metadata.parent_artifact
            || original.heads() != self.metadata.context.heads
            || original.lanes_per_head() != self.metadata.context.lanes_per_head
            || stage1.tokenizer_sha256 != parent.identity.tokenizer_sha256
        {
            return Err(error("feedback parent/snapshot binding differs"));
        }
        let mut selected = None;
        let mut period = 0;
        let mut stop = 0;
        for action in &stage1.actions {
            match action.action {
                SourceAction::Copy { source_offset } => {
                    if selected
                        .map(|(_, score, _)| action.score_q24 > score)
                        .unwrap_or(true)
                    {
                        selected = Some((source_offset, action.score_q24, action.weight_q31));
                    }
                }
                SourceAction::Period => period = action.weight_q31,
                SourceAction::Stop => stop = action.weight_q31,
            }
        }
        let occurrence = match selected {
            Some((i, _, _)) => {
                let o = prepared
                    .source_occurrences()
                    .get(i)
                    .ok_or_else(|| error("provisional Copy offset differs from prepared source"))?;
                Some(OccurrenceIdentity {
                    record: o.source.record,
                    commit: o.source.commit,
                    token_offset: o.token_offset,
                    token_id: o.token_id,
                })
            }
            None => None,
        };
        let (token, states, valid) = if mode == FeedbackInputMode::JointCopy {
            match &occurrence {
                Some(o) => (
                    o.token_id,
                    prepared
                        .source_states(o.token_offset as usize)
                        .ok_or_else(|| error("provisional source latent missing"))?,
                    true,
                ),
                None => (original.last_token(), original.states(), false),
            }
        } else {
            (original.last_token(), original.states(), true)
        };
        let values = self
            .producer
            .produce(token as usize, states, None, valid)
            .map_err(|e| error(e.to_string()))?;
        let mut actions = Vec::with_capacity(original.states().len());
        let mut reads = 0;
        for lane in 0..original.states().len() {
            let layout = self.layouts[lane];
            let packets = values.packets[layout.value_lane];
            let score = |class: usize| -> i64 {
                let row = &self.rows[layout.row_base + class];
                let mut sum = i64::from(row[768])
                    + i64::from(row[original.states()[lane].index() as usize])
                    + i64::from(row[128 + original.states()[layout.neighbor].index() as usize]);
                for (atom, packet) in packets.iter().enumerate() {
                    let (root_base, category_base) =
                        if atom == 0 { (256, 384) } else { (512, 640) };
                    let category = match packet.state() {
                        ValueState::Absent => 0,
                        ValueState::PresentZero => 1,
                        ValueState::PresentNonzero => 2 + usize::from(packet.radius_bin()),
                    };
                    sum += i64::from(row[category_base + category]);
                    if packet.state() == ValueState::PresentNonzero {
                        sum += i64::from(row[root_base + packet.root().index() as usize]);
                    }
                }
                sum
            };
            let mut best = 1;
            let mut best_score = score(1);
            for class in (0..ROOT_COUNT).filter(|c| *c != 1) {
                let candidate = score(class);
                if candidate > best_score {
                    best = class;
                    best_score = candidate;
                }
            }
            reads += ROOT_COUNT
                * (5 + packets
                    .iter()
                    .filter(|p| p.state() == ValueState::PresentNonzero)
                    .count());
            actions.push(H4Code::try_from(best as u8).map_err(|e| error(e.to_string()))?);
        }
        let updated = prepared
            .apply_actions(original, &actions)
            .map_err(|e| error(e.to_string()))?;
        let trace = FeedbackTrace {
            policy: POLICY,
            input_mode: mode,
            selected_occurrence: occurrence,
            selected_joint_copy_score_q24: selected.map(|s| s.1),
            selected_joint_copy_weight_q31: selected.map(|s| s.2),
            provisional_period_weight_q31: period,
            provisional_stop_weight_q31: stop,
            provisional_total_weight_q31: stage1.total_weight_q31,
            provisional_emitted_token_id: stage1.chosen_token_id,
            value: FeedbackValueReport::from(&values),
            actions: actions.iter().map(|a| a.index()).collect(),
            before: QuerySnapshotReport::from(original),
            after: QuerySnapshotReport::from(&updated),
            bridge_table_reads: reads,
            observation_coefficient_reads: prepared.observation_reads(),
        };
        Ok((updated, trace))
    }
}
fn read_capped(path: &Path, cap: usize) -> Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > cap as u64 {
        return Err(error("feedback file cap exceeded"));
    }
    let mut bytes = Vec::new();
    file.take(cap as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > cap {
        return Err(error("feedback file grew beyond cap"));
    }
    Ok(bytes)
}
fn basis_score(q: &[i8], coordinates: &[i32; 4]) -> i32 {
    let mut sum = 0i64;
    for (&q, &x) in q.iter().zip(coordinates) {
        let mut magnitude = q.unsigned_abs();
        let mut shifted = i64::from(x);
        let mut product = 0;
        while magnitude != 0 {
            if magnitude & 1 != 0 {
                product += shifted;
            }
            magnitude >>= 1;
            shifted <<= 1;
        }
        sum += if q < 0 { -product } else { product };
    }
    if sum < 0 {
        -(((-sum + 4) >> 3) as i32)
    } else {
        ((sum + 4) >> 3) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometric_context_q4::{self, NativeContextQ4},
        geometric_no_read::{self, NativeGeometricNoRead, NoReadConfig},
        geometric_occurrence_read::{
            FrameMetadata, FrameStatus, NativeOccurrenceReader, OccurrenceComponents,
            SelectedRecordFrame, SourceIdentity,
        },
        geometric_potential::AddressLane,
        geometric_potential_q4::{self, NativePotentialQ4, PotentialQ4Config},
        geometric_read::{EXP_TABLE_LEN, WEIGHT_ONE},
        geometric_source_actions::{ActionHeadScores, NativeSourceActions, SourceActionBinding},
        geometric_source_emission_view::SourceEmissionCompiler,
        geometric_source_realizer::{ArtifactIdentity, RealizerExecution},
        h4_tables::HistoricalH4Tables,
    };
    const TOKENIZER: &[u8] = br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"\u0120":6,"\u0120a":7,"\u0120b":8},"merges":["\u0120 a","\u0120 b"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    struct Fixture {
        context: NativeContextQ4,
        potential: crate::geometric_potential::NativePotentialTables,
        stop: NativeGeometricNoRead,
        period: NativeGeometricNoRead,
        geometry: HistoricalH4Tables,
        exp: Vec<u32>,
        binding: SourceActionBinding,
        parent: NativeArtifactBinding,
        compiler: SourceEmissionCompiler,
    }
    impl Fixture {
        fn new() -> Result<Self> {
            let c = ContextQ4Config {
                vocab_size: 9,
                heads: 1,
                lanes_per_head: 1,
            };
            let mut context_q = Vec::new();
            for (name, shape) in c.coefficient_shapes().map_err(|e| error(e.to_string()))? {
                let mut q = vec![0; shape.iter().product::<usize>()];
                if name == "self_root" {
                    q[0] = -7;
                    q[4] = 7;
                }
                if name == "token_category" {
                    for row in q.chunks_exact_mut(33) {
                        row[1] = 7;
                    }
                }
                context_q.extend(q);
            }
            let context = NativeContextQ4::new(
                c,
                &geometric_context_q4::pack_coefficients(&context_q)
                    .map_err(|e| error(e.to_string()))?,
            )
            .map_err(|e| error(e.to_string()))?;
            let pc = PotentialQ4Config {
                heads: 1,
                lanes_per_head: 1,
            };
            let mut pq = vec![0; pc.coefficient_count().map_err(|e| error(e.to_string()))?];
            pq[4] = 4; // context angular axis0 after four inactive content coefficients
            let potential = NativePotentialQ4::new(
                pc,
                &geometric_potential_q4::pack_coefficients(&pq)
                    .map_err(|e| error(e.to_string()))?,
            )
            .map_err(|e| error(e.to_string()))?
            .into_native()
            .map_err(|e| error(e.to_string()))?;
            let nc = NoReadConfig {
                vocabulary: 9,
                heads: 1,
                latent_lanes_per_head: 1,
            };
            let make_controller = |bias: i8| -> Result<NativeGeometricNoRead> {
                let mut q = vec![0; nc.coefficient_count()];
                q[0] = bias;
                q[1 + 4] = bias;
                q[1 + nc.vocabulary] = 7;
                NativeGeometricNoRead::new(
                    nc,
                    &geometric_no_read::pack_coefficients(&q).map_err(|e| error(e.to_string()))?,
                )
                .map_err(|e| error(e.to_string()))
            };
            let binding = SourceActionBinding::new(TOKENIZER)?;
            let parent = NativeArtifactBinding {
                metadata_sha256: "0".repeat(64),
                identity: ArtifactIdentity {
                    tokenizer_sha256: binding.tokenizer_sha256().into(),
                    parent_checkpoint_manifest_sha256: "1".repeat(64),
                    parent_model_sha256: "2".repeat(64),
                    parent_config_sha256: "3".repeat(64),
                },
            };
            let mut exp = vec![0; EXP_TABLE_LEN];
            exp[0] = WEIGHT_ONE as u32;
            Ok(Self {
                context,
                potential,
                stop: make_controller(7)?,
                period: make_controller(-7)?,
                geometry: HistoricalH4Tables::from_bytes(ALGEBRA)
                    .map_err(|e| error(e.to_string()))?,
                exp,
                binding,
                parent,
                compiler: SourceEmissionCompiler::new(TOKENIZER)?,
            })
        }
        fn execution(&self) -> RealizerExecution<'_> {
            RealizerExecution {
                context: &self.context,
                potential_tables: &self.potential,
                no_read: &self.stop,
                geometry: &self.geometry,
                exp: &self.exp,
                period: &self.period,
                binding: &self.binding,
            }
        }
        fn reader(&self) -> Result<NativeOccurrenceReader<'_>> {
            NativeOccurrenceReader::new(OccurrenceComponents {
                context: self.context.native(),
                potential: &self.potential,
                no_read: &self.stop,
                geometry: &self.geometry,
                exp_q31: &self.exp,
            })
            .map_err(|e| error(e.to_string()))
        }
        fn feedback(&self, audible: bool, cancel: bool) -> Result<NativeReadFeedback> {
            let c = self.context.config();
            let vc = NativeReadFeedback::value_config(c);
            let mut value = Vec::new();
            for (name, shape) in vc.coefficient_shapes().map_err(|e| error(e.to_string()))? {
                let mut q = vec![0; shape.iter().product::<usize>()];
                if name == "own_root" {
                    for (slot, row) in q.chunks_exact_mut(ROOT_COUNT * 4).enumerate() {
                        let sign = if cancel && slot & 1 == 1 { -1 } else { 1 };
                        row[0] = -7 * sign;
                        row[4] = 7 * sign;
                    }
                }
                if name == "token_category" {
                    for row in q.chunks_exact_mut(32) {
                        row[1] = 7;
                    }
                }
                value.extend(q);
            }
            let mut bridge = vec![0; NativeReadFeedback::bridge_coefficient_count(c)?];
            if audible {
                bridge[9] = -7;
                bridge[COEFFICIENTS_PER_ACTION + 9] = 7;
            }
            NativeReadFeedback::compile(
                self.parent.clone(),
                c,
                &geometric_value_q4::pack_coefficients(&value).map_err(|e| error(e.to_string()))?,
                &geometric_value_q4::pack_coefficients(&bridge)
                    .map_err(|e| error(e.to_string()))?,
            )
        }
    }
    fn frame(ids: &[u32]) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: 17,
                commit: 9,
            },
            metadata: FrameMetadata {
                scope: b"fixture",
                entity: &[4],
                relation: 7,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: ids,
        }
    }

    #[test]
    fn dependent_identity_both_input_modes_preserve_complete_legacy_trace() -> Result<()> {
        let f = Fixture::new()?;
        let v = f.compiler.compile(&[4, 5])?;
        assert_eq!(v.emitted_token_ids(), &[7, 5]);
        let baseline = f.execution().read(frame(&[4, 5]), &v, &[4], &[])?;
        let feedback = f.feedback(false, false)?;
        for mode in [FeedbackInputMode::JointCopy, FeedbackInputMode::RoleSurface] {
            let r = f.execution().read_dependent(
                frame(&[4, 5]),
                &v,
                &[4],
                &[],
                &f.parent,
                &feedback,
                mode,
            )?;
            assert_eq!(r.stage1, baseline);
            assert_eq!(r.stage2, baseline);
            assert_eq!(r.feedback.actions, vec![H4Code::IDENTITY.index()]);
            assert_eq!(r.feedback.before, r.stage2_controller_snapshot);
            assert_eq!(r.original_context_replays, 1);
            assert_eq!(r.scoring_stages, 2);
            assert!(r.logical_prepared_bytes > 0);
        }
        Ok(())
    }
    #[test]
    fn dependent_copy_feedback_under_stop_changes_one_shared_controller_snapshot() -> Result<()> {
        let f = Fixture::new()?;
        let v = f.compiler.compile(&[4, 5])?;
        assert_eq!(v.emitted_token_ids(), &[7, 5]);
        let r = f.execution().read_dependent(
            frame(&[4, 5]),
            &v,
            &[4],
            &[],
            &f.parent,
            &f.feedback(true, false)?,
            FeedbackInputMode::JointCopy,
        )?;
        assert_eq!(r.stage1.actions.chosen_token_id, f.binding.eos_token_id());
        let o = r
            .feedback
            .selected_occurrence
            .as_ref()
            .ok_or_else(|| error("selected occurrence missing"))?;
        assert_eq!(
            (o.record, o.commit, o.token_offset, o.token_id),
            (17, 9, 0, 7)
        );
        assert_ne!(r.feedback.before.states, r.feedback.after.states);
        assert_eq!(r.stage2_controller_snapshot, r.feedback.after);
        assert_eq!(r.stage1.period_context, r.stage2.period_context);
        assert_eq!(r.stage1.source.emission_view, r.stage2.source.emission_view);
        assert_eq!(
            r.stage1.source.view_kernel_trace.occurrences,
            r.stage2.source.view_kernel_trace.occurrences
        );
        assert_ne!(
            r.stage1.actions.head_scores[0].copy_q24,
            r.stage2.actions.head_scores[0].copy_q24
        );
        assert_ne!(r.stage1.period_q24, r.stage2.period_q24);
        assert_ne!(
            r.stage1.actions.head_scores[0].stop_q24,
            r.stage2.actions.head_scores[0].stop_q24
        );
        let s = &r.stage2_controller_snapshot;
        let states = s
            .states
            .iter()
            .map(|r| H4Code::try_from(*r).map_err(|e| error(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let codes = s
            .codes
            .iter()
            .map(|r| {
                AddressLane::new(r.root, r.radius_bin, r.present).map_err(|e| error(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(
            f.period
                .score(s.last_token_id as usize, &states, &codes, None)
                .map_err(|e| error(e.to_string()))?[0],
            r.stage2.period_q24[0]
        );
        assert_eq!(
            f.stop
                .score(s.last_token_id as usize, &states, &codes, None)
                .map_err(|e| error(e.to_string()))?[0],
            r.stage2.actions.head_scores[0].stop_q24
        );
        Ok(())
    }
    #[test]
    fn actual_joint_occurrence_geometry_changes_action_with_fixed_query_and_keys() -> Result<()> {
        let f = Fixture::new()?;
        let reader = f.reader()?;
        let prepared = reader
            .prepare(frame(&[4, 5]), &[4], &[])
            .map_err(|e| error(e.to_string()))?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| error(e.to_string()))?;
        let feedback = f.feedback(true, false)?;
        let mut actions = NativeSourceActions::new(f.binding.clone(), 1, &f.exp)?;
        fn make(copy: &[i64]) -> ActionHeadScores<'_> {
            ActionHeadScores {
                copy_q24: copy,
                period_q24: 0,
                stop_q24: 10 << 24,
            }
        }
        let left = actions.reduce(&[4, 5], &[make(&[1 << 24, 0])])?;
        let right = actions.reduce(&[4, 5], &[make(&[0, 1 << 24])])?;
        let (a, ta) = feedback.apply(
            &f.parent,
            &prepared,
            &snapshot,
            &left,
            FeedbackInputMode::JointCopy,
        )?;
        let (b, tb) = feedback.apply(
            &f.parent,
            &prepared,
            &snapshot,
            &right,
            FeedbackInputMode::JointCopy,
        )?;
        assert_eq!(left.chosen_token_id, f.binding.eos_token_id());
        assert_eq!(right.chosen_token_id, f.binding.eos_token_id());
        assert_eq!(ta.before, tb.before);
        assert_ne!(ta.actions, tb.actions);
        assert_ne!(a.states(), b.states());
        assert_ne!(ta.value.packets, tb.value.packets);
        assert_eq!(prepared.source_states(0).map(|s| s[0].index()), Some(0));
        assert_eq!(prepared.source_states(1).map(|s| s[0].index()), Some(1));
        let tied = actions.reduce(&[4, 4], &[make(&[0, 0])])?;
        let (_, t) = feedback.apply(
            &f.parent,
            &prepared,
            &snapshot,
            &tied,
            FeedbackInputMode::JointCopy,
        )?;
        assert_eq!(
            t.selected_occurrence.as_ref().map(|o| o.token_offset),
            Some(0)
        );
        Ok(())
    }
    #[test]
    fn cancelled_present_k2_atoms_are_not_absence_or_projected_state() -> Result<()> {
        let f = Fixture::new()?;
        let v = f.compiler.compile(&[4, 5])?;
        assert_eq!(v.emitted_token_ids(), &[7, 5]);
        let r = f.execution().read_dependent(
            frame(&[4, 5]),
            &v,
            &[4],
            &[],
            &f.parent,
            &f.feedback(true, true)?,
            FeedbackInputMode::JointCopy,
        )?;
        assert!(r.feedback.value.occurrence_valid);
        assert!(r
            .feedback
            .value
            .summed_coordinates_q16
            .iter()
            .all(|v| *v == 0));
        assert_eq!(r.feedback.value.packets[0][0].state, "PresentNonzero");
        assert_eq!(r.feedback.value.packets[0][1].state, "PresentNonzero");
        assert_ne!(
            r.feedback.value.packets[0][0].root,
            r.feedback.value.packets[0][1].root
        );
        assert_eq!(r.feedback.actions, vec![0]);
        // Empty selected views are invalid; exercise the reader's supported
        // empty-source absence boundary without constructing an unchecked view.
        let reader = f.reader()?;
        let empty = reader
            .prepare(frame(&[]), &[4], &[])
            .map_err(|e| error(e.to_string()))?;
        let snapshot = empty
            .original_snapshot()
            .map_err(|e| error(e.to_string()))?;
        let mut actions = NativeSourceActions::new(f.binding.clone(), 1, &f.exp)?;
        let terminal = actions.reduce(
            &[],
            &[ActionHeadScores {
                copy_q24: &[],
                period_q24: 0,
                stop_q24: 10 << 24,
            }],
        )?;
        let (_, absent) = f.feedback(true, false)?.apply(
            &f.parent,
            &empty,
            &snapshot,
            &terminal,
            FeedbackInputMode::JointCopy,
        )?;
        assert!(!absent.value.occurrence_valid);
        assert!(absent.selected_occurrence.is_none());
        assert_eq!(absent.value.packets[0][0].state, "Absent");
        let zero = NativeReadFeedback::compile(
            f.parent.clone(),
            f.context.config(),
            &vec![
                0;
                NativeReadFeedback::value_config(f.context.config())
                    .coefficient_count()
                    .map_err(|e| error(e.to_string()))?
                    .div_ceil(2)
            ],
            f.feedback(false, false)?.bridge_packed(),
        )?;
        let r = f.execution().read_dependent(
            frame(&[4, 5]),
            &v,
            &[4],
            &[],
            &f.parent,
            &zero,
            FeedbackInputMode::JointCopy,
        )?;
        assert!(r.feedback.value.occurrence_valid);
        assert_eq!(r.feedback.value.packets[0][0].state, "PresentZero");
        Ok(())
    }
    #[test]
    fn feedback_artifact_roundtrip_rejects_bad_binding_inventory_and_q4() -> Result<()> {
        let f = Fixture::new()?;
        let feedback = f.feedback(true, false)?;
        let root =
            std::env::temp_dir().join(format!("uor-dependent-feedback-{}", std::process::id()));
        fs::create_dir(&root)?;
        let result = (|| -> Result<()> {
            let path = root.join("artifact");
            feedback.save(&path)?;
            let digest = sha(&fs::read(path.join("metadata.json"))?);
            let loaded = NativeReadFeedback::load(&path, &digest)?;
            assert_eq!(loaded.metadata(), feedback.metadata());
            assert_eq!(loaded.bridge_packed(), feedback.bridge_packed());
            assert!(NativeReadFeedback::load(&path, &"0".repeat(64)).is_err());
            fs::write(path.join("extra"), b"unexpected")?;
            assert!(NativeReadFeedback::load(&path, &digest).is_err());
            let mut metadata = feedback.metadata().clone();
            metadata.policy = "numeric-first-tie".into();
            assert!(NativeReadFeedback::new(
                metadata,
                feedback.value_packed(),
                feedback.bridge_packed()
            )
            .is_err());
            let mut bridge = feedback.bridge_packed().to_vec();
            bridge[0] = 8;
            assert!(NativeReadFeedback::compile(
                f.parent.clone(),
                f.context.config(),
                feedback.value_packed(),
                &bridge
            )
            .is_err());
            let v = f.compiler.compile(&[4])?;
            let mut parent = f.parent.clone();
            parent.metadata_sha256 = "4".repeat(64);
            assert!(f
                .execution()
                .read_dependent(
                    frame(&[4]),
                    &v,
                    &[4],
                    &[],
                    &parent,
                    &feedback,
                    FeedbackInputMode::JointCopy
                )
                .is_err());
            Ok(())
        })();
        fs::remove_dir_all(&root)?;
        result
    }
}
