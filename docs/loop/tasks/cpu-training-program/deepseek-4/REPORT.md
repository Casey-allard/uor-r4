# REPORT: reconcile the CPU training program with merged evidence, deepseek-4

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `45540f256`.

## What did not work, first

- **The program page still described statuses from before the CPU implementation PRs.** It called #3501 "not started", said #3502 still lacked a trained observation, called the quantization row downstream, and did not mention the public trained FBS pair or corpus driver.
- **The pilot page still described the private artifact and discrete path as missing.** The remaining blocker is credentialed complete-reply/fresh-phrasing data and a FIT pre-registration, not CPU compatibility or admission controls.
- **The claims ledger retained four DeepSeek branches after their work was already merged.** Those stale rows made `uor-harness check` report claims that could no longer be developed or released as live work.

## What changed

- Reconciled the historical-family matrix with merged evidence through `45540f256`.
- Marked the discrete writer and training path as supported component paths, not future work.
- Recorded public trained M1 writer parity, exact discrete reads/proposals/apply/restart, raw/snapped FBS parity on both trained dual-c131 seeds, strict Hamming/memory restart, and the finite quantization-shadow gate.
- Kept trained-artifact float-shadow, complete-reply and fresh-phrasing outcomes explicitly `NOT_RUN`.
- Updated the child-item table so #3501/#3502/#3504/#3510 are complete, #3503/#3505 remain open for their named unrun comparisons, and #3506 remains `NOT_ADMITTED`.
- Updated pilot readiness with the public trained FBS/corpus path and the HTTP 401 blocker for sealed panels.
- Restored the roadmap parser's required `## What is missing` heading on the completed discrete path and regenerated `docs/roadmap/README.md`.
- Removed only four stale DeepSeek branch claims that point at merged work; the Kimi claim was preserved.

## Measurements and evidence

This unit produced no compute result. It cites:

- [public writer parity](../../cpu-discrete-writer-core/deepseek-2/REPORT.md);
- [discrete checkpoint and BVG-K completion](../../cpu-discrete-training-path/deepseek-13/REPORT.md);
- [public raw FBS trained-artifact parity](../../cpu-shared-vector-train/deepseek-4/REPORT.md);
- [masked-corpus driver](../../cpu-shared-vector-train/deepseek-5/REPORT.md);
- [Hamming/memory restart](../../cpu-quantization-exact-memory/deepseek-1/REPORT.md);
- [finite quantization shadow](../../cpu-quantization-exact-memory/deepseek-2/REPORT.md);
- [whole-path rejection](../../cpu-whole-path-exact-updates/deepseek-1/REPORT.md).

No CPU training run, language result or qualification claim is added.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check`: PASS, 134 items.
- `cargo run -q --locked -p xtask -- roadmap render --check`: PASS after regenerating the roadmap view.
- `cargo run -q --locked -p xtask -- links`: exit 0 on the branch and base; both report 1,225 repository-wide broken links, so this change adds none. The branch count is 24,050 checked links versus 24,039 at base; all seven newly added evidence targets were also checked directly and exist.
- Claim-wording gate: PASS.
- `cargo run -q --locked -p xtask -- docs-catalog --check`: PASS, 1,175 records and zero missing paths.
- `cargo run -q --locked -p xtask -- archive-verify --base origin/main`: PASS, 10,810 files, zero modified or deleted.
- `uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `uor-harness ratchet d25`: PASS, 93 dense-product sites and zero tensor-core sites, baseline unchanged.
- `uor-harness ratchet d32`: PASS, 31 discrete-float sites, baseline unchanged.
- `uor-harness prep-check`: PASS for parsing; one unrelated main-branch error remains in `uor-cuda-inverted-file-performance-2026-11-01.md`.
- `uor-harness loop gates --full`: PASS, all 11 gates green.
- `git diff --check` and `jq empty docs/harness/claims.json`: PASS.

Full `uor-harness check` remains repository-wide red on one unrelated malformed pre-registration and pre-existing registry, command, presence and preparation warnings. The branch does not alter their owning files.

## Skills used

No repository skill was invoked. The work used the owner-authorized D84 CPU program and the merged REPORTs as the source of truth.

## Still open

- #3503: genuine trained float shadow, complete replies and fresh phrasing.
- #3505: pre-registered bounded pilot.
- #3506: pilot execution and capability/cost disposition.

## Plain summary

The CPU program now states what is actually true after the implementation work: component paths exist and pass finite controls, while trained float comparison, complete replies, fresh phrasing and a bounded pilot have not run.
