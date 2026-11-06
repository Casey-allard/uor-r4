# REPORT: cpu-training-pilot, deepseek-227

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[branch endpoint sheet](../../../../harness/prep/cpu-training-pilot-content-branch-endpoint-2026-11-06.md).
Base: `7b1e888ef`.

## Status

Complete as **REJECT**. Both artifacts and repeats ran.

## What did not work

Top-eight retrieved branches doubled endpoint stops from 23 to 48, but still
did not solve termination. On the two artifacts, only **175/208** of 437
reachable replies completed by BOS or endpoint and **43/68** stopped
prematurely. Exact and body counts stayed exactly at the boundary values
(**14** and **936/933**). The true branch guard also tied the shuffled branch
guard at 14 exact replies, failing the required comparison.

## What changed

Added exact top-K unigram-Jaccard retrieval and a branch endpoint. The boundary
is deferred while any retrieved TRAIN reply extends the complete generated
prefix; exact branch completion stops the reply. Retrieved content is never
injected.

## Measurements

| Arm | Exact | Body positions | Premature BOS | Endpoint stops |
|---|---:|---:|---:|---:|
| top-8 branches | 14 | 936 / 933 | 43 / 68 | 48 / 48 |
| top-1 endpoint | 14 | 936 / 933 | 46 / 71 | 23 / 23 |
| original boundary | 14 | 936 / 933 | 55 / 80 | 0 / 0 |
| shuffled top-8 | 14 | 936 / 933 | 52 / 77 | 8 / 8 |
| full retrieval | 41 | 864 / 864 | 0 / 0 | 0 / 0 |

Top-8 changed **48** complete outcomes from boundary and **40** from shuffled.
Repeats were byte-identical. Panel pins: tokens
`f5a6714a579cd8f4666b5bca5c24ec4d5e35e930b9057c6fe5b086a3d4a4638e`,
mask `83cabdcfd86d0acf31d8172eb020ed50ca13eabf9571ca665fc6a3cf4f062ea9`,
manifest `a5214c2f8430a9a3c22e4b468e40d6594247dbe5ac1e9fc30025e31cba9dcbf1`.
Raw manifest: `17869456a2d79691a3e7d2b37c35f37e276c4ec5bad9bd1ca5b886c241618d31`.

## Interpretation

More retrieved branches defer more false BOS stops but do not make the
retrieved neighborhood a sufficient termination signal. This closes fixed
K=8 full-prefix branch coverage. The next stop mechanism must use generated
branch evidence itself, not membership in a prompt-level candidate set.

## Checks run at the head

- focused branch test: 1 passed; driver test: 1 passed;
- full CPU library: 407 passed, 4 expected ignores;
- both artifacts repeated byte-identically;
- independent delegation was unavailable; review is local exact-head.

Compact evidence: [result.json](evidence/result.json).

## Skills used

No system-provided Codex skill applied.

## Still open

- a generated-state branch signal that survives arbitrary content divergence;
- native termination without a retrieved candidate set;
- Claude's independent end-to-end serving track.

## Plain summary

Top-eight branch coverage doubled endpoint stops but still failed completion,
premature-stop and shuffled-control bars. The fixed K=8 endpoint set is rejected.
