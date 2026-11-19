//! Lossless audit of the pinned S2 /2 retained section. No predictor is executed.
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

type Result<T> = io::Result<T>;
const ORDERS: usize = 11;
const PAGE: usize = 128;
const MAGIC: &[u8; 8] = b"UORS2P01";
const PROVENANCE: usize = 96;
const HEADER: usize = 8 + PROVENANCE + (ORDERS + 1) * 4;
const ENTRY: usize = 20;
const ROW: usize = 20;

fn require(ok: bool, reason: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, reason))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Group {
    key: u128,
    words: [u32; 4],
    len: usize,
}

fn word(bytes: &[u8]) -> u128 {
    let mut wide = [0; 16];
    wide[..bytes.len()].copy_from_slice(bytes);
    u128::from_le_bytes(wide)
}

fn read_source(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    require(
        bytes.len() == 379_223_660 && bytes.starts_with(b"UORLUT01"),
        "expected pinned D23 carrier",
    )?;
    Ok(bytes)
}

fn row(bytes: &[u8]) -> (u128, u16, u8) {
    (word(&bytes[..16]), word(&bytes[16..18]) as u16, bytes[18])
}

fn varint(mut value: u128, out: &mut Vec<u8>) {
    while value >= 128 {
        out.push((value as u8) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}

fn take_varint(input: &mut &[u8]) -> Result<u128> {
    let mut value = 0;
    // A shifted 120-bit key and two successor-count bits fit in 18 octets.
    for shift in (0..126).step_by(7) {
        let (&byte, rest) = input.split_first().ok_or(io::ErrorKind::UnexpectedEof)?;
        *input = rest;
        value |= u128::from(byte & 127) << shift;
        if byte < 128 {
            require(shift == 0 || byte != 0, "overlong integer")?;
            return Ok(value);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "overwide integer",
    ))
}

fn write_page(groups: &[Group], directory: &mut Vec<u8>, body: &mut Vec<u8>) {
    directory.extend((body.len() as u32).to_le_bytes());
    directory.extend(groups[0].key.to_le_bytes());
    let mut keys = Vec::new();
    let mut successors = Vec::new();
    let mut previous = groups[0].key;
    let mut pending = 0;
    let mut bits = 0;
    for group in groups {
        varint(
            ((group.key - previous) << 2) | (group.len as u128 - 1),
            &mut keys,
        );
        previous = group.key;
        for &next in &group.words[..group.len] {
            pending |= next << bits;
            bits += 17;
            while bits >= 8 {
                successors.push(pending as u8);
                pending >>= 8;
                bits -= 8;
            }
        }
    }
    if bits != 0 {
        successors.push(pending as u8);
    }
    body.extend((keys.len() as u16).to_le_bytes());
    body.extend(keys);
    body.extend(successors);
}

fn encode(source: &[u8]) -> Result<Vec<u8>> {
    let mut header = MAGIC.to_vec();
    header.extend(&source[..PROVENANCE]);
    let rows = &source[PROVENANCE..];
    let mut index = 0;
    let mut directory = Vec::new();
    let mut body = Vec::new();
    let mut page = Vec::with_capacity(PAGE);
    for order in 0..ORDERS {
        header.extend(((directory.len() / ENTRY) as u32).to_le_bytes());
        while index < rows.len() {
            let (full_key, _, _) = row(&rows[index..]);
            if (full_key >> 120) as usize != order {
                break;
            }
            let shift = 120 - 12 * order;
            let key = (full_key & ((1 << 120) - 1)) >> shift;
            require(
                full_key == ((order as u128) << 120) | (key << shift),
                "source key padding",
            )?;
            let mut group = Group {
                key,
                ..Group::default()
            };
            while index < rows.len() {
                let (next_key, token, rank) = row(&rows[index..]);
                if next_key != full_key {
                    break;
                }
                require(
                    group.len < 4
                        && (1..4096).contains(&token)
                        && (1..=31).contains(&rank)
                        && rows[index + 19] == 0
                        && (group.len == 0 || group.words[group.len - 1] >> 5 < u32::from(token)),
                    "source successor fields",
                )?;
                group.words[group.len] = (u32::from(token) << 5) | u32::from(rank);
                group.len += 1;
                index += ROW;
            }
            require(
                page.last().is_none_or(|prior: &Group| prior.key < key),
                "source key order",
            )?;
            page.push(group);
            if page.len() == PAGE {
                write_page(&page, &mut directory, &mut body);
                page.clear();
            }
        }
        if !page.is_empty() {
            write_page(&page, &mut directory, &mut body);
            page.clear();
        }
    }
    require(index == rows.len(), "source order outside table")?;
    header.extend(((directory.len() / ENTRY) as u32).to_le_bytes());
    header.extend(directory);
    header.extend(body);
    Ok(header)
}

struct Packed<'a> {
    directory: &'a [u8],
    body: &'a [u8],
    orders: [usize; ORDERS + 1],
}

impl<'a> Packed<'a> {
    fn open(bytes: &'a [u8]) -> Result<Self> {
        require(
            bytes.len() >= HEADER && bytes.starts_with(MAGIC),
            "invalid packed header",
        )?;
        let orders = std::array::from_fn(|index| {
            let offset = 8 + PROVENANCE + 4 * index;
            word(&bytes[offset..offset + 4]) as usize
        });
        let directory_end = orders[ORDERS]
            .checked_mul(ENTRY)
            .and_then(|n| n.checked_add(HEADER));
        let end = directory_end
            .filter(|end| *end <= bytes.len())
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        require(
            orders[0] == 0 && orders.windows(2).all(|w| w[0] <= w[1]),
            "invalid order spans",
        )?;
        let pack = Self {
            directory: &bytes[HEADER..end],
            body: &bytes[end..],
            orders,
        };
        require(pack.offset(0) == 0, "invalid initial offset")?;
        Ok(pack)
    }

    fn offset(&self, page: usize) -> usize {
        if page == self.orders[ORDERS] {
            self.body.len()
        } else {
            word(&self.directory[page * ENTRY..page * ENTRY + 4]) as usize
        }
    }

    fn restart(&self, page: usize) -> u128 {
        word(&self.directory[page * ENTRY + 4..(page + 1) * ENTRY])
    }

    fn page(
        &self,
        order: usize,
        index: usize,
        mut visit: impl FnMut(Group) -> Result<()>,
    ) -> Result<(usize, u128)> {
        let bytes = self
            .body
            .get(self.offset(index)..self.offset(index + 1))
            .filter(|bytes| bytes.len() >= 3)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        let key_len = word(&bytes[..2]) as usize;
        let (mut keys, mut successors) = bytes[2..]
            .split_at_checked(key_len)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        let mut key = self.restart(index);
        let mut pending = 0u32;
        let mut bits = 0;
        let mut seen = 0;
        while !keys.is_empty() {
            let value = take_varint(&mut keys)?;
            let delta = value >> 2;
            key = key.checked_add(delta).ok_or(io::ErrorKind::InvalidData)?;
            require(
                key < (1 << (12 * order)) && (seen == 0) == (delta == 0) && seen < PAGE,
                "invalid packed key or page length",
            )?;
            let mut group = Group {
                key,
                len: (value & 3) as usize + 1,
                ..Group::default()
            };
            let mut previous = 0;
            for entry in &mut group.words[..group.len] {
                while bits < 17 {
                    let (&byte, rest) = successors
                        .split_first()
                        .ok_or(io::ErrorKind::UnexpectedEof)?;
                    successors = rest;
                    pending |= u32::from(byte) << bits;
                    bits += 8;
                }
                *entry = pending & 0x1ffff;
                pending >>= 17;
                bits -= 17;
                let token = *entry >> 5;
                require(token > previous && *entry & 31 != 0, "invalid successor")?;
                previous = token;
            }
            visit(group)?;
            seen += 1;
        }
        require(
            seen > 0
                && successors.is_empty()
                && pending == 0
                && (index + 1 == self.orders[order + 1]
                    || seen == PAGE && key < self.restart(index + 1)),
            "invalid page termination or directory order",
        )?;
        Ok((seen, key))
    }

    fn predecessor(&self, order: usize, key: u128) -> Result<Option<Group>> {
        let start = self.orders[order];
        let mut low = start;
        let mut high = self.orders[order + 1];
        while low < high {
            let mid = low + (high - low) / 2;
            if self.restart(mid) <= key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let mut found = None;
        if low > start {
            self.page(order, low - 1, |group| {
                if group.key <= key {
                    found = Some(group);
                }
                Ok(())
            })?;
        }
        Ok(found)
    }
}

fn verify(pack: &Packed<'_>, source: &[u8]) -> Result<()> {
    let mut rows = source[PROVENANCE..].chunks_exact(ROW);
    let mut consumed = 0;
    let mut queries = 0;
    for order in 0..ORDERS {
        let mut contexts = 0;
        let mut last = None;
        let before = consumed;
        for page in pack.orders[order]..pack.orders[order + 1] {
            let mut first = None;
            let (count, end_key) = pack.page(order, page, |group| {
                first.get_or_insert(group);
                let key = ((order as u128) << 120) | (group.key << (120 - 12 * order));
                for &next in &group.words[..group.len] {
                    let original = rows.next().ok_or(io::ErrorKind::UnexpectedEof)?;
                    require(
                        row(original) == (key, (next >> 5) as u16, (next & 31) as u8)
                            && original[19] == 0,
                        "full tuple roundtrip differs",
                    )?;
                    consumed += 1;
                }
                Ok(())
            })?;
            let first = first.unwrap();
            require(
                pack.predecessor(order, first.key)? == Some(first),
                "restart query differs",
            )?;
            queries += 1;
            if first.key > 0 {
                require(
                    pack.predecessor(order, first.key - 1)?
                        .map(|group| group.key)
                        == last,
                    "preceding page or below-floor lookup differs",
                )?;
                queries += 1;
            }
            last = Some(end_key);
            contexts += count;
        }
        // Interior lookups and exact misses use an independently indexed, flat-row oracle.
        let section = &source[PROVENANCE + before * ROW..PROVENANCE + consumed * ROW];
        let count = consumed - before;
        let mut oldest_flipped_misses = 0;
        for sample in 0..count.min(1024) {
            let offset = sample * count / count.min(1024);
            let (full, _, _) = row(&section[offset * ROW..]);
            let key = (full & ((1 << 120) - 1)) >> (120 - 12 * order);
            let adjacent = key.saturating_add(1).min((1 << (12 * order)) - 1);
            let queries_for_key = [key, adjacent, key ^ 0xfff];
            for (probe, &query) in queries_for_key[..if order == 10 { 3 } else { 2 }]
                .iter()
                .enumerate()
            {
                let original = ((order as u128) << 120) | (query << (120 - 12 * order));
                let mut low = 0;
                let mut high = count;
                while low < high {
                    let mid = low + (high - low) / 2;
                    if row(&section[mid * ROW..]).0 < original {
                        low = mid + 1;
                    } else {
                        high = mid;
                    }
                }
                let mut expected = Group {
                    key: query,
                    ..Group::default()
                };
                for bytes in section[low * ROW..].chunks_exact(ROW).take(4) {
                    let (found, token, rank) = row(bytes);
                    if found != original {
                        break;
                    }
                    expected.words[expected.len] = (u32::from(token) << 5) | u32::from(rank);
                    expected.len += 1;
                }
                require(
                    pack.predecessor(order, query)?.filter(|g| g.key == query)
                        == (expected.len > 0).then_some(expected),
                    "flat exact lookup differs",
                )?;
                if probe == 2 && expected.len == 0 {
                    oldest_flipped_misses += 1;
                }
                queries += 1;
            }
        }
        if order == 10 {
            require(oldest_flipped_misses > 0, "missing long-suffix negative")?;
            println!("order 10: {oldest_flipped_misses} exact misses after changing only the oldest token");
        }
        println!(
            "order {order}: {count} triples, {contexts} contexts, {} pages, {} payload bytes",
            pack.orders[order + 1] - pack.orders[order],
            pack.offset(pack.orders[order + 1]) - pack.offset(pack.orders[order])
        );
    }
    require(
        rows.next().is_none() && consumed == 14_634_359,
        "triple total differs",
    )?;
    println!("roundtrip {consumed} triples; {queries} directory and flat-oracle queries");
    Ok(())
}

fn negatives(bytes: &mut [u8]) -> Result<()> {
    require(
        Packed::open(&bytes[..HEADER - 1]).is_err(),
        "short header accepted",
    )?;
    let end = bytes.len() - 1;
    let short = Packed::open(&bytes[..end])?;
    require(
        short.page(10, short.orders[11] - 1, |_| Ok(())).is_err(),
        "short final page accepted",
    )?;
    let first = HEADER + Packed::open(bytes)?.directory.len();
    let original = bytes[first + 2];
    bytes[first + 2] = 4; // the first key must be the restart plus a zero delta
    let bad = Packed::open(bytes)?;
    require(
        bad.page(0, 0, |_| Ok(())).is_err(),
        "false restart accepted",
    )?;
    bytes[first + 2] = original;
    let first_successor = first + 2 + word(&bytes[first..first + 2]) as usize;
    let original = bytes[first_successor];
    bytes[first_successor] &= !31;
    let bad = Packed::open(bytes)?;
    require(bad.page(0, 0, |_| Ok(())).is_err(), "zero rank accepted")?;
    bytes[first_successor] = original;
    println!("negative control: zero rank");
    let last = bytes[HEADER];
    bytes[HEADER] = 1;
    require(Packed::open(bytes).is_err(), "false body offset accepted")?;
    bytes[HEADER] = last;
    println!("negative controls: short header, truncated final page, nonzero restart delta, bad initial offset");
    Ok(())
}

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [input, output] = args.as_slice() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: s2-pack-audit PINNED_ACTIVE_D11 NEW_PACK_PATH",
        ));
    };
    let input = read_source(Path::new(input))?;
    const BASE: usize = 86_536_132;
    const START: usize = 86_536_384;
    const CAP: usize = 125_998_708;
    let source = &input[START..];
    let bytes = encode(source)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    drop(bytes);
    let mut reopened = fs::read(output)?;
    require(
        reopened[8..8 + PROVENANCE] == source[..PROVENANCE],
        "source provenance differs",
    )?;
    let pack = Packed::open(&reopened)?;
    verify(&pack, source)?;
    println!("header {HEADER}; directory {}; body {}; section {}; base {BASE}; total {}; cap {CAP}; over cap {}",
        pack.directory.len(), pack.body.len(), reopened.len(), BASE + reopened.len(),
        (BASE + reopened.len()).saturating_sub(CAP));
    negatives(&mut reopened)
}
