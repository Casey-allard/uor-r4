//! Compatibility traits; CPU dispatch refuses before calling scalar callbacks.
use crate::{Layout, Result, WithDType};
type C = super::CpuStorage;
pub trait Map1 {
    fn f<T: WithDType>(&self, vs: &[T], layout: &Layout) -> Result<Vec<T>>;

    fn map(&self, vs: &C, layout: &Layout) -> Result<C> {
        super::refuse_cpu()
    }
}
pub trait Map1Any {
    fn f<T: WithDType, W: Fn(Vec<T>) -> C>(&self, vs: &[T], layout: &Layout, wrap: W) -> Result<C>;

    fn map(&self, vs: &C, layout: &Layout) -> Result<C> {
        super::refuse_cpu()
    }
}
pub trait Map2 {
    const OP: &'static str;
    fn f<T: WithDType>(&self, v1: &[T], l1: &Layout, v2: &[T], l2: &Layout) -> Result<Vec<T>>;

    fn map(&self, v1: &C, l1: &Layout, v2: &C, l2: &Layout) -> Result<C> {
        super::refuse_cpu()
    }
}
pub trait Map2InPlace {
    const OP: &'static str;
    fn f<T: WithDType>(&self, v1: &mut [T], l1: &Layout, v2: &[T], l2: &Layout) -> Result<()>;

    fn map(&self, v1: &mut C, l1: &Layout, v2: &C, l2: &Layout) -> Result<()> {
        super::refuse_cpu()
    }
}
pub trait Map2U8 {
    const OP: &'static str;
    fn f<T: WithDType>(&self, v1: &[T], l1: &Layout, v2: &[T], l2: &Layout) -> Result<Vec<u8>>;

    fn map(&self, v1: &C, l1: &Layout, v2: &C, l2: &Layout) -> Result<C> {
        super::refuse_cpu()
    }
}
