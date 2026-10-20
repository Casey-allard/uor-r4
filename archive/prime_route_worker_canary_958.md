# Prime-route worker canary — #958

- **Date:** 2026-08-26
- **Issue:** [#958](https://github.com/UOR-Foundation/uor-r4/issues/958)
- **Scope:** frozen source-free synthetic prime-route manifest compiler
- **Certifier/terminal verdict:** `PASS`
- **Programme disposition:** `PASS_SUBSTRATE_SCOPE`

## Claim boundary

This record qualifies worker equivalence, useful four-worker occupancy, and
release-mode compile-stage wall-time improvement for the current preliminary
exact-index substrate. It does not compile a source model, corpus-scale
artifact, complete spin manifest, attention layer, or chat product. It
therefore does not establish geometric attention, reasoning, coherent
generation, teacher parity, or an estimate for a future full compile.

No model-scale or corpus-scale run was launched to produce this record.

## Frozen workload and contract

The canary command was:

```text
target/release/r4 transformerless prime-route-canary \
  --report docs/prime_route_worker_canary_958_final.json
```

The report path had to be fresh. The parent process refused symlinks and
overwrites, persisted the terminal report atomically, killed the child at the
absolute 85-second watchdog if necessary, and actively terminated the process
at an absolute 90-second wall measured from command entry. The runner accepted
only an embedded Cargo `release` profile with optimization enabled; this run
recorded optimization level 3.

The synthetic workload contained 32 semantic atoms, a 128-address pool, 16
whole-sentence partitions, 1,954 routes, 1,938 causal transitions, 5,798 index
occurrences, and at most 8 candidates per row. Each scheduled execution batched
four complete compiles so the one-worker median had to exceed the frozen
500-millisecond timing floor. The schedule contained one warmup and three
measured executions at each of one and four workers, for 32 total compiles.

The predeclared empirical criteria were:

- exact equality with the pinned canonical-byte CID and manifest kappa on every
  compilation;
- one-worker/four-worker semantic equality;
- positive and complete transition accounting for every requested worker;
- peak active workers of four in every four-worker compilation;
- one-worker measured median of at least 500 ms;
- maximum measured-sample deviation of 15%; and
- four-worker median speedup of at least 1.200x.

The frozen workload CID was
`blake3:ce3d96826ffd7134495536d439795ad0e4b035122b41329afd2a6ec4a96cacc6`.
The manifest provenance remained on
`uor-r4.prime-route-worker-canary/1`, independently of the measurement-report
schema.

## Chronology

### Initial binding run: stop and optimize

[`prime_route_worker_canary_958.json`](prime_route_worker_canary_958.json)
recorded `OPTIMIZE_BEFORE_LONG_RUN`:

- one-worker median: 414,666,833 ns;
- four-worker median: 364,351,500 ns;
- speedup: 1.138x;
- one-worker/four-worker canonical bytes and kappa: equal; and
- all four workers: active with complete transition accounting.

The worker stage itself improved from about 74.893 ms to 24.626 ms, or 3.041x,
while roughly 339.77 ms remained serial. The effective Amdahl parallel fraction
was about 16.18%, placing the then-current ideal four-worker ceiling near
1.157x. The declared 1.200x gate was therefore unreachable without reducing
the serial tail. This verdict prohibited larger work.

The compiler was then changed to remove an internal canonical
encode/decode/re-encode validation cycle. The public canonical encoder and
strict decoder remained unchanged, one complete typed validation remained in
the compiler, and the canary retained a strict out-of-timer decode/re-encode
witness for the baseline artifact.

### Apparent pass invalidated by semantic input drift

[`prime_route_worker_canary_958_optimized.json`](prime_route_worker_canary_958_optimized.json)
reported an apparent 1.460x speedup, but it is
`INVALIDATED_INPUT_DRIFT`, not a promotion result. Advancing the report domain
from `/1` to `/2` had accidentally changed manifest provenance and therefore
changed the canonical artifact from the pinned baseline. Its canonical-byte
CID was
`blake3:2ecf5b8e23fb3fe761d4d1f95862aae9c9d4bec046348fd1ae9a15a76859dc71`
and its manifest kappa was
`blake3:78df4f754c9b6e99f5508e3d2aa8708b7016f915a77e0a640d1e864647c24437`.
Relative one/four-worker equality could not detect homogeneous drift, so the
canary was hardened to expose and freeze the semantic provenance and to reject
either reference mismatch as `REFERENCE_ARTIFACT_MISMATCH`.

### Provenance-pinned pass and final evaluator hardening

[`prime_route_worker_canary_958_pinned.json`](prime_route_worker_canary_958_pinned.json)
first re-established the original artifact identity and recorded a 1.450x
compile-stage speedup. A read-only audit then found two fail-closed evaluator
gaps: workload identity was exposed but not pinned by the decision function,
and positive per-worker sentence counts were not summed back to the frozen
sentence population. The observed report satisfied both conditions, so these
gaps did not invalidate it. The evaluator was nevertheless hardened to pin the
workload CID, checked-sum sentence coverage, reject overflow, and require peak
concurrency to equal four. That source change required a fresh current-binary
report.

[`prime_route_worker_canary_958_final.json`](prime_route_worker_canary_958_final.json)
recorded the final `PASS` at the terminal and certifier levels:

| Measurement | Result | Criterion |
|---|---:|---:|
| One-worker median | 655,910,959 ns | at least 500,000,000 ns |
| Four-worker median | 437,610,126 ns | compared with one worker |
| Median compile-stage speedup | 1.498x | at least 1.200x |
| One-worker maximum deviation | 1.9% | at most 15% |
| Four-worker maximum deviation | 0.2% | at most 15% |
| Four-worker peak-active minimum | 4 | exactly 4 requested |
| Exact reference matches | 32 / 32 | 32 / 32 required |
| Worker sentence-total matches | 32 / 32 | 32 / 32 required |
| Terminal wall time | 7,420 ms | below 90,000 ms |

All 32 compiles emitted 2,048,165 canonical bytes with CID
`blake3:d700b84d0b1ac83fab81ec81fa080365a98c44a8209cf1f3442825f2cfaa6841`
and manifest kappa
`blake3:21e7c4da52b09a192d0fef62ee9d46a137514e802b6c0cc99abee2974d594d75`.
The first artifact passed strict decode/re-encode verification; the remaining
31 were exact matches to that strict baseline. Sixteen compiles requested and
used one worker, and sixteen requested and used four. Every worker report had
equal assigned and completed transition counts.

The timed samples are checked sums of four complete `compile_spin_manifest`
calls. Canonical serialization, CID comparison, strict baseline verification,
progress I/O, and terminal publication are outside those compile timers; the
1.498x result is not an end-to-end canary-wall speedup.

The final release binary CID was
`blake3:1360759f14cf0db962a272890bf7a8f6dd90e8a3cd8c22444029819a3a0aca0f`.
The report SHA-256 was
`5d6b461c18de661c04b14c7927c22f8585204eb0b5787e15432d910f4734721e`.

For audit continuity, the initial report SHA-256 was
`c7e1aeda038787e5cc221ea46c8d23609d09e5add4cdb0fbb53a5e98b673fcc8`;
the invalidated report SHA-256 was
`b4f215864cf355395e370d0807423ecf5aa304f9bf36cac43e24a003bb06be15`.
The pre-hardening provenance-pinned report SHA-256 was
`65ef76967fc2c2cdff7b3420e92bd4c36d9e79ee7dbe7fb6b49ca10f7aafcc25`.

## Decision

The current source-free compiler has established its worker-only gate. The
next bounded action is the source-free SpiralCore v63 operator reproduction and
completion of the manifest-bound route/quantization/rebuild witnesses. No long
run is authorized: the complete-manifest stage, divisor/adjacent-spin energy,
layer-29 attention caller, intervention controls, and no-Ollama product probes
remain `NOT_YET_IMPLEMENTED`.

Any change to the manifest's semantic inputs, provenance, canonical encoding,
or representative workload shape invalidates reuse of this timing result until
the exact-reference and worker canary passes again.

## Schema-2 complete-manifest chronology

The preceding chronology is retained verbatim as the evidence for the
preliminary manifest. This appended chronology records the later schema-2
complete-manifest qualification. Its identities and measurements supersede the
preliminary manifest for current reuse; they do not retroactively alter any
earlier report or verdict.

### Immediate repeat rejection and prime-square correction

The first schema-2 attempt stopped after 116 ms with
`OPTIMIZE_BEFORE_LONG_RUN`. The compiler rejected the first adjacent repeated
route prime because it incorrectly required both factors of every semiprime
expert to be distinct. The fail-closed terminal evidence is
[`prime_route_worker_canary_958_manifest_v2_repeat_rejected.json`](prime_route_worker_canary_958_manifest_v2_repeat_rejected.json),
SHA-256
`fc422dc60d929cccbc60b9f7bdbf75f47382fdfd6abcfbfbba2cf703b93f7bd8`.

The correction retains `p^2` as the valid semiprime self-loop produced by an
adjacent repeated route atom. It does not broaden the optional SpiralCore
operator chart: its six distinct prime carriers still form exactly the 15
`J(6,2)` unordered pairs, and those 15 operator fixtures remain square-free,
distinct-edge semiprime experts.

### Required reference-mismatch stop and audit

After the prime-square correction, the canary completed its compilations but
correctly stopped at `REFERENCE_ARTIFACT_MISMATCH`. The schema-2 manifest now
bound the fixed quantization/profile record, semiprime-expert and ordered-n-let
tables, and deterministic rebuild witnesses, so the previous preliminary
artifact CID and kappa were no longer valid references. The fail-closed report
is
[`prime_route_worker_canary_958_manifest_v2_reference_mismatch.json`](prime_route_worker_canary_958_manifest_v2_reference_mismatch.json),
SHA-256
`b8dd996bd30f39153f961ca24bd424b329df2b3f6a28881f53f224d674bf6ed6`.

The mismatch audit confirmed that the representative workload had not drifted:
its CID remained
`blake3:ce3d96826ffd7134495536d439795ad0e4b035122b41329afd2a6ec4a96cacc6`.
The new artifact identity was therefore reviewed as an intended consequence of
the added semantic bindings before the canary references were updated.

### Final schema-2 pass

[`prime_route_worker_canary_958_manifest_v2_final.json`](prime_route_worker_canary_958_manifest_v2_final.json)
recorded `PASS` at both terminal and certifier levels:

| Measurement | Result | Criterion |
|---|---:|---:|
| One-worker median | 661,192,500 ns | at least 500,000,000 ns |
| Four-worker median | 445,318,333 ns | compared with one worker |
| Median compile-stage speedup | 1.484x | at least 1.200x |
| One-worker maximum deviation | 7 milli (0.7%) | at most 150 milli |
| Four-worker maximum deviation | 3 milli (0.3%) | at most 150 milli |
| Exact artifact and kappa matches | 32 / 32 | 32 / 32 required |
| Four-worker use and peak-active matches | 16 / 16 | all must use and peak at 4 |
| Terminal wall time | 7,761 ms | below 90,000 ms |

All 32 compiles emitted the schema-2 artifact CID
`blake3:973acbe598b15aa152532910ac593ab70ebd723a5e76ee16de4ef030a0285422`
and manifest kappa
`blake3:e8f1ed27755b36cfd8e3161b8c6cf46bcef3a2afeeafdd00c8a398b40e14aa4f`.
The release binary CID was
`blake3:57e493ed59a55aa5af9b31eed8ed1c0cc28135b84089c17b37ed0a3c7f0dc6c4`.
The final report SHA-256 was
`df8744c1f846753ff6eeda6de721b281e4c3c791323a8352e1c3a82dd6c017b5`.

This pass closes the complete-manifest and worker-canary portion of #958 for
the exact schema-2 inputs above. It does not establish semantic attention,
no-Ollama product behavior, inference, reasoning, or teacher parity; those
claims require their own bounded evidence.
