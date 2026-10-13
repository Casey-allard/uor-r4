//! Host storage and serialization only. D24/D29 exclude CPU model arithmetic.
//! Upstream computational implementations are preserved outside the module graph.
use crate::backend::BackendDevice;
use crate::op::{BinaryOpT, CmpOp, ReduceOp, UnaryOpT};
use crate::{DType, Error, Layout, Result, Shape, WithDType};
use float8::F8E4M3;
use half::{bf16, f16};
mod utils;
pub use utils::{Map1, Map1Any, Map2, Map2InPlace, Map2U8};
pub(super) fn refuse_cpu<T>() -> Result<T> {
    crate::bail!("D24/D29: CPU model compute is unavailable; use the admitted CUDA geometric path")
}

// These helpers move existing bytes for host staging/serialization. They do
// not evaluate callbacks, numeric transforms, comparisons or reductions.
fn copy_strided<T: Copy>(src: &[T], dst: &mut [T], offset: usize, layout: &Layout) -> Result<()> {
    let count = layout.shape().elem_count();
    let end = offset
        .checked_add(count)
        .ok_or_else(|| Error::Msg("host copy offset overflow".into()).bt())?;
    if end > dst.len() || layout.strided_index().any(|i| i >= src.len()) {
        crate::bail!("host copy layout exceeds storage")
    }
    for (target, source) in dst[offset..end].iter_mut().zip(layout.strided_index()) {
        *target = src[source];
    }
    Ok(())
}

fn copy_rows<T: Copy>(
    src: &[T],
    dst: &mut [T],
    rows: usize,
    cols: usize,
    src_stride: usize,
    dst_stride: usize,
    src_offset: usize,
    dst_offset: usize,
) -> Result<()> {
    let extent = |offset: usize, stride: usize| -> Result<usize> {
        offset
            .checked_add(
                rows.saturating_sub(1)
                    .checked_mul(stride)
                    .ok_or_else(|| Error::Msg("host copy stride overflow".into()).bt())?,
            )
            .and_then(|n| n.checked_add(cols))
            .ok_or_else(|| Error::Msg("host copy extent overflow".into()).bt())
    };
    if rows == 0 || cols == 0 {
        return Ok(());
    }
    if extent(src_offset, src_stride)? > src.len() || extent(dst_offset, dst_stride)? > dst.len() {
        crate::bail!("host copy rows exceed storage")
    }
    for row in 0..rows {
        let s = src_offset + row * src_stride;
        let d = dst_offset + row * dst_stride;
        dst[d..d + cols].copy_from_slice(&src[s..s + cols]);
    }
    Ok(())
}

macro_rules! copy_storage {
    ($src:expr, $dst:expr, $helper:ident, $($arg:expr),*) => {
        match ($src, $dst) {
            (CpuStorage::U8(s), CpuStorage::U8(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::U32(s), CpuStorage::U32(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::I16(s), CpuStorage::I16(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::I32(s), CpuStorage::I32(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::I64(s), CpuStorage::I64(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::BF16(s), CpuStorage::BF16(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F16(s), CpuStorage::F16(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F32(s), CpuStorage::F32(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F64(s), CpuStorage::F64(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F8E4M3(s), CpuStorage::F8E4M3(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F6E2M3(s), CpuStorage::F6E2M3(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F6E3M2(s), CpuStorage::F6E3M2(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F4(s), CpuStorage::F4(d)) => $helper(s, d, $($arg),*),
            (CpuStorage::F8E8M0(s), CpuStorage::F8E8M0(d)) => $helper(s, d, $($arg),*),
            _ => crate::bail!("host copy dtype mismatch"),
        }
    };
}
#[derive(Debug, Clone)]
pub enum CpuStorage {
    U8(Vec<u8>),
    U32(Vec<u32>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    BF16(Vec<bf16>),
    F16(Vec<f16>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    F8E4M3(Vec<F8E4M3>),
    // Dummy types that store raw bytes
    F6E2M3(Vec<u8>),
    F6E3M2(Vec<u8>),
    F4(Vec<u8>),
    F8E8M0(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum CpuStorageRef<'a> {
    U8(&'a [u8]),
    U32(&'a [u32]),
    I16(&'a [i16]),
    I32(&'a [i32]),
    I64(&'a [i64]),
    BF16(&'a [bf16]),
    F16(&'a [f16]),
    F32(&'a [f32]),
    F64(&'a [f64]),
    F8E4M3(&'a [F8E4M3]),
    // Dummy types that store raw bytes
    F6E2M3(&'a [u8]),
    F6E3M2(&'a [u8]),
    F4(&'a [u8]),
    F8E8M0(&'a [u8]),
}

#[derive(Debug, Clone)]
pub struct CpuDevice;

impl CpuStorage {
    pub fn as_slice<D: WithDType>(&self) -> Result<&[D]> {
        D::cpu_storage_as_slice(self)
    }

    pub fn concat(storages: &[CpuStorage]) -> Result<CpuStorage> {
        let storage0 = storages
            .first()
            .ok_or_else(|| Error::Msg("empty CPU storage concatenation".into()).bt())?;
        let s = match storage0 {
            Self::U8(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::U8(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::U8(storages)
            }
            Self::U32(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::U32(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::U32(storages)
            }
            Self::I16(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::I16(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::I16(storages)
            }
            Self::I32(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::I32(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::I32(storages)
            }
            Self::I64(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::I64(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::I64(storages)
            }
            Self::BF16(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::BF16(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::BF16(storages)
            }
            Self::F16(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F16(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F16(storages)
            }
            Self::F32(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F32(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F32(storages)
            }
            Self::F64(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F64(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F64(storages)
            }
            Self::F8E4M3(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F8E4M3(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F8E4M3(storages)
            }
            Self::F6E2M3(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F6E2M3(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F6E2M3(storages)
            }
            Self::F6E3M2(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F6E3M2(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F6E3M2(storages)
            }
            Self::F4(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F4(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F4(storages)
            }
            Self::F8E8M0(_) => {
                let storages = storages
                    .iter()
                    .map(|s| match s {
                        Self::F8E8M0(s) => Ok(s.as_slice()),
                        _ => crate::bail!("dtype mismatch"),
                    })
                    .collect::<Result<Vec<_>>>()?
                    .concat();
                Self::F8E8M0(storages)
            }
        };
        Ok(s)
    }
}
impl crate::backend::BackendStorage for CpuStorage {
    type Device = CpuDevice;

    fn try_clone(&self, _: &Layout) -> Result<Self> {
        Ok(self.clone())
    }

    fn dtype(&self) -> DType {
        match self {
            Self::U8(_) => DType::U8,
            Self::U32(_) => DType::U32,
            Self::I16(_) => DType::I16,
            Self::I32(_) => DType::I32,
            Self::I64(_) => DType::I64,
            Self::BF16(_) => DType::BF16,
            Self::F16(_) => DType::F16,
            Self::F32(_) => DType::F32,
            Self::F64(_) => DType::F64,
            Self::F8E4M3(_) => DType::F8E4M3,
            Self::F6E2M3(_) => DType::F6E2M3,
            Self::F6E3M2(_) => DType::F6E3M2,
            Self::F4(_) => DType::F4,
            Self::F8E8M0(_) => DType::F8E8M0,
        }
    }

    fn device(&self) -> &CpuDevice {
        &CpuDevice
    }

    fn const_set(&mut self, _: crate::scalar::Scalar, _: &Layout) -> Result<()> {
        refuse_cpu()
    }

    fn to_cpu_storage(&self) -> Result<CpuStorage> {
        Ok(self.clone())
    }

    fn affine(&self, _: &Layout, _: f64, _: f64) -> Result<Self> {
        refuse_cpu()
    }

    fn powf(&self, _: &Layout, _: f64) -> Result<Self> {
        refuse_cpu()
    }

    fn elu(&self, _: &Layout, _: f64) -> Result<Self> {
        refuse_cpu()
    }

    fn reduce_op(&self, _: ReduceOp, _: &Layout, _: &[usize]) -> Result<Self> {
        refuse_cpu()
    }

    fn cmp(&self, _: CmpOp, _: &Self, _: &Layout, _: &Layout) -> Result<Self> {
        refuse_cpu()
    }

    fn to_dtype(&self, _: &Layout, _: DType) -> Result<Self> {
        refuse_cpu()
    }

    fn unary_impl<B: UnaryOpT>(&self, _: &Layout) -> Result<Self> {
        refuse_cpu()
    }

    fn binary_impl<B: BinaryOpT>(&self, _: &Self, _: &Layout, _: &Layout) -> Result<Self> {
        refuse_cpu()
    }

    fn where_cond(&self, _: &Layout, _: &Self, _: &Layout, _: &Self, _: &Layout) -> Result<Self> {
        refuse_cpu()
    }

    fn conv1d(
        &self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &crate::conv::ParamsConv1D,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn conv_transpose1d(
        &self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &crate::conv::ParamsConvTranspose1D,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn conv2d(
        &self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &crate::conv::ParamsConv2D,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn conv_transpose2d(
        &self,
        _l: &Layout,
        _kernel: &Self,
        _kernel_l: &Layout,
        _params: &crate::conv::ParamsConvTranspose2D,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn index_select(&self, _: &Self, _: &Layout, _: &Layout, _: usize) -> Result<Self> {
        refuse_cpu()
    }
    fn gather(&self, _: &Layout, _: &Self, _: &Layout, _: usize) -> Result<Self> {
        refuse_cpu()
    }

    fn scatter_set(
        &mut self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: usize,
    ) -> Result<()> {
        refuse_cpu()
    }

    fn scatter_add_set(
        &mut self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: usize,
    ) -> Result<()> {
        refuse_cpu()
    }

    fn index_add(
        &self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: &Self,
        _: &Layout,
        _: usize,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn matmul(
        &self,
        _: &Self,
        _: (usize, usize, usize, usize),
        _: &Layout,
        _: &Layout,
    ) -> Result<Self> {
        refuse_cpu()
    }

    fn copy_strided_src(&self, dst: &mut Self, offset: usize, layout: &Layout) -> Result<()> {
        copy_storage!(self, dst, copy_strided, offset, layout)
    }

    fn copy2d(
        &self,
        dst: &mut Self,
        rows: usize,
        cols: usize,
        src_stride: usize,
        dst_stride: usize,
        src_offset: usize,
        dst_offset: usize,
    ) -> Result<()> {
        copy_storage!(
            self, dst, copy_rows, rows, cols, src_stride, dst_stride, src_offset, dst_offset
        )
    }

    fn avg_pool2d(&self, _: &Layout, _: (usize, usize), _: (usize, usize)) -> Result<Self> {
        refuse_cpu()
    }

    fn max_pool2d(&self, _: &Layout, _: (usize, usize), _: (usize, usize)) -> Result<Self> {
        refuse_cpu()
    }

    fn upsample_nearest1d(&self, _: &Layout, _: usize) -> Result<Self> {
        refuse_cpu()
    }

    fn upsample_nearest2d(&self, _: &Layout, _: usize, _: usize) -> Result<Self> {
        refuse_cpu()
    }

    fn upsample_bilinear2d(
        &self,
        _: &Layout,
        _: usize,
        _: usize,
        _: bool,
        _: Option<f64>,
        _: Option<f64>,
    ) -> Result<Self> {
        refuse_cpu()
    }
}
impl crate::backend::BackendDevice for CpuDevice {
    type Storage = CpuStorage;
    fn new(_: usize) -> Result<Self> {
        Ok(Self)
    }

    fn set_seed(&self, _: u64) -> Result<()> {
        refuse_cpu()
    }

    fn get_current_seed(&self) -> Result<u64> {
        refuse_cpu()
    }

    fn location(&self) -> crate::DeviceLocation {
        crate::DeviceLocation::Cpu
    }

    fn same_device(&self, _: &Self) -> bool {
        true
    }

    fn zeros_impl(&self, shape: &Shape, dtype: DType) -> Result<CpuStorage> {
        let elem_count = shape.elem_count();
        let storage = match dtype {
            DType::U8 => CpuStorage::U8(vec![0u8; elem_count]),
            DType::U32 => CpuStorage::U32(vec![0u32; elem_count]),
            DType::I16 => CpuStorage::I16(vec![0i16; elem_count]),
            DType::I32 => CpuStorage::I32(vec![0i32; elem_count]),
            DType::I64 => CpuStorage::I64(vec![0i64; elem_count]),
            DType::BF16 => CpuStorage::BF16(vec![bf16::ZERO; elem_count]),
            DType::F16 => CpuStorage::F16(vec![f16::ZERO; elem_count]),
            DType::F32 => CpuStorage::F32(vec![0f32; elem_count]),
            DType::F64 => CpuStorage::F64(vec![0f64; elem_count]),
            DType::F8E4M3 => CpuStorage::F8E4M3(vec![F8E4M3::ZERO; elem_count]),
            DType::F6E2M3 | DType::F6E3M2 | DType::F4 | DType::F8E8M0 => {
                return Err(Error::UnsupportedDTypeForOp(dtype, "zeros").bt())
            }
        };
        Ok(storage)
    }

    unsafe fn alloc_uninit(&self, shape: &Shape, dtype: DType) -> Result<CpuStorage> {
        self.zeros_impl(shape, dtype)
    }

    fn storage_from_slice<T: WithDType>(&self, s: &[T]) -> Result<CpuStorage> {
        Ok(T::to_cpu_storage(s))
    }

    fn storage_from_cpu_storage(&self, s: &CpuStorage) -> Result<CpuStorage> {
        Ok(s.clone())
    }

    fn storage_from_cpu_storage_owned(&self, s: CpuStorage) -> Result<CpuStorage> {
        Ok(s)
    }

    fn rand_uniform(&self, _: &Shape, _: DType, _: f64, _: f64) -> Result<Self::Storage> {
        refuse_cpu()
    }

    fn rand_normal(&self, _: &Shape, _: DType, _: f64, _: f64) -> Result<Self::Storage> {
        refuse_cpu()
    }

    fn synchronize(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod d29_contract_tests {
    use super::*;
    use crate::backend::BackendStorage;

    #[test]
    fn host_storage_retains_bytes_and_cpu_math_refuses() -> Result<()> {
        let device = CpuDevice;
        let src = device.storage_from_slice(&[11u32, 17, 23])?;
        let layout = Layout::contiguous(crate::Shape::from((3,)));
        let mut dst = device.zeros_impl(&crate::Shape::from((4,)), DType::U32)?;
        src.copy_strided_src(&mut dst, 1, &layout)?;
        assert_eq!(dst.as_slice::<u32>()?, &[0, 11, 17, 23]);
        assert!(src.copy_strided_src(&mut dst, 2, &layout).is_err());
        assert!(src.affine(&layout, 1., 0.).is_err());
        assert!(src.matmul(&src, (1, 1, 3, 1), &layout, &layout).is_err());
        assert!(CpuStorage::concat(&[]).is_err());
        Ok(())
    }

    #[test]
    fn host_custom_callback_is_never_dispatched() -> Result<()> {
        struct Refused;
        impl crate::CustomOp1 for Refused {
            fn name(&self) -> &'static str {
                "must-not-run-host-callback"
            }
            fn cpu_fwd(&self, _: &CpuStorage, _: &Layout) -> Result<(CpuStorage, Shape)> {
                CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
                crate::bail!("unexpected CPU callback")
            }
        }
        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let storage = crate::Storage::Cpu(CpuStorage::U32(vec![11]));
        let layout = Layout::contiguous(crate::Shape::from((1,)));
        assert!(storage.apply_op1(&layout, &Refused).is_err());
        assert!(!CALLED.load(std::sync::atomic::Ordering::SeqCst));
        Ok(())
    }
}
