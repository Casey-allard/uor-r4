// The previous unit's sealed trajectory validator is the oracle for this unit too.
#[allow(dead_code)]
mod frozen {
    include!("../../codex-13/writer/driver.rs");
    include!("chain.inc.rs");

    pub(crate) fn replay(args: &[String]) -> Result<()> {
        let [device, model, blind, out, arms @ ..] = args else {
            return Err("replay WRITER BASE BLIND NEW_DIR SELECTION...".into());
        };
        if arms.is_empty() || file_hash(Path::new(device))? != WRITER {
            return Err("missing arms or wrong frozen writer".into());
        }
        let model_sha = file_hash(Path::new(model))?;
        let blind_sha = file_hash(Path::new(blind))?;
        let prompts: Vec<super::super::Prompt> = load(blind)?;
        let out = Path::new(out);
        fs::create_dir(out)?;
        let mut keys = BTreeMap::new();
        let mut mapping = Vec::new();
        for arm in arms {
            let record: Value = load(arm)?;
            if record["blind"] != blind_sha
                || record["schema"] != "uor-r4.ordered-role.selections/1"
            {
                return Err("wrong blind input or schema".into());
            }
            let rows = record["rows"].as_array().ok_or("no selections")?;
            let mut indices = Vec::new();
            for row in rows {
                let i: usize = serde_json::from_value(row["index"].clone())?;
                let ids = &prompts.get(i).ok_or("index out of range")?.prefix;
                let digest = super::super::selection::prefix_hash(ids);
                if digest != row["prefix_sha256"] {
                    return Err("selection prefix mismatch".into());
                }
                let source: Option<u32> =
                    serde_json::from_value(row["paired"]["choice"]["source"].clone())?;
                let key = (digest, source);
                keys.insert(key.clone(), ids);
                indices.push((i, key));
            }
            mapping.push((file_hash(Path::new(arm))?, indices));
        }
        let prompts_path = out.join("bare.json");
        let picks_path = out.join("picks.json");
        let manifest = out.join("manifest.json");
        let observed = out.join("gpu.json");
        save(
            &prompts_path,
            &keys
                .values()
                .map(|p| json!({"prefix":p}))
                .collect::<Vec<_>>(),
        )?;
        let bare_sha = file_hash(&prompts_path)?;
        save(
            &picks_path,
            &json!({"schema":"uor-r4.entity-pair-chain/1","arm":"action","artifact_sha256":model_sha,"bare_sha256":bare_sha,"rows":keys.keys().map(|(digest,source)|json!({"prefix_sha256":digest,"continuation":{"source":source,"reason":0,"connector_mask":0,"onset_mask":0}})).collect::<Vec<_>>()}),
        )?;
        let picks_sha = file_hash(&picks_path)?;
        save(
            &manifest,
            &json!([{"model":model_sha,"bare":bare_sha,"picks":picks_sha}]),
        )?;
        run(Command::new(device)
            .arg("entity-writer-action")
            .arg(&manifest)
            .arg(file_hash(&manifest)?)
            .arg(model)
            .arg(&prompts_path)
            .arg(&picks_path)
            .arg(&observed))?;
        let mut cache = Cache {
            model: model_sha.clone(),
            policy_hash: WRITER.to_owned(),
            entries: BTreeMap::new(),
        };
        let transitions = cache.import(&observed, &picks_path, &prompts_path)?;
        let enumerated: BTreeMap<_, _> = keys.keys().enumerate().map(|(i, key)| (key, i)).collect();
        let arm_indices = mapping
            .into_iter()
            .map(|(sha, indices)| {
                json!({"selection_sha256":sha,
            "rows":indices.into_iter().map(|(row,key)|[row,enumerated[&key]]).collect::<Vec<_>>() })
            })
            .collect::<Vec<_>>();
        save(
            out.join("receipt.json"),
            &json!({"schema":"uor-r4.ordered-role.replay/1","gpu":file_hash(&observed)?,"model":model_sha,"writer":WRITER,"blind":blind_sha,"unique":keys.len(),"transitions":transitions,"arms":arm_indices}),
        )
    }
}
pub(super) use frozen::{chain, replay};
