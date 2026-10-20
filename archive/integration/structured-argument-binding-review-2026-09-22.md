# Principal review: retain structured binding, repair the learning and identity seams

September 22 UTC, 2026. Review of draft PR #1344, submitted `9cb02f6e77eeae6d026e830f7128625c135f7e26`, based on accepted PR #1343/main `6ee9d0f5`. This review and its receipts distinguish the original partial, the corrected draft, and the still-uncompleted language milestone. Live GitHub owns delivery status; a corrected draft is not accepted main runtime.

**Corrected draft source:** [PR #1344](https://github.com/UOR-Foundation/uor-r4/pull/1344), commit `1df229178c6b56ae6dbf95273c9e4eae3282a66e`. The corrected exposed replay returns 24/24 development and 24/24 exposed answers, cue-role 48/48 and span 48/48. All 18 focused module tests and 22 runner tests pass, with one legacy artifact test explicitly ignored. Independent audit verifies all 120 rows plus five intervention outcomes: 686 events, 811 snapshots, 167 successful reads and five rejected repeat attempts, with zero source/frame/oracle/continuation errors. The coherent object edit changes Fen through three reads to Cedar Annex through two; required terminal removal gives two reads then silent Unresolved. This source is kept in the draft; the accepted main runtime remains PR1343.


## Decision in the whole project

Keep the bounded semi-Markov argument representation. The partial contains a useful removal of the old fixed argument order, but it does not exercise ordinary forms or establish geometric advantage. Complete the same [argument-binding milestone](deepseek-structured-binding-completion-step-2026-09-22.md) through the corrected session. Do not restart architecture, promote the partial because it compiles, or spend another run only fitting the old renderer.

The current native programme still needs an integrated path from observed text through contextual roles, exact addressed evidence, learned dependent control, computed results and lexical generation. E/S assets being loaded does not mean that their prediction path supplies these answers. Q8 computation is a retained component, not yet consumed by this text session. Multiple scopes, correction history, broader prose and executed Rust remain subsequent capabilities. Consumer-machine D0-b and complete-task energy remain unqualified.

## Independent original result

The [original saved-data audit](../evidence/structured-argument-binding-principal-review-2026-09-22.json) and adjacent script reconstruct the submitted result. Roots `structured-argument-binding-1..3` contain incomplete attempts/model files; roots 4 and 5 have valid five-file seals and no missing, unlisted, byte-size or BLAKE3 errors. Root 5's three source hashes match submitted `9cb02f6e`. Preserved executable `/tmp/uor-pr1344-original-competitive-reader` SHA256 is `545223e74bef210a42b9f9c1b107de374aafa1e8999d0a567614d12387f57f0d`. The original files are retained.

All 120 saved row flags, expected answers and displayed outputs are consistent. Development/exposed primary score 23/24 each; membership 20/24, two-read cap 18/24 and NoRead 0/24. There are 669 events, 794 checkpoints, 160 successful reads and four rejected repeat attempts across 125 nonerror outcomes. Internal event/restore consistency does not imply that the semantic intervention succeeded.

Three corrections materially change the interpretation:

1. **Byte alignment was exercised.** All 48 saved world clauses, 48 fitting clauses and 120 row questions have nonempty byte lengths that independently reproduce the actual original bytes. Captures use lexical byte keys. The missing demonstration is a successful join across different BPE boundaries, not all use of aligned keys.
2. **The single answer loss is cue-role classification.** The exact span score is 0/48→48/48, while cue-role accuracy is 47/48. `Mara project follows Ivo` is classified as role 2/Emit rather than role 3/Continue. It emits `Ivo` instead of following to `Atlas`, after two development reads or three exposed reads. Perfect argument extents alone are not complete semantic prediction.
3. **Changed-source and cycle controls mutated only tokens.** Their old text and byte lengths were retained. The clause fell into token-key fallback while the request kept a lexical-byte key; both returned zero-read Unresolved. These were inconsistent observed inputs, not a negative experiment on a coherent changed fact. The recorded false intervention flag was accurate; disabling its failure exit did not validate the run.

The old authored forms, all leading-space conventions, repeated evaluation and absent H4 run remain explicit. No new final draw or general syntax evidence was delivered. See the [original submission](https://github.com/UOR-Foundation/uor-r4/blob/9cb02f6e77eeae6d026e830f7128625c135f7e26/docs/integration/structured-argument-binding-partial-2026-09-22.md) for its unchanged historical account.

## Concrete corrections to the draft

The source review found real invariant regressions among the failing tests. Conflicting role actions had become last-write values and conflicting goals majority votes. Overlapping gold spans and cue lengths outside the role scorer's support were accepted. New feature/group tables lacked loader validation. These now fail explicitly rather than changing semantics silently.

Identity now has tagged lexical-byte versus explicit legacy-token domains. Supplied malformed alignment fails; legacy token-only mode is explicit. The runner verifies concatenated token bytes against original text, not only equal total lengths. A source edit updates text, tokens and alignment together and validates retokenization. The cycle/changed-answer instruments are restored; their complete failure reports are sealed before a nonzero return, so failures remain inspectable without being passed off as successful execution.

Restore now binds each captured key to its actual source object and compares lexical identity across followed transitions. Previously a forged `TextCapture.key` could survive validation and change a later query. Model/session versions are 5/3. Original-question authentication is still outside this snapshot contract: internal source-chain consistency is not a guarantee about an omitted external request.

Decoder accumulation uses i64 over bounded i32 potentials, avoiding clipping. Cue support is explicitly limited to four tokens while subject/object/background cover a maximum 24-token clause. Background is **fixed zero**, because the current trainer does not learn it; nonzero background columns are rejected. The cue classifier receives convergent supervised passes rather than one pass. This is still staged span fitting then cue-role fitting, not a joint structured semantic objective.

H4 code maps are initialized before feature fitting. A code move must strictly improve the same decoded objective while retaining the incumbent. This removes the earlier reset from all-zero/default codes to all-one codes after fitting. It does not establish an effective optimizer or a geometric advantage; the actual comparative experiment remains outstanding.

Runtime tests retain their exact long/multiword outputs, source extents and restore checks using a named test-only boundary-potential model. Separate fitting tests exercise learned behavior. A tiny one-token training set need not generalize to every long answer for the runtime contract itself to be tested. Additional tests exercise genuine malformed-input risks, independent lexical identity domains, a different-token/same-byte dependent join, and a noncommuting interval/direct-product identity. The final [checks receipt](../evidence/structured-argument-binding-principal-checks-2026-09-22.json) and [corrected saved audit](../evidence/structured-argument-binding-corrected-replay-audit-2026-09-22.json) own actual execution outcomes. These repairs do not complete the new language fixture.

## Mathematical assessment and better next implementation

For three single-use roles the mask state has eight possibilities. A bounded semi-Markov decoder can select nonoverlapping subject, cue and object spans in any order while covering the remaining tokens. That output space directly addresses the old representation limit. It does not solve role semantics or language generalization automatically. [Semi-Markov models](https://aclanthology.org/D07-1067/) justify segment-level output/scoring as a coherent decomposition.

The next scorer must have one declared objective. For example,

`S(x,y) = sum(named span potentials) + sum(uncovered-token background potentials)`.

If background is learned, singleton background transitions avoid an unspecified latent segmentation. The structured update is the feature difference actually scored by the decoder, `Phi(x,y_gold) - Phi(x,y_pred)`. Alternatively keep background explicitly fixed. A staged cue classifier may remain useful if its exact output and full-answer quality are measured separately; a common structured loss or alternating fit is an available improvement, not a requirement to rewrite everything. [Collins's perceptron formulation](https://aclanthology.org/W02-1001/) supplies the relevant consistency criterion.

The ordered finite group here is the 120-element **binary icosahedral group 2I**, represented by canonical H4 roots; it is not the whole H4 reflection group. Associativity yields

`P_j = g(x_0)...g(x_{j-1}); inverse(P_i)*P_j = g(x_i)...g(x_{j-1})`.

An interval summary is exact as a group product but finite and collision-prone. The present H4 arm adds group features to the categorical scorer, so it is a **hybrid**, not a pure isolated geometric model. `G_INTERIOR` in the categorical arm is an unordered bag; an ordered H4 product supplies extra information. Give the categorical comparison explicit bounded order features too, or separate the new order observation from the representation claim.

Precompute interval potentials once per clause. Rebuilding features inside every position/mask/length/role loop repeats interior scans, allocations and sparse lookups. With n tokens, maximum segment length L and R=4 roles, decoding can cost O(8*n*L*R) after potential preparation. Preparation, fitting, group initialization and all memory traffic still count. At n=24 only 300 intervals exist; this is a useful local simplification, not a new optimization campaign. H4's lazy float table initialization and absent explicit table fingerprint must be accounted for before numerical-kernel or exported-geometry claims.

Identity remains separate: preserve raw byte/token extents for source/provenance/output and a declared whitespace-boundary lexical key for joins. Never convert exact names into semantic distance, silently lowercase them, or erase internal spaces/prefix distinctions. [Tokenizer offsets](https://huggingface.co/docs/tokenizers/en/api/encoding) refer to original input; individual token content and inserted prefix-space conventions are distinct. This adapter verifies its actual byte-roundtrip policy. Arbitrary tokenizer normalization/alignment is not thereby solved.

## Coherent next step and roadmap

DeepSeek should finish the same complete milestone: ordinary interrogatives, reordered arguments, trailing nonanswer material and actual cue/name overlap through the native loaded session, including full one/two/three-read answers and exact identity across real BPE differences. Demonstrate support first on a few transparent examples, then learn and execute the task. Preserve exposed panels as regressions; choose a fresh final only after selecting the model. Retain an order-aware categorical comparator and a coherently trained H4 hybrid on the same support.

After that interface works: durable role/scope/corrections and consumed Q8 results; integration with E/S prose and executed Rust; then useful complete artifact/API and measured machine efficiency. Structural banks, retained Hopf fiber and signed H4/Spin remain tools for observed interference or orientation distinctions. Paired-H4/icosian E8, S7 and harmonic/scalar fields stay available when they provide a concrete information, sharing or cost advantage. They are not a mandatory dimensional ladder.

The owner's scalar-field intuition has a precise bridge here: learned role potentials are already scalar functions over finite span/context states. Harmonic coordinates would change their parameterization and inductive bias. That could be useful, but orthogonality does not provide learned roles, unlimited independent memory or exact occurrence identity. None of those higher-dimensional choices repairs inconsistent fitting, a missing argument candidate or stale source bytes by itself.

## Systemic corrections and resources

A partial can be valuable without being merged. Keep PR #1344 draft until the language milestone and contracts are actually executed; publish this review/roadmap separately on main so the project state remains accurate. Extend the existing `ob_serve` adapter; do not drop all-arm/every-phase records or soften failed causal checks to make an experiment finish.

The shared ledger's allowance grew from 316,700,000 to 333,700,000ms, but its cumulative debit remained unchanged. The independent known interval is 1,577,000ms from worktree creation to commit, excluding earlier recovery/later delivery. We record **2,100,000ms as a retrospective engineering estimate**, including 523,000ms estimated preparation/delivery, not measured model time or a proven upper bound. The two completed model-report timers total 0.658966791s and cannot replace engineering accounting. Current principal work is projected/charged separately. Exact prior prospective receipt was unavailable; standing owner authorization covers necessary local extensions, not false accounting.

Only identified inactive debug dependency archives/metadata were removed. Unique models, partial and negative reports, all executables, research, AI conversations/knowledge and Downloads are retained. Final cumulative and physical-space figures belong to the checks/resource receipt. No paid compute or energy claim.
