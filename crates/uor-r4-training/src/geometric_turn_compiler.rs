//! Offline learning and a source-only native adapter for learned turn compilation.
//! The encoder is frozen. Ordinary CE uses quarter-Q4 forward values and an STE;
//! final checkpoint selection uses the exact packed integer heads on development.
use crate::relation_compiler::{word_spans, Example, ACTS, NONE};
use crate::stack_grounded_session::{
    CompiledAction, CompilerIdentity, EncoderIdentity, GroundedSessionError, RelationLabel,
    SourceSpan, TurnCompiler,
};
use crate::{invalid, sha256_bytes, Result};
use serde::{Deserialize, Serialize};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_potential_q4::pack_coefficients,
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer},
    geometric_turn_compiler::{NativeTurnHead, TurnHeadConfig},
    h4_tables::{H4Code, HistoricalH4Tables, TRUSTED_MATHEMATICAL_SHA256},
};
use uor_r4_tokenizer::ByteBpeTokenizer;

pub const SCHEMA: &str = "uor-r4.native-geometric-turn-compiler/1";
pub const FEATURE_POLICY: &str = "frozen-signed-H4;turn-final;word-local,inverse(turn)*word,inverse(prefix-before)*prefix-after;independent-original-slice-BPE/1";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureMode {
    #[default]
    Endpoint,
    LocalRelative,
    LocalProduct,
}
impl FeatureMode {
    fn policy(self) -> &'static str {
        match self {Self::Endpoint=>FEATURE_POLICY,Self::LocalRelative=>"frozen-signed-H4;ordered-independent-word;current-previous-inverse(previous)*current;span-current-previous-next-transport;absent-masked;aggregate-bias-once/1",Self::LocalProduct=>"frozen-signed-H4;ordered-independent-word;current-previous-previous*current;span-current-previous-next-transport;absent-masked;aggregate-bias-once/1"}
    }
    fn turn_slots(self, lanes: usize) -> usize {
        if self == Self::Endpoint {
            lanes
        } else {
            3 * lanes
        }
    }
    fn span_slots(self, lanes: usize) -> usize {
        if self == Self::Endpoint {
            3 * lanes
        } else {
            4 * lanes
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitConfig {
    #[serde(default)]
    pub feature_mode: FeatureMode,
    #[serde(default)]
    pub seed: Option<u64>,
    pub steps: usize,
    pub learning_rate: f64,
    pub max_tokens: usize,
    pub max_words: usize,
    pub max_value_words: usize,
}
impl Default for FitConfig {
    fn default() -> Self {
        Self {
            feature_mode: FeatureMode::Endpoint,
            seed: None,
            steps: 64,
            learning_rate: 0.1,
            max_tokens: 128,
            max_words: 64,
            max_value_words: 8,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    schema: String,
    feature_policy: String,
    #[serde(default)]
    feature_mode: FeatureMode,
    #[serde(default)]
    initialization_seed: Option<u64>,
    #[serde(default)]
    initialization_policy: Option<String>,
    parent: NativeArtifactBinding,
    algebra_sha256: String,
    relations: Vec<String>,
    lanes: usize,
    max_tokens: usize,
    max_words: usize,
    max_value_words: usize,
    act_packed: Vec<u8>,
    relation_packed: Vec<u8>,
    span_packed: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointReport {
    pub step: usize,
    pub native_development_ce: f64,
    pub act_correct: usize,
    pub relation_correct: usize,
    pub span_exact: usize,
    pub examples: usize,
    pub artifact_sha256: String,
}
#[derive(Clone, Debug)]
pub struct CheckpointArtifact {
    pub step: usize,
    pub native_artifact: Vec<u8>,
    pub source_parameters: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct FitResult {
    pub native_artifact: Vec<u8>,
    pub source_parameters: Vec<u8>,
    pub checkpoints: Vec<CheckpointReport>,
    pub checkpoint_artifacts: Vec<CheckpointArtifact>,
    pub diagnostics: serde_json::Value,
    pub selected_step: usize,
}
#[derive(Clone, Debug)]
struct Features {
    turn: Vec<Vec<Option<u8>>>,
    words: Vec<Vec<Option<u8>>>,
    spans: Vec<SourceSpan>,
}
#[derive(Clone, Debug)]
struct Labeled {
    features: Features,
    act: usize,
    relation: usize,
    inside: Vec<usize>,
    gold_span: Option<SourceSpan>,
}

fn verified_tokenizer(
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    bytes: &[u8],
) -> Result<String> {
    let sha = sha256_bytes(bytes);
    let parsed = ByteBpeTokenizer::from_tokenizer_json_bytes(bytes)
        .ok_or_else(|| invalid("invalid compiler tokenizer bytes"))?;
    if sha != native.artifact_binding().identity.tokenizer_sha256
        || tokenizer.address() != parsed.address()
    {
        return Err(invalid(
            "actual compiler tokenizer content does not match native parent",
        ));
    }
    Ok(sha)
}
fn encode(
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    text: &str,
    max_tokens: usize,
) -> Result<Vec<H4Code>> {
    let tokens = tokenizer.encode(text);
    if tokens.len() > max_tokens {
        return Err(invalid("native compiler text exceeds token cap"));
    }
    let (tables, geometry) = native.context_encoder_parts();
    let mut state = NativeContextState::new(tables.heads(), tables.lanes_per_head())
        .map_err(|e| invalid(e.to_string()))?;
    for token in tokens {
        state
            .step(token as usize, tables, geometry)
            .map_err(|e| invalid(e.to_string()))?;
    }
    Ok(state.states()[..tables.heads() * tables.lanes_per_head()].to_vec())
}
fn features(
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    source: &str,
    max_tokens: usize,
    max_words: usize,
    mode: FeatureMode,
) -> Result<Features> {
    // Whole-text token count is a bound only in local mode; its latent endpoint is unused.
    let token_count = tokenizer.encode(source).len();
    if token_count > max_tokens {
        return Err(invalid("native compiler text exceeds token cap"));
    }
    let spans = word_spans(source);
    if spans.len() > max_words {
        return Err(invalid("native compiler text exceeds word cap"));
    }
    let (_, geometry) = native.context_encoder_parts();
    let locals: Vec<Vec<H4Code>> = spans
        .iter()
        .map(|span| encode(native, tokenizer, &source[span.start..span.end], max_tokens))
        .collect::<Result<_>>()?;
    let mut words = Vec::with_capacity(spans.len());
    let mut rows = Vec::new();
    if mode == FeatureMode::Endpoint {
        let turn = encode(native, tokenizer, source, max_tokens)?;
        rows.push(turn.iter().map(|c| Some(c.index())).collect());
        for (span, local) in spans.iter().zip(&locals) {
            let before = encode(native, tokenizer, &source[..span.start], max_tokens)?;
            let after = encode(native, tokenizer, &source[..span.end], max_tokens)?;
            let mut roots: Vec<Option<u8>> = local.iter().map(|c| Some(c.index())).collect();
            roots.extend(
                turn.iter()
                    .zip(local)
                    .map(|(a, b)| Some(geometry.relative(*a, *b).index())),
            );
            roots.extend(
                before
                    .iter()
                    .zip(&after)
                    .map(|(a, b)| Some(geometry.relative(*a, *b).index())),
            );
            words.push(roots);
        }
    } else {
        (rows, words) = local_rows(&locals, geometry, mode)?;
    }
    Ok(Features {
        turn: rows,
        words,
        spans: spans
            .iter()
            .map(|s| SourceSpan {
                start: s.start,
                end: s.end,
            })
            .collect(),
    })
}

fn local_rows(
    locals: &[Vec<H4Code>],
    geometry: &HistoricalH4Tables,
    mode: FeatureMode,
) -> Result<(Vec<Vec<Option<u8>>>, Vec<Vec<Option<u8>>>)> {
    if mode == FeatureMode::Endpoint || locals.len() > 64 {
        return Err(invalid("invalid local feature row request"));
    }
    let lanes = locals.first().map_or(0, Vec::len);
    if !locals.is_empty() && (!(1..=8).contains(&lanes) || locals.iter().any(|r| r.len() != lanes))
    {
        return Err(invalid("local feature lane shape mismatch"));
    }
    let mut rows = Vec::with_capacity(locals.len());
    let mut words = Vec::with_capacity(locals.len());
    for (index, current) in locals.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|i| locals.get(i));
        let next = locals.get(index + 1);
        let current_roots: Vec<Option<u8>> = current.iter().map(|c| Some(c.index())).collect();
        let previous_roots: Vec<Option<u8>> = (0..lanes)
            .map(|lane| previous.map(|p| p[lane].index()))
            .collect();
        let transport: Vec<Option<u8>> = (0..lanes)
            .map(|lane| {
                previous.map(|p| match mode {
                    FeatureMode::LocalRelative => geometry.relative(p[lane], current[lane]).index(),
                    _ => geometry.compose(p[lane], current[lane]).index(),
                })
            })
            .collect();
        let mut row = current_roots.clone();
        row.extend(&previous_roots);
        row.extend(&transport);
        rows.push(row);
        let mut word = current_roots;
        word.extend(previous_roots);
        word.extend((0..lanes).map(|lane| next.map(|n| n[lane].index())));
        word.extend(transport);
        words.push(word);
    }
    Ok((rows, words))
}

fn config_valid(c: &FitConfig) -> Result<()> {
    if c.steps == 0
        || c.steps > 64
        || !c.learning_rate.is_finite()
        || c.learning_rate <= 0.0
        || c.max_tokens == 0
        || c.max_tokens > 128
        || c.max_words == 0
        || c.max_words > 64
        || c.max_value_words == 0
        || c.max_value_words > 8
    {
        return Err(invalid("invalid bounded compiler fit configuration"));
    }
    Ok(())
}
fn relation_valid(relations: &[String]) -> Result<()> {
    if relations.is_empty()
        || relations.len() > 63
        || relations.iter().any(|s| s.is_empty() || s == NONE)
        || relations
            .iter()
            .enumerate()
            .any(|(i, s)| relations[..i].contains(s))
    {
        return Err(invalid(
            "compiler relation labels must be unique non-none names",
        ));
    }
    Ok(())
}
fn label_examples(
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    relations: &[String],
    examples: &[Example],
    config: &FitConfig,
) -> Result<Vec<Labeled>> {
    examples
        .iter()
        .map(|e| {
            let f = features(
                native,
                tokenizer,
                &e.text,
                config.max_tokens,
                config.max_words,
                config.feature_mode,
            )?;
            if f.words.is_empty() {
                return Err(invalid(
                    "compiler training/development requires at least one word",
                ));
            }
            let act = ACTS
                .iter()
                .position(|a| *a == e.act)
                .ok_or_else(|| invalid("unknown compiler act"))?;
            let relation = if e.relation == NONE {
                relations.len()
            } else {
                relations
                    .iter()
                    .position(|s| s == &e.relation)
                    .ok_or_else(|| invalid("unknown compiler relation"))?
            };
            if (act == 3) != (relation == relations.len()) {
                return Err(invalid("none act and relation must agree"));
            }
            let span = if act < 2 {
                Some(
                    e.slot_span()
                        .ok_or_else(|| invalid("write label needs exact value span"))?,
                )
            } else {
                None
            };
            let mut inside = Vec::new();
            for s in &f.spans {
                let label = if let Some((start, end)) = span {
                    let overlap = s.start < end && start < s.end;
                    if overlap && !(s.start >= start && s.end <= end) {
                        return Err(invalid("gold value cuts a word"));
                    }
                    usize::from(overlap)
                } else {
                    0
                };
                inside.push(label);
            }
            if act < 2
                && (inside.iter().sum::<usize>() == 0
                    || inside.iter().sum::<usize>() > config.max_value_words)
            {
                return Err(invalid("write value exceeds supported span"));
            }
            let gold_span = span.map(|(start, end)| SourceSpan { start, end });
            if let Some(gold) = gold_span {
                let first = inside
                    .iter()
                    .position(|x| *x == 1)
                    .ok_or_else(|| invalid("no value words"))?;
                let last = inside
                    .iter()
                    .rposition(|x| *x == 1)
                    .ok_or_else(|| invalid("no value words"))?;
                if f.spans[first].start != gold.start || f.spans[last].end != gold.end {
                    return Err(invalid(
                        "gold value edges are not exactly representable by word spans",
                    ));
                }
            }
            Ok(Labeled {
                features: f,
                act,
                relation,
                inside,
                gold_span,
            })
        })
        .collect()
}
fn shape(classes: usize, slots: usize) -> TurnHeadConfig {
    TurnHeadConfig { classes, slots }
}
fn packed(shadow: &[f64]) -> Result<Vec<u8>> {
    let q: Vec<i8> = shadow
        .iter()
        .map(|x| ((*x as f32) * 4.0).round().clamp(-7.0, 7.0) as i8)
        .collect();
    pack_coefficients(&q).map_err(|e| invalid(e.to_string()))
}
fn head(config: TurnHeadConfig, shadow: &[f64]) -> Result<NativeTurnHead> {
    NativeTurnHead::new(config, &packed(shadow)?).map_err(|e| invalid(e.to_string()))
}
fn softmax_ce(scores: &[i64], target: usize) -> (f64, Vec<f64>) {
    let max = scores.iter().copied().max().unwrap_or(0) as f64 * 0.25;
    let mut p: Vec<f64> = scores
        .iter()
        .map(|s| (*s as f64 * 0.25 - max).exp())
        .collect();
    let sum = p.iter().sum::<f64>();
    for v in &mut p {
        *v /= sum;
    }
    let loss = max + sum.ln() - scores[target] as f64 * 0.25;
    p[target] -= 1.0;
    (loss, p)
}
#[cfg(test)]
fn add_gradient(
    gradient: &mut [f64],
    config: TurnHeadConfig,
    features: &[u8],
    errors: &[f64],
    weight: f64,
) {
    let stride = 1 + config.slots * 120;
    for (class, error) in errors.iter().enumerate() {
        let base = class * stride;
        gradient[base] += error * weight;
        for (slot, root) in features.iter().enumerate() {
            gradient[base + 1 + slot * 120 + usize::from(*root)] += error * weight;
        }
    }
}
fn add_gradient_rows(
    gradient: &mut [f64],
    config: TurnHeadConfig,
    rows: &[Vec<Option<u8>>],
    errors: &[f64],
    weight: f64,
) {
    let stride = 1 + config.slots * 120;
    for (class, error) in errors.iter().enumerate() {
        let base = class * stride;
        gradient[base] += error * weight;
        for row in rows {
            for (slot, root) in row.iter().enumerate() {
                if let Some(root) = root {
                    gradient[base + 1 + slot * 120 + usize::from(*root)] += error * weight;
                }
            }
        }
    }
}
fn seeded_weights(shapes: &[TurnHeadConfig], seed: Option<u64>) -> Result<Vec<Vec<f64>>> {
    let mut state = seed.unwrap_or(0);
    shapes
        .iter()
        .map(|s| {
            let n = s.coefficient_count().map_err(|e| invalid(e.to_string()))?;
            Ok((0..n)
                .map(|_| {
                    if seed.is_none() {
                        0.0
                    } else {
                        state = state.wrapping_add(0x9e3779b97f4a7c15);
                        let mut z = state;
                        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                        z ^= z >> 31;
                        match z % 3 {
                            0 => -0.25,
                            1 => 0.0,
                            _ => 0.25,
                        }
                    }
                })
                .collect())
        })
        .collect()
}
fn aggregate_signature(rows: &[Vec<Option<u8>>], slots: usize) -> Vec<usize> {
    let mut counts = vec![0; slots * 120];
    for row in rows {
        for (slot, root) in row.iter().enumerate() {
            if let Some(root) = root {
                counts[slot * 120 + usize::from(*root)] += 1;
            }
        }
    }
    counts
}
fn selected_span(
    scores: &[Vec<i64>],
    spans: &[SourceSpan],
    max_words: usize,
) -> Option<SourceSpan> {
    let mut best = 0i64;
    let mut selected = None;
    for start in 0..scores.len() {
        let mut sum = 0;
        for end in start..scores.len().min(start + max_words) {
            sum += scores[end][1] - scores[end][0];
            if sum > best {
                best = sum;
                selected = Some(SourceSpan {
                    start: spans[start].start,
                    end: spans[end].end,
                });
            }
        }
    }
    selected
}
fn argmax(scores: &[i64]) -> usize {
    let mut best = 0;
    for i in 1..scores.len() {
        if scores[i] > scores[best] {
            best = i;
        }
    }
    best
}

pub struct NativeTurnCompiler<'a> {
    artifact: Artifact,
    bytes: Vec<u8>,
    identity: CompilerIdentity,
    native: &'a NativeSourceRealizer,
    tokenizer: &'a ByteBpeTokenizer,
    act: NativeTurnHead,
    relation: NativeTurnHead,
    span: NativeTurnHead,
}
impl<'a> NativeTurnCompiler<'a> {
    pub fn load(
        bytes: &[u8],
        trusted_sha: &str,
        native: &'a NativeSourceRealizer,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
    ) -> Result<Self> {
        let trusted_tokenizer_sha = verified_tokenizer(native, tokenizer, tokenizer_bytes)?;
        if bytes.len() > 128 * 1024 {
            return Err(invalid("compiler artifact exceeds admission byte cap"));
        }
        if sha256_bytes(bytes) != trusted_sha {
            return Err(invalid("compiler artifact SHA does not match trusted root"));
        }
        let artifact: Artifact = serde_json::from_slice(bytes)?;
        relation_valid(&artifact.relations)?;
        let lanes = native.context_config().heads * native.context_config().lanes_per_head;
        if artifact.initialization_policy.as_deref().is_some_and(|p| {
            p != if artifact.initialization_seed.is_some() {
                "splitmix64-three-point-quarter-Q4[-1,0,1]/1"
            } else {
                "zero/1"
            }
        }) {
            return Err(invalid("compiler initialization identity mismatch"));
        }
        if artifact.schema != SCHEMA
            || artifact.feature_policy != artifact.feature_mode.policy()
            || artifact.parent != *native.artifact_binding()
            || artifact.lanes != lanes
            || artifact.algebra_sha256 != TRUSTED_MATHEMATICAL_SHA256.unwrap_or("")
            || trusted_tokenizer_sha != artifact.parent.identity.tokenizer_sha256
            || artifact.max_tokens == 0
            || artifact.max_tokens > 128
            || artifact.max_words == 0
            || artifact.max_words > 64
            || artifact.max_value_words == 0
            || artifact.max_value_words > 8
        {
            return Err(invalid("compiler binding or feature caps mismatch"));
        }
        let act = NativeTurnHead::new(
            shape(4, artifact.feature_mode.turn_slots(lanes)),
            &artifact.act_packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let relation = NativeTurnHead::new(
            shape(
                artifact.relations.len() + 1,
                artifact.feature_mode.turn_slots(lanes),
            ),
            &artifact.relation_packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let span = NativeTurnHead::new(
            shape(2, artifact.feature_mode.span_slots(lanes)),
            &artifact.span_packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let identity = CompilerIdentity {
            schema: SCHEMA.into(),
            artifact_sha256: trusted_sha.into(),
            tokenizer_sha256: trusted_tokenizer_sha.clone(),
            label_schema: "ordered-relation-ids-1-based;assert-update-query-none/1".into(),
            relations: artifact
                .relations
                .iter()
                .enumerate()
                .map(|(i, name)| RelationLabel {
                    id: (i + 1) as u32,
                    name: name.clone(),
                })
                .collect(),
            encoder: Some(EncoderIdentity {
                config_sha256: artifact.parent.identity.parent_config_sha256.clone(),
                model_sha256: artifact.parent.metadata_sha256.clone(),
                transport_sha256: Some(artifact.algebra_sha256.clone()),
            }),
        };
        Ok(Self {
            artifact,
            bytes: bytes.to_vec(),
            identity,
            native,
            tokenizer,
            act,
            relation,
            span,
        })
    }
    pub fn predict(&self, source: &str) -> Result<CompiledAction> {
        let f = features(
            self.native,
            self.tokenizer,
            source,
            self.artifact.max_tokens,
            self.artifact.max_words,
            self.artifact.feature_mode,
        )?;
        let act = argmax(
            &self
                .act
                .scores_rows(&f.turn)
                .map_err(|e| invalid(e.to_string()))?,
        );
        let relation = argmax(
            &self
                .relation
                .scores_rows(&f.turn)
                .map_err(|e| invalid(e.to_string()))?,
        );
        if act == 3 || relation == self.artifact.relations.len() {
            return Ok(CompiledAction::Unresolved {
                reason: "native learned none act or relation".into(),
            });
        }
        let relation = (relation + 1) as u32;
        if act == 2 {
            return Ok(CompiledAction::QueryCurrent { relation });
        }
        let scores: Vec<Vec<i64>> = f
            .words
            .iter()
            .map(|w| {
                self.span
                    .scores_masked(w)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<_>>()?;
        let Some(span) = selected_span(&scores, &f.spans, self.artifact.max_value_words) else {
            return Ok(CompiledAction::Unresolved {
                reason: "native learned span has no positive margin".into(),
            });
        };
        Ok(if act == 0 {
            CompiledAction::Assert { relation, span }
        } else {
            CompiledAction::Correct { relation, span }
        })
    }
}
impl TurnCompiler for NativeTurnCompiler<'_> {
    fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }
    fn artifact_bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn compile(&self, source: &str) -> std::result::Result<CompiledAction, GroundedSessionError> {
        self.predict(source)
            .map_err(|e| GroundedSessionError::Compiler(e.to_string()))
    }
}

pub fn fit_examples(
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    tokenizer_bytes: &[u8],
    relations: &[String],
    training: &[Example],
    development: &[Example],
    config: FitConfig,
) -> Result<FitResult> {
    config_valid(&config)?;
    relation_valid(relations)?;
    if training.is_empty() || development.is_empty() {
        return Err(invalid(
            "compiler fit requires training and development rows",
        ));
    }
    verified_tokenizer(native, tokenizer, tokenizer_bytes)?;
    let train = label_examples(native, tokenizer, relations, training, &config)?;
    let dev = label_examples(native, tokenizer, relations, development, &config)?;
    let lanes = native.context_config().heads * native.context_config().lanes_per_head;
    let shapes = [
        shape(4, config.feature_mode.turn_slots(lanes)),
        shape(relations.len() + 1, config.feature_mode.turn_slots(lanes)),
        shape(2, config.feature_mode.span_slots(lanes)),
    ];
    let mut weights = seeded_weights(&shapes, config.seed)?;
    let mut m: Vec<Vec<f64>> = weights.iter().map(|w| vec![0.0; w.len()]).collect();
    let mut v = m.clone();
    let mut checkpoints = Vec::new();
    let mut checkpoint_artifacts = Vec::new();
    let mut gradient_norms = Vec::new();
    let mut selected = None;
    let mut best = f64::INFINITY;
    for step in 0..=config.steps {
        let heads: Vec<NativeTurnHead> = shapes
            .iter()
            .zip(&weights)
            .map(|(s, w)| head(*s, w))
            .collect::<Result<_>>()?;
        if step == 0 || step % 16 == 0 || step == config.steps {
            let artifact = Artifact {
                schema: SCHEMA.into(),
                feature_policy: config.feature_mode.policy().into(),
                feature_mode: config.feature_mode,
                initialization_seed: config.seed,
                initialization_policy: Some(
                    if config.seed.is_some() {
                        "splitmix64-three-point-quarter-Q4[-1,0,1]/1"
                    } else {
                        "zero/1"
                    }
                    .into(),
                ),
                parent: native.artifact_binding().clone(),
                algebra_sha256: TRUSTED_MATHEMATICAL_SHA256
                    .ok_or_else(|| invalid("canonical H4 digest absent"))?
                    .into(),
                relations: relations.to_vec(),
                lanes,
                max_tokens: config.max_tokens,
                max_words: config.max_words,
                max_value_words: config.max_value_words,
                act_packed: packed(&weights[0])?,
                relation_packed: packed(&weights[1])?,
                span_packed: packed(&weights[2])?,
            };
            let bytes = serde_json::to_vec(&artifact)?;
            let mut loss = 0.0;
            let (mut act_correct, mut relation_correct, mut span_exact) = (0, 0, 0);
            for row in &dev {
                let a = heads[0]
                    .scores_rows(&row.features.turn)
                    .map_err(|e| invalid(e.to_string()))?;
                let r = heads[1]
                    .scores_rows(&row.features.turn)
                    .map_err(|e| invalid(e.to_string()))?;
                loss += softmax_ce(&a, row.act).0 / 3.0 + softmax_ce(&r, row.relation).0 / 3.0;
                act_correct += usize::from(argmax(&a) == row.act);
                relation_correct += usize::from(argmax(&r) == row.relation);
                let mut scores = Vec::new();
                for (word, label) in row.features.words.iter().zip(&row.inside) {
                    let s = heads[2]
                        .scores_masked(word)
                        .map_err(|e| invalid(e.to_string()))?;
                    loss += softmax_ce(&s, *label).0 / (3.0 * row.inside.len() as f64);
                    scores.push(s);
                }
                let gold = row.gold_span;
                span_exact += usize::from(
                    selected_span(&scores, &row.features.spans, config.max_value_words) == gold,
                );
            }
            loss /= dev.len() as f64;
            checkpoints.push(CheckpointReport {
                step,
                native_development_ce: loss,
                act_correct,
                relation_correct,
                span_exact,
                examples: dev.len(),
                artifact_sha256: sha256_bytes(&bytes),
            });
            let source = serde_json::to_vec(
                &serde_json::json!({"schema":"uor-r4.geometric-turn-source/1","step":step,"config":config,"native_artifact_sha256":sha256_bytes(&bytes),"shadow_f32":weights.iter().map(|w|w.iter().map(|x|*x as f32).collect::<Vec<_>>()).collect::<Vec<_>>()}),
            )?;
            checkpoint_artifacts.push(CheckpointArtifact {
                step,
                native_artifact: bytes.clone(),
                source_parameters: source,
            });
            if loss < best {
                best = loss;
                selected = Some((step, bytes, weights.clone()));
            }
        }
        if step == config.steps {
            break;
        }
        let mut gradient: Vec<Vec<f64>> = weights.iter().map(|w| vec![0.0; w.len()]).collect();
        for row in &train {
            for (h, target) in [(0, row.act), (1, row.relation)] {
                let scores = heads[h]
                    .scores_rows(&row.features.turn)
                    .map_err(|e| invalid(e.to_string()))?;
                let (_, errors) = softmax_ce(&scores, target);
                add_gradient_rows(
                    &mut gradient[h],
                    shapes[h],
                    &row.features.turn,
                    &errors,
                    1.0 / (3.0 * train.len() as f64),
                );
            }
            for (word, target) in row.features.words.iter().zip(&row.inside) {
                let scores = heads[2]
                    .scores_masked(word)
                    .map_err(|e| invalid(e.to_string()))?;
                let (_, errors) = softmax_ce(&scores, *target);
                add_gradient_rows(
                    &mut gradient[2],
                    shapes[2],
                    &[word.clone()],
                    &errors,
                    1.0 / (3.0 * train.len() as f64 * row.inside.len() as f64),
                );
            }
        }
        gradient_norms.push(
            gradient
                .iter()
                .map(|g| g.iter().map(|x| x * x).sum::<f64>().sqrt())
                .collect::<Vec<_>>(),
        );
        let t = (step + 1) as i32;
        for h in 0..3 {
            for j in 0..weights[h].len() {
                let g = gradient[h][j];
                m[h][j] = 0.9 * m[h][j] + 0.1 * g;
                v[h][j] = 0.999 * v[h][j] + 0.001 * g * g;
                let update = config.learning_rate * (m[h][j] / (1.0 - 0.9f64.powi(t)))
                    / ((v[h][j] / (1.0 - 0.999f64.powi(t))).sqrt() + 1e-8);
                weights[h][j] = (weights[h][j] - update).clamp(-1.75, 1.75);
            }
        }
    }
    let (selected_step, native_artifact, selected_weights) =
        selected.ok_or_else(|| invalid("compiler fit produced no finite checkpoint"))?;
    let mut turn_labels =
        std::collections::BTreeMap::<Vec<usize>, std::collections::BTreeSet<(usize, usize)>>::new();
    let mut span_labels =
        std::collections::BTreeMap::<Vec<Option<u8>>, std::collections::BTreeSet<usize>>::new();
    let mut coverage =
        vec![std::collections::BTreeSet::new(); config.feature_mode.span_slots(lanes)];
    for row in &train {
        turn_labels
            .entry(aggregate_signature(
                &row.features.turn,
                config.feature_mode.turn_slots(lanes),
            ))
            .or_default()
            .insert((row.act, row.relation));
        for (word, target) in row.features.words.iter().zip(&row.inside) {
            span_labels.entry(word.clone()).or_default().insert(*target);
            for (i, r) in word.iter().enumerate() {
                if let Some(root) = r {
                    coverage[i].insert(*root);
                }
            }
        }
    }
    let diagnostics = serde_json::json!({"encoder":"frozen-no-gradient-credit","feature_mode":config.feature_mode,"feature_policy":config.feature_mode.policy(),"signature_scope":"actual-additive-row-slot-root-counts;span-masked-root-tuple;not-injective-sequence","initialization_seed":config.seed,"training_examples":train.len(),"development_examples":dev.len(),"turn_unique_signatures":turn_labels.len(),"turn_conflicting_signatures":turn_labels.values().filter(|s|s.len()>1).count(),"span_unique_signatures":span_labels.len(),"span_conflicting_signatures":span_labels.values().filter(|s|s.len()>1).count(),"span_slot_root_coverage":coverage.iter().map(|s|s.len()).collect::<Vec<_>>(),"step_head_gradient_l2":gradient_norms});
    let source_parameters = serde_json::to_vec(
        &serde_json::json!({"schema":"uor-r4.geometric-turn-source/1","config":config,"policy":"ordinary-CE-f32-quarter-forward-STE-f64-Adam;frozen-encoder;equal-act-relation-mean-word-loss;deterministic-full-batch-Adam;earliest-strict-native-development-minimum/1","selected_step":selected_step,"native_artifact_sha256":sha256_bytes(&native_artifact),"selected_shadow_f32":selected_weights.iter().map(|w|w.iter().map(|x|*x as f32).collect::<Vec<_>>()).collect::<Vec<_>>(),"final_shadow_f32":weights.iter().map(|w|w.iter().map(|x|*x as f32).collect::<Vec<_>>()).collect::<Vec<_>>(),"diagnostics":diagnostics,"checkpoints":checkpoints}),
    )?;
    Ok(FitResult {
        native_artifact,
        source_parameters,
        checkpoints,
        checkpoint_artifacts,
        diagnostics,
        selected_step,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quarter_scores_have_ordinary_ce_gradient() {
        let (loss, gradient) = softmax_ce(&[0, 4], 1);
        assert!((loss - (1.0 + (-1.0f64).exp()).ln()).abs() < 1e-12);
        assert!(gradient[0] > 0.0 && gradient[1] < 0.0);
        assert!(gradient.iter().sum::<f64>().abs() < 1e-12);
        let c = shape(2, 1);
        let mut g = vec![0.0; c.coefficient_count().unwrap_or(0)];
        add_gradient(&mut g, c, &[119], &gradient, 0.5);
        assert_eq!(g[0], gradient[0] * 0.5);
        assert_eq!(g[120], gradient[0] * 0.5);
        assert_eq!(g[121], gradient[1] * 0.5);
        assert_eq!(g[241], gradient[1] * 0.5);
    }
    #[test]
    fn bounded_positive_margin_span_preserves_exact_bytes_and_ties() {
        let spans = [
            SourceSpan { start: 2, end: 5 },
            SourceSpan { start: 7, end: 12 },
            SourceSpan { start: 13, end: 17 },
        ];
        let scores = vec![vec![0, 2], vec![0, 2], vec![0, -6]];
        assert_eq!(
            selected_span(&scores, &spans, 2),
            Some(SourceSpan { start: 2, end: 12 })
        );
        assert_eq!(selected_span(&scores, &spans, 1), Some(spans[0]));
        assert_eq!(
            selected_span(&[vec![0, 0], vec![1, 0]], &spans[..2], 2),
            None
        );
    }
    #[test]
    fn quarter_forward_clips_to_native_legal_range() -> Result<()> {
        let config = shape(2, 1);
        let mut shadow = vec![
            0.0;
            config
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
        ];
        shadow[0] = 100.0;
        shadow[121] = -100.0;
        shadow[120] = 0.26;
        let h = head(config, &shadow)?;
        assert_eq!(
            h.scores(&[119]).map_err(|e| invalid(e.to_string()))?,
            vec![8, -7]
        );
        Ok(())
    }
    #[test]
    fn seeded_initialization_reproducible_and_native_distinct() -> Result<()> {
        let shapes = [shape(2, 2), shape(3, 1)];
        let a = seeded_weights(&shapes, Some(1))?;
        let b = seeded_weights(&shapes, Some(1))?;
        let c = seeded_weights(&shapes, Some(2))?;
        assert_eq!(a, b);
        assert_ne!(packed(&a[0])?, packed(&c[0])?);
        assert!(seeded_weights(&shapes, None)?
            .iter()
            .flatten()
            .all(|v| *v == 0.0));
        assert!(a.iter().flatten().all(|v| v.abs() <= 0.25));
        Ok(())
    }
    #[test]
    fn aggregated_gradient_matches_continuous_ce_extension() -> Result<()> {
        let config = shape(2, 2);
        let rows = vec![vec![Some(3), None], vec![Some(3), Some(1)]];
        let errors = softmax_ce(&[2, -1], 1).1;
        let mut gradient = vec![
            0.0;
            config
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
        ];
        add_gradient_rows(&mut gradient, config, &rows, &errors, 1.0);
        // Finite differences of the continuous CE logit extension used by the STE,
        // not a claim that the discontinuous Q4 forward has this derivative.
        let loss = |delta: f64, multiplicity: f64| {
            let a = 0.5 + delta * multiplicity;
            let b = -0.25;
            let max = a.max(b);
            max + ((a - max).exp() + (b - max).exp()).ln() - b
        };
        let epsilon = 1e-6;
        assert!(
            (gradient[0] - (loss(epsilon, 1.0) - loss(-epsilon, 1.0)) / (2.0 * epsilon)).abs()
                < 1e-8
        );
        assert!(
            (gradient[4] - (loss(epsilon, 2.0) - loss(-epsilon, 2.0)) / (2.0 * epsilon)).abs()
                < 1e-8
        );
        assert_eq!(gradient[1 + 120 + 1], errors[0]);
        assert_eq!(gradient[1 + 120], 0.0);
        Ok(())
    }
    #[test]
    fn aggregation_signature_tracks_repeats_and_ignores_absent_identity() {
        let first = vec![vec![Some(1), None], vec![Some(1), Some(2)]];
        let swapped = vec![first[1].clone(), first[0].clone()];
        assert_eq!(
            aggregate_signature(&first, 2),
            aggregate_signature(&swapped, 2)
        );
        assert_eq!(aggregate_signature(&first, 2)[1], 2);
        assert_eq!(aggregate_signature(&first, 2)[121], 0);
    }
    #[test]
    fn local_word_rows_preserve_noncommuting_orientation_and_absence() -> Result<()> {
        let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
            "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let mut witness = None;
        'search: for a in 0..120u8 {
            for b in 0..120u8 {
                let a = H4Code::try_from(a).map_err(|e| invalid(e.to_string()))?;
                let b = H4Code::try_from(b).map_err(|e| invalid(e.to_string()))?;
                if geometry.compose(a, b) != geometry.compose(b, a)
                    && geometry.relative(a, b) != geometry.compose(a, b)
                    && geometry.relative(a, b) != geometry.relative(b, a)
                {
                    witness = Some((a, b));
                    break 'search;
                }
            }
        }
        let (a, b) = witness.ok_or_else(|| invalid("noncommuting directional fixture absent"))?;
        let locals = vec![vec![a], vec![b], vec![a]];
        let (relative, span) = local_rows(&locals, &geometry, FeatureMode::LocalRelative)?;
        let (product, product_span) = local_rows(&locals, &geometry, FeatureMode::LocalProduct)?;
        assert_eq!(relative[0], vec![Some(a.index()), None, None]);
        assert_eq!(span[0], vec![Some(a.index()), None, Some(b.index()), None]);
        assert_eq!(
            relative[1],
            vec![
                Some(b.index()),
                Some(a.index()),
                Some(geometry.compose(geometry.inverse(a), b).index())
            ]
        );
        assert_eq!(
            product[1],
            vec![
                Some(b.index()),
                Some(a.index()),
                Some(geometry.compose(a, b).index())
            ]
        );
        assert_ne!(product[1][2], Some(geometry.compose(b, a).index()));
        assert_eq!(span[2][2], None);
        assert_eq!(span[1][2], Some(a.index()));
        assert_eq!(span[1][3], relative[1][2]);
        assert_eq!(product_span[1][3], product[1][2]);
        assert_ne!(relative[1][2], product[1][2]);
        let singleton = local_rows(
            &[vec![H4Code::IDENTITY]],
            &geometry,
            FeatureMode::LocalRelative,
        )?;
        assert_eq!(singleton.1[0], vec![Some(1), None, None, None]);
        Ok(())
    }
}
