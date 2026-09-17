> **UOR-R4 Project Knowledge Base — source report B (code audit).** Produced 2026-09-16 by a Claude research session at Casey's request, from the GitHub clone (main @ a5655e93, 2026-09-14) and live sources. Read-only audit; nothing in the repo was modified. Treat "[Inference]"/"[ASSESS]" as reviewer judgement and everything else as quoted/derived from the cited files. Index: `00-project-brain-index.md` (this folder) / `claude/00-project-brain-index.md` (Claude Project).

# B. Code audit: what UOR-R4 actually computes and how it learns

Repository: `/home/claude/work/uor-r4` (UOR-Foundation/uor-r4, main @ a5655e93, 2026-09-14).
Scope: `crates/uor-r4-core/src/native_geometric/` (277 `.rs` files, 122,541 lines), `src/native_geometric_cli.rs`, `src/native_geometric_service.rs`, `src/native_wasm.rs`, `crates/uor-r4-api`, plus glances at `prime_route_attention.rs`, `spiralcore_operator.rs`, the tokenizer and the evidence JSON that records artifact identities and sizes. Everything below is derived from source; where I rely on a docs/evidence file (because the artifacts themselves are gitignored and absent from the clone) I say so. Line numbers are relative to the repository root unless a path is given relative to `native_geometric/` (abbreviated `ng/`). Inferences are marked **(inference)**.

---

## 0. One-paragraph orientation

There are two unrelated model families in `native_geometric/`, sharing only the 120-element H4 (binary icosahedral) multiplication table and the fixed zeta-phase constants:

1. **Retained "normal model" (artifact `blake3:15baec48…`, 17,421,561 bytes JSON).** A *count-fitted, integer-quantized log-linear next-piece predictor* (26 hashed discrete features, Laplace-smoothed conditional log-counts, 7 five-bit mixture gates) wrapped in ~40 optional "witness" layers that implement an exact, bounded key-value memory (`Record: X in Y. … Where is X? Answer:` tasks), a copy operator, integer arithmetic operators, and learned routing tables. Its serving kernel is `ng/runtime.rs`; everything is reachable from the `r4 geometric` CLI, the HTTP service, the API crate and the WASM facade.

2. **Experimental "language core" (artifact sha256 `60d19679…`, 63,640 bytes JSON).** A stack of thirteen nested artifacts (`query_participation` → `occurrence_role` → `correspondence` → `span` → `completion` → `scheduling` → `dependent_language::runtime` → `relative_language` → `language_relation` → `ordered_state` → `recurrent_text` → `text_attention` → `dependent_attention` → `relational_attention` → `BoundGeometry`). It answers exactly two lowercase questions over exactly four supplied lowercase sentences by exact word matching, boolean-rule candidate selection, and a copy of the selected span. It is **not reachable from any CLI/service/API/WASM entry point**; it is trained and evaluated only through `#[test] #[ignore]` functions driven by environment variables.

Neither family contains a matrix product, a softmax, or a floating-point operation on the serving path. Both are learned by counting, greedy/coordinate search, or (at the bottom of the experimental stack and for a few small tables in the normal model) offline float SGD/Adam on a surrogate, followed by quantization to integer/boolean tables.

---

## 1. Inference path

### 1.1 Retained normal model (15baec48)

**Serving entry points.** `src/native_geometric_cli.rs:346-425` (`run`) dispatches `Generate` → `Model::generate`, `Evaluate` → `Model::evaluate`, `Chat`, `Serve` → `native_geometric_service::serve`. The service (`src/native_geometric_service.rs:14, 366-465`) and the API (`crates/uor-r4-api/src/native_capability_api.rs:353-581`) drive the same three calls: `session.observe(&model, token)`, `session.predict(&model)`, `model.decode(...)`. `src/native_wasm.rs` is a thin `wasm_bindgen` facade over `uor_r4_api::native_capability_api::{NativeModel, WasmModelRuntime}`. `NativeModel::load_from_bytes` refuses empty bytes (`native_capability_api.rs:226-234`) — the fix for the Sept-8 "silently trained a fixture" defect. Temperature must be 0 (`:315-322`); generation is deterministic argmax.

**Input encoding (words → pieces → primes).**
- `Model::encode` (`ng/training.rs:1191-1200`): text is split by `canonical_lexical_piece_bytes` (`crates/uor-r4-core/src/canonical_lexical_ingestion.rs:842-854`: each non-whitespace run owns its preceding whitespace). A piece present in `lexical_pieces` (top `max_lexical_pieces`=4096 by frequency, `training.rs:54-90`) becomes token `258+index`; otherwise it falls back to raw bytes, token `byte+2`. `BOS=0`, `EOS=1` (`ng/mod.rs:217-220`). Vocabulary ≤ 258+4096 = 4354.
- Each token gets fixed geometry at compile time (`training.rs:108-166`): `prime` = the token-index-th prime; `leaf = prime % 120` (`:130`, a u16 index into the 120-element group; BOS → identity); eight `phases: [u16; 8]` = `round(gamma_j * log(p/2) / 2π * 65536)` (`:119-124`, computed with `libm` **offline only**).

**Carried state** (`ng/runtime.rs:44-66`): a ring buffer of `context_tokens` (default 128) token ids; `h4: u16` = the ordered group product of the leaves in the window, with sliding eviction by left-multiplying the evicted leaf's inverse (`:335-340, 357`); `phases: [u16; 8]` wrapping sums of per-token phase deltas (`:349-352, 377-380`); `paired_coefficients: [i64; 8]` summed anchor coefficients (`:341-347, 358-364`); `radial: [i64; 2]` computed from a precompiled squares table (`:366-376`); plus optional sub-states (`memory`, `values`, `completion`, `response_entry`, `word_copy`, `field_composition`). So the "H4 120-element group element" is real, but it is a *hash of the last 128 tokens into 120 classes* (4354 tokens map onto 120 leaves via `prime % 120`), refreshed by table lookup.

**Feature extraction** (`runtime.rs:446-504`): 26 discrete `Feature{kind:u8, value:u64}` keys — last prime, (previous,last) prime pair, `h4`, `(previous_h4,h4)`, orientation class of `h4`, `h4`×top-4-bits-of-phase0, paired class, radial, eight phase channels quantized to 4 bits (`phase >> 12`), eight paired coefficients, heatmap class, projection-radius class.

**Reads / candidate selection** (`runtime.rs:558-892`): each admitted feature is binary-searched in the sorted `model.rows: Vec<ScoreRow>` (`mod.rs:541-548`). Candidates are only tokens in `prior_postings` (top-32 unigram) plus each matched row's `postings` (top-16 targets per feature); there is no vocabulary scan (`:555-557, 666-674`). Score of a candidate:

```rust
// runtime.rs:513-528
let prior = model.prior_scores[token as usize];
let mut score = i64::from(prior);
let mut groups = [0_i64; 7];
for &row_index in rows { ... conditional = row.scores[bsearch] or row.default_score;
    groups[row.feature.group()] += (i64::from(conditional) - i64::from(prior)) >> row.feature.shift(); }
for (value, &gate) in groups.into_iter().zip(gates) { score += gate_eighths(value, gate); }
```
`gate_eighths` (`:948-966`) expands a 5-bit gate (0..16, i.e. weights 0..2.0 in eighths) with shifts and adds. Gates come from `model.readout` keyed by last prime with a global fallback (`:639-646`). The shortlist is kept sorted, size ≤ `candidate_limit` (32), ties broken by token id (`:542-552`). Result: **a quantized naive-Bayes / product-of-experts over count tables**, i.e. a smoothed hashed n-gram model whose "geometric" features are group-fold and phase-sum hashes of the window.

After the base shortlist, the optional layers can override `best` in a fixed order (`:684-867`): the memory reader offers candidates; typed values (`values.offer`), completion, response-entry, word-copy, joint recurrent routing, operation transition, field composition each may replace `best`. When word-copy is "Emitting", `predict` short-circuits and returns the next copied byte with a marker score (`:566-614`).

**Exact memory reads** (`ng/memory_runtime.rs`): on `observe`, for `source_distance` in `1..=4`, the current token is indexed under the address `(cue token << source_shift | source_distance-1) << posting_shift` (`:64-80`) — an inverted index "what followed this token at distance d". On `collect` (`:128-`), the last `query_tokens` (8) tokens are looked up, up to `candidate_limit` (128) postings are visited, and each candidate is scored by a sparse *linear* model `rows: Vec<MemoryWeight{feature, score:i32}>` over 18 discrete features (`ng/memory_types.rs:4, 88-93, 133-146`). This is an induction-head/copy mechanism implemented as a hash table plus an integer linear scorer. The relation memory (`ng/relation.rs:7-12`) holds at most `RELATIONS = 16` owner→value records with a versioned directory; owner matching is exact word equality (`relation.rs:783 record.owner.matches(...)`).

**Source/typed routing** (`ng/source_routing.rs:36-67`): features → group-element "codes", composed by table product into a 2-lane state; `score` (`:68-104`) = bias + `ranks[state * landmark^-1]` per action (Angular mode) or an equality indicator. This is the one place where the group structure is used as a *learned* parameter space (codes and landmarks are u16 group indices chosen by search, see §2).

**Output.** `Prediction.token` → `Model::decode` (`training.rs:1201-1215`) → bytes; EOS stops. Typed arithmetic emits numerals via `numeral.rs`; integer `Mul` is done by shift-add (`ng/value_runtime.rs:604-640`), a scalar operation on user-supplied operands, not on parameters.

**Arithmetic on the serving path — verified.** A grep over the 29 top-level runtime files for `f64|f32|libm::|softmax|.exp(|.ln(` finds nothing. The integration test `native_kernel_source_has_no_forbidden_arithmetic_or_float_types` (`crates/uor-r4-core/tests/native_geometric_allocations.rs:120-`) textually scans the `// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN/END` regions (25 files carry the marker) for `*`, `/`, `%` operators and float types (`crates/uor-r4-core/src/transformerless/source_scan.rs:87-`). The only floats in `ng/memory_types.rs`/`value_types.rs` are report fields. **No dot product, no matmul, no softmax at inference: confirmed.** Caveat: the "no multiply" claim is a *textual* guard; it does not cover `saturating_mul` on capacities (e.g. `runtime.rs:74-76`, allocation bookkeeping) and it does not by itself prove anything about model quality.

### 1.2 Experimental language core (60d19679)

**Not on any product path.** `grep -rl dependent_language|shared_core|relational_attention|ordered_state|hamming_… src crates/uor-r4-api` returns only `src/main.rs:2993` (an unrelated `recursive_geometric_attention` probe). The 60d19679 artifact type is `dependent_language::query_participation::Artifact` (`ng/dependent_language/query_participation.rs:34-45`); its training/evaluation drivers are `#[test]` functions gated on env vars (`independent_neighbor_report.rs:428-434, 825-831`; `query_participation_report.rs:568-572`), and `mod.rs` declares every `*_report.rs` under `#[cfg(test)]`.

**Input encoding.** Raw bytes. `language_relation::runtime::words` (`ng/language_relation/runtime.rs:21-51`) accepts only lowercase ASCII words (≤16 bytes), whitespace and `.?`; anything else is `Error::Shape`. Each word becomes an `ordered_state::Query` (`ng/ordered_state/runtime.rs:27-32`): `prefixes: Vec<[u16;2]>` = for each byte position the two prefix products (lane 0: `old * leaf`, lane 1: `leaf * old`, the `CANONICAL` operators `[Right, Left]`, `:71`) and `occurrences: Vec<u32>` = the byte's prime. Leaf = `BoundGeometry::byte_leaf` = `prime(byte+2) % 120` (`ng/addressed_attention/artifact.rs:132-134`, built from `training::geometry(258,256)` at `:57`).

**Protocol shape** (`ng/dependent_language/runtime.rs:89-112, 397-454`): prompt ≤256 bytes, exactly two `?`-terminated clauses of ≤128 bytes; exactly four records (`&[Vec<u8>; 4]`), each ≤16 words; payload ≤3 words/50 bytes (`:128-144`).

**Reads.** A query word "matches" a source word iff the signed-prefix Hamming distance is zero:

```rust
// ordered_state/runtime.rs:155-163
for (q, k) in q.prefixes.iter().zip(&k.prefixes) { d += m.distance(*q, *k)?; }
for _ in 0..q.prefixes.len().abs_diff(k.prefixes.len()) { d += 240; }
```
`Metric::distance` (`ng/hamming_refinement/metric.rs:57-75`) = popcount of XOR of two 120-bit root signatures, per lane. Signatures are distinct and the metric is reflexive/symmetric per the census (`docs/evidence/native_geometric_hamming_refinement_973.json`: `distinct_signatures 120`, `colliding… 0`), so distance 0 ⇔ identical prefix-product sequences ⇔ identical *leaf* sequences. Because `a/v`, `c/x`, `d/y`, `f/z` share leaves (`prime%120` ∈ {61,77,83,91}; 22 distinct leaves for 26 letters — computed from `training.rs:130`), the geometric equality is a **strictly coarser** relation than byte identity; see §5.

Given the match matrix, `relative_language::runtime::topology` (`ng/relative_language/runtime.rs:110-170`) computes 18 boolean relational features per candidate source word (unmatched, prefix, count≥2/3, order descents, endpoints, …). A candidate is *accepted* if any learned rule `r` satisfies `f & r == r` (`:73-75`); exactly one accepted candidate → `Selected`, otherwise `NoCompatibleCandidate`/`Ambiguous` (`:253-271`). `dependent_language::runtime::updates_with_window` (`:193-272`) does the same with 8 features and ≤8 rules with ≤3 literals to pick *which query word to replace* with the first answer, splicing the payload into the second question (`splice_with_window`, `:148-172`). Upper layers add: an injective query→source occurrence correspondence search bounded by `MAX_SEARCH_NODES = 16_384` (`ng/dependent_language/occurrence.rs:17, 46-56`); per-word "content vs context" role tables keyed by `(center, left, right)` word-identity trigrams with `ANY`/`EDGE`/`UNKNOWN` wildcards (`occurrence_role.rs:22-35, 520-563`; `query_participation.rs:24-30, 164-193`); span rules (`span.rs`, ≤3 words); scheduling (`scheduling.rs: ROWS = 8`), completion (`completion.rs: ROWS = 16`), and the recurrent Emit/Read/Stop table (`recurrent_text/runtime.rs:13, actions.len()==2048`, row = `byte + 256*present + 512*has_spans + 1024*next_available`, `:181-185`).

**Output.** The selected span's bytes are emitted one at a time through `text_attention::runtime::symbol` (`ng/text_attention/runtime.rs:95-108`): a hard boolean circuit of 2-input LUTs (`relational_attention/circuit.rs:150-171`) maps (byte, present) → 9 output bits (8 data bits + EOS). Stop emits EOS (`recurrent_text/runtime.rs:252-260`). `MAX_READS = 4`, `MAX_BYTES = 128`, `MAX_STEPS = 160` (`recurrent_text/runtime.rs:14-16`); dependent scheduling caps at 96 steps.

**Arithmetic.** Table lookups (group products, signatures), XOR/popcount, u16 additions, boolean LUT evaluation, bounded DFS. No floats (`circuit::hard` is pure boolean; the f64 `Parameters.cells` are exported to `u8` truth tables at `circuit.rs:87-100` and never loaded at serving).

---

## 2. Learning: what is learned and how

### 2.1 Retained normal model

| Learned object | Location | Fitting rule |
|---|---|---|
| `prior_scores: Vec<i32>`, `rows: Vec<ScoreRow{default_score, scores:[(token,i32)], postings}>` | `mod.rs:536-548, 652-654`; `training.rs:168-373` | **Counting.** "Count estimation is the learning rule" (`training.rs:254`). For every target position, all 26 features increment `CountRow.targets[token]` (`:286-317`); `compile` writes `round(256·ln((count+1)/(total+V)))` (`:326-373, 459-461`). Storage ceilings `max_rows`/`max_associations` drop new events (`:289-292, 308-312`). |
| `readout` (7 global gates + ≤4096 per-last-prime gates, u8 in eighths) | `ng/mixture.rs:16-32, 389-416` | Offline **SGD on softmax cross-entropy over the shortlist** (`update`: `weights[g] -= 0.025·clamp(grad)`, `:411-415`), then `quantize` to 0..16. |
| `memory_read.rows: Vec<MemoryWeight{feature, score:i32}>` (≤262,144 features) | `ng/memory_types.rs:88-93`; `ng/memory_training.rs:660-860` | Offline **SGD on latent-route marginal NLL** ("pointer" pretraining, bias grid, then max-route CE; `:700-830`), `libm::exp`, quantized ×256 (`:832-840`). |
| `values`/`completion`/`response_entry`/`role_read` rows (`ValueRow{feature, weight:i32}`) | `ng/value_types.rs:100-103`; `role_read_training.rs:334`, `value_training.rs:328-335` | Offline SGD with softmax residual, quantized. |
| `SourceRouting{codes:[feature→[u16;2] group elements], landmarks, biases, ranks}` | `ng/source_routing.rs:12-27`; `source_routing_training.rs:663-720` | **Randomized pairwise coordinate search over discrete group elements**: propose `proposals` (≤120) roots per coordinate, accept if `objective` (correct count, hinge) improves, wall-clock cap ≤120 s (`:675-706`). |
| `learned_routing::RoutingBlock` (2 heads, window 8, queries/emissions as group elements) | `ng/learned_routing.rs:9-11, 117-`; `learned_routing_training.rs` | Same family of discrete search; `RoutingMode::{Angular, Equality}`. |
| Dozens of "witness" blocks (`historical_version_intent`, `current_query_handoff`, `writer_role`, …) | `mod.rs:563-665`; `training.rs:577-1000` | Each nests the parent's `artifact_cid`; `validate()` peels the layer, recomputes the parent digest and recurses (`training.rs:594-604, 663-684`). Some (e.g. the turn-window promotion that produced 15baec48) change **no parameters**, only a versioned behaviour flag (`docs/native_geometric_reader_scope_repair_973.md`, "The change / Contract"). |

There is **no gradient descent through the serving kernel** and no end-to-end objective; each layer is fitted with all others frozen ("Components are often fitted with other parameters frozen", README:97). Objectives are per-layer: next-piece log-likelihood (counts), shortlist cross-entropy (readout), pointer NLL (memory), "correct label count + hinge" (routing), and, for the response layers, exact match of the generated bytes against authored accepted answers (`crates/uor-r4-core/src/answer_oracle.rs`, `examples/native_historical_version.rs`).

### 2.2 Experimental core

Bottom of the stack — the only genuine gradient training:

- `relational_attention::learning::fit` (`ng/relational_attention/learning.rs:129-299`): a 288-input scorer circuit of unary predicates AND-reduced (`structures`, `:50-`) and a 9→9 decoder circuit. Decoder: BCE on soft multilinear forward (`soft_forward/soft_backward`, `circuit.rs:222-293`), `cell -= lr·clamp(grad)` (`:189-193`). Scorer: **Adam on a biased "hard-decision residual through log-multilinear-conjunction" surrogate**:
  ```rust
  // relational_attention/learning.rs:242-263
  let residual = f64::from(hard.outputs[0]) - label;
  ... grad[j][row] += weight * residual / (train.len() as f64) / sc.cells[j][row];
  momentum = 0.9m + 0.1d; variance = 0.999v + 0.001d²; cell = clamp(cell - lr·m̂/(√v̂+1e-8), 0.001, 0.999)
  ```
  Exported as `u8` truth tables (`export`, `circuit.rs:87-100`); model selection = best training hard-exact at 25-epoch checkpoints (`:268-281`). Similar loops in `text_attention/learning.rs:125-246` (writer) and `dependent_attention/learning.rs:172-` (24→16 query-update circuit).
- `ordered_state::learning::fit` (`ng/ordered_state/learning.rs:25-65`): **exhaustive enumeration of the 4×4 operator pairs**, choose the one with fewest token errors.
- `recurrent_text::learning::fit` (`ng/recurrent_text/learning.rs:160-220`): bounded trajectory search finds action sequences that reproduce the target; per-row action counts; a single "epoch" of softmax steps that is effectively **argmax of counts per row** (`:197-218`).
- `language_relation`/`relative_language::learning::fit` (`ng/relative_language/learning.rs:85-167`): **greedy set cover over all conjunctions of 1–4 of 18 predicates** (4,047 proposals, `:97-110`):
  ```rust
  for _ in 0..MAX_RULES { for &rule in &proposals { for (p,&(old_n,old_good)) in prepared.iter().zip(&old) {
      let (n, good) = selected(&a, p, rule);
      if !good || (old_n == 1 && old_good && n != 1) { valid = false; break; }
      count += usize::from(n == 1 && good); }
    if valid && count > best_count { best = rule; best_count = count; } }
    if best == 0 { break; } a.rules.push(best); ... }
  ```
  "Final-output credit": a candidate is `compatible` iff writing its bytes through the frozen writer reproduces `answer+EOS` (`prepare`, `:48-73`); no source/word labels are used.
- `occurrence_role`/`query_participation` (`query_participation.rs:196-233 induce`, `occurrence_role.rs:532-563`): count positive/contrary credit per `(center,left,right)` key from successful generations, then **retain all maximally general keys (wildcarding left/right) with zero contrary credit** — symbolic rule induction.
- `dependent_language::completion/scheduling/span/correspondence` learning: same count-then-argmax or greedy-cover patterns (`*_learning.rs`).

The "suffix-LOO" and "cost-to-go" estimators the coordinator asked about live only in the **failed** `addressed_attention` pilot (`ng/addressed_attention/causal_credit.rs:1-2, 59-`: REINFORCE-style particle sampling with a suffix leave-one-out baseline; `learner.rs:115-129` plain SGD). Gates: `FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT` (both evidence files). `shared_core` (13k lines) is a coordinate-search learner over H4-root parameters with a byte/EOS binary-tree decoder; its gate is `FAIL_CONTEXTUAL_TRANSFER_SMOKE` (`docs/integration/current-state.md:569`: held-out NLL −1.4% vs required 2%, accuracy 2→0 on 249 positions, 100,000 proposals in 28 s).

---

## 3. Training data

- **Base count model:** the recovery run read **329,414 bytes** of `train.jsonl` (78 documents, 75,448 target positions, 57,541–110,632 rows, 704k–764k associations; `docs/evidence/native_geometric_recovery_973.json`). The prepared corpus was a truncated prefix (~397 KB–1.4 MB read) of `TinyStoriesV2-GPT4-train.txt` plus the repository's own Rust sources (`prime_route_attention.rs`, `semantic/reasoning.rs`) — same evidence file. `r4 geometric prepare` (`native_geometric_cli.rs:427-493`) chunks text into ≤8192-byte documents, dedups, and puts every 5th into `development.jsonl`, with the preceding one into `readout.jsonl`. The documented example caps input at 1 MiB (`docs/native_geometric_workflow.md:224-227`). So the "language model" component has seen well under a megabyte of prose.
- **Normal-model task layers:** authored templates with fresh random spellings, generated in the example drivers, e.g. `crates/uor-r4-core/examples/native_historical_version.rs:235-264, 805-830, 1056-1059`: `"Record: {o} in {v0}. {o} now in {v1}. Where is {o}? Answer:"`, styles `"Name the owner first."`, intents Initial/Previous/Current/Abstain with frozen accepted strings (`answer_oracle.rs:12-24`). Construction-4b: 6,290 cases, 2,904 targets; fresh draw 3,878 cases (`docs/native_geometric_reader_scope_repair_973.md`, "Results").
- **Experimental core:** fully synthetic. `ng/relative_language/data.rs:24-32`: 16 training names, 8 development names, 4 training verbs, 2 development verbs, 12 `Construction` templates (`:44-129`), facts `[(a,v,b),(b,v,cc),(a,w,d),(d,w,cc)]` in two slot permutations (`:192-199`); 128 training families × 16, 16 dev families per split. Other generators: `dependent_language/data.rs` (same vocab), `phrase_data.rs`/`span_data.rs`/`depth_data.rs` (8 names × 3–5 variants), `recurrent_text/data.rs` (16 adjectives, 16 nouns, 8 verbs, 8 adverbs), `ordered_state/data.rs` (16 subjects, 8 endings). The failing 2,304-row "independent neighbor" panel (`independent_neighbor_report.rs:22-49`) is drawn with a seeded PRNG from the *same* four-record/two-clause grammar with one-, two-, three-word names some of which contain the auxiliary `will` (8 shapes × 3 outcomes × 4 rotations × 2 prefixes × 4 directions). **Evaluation panels are generated by the same template families as training**, differing in name/verb lexicon or construction index (`make_split`, `data.rs:156-`). The doc's own words: "That panel was independent at first evaluation; it is now exposed development evidence" (README:195).
- `models/*.json` (gpt2-124m, smollm2, t5 tokenizer) and `tools/r4-softmax-trainer` are historical dense/Python references; the trainer README says "This Python/dense reference package is not the accepted main model" (`tools/r4-softmax-trainer/README.md:13-15`). Nothing in `native_geometric` reads them.

---

## 4. Scale, limits, artifact schema and lineage

**Sizes.** 15baec48: 17.4 MB JSON (`docs/evidence/…correspondence_973.json` `retained_model`); its base tables ≈110k rows / 760k associations of `(u32 token, i32 score)` plus postings. 60d19679: **63,640 bytes** JSON — the whole experimental "model" is a few boolean rule masks, a 2048-byte action table, two 8/16-byte action tables, ≤1024-row trigram tables, ~300 gate truth tables and their frozen parents' digests.

**Hard limits (code).** `Config::validate` (`mod.rs:255-270`): context 1..4096, candidates 1..256, pieces ≤65,536, rows ≤1,000,000, associations ≤8,000,000, postings/row ≤256. Artifact/snapshot ≤256 MiB (`training.rs:15, 469-474`). Generation budget 1..4096 tokens (`training.rs:1226`; API `MAX_OUTPUT_TOKENS` `native_capability_api.rs:303`), ingest ≤1 MiB (`:304`), 32 sessions (`:248`). Memory reader: query tokens ≤32, source offsets ≤16, candidates ≤256, features ≤262,144 (`memory_types.rs:55-80`). Relations: 16 records (`relation.rs:7`), read features 512, read rows 16,384 (`role_read.rs:8-9`). Experimental: 4 records, 16 words/record, 128 bytes/record, 256-byte prompt, 2 clauses, 3-word payload, 4 reads, 96/160 steps, 16,384 search nodes, ≤8 rules, ≤1024 role rows (`independent_neighbor_report.rs:37` acceptance `limits` matches the constants above).

**Storage/loading.** `.uor-models/` and `.uor-handoff/` are gitignored (`.gitignore:16-31`); the clone contains **no artifact**. The CLI loads with `bounded_file_read(path, 256 MiB)` → `Model::from_bytes` (JSON via `serde_json`, `native_geometric_cli.rs:574-577`; `training.rs:534-537`); deserialization goes through `ModelWire` with `deny_unknown_fields` and `TryFrom` → `validate()` (`mod.rs:667-837`).

**Identity and lineage.** `refresh_identity` (`training.rs:538-546`) clears both id fields, computes `uor_model_address` (UOR JSON structural address, `uor_addr::json::address_blake3`) and `artifact_cid = blake3(json bytes)`. Every optional block carries `parent_artifact`/`baseline_artifact`; `validate()` reconstructs the parent by removing the block (or restoring a witness's `previous`), recomputes its CID and requires equality, then recurses (`training.rs:663-684, 685-724, 788-821`). The experimental stack does the same with `parent: <ParentArtifact>` nested *by value* plus `parent_digest = blake3(parent.encode())`, `source_digest` (blake3 of the module's own source via `include_str!`, e.g. `relative_language/learning.rs:41-46`) and `data_digest` (`query_participation.rs:36-45, 151-157`). Lineage is therefore verifiable but *monolithic*: the 63 KB artifact literally contains its thirteen ancestors.

---

## 5. Controls

`Control` for the normal model (`mod.rs:289-388`) has ~95 variants; `Feature::admitted` (`:421-504`) shows what geometry controls do: `GeometryDisabled` keeps only kinds 0/1 (lexical n-gram), `ZetaDisabled` drops kinds 8–15 and 5, `H4Disabled` drops 2–7 and 16–25, etc. All other controls admit every feature and instead switch off a specific layer inside its runtime file (e.g. `MemoryDisabled` skips `state.collect` at `runtime.rs:684`).

**ExactIdentity** (experimental core; `relative_language/runtime.rs:198-203`, `dependent_language/runtime.rs:222-226`, `occurrence.rs:22-29`):
```rust
let yes = if c == Control::ExactIdentity { q.geometry.occurrences == k.geometry.occurrences }
          else { ordered::distance(m, &q.geometry, &k.geometry, ...)? == 0 };
```
i.e. replace the Hamming-on-prefix-products test by plain equality of the byte-prime sequences (exact string equality). Everything downstream (topology features, rules, roles, writer) is unchanged. "Full and ExactIdentity agree on 2,304/2,304" therefore means: **on this data the H4/Hamming machinery contributes nothing beyond string equality**. From the code we can say more: because the leaf map collapses `a/v, c/x, d/y, f/z` (`training.rs:130`), Full is a *coarser* equivalence than ExactIdentity — words like `van`/`aan` would collide under Full but not under ExactIdentity. That no disagreement was observed is a property of the authored vocabulary (the 24 names/6 verbs contain no such pairs), not of the mechanism. The `Query.occurrences` field is carried precisely so that exact identity is available; the geometric distance is redundant with it wherever it is used as `== 0`.

**Read-disabled / update-disabled** (`relative_language/runtime.rs:230-241`; `dependent_language/runtime.rs:315-319, 416-420`; `ordered_state/runtime.rs:165-168, 223-229`): `ReadDisabled` makes `route` return no selection (so no payload can be copied — zero correct answers is the *only* possible outcome, since the answer is always a copied span); `UpdateDisabled` keeps the first question as the second query (`state.query.clone()`), so the dependent read cannot target the intermediate entity. These controls are *ablations that remove the copy source*; they demonstrate that the output is a copy from the records, which is true by construction, not that a learned component was necessary. `ScorerDisabled` (`dependent_language/runtime.rs:437`) removes the learned rules and is the informative control for the rule tables; `StructureDisabled` in the phrase-order record passed 16/80 (`current-state.md:83`).

---

## 6. Dead / disconnected code

Measured with `wc -l` per module family (all under `ng/`):

| Family | Lines | Reachable from CLI/service/API/WASM? | Status in evidence |
|---|---:|---|---|
| Top-level runtime modules (kernel, memory, values, routing, witnesses, snapshot, durable_memory) | ≈28,000 | **Yes** (via `Model`/`Session`) | Retained model |
| Top-level training/fitting (`training.rs`, `*_training.rs`, `mixture.rs`, `memory_training/`) | 16,079 | Yes, via `train/fit-*` CLI subcommands and `examples/` drivers (offline) | Offline |
| Top-level diagnostics (`guarantees.rs`, `groundedness.rs`, `m1_profiler.rs`, `multi_step_reasoning.rs`, `workspace_coding.rs`) | 2,718 | Referenced only by `release_qualification.rs` capability strings; `multi_step_reasoning` is a hand-written DAG executor/Rust-template printer, not a model (`multi_step_reasoning.rs:98-350`) | Withdrawn scorecard helpers (`recovery-2026-09-08.md`) |
| Top-level `*_tests.rs` | 11,495 | test-only | — |
| `dependent_language/` | 24,229 | **No** | Experimental (60d19679); FAIL_INDEPENDENT_NEIGHBOR_TRANSFER |
| `relational_/dependent_/adaptive_/text_attention`, `recurrent_text`, `ordered_state`, `language_relation`, `relative_language` | 9,892 | **No** (parents of the above) | Experimental |
| `shared_core/` + `shared_core.rs` | 13,557 | **No** ("No retained-model dispatch uses it", `shared_core.rs:1`) | FAIL_CONTEXTUAL_TRANSFER_SMOKE |
| `addressed_attention/` | 10,795 | **No** ("Isolated primitive experiment; no retained dispatch change", `mod.rs:153`) | FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT |
| `hamming_policy/`, `hamming_refinement/` | 3,027 | **No** | "initialized, untrained" (`hamming_policy/mod.rs:1`); only `Metric` is reused by the experimental core |

Roughly: **~28k of 122k lines (≈23%) execute in the serving path of the retained artifact**; ~44k (36%) if offline fitting is counted; ~32k are tests/report drivers; the remaining ~46k lines are experimental stacks with no product entry point, two of which carry recorded failures. This matches the Sept-8 recovery's finding of "disconnected serving paths" and "toy baseline substitution" (`docs/integration/recovery-2026-09-08.md`, table rows 1 and 8), which the API now guards against by refusing empty model bytes.

---

## 7. Engineering quality

- **Tests.** 601 `#[test]` in `native_geometric` (60 `#[ignore]`d env-var report drivers), 1,053 in the core crate incl. `tests/` (71 integration files), 246 in `uor-r4-api`, 20 in the CLI/service files. Kinds: unit tests of kernel invariants, an allocator-instrumented census (`tests/native_geometric_allocations.rs`, 4,510 lines: `native_observe_predict_stays_allocation_free_through_evictions` etc.), the textual no-float/no-multiply source guard, artifact round-trip/lineage tests, and the large report drivers that double as training scripts. Many tests assert exact retention of previously generated outputs (regression-by-replay) rather than capability.
- **`unsafe`.** None in `native_geometric` (the single grep hit is the string `"unsafe_proposals"` in a JSON literal). Host code uses `libc` FFI for TTY/dir handling (`src/model.rs:1229-1284`, `src/chat.rs:1863`).
- **Allocation discipline.** `Session::new` preallocates the ring and a `Vec<Candidate>` with capacity `candidate_limit` (`runtime.rs:72-110`); shortlist maintenance uses `insert/remove/pop` within capacity; feature arrays are stack `[Feature; 26]`. The API wrapper allocates (`zero_heap_alloc_hot_path: false`, `native_capability_api.rs:251`). The experimental core allocates freely (`Vec` per word, cloning `State`), by design ("Bounded host allocation is deliberate", `circuit.rs:150`).
- **Determinism.** No RNG at serving; ties broken by token id (`runtime.rs:542-545, 932-935`); seeds appear only in training (`SourceRoutingConfig.seed = 1139`, `Draw` splitmix in reports). Checkpoints are JSON (`snapshot.rs`), artifact-CID-bound (`check_model`, `runtime.rs:232-239`). Reload-equality is checked in every report.
- **Build.** Toolchain pinned to `1.97.1` (`rust-toolchain.toml`); `cargo metadata --no-deps --offline` resolves the workspace (14 members). Core depends on git-pinned `uor-addr` (UOR-Foundation) and `blake3 =1.5.0` with a vendored `arrayref` patch (`Cargo.toml:96-121`); a fresh build needs network access to GitHub. WASM: root crate is `cdylib`+`rlib`, `native_wasm.rs` gates `wasm_bindgen` exports on `target_arch = "wasm32"`, non-wasm modules are `cfg`-gated in `src/lib.rs:38-85`, and CI runs `wasm-pack build --target web` (`.github/workflows/ci.yml:170, 262-274`, `deploy.yml:23-30`). **(inference)** Build and the fast tests plausibly pass; the ignored report tests require the author's local `.uor-handoff` receipts and would not.
- **Code health.** Heavy duplication (every layer re-implements validate/encode/decode/parent-digest); `training.rs::validate` is a 600-line chain of nearly identical peel-and-compare blocks; several files exceed 1,500 lines; one-line `json!` literals of 1–2 KB are common in report code. The quality of *provenance* engineering is unusually high; the quality of *abstraction* is low.

---

## 8. Assessment

**What the learning mechanism is.** Structurally, the retained model is a smoothed hashed n-gram/count model (naive-Bayes-style combination of 26 discrete features, 7 learned mixing weights) plus an exact key-value copy memory whose selection weights are small integer linear models, plus a set of discrete routing tables found by random-restart coordinate search. The experimental core is a symbolic program: exact word equality → boolean relational features → DNF rules of ≤4 literals → unique-candidate selection → copy. Its learners are greedy set cover, count-argmax, exhaustive enumeration over 16 choices, and maximal-generalisation of trigram keys. The only gradient-trained parts are ~300 two-input LUT gates fitted with Adam on a multilinear relaxation of ≤2,048 authored examples.

**Can it scale to general prose or reasoning?** Not in its current form, for reasons that are visible in the code rather than in the documentation:

1. *Representation.* The "geometry" is a fixed hash: `leaf = prime % 120` (`training.rs:130`) collapses a 4,354-piece vocabulary onto 120 classes and even collides English letters. The H4 state is a group fold of the window — an order-sensitive hash with 120 values. Distance is used only as `== 0`. Nothing learns the embedding; the 120-element group has no free parameters that could be adapted to language statistics (the only "learned" group elements are the ≤66 routing codes and landmarks). There is no mechanism for graded similarity, generalisation across surface forms, or compositional meaning beyond exact identity.

2. *Capacity and objective.* Each learned table is ≤ a few thousand integers, fitted layer-by-layer with everything else frozen, against exact-match objectives on authored templates. There is no shared representation across layers, no end-to-end objective, and no data path other than templates whose generators are in the repo. The base count model saw <1 MB of text and is evaluated on the same TinyStories prefix distribution. The recorded result on the one prose-like attempt (shared_core: held-out NLL improved 1.4%, accuracy 0/249) is consistent with a learner that memorises constructions.

3. *Search, not learning.* The learners (greedy cover over 4,047 masks; 16-way enumeration; ≤120 root proposals per coordinate with a 120 s cap) enumerate a finite hypothesis class chosen by hand for each layer. Every new phenomenon ("interior *will*" in a name) requires a new hand-designed feature (`FEATURE_NAMES` lists), a new table, a new layer and a new template family — exactly the failure mode the README itself diagnoses at line 136–138 ("no deterministic classifier using only that observation…"). This is the bottleneck: **the observation interface is a hand-authored finite feature set, and the learner cannot invent features.**

4. *Faithfulness of the "geometric" claim.* Because Full ≡ ExactIdentity on every recorded panel, and because the code path uses distance only as an equality test, the geometric layer is currently a more expensive implementation of string equality. The honest reading — which the docs largely share (README:192) — is that no predictive benefit of H4/zeta geometry has been shown anywhere in the retained or experimental models.

**Where it could go.** The engineering around exact, versioned, auditable memory with copy/arithmetic operators is genuinely careful, deterministic, allocation-free and float-free; as a *retrieval/copy substrate* attached to a real predictive model it is defensible. As the predictive model itself, the mechanism is a finite table/template matcher. Scaling it to open prose would require replacing the hashed features and per-layer greedy fits with a learned, shared representation and an end-to-end objective — which the project's runtime contract (no matmul) constrains but does not by itself forbid (e.g. integer/LUT networks trained offline). Nothing in the repository currently implements that; the two attempts that moved in that direction (`shared_core`, `addressed_attention`) are recorded failures.

**Unknowns.** The exact component list inside 15baec48 (artifact absent from the clone); the actual prose corpus size used for the retained base tables (the recovery run used ~330 KB; later lineage may differ); whether the WASM build of the current tree succeeds; runtime cost on real hardware (the M1 profiler was withdrawn as fabricated).
