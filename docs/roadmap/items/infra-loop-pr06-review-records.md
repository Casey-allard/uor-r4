---
id: infra-loop-pr06-review-records
title: Loop PR 6: loop review with tagged review records
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr05-policy-check]
---

## Why it matters

An exact-head review is the last check before `main`. It comes from another vendor's lab or, for a loop setup PR under D59, from Claude's fresh-context reviewers. A review posted in free text cannot be checked by a machine.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`loop review` with tagged review records".

## Done when

`loop review` writes and parses the `uor-review v1` comment with the fields the loop page lists. The policy check needs an APPROVE at the current head from a lab whose vendor differs from the author's, with no later CHANGES; a doc-only or generated-only PR may carry a same-lab `fresh-context` record.

**Loop setup PRs follow D59.** While D59 is in force, a setup PR passes only with an APPROVE tagged `claude`, whichever lab built it. A record from any other lab, APPROVE or CHANGES, does not count. A Claude-built setup PR with a Claude APPROVE therefore passes.

- **What the gate checks.** Under D55 it checks tags, not sessions. That the APPROVE came from a fresh context is the reviewing lab's word, as it already is for a doc-only same-lab record.
- **Which PRs are setup PRs.** A PR counts as a setup PR only if three things hold:
  - its ORDER names a build-plan row in a `Setup row:` line (for example `Setup row: 6`);
  - that row exists in the build-plan table on the loop page;
  - the ORDER's item is `infra-loop-harness` or that row's own item (for row 6, `infra-loop-pr06-review-records`).

  Where the ORDER is filed does not count. The line is still the author's declaration, like a tag (D55), so it narrows the loophole but does not close it. Claude's reviewer confirms that the PR builds the named row, and refuses the review if it does not.
- **Which ORDERs need the line.** It applies to setup PRs that reach the gate after this PR lands. Setup PRs already merged, reviewed in free text, need nothing. This PR's own ORDER carries `Setup row: 6`.
- **When it ends.** D59 ends when the other labs run the loop. The gate reads this from a switch the owner sets, so ending the exemption needs no code change.
  - **Where the switch lives.** It sits in a file listed in `docs/harness/owner-paths.json`, so no lab can flip it with an ordinary PR. Adding that path is an owner step. This PR's ORDER proposes the file and nothing more.
  - **Who flips it.** The owner decides when it flips, and a flip records the owner's words verbatim with the time.
  - **The default.** If the switch is missing or unreadable, D59 is in force.

Identity is by tags: there is no signing, no key and no signer list (D55). The planted fault turns the check red: **an untagged review record, a same-vendor record on a code or model PR outside D59, or a setup-PR record from a lab other than Claude is refused.** Two cases are part of that fault:
- A Claude-built code PR whose ORDER names no build-plan row is outside D59, so a Claude review of it is refused.
- A `Setup row:` that names a row the table does not have (for example `Setup row: 99`) does not make a PR a setup PR.

## How to start

1. Team (D52): codex or deepseek builds it. This is itself a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read "The review record" and "Identity is by tags" on [the loop page](../../loop/README.md), and D59 in [CHARTER.md](../../../CHARTER.md) §5.5.
3. Open `crates/uor-r4-harness/src/prgates.rs`, `crates/uor-r4-harness/src/registries.rs` and `docs/harness/roles.json`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.5 (D59) and §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D55 and D59
