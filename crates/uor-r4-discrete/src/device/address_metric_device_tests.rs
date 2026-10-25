//! Finite CUDA controls for the writer's address metric word `s[15]` (#2834):
//! the device's admitted cells equal the host oracles in `address_cells.rs`,
//! recomputed on the device's own table and query vectors, for each metric.
use super::*;
use crate::address_cells::{self, AddressMetric};

const SPEC: Spec = Spec {
    vocabulary: 1024,
    blocks: 4,
    context: 4,
    kept_cells: 8,
};
const ROWS: usize = 16;

fn root_of(block: &[i32]) -> Result<usize> {
    geometry::ROOTS[..16]
        .iter()
        .position(|root| root.iter().zip(block).all(|(&a, &b)| i32::from(a) == b))
        .ok_or_else(|| invalid("table block is not an orbit root"))
}

/// The writer's top-K admission over occupied cells, ordered by
/// `(distance, cell)`, from the host oracles.
fn host_admitted(metric: AddressMetric, table: &[i32], query: &[i32]) -> Result<Vec<u32>> {
    let width = SPEC.blocks as usize * 8;
    let cells = (0..SPEC.vocabulary as usize)
        .map(|t| {
            let vector = &table[t * width..(t + 1) * width];
            Ok((root_of(&vector[..8])? * 16 + root_of(&vector[width - 8..])?) as u16)
        })
        .collect::<Result<Vec<u16>>>()?;
    let mut distances = [u64::MAX; address_cells::CELLS];
    let mut occupied = [false; address_cells::CELLS];
    for &cell in &cells {
        let c = usize::from(cell);
        occupied[c] = true;
        distances[c] = match metric {
            AddressMetric::OuterL1V1 => {
                address_cells::cell_outer_l1(query, SPEC.blocks as usize, c)?
            }
            AddressMetric::CellEnvelopeL1V1 => {
                address_cells::cell_envelope_l1(query, table, &cells, SPEC, c)?
            }
            AddressMetric::FullVectorCellL1V1 => {
                address_cells::full_vector_cell_l1(query, table, &cells, SPEC, c)?
            }
        };
    }
    Ok(
        address_cells::admit_cells_by_distances(&distances, SPEC.kept_cells as usize)
            .into_iter()
            .filter(|&c| occupied[c])
            .map(|c| c as u32)
            .collect(),
    )
}

fn identities() -> (Vec<Identity>, Vec<u32>) {
    let mut ids = Vec::new();
    let mut targets = Vec::new();
    for row in 0..ROWS {
        for position in 0..SPEC.context as usize {
            ids.push(Identity {
                token: ((row * 131 + position * 37) % 1024) as u32,
                occurrence: (row * 16 + position) as u64,
                record: 17,
                version: 9,
            });
        }
        targets.push(((row * 61 + 5) % 1024) as u32);
    }
    (ids, targets)
}

/// A model on a patterned table: token `t` sits in cell `(t % 16, (t / 16) % 16)`,
/// so all 256 cells hold 4 tokens, and the 4 differ in their interior blocks.
/// The seeded device table gives same-cell tokens one vector, so a cell's sum
/// is a fixed multiple of its envelope and the two-scale order cannot differ
/// from the envelope's (G-Q's first pod run, 25 Oct: an instrument fault).
fn patterned_model(metric: AddressMetric) -> Result<Device> {
    let mut snapshot = Device::new(SPEC, 7, 0)?.snapshot()?;
    let width = SPEC.blocks as usize * 8;
    for t in 0..SPEC.vocabulary as usize {
        let k = t / 256;
        let roots = [
            t % 16,
            (t % 16 + 3 * k) % 16,
            ((t / 16) % 16 + 5 * k + 1) % 16,
            (t / 16) % 16,
        ];
        for (block, &root) in roots.iter().enumerate() {
            for d in 0..8 {
                snapshot.table[t * width + block * 8 + d] = i32::from(geometry::ROOTS[root][d]);
            }
        }
    }
    let mut model = Device::from_snapshot(snapshot, 0)?;
    model.set_address_metric(metric);
    Ok(model)
}

#[test]
#[ignore = "requires a CUDA GPU"]
fn device_admission_matches_the_host_oracles_for_each_metric() -> Result<()> {
    let (ids, targets) = identities();
    let width = SPEC.blocks as usize * 8;
    let mut admitted = Vec::new();
    for metric in [
        AddressMetric::OuterL1V1,
        AddressMetric::CellEnvelopeL1V1,
        AddressMetric::FullVectorCellL1V1,
    ] {
        let model = patterned_model(metric)?;
        // The evaluation path (row metrics and aggregation, with its hinge range
        // contract) accepts every metric: G-Q's second pod run found two-scale's
        // raw hinge refused there with status 12.
        model.evaluate(&model.carry(&ids)?, &targets)?;
        let forward = model.forward(&model.carry(&ids)?, &targets)?;
        let table = model.snapshot()?.table;
        let queries = forward.vectors()?;
        let mut rows = Vec::new();
        for row in 0..ROWS {
            let device: Vec<u32> = forward.admitted_cells[row * 16..row * 16 + 16]
                .iter()
                .copied()
                .filter(|&c| c != ABSENT)
                .collect();
            let host = host_admitted(metric, &table, &queries[row * width..(row + 1) * width])?;
            assert_eq!(device, host, "{metric:?} row {row}");
            rows.push(device);
        }
        admitted.push(rows);
        let reloaded = Device::from_snapshot(model.snapshot()?, 0)?;
        assert_eq!(reloaded.address_metric(), metric);
    }
    // The fixture separates all three metrics, so a dropped s[15], or one
    // metric computed as another, fails here even where the oracles agree.
    for (a, b) in [(0, 1), (0, 2), (1, 2)] {
        assert_ne!(
            admitted[a], admitted[b],
            "metrics {a} and {b} admit the same cells on every row"
        );
    }
    Ok(())
}
