# REPORT: m1-address-key-full-vector, claude-110

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`m1-address-key-full-vector-2026-10-25.md`](../../../../harness/prep/m1-address-key-full-vector-2026-10-25.md).
Closes #2834.

## What did not work, first

- **Top-1 fell at checkpoint 0.**
  - Both treatments score 118 full-DEV top-1 against A8-Outer's 133. The writer now admits more target cells, but more of its picks among them are wrong.
  - Top-1 is not the Stage 1 bar, which is starting reach and capacity, measured before training. Top-8 rose: 513 for A8, against 1,126 for FV8-Env and 1,179 for FV8-TwoScale.
  - The drop is reported, not explained. No training ran.
- **The sheet's prior primary is the weaker treatment.**
  - FV8-TwoScale admits 1,802 of the 3,847 P_succ rows, and FV8-Env admits 1,980.
  - Two-scale's within-cell sum breaks envelope ties in a way that costs reach. Both pass.
- **The pre-flight took three pod runs. The first two stopped as instrument faults, exactly as the stop rule says, and no treatment number from them was graded.**
  1. **Run 1 (17:27Z).** G-Q's device-against-oracle comparison passed for all three metrics. Then its separation check found the envelope and two-scale metrics admitting identical cells. The cause was the fixture: the seeded device table gives same-cell tokens one vector, so the two orders cannot differ. The fix is a patterned table with 4 distinct tokens in every cell.
  2. **Run 2 (17:34Z).** It passed G-Q, G-Exact and G-Occ, then FV8-TwoScale's `eval-corpus` was refused with status 12. The evaluation aggregator's row contract bounds the address hinge at 72, on the outer key's scale, and two-scale distances run about 256 times larger.
     - This was a defect in this unit's kernel change. G-Q had not exercised the evaluation path.
     - The fix: the two-scale hinge keeps the envelope's scale, and admission and rank keep the two-scale order. G-Q now runs the evaluation path for every metric.
  - Both faulted outputs are archived (`fault1/`, `fault2/`).
- **The sheet changed after its FIT, before the scored run.** Its "Changed after the FIT" list names each change.
  - By the reviews:
    - re-pins of the source commit;
    - the grader's config binding and G-Q input;
    - G-Q's fixture size;
    - the cost.
  - By the two fault runs:
    - the patterned fixture;
    - the two-scale hinge's scale.

  None changes an arm, a bar or a data pin.
- **The writer's stack frame grows on every path.** The `U env[256]` and `W sum[256]` arrays add about 3 KiB to `writer`'s frame, outer default included, even though outer mode never touches them.
  - The position unit (#2843) had just cut the frame by 2,896 B, to 22,080 B on forward.
  - This unit did not measure the new frame with `ptxas`, or the default path's tokens/s. The A8-Outer eval took 30 s, the same as the K8 treatments (30–31 s). FV16 took 37 s.
  - The cost is device local-memory reservation, not speed. A separate kernel for the full-vector metrics would remove it.
- **The pod landed on a 2×5090 offer.** Only GPU 0 ran, for about 27 minutes in all. At the 2×5090 rate the D67 review quoted (about $1.98/h), that is roughly $0.90, inside the $1 cap. I did not read the billed amount before the pod was deleted.
- **What is not established here:**
  - Training under a full-vector metric. It is not registered: every update capability refuses a non-outer metric.
  - Serving: the D11 exporter refuses one too.

## What changed

**Code** (`crates/uor-r4-discrete`):
- **`discrete.cu`:** the writer's address metric word `s[15]`: `outer_l1_v1` (default, unchanged), `cell_envelope_l1_v1` and `full_vector_cell_l1_v1`.
  - The per-cell envelope and sum are built in the writer's existing token pass, only when the metric is not the default.
  - Status 13 is returned for an unknown word.
- **The host field** `address_metric`, on `DriverConfig`, `Snapshot` and the device binding. It is omitted when it is the default, so existing configs, snapshots and lineages keep their bytes. The attention scope names a non-outer metric.
- **The zero-update gate.** `CorpusUpdateAdmission::guarded` is the only constructor of an update capability, and it refuses a non-outer metric. A source scan pins that no other literal exists. The D11 exporter refuses it too.
- **`address_key_grade.rs` and the `address-key-grade` bin.** The grader:
  - joins rows by ordinal;
  - checks that each summary agrees with its rows;
  - binds each arm to its pinned config sha;
  - requires G-Q's log;
  - decides the paired floor exactly in integers.
- **Tests:**
  - the CUDA control `src/device/address_metric_device_tests.rs`;
  - `tests/address_metric_configs.rs`;
  - the DEV-B3 lock;
  - the scope strings;
  - the exporter refusal;
  - the grader's decision table.
- The driver-source pin is updated.

**Docs:**
- the final sheet, which supersedes antigravity-37's draft (FIT in round 2);
- six arm configs;
- the job;
- `grade.json`;
- the lab README's GPU section;
- the item set `done`.

**The notebook row is not in this PR.** The installed `uor-harness` loop guard refuses the item's own notebook page (`docs/research/notebook/m1-discrete-attention.md`). That is the gap #3189 closed on `main`, but the installed binary predates the fix. The row's text is in the lab README's GPU section. It goes into the notebook in the carrier-credit results unit, whose scope holds that page since claude-108 (#3227).

The tables (`full-vector` `2c886736…`, `full-vector-shuffled-20261019` `5e5abf33…`, and A8's `6600bdb1…` rebuilt byte for byte) were built on the host from TRAIN and are sealed in the HF store under `results/claude/address-key-2026-10-25/tables/`.

## Measurements

The run was on one RTX 5090 (`vast-57698566`, GPU 0), at source `77123dd995be44d5b0a5f482c8b82e270dd9aa42`, with job `docs/labs/m1-address-key-full-vector/address-key-job.sh` (sha256 `73d45cb5…`). The outputs are in the HF store under `results/claude/address-key-2026-10-25/run/`, 30 files, and the SHA256SUMS were verified after download. Each zero-update `eval-corpus` took 30–37 s.

**Instrument gates, all passed before any treatment eval ran:**
- **G-Q:** `device_admission_matches_the_host_oracles_for_each_metric` passed.
- **G-Exact:** A8-Outer reproduced 4,244 of 14,287 full-DEV admitted rows, 1,351 of 3,847 P_succ, and top-1 133, exactly.
- **G-Occ:** both pinned tables gave 4,096 distinct vectors, 256 occupied cells, 0 alias tokens, 0 non-representative rows, and an alias ceiling of 14,287.

**Starting reach on the 3,847 P_succ rows.** The paired floor is F = max(24, 2√(b+c)).

| Arm | P_succ admitted | Against A8-Outer, b / c | Against its same-metric shuffled arm, b / c | Full-DEV admitted | Top-1 | Top-8 |
|---|---:|---:|---:|---:|---:|---:|
| A8-Outer | 1,351 | — | — | 4,244 | 133 | 513 |
| **FV8-Env** | **1,980** | **730 / 101** | **1,726 / 391** (FVS8-Env admits 645) | 5,709 | 118 | 1,126 |
| FV8-TwoScale | 1,802 | 562 / 111 | 1,555 / 416 (FVS8-TwoScale admits 663) | 5,666 | 118 | 1,179 |
| FV16-TwoScale (bracket only) | — | — | — | 7,207 (A16: 6,023) | 118 | 1,185 |

- **Outcome: PASS.** Both treatments have reach, geometry and capacity. The grade is `address-key-grade` → `docs/labs/m1-address-key-full-vector/grade.json`.
  - The shuffled arms keep the metric and change only the placement, and they admit about half of A8. So the predecessor-pulled placement matters under a full-vector metric, beyond packing balance.
  - No arm reads the full-vector table with the outer key, so the metric's own share of the gain over A8 is not separated here.
- **Stage 1 bar:**
  - "a starting reach above the address-cell model's 1,351 of 3,847": met, 1,980 (FV8-Env);
  - "winnable positions clear 1,941": met, min(14,287, 5,709) = 5,709;
  - within-cell tie mass: 0.

## Checks run at the head

- The discrete host suite at `826ebc9ed` (the code is unchanged since): 369 passed, 0 failed. Stub-nvcc `--features cuda --all-targets` and `cargo fmt --check` are clean for this crate.
- **Negative gates shown failing:**
  - `zero_update_only` set aside;
  - a bypassing capability literal (the source scan);
  - the D11 refusal set aside.
- **Reviews:**
  - **FIT:** round 1 NOT FIT, round 2 FIT (#2834 `6400663507`).
  - **D67 pre-run review:** PASS (#2834 `6400724408`). The first reviewer ran out of context, and a second finished checks 1, 2 and 5.
  - **Code reviews:** round 1 CHANGES, then fixed. Round 2: the grader slice was APPROVE; the kernel slice was CHANGES for rustfmt, then fixed. The final review of every code change since then (the hinge, the patterned fixture and the grader's refusals) was APPROVE, with no Required findings.
  - **The claims check of this REPORT and the lab README** returned CHANGES. Every number and the PASS label checked out (about 45 claims). It flagged five wordings, all now fixed:
    - the post-FIT list omitted the fault-run changes;
    - a fixture line overclaimed;
    - "the gain is the geometry" is now scoped to the placement;
    - a timing nit;
    - the shuffled arms' top-8 was missing.

## Skills used

- holding-the-goal: the unit stopped at each instrument fault and repaired the instrument. It did not read any treatment number from a faulted run.

## Still open

- **`m1-wider-code-carrier` (#2847)** and **Stage T's decision review (#2836)** take this result as input. Stage T now has its dependency met.
- **Training under the envelope metric** needs its own sheet. The hinge and the update gate are both calibrated for zero-update use only.
- **The top-1 drop at checkpoint 0** (133 → 118) is the first thing a training sheet should explain.
- **The metric's own share of the gain.** An arm with the full-vector table under the outer key would separate the metric's contribution from the placement's.
- **Review advisories not taken:**
  - Bind each `eval-corpus` report to its arm the way `dev-b3` reports are bound.
  - Pin G-Q's log by its sha256.
  - Under two-scale, the hinge does not see the sum term. That matters only once a learning arm uses two-scale.
  - Measure the writer's stack frame with `ptxas`.

## Plain summary

The model chooses which small set of word groups to look at before it picks a word. Its old way of choosing could tell groups apart only coarsely. A finer way, which compares whole word vectors, now gets the right group into the shortlist for 1,980 of 3,847 test cases instead of 1,351, before any training. It slightly lowers the first-guess score for now. Two setup mistakes in the test were found and fixed before the real run.
