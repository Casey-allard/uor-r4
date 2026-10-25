//! Host and source controls for GPU position transport (`s[14] = 1..=3`),
//! device-side `from_snapshot` step-table validation (`discrete_validate` status 10),
//! and per-thread stack compaction in `src/discrete.cu` (`#2843`).
//!
//! No floating-point types or literals (D32), no dense matrix multiplication
//! (D25), and no squashing functions (D23) appear in this file.

use uor_r4_discrete::position_mechanism::{
    apply_frame_to_root, relative_holonomy, PositionConfig, PositionMode, PositionStepTable,
    ORBIT_ROOTS,
};
use uor_r4_discrete::{ReadSpec, Spec};

fn patterned_table(blocks: u32) -> PositionStepTable {
    let mut candidates = Vec::with_capacity((blocks as usize) * ORBIT_ROOTS);
    for b in 0..blocks as usize {
        for r in 0..ORBIT_ROOTS {
            candidates.push(((b * 5 + r * 3) % 15 + 1) as u8);
        }
    }
    PositionStepTable::new(blocks, candidates).expect("valid patterned table")
}

#[test]
fn cuda_position_stack_arrays_are_byte_packed() {
    let source = include_str!("../src/discrete.cu");
    assert!(
        source.contains(
            "__device__ U position_action(const U* steps,const unsigned char* prefix,const U* g,const U* s,U block,U from_root,U query_root,U from_pos,U query_pos)"
        ),
        "position_action must read byte-packed prefix frames"
    );
    assert!(
        source.contains("unsigned char prefix[MAX_POSITIONS];"),
        "prefix[243] must use 1-byte unsigned char storage (243 B instead of 972 B)"
    );
    assert!(
        source.contains("unsigned char transport_root[MAX_NODES],transport[MAX_NODES];"),
        "transport_root[364] and transport[364] must use 1-byte unsigned char storage (728 B instead of 2912 B)"
    );
    assert!(
        !source.contains("U prefix[MAX_POSITIONS];"),
        "4-byte U prefix[MAX_POSITIONS] stack allocation must be removed"
    );
    assert!(
        !source.contains("U transport_root[MAX_NODES],transport[MAX_NODES];"),
        "4-byte U transport_root/transport stack allocations must be removed"
    );
}

#[test]
fn snapshot_loader_delegates_position_step_bounds_to_cuda_validate() {
    let source = include_str!("../src/device.rs");
    let body = source
        .split("pub fn from_snapshot")
        .nth(1)
        .expect("from_snapshot exists")
        .split("fn gpu_spec")
        .next()
        .expect("gpu_spec follows from_snapshot");
    assert!(
        body.contains("snapshot.normalization_mode,\n            None,\n            snapshot.seed,"),
        "from_snapshot must construct base Device with None so discrete_validate checks position_steps on CUDA"
    );
    assert!(
        body.contains("model.position_steps = model.stream.clone_htod(&position_words)"),
        "from_snapshot must upload snapshot position words to model.position_steps before discrete_validate"
    );
    assert!(
        source.contains("mod position_device_tests"),
        "device.rs must include GPU position_device_tests"
    );
}

#[test]
fn endpoint_control_level0_sum_is_invariant_under_interior_swap_while_prefix_holonomy_changes() {
    let spec = Spec {
        vocabulary: 8,
        blocks: 2,
        context: 4,
        kept_cells: 4,
    };
    let table = patterned_table(spec.blocks);
    PositionConfig::new(PositionMode::PrefixHolonomyV1, table.clone())
        .validate(spec, ReadSpec::Flock { kept: 3 })
        .expect("valid flock position config");

    // Find a 4-token root window [r0, r1, r2, r3] where swapping interior roots
    // r1 and r2 changes the level-0 summed carried coordinates under PrefixHolonomyV1
    // while leaving EndpointControlV1's level-0 summed carried coordinates identical.
    let mut found_witness = false;
    for r1 in 0..ORBIT_ROOTS as u8 {
        for r2 in 0..ORBIT_ROOTS as u8 {
            let win_ab = [0u8, r1, r2, 3u8];
            let win_ba = [0u8, r2, r1, 3u8];
            let mut sum_hol_ab = [0i32; 8];
            let mut sum_hol_ba = [0i32; 8];
            let mut sum_end_ab = [0i32; 8];
            let mut sum_end_ba = [0i32; 8];
            for s in 0..3usize {
                let h_ab =
                    relative_holonomy(PositionMode::PrefixHolonomyV1, &table, 0, &win_ab, s, 3)
                        .expect("holonomy ab");
                let h_ba =
                    relative_holonomy(PositionMode::PrefixHolonomyV1, &table, 0, &win_ba, s, 3)
                        .expect("holonomy ba");
                let e_ab =
                    relative_holonomy(PositionMode::EndpointControlV1, &table, 0, &win_ab, s, 3)
                        .expect("endpoint ab");
                let e_ba =
                    relative_holonomy(PositionMode::EndpointControlV1, &table, 0, &win_ba, s, 3)
                        .expect("endpoint ba");
                let (_, coords_h_ab) = apply_frame_to_root(h_ab, win_ab[s]).expect("apply h_ab");
                let (_, coords_h_ba) = apply_frame_to_root(h_ba, win_ba[s]).expect("apply h_ba");
                let (_, coords_e_ab) = apply_frame_to_root(e_ab, win_ab[s]).expect("apply e_ab");
                let (_, coords_e_ba) = apply_frame_to_root(e_ba, win_ba[s]).expect("apply e_ba");
                for d in 0..8 {
                    sum_hol_ab[d] += i32::from(coords_h_ab[d]);
                    sum_hol_ba[d] += i32::from(coords_h_ba[d]);
                    sum_end_ab[d] += i32::from(coords_e_ab[d]);
                    sum_end_ba[d] += i32::from(coords_e_ba[d]);
                }
            }
            assert_eq!(
                sum_end_ab, sum_end_ba,
                "EndpointControlV1 level-0 sum over all 3 leaves must be invariant under interior swap"
            );
            if sum_hol_ab != sum_hol_ba {
                found_witness = true;
                break;
            }
        }
        if found_witness {
            break;
        }
    }
    assert!(
        found_witness,
        "PrefixHolonomyV1 level-0 sum must change under an interior swap"
    );
}
