# PR1527 independent retained-evidence review

Head:5d51f2855c4a912c1360cafc11890adf0fdce315. Reviewer: Codex contributing lab, non-author. Verdict: **CHANGES REQUIRED**, narrowly scoped below. This review checks retained historical artifacts and result labels, plus the new report provenance boundary. It is not a complete code review of the2,311-line research binaries, an executed build, or a model rerun.

## What is independently corroborated now

All13 cited S1/S2 root manifests match the SHA-256 values in the PR. I checked their complete file sets, byte counts and BLAKE3 payload hashes: all match (130,084,153bytes read; about0.30s in this observation). See sealed-verification.json. This establishes retained content integrity, not truth of arbitrary claims or build-to-source identity.

Recomparing stored reply token IDs by request ID and turn, with identical key sets, gives:

| Comparison | Equal complete turns |
|---|---:|
|QAT exported integer versus QAT served-forward replies|56/58|
|QAT exported integer versus separate float continuation|6/58|
|Float continuation's PTQ integer versus its own float replies|5/58|
|QAT exported integer versus pre-adaptation parent float|5/58|
|Unquantized float continuation versus parent float|7/58|

The parent report binds model8cb11d8f… and has SHA25649e7bd3034ff56bedd2d3f82dc25dbe93d666c6478e8c2fa7d715695cdd7550e. See greedy-recomparison.json for differing request/turn IDs. These are complete greedy-turn comparisons; they do not establish useful dialogue or an all-input numerical equivalence. The own-QAT unquantized greedy comparison remains unavailable. Its own-float NLL is separately present in the training report, so keep those two availability statements distinct.

S1 retained NLL values support the reported0.01801164 gap to the separate float continuation and7.79246e-7 integer-versus-dequantized arithmetic gap. The S2 literal and corrected parent NLL thresholds remain missed; the corrected report now acknowledges that appropriately. Prior fixes separating56/58 from5/58 and preserving the missed29/58 gate are useful and corroborated.

## Required corrections

### 1. S1 top-1 row names the wrong comparator

In `docs/integration/d4-geometric-s1-qat-result-2026-09-29.md`, the90.28% row says “vs Float Continuation.” The cited LUT evaluation instead binds `float_model` to **Arm1 QAT's own saved model**, SHA f4caf562824abf53257d7d7d60c81c43949529e3eacf5aed4a8e6b33b7f20093, and records top1_agreement0.90277099609375. Its own-float NLL is1.9624902710114909; Arm2 continuation is1.9551148883912632. The inspected evaluator calculates this agreement from the supplied float model's predictions and integer outputs. Relabel the comparator as Arm1 own unquantized float; do not claim the separate Arm2 agreement or insert unsupported equality values for other columns. No rerun needed.

### 2. Bind S2's control window NLL to the actual evaluation

`docs/evidence/d4-s2-dialogue-qat-2026-09-29.json` puts2.971783307682384 under `integer_serving_fidelity.arm2_valid_window_nll`; the table similarly gives Arm2≈2.9718. But its cited, hash-verified control `d11_eval/evaluation.json` reports **2.984152881893584** for BOTH D10 and D11, over64windows/16,384targets, artifact6c0b388588ae0916ea8d20a9006962427044fc50068858f78af57a5ba0a3bafe. Its evaluation SHA256 is ad164de6129442cc68334291bbe07bda7dab272aae62904cb8d2268494f2b618.

Arm1's corresponding D11 record does support2.971733069410822. Correct the control integer field/table. If2.971783 is a separate float evaluation, retain it only with its actual source identity and label; it cannot stand in for this D11 record. Keep these window metrics separate from161-response NLL. No new run is requested to repair the attribution.

### 3. New research-report provenance cannot use execution-time Git HEAD as build identity

`s2-behavior-codec.rs:153` runs `git rev-parse HEAD` in the process working directory and writes it as `git_commit` at1219. A binary built from another revision or dirty source can therefore report an unrelated clean checkout's head. The binary hash is useful but does not solve source binding. Record build-time source identity and cleanliness using the existing project convention, or label the current field explicitly as runtime working-directory context with source binding UNVERIFIED. Never backfill unknown historical source identities.

The codec hashes the request file at416 and reads the dynamically selected tokenizer and heldout inputs, but its final report does not retain those input hashes. Preserve request/tokenizer/data identities in the output receipt, including the selected panel/protocol. The distortion binary's report records request/model/artifact hashes but likewise needs declared executable/source and tokenizer identity. Reuse existing provenance machinery rather than introduce a new framework. Compile/focused checks for source changes remain pending normal admission; no broad model rerun is implied.

### 4. Retain the measured numerical scope; remove unbound whole-serving claims

The PR body and S2 conclusion assert all-serving zero-float/zero-multiplier/zero-divider compliance and mention D5. The cited records establish finite D10/D11 output comparison on specified artifacts, not a compiled instruction audit of every served path or selected-parameter access. Their recorded `weights_read_per_token` is7,238,304. Link an applicable exact-binary audit if one exists; otherwise label those wider properties unverified here and retain only the scoped observed equality. A hash-verified report is not itself an instruction audit.

## Delivery consequence

Keep the branch/results preserved and this PR draft until the scoped corrections and applicable source validation are complete. The independent retained-evidence reread above does not need Claude specifically, and a mandatory full model rerun should not be inferred merely from the old “Lab1 re-run pending” wording. Kimi can assign the qualified evidence/delivery role. Keep independently checked historical fields distinct from outstanding new-binary validation. Do not close the parent task or promote dialogue capability from these measurements.
