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
use uor_r4_training::{geometric_source_emission_view::SourceEmissionCompiler, sha256_file};
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
    for input in [
        &a.checkpoint,
        &a.trusted_binding,
        &a.compiled_panel,
        &a.old_fresh_spec,
    ] {
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
// Each pair shares query and every typed frame field; only source bytes change.
// These are fixed before tokenization and cannot be replaced after predictions.
const PAIRS: [(&str, &str, &str); 16] = [
    (
        "composition-order",
        "singer Brimfold dancer Louston",
        "Louston singer dancer Brimfold",
    ),
    (
        "composition-order",
        "dancer Louston singer Brimfold",
        "Brimfold Louston dancer singer",
    ),
    (
        "composition-order",
        "singer Brimfold singer Louston",
        "singer Louston singer Brimfold",
    ),
    (
        "composition-order",
        "dancer Brimfold dancer Louston",
        "dancer Louston dancer Brimfold",
    ),
    (
        "composition-repeat",
        "singer singer Brimfold Brimfold",
        "Brimfold Brimfold singer singer",
    ),
    (
        "composition-repeat",
        "dancer dancer Louston Louston",
        "Louston Louston dancer dancer",
    ),
    (
        "composition-repeat",
        "Brimfold Louston Louston Brimfold",
        "Louston Brimfold Brimfold Louston",
    ),
    (
        "composition-order",
        "singer dancer Brimfold Louston",
        "dancer singer Louston Brimfold",
    ),
    (
        "fresh-lexical-replacement",
        "Orvellan singer",
        "Orvellan dancer",
    ),
    (
        "fresh-lexical-replacement",
        "Talmeris Brimfold",
        "Talmeris Louston",
    ),
    (
        "fresh-lexical-order",
        "singer Zunavik dancer",
        "dancer Zunavik singer",
    ),
    (
        "fresh-lexical-repeat",
        "Pelthorin singer Pelthorin",
        "Pelthorin dancer Pelthorin",
    ),
    (
        "fresh-lexical-order",
        "Orvellan Talmeris",
        "Talmeris Orvellan",
    ),
    (
        "fresh-lexical-repeat",
        "Zunavik Zunavik Pelthorin",
        "Pelthorin Zunavik Zunavik",
    ),
    (
        "fresh-lexical-order",
        "Brimfold Orvellan Louston",
        "Louston Orvellan Brimfold",
    ),
    (
        "fresh-lexical-order",
        "Talmeris singer dancer Talmeris",
        "Talmeris dancer singer Talmeris",
    ),
];
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
    let mut source_familiar = 0usize;
    let mut context_familiar = 0usize;
    for (pair_index, (stratum, left, right)) in PAIRS.iter().enumerate() {
        let query = QUERIES[pair_index % 2];
        for (side, literal) in [("left", *left), ("right", *right)] {
            let id = format!("dependent-fresh-{:02}", packets.len());
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
            query_counts[pair_index % 2] += 1;
            packets.push(json!({"id":id,"record":record,"commit":commit,"scope":"m-world-v2","entity":entity,"relation":relation,"view":0,"status":"Found","original_source_ids":original,"query_ids":query_ids,"actual_prefix_ids":[]}));
            labels.push(json!({"id":id,"stratum":stratum,"pair_id":format!("dependent-pair-{pair_index:02}"),"side":side,"literal":literal,"query":query,"answers":answers,"target_ids_labels_only":target,"source_view":view,"token_support":{"unseen_emitted_source_ids_vs_training64":unseen_source,"unseen_emitted_source_ids_vs_training64_context":unseen_context,"unseen_query_ids_vs_training64_context":unseen_query,"target_copy_period_eos_alphabet_supported":true,"positive_native_target_mass":"NOT_RUN"}}));
        }
    }
    if packets.len() != 32 || query_counts != [16, 16] {
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
        json!({"schema":"uor-r4.geometric-dependent-fit-panel/1","status":"completed","predictions":"NOT_RUN","optimizer_updates":0,"cases":32,"source_literal_disjoint_training64_old32_old8_old16":true,"full_source_query_pairs_disjoint":true,"same_frame_within_every_pair":true,"query_counts":query_counts,"actual_BPE_familiar_source_rows":source_familiar,"actual_BPE_familiar_context_rows":context_familiar,"native_target_mass":"NOT_RUN; only public output-alphabet support checked","trusted_parent":parent,"checkpoint_manifest_sha256":sha256_file(&a.checkpoint.join("manifest.json"))?,"trusted_binding_sha256":sha256_file(&a.trusted_binding)?,"training_and_old32_panel_sha256":sha256_file(&a.compiled_panel)?,"old8_spec_sha256":sha256_file(&a.old_fresh_spec)?,"source_inputs_sha256":sha256_file(&a.out.join("source-inputs.json"))?,"labels_sha256":sha256_file(&a.out.join("labels.json"))?,"label_policy":"FrozenAnswers Current authored once from fixed literal intent; exact membership only; excluded from native read","scope":"32 prospective authored selected-source copy-transfer rows; no general prose/reasoning/chat/energy qualification","elapsed_seconds":start.elapsed().as_secs_f64()}),
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
