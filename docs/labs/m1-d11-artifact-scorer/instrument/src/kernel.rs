//! The row records the kernel reads and writes, shared by the device scorer
//! and the host build of the same body (`kernel_core.h`), and the comparison of
//! one scored row with `IntegerDiscreteModel::evaluate_prefix` and
//! `IntegerDiscreteModel::observe_target`.
use crate::artifact::ArtifactTables;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

/// `sizeof(D11Scratch)` in 32-bit words: frame, payload[8], prev and rel per
/// node. Every field is a 32-bit integer, so host and device layouts agree.
/// Checked against the host build in the tests.
pub const SCRATCH_WORDS: usize = 512 + 512 * 8 + 512 + 512;
/// `sizeof(D11Plan)` in words: node_count, frontier_len, frontier[32] and
/// nodes[512] of (c0, c1, c2, atom, level, first, last).
pub const PLAN_WORDS: usize = 2 + 32 + 512 * 7;
const MAX_NODES: usize = 512;
const MAX_FRONTIER: usize = 32;
/// `sizeof(D11RowOut)` in 32-bit words: status, admitted_count, top8_count,
/// prediction, admitted[16], top8[8], read_vector[16 * 8], and the five
/// target-observation loss words (token_hinge, address_hinge, rank_hinge,
/// loss_ta, loss_tr).
pub const ROW_OUT_WORDS: usize = 4 + 16 + 8 + 16 * 8 + 5;

/// Kernel status codes (`D11_ERR_*` in `kernel_core.h`); 0 is success.
pub fn status_name(status: i32) -> &'static str {
    match status {
        0 => "ok",
        1 => "prefix length must be in 1..=context",
        2 => "prefix token ID outside vocabulary",
        3 => "row plan exceeds the kernel's node bound",
        4 => "frame outside SpiralCore orbit",
        5 => "route did not reach an atom leaf",
        6 => "writer admitted zero tokens",
        7 => "shape outside the kernel's bounds",
        8 => "target token ID outside vocabulary",
        _ => "unknown kernel status",
    }
}

/// The binder plan of every prefix length `1..=context`, as `D11Plan` records:
/// a port of `build_row_plan` (`discrete_d11.rs:518-560`). It is structure only
/// (D24), a function of the length, never of the tokens or the model.
pub fn row_plans(context: u32) -> Result<Vec<i32>, String> {
    let context = context as usize;
    let mut words = vec![0i32; context * PLAN_WORDS];
    for length in 1..=context {
        // Each node is (c0, c1, c2, atom, level, first, last); -1 marks absent.
        let mut nodes: Vec<[i32; 7]> = Vec::new();
        let mut frontier: Vec<usize> = Vec::new();
        for atom in 0..length as i32 {
            frontier.push(nodes.len());
            nodes.push([-1, -1, -1, atom, 0, atom, atom]);
            while frontier.len() >= 3 {
                let n = frontier.len();
                let (c0, c1, c2) = (frontier[n - 3], frontier[n - 2], frontier[n - 1]);
                let level = nodes[c0][4];
                if nodes[c1][4] != level || nodes[c2][4] != level {
                    break;
                }
                let parent = nodes.len();
                let (first, last) = (nodes[c0][5], nodes[c2][6]);
                nodes.push([c0 as i32, c1 as i32, c2 as i32, -1, level + 1, first, last]);
                frontier.truncate(n - 3);
                frontier.push(parent);
            }
        }
        if nodes.len() > MAX_NODES || frontier.len() > MAX_FRONTIER {
            return Err(format!(
                "the plan of length {length} exceeds the kernel's bounds"
            ));
        }
        let record = &mut words[(length - 1) * PLAN_WORDS..length * PLAN_WORDS];
        record[0] = nodes.len() as i32;
        record[1] = frontier.len() as i32;
        for (slot, &node) in record[2..2 + frontier.len()].iter_mut().zip(&frontier) {
            *slot = node as i32;
        }
        for (slot, node) in record[2 + MAX_FRONTIER..].chunks_exact_mut(7).zip(&nodes) {
            slot.copy_from_slice(node);
        }
    }
    Ok(words)
}

/// Prefixes and targets laid out as the kernel reads them: `rows x stride`
/// tokens, each row zero-padded after its length, and one target token per row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedRows {
    pub prefixes: Vec<u32>,
    pub lengths: Vec<i32>,
    pub targets: Vec<u32>,
    pub stride: i32,
}

impl PackedRows {
    /// The layout both scorers rely on: a positive stride, `rows x stride`
    /// tokens, one target per row, and every length within `0..=stride`, so no
    /// row reads past its own slots. A length of 0 is kept: the kernel refuses
    /// that row.
    pub fn check(&self) -> Result<(), String> {
        let stride = usize::try_from(self.stride).map_err(|_| "negative stride")?;
        if stride == 0
            || self.prefixes.len() != self.lengths.len() * stride
            || self.targets.len() != self.lengths.len()
        {
            return Err("packed rows have the wrong length".into());
        }
        if self
            .lengths
            .iter()
            .any(|&length| !(0..=self.stride).contains(&length))
        {
            return Err("a packed row's length is outside 0..=stride".into());
        }
        Ok(())
    }
}

pub fn pack_rows(prefixes: &[&[u32]]) -> Result<PackedRows, String> {
    let targets = vec![0u32; prefixes.len()];
    pack_rows_with_targets(prefixes, &targets)
}

pub fn pack_rows_with_targets(prefixes: &[&[u32]], targets: &[u32]) -> Result<PackedRows, String> {
    if prefixes.len() != targets.len() {
        return Err("targets do not match prefixes".into());
    }
    let stride = prefixes.iter().map(|p| p.len()).max().unwrap_or(1).max(1);
    let mut packed = vec![0u32; prefixes.len() * stride];
    let mut lengths = Vec::with_capacity(prefixes.len());
    for (row, prefix) in prefixes.iter().enumerate() {
        packed[row * stride..row * stride + prefix.len()].copy_from_slice(prefix);
        lengths.push(i32::try_from(prefix.len()).map_err(|_| "prefix too long")?);
    }
    Ok(PackedRows {
        prefixes: packed,
        lengths,
        targets: targets.to_vec(),
        stride: i32::try_from(stride).map_err(|_| "stride too long")?,
    })
}

/// One scored row, decoded from its `D11RowOut` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowScore {
    pub status: i32,
    pub read_vector: Vec<i32>,
    pub admitted_cells: Vec<u32>,
    pub top8: Vec<u32>,
    pub prediction: u32,
    pub token_hinge: u64,
    pub address_hinge: u64,
    pub rank_hinge: u64,
    pub loss_ta: u64,
    pub loss_tr: u64,
}

/// Decodes `rows` records of `ROW_OUT_WORDS` words; `blocks` sets the read
/// vector's length.
pub fn decode_rows(words: &[u32], rows: usize, blocks: usize) -> Result<Vec<RowScore>, String> {
    if words.len() != rows * ROW_OUT_WORDS {
        return Err("row records have the wrong length".into());
    }
    words
        .chunks_exact(ROW_OUT_WORDS)
        .map(|w| {
            let status = w[0] as i32;
            let admitted = w[1] as usize;
            let top = w[2] as usize;
            if status == 0 && (admitted > 16 || top > 8 || top == 0) {
                return Err("row record counts out of range".to_string());
            }
            if status != 0 {
                return Ok(RowScore {
                    status,
                    read_vector: Vec::new(),
                    admitted_cells: Vec::new(),
                    top8: Vec::new(),
                    prediction: 0,
                    token_hinge: 0,
                    address_hinge: 0,
                    rank_hinge: 0,
                    loss_ta: 0,
                    loss_tr: 0,
                });
            }
            let loss_base = 28 + 16 * 8;
            Ok(RowScore {
                status,
                read_vector: w[28..28 + blocks * 8].iter().map(|&v| v as i32).collect(),
                admitted_cells: w[4..4 + admitted].to_vec(),
                top8: w[20..20 + top].to_vec(),
                prediction: w[3],
                token_hinge: u64::from(w[loss_base]),
                address_hinge: u64::from(w[loss_base + 1]),
                rank_hinge: u64::from(w[loss_base + 2]),
                loss_ta: u64::from(w[loss_base + 3]),
                loss_tr: u64::from(w[loss_base + 4]),
            })
        })
        .collect()
}

/// Compares one scored row (at `target = 0`) with the clean-room evaluator.
pub fn compare_row(
    model: &IntegerDiscreteModel,
    prefix: &[u32],
    score: &RowScore,
) -> Result<(), String> {
    compare_row_with_target(model, prefix, 0, score)
}

/// Compares one scored row and its target loss numerators against
/// `IntegerDiscreteModel::observe_target(prefix, target)`.
pub fn compare_row_with_target(
    model: &IntegerDiscreteModel,
    prefix: &[u32],
    target: u32,
    score: &RowScore,
) -> Result<(), String> {
    match model.observe_target(prefix, target) {
        Ok(obs) => {
            let reference = &obs.evaluation;
            if score.status != 0 {
                return Err(format!(
                    "kernel refused a row the reference scores: {}",
                    status_name(score.status)
                ));
            }
            if score.read_vector != reference.read_vector {
                return Err("read vector differs".into());
            }
            if score.admitted_cells != reference.admitted_cells {
                return Err("admitted cells differ".into());
            }
            if score.top8 != reference.top8 || score.prediction != reference.prediction {
                return Err("top-8 or prediction differs".into());
            }
            if score.token_hinge != obs.token_hinge
                || score.address_hinge != obs.address_hinge
                || score.rank_hinge != obs.rank_hinge
                || score.loss_ta != obs.loss_ta
                || score.loss_tr != obs.loss_tr
            {
                return Err(format!(
                    "target loss differs: kernel (tok={}, addr={}, rank={}, ta={}, tr={}) vs reference (tok={}, addr={}, rank={}, ta={}, tr={})",
                    score.token_hinge,
                    score.address_hinge,
                    score.rank_hinge,
                    score.loss_ta,
                    score.loss_tr,
                    obs.token_hinge,
                    obs.address_hinge,
                    obs.rank_hinge,
                    obs.loss_ta,
                    obs.loss_tr
                ));
            }
            Ok(())
        }
        // Both refuse: they agree only when they refuse for the same reason.
        Err(e) if score.status != 0 => {
            if e.to_string().contains(status_name(score.status)) {
                Ok(())
            } else {
                Err(format!(
                    "both refused, for different reasons: kernel {}, reference {e}",
                    status_name(score.status)
                ))
            }
        }
        Err(e) => Err(format!("kernel scored a row the reference refuses: {e}")),
    }
}

mod host {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct CModel {
        pub shape: [i32; 8],
        pub table: *const i32,
        pub cells: *const u32,
        pub routes: *const u32,
        pub gates: *const i32,
        pub plans: *const i32,
    }

    extern "C" {
        pub fn d11_sizeof_scratch() -> i64;
        pub fn d11_sizeof_row_out() -> i64;
        pub fn d11_sizeof_plan() -> i64;
        pub fn d11_host_score_rows(
            model: *const CModel,
            prefixes: *const u32,
            lengths: *const i32,
            targets: *const u32,
            rows: i32,
            stride: i32,
            scratch: *mut c_void,
            out: *mut c_void,
        );
    }
}

/// The host build's `sizeof` of `D11Scratch`, `D11RowOut` and `D11Plan`, in
/// bytes.
pub fn host_record_sizes() -> (i64, i64, i64) {
    // SAFETY: these functions take no arguments and return a constant.
    unsafe {
        (
            host::d11_sizeof_scratch(),
            host::d11_sizeof_row_out(),
            host::d11_sizeof_plan(),
        )
    }
}

/// Scores rows with the host build of the kernel body. A finite parity check
/// for the tests only: the panel is scored on the GPU.
pub fn host_score(tables: &ArtifactTables, rows: &PackedRows) -> Result<Vec<RowScore>, String> {
    let count = rows.lengths.len();
    let rows_i32 = i32::try_from(count).map_err(|_| "too many rows")?;
    rows.check()?;
    if tables.plans.len() != tables.shape[2] as usize * PLAN_WORDS {
        return Err("row plans do not cover the context".into());
    }
    let model = host::CModel {
        shape: tables.shape,
        table: tables.table.as_ptr(),
        cells: tables.cells.as_ptr(),
        routes: tables.routes.as_ptr(),
        gates: tables.gates.as_ptr(),
        plans: tables.plans.as_ptr(),
    };
    let mut scratch = vec![0u32; SCRATCH_WORDS];
    let mut out = vec![0u32; count * ROW_OUT_WORDS];
    // SAFETY: CModel mirrors D11Model (eight i32 then five pointers); the
    // pointers cover the lengths the artifact reader checked, and plans holds
    // one D11Plan per length up to the context, checked above; prefixes holds
    // rows x stride tokens and targets holds `count` u32 words; scratch is one
    // D11Scratch (SCRATCH_WORDS words) and out holds `count` D11RowOut records.
    // The host function only reads model data and writes scratch and out.
    unsafe {
        host::d11_host_score_rows(
            &model,
            rows.prefixes.as_ptr(),
            rows.lengths.as_ptr(),
            rows.targets.as_ptr(),
            rows_i32,
            rows.stride,
            scratch.as_mut_ptr().cast(),
            out.as_mut_ptr().cast(),
        );
    }
    decode_rows(&out, count, tables.shape[1] as usize)
}
