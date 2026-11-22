# ORDER for Decision D90 and Qwen3-0.6B Foundry Webharness LoRA pre-registration

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `3c01e23fd`. Records Owner Decision **D90** (updating `docs/integration/DECISIONS.md`, `docs/rules/changes.md`, and `PLAN.md`), and writes the CHARTER §4.5 pre-registration sheet (`PREP.md`) for fine-tuning `Qwen/Qwen3-0.6B` with LoRA on Foundry's full webharness toolset on Vast.ai before compiling `UORR4V13` + Stitch shards for Phase 3 Studio integration.

## Owner approval

Approved by Casey (owner) in Antigravity's chat at `2026-11-22T21:39:18Z`, recorded into `docs/integration/DECISIONS.md` (as D90, since D88/D89 were assigned on 2026-11-16) and `docs/rules/changes.md` (entry #21, correcting stale pre-existing summary count `9` -> `21`):

- Question 1 (Charter §3 Studio exemption & GPU LoRA run): "Approve D88 Studio-only exemption AND run a fresh GPU LoRA fine-tune of Qwen3-0.6B on Vast.ai specifically for Foundry's full toolset (workspace, GitHub, PrismPM, LexLean, standards, Linux VM) and publish new UORR4V13 shards"
- Question 2 (Linux terminal substrate): "Boot real Linux 6.18.7 (afflom/stitch v86 + 9p synced to Studio /workspace + Stitch Ed25519 WebRTC & host-fetch networking), keeping uor-r4-mvm RV64IMAC available as a lightweight secondary mode"
- Question 3 (GitHub authentication & push/pull): "Real browser-native GitHub via GitHub REST + Git Data API using Fine-Grained PAT sealed in core.wasm's ChaCha20-Poly1305 vault (plus optional user-configurable OAuth Worker relay URL)"

## Cancel probe

Verify `owner-paths` pass in `uor-harness loop pr-check`, `uor-harness ratchet d23/d25/d32` (`0` added), zero binaries (§3.13), `uor-r4-claim-wording`, `xtask links`, and non-author `FIT_VERDICT: FIT` + exact-head `VERDICT: APPROVE` on `PREP.md` before provisioning any GPU pod. Cancel immediately on any banned op inside `crates/**` or missing owner approval.

## Scope

Inside `docs/integration/DECISIONS.md`, `docs/rules/changes.md`, `PLAN.md`, and `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-4/**`:
- Record D90 (`2026-11-22`) authorizing the SHA-locked Studio-only interim Qwen3-0.6B `UORR4V13` runtime, the Vast.ai Foundry webharness LoRA + `UORR4V13`/Stitch recompile run (published to `Casey-allard/lexlean-r4`), `afflom/stitch` Linux 6.18.7 (`v86`) + Stitch WebRTC + secondary RV64IMAC MVM, and real browser-native GitHub REST/Git-Data API (vault-sealed PAT + optional OAuth relay URL).
- Record Entry #21 in `docs/rules/changes.md` (correcting stale summary count `9` -> `21`) and update `PLAN.md`'s 2026-11-22 owner-direction row.
- Pre-register the Vast.ai GPU LoRA run in `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-4/PREP.md` and obtain non-author `FIT` + PR A exact-head approval before spawning GPU compute.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local` (`PASS owner-paths`, `PASS commit-order`, `PASS order`, `PASS report`), `xtask links` (`0` broken links in changed files), `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if `owner-paths` fails, non-author reviewer finds `PREP.md` `NOT_FIT`, or any `crates/**` file is modified.
