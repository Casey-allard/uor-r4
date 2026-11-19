#![forbid(unsafe_code)]

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use uor_r4_discrete::d11_export::DiscreteLutHeader;
use uor_r4_discrete::metadata::{BinderPlan, ABSENT};
use uor_r4_integer::discrete_d11::{IntegerDiscreteModel, DISCRETE_TRUNK_SCHEMA};

mod geometry {
    #![allow(dead_code)]
    include!(env!("D25_DISCRETE_GEOMETRY"));
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn words(output: &mut impl Write, words: impl IntoIterator<Item = u32>) -> Result<()> {
    for word in words {
        output.write_all(&word.to_le_bytes())?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    let [_, artifact, prefixes, packet] = args.as_slice() else {
        return Err("usage: d25-context-pack BASE_D11 PREFIX_TSV OUT_PACKET".into());
    };
    let bytes = fs::read(artifact)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA {
        return Err("expected original 19-table D11 trunk".into());
    }
    let header_end = 16 + usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let header: DiscreteLutHeader = serde_json::from_slice(&bytes[16..header_end])?;
    let shape = &header.shape;
    if shape.vocabulary != 4096
        || shape.blocks != 4
        || shape.context != 192
        || shape.read_mode != "relation"
        || shape.read_kept != 8
        || shape.read_sweeps != 1
        || shape.read_position != 0
        || shape.read_initialization != "active_leaf_v1"
        || shape.kept_cells != 8
        || shape.objective_policy != "nearest_address_margin_v1"
    {
        return Err("unexpected frozen D25 trunk shape".into());
    }
    let data = &bytes[header_end.next_multiple_of(64)..];
    let table = |name: &str| -> Result<&[u8]> {
        let table = header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or("missing table")?;
        let offset = usize::try_from(table.span.offset)?;
        let len = usize::try_from(table.span.bytes)?;
        Ok(data
            .get(offset..offset.checked_add(len).ok_or("table length overflow")?)
            .ok_or("invalid table span")?)
    };

    let input = fs::read_to_string(prefixes)?;
    let mut rows = Vec::new();
    for line in input.lines() {
        let mut fields = line.split_ascii_whitespace().map(str::parse::<u32>);
        let row = fields.next().ok_or("missing row identifier")??;
        let tokens: Vec<u32> = fields.collect::<std::result::Result<_, _>>()?;
        if tokens.is_empty()
            || tokens.len() > shape.context as usize
            || tokens.iter().any(|&token| token >= shape.vocabulary)
            || rows.last().is_some_and(|(prior, _)| row <= *prior)
        {
            return Err(
                "row identifiers must increase and original prefixes must fit context".into(),
            );
        }
        rows.push((row, tokens));
    }
    if rows.is_empty() || rows.len() > 4096 {
        return Err("expected 1..4096 prefixes".into());
    }

    let mut output = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(packet)?,
    );
    output.write_all(b"D25CTX01")?;
    let max_plan = BinderPlan::new(shape.context as usize)?;
    let nodes = max_plan.nodes_count();
    if max_plan.stages() != shape.stages as usize {
        return Err("D11 and binder stage counts differ".into());
    }
    words(
        &mut output,
        [
            shape.vocabulary,
            shape.blocks,
            shape.context,
            shape.kept_cells,
            rows.len() as u32,
            shape.stages,
            nodes as u32,
            max_plan.frontier().len() as u32,
            2,
            shape.read_kept,
            shape.read_sweeps,
            0,
            0,
            0,
            0,
            0,
        ],
    )?;
    words(
        &mut output,
        geometry::ROOTS.iter().flatten().map(|&x| x as u32),
    )?;
    for bytes in [
        &geometry::AXES[..],
        &geometry::COMPOSE[..],
        &geometry::BFS[..],
        &geometry::BRANCH[..],
        &geometry::GENERATORS[..],
        &geometry::INVERSE[..],
    ] {
        words(&mut output, bytes.iter().map(|&x| u32::from(x)))?;
    }
    words(&mut output, [u32::from(geometry::IDENTITY)])?;
    for name in [
        "discrete.table",
        "discrete.routes",
        "discrete.gates",
        "discrete.trunk.step_candidates",
        "discrete.trunk.norm_candidates",
        "discrete.trunk.unigram",
    ] {
        output.write_all(table(name)?)?;
    }
    for name in [
        "discrete.trunk.query_table",
        "discrete.trunk.read_table",
        "discrete.trunk.rec_table",
        "discrete.trunk.query_read_table",
        "discrete.trunk.query_rec_table",
        "discrete.trunk.read_rec_table",
    ] {
        let triples = table(name)?;
        words(&mut output, [u32::try_from(triples.len() / 12)?])?;
        let mut previous = None;
        for triplet in triples.chunks_exact(12) {
            let key = u32::from_le_bytes(triplet[0..4].try_into()?);
            let token = u32::from_le_bytes(triplet[4..8].try_into()?);
            if previous.is_some_and(|p| p >= (key, token)) {
                return Err("trunk count records not in canonical order".into());
            }
            previous = Some((key, token));
        }
        output.write_all(triples)?;
    }
    for (row, tokens) in rows {
        let plan = BinderPlan::new(tokens.len())?;
        words(
            &mut output,
            [
                row,
                tokens.len() as u32,
                plan.nodes_count() as u32,
                plan.frontier().len() as u32,
            ],
        )?;
        words(&mut output, plan.node_words())?;
        words(
            &mut output,
            std::iter::repeat_n(ABSENT, 4 * (nodes - plan.nodes_count())),
        )?;
        words(&mut output, plan.frontier().iter().copied())?;
        words(
            &mut output,
            std::iter::repeat_n(ABSENT, 16 - plan.frontier().len()),
        )?;
        words(&mut output, tokens.iter().copied())?;
        words(
            &mut output,
            std::iter::repeat_n(tokens[0], shape.context as usize - tokens.len()),
        )?;
    }
    output.flush()?;
    Ok(())
}
