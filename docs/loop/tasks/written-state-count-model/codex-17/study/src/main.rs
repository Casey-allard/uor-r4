#![forbid(unsafe_code)]
use serde_json::{json, Value as V};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
};
use uor_r4_integer::{
    discrete_d11::{ExactCopyState, ROLE_COPY_STOPS},
    paired_role::{ROLE_CONTEXT, ROLE_VOCAB},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn require(ok: bool, msg: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(msg.into())
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hash(file: &str) -> Result<String> {
    Ok(sha(&fs::read(file)?))
}
fn read(file: &str) -> Result<V> {
    Ok(serde_json::from_slice(&fs::read(file)?)?)
}
fn write(file: &str, data: &V) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)?;
    f.write_all(&serde_json::to_vec(data)?)?;
    f.sync_all()?;
    Ok(())
}
fn arr(v: &V) -> Result<&[V]> {
    v.as_array()
        .map(Vec::as_slice)
        .ok_or("expected array".into())
}
fn num(v: &V) -> Result<u32> {
    v.as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("invalid unsigned word".into())
}
fn text(v: &V) -> Result<&str> {
    v.as_str()
        .filter(|s| !s.is_empty())
        .ok_or("invalid string".into())
}
fn opt(v: &V) -> Result<Option<u32>> {
    if v.is_null() {
        Ok(None)
    } else {
        Ok(Some(num(v)?))
    }
}
fn ids(v: &V) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn row_idx(row: &V) -> Result<usize> {
    Ok(num(&row["ordinal"])? as usize)
}
fn id_hash(input: &[u32]) -> String {
    let mut h = Sha256::new();
    for id in input {
        h.update(id.to_le_bytes());
    }
    format!("{:x}", h.finalize())
}
fn caprows(c: &V) -> Result<&[V]> {
    require(
        c["schema"] == "uor-r4.legacy-assertion-candidates/1",
        "wrong capture schema",
    )?;
    let r = arr(&c["rows"])?;
    for (i, x) in r.iter().enumerate() {
        require(row_idx(x)? == i, "capture ordinal changed")?;
    }
    Ok(r)
}
fn grade_binding(g: &V, row: &V) -> Result<()> {
    require(
        g["prefix_sha256"] == row["prefix_sha256"] && g["requested"] == row["requested_span"],
        "scoring key and requested capture differ",
    )
}
fn gold_fact(g: &V) -> Result<&V> {
    arr(&g["facts"])?
        .get(num(&g["gold_assertion_index"])? as usize)
        .ok_or("gold assertion missing".into())
}
fn verify_input_rows(blind: &str, capture: &str, grades: &str, supplement: &str) -> Result<()> {
    let blind_bytes = fs::read(blind)?;
    let blind: V = serde_json::from_slice(&blind_bytes)?;
    let capture = read(capture)?;
    let base = read(grades)?;
    let extra = if supplement == "-" {
        json!([])
    } else {
        read(supplement)?
    };
    let originals = arr(&blind)?;
    let rows = caprows(&capture)?;
    let base = arr(&base)?;
    let extra = arr(&extra)?;
    require(
        capture["arm"] == "requested-diagnostic"
            && capture["blind_sha256"] == sha(&blind_bytes)
            && base.len() == 42
            && extra.len() == (if supplement == "-" { 0 } else { 52 })
            && originals.len() == rows.len()
            && rows.len() == base.len() + extra.len(),
        "wrong pre-rank custody cohort or original blind checksum",
    )?;
    let mut seen = BTreeSet::new();
    let mut controls = BTreeMap::new();
    for (i, ((original, row), g)) in originals
        .iter()
        .zip(rows)
        .zip(base.iter().chain(extra))
        .enumerate()
    {
        let mut verify = || -> Result<()> {
            let tape = ids(&original["prefix"])?;
            let key = id_hash(&tape);
            require(
                tape.first() == Some(&0)
                    && tape.len() <= ROLE_CONTEXT
                    && tape.iter().all(|&id| id < ROLE_VOCAB as u32)
                    && num(&row["original_tape_len"])? as usize == tape.len()
                    && row["prefix_sha256"] == key,
                "wrong original u32le tape binding",
            )?;
            grade_binding(g, row)?;
            let id = text(&g["id"])?;
            require(!id.trim().is_empty(), "blank grade ID")?;
            match text(&g["stratum"])? {
                "supported" => {
                    let gold = num(&g["gold_start"])? as usize;
                    let expected = ids(&g["expected"])?;
                    let (literal, terminal) = expected.split_at(expected.len().saturating_sub(1));
                    let _ = gold_fact(g)?;
                    require(
                        gold > 0
                            && gold < tape.len()
                            && !literal.is_empty()
                            && terminal == [1]
                            && literal.iter().all(|&id| id > 2 && id < ROLE_VOCAB as u32),
                        "malformed supervised source or expected EOS sequence",
                    )?;
                }
                stratum @ ("conflict" | "unmentioned" | "unspecified") => {
                    require(
                        i < base.len()
                            && g.get("gold_start") == Some(&V::Null)
                            && g.get("gold_assertion_index") == Some(&V::Null)
                            && g.get("expected") == Some(&V::Null),
                        "control outside base roster or carrying a supervised value",
                    )?;
                    *controls.entry(stratum.to_owned()).or_insert(0) += 1;
                }
                _ => return Err("unrecognized v5 stratum".into()),
            }
            Ok(())
        };
        verify().map_err(|e| format!("input ordinal {i}: {e}"))?;
        require(
            seen.insert(text(&g["id"])?),
            "duplicate grade ID across input split",
        )?;
    }
    require(
        controls
            == BTreeMap::from([
                ("conflict".to_owned(), 2),
                ("unmentioned".to_owned(), 2),
                ("unspecified".to_owned(), 2),
            ]),
        "v5 controls must contain two rows in each of conflict, unmentioned and unspecified",
    )
}
fn exact_assertion<'a>(row: &'a V, g: &V) -> Result<(&'a V, usize)> {
    let fact = gold_fact(g)?;
    let w = arr(&row["assertions"])?;
    require(
        w.len() == 1
            && w[0]["fact_span"] == fact["name_span"]
            && w[0]["fence"] == fact["delimiter"][0],
        "requested assertion differs from exact original annotated span and fence",
    )?;
    Ok((&w[0], 0))
}
fn reachable(a: &V) -> Result<Vec<u32>> {
    arr(&a["legacy_candidates"])?
        .iter()
        .filter(|c| c["writer_reachable"] == true)
        .map(|c| num(&c["original_start"]))
        .collect()
}
fn fit_labels(c: &str, b: &str, x: &str, out: &str) -> Result<()> {
    let capture = read(c)?;
    let base = read(b)?;
    let extra = read(x)?;
    let grades = arr(&base)?.iter().chain(arr(&extra)?).collect::<Vec<_>>();
    let rows = caprows(&capture)?;
    require(
        rows.len() == 94 && grades.len() == 94 && capture["arm"] == "requested-diagnostic",
        "wrong combined FIT shape",
    )?;
    let families = grades
        .iter()
        .filter(|g| g["stratum"] == "supported")
        .map(|g| text(&g["family"]))
        .collect::<Result<BTreeSet<_>>>()?
        .into_iter()
        .enumerate()
        .map(|(i, f)| (f, (i + 1) as u32))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, (row, g)) in rows.iter().zip(grades).enumerate() {
        grade_binding(g, row)?;
        require(seen.insert(text(&g["id"])?), "duplicate FIT case ID")?;
        if g["stratum"] != "supported" {
            require(
                matches!(
                    g["stratum"].as_str(),
                    Some("conflict" | "unmentioned" | "unspecified")
                ) && g["gold_start"].is_null()
                    && g["expected"].is_null(),
                "negative FIT row carries supervised target",
            )?;
            continue;
        }
        let (site, j) = exact_assertion(row, g)?;
        let gold = num(&g["gold_start"])?;
        require(
            reachable(site)?.contains(&gold) && g["expected"].as_array().is_some(),
            "FIT gold not reachable by actual frozen chain",
        )?;
        output.push(json!({"ordinal":i,"assertion_index":j,"family":families[text(&g["family"])?],"source":gold}));
    }
    write(out, &json!(output))
}
fn completed(tape: &[u32], start: u32, fence: u32) -> Result<Vec<u32>> {
    require(
        start < fence && fence < (tape.len() as u32),
        "source/fence out of range",
    )?;
    let mut cur = ExactCopyState::new(Some(start), u32::try_from(tape.len())?);
    let mut emit = Vec::new();
    // Known source must finish by the original delimiter, including ordinary EOS.
    loop {
        let s = cur
            .write(tape, u32::MAX, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        emit.push(s.emitted);
        if s.release != 0 {
            require(
                s.completed_span() && s.state.cursor == fence && s.emitted == 1,
                "source does not reach same original delimiter through cursor",
            )?;
            return Ok(emit);
        }
        require(
            emit.len() < 8 && s.copied == 1 && s.emitted != 1,
            "copy cap or early EOS",
        )?;
        cur = s.state;
    }
}
fn consensus(tape: &[u32], assertions: &[V], proposed: &[Option<u32>]) -> Result<(V, V)> {
    require(
        assertions.len() == proposed.len(),
        "source/claim cardinality mismatch",
    )?;
    let mut first: Option<(u32, u32, Vec<u32>)> = None;
    let mut complete = 0;
    let mut disagree = false;
    let mut records = Vec::new();
    for (a, source) in assertions.iter().zip(proposed) {
        match source {
            None => records.push(V::Null),
            Some(s) => {
                require(
                    reachable(a)?.contains(s),
                    "new source outside reachable legacy capture",
                )?;
                let sequence = completed(tape, *s, num(&a["fence"])?)?;
                let fact = num(&a["fact_span"][0])?;
                complete += 1;
                if let Some((_, _, original)) = &first {
                    disagree |= *original != sequence;
                } else {
                    first = Some((fact, *s, sequence.clone()));
                }
                records.push(json!({"source":s,"ids_through_eos":sequence}));
            }
        }
    }
    let agreed = if complete == assertions.len() && !disagree {
        first
    } else {
        None
    };
    Ok((
        json!({"supporting_facts":assertions.len(),"complete_values":complete,
       "values_disagree":disagree,"fact":agreed.as_ref().map(|x|x.0),"source":agreed.as_ref().map(|x|x.1)}),
        json!(records),
    ))
}
fn trace_map(trace: &V) -> Result<BTreeMap<(u32, Option<u32>), &V>> {
    let mut result = BTreeMap::new();
    for row in arr(&trace["rows"])? {
        let k = (num(&row["ordinal"])?, opt(&row["assertion_index"])?);
        require(
            result.insert(k, row).is_none(),
            "duplicate decoded original assertion key",
        )?;
    }
    Ok(result)
}
fn rewrite(kind: &str, c: &str, b: &str, s: &str, t: &str, out: &str, audit: &str) -> Result<()> {
    let capture = read(c)?;
    let blind = read(b)?;
    let mut select = read(s)?;
    let cap = caprows(&capture)?;
    let blind = arr(&blind)?;
    require(
        capture["blind_sha256"] == hash(b)?
            && capture["selection_sha256"] == hash(s)?
            && select["blind"] == capture["blind_sha256"]
            && arr(&select["rows"])?.len() == cap.len()
            && blind.len() == cap.len(),
        "capture does not bind blind selection",
    )?;
    let score = if kind == "word-start" {
        require(t == "-", "WordStart received native trace")?;
        None
    } else {
        Some(read(t)?)
    };
    if let Some(v) = &score {
        require(
            v["schema"] == "uor-r4.all-starts-original-sources/1"
                && v["blind_capture_sha256"] == hash(c)?
                && v["mode"] == kind,
            "score does not bind original capture or mode",
        )?;
    }
    let mut by_key = match &score {
        Some(t) => trace_map(t)?,
        None => BTreeMap::new(),
    };
    let mut audits = Vec::new();
    for (i, (row, selected)) in cap
        .iter()
        .zip(
            select["rows"]
                .as_array_mut()
                .ok_or("selection missing rows")?,
        )
        .enumerate()
    {
        let tape = ids(&blind[i]["prefix"])?;
        let assertions = arr(&row["assertions"])?;
        let old = arr(&selected["assertions"])?;
        require(
            tape.len() == num(&row["original_tape_len"])? as usize
                && row["prefix_sha256"] == id_hash(&tape)
                && selected["prefix_sha256"] == row["prefix_sha256"]
                && selected["index"] == i
                && old.len() == assertions.len(),
            "selection/capture assertion order changed",
        )?;
        let mut sources = Vec::new();
        if score.is_some() && assertions.is_empty() {
            let empty = by_key
                .remove(&(i as u32, None))
                .ok_or("missing no-fact sentinel")?;
            require(
                empty["status"] == "no_candidate"
                    && empty["source"].is_null()
                    && arr(&empty["candidates"])?.is_empty(),
                "no-fact sentinel nominated source",
            )?;
        }
        for (j, (a, prior)) in assertions.iter().zip(old).enumerate() {
            require(
                a["fact_span"] == prior["fact_span"] && a["fence"] == prior["assertion"]["fence"],
                "old assertion and bounded record disagree",
            )?;
            let candidate = if score.is_some() {
                let result = by_key
                    .remove(&(i as u32, Some(j as u32)))
                    .ok_or("missing decoded assertion")?;
                require(
                    result["fence"] == a["fence"]
                        && result["fact_end_inclusive"] == num(&a["fact_span"][1])? - 1
                        && result["query_start"] == row["requested_span"][0]
                        && result["original_tape_len"] == tape.len()
                        && result["eligible_mask"] == a["eligible_mask"]
                        && result["word_start_mask"] == a["word_start_mask"],
                    "native witness addresses differ from immutable capture",
                )?;
                let decoded = arr(&result["candidates"])?;
                let original = arr(&a["legacy_candidates"])?;
                require(
                    original.len() == decoded.len()
                        && original.iter().zip(decoded).all(|(x, y)| {
                            x["original_start"] == y["source"]
                                && x["first_word_end"] == y["first_word_end"]
                                && ((x["writer_reachable"] == true)
                                    == (y["status"] != "cap_rejected"))
                        }),
                    "native tape changed original legacy candidate words",
                )?;
                let source = opt(&result["source"])?;
                require(
                    (result["status"] == "selected") == source.is_some()
                        && matches!(
                            result["status"].as_str(),
                            Some("selected" | "no_candidate" | "unseen" | "tie" | "nonpositive")
                        ),
                    "invalid native decision status",
                )?;
                source
            } else {
                let old = opt(&prior["assertion"]["value"]["word_start"])?;
                old.filter(|p| reachable(a).is_ok_and(|x| x.contains(p)))
            };
            sources.push(candidate);
        }
        let (c, evidence) = consensus(&tape, assertions, &sources)?;
        let paired = &mut selected["paired"];
        paired["choice"]["fact"] = c["fact"].clone();
        paired["choice"]["source"] = c["source"].clone();
        for name in ["supporting_facts", "complete_values", "values_disagree"] {
            paired[name] = c[name].clone();
        }
        audits.push(json!({"ordinal":i,"prefix_sha256":row["prefix_sha256"],"per_assertion":evidence,"consensus":c}));
    }
    require(by_key.is_empty(), "unmatched native assertion keys")?;
    select["arm"] = json!(format!("diagnostic-{kind}"));
    write(out, &select)?;
    write(
        audit,
        &json!({"selection_sha256":hash(out)?,"candidate_capture_sha256":hash(c)?,
       "trace":if kind=="word-start"{V::Null}else{json!(hash(t)?)},"rows":audits}),
    )
}

fn held_grades(v: &V) -> Result<&[V]> {
    let g = arr(v)?;
    require(
        g.len() == 42
            && g.iter().filter(|r| r["stratum"] == "supported").count() == 36
            && g.iter().filter(|r| !r["matched_triple"].is_null()).count() == 24
            && g.iter()
                .filter(|r| r["stratum"] == "supported" && !r["matched_triple"].is_null())
                .count()
                == 24,
        "held sealing counts differ from 36/42 and 24 matched",
    )?;
    let mut ids = BTreeSet::new();
    let mut triples = BTreeMap::<&str, Vec<u32>>::new();
    for l in g {
        require(ids.insert(text(&l["id"])?), "duplicate held id")?;
        if !l["matched_triple"].is_null() {
            triples
                .entry(text(&l["matched_triple"])?)
                .or_default()
                .push(num(&l["gold_offset"])?);
        }
    }
    require(
        triples.len() == 8
            && triples.values_mut().all(|v| {
                v.sort();
                v.as_slice() == [1, 2, 3]
            }),
        "matched blocks are not the eight frozen 1/2/3 reversals",
    )?;
    Ok(g)
}

fn le(bytes: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or("short native bank")?
            .try_into()?,
    ))
}
fn availability(mode: &str, b: &str, c: &str, g: &str, out: &str) -> Result<()> {
    let mode_id = match mode {
        "ordered" => 0,
        "neutral" => 1,
        "position" => 2,
        _ => return Err("unknown mode".into()),
    };
    let bank = fs::read(b)?;
    require(
        bank.len() == 8416
            && Sha256::digest(&bank[208..]).as_slice() == &bank[176..208]
            && le(&bank, 208)? == 0x31425355
            && le(&bank, 212)? == mode_id
            && le(&bank, 220)? == 256,
        "unbound native bank layout",
    )?;
    let used = le(&bank, 216)? as usize;
    require(used <= 256 && used > 0, "invalid native allocation")?;
    let mut lookup = BTreeMap::<(u32, u32), [bool; 3]>::new();
    for i in 0..used {
        let word = le(&bank, 224 + i * 32)?;
        let class = (word >> 16) as usize;
        require(class < 2, "invalid native class")?;
        let sig = le(&bank, 228 + i * 32)?;
        let radius = le(&bank, 232 + i * 32)?;
        let entry = lookup.entry((word & 0xffff, sig)).or_insert([false; 3]);
        entry[class] = true;
        entry[2] |= radius != 65535;
    }
    let capture = read(c)?;
    let held = read(g)?;
    let membership = arr(&held)?;
    let rows = caprows(&capture)?;
    require(
        rows.len() == 42 && membership.len() == 42 && capture["arm"] == "requested-diagnostic",
        "support gate needs requested held capture",
    )?;
    let mut output = Vec::new();
    let mut ready = 0;
    let mut groups = BTreeMap::<&str, usize>::new();
    for (row, item) in rows.iter().zip(membership) {
        let fields = item.as_object().ok_or("invalid matched membership")?;
        require(
            fields.len() == 3
                && fields.contains_key("matched_triple")
                && item["ordinal"] == row["ordinal"]
                && item["prefix_sha256"] == row["prefix_sha256"],
            "matched membership contains extra data or a shifted row",
        )?;
        let matched = !item["matched_triple"].is_null();
        if matched {
            *groups.entry(text(&item["matched_triple"])?).or_default() += 1;
        }
        let sites = arr(&row["assertions"])?;
        if sites.len() != 1 {
            output.push(
                json!({"ordinal":row["ordinal"],"matched":matched,"blind_sites":sites.len(),
                "all_active_shapes_available":false}),
            );
            continue;
        }
        let assertion = &sites[0];
        let mut candidates = Vec::new();
        let mut eligible = true;
        for q in arr(&assertion["legacy_candidates"])?
            .iter()
            .filter(|x| x["writer_reachable"] == true)
        {
            let depth = num(&q["connector_slots"])?;
            let shape_width = if mode_id == 2 {
                num(&assertion["fence"])?
                    .checked_sub(num(&q["original_start"])?)
                    .ok_or("negative length")?
            } else {
                num(&q["word_slots"])?
            };
            let mut sig = 0;
            let classes = arr(&q["slot_classes"])?;
            require(
                classes.len() <= 10 && depth <= 3 && shape_width <= 7,
                "bad candidate shape",
            )?;
            if mode_id != 2 {
                for (i, v) in classes.iter().enumerate() {
                    let bit = num(v)?;
                    require(bit <= 2, "bad slot class")?;
                    sig |= bit << (2 * i);
                }
            }
            let s = depth | (shape_width << 8);
            let [p, n, radius] = lookup.get(&(s, sig)).copied().unwrap_or_default();
            let available = if mode_id == 2 {
                p || n
            } else {
                p && n && radius
            };
            eligible &= available;
            candidates.push(json!({"source":q["original_start"],"shape_word":s,"signature":sig,
               "positive_shape":p,"negative_shape":n,"fitted_family_radius":if mode_id==2{V::Null}else{json!(radius)},
               "available_without_distance":available}));
        }
        if matched && eligible && candidates.len() == 3 {
            ready += 1;
        }
        output.push(
            json!({"ordinal":row["ordinal"],"matched":matched,"blind_sites":sites.len(),
           "all_active_shapes_available":!candidates.is_empty()&&eligible,"candidates":candidates}),
        );
    }
    require(
        groups.len() == 8 && groups.values().all(|&n| n == 3),
        "matched membership is not eight frozen triples",
    )?;
    write(
        out,
        &json!({"mode":mode,"bank_sha256":sha(&bank),"capture_sha256":hash(c)?,
      "membership_sha256":hash(g)?,
      "allocated_bytes":bank.len(),"used_records":used,"matched_ready":ready,"matched_required":24,
      "no_scoring":true,"rows":output}),
    )
}

fn membership(g: &str, out: &str) -> Result<()> {
    let labels = read(g)?;
    let result=held_grades(&labels)?.iter().enumerate().map(|(i,row)| {
        json!({"ordinal":i,"prefix_sha256":row["prefix_sha256"],"matched_triple":row["matched_triple"]})
    }).collect::<Vec<_>>();
    write(out, &json!(result))
}

fn gold_reach(c: &str, g: &str, out: &str) -> Result<()> {
    let capture = read(c)?;
    let labels = read(g)?;
    let rows = caprows(&capture)?;
    let grades = held_grades(&labels)?;
    require(
        rows.len() == 42 && capture["arm"] == "requested-diagnostic",
        "wrong post-seal reach audit",
    )?;
    let mut audited = Vec::new();
    for (row, g) in rows.iter().zip(grades) {
        grade_binding(g, row)?;
        if g["stratum"] != "supported" {
            continue;
        }
        let gold = num(&g["gold_start"])?;
        let assertion = exact_assertion(row, g)?;
        let legal = reachable(assertion.0)?;
        audited.push(json!({"ordinal":row["ordinal"],"id":g["id"],"matched":!g["matched_triple"].is_null(),
          "authored_gold":gold,"gold_in_reachable_legacy":legal.contains(&gold),"legacy_sources":legal}));
    }
    write(
        out,
        &json!({"rows":audited,"capture_sha256":hash(c)?,"grade_sha256":hash(g)?}),
    )
}

fn supported_view(g: &str, s: &str, out: &str) -> Result<()> {
    let g = read(g)?;
    let g = held_grades(&g)?;
    let mut sel = read(s)?;
    let mut next = Vec::new();
    for (row, label) in arr(&sel["rows"])?.iter().zip(g) {
        require(
            row["prefix_sha256"] == label["prefix_sha256"],
            "selection and grade view prefixes differ",
        )?;
        if label["stratum"] == "supported" {
            next.push(row.clone());
        }
    }
    require(
        next.len() == 36 && arr(&sel["rows"])?.len() == 42,
        "wrong grade-view cohort",
    )?;
    sel["rows"] = json!(next);
    write(out, &sel)
}

fn force(kind: &str, c: &str, b: &str, s: &str, g: &str, out: &str, plan: &str) -> Result<()> {
    let capture = read(c)?;
    let blind = read(b)?;
    let labels = read(g)?;
    let mut sel = read(s)?;
    let cap = caprows(&capture)?;
    let blind = arr(&blind)?;
    let grades = held_grades(&labels)?;
    require(
        cap.len() == 42
            && blind.len() == 42
            && arr(&sel["rows"])?.len() == 42
            && capture["selection_sha256"] == hash(s)?
            && capture["blind_sha256"] == hash(b)?
            && capture["arm"] == "requested-diagnostic",
        "controlled intervention does not bind held inputs",
    )?;
    let mut output = Vec::new();
    let mut planned = Vec::new();
    for (i, ((label, row), old)) in grades.iter().zip(cap).zip(arr(&sel["rows"])?).enumerate() {
        grade_binding(label, row)?;
        let tape = ids(&blind[i]["prefix"])?;
        require(
            row["prefix_sha256"] == id_hash(&tape) && row["prefix_sha256"] == old["prefix_sha256"],
            "controlled intervention prefix changed",
        )?;
        if label["stratum"] != "supported" {
            continue;
        }
        let (assertion, _) = exact_assertion(row, label)?;
        let gold = num(&label["gold_start"])?;
        let fence = num(&assertion["fence"])?;
        let fact = gold_fact(label)?;
        require(
            gold == num(&fact["value_span"][0])?
                && fence == num(&fact["value_span"][1])?
                && completed(&tape, gold, fence)? == ids(&label["expected"])?,
            "gold source does not reproduce authored literal bytes through same frozen cursor",
        )?;
        let rivals = reachable(assertion)?
            .into_iter()
            .filter(|s| *s != gold)
            .collect::<Vec<_>>();
        let matched = !label["matched_triple"].is_null();
        if matched {
            require(
                rivals.len() == 2,
                "matched forced row lacks two genuine reachable rival sources",
            )?;
        }
        let source = match kind {
            "gold" => Some(gold),
            "rival-1" => rivals.first().copied(),
            "rival-2" => rivals.get(1).copied(),
            _ => return Err("unknown forced control".into()),
        };
        let expected = source.map(|s| completed(&tape, s, fence)).transpose()?;
        if kind != "gold" {
            require(
                expected
                    .as_ref()
                    .is_none_or(|e| json!(e) != label["expected"]),
                "rival literal is indistinguishable from gold",
            )?;
        }
        let mut selected = old.clone();
        selected["paired"]["choice"]["source"] = json!(source);
        planned.push(
            json!({"ordinal":i,"source":source,"fence":fence,"matched":matched,
           "ids_through_eos":expected,"id":label["id"]}),
        );
        output.push(selected);
    }
    require(output.len() == 36, "wrong controlled row count")?;
    sel["arm"] = json!(format!("forced-{kind}"));
    sel["rows"] = json!(output);
    write(out, &sel)?;
    write(
        plan,
        &json!({"kind":kind,"selection_sha256":hash(out)?,"rows":planned}),
    )
}

fn wrong_key(g: &str, out: &str) -> Result<()> {
    let mut labels = read(g)?;
    held_grades(&labels)?;
    for r in labels.as_array_mut().ok_or("no labels")? {
        if r["stratum"] != "supported" {
            continue;
        }
        let other = arr(&r["facts"])?
            .iter()
            .filter(|x| x["name"] != r["requested_name"])
            .collect::<Vec<_>>();
        require(other.len() == 1, "wrong-key target not unambiguous")?;
        let other = other[0];
        let start = &other["value_span"][0];
        let value = other["value"].clone();
        let choices = arr(&other["legal"])?
            .iter()
            .filter(|a| a["start"] == *start)
            .collect::<Vec<_>>();
        require(
            choices.len() == 1 && choices[0]["copy_ids_through_eos"] != r["expected"],
            "invalid wrong-key literal",
        )?;
        let target = choices[0]["copy_ids_through_eos"].clone();
        r["answer_text"] = value;
        r["expected"] = target;
    }
    write(out, &labels)
}

fn verify_forced(plan: &str, writer: &str, out: &str) -> Result<()> {
    let plan = read(plan)?;
    let writer = read(writer)?;
    let docs = arr(&plan["rows"])?;
    let values = arr(&writer["rows"])?;
    require(
        writer["selection"] == plan["selection_sha256"] && docs.len() == 36 && values.len() == 36,
        "wrong forced writer receipt",
    )?;
    let mut count = 0;
    let mut matched = 0;
    for (p, w) in docs.iter().zip(values) {
        require(
            w["index"] == p["ordinal"] && w["id"] == p["id"] && w["source"] == p["source"],
            "forced writer changed selected address",
        )?;
        if p["source"].is_null() {
            require(
                p["matched"] == false && plan["kind"] != "gold",
                "matched control lacks source",
            )?;
        } else {
            require(
                w["generated"] == p["ids_through_eos"]
                    && w["terminal_eos"] == true
                    && w["exact"] == (plan["kind"] == "gold"),
                "real GPU writer did not produce the forced original extent",
            )?;
            count += 1;
            if p["matched"] == true {
                matched += 1;
            }
        }
    }
    require(matched == 24, "matched forced controls incomplete")?;
    write(
        out,
        &json!({"kind":plan["kind"],"archived_gpu":writer["gpu"],"valid_on_matched":matched,"valid_all":count}),
    )
}

fn main() -> Result<()> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    match a.as_slice(){
        [op,c,b,x,o] if op=="fit-labels"=>fit_labels(c,b,x,o),
        [op,b,c,g,x] if op=="verify-input-rows"=>verify_input_rows(b,c,g,x),
        [op,mode,b,c,g,o] if op=="availability"=>availability(mode,b,c,g,o),
        [op,g,o] if op=="membership"=>membership(g,o),
        [op,c,g,o] if op=="gold-reach"=>gold_reach(c,g,o),
        [op,mode,c,b,s,t,o,a] if op=="rewrite"=>rewrite(mode,c,b,s,t,o,a),
        [op,g,s,o] if op=="supported-view"=>supported_view(g,s,o),
        [op,k,c,b,s,g,o,p] if op=="force"=>force(k,c,b,s,g,o,p),
        [op,g,o] if op=="wrong-key"=>wrong_key(g,o),
        [op,p,w,o] if op=="verify-forced"=>verify_forced(p,w,o),
        _=>Err("use verify-input-rows BLIND CAPTURE GRADES SUPPLEMENT_OR_DASH | fit-labels CAP BASE_FIT SUPPLEMENT OUT | membership HELD_GRADES OUT | availability MODE BANK CAP MATCHED_MEMBERSHIP OUT | gold-reach CAP HELD_GRADES OUT | rewrite KIND CAP BLIND SELECTION TRACE_OR_MINUS OUT AUDIT | supported-view GRADES SELECTION OUT | force KIND CAP BLIND SELECTION HELD_GRADES OUT PLAN | wrong-key GRADES OUT | verify-forced PLAN WRITER_GRADE OUT".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_rows_bind_real_words_and_refuse_shifted_or_invalid_keys() {
        let temp = std::env::temp_dir().join(format!("uor-study-rows-{}", std::process::id()));
        fs::create_dir(&temp).unwrap();
        let path = |name| temp.join(name).to_str().unwrap().to_owned();
        let (blind, capture, grade, supplement) = (
            path("blind.json"),
            path("capture.json"),
            path("grade.json"),
            path("extra.json"),
        );
        let mut keys = Vec::new();
        let mut rows = Vec::new();
        let mut tapes = Vec::new();
        for i in 0_u32..94 {
            let tape = [0, 11 + i, 23, 16];
            let key = id_hash(&tape);
            let mut g = json!({"id":format!("sample-{i}"),"prefix_sha256":key,
                "requested":[1,2],"stratum":"supported","expected":[23,1],
                "gold_start":2,"gold_assertion_index":0,"facts":[{}]});
            if (36..42).contains(&i) {
                g["stratum"] =
                    json!(["conflict", "unmentioned", "unspecified"][((i - 36) / 2) as usize]);
                for field in ["expected", "gold_start", "gold_assertion_index"] {
                    g[field] = V::Null;
                }
            }
            keys.push(g);
            rows.push(
                json!({"ordinal":i,"original_tape_len":tape.len(),"prefix_sha256":key,
                "requested_span":[1,2]}),
            );
            tapes.push(json!({"prefix":tape,"text":"finite"}));
        }
        let stage = |tapes: &[V], rows: &[V]| {
            let bytes = serde_json::to_vec(tapes).unwrap();
            fs::write(&blind, &bytes).unwrap();
            fs::write(
                &capture,
                json!({"schema":"uor-r4.legacy-assertion-candidates/1",
                "arm":"requested-diagnostic","blind_sha256":sha(&bytes),"rows":rows})
                .to_string(),
            )
            .unwrap();
        };
        let set_keys = |keys: &[V]| fs::write(&grade, json!(keys).to_string()).unwrap();
        set_keys(&keys[..42]);
        stage(&tapes[..42], &rows[..42]);
        assert!(verify_input_rows(&blind, &capture, &grade, "-").is_ok());
        stage(&tapes, &rows);
        fs::write(&supplement, json!(&keys[42..]).to_string()).unwrap();
        assert!(verify_input_rows(&blind, &capture, &grade, &supplement).is_ok());

        tapes[0]["prefix"][1] = json!(14);
        stage(&tapes, &rows);
        assert!(verify_input_rows(&blind, &capture, &grade, &supplement)
            .unwrap_err()
            .to_string()
            .contains("u32le"));
        tapes[0]["prefix"][1] = json!(11);
        stage(&tapes, &rows);
        keys[0]["expected"][1] = json!(2);
        set_keys(&keys[..42]);
        assert!(verify_input_rows(&blind, &capture, &grade, &supplement)
            .unwrap_err()
            .to_string()
            .contains("EOS"));
        keys[0]["expected"][1] = json!(1);
        keys[1]["id"] = keys[0]["id"].clone();
        set_keys(&keys[..42]);
        assert!(verify_input_rows(&blind, &capture, &grade, &supplement)
            .unwrap_err()
            .to_string()
            .contains("duplicate"));
        keys[1]["id"] = json!("sample-1");
        keys[36]["stratum"] = json!("unmentioned");
        set_keys(&keys[..42]);
        assert!(verify_input_rows(&blind, &capture, &grade, &supplement)
            .unwrap_err()
            .to_string()
            .contains("two rows"));
        fs::remove_dir_all(temp).unwrap();
    }
    fn a(start: u32, fence: u32) -> V {
        json!({"fact_span":[start-2,start-1],"fence":fence,
       "legacy_candidates":[{"original_start":start,"writer_reachable":true}]})
    }
    #[test]
    fn frozen_consensus_uses_first_original_source_and_refuses_conflict_or_incomplete() {
        let tape = [0, 9, 30, 42, 50, 16, 9, 30, 42, 50, 16, 81];
        let cases = [a(3, 5), a(8, 10)];
        let (yes, rows) = consensus(&tape, &cases, &[Some(3), Some(8)]).unwrap();
        assert_eq!(yes["source"], 3);
        assert_eq!(rows[0]["ids_through_eos"], json!([42, 50, 1]));
        assert_eq!(yes["values_disagree"], false);
        let mut conflict = tape;
        conflict[9] = 51;
        let (no, _) = consensus(&conflict, &cases, &[Some(3), Some(8)]).unwrap();
        assert_eq!(no["source"], V::Null);
        assert_eq!(no["values_disagree"], true);
        let (c, _) = consensus(&tape, &cases, &[None, Some(8)]).unwrap();
        assert_eq!(c["source"], V::Null);
        assert!(consensus(&tape, &cases, &[Some(4), Some(8)]).is_err());
    }
}
