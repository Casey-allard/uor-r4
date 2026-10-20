# #2278 — D27: Rust first, and the uor-foundation ecosystem as the shared floor

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:45:13Z
- **Updated:** 2026-10-13T03:48:12Z
- **Closed:** 2026-10-13T03:46:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2278
- **Merged:** 2026-10-13T03:46:53Z
- **Merge commit:** 56350f55a3e5c3e39cca46a0e79ed7d102b9c8ed
- **Base / Head:** main / UOR-Foundation:kimi/d27-rust-first-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## D27: Rust first, and the uor-foundation ecosystem as the shared floor

Governance docs only. The decision itself is the record: `docs/integration/DECISIONS.md` § D27.
Decision: **KEEP** (a recorded owner direction; no capability, artifact or measurement is claimed or moved).

### What landed

1. **`docs/integration/DECISIONS.md` — D27**, in the file's existing format (Authority line quoting the owner's
   chat direction of 2026-10-13, drafted by the Kimi lab; then the substance):
   - **Rust is the default for all new project code** — model, training, tooling, data preparation, scripts — for
     efficiency, native UOR compatibility and WASM reach. Python only where Rust genuinely cannot do the job;
     the existing `scripts/harness/` scripts are grandfathered, and no new Python is added where Rust works.
   - **The uor-foundation sibling repositories are the shared floor, reuse before rewriting**, at the pinned
     revisions: `uor-addr` (canonical content addressing — already pinned, rev `165b51e3e`), `UOR-Framework`
     (the typed UOR ontology — already pinned as `uor-foundation` / `uor-foundation-sdk` /
     `uor-foundation-verify`, rev `51c013822`), **`uor-matmul` in its lawful parts only** (exact dyadic
     accumulation with one final rounding, the codec/codebook identity machinery, the tabulated no-multiply
     path, and the tropical (max,+) lane `uor_matmul_core::dot_tropical_ref` — already pinned for parity in
     `crates/uor-r4-graph-cli`) with **its dense GEMM entry points banned in every executed training and
     serving path** under D25, `prism` (standard-library facade, transitive through `uor-addr`), and
     `LexLean` / `PrismPM` as **pattern-level** reuse (claim register, honesty gate, packaging/attestation).
   - **Every interface surface imported from a sibling gets a D25 compliance check first**; a dormant upstream
     does not block pattern-level reuse with attribution.
   - **Nothing is retroactive**: existing artifacts, measurements and the historical Python/dense references
     keep their exact scope as inactive evidence, and no hold changes.
2. **`AGENTS.md`** — the "Use Rust for data preparation, training, artifact construction and inference" bullet
   in the native-geometric policy block gains the one-sentence D27 default, with the sibling crates named as
   the ecosystem floor to reuse and the Python-only-where-Rust-cannot rule.
3. **`docs/integration/current-state.md`** — newest-first entry: D27 recorded, what it changes (nothing
   retroactive; new code defaults to Rust; sibling reuse before rewrite; D25 check on imported surfaces), and
   headline unchanged.

### Every factual claim in D27 was checked against the tree before it was written

- `uor-addr` pinned at rev `165b51e3e` in the root `Cargo.toml` and `crates/uor-r4-core/Cargo.toml`; `uor-prism`
  appears in `Cargo.lock` as its transitive; `UOR-Framework` pinned at rev `51c013822` (three crates).
- `uor-matmul` is pinned at rev `b13c984` and already used by name: `uor-r4-model-source` (its exact
  arithmetic) and `uor-r4-graph-cli` (`uor_matmul_core::dot_tropical_ref`, for the ⊗/⊕/`-inf` parity test).
  The arithmetic and codec descriptions come from `docs/integration/uor-source-audit.md`, which also records
  that upstream is 7 commits ahead of the pin — hence D27's "a repin is its own compatibility decision".
- `LexLean` is already inventoried in `docs/integration/afflom-ecosystem-sources.json` /
  `architecture-2026-09/imports.md`; D27 states plainly that neither it nor `PrismPM` is a dependency here.

### Verification

- [x] Checks run at this exact head, offline, on the owner's laptop
- [x] `python3 scripts/harness/cli.py check` — **0 errors**; the sheet's `prep.fit_before_run` warn (rung 0 is
      `PENDING`, correctly), four other milestones' unfrozen panels, other labs' orphan branches and the
      standing caps notice are pre-existing
- [x] `python3 scripts/check_claim_wording.py` — PASS (it ran because D27 and the AGENTS bullet are policy text
      adjacent to capability claims)
- [x] `python3 scripts/check_no_softmax.py` — **OK, 1649** (baseline 1649; docs-only, untouched)
- [x] `python3 scripts/check_no_matmul.py` — **OK, 95 sites / 26 files** (baseline 95; untouched)
- [x] Every relative link in the three edited files resolves, except the **10 pre-existing** broken links in
      other labs' older `current-state.md` entries (verified identical on the base commit — not introduced here)
- [x] Git-tracked harness bookkeeping from the required claim/heartbeat (`docs/harness/claims.json`,
      `docs/harness/ledger/kimi-2026-10.jsonl`, `docs/harness/presence/kimi.json`) is included, as the harness
      intends the committed record to be the fallback
- [x] Docs only — no Rust source touched, so no cargo build or test was run
- [x] Files the same *deliberate* `prlint.record_path` warn as the rung-0 sheet PR: a governance decision's
      record is the decision itself (`DECISIONS.md` § D27), so no dated `docs/labs/…` copy was created

First application, named so the record is checkable: the rung-0 source work already registered on M1 #2029
(the SpiralCore/FBS projection, the OP3a head, the recurrence replacements, the device-qualification gate)
is now a D27 test as well as a D25 one — those items are Rust source, reusing the sibling crates where
they fit.

Pre-registration: not applicable — no compute and no model numbers in this PR. D27 changes no hold.

Cost: one documentation session on the owner's laptop, no compute, no pod leased.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

