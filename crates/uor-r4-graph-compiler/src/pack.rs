//! Profile-guided cache-line aligned packing & emission deduplication (Phase 7).

#[cfg(not(target_arch = "wasm32"))]
use crate::executor::RayonExecutor;
use crate::executor::{CompilerExecutor, SequentialExecutor};

/// 64-byte cache line alignment for high-performance memory bandwidth.
pub const CACHE_LINE_BYTES: usize = 64;

/// Pad a byte vector to the nearest 64-byte cache line boundary.
pub fn pad_to_cache_line(buffer: &mut Vec<u8>) {
    let remainder = buffer.len() % CACHE_LINE_BYTES;
    if remainder != 0 {
        let padding = CACHE_LINE_BYTES - remainder;
        buffer.resize(buffer.len() + padding, 0u8);
    }
}

/// Emission table deduplication result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEmissionTable {
    /// Contiguous packed EMIT byte section.
    pub bytes: Vec<u8>,
    /// Per-region emission start offsets (relative to EMIT remainder) and byte lengths.
    pub ranges: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmissionFragment {
    region_index: usize,
    table: Vec<u8>,
}

/// Deduplicate identical emission tables across co-activated regions
/// and align emission blocks to 64-byte cache lines.
pub fn pack_emission_tables(region_emissions: &[Vec<u8>]) -> PackedEmissionTable {
    pack_emission_tables_with_threads(region_emissions, 1)
        .expect("pack_emission_tables with a sequential executor must not fail")
}

/// Deduplicate emission tables with bounded parallel fragment preparation and
/// deterministic canonical assembly.
pub fn pack_emission_tables_with_threads(
    region_emissions: &[Vec<u8>],
    threads: usize,
) -> Result<PackedEmissionTable, String> {
    let indices: Vec<usize> = (0..region_emissions.len()).collect();
    let mut fragments = map_with_threads(&indices, threads, |&idx| {
        Ok(EmissionFragment {
            region_index: idx,
            table: region_emissions[idx].clone(),
        })
    })?;
    fragments.sort_by_key(|fragment| fragment.region_index);

    let mut packed_bytes = Vec::new();
    let mut ranges = Vec::with_capacity(fragments.len());
    let mut unique_tables: Vec<(Vec<u8>, u32)> = Vec::new();

    // Storage descriptor prefix [2, 0, 0, 0] (4 bytes)
    packed_bytes.extend_from_slice(&[2, 0, 0, 0]);

    for fragment in fragments {
        if fragment.table.is_empty() {
            ranges.push((0, 0));
            continue;
        }

        if let Some((_, start)) = unique_tables
            .iter()
            .find(|(table, _)| *table == fragment.table)
        {
            ranges.push((*start, fragment.table.len() as u32));
        } else {
            let start_offset = (packed_bytes.len() - 4) as u32;
            let table_len = fragment.table.len() as u32;
            packed_bytes.extend_from_slice(&fragment.table);
            unique_tables.push((fragment.table, start_offset));
            ranges.push((start_offset, table_len));
        }
    }

    pad_to_cache_line(&mut packed_bytes);

    Ok(PackedEmissionTable {
        bytes: packed_bytes,
        ranges,
    })
}

fn map_with_threads<I, O, F>(inputs: &[I], threads: usize, map_fn: F) -> Result<Vec<O>, String>
where
    I: Sync,
    O: Send,
    F: Fn(&I) -> Result<O, String> + Sync,
{
    if threads == 1 {
        return SequentialExecutor::new()
            .map(inputs, map_fn)
            .map_err(|e| e.to_string());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        RayonExecutor::new(threads)
            .and_then(|executor| executor.map(inputs, map_fn))
            .map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        SequentialExecutor::new()
            .map(inputs, map_fn)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_line_padding() {
        let mut buf = vec![1, 2, 3];
        pad_to_cache_line(&mut buf);
        assert_eq!(buf.len() % CACHE_LINE_BYTES, 0);
        assert_eq!(buf.len(), 64);
    }

    #[test]
    fn test_pack_emission_tables_deduplication() {
        let t1 = vec![10, 0, 0, 0, 100, 0, 0, 0];
        let t2 = vec![20, 0, 0, 0, 200, 0, 0, 0];
        let t3 = t1.clone(); // identical to t1

        let result = pack_emission_tables(&[t1, t2, t3]);
        assert_eq!(result.ranges.len(), 3);
        assert_eq!(result.ranges[0], result.ranges[2]); // t3 reuses t1 offset
        assert_eq!(result.bytes.len() % CACHE_LINE_BYTES, 0);
    }

    #[test]
    fn test_pack_emission_tables_thread_invariance() {
        let tables = vec![
            vec![1, 2, 3, 4, 5, 6],
            vec![9, 8, 7, 6],
            vec![1, 2, 3, 4, 5, 6],
            vec![],
            vec![9, 8, 7, 6],
            vec![0, 0, 1, 1, 2, 2, 3, 3],
        ];
        let seq = pack_emission_tables_with_threads(&tables, 1).unwrap();
        let par2 = pack_emission_tables_with_threads(&tables, 2).unwrap();
        let par4 = pack_emission_tables_with_threads(&tables, 4).unwrap();
        assert_eq!(seq, par2);
        assert_eq!(seq, par4);
    }

    #[test]
    fn test_pack_emission_tables_threads_zero_matches_sequential() {
        let tables = vec![vec![1, 2, 3], vec![4, 5], vec![1, 2, 3], vec![]];
        let seq = pack_emission_tables_with_threads(&tables, 1).unwrap();
        let auto = pack_emission_tables_with_threads(&tables, 0).unwrap();
        assert_eq!(seq, auto);
    }
}
