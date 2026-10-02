//! Source-bound offline compiler for the finite learned K2 value producer.
//!
//! Compilation expands each four-coordinate own/neighbor/span factor over
//! the 120 signed canonical roots. It is not a joint-state lookup: token,
//! own, next neighbor, held span and validity remain separate factors. Native
//! execution chooses two packets per lane using integer Q24 scores and the
//! checked fixed-basis Q16 pair decoder. No donor vectors or labels enter it.
//!
//! Canonical state coordinates are cast to F32 as in the actual ContextOp
//! output, then each factor dot is accumulated in F64 and rounded once to Q24
//! nearest/ties-away. The training head uses F32 factor operations, so this is
//! a declared numerical lowering, not a guarantee of identical hard choices
//! near ties. No further fit or hidden-state approximation occurs here.
//!
//! A saved head and finite-choice context, their actual dependency files and
//! tokenizer are independently admitted before compilation. Reload regenerates
//! every table and compares both bytes and metadata. This component replaces
//! value production only; NoRead, reader output and the surrounding trunk have
//! their separately declared boundaries.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::Var;
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_integer::geometric_value::RUNTIME_COORDINATE_BYTES;
use uor_r4_integer::geometric_value_producer::{
    NativeValueProducer, ProducedValues, ValueProducerTableSlices,
};
use uor_r4_integer::h4_tables::H4Code;

use crate::geometric_context::{
    BoundFile, CompiledContext, CompiledContextMetadata, ContextSourceMetadata, ContextSourcePaths,
    ContextWeights, FINITE_CHOICE_SURROGATE,
};
use crate::geometric_stack::{GEOMETRIC_ADDRESS_RECORD, GEOMETRIC_SPAN_RECORD};
use crate::geometric_value_producer::{ValueProducerConfig, ValueProducerWeights};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-value-producer-native/1";
const POLICY: &str = "canonical-state-components-cast-F32;F64-sequential-four-term-factor-dot;each-factor-signed-i32-Q24-nearest-ties-away;overflow-reject;no-centering;integer-i64-factor-sum-earliest-choice/1";
const LAYOUT: &str = "root,category;each(token[V,H,4,2,S],own[H,4,2,128,S],neighbor[H,4,2,128,S]-iff-L>1,span[H,4,2,128,S],span-valid[H,4,2,S]);S128,32;root-choices>=120-and-state>=120-zero/1";
const FILES: [&str; 3] = [
    "metadata.json",
    "value-tables-i32le.bin",
    "tokenizer-identity.bin",
];
const FAMILIES: [(&str, usize, usize); 2] = [("root", 120, 128), ("category", 32, 32)];

#[derive(Clone, Copy)]
pub struct ValueProducerSourcePaths<'a> {
    pub value_source: &'a Path,
    pub context_source: &'a Path,
    pub context_dependencies: ContextSourcePaths<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueParameterIdentity {
    pub shape: Vec<usize>,
    pub f32_bytes_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledValueProducerMetadata {
    pub schema: String,
    pub config: ValueProducerConfig,
    pub parameters: BTreeMap<String, ValueParameterIdentity>,
    pub context_source: ContextSourceMetadata,
    pub context_source_metadata_sha256: String,
    pub context_native: CompiledContextMetadata,
    pub context_parameters: BTreeMap<String, ValueParameterIdentity>,
    pub source_files: BTreeMap<String, BoundFile>,
    pub tokenizer: BoundFile,
    pub coefficient_policy: String,
    pub table_layout: String,
    pub table_lengths: Vec<usize>,
    pub table: BoundFile,
    /// Coordinate storage only; excludes Rust allocation/structure overhead.
    pub runtime_codec_coordinate_bytes: usize,
    pub max_coefficient_reads_per_valid_occurrence: usize,
}

fn bound(bytes: &[u8]) -> BoundFile {
    BoundFile {
        bytes: bytes.len(),
        sha256: sha256_bytes(bytes),
    }
}
fn identities(
    parameters: &BTreeMap<String, Var>,
) -> Result<BTreeMap<String, ValueParameterIdentity>> {
    parameters
        .iter()
        .map(|(name, value)| {
            let values = value.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|v| !v.is_finite()) {
                return Err(invalid("nonfinite value/context coefficient"));
            }
            let bytes = values
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((
                name.clone(),
                ValueParameterIdentity {
                    shape: value.dims().to_vec(),
                    f32_bytes_sha256: sha256_bytes(&bytes),
                },
            ))
        })
        .collect()
}
fn snapshot(paths: ValueProducerSourcePaths<'_>) -> Result<BTreeMap<String, BoundFile>> {
    let deps = paths.context_dependencies;
    let mut files = BTreeMap::new();
    for (group, directory, names) in [
        (
            "value_source",
            paths.value_source,
            vec!["metadata.json", "value-parameters.safetensors"],
        ),
        (
            "context_source",
            paths.context_source,
            vec![
                "metadata.json",
                "context-parameters.safetensors",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "base",
            deps.base,
            vec![
                "model.safetensors",
                "config.json",
                GEOMETRIC_ADDRESS_RECORD,
                GEOMETRIC_SPAN_RECORD,
            ],
        ),
        (
            "event_source",
            deps.event_source,
            vec![
                "metadata.json",
                "event-parameters.safetensors",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "event_native",
            deps.event_native,
            vec![
                "metadata.json",
                "event-tables-i32le.bin",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "span_native",
            deps.span_native,
            vec![
                "metadata.json",
                "token-actions.bin",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "potential_native",
            deps.potential_native,
            vec![
                "metadata.json",
                "potential-i32le.bin",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
    ] {
        for name in names {
            files.insert(
                format!("{group}/{name}"),
                bound(&fs::read(directory.join(name))?),
            );
        }
    }
    Ok(files)
}
fn quantize(value: f64) -> Result<i32> {
    let rounded = (value * 16_777_216.).round();
    if !rounded.is_finite() || rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
        return Err(invalid("value producer finite factor overflows signed Q24"));
    }
    Ok(rounded as i32)
}
fn canonical_states() -> [[f64; 4]; 120] {
    std::array::from_fn(|root| {
        canonical_h4_roots()[root]
            .to_array()
            .map(|x| f64::from(x as f32))
    })
}

struct Expanded {
    token: [Vec<i32>; 2],
    own: [Vec<i32>; 2],
    neighbor: [Option<Vec<i32>>; 2],
    span: [Vec<i32>; 2],
    valid: [Vec<i32>; 2],
}
impl Expanded {
    fn compile(weights: &ValueProducerWeights) -> Result<Self> {
        let c = weights.config();
        let slots = c.heads * 4 * 2;
        let values = weights
            .parameters()
            .iter()
            .map(|(name, value)| Ok((name.clone(), value.flatten_all()?.to_vec1::<f32>()?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut result = Self {
            token: std::array::from_fn(|f| vec![0; c.vocab_size * slots * FAMILIES[f].2]),
            own: std::array::from_fn(|f| vec![0; slots * 128 * FAMILIES[f].2]),
            neighbor: std::array::from_fn(|f| {
                if c.latent_lanes_per_head > 1 {
                    Some(vec![0; slots * 128 * FAMILIES[f].2])
                } else {
                    None
                }
            }),
            span: std::array::from_fn(|f| vec![0; slots * 128 * FAMILIES[f].2]),
            valid: std::array::from_fn(|f| vec![0; slots * FAMILIES[f].2]),
        };
        let roots = canonical_states();
        for (f, (family, classes, stride)) in FAMILIES.into_iter().enumerate() {
            let token = &values[&format!("token_{family}")];
            let valid = &values[&format!("span_valid_{family}")];
            for v in 0..c.vocab_size {
                for slot in 0..slots {
                    for choice in 0..classes {
                        result.token[f][(v * slots + slot) * stride + choice] =
                            quantize(f64::from(token[(v * slots + slot) * classes + choice]))?;
                    }
                }
            }
            for slot in 0..slots {
                for choice in 0..classes {
                    result.valid[f][slot * stride + choice] =
                        quantize(f64::from(valid[slot * classes + choice]))?;
                }
            }
            for (factor, output) in [
                ("own", Some(&mut result.own[f])),
                ("neighbor", result.neighbor[f].as_mut()),
                ("span", Some(&mut result.span[f])),
            ] {
                if let Some(output) = output {
                    let basis = &values[&format!("{factor}_{family}")];
                    for slot in 0..slots {
                        for (state, root) in roots.iter().enumerate() {
                            for choice in 0..classes {
                                let mut score = 0.;
                                for coordinate in 0..4 {
                                    score += f64::from(
                                        basis[(slot * classes + choice) * 4 + coordinate],
                                    ) * root[coordinate];
                                }
                                output[(slot * 128 + state) * stride + choice] = quantize(score)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    fn native(&self, c: &ValueProducerConfig) -> Result<NativeValueProducer> {
        NativeValueProducer::new(
            c.vocab_size,
            c.heads,
            c.latent_lanes_per_head,
            ValueProducerTableSlices {
                token_root: &self.token[0],
                own_root: &self.own[0],
                neighbor_root: self.neighbor[0].as_deref(),
                span_root: &self.span[0],
                span_valid_root: &self.valid[0],
                token_category: &self.token[1],
                own_category: &self.own[1],
                neighbor_category: self.neighbor[1].as_deref(),
                span_category: &self.span[1],
                span_valid_category: &self.valid[1],
            },
        )
        .map_err(|e| invalid(e.to_string()))
    }
    fn slices(&self) -> Vec<&[i32]> {
        let mut slices = Vec::new();
        for f in 0..2 {
            slices.extend([
                self.token[f].as_slice(),
                self.own[f].as_slice(),
                self.neighbor[f].as_deref().unwrap_or(&[]),
                self.span[f].as_slice(),
                self.valid[f].as_slice(),
            ]);
        }
        slices
    }
    fn bytes(&self) -> Vec<u8> {
        self.slices()
            .into_iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect()
    }
    fn lengths(&self) -> Vec<usize> {
        self.slices().iter().map(|x| x.len()).collect()
    }
}

pub struct CompiledValueProducer {
    metadata: CompiledValueProducerMetadata,
    native: NativeValueProducer,
    table_bytes: Vec<u8>,
    tokenizer: Vec<u8>,
}
impl CompiledValueProducer {
    pub fn compile(paths: ValueProducerSourcePaths<'_>, tokenizer: &[u8]) -> Result<Self> {
        if tokenizer.is_empty() {
            return Err(invalid(
                "value producer requires explicit tokenizer identity",
            ));
        }
        let before = snapshot(paths)?;
        let weights = ValueProducerWeights::load_source(paths.value_source)?;
        let context = ContextWeights::load_source(
            paths.context_source,
            paths.context_dependencies,
            tokenizer,
        )?;
        let c = weights.config();
        let cc = context.config();
        if cc.vocab_size != c.vocab_size
            || cc.heads != c.heads
            || cc.lanes_per_head != c.latent_lanes_per_head
            || cc.surrogate != FINITE_CHOICE_SURROGATE
        {
            return Err(invalid(
                "value producer requires matching saved finite-choice context",
            ));
        }
        let context_bytes = fs::read(paths.context_source.join("metadata.json"))?;
        let context_source: ContextSourceMetadata = serde_json::from_slice(&context_bytes)?;
        let context_native = CompiledContext::compile(
            &context,
            paths.context_source,
            paths.context_dependencies,
            tokenizer,
        )?;
        let expanded = Expanded::compile(&weights)?;
        let native = expanded.native(c)?;
        let table_bytes = expanded.bytes();
        if snapshot(paths)? != before {
            return Err(invalid("value/context source changed during compilation"));
        }
        let metadata = CompiledValueProducerMetadata {
            schema: SCHEMA.into(),
            config: c.clone(),
            parameters: identities(weights.parameters())?,
            context_source,
            context_source_metadata_sha256: sha256_bytes(&context_bytes),
            context_native: context_native.metadata().clone(),
            context_parameters: identities(context.parameters())?,
            source_files: before,
            tokenizer: bound(tokenizer),
            coefficient_policy: POLICY.into(),
            table_layout: LAYOUT.into(),
            table_lengths: expanded.lengths(),
            table: bound(&table_bytes),
            runtime_codec_coordinate_bytes: RUNTIME_COORDINATE_BYTES,
            max_coefficient_reads_per_valid_occurrence: native.stats().with_span_reads,
        };
        Ok(Self {
            metadata,
            native,
            table_bytes,
            tokenizer: tokenizer.to_vec(),
        })
    }
    pub fn metadata(&self) -> &CompiledValueProducerMetadata {
        &self.metadata
    }
    pub fn config(&self) -> &ValueProducerConfig {
        &self.metadata.config
    }
    pub fn validate_for(
        &self,
        weights: &ValueProducerWeights,
        context: &ContextWeights,
    ) -> Result<()> {
        if *weights.config() != self.metadata.config
            || *context.config() != self.metadata.context_source.config
            || identities(weights.parameters())? != self.metadata.parameters
            || identities(context.parameters())? != self.metadata.context_parameters
        {
            return Err(invalid(
                "compiled value producer is stale for live value/context parameters",
            ));
        }
        Ok(())
    }
    pub fn validate_native_context(&self, context: &CompiledContext) -> Result<()> {
        if *context.metadata() != self.metadata.context_native {
            return Err(invalid(
                "compiled value producer belongs to a different compiled state generator",
            ));
        }
        Ok(())
    }
    pub fn produce(
        &self,
        token: usize,
        latent: &[H4Code],
        held: Option<&[H4Code]>,
        occurrence_valid: bool,
    ) -> Result<ProducedValues> {
        self.native
            .produce(token, latent, held, occurrence_valid)
            .map_err(|e| invalid(e.to_string()))
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        let metadata = serde_json::to_vec_pretty(&self.metadata)?;
        fs::create_dir(directory)?;
        for (name, bytes) in [
            (FILES[0], metadata.as_slice()),
            (FILES[1], self.table_bytes.as_slice()),
            (FILES[2], self.tokenizer.as_slice()),
        ] {
            fs::File::create_new(directory.join(name))?.write_all(bytes)?;
        }
        Ok(())
    }
    pub fn load(
        directory: &Path,
        paths: ValueProducerSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<Self> {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("native value artifact has nonregular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("native value artifact non-UTF8 filename"))?,
            );
        }
        if names != FILES.iter().map(|x| x.to_string()).collect() {
            return Err(invalid("native value artifact file set differs"));
        }
        let expected = Self::compile(paths, tokenizer)?;
        if fs::metadata(directory.join(FILES[1]))?.len() != expected.table_bytes.len() as u64
            || fs::metadata(directory.join(FILES[2]))?.len() != tokenizer.len() as u64
        {
            return Err(invalid("native value artifact payload lengths differ"));
        }
        let saved: CompiledValueProducerMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        if saved != expected.metadata
            || fs::read(directory.join(FILES[1]))? != expected.table_bytes
            || fs::read(directory.join(FILES[2]))? != tokenizer
        {
            return Err(invalid(
                "native value artifact differs from independent source recompilation",
            ));
        }
        Ok(expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_address::GeometricAddressConfig;
    use crate::geometric_event::{CompiledEvents, EventWeights};
    use crate::geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding};
    use crate::geometric_span::GeometricSpanConfig;
    use crate::geometric_span_native::{CompiledSpanActions, SpanSourceBinding};
    use crate::geometric_stack::{
        ReadIdentityLatch, ReadScore, StackArch, StackConfig, StackModel,
    };
    use candle_core::{Device, Tensor};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const REGISTRY: &[u8] = b"finite value producer fixture tokens 0..3/v1";
    fn set(weights: &ValueProducerWeights, name: &str, values: Vec<f32>) -> Result<()> {
        let variable = &weights.parameters()[name];
        variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
        Ok(())
    }
    fn code(value: u8) -> Result<H4Code> {
        H4Code::try_from(value).map_err(|e| invalid(e.to_string()))
    }

    #[test]
    fn geometric_value_producer_native_quantization_domain_and_ties() -> Result<()> {
        assert_eq!(quantize(0.5 / 16_777_216.)?, 1);
        assert_eq!(quantize(-0.5 / 16_777_216.)?, -1);
        assert_eq!(quantize(-128.)?, i32::MIN);
        assert_eq!(quantize(f64::from(i32::MAX) / 16_777_216.)?, i32::MAX);
        for bad in [128., -128. - 1. / 16_777_216., f64::INFINITY, f64::NAN] {
            assert!(quantize(bad).is_err());
        }
        Ok(())
    }

    #[test]
    fn geometric_value_producer_native_factor_orientation_padding_and_inventory() -> Result<()> {
        for lanes in [1, 4] {
            let weights = ValueProducerWeights::new(2, 2, lanes, 27)?;
            for (name, variable) in weights.parameters() {
                set(&weights, name, vec![0.; variable.elem_count()])?;
            }
            let mut own = vec![0.; weights.parameters()["own_root"].elem_count()];
            own[4] = 0.25; // first atom, choice +identity, first state coordinate
            set(&weights, "own_root", own)?;
            let expanded = Expanded::compile(&weights)?;
            assert_eq!(expanded.own[0][128 + 1], 1 << 22); // state +identity
            assert_eq!(expanded.own[0][1], -(1 << 22)); // state -identity
            for (f, (_, classes, stride)) in FAMILIES.into_iter().enumerate() {
                for row in expanded.token[f].chunks_exact(stride) {
                    assert!(row[classes..].iter().all(|&x| x == 0));
                }
                for table in [
                    Some(&expanded.own[f]),
                    expanded.neighbor[f].as_ref(),
                    Some(&expanded.span[f]),
                ] {
                    if let Some(table) = table {
                        for atom in table.chunks_exact(128 * stride) {
                            assert!(atom[120 * stride..].iter().all(|&x| x == 0));
                            for row in atom.chunks_exact(stride) {
                                assert!(row[classes..].iter().all(|&x| x == 0));
                            }
                        }
                    }
                }
            }
            assert_eq!(expanded.neighbor[0].is_some(), lanes > 1);
            let native = expanded.native(weights.config())?;
            assert_eq!(native.stats().stored_bytes, expanded.bytes().len());
            // Padding is structural, even if a producer tries to reseal it.
            let mut bad = Expanded::compile(&weights)?;
            bad.own[0][120 * 128] = 1;
            assert!(bad.native(weights.config()).is_err());
            let mut oversized = vec![0.; weights.parameters()["token_root"].elem_count()];
            oversized[0] = 128.;
            set(&weights, "token_root", oversized)?;
            assert!(Expanded::compile(&weights).is_err());
        }
        Ok(())
    }

    struct Fixture {
        root: PathBuf,
        base: PathBuf,
        event_source: PathBuf,
        event_native: PathBuf,
        span_native: PathBuf,
        potential_native: PathBuf,
        value_source: PathBuf,
        context_source: PathBuf,
    }
    impl Fixture {
        fn dependencies(&self) -> ContextSourcePaths<'_> {
            ContextSourcePaths {
                base: &self.base,
                event_source: &self.event_source,
                event_native: &self.event_native,
                span_native: &self.span_native,
                potential_native: &self.potential_native,
            }
        }
        fn paths(&self) -> ValueProducerSourcePaths<'_> {
            ValueProducerSourcePaths {
                value_source: &self.value_source,
                context_source: &self.context_source,
                context_dependencies: self.dependencies(),
            }
        }
    }
    fn fixture() -> Result<(Fixture, ValueProducerWeights, ContextWeights)> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-compiled-value-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let f = Fixture {
            base: root.join("base"),
            event_source: root.join("event-source"),
            event_native: root.join("event-native"),
            span_native: root.join("span-native"),
            potential_native: root.join("potential-native"),
            value_source: root.join("value-source"),
            context_source: root.join("context-source"),
            root,
        };
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 4,
            width: 16,
            heads: 1,
            mlp_hidden: 16,
            context: 16,
            pattern: "rra".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 19,
            memory: None,
            select: None,
            pointer: None,
        };
        let mut model = StackModel::new(config, &Device::Cpu)?;
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        let address = GeometricAddressConfig::new(16, 1)?;
        model.set_geometric_address(address.clone())?;
        let span = GeometricSpanConfig::new(16)?;
        model.set_geometric_span(span.clone())?;
        model.save(&f.base)?;
        let event = EventWeights::new(4, 2, 173)?;
        event.save_source(&f.event_source, &f.base, REGISTRY)?;
        CompiledEvents::compile(&event, &f.event_source, &f.base, REGISTRY)?
            .save(&f.event_native)?;
        let span_source = SpanSourceBinding::from_files(
            &f.base.join("model.safetensors"),
            &f.base.join("config.json"),
            REGISTRY,
        )?;
        CompiledSpanActions::compile(
            model.variables()["embedding.weight"].as_tensor(),
            &span,
            &span_source,
        )?
        .save(&f.span_native)?;
        let potential_source = PotentialSourceBinding::from_directory(&f.base, REGISTRY)?;
        CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            &address,
            &potential_source,
        )?
        .save(&f.potential_native)?;
        let value = ValueProducerWeights::new(4, 1, 4, 21)?;
        value.save_source(&f.value_source)?;
        let context = ContextWeights::new_finite_choice(4, 16, 1, 27)?;
        context.save_source(&f.context_source, f.dependencies(), REGISTRY)?;
        Ok((f, value, context))
    }

    #[test]
    fn geometric_value_producer_native_source_reload_resealed_and_live_binding() -> Result<()> {
        let (f, value, context) = fixture()?;
        let compiled = CompiledValueProducer::compile(f.paths(), REGISTRY)?;
        let artifact = f.root.join("compiled");
        compiled.save(&artifact)?;
        assert!(compiled.save(&artifact).is_err());
        let restored = CompiledValueProducer::load(&artifact, f.paths(), REGISTRY)?;
        assert_eq!(compiled.metadata(), restored.metadata());
        compiled.validate_for(&value, &context)?;
        let context_native =
            CompiledContext::compile(&context, &f.context_source, f.dependencies(), REGISTRY)?;
        compiled.validate_native_context(&context_native)?;
        assert!(CompiledValueProducer::compile(f.paths(), b"wrong token registry").is_err());
        let original = fs::read(artifact.join(FILES[1]))?;
        let mut changed = original.clone();
        changed[0] ^= 1;
        let original_metadata = fs::read(artifact.join(FILES[0]))?;
        let mut lied = compiled.metadata().clone();
        lied.table = bound(&changed);
        fs::write(artifact.join(FILES[1]), changed)?;
        fs::write(artifact.join(FILES[0]), serde_json::to_vec(&lied)?)?;
        assert!(CompiledValueProducer::load(&artifact, f.paths(), REGISTRY).is_err());
        fs::write(artifact.join(FILES[1]), original)?;
        fs::write(artifact.join(FILES[0]), original_metadata)?;
        let variable = &value.parameters()["token_root"];
        let mut values = variable.flatten_all()?.to_vec1::<f32>()?;
        values[0] = f32::from_bits(values[0].to_bits() ^ 1);
        variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
        assert!(compiled.validate_for(&value, &context).is_err());
        let variable = &context.parameters()["token_transition"];
        let mut values = variable.flatten_all()?.to_vec1::<f32>()?;
        values[0] = f32::from_bits(values[0].to_bits() ^ 1);
        variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
        let other_source = f.root.join("changed-context");
        context.save_source(&other_source, f.dependencies(), REGISTRY)?;
        let other = CompiledContext::compile(&context, &other_source, f.dependencies(), REGISTRY)?;
        assert!(compiled.validate_native_context(&other).is_err());
        fs::write(artifact.join("orphan"), b"unbound")?;
        assert!(CompiledValueProducer::load(&artifact, f.paths(), REGISTRY).is_err());
        fs::remove_dir_all(f.root)?;
        Ok(())
    }

    #[test]
    fn geometric_value_producer_native_observed_fixture_packets_and_status_match() -> Result<()> {
        let (f, value, _context) = fixture()?;
        let compiled = CompiledValueProducer::compile(f.paths(), REGISTRY)?;
        let roots = canonical_states();
        for ids in [[1u8, 3, 24, 119], [0, 2, 25, 118]] {
            let latent = ids.into_iter().map(code).collect::<Result<Vec<_>>>()?;
            let held = [3u8, 1, 119, 24]
                .into_iter()
                .map(code)
                .collect::<Result<Vec<_>>>()?;
            let tensor = |codes: &[H4Code]| -> Result<Tensor> {
                Ok(Tensor::from_vec(
                    codes
                        .iter()
                        .flat_map(|c| roots[usize::from(c.index())].map(|x| x as f32))
                        .collect::<Vec<_>>(),
                    vec![1, 1, 1, 4, 4],
                    &Device::Cpu,
                )?)
            };
            let state_tensor = tensor(&latent)?;
            let held_tensor = tensor(&held)?;
            for has_span in [false, true] {
                for valid in [false, true] {
                    let ordinary =
                        value.forward(&[2], &state_tensor, &held_tensor, &[has_span], &[valid])?;
                    let native = compiled.produce(
                        2,
                        &latent,
                        if has_span { Some(&held) } else { None },
                        valid,
                    )?;
                    assert_eq!(ordinary.trace.values_q16, native.values_q16[..16]);
                    for (lane, pair) in ordinary.trace.packets.iter().enumerate() {
                        for atom in 0..2 {
                            assert_eq!(pair[atom].packet()?, native.packets[lane][atom]);
                        }
                    }
                    assert_eq!(native.occurrence_valid, valid);
                    assert!(
                        native.coefficient_reads
                            <= compiled.metadata.max_coefficient_reads_per_valid_occurrence
                    );
                }
            }
        }
        fs::remove_dir_all(f.root)?;
        Ok(())
    }
}
