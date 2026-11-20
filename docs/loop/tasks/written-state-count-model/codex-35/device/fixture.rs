use std::{fs, io, path::Path};

const NIL: u32 = u32::MAX;
const WIDTH: usize = 32;
const MAX_NORM: u64 = 32 * (65535 + 4294967295);

#[derive(Clone, Copy)]
struct Candidate {
    start: u32,
    local: [i32; WIDTH],
    contextual: [i32; WIDTH],
}

fn point(start: u32, contextual_first: i32) -> Candidate {
    let mut contextual = [0; WIDTH];
    contextual[0] = contextual_first;
    Candidate {
        start,
        local: [1; WIDTH],
        contextual,
    }
}

fn extreme(start: u32, local: i16, contextual: i32) -> Candidate {
    Candidate {
        start,
        local: [i32::from(local); WIDTH],
        contextual: [contextual; WIDTH],
    }
}

fn word(bytes: &mut Vec<u8>, word: u32) {
    bytes.extend(word.to_le_bytes());
}

fn span(bytes: &mut Vec<u8>, candidate: &Candidate) {
    word(bytes, candidate.start);
    word(bytes, candidate.start + 2);
}

fn packet(mode: u32, rows: &[(u32, &[Candidate])]) -> Vec<u8> {
    let mut bytes = b"D351".to_vec();
    word(&mut bytes, mode);
    word(&mut bytes, rows.len() as u32);
    for (marker, candidates) in rows {
        word(&mut bytes, candidates.len() as u32);
        word(&mut bytes, *marker);
        for candidate in *candidates {
            span(&mut bytes, candidate);
            for coordinate in candidate.local.into_iter().chain(candidate.contextual) {
                bytes.extend(coordinate.to_le_bytes());
            }
        }
    }
    bytes
}

fn bank() -> Vec<u8> {
    let mut bytes = b"D35PAIR1".to_vec();
    word(&mut bytes, 3);
    for (local, contextual) in [
        (
            [0i32; WIDTH],
            std::array::from_fn(|d| if d == 0 { 6i64 } else { 0 }),
        ),
        (
            [0; WIDTH],
            std::array::from_fn(|d| if d == 0 { 8 } else { 0 }),
        ),
        ([65535; WIDTH], [4294967295; WIDTH]),
    ] {
        for coordinate in local {
            bytes.extend(coordinate.to_le_bytes());
        }
        for coordinate in contextual {
            bytes.extend(coordinate.to_le_bytes());
        }
    }
    bytes
}

struct Expected {
    forward: u32,
    reverse: u32,
    distances: &'static [(u64, u64)],
}

fn output(rows: &[(u32, &[Candidate])], expected: &[Expected]) -> io::Result<Vec<u8>> {
    let mut bytes = b"D35RANK1".to_vec();
    word(&mut bytes, rows.len() as u32);
    for ((_, pool), rank) in rows.iter().zip(expected) {
        let rivals = pool.len().saturating_sub(1);
        if rank.distances.len() != pool.len() * rivals {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "fixture pair count",
            ));
        }
        word(&mut bytes, pool.len() as u32);
        word(&mut bytes, rank.forward);
        word(&mut bytes, rank.reverse);
        for (i, candidate) in pool.iter().enumerate() {
            span(&mut bytes, candidate);
            for (forward, reverse) in &rank.distances[i * rivals..(i + 1) * rivals] {
                bytes.extend(forward.to_le_bytes());
                bytes.extend(reverse.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

fn fixture(root: &Path) -> io::Result<()> {
    fs::create_dir(root)?;
    let positive = point(2, 6);
    let middle = point(4, 0);
    let negative = point(6, -2);
    let minimum = extreme(5, i16::MIN, i32::MIN);
    let maximum = extreme(9, i16::MAX, i32::MAX);
    let train = packet(
        1,
        &[(2, &[positive, middle, negative]), (9, &[minimum, maximum])],
    );
    let held: &[(u32, &[Candidate])] = &[
        (NIL, &[positive, middle, negative]),
        (NIL, &[point(2, -2), middle, point(6, 6)]),
        (NIL, &[point(2, 0), middle]),
        (NIL, &[minimum, maximum]),
        (NIL, &[]),
        (2, &[positive]),
        (NIL, &[positive]),
    ];
    let expected = [
        Expected {
            forward: 2,
            reverse: 6,
            distances: &[(0, 12), (0, 14), (12, 0), (4, 8), (14, 0), (8, 4)],
        },
        Expected {
            forward: 6,
            reverse: 2,
            distances: &[(8, 4), (14, 0), (4, 8), (12, 0), (0, 14), (0, 12)],
        },
        Expected {
            forward: NIL,
            reverse: NIL,
            distances: &[(6, 6), (6, 6)],
        },
        Expected {
            forward: 9,
            reverse: 5,
            distances: &[(MAX_NORM + 6, 0), (0, MAX_NORM + 6)],
        },
        Expected {
            forward: NIL,
            reverse: NIL,
            distances: &[],
        },
        Expected {
            forward: 2,
            reverse: 2,
            distances: &[],
        },
        Expected {
            forward: NIL,
            reverse: NIL,
            distances: &[],
        },
    ];
    let bank = bank();
    let score = packet(2, held);
    let mut corrupt_bank = bank.clone();
    corrupt_bank[140..148].copy_from_slice(&4294967296i64.to_le_bytes());
    let mut corrupt_score = score.clone();
    // The first SCORE row has three candidates, so singleton metadata must be NIL.
    corrupt_score[16..20].copy_from_slice(&2u32.to_le_bytes());
    let mut changed_score = output(held, &expected)?;
    changed_score[16..20].copy_from_slice(&6u32.to_le_bytes());
    for (file, bytes) in [
        ("train.bin", train),
        ("ordinary.bin", score),
        ("bank.expected", bank),
        ("ordinary.expected", output(held, &expected)?),
        ("bank.corrupt", corrupt_bank),
        ("ordinary.corrupt", corrupt_score),
        ("ordinary.changed", changed_score),
    ] {
        fs::write(root.join(file), bytes)?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let root = std::env::args_os().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected new fixture directory",
        )
    })?;
    fixture(Path::new(&root))
}
