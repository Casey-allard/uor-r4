use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{
    all_starts::{Bank, OnsetMode, OnsetProfile, BANK_BYTES, CANDIDATE_WORDS, DECISION_WORDS},
    discrete_d11::{pack_all_starts_artifact, spiralcore_orbit_roots, IntegerDiscreteModel},
};

const NIL: u32 = u32::MAX;
type Row = [u32; CANDIDATE_WORDS];
type Trace = [u32; DECISION_WORDS];
type Record = (usize, u32, Vec<u16>);

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn words(out: &mut Vec<u8>, source: impl IntoIterator<Item = u32>) {
    out.extend(source.into_iter().flat_map(u32::to_le_bytes));
}
fn distance(a: u16, b: u16) -> u32 {
    let orbit = spiralcore_orbit_roots();
    (0..4)
        .map(|i| {
            let x = orbit[((a >> (4 * i)) & 15) as usize];
            let y = orbit[((b >> (4 * i)) & 15) as usize];
            x.into_iter()
                .zip(y)
                .map(|(x, y)| u32::from(x.abs_diff(y)))
                .sum::<u32>()
        })
        .sum()
}
fn row(ordinal: u32, length: u32, query: u32, roots: &[u16], classes: &[u8]) -> Row {
    assert!(roots.len() == classes.len() && (3..=4).contains(&roots.len()));
    let mut out = [0; CANDIDATE_WORDS];
    out[..8].copy_from_slice(&[ordinal, 0, length, 6, query, 7 + roots.len() as u32, 3, 3]);
    out[26] = NIL;
    for (i, used) in [2, roots.len()].into_iter().enumerate() {
        let r = &mut out[8 + 9 * i..17 + 9 * i];
        r[..3].copy_from_slice(&[8 + i as u32, 7 + used as u32, 1]);
        for j in 0..used {
            r[3] |= u32::from(classes[j]) << (2 * j);
            r[4 + j / 2] |= u32::from(roots[j]) << (16 * (j % 2));
        }
    }
    out
}
fn plant(roots: &[u16], classes: &[u8]) -> Vec<Record> {
    assert!(roots[0] != roots[1] && classes[..3] == [1, 1, 1]);
    let early_alt = (0..=u16::MAX)
        .filter(|&r| r != roots[0])
        .min_by_key(|&r| distance(r, roots[0]))
        .unwrap();
    let early_margin = distance(early_alt, roots[0]);
    let late_order_cost = 2 * distance(roots[0], roots[1]);
    let last = roots.len() - 1;
    let late_alt = (0..=u16::MAX)
        .find(|&r| {
            let late_margin = 2 * distance(r, roots[last]);
            let early_bound = roots.len() as u32 * early_margin;
            late_margin > early_bound && late_margin < early_bound + 2 * late_order_cost
        })
        .unwrap();
    let early = roots[..2].to_vec();
    let mut later_pos = roots.to_vec();
    later_pos.swap(0, 1);
    let mut later_neg = roots.to_vec();
    later_neg[last] = late_alt;
    vec![
        (1, 0, early),
        (1, 1, vec![early_alt, roots[1]]),
        (2, 0, later_pos),
        (2, 1, later_neg),
    ]
}
fn pack(
    records: &[Record],
    classes: &[u8],
    bindings: [[u8; 32]; 3],
    mode: OnsetMode,
) -> (Vec<u8>, OnsetProfile) {
    let train = digest(b"finite analytic geometry only, never TRAIN");
    let labels = digest(b"analytic positive-negative prototypes, no collected labels");
    let profile = OnsetProfile {
        mode,
        train_sha256: hex::encode(train),
        labels_sha256: hex::encode(labels),
    };
    let mut raw = Vec::new();
    words(
        &mut raw,
        [0x31425355, mode as u32, records.len() as u32, 256],
    );
    for (depth, label, roots) in records {
        let mut geometry = [0u16; 10];
        geometry[..roots.len()].copy_from_slice(roots);
        if mode == OnsetMode::Neutral {
            for segment in [0..*depth, *depth..roots.len()] {
                for class in [1, 2] {
                    let positions: Vec<_> =
                        segment.clone().filter(|&p| classes[p] == class).collect();
                    let mut values: Vec<_> = positions.iter().map(|&p| geometry[p]).collect();
                    values.sort_unstable();
                    for (i, v) in positions.into_iter().zip(values) {
                        geometry[i] = v;
                    }
                }
            }
        }
        let sig = classes[..roots.len()]
            .iter()
            .enumerate()
            .map(|(i, c)| u32::from(*c) << (2 * i))
            .sum();
        words(
            &mut raw,
            [
                *depth as u32 | ((roots.len() - depth) as u32) << 8 | label << 16,
                sig,
                1280,
            ],
        );
        words(
            &mut raw,
            geometry
                .chunks_exact(2)
                .map(|pair| u32::from(pair[0]) | (u32::from(pair[1]) << 16)),
        );
    }
    raw.resize(BANK_BYTES - 208, 0);
    let mut bytes = Vec::new();
    words(&mut bytes, [0x32425355, 6, raw.len() as u32, 0]);
    for hash in bindings.into_iter().chain([train, labels, digest(&raw)]) {
        bytes.extend(hash);
    }
    bytes.extend(raw);
    (bytes, profile)
}
fn summary(trace: &Trace) -> Value {
    let candidates: Vec<_> = [8, 28].into_iter().map(|c| {
        json!({"address":trace[c],"first_word_end":trace[c+1],"status":trace[c+2],
            "shape":trace[c+3],"signature":trace[c+4],"positive_distance":trace[c+5],
            "negative_distance":trace[c+6],"margin":[trace[c+8] as i32,trace[c+9]],
            "geometry":trace[c+10..c+20].iter().copied().take_while(|&x| x!=NIL).collect::<Vec<_>>()})
    }).collect();
    json!({"ordinal":trace[0],"assertion":trace[1],"status":trace[2],
        "address":(trace[3]!=NIL).then_some(trace[3]), "candidates":candidates,
        "trace":trace.as_slice()})
}
fn load(bytes: &[u8], bindings: [[u8; 32]; 3], profile: &OnsetProfile) -> Bank {
    Bank::from_profile(bytes, bindings[0], bindings[1], bindings[2], profile).unwrap()
}
fn assert_selected(trace: &Trace, address: u32) {
    assert_eq!(&trace[2..4], &[0, address]);
}

fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = env::args_os().skip(1).collect();
    let [carrier, span, old_json, out] = a.as_slice() else {
        return Err("expected CARRIER SPAN PRIOR_FINITE_JSON OUT".into());
    };
    let out = Path::new(out);
    let modes = [OnsetMode::Ordered, OnsetMode::Neutral];
    let synthetic_classes = [1, 1, 1, 2];
    let original = [0x0000, 0xffff, 0x1234, 0x5a5a];
    let mut within_class = original;
    within_class.swap(0, 1);
    let mut across_class = original;
    across_class.swap(0, 3);
    let synthetic_bindings = [[21; 32], [22; 32], [23; 32]];
    let synthetic_records = plant(&original, &synthetic_classes);
    let mut synthetic_report = Vec::new();
    for mode in modes {
        let (bytes, profile) = pack(
            &synthetic_records,
            &synthetic_classes,
            synthetic_bindings,
            mode,
        );
        let bank = load(&bytes, synthetic_bindings, &profile);
        let rows = [original, within_class, across_class]
            .map(|roots| row(0, 18, 15, &roots, &synthetic_classes));
        let traces = rows.map(|r| bank.score_legacy_row(&r).unwrap());
        assert_selected(&traces[0], if mode == OnsetMode::Ordered { 8 } else { 9 });
        assert_selected(&traces[1], 9);
        if mode == OnsetMode::Neutral {
            assert_eq!(&traces[0][28..48], &traces[1][28..48]);
            assert_ne!(&traces[0][38..48], &traces[2][38..48]);
            assert_eq!(traces[0][33], 0);
            assert!(traces[2][33] > 0);
        }
        synthetic_report.push(json!({"mode":mode,"bank_sha256":hex::encode(digest(&bytes)),
            "row_words":rows.iter().map(|r| r.as_slice()).collect::<Vec<_>>(),
            "original":summary(&traces[0]),"within_class":summary(&traces[1]),"across_class":summary(&traces[2])}));
    }

    let carrier = fs::read(carrier)?;
    let model = IntegerDiscreteModel::from_bytes(&carrier)?;
    let span_bytes = fs::read(span)?;
    let span: Value = serde_json::from_slice(&span_bytes)?;
    let prior_public: Value = serde_json::from_slice(&fs::read(old_json)?)?;
    let ids: Vec<u32> = serde_json::from_value(prior_public[0]["public"]["prompt_ids"].clone())?;
    let classes: Vec<u8> = serde_json::from_value(span["bank"]["classes"].clone())?;
    let role = model.paired_roles_diagnostic(&ids, 0..ids.len(), false)?;
    let (_, witnesses) = model.paired_copy_witness(&ids, role, None)?;
    assert_eq!(witnesses.len(), 1);
    let w = &witnesses[0];
    assert_eq!((w.fact_span[1] - 1, w.assertion.fence), (6, Some(10)));
    assert_eq!(
        (
            w.assertion.value.eligible_mask,
            w.assertion.value.word_start_mask
        ),
        (3, 3)
    );
    let root_table = model.packed_codebook_roots();
    let roots: Vec<_> = ids[7..10]
        .iter()
        .map(|&id| root_table[id as usize] as u16)
        .collect();
    let slots: Vec<_> = ids[7..10].iter().map(|&id| classes[id as usize]).collect();
    let mut sites = Sha256::new();
    sites.update(b"uor-r4/all-starts-sites/1");
    sites.update(digest(&carrier));
    sites.update(digest(
        &root_table
            .iter()
            .flat_map(|&r| (r as u16).to_le_bytes())
            .collect::<Vec<_>>(),
    ));
    sites.update(digest(&classes));
    sites.update(digest(&span_bytes));
    for anchor in spiralcore_orbit_roots() {
        sites.update(anchor.map(|x| x as u8));
    }
    let bind = [
        hex::decode(span["artifact_sha256"].as_str().unwrap())?
            .try_into()
            .unwrap(),
        hex::decode(model.tokenizer_sha256().unwrap())?
            .try_into()
            .unwrap(),
        sites.finalize().into(),
    ];
    let plain_row = row(
        0,
        ids.len() as u32,
        role.requested.unwrap().start as u32,
        &roots,
        &slots,
    );
    let records = plant(&roots, &slots);
    let mut public_report = Vec::new();
    for (name, mode) in ["ordered", "neutral"].into_iter().zip(modes) {
        let (bytes, profile) = pack(&records, &slots, bind, mode);
        let trace = load(&bytes, bind, &profile).score_legacy_row(&plain_row)?;
        let address = if mode == OnsetMode::Ordered { 8 } else { 9 };
        assert_selected(&trace, address);
        let output = pack_all_starts_artifact(&carrier, &span_bytes, &bytes, &profile)?;
        let loaded = IntegerDiscreteModel::from_bytes(&output)?;
        let (decision, writer) = loaded.paired_copy_witness(&ids, role, None)?;
        assert_eq!(decision.choice.source, Some(address));
        assert_eq!(writer[0].onset.unwrap().source, Some(address));
        fs::write(out.join(format!("{name}.d11")), &output)?;
        public_report.push(json!({"mode":mode,"bank_bytes":bytes.len(),"artifact_bytes":output.len(),
            "bank_sha256":hex::encode(digest(&bytes)),"artifact_sha256":hex::encode(digest(&output)),
            "source_rows":summary(&trace)}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"synthetic":synthetic_report,"public_packet":plain_row.as_slice(),"public":public_report})
        )?
    );
    Ok(())
}
