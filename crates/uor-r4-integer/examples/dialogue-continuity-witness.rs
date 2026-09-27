//! One actual-bundle two-turn exact-history witness, not a quality evaluation.
//! Build with UOR_BUILD_SOURCE_COMMIT, then pass BUNDLE_ROOT NEW_REPORT_ROOT.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io, path::PathBuf, time::Instant};
use uor_r4_integer::{
    generation::{ConversationRequest, Decision, Selection, Stop, TurnClosure},
    report_output, sha256_file, Bundle, ReadMode, SamplePolicy, Sampler, PROBABILITY_TOTAL,
};
use uor_r4_tokenizer::dialogue::DialogueProtocol;

const MAX_NEW_TOKENS: usize = 4;
const SEED: u64 = 20260926;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: dialogue-continuity-witness BUNDLE_ROOT NEW_REPORT_ROOT",
        )
        .into());
    }
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    if source_commit.len() != 40 || !source_commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "build with UOR_BUILD_SOURCE_COMMIT set to full source commit",
        )
        .into());
    }
    let bundle_root = PathBuf::from(&args[0]);
    let report_root = PathBuf::from(&args[1]);
    report_output::claim(&report_root)?;
    let result = (|| -> Result<bool, Box<dyn Error>> {
        let clock = Instant::now();
        let bundle = Bundle::load(&bundle_root)?;
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer())?;
        let mut conversation =
            bundle.dialogue_conversation(&protocol, &[], SEED, ReadMode::Enabled)?;
        let mut exact_history = vec![protocol.bos_id];
        let mut expected_rng = Sampler::new(SEED).state();
        let mut previous_stop = None;
        let mut comparisons = Vec::new();
        let mut all_equal = true;
        for (index, user) in ["Hi", "Go on."].iter().enumerate() {
            // Independent framing, with the previous response's actual IDs retained.
            // Deliberately do not call DialogueEncoder or decode/re-encode history.
            let mut suffix = Vec::new();
            if index != 0 {
                if previous_stop != Some(Stop::Eos) {
                    suffix.push(protocol.eos_id);
                }
                suffix.extend(bundle.tokenizer().encode("\n"));
            }
            for segment in ["User: ", *user, "\n", "Assistant: "] {
                suffix.extend(bundle.tokenizer().encode(segment));
            }
            exact_history.extend_from_slice(&suffix);
            let turn = conversation.respond(ConversationRequest {
                user: *user,
                max_new_tokens: MAX_NEW_TOKENS,
                policy: SamplePolicy::Categorical { top_k: 40 },
                first_sentence: false,
                closure: TurnClosure::InterruptAssistant,
            })?;
            let generation = &turn.dialogue.generation;
            let direct = direct_decisions(
                &bundle,
                &exact_history,
                Selection::Categorical {
                    top_k: 40,
                    seed: expected_rng,
                },
                generation.generated_token_ids.len(),
            )?;
            let prompt_equal = generation.prompt_token_ids == exact_history;
            let direct_equal = direct["generated_token_ids"]
                == json!(generation.generated_token_ids)
                && direct["decisions"] == serde_json::to_value(&generation.decisions)?
                && direct["sampler_state_after"] == json!(generation.sampler_state_after);
            let expected_steps = exact_history.len() + generation.generated_token_ids.len() - 1;
            let no_replay = conversation.step_calls() == expected_steps;
            let counted_pending = conversation.len() == expected_steps + 1;
            let appended_equal = turn.appended_token_ids
                == if index == 0 {
                    exact_history.clone()
                } else {
                    suffix.clone()
                };
            let rng_equal = conversation.sampler_state()? == generation.sampler_state_after;
            let bytes_equal = turn.raw_generated_bytes
                == bundle
                    .tokenizer()
                    .decode_bytes(&generation.generated_token_ids);
            all_equal &= prompt_equal
                && direct_equal
                && no_replay
                && counted_pending
                && appended_equal
                && rng_equal
                && bytes_equal;
            expected_rng = generation.sampler_state_after;
            previous_stop = Some(generation.stop);
            exact_history.extend_from_slice(&generation.generated_token_ids);
            comparisons.push(json!({
                "turn": turn, "direct_integer_steps": direct,
                "independent_suffix_ids": suffix,
                "exact_history_equal": prompt_equal,
                "all_prediction_decisions_equal": direct_equal,
                "no_prefix_replay": no_replay,
                "pending_counted_once": counted_pending,
                "appended_input_ids_equal": appended_equal,
                "sampler_cursor_equal": rng_equal,
                "raw_bytes_equal": bytes_equal,
                "expected_total_step_calls": expected_steps,
                "actual_total_step_calls": conversation.step_calls(),
                "conversation_rng_after": conversation.sampler_state()?,
            }));
        }
        let receipt = json!({
            "schema": "uor-r4.dialogue-continuity-witness/1",
            "source_commit": source_commit,
            "source_sha256": {
                "conversation.rs": hex::encode(Sha256::digest(include_bytes!("../src/generation/conversation.rs"))),
                "generation.rs": hex::encode(Sha256::digest(include_bytes!("../src/generation.rs"))),
                "dialogue.rs": hex::encode(Sha256::digest(include_bytes!("../../uor-r4-tokenizer/src/dialogue.rs"))),
                "witness.rs": hex::encode(Sha256::digest(include_bytes!("dialogue-continuity-witness.rs"))),
            },
            "executable_sha256": sha256_file(&std::env::current_exe()?)?,
            "bundle_root": bundle_root, "bundle_sha256": bundle.identity(),
            "model_config": bundle.model().config(),
            "tokenizer_cid": bundle.tokenizer().address(),
            "protocol": protocol, "protocol_identity": protocol.identity()?,
            "max_new_tokens_per_turn": MAX_NEW_TOKENS,
            "seed": SEED, "comparisons": comparisons,
            "all_comparisons_equal": all_equal,
            "whole_witness_nanoseconds": clock.elapsed().as_nanos(),
            "binding_scope": "unchanged bundle, evaluation framing only; no adopted dialogue training contract",
            "claim": "Exact two-turn input, state and prediction transport only; no chat/prose/geometry/speed/energy qualification"
        });
        let mut file = fs::File::create_new(report_root.join("result.json"))?;
        serde_json::to_writer_pretty(&mut file, &receipt)?;
        Ok(all_equal)
    })();
    if let Err(error) = &result {
        let mut file = fs::File::create_new(report_root.join("failed-attempt.json"))?;
        serde_json::to_writer_pretty(
            &mut file,
            &json!({
                "source_commit": source_commit,
                "status": "EXECUTION_FAILED",
                "error": error.to_string(),
                "scope": "No completed transport or capability result"
            }),
        )?;
    }
    report_output::seal(&report_root)?;
    report_output::verify(&report_root)?;
    let all_equal = result?;
    println!(
        "{}",
        json!({"report_root": report_root, "all_comparisons_equal": all_equal})
    );
    if !all_equal {
        return Err(
            io::Error::other("sealed continuity witness differs; inspect result.json").into(),
        );
    }
    Ok(())
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
