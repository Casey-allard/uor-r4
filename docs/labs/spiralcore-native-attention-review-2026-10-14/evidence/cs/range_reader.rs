//! Byte DAG experiment only. No tokenizer, model, attention, or learning execution.
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
type Id = [u8; 12];
#[derive(Clone)]
struct Node {
    scale: u8,
    data: Vec<u8>,
}
#[derive(Clone)]
struct File {
    len: usize,
    roots: Vec<(Id, u8)>,
}
#[derive(Debug, PartialEq)]
enum Error {
    Format,
    Range,
    Missing(Id),
    Scale,
    Integrity,
    Padding,
}
#[derive(Default)]
struct Stats {
    visits: usize,
    leaves: usize,
    authenticated: HashSet<Id>,
    record_bytes: usize,
}
const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];
const SIG: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];
fn mix(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}
fn id_for(node: &Node) -> Id {
    let mut bytes = [0u8; 128];
    bytes[0] = node.scale;
    bytes[1..1 + node.data.len()].copy_from_slice(&node.data);
    let mut m = [0u64; 16];
    for (i, c) in bytes.chunks_exact(8).enumerate() {
        m[i] = u64::from_le_bytes(c.try_into().unwrap());
    }
    let mut h = IV;
    h[0] ^= 0x0101000c;
    let mut v = [0u64; 16];
    v[..8].copy_from_slice(&h);
    v[8..].copy_from_slice(&IV);
    v[12] ^= (node.data.len() + 1) as u64;
    v[14] = !v[14];
    for r in 0..12 {
        let s = &SIG[r % 10];
        mix(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        mix(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        mix(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        mix(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
        mix(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        mix(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        mix(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        mix(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }
    for i in 0..8 {
        h[i] ^= v[i] ^ v[i + 8];
    }
    let mut out = [0u8; 12];
    out[..8].copy_from_slice(&h[0].to_le_bytes());
    out[8..].copy_from_slice(&h[1].to_le_bytes()[..4]);
    out
}
fn parse_id(s: &str) -> Result<Id, Error> {
    if s.len() != 24 {
        return Err(Error::Format);
    }
    let mut id = [0u8; 12];
    for (i, x) in id.iter_mut().enumerate() {
        *x = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| Error::Format)?;
    }
    Ok(id)
}
fn span(scale: u8) -> Result<usize, Error> {
    32usize
        .checked_mul(3usize.checked_pow(scale as u32).ok_or(Error::Format)?)
        .ok_or(Error::Format)
}
fn load(dir: &Path, name: &str) -> Result<(File, HashMap<Id, Node>), Error> {
    let meta =
        fs::read_to_string(dir.join(format!("{name}.manifest"))).map_err(|_| Error::Format)?;
    let mut it = meta.split_whitespace();
    let len = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    let atoms = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    let nr = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    if atoms != len.div_ceil(32) {
        return Err(Error::Format);
    }
    let mut scales = vec![];
    let mut n = atoms;
    let mut s = 0u8;
    while n > 0 {
        for _ in 0..n % 3 {
            scales.push(s)
        }
        n /= 3;
        s += 1;
    }
    scales.reverse();
    if nr != scales.len() {
        return Err(Error::Format);
    }
    let mut roots = vec![];
    for expected in scales {
        let id = parse_id(it.next().ok_or(Error::Format)?)?;
        let scale = it
            .next()
            .ok_or(Error::Format)?
            .parse::<u8>()
            .map_err(|_| Error::Format)?;
        if scale != expected {
            return Err(Error::Format);
        }
        span(scale)?;
        roots.push((id, scale));
    }
    if it.next().is_some() {
        return Err(Error::Format);
    }
    let bytes = fs::read(dir.join(format!("{name}.records"))).map_err(|_| Error::Format)?;
    let mut map = HashMap::new();
    let mut at = 0;
    while at < bytes.len() {
        let scale = bytes[at];
        if scale > 40 {
            return Err(Error::Format);
        }
        let size = if scale == 0 { 32 } else { 36 };
        if bytes.len() - at < 13 + size {
            return Err(Error::Format);
        }
        let id: Id = bytes[at + 1..at + 13]
            .try_into()
            .map_err(|_| Error::Format)?;
        let node = Node {
            scale,
            data: bytes[at + 13..at + 13 + size].to_vec(),
        };
        if map.insert(id, node).is_some() {
            return Err(Error::Format);
        }
        at += 13 + size;
    }
    Ok((File { len, roots }, map))
}
fn visit(
    id: Id,
    scale: u8,
    start: usize,
    a: usize,
    b: usize,
    file: &File,
    nodes: &HashMap<Id, Node>,
    out: &mut Vec<u8>,
    stats: &mut Stats,
) -> Result<(), Error> {
    let end = start.checked_add(span(scale)?).ok_or(Error::Format)?;
    if b <= start || a >= end {
        return Ok(());
    }
    let node = nodes.get(&id).ok_or(Error::Missing(id))?;
    if node.scale != scale {
        return Err(Error::Scale);
    }
    stats.visits += 1;
    if stats.authenticated.insert(id) {
        if id_for(node) != id {
            return Err(Error::Integrity);
        }
        stats.record_bytes += 13 + node.data.len();
    }
    if scale == 0 {
        stats.leaves += 1;
        let lo = a.saturating_sub(start);
        let hi = (b - start).min(32);
        if end > file.len && node.data[file.len - start..].iter().any(|&x| x != 0) {
            return Err(Error::Padding);
        }
        out.extend_from_slice(&node.data[lo..hi]);
    } else {
        let step = span(scale - 1)?;
        for j in 0..3 {
            let child: Id = node.data[j * 12..j * 12 + 12]
                .try_into()
                .map_err(|_| Error::Format)?;
            visit(
                child,
                scale - 1,
                start + j * step,
                a,
                b,
                file,
                nodes,
                out,
                stats,
            )?;
        }
    }
    Ok(())
}
fn read(
    file: &File,
    nodes: &HashMap<Id, Node>,
    a: usize,
    b: usize,
) -> Result<(Vec<u8>, Stats), Error> {
    if a > b || b > file.len {
        return Err(Error::Range);
    }
    let mut out = Vec::with_capacity(b - a);
    let mut stats = Stats::default();
    if a == b {
        return Ok((out, stats));
    }
    let mut at = 0;
    for &(id, scale) in &file.roots {
        visit(id, scale, at, a, b, file, nodes, &mut out, &mut stats)?;
        at = at.checked_add(span(scale)?).ok_or(Error::Format)?;
    }
    if out.len() != b - a {
        return Err(Error::Format);
    }
    Ok((out, stats))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args()
        .nth(1)
        .ok_or("expected fixture directory")?;
    let dir = Path::new(&directory);
    let mut checks = 0usize;
    println!("scope=BYTE_DAG_SELECTIVE_RECONSTRUCTION_ONLY");
    for name in ["unique", "repeated", "utf8", "empty"] {
        let raw = fs::read(dir.join(format!("{name}.raw")))?;
        let (file, nodes) = load(dir, name).map_err(|e| format!("load {e:?}"))?;
        // Authenticate all records once as an ingestion control, explicitly outside range cost.
        for (id, node) in &nodes {
            assert_eq!(&id_for(node), id);
            checks += 1;
        }
        let mut ranges = vec![(0, 0), (0, raw.len()), (raw.len(), raw.len())];
        if !raw.is_empty() {
            for i in 0..257 {
                let a = (i * 7919 + 31) % (raw.len() + 1);
                let b = (a + (i * 17) % 129).min(raw.len());
                ranges.push((a, b));
            }
            for p in [1usize, 31, 32, 33, 63, 64, 65, 95, 96, 97] {
                if p < raw.len() {
                    ranges.push((p.saturating_sub(2), (p + 5).min(raw.len())));
                }
            }
        }
        for &(a, b) in &ranges {
            let (got, _) = read(&file, &nodes, a, b).map_err(|e| format!("read {e:?}"))?;
            assert_eq!(got, &raw[a..b], "{name} {a}..{b}");
            checks += 1;
        }
        assert!(matches!(
            read(&file, &nodes, 0, raw.len() + 1),
            Err(Error::Range)
        ));
        checks += 1;
        if name == "unique" {
            let a = 16379;
            let b = 16402;
            let (_, s) = read(&file, &nodes, a, b).unwrap();
            assert!(s.leaves < raw.len().div_ceil(32));
            let mut reduced = nodes.clone();
            let unrelated = *nodes
                .iter()
                .find(|(id, n)| n.scale == 0 && !s.authenticated.contains(*id))
                .unwrap()
                .0;
            reduced.remove(&unrelated);
            assert_eq!(read(&file, &reduced, a, b).unwrap().0, &raw[a..b]);
            assert!(matches!(
                read(&file, &reduced, 0, raw.len()),
                Err(Error::Missing(_))
            ));
            checks += 2;
            let required = *nodes
                .iter()
                .find(|(id, n)| n.scale == 0 && s.authenticated.contains(*id))
                .unwrap()
                .0;
            let mut missing = nodes.clone();
            missing.remove(&required);
            assert!(matches!(
                read(&file, &missing, a, b),
                Err(Error::Missing(_))
            ));
            checks += 1;
            let mut corrupt = nodes.clone();
            corrupt.get_mut(&required).unwrap().data[0] ^= 1;
            assert!(matches!(read(&file, &corrupt, a, b), Err(Error::Integrity)));
            checks += 1;
            let mut wrong = nodes.clone();
            wrong.get_mut(&required).unwrap().scale = 1;
            assert!(matches!(read(&file, &wrong, a, b), Err(Error::Scale)));
            checks += 1;
            println!("unique_range={a}..{b} output_bytes={} node_visits={} leaf_occurrences={} unique_authenticated_nodes={} serialized_node_bytes={} total_stored_nodes={}",b-a,s.visits,s.leaves,s.authenticated.len(),s.record_bytes,nodes.len());
        }
        if name == "utf8" {
            assert!(std::str::from_utf8(&raw).is_ok());
            let (got, _) = read(&file, &nodes, 32, 34).unwrap();
            assert_eq!(got, &raw[32..34]);
            assert!(std::str::from_utf8(&got).is_err());
            let (got, _) = read(&file, &nodes, 31, 35).unwrap();
            assert_eq!(std::str::from_utf8(&got).unwrap(), "🌀");
            checks += 3;
        }
        let (_, full) = read(&file, &nodes, 0, raw.len()).unwrap();
        println!("case={name} bytes={} stored_nodes={} range_cases={} full_leaf_occurrences={} unique_records_authenticated={}",raw.len(),nodes.len(),ranges.len(),full.leaves,full.authenticated.len());
    }
    println!("checks={checks} result=PASS whole_object_sha256_in_rust=NOT_CHECKED tokenizer=NOT_RUN model=NOT_RUN");
    Ok(())
}
