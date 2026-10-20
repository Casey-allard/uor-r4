# GNAF vendor-in provenance (#653 phase 1)

Records the exact, reproducible provenance of the `proofs/wasm-gemm-gnaf/`
vendor-in — the source/provenance merge step of #653's proposed
implementation shape ("Land the source/provenance merge first, then a
small r4 adapter layer..."). This is **phase 1 only**: an inert,
non-serving, non-wired addition. The full integration matrix required by
#653's scope ("classify every top-level GNAF layer and claim as
`adopt-active`/`retain-formal-reference`/`defer-open`/`reject`... produce
`docs/gnaf_integration_653.md`") is explicit follow-up work, not attempted
here — see "Not in this phase" below.

## Exact provenance

- **Upstream repository:** [`afflom/WASM-GEMM-GNAF`](https://github.com/afflom/WASM-GEMM-GNAF)
- **Pinned commit:** `171652cd95c0b8e8620f76151b7e0c485e30ccfc` — the exact
  commit #653's own issue text audited (`WorkloadIncomplete`, specific
  named open obligations). Upstream `main` has moved past this commit since
  (`917306fd2b5a397ab02c5d38918fb8620fcc5ae0` as of 2026-08-16); this
  import deliberately stays on the already-audited pin rather than
  following HEAD. Bumping the pin is separate, reviewed future work that
  re-runs the integration matrix, authority/toolchain/license checks, and
  mutation suite (per #653's own "Dependencies and blockers" section).
- **GNAF authority document:** `proofs/wasm-gemm-gnaf/authority/UOR-GNAF-v1-draft.2.md`,
  independently re-verified at this pin: `sha256sum` =
  `5c342373b2ff809bfd607c413cafd0582d32bb097544c6597ff7d674fe99200a`
  (matches the digest recorded in #653's issue text).
- **Lean toolchain:** `leanprover/lean4:v4.30.0` (from
  `proofs/wasm-gemm-gnaf/lean-toolchain`, verified present at the pin).
- **Licenses:** dual `LICENSE-APACHE` (Apache License 2.0) and
  `LICENSE-MIT` (MIT), both vendored verbatim at
  `proofs/wasm-gemm-gnaf/LICENSE-APACHE`/`LICENSE-MIT`.
- **Import method:** `git subtree add --prefix=proofs/wasm-gemm-gnaf
  https://github.com/afflom/WASM-GEMM-GNAF.git
  171652cd95c0b8e8620f76151b7e0c485e30ccfc` (no `--squash`) — the upstream
  repository's 5 commits up to the pin (`fdd58db9` initial commit through
  `171652cd`) are preserved as real commit objects in `r4`'s own history,
  reachable via `git log --graph`; the merge commit message records the
  exact prefix and pinned SHA. This is a real repository merge, not a
  submodule reference, so **a fresh `git clone` of `r4` already has the
  vendored source with no extra checkout step** — satisfying #653's own
  "allows a fresh r4 clone to verify without an extra checkout" scope item.
- **Size:** ~3.5 MB vendored (`du -sh proofs/wasm-gemm-gnaf`) — no file
  over ~700 KB, no binaries; no git-lfs tracking needed. Marked
  `linguist-vendored` in `.gitattributes` so GitHub's language stats/diff
  treat it as third-party code.

## Update policy

The pin does not float. A future bump to a newer upstream commit is a
separate, reviewed change (per #653's own text) that re-runs: the
integration matrix, the authority/toolchain/license audit, the spec-derived
obligation inventory, and the mutation/falsifier suite — not a routine
`git subtree pull`.

## Not in this phase (deferred, tracked on #653)

- The full integration matrix (`docs/gnaf_integration_653.md`) classifying
  every top-level GNAF layer/claim.
- Any adapter wiring of GNAF claim vocabulary into graph-certify/
  target-operator certificate or API result records.
- Any CI wiring for Lean verification (build, mutation, reproducibility,
  or authority/axiom-audit checks) — the upstream `.github/workflows/`
  (`mutation.yml`, `reproducible.yml`, `verify.yml`) are vendored as files
  but **not** registered as r4 CI workflows; no `lean-toolchain` install
  happens anywhere in `r4`'s own `.github/workflows/`.
- The Atlas update/rebuild discipline applied to any r4 pipeline seam.
- Falsifiers ported into r4's own repo-conformance/xtask ownership.
- Anything that changes a serving path, runtime operation vocabulary, or
  default engine (explicitly out of scope per #653's own non-goals).

This vendor-in is, deliberately, dead weight from r4's own build/serve
perspective today: nothing in `Cargo.toml`'s workspace `members` references
`proofs/wasm-gemm-gnaf` (it is a Lean/Lake project, not a Rust crate, and
is not auto-discovered by any glob), and no Rust code imports or reads
anything under this path. It exists to be built on in #653's next phase.
