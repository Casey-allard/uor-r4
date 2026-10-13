//! Dtype compatibility without CPU model dot/reduction implementations.
pub trait VecOps: num_traits::NumAssign + Copy {
    fn min(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
}
impl VecOps for f32 {
    fn min(self, other: Self) -> Self {
        Self::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        Self::max(self, other)
    }
}
impl VecOps for half::f16 {
    fn min(self, other: Self) -> Self {
        Self::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        Self::max(self, other)
    }
}
impl VecOps for f64 {
    fn min(self, other: Self) -> Self {
        Self::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        Self::max(self, other)
    }
}
impl VecOps for half::bf16 {
    fn min(self, other: Self) -> Self {
        Self::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        Self::max(self, other)
    }
}
impl VecOps for u8 {
    fn min(self, other: Self) -> Self {
        <Self as Ord>::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        <Self as Ord>::max(self, other)
    }
}
impl VecOps for u32 {
    fn min(self, other: Self) -> Self {
        <Self as Ord>::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        <Self as Ord>::max(self, other)
    }
}
impl VecOps for i16 {
    fn min(self, other: Self) -> Self {
        <Self as Ord>::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        <Self as Ord>::max(self, other)
    }
}
impl VecOps for i32 {
    fn min(self, other: Self) -> Self {
        <Self as Ord>::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        <Self as Ord>::max(self, other)
    }
}
impl VecOps for i64 {
    fn min(self, other: Self) -> Self {
        <Self as Ord>::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        <Self as Ord>::max(self, other)
    }
}
impl VecOps for float8::F8E4M3 {
    fn min(self, other: Self) -> Self {
        Self::min(self, other)
    }
    fn max(self, other: Self) -> Self {
        Self::max(self, other)
    }
}
