#![forbid(unsafe_code)]
mod factorial;

use all_starts_native::{packet::*, score::Bank as ReferenceBank};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use uor_r4_integer::{
    all_starts::{Bank, OnsetMode, OnsetProfile},
    discrete_d11::{ExactCopyState, ROLE_COPY_STOPS},
};

fn policy(text: &OsStr) -> Result<OnsetMode> {
    match text.to_str() {
        Some("ordered") => Ok(OnsetMode::Ordered),
        Some("neutral") => Ok(OnsetMode::Neutral),
        Some("position") => Ok(OnsetMode::Position),
        Some("ranked") => Ok(OnsetMode::Ranked),
        Some("veto") => Ok(OnsetMode::Veto),
        Some("both") => Ok(OnsetMode::Both),
        _ => Err("mode must be ordered, neutral, position, ranked, veto, or both".into()),
    }
}

fn fit_mode(text: &OsStr) -> Result<u32> {
    let mode = policy(text)?;
    if mode.fitted_mode() != mode {
        return Err("selected policy uses the frozen ordered bank".into());
    }
    Ok(mode as u32)
}

fn workdir(out: &Path) -> Result<PathBuf> {
    let mut path = out.as_os_str().to_os_string();
    path.push(".work");
    let path = PathBuf::from(path);
    fs::create_dir(&path)?;
    Ok(path)
}

fn call(device: &Path, args: &[&OsStr], log: &Path) -> Result<()> {
    let report = Command::new(device).args(args).output()?;
    save(log, &report.stderr)?;
    if !report.status.success() {
        return Err(format!(
            "device declined execution: {}",
            String::from_utf8_lossy(&report.stderr)
        )
        .into());
    }
    Ok(())
}

fn fit(device: &Path, input: &Path, labels: &Path, chosen: u32, output: &Path) -> Result<()> {
    let rows_bytes = fs::read(input)?;
    let rows = Rows::parse(&rows_bytes)?;
    let labels: Vec<Label> = serde_json::from_slice(&fs::read(labels)?)?;
    let label_bytes = fit_labels(&rows, &labels)?;
    let work = workdir(output)?;
    let bound_input = work.join("fit.packet");
    save(&bound_input, &rows_bytes)?;
    let label_file = work.join("fit.labels");
    let bank = work.join("fit.raw");
    save(&label_file, &label_bytes)?;
    let m = chosen.to_string();
    call(
        device,
        &[
            OsStr::new("fit"),
            bound_input.as_os_str(),
            label_file.as_os_str(),
            OsStr::new(&m),
            bank.as_os_str(),
        ],
        &work.join("device.log"),
    )?;
    let raw = fs::read(bank)?;
    if validate_raw(&raw)?[1] != chosen {
        return Err("device bank mode mismatch".into());
    }
    let sealed = seal(&rows, &rows_bytes, &label_bytes, &raw)?;
    if open(&rows, &sealed)? != raw {
        return Err("lossless bank round trip failed".into());
    }
    save(output, &sealed)?;
    println!(
        "fit mode={chosen} bytes={} sha256={} labels_sha256={}",
        sealed.len(),
        hex(digest(&sealed)),
        hex(digest(&label_bytes))
    );
    Ok(())
}

fn score(
    device: &Path,
    input: &Path,
    trained: &Path,
    policy: Option<OnsetMode>,
    output: &Path,
) -> Result<()> {
    let rows_bytes = fs::read(input)?;
    let rows = Rows::parse(&rows_bytes)?;
    let trained = fs::read(trained)?;
    let raw = open(&rows, &trained)?;
    let meta = validate_raw(raw)?;
    let reference = policy
        .map(|mode| load_policy(&rows, &trained, mode))
        .transpose()?;
    let work = workdir(output)?;
    let bound_input = work.join("score.packet");
    save(&bound_input, &rows_bytes)?;
    let bank = work.join("score.raw");
    let capture = work.join("score.trace");
    save(&bank, raw)?;
    let mut args = vec![
        OsStr::new("score"),
        bound_input.as_os_str(),
        bank.as_os_str(),
        capture.as_os_str(),
    ];
    let mode = policy.map_or(meta[1], |mode| mode as u32);
    let mode_code = mode.to_string();
    if policy.is_some() {
        args.push(OsStr::new(&mode_code));
    }
    call(device, &args, &work.join("device.log"))?;
    let trace = fs::read(capture)?;
    let words = trace_rows(&rows, mode, &trace)?;
    if let Some(reference) = reference {
        verify_replay(&rows, &words, &reference)?;
    }
    save(output, &trace)?;
    let receipt = serde_json::to_vec(&serde_json::json!({
        "bank_sha256":hex(digest(&trained)), "input_packet_sha256":hex(digest(&rows_bytes)),
        "blind_capture_sha256":hex(rows.provenance[3]), "trace_sha256":hex(digest(&trace)),
        "rows":rows.rows.len(), "mode":mode, "allocated_bytes":trained.len(), "used_records":meta[2],
    }))?;
    save(&work.join("bindings.json"), &receipt)?;
    Ok(())
}

fn load_policy(rows: &Rows, sealed: &[u8], mode: OnsetMode) -> Result<Bank> {
    open(rows, sealed)?;
    let digest_field = |start| {
        let hash: [u8; 32] = sealed[start..start + 32].try_into()?;
        Ok::<_, std::array::TryFromSliceError>(hex(hash))
    };
    let profile = OnsetProfile {
        mode,
        train_sha256: digest_field(112)?,
        labels_sha256: digest_field(144)?,
    };
    Ok(Bank::from_profile(
        sealed,
        rows.provenance[0],
        rows.provenance[1],
        rows.provenance[2],
        &profile,
    )?)
}

fn verify_replay(rows: &Rows, words: &[u32], reference: &Bank) -> Result<()> {
    for (row, raw) in rows.rows.iter().zip(words[4..].chunks_exact(OUT)) {
        let expected = reference.score_legacy_row(&row.0)?;
        if let Some((offset, (&want, &got))) = expected
            .iter()
            .zip(raw)
            .enumerate()
            .find(|(_, (want, got))| want != got)
        {
            return Err(format!(
                "native replay mode {} ordinal {} assertion {} word {offset}: Rust {want}, CUDA {got}",
                reference.mode().name(),
                row.0[0],
                row.0[1],
            ).into());
        }
    }
    Ok(())
}

fn decode_trace(
    input: &Path,
    bank: &Path,
    trace: &Path,
    policy: Option<OnsetMode>,
    output: &Path,
) -> Result<()> {
    let packet = fs::read(input)?;
    let rows = Rows::parse(&packet)?;
    let sealed = fs::read(bank)?;
    let raw = fs::read(trace)?;
    let mode = match policy {
        Some(mode) => mode,
        None => OnsetMode::from_code(ReferenceBank::load(&rows, &sealed)?.mode())
            .ok_or("bad fitted mode")?,
    };
    let trace = trace_rows(&rows, mode as u32, &raw)?;
    if policy.is_some() {
        verify_replay(&rows, &trace, &load_policy(&rows, &sealed, mode)?)?;
    }
    let named_status = ["selected", "no_candidate", "unseen", "tie", "nonpositive"];
    let named_slot = [
        "absent",
        "cap_rejected",
        "unseen",
        "scored",
        "outside_radius",
    ];
    let maybe = |word| (word != NIL).then_some(word);
    let ratio = |num: u32, den: u32| {
        (den != NIL).then(|| {
            serde_json::json!({
                "numerator":num as i32,"denominator":den
            })
        })
    };
    let mut output_rows = Vec::with_capacity(rows.rows.len());
    for (data, row) in trace[4..].chunks_exact(OUT).zip(&rows.rows) {
        let mut candidates = Vec::new();
        for i in 0..3 {
            let slot = &data[8 + 20 * i..28 + 20 * i];
            if slot[0] == NIL {
                continue;
            }
            let descriptor = slot[3] != NIL;
            let geometry = if descriptor {
                let n = (slot[3] & 255) + (slot[3] >> 8);
                if n > 10 {
                    return Err("trace has invalid used geometry width".into());
                }
                Some(&slot[10..10 + n as usize])
            } else {
                None
            };
            candidates.push(serde_json::json!({
                "source":slot[0], "first_word_end":slot[1], "status":named_slot[slot[2] as usize],
                "connector_depth":i+1,"shape_word":maybe(slot[3]),"signature":maybe(slot[4]),
                "positive_evidence":maybe(slot[5]),"negative_evidence":maybe(slot[6]),
                "radius":(slot[7]!=NIL && slot[7]!=NO_RADIUS).then_some(slot[7]),
                "margin":ratio(slot[8],slot[9]),"packed_roots":geometry,
            }));
        }
        output_rows.push(serde_json::json!({
            "ordinal":data[0],"assertion_index":maybe(data[1]),"status":named_status[data[2] as usize],
            "source":maybe(data[3]),"best_margin":ratio(data[4],data[5]),"runner_up":ratio(data[6],data[7]),
            "original_tape_len":row.0[2],"fact_end_inclusive":(row.0[3]>0).then_some(row.0[3]),
            "query_start":(row.0[4]>0).then_some(row.0[4]),"fence":maybe(row.0[5]),
            "eligible_mask":row.0[6],"word_start_mask":row.0[7],"candidates":candidates
        }));
    }
    let report = serde_json::to_vec(&serde_json::json!({
        "schema":"uor-r4.all-starts-original-sources/1",
        "mode":mode.name(),
        "positive_negative_evidence":if mode==OnsetMode::Position {"votes"}else{"L1 distances"},
        "bank_sha256":hex(digest(&sealed)), "input_packet_sha256":hex(digest(&packet)),
        "blind_capture_sha256":hex(rows.provenance[3]), "trace_sha256":hex(digest(&raw)),
        "rows":output_rows,
    }))?;
    save(output, &report)
}

fn trace_rows(rows: &Rows, mode: u32, trace: &[u8]) -> Result<Vec<u32>> {
    let words = decode(trace)?;
    if words.len() != 4 + rows.rows.len() * OUT
        || words[..4] != [TRACE, rows.rows.len() as u32, OUT as u32, mode]
    {
        return Err("trace framing mismatch".into());
    }
    for (raw, row) in words[4..].chunks_exact(OUT).zip(&rows.rows) {
        if raw[..2] != row.0[..2]
            || raw[2] >= 5
            || (raw[2] != 0 && raw[3] != NIL)
            || (raw[2] == 0
                && !(0..3).any(|i| row.candidate(i)[0] == raw[3] && row.candidate(i)[2] == 1))
        {
            return Err(format!("invalid native trace row [{},{}]", raw[0], raw[1]).into());
        }
        for i in 0..3 {
            let source = row.candidate(i);
            let r = &raw[8 + 20 * i..28 + 20 * i];
            if r[0] != source[0]
                || (r[0] != NIL
                    && (r[1] != source[1]
                        || (source[2] == 0 && r[2] != 1)
                        || (source[2] == 1
                            && !([2, 3].contains(&r[2]) || matches!(mode, 4 | 5) && r[2] == 4))))
            {
                return Err("raw trace changed original address or eligibility".into());
            }
        }
    }
    Ok(words)
}

fn geometry(depth: usize) -> Vec<u32> {
    if depth == 3 {
        vec![1, 1, 1, 1, 2]
    } else {
        vec![1; depth + 1]
    }
}
// These are coherent original-position geometry tapes. All descriptors
// that overlap a slot see the same class and packed roots at that slot.
const ANCHORS: [[u16; 5]; 3] = [[0, 0, 0x0211, 0, 0], [0, 1, 0, 0, 0], [0, 3, 0x0211, 0, 0]];
fn row(ordinal: u32, fact: u32, original: [u16; 5]) -> Result<Row> {
    let mut out = Row::new(ordinal, 0, 192, fact, fact + 10, fact + 7, [7, 7]);
    for i in 0..3 {
        let cls = geometry(i + 1);
        let source = fact + 2 + i as u32;
        let end = source + if i == 2 { 2 } else { 1 };
        out.insert(i, source, end, true, &cls, &original[..cls.len()])?;
    }
    Ok(out)
}

fn synthetic_hashes() -> [[u8; 32]; 4] {
    [
        b"finite-model".as_slice(),
        b"finite-tokenizer".as_slice(),
        b"finite-sites".as_slice(),
        b"finite-packed-tape".as_slice(),
    ]
    .map(digest)
}
fn fixture_set(out: &Path, name: &str, missing: Option<usize>, permute: bool) -> Result<()> {
    let mut rows = Vec::new();
    let mut labels = Vec::new();
    for (role, anchor) in ANCHORS.into_iter().enumerate() {
        for family_variant in 0..2 {
            if missing == Some(role) && family_variant == 1 {
                continue;
            }
            let at = rows.len() as u32;
            let mut original = anchor;
            if family_variant == 1 {
                original[0] = 0x1111;
            }
            rows.push(row(at, 2, original)?);
            let target = if permute && role < 2 { 1 - role } else { role };
            labels.push(
                serde_json::json!({"ordinal":at,"assertion_index":0,"family":at,"source":4+target}),
            );
        }
    }
    let packet = Rows {
        provenance: synthetic_hashes(),
        rows,
    };
    save(&out.join(format!("{name}.packet")), &packet.encode()?)?;
    save(
        &out.join(format!("{name}.labels.json")),
        &serde_json::to_vec(&labels)?,
    )?;
    Ok(())
}
fn probe_rows(out: &Path, name: &str, original: [u16; 5]) -> Result<()> {
    let mut rows = Vec::new();
    for i in 0..1024 {
        rows.push(row(i, if i == 0 { 2 } else { 174 }, original)?);
    }
    rows.push(Row::new(1024, NIL, 192, 0, 0, NIL, [0, 0]));
    let probes = Rows {
        provenance: synthetic_hashes(),
        rows,
    };
    save(&out.join(name), &probes.encode()?)
}
fn make_fixtures(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    fixture_set(out, "right", None, false)?;
    fixture_set(out, "wrong", None, true)?;
    fixture_set(out, "unseen", Some(1), false)?;
    probe_rows(out, "infer.packet", ANCHORS[1])?;
    probe_rows(out, "tie-infer.packet", [0; 5])?;
    let mut overflow = Vec::new();
    let mut labels = Vec::new();
    for i in 0..86u32 {
        overflow.push(row(i, 2, ANCHORS[0])?);
        labels.push(serde_json::json!({"ordinal":i,"assertion_index":0,"family":i,"source":4}));
    }
    let packet = Rows {
        provenance: synthetic_hashes(),
        rows: overflow,
    };
    save(&out.join("overflow.packet"), &packet.encode()?)?;
    save(
        &out.join("overflow.labels.json"),
        &serde_json::to_vec(&labels)?,
    )?;
    Ok(())
}

fn finite_writer(source: u32) -> Result<Vec<u32>> {
    let ids = [0, 55, 42, 10, 11, 13, 15, 14, 45, 16, 46, 47, 42, 201];
    let mut state = ExactCopyState::new(Some(source), ids.len() as u32);
    let mut out = Vec::new();
    loop {
        let step = state
            .write(&ids, NIL, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        out.push(step.emitted);
        if step.release != 0 {
            if !step.completed_span() {
                return Err("synthetic source failed writer".into());
            }
            return Ok(out);
        }
        state = step.state;
    }
}

fn verify_finite(name: &str, input: &Path, raw: &Path, bank: &Path) -> Result<()> {
    let rows = Rows::parse(&fs::read(input)?)?;
    let trace = fs::read(raw)?;
    let header = decode(&trace[..trace.len().min(16)])?;
    let mode = *header.get(3).ok_or("short finite trace")?;
    let words = trace_rows(&rows, mode, &trace)?;
    let reference = ReferenceBank::load(&rows, &fs::read(bank)?)?;
    if reference.mode() != mode {
        return Err("Rust and CUDA bank modes disagree".into());
    }
    for (i, (row, raw)) in rows
        .rows
        .iter()
        .zip(words[4..].chunks_exact(OUT))
        .enumerate()
    {
        let expected = reference.select(row)?;
        if let Some((offset, (&want, &got))) = expected
            .iter()
            .zip(raw)
            .enumerate()
            .find(|(_, (want, got))| want != got)
        {
            return Err(format!(
                "independent parity {name} row {i} word {offset}: Rust {want}, CUDA {got}"
            )
            .into());
        }
    }

    let expected = match (name, mode) {
        ("right", 0 | 1) => (0, 5),
        ("wrong", 0) => (0, 4),
        ("tie", 0) => (3, NIL),
        ("unseen", 0) => (2, NIL),
        ("right", 2) => (4, NIL),
        _ => return Err("unknown finite result".into()),
    };
    for (i, row) in words[4..].chunks_exact(OUT).enumerate() {
        let offset = if i != 0 && expected.1 != NIL {
            172 + expected.1
        } else {
            expected.1
        };
        let goal = if row[1] == NIL {
            (1, NIL)
        } else {
            (expected.0, offset)
        };
        if (row[2], row[3]) != goal {
            return Err(format!("finite {name} row {i} status/source {:?}", &row[2..4]).into());
        }
    }
    let first = &words[4..4 + OUT];
    match (name, mode) {
        ("right", 0 | 1) => {
            assert_eq!(&first[8 + 20 + 5..8 + 20 + 8], &[0, 28, 32]); // Actual TRAIN-only radius.
            assert_eq!(finite_writer(first[3])?, [13, 15, 14, 45, 1]);
        }
        ("right", 2) => {
            assert_eq!(&first[8 + 20 + 5..8 + 20 + 8], &[2, 4, NO_RADIUS]);
        }
        ("wrong", 0) => assert_eq!(finite_writer(first[3])?, [11, 13, 15, 14, 45, 1]),
        ("tie", 0) => assert_eq!(&first[4..8], &[8, 2, 12, 3]),
        ("unseen", 0) => assert_eq!(first[8 + 20 + 7], NO_RADIUS),
        _ => (),
    }
    println!(
        "{name}: mode={mode} rows={} first_status={} first_source={}",
        rows.rows.len(),
        first[2],
        first[3]
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [op, input] if op=="site-key" => {
            let capture=serde_json::from_slice(&fs::read(input)?)?;
            println!("{}",hex(site_key(&capture)?));
        },
        [op, input, model, tokenizer, sites, out] if op=="pack-json" => {
            let bytes=fs::read(input)?;
            let parse=|value:&OsStr|hex_digest(value.to_str().ok_or("non-UTF8 hash")?);
            let rows=pack_json(&bytes,parse(model)?,parse(tokenizer)?,parse(sites)?)?;
            save(Path::new(out),&rows.encode()?)?;
            println!("packed {} assertion entries",rows.rows.len());
        },
        [op, device, input, labels, m, out] if op=="fit" => fit(Path::new(device),Path::new(input),Path::new(labels),fit_mode(m)?,Path::new(out))?,
        [op, device, input, bank, out] if op=="score" => score(Path::new(device),Path::new(input),Path::new(bank),None,Path::new(out))?,
        [op, device, input, bank, m, out] if op=="score-as" => score(Path::new(device),Path::new(input),Path::new(bank),Some(policy(m)?),Path::new(out))?,
        [op, out] if op=="fixtures" => make_fixtures(Path::new(out))?,
        [op, device, out] if op=="factorial-finite" => factorial::run(Path::new(device),Path::new(out))?,
        [op, input, bank, trace, out] if op=="decode-trace" => decode_trace(Path::new(input),Path::new(bank),Path::new(trace),None,Path::new(out))?,
        [op, input, bank, m, trace, out] if op=="decode-trace-as" => decode_trace(Path::new(input),Path::new(bank),Path::new(trace),Some(policy(m)?),Path::new(out))?,
        [op, name, input, trace, bank] if op=="verify-finite" => verify_finite(name.to_str().ok_or("bad fixture name")?,Path::new(input),Path::new(trace),Path::new(bank))?,
        _=>return Err("use site-key INPUT | pack-json INPUT MODEL_SHA TOKENIZER_SHA SITE_SHA OUT | fit DEVICE INPUT TRAIN_LABELS MODE OUT | score DEVICE INPUT BANK OUT | score-as DEVICE INPUT BANK MODE OUT | decode-trace INPUT BANK TRACE OUT_JSON | decode-trace-as INPUT BANK MODE TRACE OUT_JSON | fixtures OUTDIR | factorial-finite DEVICE OUTDIR | verify-finite NAME INPUT TRACE BANK".into()),
    }
    Ok(())
}
