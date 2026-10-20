# #2261 — uor-pod: Vast.ai provider, the default for new pods (D26) (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T01:48:46Z
- **Updated:** 2026-10-13T02:28:39Z
- **Closed:** 2026-10-13T02:25:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2261
- **Merged:** 2026-10-13T02:25:10Z
- **Merge commit:** f8cff324a736964794a20c3878465c0852cdf59d
- **Base / Head:** main / UOR-Foundation:claude/vast-backend-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: infrastructure (compute provider) · no count (owner-directed tooling, no model experiment) · headline unchanged: M1 v5 memory 20/40, reply 37/232. Training and M2 rental holds unchanged.

## What and why

Owner direction 13 October 2026, recorded as **D26**: new pods go to Vast.ai, which listed verified-datacenter RTX 5090s at $0.469/h against Runpod Secure's $1.19/h; Runpod stays available (`up --provider runpod` / `UOR_POD_PROVIDER=runpod`).

- `scripts/pod/uor-pod`: Vast instances are listed in the same shape as Runpod pods (id `vast-<contract>`), so leases, status, run, ssh, reap and the caps are shared code. Caps (≤ 4 pods, ≤ $8/h) count both providers; `up` refuses if either listing fails; a failed listing never archives live leases.
- `up` on Vast rents the cheapest verified datacenter offer (reliability > 0.98, CUDA 12.8+, 150 GB, ≥ 200 Mbit/s) on the usual ladder, tries the next offer when one is taken, bootstraps `--off-volume --non-canonical` (HF store) and skips the hot-set relay. The instance key in the create response is discarded.
- `down` archives `/workspace/uor-r4/{jobs,pods,runs,evals}` to the laptop before destroying (refused above 5 GB); destroy answers the CLI's `[y/N]` prompt and confirms the instance is gone.
- `scripts/pod/uor-reaper.sh`: on Vast it destroys an idle unleased instance through Vast's REST API with the instance-scoped `CONTAINER_API_KEY` (stdin only; response bodies are never echoed).
- Docs: D26, AGENTS GPU-pod rules, policy, `compute.md § Vast.ai`, README compute line, [dated record](docs/labs/vast-provider-2026-10-13/README.md), current-state entry.

## Checks at head `04550537e2d15dac92b5519931a56cf5fad9ad5b`

- `bash scripts/pod/tests/uor-pod-dryrun.sh` → **passed 292, failed 0** (246 existing cases on the Runpod path, 46 new Vast cases with a fake `vastai`).
- Negative checks (each property removed on a scratch copy): unanswered destroy prompt → 3 failures; archiving leases on a failed listing → 2; echoing the create response (key) → 1; missing-result-dir exit status → 6.
- `bash -n` clean; `shellcheck -S warning`: 1 warning, pre-existing on main. `check_no_softmax.py` 1649/1649; `check_claim_wording.py` pass; `harness check` rc 0 (warnings only, none from this change).
- **Live** `uor-pod up → run → down` on a Vast 5090 (South Korea, $0.536/h): create→SSH ≈ 2.5 min, bootstrap 142 s, parity 38/38 PASS, reaper `api-ok`, `nvidia-smi -L` job ran, `down` archived 3 logs and destroyed (0 instances left). The live run found the missing-dir bug, fixed in this PR. Vast credit used for trial + smoke ≈ $0.09.

## Limits

One host, two runs; tokens/s on Vast vs Runpod not yet compared. The smoke pod's HF fetch got 401 from a stale laptop token; the owner has since re-logged in and a laptop fetch from the store succeeds. The Runpod volume is untouched (retiring it is a later owner step).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6136832385 — 2026-10-13T02:24:19Z — Casey-allard

**Exact-head receipt — `b4a57684de7389d7288cd600d7e6f01eb5de9e1f`** (rebased onto `8a944609a`, #2263). Self-review per the owner's merge rule, after two independent review rounds.

- `bash scripts/pod/tests/uor-pod-dryrun.sh` → **passed 312, failed 0** (246 existing Runpod-path cases + 66 Vast cases).
- `bash -n` uor-pod / uor-reaper.sh / uor-pod-bootstrap.sh: clean. `shellcheck -S warning`: 1 warning, pre-existing on main.
- `check_no_softmax.py` 1649/1649 · `check_no_matmul.py` rc 0, 95/95 · `check_claim_wording.py` pass · `harness check` rc 0.
- Diff vs main: 12 files, +740 / −69 (scripts/pod, D26, AGENTS, policy, compute.md, README, current-state, dated record).

**Review.** Round 1 (8 findings): 1–7 fixed, each pinned by cases that fail with the fix set aside (3/3/2/1/3/1/1); 8 verified live — the pod-side reaper destroyed its own Vast instance with the instance-scoped key. Round 2 (empty listing accepted in two places; HF token copy onto the shared volume): fixed, pinned (1/2/1).

**Live** (Vast, 13 Oct): trial + `up → run → down` smoke + two diagnostic pods, $0.195 total; parity 38/38 at `8773ac8dd`; `down` archived and destroyed; 0 instances left.

**Found outside this PR, for the operator line (Kimi):** on `main` since #2260/#2262, the CUDA parity test does not compile — `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs:2445` uses `Gate::Swiglu` without importing `Gate` — so every pod bootstrap on either provider reports parity FAIL. It only builds with `--features cuda`, so laptop/CI checks miss it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)



---
_Comment/review ids in this file: 6136832385_

