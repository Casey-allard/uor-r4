# REPORT: cpu-training-pilot, deepseek-151

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `1b501c2f1`.

## Status

Decision: **REJECT**. The fixed depth-and-age precision router failed the
pre-registered comparison against longest-first on the new seed-2602 panel. It
beat `k2>k1` and random depth, but all three comparisons were required. This
closes only this 36-cell trained router; it does not revoke deepseek-149's
longest-first exact-copy candidate or establish a model, language or serving
result.

## What did not work

- The router was right on **913 / 1,655** copyable test positions, eight below
  longest-first at **921 / 1,655**.
- Its longest comparison had only 1 router-only decision against 9
  longest-only decisions; the registered threshold was 6, and the router sum
  was not greater.
- Training was also diagnostic-only: the router reached **832 / 1,485** versus
  **838 / 1,485** for longest-first on seed 2601.
- The failure is specific to empirical precision over four depth bins and nine
  age bins. It does not show that deeper keys, a different backoff, or trained
  vector reads are ineffective.

## What changed

Added a discrete copy-key router over the deepseek-149 exact 192-token
BOS/window scan. Each available depth predicts from its latest exact suffix
candidate. Fitting records candidate correctness by depth and fixed age uppers
`1,2,4,8,16,32,64,128,192`; evaluation chooses the learned
`right / matches` precision, ties toward the deeper key, and falls back to
longest-first when no training cell is available.

The router was fitted only on new `cell=train,train` seed 2601. The decision
panel was new `cell=dev,dev` seed 2602. Public M1 DEV remained instrument-only.
Random novelty counts changed emitted tokens on copyable positions, not
candidate-age changes. Malformed table shape/order/bounds and cross-product
overflow are rejected, and the paired threshold uses exact integer arithmetic
through `u64::MAX`.

## Measurements

Public M1 DEV reproduced deepseek-149 exactly: **8,243** copyable positions,
depth 1 **6,306 / 3,859**, depth 2 **4,383 / 3,506**, and `k2>k1`
**4,388**.

Training panel, seed 2601 (`train,train`):

| Arm | Right / copyable |
|---|---:|
| trained depth/age router | 832 / 1,485 |
| longest-first `4->1` | **838 / 1,485** |
| `k2>k1` | 817 / 1,485 |
| random depth | 792 / 1,485 |

Decision panel, seed 2602 (`dev,dev`):

| Arm | Right / copyable |
|---|---:|
| trained depth/age router | 913 / 1,655 |
| longest-first `4->1` | **921 / 1,655** |
| `k2>k1` | 912 / 1,655 |
| random depth | 876 / 1,655 |

| Comparison | Candidate / control | Candidate-only / control-only | Threshold | Pass |
|---|---:|---:|---:|---|
| router vs longest | 913 / 921 | 1 / 9 | 6 | no |
| router vs `k2>k1` | 913 / 912 | 7 / 6 | 5 | yes |
| router vs random | 913 / 876 | 52 / 15 | 8 | yes |

Both panels exceeded 200 eligible positions: **2,487** training and **2,704**
test. The random control changed **79** copyable longest-first decisions. The
fresh-process router repeat was byte-identical. All 36 table cells were valid;
27 contained training matches.

Resources:

- release build **18.76 s**, peak **1,191,812 KiB**;
- generator build **9.86 s**, peak **1,260,504 KiB**;
- panel generation **0.40 s + 0.47 s**, peaks **8,024 / 8,084 KiB**;
- public parity **0.02 s**, both router processes **0.01 s each**;
- measured component total **29.53 s**;
- fresh target storage **1,498,741,605 bytes**, below the 16 GiB cap;
- retained published evidence **34,846 bytes**, below 32 MiB;
- final free storage **471,399,149,568 bytes**, above `30 GiB + 128 MiB`;
- router binary **1,257,760 bytes**, SHA-256
  `71ce396c6760729cbe57b330b763b031b8904afc1c269221710dbfd30becc2da`;
- admitted cumulative projection: **72,347.677487699 s**.

The raw sealed manifest `RAW-SHA256SUMS` has SHA-256
`8e9f04a86fbbe8c0846a6bd4f24f94d176eb43ade80744be4bd605ccd9894d27`.
Generated corpora are omitted from Git; the committed panel manifests preserve
their token/mask hashes and seeds, and every committed evidence file verifies
with `PUBLISHED-SHA256SUMS`.

## Checks run at the head

- Fresh FIT and independent GO preceded the new panel draws.
- Public DEV parity, all input hashes, both 200-row floors, random
  non-vacuity, the two new seeds/cells, target/free-space limits, RSS limits
  and byte-identical fresh-process repeat passed.
- **275** CPU library tests passed with 4 expected ignores; **14** focused
  copy-key tests and the router binary test passed; strict scoped Clippy,
  rustfmt, recipe `bash -n`, `git diff --check` and evidence manifests passed.
- No float-training, integer-serving, CUDA or other Claude-owned source
  changed.
- Exact-head review approved `0cba12e96` in
  [#issuecomment-6648446089](https://github.com/UOR-Foundation/uor-r4/pull/3693#issuecomment-6648446089).
  PR [#3693](https://github.com/UOR-Foundation/uor-r4/pull/3693) merged as
  `292a572133774a228818e4eaf2dc0ac832ea26cd`.
- Fresh merge-main verification passed **275** CPU library tests with 4
  expected ignores, the router binary test, all published evidence manifests
  and the recorded rejection fields.

## Size

Against base `1b501c2f1`, the branch changes **3** code files (**+833 / -24**),
**9** documentation files (**+541 / -0**), and **21** evidence/resource files
(**+1,195 / -0**): **33 files, +2,569 / -24**. No standalone test file changed
because the focused tests remain in `copy_key_depth.rs`. The ORDER gave no line
estimate, so a ratio to estimate is not defined. No generated corpus or binary
is tracked.

## Skills used

The project-provided `uor-research-architect` skill was used for the
research and delivery reviews. Project-specific FIT, independent GO and
exact-head review procedures were followed.

## Still open

- Keep longest-first as deepseek-149's selected exact-copy construction.
- Test a different routing feature or explicit longest-first backoff before
  another trained copy read.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

Learning an age-conditioned depth preference did not beat the simpler
longest-first exact copy rule. On the untouched decision panel it made eight
more mistakes than longest-first despite beating the older two-token and
random controls. Reject this router and keep the longest-first candidate.
