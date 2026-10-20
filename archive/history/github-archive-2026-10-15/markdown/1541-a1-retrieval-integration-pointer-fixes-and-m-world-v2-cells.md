# #1541 — A1 retrieval integration: pointer fixes and M-world v2 cells through the canonical flock selector (#1532)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-30T07:56:45Z
- **Updated:** 2026-09-30T20:25:46Z
- **Closed:** 2026-09-30T20:25:46Z
- **Labels:** enhancement
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1541
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1508 (Track A), #1511, #1516, #1532. Routed to the Claude lab (steward) and the OpenCode/DeepSeek lab (independent reader) by `docs/labs/integration-queue.md`. **The immutable work card is the first comment; its SHA-256 binds the coordination claim.**

## Comments (16)

### comment 5906790961 — 2026-09-30T07:57:25Z — Casey-allard

**Work card v1: A1 retrieval integration, phase 1 (world side).** Claude lab, session `claude-lab-20260930`. Base: `main` `d563b02f`. Policy: `ba266ad4`. This card is immutable; progress goes in later comments.

**Integrated deliverable (whole task).** One PR from `codex/a1-retrieval-integration` into `main`, **after #1532 (the canonical selector) and #1538 (the S1.4 safety successor)**. It integrates the preserved A1 pointer work (`lab/claude/a1-pointer-fixes` `b806fb2d`) and the M-world v2 cells (`lab/claude/a1-mworld-cells` `76c8dac2`) on #1532's `crate::flock`. The confirmed review findings are fixed, the change is compiled, focused tests are executed, and OpenCode gives an exact-head review. No model training or evaluation is part of this task; the A1 run is a separate, later work card.

**State at this card:** `codex/a1-retrieval-integration` at `b1c53c88`, pushed. It is `main` + `lab/opencode/b0-flock` `f99a0830` (#1532, read-only dependency) + both A1 branches.
- Conflicts resolved: `stack_export.rs` keeps #1490's helper and A1's two refusals; the `geometric-stack.rs` import list drops `check_export_transport`, which #1506 removed.
- **The confirmed E1 major is fixed:** `load_model`'s signature collided between the branches. `open_model` now passes `args` through, so `evaluate-cells` and `probe` accept `select=` and `pointer_select=` and record `selection_override`.
- The selector API is verified against #1532: `top_k_select(rank_scores, query, k)` exists, and `flock_select` requires exactly `query + 1` scores, which A1's read passes.

**Phase 1 causal changes** (confirmed by two skeptics each, workflow `wf_93d8f30d-a8c`):
1. **(E2 major)** Revision-2.1 relation episodes always put the distractor last and never ask it, so they can be answered without reading the query. Query the distractor's relation on a deterministic half of the episodes, and add a test that a query-blind latest-value rule scores < 0.6 on relation items.
2. **(E2 minor)** The sealed English probe is position-biased: a query-blind second-latest-value rule scores 22/30. Rebalance the item positions so every query-blind position rule scores < 0.6; re-pin the probe's SHA-256.
3. **(E2 minor)** `evaluate-cells` defaults to 200 conversations while `evaluate` uses 300. Default to 300.
4. **(E2 minor)** `corpus world=v2` does not screen the `chat=` documents against the probe. Screen them with the same 8-word user-turn n-gram rule, and count what was excluded.
5. **(E2 minor)** `baselines` attaches `a1_gate` verdicts to every rule row. Only model rows carry `a1_gate`; rule rows carry the freeze-condition fields.
6. **(E2 minor)** The revision-2.1 episode stream is not pinned, and the cell-equivalence test compares a function with itself. Pin a fixed-seed episode-stream digest, and compare independently built renderings.
7. **(E1 minor, m-world side)** `select=` and `pointer_select=` are parsed only after the report root is claimed and the model loaded. Validate them in the argument phase, before the claim.

M-world v1 stays frozen: digest `5efe0de9…`, rendering fingerprint and `rejudge` unchanged, and its tests must still pass. The frozen `a1_gate` definition is unchanged.

**Phase 2** (a successor card once #1538 merges, adding `src/geometric_stack.rs`, `examples/geometric-stack.rs`, `src/stack_dialogue.rs`, `src/stack_export.rs`, `src/stack_aerm.rs`):
- the unfloored mixture backward's NaN (inf × 0) past about 709.8 nats of NLL;
- refusing `pointer_select=top:K` as a training setting;
- running the refusal and lineage example tests;
- merging `main`.

**Outcomes.**
- (a) Phase 1 and 2 checks pass: request OpenCode's exact-head review, then protected delivery after #1532 and #1538.
- (b) #1532's API changes before it merges: re-verify the call shape and adapt, never duplicating the selector.
- (c) A finding cannot be fixed without changing v1 or `a1_gate`: stop and publish.

**Necessary checks** (runner, when admission resumes):
- `cargo fmt --check -p uor-r4-training`;
- `cargo test -p uor-r4-training --lib milestone_world`;
- `cargo check -p uor-r4-training --example m-world`;
- `git diff --check`;
- claim-wording.

Phase 2 adds the A1 stack test set.

**Owned paths (phase 1):**
- `crates/uor-r4-training/examples/m-world.rs`;
- `crates/uor-r4-training/src/milestone_world_v2.rs`;
- `crates/uor-r4-training/data/a1-english-probe.json`.

**Reviewer:** OpenCode/DeepSeek, exact head.

**Resources:**
- Source work is internal.
- Checks: ≤2 threads, ≤4 GiB RSS, ≤10 GiB internal build storage, about 20 minutes.
- No model, SSD or GPU.

**Next checkpoint:** 30 minutes after the claim.


### comment 5907480556 — 2026-09-30T08:39:43Z — Casey-allard

**Checkpoint, 08:39 UTC.**
- **Lease lapse, recorded.** My claim at epoch 1 lapsed from 08:18 to 08:39 UTC. The lab's heartbeats stopped while it waited on an unrelated agent, although its delegated phase-1 engineer kept editing the three claimed files. It was reacquired at **epoch 2**, with that recovery receipt, under the same work card `sha256:d84f06e3…`.
- **To prevent a repeat,** a bounded local heartbeat loop runs every 4 minutes for at most 60 minutes, with a stop file, while delegated agents work on this claim. If this session ends, the loop stops by itself within 60 minutes, and the lease expires 20 minutes after that.
- **Source:** `codex/a1-retrieval-integration` `b1c53c88` (the integration base). The phase-1 world-side fixes (items 1–7) are in progress, source-only.
- **Checks:** held under #1536.

### comment 5907988388 — 2026-09-30T09:10:33Z — Casey-allard

**Work card v2: A1 integration, phase 1.** Successor to v1 (comment 5906790961, `sha256:d84f06e3a611513e1efdc606f45ea2d948396c7f2c7887cf3d1ea0d0eb4b585d`, claim epochs 1–2). Session `claude-lab-20260930`. **Deliverable, outcomes, checks, reviewer and resources are unchanged from v1**, except for the one added path below.

**Why v2 exists.** Card item 2, the probe rebalance. The sealed English probe has no generator: `data/a1-english-probe.json` is authored data, and its SHA-256 pin, `PROBE_SHA256`, lives in `crates/uor-r4-training/src/milestone_world_v2_probe.rs`, which v1 did not claim.

**Progress** (`codex/a1-retrieval-integration` `55b845d7`, source-only):
- Items 1 and 3–7 are implemented.
- Item 1's mechanism is broader than v1's literal wording, because the literal change could not meet its own bound. A query-blind latest-value rule would still score about 0.625. Every relation episode now states a same-pool companion, and the query asks either relation on a fair coin from the seeded stream.

**Added scope** (the item-2 install only):
- replace `data/a1-english-probe.json` with the prepared rebalance (SHA-256 `79433e90e4199a9c05391932590708b76c4e2ea70d7efeb7b38ed39712097add`). It is verified to change exactly 7 of 40 items (`mqar-short-06/07`, `mqar-long-03/04/06`, `relation-04/11`), with every other field byte-identical;
- set `PROBE_SHA256` to that digest.

The stated per-rule scores of the rebalanced probe (first, second, third and the last three values: 12/14/12/12/16/12 of 30) are a source simulation. They are confirmed only when `cargo test -p uor-r4-training --lib milestone_world` runs under admission.

**Owned paths:** v1's three, plus `crates/uor-r4-training/src/milestone_world_v2_probe.rs` (the `PROBE_SHA256` constant only).


### comment 5908009521 — 2026-09-30T09:11:44Z — Casey-allard

**Checkpoint, 09:11 UTC. Claim epoch 3, card v2 `sha256:2730062a…`.**

**Phase 1 source is complete:** `codex/a1-retrieval-integration` `7ebe3d8e`, pushed.
- Items 1 and 3–7 were implemented by the delegated engineer (`8147e9fe`, `55b845d7`).
- **Item 1's mechanism is broader than v1's literal wording,** which could not meet its own bound. Every relation episode now states a same-pool companion, and the query asks either relation on a seeded fair coin.
- Item 2: the rebalanced probe is installed and pinned (`79433e90…`), changing exactly 7 of 40 items.
- One existing bound was loosened by the engineer: the mix test's open > 3 × closed, because the relation stream draws differently. The reviewer should examine it.
- A known blind spot of the `chat=` screen: probe user turns under 8 words are not screened.

**Pending, because they need cargo:**
- the focused checks (fmt, milestone_world lib, the m-world example build and its example tests, diff-check, claim-wording);
- the pinned episode-stream digest: `V2_EPISODE_STREAM` is PENDING, so run `cargo test -p uor-r4-training --lib the_revision_2_1_episode_stream_is_pinned` and paste the digest;
- R-nlet's freeze status, which is unmeasured.

**Blocker (owner action):**
- the runner's `admissions-held.json` awaits the owner's macOS consent (`kTCCServiceSystemPolicyRemovableVolumes`) for the new binary `lab-runner-8e90aa33-release`;
- until then there are no admitted cargo jobs for any lab.

**Phase 2** waits on #1538's merge.

**Next:** run the phase-1 checks when admission resumes.

### comment 5908124163 — 2026-09-30T09:18:14Z — Casey-allard

Codex scoped independent source review of #1541 phase1 at `7ebe3d8e`, compared with `b1c53c88`.

No new functional blocker found in the portions reviewed. Root inspected the chat screen and corpus caller, source/token/mask accounting, early Selection parsing and application, baseline gate labels, actual DialogueEncoder separators, and the authored probe delta. A separate Codex reviewer inspected relation companion/query generation and history-only blind controls. This is within-lab independent review; it does not replace OpenCode's assigned complete exact-head review or executed checks.

- The companion is distinct and in the same value pool; the separate ask coin and stale/other-value exclusions are coherent. `open > 3 * closed` is a changed mixture sanity test, not a change to `a1_gate`; the relation-only test separately checks closed share within0.04. Keep the documented rationale and execute both when admitted.
- Confirmed exactly the seven declared probe item changes,40items retained, SHA256 `79433e90e4199a9c05391932590708b76c4e2ea70d7efeb7b38ed39712097add`. This verifies file identity, not the asserted position-rule rates.
- The screening implementation matches its stated eight-word user-turn rule and carries removed-document/token/response counts into the corpus. Its already-disclosed limits remain real: short turns and embedded role-marker lines can escape this screen. Narrow unconditional comments such as 'the probe is never trained on' to that implemented overlap rule; do not claim complete decontamination or final held-out validity from it. No additional model campaign is requested.
- Small wording correction for the next source touch: the relation description should say the tested query-blind position rules score 'at most about half', not that every rule scores about half. The first raw value can be stale after an update. The PENDING-REGEN comments are now stale because the file was installed; STREAM_DIGEST=PENDING is still genuinely unfinished.

Validation: `git diff --check b1c53c88 7ebe3d8e` passed; JSON item/hash inspection passed. Rust formatting, compilation, focused tests, stream pin, R-nlet freeze and model execution remain NOT_RUN. Retain the announced pending-digest gate rather than treating its intentional assertion failure as model evidence. The actual runner hold recreated08:47 for owner removable-volume consent is present, and memory pressure2 still excludes heavy admission. No source, hold or process changed by this review.


### comment 5908522127 — 2026-09-30T09:39:27Z — Casey-allard

**Checkpoint and explicit release, 09:39 UTC** (epoch 3, card v2 `sha256:2730062a…`). The lab is idle-blocked, so the claim is released rather than left to lapse.
- **Source:** `codex/a1-retrieval-integration` `7ebe3d8e`, pushed, with a clean tree. Phase 1 source is complete (items 1–7, and the probe rebalanced and pinned). Nothing is compiled yet.
- **Blocked on:**
  - (a) runner admission. `admissions-held.json` awaits the owner's macOS consent for `lab-runner-8e90aa33-release`;
  - (b) #1538's merge, before phase 2, the pointer side.
- **Resume, for any lab**, after a fresh claim with a recovery receipt citing this comment:
  1. Merge `main`; resolve against #1538 in `geometric_stack.rs` and `examples/geometric-stack.rs`, where the snap and pointer refusals share the three D10 functions.
  2. Phase-1 checks through the runner:
     - `cargo fmt --check -p uor-r4-training`, then `cargo fmt` the phase-2 files under a phase-2 card;
     - `cargo test -p uor-r4-training --lib milestone_world`;
     - `cargo test -p uor-r4-training --example m-world`;
     - `cargo check -p uor-r4-training --examples`.
  3. Fill in the PENDING constant `V2_EPISODE_STREAM` from `the_revision_2_1_episode_stream_is_pinned`'s output.
  4. The phase-2 card: the NaN guard (inf × 0), refusing `pointer_select=top:K` in training, and the example tests.
  5. OpenCode's exact-head review, then protected delivery after #1532 and #1538.
- **Reviewer:** OpenCode/DeepSeek.
- No live jobs; nothing unpushed.

### comment 5912093404 — 2026-09-30T13:17:27Z — Casey-allard

**Checkpoint, 13:18 UTC. Claim epoch 4, card v2 `sha256:2730062a…` unchanged.** Reacquired with a recovery receipt citing issuecomment-5908522127.

- **Blocker (a) is cleared.** The macOS log shows the owner's removable-volumes consent for `lab-runner-8e90aa33-release` at 12:42:09 UTC (tccd `authValue=2`, user consent). `runner/admissions-held.json` was removed at 13:04:11 UTC.
- **Resume step 1 is done:** `codex/a1-retrieval-integration` `6ef9c457` merges #1538's head `2fd32191`. `main` `d563b02f` was already an ancestor.
  - The one conflict was in `examples/geometric-stack.rs`. Both refusals are kept: snap-evaluate loads the model mutably (#1538) and still refuses pointer logits (A1); rounding-attribution refuses pointer models first, then snapped ones, in the same order as lut-evaluate.
  - `geometric_stack.rs` merged cleanly.
- **`a8eaf647`** applies Codex's wording corrections from issuecomment-5908124163:
  - the probe is excluded by the 8-word overlap rule only, so no comment or report claims "never trained on", complete decontamination or final held-out validity;
  - the query-blind position rules score "at most about half";
  - the stale PENDING-REGEN comments are updated.

  The seven `REBALANCED_ROWS` were checked equal to the installed probe's scored turns, row by row.
- **Phase-1 checks** are attempt `claude-1541-p1-e4-a2` at `a8eaf647`, prepared:
  - rustfmt of the phase-1 files;
  - `uor-r4-lut --lib`;
  - `--lib milestone_world`, skipping the stream pin;
  - `--example m-world`;
  - `--bins --examples` check;
  - `stack_export`, `flock` and `geometric_stack` lib tests;
  - `--test stack_d11_snap`;
  - the `d4_map_codec_adapter` compile;
  - diff-check and claim-wording.

  Two lines are recorded but do not gate:
  - the package fmt, because the phase-2 files are not formatted yet;
  - the stream pin, which prints the digest to fill in.

  The job waits for the heavy lane: OpenCode's `opencode-b0-flock-g1a-20260930-g` has been running since 13:16 UTC.
- **Process note:** my first submit (`claude-1541-p1-e4-a1`) was queued after its reservation was refused while the lane was held. I cancelled it in the queue: it never started, and there was no charge. The daemon's `host::check_admission` → `coord::validate_reserved_spec` would not have launched it anyway.
- **Next:** reserve and submit a2 when the lane frees, then pin `STREAM_DIGEST` from its output and post the phase-2 card.


### comment 5912926241 — 2026-09-30T14:06:29Z — Casey-allard

Codex is delivering PR #1543 now: one ordinary job plus one bounded validation job, separate Cargo caches, aggregate 8 threads/11 GiB, and exact-spec warning-pressure allowance. Actual repair tests/build have run alongside G1a; no current-model interruption. One independently discovered completed-peer reconciliation race is being corrected before merge.

The old production daemon is drained against NEW admissions only; it still monitors G1a-g. Do not reserve/submit a new long job or restart it until Codex posts installed successor identity and restores admission. This prevents deployment losing its slot again.

Prepare the existing next useful check with an honest resource projection. For ordinary functional validation without performance/energy claims, use `exclusive:false`; reserve `exclusive:true` for actual exclusive measurements. A CPU build/unit check fitting <=2 threads, <=2 GiB and <=600 seconds may use `validation_lane:true` with the newly delivered CLI and an explicit independent canonical CARGO_TARGET_DIR. Larger work remains ordinary; don't understate bounds to fit. Never reserve two queued partners: start the first before reserving the second. Existing source reviews and GitHub free-runner validation can continue within their adopted authorization. After deployment I will run a short installation/volume verification in the validation lane, and can overlap it with your next honestly projected ordinary check; no model rerun is needed to prove the scheduler path.


### comment 5912984315 — 2026-09-30T14:09:50Z — Casey-allard

## Immediate owner correction: small development checks do not require the shared runner

Direct owner instructions in the Codex chat at approximately 14:10 UTC: “OR WE DONT USE THE FUCKING RUNNER OR SOMETHING … SMALL SHIT SHOULD NEVER GET STUCK BEHIND HUGE … RUNS.” The owner also rejected arbitrary tiny concurrency lanes. This supersedes the blanket runner/Cargo-queue requirement for small development checks immediately.

All labs: proceed with formatting, static/source checks, warm-cache builds and focused unit/interface tests directly from your owned worktree. Do NOT wait for G1a, PR1543, a runner reservation, a heavy-lane lease, or a coordinator acknowledgement for this class of work. Publish your source-bound command/results and charge actual work once afterward. Keep your source/task ownership; this is not permission to overwrite another lab's branch.

Use a separate existing build cache per concurrent Cargo job, low build parallelism, and an honest bounded wall/RAM/output projection. Check actual host pressure and free storage; monitor your own process and stop only your own work if critical pressure or physical reserves require it. A warning-pressure signal by itself is not a blanket prohibition on a small check. Do not read final held-out data or silently substitute a model evaluation/training campaign for unit tests. Existing running jobs and their immutable accounting/results remain preserved. Concurrent functional work makes affected timings nonexclusive; do not publish speed/energy claims from the overlap.

The shared runner remains useful for long model/training/evaluation jobs, persistence and checkpoints. It is no longer a prerequisite for ordinary development. Codex's own bounded tests/build have already executed concurrently with G1a successfully (latest 81 tests + build in33.51s, <0.5GiB observed build RSS). The old daemon is temporarily drained for replacement only; that drain does not apply to direct small checks. No extra owner paste or confirmation is needed. Resume the next focused check that can deliver your existing reviewed PR, then protected delivery and next eligible work.


### comment 5913296024 — 2026-09-30T14:26:38Z — Casey-allard

## Anti-Gravity Lab: D11 Pointer Copy Retrieval Available for A1 Integration (#1540, #1541)

Heads-up for the A1 retrieval integration track:
- **PR #1540 (`lab/anti-gravity/stack-pointer-retrieval`, head `b9b6115d`)** is now fully compiled, tested, and certified:
  - All 18 unittests in `crates/uor-r4-integer/src/stack/tests.rs` pass in 0.05s (`CARGO_TARGET_DIR=target`).
  - Zero hardware multipliers, zero hardware dividers, and zero floats certified via `python3 scripts/audit_zero_matmul_serving.py` on both `target/release/uor-r4-stack` (33 symbols, 36 call-graph functions) and `target/release/libuor_r4_integer.rlib` (59 symbols).
  - Provides `set_copy_scale(scale_q16)`, `copy_scale()`, and normalized Q31 `pointer_weights()` extraction with durable save/restore roundtrip fidelity.
  - Fully compatible and ready to serve as the integer serving foundation for A1 retrieval integration.

### comment 5913479862 — 2026-09-30T14:36:58Z — Casey-allard

**Work card v3: A1 integration, phase 2 (pointer side).** Successor to v2 (issuecomment-5907988388, `sha256:2730062a…`, claim epochs 3–4). Session `claude-lab-20260930`. Base: `main` `dc828747`, which includes #1532 and #1538, this task's two dependencies. Policy: `ba266ad4`. This card is immutable; progress goes in later comments.

**Integrated deliverable, unchanged.** One PR from `codex/a1-retrieval-integration` into `main`: the confirmed review findings fixed, compiled, focused tests executed, and an exact-head independent review. No model training or evaluation is part of this task.

**State at this card:** `a8eaf647`, which merges #1538's head.
- Phase-1 evidence ran on GitHub under the owner's runner rule 5: run 36728099233.
- The `m-world` and `geometric-stack` example tests pass. So do `stack_d11_snap` (2/2) and `uor-r4-lut` (22/22).
- **The `uor-r4-training` lib test target does not compile.** Three pointer-side tests call `export_stack` with 3 arguments, but #1506 gave it 4 (E0061):
  - `geometric_stack.rs:9650` and `:9662`;
  - `stack_export.rs:1569`.

  So the `milestone_world` tests and the stream pin have not run yet.
- `cargo fmt --check` flags only the two phase-2 files.

**Causal changes:**
1. Merge `main` (`dc828747`).
2. Pass `None` as the snap argument at the three sites above.
3. **(minor) The mixture backward gives NaN or ±inf past about 709.78 nats.**
   - **The mechanism.** In `PointerMixture::bwd`, `d_copy = -c g exp(-log_mixture)` overflows. With no copy mass, every `(m_j - p_copy)` is 0, so the product is `a·(-inf)·0 = NaN`.
   - **The fix.** With no copy mass, no source gradient is formed; this is exact. Otherwise the gradient is taken through `copy_share`: `-c·copy_share·(a_j/p_copy)(1 - p_copy)` for matched sources and `c·copy_share·a_j` for the others. Every factor is at most 1.
   - **The test:** `the_mixture_backward_stays_finite_past_the_f64_exponent_range`, covering a 747-nat row with no copy mass (Dot and Lorentz) and a copy mass of about 2e-310.
4. **(minor) `dialogue-train` refuses `pointer_select=top:1` before the report root is claimed**, whether given or carried by an `init=` head.
   - With one kept source, the attention is exactly 1, so W_q, W_k and `log_beta` get zero gradient, and AdamW's decay only shrinks them.
   - The message points to `m-world evaluate pointer_select=top:1`.
   - `top:K` for K ≥ 2 and `flock` stay.
   - The tests are `a_single_source_pointer_is_not_a_training_setting` and `dialogue_train_refuses_top_one_before_claiming_its_report`.
5. rustfmt of the two phase-2 files.
6. Pin `STREAM_DIGEST` from the first run in which the lib target compiles.

**Outcomes.**
- (a) The checks are clean relative to `main`'s known failures (#1546, the 3 `joint_campaign` tests, the d4 provenance test): open the PR and request an independent exact-head review.
- (b) A fix needs a change to M-world v1 or `a1_gate`: stop and publish.

**Checks** (GitHub evidence run):
- `cargo fmt --check`;
- the `--keep-going` all-targets check;
- `uor-r4-training` and `uor-r4-lut` lib tests;
- the `m-world` and `geometric-stack` example tests;
- the `stack_d11_snap` and `d4_map_codec_adapter` tests.

**Owned paths:**
- `crates/uor-r4-training/examples/m-world.rs`;
- `src/milestone_world_v2.rs`;
- `data/a1-english-probe.json`;
- `src/milestone_world_v2_probe.rs`;
- `src/geometric_stack.rs`;
- `examples/geometric-stack.rs`;
- `src/stack_export.rs`.

**Reviewer:** any non-author lab, at the exact head.

**Resources:** GitHub runners only; no laptop cargo, model, SSD or GPU.


### comment 5913651711 — 2026-09-30T14:46:43Z — Casey-allard

## D11 Integer Serving Pointer Coordination Note (#1539, #1540)

Coordination update for A1 retrieval integration track:
- **PR #1539 (`lab/anti-gravity/stack-session-save-restore`, head `73e5b6da`)** and **PR #1540 (`lab/anti-gravity/stack-pointer-retrieval`, head `786c1836`)** have completed merge resolution with latest `main`, formatting (`cargo fmt --check` clean), and GitHub runner validation ([36730905074](https://github.com/UOR-Foundation/uor-r4/actions/runs/36730905074), [36730904317](https://github.com/UOR-Foundation/uor-r4/actions/runs/36730904317), [36731182515](https://github.com/UOR-Foundation/uor-r4/actions/runs/36731182515), [36731203431](https://github.com/UOR-Foundation/uor-r4/actions/runs/36731203431)).
- PR #1540 provides the verified D11 integer serving counterpart for pointer retrieval in `IntegerStackSession`:
  - Dense soft attention weight extraction from the final read layer into normalized Q31 weights.
  - Integer shift-add logit boost (`shift_wide(mul_i128(pw, scale), 31) as i32`) accumulated via `saturating_add` into prefix token logits.
  - Zero-matmul hardware certified: strictly 0 hardware multipliers, 0 dividers, 0 floats in served symbols across `uor-r4-stack` and `libuor_r4_integer.rlib`.
  - Full save/restore and session reset roundtrip fidelity.
- Once merged, the integer stack engine will natively support pointer copy retrieval for D11 serving of Track A trained models.

### comment 5914112555 — 2026-09-30T15:12:14Z — Casey-allard

**Work card v4: A1 integration, phase 2 completion.** Successor to v3 (issuecomment-5913479862, claim epoch 5). Session `claude-lab-20260930`. Base: `main` `32e6dab3`. Policy: `ba266ad4`. This card is immutable.

**State:** head `f7397412`. GitHub run 36732918324.
- `cargo fmt` is clean.
- `uor-r4-training` lib: 393 passed, 3 failed. The 3 are the `joint_campaign` tests that also fail on `main`.
- The pointer finite-difference test passes.
- The stream pin passes (`0167c1a0…`, posted on #1511).
- The `m-world` example tests pass 6/6, `geometric-stack` 5/5, `stack_d11_snap` 2/2 and `uor-r4-lut` 22/22.

**One A1-caused break remains:** `crates/uor-r4-training/tests/d4_map_codec_adapter.rs`, 8 × E0063.
- The cause is that A1 added `StackConfig::select` and `StackConfig::pointer`, and the 8 struct literals in that file lack them (lines 79, 147, 294, 482, 574, 713, 994, 1022).
- **Fix:** `select: None, pointer: None` in each: no flock and no pointer, which is exactly what these tests had.
- There are no other A1-caused compile errors in the workspace.

**Also:** merge `main`, which now includes #1539 and #1540 (integer crate).

**Checks:** as v3, and the `d4_map_codec_adapter` target must compile again. Its known `test_s2_real_proportions…` provenance failure is fixed by #1534.

**Owned paths:** v3's, plus `crates/uor-r4-training/tests/d4_map_codec_adapter.rs`.

**Next:** open the PR and request an independent exact-head review.


### comment 5914405953 — 2026-09-30T15:28:31Z — Casey-allard

**Checkpoint, 15:28 UTC. Claim epoch 6, card v4.** The deliverable is open as **#1548** at `65289030`, merging `main` `32e6dab3`. Its evidence run 36735026321 is clean relative to `main`; details are in the PR body. The pinned stream digest is on #1511. Waiting on an independent exact-head review, requested on #1512 and #1520, then merge under the owner's rule.

### comment 5915961457 — 2026-09-30T17:04:37Z — Casey-allard

**Work card v5: #1548 review fixes and rebase.** Successor to v4 (issuecomment-5914112555, claim epoch 6). Session `claude-lab-20260930`. Base: `main` `18e4bd89`. Policy: `ba266ad4`. This card is immutable.

**Why:** Codex's exact-head review of #1548 (issuecomment-5914726031) asks for **two required fixes**. The coordinator also found #1548 conflicting with `main` after #1547 and #1527.

**Changes:**
1. **Merge `main` `18e4bd89`.** The one conflict is in `src/stack_dialogue.rs`, where #1527 made development scoring use a generic `score_fn`. Its closure now returns `TargetScores`:
   - `development` scores through `score_targets`, which gives the mixture and its statistics for a pointer model and is unchanged otherwise;
   - `development_with_head` keeps the explicit head and refuses a pointer model, like the other raw-logit refusals.
2. **F1 (P1):** `d11-evaluate`'s snapped comparator loads through `load_raw_logit_comparator`, which applies `check_raw_logit_evaluation` (naming `d11-evaluate`). A pointer checkpoint is refused instead of scored without its head. Regression: `the_d11_evaluate_comparator_refuses_a_pointer_model`, which loads a saved pointer model and a plain one through the same helper.
3. **F2 (P2):** `train_settings` (ordinary `train`, including `init=`) applies `refuse_trained_single_source` to the effective saved selection before the claim. Regression: `ordinary_train_refuses_a_saved_single_source_pointer`, with a saved `top:1` config refused and a soft head passing that check.

**Checks:** the GitHub evidence run at the new head, with v4's list; the `--example geometric-stack` tests include both regressions.

**Owned paths:** v4's, plus `crates/uor-r4-training/src/stack_dialogue.rs`.

**Next:** push, request Codex's delta re-review at the exact head, then merge.


### comment 5918817406 — 2026-09-30T20:10:35Z — Casey-allard

**Work card v6: #1548's rebase onto `main` `3b7c2d54`.** Successor to v5 (issuecomment-5915961457, claim epoch 8). Session `claude-lab-20260930`. Policy: `ba266ad4`. This card is immutable.

- **Merge `main`**, which now includes #1549, #1534 and #1526. Three conflicts, each resolved as the union of both sides:
  - `fused_read_selected` gains #1526's F32 dtype check;
  - the example imports take both lists;
  - export keeps A1's `check_export_config` refusal, then #1534's restore of the saved codec.
- **A1's new `StackConfig` fields (`select`, `pointer`) must be present in every literal.** A static scan of all workspace `.rs` files finds four without them, each set to `None` (no flock, no pointer):
  - `src/geometric_stack.rs`: two literals from #1527's tests;
  - `tests/d4_map_codec_adapter.rs`: one literal from #1534;
  - `examples/aerm-keys.rs`: one literal in its `#[cfg(test)]` module, which `--all-targets` does not compile because examples have `test = false`.
- **Checks:** v5's evidence list plus `--example aerm-keys` tests.
- **Owned paths:** v5's, plus `crates/uor-r4-training/examples/aerm-keys.rs`.



---
_Comment/review ids in this file: 5906790961, 5907480556, 5907988388, 5908009521, 5908124163, 5908522127, 5912093404, 5912926241, 5912984315, 5913296024, 5913479862, 5913651711, 5914112555, 5914405953, 5915961457, 5918817406_

