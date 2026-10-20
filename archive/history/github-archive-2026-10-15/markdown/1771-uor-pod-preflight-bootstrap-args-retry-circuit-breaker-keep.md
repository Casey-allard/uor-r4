# #1771 — uor-pod: preflight bootstrap args, retry circuit breaker, keep failed bootstrap logs (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T00:44:53Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-10-06T00:46:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1771
- **Merged:** 2026-10-06T00:46:52Z
- **Merge commit:** 2c2abb7e96a127b9bb0478cb2913d4509575b42a
- **Base / Head:** main / UOR-Foundation:claude/uor-pod-preflight-breaker
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

On 2026-10-05 one session (`claude/gate7c`) ran `uor-pod up` 14 times in 2 h. Every bootstrap exited rc=2 within 4–7 s, and each pod was created, billed and then deleted. The ledger shows the bootstrap received a valid 40-character SHA (`a0747e97…`, which is on `origin/claude/step7c-gate-supervision`). Per the coordinator, the cause was `--with-ollama`: `ollama-linux-amd64.tgz` now returns 404, so `tar` failed (the build itself passed). The reason was deleted along with each pod.

Based on `origin/main` at 258e1c66, which already includes #1765.

## What changes

**1. Preflight in `up`, before any Runpod API call**
- `up` now refuses options it does not take (for example `--gpus` or `--via`) and any stray positional word.
- `--ref` accepts a branch, a tag or a 7–40 hex SHA (upper-case is accepted) and resolves it to the full 40-character commit. That commit must be on a branch of the public repository, because the pod fetches from GitHub.
  - A SHA that is not a branch tip is expanded and checked in the local checkout. First the remote's branches are fetched into the private namespace `refs/uor-pod/origin/*`; this does not touch `origin/*`.
  - Unknown, ambiguous, local-only, malformed and too-short values are all refused, and each message says nothing was created.
- The exact bootstrap argument vector is run through the bootstrap's own parser on the laptop (`uor-pod-bootstrap.sh --check-args`). It is built once by `bootstrap_args`, and the real run uses the same vector.
- Dry runs now resolve and check `--ref` too.
- Existing checks (`--gpu`, `--count`, `--hours`, `--wait-hours`, helper files) still run before the API.

**2. Circuit breaker**
- If the same lab+session has ≥ 2 `up-failed` events within 60 min, the next `up` exits 5. It prints each failure's time, pod, reason, last output line and log paths.
- The refusal is recorded as an `up-refused` ledger event.
- `--force-retry` overrides the breaker and is recorded as `up-force-retry`.
- Other sessions are not affected, and older failures do not count.

**3. Failed bootstrap logs are kept**
- On the pod, the bootstrap output is teed to `/workspace/uor-r4/pods/bootstrap-<pod>-<UTC>.log`, followed by an exit-status marker. This log is lost with an off-volume pod, and the ledger marks that case with `log_on_volume`.
- A full copy is saved on the laptop at `$STATE/logs/bootstrap-<pod>-<UTC>.log`.
- Before deleting the pod, `up_failed` prints the last 30 lines. The ledger `up-failed` event records `log`, `local_log`, `log_on_volume` and `last`, and the board comment includes the last line. `--keep-on-failure` prints the same tail.
- Latent bug fixed: `do_bootstrap` runs inside an `if !`, where `set -e` is off. If `resolve_ref` failed there, the bootstrap ran with an empty `--sha`. It now returns a failure instead.

**4. Ollama is non-fatal (coordinator's addition)**
- The install tries the current `ollama-linux-amd64.tar.zst` first, then the legacy `.tgz`, then the official `install.sh`. Files from the installer are copied onto the volume, and models go in `OLLAMA_MODELS=/workspace/ollama`.
- If `zstd` is missing, the bootstrap installs it with apt (best effort).
- A failed install or model pull warns and sets `"ollama": "FAILED"` or `"pull-failed"` in `BOOTSTRAP_RESULT`. `up` warns, keeps the pod and records `ollama` in the `bootstrap` ledger event. Only build/parity failures are fatal.

The `uor-pod help` text now prints the whole header, and `docs/labs/compute.md` is updated.

## Checks at this head (c1d79832)

- `scripts/pod/tests/uor-pod-dryrun.sh`: **passed 187, failed 0** (main: 123).
  - Preflight: unknown, short, local-only, absent-40-char and metacharacter refs; short-SHA expansion; branch and default `main` resolution; foreign options; positional args; bad count, hours, GPU and wait-hours. Each refusal also asserts that no `runpodctl` call was made.
  - A bootstrap that rejects `--with-ollama` stops `up` on the laptop.
  - Direct `--check-args` cases.
  - Breaker (dry and non-dry): trips at 2 failures, shows reasons, last output and logs, records the ledger event, honours `--force-retry`, and ignores other sessions and old failures.
  - Non-dry failed bootstrap against a fake pod: 30-line tail, full local log, pod-side tee command, ledger fields, pod deleted.
  - `--force-retry` run with a failed Ollama: pod kept, warning shown, ledger `ollama: FAILED`.
  - Bootstrap `ollama_step` sourced under `set -e` with a fake curl: all sources 404 gives `FAILED` and the bootstrap continues, with the `.tar.zst` → `.tgz` → `install.sh` order asserted; `.tar.zst` install works; a failed pull gives `pull-failed`; the legacy `.tgz` still works.
- `shellcheck` is clean on `uor-pod`, `uor-pod-bootstrap.sh`, `uor-reaper.sh` and the test script.
- Dry-run and fake binaries only: no real pods, no Runpod API, no network fetch to GitHub in tests (a local bare repository stands in for origin).

Not exercised: a real pod running the new Ollama install against ollama.com (the `.tar.zst` asset and the apt `zstd` fallback are untested on Runpod).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6006800678 — 2026-10-06T00:45:58Z — Casey-allard

Exact-head check at c1d79832 (Claude lab): shellcheck clean on all four scripts; dry-run suite 187 passed / 0 failed; check_claim_wording exit 0. Preflight before any API call, per-session circuit breaker (≥2 up-failed in 60 min → exit 5 unless --force-retry), failed bootstrap logs kept on volume + laptop, Ollama install non-fatal with .tar.zst → .tgz → install.sh fallback. Real-pod Ollama path not yet exercised (next real `up --with-ollama` is the test). Approve at this head.


---
_Comment/review ids in this file: 6006800678_

