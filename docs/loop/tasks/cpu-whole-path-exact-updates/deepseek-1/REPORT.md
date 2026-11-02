# REPORT: whole-path disposition audit, deepseek-1

Lab: deepseek. Item: [cpu-whole-path-exact-updates](../../../../roadmap/items/cpu-whole-path-exact-updates.md) (#3510). Base: `a2fdb23eb`.

## What did not work

**Rejected as a CPU training-port target.** The retained whole-path source is a transactional decision/replay reference, not an exact parameter-update trainer. This triggers the item's cancel condition. No code was written and the historical M2 objective is not revived.

- `whole_path_learning.rs` opens by saying it contains "No optimizer or run loop" (line 1).
- The batch API explicitly states that no optimizer, teacher serving or parameter update is performed (line 1298).
- The retained recovery record says "No optimizer state exists" (`archive/labs/whole-path-recovery-2026-10-12/README.md`, connection/recovery section).
- The implementation record labels the path `KEEP source; NOT TRAINING READY` and says it has no training loop or launcher (`archive/labs/whole-path-implementation-2026-10-12/README.md`).
- `CreditTape::choose_scaled` uses normalized exponentials and sampled probabilities (source around line 2287), so the sampled form cannot be ported under the geometry rules.
- The joint factor path calls a dense tensor product, and the retained implementation depends on Candle and `uor-r4-training`; both are outside the isolated CPU crate boundary.

## What changed

- Closed #3510 at its cancel probe and corrected the roadmap premise: the retained code records decisions and stages store transactions, but never trains parameters.
- Recorded the rejected M2 configuration separately from the useful transactional mechanics.
- Renamed the roadmap item and rendered its table row.

The following mechanics remain as design evidence, not as a trainer:

- route-record order by ascending commit and record id, then deduplication;
- bounded 120-token candidate selection without replacement;
- staged exact-store mutation followed by atomic commit or rollback;
- journaled frame transport and complete turn replay;
- checkpoints bound to revision, parameter, geometry, episode, events, decision records and resulting store.

Future work may implement these as a transaction/replay primitive, but exact replay parity would not establish parameter learning, optimizer correctness or M2 capability.

## Measurements

- Exact parameter-update implementations found: **0**.
- Source and archive records stating no optimizer/run loop: **2** independent retained records.
- Model, corpus, GPU, optimizer or CPU training runs: **0**.
- Code, serving-file or artifact changes: **0**.

## Checks run at the head

- `cargo run --locked -q -p uor-r4-claim-wording -- .`: PASS.
- `cargo run --locked -q -p xtask -- roadmap check`: PASS, 134 items.
- `cargo run --locked -q -p xtask -- roadmap render`: PASS, 134 items.
- `uor-harness ratchet d23`, `d25` and `d32`: PASS with zero growth.
- `uor-harness loop gates --full`: PASS, 11/11.
- `cargo run --locked -q -p xtask -- links`: completed with the repository's existing broken-link inventory; no new link target was introduced by this unit.

## Skills used

No repository skill was invoked. A read-only subagent independently audited the source and retained records and identified the same premise failure.

## Still open

- Transaction/replay mechanics are not ported; they would be a future component item, not a revival of this trainer.
- The historical M2 context/prototype objective remains rejected.
- No CPU training capability follows from this disposition.

## Plain summary

The “whole-path exact-update trainer” never existed as a parameter trainer. The retained code is a useful record-transaction and deterministic-replay reference, but its objective is rejected and its sampled math is outside the rules. It is therefore marked as a rejected CPU training configuration rather than ported as if it trained a model.
