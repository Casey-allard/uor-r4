# Whole-Path CUDA, Replay And Episode Credit (12 October 2026)

**Status: KEEP source; CPU-reference checks pass. CUDA execution NOT_RUN.**
Exact-head checks, review and protected delivery are recorded on PR #2246.
This is source integration, not trained-model or device qualification.
Owner request: finish the CUDA producer,
sampled checkpoint recovery and multi-episode learning connection, preserving
shared geometry; do not train or rent a pod. References #2030 and #2037.

Line: unified geometric route/action/learning integration. Count5/3 remains;
this expressly requested continuation would become6/3 only on completed
delivery, without reset. Headline saved437/512 plus opened0/128 -> unchanged
panel fitting. Future `unified-bound-content-437` remains NOT_RUN and has no
TEST FITNESS admission. The owner hold supersedes the old run automation.

## Implemented Connections

- Device compiler, Context, exact-store routing decisions, Copy/donor/bridge,
  Generate, continuation and status use the same named original Var inventory.
  Rust performs admitted exact-store transactions and bounded branch/output
  metadata handling. Source scores and loss construction stay on the device.
- The native OLD/right-action/NEW order, independent Cue query/source encodings,
  response Prefix and one incremental pre-token source Prefix trace are
  retained. An absent Cue stays absent. The final source token has no redundant
  Prefix transition. All physical Copy aliases enter the same token pool.
- CDF sampling and alias reductions have linear custom operations and CUDA
  kernels. They avoid Candle's triangular-matrix cumsum and per-token rescanning
  of every alias. Actual speed, peak memory and energy are NOT_MEASURED.
- Replay checkpoints bind revision/master/geometry identities, episode limits,
  sampling configuration, raw events, decisions/RNG, native store snapshot and
  route index. A fresh session reconstructs the graph and must reproduce the
  complete checkpoint. A failed replay cannot overwrite an existing session.
  Frame transports are journaled and replayed too. No optimizer state exists.
- CPU reference and CUDA batch APIs isolate stores and seeds while sharing
  immutable factors and original Vars. Equal episode objectives add global
  prototype credit once with the mean advantage. Credits from another episode,
  the wrong order, stale parameters or subsequently extended trajectories fail.
- Prepared packets bind a live episode owner independently of saved revision
  and lineage. Even identical snapshots in separate episodes cannot exchange
  compiler decisions or teacher credit. Device teacher and native emission
  share compiler/session/event validation; rejection leaves the receiving
  episode's own pending packet usable.
- Rebinding may pack admitted CUDA masters at the artifact boundary. The host
  reference session rejects CUDA masters. The CUDA constructor rejects CPU
  masters; source test helpers are explicitly CPU numerical references.

SpiralCore's existing64 Cl(0,6) actions and Claude's exact240 E8 address tables
remain the reused geometry. Transported profiles are consumed in their canonical
frame; exact occurrence/version/payload identity remains separate from route
collisions. No new semantic distance on hashes, lossy Hopf inversion, tree
replacement, transformer path or serving-contract change is introduced.

## Qualification And Repairs

At admission, disk was33GiB. While this work was underway it fell to25GiB and
then24GiB. Builds stopped below the mandatory30GiB floor. The five exited
compile attempts exposed sparse-checkout include omissions and then source
errors; known reported errors were edited, but no later compile was admitted.
These attempts are failures of preparation, not passing tests or model negatives.
That earlier draft at `eb3c4dd4277e629d127d4a91732d538309d880c6` remains historical
UNCOMPILED/UNQUALIFIED evidence. After owner-authorized disk cleanup, current
main `503a6d0b5` was reconciled and the changed library/fixtures compiled. A
private checkpoint return type was repaired. A new foreign teacher-packet test
failed before the live-owner/native-validator repair and passed afterwards.
A device credit fixture initially admitted only one ranking candidate; it now
admits both occupied families to exercise ranking, without changing production
sampling or evaluation criteria.

The prior37 passing checks belong to PR #2242's f03fa07a head, not this result.
Current source qualification:28 whole-path fixtures,3 native grounded-store
regressions and5 E8 regressions pass. They cover sampled cold recovery,
reproduced gradients/next output, native Q24/compiler/store parity, linear alias
gradients, equal-episode prototype credit, stale-credit rejection and tampered
RNG recovery without destination mutation. Fifteen named shared parameter groups
have finite nonzero device-reference credit; masters remain unchanged. All64
frame actions preserve the tiny end-to-end fixture's outcomes and Generate/Copy
Q24 scores; all240 roots pull back for each frame.
An additional arithmetic fixture activates nonzero quarter-coded Copy/Cue/
Prefix/bridge/Generate/continuation/status factors and matches native Q24 scores
at empty, one-token and two-token response prefixes.

These are authored software fixtures, not the512 panel, fresh transfer,
learning success or efficiency measurements. CUDA-feature build, NVRTC and
actual-device execution remain NOT_RUN on this Mac. The actual-CUDA fixture is
feature-gated and deliberately ignored pending an authorized environment.
Scoped formatting passes; workspace-wide `cargo fmt --all --check` finds
unrelated existing core/integer formatting differences, left unchanged.
Exact-head library/check/format/claim receipts and self-review are on PR #2246;
queue acknowledgements execute no tests. No optimizer recovery is claimed.

No training, optimizer update, panel, diagnostic, fresh draw or pod rental was
performed. The preceding owner-authorized storage cleanup removed only an idle
debug profile after a Cargo dry-run inventory, preserving release binaries,
source and research. This source continuation leaves peer work/processes/leases
untouched. The training automation remains paused.

## Reproduction And Remaining Gates

1. Refresh main/claims and verify the actual disk/RAM floor. Use an owned
   checkout. Do not copy source into the primary
   checkout or reactivate the paused training automation.
2. Reconcile the latest main into this branch, then compile the touched training
   library using `CARGO_TARGET_DIR=target/codex-whole-path`, incremental off,
   debug info off, two Cargo jobs and one test thread. Local temporary test
   artifacts belong under this worktree's `local/tmp`, not `/tmp`.
3. Run only the focused whole-path and native-boundary regression fixtures,
   without an optimizer. Repair all type, arithmetic, causality, replay and
   gradient mismatches before presenting source qualification.
   Reproduce with `cargo check --offline -p uor-r4-training --lib` and
   `cargo test --offline -p uor-r4-training --lib <filter> -- --test-threads=1`,
   filters `whole_path`, `native_grounded_session`, and `e8_memory::tests`.
   Set `UOR_BUILD_SOURCE_COMMIT=$(git rev-parse HEAD)`, use rustup Cargo,
   `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
   `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=2`, and `TMPDIR=$PWD/local/tmp`.
4. Device compilation/execution requires an available approved CUDA environment;
   the current owner instruction forbids renting one. Keep CUDA qualification
   NOT_RUN until there is an authorized environment. Do not run training.
5. Update this record, current-state and affected mechanism/README claims with
   actual evidence; self-review the exact head, use protected delivery, verify
   fresh-main tree contents, then preserve receipts and remove owned transport
   branches/worktree. A draft PR or compatibility acknowledgement is not delivery.

Opposite outcomes change the next action: any mismatch blocks activation and
is repaired as tooling; passing focused checks allows source delivery and later
prospective device/data/FIT review, not a model run. Raw episode/instrument
fitness, fresh transfer and M2 acceptance remain separate unfinished work.

## Cumulative Cost

Carry1,639,452,185/1,650,772,756ms through2026-10-12T06:00:00Z from the immutable
PR #2242 closeout. Charge all nonoverlapping preparation, failed builds, review,
resource waiting and pending-delivery work after06:00 once. Initial inclusive
estimate180min, two Cargo jobs/one test thread/8GiB RAM; the2GiB target reservation
does not override the30GiB disk floor. New owned paid compute$0. Native ledger
mapping remains UNRESOLVED, so there is no model-compute admission. Current
checkpoint and final tail charges are posted on the owning issue; no reset.

Resume carry1,644,852,185ms through07:30UTC; cumulative allowance extended by
28,800,000ms to1,679,572,756ms on #2037, without reset. Conservative elapsed-tail
charges include waiting, cleanup, failed checks, review and delivery. Resume
admission: existing644MiB target reused, <=3GiB new target storage, <=8GiB RAM,
two build jobs/one test thread, disk above30GiB throughout scoped checks.
