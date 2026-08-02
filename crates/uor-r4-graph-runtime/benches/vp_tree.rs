//! Exact VP-tree versus linear ROUT lookup benchmark for issue #277.

extern crate alloc;

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_graph_format::{GraphView, SectionId};

#[path = "../src/vp_tree.rs"]
mod vp_tree;

const DEFAULT_GRAPH: &str = ".uor-models/compiled/smollm2-135m-instruct/graph/score.r4g1";
const DEFAULT_ITERATIONS: usize = 100;

fn read_graph(path: &str) -> Result<(PathBuf, Vec<u8>), String> {
    let requested = Path::new(path);
    let mut candidates = vec![requested.to_owned()];
    if requested.is_relative() {
        candidates.push(Path::new("../..").join(requested));
    }
    for candidate in candidates {
        if let Ok(bytes) = fs::read(&candidate) {
            return Ok((candidate, bytes));
        }
    }
    Err(format!("cannot read graph artifact {path}"))
}

fn masked_hamming(signature: &[u8], prototype: &[u8], mask: &[u8]) -> u32 {
    signature
        .iter()
        .zip(prototype)
        .zip(mask)
        .map(|((&s, &p), &m)| ((s ^ p) & m).count_ones())
        .sum()
}

fn linear_query(
    view: &GraphView<'_>,
    signature: &[u8],
    active: &mut [u32; 8],
) -> (u32, u32, usize) {
    let rout = view.section(SectionId::ROUT).unwrap_or(&[]);
    let mut best_node = 0;
    let mut best_distance = u32::MAX;
    let mut active_len = 0;
    for (node_id, node) in view.nodes().enumerate().skip(1) {
        let proto_start = (node.prototype_word_start as usize) << 3;
        let mask_start = (node.mask_word_start as usize) << 3;
        let Some(prototype) = rout.get(proto_start..proto_start + signature.len()) else {
            continue;
        };
        let Some(mask) = rout.get(mask_start..mask_start + signature.len()) else {
            continue;
        };
        let distance = masked_hamming(signature, prototype, mask);
        let node_id = node_id as u32;
        if distance < best_distance || (distance == best_distance && node_id < best_node) {
            best_node = node_id;
            best_distance = distance;
        }
        if distance <= u32::from(node.radius.0).max(120) && active_len < active.len() {
            active[active_len] = node_id;
            active_len += 1;
        }
    }
    (best_node, best_distance, active_len)
}

fn parse_iterations(raw: Option<&String>) -> Result<usize, String> {
    let iterations = raw
        .map(|value| value.parse::<usize>().map_err(|_| "invalid iterations"))
        .transpose()?
        .unwrap_or(DEFAULT_ITERATIONS);
    if iterations == 0 {
        return Err("iterations must be greater than zero".to_owned());
    }
    Ok(iterations)
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1).filter(|arg| arg != "--bench");
    let graph_path = args.next().unwrap_or_else(|| DEFAULT_GRAPH.to_owned());
    let iterations = parse_iterations(args.next().as_ref())?;
    if args.next().is_some() {
        return Err("usage: vp_tree [GRAPH] [ITERATIONS]".to_owned());
    }

    let (resolved_path, bytes) = read_graph(&graph_path)?;
    let view = GraphView::parse(&bytes).map_err(|error| format!("invalid graph: {error:?}"))?;
    let signature_bytes = usize::from(view.head().ok_or("graph has no HEAD")?.signature_bytes());
    let queries: Vec<Vec<u8>> = (0..32)
        .map(|seed| {
            (0..signature_bytes)
                .map(|index| {
                    (seed as u8)
                        .wrapping_mul(29)
                        .wrapping_add(index as u8)
                        .rotate_left((index % 8) as u32)
                })
                .collect()
        })
        .collect();
    let tree = vp_tree::VpTree::from_graph(&view)
        .ok_or("graph has varying ROUT masks or too few indexable nodes")?;

    for query in &queries {
        let mut linear_active = [0u32; 8];
        let mut tree_active = [0u32; 8];
        let linear_result = linear_query(&view, query, &mut linear_active);
        let tree_result = tree.query(query, &mut tree_active);
        if linear_result != tree_result || linear_active != tree_active {
            return Err(format!(
                "tree mismatch: linear={linear_result:?}/{linear_active:?}, tree={tree_result:?}/{tree_active:?}"
            ));
        }
    }

    let mut linear_checksum = 0u64;
    let linear_start = Instant::now();
    for _ in 0..iterations {
        for query in &queries {
            let mut active = [0u32; 8];
            let result = black_box(linear_query(&view, black_box(query), &mut active));
            linear_checksum = linear_checksum.wrapping_add(u64::from(result.0));
            black_box(active);
        }
    }
    let linear_elapsed = linear_start.elapsed();

    let mut tree_checksum = 0u64;
    let tree_start = Instant::now();
    for _ in 0..iterations {
        for query in &queries {
            let mut active = [0u32; 8];
            let result = black_box(tree.query(black_box(query), &mut active));
            tree_checksum = tree_checksum.wrapping_add(u64::from(result.0));
            black_box(active);
        }
    }
    let tree_elapsed = tree_start.elapsed();

    if linear_checksum != tree_checksum {
        return Err(format!(
            "checksum mismatch: linear={linear_checksum}, tree={tree_checksum}"
        ));
    }

    let query_count = (iterations * queries.len()) as f64;
    let linear_ns = linear_elapsed.as_secs_f64() * 1e9 / query_count;
    let tree_ns = tree_elapsed.as_secs_f64() * 1e9 / query_count;
    println!("graph={}", resolved_path.display());
    println!("nodes={}", view.node_count().unwrap_or(0));
    println!("queries={}", queries.len());
    println!("iterations={iterations}");
    println!(
        "runtime_tree_cutoff_nodes={}",
        vp_tree::MIN_ROUTE_INDEX_NODES
    );
    println!("linear_ns_per_query={linear_ns:.1}");
    println!("vptree_ns_per_query={tree_ns:.1}");
    println!("speedup={:.3}", linear_ns / tree_ns);
    println!("checksum={tree_checksum}");
    Ok(())
}
