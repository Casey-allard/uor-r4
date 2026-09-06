//! Small relation-memory construction and complete-answer check. Labels and
//! source inspection stay here; inference sees only the raw prompt/model.
use super::*;
use uor_r4_core::native_geometric::{RelationExample, RelationLabel};

#[derive(Serialize, Deserialize)]
struct RelationSource {
    schema: String,
    fit: Vec<RelationExample>,
    development: Vec<RelationExample>,
    first_use: Vec<RelationExample>,
    acceptance: String,
}
fn example(split: &str, world: usize, task: usize, long: bool) -> RelationExample {
    let (names, places) = if split == "first-use-admission" {
        (
            ["quorvik", "ravnel", "senvar", "torvek"],
            ["Weldor", "Xarven", "Yelvik", "Zorlan"],
        )
    } else if split == "first-use-role-path" {
        (
            ["marvik", "nelren", "orveth", "pyrlen"],
            ["Selvik", "Turven", "Uldor", "Vexran"],
        )
    } else if split == "fit-transfer" {
        (
            ["avren", "belvik", "cendra", "dovran"],
            ["Faren", "Belor", "Cevik", "Dulen"],
        )
    } else if split == "fit-transfer-b" {
        (
            ["elvar", "fioren", "girvak", "horven"],
            ["Javor", "Keldin", "Luros", "Morven"],
        )
    } else if split == "fit-transfer-c" {
        (
            ["ivren", "javik", "keldra", "lorvin"],
            ["Norel", "Pavon", "Queris", "Rovak"],
        )
    } else if split == "first-use-v2" {
        (
            ["fenvik", "galdra", "hestin", "jorven"],
            ["Revik", "Soren", "Tavor", "Wexel"],
        )
    } else if split == "first-use" {
        (
            ["nuvra", "peldin", "savren", "talvik"],
            ["Vordel", "Kesnur", "Malven", "Dorvik"],
        )
    } else {
        (
            ["ada", "cyra", "ivy", "mina"],
            ["Rome", "Cairo", "Perth", "Dover"],
        )
    };
    let a = names[world % 4];
    let b = names[(world + 1) % 4];
    let missing = names[(world + 2) % 4];
    let x = places[world % 4];
    let y = places[(world + 1) % 4];
    let z = places[(world + 2) % 4];
    let mut prompt = String::new();
    let mut writes = Vec::new();
    let mut fact = |text: String, owner: &str, value: &str, action| {
        let start = prompt.len();
        let oi = text.find(owner).unwrap();
        let vi = text.find(value).unwrap();
        writes.push(RelationLabel {
            owner_end_byte: (start + oi + owner.len() - 1) as u64,
            value_end_byte: (start + vi + value.len() - 1) as u64,
            action,
        });
        prompt.push_str(&text);
        prompt.push(' ');
    };
    if world % 2 == 0 {
        fact(format!("{a} in {x}."), a, x, 1);
    } else {
        fact(format!("{x} holds {a}."), a, x, 1);
    }
    fact(format!("{b} in {y}."), b, y, 1);
    let (query, answer) = match task {
        0 => (a, x),
        1 => {
            fact(format!("{a} now in {z}."), a, z, 2);
            (a, z)
        }
        2 => {
            fact(format!("{a} now in {z}."), a, z, 2);
            (b, y)
        }
        3 => {
            fact(format!("{a} in {z}."), a, z, 1);
            (a, "Unknown")
        }
        4 => (missing, "Unknown"),
        5 => {
            fact(format!("{a} not in {x}."), a, x, 3);
            (a, "Unknown")
        }
        _ => {
            fact(format!("{a} in {z}."), a, z, 1);
            fact(format!("{a} now in {x}."), a, x, 2);
            (a, x)
        }
    };
    // Fitting includes the same non-fact local context at a small dose. Long
    // evaluation repeats it enough to evict all facts from the 512-token ring.
    for _ in 0..if long { 96 } else { 4 } {
        prompt.push_str("quiet sky. ");
    }
    prompt.push_str(&match world % 3 {
        0 => format!("Where is {query}? Answer:"),
        1 => format!("Which city is {query} in? Answer:"),
        _ => format!("What city does {query} live in? Answer:"),
    });
    RelationExample {
        id: format!("relation/{split}/{world}/{task}"),
        prompt,
        response: format!(" {answer}.\n"),
        writes,
    }
}

pub(super) fn run(mode: &str) -> ProbeResult<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    if mode == "prepare-admission-source" {
        if args.len() != 2 {
            return Err("prepare-admission-source SOURCE NEW_SOURCE".into());
        }
        let mut source: RelationSource = serde_json::from_slice(&fs::read(&args[0])?)?;
        if source.fit.len() != 112 || source.development.len() != 84 || source.first_use.len() != 28
        {
            return Err("expected accepted role-path source".into());
        }
        let prior = serde_json::to_string(&source)?;
        for word in [
            "quorvik", "ravnel", "senvar", "torvek", "Weldor", "Xarven", "Yelvik", "Zorlan",
        ] {
            if prior.contains(word) {
                return Err(format!("new word overlaps prior source: {word}").into());
            }
        }
        source.development.append(&mut source.first_use);
        source.first_use = (0..4)
            .flat_map(|w| {
                (0..7).map(move |t| {
                    let mut c = example("first-use-admission", w, t, true);
                    c.prompt = c
                        .prompt
                        .replace(&"quiet sky. ".repeat(96), &"quiet sky. ".repeat(384));
                    c
                })
            })
            .collect();
        source.acceptance = "112/112 prior answers AND exact writes;62+24prior+8binding and5session reads. Then28 reserved with new source vocabulary and4x filler length, all answers/writes preserved. Compile only unchanged112fit prompts. Compare same64exact keys under geometric, sparse and collapsed partitions; five rotated timing rounds on first7long cases after selection. No learned geometric ranking or general sparse advantage inferred from generic NoWrite reuse.".into();
        write_json(Path::new(&args[1]), &source)?;
        println!(
            "{}",
            json!({"fit":112,"open":112,"reserved":28,"padding_words":768,"acceptance":source.acceptance})
        );
        return Ok(());
    }
    if mode == "compile-admission" {
        use uor_r4_core::native_geometric::RelationAdmissionMode;
        if args.len() != 4 {
            return Err("compile-admission PARENT SOURCE NEW_DIRECTORY NEW_REPORT".into());
        }
        let model = Model::from_bytes(&fs::read(&args[0])?)?;
        let source: RelationSource = serde_json::from_slice(&fs::read(&args[1])?)?;
        fs::create_dir(&args[2])?;
        let (geometric, report) = model.compile_relation_admission(&source.fit)?;
        let mut artifacts = Vec::new();
        for (name, m) in [
            ("geometric", geometric.clone()),
            (
                "sparse",
                geometric.with_relation_admission_mode(RelationAdmissionMode::Sparse)?,
            ),
            (
                "collapsed",
                geometric.with_relation_admission_mode(RelationAdmissionMode::Collapsed)?,
            ),
        ] {
            let bytes = m.to_bytes()?;
            write_new(&Path::new(&args[2]).join(format!("{name}.json")), &bytes)?;
            artifacts.push(json!({"mode":name,"artifact":m.artifact_cid(),"bytes":bytes.len()}));
        }
        write_json(
            Path::new(&args[3]),
            &json!({"compilation":report,"artifacts":artifacts}),
        )?;
        println!("{}", json!({"compilation":report,"artifacts":artifacts}));
        return Ok(());
    }
    if mode == "time-admission" {
        if args.len() != 4 {
            return Err("time-admission PARENT MODEL_DIRECTORY SOURCE NEW_REPORT".into());
        }
        let source: RelationSource = serde_json::from_slice(&fs::read(&args[2])?)?;
        let mut models = Vec::new();
        let mut loads = Vec::new();
        for name in ["parent", "geometric", "sparse", "collapsed"] {
            let path = if name == "parent" {
                PathBuf::from(&args[0])
            } else {
                Path::new(&args[1]).join(format!("{name}.json"))
            };
            let started = Instant::now();
            let bytes = fs::read(path)?;
            let model = Model::from_bytes(&bytes)?;
            loads.push(json!({"mode":name,"load_validate_us":started.elapsed().as_micros(),"serialized_bytes":bytes.len(),"artifact":model.artifact_cid()}));
            models.push((name, model));
        }
        let mut samples = Vec::new();
        for round in 0..5 {
            for i in 0..4 {
                let (name, model) = &models[(i + round) % 4];
                for case in source.first_use.iter().take(7) {
                    let started = Instant::now();
                    let generation = model.generate(&case.prompt, 32, Control::Full)?;
                    let elapsed = started.elapsed().as_micros();
                    if generation.text != case.response {
                        return Err("timed answer differs".into());
                    }
                    samples.push(json!({"round":round,"mode":name,"case":case.id,"generation_us":elapsed,"work":generation.work}));
                }
            }
        }
        write_json(
            Path::new(&args[3]),
            &json!({"loads":loads,"samples":samples,"scope":"One cold file read/load/validation sample per arm;35complete encode-observe-route-score-gather-operate-write-decode generations per arm in five rotated blocks; no timing-only shortcut, all expected text checked. Loading amortization must be reported separately."}),
        )?;
        println!("{}", json!({"samples":samples.len(),"report":args[3]}));
        return Ok(());
    }
    if mode == "role-relations-source" {
        if args.len() != 3 {
            return Err("role-relations-source PARENT_MODEL SOURCE NEW_SOURCE".into());
        }
        let parent: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
        let mut source: RelationSource = serde_json::from_slice(&fs::read(&args[1])?)?;
        if source.fit.len() != 112 || source.development.len() != 56 || source.first_use.len() != 28
        {
            return Err("expected preserved relation coverage source".into());
        }
        let prior = serde_json::to_string(&source)?;
        let dictionary = parent["response_entry"]["copy"]["role_read"]["dictionary"]
            .as_array()
            .ok_or("no parent dictionary")?;
        for novel in [
            "marvik", "nelren", "orveth", "pyrlen", "Selvik", "Turven", "Uldor", "Vexran",
        ] {
            if prior.contains(novel)
                || dictionary.iter().any(|row| {
                    row["bytes"].as_array().is_some_and(|bytes| {
                        bytes
                            .iter()
                            .take(row["len"].as_u64().unwrap_or(0) as usize)
                            .filter_map(Value::as_u64)
                            .map(|b| b as u8)
                            .collect::<Vec<_>>()
                            == novel.as_bytes()
                    })
                })
            {
                return Err(
                    format!("reserved word overlaps previous source/dictionary: {novel}").into(),
                );
            }
        }
        source.development.append(&mut source.first_use);
        source.first_use = (0..4)
            .flat_map(|w| (0..7).map(move |t| example("first-use-role-path", w, t, true)))
            .collect();
        source.acceptance = "84/84 OPEN answers and exact write sequences;62+24prior+8binding preservation and5restored session reads; then28/28 reserved answers AND writes after design selection. Construction112 unchanged. New reserved vocabulary absent from supplied prior relation source and parent dictionary. Known grammar families, not final programme qualification.".into();
        write_json(Path::new(&args[2]), &source)?;
        println!(
            "{}",
            json!({"fit":112,"fit_unchanged":true,"open":84,"reserved":28,"novelty":"PASS","acceptance":source.acceptance})
        );
        return Ok(());
    }
    if mode == "broaden-relations-source" {
        if args.len() != 3 {
            return Err("broaden-relations-source PARENT_MODEL SOURCE NEW_SOURCE".into());
        }
        let parent: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
        let mut source: RelationSource = serde_json::from_slice(&fs::read(&args[1])?)?;
        if source.fit.len() != 56 {
            return Err("expected the first transfer construction".into());
        }
        let old: BTreeSet<_> = source
            .fit
            .iter()
            .chain(&source.development)
            .chain(&source.first_use)
            .flat_map(|c| {
                c.prompt
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .chain(c.response.split(|c: char| !c.is_ascii_alphanumeric()))
            })
            .map(str::to_owned)
            .collect();
        let dictionary = parent["response_entry"]["copy"]["role_read"]["dictionary"]
            .as_array()
            .ok_or("no role dictionary")?;
        for novel in [
            "elvar", "fioren", "girvak", "horven", "Javor", "Keldin", "Luros", "Morven", "ivren",
            "javik", "keldra", "lorvin", "Norel", "Pavon", "Queris", "Rovak",
        ] {
            if old.contains(novel)
                || dictionary.iter().any(|row| {
                    row["bytes"].as_array().is_some_and(|bytes| {
                        bytes
                            .iter()
                            .take(row["len"].as_u64().unwrap_or(0) as usize)
                            .filter_map(Value::as_u64)
                            .map(|b| b as u8)
                            .collect::<Vec<_>>()
                            == novel.as_bytes()
                    })
                })
            {
                return Err(format!("new construction word overlaps: {novel}").into());
            }
        }
        for split in ["fit-transfer-b", "fit-transfer-c"] {
            source
                .fit
                .extend((0..4).flat_map(|w| (0..7).map(move |t| example(split, w, t, false))));
        }
        write_json(Path::new(&args[2]), &source)?;
        println!(
            "{}",
            json!({"fit":source.fit.len(),"open":source.development.len(),"first_use":source.first_use.len(),"first_use_unchanged":true,"novelty_and_dictionary_assertions":"PASS"})
        );
        return Ok(());
    }
    if mode == "repair-relations-source" {
        if args.len() != 3 {
            return Err("repair-relations-source PARENT_MODEL OLD_SOURCE NEW_SOURCE".into());
        }
        let parent: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
        let mut source: RelationSource = serde_json::from_slice(&fs::read(&args[1])?)?;
        let old_words: BTreeSet<_> = source
            .fit
            .iter()
            .chain(&source.development)
            .chain(&source.first_use)
            .flat_map(|c| {
                c.prompt
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .chain(c.response.split(|c: char| !c.is_ascii_alphanumeric()))
            })
            .map(str::to_owned)
            .collect();
        let dictionary = parent["response_entry"]["copy"]["role_read"]["dictionary"]
            .as_array()
            .ok_or("no parent role dictionary")?;
        for novel in [
            "avren", "belvik", "cendra", "dovran", "Faren", "Belor", "Cevik", "Dulen", "fenvik",
            "galdra", "hestin", "jorven", "Revik", "Soren", "Tavor", "Wexel",
        ] {
            if old_words.contains(novel)
                || dictionary.iter().any(|row| {
                    row["bytes"].as_array().is_some_and(|bytes| {
                        bytes
                            .iter()
                            .take(row["len"].as_u64().unwrap_or(0) as usize)
                            .filter_map(Value::as_u64)
                            .map(|b| b as u8)
                            .collect::<Vec<_>>()
                            == novel.as_bytes()
                    })
                })
            {
                return Err(format!("transfer word is not new/dictionary-absent: {novel}").into());
            }
        }
        source
            .fit
            .extend((0..4).flat_map(|w| (0..7).map(move |t| example("fit-transfer", w, t, false))));
        source.development.append(&mut source.first_use);
        source.first_use = (0..4)
            .flat_map(|w| (0..7).map(move |t| example("first-use-v2", w, t, true)))
            .collect();
        source.acceptance="28/28 new first-use exact answers AND exact labeled write sequences;62prior+24role+8binding preserved;5live restored/isolation checks. Original16/28 first-use is now OPEN, retained separately. New construction and new first-use vocabularies are disjoint from prior material and absent from parent dictionary. Known grammar families; no parameter/selection changes after opening new first-use.".into();
        write_json(Path::new(&args[2]), &source)?;
        println!(
            "{}",
            json!({"fit":source.fit.len(),"open":source.development.len(),"first_use":source.first_use.len(),"novelty_and_dictionary_assertions":"PASS","acceptance":source.acceptance})
        );
        return Ok(());
    }
    if mode == "prepare-relations" {
        if args.len() != 2 {
            return Err("prepare-relations SOURCE_V3 NEW_SOURCE".into());
        }
        let prior: Source = serde_json::from_slice(&fs::read(&args[0])?)?;
        validate_source(&prior)?;
        let old: BTreeSet<_> = prior
            .fit
            .iter()
            .chain(&prior.development)
            .flat_map(|c| {
                c.prompt
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .chain(c.response.split(|c: char| !c.is_ascii_alphanumeric()))
            })
            .collect();
        for word in [
            "nuvra", "peldin", "savren", "talvik", "Vordel", "Kesnur", "Malven", "Dorvik",
        ] {
            if old.contains(word) {
                return Err("new relation names overlap supplied prior material".into());
            }
        }
        let source=RelationSource{schema:"uor-r4.relation-source/1".into(),fit:(0..4).flat_map(|w|(0..7).map(move|t|example("fit",w,t,false))).collect(),development:(0..4).flat_map(|w|(0..7).map(move|t|example("development",w,t,true))).collect(),first_use:(0..4).flat_map(|w|(0..7).map(move|t|example("first-use",w,t,true))).collect(),acceptance:"28/28 exact first-use after design selection; all62 prior preservation plus24 role-reader cases and8 binding; original facts absent from both512-token ring and16-word capture; update/unrelated/contradiction/absence separate; restore and scope isolation checks. Known grammar; no final programme heldout claim.".into()};
        let fit: BTreeSet<_> = source.fit.iter().map(|c| &c.prompt).collect();
        if source.first_use.iter().any(|c| fit.contains(&c.prompt)) {
            return Err("relation prompt overlap".into());
        }
        write_json(Path::new(&args[1]), &source)?;
        println!(
            "{}",
            json!({"fit":28,"development":28,"first_use":28,"novelty_and_split_assertions":"PASS","acceptance":source.acceptance})
        );
        return Ok(());
    }
    if args.len() != 4 {
        return Err("fit-relations MODEL SOURCE NEW_MODEL NEW_REPORT; evaluate-relations MODEL SOURCE development|first_use NEW_REPORT".into());
    }
    let model = Model::from_bytes(&fs::read(&args[0])?)?;
    let source: RelationSource = serde_json::from_slice(&fs::read(&args[1])?)?;
    if source.schema != "uor-r4.relation-source/1" {
        return Err("unknown relation source".into());
    }
    if matches!(mode, "fit-relations" | "fit-role-relations") {
        let (next, report) = if mode == "fit-role-relations" {
            model.fit_relations_with_role_paths(&source.fit, 64)?
        } else {
            model.fit_relations(&source.fit, 64)?
        };
        write_new(Path::new(&args[2]), &next.to_bytes()?)?;
        write_json(Path::new(&args[3]), &report)?;
        println!("{report}");
        return Ok(());
    }
    let cases = match args[2].as_str() {
        "development" => &source.development,
        "first_use" => &source.first_use,
        _ => return Err("unknown relation split".into()),
    };
    let start = Instant::now();
    let mut rows = Vec::new();
    for case in cases {
        let encoded = model.encode(&case.prompt)?;
        let fact_end = case
            .writes
            .iter()
            .map(|w| w.owner_end_byte.max(w.value_end_byte))
            .max()
            .ok_or("no facts")?;
        let trailing = model.encode(&case.prompt[fact_end as usize + 1..])?;
        if trailing.len() <= 512 {
            return Err(
                format!("facts not evicted: {} trailing {}", case.id, trailing.len()).into(),
            );
        }
        let began = Instant::now();
        let generation = model.generate(&case.prompt, 32, Control::Full)?;
        let elapsed = began.elapsed().as_micros();
        let mut session = model.session(Control::Full)?;
        session.observe(&model, 0)?;
        for token in &encoded {
            session.observe(&model, *token)?;
        }
        session.begin_response(&model)?;
        let wire: Value = serde_json::from_slice(&session.checkpoint()?)?;
        let words = wire["values"]["lexemes"]["queries"]
            .as_array()
            .ok_or("no recent word capture")?;
        if words.iter().any(|w| {
            w["len"].as_u64().unwrap_or(0) > 0 && w["byte_end"].as_u64().unwrap_or(0) <= fact_end
        }) {
            return Err("fact member still in recent capture".into());
        }
        let restored = model.restore_session(&session.checkpoint()?)?;
        let written: Vec<_> =
            wire["values"]["relations"]["records"]
                .as_array()
                .map_or(Vec::new(), |rs| {
                    rs.iter()
                        .filter(|r| r["id"].as_u64().unwrap_or(0) > 0)
                        .collect()
                });
        let writes_exact = written.len() == case.writes.len()
            && case.writes.iter().all(|label| {
                written.iter().any(|r| {
                    r["owner"]["byte_end"] == label.owner_end_byte
                        && r["value"]["byte_end"] == label.value_end_byte
                        && r["action"] == label.action
                })
            });
        rows.push(json!({"id":case.id,"prompt":case.prompt,"expected":case.response,"exact":generation.text==case.response,"writes_exact":writes_exact,"generation":generation,"generation_elapsed_us":elapsed,"ingested_tokens":encoded.len()+1,"trailing_tokens_after_facts":trailing.len(),"facts_evicted":true,"relation_state":wire["values"]["relations"],"restore_state_equal":restored.state()==session.state()}));
    }
    let exact = rows.iter().filter(|r| r["exact"] == true).count();
    let report = json!({"schema":"uor-r4.relation-evaluation/1","artifact":model.artifact_cid(),"split":args[2],"cases":rows,"exact":exact,"writes_exact":rows.iter().filter(|r|r["writes_exact"]==true).count(),"total":cases.len(),"elapsed_ms":start.elapsed().as_millis(),"scope":"Actual assembled learned write/read and ordinary native generation. A second source ingestion/checkpoint restore supplies eviction and state evidence; its cost is in total elapsed/parent monitor, outside generation subtotals. No semantic labels enter serving."});
    write_json(Path::new(&args[3]), &report)?;
    println!(
        "{}",
        json!({"exact":exact,"total":cases.len(),"report":args[3]})
    );
    Ok(())
}

fn ingest(
    session: &mut uor_r4_core::native_geometric::Session,
    model: &Model,
    text: &str,
) -> ProbeResult<()> {
    for token in model.encode(text)? {
        session.observe(model, token)?;
    }
    Ok(())
}
fn emit_checked(
    session: &mut uor_r4_core::native_geometric::Session,
    model: &Model,
) -> ProbeResult<(String, Value)> {
    session.begin_response(model)?;
    let first = session.predict(model)?.token;
    session.observe(model, first)?;
    let saved = session.checkpoint()?;
    let mut restored = model.restore_session(&saved)?;
    let mut wire: Value = serde_json::from_slice(&saved)?;
    if wire["word_copy"]["read_commit"]["relation_id"].is_u64() {
        let id = wire["word_copy"]["read_commit"]["relation_id"]
            .as_u64()
            .ok_or("missing relation id")?;
        wire["word_copy"]["read_commit"]["relation_id"] = json!(id + 1);
        if model.restore_session(&serde_json::to_vec(&wire)?).is_ok() {
            return Err("forged committed relation version accepted".into());
        }
    }
    let mut output = vec![first];
    for _ in 0..31 {
        let next = session.predict(model)?;
        if restored.predict(model)? != next {
            return Err("restored relation response diverged".into());
        }
        session.observe(model, next.token)?;
        restored.observe(model, next.token)?;
        if next.token == 1 {
            break;
        }
        output.push(next.token);
    }
    let mut original: Value = serde_json::from_slice(&session.checkpoint()?)?;
    let mut resumed: Value = serde_json::from_slice(&restored.checkpoint()?)?;
    let original_stale = original["work"]["memory_stale_rejections"].take();
    let resumed_stale = resumed["work"]["memory_stale_rejections"].take();
    if original != resumed {
        return Err(
            "restored state differs beyond the independently identified stale-index work counter"
                .into(),
        );
    }
    Ok((
        String::from_utf8(model.decode(&output)?)?,
        json!({"all_other_checkpoint_fields_equal":true,"original_memory_stale_rejections":original_stale,"restored_memory_stale_rejections":resumed_stale,"scope":"Existing /4 memory reconstructs stale index slots. Counts are actual work and are preserved separately; no equal-work restore claim."}),
    ))
}

pub(super) fn verify(args: &[String]) -> ProbeResult<()> {
    if args.len() != 2 {
        return Err("verify-relations MODEL NEW_REPORT".into());
    }
    let model = Model::from_bytes(&fs::read(&args[0])?)?;
    let mut live = model.session(Control::Full)?;
    live.observe(&model, 0)?;
    let padding = "quiet sky. ".repeat(96);
    let mut rows = Vec::new();
    for (input, query, expected) in [
        (
            "ada in Rome. cyra in Cairo. ",
            "Where is ada? Answer:",
            " Rome.\n",
        ),
        ("ada now in Perth. ", "Where is ada? Answer:", " Perth.\n"),
        ("", "Where is cyra? Answer:", " Cairo.\n"),
        ("ada in Dover. ", "Where is ada? Answer:", " Unknown.\n"),
        ("", "Where is cyra? Answer:", " Cairo.\n"),
    ] {
        live.end_response(&model)?;
        ingest(&mut live, &model, input)?;
        ingest(&mut live, &model, &padding)?;
        ingest(&mut live, &model, query)?;
        let (text, restore_work) =
            emit_checked(&mut live, &model).map_err(|e| format!("session turn {query}: {e}"))?;
        rows.push(json!({"input":input,"query":query,"expected":expected,"actual":text,"exact":text==expected,"restore_work":restore_work}));
    }
    let mut isolated = model.session(Control::Full)?;
    isolated.observe(&model, 0)?;
    ingest(&mut isolated, &model, "Where is ada? Answer:")?;
    let (isolated_text, isolated_restore_work) = emit_checked(&mut isolated, &model)?;
    let wire: Value = serde_json::from_slice(&live.checkpoint()?)?;
    let history = &wire["values"]["relations"];
    let preserved_old_values = history["records"]
        .as_array()
        .ok_or("no relation records")?
        .iter()
        .filter(|r| r["id"].as_u64().unwrap_or(0) > 0)
        .count();
    let exact = rows.iter().filter(|r| r["exact"] == true).count();
    let report = json!({"schema":"uor-r4.relation-session-check/1","artifact":model.artifact_cid(),"turns":rows,"exact_turns":exact,"total_turns":5,"isolated_response":isolated_text,"isolated_restore_work":isolated_restore_work,"isolated_no_shared_records":isolated_text==" Unknown.\n","preserved_versions":preserved_old_values,"retained_relation_state":history,"restore_and_forged_commit_checks":"PASS","complete_work":live.work,"scope":"One native session reads, revises, retains an unrelated association, records a contradiction, and re-reads the unrelated association after repeated raw-window eviction. Restore is checked after actual first-token commitment; a new session has no shared relation state. Snapshot consistency is not source authentication."});
    write_json(Path::new(&args[1]), &report)?;
    println!(
        "{}",
        json!({"exact_turns":exact,"total_turns":5,"isolated":isolated_text,"report":args[1]})
    );
    Ok(())
}
