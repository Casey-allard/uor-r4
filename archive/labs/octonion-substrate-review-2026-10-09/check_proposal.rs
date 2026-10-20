//! Standalone exact small-integer audit, not a model or runtime implementation.
//! Run from the repository root; see README for source identity and scope.
use std::{error::Error, fs};

type V = [i64; 8];
type Cycles = [[usize; 3]; 7];
const PROPOSED: Cycles = [
    [1, 2, 3],
    [1, 4, 5],
    [1, 6, 7],
    [2, 4, 6],
    [2, 5, 7],
    [3, 4, 7],
    [3, 5, 6],
];

fn unit(i: usize) -> V {
    let mut v = [0; 8];
    v[i] = 1;
    v
}
fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
fn product(a: V, b: V, cycles: &Cycles) -> V {
    let mut out = [0; 8];
    for i in 0..8 {
        for j in 0..8 {
            let (sign, k) = if i == 0 {
                (1, j)
            } else if j == 0 {
                (1, i)
            } else if i == j {
                (-1, 0)
            } else {
                let mut found = None;
                for c in cycles {
                    for t in 0..3 {
                        let (x, y, z) = (c[t], c[(t + 1) % 3], c[(t + 2) % 3]);
                        if (i, j) == (x, y) {
                            assert!(found.is_none());
                            found = Some((1, z));
                        }
                        if (i, j) == (y, x) {
                            assert!(found.is_none());
                            found = Some((-1, z));
                        }
                    }
                }
                // Malformed audit fixtures are fatal; this is not a library boundary.
                found.expect("complete unique signed basis table")
            };
            out[k] += sign * a[i] * b[j];
        }
    }
    out
}
fn norm(v: V) -> i64 {
    v.iter().map(|x| x * x).sum()
}
fn left_alternative(x: V, y: V, c: &Cycles) -> bool {
    product(x, product(x, y, c), c) == product(product(x, x, c), y, c)
}
fn read_repository_cycles() -> Result<Cycles, Box<dyn Error>> {
    let s = fs::read_to_string("crates/uor-r4-core/src/spiralcore_operator.rs")?;
    let body = s
        .split_once("pub const OCTONION_FANO_CYCLES: [[u8; 3]; 7] = [")
        .ok_or("missing source table")?
        .1;
    let body = body.split_once("];").ok_or("missing table end")?.0;
    let numbers: Vec<usize> = body
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    if numbers.len() != 21 || numbers.iter().any(|n| !(1..8).contains(n)) {
        return Err("unexpected source table".into());
    }
    Ok(std::array::from_fn(|i| {
        std::array::from_fn(|j| numbers[3 * i + j])
    }))
}
fn project(mut v: V) -> V {
    v[5..].fill(0);
    v
}
fn main() -> Result<(), Box<dyn Error>> {
    let repository = read_repository_cycles()?;
    println!("repository_cycles={repository:?}");
    let x = add(unit(1), unit(2));
    let y = unit(4);
    let lhs = product(x, product(x, y, &PROPOSED), &PROPOSED);
    let rhs = product(product(x, x, &PROPOSED), y, &PROPOSED);
    assert_eq!(lhs, [0, 0, 0, 0, -2, 0, 0, 2]);
    assert_eq!(rhs, [0, 0, 0, 0, -2, 0, 0, 0]);
    println!("proposed_x_xy={lhs:?}; proposed_xx_y={rhs:?}");
    let z = add(unit(4), unit(7));
    let xz = product(x, z, &PROPOSED);
    assert_eq!(xz, [0; 8]);
    assert_eq!(norm(x) * norm(z), 4);
    println!(
        "proposed_norm_product={}; product_of_norms={}",
        norm(xz),
        norm(x) * norm(z)
    );
    for (label, c) in [("proposed", &PROPOSED), ("repository", &repository)] {
        let mut basis_pass = 0;
        let mut mixed_pass = 0;
        for i in 0..8 {
            for j in 0..8 {
                basis_pass += usize::from(left_alternative(unit(i), unit(j), c));
            }
        }
        // 28 positive sums of two distinct basis units, against 8 basis operands.
        for i in 0..8 {
            for j in i + 1..8 {
                for k in 0..8 {
                    mixed_pass += usize::from(left_alternative(add(unit(i), unit(j)), unit(k), c));
                }
            }
        }
        assert_eq!(basis_pass, 64);
        if label == "repository" {
            assert_eq!(mixed_pass, 224);
        } else {
            assert!(mixed_pass < 224);
        }
        println!("{label}: left_alternativity basis={basis_pass}/64 mixed={mixed_pass}/224");
    }
    let routed = product(unit(4), unit(1), &PROPOSED);
    assert_eq!(routed, [0, 0, 0, 0, 0, -1, 0, 0]);
    assert_eq!(
        project(routed),
        project(product([0; 8], unit(1), &PROPOSED))
    );
    println!(
        "proposed_route_e4={routed:?}; projected={:?}; collision_with_zero=true",
        project(routed)
    );
    println!("PASS: documented counterexamples reproduced; finite comparison is not a general proof or model result");
    Ok(())
}
