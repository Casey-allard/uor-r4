# Completion quarter scoring runbook — 14 October 2026

**Receipt-only runbook, rehearsed end to end on sealed Q1.** This closes the missing
operational handoff after #2470. Kimi covered Q1 at [6174365494](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6174365494).
Codex independently pulled the same sealed pod files and reproduced her complete
audit JSON byte-for-byte. No model execution, GPU work, run modification, owner
record, training release or new scientific result occurs here. Seed-2/M2 stay HELD.

Line: evaluation integrity; model headline and research count unchanged.

## Exact commands for the remaining quarters

Work only in an owned worktree. Set PATH explicitly: on this machine the initial
unqualified `cargo` resolved to cargo-ai, which does not implement Rust `install`.
The pod wrapper requires its **Rust** provider adapter; a missing local adapter
can misleadingly report “no SSH address”. Build it below or set `UOR_POD_VAST_BIN`
to a verified existing Rust adapter. Do not substitute the Python Vast client.

```sh
export PATH="$HOME/.cargo/bin:$PATH" UOR_LAB=codex
# If using a different lab identity, set it to your own lab.
CARGO_TARGET_DIR=local/pod-provider-target cargo build --locked --offline -p uor-r4-pod-provider
CARGO_TARGET_DIR=local/scoring-target cargo build --release --locked --offline \
  --manifest-path docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/Cargo.toml
export SCORING_AUDIT_BIN="$PWD/local/scoring-target/release/completion-scoring-audit"
export ROUTE_ROWS_BIN="$PWD/local/scoring-target/release/route-learning-slot"
export UOR_POD_VAST_BIN="$PWD/local/pod-provider-target/debug/uor-r4-pod-provider"
```

Restore the archived pilot `report.json` from
`icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar` into your own `local/`
directory if it is not already available. The runbook requires its exact SHA-256
`e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`.
It also accepts a byte-identical copy of that report from a peer's preserved
working material. The runbook never alters that source. Set `PILOT_REPORT` to its
absolute path. Each destination below must be new; missing quarters fail rather
than falling back to an earlier result.

```sh
export PILOT_REPORT="$PWD/local/pilot-report.json"
# Run each only after Claude has sealed that quarter, not merely at its ETA.
bash docs/labs/completion-quarter-runbook-2026-10-14/score-quarter.sh \
  8202 "$PILOT_REPORT" "$PWD/local/seed1-q2-audit"
bash docs/labs/completion-quarter-runbook-2026-10-14/score-quarter.sh \
  12303 "$PILOT_REPORT" "$PWD/local/seed1-q3-audit"
bash docs/labs/completion-quarter-runbook-2026-10-14/score-quarter.sh \
  16404 "$PILOT_REPORT" "$PWD/local/seed1-q4-audit"
```

| Quarter | Completed cursor | Last batch index | Approximate arrival (UTC; not a readiness test) |
|---|---:|---:|---|
| Q2 | 8,202 | 8,201 | October 14 21:26 |
| Q3 | 12,303 | 12,302 | October 15 01:44 |
| Q4 | 16,404 | 16,403 | October 15 06:01 |

The script uses `uor-pod ssh vast-55939635` for read-only `tar`/`cat` and pulls:

- `/workspace/uor-r4/rung0-completion-20261014/seed1-run-state/evaluation-NNNNN.json`
- Its adjacent `evaluation-NNNNN.pin.json`.
- `/workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1/checkpoint-NNNNN.json`.

It rereads the pin after transfer and requires identical bytes; checks the pin's
registered evaluation path and SHA; requires the exact requested cursor, the
checkpoint path and checkpoint byte SHA, and the released seed-1 admission SHA
`2063239bd9c65f6ccdd1fcdec21c1e6898e076dbd6cc9d171f6de9f582e51203`;
then calls the audited scorer. A new attempt path after a resume is a visible
runbook update coordinated with Claude, never a guessed location. Every failure
preserves the local attempt and exits nonzero. Retry to a fresh local directory;
never repair or replace the pod's immutable evidence.

The only scorer source change from #2470 is Kimi's existing
`SCORING_AUDIT_CHECKPOINT_PATH` resolution override. The original remote FilePin
and artifact bytes remain intact, and the same expected SHA is still mandatory.
**All scoring and validation functions remain byte-identical.** The output schema,
bar arithmetic and limitations also remain unchanged. This is a host receipt
instrument: it does not load CUDA or run a predictor/reference/initializer.

## What to post on #2029

Post cursor, input pins, scorer source/executable SHA, audit result, top-1/top-8,
guard strict wins with checkpoint-zero baseline, admitted strict wins, B3
admitted-target rows, loss contributions and per-kind attempted/differs/accepted
counters, plus writer occupancy. Retain `SHA256SUMS`, `audit.json`, stdout/stderr
and the transferred receipts together. Link the canonical plan and source.

For Q4, the script now also invokes `route-learning-slot` **after the audited scorer
succeeds**, saving `route-learning-slot.json` and its stdout/stderr. The standalone
Rust binary is built by the same Cargo build above; Q1–Q3 do not invoke it. It checks
the exact seed-1 final cursor, evaluation pin, checkpoint hash, registered admission,
plan/baseline pins, model binding, epochs and matching per-kind counters. It records
the observed VectorBlock/Route attempts, cost-difference and accepted counts. Gate is
explicitly **excluded by the registered Route executor**, with no serialized counter
and no Gate efficacy claim. See the [source-only tooling record](../route-learning-slot-2026-10-15/README.md).

To generate the sidecar from an already successfully audited, sealed Q4 directory:

```sh
"$ROUTE_ROWS_BIN" "$PWD/local/seed1-q4-audit" \
  "$PWD/local/seed1-q4-audit/route-learning-slot.json"
```

Output must be new; a previous output is never replaced. The tool consumes no
`steps.jsonl`, so coordinate indices are schedule-derived, not an observed trace.
It cannot identify which coordinates were accepted or isolate a Route-only quality
delta from Q3→Q4 (that interval also contains VectorBlock updates). This sidecar
provides the learning-slot evidence for the later Relation sheet; it adopts no
interleaving ratio and grants no compute or seed-2 release. Do not run it before the
final receipts are sealed; its development controls use synthetic metadata only.

Q2/Q3 are **INTERMEDIATE_CHECKPOINT_DIAGNOSTIC**. Boolean comparisons in
`registered_comparisons` are arithmetic diagnostics; they are not an interim bar
verdict. Bars are decided at completed cursor **16,404** only. Q4 is
**SINGLE_SEED_FINAL_RECEIPT**, not a two-seed KEEP, fresh-transfer result or seed-2
release. The scorer intentionally does not authenticate an owner release.

B3 and guard definitions stay exactly as audited: `admitted_cells-address_rivals`
is target-cell admission; `token_rivals>0 && token_hinge<32` is a strict win even
when the target is omitted. Report their intersection separately. Expected nesting:
admitted strict wins ≤ top-1 ≤ top-8 ≤ admitted rows. Expected VectorBlock attempts
at Q2/Q3/Q4 are 8,202/12,303/16,384; Route attempts are 0/0/20.

## Rehearsal evidence

The retained [Q1 audit](q1-audit.json) is byte-identical to Kimi's scratch audit.
Its input hashes were independently checked after a fresh pod pull:

| Input | SHA-256 |
|---|---|
| Q1 evaluation pin bytes | `b810541c8d2791a2a34169af55e4f8340dd576c8c8cbfc873ffec1addce04d9b` |
| Q1 evaluation | `cc1e584ea0e42ca8be0cda3374a0e35532acc3007011738488c2536b97900c36` |
| Q1 checkpoint | `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd` |
| Pilot baseline report | `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3` |
| Canonical SEED1_PLAN file | `5dc0243caf50ab33edb7c02c075eb288e570d6315e1df8a88ce3fea82167f4be` |

Q1 reproduces top-1 **96**, top-8 **1,090**, guard strict wins **177** from baseline
**105**, admitted strict wins **41**, B3 **2,201**, and loss numerator delta
**−136,416**. VectorBlock **4,101/2,332/315**; Route **0/0/0**; distinct patterns
**206**, occupied writer cells **150**. Row losses: **6,627 down / 6,695 up / 965
unchanged**. These are existing GPU measurements re-counted from saved rows.

The optimized local receipt audit took **0.34 s wall**, 0.19 s user, 0.02 s system,
**85,114,880 bytes maximum RSS** on arm64 Rust 1.97.1. This is one observed replay,
not future GPU evaluation latency or a timing gate. Release build: 6.60 s.
The complete transfer-and-score script also passed. Cost: local metadata work and
read-only transfer only; no new pod or model compute. Full Q2–Q4 inputs are not
available yet and have not been scored by this rehearsal.

All **12 existing scorer tests pass**, using the authenticated full pilot report
and checkpoint-1640. A deliberately wrong checkpoint-path override is refused
with `checkpoint FilePin byte SHA mismatch`, before any output is created.
`bash -n` passes. Source diff inspection confirms that the checkpoint resolver is
the scorer's only changed code block.
