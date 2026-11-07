//! One-pass exact selection of a row's best allowed positions.
//!
//! `in_space_long::best_positions` made `k` whole-vocabulary `argmax` calls and masked one result
//! after each. The selector here performs one scan into a warp-local top-k, then merges the 32
//! lane lists. Its order is score descending and token id ascending on ties. Non-tied choices
//! match repeated argmax; ties are made deterministic. It lives in its own CUDA module so the
//! pinned stack sources and checkpoint fingerprints stay unchanged.

use candle_core::{CpuStorage, CustomOp2, CustomOp3, Layout, Shape, Tensor};

type CResult<T> = candle_core::Result<T>;

pub(crate) const MAX_RIVALS: usize = 8;

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
struct TopEight {
    k: usize,
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
struct NearestTopEight {
    k: usize,
}

/// The `k` best positions of each `scores` row among the nonzero `allowed` entries, as
/// `(rows, k)`, by score descending and token id ascending on ties.
pub(crate) fn positions(scores: &Tensor, allowed: &Tensor, k: usize) -> CResult<Tensor> {
    if k == 0 || k > MAX_RIVALS {
        candle_core::bail!("the exact rival selector takes 1 to {MAX_RIVALS} positions");
    }
    scores.apply_op2_no_bwd(allowed, &TopEight { k })
}

/// The `k` nearest non-target table rows to each `query` row in negative mean L1, as
/// `(rows, k)`, computed without writing the `(rows, vocabulary)` score matrix.
pub(crate) fn nearest_positions(
    query: &Tensor,
    table: &Tensor,
    targets: &Tensor,
    k: usize,
) -> CResult<Tensor> {
    if k == 0 || k > MAX_RIVALS {
        candle_core::bail!("the fused rival selector takes 1 to {MAX_RIVALS} positions");
    }
    query.apply_op3_no_bwd(table, targets, &NearestTopEight { k })
}

impl CustomOp2 for TopEight {
    fn name(&self) -> &'static str {
        "top8-positions"
    }

    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> CResult<(CpuStorage, Shape)> {
        candle_core::bail!(
            "D24/D29: the exact rival selector is a CUDA training kernel; no CPU fallback"
        )
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        scores: &candle_core::CudaStorage,
        scores_layout: &Layout,
        allowed: &candle_core::CudaStorage,
        allowed_layout: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        cuda::positions(scores, scores_layout, allowed, allowed_layout, self.k)
    }
}

impl CustomOp3 for NearestTopEight {
    fn name(&self) -> &'static str {
        "nearest-top8-positions"
    }

    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> CResult<(CpuStorage, Shape)> {
        candle_core::bail!(
            "D24/D29: the fused rival selector is a CUDA training kernel; no CPU fallback"
        )
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        query: &candle_core::CudaStorage,
        query_layout: &Layout,
        table: &candle_core::CudaStorage,
        table_layout: &Layout,
        targets: &candle_core::CudaStorage,
        targets_layout: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        cuda::nearest_positions(
            query,
            query_layout,
            table,
            table_layout,
            targets,
            targets_layout,
            self.k,
        )
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use candle_core::backend::BackendStorage;
    use candle_core::{CudaStorage, DType, Layout, Shape};
    use cudarc::driver::CudaView;

    use crate::cuda_stack_kernels::cuda::{launch_groups_prebuilt, uninit, Arg};
    use crate::cuda_stack_kernels::TOP8_POSITIONS_MODULE;

    use super::{CResult, MAX_RIVALS};

    const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/top8_positions_f32.ptx"));

    fn f32_view<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<CudaView<'a, f32>> {
        if storage.dtype() != DType::F32 {
            candle_core::bail!("the exact rival selector needs an f32 input");
        }
        let Some((start, end)) = layout.contiguous_offsets() else {
            candle_core::bail!("the exact rival selector needs a contiguous f32 input");
        };
        match storage.as_cuda_slice::<f32>()?.try_slice(start..end) {
            Some(view) => Ok(view),
            None => candle_core::bail!("the exact rival selector's input exceeds its buffer"),
        }
    }

    fn u8_view<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<CudaView<'a, u8>> {
        if storage.dtype() != DType::U8 {
            candle_core::bail!("the exact rival selector needs a u8 allowed mask");
        }
        let Some((start, end)) = layout.contiguous_offsets() else {
            candle_core::bail!("the exact rival selector needs a contiguous allowed mask");
        };
        match storage.as_cuda_slice::<u8>()?.try_slice(start..end) {
            Some(view) => Ok(view),
            None => candle_core::bail!("the exact rival selector's mask exceeds its buffer"),
        }
    }

    fn u32_view<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<CudaView<'a, u32>> {
        if storage.dtype() != DType::U32 {
            candle_core::bail!("the fused rival selector needs u32 target ids");
        }
        let Some((start, end)) = layout.contiguous_offsets() else {
            candle_core::bail!("the fused rival selector needs contiguous target ids");
        };
        match storage.as_cuda_slice::<u32>()?.try_slice(start..end) {
            Some(view) => Ok(view),
            None => candle_core::bail!("the fused rival selector's targets exceed their buffer"),
        }
    }

    pub(super) fn positions(
        scores: &CudaStorage,
        scores_layout: &Layout,
        allowed: &CudaStorage,
        allowed_layout: &Layout,
        k: usize,
    ) -> CResult<(CudaStorage, Shape)> {
        if !(1..=MAX_RIVALS).contains(&k) {
            candle_core::bail!("the exact rival selector takes 1 to {MAX_RIVALS} positions");
        }
        let [rows, vocabulary] = scores_layout.shape().dims() else {
            candle_core::bail!("the exact rival selector needs a (rows, vocabulary) score table");
        };
        if allowed_layout.shape().dims() != [*rows, *vocabulary] {
            candle_core::bail!("the exact rival selector's allowed mask must match its scores");
        }
        if *rows == 0 || *vocabulary == 0 || k > *vocabulary {
            candle_core::bail!("the exact rival selector has an empty or short row");
        }
        let vocabulary_u32 = u32::try_from(*vocabulary)
            .map_err(|_| candle_core::Error::Msg("exact rival vocabulary exceeds u32".into()))?;
        let device = &scores.device;
        let scores_view = f32_view(scores, scores_layout)?;
        let allowed_view = u8_view(allowed, allowed_layout)?;
        let out = uninit::<u32>(device, *rows * k)?;
        launch_groups_prebuilt(
            device,
            TOP8_POSITIONS_MODULE,
            PTX,
            "top8_positions",
            (*rows, 1, 1),
            (32, 1, 1),
            &[
                Arg::F(scores_view),
                Arg::U8(allowed_view),
                Arg::u(&out),
                Arg::U32(vocabulary_u32),
                Arg::U32(k as u32),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            Shape::from_dims(&[*rows, k]),
        ))
    }

    pub(super) fn nearest_positions(
        query: &CudaStorage,
        query_layout: &Layout,
        table: &CudaStorage,
        table_layout: &Layout,
        targets: &CudaStorage,
        targets_layout: &Layout,
        k: usize,
    ) -> CResult<(CudaStorage, Shape)> {
        if !(1..=MAX_RIVALS).contains(&k) {
            candle_core::bail!("the fused rival selector takes 1 to {MAX_RIVALS} positions");
        }
        let [rows, width] = query_layout.shape().dims() else {
            candle_core::bail!("the fused rival selector needs a (rows, width) query");
        };
        let [vocabulary, table_width] = table_layout.shape().dims() else {
            candle_core::bail!("the fused rival selector needs a (vocabulary, width) table");
        };
        if width != table_width || *width == 0 || *width > 1024 {
            candle_core::bail!("the fused rival selector needs 1..=1024 matching widths");
        }
        if *rows == 0 || *vocabulary == 0 || k > *vocabulary {
            candle_core::bail!("the fused rival selector has an empty or short row");
        }
        if targets_layout.shape().elem_count() != *rows {
            candle_core::bail!("the fused rival selector needs one target a query row");
        }
        let vocabulary_u32 = u32::try_from(*vocabulary)
            .map_err(|_| candle_core::Error::Msg("fused rival vocabulary exceeds u32".into()))?;
        let width_u32 = u32::try_from(*width)
            .map_err(|_| candle_core::Error::Msg("fused rival width exceeds u32".into()))?;
        let device = &query.device;
        let query_view = f32_view(query, query_layout)?;
        let table_view = f32_view(table, table_layout)?;
        let targets_view = u32_view(targets, targets_layout)?;
        let out = uninit::<u32>(device, *rows * k)?;
        launch_groups_prebuilt(
            device,
            TOP8_POSITIONS_MODULE,
            PTX,
            "nearest_top8_positions",
            (*rows, 1, 1),
            (256, 1, 1),
            &[
                Arg::F(query_view),
                Arg::F(table_view),
                Arg::U(targets_view),
                Arg::u(&out),
                Arg::U32(vocabulary_u32),
                Arg::U32(width_u32),
                Arg::U32(k as u32),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            Shape::from_dims(&[*rows, k]),
        ))
    }
}

#[cfg(test)]
mod tests {
    fn hosted(scores: &[f32], allowed: &[u8], vocab: usize, k: usize) -> Vec<u32> {
        let mut lane_lists: Vec<Vec<(f32, u32)>> = (0..32).map(|_| Vec::new()).collect();
        for (lane, list) in lane_lists.iter_mut().enumerate() {
            for index in (lane..vocab).step_by(32) {
                if allowed[index] == 0 {
                    continue;
                }
                list.push((scores[index], index as u32));
                list.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
                list.truncate(k);
            }
        }
        let mut merged: Vec<(f32, u32)> = lane_lists.into_iter().flatten().collect();
        merged.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        merged.truncate(k);
        (0..k)
            .map(|i| merged.get(i).map_or(0, |(_, index)| *index))
            .collect()
    }

    fn repeated(scores: &[f32], allowed: &[u8], k: usize) -> Vec<u32> {
        let mut left: Vec<Option<f32>> = scores
            .iter()
            .zip(allowed)
            .map(|(&score, &keep)| (keep != 0).then_some(score))
            .collect();
        (0..k)
            .map(|_| {
                let mut best: Option<(f32, usize)> = None;
                for (index, score) in left.iter().enumerate() {
                    let Some(score) = score else {
                        continue;
                    };
                    let better = match best {
                        None => true,
                        Some((at, best_index)) => {
                            *score > at || (*score == at && index < best_index)
                        }
                    };
                    if better {
                        best = Some((*score, index));
                    }
                }
                match best {
                    Some((_, index)) => {
                        left[index] = None;
                        index as u32
                    }
                    None => 0,
                }
            })
            .collect()
    }

    #[test]
    fn the_warp_merge_is_repeated_argmax_with_lower_ids_on_ties() {
        let scores: Vec<f32> = (0..96)
            .map(|i| if i < 16 { 2.0 } else { (96 - i) as f32 / 100.0 })
            .collect();
        let allowed: Vec<u8> = (0..96).map(|i| (i % 7 != 0) as u8).collect();
        assert_eq!(
            hosted(&scores, &allowed, 96, 8),
            repeated(&scores, &allowed, 8)
        );

        let few: Vec<u8> = (0..96).map(|i| (i == 41 || i == 73) as u8).collect();
        assert_eq!(hosted(&scores, &few, 96, 8), repeated(&scores, &few, 8));
    }
}
