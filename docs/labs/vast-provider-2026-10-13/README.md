# Vast.ai provider for `uor-pod` (default for new pods)

References #2028. Claude lab, session `claude/vast-backend-2026-10-13`, 13 October
2026. Owner direction recorded as [D26](../../integration/DECISIONS.md#d26---vastai-is-the-default-provider-for-new-pods).
Infrastructure only: no model training, evaluation or headline change.

## Why

GPU-hours were about 98 % of Runpod spend (week of 5–12 October: $426 of $435).
On 13 October Vast.ai's public offers API listed verified datacenter RTX 5090s from
$0.469/h and RTX 4090s from $0.40/h, against Runpod Secure Cloud's $1.19/h and
$0.89/h for the same cards. Prices move daily; the cheapest offer changes hour to hour.

## What changed

`scripts/pod/uor-pod` gained a Vast.ai provider and uses it for new pods; every
other command works on either kind of pod. Details are in
[compute.md § Vast.ai](../compute.md#vastai-default-provider-13-october-2026). In short:
Vast instances are listed in the same shape as Runpod pods, so leases, caps,
status, run, ssh and reap are shared code; `up` rents the cheapest verified
datacenter offer on the usual GPU ladder; `down` archives results to the laptop
before destroying; the pod-side reaper destroys an idle unleased Vast instance
through Vast's API with the instance's own scoped key.

Safety properties, each pinned by a test that fails when the property is removed:

| Property | Tests failing with it set aside |
| --- | --- |
| Destroy answers the CLI's `[y/N]` prompt and checks the instance is gone | 3 (incl. "no instance is left billing") |
| A failed provider listing never archives that provider's live leases | 2 |
| The instance key `vastai create` returns never reaches output, ledger, state or a command line | 1 |
| A missing result directory is not mistaken for an SSH failure at `down` | 6 |
| Destroy is confirmed only by a well-formed list without the instance (review 1) | 3 |
| A listing that is not a JSON array is a failed listing (review 2) | 3 |
| `status` judges "gone" from the listing it was built from, not a second call (review 3) | 2 |
| A missing provider CLI stops `up` (its pods would be uncounted) (review 4) | 1 |
| A rental whose reply does not parse is found by label, never repeated (review 5) | 3 |
| `prune-stopped` never destroys a stopped Vast pod's only copy (review 6) | 1 |
| Offers are priced with the 150 GB disk (review 7) | 1 |

`scripts/pod/tests/uor-pod-dryrun.sh`: **312 passed, 0 failed** (246 existing
cases on the Runpod path, 66 new cases against a fake `vastai`). Independent
review, round 1: 8 findings; 1–7 fixed with the cases above, 8 (may the
instance-scoped key destroy the instance?) verified live. Round 2: an empty reply
(exit 0) still passed as a listing in two places, and the HF token copy belonged
off the shared volume; fixed, with cases that fail when each fix is removed. `bash -n`
and `shellcheck -S warning` are clean apart from one warning already on main.

## Measured

| Run | Host | Price | Empty pod → CUDA build + parity | Result |
| --- | --- | --- | --- | --- |
| Manual trial (01:07Z) | 1 × RTX 5090, verified datacenter, South Korea | $0.469/h | 137 s (Rust 7 s, build 93 s, parity 26 s) | parity 38/38 PASS |
| `uor-pod up` live smoke (01:34Z) | 1 × RTX 5090, verified datacenter, South Korea | $0.536/h | 142 s; create → SSH ≈ 2.5 min | parity 38/38 PASS, reaper `api-ok`, `nvidia-smi -L` job ran, `down` archived 3 logs and destroyed |
| Runpod reference (ledger) | RTX 5090, Secure Cloud | $1.19/h | 164–191 s (build 112–129 s) | — |

| Reaper self-destroy (02:11Z) | 1 × RTX 5090, South Korea | $0.536/h | — | lease released, idle limit 0: the reaper destroyed its own instance in seconds (review finding 8) |

The four Vast pods of 13 October (trial, smoke, two diagnostic pods) cost $0.195 of
the $100 prepaid credit in all. The live smoke
found one defect the fake had hidden (a missing `uor-r4/evals` made `down` report an
SSH failure and refuse); it is fixed, and the fake `ssh` now runs the archive
commands for real so the tests see real exit statuses.

## Limits

- One host, two runs. Vast hosts vary; the reliability and datacenter filters and
  the next-offer retry are the mitigation, not a guarantee.
- Training throughput (tokens/s) on a Vast host has not been compared with Runpod.
  The card is the same, but the host CPU and memory differ; the first real run on
  Vast reports it beside a Runpod number.
- HF fetch: the first smoke pod had a stale laptop token (401). After the owner's
  new `hf auth login` (the API answers 200 for the store) the pods still got 401,
  because the image sets `HF_HOME=/workspace/.cache/huggingface/` and the token
  was only written to `/root/.cache/huggingface/`. With the token visible to `hf`
  the fetch worked on the pod. The bootstrap now sets `HF_TOKEN_PATH` to the
  `/root` copy, so the token never lands on `/workspace` (on Runpod, a shared
  volume that outlives the pod). Runpod non-canonical pods had the same gap. The
  bootstrap still treats a failed fetch as a warning.
- The owner's new token has the `write` role on the whole account and is copied
  to rented hosts; a fine-grained read token for `caseyallard/uor-r4-store` would
  limit what a compromised host could do.
- On `main` at `7567cf725` and `19500bf22` the bootstrap's CUDA parity build fails
  to compile (`tests/cuda_stack_ops_parity.rs:2445`: `Gate` is not imported after
  the OP1 gate field), so every pod bootstrap at those commits reports parity
  FAIL on either provider. That test builds only with the CUDA feature, so laptop
  and CI checks do not see it. It is outside this change and reported to the
  operator line.
- The Runpod network volume (`lmd1pfah3y`) is untouched and still bills a few
  dollars a week; retiring it is a later owner decision.
