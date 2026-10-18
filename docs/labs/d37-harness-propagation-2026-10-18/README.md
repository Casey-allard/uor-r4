# D37 harness propagation — 2026-10-18

**Status:** host-only documentation and harness propagation complete; no source behavior, compute,
permission or hold changed.

**Base:** `origin/main` `35c4c40380ad25b2ba4cde0239b93110b26768db`; branch
`kimi/d37-propagation-2026-10-18`.

**Authority:** merged [D37](../../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)
at `6788a8fc5`. This record mechanically propagates that owner decision; it adds no permission and
does not reinterpret it.

## Scope

D37 permits float forward passes, gradients and optimizers, and offline numerical training
instruments, in a separately scanned float module with a reported export gap. The float lane must
remain geometric (SpiralCore/E8/FBS, icosian/quaternion, prime/zeta), as D32 item 2 requires, and
the module gets its own scanned allowlist entry, added and reviewed with its pre-registration,
never by widening the discrete crate's scan. D37 leaves D23/D25 squashing bans, D25/D30 dense
matrix products, D24 GPU-only placement, D11 integer/table serving with no
float/transcendental/multiplier instruction, the October 14 complex-plane zeta-zero exception, the
active discrete path (`crates/uor-r4-discrete`, Dose 1) and every hold unchanged.

The work was limited to making the harness and current-authority documents state exactly that.

## Files

- `docs/harness/milestones.json` — D32 summary scoped to the active discrete path and Dose 1; new
  all-scope `D37` owner rule sourced to `docs/integration/DECISIONS.md`.
- `crates/uor-r4-harness/src/prep.rs` — `APPLICABLE_RULES` 9→10 with D37; both applicable-rules
  tests and their count updated.
- `scripts/harness/prep.py` — matching rule tuple includes D37.
- `docs/harness/pm-rules.md` — R5 pins policy schema `/7` and the serving
  float/transcendental/multiplier false commitments; R13 is scoped to the active discrete path,
  with a separate D37 offline float lane under R12/D31 that cannot weaken discrete exact
  instruments.
- `.kimi-code/AGENTS.md` — offline float only under D37 in a separately scanned module; dense
  matrix products remain banned under D25/D30; stale serving reference corrected to D11.
- `docs/integration/project-track.md` — current direction, D25 note, post-dose path and the
  ~1280/1284 note now state D37 without rewriting dated history.
- `docs/labs/optimization-path-2026-10-18/README.md` — replaces the stale 2026-10-12 offline-float
  sentence with D37 authority and names the separate zeta exception.
- `docs/harness/claims.json` — only `kimi/d37-propagation-2026-10-18`, unioned with peer claims.
- `docs/integration/current-state.md` — one newest-first entry for this propagation.

No other path changed.

## Checks

| Check | Result |
|---|---|
| `CARGO_TARGET_DIR=local/harness-target cargo test --locked --offline -p uor-r4-harness prep::` | PASS — 14 selected tests, including both `applicable_rules_tests`; 0 failed |
| `python3.12 scripts/harness/parity.py --build` | PASS — prep sheets, milestone registry and rendered sheet identical; all agreement checks ok |
| `local/harness-target/debug/uor-harness check` | PASS — exit 0. Reviewed run: 122 rules, 0 errors, 87 warnings, 1 info finding (88 non-ok). Final pre-merge re-run: 120 rules, 0 errors, 86 warnings, 1 info (87 non-ok). Merged-head run: 114 rules, 0 errors, 79 warnings, 1 info (80 non-ok). Live peer claim/branch state moved each time; no `harness.owner_rules_applied` finding |
| `local/harness-target/debug/uor-harness prep-render -m m1` (stdout) | contains `- D37:` once; stale “Supersedes D25's offline scalar floating-point allowance, which is now inactive” count 0 |
| `CARGO_TARGET_DIR=local/harness-target cargo run --locked --offline -p uor-r4-core --bin r4-policy-check -- .` | PASS — `uor-r4.agent-execution-policy/7`; serving float/transcendental/multiplier false, training matmul false |
| `local/harness-target/debug/uor-harness ratchet d23` | PASS — 2,253 sites / 154 files, baseline 2,253 |
| `local/harness-target/debug/uor-harness ratchet d25` | PASS — 93 sites / 26 files, baseline 93; 0 tensor-core sites |
| `local/harness-target/debug/uor-harness ratchet d32` | PASS — 31 sites / 1 file, baseline 31 |
| `python3 scripts/check_claim_wording.py` | PASS |
| `cargo fmt --check -p uor-r4-harness` | PASS |
| `git diff --check` | PASS |

The `uor-harness check` warnings are pre-existing peer-claim, presence-staleness, PENDING-sheet,
claim-hygiene and orphan-branch findings plus the owner spending-cap info reminder. The live count
drifts as peers merge and release claims; both runs exit 0, and none concerns D37, this diff, or
`harness.owner_rules_applied`.

**Python-version note.** The system `python3` is 3.9.6, whose `datetime.fromisoformat` cannot parse
the ledger's `Z` receipt timestamps; it therefore omits the brief's 24-hour receipts section and
`python3 scripts/harness/parity.py --build` exits 1 on that pre-existing brief-only mismatch.
Rule/prep/render parity is still reported `ok`, and the same parity script passes every check under
`python3.12`. This change does not touch `briefing.py`/`briefing.rs`.

## Cost and boundaries

$0. No pods, GPU/model compute, training, evaluation, inference, paid service, push, PR or merge.
The D32 discrete crate, Dose 1 sheet/packet, policy JSON, D23/D25 baselines and all holds are
unchanged.

## Next dependency

Independent exact-head review followed by protected PM delivery. Dose 1 dispatch/decision remains
the current path dependency. Any offline float work still requires its own new FIT'd
pre-registration, a separately scanned module and a reported export gap; this propagation releases
no hold, FIT or compute.
