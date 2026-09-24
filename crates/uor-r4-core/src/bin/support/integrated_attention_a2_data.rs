//! Exposed A2 development curriculum: exact pinned repository prose/Rust plus
//! balanced raw-text correction worlds. Offline source anchors never enter Session.
use std::collections::HashMap;
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::training::TrainingEpisode;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

#[path = "integrated_attention_data.rs"]
mod inherited_a1;

const VOCAB: u32 = 4096;
const MAX_SOURCE_BYTES: usize = 1 << 20;
const MAX_NATURAL_FIT_TOKENS_PER_SOURCE: usize = 1024;
const MAX_NATURAL_DEV_TOKENS_PER_SOURCE: usize = 768;
const MAX_EPISODE_TOKENS: usize = 512;
const MAX_FIT_TOKENS: usize = 150_000;
const MAX_DEV_TOKENS: usize = 16_000;
const MIN_CORRECTION_LAG: usize = 17;
const SOURCE_SNAPSHOT: &str = "b020f34a09816ba235984cedde1c50f4c8e9cae2";
const TOKENIZER_SHA256: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";

// Each line is path|SHA-256 from the committed source snapshot above. The fit
// source families exclude the dev r4_zoology/serving docs and graph-runtime crate.
const FIT_SOURCES: &str = r#"
docs/CONFIGURATION.md|b35f5a2bbb8b5c0349df1f6f29a9b0f6a3771199456d8dd54a3836fe4e6c7eab
docs/RELEASE_PIPELINE.md|14f9bf9eed2560fadc74a8474679c6c6315a357e71de279118a7958530535295
docs/associative_ordered_route_summaries_a1r_967.md|0b9d6625b628c3f97c47d00509c2da025c1940448a661e739f49b501692bfb56
docs/bounded_semantic_transitions_spec_843.md|a0da527c675cee05ff0c4bc91ab01370c6f17fe83af6feb9297fd0ee2e742ae4
docs/capacity_scaling_432.md|68309a58746071903655ce0e8b201bd9360b3c26551785debfed398022b07104
docs/compiler_memory_budget.md|fc0541462077ba829802fcf8f8990a6c09e76cc1e946dafe869e8392923860d7
docs/compositional_planning_certification_spec_846.md|ffabb86d6fb2e222a3d98ceffd3d4fd4191c81b89f139d9a08fad65adabb4857
docs/construction_causal_return_attention_983.md|4260d99adfaa9e12bd4f31026e296e2609448fdcaee2e987a1c6d3b38c5b5039
docs/corpus_induced_document_spin_placement_973.md|ea286dddac74968018192d17960be96a8b94ec5adca9e360b4224a081352afa3
docs/d2_canonical_compile.md|1e6c70c25cc5f31ba7922f7962e987016a7000abcfa0d64d1dbb87b017366e77
docs/dormant_reassessment_786.md|93470ecb3c53c5ac67c8f4b34cad1ae8d8caed5c982fba1605ed10899c36a7b8
docs/formal_vocabulary.md|8b2aa90a1cf56a3c6a5dc43ba2716ff40882f6f4fdee4928d02cd226314cfdda
docs/geometric_causal_decoder_plan.md|433476691eed675db07223935f7b1324e531a50a95bcf9dea804923c1587ccb1
docs/geometric_intelligence_programme.md|fa08272cc145169237ca50f7abadd25c5c66fddde07db7eacb3344d286becdff
docs/gnaf_import_provenance.md|50742ba69c080c4bd45c720c7d1f37b1b2fde981ec299e8a3892f71bdfb9c39f
docs/helm_d_learned_manifold_r4_construction_973.md|bca40cfa295531686d91353e54306adbfd31dc49b41ebba190ae56be6900922f
docs/hologram_formal_analysis_direction.md|49e37af44ed27d52ee5254874b3393220a5fc994da253cadddf226b34f1dca7f
docs/inference_contract.md|8856b1381a7e6341b06e09c397da16cbc1371aaf86fbc3de56b2582f5fe58194
docs/local_geometric_attention_969.md|deff51c967f5fa645af87724d9929a013f6cdfa693d39f2053e159c56b1f22fd
docs/multi_resonance_attention_sieve_audit_973.md|1bc474c25d6d1c81db996d51c8b523b308b610ad5a0fa2057d3f514a75bf4a5b
docs/native_geometric_addressed_attention_causal_credit_973.md|fead194816083338f0697b16f9f978b6a0ff4910c6979c1510ea31a86a68a05a
docs/native_geometric_addressed_attention_primitives_973.md|61f98ff0a4362ff95774cfe8793d8bdaaf39456b2d4c188cad5bb5aa88fee411
docs/native_geometric_attention_reassessment_973.md|e8bc2d777affaad658058e82e6f6919419900361a08c0769aecf739b34d05eec
docs/native_geometric_context_augmentation_973.md|ceb479f47096a5a6262c68375d5e627620673933d015b4bec44bfa570c10b6b5
docs/native_geometric_correspondence_973.md|1a26c8b81cb00c66b12fd2dbd57dd9fcc02d25f36b2fc6b5a6b88d1829243479
docs/native_geometric_current_query_1140.md|7a3ade699ea9c7cbd23e3ee9193f5917d6216573f2a4d89f9489d5530de7aabc
docs/native_geometric_decoder_validation_973.md|ad432de80538289419b50eb0f04494743c9b13a697e3dd3fe55f02cab1d89228
docs/native_geometric_direction_review_973.md|996f4c1e35f985631a6393a80246f1a18937562cf1487eb18252b4f545701a12
docs/native_geometric_hamming_policy_973.md|09b2bd9c29e033f88d1a403e30c0235932342d987cfeb5c3d09be20caa849999
docs/native_geometric_historical_query_context_973.md|9dd936d4ee66bf07a6e5f8ad0d6cce03c7e7766dbc5f36ac4db6615024e49825
docs/native_geometric_instruction_binding_1140.md|5465ddd7512b9dfcecf43860033beffd6aef18eec9790c6b01791eaece885ff0
docs/native_geometric_language_credit_973.md|9c8aaeb763a835c3142828e1c67c49be027c178dfd1269aba8add408486b0ca3
docs/native_geometric_language_span_973.md|fcb83524a752c102068d2f760e2ae2e9870786ea5a7d04878ddd40e61d1ae326
docs/native_geometric_literal_admission_1139.md|14c2cd6464b7594926c6496671b7cd6d949ed1e66edf2eed5e94f7fbe3e9798f
docs/native_geometric_minimal_head_973.md|f51008bbae15085e78006f2bd5c44cbb08ed4bf14467c313431428fe1c076f66
docs/native_geometric_occurrence_973.md|f91a15c63a6ea48edba7bc831e03f205fc423fb7f5063a0d69f49778b45e4b10
docs/native_geometric_operation_transition_1140.md|cf863b2e1bc46d25bdfad6bc8abaf146277242d1220d2fb5ff055882921c98bb
docs/native_geometric_phrase_update_973.md|1415a4c1faf7da5fe861510dfeeb5c6da2f86db3d48246614f458c09ba211198
docs/native_geometric_reader_scope_repair_973.md|5bc5d928f32f8ef90c419db3d21ca2679c3de331f8bd3b4938a6e356a3938147
docs/native_geometric_relation_admission_1139.md|c66883318aea2077c7d7956bb4eaadd71e216656664edca1192b5a7ad98afc0e
docs/native_geometric_relational_attention_973.md|f468fc828b95f6b98277c02cbac60340c1febb79b8f60f6241f598d2e457727e
docs/native_geometric_resumable_memory_973.md|661c24ae6c1779124051bab243abf8b453649f235ec2f6035f3030d34cf4d78f
docs/native_geometric_role_read_1137.md|776fcf4aac0a72c9f3ad71dde953efa8718a7ac9961b8983307310dd468fe601
docs/native_geometric_source_noread_1139.md|cee3080b4451af3ed291ce3ca1a346badd0c000de661090b2b57a42ab654b51d
docs/native_geometric_span_context_1139.md|f452b8b08ea108c0621434cbdd93d952f988704d5dbebf73cb0dedfd764604b4
docs/native_geometric_tree_state_pair_973.md|1e7142d35c40a23ab5ba1ebcbf9ad2e0a41bcf2d71119f16b249a6349ef8cfcb
docs/native_geometric_typed_routing_1139.md|f0fd86edbb70607719b956cec3a53062690a43dc9abe2a923c35af6b7fd7c8f8
docs/native_geometric_word_copy_973.md|938dda12cffde520f941881e90e90323c99b26f5afb09ceccbe59cc04cc2dc3c
docs/native_geometric_writer_binding_1139.md|2e5d2379d154b65eb63ed55413da6fa652aed05c8444e66621cf1fab1e2bfd15
docs/normative_r4g1_quality_933.md|a63a96a633940fe4940e066aabe07f09aee7edf9b78887e9f267f9d8d7862e05
docs/paragraph_entity_spin_path_attention_973.md|e25af4a9ed0487089c201eb8108fbd243a367ccb4ce61254ea22512777646dd7
docs/prime_route_attention_qualification_958.md|b54891cae9b692794373747e52124267ab39fc0c9f04583e4c06076a46dedb68
docs/probability_metadata_127.md|604a27eedd824c007e3ee65f30eba71eeb271aa4f039d8b21d4e2352964c7a70
docs/prompt_state_spec_835.md|8ed9e5a6821a0e732d306cb80046408cad54355abdb50f73d8562464b17222a8
docs/r4_direct_retained_readout_prompt_capacity_973.md|b59ae2a7fae442ee8b14a529b416c7fc2336ddc8b9bfecab78ab2e0dda78aa8f
docs/r4_group_addressed_retention_973.md|bd4193ccde30932afe3aa3cea849ad08c55bdd432248acfb075f8763e4607365
docs/r4_isolated_runtime_readiness_1096.md|952655fa2af59ac2e94842aa4d61a4f44b400b85b64f5405e736f2b83a5a12f5
docs/r4_isolated_runtime_readiness_1096_sources.md|d00e858994e5b717697345418928abdf471406a855256764a2545941c4dccef4
docs/r4_native_bridge_1102.md|c3accdb59267a11078f63d597b73e2c6d56dd3bf55b0c8e9e970f4fbe04dfdaf
docs/r4_native_bridge_1102_input_audit.md|85295b6f4d483b781fa5a491752afd403eb4280aa1bbabf0d6a6dc35829a3117
docs/r4_native_reference_1086_review.md|05fcee5c9ee31713c6f678036703f18b1af13676dd3c7641e2af738eb0f82e63
docs/r4_predictive_block_delta_binding_prompt_capacity_973.md|c0c5eb26e3968243baa4ea7adce519c08865c4b06e5f5140bfc24da1e4985605
docs/r4_retained_assembly_1094_delivery_review.md|f49b5011424ef887475b866f299820c2de6ff1b7545d1f88a4e2e8c5a61225e1
docs/r4_retained_comparison_1094_delivery_review.md|3c3edddf4d8a51803e59a605b02ff3fca7d54e2a3c76e45e3fe7f0cf5cadd49f
docs/r4_role_tagged_associative_curriculum_1045.md|c0dfe26e00d061a8182524d92c9287dd9cf3ba13e9c51998749c286b477d8845
docs/r4_softmax_end_to_end_attention_1014.md|753d49a12dba09bca5b85586b66cb536dd246cfded9b512b80b2f5a5f7e888cb
docs/r4_softmax_quality_capacity_continuation_1017.md|ac2c32fc173703b0314631650affdecd4efdb54ca8b1a844b67896adce2069a0
docs/r4_softmax_trace_state_student_1011.md|cf9f5caaac0ae37aa11354f77e502b3f451c213f522b40de3be62ecd41b88a5d
docs/r4_text_clause_adapter_1094_delivery_review.md|741f66ab7bee6462e8930eadd080c55a47290cb334046d17e92dd7f757ca1cba
docs/r4_text_clause_preparation_1094_review.md|5cb3db7196c968b62049190e35856b3ce9af5be34ef2ca2aad2b24adc2306694
docs/r4_workbench_candidate_1107.md|4c32f0041941dcc3f0c5a72600656f0508c35ee172a35f8fab881fe0006f44c8
docs/recursive_geometric_attention_a1_952.md|87ef3290dbaa7abfe5d44e759807315ac04ecd23ce67b69ca178605a3f7fd0f2
crates/uor-r4-core/src/answer_oracle.rs|b6c3e2be264fe2ea8f49107d455aa8ca05219fb1e57e262f9f24f29afcdfd20b
crates/uor-r4-core/src/learned_reference/adapter.rs|90caf4d8873d754a3266f664eb284f52bb0cae118e96b4dc890d6139b9ea598e
crates/uor-r4-core/src/native_geometric/adaptive_attention/mod.rs|75e3f83f000125e5594d6b20250264a867cfb3ea41375e3b2fb1851a8fe9c554
crates/uor-r4-core/src/native_geometric/addressed_attention/circuit.rs|8c714ab5457e36f64f5523fa1f691c6bd2852e94ab1b145698d23a57994078df
crates/uor-r4-core/src/native_geometric/addressed_attention/pilot.rs|392fba36239c2d5e7ca10d2948605457c86c4b6b7041308ceab940aad6f32533
crates/uor-r4-core/src/native_geometric/anchors.rs|eb653f80cb3160ad083cc16d10952218454bbaefeb9bdd02b38e92c34a32ba8a
crates/uor-r4-core/src/native_geometric/dependent_attention/learning.rs|4ebb8a1c7d7254354170957082713f93c793876ebee16aa85349354b115e7f27
crates/uor-r4-core/src/native_geometric/dependent_language/correspondence_learning.rs|34771d35a31ab19e44ee8c668b300cb396d81ef83b20269f41d8e6462f040056
crates/uor-r4-core/src/native_geometric/dependent_language/occurrence.rs|6f7dc7e5129b991aa41961cbed579080781dcd0d3085ff1242d0a072ff5d09d8
crates/uor-r4-core/src/native_geometric/dependent_language/query_participation.rs|79ee24638b36429393515dfe89d8679c3c90be1a25acc5cfc55cd7f560d19a72
crates/uor-r4-core/src/native_geometric/dependent_language/span_learning.rs|f66c4b2abce4cbde32d49cc9f1cf4fee6394d0fda8344e676cdc5473765aaa91
crates/uor-r4-core/src/native_geometric/general_prose_tests.rs|a781bc6f7ed2971a90d5a44829efbfd8ad7aef1459f6577fc2f0c21010d6de75
crates/uor-r4-core/src/native_geometric/hamming_policy/report.rs|98825086f28a4b77e2a7e0b7bc3c344cf1224a11c1f3f23e1d89c134080bd31e
crates/uor-r4-core/src/native_geometric/historical_read_training.rs|1aa5cae2d698928d3ec5c0266515f7148cd12df67fe287e2aec22277294e53e3
crates/uor-r4-core/src/native_geometric/language_relation/runtime.rs|9a6e63059f448eb7d58cb0e4b449211c688991ba4f63204c5ea83654e8d1fe88
crates/uor-r4-core/src/native_geometric/learner/geometric_attention.rs|e7682b5a3203d49a565cd29de27eca0ad9ed6eea550a01a6096a701fb3e59658
crates/uor-r4-core/src/native_geometric/learner/mod.rs|2d250b4104eab5fd49c3ba8d415ccdb2ab9d7d4e287958f6033df239138af6ec
crates/uor-r4-core/src/native_geometric/learner/relative_action_learning.rs|8ea7ecd3cc778f7cc69a9797924a73dbc5976069f825a40248179b6cac9af115
crates/uor-r4-core/src/native_geometric/learner/vsa_codes.rs|7941cbef9726ec49cdfbc27dfdd53f5828f3d64b4d4c01a1ddae0e33b327f742
crates/uor-r4-core/src/native_geometric/mixed_operator_tests.rs|4ce2adf49e652853c3a502378cbe5e620ff6676ae0bcde924c6ec20614fa6943
crates/uor-r4-core/src/native_geometric/ordered_state/mod.rs|1c07a8f8c33ad7d340b01654ce057d747e5dc980bdcf9dab4e299c9f32ed0fed
crates/uor-r4-core/src/native_geometric/relation_admission.rs|c9c3ae3a97691f6d1726081b0d08e886606538eaa4e7dd02631965450776544c
crates/uor-r4-core/src/native_geometric/relational_attention/runtime.rs|3232302df67b0ca2807c46f9953e0169814bd226e6a775894bebc1ddb912d7ff
crates/uor-r4-core/src/native_geometric/response_runtime.rs|79548be47acc125a6025ae946bb07a5a12ef06ac805766a0233711f8360dabd9
crates/uor-r4-core/src/native_geometric/shared_core/tests/active_sensitivity.rs|2abbc7cb19d22ecdbb976d46aef3d3f83dbb0e540505568c029b8cd160937c66
crates/uor-r4-core/src/native_geometric/shared_core/tests/final_emission.rs|31a08e17ac1ce82e76ed5fd4d98bba86ba60172e2c2d2846beb922b10bf44a21
crates/uor-r4-core/src/native_geometric/source_routing_tests.rs|f19c78f1fd6e684ce8d8e2e758701ea54dde8bba8759d2fa49b00889fb45a526
crates/uor-r4-core/src/native_geometric/typed_routing_training.rs|4d6d4b15cb8a0264d877d0f683f99b9fe9961f05ed085018c7d113b9d63d6166
crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs|40ca43bb829160329885dc2f966c158c4dfb89054d8ea8cdd51084c362455ccf
crates/uor-r4-core/src/native_geometric/writer_choice.rs|9b7ff8ef6f4561170cffb934e537092c74c94eaf4ba7507a0a8af32d4528edfc
"#;
const DEV_SOURCES: &str = r#"
docs/r4_zoology_checkpoint_continuation_1057.md|e32a10e123e800b6e6a72ec63e9a11d87ab5d41b095358706af66936045d8ba4
docs/r4_zoology_cyclic_facts_1071.md|df6471cd1e42b2363ebd0aa523cc7bd2ed0d13b31f589ebf0702c96a1d13d3ad
docs/r4_zoology_exact_transfer_1053.md|270e9fb63cb0d9da05bcf683256d00c911a8c8c7a8b80c7e53a6fbcdc7ba3622
docs/r4_zoology_mqar_control_1047.md|eeabec9454657231559edb7649d7199a6ba3fc42152439b0cc69fdf9e13f419b
docs/serving_default_flip_655_f.md|9bb28124e9c6a7423231d9c80ea8e5b88b2042e673a92460812694ce8061d890
docs/serving_engine_profiles_655_e.md|b8bc11a872505ceab16258edcc5d10351e3de1f5e92352589122c0b908201624
docs/serving_release_packaging_655_d.md|9f700ffe161c2609cb0ccde38745b41ddf95e781c592826d2d502a1688a202ec
docs/serving_rename_inventory_655_f.md|1f6c5806faad3d23e461efb7ea3bbd8dd1a76f71ab257f6d6b57a2fad8bc23b7
crates/uor-r4-graph-runtime/src/engine.rs|a8c6322ad607f5ea104f7699f1748d0133213a8d927caa0afcf93967c42b1337
crates/uor-r4-graph-runtime/src/packed_kernels.rs|d1a810c8cc95b1e1572fc0deadb4fff15459ee573efdaa53dc3726125e355f50
crates/uor-r4-graph-runtime/src/route_attention.rs|6ca45380a9404661ef1bb721706c5a057e6da8b85f4250441e3fd9b9305e2637
crates/uor-r4-graph-runtime/src/scoring.rs|7179b2ccf062275d6fdc2571e1bb957b581e644ef763f1e71af672be35825bbe
"#;

#[derive(Clone, Debug, serde::Serialize)]
pub struct PromptEdit {
    pub original: String,
    pub changed_source: String,
    pub original_value: String,
    pub changed_value: String,
    pub original_answer: String,
    pub changed_answer: String,
    /// Byte extent of the changed value in the original raw prompt.
    pub changed_byte_range: [usize; 2],
}

pub struct DataSet {
    pub fit: Vec<TrainingEpisode>,
    pub dev: Vec<TrainingEpisode>,
    pub manifest: Value,
    edits: HashMap<String, PromptEdit>,
}

impl DataSet {
    pub fn prompt_edit(&self, episode: &TrainingEpisode) -> Result<Option<PromptEdit>, String> {
        match self.edits.get(&episode.name) {
            Some(edit) => Ok(Some(edit.clone())),
            None if episode.prompt_len.is_none() => Ok(None),
            None => Err(format!("missing raw source edit for {}", episode.name)),
        }
    }
}

pub fn prompt_edit(
    data: &DataSet,
    episode: &TrainingEpisode,
) -> Result<Option<PromptEdit>, String> {
    data.prompt_edit(episode)
}

#[derive(Clone)]
struct Aligned {
    ids: Vec<u16>,
    bytes: Vec<Vec<u8>>,
    starts: Vec<usize>,
    ends: Vec<usize>,
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn tokenize_aligned(tokenizer: &HfBpeTokenizer, text: &str) -> Result<Aligned, String> {
    if text.len() > MAX_SOURCE_BYTES {
        return Err("source exceeds 1 MiB".into());
    }
    let mut out = Aligned {
        ids: Vec::new(),
        bytes: Vec::new(),
        starts: Vec::new(),
        ends: Vec::new(),
    };
    let mut offset = 0usize;
    for id in tokenizer.encode(text) {
        if id >= VOCAB {
            return Err(format!("token {id} exceeds pinned vocabulary"));
        }
        let bytes = tokenizer.decode_bytes(&[id]);
        let end = offset
            .checked_add(bytes.len())
            .ok_or_else(|| "byte offset overflow".to_string())?;
        if bytes.is_empty() || end > text.len() || text.as_bytes()[offset..end] != bytes {
            return Err(format!(
                "whole-stream byte mismatch at {offset}, token {id}"
            ));
        }
        out.ids.push(id as u16);
        out.bytes.push(bytes);
        out.starts.push(offset);
        out.ends.push(end);
        offset = end;
    }
    if offset != text.len() {
        return Err(format!("tokenizer ended at {offset}/{} bytes", text.len()));
    }
    Ok(out)
}

/// Equal (predecessor,target) is only a weak lexical recurrence opportunity.
/// Many earlier occurrences are equivalent; this is not an exact semantic ID.
fn recurrence_targets(tokens: &[u16]) -> Vec<Option<usize>> {
    let mut targets = vec![None; tokens.len()];
    for target in 9..tokens.len() {
        for source in (1..=target - 8).rev() {
            if tokens[source - 1] == tokens[target - 1] && tokens[source] == tokens[target] {
                targets[target] = Some(source);
                break;
            }
        }
    }
    targets
}

fn source_specs(raw: &str) -> Result<Vec<(&str, &str)>, String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (path, digest) = line
                .split_once('|')
                .ok_or_else(|| format!("invalid source manifest line: {line}"))?;
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(format!("invalid pinned SHA for {path}"));
            }
            Ok((path, digest))
        })
        .collect()
}

fn add_natural(
    repo: &Path,
    tokenizer: &HfBpeTokenizer,
    split: &str,
    specs: &[(&str, &str)],
    next_source: &mut u64,
    episodes: &mut Vec<TrainingEpisode>,
    manifest: &mut Vec<Value>,
) -> Result<(), String> {
    let per_source = if split == "fit" {
        MAX_NATURAL_FIT_TOKENS_PER_SOURCE
    } else {
        MAX_NATURAL_DEV_TOKENS_PER_SOURCE
    };
    for &(relative, expected_sha) in specs {
        let path = repo.join(relative);
        let raw = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if raw.len() > MAX_SOURCE_BYTES {
            return Err(format!("{relative} exceeds source byte cap"));
        }
        let actual_sha = sha(&raw);
        if actual_sha != expected_sha {
            return Err(format!(
                "pinned source mismatch {relative}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        let text =
            String::from_utf8(raw).map_err(|e| format!("{relative} is not exact UTF-8: {e}"))?;
        let aligned =
            tokenize_aligned(tokenizer, &text).map_err(|e| format!("tokenize {relative}: {e}"))?;
        let used = aligned.ids.len().min(per_source);
        if used < 16 {
            return Err(format!("{relative} has too few tokens"));
        }
        let source_id = *next_source;
        *next_source = next_source
            .checked_add(1)
            .ok_or_else(|| "source ID exhausted".to_string())?;
        let tokens = aligned.ids[..used].to_vec();
        let weak = recurrence_targets(&tokens);
        let weak_count = weak.iter().filter(|v| v.is_some()).count();
        episodes.push(TrainingEpisode {
            name: format!("{split}:natural:{relative}"),
            source_id,
            tokens,
            token_bytes: aligned.bytes[..used].to_vec(),
            source_targets: weak,
            prompt_len: None,
        });
        manifest.push(json!({
            "path":relative,
            "split":split,
            "sha256":actual_sha,
            "bytes":text.len(),
            "whole_stream_tokens":aligned.ids.len(),
            "used_tokens":used,
            "source_id":source_id,
            "weak_equal_bigram_pointers":weak_count,
            "exposure":"pinned repository open-development source; not a final holdout",
        }));
    }
    Ok(())
}

const FIT_NAMES: [&str; 24] = [
    "Cedar", "Aster", "Maple", "Willow", "Juniper", "Elm", "Oak", "Pine", "Birch", "Spruce", "Ash",
    "Linden", "Rowan", "Alder", "Hawthorn", "Yew", "Sequoia", "Cypress", "Poplar", "Acacia",
    "Beech", "Fir", "Larch", "Laurel",
];
const DEV_NAMES: [&str; 6] = ["Orion", "Vega", "Lyra", "Cygnus", "Perseus", "Cassiopeia"];

struct WorldText {
    before_value: String,
    after_value: String,
    question: String,
    positive_answer: String,
    negative_answer: String,
    positive_value: &'static str,
    negative_value: &'static str,
}

fn world_text(name: &str, other: &str, style: usize, distractor_positive: bool) -> WorldText {
    let distractor = if distractor_positive {
        "allowed"
    } else {
        "denied"
    };
    let long_gap = format!(
        "A separate ledger lists Project {other} with status {distractor} for its own queue. \
         That line belongs to {other}, not {name}. The audit also inventories a blue cable, \
         two empty racks, a scheduled inspection, an unused staging key, a cooling fan, \
         a closed ticket, and a note about a different account. None of these entries \
         changes the later signed entry for Project {name}.\n"
    );
    let (before_value, after_value, question) = match style {
        0 => (
            format!("The earlier memo for Project {name} was provisional.\nThe signed correction records status "),
            format!(" for Project {name}'s archive access.\n{long_gap}"),
            format!("Question: May Project {name} use its archive now? Give the consequence.\nAnswer:"),
        ),
        1 => (
            format!("Project {name} once had an unresolved archive request.\nThe replacement entry for Project {name} says archive access is "),
            format!(".\n{long_gap}"),
            format!("Question: Is archive use permitted for Project {name} now? Explain briefly.\nAnswer:"),
        ),
        2 => (
            format!("// The earlier deployment note for {name} is obsolete.\nlet current_status = Status::"),
            format!("; // current archive status for Project {name}\n{long_gap}"),
            format!("Question: Can the current status let Project {name} read the archive?\nAnswer:"),
        ),
        3 => (
            format!("// The old flag for Project {name} was not signed.\nconst PROJECT_{}_ARCHIVE: Status = Status::", name.to_uppercase()),
            format!(";\n{long_gap}"),
            format!("Question: Does the current constant permit archive access for Project {name}?\nAnswer:"),
        ),
        4 => (
            format!("A reviewer crossed out the preliminary line for Project {name}.\nThe replacement line reads: "),
            format!(" is the live permission for Project {name} to open the registry.\n{long_gap}"),
            format!("Question: Should Project {name} open the registry under the live permission?\nAnswer:"),
        ),
        _ => (
            format!("// A revised permission replaced the earlier draft for Project {name}.\nfn permission_for_{}() -> Permission {{ Permission::", name.to_lowercase()),
            format!(" }}\n{long_gap}"),
            format!("Question: Does the revised function permit Project {name} to open the registry?\nAnswer:"),
        ),
    };
    let positive_answer = format!(" Yes, Project {name} can use the current permission now.");
    let negative_answer = format!(" No, Project {name} cannot use the current permission now.");
    let (positive_value, negative_value) = if matches!(style, 2 | 3 | 5) {
        ("Allowed", "Denied")
    } else {
        ("allowed", "denied")
    };
    WorldText {
        before_value,
        after_value,
        question,
        positive_answer,
        negative_answer,
        positive_value,
        negative_value,
    }
}

fn first_overlapping_token(aligned: &Aligned, start: usize, end: usize) -> Option<usize> {
    aligned
        .starts
        .iter()
        .zip(&aligned.ends)
        .enumerate()
        .find_map(|(index, (&token_start, &token_end))| {
            (token_start < end && token_end > start).then_some(index)
        })
}

fn add_world_pair(
    tokenizer: &HfBpeTokenizer,
    split: &str,
    name: &str,
    other: &str,
    style: usize,
    distractor_positive: bool,
    next_source: &mut u64,
    episodes: &mut Vec<TrainingEpisode>,
    manifest: &mut Vec<Value>,
    edits: &mut HashMap<String, PromptEdit>,
) -> Result<(), String> {
    let world = world_text(name, other, style, distractor_positive);
    let values = [world.positive_value, world.negative_value];
    let answers = [&world.positive_answer, &world.negative_answer];
    let mut anchors = [0u16; 2];
    for polarity in 0..2 {
        let value = values[polarity];
        let answer = answers[polarity];
        let prompt = format!(
            "{}{}{}{}",
            world.before_value, value, world.after_value, world.question
        );
        let changed_source = format!(
            "{}{}{}{}",
            world.before_value,
            values[1 - polarity],
            world.after_value,
            world.question
        );
        let full = format!("{prompt}{answer}");
        let aligned = tokenize_aligned(tokenizer, &full)
            .map_err(|e| format!("correction {name}/{style}/{polarity}: {e}"))?;
        let prompt_ids = tokenizer.encode(&prompt);
        let prompt_len = prompt_ids.len();
        if aligned.ids.len() > MAX_EPISODE_TOKENS
            || prompt_len >= aligned.ids.len()
            || aligned.starts[prompt_len] != prompt.len()
            || !aligned
                .ids
                .iter()
                .take(prompt_len)
                .map(|&id| u32::from(id))
                .eq(prompt_ids.iter().copied())
        {
            return Err(format!(
                "correction {name}/{style}/{polarity} has unstable BPE prompt boundary"
            ));
        }
        let source_start = world.before_value.len();
        let source_end = source_start + value.len();
        let source_token = first_overlapping_token(&aligned, source_start, source_end)
            .ok_or_else(|| format!("correction {name}/{style}/{polarity} has no source anchor"))?;
        if prompt_len.saturating_sub(source_token) < MIN_CORRECTION_LAG {
            return Err(format!(
                "correction {name}/{style}/{polarity} source lag below 17"
            ));
        }
        anchors[polarity] = aligned.ids[source_token];
        let decision_start = prompt.len() + 1; // answer begins with one space
        let decision_end = decision_start + if polarity == 0 { 3 } else { 2 }; // Yes / No
        let mut targets = vec![None; aligned.ids.len()];
        for (position, (&start, &end)) in aligned.starts.iter().zip(&aligned.ends).enumerate() {
            if position >= prompt_len && start < decision_end && end > decision_start {
                targets[position] = Some(source_token);
            }
        }
        if !targets[prompt_len..].iter().any(Option::is_some) {
            return Err(format!(
                "correction {name}/{style}/{polarity} lost decision tokens"
            ));
        }
        let source_id = *next_source;
        *next_source = next_source
            .checked_add(1)
            .ok_or_else(|| "source ID exhausted".to_string())?;
        let episode_name = format!(
            "{split}:correction:{name}:style-{style}:{}",
            if polarity == 0 {
                "positive"
            } else {
                "negative"
            }
        );
        manifest.push(json!({
            "name":episode_name,
            "split":split,
            "kind":if matches!(style, 2 | 3 | 5) {"code_shaped_correction"} else {"prose_correction"},
            "source_id":source_id,
            "sha256_full_text":sha(full.as_bytes()),
            "prompt_tokens":prompt_len,
            "tokens":aligned.ids.len(),
            "source_value_byte_span":[source_start,source_end],
            "source_anchor_token":anchors[polarity],
            "source_anchor_position":source_token,
            "answer_decision_targets":targets.iter().filter(|x|x.is_some()).count(),
            "pair_id":format!("{split}:{name}:style-{style}"),
            "exposure":"balanced authored open-development pair; not semantic qualification",
        }));
        edits.insert(
            episode_name.clone(),
            PromptEdit {
                original: prompt,
                changed_source,
                original_value: value.to_owned(),
                changed_value: values[1 - polarity].to_owned(),
                original_answer: answer.clone(),
                changed_answer: answers[1 - polarity].clone(),
                changed_byte_range: [source_start, source_end],
            },
        );
        episodes.push(TrainingEpisode {
            name: episode_name,
            source_id,
            tokens: aligned.ids,
            token_bytes: aligned.bytes,
            source_targets: targets,
            prompt_len: Some(prompt_len),
        });
    }
    if anchors[0] == anchors[1] {
        return Err(format!(
            "correction {name}/{style} has identical positive/negative source anchor IDs {}",
            anchors[0]
        ));
    }
    Ok(())
}

fn add_inherited_a1(
    repo: &Path,
    tokenizer: &HfBpeTokenizer,
    next_source: &mut u64,
    episodes: &mut Vec<TrainingEpisode>,
    manifest: &mut Vec<Value>,
    edits: &mut HashMap<String, PromptEdit>,
) -> Result<(), String> {
    let old = inherited_a1::load_development_data(repo, tokenizer)?;
    for mut episode in old.dev.into_iter().filter(|item| item.prompt_len.is_some()) {
        let prompt_len = episode.prompt_len.ok_or("inherited prompt missing")?;
        let ids: Vec<u32> = episode.tokens[..prompt_len]
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        let prompt = tokenizer.decode(&ids);
        let (old_value, new_value, old_answer, new_answer, needle) =
            if episode.name.ends_with("orion-allowed") {
                (
                    "allowed",
                    "denied",
                    " Yes, Orion can reuse an archived response now.",
                    " No, Orion cannot reuse an archived response now.",
                    "revision allowed Project Orion",
                )
            } else if episode.name.ends_with("larch-denied") {
                (
                    "denied",
                    "allowed",
                    " No, Larch cannot rely on an expired record now.",
                    " Yes, Larch can rely on an expired record now.",
                    "corrected operator note denied Project Larch",
                )
            } else {
                return Err(format!("unknown inherited correction {}", episode.name));
            };
        let match_start = prompt.find(needle).ok_or_else(|| {
            format!(
                "inherited prompt missing correction phrase: {}",
                episode.name
            )
        })?;
        let value_in_needle = needle
            .find(old_value)
            .ok_or("inherited correction phrase missing value")?;
        let value_start = match_start + value_in_needle;
        let mut changed_source = prompt.clone();
        changed_source.replace_range(value_start..value_start + old_value.len(), new_value);
        edits.insert(
            episode.name.clone(),
            PromptEdit {
                original: prompt,
                changed_source,
                original_value: old_value.into(),
                changed_value: new_value.into(),
                original_answer: old_answer.into(),
                changed_answer: new_answer.into(),
                changed_byte_range: [value_start, value_start + old_value.len()],
            },
        );
        episode.source_id = *next_source;
        *next_source = next_source
            .checked_add(1)
            .ok_or_else(|| "source ID exhausted".to_string())?;
        manifest.push(json!({
            "name":episode.name,
            "source_id":episode.source_id,
            "kind":"inherited_a1_exposed_correction",
            "tokens":episode.tokens.len(),
            "prompt_tokens":prompt_len,
            "sha256_tokens":sha(&episode.tokens.iter().flat_map(|id|id.to_le_bytes()).collect::<Vec<_>>()),
            "exposure":"inherited A1 development probe; never fresh qualification",
        }));
        episodes.push(episode);
    }
    Ok(())
}

pub fn load_development_data(repo: &Path, tokenizer: &HfBpeTokenizer) -> Result<DataSet, String> {
    if tokenizer.vocab_size() != VOCAB as usize {
        return Err(format!(
            "expected {VOCAB} token vocabulary, got {}",
            tokenizer.vocab_size()
        ));
    }
    let fit_specs = source_specs(FIT_SOURCES)?;
    let dev_specs = source_specs(DEV_SOURCES)?;
    let mut fit_natural = Vec::new();
    let mut fit_corrections = Vec::new();
    let mut dev = Vec::new();
    let mut natural_manifest = Vec::new();
    let mut correction_manifest = Vec::new();
    let mut edits = HashMap::new();
    let mut source_id = 1u64;
    add_natural(
        repo,
        tokenizer,
        "fit",
        &fit_specs,
        &mut source_id,
        &mut fit_natural,
        &mut natural_manifest,
    )?;
    add_natural(
        repo,
        tokenizer,
        "dev",
        &dev_specs,
        &mut source_id,
        &mut dev,
        &mut natural_manifest,
    )?;
    for (index, &name) in FIT_NAMES.iter().enumerate() {
        let other = FIT_NAMES[(index + 7) % FIT_NAMES.len()];
        for style in 0..4 {
            add_world_pair(
                tokenizer,
                "fit",
                name,
                other,
                style,
                (index + style) % 2 == 0,
                &mut source_id,
                &mut fit_corrections,
                &mut correction_manifest,
                &mut edits,
            )?;
        }
    }
    for (index, &name) in DEV_NAMES.iter().enumerate() {
        let other = DEV_NAMES[(index + 2) % DEV_NAMES.len()];
        for style in [if index < 3 { 0 } else { 4 }, if index < 3 { 2 } else { 5 }] {
            add_world_pair(
                tokenizer,
                "dev",
                name,
                other,
                style,
                index % 2 == 0,
                &mut source_id,
                &mut dev,
                &mut correction_manifest,
                &mut edits,
            )?;
        }
    }
    add_inherited_a1(
        repo,
        tokenizer,
        &mut source_id,
        &mut dev,
        &mut correction_manifest,
        &mut edits,
    )?;
    if fit_corrections.len() % 2 != 0 {
        return Err("A2 fit correction worlds did not form complete pairs".into());
    }
    let pair_count = fit_corrections.len() / 2;
    let natural_count = fit_natural.len();
    let mut fit = Vec::with_capacity(natural_count + fit_corrections.len());
    let mut natural_iter = fit_natural.into_iter();
    let mut correction_iter = fit_corrections.into_iter();
    // Proportionally distribute natural sources among paired counterfactuals.
    // Alternate pair polarity order so every epoch does not end each pair on
    // the same answer. Both worlds remain adjacent; no episode is duplicated.
    for pair_index in 0..pair_count {
        let previous_natural = pair_index * natural_count / (pair_count + 1);
        let through_natural = (pair_index + 1) * natural_count / (pair_count + 1);
        for _ in previous_natural..through_natural {
            fit.push(
                natural_iter
                    .next()
                    .ok_or("A2 natural interleave count mismatch")?,
            );
        }
        let positive = correction_iter
            .next()
            .ok_or("A2 correction pair missing positive world")?;
        let negative = correction_iter
            .next()
            .ok_or("A2 correction pair missing negative world")?;
        if pair_index % 2 == 0 {
            fit.push(positive);
            fit.push(negative);
        } else {
            fit.push(negative);
            fit.push(positive);
        }
    }
    fit.extend(natural_iter);
    if correction_iter.next().is_some() {
        return Err("A2 correction interleave left unused worlds".into());
    }
    let fit_episode_order: Vec<Value> = fit
        .iter()
        .map(|episode| {
            json!({
                "name":episode.name,
                "source_id":episode.source_id,
                "tokens":episode.tokens.len(),
            })
        })
        .collect();
    let fit_order_sha256 = sha(&serde_json::to_vec(&fit_episode_order)
        .map_err(|e| format!("serialize A2 fit order: {e}"))?);
    let fit_tokens: usize = fit.iter().map(|item| item.tokens.len()).sum();
    let dev_tokens: usize = dev.iter().map(|item| item.tokens.len()).sum();
    if fit_tokens > MAX_FIT_TOKENS || dev_tokens > MAX_DEV_TOKENS {
        return Err(format!("A2 episode token cap exceeded: fit {fit_tokens}/{MAX_FIT_TOKENS}, dev {dev_tokens}/{MAX_DEV_TOKENS}"));
    }
    let manifest = json!({
        "schema":"uor-r4.integrated-attention-a2-data/1",
        "source_snapshot":SOURCE_SNAPSHOT,
        "tokenizer_sha256":TOKENIZER_SHA256,
        "tokenizer_address":tokenizer.address(),
        "limits":{
            "max_source_bytes":MAX_SOURCE_BYTES,
            "max_natural_fit_tokens_per_source":MAX_NATURAL_FIT_TOKENS_PER_SOURCE,
            "max_natural_dev_tokens_per_source":MAX_NATURAL_DEV_TOKENS_PER_SOURCE,
            "max_episode_tokens":MAX_EPISODE_TOKENS,
            "max_fit_tokens":MAX_FIT_TOKENS,
            "max_dev_tokens":MAX_DEV_TOKENS,
            "minimum_correction_source_lag":MIN_CORRECTION_LAG,
        },
        "fit_episodes":fit.len(),
        "fit_tokens":fit_tokens,
        "fit_episode_order":fit_episode_order,
        "fit_episode_order_sha256":fit_order_sha256,
        "fit_interleave":"proportional natural episodes between adjacent positive/negative worlds; pair polarity alternates by pair index",
        "dev_episodes":dev.len(),
        "dev_tokens":dev_tokens,
        "natural_sources":natural_manifest,
        "correction_cases":correction_manifest,
        "claim_boundary":"open development: ~100 pinned repository prose/Rust sources, balanced authored counterfactuals and inherited exposed A1 probes; one-token corrected-value anchors are weak fit-only hints, not semantic spans; no final qualification",
    });
    Ok(DataSet {
        fit,
        dev,
        manifest,
        edits,
    })
}
