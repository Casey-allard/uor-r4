//! UOR D29: unused vendor rotary operations are inactive.
//! Original implementations remain in historical/src/rotary_emb.rs.
//! Native Hamilton/FBS transport is provided by the native model, not this API.
use candle::{Result, Tensor};

pub fn rope_i(xs: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let _ = (xs, cos, sin);
    candle::bail!("D29: historical vendor rotary operation is inactive; no kernel or host fallback")
}
pub fn rope_i_slow(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let _ = (x, cos, sin);
    candle::bail!("D29: historical vendor rotary operation is inactive; no kernel or host fallback")
}
pub fn rope(xs: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let _ = (xs, cos, sin);
    candle::bail!("D29: historical vendor rotary operation is inactive; no kernel or host fallback")
}
pub fn rope_slow(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let _ = (x, cos, sin);
    candle::bail!("D29: historical vendor rotary operation is inactive; no kernel or host fallback")
}
pub fn rope_thd(xs: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let _ = (xs, cos, sin);
    candle::bail!("D29: historical vendor rotary operation is inactive; no kernel or host fallback")
}
