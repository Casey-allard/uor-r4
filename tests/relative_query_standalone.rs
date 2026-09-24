//! Standalone tests of the actual source modules, with no model or Cargo downloads.
#[path = "../crates/uor-r4-core/src/native_geometric/learner/hamilton_transport.rs"]
mod hamilton_transport;
#[path = "../crates/uor-r4-core/src/native_geometric/learner/relative_action_learning.rs"]
mod relative_action_learning;

use relative_action_learning::{CompiledRelativePath, RelativeActionModel};
use std::hint::black_box;
use std::time::Instant;

fn model() -> RelativeActionModel {
    let mut bytes = b"Q8L1\x08\x00".to_vec();
    bytes.extend(0..8);
    RelativeActionModel::from_bytes(&bytes).unwrap()
}

// Independent selection loop retained from the original implementation.
fn sequential(
    model: &RelativeActionModel,
    path: &[usize],
    query: [i32; 4],
    keys: &[[i32; 4]],
) -> Result<usize, String> {
    if keys.is_empty() || keys.len() > 1024 {
        return Err("invalid candidate count".into());
    }
    let mut best = (u64::MAX, 0);
    for (i, &key) in keys.iter().enumerate() {
        let transformed = model.act(path, key)?;
        let d: u64 = query
            .iter()
            .zip(transformed)
            .map(|(&x, y)| (i64::from(x) - i64::from(y)).unsigned_abs())
            .sum();
        if d < best.0 {
            best = (d, i);
        }
    }
    Ok(best.1)
}

#[test]
fn compiled_path_executes_a_nontrivial_learned_action() {
    let m = model();
    let p = CompiledRelativePath::compile(&m, &[2]).unwrap();
    assert_eq!(p.act([1, 2, 3, 4]).unwrap(), [-2, 1, -4, 3]);
    let keys = [[1, 2, 3, 4], [9, -7, 4, 2]];
    assert_eq!(p.select([-2, 1, -4, 3], &keys).unwrap(), 0);
}

#[test]
fn all_paths_through_length_four_match_sequential_checked_execution() {
    let m = model();
    let ordinary = [[1, 2, 3, 4], [-7, 11, 2, -3], [0; 4], [i32::MAX; 4]];
    let queries = [[0; 4], [9, -7, 4, 2], [i32::MIN; 4], [i32::MAX; 4]];
    let mut boundary = ordinary.to_vec();
    for mask in 0..16 {
        boundary.push(std::array::from_fn(|j| {
            if mask & (1 << j) == 0 {
                i32::MIN
            } else {
                i32::MAX
            }
        }));
    }
    let mut count = 0;
    for len in 0..=4u32 {
        for mut encoded in 0..8usize.pow(len) {
            let mut path = Vec::new();
            for _ in 0..len {
                path.push(encoded % 8);
                encoded /= 8;
            }
            let plan = CompiledRelativePath::compile(&m, &path).unwrap();
            for &key in &boundary {
                assert_eq!(plan.act(key), m.act(&path, key), "path={path:?}");
            }
            for &query in &queries {
                for keys in [&ordinary[..], &boundary[..]] {
                    let expected = sequential(&m, &path, query, keys);
                    assert_eq!(plan.select(query, keys), expected, "path={path:?}");
                    assert_eq!(m.select(&path, query, keys), expected, "path={path:?}");
                }
            }
            count += 1;
        }
    }
    assert_eq!(count, 4681);
    println!("EXHAUSTIVE_PATHS={count}");
}

#[test]
fn cancelled_signs_do_not_erase_intermediate_overflow() {
    let m = model();
    let zero = [0; 4];
    let minimum = [i32::MIN, 0, 0, 0];
    let plan = CompiledRelativePath::compile(&m, &[1, 1]).unwrap();
    assert_eq!(plan.act([1, 2, 3, 4]).unwrap(), [1, 2, 3, 4]);
    assert_eq!(plan.act(minimum), Err("signed transport overflow".into()));
    assert_eq!(
        plan.select(zero, &[zero, minimum]),
        sequential(&m, &[1, 1], zero, &[zero, minimum])
    );
    let identity = CompiledRelativePath::compile(&m, &[]).unwrap();
    assert_eq!(identity.act(minimum).unwrap(), minimum);
    assert_eq!(identity.select(minimum, &[minimum, zero]).unwrap(), 0);
}

#[test]
fn long_paths_match_sequential_where_a_coordinate_is_first_negated_after_step_four() {
    let m = model();
    let keys = [[1, 2, 3, 4], [-7, 11, 2, -3], [i32::MIN, 0, 0, 0], [0; 4]];
    let queries = [[0; 4], [i32::MIN; 4], [9, -7, 4, 2]];
    let check = |path: &[usize]| {
        let plan = CompiledRelativePath::compile(&m, path).unwrap();
        for &query in &queries {
            assert_eq!(
                plan.select(query, &keys),
                sequential(&m, path, query, &keys),
                "path={path:?}"
            );
        }
    };
    // The length-four test cannot reach a coordinate first negated after step four.
    for len in 5..=6u32 {
        for mut encoded in 0..8usize.pow(len) {
            let mut path = Vec::new();
            for _ in 0..len {
                path.push(encoded % 8);
                encoded /= 8;
            }
            check(&path);
        }
    }
    for path in [
        &[0usize, 0, 0, 0, 1][..],
        &[0, 0, 0, 0, 2][..],
        &[7, 7, 7, 7, 1][..],
        &[0, 0, 0, 0, 4, 0, 1][..],
        &[1, 2, 3, 4, 5, 6, 7, 1][..],
    ] {
        check(path);
    }
}

#[test]
fn inverse_query_widens_before_negating_i32_minimum() {
    let m = model();
    let keys = [[i32::MAX, 0, 0, 0], [-1, 2, 3, 4]];
    let query = [i32::MIN, 0, 0, 0];
    let p = CompiledRelativePath::compile(&m, &[1]).unwrap();
    assert_eq!(p.select(query, &keys).unwrap(), 0);
    assert_eq!(p.select(query, &keys), sequential(&m, &[1], query, &keys));
}

#[test]
fn ties_and_invalid_input_error_order_are_preserved() {
    let m = model();
    let keys = [[i32::MIN, 0, 0, 0], [0; 4]];
    for path in [&[1, 999][..], &[999, 1][..], &[][..]] {
        for candidates in [&keys[..], &keys[1..], &[][..]] {
            assert_eq!(
                m.select(path, [0; 4], candidates),
                sequential(&m, path, [0; 4], candidates)
            );
        }
    }
    assert!(CompiledRelativePath::compile(&m, &[999]).is_err());
    let p = CompiledRelativePath::compile(&m, &[2, 4, 6]).unwrap();
    assert_eq!(p.select([0; 4], &[[0; 4]; 3]).unwrap(), 0);
    assert_eq!(p.select([0; 4], &[]), Err("invalid candidate count".into()));
    assert!(p.select([0; 4], &[[0; 4]; 1025]).is_err());
}

#[test]
fn compilation_is_constant_size_and_does_not_change_learned_artifact() {
    let m = model();
    let before = m.to_bytes();
    let path: Vec<_> = (0..4096).map(|i| i % 8).collect();
    let plan = CompiledRelativePath::compile(&m, &path).unwrap();
    assert!(std::mem::size_of_val(&plan) <= 16);
    assert_eq!(plan.act([1, 2, 3, 4]), m.act(&path, [1, 2, 3, 4]));
    assert_eq!(m.to_bytes(), before);
    println!("COMPILED_PATH_BYTES={}", std::mem::size_of_val(&plan));
}

#[test]
fn relation_labels_are_resolved_through_the_learned_map() {
    let m = RelativeActionModel::from_bytes(b"Q8L1\x06\x00\x07\x02\x02\x01\x06\x00").unwrap();
    let keys = [[11, 23, -5, 7], [-8, 4, 9, -3], [2, 1, 0, -7]];
    for path in [&[0, 2, 4][..], &[5, 3, 1, 2][..], &[1, 2][..]] {
        let p = CompiledRelativePath::compile(&m, path).unwrap();
        for &key in &keys {
            assert_eq!(p.act(key), m.act(path, key));
        }
        assert_eq!(
            p.select([9, 2, -8, 1], &keys),
            sequential(&m, path, [9, 2, -8, 1], &keys)
        );
    }
}

// Unsafe allocator delegation exists only in this standalone test instrument.
// Both production modules forbid or contain no unsafe code.
mod allocation_census {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
        static CALLS: Cell<usize> = const { Cell::new(0) };
    }

    struct Counter;

    fn count() {
        if ENABLED.try_with(Cell::get).unwrap_or(false) {
            let _ = CALLS.try_with(|n| n.set(n.get() + 1));
        }
    }

    // SAFETY: every allocation operation is delegated unchanged to System.
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            count();
            unsafe { System.alloc(layout) }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            count();
            unsafe { System.alloc_zeroed(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            count();
            unsafe { System.realloc(ptr, layout, size) }
        }
    }

    #[global_allocator]
    static ALLOCATOR: Counter = Counter;

    pub fn measure<T>(f: impl FnOnce() -> T) -> (T, usize) {
        CALLS.with(|n| n.set(0));
        ENABLED.with(|on| on.set(true));
        let value = f();
        ENABLED.with(|on| on.set(false));
        (value, CALLS.with(Cell::get))
    }
}

#[test]
fn valid_compilation_and_both_selection_interfaces_allocate_nothing() {
    let m = model();
    let path = [2, 4, 6, 1, 7, 3];
    let keys = [[3, 7, -8, 11], [0; 4], [-9, 2, 5, 1]];
    let (result, calls) = allocation_census::measure(|| {
        let plan = CompiledRelativePath::compile(&m, &path)?;
        Ok::<_, String>((
            plan.select([1, 2, 3, 4], &keys)?,
            m.select(&path, [1, 2, 3, 4], &keys)?,
        ))
    });
    let (a, b) = result.unwrap();
    assert_eq!(a, b);
    assert_eq!(calls, 0);
    println!("SUCCESS_PATH_ALLOCATIONS={calls}");
}

fn measured_case(path_length: usize, candidate_count: i32) {
    let m = model();
    let path: Vec<_> = (0..path_length).map(|i| (i * 5 + 2) % 8).collect();
    let keys: Vec<_> = (0..candidate_count)
        .map(|i| [i, 2 * i - 31, 3 - i, i % 7])
        .collect();
    let plan = CompiledRelativePath::compile(&m, &path).unwrap();
    let mut timing = [Vec::new(), Vec::new(), Vec::new()];
    let mut checksum = [0usize; 3];
    for round in 0..11 {
        for offset in 0..3 {
            let arm = (offset + round) % 3;
            let start = Instant::now();
            for n in 0..32 {
                let query = black_box([n, 5, -7, 11]);
                let result = match arm {
                    0 => sequential(black_box(&m), black_box(&path), query, black_box(&keys)),
                    1 => m.select(black_box(&path), query, black_box(&keys)),
                    _ => plan.select(query, black_box(&keys)),
                };
                checksum[arm] += black_box(result.unwrap());
            }
            timing[arm].push(start.elapsed().as_nanos() / 32);
        }
    }
    assert_eq!(checksum[0], checksum[1]);
    assert_eq!(checksum[0], checksum[2]);
    for t in &mut timing {
        t.sort_unstable();
    }
    println!("CPU_NS_PER_QUERY sequential={} integrated={} prepared={} candidates={} path_length={} rounds=11", timing[0][5], timing[1][5], timing[2][5], candidate_count, path_length);
}

#[test]
fn measured_cpu_cost_is_reported_without_a_timing_pass_threshold() {
    for path_length in [0, 1, 6, 32] {
        for candidate_count in [1, 16, 256] {
            measured_case(path_length, candidate_count);
        }
    }
}
