//! Actual-bundle token/session witness, not a dialogue-quality evaluation.
//! Build with UOR_BUILD_SOURCE_COMMIT, then pass BUNDLE_ROOT NEW_REPORT_ROOT.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io, path::PathBuf, time::Instant};
use uor_r4_integer::{
    generation::{Decision, Generation, Selection},
    report_output, sha256_file, Bundle, ReadMode, SamplePolicy, Sampler, PROBABILITY_TOTAL,
};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

const RETAINED_TOKENIZER: &str =
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc";
// Recorded by the independent tokenizer witness accompanying PR1402.
const HI_PREFIX: &[u32] = &[0, 55, 2728, 28, 223, 1094, 201, 35, 560, 652, 714, 28, 223];
const MAX_NEW_TOKENS: usize = 8;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: dialogue-token-witness BUNDLE_ROOT NEW_REPORT_ROOT",
        )
        .into());
    }
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    if source_commit.len() != 40 || !source_commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "build witness with UOR_BUILD_SOURCE_COMMIT set to its full source commit",
        )
        .into());
    }
    let bundle_root = PathBuf::from(&args[0]);
    let report_root = PathBuf::from(&args[1]);
    report_output::claim(&report_root)?;
    let clock = Instant::now();
    let bundle = Bundle::load(&bundle_root)?;
    let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer())?;
    let messages = [Message {
        role: "user",
        content: "Hi",
    }];
    let prefix = protocol
        .bind(bundle.tokenizer())?
        .encode_assistant_prefix(&messages);
    let reference_applies = bundle.tokenizer().address() == RETAINED_TOKENIZER;
    let reference_equal = reference_applies && prefix.tokens == HI_PREFIX;
    let mut comparisons = Vec::new();
    let mut all_equal = reference_equal;
    for selection in [
        Selection::Greedy,
        Selection::Categorical {
            top_k: 40,
            seed: 20260926,
        },
    ] {
        let adapted = bundle.generate_dialogue(
            &protocol,
            &messages,
            MAX_NEW_TOKENS,
            selection,
            ReadMode::Enabled,
            false,
        )?;
        let mut token_session = bundle.text_session(ReadMode::Enabled)?;
        token_session.append_tokens(&prefix.tokens[1..])?;
        let appended = token_session.generate(MAX_NEW_TOKENS, selection, false)?;

        // Replay every emitted decision independently of TextSession. The count
        // is the adapter's emitted count: this checks causal ingestion and
        // predictions, not an independent implementation of its stopping policy.
        let direct = direct_decisions(
            &bundle,
            &prefix.tokens,
            selection,
            adapted.generation.generated_token_ids.len(),
        )?;
        let append_equal = observables(&adapted.generation) == observables(&appended);
        let direct_equal = direct["generated_token_ids"]
            == json!(adapted.generation.generated_token_ids)
            && direct["decisions"] == serde_json::to_value(&adapted.generation.decisions)?
            && direct["sampler_state_after"] == json!(adapted.generation.sampler_state_after);
        let prompt_equal = adapted.generation.prompt_token_ids == prefix.tokens;
        all_equal &= append_equal && direct_equal && prompt_equal;
        comparisons.push(json!({
            "selection": selection,
            "adapter": adapted,
            "append_tokens": appended,
            "direct_integer_steps": direct,
            "exact_prompt_ids_equal": prompt_equal,
            "append_tokens_observables_equal": append_equal,
            "direct_prediction_decisions_equal": direct_equal
        }));
    }
    let receipt = json!({
        "schema": "uor-r4.dialogue-token-witness/1",
        "source_commit": source_commit,
        "generation_rs_sha256": hex::encode(Sha256::digest(include_bytes!("../src/generation.rs"))),
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "bundle_root": bundle_root,
        "bundle_sha256": bundle.identity(),
        "model_config": bundle.model().config(),
        "tokenizer_cid": bundle.tokenizer().address(),
        "protocol": protocol,
        "protocol_identity": protocol.identity()?,
        "binding_scope": "evaluation input and EOS semantics only; unchanged bundle has no adopted dialogue training contract",
        "messages": [{"role":"user","content":"Hi"}],
        "encoded_prefix_ids": prefix.tokens,
        "retained_tokenizer_reference_applies": reference_applies,
        "retained_independent_prefix_ids": HI_PREFIX,
        "retained_independent_prefix_equal": reference_equal,
        "max_new_tokens": MAX_NEW_TOKENS,
        "comparisons": comparisons,
        "all_comparisons_equal": all_equal,
        "whole_witness_nanoseconds": clock.elapsed().as_nanos(),
        "claim": "Exact input and per-prediction transport witness only; no chat/prose/geometry/speed/energy qualification"
    });
    let mut file = fs::File::create_new(report_root.join("result.json"))?;
    serde_json::to_writer_pretty(&mut file, &receipt)?;
    report_output::seal(&report_root)?;
    report_output::verify(&report_root)?;
    println!(
        "{}",
        json!({"report_root": report_root, "all_comparisons_equal": all_equal})
    );
    if !all_equal {
        return Err(
            io::Error::other("sealed dialogue token witness differs; inspect result.json").into(),
        );
    }
    Ok(())
}

fn observables(generation: &Generation) -> Value {
    json!({
        "prompt_token_ids": generation.prompt_token_ids,
        "generated_token_ids": generation.generated_token_ids,
        "response_text": generation.response_text,
        "raw_decoded": generation.raw_decoded,
        "utf8_decodable": generation.utf8_decodable,
        "stop": generation.stop,
        "decisions": generation.decisions,
        "sampler_state_after": generation.sampler_state_after,
        "session_tokens_including_pending": generation.session_tokens_including_pending,
        "context_capacity": generation.context_capacity
    })
}

fn direct_decisions(
    bundle: &Bundle,
    prefix: &[u32],
    selection: Selection,
    count: usize,
) -> Result<Value, Box<dyn Error>> {
    let clock = Instant::now();
    let mut session = bundle.model().new_session();
    let mut last = None;
    for &token in prefix {
        last = Some(
            bundle
                .model()
                .step(&mut session, token, ReadMode::Enabled)?,
        );
    }
    let mut step = last.ok_or_else(|| io::Error::other("empty direct prefix"))?;
    let (policy, seed) = match selection {
        Selection::Greedy => (SamplePolicy::Greedy, 0),
        Selection::Categorical { top_k, seed } => (SamplePolicy::Categorical { top_k }, seed),
    };
    let mut sampler = Sampler::new(seed);
    let mut tokens = Vec::new();
    let mut decisions = Vec::new();
    for position in 0..count {
        let selected = sampler.select(&step.probabilities, policy)?;
        let mut hash = Sha256::new();
        for mass in &step.probabilities {
            hash.update(mass.to_le_bytes());
        }
        decisions.push(Decision {
            selected_token: selected as u32,
            probability_q48: step.probabilities[selected],
            probability_sum_q48: PROBABILITY_TOTAL,
            probability_sha256_le_u64: hex::encode(hash.finalize()),
            exposed_causal_slots: step.read_masses.len(),
            no_read_mass_q48: step.no_read_mass,
        });
        tokens.push(selected as u32);
        if position + 1 < count {
            step = bundle
                .model()
                .step(&mut session, selected as u32, ReadMode::Enabled)?;
        }
    }
    Ok(json!({
        "generated_token_ids": tokens,
        "raw_decoded": bundle.tokenizer().decode(&tokens),
        "decisions": decisions,
        "sampler_state_after": sampler.state(),
        "incremental_step_calls": session.len(),
        "model_and_sampling_nanoseconds": clock.elapsed().as_nanos()
    }))
}
