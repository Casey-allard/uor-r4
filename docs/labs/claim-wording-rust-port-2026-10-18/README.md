# Claim-wording gate: Rust port (2026-10-18, claude)

**What.** The claim-wording gate (issue #123, [formal vocabulary §2.1](../../formal_vocabulary.md))
is now the Rust crate `crates/uor-r4-claim-wording` (library plus binary
`uor-r4-claim-wording`; its only dependency is `regex`). The harness exposes it as
`uor-harness claim-wording [--repo DIR]`. Run it with:

```sh
cargo run -q --locked -p uor-r4-claim-wording -- .
```

**Why.** Owner rule D27 makes Rust the default for new tooling, with no Python where Rust will do.
The gate is a small text scan, which Rust handles directly.

**Semantics.** The crate is a line-for-line port of `scripts/check_claim_wording.py`:
- the scan set is `docs/**/*.md`, then `crates/*/README.md`, then `README.md`, with each group sorted by path components (Python `Path` order), and `docs/history/github-archive-*` is excluded;
- files that are not valid UTF-8 are skipped;
- the PROHIBITED, DISAVOWAL and PROOF_LINK patterns are copied verbatim and matched case-insensitively with Unicode `\b`/`\w`;
- the `claim-wording: allow` marker works as before;
- lines are split with Python `str.splitlines()` boundaries and trimmed like `str.strip()`;
- the output is unchanged: the same header, the same `  <path>:<line>: <text>` lines and the same pass line.

The exit code is 0 when clean and 1 on a violation. There is a new code, 2, for an unreadable tree; the Python script raised a traceback in that case.

**Parity evidence (this checkout).** The script and the binary produced byte-identical stdout with the same exit code in two runs:
- on the real repository, both exited 0;
- with a throwaway `docs/zz-parity-probe.md` containing one bare violation and one disavowed line, both exited 1 and reported one violation. The probe file was deleted afterwards.

The crate has 13 tests, including `real_repo_passes`. Two rules were broken on purpose to check that the tests catch them:
- dropping the DISAVOWAL clause made 2 tests fail;
- making PROHIBITED never match made 6 tests fail.

**Callers rewired.**
- `.github/workflows/ci.yml`: both claim-wording steps.
- `.github/pull_request_template.md`, `AGENTS.md`, `CONTRIBUTING.md`, `docs/harness/pm-rules.md`, `docs/formal_vocabulary.md` §1/§6 and `scripts/verify_qualification.sh`.
- The PR-body checklist label in `crates/uor-r4-harness/src/prgates.rs`, its fixtures in `rules/prlint-format-fixtures.json`, the matching Python `scripts/harness/prlint.py` label (kept so the two prlint implementations still agree) and the template copy in `tests/harness/test_prlint.py`.

Dated records under `docs/labs/`, `docs/history/`, `docs/evidence/`, `archive/` and the dated `docs/integration/*` step notes keep their original text.

**CI coverage is unchanged.** The claim step stays in the same two jobs: `fast-gate` (dispatch `product_path`) and `gates` (dispatch `release_qa`). `fast-gate` installs Rust only for code changes, so it gains one bare `dtolnay/rust-toolchain@stable` step for docs-only changes. That keeps the claim gate running on docs-only diffs without making the job's compile unconditional. As before, ordinary `pull_request`/`merge_group` runs publish only the `required-transport` acknowledgements and do not run this gate.

**Python retired, not deleted.** `scripts/check_claim_wording.py` keeps its logic. A `RETIRED 2026-10-18` line under its docstring points to the crate, and the file stays as dated evidence.

**Known parity edges.** A fresh-context review found these. None is hit by any file in the repo today.
- **Unicode `\w`:** Rust's `\w` includes combining marks (category M), and Python's does not. A disavowal word containing a combining mark (for example `nó`) is matched differently. Parity is claimed for the repo's ASCII disavowal words.
- **IO errors:** Rust prints a clean error and exits 2; Python crashed with a traceback and exit 1. A directory named `*.md`, which crashed Python, is recursed into by Rust.
- **Symlink loops:** the walker does not follow-guard them. No symlinks exist under `docs/` or `crates/`.
- **`real_repo_passes`:** this test ties the crate's own test suite to the live docs, so a future unlinked phrase anywhere in `docs/` also fails `cargo test -p uor-r4-claim-wording`. That is intended for a gate.

In `fast-gate`, the claim step runs after the cargo cache step, so code changes reuse the cached `regex` build.

No model compute, GPU or paid resource was used.
