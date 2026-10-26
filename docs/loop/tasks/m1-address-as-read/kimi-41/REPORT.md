# REPORT: m1-address-as-read, kimi-41 (object-store oracle successor design)

Lab: kimi. Order: [ORDER.md](ORDER.md). References #2833; this design unit does not close the item.

## What did not work, first

- Stage 0 remains a scoped model-copy failure: 0/40 exact matches in every arm, despite exact D11 parity on 14,287/14,287 rows. This unit did not rerun that configuration or call it a mechanism negative. A curriculum/optimization explanation for model copy remains open.
- The first successor design proposed a second token store in `crates/uor-r4-integer/**` and a whole-crate scope expansion. D68 rejected it: the delivered object store already provides verified exact/sieve reads and the existing Stage 0 runner provides the GPU path. No integer-scope or code change was made.
- The sieve has no version selector. Previous-version sieve output is therefore only a descriptive `Current` fallback and is excluded from true-address-versus-sieve, rather than conflating version choice with address formation.
- No GPU evaluation, hidden-fact panel, training, BPB or new model capability was run or claimed in this unit.

## What changed

- Added [the object-store successor design](../../../../labs/m1-address-as-read/integer-read-successor.md): `AddressDirectory` exact/sieve reads in the existing Stage 0 GPU instrument, with no second store and no integer-crate scope expansion.
- Defined no-memory, no-address, deterministic-sieve, true-address and random-address arms. True-address hard emission must be exactly 40/40 current, 20/20 previous and 20/20 typed abstention, with zero wrong-value substitutions.
- Separated store correctness, oracle assistance, address formation and model generation. Added the baseline-ceiling outcome, `sieve_address_match` apart from `sieve_value_check_pass`, and no-memory/no-address byte identity.
- Updated the item's start text to link the reviewed design and use the five-arm protocol. Its scope, `status: ready` and done-when are unchanged.
- Size at the REPORT commit: 4 files, 201 additions and 2 deletions, all documentation. There was no numeric estimate.

## Measurements

No new model measurement. The design is based on the sealed kimi-40 v2 receipt: parity 14,287/14,287 and `COPY_GATE_FAIL`, tar SHA-256 `cb72cdf4eef5316c56d3c732eb5bc523ff55918ddc4725252afe7dfc52108395`. The proposed 40/20/20 gates are implementation acceptance conditions, not results.

D68 whole-project review (agent-37, DeepSeek/deepseek-flash): REJECT the integer-store/scope route; APPROVE the revised object-store design at `20ef271577bc4d38de1f3c569fa1dd121068e517`. The lab's decision is logged on #2833 at comment `6418501213`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render`: PASS.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check`: 94 items OK.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -q --locked -p xtask -- links --json local/kimi/links-3282.json`: no broken link in this unit; the pre-existing repository report has 1,088 other broken links.
- `git diff --check origin/main`: PASS.
- `uor-harness loop gates --full`: all 11 gates GREEN at `a00b24732`.
- `uor-harness loop report --check --order docs/loop/tasks/m1-address-as-read/kimi-41/ORDER.md`: PASS at `a00b24732`.
- `uor-harness loop pr-check --local`: PASS at `a00b24732`; one advisory item-scope warning only.

## Skills used

- `uor-research-architect`: mechanism scope, causal claim boundaries, matched controls and exact evidence identity.
- `scope-guard`: refused the unnecessary integer-crate expansion and kept the delivered object-store path.
- `solution-integrity`, `holding-the-goal`: changed the design after falsifying review instead of defending the first route.

## Still open

- Implement the pure manifest/edge controls and five-arm composition in the existing object-store runner under a later ORDER.
- Pre-register every GPU evaluation with fresh non-author FIT and D67 review. Do not freeze a hidden-fact panel until the exact hard-emission instrument passes.
- Address formation after the oracle, default-serving promotion and any model-mediated read remain separate units with their own evidence and cost reviews.

## Plain summary

The model still cannot copy the supplied value by itself. Rather than build another store, the next instrument will use the verified object store we already have and compare no memory, no address, sieve, true address and a planted wrong address. This design is approved; no new model result exists yet.
