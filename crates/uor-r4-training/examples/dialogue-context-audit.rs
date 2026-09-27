//! Data-only reconstruction of the retained R1d response-window sampling stream.
//! No model loading, training, heldout token access or sampler adoption.
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{native_geometric::mmap_corpus::MmapCorpusReader, report_output};
use uor_r4_training::sha256_file;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const CONTEXT: usize = 256;
const BATCH: usize = 16;
const STEPS: usize = 2237;
const SEED: u64 = 20260926;
const FIT_SHA: &str = "a192916c17d8d124e9a32c21c3b31ef6b622890b148d37549eb219d11cd641fa";
const PREPARED_SHA: &str = "a66d52473cac24b28cc09a751ede7b680c40e7b56218e9074e246cf4fd1a99a5";
const HISTORICAL_SOURCE: &str = "5109861c93dee0e39d3aa189c325b3a13c4ededc";

fn require(ok: bool, message: &str) -> Result<()> {
    if !ok {
        return Err(io::Error::other(message).into());
    }
    Ok(())
}
fn integer(v: &Value, key: &str) -> Result<usize> {
    v.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| io::Error::other(format!("missing integer {key}")).into())
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::other(format!("missing string {key}")).into())
}
fn save(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Response {
    document: usize,
    start: usize,
    end: usize,
    source: usize,
}

#[derive(Default, Clone, Debug, Serialize)]
struct Counts {
    supervised_positions: u64,
    response_start_before_input_start: u64,
    response_start_at_input_start: u64,
    response_start_after_input_start: u64,
    response_start_at_or_before_input_start: u64,
    response_start_at_first_target: u64,
    document_start_before_input_start: u64,
    document_start_at_input_start: u64,
    document_start_after_input_start: u64,
    supervised_eos_positions: u64,
}
impl Counts {
    fn add(&mut self, run: Response, s: usize, a: usize, b: usize) {
        let lo = a.max(run.start);
        let hi = b.min(run.end);
        if lo >= hi {
            return;
        }
        let n = (hi - lo) as u64;
        self.supervised_positions += n;
        if run.start < s {
            self.response_start_before_input_start += n;
        } else if run.start == s {
            self.response_start_at_input_start += n;
        } else {
            self.response_start_after_input_start += n;
        }
        if run.start <= s {
            self.response_start_at_or_before_input_start += n;
        }
        if run.start == a {
            self.response_start_at_first_target += n;
        }
        if run.document < s {
            self.document_start_before_input_start += n;
        } else if run.document == s {
            self.document_start_at_input_start += n;
        } else {
            self.document_start_after_input_start += n;
        }
        if a <= run.end - 1 && run.end - 1 < b {
            self.supervised_eos_positions += 1;
        }
    }
}

#[derive(Default, Serialize)]
struct Population {
    documents: usize,
    response_runs: usize,
    supervised_tokens: usize,
    eos_tokens: usize,
    documents_total_ids_le_256: usize,
    documents_stored_tokens_le_257: usize,
    response_through_eos_from_document_start_total_ids_le_256: usize,
    response_through_eos_from_document_start_stored_tokens_le_257: usize,
}

fn finish_response(
    runs: &mut Vec<Response>,
    start: usize,
    end: usize,
    document: usize,
    source: usize,
    tokens: &[u16],
    population: &mut Population,
) -> Result<()> {
    require(
        start < end && tokens[end - 1] == 1,
        "response mask run must terminate at EOS",
    )?;
    require(
        !tokens[start..end - 1].contains(&1),
        "EOS inside response content",
    )?;
    runs.push(Response {
        document,
        start,
        end,
        source,
    });
    population.response_runs += 1;
    population.response_through_eos_from_document_start_total_ids_le_256 +=
        usize::from(end - document <= 256);
    population.response_through_eos_from_document_start_stored_tokens_le_257 +=
        usize::from(end - document <= 257);
    Ok(())
}

// Verbatim historical dialogue.rs splitmix and counter arithmetic. `step` is
// zero-based, whereas learning-curve rows record step+1.
fn splitmix(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d049bb133111eb);
    state ^ (state >> 31)
}
fn sampled_start(step: usize, lane: usize, tokens: usize) -> usize {
    let counter = SEED
        ^ (step as u64).wrapping_mul(0xd1342543de82ef95)
        ^ (lane as u64).wrapping_mul(0x9e3779b97f4a7c15);
    (splitmix(counter) % (tokens - CONTEXT) as u64) as usize
}

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    require(
        args.len() == 3,
        "usage: dialogue-context-audit LEGACY_FIT_ROOT PREPARED_MANIFEST NEW_REPORT_ROOT",
    )?;
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    require(
        source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()),
        "build with full UOR_BUILD_SOURCE_COMMIT",
    )?;
    report_output::claim(&args[2])?;
    let result = audit(&args, source);
    if let Err(error) = &result {
        save(
            &args[2].join("failed-attempt.json"),
            &json!({
                "status":"UNVERIFIED", "error":error.to_string(), "source_commit":source,
                "scope":"Incomplete data audit, not model-quality evidence"
            }),
        )?;
    }
    report_output::seal(&args[2])?;
    report_output::verify(&args[2])?;
    result?;
    println!(
        "{}",
        json!({"status":"DATA_AUDIT_COMPLETE","report_root":args[2]})
    );
    Ok(())
}

fn audit(args: &[PathBuf], source: &str) -> Result<()> {
    let started = Instant::now();
    let fit_root = &args[0];
    let out = &args[2];
    report_output::verify(fit_root)?;
    require(
        !fit_root.join("failed-attempt.json").exists(),
        "fit has failure marker",
    )?;
    let fit_path = fit_root.join("dialogue-fit.json");
    require(
        sha256_file(&fit_path)? == FIT_SHA,
        "retained R1d fit identity mismatch",
    )?;
    require(
        sha256_file(&args[1])? == PREPARED_SHA,
        "prepared manifest identity mismatch",
    )?;
    let fit: Value = serde_json::from_slice(&fs::read(&fit_path)?)?;
    // Deserialize only train metadata. No heldout token/mask/panel is accessed
    // and no new scoring is performed. Verifying the retained fit seal above
    // may hash its historical report/curve files without interpreting them.
    #[derive(serde::Deserialize)]
    struct TrainOnly {
        train: Value,
    }
    let prepared: TrainOnly = serde_json::from_slice(&fs::read(&args[1])?)?;
    let train = &prepared.train;
    let cfg = &fit["configuration"];
    require(
        fit["schema"] == "uor-r4.dialogue-fit-report/1"
            && fit["status"] == "FIT_COMPLETE"
            && fit["source_commit"] == HISTORICAL_SOURCE
            && integer(&fit, "steps_completed")? == STEPS
            && integer(cfg, "batch")? == BATCH
            && integer(cfg, "context")? == CONTEXT
            && integer(cfg, "data_seed")? as u64 == SEED
            && integer(&fit, "sampled_targets")? == STEPS * BATCH * CONTEXT,
        "R1d sampling configuration mismatch",
    )?;
    require(
        train["schema"] == "uor-r4-chat-corpus/v1"
            && train["split"] == "train"
            && train["mask_schema"] == "uor-r4-response-mask/u8/v1"
            && train["tokenizer"]["bos_id"] == 0
            && train["tokenizer"]["eos_id"] == 1
            && train["tokenizer"]["unk_id"] == 2
            && train["tokenizer"]["vocab_size"] == 4096
            && train["drops"]["special_token_occurrences"] == 0,
        "train boundary interpretation unsupported",
    )?;
    let root = args[1]
        .parent()
        .ok_or_else(|| io::Error::other("manifest has no parent"))?;
    let train_manifest_path = root.join("train/manifest.json");
    let train_manifest: Value = serde_json::from_slice(&fs::read(&train_manifest_path)?)?;
    require(
        train_manifest == *train,
        "top-level and train manifest differ",
    )?;
    let token_path = PathBuf::from(text(cfg, "train_tokens")?);
    let mask_path = PathBuf::from(text(cfg, "train_mask")?);
    require(
        fs::canonicalize(&token_path)? == fs::canonicalize(root.join("train/tokens.u16"))?
            && fs::canonicalize(&mask_path)?
                == fs::canonicalize(root.join("train/response_mask.u8"))?
            && fs::canonicalize(&token_path)?
                == fs::canonicalize(text(&fit["train_tokens"], "path")?)?,
        "train path identity mismatch",
    )?;
    let token_sha = sha256_file(&token_path)?;
    let mask_sha = sha256_file(&mask_path)?;
    require(
        token_sha == text(train, "tokens_sha256")?
            && token_sha == text(&fit["train_tokens"], "sha256")?
            && mask_sha == text(train, "mask_sha256")?,
        "train token/mask digest mismatch",
    )?;
    let reader = MmapCorpusReader::open(&token_path)?;
    let tokens = reader.as_slice();
    let masks = fs::read(&mask_path)?;
    require(
        cfg!(target_endian = "little"),
        "historical native u16 sampler requires little-endian host",
    )?;
    require(
        reader.vocab_size() == 4096
            && tokens.len() == integer(train, "tokens")?
            && tokens.len() == integer(&fit["train_tokens"], "tokens")?
            && tokens.len() > CONTEXT
            && masks.len() == tokens.len()
            && fs::metadata(&token_path)?.len() == integer(train, "tokens_bytes")? as u64
            && fs::metadata(&token_path)?.len() == 64 + tokens.len() as u64 * 2
            && masks.len() == integer(train, "mask_bytes")?,
        "train length/header mismatch",
    )?;
    let files = train["files"]
        .as_array()
        .ok_or_else(|| io::Error::other("train source metadata missing"))?;
    let mut source_ends = Vec::new();
    let mut source_metadata = Vec::new();
    let mut end = 0usize;
    for file in files {
        let start = end;
        end = end
            .checked_add(integer(file, "tokens")?)
            .ok_or_else(|| io::Error::other("source range overflow"))?;
        require(
            file["special_token_occurrences"] == 0 && end <= tokens.len() && end > start,
            "invalid source token range or literal special token",
        )?;
        require(
            tokens[start] == 0,
            "ordered source range must begin with document BOS",
        )?;
        source_ends.push(end);
        source_metadata
            .push(json!({"token_start":start,"token_end_exclusive":end,"manifest":file}));
    }
    require(end == tokens.len(), "source token totals mismatch")?;
    let mut population = Population::default();
    let mut source_populations: Vec<Population> =
        files.iter().map(|_| Population::default()).collect();
    let mut documents = Vec::new();
    let mut responses = Vec::new();
    let mut d = 0usize;
    let mut run = None;
    let mut source_idx = 0usize;
    for i in 0..tokens.len() {
        let token = tokens[i];
        require(
            token < 4096 && masks[i] <= 1,
            "out-of-vocabulary token or nonbinary mask",
        )?;
        if let Some(r) = run {
            if masks[i] == 0 {
                finish_response(&mut responses, r, i, d, source_idx, tokens, &mut population)?;
                let p = &mut source_populations[source_idx];
                p.response_runs += 1;
                p.response_through_eos_from_document_start_total_ids_le_256 +=
                    usize::from(i - d <= 256);
                p.response_through_eos_from_document_start_stored_tokens_le_257 +=
                    usize::from(i - d <= 257);
                run = None;
            }
        }
        if token == 0 {
            require(masks[i] == 0, "document BOS must not be supervised")?;
            if i > 0 {
                require(tokens[i - 1] == 1, "document must terminate with EOS")?;
                population.documents_total_ids_le_256 += usize::from(i - d <= 256);
                population.documents_stored_tokens_le_257 += usize::from(i - d <= 257);
                source_populations[source_idx].documents_total_ids_le_256 +=
                    usize::from(i - d <= 256);
                source_populations[source_idx].documents_stored_tokens_le_257 +=
                    usize::from(i - d <= 257);
            }
            while source_idx < source_ends.len() && i >= source_ends[source_idx] {
                source_idx += 1;
            }
            require(
                source_idx < source_ends.len(),
                "document outside source range",
            )?;
            d = i;
            documents.push(i);
            population.documents += 1;
            source_populations[source_idx].documents += 1;
        }
        if i == 0 {
            require(token == 0, "train corpus must begin with BOS")?;
        }
        if masks[i] == 1 {
            if run.is_none() {
                run = Some(i);
            }
            population.supervised_tokens += 1;
            source_populations[source_idx].supervised_tokens += 1;
        }
        if token == 1 {
            population.eos_tokens += 1;
            source_populations[source_idx].eos_tokens += 1;
            require(
                masks[i] == 1 || i + 1 == tokens.len() || tokens[i + 1] == 0,
                "unmasked EOS must terminate a document",
            )?;
        }
    }
    if let Some(r) = run {
        finish_response(
            &mut responses,
            r,
            tokens.len(),
            d,
            source_idx,
            tokens,
            &mut population,
        )?;
        let p = &mut source_populations[source_idx];
        p.response_runs += 1;
        p.response_through_eos_from_document_start_total_ids_le_256 +=
            usize::from(tokens.len() - d <= 256);
        p.response_through_eos_from_document_start_stored_tokens_le_257 +=
            usize::from(tokens.len() - d <= 257);
    }
    require(
        tokens.last() == Some(&1),
        "last document must terminate with EOS",
    )?;
    population.documents_total_ids_le_256 += usize::from(tokens.len() - d <= 256);
    population.documents_stored_tokens_le_257 += usize::from(tokens.len() - d <= 257);
    source_populations[source_idx].documents_total_ids_le_256 +=
        usize::from(tokens.len() - d <= 256);
    source_populations[source_idx].documents_stored_tokens_le_257 +=
        usize::from(tokens.len() - d <= 257);
    require(
        population.documents == integer(train, "rows_used")?
            && population.supervised_tokens == integer(train, "response_tokens")?
            && population.supervised_tokens == integer(&fit["train_tokens"], "response_tokens")?,
        "observed corpus population differs from manifests/fit",
    )?;
    for (file, observed) in files.iter().zip(&source_populations) {
        require(
            observed.documents == integer(file, "rows_used")?
                && observed.supervised_tokens == integer(file, "response_tokens")?,
            "source population differs from metadata",
        )?;
    }
    let curve_path = fit_root.join("learning-curve.jsonl");
    let curve = BufReader::new(fs::File::open(&curve_path)?);
    let row_path = out.join("sampling-reconstruction.jsonl");
    let mut writer = BufWriter::new(fs::File::create_new(&row_path)?);
    let mut total = Counts::default();
    let mut source_counts = vec![Counts::default(); files.len()];
    let mut steps = 0usize;
    let mut windows_cross_document = 0usize;
    let mut windows_with_eos = 0usize;
    let mut sampled_eos_positions = 0usize;
    let mut windows_with_no_supervision = 0usize;
    for line in curve.lines() {
        let row: Value = serde_json::from_str(&line?)?;
        require(
            steps < STEPS
                && integer(&row, "step")? == steps + 1
                && integer(&row, "sampled_targets")? == BATCH * CONTEXT,
            "curve step inventory mismatch",
        )?;
        let mut row_counts = Counts::default();
        let mut starts = Vec::with_capacity(BATCH);
        for lane in 0..BATCH {
            let s = sampled_start(steps, lane, tokens.len());
            let a = s + 1;
            let b = s + CONTEXT + 1;
            starts.push(s);
            let first_doc = documents.partition_point(|&v| v <= s);
            windows_cross_document += usize::from(documents.get(first_doc).is_some_and(|&v| v < b));
            let eos = tokens[a..b].iter().filter(|&&v| v == 1).count();
            sampled_eos_positions += eos;
            windows_with_eos += usize::from(eos > 0);
            let observed_mask = masks[a..b].iter().map(|&m| u64::from(m)).sum::<u64>();
            windows_with_no_supervision += usize::from(observed_mask == 0);
            let before = row_counts.supervised_positions;
            let first = responses.partition_point(|r| r.end <= a);
            for &response in &responses[first..] {
                if response.start >= b {
                    break;
                }
                row_counts.add(response, s, a, b);
                total.add(response, s, a, b);
                source_counts[response.source].add(response, s, a, b);
            }
            require(
                row_counts.supervised_positions - before == observed_mask,
                "interval/window mask reconstruction differs",
            )?;
        }
        let recorded = integer(&row, "supervised_targets")? as u64;
        let matches = row_counts.supervised_positions == recorded;
        serde_json::to_writer(
            &mut writer,
            &json!({"step":steps+1,"window_input_starts":starts,
            "recorded_supervised_targets":recorded,"reconstructed":row_counts,"matches_curve":matches}),
        )?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        require(
            matches,
            "reconstructed mask count differs from historical curve row",
        )?;
        steps += 1;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    require(steps == STEPS, "historical curve ended early")?;
    save(
        &out.join("result.json"),
        &json!({
            "schema":"uor-r4.dialogue-context-audit/1", "status":"DATA_AUDIT_COMPLETE",
            "source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "audit_source_sha256":hex::encode(Sha256::digest(include_bytes!("dialogue-context-audit.rs"))),
            "elapsed_seconds":started.elapsed().as_secs_f64(), "scope":"Train data only; zero model steps, optimizer updates or generated tokens. Missing document start does not establish missing immediate user request or a cause of weak output.",
            "identities":{"fit_root":fit_root,"fit_report_sha256":FIT_SHA,
                "fit_seal_sha256":sha256_file(&fit_root.join(report_output::MANIFEST_FILE))?,
                "historical_source_label":fit["source_commit"],"historical_executable_sha256":fit["executable_sha256"],
                "historical_parameter_identity_from_verified_fit":fit["final_parameter_artifact"]["sha256"],
                "historical_configuration":cfg,"curve_sha256":sha256_file(&curve_path)?,
                "historical_sampler_source":"https://github.com/UOR-Foundation/uor-r4/blob/d22e2b50da85e527efcd4f60acc1e82644194b91/crates/uor-r4-training/src/dialogue.rs#L163-L195",
                "prepared_manifest_path":args[1],"prepared_manifest_sha256":PREPARED_SHA,
                "train_manifest_sha256":sha256_file(&train_manifest_path)?,"train_tokens_path":token_path,
                "train_tokens_sha256":token_sha,"train_mask_path":mask_path,"train_mask_sha256":mask_sha,
                "tokenizer_manifest_identity_not_loaded":train["tokenizer"],
                "mask_binding_scope":"Prepared manifest and live digest bind mask retrospectively; the historical fit recorded token hashes but no mask hash. Every learning-curve denominator is reconstructed independently."},
            "definitions":{"s":"input start; inputs [s,s+256), targets [s+1,s+257)",
                "r":"first supervised token in a contiguous response-mask run [r,e); e-1 is its EOS",
                "d":"document BOS position containing that response",
                "position_counts":"All Counts fields count supervised target visits, except supervised_eos_positions which counts their EOS subset. r<=s overlaps r<s and r==s; r==s means first response token is observed input, not supervised target.",
                "cross_document_window":"A new document BOS lies in (s,s+257), considering combined input/target span.",
                "capacity_counts":"<=256 counts all stored IDs from BOS through complete document/response EOS. <=257 separately permits a final EOS target after 256 training input positions; it does not fit a 256-total-ID generation horizon."},
            "sampling":{"updates":steps,"batch":BATCH,"training_positions":CONTEXT,"data_seed":SEED,
                "windows":steps*BATCH,"sampled_target_positions":steps*BATCH*CONTEXT,
                "all_curve_denominators_match":true,"counts":total,
                "windows_cross_document_combined_span":windows_cross_document,"windows_with_target_eos":windows_with_eos,
                "all_target_eos_positions":sampled_eos_positions,"windows_without_supervised_targets":windows_with_no_supervision},
            "corpus_population":population,"sources":source_metadata,"source_populations":source_populations,
            "sampled_supervision_by_response_source":source_counts,
            "rowwise_reconstruction":{"file":"sampling-reconstruction.jsonl","sha256":sha256_file(&row_path)?},
            "decision":"Descriptive context-availability evidence only; no sampler, objective, fit budget or model-quality decision is adopted."
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_start_at_input_start_is_observed_but_not_targeted() {
        let mut c = Counts::default();
        c.add(
            Response {
                document: 0,
                start: 10,
                end: 15,
                source: 0,
            },
            10,
            11,
            267,
        );
        assert_eq!(c.supervised_positions, 4);
        assert_eq!(c.response_start_before_input_start, 0);
        assert_eq!(c.response_start_at_input_start, 4);
        assert_eq!(c.response_start_at_or_before_input_start, 4);
        assert_eq!(c.supervised_eos_positions, 1);
    }
    #[test]
    fn first_response_target_is_not_a_missing_response_start() {
        let mut c = Counts::default();
        c.add(
            Response {
                document: 0,
                start: 11,
                end: 15,
                source: 0,
            },
            10,
            11,
            267,
        );
        assert_eq!(c.supervised_positions, 4);
        assert_eq!(c.response_start_at_first_target, 4);
        assert_eq!(c.response_start_at_or_before_input_start, 0);
        assert_eq!(c.document_start_before_input_start, 4);
    }
    #[test]
    fn later_document_and_half_open_response_boundary_are_distinct() {
        let mut c = Counts::default();
        c.add(
            Response {
                document: 20,
                start: 25,
                end: 30,
                source: 0,
            },
            10,
            11,
            28,
        );
        assert_eq!(c.supervised_positions, 3);
        assert_eq!(c.document_start_after_input_start, 3);
        assert_eq!(c.supervised_eos_positions, 0);
        c.add(
            Response {
                document: 0,
                start: 3,
                end: 11,
                source: 0,
            },
            10,
            11,
            28,
        );
        assert_eq!(c.supervised_positions, 3);
    }
}
