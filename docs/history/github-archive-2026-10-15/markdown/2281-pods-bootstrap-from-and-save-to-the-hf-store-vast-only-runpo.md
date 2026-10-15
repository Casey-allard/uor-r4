# #2281 — Pods bootstrap from and save to the HF store; Vast only, Runpod as emergency backup (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:55:31Z
- **Updated:** 2026-10-13T04:00:05Z
- **Closed:** 2026-10-13T03:58:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2281
- **Merged:** 2026-10-13T03:58:55Z
- **Merge commit:** a10aac63328d41e950c3165f7e0d1acc3725ea6c
- **Base / Head:** main / UOR-Foundation:claude/pod-hf-store-2026-10-13
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: infrastructure · no count · headline unchanged (M1 v5 memory 20/40, reply 37/232). Holds unchanged (no training ran). Owner requests 13 Oct: everything a pod needs for CUDA development, tests and training must come from (and go back to) the HF store; "we dont use runpod anymore" with Runpod "as a backup secondary only if … all work is blocked"; "gpu (cuda) and rust if possible over python".

## What

- **Bootstrap from the store** (`uor-pod-bootstrap.sh`): Rust toolchain, crate cache and the binaries for the commit/GPU (`bin/<sha>-sm<cap>.tar`) come from `caseyallard/uor-r4-store` when `/workspace` lacks them; a pod that made one uploads it. A store build is used only if it holds plain files, `BUILD.json` names this commit and GPU with parity PASS, and `SHA256SUMS` lists exactly the expected binaries and verifies. A bad archive falls back to rustup/crates.io. Build jobs come from cgroup `cpu.max` (Vast reported nproc=128 on 16 CPUs).
- **`uor-store`** on every pod: `put` / `get` (fresh-dir download, never reports an old local copy) / `ls` (curl + jq, all pages).
- **`down --to-store`**: a Vast pod uploads its results to `vast-archive/<pod>/` (no laptop copy, no size cap).
- **Vast only; Runpod an emergency backup**: `UOR_POD_PROVIDERS` defaults to `vast`; `up --wait` falls back to Runpod after 30 min with no Vast offer when Runpod has stock (ledgered, boarded); Runpod by hand needs `--owner-approved`. D26 amended; AGENTS, compute.md, README updated.
- CUDA toolkit stays with the image / NVIDIA apt (not redistributable). Records: [pod-hf-store](docs/labs/pod-hf-store-2026-10-13/README.md), current-state.

## Checks at head `c2cc969f625315129f659e692b3d2835060bfcb6`

- `scripts/pod/tests/uor-pod-dryrun.sh` → **passed 348, failed 0**. Each safeguard set aside fails its cases: checksums 2, exact binary list 2, upload after a build 2, no store call inside the build step 1, `get` missing path 1, `get` old local copy 1, Runpod-by-hand refusal 1, 30-min stretch 2.
- `bash -n` clean; shellcheck: 2 warnings, both pre-existing on main. `check_no_softmax` 1649/1649, `check_no_matmul` 95/95, claim wording pass, `harness check` rc 0.
- Independent review: REQUEST CHANGES (registry upload before publish; stale `get`; stricter store-build acceptance; bad archive aborting; cpu.max noise; `--to-store` messages) — all fixed with tests.
- **Live (Vast 5090):** cold pod built and uploaded toolchain (612 MB), crate cache (149 MB), binaries (143 MB) in 330 s; a fresh pod took everything from the store and was ready in **57 s** with no build; offline `cargo check --features cuda --tests` passed in 43 s from the store's crate cache; `nvcc` kernel 0 wrong of 1,048,576, `compute-sanitizer` 0 errors, NVRTC 12.8, cuda-gdb/ncu present, `nsys` via `cuda-nsight-systems-12-8`; `uor-store` put→get round trip, verified from the laptop. The real store build passes the stricter check (re-verified after the review). Both pods destroyed (~$0.15).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5498645768 — 2026-10-13T04:00:05Z — Casey-allard — COMMENTED

PM review (kimi/pm1) — LGTM. Store-bootstrapped pods are the right shape for Vast (no shared volume): binaries keyed by commit+GPU, BUILD.json must name this commit/GPU with parity PASS, SHA256SUMS verified, and a bad archive falls back to rustup/crates.io rather than executing unverified bytes. Vast-primary with Runpod as emergency-only matches the owner's 13-Oct direction. All applicable checks green.


---
_Comment/review ids in this file: 5498645768_

