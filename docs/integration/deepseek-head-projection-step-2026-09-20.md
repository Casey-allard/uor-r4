# DeepSeek execution prompt — export the retained head gain through the existing ternary path

Work on **UOR-R4 Geometric Language Model**, `UOR-Foundation/uor-r4`. Refresh origin/main beyond audited merge `d15360527f7c69ac8b83eef0bbd5839b87c26f02` (PR #1306). Read this prompt and `docs/integration/readout-result-review-2026-09-20.md` completely, then the current plan/state/policy. They supersede the last receipt's recommendation to widen the feature map.

**Deliver one bounded post-training projection experiment.** Reuse the retained floating output weights and frozen integer features. Compare Q0, the existing ternary quantizer, against QG, one activation-aware dyadic ternary calibration defined below. Export/reload through the existing CPL2 serving path and measure actual losses, controls, greedy generation and cost. Complete the experiment; do not stop after a module, proposed study or fixture. No new Adam fit, wider hidden layer, new precision, new corpus or prefix refit is part of this run.

## 1. Authority, source and preservation

Read AGENTS.md, DECISIONS.md D0-b/D1/D2, the execution policy, canonical plan, current state and latest review. Owner instructions override older stricter policy wording. Rust is required for model preparation/calibration/artifacts/evaluation/inference. Offline floating point and matrix products are allowed. Serving retains bounded ternary add/subtract/shift/table operations with no multiplier instruction or floating-point numerical kernel. Keep the geometric architecture goal and frozen R4G1 contract distinct from this local emission component.

Use an isolated full `codex/…` worktree from refreshed main. Preserve the owner checkout, all historical research, negative candidates and every sealed attempt. Refresh #973/#820/#963/#964; assign work actually active. Do not delete, replace or write derived files inside old report roots. Claim each new report directory exclusively immediately after argument validation, before model loading; seal and verify its complete member set.

The retained root is `/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`. Verify these bytes before use:

| File relative to root | SHA256 |
| --- | --- |
| `attempt-1/prior_realtext.cpl2` | `cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00` |
| `attempt-1/prior_realtext.ckpt` | `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2` |
| `readout-diagnostic-2/result.json` | `687cf58696f221f0faddc8e5f7e61c15502198aab14abe6a06bdf60634dce644` |
| `readout-diagnostic-2/hard/empirical.cpl2` | `d8a1fb14457481d48faa47b3f0794f42c6ca265b8de11b40af795fa03cd0215c` |
| `readout-diagnostic-2/hard/smoothed.cpl2` | `8a0950332b6f3638d8e46e797c6dafb7dda1a20cd2cd84901cdad4f2591447f8` |
| `readout-diagnostic-2/float-head.bin` | `84f28bbd04b9d14cdedbfc2f9c13c24ad27f271d92401f467da5cf960a7bb1c0` |
| `prefix-recovery-2/recovery.json` | `15ab737cefd758d707e0f74fa18881c75bdca9f93faf00ba1d3c688fa4abf0d8` |

The actual derived V4096 tokenizer digest is `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`. Source tokenizer digest is `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c`. Use the same verified HfBpeTokenizer derivation and pinned `inputs/docs` corpus commit `e9c04e80`, existing corrected indices in `derived-corrections-1`, exposure in `eval-replay-2`, and references in `prefix-pilot-2`. Do not reconstruct the corpus from the changing working tree.

Useful source: `learner/{lowbit,prior_learning,realtext_support,prefix_artifact}.rs`, `bin/readout-diagnostic.rs`, and the retained evaluator. Reuse the common feature/forward/evaluation helpers; do not copy another full learner. Scope the new code to calibration, a small runner/replay mode and the touched reporting defects.

## 2. Complete projection before execution

Refresh `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json` and the established storage accounting receipt/tool. Review snapshot is **166038565 /167600000 ms**, remaining **1561435 ms**. Preserve all prior charges, including the latest conservatively recorded 6600000 ms; its full measured-wall provenance is unverified. No refund or duplicate charge.

Propose **3600000 ms** total: 900 s implementation/build/focused tests; 450 s identity correction and frozen replay; 450 s fit-feature Gram construction, row probe and bounded projection; 1200 s common-objective/development/control/generation/cost evaluation; 600 s report/delivery/checkpoint/stop reserve. Before consuming work, record the necessary standing-authorized **+2400000 ms** limit extension, proposed limit **170000000 ms**, headroom **3961435 ms** at this snapshot. Refresh actual values and revise the complete projection before exceeding it. Recording an increase just before charging already executed work is too late.

One worker, at most four Cargo jobs, peak RSS <=8 GiB, <=256 MiB new retained/temporary model/report data plus <=1 GiB incremental reused build output. Preserve the 128 MiB storage stop margin using actual accounting, not `git status`. No paid/external compute, corpus download or destructive cleanup. Record UTC boundaries and monotonic elapsed durations per nonoverlapping phase, including failed attempts/builds/tests; do not charge the entire reservation. Save a row-index/calibration-state checkpoint if stopping mid-projection; restarting must validate parent, floating weights, Gram, algorithm configuration and input identities. This is deterministic calibration continuation, not an Adam-resume implementation.

## 3. Close only the evidence gaps needed for this experiment

Do not rerun the prior head or prefix training. Preserve the actual previous teacher λ=(0.8,0.7) when reconstructing old objectives. The prescribed consumed-reference λ=(0.7,0.5) was not used by S/F; disclose that deviation. It need not be retrained to perform this export experiment.

Re-export retained E/S CPL2 files into the new root with the real derived-tokenizer digest instead of `[7;32]`. All numerical fields must be identical. Store old/new hashes and the sole metadata change. Verify complete-panel integer logits before and after reload, then evaluate the reloaded artifacts. Also bind the raw float file (exactly 4096x128 little-endian finite f32 values) to the frozen parent, tokenizer, dimensions, F=10, bias, actual teacher, source/binary and data identities in the new manifest. Evaluate it with the recorded floating scorer; do not round its intermediate logits to integers. Reproduce E/S/F development means within a declared 1e-8-bit tolerance, or stop and diagnose the mismatch before claiming a new result. Preserve raw old files.

Reconstruct the missing common-objective matrix for parent/E/S/F while doing the shared evaluation: true-label fit CE, empirical-target CE/KL, and actual smoothed-target CE/KL on the same 257113 fit occurrences. Evaluate each unique `(prev,cur)` once and weight by occurrence count; check equality to direct occurrence scoring on a small fixture. Save teacher entropy, mass and target identity. E-CE and true-label fit CE must agree mathematically on this whole population. Compare different heads under the SAME teacher; never compare E-KL to S-KL as a common-target score. The old floating KL implied by saved results is 2.8669577896 bits. Do not fabricate missing historical fixed-population curves.

Fix the small reusable screen helper: practical gain must be against its declared baseline, smoothing attribution must use paired S-versus-E, and the old conditional floating gate must use S's teacher KL. Cover a fixture in which both heads beat the parent but S does not beat E. Existing decisions stay attribution FALSE and floating RUN; this repair requires no rerun. The unseen-context teacher helper must use the existing normalized backoff if reused outside seen fit contexts; verify complete mass including missing one/two-token contexts. The original fit used seen contexts, so that issue does not invalidate its recorded outcomes.

Prefix loader bounds and generalized RDO1 continuation remain explicit #964 limitations unless a directly touched dependency requires a small correction. Do not spend this run on a broad loader, optimizer or exact-table replay campaign. Already verified prefix recovery need not run again.

## 4. Freeze the causal boundary and construct Q0

Keep parent `e_old`, `e_new`, all their row shifts, bounded ReLU/normalization, bias codes/scale, `elements`, vocabulary, F and argmax tie rule byte-identical. The feature dimension stays 128. Only the output head changes. Features come from the unchanged CPL2 feature primitive, never floating embedding masters.

Let W_F be the saved floating head. Construct **Q0** by calling the existing `TernaryLinear::quantize(W_F,4096,128)` exactly once. Preserve its original packed codes and shifts as a candidate. Validate its complete arithmetic envelope with the existing `PriorCore::validate`; reject a violation rather than silently clamping the quantizer. Export a real CPL2 carrying the actual tokenizer identity, reload and verify full numerical equality. Do not feed Q0's effective integer weights back through the quantizer, because re-quantization could change scales.

Q0 asks whether the retained better floating solution already converts usefully under the current export rule. No new gradient fitting or target distribution is involved.

## 5. Construct exactly one activation-aware ternary candidate QG

Calibration population: the same first **4096 consumed windows /257113 n−1 targets** recovered from the legacy permutation. No tune/dev features, labels or trajectories participate in code/scale selection. Aggregate fit context occurrences with their exact multiplicities, preserving the reset/PAD policy. Sorting context keys makes accumulation/restart deterministic.

Compute the **uncentered** second-moment Gram:

`G = (1/N) sum_c n_c h(c) h(c)^T`.

Accumulate the upper-triangular numerator using checked wide integers from integer features and occurrence counts, mirror exactly, then convert to f64 and divide by N. Bind the numerator, population and feature-parent digest. Do not center features: with frozen bias, that would discard mean-output error. Do not invert G, add jitter, regularize, truncate rank or whiten features. G may be singular without invalidating the objective.

For floating row w, ternary row q and scale a=2^s, minimize the finite calibration surrogate

`J(q,s) = (a q − w)^T G (a q − w)`.

It equals occurrence-weighted squared integer-score reconstruction error. Multiplication by 2^(-2F) gives natural-logit error; a vocabulary average is another common constant. Define the reported units explicitly. Bias is frozen and cancels. This is **post-training calibration of hard weights**, not a claim of no learning or a direct CE optimizer.

Use the following fixed search without a hyperparameter sweep:

1. For each row, let s0 be Q0's existing shift and sE the retained empirical head's shift. Form the sorted unique set `{s0-1, s0, s0+1, sE}`, removing negative and invalid-envelope values without unsigned underflow. Retain the original Q0 row as an explicit candidate. With the current d128/norm_bits6 envelope, the conservative output-shift check admits shifts through 15; derive/check this from source rather than assuming observed activations bound inference.
2. At each valid shift initialize nearest ternary coefficients `clip(round_away_from_zero(w/2^s),-1,1)`. At sE also include the empirical row as a second seed if distinct. Save exact seed and tie semantics.
3. For each seed perform exactly **two coordinate sweeps**, first j=0..127, then j=127..0. Let e=a q−w, g=G e. For alternative code b in {-1,0,1}, delta=a(b−q_j) and exact surrogate change `dJ=2 delta g_j + delta^2 G_jj`. Keep the current code on a tie; among equally improving alternatives use ascending code order. Accept only a strictly negative minimum, then update `e_j += delta` and `g_k += delta G_kj` for every k. No convergence loop, restarts or extra sweeps after viewing development results.
4. Independently recompute each candidate's full `e^T G e`. Choose the smallest fit J only. Prefer untouched Q0 on a numerical tie, then a fixed shift/seed ordering. Define comparison tolerance in the manifest and use a separately declared numerical verification tolerance; do not use development CE to resolve it. Keeping Q0 in the candidate set makes QG's fit reconstruction error non-increasing in exact arithmetic. Verify the selected sum against Q0 and direct reconstruction on a small fixture.
5. Serialize the selected packed ternary codes and row shifts directly using `TernaryLinear::from_packed` into CPL2. Assert all other fields match parent, validate global arithmetic bounds, export, reload, and verify full-panel integer logits. Same 2-bit storage/code set and serving primitive; no new integer multiplication, float, dense floating fallback or lookup response cache.

Probe the first 64 rows for projected calibration time before the complete search; retain/reuse those deterministic results rather than rerun them. Charge their cost. If the projection needs more time, record a revised complete local allowance before continuing; do not quietly change the algorithm/dose or make a development-selected approximation.

## 6. Evaluate both candidates once and make the scoped decision

Use the exact existing 36-document /288-window /17342-target development panel, aligned by full occurrence keys. Preserve parent/E/S/F references and add Q0/QG. Save all per-occurrence losses, document sums/counts, target identities and panel metadata so every reported interval can be reconstructed. Save actual reference exposure and smoothing parameters: 512 conditional windows, 4096 conditional windows and full-fit counts are different comparators, all sharing the full-fit marginal. No count result is a ceiling or mandatory admission target.

Report the fit common-objective matrix for Q0/QG too, fit squared-score error against F, true-label tune CE on the existing first/last windows of all 38 tune docs as descriptive only, and dev micro/macro CE. Report row-scale/code changes, zero fractions, per-row and total reconstruction errors, and actual artifact bytes. Nothing on tune/dev selects calibration coefficients or an extra pass.

Predeclare **QG as the primary candidate**, E as the incumbent. Practical screen: `CE_E−CE_QG >=0.10 bits/target` and its nominal 95% paired document-bootstrap lower bound >0. Separately report Q0 under the same practical screen. Method attribution: `CE_Q0−CE_QG` has a paired lower bound >0. Use the existing 2000 draws /seed `0x12345678`, ratio-of-resampled-loss-sums, with exact document alignment. Do not declare success from the minimum of two losses without naming which prespecified comparison passed; these are open-development screens, not fresh significance claims.

Retain contextual controls: frozen bias/context-disabled output invariant to context permutation; same paired `(prev,cur)`-association permutation with target occurrences unmoved for E/Q0/QG, using the per-document/PAD occurrence-permutation semantics in `prior-frozen-evaluate.rs`. Do not use `realtext_support::build_perm` for this control: that older-prefix intervention groups identical local pairs and therefore cannot test a local head's pair association. Save donor IDs and changed-context support; if no retained donor map applies to this panel, predeclare a fixed seed before computing outputs, build one shared per-document/PAD-status Fisher–Yates bijection and bind it. Report the intervention's scope; it is not the older-prefix conditional permutation. All candidate context-disabled logits must equal the same frozen bias. Evaluate before selecting any later baseline.

Generate the same six retained prompts for 64 greedy tokens from reloaded E/Q0/QG (reuse verified parent/S/count outputs by exact identity, or replay if needed). Lowest-ID ties; no sampler, repetition penalty, blacklist or output override. Save token IDs and bytes. For these local heads, `(prev,cur)` is sufficient state, so report pair-cycle entry/period. Absence of a longer ring cycle cannot excuse a pair cycle. Report raw failures even if CE improves. F is offline-only and not served.

Measure optimized actual inference latency, peak RSS and artifact storage for E/Q0/QG with the same declared prompt/token scope and timing protocol; exclude calibration from inference while reporting its separate cost. Parameter count and operation envelope are unchanged, but sparsity can affect branch/runtime cost, so do not assume identical latency. Physical energy stays UNAVAILABLE without a real measurement. Keep assembly/primitive claims limited to the touched numerical path; no need to requalify every frozen runtime.

## 7. Tests, preservation, outcome and roadmap

Focused tests must exercise: exact integer Gram versus direct repeated-occurrence calculation; coefficient-update dJ versus direct objective; zero/collinear features and tie behavior; deterministic two-sweep order; Q0 inclusion/non-increasing fit surrogate; valid/invalid row shifts; unchanged non-output fields; real-tokenizer binding; export/reload integer parity; common-teacher entropy/CE/KL identity; matched paired decision versus a misleading parent-only interval; malformed raw float file and nonfinite values. Compile and exercise the changed Rust path using rustup-managed Cargo, formatting, relevant offline package checks and named tests. A queue acknowledgement is not a test. No blanket full suite or new continuation framework.

Deliver one sealed report with complete file-set verification, source/binary/config/input/parent hashes, corrected E/S descendants, F binding, Gram/counts, Q0/QG exports, fixed algorithm parameters, saved metrics/controls/generations, phase timing and storage receipts. Preserve even negative candidates. Refresh final parent hashes and owner checkout state. Record actual nonoverlapping cumulative charge once, leaving the margin intact.

Decision rules:

- If QG passes the practical and method screens, it is evidence that a better ternary export recovers useful head performance at unchanged representation width. If Q0 passes but QG adds no benefit, prefer the simpler existing projection for a subsequent baseline. Either requires artifact/control/cost checks; neither establishes fluent language.
- If both fail, preserve E as the stronger qualified numerical baseline at its scoped development result. The experiment rejects these bounded conversions; it does not prove ternary infeasibility, feature-rank saturation or a need to widen. A <=4-bit head, quantization-aware refinement or joint feature training remains a separately justified option, not an automatic new run.
- After this single reuse experiment, return the architectural recommendation to **query-conditioned geometric read/update on the strengthened local baseline**, with a separable-prefix and local-only matched control. Do not launch that follow-on fit in this task. Explain the next falsifiable hypothesis and resources from the result; no perfect-count-match stage lock. Exact occurrence/version memory, learned write/reset, shared composition, conversation and executable Rust qualification remain the larger programme.

Update README, current-state, project-track dependencies, model-direction, PROJECT_MAP, CONTINUE, EVIDENCE and the resource ledger without rewriting raw historical receipts. Mark this prompt executed with a successor pointer only after actual execution. Update GitHub issue bodies/statuses and any existing project items; preserve historical bodies, leave broad acceptance open, and do not fabricate a board status where no item exists. Stage named paths, push a `codex/…` branch, create/attach a protected PR, use ordinary merge/queue without bypass, and verify actual merged tree/content. Return the precise outcome, surviving artifact, limitations, actual budget and ONE next architectural recommendation.

Research basis: [GPTQ](https://arxiv.org/abs/2210.17323), [AdaRound](https://arxiv.org/abs/2004.10568), [ParetoQ v2](https://arxiv.org/abs/2502.02631). This is an exact fit squared-output objective with a small local discrete solver, not those papers' full algorithms. Lower logit reconstruction error does not guarantee lower CE or better generation; execute the controlled comparison to find out.
