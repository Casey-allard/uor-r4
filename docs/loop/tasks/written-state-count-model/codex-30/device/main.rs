#![forbid(unsafe_code)]

use serde_json::Value;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_discrete::{
    d11_export::DiscreteLutHeader,
    normalization_mechanism::{NormFrameTable, NormalizationMode},
    position_mechanism::PositionStepTable,
    trunk::{compute_recurrence_state, RecurrenceMode},
};
use uor_r4_integer::{
    discrete_d11::{
        contextual_source::{SourceArm, SourceBank, SourceCandidate, SourceOutcome},
        IntegerDiscreteModel,
    },
    paired_role::RoleSpan,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn create(path: &Path) -> Result<BufWriter<File>> {
    Ok(BufWriter::new(File::create_new(path)?))
}

fn unsigned(value: &Value) -> Result<usize> {
    Ok(usize::try_from(
        value.as_u64().ok_or("expected unsigned JSON number")?,
    )?)
}

fn ids(value: &Value) -> Result<Vec<u16>> {
    value
        .as_array()
        .ok_or("missing tape")?
        .iter()
        .map(|v| Ok(u16::try_from(unsigned(v)?)?))
        .collect()
}

fn pairs(map: &Path) -> Result<Vec<(u32, u32)>> {
    let mut donors = BTreeMap::new();
    for line in fs::read_to_string(map)?.lines() {
        let (left, right) = line.split_once('\t').ok_or("malformed donor map")?;
        let (left, right) = (left.parse()?, right.parse()?);
        if left == right || donors.insert(left, right).is_some() {
            return Err("donor map repeats an identity".into());
        }
    }
    if donors.is_empty() || donors.iter().any(|(a, b)| donors.get(b) != Some(a)) {
        return Err("donor map must consist of reciprocal pairs".into());
    }
    Ok(donors.into_iter().filter(|(a, b)| a < b).collect())
}

fn prepare(grade: &Path, map: &Path, out: &Path) -> Result<()> {
    let grade: Vec<Value> = serde_json::from_slice(&fs::read(grade)?)?;
    let rows: BTreeMap<_, _> = grade
        .iter()
        .map(|row| Ok((u32::try_from(unsigned(&row["row"])?)?, row)))
        .collect::<Result<_>>()?;
    if rows.len() != grade.len() {
        return Err("duplicate grade row".into());
    }
    let mut packet = create(&out.join("trajectory-input.tsv"))?;
    let mut manifest = create(&out.join("trajectory-input-index.tsv"))?;
    let mut serial = 0u32;
    for (a, b) in pairs(map)? {
        let a_row = rows.get(&a).ok_or("missing first grade row")?;
        let b_row = rows.get(&b).ok_or("missing second grade row")?;
        let a_ids = ids(&a_row["original_ids"])?;
        let b_ids = ids(&b_row["original_ids"])?;
        let a_fact = unsigned(&a_row["fact_and_query"][0][0])?;
        let b_fact = unsigned(&b_row["fact_and_query"][0][0])?;
        let a_suffix = a_ids.get(a_fact..).ok_or("invalid first fact start")?;
        let b_suffix = b_ids.get(b_fact..).ok_or("invalid second fact start")?;
        if a_suffix.is_empty() || a_suffix != b_suffix {
            return Err(format!("original suffix mismatch for {a} and {b}").into());
        }
        if a_fact.min(b_fact) == 0 || a_ids.len().max(b_ids.len()) > 192 {
            return Err("tape outside CUDA prefix contract".into());
        }
        // Retain BOS alignment for the first divergence and factual-start
        // alignment for the recurrence comparison over the identical suffix.
        for (row, fact, tape) in [(a, a_fact, &a_ids), (b, b_fact, &b_ids)] {
            for end in 1..=tape.len() {
                serial += 1;
                let offset = isize::try_from(end)? - isize::try_from(fact)?;
                write!(packet, "{serial}")?;
                for id in &tape[..end] {
                    write!(packet, "\t{id}")?;
                }
                writeln!(packet)?;
                writeln!(manifest, "{serial}\t{a}\t{b}\t{offset}\t{row}\t{end}")?;
            }
        }
    }
    if serial > 4096 {
        return Err("trajectory has more than 4096 GPU prefix rows".into());
    }
    Ok(())
}

fn recurrence_oracle(base: &Path, packets: &Path, trace: &Path, out: &Path) -> Result<()> {
    let raw = fs::read(base)?;
    let model = IntegerDiscreteModel::from_bytes(&raw)?;
    let end = 16 + usize::try_from(u64::from_le_bytes(raw[8..16].try_into()?))?;
    let header: DiscreteLutHeader = serde_json::from_slice(&raw[16..end])?;
    let data = &raw[end.next_multiple_of(64)..];
    let frames = |name: &str| -> Result<Vec<u8>> {
        let table = header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or("missing trunk table")?;
        let start = usize::try_from(table.span.offset)?;
        let bytes = usize::try_from(table.span.bytes)?;
        let table = data
            .get(start..start.checked_add(bytes).ok_or("table overflow")?)
            .ok_or("trunk table outside artifact")?;
        table
            .chunks_exact(4)
            .map(|w| Ok(u8::try_from(u32::from_le_bytes(w.try_into()?))?))
            .collect()
    };
    let steps = PositionStepTable::new(4, frames("discrete.trunk.step_candidates")?)?;
    let norms = NormFrameTable::new(4, frames("discrete.trunk.norm_candidates")?)?;
    let roots: Vec<[u8; 4]> = model
        .packed_codebook_roots()
        .iter()
        .map(|&root| std::array::from_fn(|block| ((root >> (4 * block)) & 15) as u8))
        .collect();
    let packets = fs::read_to_string(packets)?;
    let trace = fs::read_to_string(trace)?;
    let index = fs::read_to_string(out.join("trajectory-input-index.tsv"))?;
    if packets.lines().count() != trace.lines().count()
        || trace.lines().count() != index.lines().count()
    {
        return Err("trajectory prefix, device and manifest counts differ".into());
    }
    let mut output = create(&out.join("trajectory.tsv"))?;
    let mut tracks = BTreeMap::<(u32, u32), BTreeMap<u32, Vec<(isize, u16, u16)>>>::new();
    writeln!(
        output,
        "prefix\tleft\tright\toffset\trow\tlength\tlast_id\tpacked_code"
    )?;
    for ((prefix, gpu), index) in packets.lines().zip(trace.lines()).zip(index.lines()) {
        let mut fields = prefix.split_whitespace();
        let id = fields.next().ok_or("missing prefix row")?;
        let ids: Vec<u16> = fields
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;
        let gpu: Vec<_> = gpu.split_whitespace().collect();
        if gpu.len() != 42
            || gpu[0] != id
            || gpu[1].parse::<usize>()? != ids.len()
            || !index.starts_with(&format!("{id}\t"))
        {
            return Err(format!("invalid trajectory device record {id}").into());
        }
        let (_, code) = compute_recurrence_state(
            &ids,
            &roots,
            &steps,
            &norms,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            0,
        )?;
        let last = ids.last().ok_or("empty prefix")?;
        if gpu[2].parse::<u16>()? != *last || gpu[4].parse::<u16>()? != code {
            return Err(format!("recurrence parity failure on trajectory prefix {id}").into());
        }
        writeln!(output, "{index}\t{last}\t{code}")?;
        let fields: Vec<_> = index.split_whitespace().collect();
        if fields.len() != 6 || fields[5].parse::<usize>()? != ids.len() {
            return Err(format!("invalid trajectory manifest row {id}").into());
        }
        tracks
            .entry((fields[1].parse()?, fields[2].parse()?))
            .or_default()
            .entry(fields[4].parse()?)
            .or_default()
            .push((fields[3].parse()?, *last, code));
    }
    let mut summary = create(&out.join("trajectory-pairs.tsv"))?;
    writeln!(summary, "left\tright\tfirst_token_difference_bos_length\tfirst_token_left\tfirst_token_right\tfirst_recurrence_difference_bos_length\tboundary_left\tboundary_right\tfirst_equal_shared_offset\tequal_prefixes_shared\tshared_prefixes_including_boundary")?;
    for ((a, b), track) in tracks {
        let a_track = track.get(&a).ok_or("missing left trajectory")?;
        let b_track = track.get(&b).ok_or("missing right trajectory")?;
        let first_id = a_track
            .iter()
            .zip(b_track)
            .enumerate()
            .find(|(_, (a, b))| a.1 != b.1);
        let first_code = a_track.iter().zip(b_track).position(|(a, b)| a.2 != b.2);
        let a_shared: Vec<_> = a_track.iter().filter(|x| x.0 >= 0).collect();
        let b_shared: Vec<_> = b_track.iter().filter(|x| x.0 >= 0).collect();
        let (Some(a_boundary), Some(b_boundary)) = (a_shared.first(), b_shared.first()) else {
            return Err("missing factual boundary observation".into());
        };
        if a_shared.len() != b_shared.len() || a_boundary.0 != 0 || b_boundary.0 != 0 {
            return Err("invalid shared trajectory lengths".into());
        }
        let equal: Vec<_> = a_shared
            .iter()
            .zip(&b_shared)
            .filter_map(|(a, b)| (a.2 == b.2).then_some(a.0))
            .collect();
        let (first_len, first_a, first_b) = first_id.map_or((None, None, None), |(i, (a, b))| {
            (
                Some(i as i64 + 1),
                Some(i64::from(a.1)),
                Some(i64::from(b.1)),
            )
        });
        writeln!(
            summary,
            "{a}\t{b}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            number(first_len),
            number(first_a),
            number(first_b),
            number(first_code.map(|i| i as i64 + 1)),
            a_boundary.2,
            b_boundary.2,
            number(equal.first().map(|&i| i as i64)),
            equal.len(),
            a_shared.len()
        )?;
    }
    Ok(())
}

fn zero(input: &Path, output: &Path) -> Result<()> {
    let text = fs::read_to_string(input)?;
    let mut lines = text.lines();
    if lines.next() != Some("D25ROWS1") {
        return Err("bad source-row header".into());
    }
    let mut out = create(output)?;
    writeln!(out, "D25ROWS1")?;
    for line in lines {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 69 {
            return Err("invalid source-row record".into());
        }
        write!(out, "{}", fields[..37].join(" "))?;
        for _ in 0..32 {
            write!(out, " 0")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn field_array<T: std::str::FromStr, const N: usize>(fields: &[&str]) -> Result<[T; N]>
where
    T::Err: Error + 'static,
{
    fields
        .iter()
        .map(|s| Ok(s.parse()?))
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| "invalid vector length".into())
}

fn rows(input: &Path) -> Result<BTreeMap<u32, Vec<(SourceCandidate, bool)>>> {
    let text = fs::read_to_string(input)?;
    let mut lines = text.lines();
    if lines.next() != Some("D25ROWS1") {
        return Err("bad source-row header".into());
    }
    let mut rows = BTreeMap::<_, Vec<_>>::new();
    for line in lines {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 69 || fields[4] != "0" {
            return Err("invalid prediction row".into());
        }
        let [row, address] = field_array::<u32, 2>(&fields[..2])?;
        let start = usize::try_from(address)?;
        rows.entry(row).or_default().push((
            SourceCandidate {
                span: RoleSpan {
                    start,
                    end: start + 1,
                },
                narrow: fields[2] == "1",
                local: field_array(&fields[5..37])?,
                contextual: field_array(&fields[37..])?,
                completed_source_range: [address, address + 1],
            },
            fields[3] == "1",
        ));
    }
    Ok(rows)
}

fn number(value: Option<i64>) -> String {
    value.map_or_else(|| "NA".to_owned(), |v| v.to_string())
}

fn score_oracle(input: &Path, bank: &Path, gpu_trace: &Path, gpu_results: &Path) -> Result<()> {
    let bank = SourceBank::from_bytes(&fs::read(bank)?)?;
    let mut trace = BTreeMap::new();
    let mut results = String::new();
    for (row, candidates) in rows(input)? {
        for (i, arm) in [
            SourceArm::NarrowLocal,
            SourceArm::WideLocal,
            SourceArm::NarrowContextual,
            SourceArm::WideContextual,
        ]
        .into_iter()
        .enumerate()
        {
            let selection = bank.select(
                arm,
                candidates
                    .iter()
                    .filter(|(_, wide)| *wide)
                    .map(|(candidate, _)| Ok(candidate.clone())),
                |_, _| {},
            )?;
            for (candidate, wide) in &candidates {
                let score = bank.score(arm, candidate);
                let eligible = *wide && score.eligible;
                let address = score.source;
                trace.insert(
                    (row, address, i),
                    format!(
                        "{row}\t{address}\t{i}\t{}\t{}\t{}\t{}\n",
                        u8::from(eligible),
                        number(score.nearest_positive),
                        number(score.nearest_negative),
                        number(score.margin)
                    ),
                );
            }
            let reason = match selection.outcome {
                SourceOutcome::Pick => "pick",
                SourceOutcome::Empty => "empty",
                SourceOutcome::Nonpositive => "nonpositive",
                SourceOutcome::Tie => "tie",
            };
            results.push_str(&format!(
                "{row}\t{i}\t{reason}\t{}\n",
                number(selection.source.map(i64::from))
            ));
        }
    }
    if trace.into_values().collect::<String>() != fs::read_to_string(gpu_trace)?
        || results != fs::read_to_string(gpu_results)?
    {
        return Err("native integer and CUDA score traces or decisions differ".into());
    }
    Ok(())
}

fn loaded_oracle(carrier: &Path, grade: &Path, input: &Path) -> Result<()> {
    let model = IntegerDiscreteModel::from_bytes(&fs::read(carrier)?)?;
    if model.contextual_source_arm() != Some(SourceArm::WideRecurrence) {
        return Err("loaded carrier is not the D26 wide recurrence arm".into());
    }
    let grades: Vec<Value> = serde_json::from_slice(&fs::read(grade)?)?;
    let mut groups = rows(input)?;
    let span = |value: &Value| -> Result<RoleSpan> {
        Ok(RoleSpan {
            start: unsigned(&value[0])?,
            end: unsigned(&value[1])?,
        })
    };
    let mut compared = 0;
    for grade in grades {
        let row = u32::try_from(unsigned(&grade["row"])?)?;
        let packed = groups
            .remove(&row)
            .ok_or("missing packed candidate group")?;
        let spans = &grade["fact_and_query"];
        if spans.is_null() {
            if packed.iter().any(|(_, wide)| *wide) {
                return Err(format!("row {row} has candidates without a supplied fact").into());
            }
            continue;
        }
        let original: Vec<_> = ids(&grade["original_ids"])?
            .into_iter()
            .map(u32::from)
            .collect();
        let projected =
            model.contextual_source_candidates(&original, span(&spans[0])?, span(&spans[1])?)?;
        let packed: Vec<_> = packed
            .into_iter()
            .filter_map(|(p, wide)| wide.then_some(p))
            .collect();
        if projected.len() != packed.len()
            || projected.iter().zip(&packed).any(|(native, p)| {
                native.span.start != p.span.start
                    || native.narrow != p.narrow
                    || native.local != p.local
                    || native.contextual != p.contextual
            })
        {
            return Err(format!(
                "loaded carrier and nominal CUDA candidate words differ at row {row}"
            )
            .into());
        }
        compared += projected.len();
    }
    if !groups.is_empty() || compared == 0 {
        return Err("nominal CUDA and carrier coverage differ".into());
    }
    println!("PASS loaded D26 native original address, eligibility, local and context words for {compared} candidates");
    Ok(())
}

fn relative_calls(
    path: &Path,
    arm: &str,
    fact: &BTreeMap<u32, usize>,
) -> Result<BTreeMap<u32, (String, Option<usize>)>> {
    let text = fs::read_to_string(path)?;
    let mut calls = BTreeMap::new();
    for line in text.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err("invalid source decision record".into());
        }
        let row = fields[0].parse()?;
        if fields[1] == arm && fact.contains_key(&row) {
            let start = if fields[3] == "NA" {
                None
            } else {
                Some(
                    fields[3]
                        .parse::<usize>()?
                        .checked_sub(fact[&row])
                        .ok_or("decision before factual start")?,
                )
            };
            if calls.insert(row, (fields[2].into(), start)).is_some() {
                return Err("duplicate source decision".into());
            }
        }
    }
    Ok(calls)
}

fn compare(grade: &Path, map: &Path, dir: &Path, output: &Path) -> Result<()> {
    let grades: Vec<Value> = serde_json::from_slice(&fs::read(grade)?)?;
    let facts: BTreeMap<u32, usize> = grades
        .iter()
        .filter(|g| !g["fact_and_query"].is_null())
        .map(|g| {
            Ok((
                u32::try_from(unsigned(&g["row"])?)?,
                unsigned(&g["fact_and_query"][0][0])?,
            ))
        })
        .collect::<Result<_>>()?;
    let reference = rows(&dir.join("score-in/nominal.rows"))?;
    let swap = rows(&dir.join("score-in/swap.rows"))?;
    let zero = rows(&dir.join("score-in/zero.rows"))?;
    let pairs = pairs(map)?;
    let donors: BTreeMap<_, _> = pairs.iter().flat_map(|&(a, b)| [(a, b), (b, a)]).collect();
    let same = |p: &SourceCandidate, wide, q: &SourceCandidate, other| {
        p.span == q.span && p.narrow == q.narrow && p.local == q.local && wide == other
    };
    for (&row, col) in &reference {
        let swapped = swap.get(&row).ok_or("missing swapped group")?;
        let zeroed = zero.get(&row).ok_or("missing zero group")?;
        let donor = reference
            .get(donors.get(&row).unwrap_or(&row))
            .ok_or("missing donor candidate group")?;
        if col.len() != swapped.len()
            || col.len() != zeroed.len()
            || col.iter().zip(swapped).zip(zeroed).any(
                |(((p, wide), (s, swapped_wide)), (z, zero_wide))| {
                    !same(p, *wide, s, *swapped_wide)
                        || !same(p, *wide, z, *zero_wide)
                        || z.contextual != [0; 32]
                        || s.contextual != donor[0].0.contextual
                },
            )
        {
            return Err(format!("noncontext words changed in intervention for row {row}").into());
        }
    }
    let nominal = relative_calls(&dir.join("readback/score/nominal.results"), "3", &facts)?;
    let swap = relative_calls(&dir.join("readback/score/swap.results"), "3", &facts)?;
    let mut out = create(output)?;
    writeln!(out, "left\tright\tlocal_masks_and_words_equal\tleft_swap_equals_right_nominal\tright_swap_equals_left_nominal")?;
    for (a, b) in pairs {
        let row_a = reference.get(&a).ok_or("missing left group")?;
        let row_b = reference.get(&b).ok_or("missing right group")?;
        let fa = *facts.get(&a).ok_or("missing left fact")?;
        let fb = *facts.get(&b).ok_or("missing right fact")?;
        let matches = row_a.len() == row_b.len()
            && row_a.iter().zip(row_b).all(|((a, aw), (b, bw))| {
                a.span.start.checked_sub(fa) == b.span.start.checked_sub(fb)
                    && a.narrow == b.narrow
                    && a.local == b.local
                    && aw == bw
            });
        let a_replays = swap.get(&a).ok_or("missing left swap")?
            == nominal.get(&b).ok_or("missing right nominal")?;
        let b_replays = swap.get(&b).ok_or("missing right swap")?
            == nominal.get(&a).ok_or("missing left nominal")?;
        if matches && !(a_replays && b_replays) {
            return Err(format!("matched relative donor parity failed for {a}/{b}").into());
        }
        writeln!(out, "{a}\t{b}\t{matches}\t{a_replays}\t{b_replays}")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, grade, map, out] if mode == "prepare" =>
            prepare(Path::new(grade), Path::new(map), Path::new(out)),
        [mode, base, packets, trace, out] if mode == "trajectory" =>
            recurrence_oracle(Path::new(base), Path::new(packets), Path::new(trace), Path::new(out)),
        [mode, input, output] if mode == "zero" => zero(Path::new(input), Path::new(output)),
        [mode, input, bank, trace, results] if mode == "parity" =>
            score_oracle(Path::new(input), Path::new(bank), Path::new(trace), Path::new(results)),
        [mode, carrier, grade, input] if mode == "loaded" =>
            loaded_oracle(Path::new(carrier), Path::new(grade), Path::new(input)),
        [mode, grade, donors, dir, output] if mode == "compare" =>
            compare(Path::new(grade), Path::new(donors), Path::new(dir), Path::new(output)),
        _ => Err("usage: prepare GRADE DONORS OUTPUT_DIR | trajectory BASE PREFIXES GPU_TRACE OUTPUT_DIR | zero ROWS OUTPUT_ROWS | parity ROWS BANK GPU_TRACE GPU_RESULTS | loaded CARRIER GRADE NOMINAL_ROWS | compare GRADE DONORS DEVICE_DIR OUT_TSV".into()),
    }
}
