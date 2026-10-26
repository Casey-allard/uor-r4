//! Explicit opt-in wider code carrier CUDA handle (`m1-wider-code-carrier`, #2847).
//! Loads the ahead-of-time `nvcc`-compiled `wider-carrier.cubin` module and exposes
//! GPU inspection of `B`-block read vectors, 256-cell distance envelopes/two-scale
//! totals, admitted cell order, and full/outer/interior $L_1$ token distances.
use super::*;

pub(super) const WIDER_CARRIER_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/wider-carrier.cu"));
pub(super) const WIDER_CARRIER_PTX: &str =
    include_str!(concat!(env!("OUT_DIR"), "/wider-carrier.ptx"));
pub(super) const WIDER_CARRIER_CUBIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wider-carrier.cubin"));
pub(super) const WIDER_CARRIER_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/wider-carrier-resources.txt"));

pub struct WiderCarrierKernels {
    cell_distances: CudaFunction,
    token_distances: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiderCarrierRowInspection {
    pub query: Vec<i32>,
    pub env: [u32; 256],
    pub sum: [u64; 256],
    pub cell_dist: [u64; 256],
    pub admitted_cells: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiderCarrierTokenDistanceRow {
    pub full_l1: Vec<u32>,
    pub outer_l1: Vec<u32>,
    pub interior_l1: Vec<u32>,
}

/// Identity of the embedded wider-carrier image, without a CUDA context.
pub fn compiled_wider_carrier_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(WIDER_CARRIER_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(WIDER_CARRIER_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(WIDER_CARRIER_CUBIN)),
        format!(
            "{:x}",
            Sha256::digest(WIDER_CARRIER_RESOURCE_REPORT.as_bytes())
        ),
    ]
}

impl Device {
    /// Load the ahead-of-time compiled wider-carrier module for this device's model and stream.
    pub fn wider_carrier_kernels(&self) -> Result<WiderCarrierKernels> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(WIDER_CARRIER_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(WiderCarrierKernels {
            cell_distances: load(crate::wider_carrier_source::ENTRIES[0])?,
            token_distances: load(crate::wider_carrier_source::ENTRIES[1])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// Run `discrete_wider_carrier_cell_distances` on `batch` and return per-row
    /// `B`-block read vector `q`, 256-cell envelope/sum/metric distances, and
    /// admitted cell IDs in admission order.
    pub fn inspect_wider_carrier_rows(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        kernels: &WiderCarrierKernels,
    ) -> Result<Vec<WiderCarrierRowInspection>> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid(
                "wider-carrier handle belongs to another model/stream",
            ));
        }
        let _epoch = self.admit_batch(batch, targets)?;
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let rows = batch.rows as usize;
        let width = self.spec().blocks as usize * 8;
        let mut out_q = self.stream.alloc_zeros::<i32>(rows * width).map_err(cuda)?;
        let mut out_env = self.stream.alloc_zeros::<u32>(rows * 256).map_err(cuda)?;
        let mut out_sum = self.stream.alloc_zeros::<u64>(rows * 256).map_err(cuda)?;
        let mut out_cell_dist = self.stream.alloc_zeros::<u64>(rows * 256).map_err(cuda)?;
        let mut out_admitted = self.stream.alloc_zeros::<u32>(rows * 16).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&kernels.cell_distances);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(plan)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&self.position_steps)
            .arg(&mut out_q)
            .arg(&mut out_env)
            .arg(&mut out_sum)
            .arg(&mut out_cell_dist)
            .arg(&mut out_admitted)
            .arg(&mut status);
        // SAFETY: all slices are allocated on this stream with exact row capacities.
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "wider-carrier cell distances")?;

        let host_q = self.stream.clone_dtoh(&out_q).map_err(cuda)?;
        let host_env = self.stream.clone_dtoh(&out_env).map_err(cuda)?;
        let host_sum = self.stream.clone_dtoh(&out_sum).map_err(cuda)?;
        let host_dist = self.stream.clone_dtoh(&out_cell_dist).map_err(cuda)?;
        let host_adm = self.stream.clone_dtoh(&out_admitted).map_err(cuda)?;

        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let mut env = [0u32; 256];
            let mut sum = [0u64; 256];
            let mut cell_dist = [0u64; 256];
            env.copy_from_slice(&host_env[r * 256..(r + 1) * 256]);
            sum.copy_from_slice(&host_sum[r * 256..(r + 1) * 256]);
            cell_dist.copy_from_slice(&host_dist[r * 256..(r + 1) * 256]);
            let admitted_cells: Vec<u32> = host_adm[r * 16..(r + 1) * 16]
                .iter()
                .copied()
                .take_while(|&c| c != ABSENT)
                .collect();
            out.push(WiderCarrierRowInspection {
                query: host_q[r * width..(r + 1) * width].to_vec(),
                env,
                sum,
                cell_dist,
                admitted_cells,
            });
        }
        Ok(out)
    }

    /// Run `discrete_wider_carrier_token_distances` on `queries` (`query_count * blocks * 8` ints)
    /// and return per-query full, outer, and interior $L_1$ distances across the vocabulary.
    pub fn inspect_wider_carrier_token_distances(
        &self,
        queries: &[i32],
        query_count: u32,
        kernels: &WiderCarrierKernels,
    ) -> Result<Vec<WiderCarrierTokenDistanceRow>> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid(
                "wider-carrier handle belongs to another model/stream",
            ));
        }
        let width = self.spec().blocks as usize * 8;
        let q_rows = query_count as usize;
        if q_rows == 0 || queries.len() != q_rows * width {
            return Err(invalid(
                "queries length differs from query_count * blocks * 8",
            ));
        }
        let vocab = self.spec().vocabulary as usize;
        let total = query_count
            .checked_mul(self.spec().vocabulary)
            .ok_or_else(|| invalid("wider-carrier token distance count overflows"))?;
        let spec = self.gpu_spec(query_count)?;
        let d_queries = self.stream.clone_htod(queries).map_err(cuda)?;
        let mut out_full = self
            .stream
            .alloc_zeros::<u32>(total as usize)
            .map_err(cuda)?;
        let mut out_outer = self
            .stream
            .alloc_zeros::<u32>(total as usize)
            .map_err(cuda)?;
        let mut out_int = self
            .stream
            .alloc_zeros::<u32>(total as usize)
            .map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&kernels.token_distances);
        launch
            .arg(&self.table)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&d_queries)
            .arg(&query_count)
            .arg(&mut out_full)
            .arg(&mut out_outer)
            .arg(&mut out_int)
            .arg(&mut status);
        // SAFETY: one thread per (query, token) pair; all buffers sized to `total`.
        unsafe { launch.launch(launch_config(total)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "wider-carrier token distances")?;

        let h_full = self.stream.clone_dtoh(&out_full).map_err(cuda)?;
        let h_outer = self.stream.clone_dtoh(&out_outer).map_err(cuda)?;
        let h_int = self.stream.clone_dtoh(&out_int).map_err(cuda)?;

        let mut out = Vec::with_capacity(q_rows);
        for q_idx in 0..q_rows {
            out.push(WiderCarrierTokenDistanceRow {
                full_l1: h_full[q_idx * vocab..(q_idx + 1) * vocab].to_vec(),
                outer_l1: h_outer[q_idx * vocab..(q_idx + 1) * vocab].to_vec(),
                interior_l1: h_int[q_idx * vocab..(q_idx + 1) * vocab].to_vec(),
            });
        }
        Ok(out)
    }
}
