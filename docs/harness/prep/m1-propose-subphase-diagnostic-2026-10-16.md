# Pre-registration — M1 checkpoint-0 propose sub-phase diagnostic

- Owner: Codex/Sol; source review: DeepSeek; TEST FITNESS: Kimi PM with an independent reviewer.
- Authority: [owner instrument+optimize-first decision](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6210107580), [source analysis §3](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6209851483).
- Line: M1 qualification instrumentation; count 0/3 and suspended headline 37/232 unchanged. No mechanism negative or learning claim.
- Scope: **diagnostic, not qualification / not a p95 estimate / not a training rate**. No training, fixtures, corpus apply or model-state update. Q4 NOT_EXECUTABLE; training/M2/Stage B HELD.
- State: the one bounded diagnostic was completed after [PM execution release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6216273515), instrument/bridge merges and identity verification. See the [sealed receipt](../../labs/propose-subphase-diagnostic-run-2026-10-16/README.md). The prospective contract below is retained; this completion does not authorize a repeat, qualification or training.

## Question and bar

The sealed restart-5 receipts place 99.993808% of the approximately 14.5-second
forward-only batch inside the two synchronized `Device::propose` calls. They
cannot distinguish GPU counterfactual/chooser execution from API allocations,
copies, waits or cleanup. This diagnostic locates that cost without changing
candidate enumeration, numerical operations, admission/refusal, ordering or
the original synchronization. It does not assess model quality.

## Base artifact

Use the same restart-5 seed-1, epoch-0 checkpoint and deterministic synthetic
timing windows/targets, not corpus rows: rows64, V4096/B4/C192/K16,
`ReadSpec::Route`, `Route(0)` then `VectorBlock(0)`, sixteen ordinals per proposal.
The A and R cells use respectively `nearest_address_margin_v1` and
`occupied_cell_rank_hinge_v1`; no optimizer, apply or resume. The existing
policy selection occurs before each cell's before-snapshot.

| Pin | Sealed value |
| --- | --- |
| Checkpoint file SHA-256 | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Original source SHA-256 | `f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7` |
| Original PTX SHA-256 | `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28` |
| Geometry SHA-256 | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Initialization config | `docs/labs/admission-redirect-packet-2026-10-15/configs/k16-A-seed1.json` |
| Config file SHA-256 | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |
| Typed DriverConfig SHA-256 | `58b99b595aa6110c8250e8183b3875e93d19d887a941d295c20937dea73b370d` |

The archive is `icloud:UOR-R4/results/codex/codex-k16-qualification-20261016-5.tar`,
MD5 `21a4633d605289bbede3f19c4581b3dc`. Preserve the original snapshot bytes.
The [pin template](../../labs/propose-subphase-diagnostic-2026-10-16/diagnostic-pin.json)
retains the original artifact/PTX/config identities and explicitly declares the
fixed old/new source mapping in the [separate bridge](../../labs/checkpoint0-identity-bridge-2026-10-16/README.md).
Paths may be relocated with file hashes unchanged. The bridge does not change
this sheet's diagnostic dose, timing accounting or outcome decisions.

## Data split

Only the existing deterministic synthetic timing windows and targets are used.
No train/dev/fresh-transfer corpus rows are consumed; this is not a language
evaluation or a replacement for the D-Packet fresh-transfer requirement.

### Required identity bridge — satisfied for the sealed execution

The dependency below was satisfied by merged #2542 at `8199af0076d861bade607b7aa2fc29bfaecb81ae`, tree-identical to approved `38c081603`. Its two executed mapping receipts and unchanged source/PTX/config/artifact pins are preserved in the [run record](../../labs/propose-subphase-diagnostic-run-2026-10-16/README.md). The original prospective requirements follow unchanged.

`device.rs` and `lib.rs` participate in the composite source identity; adding
profiling changes that identity even with byte-identical CUDA kernels. The
loader must refuse the old checkpoint under the new executable. This PR does
not rewrite checkpoint identities or relax `Device::from_snapshot`.

Before diagnostic execution, separately deliver an exact-head reviewed,
evaluation-only checkpoint-0 bridge binding old and new source identities,
original snapshot hash, identical model payload/epoch/seed/shape/policy,
config and PTX/geometry identities. Preserve both artifacts and the bridge
receipt, with training resume forbidden. If PTX differs, stop for explicit
source/PTX review; no automatic substitution. PM must accept this dependency
and the final executable pin before the separate diagnostic lease. Merely
editing the pin's source hash is not a bridge and does not bypass the loader.

## Instrument and accounting

Four timing-enabled CUDA events are pre-created outside measured calls, bound
to the model and its stream. Record pairs immediately around the existing
counterfactual launch (base `device.rs:1354`) and chooser launch (base `:1373`).
No new kernel, launch geometry, stream, candidate order, status check or model
write is introduced. The event endpoints and host submission costs include
instrument overhead; these are coarse diagnostic intervals, not pure instruction
execution counts. Retrieve elapsed events only after existing synchronization,
outside the original caller's propose timer. Retain the vendor millisecond
IEEE754 bits and decode to integer nanoseconds by integer arithmetic; clock
metadata is not floating-point model computation.

Partition each profiled API call into these disjoint host wall intervals:

1. Parameter admission and epoch read.
2. Targets plus spec allocation/upload.
3. Losses/status allocation and zeroing.
4. Counterfactual launch preparation and event/launch submission.
5. Status read and check (including any device wait).
6. Proposal allocation/zeroing.
7. Chooser launch preparation and event/launch submission.
8. Existing final stream synchronization.
9. Return construction and local-buffer cleanup (not allocator-only time).

The completed spans must sum exactly to the profiled call wall duration.
Retain the caller's original synchronized propose timer and original full-unit
sum (carry + both proposes + both observations), with the additional existing
caller context-sync interval separately labeled. Record event-readback wall
time outside that sum and a diagnostic batch wall interval for readback/gaps;
serialization after that interval is not included. Record incomplete spans on
fault where available. **NEVER add GPU-event durations to CPU wait durations:**
events overlap status/final-sync waits. Submission, status-read and sync wall
times alone do not establish allocation or transfer causality.

## Steps and Minimum test power

After FIT, source review, merge identity verification and the reviewed bridge:

1. Fresh authorized `uor-pod` lease, exact merged source/toolchain/PTX/config
   inventory, pin hash and GPU/driver/clock context. Recheck every file hash.
2. Run only `k16-timing diagnose --pin FILE` from the CUDA build. It requires
   the distinct `k16-propose-subphase-pin/v1` schema, two checkpoint-0 cells,
   one excluded warmup and exactly three timed batches per A/R arm. Q4 and
   qualification pins are refused in this mode; `run` refuses diagnostic pins.
3. Save the warmup profiles separately. Six timed batches yield twelve timed
   proposal profiles; including warmups there are sixteen proposal calls.
   Keep raw observations and per-call records, not a p95 summary. The three
   samples are a bounded coarse localization probe, not a statistical tail estimate.
4. Verify snapshot digest and epoch unchanged before/after each arm, zero apply,
   correct operation/count inventory, complete partitioned host spans and finite
   nonnegative event durations. Cross-check events against enclosing wall time;
   a material contradiction (greater than max(1 ms, 1% of enclosing wall)) is
   an instrument blocker, not cost evidence. This tolerance is a prospective
   clock-consistency check, not a claim of GPU clock precision.
5. Seal source/config/PTX/geometry/pin/bridge identities, full command logs and
   exit statuses, device/toolchain metadata, raw diagnostic record, before/after
   invariants, interpretation and all costs. Archive SHA-256 manifest and verify
   cloud MD5 round-trip BEFORE release/delete; verify fleet empty.

## Distinct outcome decisions

- **Counterfactual dominated:** counterfactual event duration is at least 80%
  of enclosing propose wall in every timed sample of an operation/policy cell.
  Decompose that kernel's candidate/token/context work in a separately reviewed
  next instrument/optimization; do not blame carry or extrapolate a training rate.
- **API wall dominated:** combined counterfactual + chooser event time is at
  most 20% of enclosing propose wall in every sample of a cell. Inspect the
  named dominant disjoint host spans; distinguish status/sync waits from
  actual allocation/upload work. Propose only a targeted, separately reviewed
  change; no automatic pooling or synchronization removal.
- **Neither / mixed / chooser dominated:** publish every cell separately and
  the unexplained wall residual, event-readback and batch-wall overhead. If
  chooser dominates, decompose chooser; if coverage or intervention overhead
  prevents attribution, propose a matched profiling-off control or narrower
  interval under a new reviewed sheet. Do not average away disagreement, run
  extra batches silently, optimize by assumption, or resume qualification.

The 80/20 boundaries are deliberately coarse prospective routing rules. A
result between them is unresolved, not a negative. No mechanism acceptance
bar, training dose or headline changes here.

## Cost and stop rule

At the sealed approximately 14.5 s/batch, two arms times four batches is about
116 seconds of propose work: **~$0.027 propose-work component** at $0.842/h.
This excludes provisioning, builds, transfers, event/readback overhead and
archive/teardown. Record the actual lease quote, provider uptime, full wall
time, storage and cumulative episode overhead separately; it is not an all-in
cost cap. The current PR's compile-only lease has the separately authorized
15-minute cap and cannot execute this diagnostic. No extra GPU lease now.

First identity, admission, timing-integrity, zero-update or command failure:
stop; seal partial evidence, archive, teardown and report D22/D31 blocker-to-fix.
No patch/retry on the diagnostic lease. Never classify it as mechanism failure.
No qualification, fixture, training or Q4 execution piggybacks on either lease.

## Review and dependencies

DeepSeek reviews exact-head source; Codex produces exact-head CUDA check and
strict clippy transcripts only. Kimi PM plus independent reviewer rules on
this sheet. The separately reviewed checkpoint-0 identity bridge, protected
merge/byte identity and final pinned execution packet are mandatory before
any diagnostic run. Owner instrument+optimize-first defers the 100-batch
qualification rate to the optimized stack; this three-batch diagnostic cannot
fill that D-Packet slot.

TEST FITNESS: FIT (design) — Kimi PM, [#2541 review at 5a2b16f0a](https://github.com/UOR-Foundation/uor-r4/pull/2541#issuecomment-6210781684). Execution still requires the separately reviewed identity bridge, merge identity and final pinned execution packet.
