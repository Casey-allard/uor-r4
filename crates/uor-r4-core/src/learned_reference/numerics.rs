//! The single scalar candidate frozen by `cpu-scalar-f32-f64-1086/1`.
//!
//! This research reference retains both R4 transport stages. Its scalar
//! reduction order is deliberately explicit; it is not Torch/Accelerate's
//! reduction order. Artifact admission and qualification belong to the caller.
//! The admitted runtime must preserve nearest-even rounding and subnormals and
//! compile without contraction, reassociation, fast math or vectorization.

#![forbid(unsafe_code)]

use serde::Serialize;

const CLAUSES: usize = 5;
const LENGTH: usize = 13;
const ROLES: usize = 3;
const WIDTH: usize = 64;
const VOCABULARY: usize = 4096;
const FRAME_COUNT: usize = 120;
const ZERO: f32 = f32::from_bits(0x0000_0000);
const HALF: f32 = f32::from_bits(0x3f00_0000);
const ONE: f32 = f32::from_bits(0x3f80_0000);
const SQRT_TWO: f32 = f32::from_bits(0x3fb5_04f3);
const EIGHT: f32 = f32::from_bits(0x4100_0000);
const WIDTH_64: f32 = f32::from_bits(0x4280_0000);
const WIDTH_128: f32 = f32::from_bits(0x4300_0000);
const EPSILON: f32 = f32::from_bits(0x3727_c5ac);
const MASKED_SCORE: f32 = f32::from_bits(0xff80_0000);
const ZERO_F64: f64 = f64::from_bits(0x0000_0000_0000_0000);

type Matrix4 = [[f64; 4]; 4];

/// The fourteen original tensors in their declared contiguous C-order layout.
/// The vocabulary head reads `core_embedding_weight` directly; no copy or
/// separately mutable head tensor exists.
pub(super) struct Weights {
    pub(super) reader_context_bias: Vec<f32>,
    pub(super) reader_context_weight: Vec<f32>,
    pub(super) reader_embedding_weight: Vec<f32>,
    pub(super) reader_role_projection_bias: Vec<f32>,
    pub(super) reader_role_projection_weight: Vec<f32>,
    pub(super) core_embedding_weight: Vec<f32>,
    pub(super) core_key_projection_weight: Vec<f32>,
    pub(super) core_null_key: Vec<f32>,
    pub(super) core_null_value: Vec<f32>,
    pub(super) core_output_norm_bias: Vec<f32>,
    pub(super) core_output_norm_weight: Vec<f32>,
    pub(super) core_output_projection_weight: Vec<f32>,
    pub(super) core_query_projection_weight: Vec<f32>,
    pub(super) core_value_projection_weight: Vec<f32>,
}

/// Verified original f64 frame bits and integer maps, never regenerated from
/// f32 actions or recomputed irrational coordinates.
pub(super) struct Frames {
    pub(super) matrices: Vec<f64>,
    pub(super) multiplication: Vec<usize>,
    pub(super) token_leaves: Vec<usize>,
    pub(super) identity: usize,
}

/// Complete B=1 comparison seams, flattened in the original C-order layouts.
/// These diagnostics are for the admitted comparison harness, not model inputs.
#[derive(Debug, Serialize)]
pub struct Diagnostics {
    /// `[1,5,3,13]`, including exact positive-zero padding probabilities.
    pub role_attention: Vec<f32>,
    /// `[1,5,3,64]`, including the unused query-location mixture.
    pub role_vectors: Vec<f32>,
    /// `[1,5]`, four facts followed by the learned null.
    pub binding_attention: Vec<f32>,
    /// `[1,4096]`, the entire tied vocabulary head.
    pub logits: Vec<f32>,
    /// `[1,5,3]`, diagnostic only; no argmax routes a mixture.
    pub role_argmax: Vec<usize>,
    pub token_id: u32,
    /// `[1,5,13]`; padded entries retain the native identity sentinel.
    pub token_frame_indices: Vec<usize>,
    /// `[1,5]`; cumulative folding continues across clause boundaries.
    pub clause_frame_indices: Vec<usize>,
}

#[inline]
fn finite32(value: f32, stage: &'static str) -> Result<f32, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!("nonfinite value in {stage}"))
    }
}

#[inline]
fn finite64(value: f64, stage: &'static str) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!("nonfinite value in {stage}"))
    }
}

// Each helper contains one operation of the declared type. In particular,
// multiply and accumulation are separate operations, never `mul_add`.
#[inline]
fn add32(left: f32, right: f32, stage: &'static str) -> Result<f32, String> {
    finite32(left + right, stage)
}

#[inline]
fn sub32(left: f32, right: f32, stage: &'static str) -> Result<f32, String> {
    finite32(left - right, stage)
}

#[inline]
fn mul32(left: f32, right: f32, stage: &'static str) -> Result<f32, String> {
    finite32(left * right, stage)
}

#[inline]
fn div32(left: f32, right: f32, stage: &'static str) -> Result<f32, String> {
    finite32(left / right, stage)
}

#[inline]
fn add64(left: f64, right: f64, stage: &'static str) -> Result<f64, String> {
    finite64(left + right, stage)
}

#[inline]
fn mul64(left: f64, right: f64, stage: &'static str) -> Result<f64, String> {
    finite64(left * right, stage)
}

#[inline]
fn narrow32(value: f64, stage: &'static str) -> Result<f32, String> {
    finite64(value, stage)?;
    finite32(value as f32, stage)
}

fn check_layout(weights: &Weights, frames: &Frames) -> Result<(), String> {
    // Loader validation owns full identities and finite-state admission. These
    // fixed sizes prevent a malformed internal caller from indexing a short
    // tensor; arithmetic checks retain finite-value failures at the used seam.
    let layouts = [
        (weights.reader_context_bias.len(), 64, "reader.context.bias"),
        (
            weights.reader_context_weight.len(),
            64 * 32 * 5,
            "reader.context.weight",
        ),
        (
            weights.reader_embedding_weight.len(),
            4096 * 32,
            "reader.embedding.weight",
        ),
        (
            weights.reader_role_projection_bias.len(),
            3,
            "reader.role_projection.bias",
        ),
        (
            weights.reader_role_projection_weight.len(),
            3 * 64,
            "reader.role_projection.weight",
        ),
        (
            weights.core_embedding_weight.len(),
            4096 * 64,
            "core.embedding.weight",
        ),
        (
            weights.core_key_projection_weight.len(),
            64 * 128,
            "core.key_projection.weight",
        ),
        (weights.core_null_key.len(), 64, "core.null_key"),
        (weights.core_null_value.len(), 64, "core.null_value"),
        (
            weights.core_output_norm_bias.len(),
            64,
            "core.output_norm.bias",
        ),
        (
            weights.core_output_norm_weight.len(),
            64,
            "core.output_norm.weight",
        ),
        (
            weights.core_output_projection_weight.len(),
            64 * 64,
            "core.output_projection.weight",
        ),
        (
            weights.core_query_projection_weight.len(),
            64 * 128,
            "core.query_projection.weight",
        ),
        (
            weights.core_value_projection_weight.len(),
            64 * 64,
            "core.value_projection.weight",
        ),
    ];
    for (actual, expected, name) in layouts {
        if actual != expected {
            return Err(format!("invalid frozen tensor layout: {name}"));
        }
    }
    if frames.matrices.len() != FRAME_COUNT * 4 * 4
        || frames.multiplication.len() != FRAME_COUNT * FRAME_COUNT
        || frames.token_leaves.len() != 8192
        || frames.identity >= FRAME_COUNT
    {
        return Err("invalid frozen frame layout".to_owned());
    }
    Ok(())
}

fn gelu(value: f32) -> Result<f32, String> {
    const STAGE: &str = "reader GELU";
    let t1 = div32(value, SQRT_TWO, STAGE)?;
    let t2 = finite32(libm::erff(t1), STAGE)?;
    let t3 = add32(ONE, t2, STAGE)?;
    let t4 = mul32(HALF, value, STAGE)?;
    mul32(t4, t3, STAGE)
}

fn linear<const IN: usize, const OUT: usize>(
    input: &[f32; IN],
    weight: &[f32],
    bias: Option<&[f32]>,
    stage: &'static str,
) -> Result<[f32; OUT], String> {
    if weight.len() != IN * OUT || bias.is_some_and(|values| values.len() != OUT) {
        return Err(format!("invalid dense layout in {stage}"));
    }
    let mut output = [ZERO; OUT];
    for destination in 0..OUT {
        let mut sum = ZERO;
        for source in 0..IN {
            let product = mul32(weight[destination * IN + source], input[source], stage)?;
            sum = add32(sum, product, stage)?;
        }
        output[destination] = match bias {
            Some(values) => add32(sum, values[destination], stage)?,
            None => sum,
        };
    }
    Ok(output)
}

fn softmax<const N: usize>(
    scores: &[f32; N],
    valid: usize,
    stage: &'static str,
) -> Result<[f32; N], String> {
    if valid == 0 || valid > N {
        return Err(format!("empty or overlong softmax support in {stage}"));
    }
    let mut maximum = finite32(scores[0], stage)?;
    for &score in &scores[1..valid] {
        finite32(score, stage)?;
        if score > maximum {
            maximum = score;
        }
    }
    let mut output = [ZERO; N];
    let mut denominator = ZERO;
    for position in 0..valid {
        let difference = sub32(scores[position], maximum, stage)?;
        let exponential = finite32(libm::expf(difference), stage)?;
        output[position] = exponential;
        denominator = add32(denominator, exponential, stage)?;
    }
    if denominator <= ZERO {
        return Err(format!("nonpositive softmax denominator in {stage}"));
    }
    for probability in &mut output[..valid] {
        *probability = div32(*probability, denominator, stage)?;
    }
    // Padded scores are not exponentiated and padded outputs remain +0.
    Ok(output)
}

fn layer_norm<const N: usize>(
    input: &[f32; N],
    affine: Option<(&[f32], &[f32])>,
) -> Result<[f32; N], String> {
    const STAGE: &str = "layer norm";
    let divisor = match N {
        64 => WIDTH_64,
        128 => WIDTH_128,
        _ => return Err("unsupported layer norm width".to_owned()),
    };
    if affine.is_some_and(|(scale, bias)| scale.len() != N || bias.len() != N) {
        return Err("invalid layer norm affine layout".to_owned());
    }
    let mut sum = ZERO;
    for &value in input {
        sum = add32(sum, value, STAGE)?;
    }
    let mean = div32(sum, divisor, STAGE)?;
    let mut deviations = [ZERO; N];
    let mut squared_sum = ZERO;
    for coordinate in 0..N {
        let deviation = sub32(input[coordinate], mean, STAGE)?;
        deviations[coordinate] = deviation;
        let squared = mul32(deviation, deviation, STAGE)?;
        squared_sum = add32(squared_sum, squared, STAGE)?;
    }
    let variance = div32(squared_sum, divisor, STAGE)?;
    let stabilized = add32(variance, EPSILON, STAGE)?;
    let denominator = finite32(libm::sqrtf(stabilized), STAGE)?;
    if denominator <= ZERO {
        return Err("nonpositive layer norm denominator".to_owned());
    }
    let mut output = [ZERO; N];
    for coordinate in 0..N {
        let normalized = div32(deviations[coordinate], denominator, STAGE)?;
        output[coordinate] = match affine {
            Some((scale, bias)) => {
                let scaled = mul32(normalized, scale[coordinate], STAGE)?;
                add32(scaled, bias[coordinate], STAGE)?
            }
            None => normalized,
        };
    }
    Ok(output)
}

fn first_argmax(values: &[f32], stage: &'static str) -> Result<usize, String> {
    let Some(&first) = values.first() else {
        return Err(format!("empty argmax in {stage}"));
    };
    let mut maximum = finite32(first, stage)?;
    let mut selected = 0;
    for (index, &value) in values.iter().enumerate().skip(1) {
        finite32(value, stage)?;
        // Strict greater-than retains the first index for exact ties.
        if value > maximum {
            maximum = value;
            selected = index;
        }
    }
    Ok(selected)
}

fn frame_matrix(frames: &Frames, index: usize) -> Result<Matrix4, String> {
    if index >= FRAME_COUNT {
        return Err("frame index outside the frozen atlas".to_owned());
    }
    let mut matrix = [[ZERO_F64; 4]; 4];
    for row in 0..4 {
        for column in 0..4 {
            matrix[row][column] = finite64(
                frames.matrices[index * 16 + row * 4 + column],
                "original frame matrix",
            )?;
        }
    }
    Ok(matrix)
}

fn widen_vector(input: &[f32; WIDTH]) -> Result<[f64; WIDTH], String> {
    let mut output = [ZERO_F64; WIDTH];
    for coordinate in 0..WIDTH {
        output[coordinate] = f64::from(finite32(input[coordinate], "R4 source vector")?);
    }
    Ok(output)
}

fn frame_action(
    matrix: &Matrix4,
    input: &[f64; WIDTH],
    transpose: bool,
    stage: &'static str,
) -> Result<[f64; WIDTH], String> {
    let mut output = [ZERO_F64; WIDTH];
    for block in 0..16 {
        for destination in 0..4 {
            let mut sum = ZERO_F64;
            for source in 0..4 {
                let coefficient = if transpose {
                    matrix[source][destination]
                } else {
                    matrix[destination][source]
                };
                let product = mul64(coefficient, input[block * 4 + source], stage)?;
                sum = add64(sum, product, stage)?;
            }
            output[block * 4 + destination] = sum;
        }
    }
    Ok(output)
}

fn connection(destination: &Matrix4, source: &Matrix4) -> Result<Matrix4, String> {
    let mut output = [[ZERO_F64; 4]; 4];
    for row in 0..4 {
        for column in 0..4 {
            let mut sum = ZERO_F64;
            for contracted in 0..4 {
                let product = mul64(
                    destination[contracted][row],
                    source[contracted][column],
                    "R4 connection",
                )?;
                sum = add64(sum, product, "R4 connection")?;
            }
            output[row][column] = sum;
        }
    }
    Ok(output)
}

fn reader_attention(
    weights: &Weights,
    tokens: &[[usize; LENGTH]; CLAUSES],
    lengths: &[usize; CLAUSES],
) -> Result<(Vec<f32>, Vec<usize>), String> {
    let mut attention = vec![ZERO; CLAUSES * ROLES * LENGTH];
    let mut role_argmax = vec![0; CLAUSES * ROLES];
    for clause in 0..CLAUSES {
        let mut embedded = [[ZERO; 32]; LENGTH];
        for position in 0..lengths[clause] {
            let start = tokens[clause][position] * 32;
            for channel in 0..32 {
                embedded[position][channel] = finite32(
                    weights.reader_embedding_weight[start + channel],
                    "reader embedding",
                )?;
            }
        }
        let mut scores = [[MASKED_SCORE; LENGTH]; ROLES];
        // As in the source, all thirteen positions pass through convolution
        // and projection before padded role scores are masked.
        for position in 0..LENGTH {
            let mut hidden = [ZERO; WIDTH];
            for output_channel in 0..WIDTH {
                let mut sum = ZERO;
                for input_channel in 0..32 {
                    for kernel_offset in 0..5 {
                        let padded_position = position + kernel_offset;
                        let value = if (2..LENGTH + 2).contains(&padded_position) {
                            embedded[padded_position - 2][input_channel]
                        } else {
                            ZERO
                        };
                        let weight_index =
                            (output_channel * 32 + input_channel) * 5 + kernel_offset;
                        let product = mul32(
                            weights.reader_context_weight[weight_index],
                            value,
                            "reader convolution",
                        )?;
                        sum = add32(sum, product, "reader convolution")?;
                    }
                }
                let biased = add32(
                    sum,
                    weights.reader_context_bias[output_channel],
                    "reader convolution bias",
                )?;
                hidden[output_channel] = gelu(biased)?;
            }
            let projected = linear::<WIDTH, ROLES>(
                &hidden,
                &weights.reader_role_projection_weight,
                Some(&weights.reader_role_projection_bias),
                "reader role projection",
            )?;
            if position < lengths[clause] {
                for role in 0..ROLES {
                    scores[role][position] = projected[role];
                }
            }
        }
        for role in 0..ROLES {
            let probabilities = softmax(&scores[role], lengths[clause], "reader softmax")?;
            let offset = (clause * ROLES + role) * LENGTH;
            attention[offset..offset + LENGTH].copy_from_slice(&probabilities);
            role_argmax[clause * ROLES + role] =
                first_argmax(&probabilities[..lengths[clause]], "diagnostic role argmax")?;
        }
    }
    Ok((attention, role_argmax))
}

fn pool_roles(
    weights: &Weights,
    frames: &Frames,
    tokens: &[[usize; LENGTH]; CLAUSES],
    lengths: &[usize; CLAUSES],
    attention: &[f32],
    token_frames: &[usize],
    clause_frames: &[usize],
) -> Result<Vec<f32>, String> {
    let mut roles = vec![ZERO; CLAUSES * ROLES * WIDTH];
    for clause in 0..CLAUSES {
        let destination = frame_matrix(frames, clause_frames[clause])?;
        let mut transported = [[ZERO_F64; WIDTH]; LENGTH];
        // Encode and transport each valid token once, reused by all three
        // full mixtures. Padding receives neither a frame lookup nor R4 work.
        for position in 0..lengths[clause] {
            let source = frame_matrix(frames, token_frames[clause * LENGTH + position])?;
            let start = tokens[clause][position] * WIDTH;
            let mut value = [ZERO; WIDTH];
            value.copy_from_slice(&weights.core_embedding_weight[start..start + WIDTH]);
            let model_value = widen_vector(&value)?;
            let encoded = frame_action(&source, &model_value, true, "token R4 encode")?;
            let transport = connection(&destination, &source)?;
            transported[position] =
                frame_action(&transport, &encoded, false, "token R4 transport")?;
        }
        for role in 0..ROLES {
            let mut pooled = [ZERO_F64; WIDTH];
            for block in 0..16 {
                for lane in 0..4 {
                    let coordinate = block * 4 + lane;
                    let mut sum = ZERO_F64;
                    for position in 0..lengths[clause] {
                        let coefficient =
                            f64::from(attention[(clause * ROLES + role) * LENGTH + position]);
                        let product = mul64(
                            coefficient,
                            transported[position][coordinate],
                            "role weighted mixture",
                        )?;
                        sum = add64(sum, product, "role weighted mixture")?;
                    }
                    pooled[coordinate] = sum;
                }
            }
            let decoded = frame_action(&destination, &pooled, false, "role R4 decode")?;
            for coordinate in 0..WIDTH {
                roles[(clause * ROLES + role) * WIDTH + coordinate] =
                    narrow32(decoded[coordinate], "decoded role cast")?;
            }
        }
    }
    Ok(roles)
}

fn binding_context(
    frames: &Frames,
    query: &[f32; WIDTH],
    keys: &[[f32; WIDTH]; 5],
    values: &[[f32; WIDTH]; 5],
    clause_frames: &[usize],
) -> Result<([f32; WIDTH], [f32; 5]), String> {
    let query_frame = frame_matrix(frames, clause_frames[4])?;
    let query_model = widen_vector(query)?;
    let query_local = frame_action(&query_frame, &query_model, true, "query R4 encode")?;
    let mut scores = [ZERO; 5];
    let mut transported_values = [[ZERO_F64; WIDTH]; 5];
    for source_index in 0..5 {
        let frame_index = if source_index < 4 {
            clause_frames[source_index]
        } else {
            frames.identity
        };
        let source_frame = frame_matrix(frames, frame_index)?;
        let key_model = widen_vector(&keys[source_index])?;
        let value_model = widen_vector(&values[source_index])?;
        let key_local = frame_action(&source_frame, &key_model, true, "key R4 encode")?;
        let value_local = frame_action(&source_frame, &value_model, true, "value R4 encode")?;
        let transport = connection(&query_frame, &source_frame)?;
        let transported_key = frame_action(&transport, &key_local, false, "key R4 transport")?;
        transported_values[source_index] =
            frame_action(&transport, &value_local, false, "value R4 transport")?;
        let mut dot = ZERO_F64;
        for block in 0..16 {
            for lane in 0..4 {
                let coordinate = block * 4 + lane;
                let product = mul64(
                    query_local[coordinate],
                    transported_key[coordinate],
                    "binding full-width dot",
                )?;
                dot = add64(dot, product, "binding full-width dot")?;
            }
        }
        // This source boundary is intentional: do not divide the f64 dot or
        // prescale keys before rounding the completed dot to f32.
        let rounded_dot = narrow32(dot, "binding score cast")?;
        scores[source_index] = div32(rounded_dot, EIGHT, "binding score scale")?;
    }
    let attention = softmax(&scores, 5, "binding softmax")?;
    let mut pooled = [ZERO_F64; WIDTH];
    for block in 0..16 {
        for lane in 0..4 {
            let coordinate = block * 4 + lane;
            let mut sum = ZERO_F64;
            for source_index in 0..5 {
                let product = mul64(
                    f64::from(attention[source_index]),
                    transported_values[source_index][coordinate],
                    "binding weighted mixture",
                )?;
                sum = add64(sum, product, "binding weighted mixture")?;
            }
            pooled[coordinate] = sum;
        }
    }
    let decoded = frame_action(&query_frame, &pooled, false, "binding R4 decode")?;
    let mut context = [ZERO; WIDTH];
    for coordinate in 0..WIDTH {
        context[coordinate] = narrow32(decoded[coordinate], "decoded binding cast")?;
    }
    Ok((context, attention))
}

/// Evaluate one admitted row. No file, environment, RNG, target, role label,
/// cached answer or alternate execution path is available to this function.
/// The caller maps a numerical failure to its typed `NUMERICAL_FAILURE` error.
pub(super) fn evaluate(
    weights: &Weights,
    frames: &Frames,
    inputs: &[[i64; 13]; 5],
    lengths: &[usize; 5],
) -> Result<Diagnostics, String> {
    check_layout(weights, frames)?;
    let mut tokens = [[0; LENGTH]; CLAUSES];
    let mut token_frame_indices = vec![frames.identity; CLAUSES * LENGTH];
    let mut clause_frame_indices = vec![frames.identity; CLAUSES];
    let mut current = frames.identity;
    for clause in 0..CLAUSES {
        if lengths[clause] == 0 || lengths[clause] > LENGTH {
            return Err("clause length outside 1..=13".to_owned());
        }
        for position in 0..lengths[clause] {
            let token = usize::try_from(inputs[clause][position])
                .map_err(|_| "negative valid token ID".to_owned())?;
            if token >= VOCABULARY {
                return Err("valid token ID outside the frozen vocabulary".to_owned());
            }
            tokens[clause][position] = token;
            let leaf = frames.token_leaves[token];
            if leaf >= FRAME_COUNT {
                return Err("token leaf outside the frozen atlas".to_owned());
            }
            current = frames.multiplication[current * FRAME_COUNT + leaf];
            if current >= FRAME_COUNT {
                return Err("frame product outside the frozen atlas".to_owned());
            }
            token_frame_indices[clause * LENGTH + position] = current;
        }
        clause_frame_indices[clause] = current;
    }

    let (role_attention, role_argmax) = reader_attention(weights, &tokens, lengths)?;
    let role_vectors = pool_roles(
        weights,
        frames,
        &tokens,
        lengths,
        &role_attention,
        &token_frame_indices,
        &clause_frame_indices,
    )?;

    // Concatenate the complete owner and object mixtures before one 128-wide
    // normalization. The query-location role remains computed but unused.
    let mut compound = [ZERO; 128];
    let query_start = 4 * ROLES * WIDTH;
    compound.copy_from_slice(&role_vectors[query_start..query_start + 128]);
    let query_normalized = layer_norm(&compound, None)?;
    let query = linear::<128, WIDTH>(
        &query_normalized,
        &weights.core_query_projection_weight,
        None,
        "query projection",
    )?;
    let mut keys = [[ZERO; WIDTH]; 5];
    let mut values = [[ZERO; WIDTH]; 5];
    for fact in 0..4 {
        let start = fact * ROLES * WIDTH;
        compound.copy_from_slice(&role_vectors[start..start + 128]);
        let normalized = layer_norm(&compound, None)?;
        keys[fact] = linear::<128, WIDTH>(
            &normalized,
            &weights.core_key_projection_weight,
            None,
            "key projection",
        )?;
        let mut location = [ZERO; WIDTH];
        location.copy_from_slice(&role_vectors[start + 128..start + 192]);
        let location_normalized = layer_norm(&location, None)?;
        values[fact] = linear::<WIDTH, WIDTH>(
            &location_normalized,
            &weights.core_value_projection_weight,
            None,
            "value projection",
        )?;
    }
    keys[4].copy_from_slice(&weights.core_null_key);
    values[4].copy_from_slice(&weights.core_null_value);
    let (context, binding_attention) =
        binding_context(frames, &query, &keys, &values, &clause_frame_indices)?;
    let projected = linear::<WIDTH, WIDTH>(
        &context,
        &weights.core_output_projection_weight,
        None,
        "output projection",
    )?;
    let hidden = layer_norm(
        &projected,
        Some((
            &weights.core_output_norm_weight,
            &weights.core_output_norm_bias,
        )),
    )?;
    let logits = linear::<WIDTH, VOCABULARY>(
        &hidden,
        &weights.core_embedding_weight,
        None,
        "tied vocabulary head",
    )?;
    let selected = first_argmax(&logits, "full vocabulary argmax")?;
    let token_id = u32::try_from(selected)
        .map_err(|_| "vocabulary argmax does not fit the declared ID type".to_owned())?;

    Ok(Diagnostics {
        role_attention,
        role_vectors,
        binding_attention: binding_attention.to_vec(),
        logits: logits.to_vec(),
        role_argmax,
        token_id,
        token_frame_indices,
        clause_frame_indices,
    })
}
