# REPORT: one-model-end-to-end, deepseek-273 (aligned-corpus GPU smoke)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base code: `7a450a88f`; launch code:
`f44afe7a973039fd4b28d69b6050a65ca6680d4d`; contract implementation source:
`4add144ebdc9087e47fe3fa830fb21116278644e`. All invoked-script hashes and
the exact reviewed execution-head procedure are in
[`launch-pins.env`](./launch-pins.env).

**Status:** complete as a registered outcome row 0. One GPU-trained integer
artifact was exported, sealed, recovered through a laptop audit, and scored on
all 1,024 development and 1,024 Fresh rows. P3 (the copy positive control)
failed, so this is an underfit/instrument result, not a mechanism negative.

## What did not work

The first registration did not pass adversarial review: it changed the arm-r
batch without saying so, omitted the scenario panels from the execution path,
used an unbound legacy artifact, had no lawful non-neural baseline, and did not
seal failures or verify current compute capacity. A second review then found
that the source/job identities were only self-referential, the watcher had no
stall/cost control, scoring was not source-pinned, and the first failure seal
started too late. A third review found that the terminal marker preceded
readback, teardown did not wait for a verified seal, the cumulative cost and
time bounds were not implemented, scoring did not compare every row and
identity, the P1 classifier was wrong, and the stack contract omitted the
training seed. A fourth review then found a guaranteed 192-versus-256 corpus
context failure in scoring, stale contract implementation provenance, an
incomplete score command graph, a store check that did not reserve the prefix,
and a healthy-run cancellation that conflicted with CHARTER §5.4. Those
findings were repaired. A fifth review then found that preflight reports were
field-checked but not byte-pinned, a fixed reservation path could overwrite an
older claim, a failed TERM had no escalation, and the score command graph
omitted `--out`. Those findings were repaired. A sixth review then found that
P2 used 914 rows instead of the registered 882, caller-supplied run IDs could
bypass the 128-bit nonce requirement, unsealed exits did not always preserve a
pending-stop record, the command graph still abstracted required paths, and
pre-watcher cleanup did not record its billed interval. Those findings are
repaired. A seventh review then found that the launcher audited its own empty
run directory, both entry points defaulted to the failed review-6 receipt,
pre-watcher and watcher failure exits did not consistently persist both budget
and pending-stop records first, the remote claim was still overwrite-capable,
the command graph omitted smoke paths and had mismatched timeouts, and a score
seal failure could retain a classified decision. Those findings were repaired.
An eighth review then found that an active GPU run could cross the budget
without an extension record, the pre-watcher budget omitted staging and lacked
an inter-process lock, the contract implementation source was stale, and
pre-watcher network calls could hang unmonitored. Those findings are repaired
in the current registration. A ninth review then found that healthy extension
still depended on an idle GPU, budget-result writes were outside the launcher
lock, unknown pods could be marked KEEP, pre-watcher hangs remained, job
failure provenance was incomplete, and the Markdown renderer dropped
timeout-only command details. Those findings are repaired in the current
registration.

The first billed launch at review 10 stopped before watcher or job start:
`uor-pod` now bootstraps source at `/root/build/src-<sha>`, while the launcher
still verified `/workspace/uor-r4`. The launcher recorded the failure and
released and destroyed the pod. The measured failed-launch cost was
`$0.041756`; no model process ran. The reviewed path now binds the bootstrap
source root and writes only inputs, runs and seals under `/workspace/uor-r4`.

The replacement launch at review 11 also stopped before watcher or job start.
The bootstrap checkout is shallow and did not contain the older code-pin
object, so the remote provenance check failed. The same cleanup released and
destroyed the pod; its measured cost was `$0.043007`. The reviewed launcher
now fetches that one pin object with a 120-second bound and compares the exact
code/config trees. The laptop still proves pin ancestry before the pod exists.
Total failed-launch cost is `$0.084763`; no model process has run.

The third launch passed source, input, GPU-test, smoke and 1,024-step training
checks. It then stopped at `verify-config` because the assertion required an
explicit run-config `context` key, while the trainer intentionally omits that
key for its 192 default; the sealed model config records `context: 192` and
every other asserted field matched. The failed job sealed all 83 files,
including the 1,024-step checkpoint, and the watcher destroyed the pod. The run
cost `$0.079289`; cumulative spend reached `$0.164052`. This is an instrument
failure, not a model result.

The fourth launch completed source, input, GPU-test, smoke, training, config,
snapshot and artifact-export checks. It stopped at the on-pod stack audit
because the pod's offline Cargo cache lacked `serde_spanned v0.6.9`. The job
sealed 88 files and the watcher destroyed the pod. This run cost `$0.081948`;
total spend across all four attempts is `$0.246000`.

The sealed artifact was then audited and scored on the laptop. The stack audit
had no mismatches and retained the expected source and contract identities;
its only unavailable check was the optional integer-session binary. The
artifact parsed and served all 2,048 held-out rows.

## What changed

The scorer keeps the original schema `/1` when the control is off and emits
schema `/2` with an optional previous-token count table on exactly the selected
artifact cases. The planned run uses arm r's batch-16 geometry recipe plus
explicit train-as-served rounding, a complete receipt-bound input bundle, a
bound `/10` stack contract, exact source/job hashes, a same-built-in smoke
score, exact file-set failure sealing, a post-readback teardown sentinel,
persistent cumulative time/cost accounting, stage heartbeats and full
model/table/base row identity checks. The contract now names the implementing
source, describes all four score commands plus classification and preflight,
and an absent internally generated 128-bit prefix is claimed by an exclusive
create/read-back
per-run object before model work. Preflight reports are byte-pinned as well as
structurally checked. Healthy ceiling crossings are explicitly extended and
persisted only while the marker is RUNNING, the job is present, GPU compute is
active, and progress is fresh; stalled process groups are TERMed, retried, then
KILLed with an unsealed pod preserved.
P2 now counts exactly the 882 scored non-copy rows, with the 32 unscored
general rows excluded. Job and scoring failures write row 0 explicitly.
Pre-watcher and watcher failure paths acquire the shared budget lock and persist
budget and pending-stop records atomically before release or keep. An
unverified pod is never marked KEEP. The command graph includes the smoke path,
exact wrappers and matching timeouts.

## Measurements

The one RTX 5090 bootstrap, build and D29 gate took 103 seconds. The registered
GPU Hamming subset passed 79 tests with one timing test ignored. The 64-step
smoke and its score completed. The 1,024-step run completed in 112 seconds of
training; in-run DEV top-1 moved only from 4 to 31 of 14,287 and token loss
stayed near 1.0.

The exported artifact SHA-256 is
`98e42e3a60501312c86360f3d362e450456507bf02d67bc5d19ece55174f1d90`.
Model, base and previous-token rows had identical scenario IDs, identities and
selected ranks.

| Split | Complete | EOS | Short-cycle | Cap | Exact | Grounded asked value | Control abstentions | Distractors | Copy asked value | Distinct replies |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| development | 1,017 | 1,017 | 7 | 0 | 0 | 0/441 | 4/220 | 0/882 | 0/110 | 793 |
| Fresh | 1,019 | 1,019 | 5 | 0 | 0 | 0/441 | 14/220 | 0/882 | 0/110 | 825 |

Fresh grounded asked-value counts were memorize `0/110`, paraphrase `0/110`,
selection `0/110` and update `0/111`. The previous-token table was zero on
every scored metric.

The registered classification is **row 0**: P1 fails (`0/441`), P2 passes
(`14/220` abstentions and `0/882` distractors), and P3 fails (`0/110`).
Twenty inspected copy/memorize replies were malformed and contained no asked
value. This is underfit instrument evidence, not evidence that geometry cannot
carry language.

The sealed original run is under
`results/deepseek/one-model-end-to-end-scenario-gpu-smoke-2026-11-11/smoke-20261111t094608z-cee5a565d23347403b49eaeefab7507f/`.
The repaired evidence bundle, including the artifact, snapshot, audit, both
reports, decision, recovery record and SHA-256 manifest, is under the matching
`...-salvage/` path and passed download read-back.

## Checks run at the head

- `scenario_grade`: 11 passed, including the TRAIN-context mismatch refusal.
- CPU scenario-grade CLI: 3 passed.
- Full `uor-r4-cpu-train` library: 506 passed, 5 ignored.
- Stack-contract generation and check: 5 contracts current.
- `xtask`: 228 passed.
- Byte-pinned preflight gate: both reports match their frozen SHA-256s and pass
  with corpus context 256.
- Decision program: parses the four pinned preflight reports and emits schema
  `/3` row 4 as expected for the unchanged base.
- Shell syntax and ShellCheck: pass.
- Mocked launcher/watch safety: 7 passed, covering an early identity failure,
  untouched unknown and mismatched pods, durable records before owned-pod
  teardown, lock-serialized sealed-failure publication, healthy `gpu=1`
  extension, and stale `gpu=0` stop instead of extension.
- On-pod training run: input hashes, 79/79 selected GPU tests, smoke checkpoint,
  smoke score, 1,024-step checkpoint, config and bound export passed.
- Laptop stack audit: no mismatches; artifact parsed and served.
- Scenario rows: 1,024/1,024 eligible on both splits, zero prompt-ineligible
  rows, exact model/base/table identity and rank agreement.
- Exact-head transcripts are retained under
  `local/deepseek/scenario-gpu-smoke-2026-11-11/checks/`.
- Clippy with `-D warnings`: stops on pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`, outside this change.

## Skills used

`working-discipline`, `solution-integrity`, `scope-guard` and `context-guard`
kept the correction bounded and checked against the registered evidence path.

## Still open

Do not extend this 1,024-step configuration. The model did not learn even the
direct-copy positive control on aligned data, so more steps or more aligned
examples are not the next purchase. The next unit must change the learned
transform or read path and rerun the positive control before repeating this
panel-level question. The sealed artifact remains available for tracing which
stage loses the copy signal.

## Plain summary

The current geometric training path learned no usable copy behavior in this
bounded pilot. The run is a valid underfit result, not a test of the proposed
state-keyed attention replacement. The next work should repair the learned
transform/read path rather than add steps to the same configuration.
