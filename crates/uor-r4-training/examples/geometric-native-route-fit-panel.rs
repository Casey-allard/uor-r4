//! Freeze a new source-only transfer panel and separate typed scoring labels.
//! No model predictions or training; tokenization is the retained public protocol.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding, geometric_source_realizer::NativeArtifactBinding,
};
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    geometric_source_emission_view::SourceEmissionCompiler, sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    out: PathBuf,
    checkpoint: PathBuf,
    trusted_binding: PathBuf,
    compiled_panel: PathBuf,
    old_fresh_spec: PathBuf,
    seed: u64,
    prior_exposed_panel_manifest_sha256: String,
    exposed_panel_roots: Vec<ExposedRoot>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExposedRoot {
    root: PathBuf,
    manifest_sha256: String,
}
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn args() -> Result<Args> {
    let mut argv = std::env::args().skip(1);
    let config = argv
        .next()
        .ok_or_else(|| invalid("one JSON config path required"))?;
    if argv.next().is_some() {
        return Err(invalid("only one config argument accepted").into());
    }
    let a: Args = serde_json::from_slice(&fs::read(config)?)?;
    let output = output_support::prospective_output(&a.out)?;
    if a.exposed_panel_roots.is_empty()
        || a.exposed_panel_roots.len() > 32
        || !valid_sha(&a.prior_exposed_panel_manifest_sha256)
    {
        return Err(invalid("declared sealed exposures and prior32 manifest SHA required").into());
    }
    for input in [
        &a.checkpoint,
        &a.trusted_binding,
        &a.compiled_panel,
        &a.old_fresh_spec,
    ]
    .into_iter()
    .chain(a.exposed_panel_roots.iter().map(|r| &r.root))
    {
        let canonical = fs::canonicalize(input)?;
        if output.starts_with(&canonical) {
            return Err(invalid("output beneath input").into());
        }
        for ancestor in canonical.ancestors() {
            if ancestor.is_dir()
                && ancestor.join(report_output::MANIFEST_FILE).exists()
                && output.starts_with(ancestor)
            {
                return Err(invalid("output beneath sealed ancestor").into());
            }
        }
    }
    Ok(a)
}
const QUERIES: [&str; 2] = ["Remind me what my job is.", "Tell me what my job is."];
const POOL_VERSION: &str = "native-route-transfer-pairs-v1";
const MAX_POOL_PAIRS: usize = 512;
#[derive(Clone, serde::Serialize)]
struct PoolPair {
    stratum: String,
    left: String,
    right: String,
}
fn pair_pool() -> Vec<PoolPair> {
    let familiar = ["singer", "dancer", "Brimfold", "Louston"];
    let mut out = Vec::new();
    for a in familiar {
        for b in familiar {
            for c in familiar {
                for d in familiar {
                    let left = format!("{a} {b} {c} {d}");
                    let right = format!("{d} {c} {b} {a}");
                    if left < right {
                        out.push(PoolPair {
                            stratum: "known-token-order-repeat".into(),
                            left,
                            right,
                        });
                    }
                }
            }
        }
    }
    // Fixed finite lexical pool; admission depends only on actual tokenizer IDs,
    // disjointness and public output alphabet, never predictions or native mass.
    let novel = [
        "Norvashiel",
        "Zelmorath",
        "Quivandrel",
        "Tervunath",
        "Vorthalune",
        "Jaxmerith",
        "Caldovren",
        "Sylquanthe",
        "Praxolune",
        "Yevandris",
        "Bexorath",
        "Wulmerith",
        "Kelvazune",
        "Draxomire",
        "Felnovath",
        "Huxariel",
    ];
    for word in novel {
        for (left, right) in [
            (format!("{word} singer"), format!("{word} dancer")),
            (
                format!("singer {word} dancer"),
                format!("dancer {word} singer"),
            ),
            (
                format!("{word} Brimfold {word}"),
                format!("{word} Louston {word}"),
            ),
            (
                format!("Brimfold {word} Louston"),
                format!("Louston {word} Brimfold"),
            ),
        ] {
            out.push(PoolPair {
                stratum: "unseen-token-lexical-composition".into(),
                left,
                right,
            });
        }
    }
    out
}
fn valid_sha(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn seeded_order(count: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<_> = (0..count).collect();
    let mut state = seed;
    for end in (1..count).rev() {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        order.swap(end, (z % (end as u64 + 1)) as usize);
    }
    order
}
// Inspect only declared source/label preparation files, not prediction reports.
fn collect_exposed(
    value: &Value,
    tok: &ByteBpeTokenizer,
    literals: &mut BTreeSet<String>,
    pairs: &mut BTreeSet<(Vec<u32>, Vec<u32>)>,
    fingerprints: &mut BTreeSet<String>,
) -> Result<()> {
    match value {
        Value::Array(rows) => {
            for row in rows {
                collect_exposed(row, tok, literals, pairs, fingerprints)?;
            }
        }
        Value::Object(map) => {
            let declared_literal = map
                .get("literal")
                .or_else(|| map.get("source_text"))
                .and_then(Value::as_str);
            let original = map.get("original_source_ids").or_else(|| {
                value
                    .get("source_view")
                    .and_then(|v| v.get("original_token_ids"))
            });
            if declared_literal.is_some() || original.is_some() {
                let original: Vec<u32> = match original {
                    Some(v) => serde_json::from_value(v.clone())?,
                    None => tok
                        .encode(declared_literal.ok_or_else(|| invalid("exposed literal absent"))?),
                };
                let literal = match declared_literal {
                    Some(v) => v.to_owned(),
                    None => String::from_utf8(tok.decode_bytes(&original))?,
                };
                if tok.decode_bytes(&original) != literal.as_bytes() {
                    return Err(invalid("exposed source bytes/IDs differ").into());
                }
                literals.insert(literal);
                if let Some(query) = map.get("query_ids") {
                    let query: Vec<u32> = serde_json::from_value(query.clone())?;
                    pairs.insert((original.clone(), query.clone()));
                    fingerprints.insert(input_fingerprint(value, &original, &query)?);
                } else if let Some(query) = map.get("query").and_then(Value::as_str) {
                    pairs.insert((original, tok.encode(query)));
                }
            }
            // Fixed old source-swap specs use literal sides rather than compiled rows.
            for side in ["left", "right"] {
                if let Some(literal) = map.get(side).and_then(Value::as_str) {
                    literals.insert(literal.into());
                }
            }
            for nested in map.values() {
                if nested.is_array() || nested.is_object() {
                    collect_exposed(nested, tok, literals, pairs, fingerprints)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
fn input_fingerprint(row: &Value, original: &[u32], query: &[u32]) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(
        &json!({"original_source_ids":original,"query_ids":query,"record":row["record"],"commit":row["commit"],"scope":row["scope"],"entity":row["entity"],"relation":row["relation"],"view":row["view"],"status":row["status"],"actual_prefix_ids":row["actual_prefix_ids"]}),
    )?))
}
fn ids(value: &Value, key: &str) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(value[key].clone())?)
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    report_output::verify(&a.checkpoint)?;
    report_output::verify(
        a.compiled_panel
            .parent()
            .ok_or_else(|| invalid("panel parent absent"))?,
    )?;
    let parent: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.trusted_binding)?)?;
    let native = a.checkpoint.join("realizer-native");
    if sha256_file(&native.join("metadata.json"))? != parent.metadata_sha256 {
        return Err(invalid("trusted parent native metadata differs").into());
    }
    let metadata = read(&native.join("metadata.json"))?;
    if metadata["identity"] != serde_json::to_value(&parent.identity)? {
        return Err(invalid("trusted native identity differs").into());
    }
    let tokenizer_bytes = fs::read(native.join("tokenizer.json"))?;
    let binding = SourceActionBinding::new(&tokenizer_bytes)?;
    if binding.protocol().schema != SCHEMA_V2
        || binding.vocab_size() != 4096
        || binding.tokenizer_sha256() != parent.identity.tokenizer_sha256
    {
        return Err(invalid("protocol/tokenizer public binding differs").into());
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("tokenizer unreadable"))?;
    let compiler = SourceEmissionCompiler::new(&tokenizer_bytes)?;
    let panel = read(&a.compiled_panel)?;
    let training = panel["training64"]
        .as_array()
        .ok_or_else(|| invalid("training64 absent"))?;
    let old32 = panel["untouched32"]
        .as_array()
        .ok_or_else(|| invalid("opened old32 absent"))?;
    if training.len() != 64 || old32.len() != 32 {
        return Err(invalid("retained development/old32 count differs").into());
    }
    let mut excluded = BTreeSet::<String>::new();
    let mut old_ids = BTreeSet::<String>::new();
    let mut exposed_pairs = BTreeSet::<(Vec<u32>, Vec<u32>)>::new();
    let mut source_ids = BTreeSet::<u32>::new();
    let mut context_ids = BTreeSet::<u32>::new();
    for row in training {
        let literal = row["literal"]
            .as_str()
            .ok_or_else(|| invalid("training literal missing"))?;
        excluded.insert(literal.into());
        old_ids.insert(
            row["id"]
                .as_str()
                .ok_or_else(|| invalid("training id missing"))?
                .into(),
        );
        let original = ids(row, "original_source_ids")?;
        let query = ids(row, "query_ids")?;
        let view = compiler.compile(&original)?;
        if view.original_bytes() != literal.as_bytes()
            || serde_json::to_value(&view)? != row["source_view"]
        {
            return Err(invalid("training source-only view identity differs").into());
        }
        let target = ids(row, "target_ids_labels_only")?;
        if target.last().copied() != Some(binding.eos_token_id()) {
            return Err(invalid("training EOS binding differs").into());
        }
        source_ids.extend(view.emitted_token_ids());
        context_ids.extend(view.emitted_token_ids());
        context_ids.extend(&query);
        context_ids.extend(target.iter().take(target.len().saturating_sub(1)));
        exposed_pairs.insert((original, query));
    }
    for wrapper in old32 {
        let row = &wrapper["compiled"];
        let literal = row["literal"]
            .as_str()
            .ok_or_else(|| invalid("old32 literal missing"))?;
        excluded.insert(literal.into());
        old_ids.insert(
            row["id"]
                .as_str()
                .ok_or_else(|| invalid("old32 id missing"))?
                .into(),
        );
        exposed_pairs.insert((ids(row, "original_source_ids")?, ids(row, "query_ids")?));
    }
    let old8 = read(&a.old_fresh_spec)?;
    let old8pairs = old8["pairs"]
        .as_array()
        .ok_or_else(|| invalid("old8 spec absent"))?;
    if old8pairs.len() != 4 {
        return Err(invalid("old8 fixed pair count differs").into());
    }
    for pair in old8pairs {
        for side in ["left", "right"] {
            excluded.insert(
                pair[side]
                    .as_str()
                    .ok_or_else(|| invalid("old8 literal missing"))?
                    .into(),
            );
        }
    }
    // Retained old16 source literals, including duplicated shared-prefix rows.
    for literal in [
        "singer",
        "dancer",
        "Brimfold",
        "Louston",
        "singer dancer",
        "dancer singer",
        "singer singer",
        "dancer dancer",
        "singersinger",
        "dancerdancer",
        "Vexalnorp",
        "Quendazith",
        "Vexalnorp singer",
        "Vexalnorp dancer",
    ] {
        excluded.insert(literal.into());
    }
    let mut exposed_receipts = Vec::new();
    let mut complete_fingerprints = BTreeSet::new();
    let mut seen_roots = BTreeSet::new();
    let mut prior_seen = false;
    let mut training_seen = false;
    let compiled_parent = fs::canonicalize(
        a.compiled_panel
            .parent()
            .ok_or_else(|| invalid("compiled parent absent"))?,
    )?;
    for exposure in &a.exposed_panel_roots {
        let root = fs::canonicalize(&exposure.root)?;
        if !seen_roots.insert(root.clone()) || !valid_sha(&exposure.manifest_sha256) {
            return Err(invalid("duplicate exposed root or malformed manifest SHA").into());
        }
        report_output::verify(&root)?;
        let manifest_sha = sha256_file(&root.join(report_output::MANIFEST_FILE))?;
        if manifest_sha != exposure.manifest_sha256 {
            return Err(invalid("exposed manifest identity differs").into());
        }
        training_seen |= root == compiled_parent;
        let mut input_receipts = Vec::new();
        for name in [
            "compiled.json",
            "panel.json",
            "labels.json",
            "panel-spec.json",
            "source-inputs.json",
        ] {
            let file = root.join(name);
            if file.is_file() {
                collect_exposed(
                    &read(&file)?,
                    &tok,
                    &mut excluded,
                    &mut exposed_pairs,
                    &mut complete_fingerprints,
                )?;
                input_receipts.push(json!({"file":name,"sha256":sha256_file(&file)?}));
            }
        }
        if input_receipts.is_empty() {
            return Err(invalid("sealed exposure has no declared preparation inputs").into());
        }
        if manifest_sha == a.prior_exposed_panel_manifest_sha256 {
            let prior = read(&root.join("report.json"))?;
            if prior["schema"] != "uor-r4.geometric-dependent-fit-panel/1"
                || prior["cases"] != 32
                || prior["status"] != "completed"
            {
                return Err(
                    invalid("required previous dependent32 preparation identity differs").into(),
                );
            }
            let prior_labels = read(&root.join("labels.json"))?;
            if prior_labels["cases"].as_array().map(Vec::len) != Some(32) {
                return Err(invalid("prior32 labels absent").into());
            }
            prior_seen = true;
        }
        exposed_receipts.push(
            json!({"root":root,"manifest_sha256":manifest_sha,"preparation_files":input_receipts}),
        );
    }
    if !prior_seen || !training_seen {
        return Err(invalid("required prior32 and training64 sealed roots missing").into());
    }
    write(
        &a.out,
        "exclusions.json",
        &json!({"schema":"uor-r4.native-route-transfer-exclusions/1","exposed_panel_roots":exposed_receipts,"prior_exposed_panel_manifest_sha256":a.prior_exposed_panel_manifest_sha256,"source_literals":excluded,"source_query_pairs":exposed_pairs,"complete_input_fingerprints":complete_fingerprints,"old8_spec_sha256":sha256_file(&a.old_fresh_spec)?}),
    )?;
    write(
        &a.out,
        "training-known-ids.json",
        &json!({"source_ids":source_ids,"context_ids":context_ids,"training_cases":64,"training_panel_sha256":sha256_file(&a.compiled_panel)?}),
    )?;
    let pool = pair_pool();
    if pool.len() > MAX_POOL_PAIRS {
        return Err(invalid("finite pool bound exceeded").into());
    }
    write(
        &a.out,
        "pair-pool.json",
        &json!({"version":POOL_VERSION,"pairs":pool,"seed":a.seed,"shuffle":"SplitMix64 Fisher-Yates v1; explicit wrapping offline arithmetic"}),
    )?;
    let mut eligible = Vec::new();
    let mut selected = Vec::new();
    let mut chosen_literals = BTreeSet::new();
    let mut class_counts = [0usize; 2];
    for pool_index in seeded_order(pool.len(), a.seed) {
        let pair = &pool[pool_index];
        let class = usize::from(pair.stratum == "unseen-token-lexical-composition");
        let mut reasons = Vec::new();
        for literal in [&pair.left, &pair.right] {
            let original = tok.encode(literal);
            let view = compiler.compile(&original)?;
            let unseen = view
                .emitted_token_ids()
                .iter()
                .any(|id| !source_ids.contains(id));
            if unseen != (class == 1) {
                reasons.push("actual-token-familiarity");
            }
            if excluded.contains(literal) || chosen_literals.contains(literal) {
                reasons.push("exposed-or-already-selected-literal");
            }
            let mut target = tok.encode(&format!(" {literal}."));
            target.push(binding.eos_token_id());
            let mut supported = view.emitted_token_ids().to_vec();
            supported.extend([binding.period_token_id(), binding.eos_token_id()]);
            if target != supported
                || target.len() > 64
                || original.is_empty()
                || view.original_bytes() != literal.as_bytes()
            {
                reasons.push("public-alphabet-or-bytes");
            }
            for query in QUERIES {
                let q = tok.encode(query);
                if original.len() + q.len() + 64 > 128
                    || view.emitted_token_ids().len() + q.len() + 64 > 128
                {
                    reasons.push("context-window");
                }
                if exposed_pairs.contains(&(original.clone(), q)) {
                    reasons.push("exposed-source-query");
                }
            }
        }
        let admitted = reasons.is_empty() && class_counts[class] < 8;
        eligible.push(json!({"pool_index":pool_index,"class":class,"selected":admitted,"eligibility_reasons":reasons,"class_quota_full":class_counts[class]>=8}));
        if admitted {
            chosen_literals.insert(pair.left.clone());
            chosen_literals.insert(pair.right.clone());
            selected.push((pool_index, class_counts[class], class));
            class_counts[class] += 1;
        }
    }
    write(
        &a.out,
        "eligibility.json",
        &json!({"schema":"uor-r4.native-route-transfer-eligibility/1","seed":a.seed,"pool_version":POOL_VERSION,"rows":eligible,"selected_pool_indices":selected,"class_pair_counts":class_counts,"predictions":"NOT_RUN","native_target_mass":"NOT_RUN"}),
    )?;
    if class_counts != [8, 8] {
        return Err(invalid(
            "finite seeded pool lacks eight eligible pairs per measured token class",
        )
        .into());
    }
    selected.sort_by_key(|(_, class_index, class)| (*class, *class_index));
    let template = &training[0];
    let record = template["record"]
        .as_u64()
        .ok_or_else(|| invalid("frame record missing"))?;
    let commit = template["commit"]
        .as_u64()
        .ok_or_else(|| invalid("frame commit missing"))?;
    let relation = template["relation"]
        .as_u64()
        .ok_or_else(|| invalid("frame relation missing"))?;
    let entity = ids(template, "entity")?;
    let mut packets = Vec::new();
    let mut labels = Vec::new();
    let mut literals = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    let mut query_counts = [0usize; 2];
    let mut class_query_counts = [[0usize; 2]; 2];
    let mut source_familiar = 0usize;
    let mut context_familiar = 0usize;
    for (pair_index, (pool_index, class_index, class)) in selected.iter().enumerate() {
        let pool_pair = &pool[*pool_index];
        let stratum = &pool_pair.stratum;
        let query = QUERIES[*class_index % 2];
        for (side, literal) in [
            ("left", pool_pair.left.as_str()),
            ("right", pool_pair.right.as_str()),
        ] {
            let id = format!("native-route-fresh-{:02}", packets.len());
            if !literals.insert(literal.to_owned())
                || excluded.contains(literal)
                || old_ids.contains(&id)
            {
                return Err(invalid(format!("prospective literal/id collision {literal}")).into());
            }
            let original = tok.encode(literal);
            let query_ids = tok.encode(query);
            if !pairs.insert((original.clone(), query_ids.clone()))
                || exposed_pairs.contains(&(original.clone(), query_ids.clone()))
            {
                return Err(invalid("full source/query pair is not fresh").into());
            }
            let view = compiler.compile(&original)?;
            let answers = FrozenAnswers {
                intent: RecordedValueIntent::Current,
                accepted: vec![format!("{literal}.")],
            };
            answers.validate()?;
            let mut target = tok.encode(&format!(" {}", answers.accepted[0]));
            target.push(binding.eos_token_id());
            let mut supported = view.emitted_token_ids().to_vec();
            supported.push(binding.period_token_id());
            supported.push(binding.eos_token_id());
            if target != supported
                || original.is_empty()
                || query_ids.is_empty()
                || target.len() > 64
                || original.len() + query_ids.len() + 64 > 128
                || view.emitted_token_ids().len() + query_ids.len() + 64 > 128
                || original
                    .iter()
                    .chain(&query_ids)
                    .chain(&target)
                    .any(|id| *id as usize >= binding.vocab_size())
                || view.original_bytes() != literal.as_bytes()
            {
                return Err(invalid(
                    "actual BPE/protocol/window or Copy+Period+EOS alphabet admission failed",
                )
                .into());
            }
            let unseen_source = view
                .emitted_token_ids()
                .iter()
                .filter(|id| !source_ids.contains(id))
                .copied()
                .collect::<BTreeSet<_>>();
            let unseen_context = view
                .emitted_token_ids()
                .iter()
                .filter(|id| !context_ids.contains(id))
                .copied()
                .collect::<BTreeSet<_>>();
            let unseen_query = query_ids
                .iter()
                .filter(|id| !context_ids.contains(id))
                .copied()
                .collect::<BTreeSet<_>>();
            source_familiar += usize::from(unseen_source.is_empty());
            context_familiar += usize::from(unseen_context.is_empty());
            query_counts[*class_index % 2] += 1;
            class_query_counts[*class][*class_index % 2] += 1;
            let packet = json!({"id":id,"record":record,"commit":commit,"scope":"m-world-v2","entity":entity,"relation":relation,"view":0,"status":"Found","original_source_ids":original,"query_ids":query_ids,"actual_prefix_ids":[]});
            if complete_fingerprints.contains(&input_fingerprint(&packet, &original, &query_ids)?) {
                return Err(invalid("complete source/frame/query fingerprint was exposed").into());
            }
            packets.push(packet);
            labels.push(json!({"id":id,"stratum":stratum,"pair_id":format!("native-route-pair-{pair_index:02}"),"side":side,"pool_index":pool_index,"literal":literal,"query":query,"answers":answers,"target_ids_labels_only":target,"source_view":view,"token_support":{"unseen_emitted_source_ids_vs_training64":unseen_source,"unseen_emitted_source_ids_vs_training64_context":unseen_context,"unseen_query_ids_vs_training64_context":unseen_query,"target_copy_period_eos_alphabet_supported":true,"positive_native_target_mass":"NOT_RUN"}}));
        }
    }
    if packets.len() != 32
        || query_counts != [16, 16]
        || source_familiar != 16
        || class_query_counts != [[8, 8], [8, 8]]
    {
        return Err(invalid("prospective panel32/query balance differs").into());
    }
    write(
        &a.out,
        "source-inputs.json",
        &json!({"schema":"uor-r4.geometric-dependent-fit-source-inputs/1","cases":packets}),
    )?;
    write(
        &a.out,
        "labels.json",
        &json!({"schema":"uor-r4.geometric-dependent-fit-labels/1","cases":labels}),
    )?;
    Ok(
        json!({"schema":"uor-r4.geometric-dependent-fit-panel/1","panel_family":"native-route-seeded-transfer-v1","preparation_seed":a.seed,"finite_pool_version":POOL_VERSION,"finite_pool_sha256":sha256_file(&a.out.join("pair-pool.json"))?,"exclusion_receipt_sha256":sha256_file(&a.out.join("exclusions.json"))?,"training_known_ids_sha256":sha256_file(&a.out.join("training-known-ids.json"))?,"eligibility_receipt_sha256":sha256_file(&a.out.join("eligibility.json"))?,"prior_exposed_panel_manifest_sha256":a.prior_exposed_panel_manifest_sha256,"exposed_panel_roots":exposed_receipts,"status":"completed","predictions":"NOT_RUN","optimizer_updates":0,"cases":32,"source_literal_disjoint_training64_old32_old8_old16":true,"full_source_query_pairs_disjoint":true,"same_frame_within_every_pair":true,"source_literal_disjoint_all_declared_sealed_exposures":true,"complete_input_fingerprints_disjoint":true,"query_counts":query_counts,"class_query_counts":class_query_counts,"actual_BPE_familiar_source_rows":source_familiar,"actual_BPE_familiar_context_rows":context_familiar,"native_target_mass":"NOT_RUN; only public output-alphabet support checked","trusted_parent":parent,"checkpoint_manifest_sha256":sha256_file(&a.checkpoint.join("manifest.json"))?,"trusted_binding_sha256":sha256_file(&a.trusted_binding)?,"training_and_old32_panel_sha256":sha256_file(&a.compiled_panel)?,"old8_spec_sha256":sha256_file(&a.old_fresh_spec)?,"source_inputs_sha256":sha256_file(&a.out.join("source-inputs.json"))?,"labels_sha256":sha256_file(&a.out.join("labels.json"))?,"label_policy":"FrozenAnswers Current authored once from fixed literal intent; exact membership only; excluded from native read","scope":"32 prospective authored selected-source copy-transfer rows; no general prose/reasoning/chat/energy qualification","elapsed_seconds":start.elapsed().as_secs_f64()}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = (|| -> Result<Value> {
        let mut report = run(&a, start)?;
        let head = option_env!("UOR_BUILD_SOURCE_COMMIT")
            .ok_or_else(|| invalid("build source commit absent"))?;
        if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid("build source commit invalid").into());
        }
        report["source_commit"] = json!(head);
        Ok(report)
    })();
    match &result {
        Ok(report) => write(&a.out, "report.json", report)?,
        Err(error) => write(
            &a.out,
            "failure.json",
            &json!({"status":"failed","error":error.to_string(),"predictions":"NOT_RUN","optimizer_updates":0,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?,
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_pair_pool_has_unique_changed_sides_and_bounded_capacity() {
        let pool = pair_pool();
        assert!(pool.len() <= MAX_POOL_PAIRS);
        let mut pairs = BTreeSet::new();
        for p in &pool {
            assert_ne!(p.left, p.right);
            assert!(pairs.insert((p.left.clone(), p.right.clone())));
        }
        assert!(
            pool.iter()
                .filter(|p| p.stratum == "known-token-order-repeat")
                .count()
                >= 8
        );
        assert!(
            pool.iter()
                .filter(|p| p.stratum == "unseen-token-lexical-composition")
                .count()
                >= 8
        );
    }
    #[test]
    fn seeded_pool_order_is_complete_reproducible_and_seed_sensitive() {
        let n = pair_pool().len();
        let a = seeded_order(n, 42);
        let b = seeded_order(n, 43);
        assert_eq!(a, seeded_order(n, 42));
        assert_ne!(a, b);
        assert_eq!(a.iter().copied().collect::<BTreeSet<_>>(), (0..n).collect());
    }
}
