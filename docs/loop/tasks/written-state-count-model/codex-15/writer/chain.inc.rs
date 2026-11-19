/// Resume the frozen nomination, assertion fence, agreement and exact writer
/// from candidate endpoints. The pre-existing CUDA jobs repeat for each cold run.
pub(crate) fn chain(args: &[String]) -> Result<()> {
    let [top, seed, blind, out, arm_files @ ..] = args else {
        return Err("chain WICK_ROOT SEED BLIND NEW_DIR SELECTION...".into());
    };
    if arm_files.is_empty() || !matches!(seed.as_str(), "1" | "2") {
        return Err("missing seed/arm".into());
    }
    let top = Path::new(top);
    let out = Path::new(out);
    fs::create_dir(out)?;
    let blind_sha = file_hash(Path::new(blind))?;
    let prompts: Vec<super::super::Prompt> = load(blind)?;
    let mut unique = BTreeMap::new();
    let mut mapping = Vec::new();
    for path in arm_files {
        let v: Value = load(path)?;
        if v["blind"] != blind_sha || v["schema"] != "uor-r4.ordered-role.selections/1" {
            return Err("selection does not bind panel".into());
        }
        let mut map = Vec::new();
        for row in v["rows"].as_array().ok_or("missing selections")? {
            let index: usize = serde_json::from_value(row["index"].clone())?;
            let prompt = prompts.get(index).ok_or("invalid row index")?;
            let digest = super::super::selection::prefix_hash(&prompt.prefix);
            if row["prefix_sha256"] != digest {
                return Err("unbound candidate id".into());
            }
            let roles: Option<[u32; 8]> =
                serde_json::from_value(row["paired"]["role_words"].clone())?;
            let key = (
                digest,
                roles.and_then(|r| (r[0] != X).then_some([r[0], r[1]])),
            );
            unique.insert(key.clone(), &prompt.prefix);
            let source: Option<u32> =
                serde_json::from_value(row["paired"]["choice"]["source"].clone())?;
            map.push((index, key, source));
        }
        mapping.push((file_hash(Path::new(path))?, map));
    }
    // The historical cross engine enforces unique bare prefixes within a call.
    // Put competing nominations for one prefix in separate tapes.
    let mut groups: Vec<BTreeMap<Endpoint, &Vec<u32>>> = Vec::new();
    let mut prefix_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (key, ids) in &unique {
        let level = prefix_counts.entry(key.0.clone()).or_default();
        if *level == groups.len() {
            groups.push(BTreeMap::new());
        }
        groups[*level].insert(key.clone(), ids);
        *level += 1;
    }
    let mut rows = Vec::new();
    let mut indices = BTreeMap::new();
    let mut model = None;
    let mut transitions = 0usize;
    for (g, group) in groups.into_iter().enumerate() {
        let entries: Vec<_> = group.into_iter().collect();
        for (c, chunk) in entries.chunks(256).enumerate() {
            let dir = out.join(format!("level-{g}-chunk-{c}"));
            fs::create_dir(&dir)?;
            let tape: BTreeMap<_, _> = chunk.iter().cloned().collect();
            let observed = batch(top, seed, &dir, &tape)?;
            let chunk_rows = observed["rows"]
                .as_array()
                .ok_or("missing chosen trajectories")?;
            if chunk_rows.len() != tape.len() {
                return Err("incomplete device tape".into());
            }
            if model.as_ref().is_some_and(|m| m != &observed["model"]) {
                return Err("model changed between chunks".into());
            }
            model = Some(observed["model"].clone());
            transitions += observed["transitions"]
                .as_u64()
                .ok_or("missing transition count")? as usize;
            for (key, row) in tape.keys().zip(chunk_rows) {
                indices.insert(key.clone(), rows.len());
                rows.push(row.clone());
            }
        }
    }
    fs::create_dir(out.join("nominees"))?;
    let chosen_path = out.join("nominees/chosen.json");
    save(
        &chosen_path,
        &json!({"schema":"uor-r4.ordered-role.joined-trajectories/1","rows":rows,"model":model,"transitions":transitions}),
    )?;
    let mut arms = Vec::new();
    for (sha, map) in mapping {
        let mut arm = Vec::new();
        for (i, key, source) in map {
            let j = indices[&key];
            if rows[j]["source"] != json!(source) || rows[j]["prefix_sha256"] != key.0 {
                return Err(format!(
                    "device source differs from served source at row {i}, arm {sha}"
                )
                .into());
            }
            arm.push([i, j]);
        }
        arms.push(json!({"selection_sha256":sha,"rows":arm}));
    }
    save(
        out.join("receipt.json"),
        &json!({"schema":"uor-r4.ordered-role.replay/1","gpu":file_hash(&chosen_path)?,"blind":blind_sha,"model":model,"writer":WRITER,"unique":unique.len(),"transitions":transitions,"arms":arms}),
    )
}

type Endpoint = (String, Option<[u32; 2]>);
fn batch(
    top: &Path,
    seed: &str,
    out: &Path,
    unique: &BTreeMap<Endpoint, &Vec<u32>>,
) -> Result<Value> {
    let bare = out.join("bare.json");
    save(
        &bare,
        &unique
            .values()
            .map(|p| json!({"prefix":p}))
            .collect::<Vec<_>>(),
    )?;
    let bare_hash = file_hash(&bare)?;
    let prior = top.join("20261118-paired-role-fence");
    let old = top.join("20261118-role-boundary-count");
    let bank = top.join("20261118-content-support/nominee");
    let value = top.join("20261118-value-connectors");
    let model = value.join(format!("inputs/seed{seed}.d11"));
    let tokenizer = value.join(if seed == "1" {
        "inputs/tokenizer.json"
    } else {
        "inputs/tokenizer-seed-2.json"
    });
    let model_hash = file_hash(&model)?;
    let token_hash = file_hash(&tokenizer)?;
    let stage = "primary";
    let decision_rows=unique.keys().map(|(digest,q)|json!({"query_start":q.map(|s|s[0]),"query_end":q.map(|s|s[1]),"prefix_sha256":digest})).collect::<Vec<_>>();
    let endpoint_sha = hash(&serde_json::to_vec(&decision_rows)?);
    fs::create_dir(out.join(stage))?;
    save(
        out.join(stage)
            .join(format!("seed{seed}-panel-nominees.json")),
        &json!({"schema":"uor-r4.paired-role-decisions/1","artifact_sha256":model_hash,"tokenizer_sha256":token_hash,"state_sha256":endpoint_sha,"bare_sha256":bare_hash,"rows":decision_rows,"state_binding":"digest of supplied endpoint tape"}),
    )?;
    let r = Run {
        stage: stage.into(),
        split: "panel".into(),
        seed: seed.to_owned(),
        out: out.into(),
        model_hash: model_hash.clone(),
        bare_hash,
        model,
        bare: bare.clone(),
        tokenizer,
        role_bank: prior.join(format!("role/primary/seed{seed}.role")),
        span: top.join(format!(
            "20261118-span-connectors/results/span-01/seed{seed}-state.json"
        )),
        exact: value.join(format!("results/value-01/seed{seed}-state.json")),
        content: top.join(format!(
            "20261118-content-selector/results/selector-run-01/seed{seed}/trained.json"
        )),
        prior_bank: bank.join(format!("results/seed{seed}-bank.json")),
        fence: prior.join("fence/bin/assertion-fence-final"),
        fence_gpu: prior.join("fence/bin/fence-device"),
        agree: bank.join("agreement/bin/agree"),
        agree_gpu: bank.join("agreement/bin/agree-device"),
        chain: old.join("bin/chain"),
        writer: old.join("bin/action-writer"),
        region: out.into(),
        prior,
        lock: top.join("20261118-diagnostics/wick-codex-gpu.lock"),
        script: top.join("20261118-query-region-pick/writer/run-wick.sh"),
    };
    if file_hash(&r.writer)? != WRITER {
        return Err("unexpected writer".into());
    }
    let mut cache = Cache {
        model: model_hash,
        policy_hash: WRITER.into(),
        entries: BTreeMap::new(),
    };
    let normalized: Vec<Prompt> = load(&bare)?;
    r.arm(
        "nominees",
        &mut cache,
        &normalized,
        &[token_hash, endpoint_sha],
    )?;
    load(out.join("nominees/chosen.json"))
}
