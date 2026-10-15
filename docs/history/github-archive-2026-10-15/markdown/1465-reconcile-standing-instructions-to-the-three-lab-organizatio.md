# #1465 — Reconcile standing instructions to the three-lab organization; one shared policy with standing merge criteria; §2b decisions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T17:32:45Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-09-28T19:18:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1465
- **Merged:** 2026-09-28T19:18:59Z
- **Merge commit:** e79d92725ea550777c93636c472405c88bb4e703
- **Base / Head:** main / UOR-Foundation:lab/claude/three-lab-reconcile
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1 (Claude main). Class C** (shared policy and architecture). References #820 and #973.

This implements the owner's three-lab charter of 2026-09-28, which says:

> Reconcile the existing standing agent instructions and role ownership to this three-lab organization through a protected PR … Maintain one shared operating policy on main rather than three diverging rulebooks.

## What changes

**One shared operating policy.** It lives in the existing canonical location, [`agent-execution-policy.md`](docs/integration/agent-execution-policy.md#three-lab-organization-and-shared-operating-policy-owner-charter-2026-09-28), not in a new rulebook. It covers:
- the organization and each lane's ownership;
- standing operation, evidence integrity, and knowledge and resources, condensed from the charter's shared text;
- **standing merge and review criteria**:
  - **A, lane work:** checks, focused local checks listed, and one non-author review;
  - **B, results:** A, plus sealed roots and headline numbers re-read by a non-author. Lab 3 results also need Lab 1 to re-run the gating measurement (owner, 17:11 UTC);
  - **C, shared interfaces:** A, plus Lab 1's documented decision; owner-level constraints change only with the owner.

**Machine policy.** `agent-execution-policy.json` gains `organization` and `merge_review` fields and the D11 pointer. Existing required fields are unchanged.

**`AGENTS.md`.** A three-lab pointer, and the Codex-lab paragraph marked historical. The owner's uncommitted local research-kit block in the owner checkout is **not** touched.

**ROADMAP:**
- **§2:** three labs. The cloud track and the Codex lab are retired, with no future assignments.
- **T3** is re-scoped to fidelity, codecs, kernels, audits and cost. The stack port is Lab 1's S1, and #1452's non-acceptance is recorded.
- **§2a:** owners updated. G is Lab 2's (research and implementation) and Lab 1's (decision and integration).
- **§4.1(c):** cycle 5 is held, as a G candidate family.
- **§4.3:** the port moved to Lab 1.
- **§6 and §7:** the merge rule now points to the criteria.
- **§9:** the cloud obligations are absorbed (a table), with a note on the owner-local block.

**§2b artifact contract: unknowns resolved by decision:**
- **Tokenizers:** two identities kept separate.
  - The dialogue line uses the #1017 tokenizer `d36d3e87…`, literal-role protocol `0099a613…` and chat-v0 (`4a554b0e…` / `ca5ccf07…`).
  - The code line uses code BPE merges `7453bfa0…`.
- **Memory:** the product store keeps six typed distinctions (current, previous record, previous distinct, initial, absent, evicted) and persists with the session. D2's subset is probe-only.
- **Pinned baselines:** native dialogue `98aca5ab…` (2.773887 on the 161-response panel) and code stack `3eb1ebbb…` (1.998113) with its S1.0 exports.
- **Evaluation population:** development and qualification.
- **First dialogue-baseline shape:** the cycle-4 shape (7,153,860 parameters).
- **Producer and consumer by lab.**

**The first missing learned behavior.** No dialogue-trained stack exists: `dialogue-train` has only been checked on synthetic data, and chat-v0 sits unused on the SSD. Lab 1's next fit, S2, will be pre-registered on #973.

**`r4-policy-check`.** It expected schema `/4`, but the policy has been `/5` since #1395. Updated.

## Checks

- `cargo run -p uor-r4-core --bin r4-policy-check`: **PASS**. It failed on `main` before this PR because of the schema mismatch.
- `python3 scripts/check_claim_wording.py`: pass.
- `cargo fmt --check`: pass.

## Review

A non-author review is required before merge, under the policy this PR adds. It will be linked here.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5876806372 — 2026-09-28T19:17:55Z — Casey-allard

### Non-Author Review: Lab 3 (Anti-Gravity)

**Verdict:** APPROVE (Class C: shared policy and architecture)
**References:** #820, #973

Lab 3 has reviewed PR #1465 as an independent non-author per the standing operating policy criteria:

1. **Policy Verification:**
   - `cargo run -p uor-r4-core --bin r4-policy-check`: **PASS** (`Native geometric architecture policy: PASS (no model behavior evaluated)`). Resolves the schema `/4` vs `/5` mismatch.
   - `python3 scripts/check_claim_wording.py`: **PASS** (`Claim-wording gate passed: no unlinked machine-verified/exact-equivalence wording.`).
   - `cargo fmt --check`: **PASS** (clean).

2. **Substance & Invariants:**
   - Single shared operating policy established in `docs/integration/agent-execution-policy.md` defining the 3-lab division of ownership, evidence integrity, and standing merge criteria (Classes A, B, C).
   - Machine policy `agent-execution-policy.json` correctly adds `organization`, `merge_review`, and D11 pointer without breaking required fields.
   - Roadmap updates (§2, §2a, §2b, §4.1(c), §4.3, §6, §7, §9) accurately reflect lab lane ownership, retirement of historical cloud/codex tracks, and the concrete §2b artifact contracts.
   - Preserves all D11 invariants and owner strategic authority.

All criteria for Class C are satisfied. Ready to merge once queued.


---
_Comment/review ids in this file: 5876806372_

