/// A trained cache memory (the `save=true` output of the `cache-memory`
/// example) to export beside the backbone (lab M4b).
pub struct CacheWeights {
    pub geometry: String,
    pub gap: usize,
    pub dim: usize,
    /// `width x dim` maps, row-major.
    pub query: Vec<f32>,
    pub key: Vec<f32>,
    pub log_beta: f32,
    pub gate_weight: Vec<f32>,
    pub gate_bias: f32,
}

impl CacheWeights {
    /// Read `PATH.safetensors` and the `PATH.json` written beside it.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let meta: Value = serde_json::from_slice(&std::fs::read(path.with_extension("json"))?)?;
        if meta["schema"] != "uor-r4.cache-memory-model/1" {
            return Err(invalid("not a cache-memory model"));
        }
        let geometry = meta["geometry"]
            .as_str()
            .ok_or_else(|| invalid("cache geometry missing"))?
            .to_owned();
        let number = |key: &str| -> Result<usize> {
            meta[key]
                .as_u64()
                .map(|v| v as usize)
                .ok_or_else(|| invalid(format!("cache {key} missing")))
        };
        let (gap, dim, width) = (number("gap")?, number("dim")?, number("width")?);
        let tensors = candle_core::safetensors::load(path, &candle_core::Device::Cpu)?;
        let get = |name: &str, len: usize| -> Result<Vec<f32>> {
            let values = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("cache tensor {name} missing")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if values.len() != len || values.iter().any(|v| !v.is_finite()) {
                return Err(invalid(format!("cache tensor {name} has the wrong size")));
            }
            Ok(values)
        };
        Ok(Self {
            query: get("query", width * dim)?,
            key: get("key", width * dim)?,
            log_beta: get("log_beta", 1)?[0],
            gate_weight: get("gate_weight", width)?,
            gate_bias: get("gate_bias", 1)?[0],
            geometry,
            gap,
            dim,
        })
    }
}

/// `(m, e)` with `(16 + m) 2^(e - 4)` nearest to `value > 0`.
fn grid_nearest(value: f64) -> (u8, i32) {
    let e = value.log2().floor() as i32;
    let m = (value / 2f64.powi(e - 4)).round() as i32 - 16;
    if m >= 16 {
        (0, e + 1)
    } else {
        (m.clamp(0, 15) as u8, e)
    }
}

/// Export `checkpoint` for integer serving with sessions of up to
/// `max_positions` tokens, with GPTQ when `calibration` gives input moments
/// (and its damping), and a trained Lorentz cache when `cache` is given.
/// Returns the artifact bytes and a quantization report.
pub fn export_llama(
    checkpoint: &Checkpoint,
    max_positions: usize,
    source: Value,
    calibration: Option<(&Calibration, f64)>,
    cache: Option<&CacheWeights>,
) -> Result<(Vec<u8>, Value)> {
