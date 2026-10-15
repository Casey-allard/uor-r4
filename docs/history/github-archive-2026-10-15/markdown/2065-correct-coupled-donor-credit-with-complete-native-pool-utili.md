# #2065 — Correct coupled donor credit with complete native pool utility

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:49:11Z
- **Updated:** 2026-10-09T17:53:33Z
- **Closed:** 2026-10-09T17:50:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2065
- **Merged:** 2026-10-09T17:50:59Z
- **Merge commit:** 4f7eee35b250b6d5bf9e250fc0ea00d356998695
- **Base / Head:** main / UOR-Foundation:codex/occurrence-joint-credit-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The existing coupled Prefix/Generate learner assigns indirect donor credit through separate state-lane contrasts. A complete donor transition can change both endpoints of a native Generate pair, leaving a finite interaction outside that surrogate. Add explicit `donor_credit: "full_pool_utility"`: detached complete native forced-donor pool losses supply softmax(BASE Copy) credit in place of the old state tangent. Direct Copy, factual Generate coefficient credit, frozen U and native construction remain intact. Historical configurations retain their previous mode; authenticated recovery cannot relabel the producer's credit rule.

## Result

**NOT YET PROMOTED.** The native pair fixture demonstrates the omitted target-score interaction, and focused tests check zero-forward loss, unchanged factual Generate gradients, direct plus indirect Copy credit weighted once, physical aliases/clipping and recovery provenance. This implementation does not run the original parent's 31 backwards, constructor or generated replies. Accepted 8/512, the separate unselected pair 6/15 and coupled 9/15 remain unchanged. No serving rule or parameter family is added.

Execution source, hashes, exact commands, review scope, costs and preservation: `docs/labs/occurrence-joint-credit-2026-10-09/`. The next admitted model run must use the original parent and the unchanged 15-position/17-reference/380-guard gate before whole-answer qualification.

## Checklist

- [x] Checks run at this exact head (commands and outcomes posted on this PR)
- [x] Research record: `docs/labs/occurrence-joint-credit-2026-10-09/README.md`
- [x] New entry at the top of `docs/integration/current-state.md`, with **Next:**
- [x] Every document this change makes stale is updated (donor-credit/state-tangent/coupled-learning Markdown search; dated negatives retained as historical evidence)
- [x] STATUS.md / ROADMAP.md implementation position updated; milestone status and accepted numbers unchanged; #2028 table updated on verified delivery
- [x] README Training section and training crate README document the opt-in mechanism and unmeasured model benefit
- [x] `python3 scripts/check_claim_wording.py` passes at final head
- [x] After merge: verify on fresh `origin/main`, delete branch/worktree, preserve results, and run hygiene; one final milestone result

References #2030


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6086259263 — 2026-10-09T17:50:01Z — Casey-allard

Exact-head validation and review: **PASS at `e7675c6340999d8d193abc9c69301650b085d66d`**.

Executed at that clean head with owned `CARGO_TARGET_DIR=local/joint-target`, two build workers and `UOR_BUILD_SOURCE_COMMIT=2f64255db593f375c7a121561c6221e182432051` (the reviewed implementation commit; delivery head changes documentation only):

- `cargo test --offline --locked --release --jobs 2 -p uor-r4-training --example geometric-frozen-map-fit coupled_episode_learning::tests`: **16 passed**, 0 failed, 111 unrelated tests filtered; 0.83s including cached compile, 83,116,032B max RSS. The repaired-source uncached recompile/test took123.15s.
- `cargo build --offline --locked --release --jobs 2 -p uor-r4-training --example geometric-frozen-map-fit`: PASS; cached0.41s. Production build117.72s, 887,275,520B max RSS.
- Named changed-file `rustfmt --check --edition 2021`, `python3 scripts/check_claim_wording.py`, and `git diff --check`: PASS.
- Changed child SHA0350c3e1ca574741a283cfc3727ccf36522533c89a710ce3717c559cc39d6923 is byte-identical across implementation and delivery heads. Production executable SHA d8f27809e17032b1b4e904d5c74543976b0c83dc176374cae0c70e1082b45fda; test executable SHA970ee888686816f16fc5f19153ed2c7199502a8bc420be09642415d081e0e286.

Mathematical review: PASS_SOURCE_ONLY on the repaired implementation. Adversarial **exact-head source and claims review: PASS** on e7675c6340999d8d193abc9c69301650b085d66d. Its original recovery-mode relabeling blocker was repaired before validation; producer config, requested mode and gradient receipt now agree in both recovery loaders. Reviewers performed no builds/model runs; the checks above are parent-executed.

Scope: authored native two-lane pair/alias fixtures, zero-forward/gradient isolation and recovery compatibility. No original-parent31-frame backward, model constructor, export, generated reply, held-out run or CUDA model execution occurred. Accepted8/512 and separate negative conditional pair6/15/coupled9/15 are unchanged. Required GitHub status names are compatibility acknowledgements, not these test results.

Runtime/source/logs preserved through fresh iCloud MD5 round-trip plus index: `codex-occurrence-joint-credit-20261009`, 19,009,536 bytes, MD5 `6b862a261543a357621db37748b3e8ff`. Record: docs/labs/occurrence-joint-credit-2026-10-09/.


### comment 6086286191 — 2026-10-09T17:51:34Z — Casey-allard

Protected merge verified: `4f7eee35b250b6d5bf9e250fc0ea00d356998695` is on fresh `origin/main`. All **11 changed blobs** equal reviewed head `e7675c6340999d8d193abc9c69301650b085d66d` at the merge and fresh main. M2 and the tracker table now record the implementation with model benefit explicitly unmeasured. Delivery branch/worktree cleanup follows; no successor started.


### comment 6086320822 — 2026-10-09T17:53:33Z — Casey-allard

Closeout complete. Protected merge `4f7eee35b250b6d5bf9e250fc0ea00d356998695` and all11 changed blobs verified on fresh main. Casey's checkout is clean main at that commit. GitHub and local delivery branches are deleted; the owned worktree and its785,212KiB local work area are gone. No `codex/*` branch remains locally or on GitHub, including retired `codex/lab-state`.

`uor-hygiene` dry-run/apply completed: GitGC done, active dirty `answer-span` peer worktree preserved, busy uv cache skipped, all temporary peer locks released. Free disk about55GiB. No successor compute started during delivery.

Primary runtime/source/check evidence: verified iCloud `codex-occurrence-joint-credit-20261009` (19,009,536B, MD5 `6b862a261543a357621db37748b3e8ff`). Final-head check logs, merge/blobs and final result receipt: verified fresh-roundtrip/index `codex-occurrence-joint-credit-closeout-20261009` (48,640B, MD5 `e29f4de1e32518db61bbb42c131a04c7`). Main keeps source and durable small records.

M2 result: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6086286608 ; M2 and #2028 bodies updated. Accepted8/512 and separate conditional negatives are unchanged. Complete claim-to-closeout elapsed **1392.069s**, including preparation, cold builds, scoped tests, reviews, preservation, protected delivery and cleanup; cumulative projected allowance remains628170s (prior624570+3600). No actual-parent gradients/construction/answers, GPU run or new pod in this piece. The unbudgeted useful-model goal remains active; the declared original-parent causal run is the next piece after this cleanup.



---
_Comment/review ids in this file: 6086259263, 6086286191, 6086320822_

