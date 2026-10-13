//! Historical CPU attention source is preserved but excluded from compilation.
use candle::{Result, Tensor, WithDType};
use std::iter::Sum;
pub fn run_flash_attn_cpu<T>(
    _: &Tensor,
    _: &Tensor,
    _: &Tensor,
    _: Option<&Tensor>,
    _: f32,
    _: Option<f32>,
    _: Option<f32>,
) -> Result<Tensor>
where
    T: WithDType + Sum + num_traits::real::Real,
{
    candle::bail!("D24/D29: CPU model attention is inactive; no host fallback")
}
