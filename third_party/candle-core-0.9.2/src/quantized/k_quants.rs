//! Preserved GGML block layouts and metadata. CPU quantization and contraction
//! implementations are archived outside the compiled module surface.
use super::GgmlDType;
use crate::Result;
use half::{bf16, f16};

// Default to QK_K 256 rather than 64.
pub const QK_K: usize = 256;
pub const K_SCALE_SIZE: usize = 12;

pub const QK4_0: usize = 32;
pub const QK4_1: usize = 32;
pub const QK5_0: usize = 32;
pub const QK5_1: usize = 32;
pub const QK8_0: usize = 32;
pub const QK8_1: usize = 32;

/// Block storage metadata, with no CPU scalar conversion or dot-product API.
pub trait GgmlType: Sized + Clone + Send + Sync {
    const DTYPE: GgmlDType;
    const BLCK_SIZE: usize;
    const DIRECT_COPY: bool = false;
    type VecDotType: GgmlType;

    // This is only safe for types that include immediate values such as float/int/...
    fn zeros() -> Self {
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ4_0 {
    pub(crate) d: f16,
    pub(crate) qs: [u8; QK4_0 / 2],
}
const _: () = assert!(std::mem::size_of::<BlockQ4_0>() == 18);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ4_1 {
    pub(crate) d: f16,
    pub(crate) m: f16,
    pub(crate) qs: [u8; QK4_1 / 2],
}
const _: () = assert!(std::mem::size_of::<BlockQ4_1>() == 20);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ5_0 {
    pub(crate) d: f16,
    pub(crate) qh: [u8; 4],
    pub(crate) qs: [u8; QK5_0 / 2],
}
const _: () = assert!(std::mem::size_of::<BlockQ5_0>() == 22);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ5_1 {
    pub(crate) d: f16,
    pub(crate) m: f16,
    pub(crate) qh: [u8; 4],
    pub(crate) qs: [u8; QK5_1 / 2],
}
const _: () = assert!(std::mem::size_of::<BlockQ5_1>() == 24);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ8_0 {
    pub(crate) d: f16,
    pub(crate) qs: [i8; QK8_0],
}
const _: () = assert!(std::mem::size_of::<BlockQ8_0>() == 34);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ8_1 {
    pub(crate) d: f16,
    pub(crate) s: f16,
    pub(crate) qs: [i8; QK8_1],
}
const _: () = assert!(std::mem::size_of::<BlockQ8_1>() == 36);

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ2K {
    pub(crate) scales: [u8; QK_K / 16],
    pub(crate) qs: [u8; QK_K / 4],
    pub(crate) d: f16,
    pub(crate) dmin: f16,
}
const _: () = assert!(QK_K / 16 + QK_K / 4 + 2 * 2 == std::mem::size_of::<BlockQ2K>());

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ3K {
    pub(crate) hmask: [u8; QK_K / 8],
    pub(crate) qs: [u8; QK_K / 4],
    pub(crate) scales: [u8; 12],
    pub(crate) d: f16,
}
const _: () = assert!(QK_K / 8 + QK_K / 4 + 12 + 2 == std::mem::size_of::<BlockQ3K>());

#[derive(Debug, Clone, PartialEq)]
// https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/k_quants.h#L82
#[repr(C)]
pub struct BlockQ4K {
    pub(crate) d: f16,
    pub(crate) dmin: f16,
    pub(crate) scales: [u8; K_SCALE_SIZE],
    pub(crate) qs: [u8; QK_K / 2],
}
const _: () = assert!(QK_K / 2 + K_SCALE_SIZE + 2 * 2 == std::mem::size_of::<BlockQ4K>());

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ5K {
    pub(crate) d: f16,
    pub(crate) dmin: f16,
    pub(crate) scales: [u8; K_SCALE_SIZE],
    pub(crate) qh: [u8; QK_K / 8],
    pub(crate) qs: [u8; QK_K / 2],
}
const _: () =
    assert!(QK_K / 8 + QK_K / 2 + 2 * 2 + K_SCALE_SIZE == std::mem::size_of::<BlockQ5K>());

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ6K {
    pub(crate) ql: [u8; QK_K / 2],
    pub(crate) qh: [u8; QK_K / 4],
    pub(crate) scales: [i8; QK_K / 16],
    pub(crate) d: f16,
}
const _: () = assert!(3 * QK_K / 4 + QK_K / 16 + 2 == std::mem::size_of::<BlockQ6K>());

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct BlockQ8K {
    pub(crate) d: f32,
    pub(crate) qs: [i8; QK_K],
    pub(crate) bsums: [i16; QK_K / 16],
}
const _: () = assert!(4 + QK_K + QK_K / 16 * 2 == std::mem::size_of::<BlockQ8K>());

impl GgmlType for BlockQ4_0 {
    const DTYPE: GgmlDType = GgmlDType::Q4_0;
    const BLCK_SIZE: usize = QK4_0;
    type VecDotType = BlockQ8_0;

    // https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/ggml.c#L1525

    // https://github.com/ggerganov/llama.cpp/blob/b5ffb2849d23afe73647f68eec7b68187af09be6/ggml.c#L2361C10-L2361C122
}

impl GgmlType for BlockQ4_1 {
    const DTYPE: GgmlDType = GgmlDType::Q4_1;
    const BLCK_SIZE: usize = QK4_1;
    type VecDotType = BlockQ8_1;

    // https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/ggml.c#L1545
}

impl GgmlType for BlockQ5_0 {
    const DTYPE: GgmlDType = GgmlDType::Q5_0;
    const BLCK_SIZE: usize = QK5_0;
    type VecDotType = BlockQ8_0;

    // https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/ggml.c#L1566
}

impl GgmlType for BlockQ5_1 {
    const DTYPE: GgmlDType = GgmlDType::Q5_1;
    const BLCK_SIZE: usize = QK5_1;
    type VecDotType = BlockQ8_1;

    // https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/ggml.c#L1592
}

impl GgmlType for BlockQ8_0 {
    const DTYPE: GgmlDType = GgmlDType::Q8_0;
    const BLCK_SIZE: usize = QK8_0;
    type VecDotType = BlockQ8_0;

    // https://github.com/ggerganov/llama.cpp/blob/468ea24fb4633a0d681f7ac84089566c1c6190cb/ggml.c#L1619
}

impl GgmlType for BlockQ8_1 {
    const DTYPE: GgmlDType = GgmlDType::Q8_1;
    const BLCK_SIZE: usize = QK8_1;
    type VecDotType = BlockQ8_1;
}

impl GgmlType for BlockQ2K {
    const DTYPE: GgmlDType = GgmlDType::Q2K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L279

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L354
}

impl GgmlType for BlockQ3K {
    const DTYPE: GgmlDType = GgmlDType::Q3K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L533
}

impl GgmlType for BlockQ4K {
    const DTYPE: GgmlDType = GgmlDType::Q4K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L735
}

// https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L928
impl GgmlType for BlockQ5K {
    const DTYPE: GgmlDType = GgmlDType::Q5K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L793

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L928
}

impl GgmlType for BlockQ6K {
    const DTYPE: GgmlDType = GgmlDType::Q6K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;

    // https://github.com/ggerganov/llama.cpp/blob/8183159cf3def112f6d1fe94815fce70e1bffa12/k_quants.c#L1067
}

impl GgmlType for BlockQ8K {
    const DTYPE: GgmlDType = GgmlDType::Q8K;
    const BLCK_SIZE: usize = QK_K;
    type VecDotType = BlockQ8K;
}

// https://github.com/ggml-org/llama.cpp/blob/aa3ee0eb0b80efca126cedf9bcb4fb5864b46ce3/ggml/src/ggml-cpu/ggml-cpu.c#L1205
pub fn matmul<T: GgmlType>(
    (m, k, n): (usize, usize, usize),
    lhs: &[f32],
    rhs_t: &[T],
    dst: &mut [f32],
) -> Result<()> {
    let _ = (m, k, n, lhs, rhs_t, dst);
    crate::bail!(
        "D29: CPU quantized matrix multiplication is forbidden; no host or vendor fallback"
    )
}
pub fn matmul_f16<T: GgmlType>(
    mkn: (usize, usize, usize),
    lhs: &[f16],
    rhs_t: &[T],
    dst: &mut [f16],
) -> Result<()> {
    let _ = (mkn, lhs, rhs_t, dst);
    crate::bail!(
        "D29: CPU quantized matrix multiplication is forbidden; no host or vendor fallback"
    )
}
impl GgmlType for f32 {
    const DTYPE: GgmlDType = GgmlDType::F32;
    const BLCK_SIZE: usize = 1;
    const DIRECT_COPY: bool = true;
    type VecDotType = f32;
}

impl GgmlType for f16 {
    const DTYPE: GgmlDType = GgmlDType::F16;
    const BLCK_SIZE: usize = 1;
    const DIRECT_COPY: bool = true;
    type VecDotType = f16;
}

impl GgmlType for bf16 {
    const DTYPE: GgmlDType = GgmlDType::BF16;
    const BLCK_SIZE: usize = 1;
    const DIRECT_COPY: bool = true;
    type VecDotType = bf16;
}

macro_rules! verify_block_size {
    ( $block_type:ident ) => {
        const _: () =
            assert!($block_type::BLCK_SIZE == <$block_type as GgmlType>::VecDotType::BLCK_SIZE);
    };
}

macro_rules! verify_block_sizes {
    ( $( $block_type:ident ),* ) => {
        $(
            verify_block_size!($block_type);
        )*
    };
}

verify_block_sizes!(
    BlockQ4_0, BlockQ4_1, BlockQ5_0, BlockQ5_1, BlockQ8_0, BlockQ8_1, BlockQ2K, BlockQ3K, BlockQ4K,
    BlockQ5K, BlockQ6K, BlockQ8K, f32, f16, bf16
);

#[cfg(test)]
mod refusal_tests {
    #[test]
    fn quantized_matmul_refuses_without_touching_outputs() {
        let mut output = [7.0f32];
        assert!(super::matmul::<f32>((1, 1, 1), &[2.0], &[3.0], &mut output).is_err());
        assert_eq!(output, [7.0]);
        let mut half_output = [half::f16::from_f32(7.0)];
        assert!(super::matmul_f16::<f32>(
            (1, 1, 1),
            &[half::f16::from_f32(2.0)],
            &[3.0],
            &mut half_output
        )
        .is_err());
        assert_eq!(half_output[0].to_bits(), half::f16::from_f32(7.0).to_bits());
    }
}
