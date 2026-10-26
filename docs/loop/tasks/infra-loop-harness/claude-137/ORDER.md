# ORDER: infra-loop-harness, claude-137 (widen the item's scope to take codex out of the registry)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-106, -108 and -121). Base: `1b9b87b39`.

## Goal

**This unit is the item PR for a D68 scope and bar decision, at the owner's instruction (below).** It edits one file, `docs/roadmap/items/infra-loop-harness.md`, so that the next unit can take codex out of the lab registry. Codex works independently on lexlean-r4 and is no longer part of the UOR-R4 loop.

**Why a separate PR.**
- The loop guard takes a unit's scope from the item as it is on `origin/main`, so a branch cannot widen its own scope (`loopguard.rs`, `check_repo_rel_write`).
- No item's `scope:` holds the registry, its two tests, the no-identity hints or the branch archive. The guard refused the first test edit: "REFUSED: crates/uor-r4-harness/src/labs.rs is outside active ORDER … scope [PLAN.md, …]".
- The loop page says "Widening scope takes a separate item PR" ([docs/loop/README.md](../../../README.md), "Checked at the base"). The charter lets it go ahead when the lab's own decision review passes (§5.1, D68).

**The two edits.**
1. **Scope:** `scope:` gains ten globs: the files that read the real registry and name or count its labs, the loop page that lists the labs, and the archive. Nothing else is added.
   - `docs/harness/labs.json`: the registry. It stays owner-only, so the guard still refuses every lab write and Casey makes that one edit.
   - `crates/uor-r4-harness/src/labs.rs`: its tests read the real registry and list five labs.
   - `crates/uor-r4-harness/src/receipt.rs`: the no-identity hints name the labs.
   - `crates/uor-r4-harness/tests/lab_registry_cli.rs`: its tests read the real registry and the hints.
   - `crates/uor-r4-harness/src/onboard.rs`: `an_unknown_lab_fails_and_names_the_registered_labs` reads the real registry and expects "codex" in the refusal.
   - `crates/uor-r4-harness/src/loopguard.rs`: `check_all_receipts` walks `CLIENTS` (five, codex included) and returns "codex is not in docs/harness/labs.json" once the registry drops it.
   - `crates/uor-r4-harness/tests/loop_guard.rs`: `make_uor_fixture` copies the real registry. So the codex-client tests (`evaluate_guard(root, "codex", "codex", …)`, `run_onboard` over `CLIENTS`) and the real-repo receipt test (`assert_eq!(verified, CLIENTS)`) change with it.
   - `docs/harness/README.md`: "`--check-receipts` verifies all five committed receipts".
   - `docs/loop/README.md`: its title and "## The five labs" table (added after the second review's advisory A1). Its probe and history rows stay as history.
   - `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*`: codex's closed #3169, kept as a patch and its record.

   The other harness tests that name codex build their own registry fixtures, and none of them copy the real one: `loop_bench`, `loop_board`, `loop_claim`, `loop_close`, `loop_next`, `loop_prcheck`, `loop_queue`, `loop_repair_orders`, `loop_review` and `loop_shared_files`. The two heartbeat tests copy it but run as `claude` and `deepseek`. The client-adapters item (`infra-loop-pr09-client-adapters`) owns `loopguard.rs` and `onboard.rs` but is `done`, so it cannot be claimed.
2. **Bar:** the done-when's "The five labs (Kimi, Claude, Codex, DeepSeek, Antigravity) are onboarded" becomes "The four labs (Kimi, Claude, DeepSeek, Antigravity)". The bar keeps the same shape. It no longer asks for a receipt and a loop task from a lab that has left. Codex's onboarding receipt and its past units stay in the repository as history.

**How the receipt gate follows the bar (decided here, built in claude-138).** The bar asks each registered lab for a committed onboarding receipt, and `check_all_receipts` is the gate that counts them.
- **`CLIENTS` stays five.** It lists the client adapters the guard supports, and codex's adapter stays as code. So `loop guard --client codex` still parses codex's hook payloads. Inside UOR-R4 it then refuses, because `check_lab` no longer finds codex. Outside UOR-R4 it allows, as for every client.
- **`check_all_receipts` walks the registry's labs, not `CLIENTS`.** Each registered lab must be a known client with a valid receipt. So it verifies four.
- **`docs/harness/onboarding/codex.json` stays where it is,** committed history that is no longer checked.
- **The guard tests that exercise the codex adapter register codex in their own fixture copy.** Codex's adapter keeps its coverage.

Rejected: cutting `CLIENTS` to four. It would remove codex's adapter and rewrite about ten guard tests, for no gain while the registry already refuses codex.

**Left as they are.**
- **The item's title and its "Why it matters" prose** ("five-lab", "Five labs from five vendors"). They describe the build and no gate reads them; the REPORT lists them.
- **`AGENTS.md`** (`UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`): an owner path this instruction does not name. The REPORT lists it for the owner.

**Next unit.** Once this merges, claude-138 claims the item for the registry change: the tests and hints move to four labs, the patch goes into the archive, and Casey edits `docs/harness/labs.json` on that branch.

## Owner approval

Casey, in claude's chat, received at about 2026-10-26T21:34Z, naming `docs/harness/labs.json`:

> "Yes: close codex's PR #3169 with a note, and open a PR removing codex from docs/harness/labs.json, quoting this instruction in its ORDER."

It answered claude's question "Close codex's #3169 and take codex out of the lab registry?", after Casey wrote: "codex is not currently working with the rest of the groups, it is working independently on lexlean-r4 so it should not be included."

## Cancel probe (run first)

Cancel if codex is already out of the registry, or another open PR changes the item file or `docs/harness/labs.json`. To check:
- `docs/harness/labs.json` and the item file on `origin/main`;
- the open PRs' files.

Result (2026-10-26T21:40Z, re-checked 22:05Z, `origin/main` at `1b9b87b39`):
- `docs/harness/labs.json` still lists codex, and the item's `scope:` holds none of the globs listed below.
- The open PRs are #3304 (antigravity), #3306 (kimi) and this unit's #3307. None of them touches the item file, the registry, the two registry tests or the branch archive.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line and the done-when sentence)
- this task folder

## Done when

1. The two edits are made, and `xtask roadmap check` passes.
2. A fresh decision review, reading the whole project, approves the scope and bar change. Its record goes to the owner's log on #2028.
3. A fresh exact-head review approves the PR, and `loop report --check`, `loop pr-check --local` and the required checks pass.
4. The REPORT says `References #2904`.

## Stop and ask if

Stop and decide; never wait.
- **The review finds a narrower way** to make the registry change, for example a glob already in another item's scope. Take it, and drop the globs it makes unneeded.
- **The bar edit needs more than the lab list**, for example a gate that counts onboarding receipts. Record it, and keep this unit to the two edits. *(It does: `check_all_receipts`. The first review found it, and the decision above records it.)*
