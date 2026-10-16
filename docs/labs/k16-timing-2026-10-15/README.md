# K16 synchronized 100-batch timing instrument (2026-10-15)

**Current direction, 16 October:** #2539 fixes the warmup window count. The
[owner instrument+optimize-first decision](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6210107580)
defers 100-batch qualification timing to the optimized stack. The separate
[`diagnose` mode](../propose-subphase-diagnostic-2026-10-16/README.md) adds
checkpoint0 proposal sub-phase clocks under a PENDING sheet, never a qualified
rate or p95. This instrumentation changes production host source, not CUDA
kernels. Its checkpoint identity bridge is still required; no diagnostic run.

**Latest qualification update, 16 October:** [restart 5 at `812459c7f`](../k16-qualification-restart5-2026-10-16/README.md)
passes all step-0 gates, G-Pre and C-R1..C-R6. This timing runner then executes
both checkpoint-0 cells, but generates 100 windows and skips one warmup, emitting
only 99 timed batches per cell. Its process exits 0; the separate 100-batch
receipt check fails and stops the attempt. Diagnostic-only padded-context rates
are A=846 and R=845 tokens/s, not qualified 100-batch rates. Zero apply and
unchanged epoch/snapshot checks pass. No patch/retry; Q4 stays NOT_EXECUTABLE.
Source repair/review/merge and PM disposition precede further execution.

**Earlier 16 October qualification update:** the timing CUDA release build passed on
RTX 5090 at `3c651df97`, but [step 0 halted on the separate fixture runner's
build](../k16-qualification-2026-10-16/README.md) before clippy or model execution.
The timing instrument has not run: K16 rates remain UNMEASURED, and the Q4 leg
remains NOT_EXECUTABLE. A successful build is not runtime qualification.

Historical original source-only delivery of the runner Codex delegated ([#2029 6201508191](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201508191)) under
the PM ruling ([#2029 6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519)). The instrument is a **standalone dated-record crate**: `measurement.rs`
and every production kernel are untouched, and nothing under `crates/` changes.

**Original source-only delivery status (superseded by the execution update above):** G-Pre and the timing benchmark stay NOT_RUN / UNMEASURED until this source is reviewed, merged,
and run on a pod at the qualification stage. No pod was leased for this delivery.

## What it measures

The carry + propose + observe path over a **frozen** snapshot at the K16 shape: V4096 / B4 / C192 / K16, Route read policy,
64 rows, **100 distinct batches**, **both objective policies**, **one excluded warmup per cell**, four cells (checkpoint0 and the
sealed seed-1 Q4 fixture, occupancy 189, crossed with the two policies).

- Every timed step is bracketed by `Device::synchronize()` — before and after `carry_windows`, and before and after each
  `propose` + `observation`. The numbers are integer host nanoseconds of a synchronized native API.
- **Zero apply.** Each cell asserts an identical snapshot digest and epoch before and after; a change is a fault, not a footnote.
- **Two rates, on purpose, over the FULL timed unit** (carry + propose + observation for both parameters, named in the plan as
  `rates_basis`): target-rows/s (acceptance-facing) and, separately, physical **padded-context-tokens/s**
  (64 rows x 192 positions = 12,288 per full batch). **Carry-only nanoseconds are reported as a separately named secondary**
  (`carry_only`), never as the headline: a carry-only divisor understates per-batch cost in the expensive direction, and this number fills the D-Packet tokens/s slot that sizes training wall time. Per-family floors (carry, route propose/observation, vector propose/observation) are reported alongside, and the record states its dtype: integer nanoseconds over an integer-only discrete stack, no floating point.
- **Per-batch receipts** carry the carry nanoseconds, both proposals (candidate, before/after epoch, whether any active candidate
  cost differs from identity, active candidate count) and the epoch, with `applied: false` on every record.
- **Identities** retained: the pinned fixture pins (snapshot SHA256, seed, epoch, occupancy, original K8 SHA, evaluation-only
  rebind), the objective policy, the shape, and the compiled/device identity from the crate.

## Fail closed

`validate` refuses the run, before any device is touched, on: an absent or malformed fixture checksum (snapshot or the retained
original), a shape that is not K16, a read policy that is not Route, a rows/batches/warmup mismatch, `evaluation_only_rebind:
false`, `training_resume: true`, an empty cell list, a missing objective-policy arm, a duplicate cell, or a pin whose schema is
wrong. `Pin` is `deny_unknown_fields`, so a typo in a pin is an error rather than a silently ignored field. At run time the
fixture file is checksummed against the pin before it is admitted, and a changed snapshot or epoch after a cell is a fault.

## Faults preserve evidence

A timing run that faults on batch 60 is still evidence about batches 1..59: the ledger writes the **partial** record (status
`partial`, with the fault stage and the preserved record count) before the error propagates. The record is written atomically
(temp file plus rename), so a reader never sees a half-written receipt and a failed write leaves nothing behind.

## The exclusion, in every record

```
no apply/update is timed: every cell asserts an unchanged snapshot digest and epoch, so these are
forward-only carry+propose+observe batches and NOT a training dose
```

Any use of these numbers as a training-dose estimate must quote it. A route-family timing arm, if anyone adds one, needs its own
label; this instrument reports the four pinned cells and nothing else.

## Host verification (no device)

14 host tests, all passing, and a plan command that prints exactly what a run would do without a GPU:

```
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

They cover: the fail-closed set above (shape, read policy, absent/short fixture pins, resume-as-rebind, missing policy arm,
duplicate cell, unknown policy, unknown field), batch distinctness and canonicity (100 x 64 x 192 identities, tokens inside the
vocabulary), the integer rate arithmetic including the 12,288 padded-token identity and the zero-nanosecond guard, partial-receipt
preservation on a fault, the record carrying the exclusion and `apply: false`, the plan naming every timed operation, and the
atomic write leaving no temp file on a failed rename. `cargo clippy --all-targets` reports nothing for this crate
(the warnings in the output come from pre-existing dead code in workspace dependencies).

Full transcript: [host-tests.txt](host-tests.txt). The example pin a reviewer can validate on any host:
[pin.example.json](pin.example.json).

## What a reviewer should check

1. **The timed set matches the delegation**: carry, propose and observation only — no apply, no forward, no evaluation.
2. **The K16 rebind is evaluation-only**: `original_k: 8` and `original_sha256` are required pins and `training_resume` must be false.
3. **The zero-apply assertion is a fault, not a note** — see the `zero_apply` branch in `run_cell`.
4. **The exclusion cannot be lost**: it is a constant in the header and in every cell record.
5. **Nothing production changed**: `git diff --stat origin/main -- crates/` is empty for this branch.

## Cost

Laptop CPU only: source, one host build, one test run. No pod, no GPU, no corpus write, no model compute, no lease.

## Repair, 2026-10-16 - cells now execute their source and policy

Codex found a real defect in the first delivery (PRELEASE_SOURCE_BLOCKER, #2029 6205684341) and refused to
work around it, correctly: the executor called load_fixture unconditionally, validate demanded the Q4 pins
whatever the selected cells were, and - the serious one - every cell reused one loaded device and one
objective policy, with source and policy printed as labels. A four-cell record would have measured one arm.

The repair: one device per cell, built from that cell artifact (a pinned checkpoint0 artifact or the sealed
Q4 fixture), with the cell objective policy applied to the snapshot before admission and asserted after
admission (cell_policy fault if it disagrees). Pin requirements now follow the selected cells: a
checkpoint0-only panel needs no Q4 fixture, and Q4 cells are still refused without the sealed pins.

Until this repair lands, the previously merged instrument must be read as plan-only for the multi-cell
cross product. Three tests pin the boundary: a checkpoint0-only panel validating with blank Q4 pins, a Q4
cell still refusing without them, and the four-arm cross product being distinct rather than relabelled.
