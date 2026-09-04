# Contributing to R⁴

[AGENTS.md](AGENTS.md) is the full operating manual — gates, normative
invariants, the κ re-pin procedure, long-run discipline. Read it before your
first change. The authoritative issue order lives in the
[R4 Intelligence Completion Plan](docs/r4_intelligence_completion_plan.md),
with architecture and claim boundaries in the
[Geometric Intelligence Programme](docs/geometric_intelligence_programme.md).
The accepted #973 attention reference, completed learned-manifold/localization
results, parked intrinsic-replacement lane, autonomous-generation gate, and
native bridge result are frozen in
[ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md).
The current handoff is #973 V5's independently verified terminal
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` (result
`blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`;
verification
`blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`).
Integrity and fresh-language nonregression passed. Geometric prompt gain was
`0.03896945868086732` with `375/512` wins and beat V1 and pooled, but missed
the `0.04332169878499658` absolute floor. Its margin over independently fitted
plain delta was `0.023929811749894725`, below
`0.025341569256760274`, with worse own NLL; transport permutation passed.
Delta-overwrite attribution against independently fitted additive failed at
`-0.006512463228773413` and `234/512`. Preserve the original scoring-harness
failure as `NOT_RUN`: the corrected scoring recovery performed zero retraining
and zero optimizer steps, reused the frozen arms, and passed exact independent
replay. The binding law is retired at `STOP_WITHOUT_GENERATION`; do not retry,
generate, claim reasoning, or begin integer/table lowering from it. Ordinary
softmax and the qualified retained-attention baseline remain established and
the larger programme continues, but #954 remains blocked. See the
[#973 V5 record](docs/r4_predictive_block_delta_binding_prompt_capacity_973.md).

The retained baseline remains qualified as
[`R4RetainedLanguagePathV1`](docs/r4_retained_language_path_v1_973.md) and
records its sole layerwise-normalized candidate as
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. With every V1
budget fixed, `E @ [N(h) + (g / sqrt(2)) * (N(a1) + N(a2))]` used fixed `g=1`
versus equal-work `g=0`, zero new parameters/state, and one vocabulary matmul.
Candidate prompt gain was `0.02869802096506591` versus matched V1 at
`0.007331623694789724` (delta `0.021366397270276186`), with `339/512` wins and
own NLL `3.479876528760464` versus `3.6930405921095097`. Fresh held-out NLL/
top-1 improved to `3.712641167679153`/`31.661826%` from
`3.8850003882891597`/`29.728138%`, and state-off cost
`1.3495375636624845` NLL plus `20,595` correct decisions. It still missed the
frozen absolute `0.04332169878499658` and incremental
`0.025341569256760274` gain floors. Result, candidate, population, reveal, and
verification CIDs are respectively
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`,
`blake3:8d31e15c355aade1ccc2592dc5fb1caf14a5f056862621e7b467858569a1c1e4`,
`blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93`,
`blake3:079bee84db32513c5d6c0cb54cbff1e70b163902efa934d950204090985b3f5a`,
and `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.
The [binding #973 layerwise-readout record](docs/r4_layerwise_normalized_retained_readout_prompt_capacity_973.md)
contains the complete evidence ledger. Mechanics, replay, and all `13/13`
fresh-process comparisons passed, but generation, reasoning, lowering, and
geometry-native lowering are `NOT_RUN`; no coherence, H4-superiority,
exact-runtime, browser, or release claim follows.

The preceding separately frozen learned-associative successor completed
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` (result
`blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
independent verification
`blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`).
Geometric and pooled prompt gains were `0.0063767854348491465` (`299/512` wins)
and `0.010263234571452827` (`324/512`) versus V1 at
`0.006423652456300697`; neither learned arm met the absolute or incremental
capacity floor. The pooled arm nevertheless improved 247,920-decision fresh
NLL/top-1 from V1's `3.9036360153193317`/`29.628509%` to
`3.8737562215878296`/`30.042756%`, while state-off cost `0.3654355381796077`
NLL and `17,808` correct decisions. Retain that result only as a matched
non-geometric control. Geometry attribution failed against both pooled
(`-0.0038864491366036808`, `209/512` paired improvements) and deranged
(`-0.0002888663472835149`, `251/512`). Mechanics and verification passed, but
no prompt-capacity, geometry-advantage, generation, or reasoning claim follows.
That result motivated the terminal V5 campaign above. Do not tune or retry this
readout, widen it, or run generation from it.
Ordinary
causal R4/Spin Q/K/V plus stable softmax remains the bounded source-backed
attention baseline, while C1-SB5 paired-query binding fit `56/56` but reached
only `14/28` sealed. Row-swap equivariance was bit-exact; mean-query and
attention-off controls were each `0/28`. Products remained unopened and no
checkpoint/head/Rust/development stage followed. The rung is retired without
retry and establishes neither generation, reasoning, correctness, nor a
source-free runtime. Do not contribute a C1-SB5 retry, C1-SB6, or its downstream
artifact work; #954's final source-free terminal remains blocked behind #973,
with #955
downstream of positive correctness. Offline
teacher/compiler floats, matrix operations, and softmax are permitted; deployed
runtime remains exact and source-free. Hosted Pages is currently a static,
WASM-offline surface without a functioning chat backend/artifact lowering, not
product evidence or the active research gate.
This file is the short version.

## Deterministic source-only agent policy

Automated repository work follows the canonical
[`docs/integration/agent-execution-policy.json`](docs/integration/agent-execution-policy.json)
contract and the binding section in [AGENTS.md](AGENTS.md). Agents use a full
Git worktree from refreshed `origin/main`; sparse or hand-copied workspaces and
automatic retries are prohibited. Agents do not run or dispatch builds, tests,
probes, model work, or QA. The protected path runs a static text-only policy
guard plus ruleset transport; owner-operated manual release QA remains
separate.

## The loop

1. **Assign yourself the issue.** In this repo assignment means
   *actively-being-worked-right-now*, not "someday" — queued work stays
   unassigned so anyone can pick it up. Never start an unassigned issue without
   assigning it first.
2. Branch `issue-<n>-<slug>` off `main`. **No direct pushes to `main`.**
3. Work and produce the declared product/release evidence. Run a check only if
   the issue explicitly activates it.
4. Open a PR. Automatic QA remains dormant. The immutable repository ruleset
   forces a merge queue, so five instantaneous no-QA acknowledgements carry the
   reviewed commit through it; they are transport metadata, not test results.
5. **Close the issue with the evidence** — the numbers, the verdict against the
   pre-declared exit rule, and the merge commit. Then unassign yourself.
6. Follow-up work discovered mid-stream gets **filed as an issue immediately**,
   not left in a PR body.

## Decision-bearing checks (dormant by default)

Testing and QA do not run automatically. A product or release contract must
name the check, the decision it can change, fixture identity, outcome actions,
and resource budget. These are reference commands, not a pre-commit list:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py  # claims/docs only
```

Activate cross-target or certification work only when the product/release
decision requires it. For example, a release decision that names the WASM
boundary may activate:

```bash
cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib
```

Automatic QA is disabled. Legacy ruleset `19597522` still requires five status
names and a merge queue, so pull-request and merge-group events emit transparent
no-QA acknowledgements with those names. They perform no checkout, build, test,
audit, fuzz, WASM, or model work and must never be cited as PASS evidence. A
future product-ready release issue may activate only the QA required for that
frozen release decision. #940 tracks eventual administrator cleanup.
BDD, doctests, no_std, deterministic rebuild, kappa, Gate C, all-features,
WASM, fuzz, Kani, conformance, audit, and corpus-scale work remain dormant
unless the active product/release decision activates them. Source-free
attention probes, anti-recall controls, product transcripts, and serving
censuses are product evidence only when their programme stage calls for them.
Do not create a new test framework for routine confidence.

**Check the check.** Some ways these have silently lied before:

- `cargo … | grep …; echo $?` reports *grep's* exit status. Read
  `${PIPESTATUS[0]}` or run bare.
- After changing a public signature, `cargo clean -p <crate>` before the
  verifying run — stale test targets have "passed" pre-edit code four times.
- A κ test that finishes suspiciously fast has skipped. Confirm
  `/tmp/ref/out/model.bin` exists before trusting green.

## Execution-lane invariants

- **Active geometric intelligence path:** compiler-side floating point and
  allocation are allowed for witnessed chart construction. Source weights,
  `uor-matmul` intelligence projections, transformers, dense matrix
  intelligence, MoE, and sparse learned routing are not serving dependencies.
  Bind lexical codec, prime registry, recursive hierarchy, charts, payload,
  geometry, and decode identities; fixed inputs remain deterministic.
- **Frozen TLA/R4G1 runtime:** its multiplication-free, allocation-free,
  `no_std`, packed-format, and witness contracts remain in force. Do not weaken
  them when touching that lane.
- **Geometry is the route.** Kappa is canonical identity, not a tokenizer or
  semantic distance. Preserve the project shorthand `E8 = H4 x H4`; its exact
  code/serialization contract is the golden/Galois-coupled icosian pair
  `H4 ⊕ phi H4` with fixed basis, glue, and inverse witness.
- **Errors**: library boundaries return `Result` with focused error enums. No
  `unwrap`/`expect`/panic on recoverable paths.
- **No `unsafe`** in the portable runtime or the format crate.
- **Claim language**: `docs/formal_vocabulary.md` is normative, and
  `scripts/check_claim_wording.py` is the dormant automated wording check.
  Exact-equivalence wording
  needs a linked proof artifact.

## If your change is product research

The experiment must be able to change the next programme decision:

- **Preserve the established B0 reference.** #989 reached
  `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`: 99,362/446,342 (22.261404%)
  held-out table top-1 versus 24,163/446,342 (5.413561%) unigram, a
  +16.847843-point uplift, with bounded exact decoding and byte-identical full
  replay. This is statistical lexical prediction, not semantics, attention,
  geometry, correctness, reasoning, chat, or release evidence. The one
  permitted #953 intervention has since been accepted over this unchanged
  reference. #973 retained bounded prior-prefix, paragraph, and conversation
  mechanisms, retained a repaired bounded-global exact-spin witness, and then
  rejected the first natural componentwise-Frechet document placement: real
  8.367592% versus unchanged #953 12.221651%, with shuffled/permuted controls
  also stronger. The bounded gated-delta core later trailed plain delta.
  Direct-attention V2 is non-promotable because its comparators had fewer
  raw-manifold degrees of freedom; equal-manifold-budget V3 returned full H4 3/12 versus plain
  12/12 and isolated the connection/gauge seam. V4 then passed construction
  covariance but failed held-out functional binding at 13/24 for every main
  arm, with insufficient destructive-control separation. Do not tune any
  revealed fixture. The pinned HELM-D architecture audit uses the official MIT
  source at `7501deca8f413848bfef804be64ce874b72a3cd7` only as a credited
  architectural reference; no HELM checkpoint or upstream generation code was
  executed. The frozen ordinary full-decoder donor and gauge-equivalent
  ordinary-softmax parity in transported R4/Spin frames now pass. Intrinsic
  Lorentz V1 attempt 02 stopped unavailable
  before D3 on its covariance audit, with diagnostic NLL worse than donor and
  flat. The subsequent source-faithful learned-manifold V2 run was valid but
  negative: learned-Lorentz NLL `7.710618` failed donor retention
  (`3.667626`) and matched Euclidean parity (`4.483154`), although all three
  destructive controls separated. The 8/8-contract attempt stopped at its
  two-document preflight and rejected tangent readout (pooled normalized audit-MSE ratio
  `1.0643688804269025`). Provider-free-at-execution, source-backed native CPU
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation now passes using the
  credited HELM attention seam and UOR's pinned SmolLM2
  `HuggingFaceLlamaOracle` for
  embeddings, RoPE, residual/RMSNorm, MLP, final normalization, and the
  language-model head: 4/5 quality in both passes, 5/5 exact replay after
  deleting timing, exact all-layer audits with zero future reads, and donor
  reproduction. Its explicit opt-in, loopback-only dedicated native HTTP
  endpoint now passes the frozen eight-token canary with the same token sequence,
  decoded text, decision CID, persistent-state CID, all-30-layer exact audits,
  and zero future reads as the CLI. Dashboard wiring, native-readiness gating,
  and static/WASM isolation checks pass, but hosted Pages remains static/offline
  without a functioning chat backend/artifact lowering. The feature is disabled
  by default and does not change the default engine. The Q16 suffix trace
  student and `R4SoftmaxTraceStateStudentV1` are complete bounded negatives.
  `R4GroupAddressedRetentionDecoderV1CpuRecovery` has now completed its full
  construction run. Retained state is load-bearing on the disjoint construction
  validation partition,
  but the exact 3.17M-parameter, two-block, data/dose recipe did not satisfy its
  frozen full-decoder generalization criterion. Formal H4 specificity remained
  `NOT_EVALUATED`; diagnostic scrambled-transport CE was `0.033049` nats better.
  Preserve the qualified read/write component and the completed V1-through-V5
  evidence chain. The language-path decoder qualified, but its paired-H4,
  readout, learned-associative, and predictive block-delta promotion rungs did
  not. Do not reopen those frozen mechanisms. New #973 research requires a
  separately authorized contract; product-facing work uses the established
  #1017 `r4 generate` path. #1041 bounds its presentation to raw single-turn
  story continuation; do not add a source-backed history or multi-turn/chat
  adapter around that checkpoint.
  See the
  [#989 record](docs/source_free_table_baseline_989.md).
- **Preserve the GI evidence lineage.** GI-1 makes lexical/address state
  reversible;
  #952 found the reusable ordered-state defect; A1R/#967 repaired the fold but
  terminated `RETAIN_STATE_ONLY` after its scalar readout tied distinct
  candidate-relative states on 6/6 queries; A1P/#970's target-free paired-H4
  exact R4-heatmap gate exercised 14 classes across 36 decisions, had pure
  12/12 construction coverage, covered 10/12 validation decisions with a 10/12
  oracle ceiling, transferred 0/6 queries, and found eight incompatible heatmap
  classes. Downstream readout, selection-control, and placement work is
  `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS.
  That negative is bounded to the heatmap readout; fixed-zeta phases, ordered
  n-lets, exact `phi` radial transport, and typed geometry adapters remain
  structural. A1Q-L/#969 subsequently qualified one local causal R4/S3 path
  selector. GI-3/#953 implemented bounded provider-free library/CLI loop
  plumbing, but its rank-preserving relabel smoke terminated
  `REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` repaired
  the later natural agreement admission: I1/I2/ordered-sentence plus divisor
  recovered exact `{still}` then `{run,runs}` support under equal work, while
  adjacent-spin stayed consulted, truthfully reported physical presence, and
  remained non-admitting until the primary tier was empty. The one permitted
  four-arm run produced `still run` for both full-path prompts
  and `still runs` for both state-disabled prompts, with deterministic replay.
  The first frozen local same-object, order-sensitive candidate-placement
  preflight then reproduced 7/7 construction prototypes with zero class
  collisions, but real placement selected 0/2 intended candidates while the
  same-artifact placement-permuted control selected 2/2. Generation and replay
  were `NOT_RUN`; the terminal remains `REVISE_I1_GENERATOR_IN_PLACE`. A1Q-L2/
  #983 then formed pure construction classes on an independent population but
  transferred on 0/6 held-out decisions and closed
  `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`. A1Q-L3/#986 then closed
  `UNAVAILABLE_FRAME_OR_POPULATION`: its raw corpus reproduced, but the exact
  corpus-scale codec/pair commitment and complete same-frame lexical
  Cl(0,6)/SpiralCore operator map were unavailable. Gate 0, labels, selection,
  and #953 were `NOT_RUN`. That pre-reset handoff left #953 parked pending a
  fresh successor; B0/#989 and the later accepted matched #953 intervention
  supersede that action. A1Q-H/#973 then retained bounded Gate 0, paragraph,
  conversation, and repaired noncommuting-global witnesses are retained; the
  first natural document placement is negative at
  `RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT`. Its first bounded
  transported gated-delta core was also weaker than plain delta on its sealed
  construction fixture. The literal causal Q/K/V/O scaffold now exists, but
  equal-manifold-budget V3 rejected its current H4 parameterization while matched
  plain attention worked. V4 preserved construction covariance but failed its
  held-out function/control gates. #973's `HELM-D-R4` full-decoder softmax
  parity now passes. Intrinsic Lorentz V1 attempt 02 is unavailable before D3;
  the source-faithful learned-manifold V2 attempt is a valid non-D3
  construction-validation negative. Its 8/8-contract localization attempt
  stopped at the two-document preflight and rejected tangent readout.
  Provider-free-at-execution, source-backed
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its explicit
  opt-in, loopback-only dedicated native HTTP endpoint now pass. Dashboard
  wiring/readiness and static/WASM-isolation checks pass, but hosted Pages is
  static/offline without a functioning chat backend/artifact lowering. The Q16
  suffix trace student, `R4SoftmaxTraceStateStudentV1`, and observability rung
  are complete bounded negatives. #1014 established load-bearing ordinary
  attention and #1017 remains the working bounded generator. #954 C1-SB4's
  full-source record-margin successor failed at `70/126` fit and `35/63` sealed
  exact records; it stopped before Rust/checkpoint/product and must not be
  retried. C1-SB5 then fit `56/56` paired records but reached only `14/28`
  sealed and retired before checkpoint/head/Rust/development, with products
  unopened.
  #973's independently frozen `R4RetainedLanguagePathV1` is the qualified
  retained-attention baseline. Its paired-H4 addressing successor failed the
  prompt-capacity criterion and is frozen without generation; see the
  [machine result](docs/r4_paired_h4_prompt_capacity_result_973_raw.json). The
  old gated-delta,
  trace-state, intrinsic/readout, resonance, full-model recurrent-lowering, and
  exact-deployment lanes remain negative or parked. Do not scale or tune the
  rejected paired candidate, the partial direct readout, or the partial
  layerwise-normalized readout. The parameter-free ladder is closed, and the
  separately frozen learned-associative campaign completed
  `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` with independent verification.
  Preserve its pooled fresh-language signal only as a non-geometric control.
  Its independently frozen V5 predictive write/binding successor then stopped
  `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`; fresh-language and integrity passed,
  but capacity, geometry attribution, and delta-overwrite attribution did not.
  Preserve the harness incident as `NOT_RUN` and the scoring-only recovery as
  zero retraining. Retire the write/binding law and
  `STOP_WITHOUT_GENERATION`. #973 still blocks GI-4/#954's final source-free
  terminal, with GI-5/#955 downstream of positive correctness.
- **Exercise the accepted route path.** #953 geometry runs before token choice
  and emits admitted support and its fixed-point radius trace. The completed
  #973 localization kept its frozen split, trace identity, score-paired common
  tensors, separate score/aggregate metrics, and exact work ledgers; it rejected
  tangent readout. The qualified autonomous-generation gate used actual causal
  R4/Spin transport and reports its provider, weight, decode, cache, and replay
  provenance explicitly. The bridge preserves that exact policy and provenance;
  the trace/compiler rung must preserve them too.
- **Use the smallest falsifier.** `HELM-D-R4` first requires donor/reference
  parity, then splits every learned head into R4 blocks, encodes exact cumulative
  Spin/H4 frames, transports K/V to the query frame, applies unchanged ordinary
  stable softmax and value aggregation, maps back, and applies unchanged `W_o`.
  That gate now passes, including deterministic donor/R4 replay, zero future
  reads, and a frame-permutation liveness control. Parity is not geometric
  advantage. Intrinsic Lorentz V1 attempt 02 failed its construction covariance
  audit (`9.121400701417315e-08` versus `1e-08`) and therefore terminated
  unavailable with D3 sealed; its curved NLL was diagnostically worse than donor
  and flat. Learned-manifold V2 later completed validly but failed donor
  retention and matched Euclidean parity while all destructive interventions
  separated. The 8/8-contract attempt stopped at its two-document preflight,
  rejected tangent readout, and retained score only as a future parked seam. Actual paired-E8 hierarchy/
  fiber/torsion binding remains `NOT_IMPLEMENTED`. The prior smallest falsifier
  was provider-free-at-execution, source-backed
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
  generation with the credited HELM attention seam and UOR's pinned SmolLM2
  `HuggingFaceLlamaOracle` decoder path; that gate now passes. The opt-in,
  loopback-only dedicated native HTTP canary through the identical policy also
  passes. Dashboard wiring/readiness and static/WASM-isolation checks pass, but
  hosted Pages is static/offline without a functioning chat backend/artifact
  lowering. The paired-query C1-SB5 source-backed correctness falsifier has now
  closed negative at `56/56` fit and `14/28` sealed; its controls do not rescue
  promotion, and neither C1-SB4 nor C1-SB5 may be retried.
  Pinned-source provenance, donor reproduction, and transported-R4 parity are
  recorded in `docs/helm_d_r4_softmax_decoder_973.md`; V1 and V2 outcomes are in
  `docs/intrinsic_lorentz_r4_attention_973.md` and
  `docs/helm_d_learned_manifold_r4_construction_973.md`, with localization in
  `docs/helm_d_score_centroid_localization_973.md`. Intrinsic/readout
  alternatives, resonance-based softmax replacement, full-model recurrent
  lowering, and exact deployment are parked; #954's final source-free terminal
  remains blocked by #973.
  The PASS does not establish geometry advantage, softmax removal,
  source-free/table-native serving, correctness, reasoning, frontier quality,
  release readiness, or a static-WASM decoder.
  See the
  [generation record](docs/r4_softmax_reference_generation_973.md) and
  [compact aggregate](docs/r4_softmax_reference_generation_attempt_01_result_973.json),
  then the
  [native bridge result](docs/r4_softmax_reference_http_bridge_973.md).
- **Calibrate deterministic offline compute before a substantial run.** Benchmark
  the scientifically eligible CPU-BLAS/Accelerate and, only when the contract
  permits it, MPS backends across representative thread and worker/process
  counts. Select the measured-fast stable plan that preserves canonical results
  and fits memory; do not silently default to one core and do not blindly max
  workers when contention is slower. Record hardware, backend/BLAS provider,
  intra/inter-op threads, worker processes, utilization, representative-step
  timing, and equivalence evidence in the run contract. Partition corpus work by
  content identity with canonical ordered reductions when workers help. CUDA is
  eligible only when an issue explicitly places it in scope. None of these
  offline accelerators changes the CPU/table-native deployed-runtime target.
- **Use non-degenerate anti-recall controls.** Run the active issue's
  predeclared matched controls under equal information and work budgets. The
  historical current-only, additive-summary, factor/count-only,
  ordered-state-permutation, hierarchy-disabled, and exact-recall-only arms
  remain examples only when that issue explicitly activates them.
- **Preserve unseen global context.** Exact route identity remains separate
  from transported trajectory, hypersphere/window summaries, shared-factor
  retrieval, resonance, and accumulated Hopf phase. Those fields are storage,
  diagnostics, or controls until A1Q qualifies them for semantic scoring; an
  exact kappa miss still cannot collapse to a suffix-only default.
- **Keep teachers offline.** Teacher labels/comparisons begin only after a
  source-free report freezes and are never substituted for product output.
- **Include free-running output only when GI-3 or later is in scope.** Attention
  evidence at GI-2 is anti-recall candidate selection, not readable text.
- **Pre-declare the exit rule, the null baseline and the falsifier** before you
  run anything. Write them in the issue.
- **Compute the reachability ceiling first.** If the ceiling is below the effect
  you are hoping for, do not launch. This is a five-minute calculation that has
  invalidated four-hour runs.
- **Activate only a decision-bearing instrument.** Testing/QA stays dormant
  until a product or release contract names the instrument and its action.
- **Pre-declare what each outcome causes.** If the positive and negative branches
  lead to the same next action, the run has no decision value — drop it or
  redesign it.
- **Record negatives.** They are kept, not discarded, and several have redirected
  the whole programme.
- **Make sure your instrument can fail.** Assert the control arm is
  non-degenerate before comparing. An all-zero result across every arm is a
  harness bug until proven otherwise — seven have been found here.
- **Do not turn a negative into infrastructure by default.** A failed operator
  stops or returns to representation design; it does not automatically justify
  a larger corpus, graph format, proof lane, or benchmark suite.

For any run measured in hours, paste the run contract into the issue before
launching:

```
metric to move:       <name, current value>
reachability ceiling: <arithmetic, with the numbers it came from>
instrument + verdict: <which cheap test, what it must report to proceed>
exit rule:            <threshold, pre-declared>
if positive:          <the next action>
if negative:          <the next action, and it must differ>
cost estimate:        <wall clock, and what else it blocks>
```

## Documentation

Records live in `docs/` as `<topic>_<issue>.md` and are **appended to, not
rewritten**, when a later result revises them — the history of what was believed
and when is part of the record. If your work changes a claim in `README.md`,
`docs/RESEARCH.md` or another record, correct it in place and say what revised
it.

## License

By contributing you agree your contributions are licensed under the MIT License.
