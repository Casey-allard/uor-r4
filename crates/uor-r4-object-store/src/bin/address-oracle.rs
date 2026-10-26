use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;
use uor_r4_object_store::address_oracle::build_panel;
#[cfg(feature = "cuda")]
use uor_r4_object_store::address_oracle::{
    baseline_records_identical, hard_gates, random_control_holds, Arm,
};
use uor_r4_object_store::address_stage0::sha256_hex;
use uor_r4_tokenizer::ByteBpeTokenizer;

const ORACLE_PANEL_SHA256: &str =
    "a4aa548f3e38478fdc8cd4ad077d9886cefa3aaa8e2712abe8f38abe0dba4103";

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("refused: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, tokenizer_path, output] if command == "manifest" => {
            let bytes = read_file(tokenizer_path, "tokenizer")?;
            let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
                .ok_or_else(|| format!("{tokenizer_path}: invalid pinned tokenizer"))?;
            let panel = build_panel(&tokenizer)?;
            panel.validate(Some(&tokenizer))?;
            let encoded = panel.encode()?;
            let digest = sha256_hex(&encoded);
            if ORACLE_PANEL_SHA256 != "TO_PIN_AFTER_GENERATION" && digest != ORACLE_PANEL_SHA256 {
                return Err(format!(
                    "generated oracle manifest {digest} differs from pin {ORACLE_PANEL_SHA256}"
                ));
            }
            write_new(std::path::Path::new(output), &encoded)?;
            println!("address-oracle-panel: 80 rows, sha256 {digest}");
            Ok(())
        }
        [command, artifact, config, receipt, tokenizer, stage0_manifest, oracle_manifest, out_dir]
            if command == "run" =>
        {
            #[cfg(feature = "cuda")]
            {
                return run_oracle(
                    artifact,
                    config,
                    receipt,
                    tokenizer,
                    stage0_manifest,
                    oracle_manifest,
                    out_dir,
                );
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = (
                    artifact,
                    config,
                    receipt,
                    tokenizer,
                    stage0_manifest,
                    oracle_manifest,
                    out_dir,
                );
                return Err("address-oracle run requires --features cuda".into());
            }
        }
        _ => Err(concat!(
            "usage: address-oracle manifest TOKENIZER_JSON OUT_JSON | ",
            "address-oracle run ARTIFACT_D11 CONFIG_JSON ORIGINAL_FINAL_DEV_JSON ",
            "TOKENIZER_JSON STAGE0_MANIFEST_JSON ORACLE_MANIFEST_JSON OUT_DIR"
        )
        .into()),
    }
}

fn read_file(path: &str, label: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("{label} {path}: {e}"))
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e} (outputs are never overwritten)", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(feature = "cuda")]
fn write_json(path: &std::path::Path, value: &serde_json::Value) -> Result<(), String> {
    let mut encoded = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    encoded.push('\n');
    write_new(path, encoded.as_bytes())
}

#[cfg(feature = "cuda")]
fn cuda_driver_api_version() -> Result<i32, String> {
    let mut raw = 0;
    // SAFETY: cuDriverGetVersion writes one c_int to a valid stack pointer.
    let status = unsafe { cudarc::driver::sys::cuDriverGetVersion(&mut raw) };
    if status == cudarc::driver::sys::CUresult::CUDA_SUCCESS && raw > 0 {
        Ok(raw)
    } else {
        Err(format!(
            "cuDriverGetVersion returned {status:?} (raw {raw})"
        ))
    }
}

#[cfg(feature = "cuda")]
fn run_oracle(
    artifact: &str,
    config: &str,
    receipt: &str,
    tokenizer: &str,
    stage0_manifest: &str,
    oracle_manifest: &str,
    out_dir: &str,
) -> Result<(), String> {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;
    use uor_r4_discrete::training::DriverConfig;
    use uor_r4_object_store::address_oracle::{
        corpus_value_leaks, form_sieve_query, prompt_value_leaks, score_model_arm,
        score_store_read, ArmRecord, OracleStore, PanelManifest,
    };
    use uor_r4_object_store::address_stage0::runner::{run_parity, RunPaths};
    use uor_r4_object_store::address_stage0::{
        normalize_emission, u16_le_tokens, ARTIFACT_SHA256, DEV_MASK_SHA256, DEV_TOKENS_SHA256,
        ORIGINAL_CONFIG_SHA256, STAGE0_HEAD_ENV, STAGED_INPUT_CONFIG_SHA256, TOKENIZER_SHA256,
        TRAIN_MASK_SHA256, TRAIN_TOKENS_SHA256,
    };
    use uor_r4_object_store::{NodeId, Record, RecordBackend};

    let out = PathBuf::from(out_dir);
    let parity_paths = RunPaths {
        artifact: PathBuf::from(artifact),
        config: PathBuf::from(config),
        retained_receipt: PathBuf::from(receipt),
        tokenizer: PathBuf::from(tokenizer),
        manifest: PathBuf::from(stage0_manifest),
        out_dir: out.clone(),
    };
    let parity = run_parity(&parity_paths, cuda_driver_api_version())?;
    if !parity.parity_pass {
        return Err("D11 parity did not pass".into());
    }

    let tokenizer_bytes = read_file(tokenizer, "tokenizer")?;
    if sha256_hex(&tokenizer_bytes) != TOKENIZER_SHA256 {
        return Err("tokenizer hash differs from pin".into());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "invalid pinned tokenizer".to_string())?;
    let panel_bytes = read_file(oracle_manifest, "oracle manifest")?;
    let panel_sha256 = sha256_hex(&panel_bytes);
    if ORACLE_PANEL_SHA256 == "TO_PIN_AFTER_GENERATION" || panel_sha256 != ORACLE_PANEL_SHA256 {
        return Err(format!(
            "oracle manifest sha256 {panel_sha256} is not the registered pin"
        ));
    }
    let panel = PanelManifest::parse(&panel_bytes)?;
    panel.validate(Some(&tokenizer))?;
    let oracle = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;

    let prompt_leaks = prompt_value_leaks(&panel);
    if !prompt_leaks.is_empty() {
        return Err(format!("panel prompt value leaks: {prompt_leaks:?}"));
    }
    let config_bytes = read_file(config, "config")?;
    let normalized_config_sha256 = sha256_hex(&config_bytes);
    let driver_config: DriverConfig =
        serde_json::from_slice(&config_bytes).map_err(|e| format!("driver config: {e}"))?;
    let train_tokens = u16_le_tokens(
        &driver_config
            .train
            .tokens
            .read()
            .map_err(|e| format!("TRAIN tokens: {e}"))?,
    )?;
    let dev_tokens = u16_le_tokens(
        &driver_config
            .dev
            .tokens
            .read()
            .map_err(|e| format!("DEV tokens: {e}"))?,
    )?;
    let train_leaks = corpus_value_leaks(&panel, &train_tokens);
    let dev_leaks = corpus_value_leaks(&panel, &dev_tokens);
    if !train_leaks.is_empty() || !dev_leaks.is_empty() {
        return Err(format!(
            "oracle corpus leaks: TRAIN {train_leaks:?}, DEV {dev_leaks:?}"
        ));
    }

    struct CountingBackend<'a> {
        store: &'a uor_r4_object_store::Store,
        fetches: AtomicUsize,
    }
    impl RecordBackend for CountingBackend<'_> {
        fn fetch(&self, id: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
            self.fetches.fetch_add(1, Ordering::SeqCst);
            self.store.fetch(id)
        }
    }

    let artifact_bytes = read_file(artifact, "artifact")?;
    if sha256_hex(&artifact_bytes) != ARTIFACT_SHA256 {
        return Err("artifact hash differs from pin".into());
    }
    let (tables, _model) =
        m1_d11_artifact_scorer_instrument::artifact::ArtifactTables::from_bytes(&artifact_bytes)?;
    let scorer = m1_d11_artifact_scorer_instrument::device::DeviceScorer::new(0, &tables)
        .map_err(|e| format!("device 0: {e}"))?;
    let context = driver_config.spec.context as usize;
    let mut all_records = serde_json::Map::new();
    let mut no_memory = Vec::new();
    let mut no_address = Vec::new();
    let mut sieve = Vec::new();
    let mut true_address = Vec::new();
    let mut random_address = Vec::new();
    let mut no_memory_tokens = Vec::new();
    let mut no_address_tokens = Vec::new();

    for row in &panel.rows {
        let mut per_arm = serde_json::Map::new();
        for arm in Arm::ALL {
            let started = Instant::now();
            let candidate_counts = match arm {
                Arm::Sieve => {
                    let query = form_sieve_query(&panel.lexicon, &row.prompt)?;
                    oracle.directory().sieve_candidate_counts(&query)
                }
                _ => Default::default(),
            };
            let mut generated_tokens = Vec::new();
            let mut fetches = 0usize;
            let mut emission_utf8 = true;
            let record: ArmRecord = if arm.is_model() {
                let mut prefix = row.prompt_tokens.clone();
                if prefix.is_empty() || prefix.len() >= context {
                    return Err(format!("{} prompt is outside the artifact context", row.id));
                }
                let mut generated = Vec::new();
                while generated.len() < 16 && prefix.len() < context {
                    let lowercase = tokenizer
                        .decode_bytes(&generated)
                        .iter()
                        .filter(|byte| byte.is_ascii_lowercase())
                        .count();
                    if lowercase >= 8 {
                        break;
                    }
                    let packed =
                        m1_d11_artifact_scorer_instrument::kernel::pack_rows(&[prefix.as_slice()])?;
                    let scores = scorer.score(&packed)?;
                    if scores.len() != 1 || scores[0].status != 0 {
                        return Err(format!("{} model scorer status is nonzero", row.id));
                    }
                    generated.push(scores[0].prediction);
                    prefix.push(scores[0].prediction);
                }
                let (decoded, valid_utf8) =
                    uor_r4_object_store::address_oracle::decode_emission_bytes(
                        &tokenizer.decode_bytes(&generated),
                    );
                emission_utf8 = valid_utf8;
                let normalized = normalize_emission(&decoded).unwrap_or(decoded);
                generated_tokens = generated;
                if arm == Arm::NoMemory {
                    no_memory_tokens.push(generated_tokens.clone());
                } else {
                    no_address_tokens.push(generated_tokens.clone());
                }
                score_model_arm(row, arm, Some(&normalized))?
            } else {
                let backend = CountingBackend {
                    store: oracle.store(),
                    fetches: AtomicUsize::new(0),
                };
                let outcome = match arm {
                    Arm::TrueAddress => oracle.directory().read_exact_from(
                        &backend,
                        oracle.store().limits(),
                        None,
                        &row.true_address()?,
                        row.version.into(),
                    ),
                    Arm::RandomAddress => oracle.directory().read_exact_from(
                        &backend,
                        oracle.store().limits(),
                        None,
                        &row.random_address()?,
                        uor_r4_object_store::address_read::VersionSelect::Current,
                    ),
                    Arm::Sieve => {
                        let query = form_sieve_query(&panel.lexicon, &row.prompt)?;
                        oracle.directory().read_sieve_from(
                            &backend,
                            oracle.store().limits(),
                            None,
                            &query,
                        )
                    }
                    Arm::NoMemory | Arm::NoAddress => unreachable!(),
                }
                .map_err(|e| format!("{} {arm:?}: {e:?}", row.id))?;
                fetches = backend.fetches.load(Ordering::SeqCst);
                score_store_read(row, arm, outcome, Some(&panel.lexicon))?
            };
            let elapsed_ns = started.elapsed().as_nanos() as u64;
            let slots = if arm == Arm::NoMemory {
                0
            } else {
                oracle.directory().len()
            };
            let emission_token_count = uor_r4_object_store::address_oracle::emission_token_count(
                Some(&tokenizer),
                &record,
                generated_tokens.len(),
            )?;
            let value = serde_json::json!({
                "record": record,
                "generated_tokens": generated_tokens,
                "emission_token_count": emission_token_count,
                "emission_utf8": emission_utf8,
                "elapsed_ns": elapsed_ns,
                "slots": slots,
                "backend_fetches": fetches,
                "candidate_counts": candidate_counts,
                "candidates": match arm {
                    Arm::TrueAddress | Arm::RandomAddress => 1,
                    Arm::Sieve => candidate_counts.active_admitted + candidate_counts.evicted_admitted,
                    Arm::NoMemory | Arm::NoAddress => 0,
                },
                "value_bytes": record.emitted_text.as_ref().map_or(0, String::len),
            });
            per_arm.insert(format!("{arm:?}"), value);
            match arm {
                Arm::NoMemory => no_memory.push(record),
                Arm::NoAddress => no_address.push(record),
                Arm::Sieve => sieve.push(record),
                Arm::TrueAddress => true_address.push(record),
                Arm::RandomAddress => random_address.push(record),
            }
        }
        all_records.insert(row.id.clone(), serde_json::Value::Object(per_arm));
    }

    let mut rates = serde_json::Map::new();
    for arm in Arm::ALL {
        let key = format!("{arm:?}");
        let mut tokens = 0u128;
        let mut rows = 0u128;
        let mut elapsed_ns = 0u128;
        for record in all_records.values() {
            let value = &record[&key];
            tokens += u128::from(
                value["emission_token_count"]
                    .as_u64()
                    .ok_or("missing emission_token_count")?,
            );
            rows += 1;
            elapsed_ns += u128::from(value["elapsed_ns"].as_u64().ok_or("missing elapsed_ns")?);
        }
        let divisor = elapsed_ns.max(1);
        rates.insert(
            key,
            serde_json::json!({
                "emission_tokens": tokens as u64,
                "context_rows": rows as u64,
                "elapsed_ns": elapsed_ns as u64,
                "decode_tokens_per_second_milli": (tokens * 1_000_000_000_000u128 / divisor) as u64,
                "context_rows_per_second_milli": (rows * 1_000_000_000_000u128 / divisor) as u64,
            }),
        );
    }

    let baseline_identical = baseline_records_identical(&no_memory, &no_address)?
        && no_memory_tokens == no_address_tokens;
    let random_holds = random_control_holds(&panel, &random_address)?;
    let hard = hard_gates(&panel, &true_address)?;
    let head = std::env::var(STAGE0_HEAD_ENV)
        .map_err(|_| format!("{STAGE0_HEAD_ENV} must be the full commit"))?;
    let executable = std::env::current_exe().map_err(|e| format!("current exe: {e}"))?;
    let display_driver = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "UNVERIFIED".into());
    let stage0_identity_bytes = std::fs::read(out.join("identity.json"))
        .map_err(|e| format!("read Stage 0 identity: {e}"))?;
    let stage0_identity_sha256 = sha256_hex(&stage0_identity_bytes);
    let stage0_identity: serde_json::Value = serde_json::from_slice(&stage0_identity_bytes)
        .map_err(|e| format!("parse Stage 0 identity: {e}"))?;
    let nonempty = |key: &str| {
        stage0_identity
            .get(key)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty())
    };
    let stage0_runtime_complete = [
        "cubin_sha256",
        "ptx_sha256",
        "driver_source_sha256",
        "UOR_NVCC_ARCH",
        "IMAGE_TARGET",
        "RESOURCES",
    ]
    .into_iter()
    .all(nonempty)
        && stage0_identity["compiled_identity"]
            .as_object()
            .is_some_and(|value| !value.is_empty())
        && stage0_identity["device_capability"]
            .as_array()
            .is_some_and(|value| value.len() == 2)
        && stage0_identity["cuda_toolkit_version"]["available"].as_bool() == Some(true);
    let serialized_store_bytes = oracle
        .store()
        .export_records()
        .map_err(|e| format!("serialize store: {e:?}"))?
        .len();
    let identity = serde_json::json!({
        "schema": "uor-r4.address-oracle-identity/1",
        "HEAD": head,
        "artifact_sha256": ARTIFACT_SHA256,
        "checkpoint_sha256": uor_r4_object_store::address_stage0::CHECKPOINT_SHA256,
        "tokenizer_sha256": TOKENIZER_SHA256,
        "panel_sha256": panel_sha256,
        "original_config_sha256": ORIGINAL_CONFIG_SHA256,
        "staged_input_config_sha256": STAGED_INPUT_CONFIG_SHA256,
        "normalized_config_sha256": normalized_config_sha256,
        "train_tokens_sha256": TRAIN_TOKENS_SHA256,
        "train_mask_sha256": TRAIN_MASK_SHA256,
        "dev_tokens_sha256": DEV_TOKENS_SHA256,
        "dev_mask_sha256": DEV_MASK_SHA256,
        "executable_sha256": sha256_hex(&std::fs::read(&executable).map_err(|e| format!("executable: {e}"))?),
        "cuda_driver_api": cuda_driver_api_version()?,
        "nvidia_display_driver": display_driver,
        "store_slots": oracle.directory().len(),
        "serialized_store_bytes": serialized_store_bytes,
        "resident_heap_bytes": "UNVERIFIED",
        "stage0_identity_sha256": stage0_identity_sha256,
        "stage0_runtime_complete": stage0_runtime_complete,
        "baseline_note": "Both arms are model-only controls. No-memory's slots=0 is a logical arm label; the process-level store remains populated for the store arms. No-address performs no store read. Byte identity checks determinism, not a store-presence effect.",
    });
    write_json(&out.join("oracle-identity.json"), &identity)?;
    let required_identity = [
        "HEAD",
        "artifact_sha256",
        "checkpoint_sha256",
        "tokenizer_sha256",
        "panel_sha256",
        "original_config_sha256",
        "staged_input_config_sha256",
        "normalized_config_sha256",
        "train_tokens_sha256",
        "train_mask_sha256",
        "dev_tokens_sha256",
        "dev_mask_sha256",
        "executable_sha256",
        "cuda_driver_api",
        "nvidia_display_driver",
        "store_slots",
        "serialized_store_bytes",
        "resident_heap_bytes",
        "stage0_identity_sha256",
        "stage0_runtime_complete",
        "baseline_note",
    ];
    let identity_complete = stage0_runtime_complete
        && required_identity
            .iter()
            .all(|key| identity.get(*key).is_some());
    let leak_free = prompt_leaks.is_empty() && train_leaks.is_empty() && dev_leaks.is_empty();
    let composed = uor_r4_object_store::address_oracle::composed_gate(
        hard,
        baseline_identical,
        random_holds,
        parity.parity_pass,
        leak_free,
        identity_complete,
    );
    let composed_pass = composed.passed();
    let mut families = serde_json::Map::new();
    for family in [
        uor_r4_object_store::address_oracle::ContrastPopulation::Current,
        uor_r4_object_store::address_oracle::ContrastPopulation::Previous,
        uor_r4_object_store::address_oracle::ContrastPopulation::Abstention,
    ] {
        let true_no = uor_r4_object_store::address_oracle::classify_family(
            &panel,
            family,
            &hard,
            &true_address,
            &no_memory,
        )?;
        let (true_sieve, sieve_no) =
            if family == uor_r4_object_store::address_oracle::ContrastPopulation::Previous {
                (
                    serde_json::Value::String("excluded_current_fallback".into()),
                    serde_json::Value::String("excluded_current_fallback".into()),
                )
            } else {
                (
                    serde_json::to_value(uor_r4_object_store::address_oracle::classify_family(
                        &panel,
                        family,
                        &hard,
                        &true_address,
                        &sieve,
                    )?)
                    .map_err(|e| e.to_string())?,
                    serde_json::to_value(uor_r4_object_store::address_oracle::classify_family(
                        &panel, family, &hard, &sieve, &no_memory,
                    )?)
                    .map_err(|e| e.to_string())?,
                )
            };
        families.insert(
            format!("{family:?}"),
            serde_json::json!({
                "true_vs_no_memory": true_no,
                "true_vs_sieve": true_sieve,
                "sieve_vs_no_memory": sieve_no,
            }),
        );
    }
    let report = serde_json::json!({
        "schema": "uor-r4.address-oracle-report/1",
        "HEAD": head,
        "panel_sha256": panel_sha256,
        "composed": composed,
        "composed_pass": composed_pass,
        "decision": if composed_pass { "ORACLE_INSTRUMENT_PASS" } else { "INSTRUMENT_FAIL" },
        "families": families,
        "rates": rates,
        "rows": all_records,
    });
    write_json(&out.join("oracle-report.json"), &report)?;
    println!(
        "address oracle: composed_pass={composed_pass}; outputs in {}",
        out.display()
    );
    Ok(())
}
