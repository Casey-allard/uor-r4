# [codex] Closed-loop writer replay on Wick

The compiled input inventory is [BLIND-INPUTS.json](../BLIND-INPUTS.json). The scorer uses the original SM89 native-generation and exact-copy cubin from #4090 and a separate integer-only AOT finalizer. The finalizer receives the returned eight-word copy record and the arm's policy bit on the same CUDA stream. A delimiter release after cursor progress substitutes EOS for the native proposal; all other words and actions are preserved. The actual committed ID feeds the next native step. Rust checks the proposal, read vector, admitted cells, top eight, exact source ID, cursor, release and committed ID on the same prefix. No arm consumes the target to decide EOS.

The original GPU fixture compared both policies on a native prediction that already equaled EOS. Its [records](finite-initial/fixtures.json) are preserved. The strengthened tests require native ID different from EOS and a changed emission for every positive case. Both released artifacts proposed ID 1155. Each emitted EOS on four nonempty delimiter releases and preserved the native/copy path on all seven negative cases. The fixture includes immediate delimiter, original fence at delimiter, cap at delimiter, simultaneous cap and fence, inactive previous source, abstention and ordinary copy. [Final binary and device](finite/program.sha256) · [Seed 1 words](finite/seed1.json) · [Seed 2 words](finite/seed2.json)

From the standalone scorer instrument, build `d11-scorer` on Wick with `--features copy-completion --target x86_64-unknown-linux-gnu --release` and `UOR_NVCC_ARCH=compute_89`. The binary checks both original writer identity and the compiled blind input digests. The first command exercises finite controls. The second takes only a released model, bare token prefixes and a sealed selector file. The third independently executes the graded actual-feedback rollouts, including both oracle controls, and rejects any drift from the second command's nonoracle witnesses.

```sh
d11-scorer completion-finite MODEL
d11-scorer completion-blind MODEL BARE CONTINUATION NEW_BLIND_REPORT
d11-scorer completion-rollout PATHS BLIND_REPORT NEW_GRADED_REPORT
```

PATHS is a JSON object with the path fields `artifact`, `grade`, `bare`, `span_state`, `span_picks`, `exact_state`, `value_picks` and `continuation_picks`. Each blind witness binds the frozen entity record, chosen address, every emitted ID, first native step and all step fields (source, cursor, release, native proposal/top eight and actual emission). The graded report stores `eos_action_step` separately from `first_free`; an EOS completion does not count as a free native generation. Output files are created exclusively.
