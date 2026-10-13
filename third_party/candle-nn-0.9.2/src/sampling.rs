//! Historical normalized/probability sampling is preserved outside compiled code.
use candle::{Result, Tensor};
pub fn gumbel_softmax<D: candle::shape::Dim>(_: &Tensor, _: f64, _: D) -> Result<Tensor> {
    candle::bail!("D29: historical probability sampling is inactive")
}
