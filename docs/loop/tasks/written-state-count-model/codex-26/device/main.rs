#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_discrete::{
    d11_export::{DiscreteLutHeader, MAGIC},
    normalization_mechanism::{NormFrameTable, NormalizationMode},
    position_mechanism::PositionStepTable,
    trunk::{compute_recurrence_state, RecurrenceMode},
};
use uor_r4_integer::discrete_d11::{
    spiralcore_orbit_roots, IntegerDiscreteModel, DISCRETE_SCHEMA, DISCRETE_TRUNK_SCHEMA,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Coordinates = [i32; 32];

#[derive(Clone)]
struct Candidate {
    row: u32,
    address: u32,
    narrow: u8,
    wide: u8,
    label: i8,
    local: Coordinates,
    context: Coordinates,
}

fn words<T: std::str::FromStr, const N: usize>(fields: &[&str]) -> Result<[T; N]>
where
    T::Err: Error + 'static,
{
    if fields.len() != N {
        return Err(format!("expected {N} fields, found {}", fields.len()).into());
    }
    let mut values = Vec::with_capacity(N);
    for field in fields {
        values.push(field.parse()?);
    }
    values
        .try_into()
        .map_err(|_| "incorrect field count".into())
}

fn root_coordinates(code: u16) -> Coordinates {
    let roots = spiralcore_orbit_roots();
    std::array::from_fn(|i| i32::from(roots[usize::from((code >> (4 * (i / 8))) & 15)][i % 8]))
}

fn read_rows(path: &Path) -> Result<Vec<Candidate>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some("D25ROWS1") {
        return Err("missing D25ROWS1 header".into());
    }
    let mut rows: Vec<Candidate> = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() != 69 {
            return Err("expected 69 candidate fields".into());
        }
        let [row, address] = words(&fields[..2])?;
        let [narrow, wide] = words(&fields[2..4])?;
        let [label] = words(&fields[4..5])?;
        let local: Coordinates = words(&fields[5..37])?;
        let context = words(&fields[37..])?;
        if narrow > wide
            || wide > 1
            || !(-1..=1).contains(&label)
            || local.iter().any(|x| !(-48..=48).contains(x))
            || (wide == 0 && (address != 0 || label != 0 || local != [0; 32] || context != [0; 32]))
            || rows.last().is_some_and(|p| {
                (p.row, p.address) >= (row, address)
                    || (p.row == row && (p.wide == 0 || wide == 0 || p.context != context))
            })
        {
            return Err(format!("invalid D25 candidate row {row}, address {address}").into());
        }
        rows.push(Candidate {
            row,
            address,
            narrow,
            wide,
            label,
            local,
            context,
        });
    }
    if rows.is_empty() {
        return Err("empty D25ROWS1 packet".into());
    }
    Ok(rows)
}

fn write_rows(
    path: &Path,
    rows: &[Candidate],
    contexts: &BTreeMap<u32, Coordinates>,
) -> Result<()> {
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    writeln!(out, "D25ROWS1")?;
    for row in rows {
        let context = contexts.get(&row.row).unwrap_or(&row.context);
        write!(
            out,
            "{} {} {} {} {}",
            row.row, row.address, row.narrow, row.wide, row.label
        )?;
        for word in row.local.iter().chain(context) {
            write!(out, " {word}")?;
        }
        writeln!(out)?;
    }
    out.flush()?;
    Ok(())
}

fn oracle(
    base: &Path,
) -> Result<(
    IntegerDiscreteModel,
    Vec<[u8; 4]>,
    PositionStepTable,
    NormFrameTable,
)> {
    let bytes = fs::read(base)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA {
        return Err("expected original 19-table D11 trunk".into());
    }
    let head_end = 16 + usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let mut header: DiscreteLutHeader = serde_json::from_slice(&bytes[16..head_end])?;
    let data = &bytes[head_end.next_multiple_of(64)..];
    let section = |name: &str| -> Result<&[u8]> {
        let span = &header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or("missing D11 table")?
            .span;
        let start = usize::try_from(span.offset)?;
        let end = start + usize::try_from(span.bytes)?;
        Ok(&data[start..end])
    };
    let frames = |name| -> Result<Vec<u8>> {
        section(name)?
            .chunks_exact(4)
            .map(|x| Ok(u8::try_from(u32::from_le_bytes(x.try_into()?))?))
            .collect()
    };
    let steps = PositionStepTable::new(4, frames("discrete.trunk.step_candidates")?)?;
    let norms = NormFrameTable::new(4, frames("discrete.trunk.norm_candidates")?)?;
    let roots = model
        .packed_codebook_roots()
        .iter()
        .map(|&root| std::array::from_fn(|b| ((root >> (4 * b)) & 15) as u8))
        .collect();
    // Use the same native serving Relation read on the artifact's exact original
    // base tables. Omitting the optional trunk is solely the oracle for its input.
    let table_names = [
        "discrete.table",
        "discrete.cells",
        "discrete.routes",
        "discrete.gates",
    ];
    let mut read_tables = Vec::new();
    for name in table_names {
        let entry = header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or("missing base D11 table")?;
        read_tables.push((entry.clone(), section(name)?));
    }
    header.schema = DISCRETE_SCHEMA.into();
    header.tables.clear();
    let mut projection = Vec::new();
    for (mut entry, table) in read_tables {
        projection.resize(projection.len().next_multiple_of(64), 0);
        entry.span.offset = projection.len() as u64;
        projection.extend_from_slice(table);
        header.tables.push(entry);
    }
    let encoded = serde_json::to_vec(&header)?;
    let mut read_bytes = MAGIC.to_vec();
    read_bytes.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    read_bytes.extend_from_slice(&encoded);
    read_bytes.resize(read_bytes.len().next_multiple_of(64), 0);
    read_bytes.extend_from_slice(&projection);
    Ok((
        IntegerDiscreteModel::from_bytes(&read_bytes)?,
        roots,
        steps,
        norms,
    ))
}

fn load_prefixes(path: &Path) -> Result<BTreeMap<u32, Vec<u16>>> {
    let mut prefixes = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let mut fields = line.split_ascii_whitespace();
        let row = fields.next().ok_or("missing prefix row")?.parse()?;
        let tokens = fields
            .map(str::parse)
            .collect::<std::result::Result<Vec<u16>, _>>()?;
        if tokens.is_empty()
            || prefixes
                .last_key_value()
                .is_some_and(|(&prior, _)| row <= prior)
        {
            return Err("empty or unordered prefix".into());
        }
        prefixes.insert(row, tokens);
    }
    Ok(prefixes)
}

fn pack(
    base: &Path,
    prefixes: &Path,
    input: &Path,
    post: &Path,
    trace: &Path,
    out: &Path,
) -> Result<()> {
    let rows = read_rows(input)?;
    let prefixes = load_prefixes(prefixes)?;
    let (read_model, codebook, steps, norms) = oracle(base)?;
    let post = fs::read_to_string(post)?;
    let trace = fs::read_to_string(trace)?;
    let expected_post: BTreeMap<_, _> = rows
        .iter()
        .filter(|r| r.wide == 1)
        .map(|r| (r.row, r.context))
        .collect();
    if expected_post.keys().ne(prefixes.keys())
        || post.lines().count() != prefixes.len()
        || trace.lines().count() != prefixes.len()
    {
        return Err("prefix, candidate and GPU record coverage differ".into());
    }
    let mut relation = BTreeMap::new();
    let mut recurrence = BTreeMap::new();
    for ((&row, tokens), (post_line, trace_line)) in
        prefixes.iter().zip(post.lines().zip(trace.lines()))
    {
        let post: Vec<_> = post_line.split_ascii_whitespace().collect();
        let trace: Vec<_> = trace_line.split_ascii_whitespace().collect();
        if post.len() != 33 || trace.len() != 42 {
            return Err(format!("bad GPU record widths at row {row}").into());
        }
        let [trace_row, length, query] = words::<u32, 3>(&trace[..3])?;
        let [read_code, rec_code] = words::<u16, 2>(&trace[3..5])?;
        let [output_code] = words::<u16, 1>(&trace[9..10])?;
        let original_prefix: Vec<u32> = tokens.iter().copied().map(u32::from).collect();
        let raw: Coordinates = words(&trace[10..])?;
        let post_coords: Coordinates = words(&post[1..])?;
        let (native_recurrence, native_rec_code) = compute_recurrence_state(
            tokens,
            &codebook,
            &steps,
            &norms,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            0,
        )?;
        let rec_coords = root_coordinates(rec_code);
        if post[0].parse::<u32>()? != row
            || trace_row != row
            || length as usize != tokens.len()
            || Some(&query) != original_prefix.last()
            || post_coords != expected_post[&row]
            || post_coords != root_coordinates(output_code)
            || raw != root_coordinates(read_code)
            || raw.as_slice() != read_model.evaluate_prefix(&original_prefix)?.read_vector
            || rec_code != native_rec_code
            || rec_coords != native_recurrence
        {
            return Err(format!("native parity or GPU record identity failed at row {row}").into());
        }
        relation.insert(row, raw);
        recurrence.insert(row, rec_coords);
    }
    fs::create_dir(out)?;
    write_rows(&out.join("post.rows"), &rows, &BTreeMap::new())?;
    write_rows(&out.join("relation.rows"), &rows, &relation)?;
    write_rows(&out.join("recurrence.rows"), &rows, &recurrence)?;
    println!(
        "PASS post, Relation and occurrence recurrence parity for {} original prefixes",
        prefixes.len()
    );
    Ok(())
}

fn donors(input: &Path, map: &Path, out: &Path) -> Result<()> {
    let rows = read_rows(input)?;
    if rows.iter().any(|r| r.label != 0) {
        return Err("donor map requires prediction-only rows".into());
    }
    let contexts: BTreeMap<_, _> = rows
        .iter()
        .filter(|r| r.wide == 1)
        .map(|r| (r.row, r.context))
        .collect();
    let mut swaps = BTreeMap::new();
    for line in fs::read_to_string(map)?.lines() {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let [row, donor] = words::<u32, 2>(&fields)?;
        if row == donor
            || !contexts.contains_key(&row)
            || !contexts.contains_key(&donor)
            || swaps.insert(row, donor).is_some()
        {
            return Err(format!("invalid donor map entry {row} {donor}").into());
        }
    }
    let targets: BTreeSet<_> = swaps.keys().copied().collect();
    let sources: BTreeSet<_> = swaps.values().copied().collect();
    if swaps.is_empty() || sources != targets {
        return Err("donors must form a nonempty permutation".into());
    }
    let donated = swaps
        .into_iter()
        .map(|(row, donor)| (row, contexts[&donor]))
        .collect();
    write_rows(out, &rows, &donated)
}

fn finite(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut train = Vec::new();
    let mut held = Vec::new();
    for (row, code, positive) in [(1, 0, 1), (2, u16::MAX, 2)] {
        for address in 1..=2 {
            let mut candidate = Candidate {
                row,
                address,
                narrow: 1,
                wide: 1,
                label: if address == positive { 1 } else { -1 },
                local: [0; 32],
                context: root_coordinates(code),
            };
            candidate.local[0] = 4 * (address as i32 - 1);
            train.push(candidate.clone());
            candidate.label = 0;
            held.push(candidate);
        }
    }
    let map = out.join("donors.tsv");
    let input = out.join("held.rows");
    fs::write(&map, "1\t2\n2\t1\n")?;
    write_rows(&out.join("train.rows"), &train, &BTreeMap::new())?;
    write_rows(&input, &held, &BTreeMap::new())?;
    donors(&input, &map, &out.join("held-donors.rows"))?;
    for (name, addresses) in [("native", [1, 2]), ("donors", [2, 1])] {
        let mut expected = String::new();
        for (index, address) in addresses.into_iter().enumerate() {
            let row = index + 1;
            expected.push_str(&format!(
                "{row}\t0\tnonpositive\tNA\n{row}\t1\tnonpositive\tNA\n{row}\t2\tpick\t{address}\n{row}\t3\tpick\t{address}\n"
            ));
        }
        fs::write(out.join(format!("{name}.expected.tsv")), expected)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os()
        .skip(1)
        .map(std::path::PathBuf::from)
        .collect();
    match args.as_slice() {
        [base, prefixes, input, post, trace, out] => pack(base, prefixes, input, post, trace, out),
        [command, input, map, out] if command == Path::new("donors") => donors(input, map, out),
        [command, out] if command == Path::new("finite") => finite(out),
        _ => Err("usage: d26-view-pack BASE_D11 PREFIX_TSV SOURCE_ROWS GPU_POST GPU_TRACE OUTPUT_DIR | donors INPUT_ROWS DONOR_MAP OUTPUT_ROWS | finite OUTPUT_DIR".into()),
    }
}
