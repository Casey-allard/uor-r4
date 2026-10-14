# Rung-0 completion — owner release plan (D4 gate, per seed)

Date: 2026-10-14. Milestone: M1 #2029. PM: Kimi (kimi/pm1); drafted by the Kimi
lab on `kimi/completion-release-plan` for PM review.

This directory is the non-authorizing release packet for the rung-0 completion
(seed 1: resume from checkpoint 1,640 through batch 16,404; seed 2: full pass
0..16,404), governed by the completion sheet
`docs/harness/prep/m1-discrete-rung0-completion-2026-10-14.md` (FIT (design),
Amendments 1–3, merged #2456 at `4efd3f95b`). It contains no owner decision and
releases nothing. Under **R15**, no lab writes, edits, copies or generates
`owner.json` or any stand-in; **D33's delegated pilot authority does NOT extend
to the completion** (sheet Amendment 3 item B) — each seed needs its own
owner-personal release and a seed-1 release cannot chain seed 2. Training and
M2 remain HELD until those releases exist; a merged packet is pending, not a
release.

## Contents

- `SEED1_PLAN.json` — the seed-1 `CompletionPlan` (schema
  `uor-r4.completion-owner-release-plan/1`, status `HELD_OWNER_ACTION_REQUIRED`).
  It binds the execution identity, the seed-1 `DriverConfig`, the exact
  checkpoint-1,640 parent `FilePin`, and the SHA-pinned `sheet.md` below.
- `SEED2_PLAN.json` — the seed-2 `CompletionPlan`; same identity, seed-2
  `DriverConfig`, no parent (`pilot_parent: null`: fresh initialization).
- `seed1-config.json` / `seed2-config.json` — the same `DriverConfig`s as
  standalone files for the pod-side `completion` and
  `completion-build-identity` commands (spec V4096/B4/C192/K8, batch 64,
  schedule `canonical_vector_then_read_v1`, `bos <|bos|>`/0; identical to the
  respective embedded configs; the two differ only by `seed`).
- `sheet.md` — byte copy of the completion sheet at merge `4efd3f95b` (#2456;
  FIT (design), Amendments 1–3), sha256
  `4c317f1eb344d60de520d9f82786af99b2353ca14a08c690fa568e6c59d6f8a3`.
- `seed1-review.json` / `seed2-review.json` — **not yet present**: Claude's
  binding `APPROVE_A2_COMPLETION_ADMISSION_DIFF` records, written against the
  final pinned identities after the qualified build (§ Open items b/c).
  Until then the plans' `claude_review` pins carry an explicit placeholder.

Each plan carries exactly two placeholder fields — `execution.executable_sha256`
(`PENDING_QUALIFIED_COMPLETION_BUILD`) and `claude_review.sha256`
(`PENDING_CLAUDE_BINDING_REVIEW`). Both are filled only from the qualified build
and Claude's binding review (§ Open items b/c); every other pin is final in this
draft. `completion-check-plan` fails closed on any placeholder, so the packet is
deliberately not executable in this state.

## Execution identity (pinned)

| Field | Seed 1 | Seed 2 |
| --- | --- | --- |
| scope | `m1-a2-completion-seed1-checkpoint1640..16404` | `m1-a2-completion-seed2-checkpoint0..16404` |
| base_source | `e7d926fbe38537f4faa90b817705c61a780778d6` | same |
| reviewed_source_commit | `33564b75e02e945ec5d5c94740c5741362289236` | same |
| source_sha256 (CUDA) | `e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a` | same |
| kernel_sha256 (PTX) | `931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b` | same |
| geometry_sha256 | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` | same |
| executable_sha256 | *pending qualified build (b)* | *pending qualified build (b)* |
| driver_source_sha256 | `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4` | same |
| config_sha256 | `6252b613c6bf84efeb0dbdcfde69877eb75da1ec6d868eaac3461d93d75a3d04` | `029965ae487cf1b869ec38ee406929c765f650edc445d4b0935d833be5dbfd4d` |
| lineage_sha256 | `94330d90f764c9245a7e9f4acabdd580c19d4e518a8a339a0bf772c0899da402` | `36bd0c03bfb40121366320c1c81b656705f56de5258f1c20161a0c56080799bc` |
| sheet_sha256 | `4c317f1eb344d60de520d9f82786af99b2353ca14a08c690fa568e6c59d6f8a3` | same |
| run_state_directory | `/workspace/uor-r4/rung0-completion-20261014/seed1-run-state` | `/workspace/uor-r4/rung0-completion-20261014/seed2-run-state` |
| pilot_parent | checkpoint-1640.json, sha256 `46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440` | none (fresh init) |

Source/executable/PTX/geometry provenance:

- `source_sha256`/`kernel_sha256`/`geometry_sha256` are retained **exactly**
  from the pilot plan (`docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json`);
  the executing code refuses any other value (`completion must retain the exact
  qualified numerical source/PTX/geometry`). The qualified completion build must
  therefore compile the frozen `#2454` source unmodified and reproduce them
  (Claude N3; § Open items b).
- `reviewed_source_commit` is the frozen discrete source Claude approved:
  `33564b75e02e945ec5d5c94740c5741362289236` (`APPROVE_A2_COMPLETION_ADMISSION_DIFF`,
  [#2454 comment 6164246420](https://github.com/UOR-Foundation/uor-r4/pull/2454#issuecomment-6164246420)
  at head `c8bf51909c4c767aab04e51797c995ad93d936f7`; `git diff 33564b75e c8bf51909
  -- crates/uor-r4-discrete` is empty, re-verified here). PR #2454 is still
  **OPEN** — its merge sha is recorded at delivery (§ Open items a). The
  `driver_source_sha256` below binds exactly these source bytes: the driver hash
  is SHA-256 over the concatenation of `corpus.rs`, `training.rs`,
  `controller.rs`, `completion.rs`, `main.rs` in that include order.
- `driver_source_sha256`, `config_sha256` and `lineage_sha256` were
  reconstructed with serde declaration-order encoding. The method was validated
  by reproducing three published values **exactly**: the seed-7 lineage
  `2cebef05e321cb52654b9866900b874c079d73a81b4da4f27d28d72156d90f9c`
  (`docs/labs/discrete-driver-cost-2026-10-13/receipts/coverage-plan.json`), the
  pilot seed-1 `config_sha256` `6252b613…`, and the pilot seed-1
  `lineage_sha256` `6db5ecf5d6e1b334b6f788a28b741b7ff2e012e7bf451e5f89ed29f89ef82a12`
  (both cross-confirmed on the qualified pilot binary, #2029 comment 6160885825);
  the driver-hash method additionally reproduced the pilot driver
  `236e7fb6…` from the pilot source. Seed 1's lineage changes only in the
  driver hash (migration); seed 2's differs by the seed in the config.
- `sheet_sha256` is the byte hash of `sheet.md`, byte-identical to the merged
  sheet. The executing code requires a line starting `TEST FITNESS: FIT` and the
  completion scope string, both present.

The checkpoint-1,640 parent is the pilot's final checkpoint, preserved in
`icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar` (MD5
`fb81eaa335a1d21ca1d05bde9771b3e3`, download round-trip verified). At staging it
must exist at the pinned path as the extracted archive lays it out
(`…/seed1-parent/rung0-pilot-20261014/pilot-attempt-1/checkpoint-1640.json`);
the completion refuses any other parent, digest or cursor. The parent file must
remain in place for the whole seed-1 run: every resume re-reads it.

## What the completion is (and is not)

**Seed 1 — resume.** Batches 1,640..16,403 inclusive (14,764 batches) from the
pilot's checkpoint 1,640 (`46e8ed5e…`, preserved as above; cursor
1,640/236/104,960, vector counters 1,640/1,179/236 under the old driver). The
236 accepted pilot updates are part of the trajectory; restarting from
initialization would discard a measured, admitted, paid-for prefix. Seed 1
**keeps its old-lineage baseline** (Amendment 3 item A): the checkpoint-0
evaluation and `u = 105/14,287 ≈ 0.735%` stay exactly as measured under the
parent lineage — no bar, guard, baseline or dose is re-derived — while quarter
evaluations run under the completion lineage. The only permitted lineage delta
is the operational driver hash (model bytes, cursor, config, corpora and
baseline unchanged).

**Seed 2 — full pass from initialization.** Batches 0..16,403 inclusive
(16,404 batches), same deterministic schedule and corpus order as seed 1, with
`u` measured at its checkpoint 0 **before any apply**. Seed 2 is **not chained
to seed 1**: it runs only under its own owner-personal release, decided by the
owner after the seed-1 100% record (§ Open items d, sheet Stop rule and D6).

**Not in scope:** no mechanism, law, schedule, initialization, data or evaluator
change; the collision-free initializer, frequency-ordered revisit, admission/
address-objective configuration and flock/gate reads all remain named successors,
not adopted here. Route (kind 0) gets no corpus-scale coverage beyond its
registered 20 coordinates; the pilot's Route row (attempted 0 / NOT_RUN) becomes
a real sensitivity row when the pass reaches it at batch 16,384. That first
kind-0 apply is covered as a **finite fixture** by
`docs/labs/discrete-driver-cost-2026-10-13/receipts/driver-controls.json`
control 2 (`route_apply_reality`: Route(0) candidate 1 proposed, applied on the
numerical path, loss 640→256, PASS at base source `e7d926fbe`) — cited as
finite-fixture coverage of a kind-0 apply, not corpus-scale or completion-runtime
qualification (Claude N2, closed by this evidence).

**Dose and coverage.** The registered pass is 16,404 one-coordinate batches:
VectorBlock 0..16,383 then Route 0..19, one coordinate per batch, no wrap. Seed 1
finishes batches 1,640..16,403; seed 2 runs all 16,404. Each seed consumes
1,049,856 of 2,425,925 eligible masked TRAIN target positions (1,376,069
unvisited); both seeds traverse the same prefix, so unique exposure does not
double. P1/P2-style reports are produced at every scored checkpoint.

**Decisive checkpoints:** 0 / 4,101 / 8,202 / 12,303 / 16,404 per seed; bars,
guard and any transfer are decided at 16,404 only. Seed 1's checkpoint 1,640 is
an extra diagnostic point already measured (pilot end: top-1 82, top-8 1,169,
strict wins 149); intermediate checkpoints are never selected post hoc.

**Cost (estimate, not invoice; charged cumulatively):** seed-1 remainder 14,764
batches at the pilot-measured 3.7457 s/batch ≈ **15.36 h ≈ $13.70** at
$0.892/h; seed-2 full pass 16,404 batches ≈ **17.07 h ≈ $15.23**; combined
≈ **32.4 h GPU ≈ $29**, plus dev evals (~19.5 s each × 3 quarter + 1 final per
seed), checkpoints, preservation, setup/build and release round on the same
lease; retries inside the same lease are charged. **Working storage:** the
measured checkpoint is 3,138,466 bytes uncompressed; at ~260 checkpoints per
seed the budget is **~0.8 GB per seed** before evaluations, logs and overhead —
the pilot's ~6 MB compressed archive is not the working-storage figure.

## Bars, guard and diagnostics (pre-registered, unchanged)

Carried over verbatim from the sheet; nothing here re-derives them.

- **Bar (i):** raw-score top-1 on the 14,287 pinned dev targets beats the
  frozen masked-TRAIN count predictor by ≥ 5pp on **both seeds** (reference
  1,226 top-1 = 8.58% → bar ≈ 13.6% ≈ 1,941/14,287).
- **Bar (ii):** recall@8 over the declared kept list beats the reference by
  ≥ 5pp on **both seeds** (reference 4,060 top-8 = 28.42% → bar ≈ 33.4%
  ≈ 4,775/14,287).
- **Guard:** strict distance-win rate ≥ max(1%, min(5u, u+5pp)) on both seeds,
  `u` per seed at checkpoint 0. Seed 1: `u = 105/14,287 ≈ 0.735%` → threshold
  ≈ **3.67% ≈ 525 strict wins** (old-lineage constant, Amendment 3 item A).
  Seed 2: `u` measured at its checkpoint 0. `u > 95%` → INSTRUMENT-INVALID and
  bars decide alone; never a negative.
- **Guard meaning (Amendment 3 item E):** the guard predicate
  (`token_rivals > 0 && token_hinge < width`) does **not** require the target
  cell to be admitted; baseline 105 strict wins = 0 admitted; pilot end 149 =
  36 admitted / 113 omitted. A guard pass is not a count of executable writer
  successes.
- **Admission ceiling (B3):** `discrete_evaluate` ranks only over admitted
  cells, so top-1/top-8 hits ≤ target-cell admitted rows =
  `admitted_cell_count_sum − address_rival_count_sum`, reported at every
  checkpoint with each seed's checkpoint-0 value as baseline. Pilot receipts:
  baseline 114,296 − 108,616 = 5,680; pilot end 114,296 − 112,116 = 2,180 —
  below bar (ii)'s 4,775 and only just above bar (i)'s 1,941. On an
  admission-limited bar miss the first attribution is writer admission,
  alongside the schedule-first step; bars are never lowered.
- **Per-checkpoint diagnostics (sheet step 7 + Amendments 2/3):** P1/P2-style
  per-kind sensitivity (attempted / any-candidate-differs / accepted) for
  vector and Route; distinct full-vector patterns and occupied writer cells;
  token/address hinge sums and the aggregate numerator; count-reference
  recomputation check (mismatch reported, not absorbed); target-cell admitted
  rows; **admitted strict wins** (strict win ∩ target-cell admitted), nested
  `admitted strict wins ≤ top-1 hits ≤ target-cell admitted rows`.
- **Noise floor:** Wilson 95% half-width < 0.83pp at n = 14,287; bars wider
  than noise. **Fresh transfer: UNAVAILABLE / NOT_RUN** — a KEEP-with-transfer
  claim needs its own prospective amendment first.

## How D4 is released (owner-personal, one per seed)

The release path is Casey personally, at a live terminal, with `UOR_LAB` unset
(the binary refuses a lab actor, a non-TTY, or any D33 invocation). From a
checkout on a main containing this directory, the completion binary built for
this purpose is run as:

```sh
completion-owner-release docs/labs/rung0-completion-release-2026-10-14/SEED1_PLAN.json NEW_RELEASE_DIR
completion-owner-release docs/labs/rung0-completion-release-2026-10-14/SEED2_PLAN.json NEW_RELEASE_DIR
```

On the host that runs the release and launch, the pinned `sheet.md` and
`seedN-review.json` absolute paths must resolve byte-identically (pilot
discipline: the pod mirrored the absolute paths without changing the plan);
the release refuses on any content-hash difference. The binary path is the one
Codex's pod build produces and confirms when the completion pod is staged.

It prints the pinned plan and requires typing exactly:

- seed 1: `I am Casey; release D4 for completion seed 1 only through batch 16404`
- seed 2: `I am Casey; release D4 for completion seed 2 only through batch 16404`

It writes `owner.json` (owner `Casey`, decision
`RELEASE_D4_FOR_THIS_COMPLETION_SEED_ONLY`, the plan sha256 and the
`owner-terminal:` receipt channel) plus `admission.json` into the fresh
`NEW_RELEASE_DIR`, and nothing else. The completion is then launched per seed:

```sh
completion docs/labs/rung0-completion-release-2026-10-14/seed1-config.json SEED1_RELEASE/admission.json NEW_REPORT_DIR
completion-resume ...   # same admission, a new report dir, resume only
```

Run-state, release and report directories must all be fresh and cannot overlap
the parent directory; a mismatch anywhere refuses before any apply (fail-closed).
A wrong pin makes the completion refuse, never misrun. Each release decision is
Casey's; every other hold stays owner-personal.

**Order of operations at release time** (nothing here is executed by this
packet):

1. Qualified build on the pod from the frozen source (N3); no source edits.
2. `completion-build-identity seed1-config.json` and
   `completion-build-identity seed2-config.json` on that binary (no CUDA
   context): cross-confirm compiled source/kernel/geometry equal the pilot pins
   and driver/config/lineage equal this packet's values for both seeds; record
   the executable sha256 and the binary path (§ Open items b).
3. Fill `execution.executable_sha256` in both plans from the qualified binary —
   the only identity field the build supplies.
4. Claude posts and writes the binding records
   `seed1-review.json`/`seed2-review.json` (verdict
   `APPROVE_A2_COMPLETION_ADMISSION_DIFF`, scope per seed, `execution` exactly
   the plan's filled execution, `receipt_url` a `#issuecomment-` receipt on
   #2029); fill the `claude_review.sha256` pins (§ Open items c).
5. `completion-check-plan` passes on both plans (read-only metadata; no CUDA
   context, no owner record, no release).
6. Casey releases seed 1, then seed 2 only per his separate decision (§ Open
   items d); the PM verifies each owner/admission record against the plan pins
   before launch, using the pilot's 14/14 identity-check discipline
   (#2029 comment 6160999031).

Filling steps 3–4 changes the plan bytes; any earlier verification is void and
the checks rerun. A crash during the very first checkpoint save (before
`latest.json` exists) leaves neither resume nor a fresh start — that needs a
new run-state directory, a re-pinned plan and a new owner release; it exposes
only the pre-run baseline and is noted, not required (Claude residual edge).

## Resource admission

- One RTX 5090 pod (Vast.ai default, D26) via `scripts/pod/uor-pod`, one job
  per GPU, `--lab`/`--session` identity and GPU lease for the whole job plus
  margin; renew (`uor-pod renew`) rather than cancel a healthy run — an
  expiring lease is never a reason to stop (standing authorization, recorded
  on #2029). Owner caps: ≤ 4 pods and ≤ $8/h across all labs.
- Wall/GPU: seed-1 ≈ 15.36 h, seed-2 ≈ 17.07 h GPU, plus evals, checkpoints,
  preservation and setup on the same lease; combined projection ≈ 32.4 h
  ≈ $29 (estimate; actuals charged cumulatively).
- Working storage ≈ 0.8 GB per seed (checkpoints, ungzipped) plus quarter
  evaluations, logs, the staged parent and build outputs; keep the
  128 MiB physical stop margin and the 30 GiB free-space floor; no `/tmp`
  work.
- Preservation: run-state evaluation files plus every attempt directory and the
  original parent together; iCloud `cloud-store put` with MD5 round-trip;
  never copy the owner record. Do not keep stopped pods as storage.
- D24/D32: model computation is GPU-only; no CPU model compute and no
  non-decisive GPU work. All staging is metadata-only; `completion-check-plan`
  and `completion-build-identity` create no CUDA context.

## Open items (before any release)

- **(a) #2454 merge sha.** The D5 admission diff PR #2454 is APPROVED
  (Claude, comment 6164246420) but still OPEN; Codex merges it. Record its
  merge sha at delivery on #2029 and confirm the discrete crate stays
  byte-identical to `33564b75e` (this packet binds the frozen source bytes, so
  the merge sha is a provenance record, not a re-pin).
- **(b) Qualified build pins (Claude N3).** The executing build must compile
  the exact frozen source, unmodified (any source edit voids the D5 review and
  this packet's identity fields). From that build: fill
  `executable_sha256` in both plans; cross-confirm `driver_source_sha256`,
  `config_sha256` and `lineage_sha256` for **both seeds** and the retained
  source/kernel/geometry pins via `completion-build-identity` (the pilot's
  build-identity discipline, #2029 comment 6160885825: seven
  executable/driver/source/PTX/geometry/config/lineage comparisons, no CUDA
  context). New CUDA compilation/runtime is **UNVERIFIED** until then. Record
  the emitted pins, the binary path and the cross-confirmation on #2029.
- **(c) Claude's binding review records (N4).** `validate_plan` requires
  `claude_review.execution == plan.execution`, including the build-supplied
  `executable_sha256`, so the binding `APPROVE_A2_COMPLETION_ADMISSION_DIFF`
  records can only be written after (b). Two records, one per seed scope, with
  a receipt URL on #2029; their sha256s become the plans' `claude_review` pins.
  Claude's source-level APPROVE is already posted; this is the
  executable-bound record.
- **(d) Owner's two release decisions (D6).** Two separate owner-personal
  releases, one per seed; a seed-1 release cannot chain seed 2. After the
  seed-1 100% record, the owner decides whether seed 2 proceeds (the sheet's
  stop rule: a far-below-bar seed-1 reading is reported first and is iteration
  evidence, never a stop or a negative by itself), and at release time the
  owner also decides the dose-sequencing question (continuous two-seed vs
  seed-1-then-decide). This packet releases nothing.
- **(e) Claude's one-line re-confirmation of sheet Amendment 3** is still
  outstanding on #2029 ("I'll confirm Amendment 3 in one line when it
  merges"; Amendment 3 merged as #2456). Not a code or build gate; recorded
  here so the sheet's FIT trail is complete before the owner releases.
