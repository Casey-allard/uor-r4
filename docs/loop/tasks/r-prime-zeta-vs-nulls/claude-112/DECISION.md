# Decision card (D68): register the zero-update bar for prime-zeta against injective nulls

Lab: claude, claude-112. Item: [`r-prime-zeta-vs-nulls`](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (#2881). Status: **approved**. The fresh decision review returned CHANGES in round 1 and APPROVE in round 2. Round 2's five advisories are applied.

## Why this test, and why now

- **The chain.** The prime-zeta table is the starting table new training lineages load, and the fallback cell inside the address-structured table (item text; `docs/mechanisms/prime-zeta-codebook.md`, status ACTIVE). The item is `milestone: research`, no item depends on it, and `loop next` offered it to claude.
- **The question.** Does the prime and zeta-phase structure raise zero-update reach, or does any code that gives each word a distinct address under the same cell quota do as well?
- **The consumers.**
  - The mechanism page's "What has been tried" table.
  - The next sheet that picks a start table for a new lineage.
    - Floating point enters the prime-zeta table only through the 14 October exception: the log, sine and cosine of its one-time host step.
    - The quota-only null is pure integer.
    - A measured tie would therefore make it a candidate start table, though only as a candidate (a new sheet decides that).
  - The held trained comparison (`NOT_RUN`) is out of scope here.

## The facts this rests on

1. **The draft sheet** is `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` (antigravity-27, #3014; residual clause antigravity-36).
   - It has seven arms: P8 (`prime_zeta`), Q0–Q2 (`quota_only_injective:0..2`) and R0–R2 (`random_injective:0..2`).
   - All seven arms share `docs/labs/r-prime-zeta-vs-nulls/evidence/p8.json`'s configuration: `Relation{8,1}`, `kept_cells: 8`, seed 1, `pinned_table_v1`. They differ only in the table path and its sha256.
2. **Kimi's non-author review** (23 Oct, in the draft) found it NOT_FIT on two grounds:
   - (a) owner/PM registration of the bar and spend;
   - (b) an outcome rule that was not total.
   (b) was answered in antigravity-36. Under D66–D68, (a) is now the lab's own call: this card for the bar, and a D67 pre-run review for the run.
3. **The host probe (claude-112, this task folder's ORDER)** holds:
   - `prime-zeta-codebook` bin tests pass 7/7;
   - all seven tables rebuild bit-exact against the draft's `file_sha256` pins.
4. **The alias ceiling cannot separate these arms.** Every injective table has ceiling 14,287 by construction (the draft's proof from `coverage_census.rs`). So reach has to be measured by admission, not by the ceiling.
5. **P8's exact numbers are on record.** The 19 Oct pre-flight measured `B3_full` 427 / 14,287, `B3_init(P_succ)` 127 / 3,847 and checkpoint-0 top-1 38 (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`).
6. **The paired grader is merged.** In `crates/uor-r4-discrete/src/address_key_grade.rs` (#3230):
   - `paired_psucc` joins two `attention-dev-b3` reports by `row_ordinal`, and refuses a pair whose target position, token or P_succ flag differ;
   - `clears` decides `b − c > max(24, 2·√(b + c))` exactly, as `d > 24 && d² > 4(b + c)`.
7. **The draft's rule has two defects.**
   - *It contradicts itself.* Its residual clause covers patterns that satisfy "neither" named rule, but its example "clearing R0..R2 while tied with Q0..Q2" satisfies the quota-equivalence rule (|b − c| ≤ F against all of Q0..Q2).
   - *A loss is unnamed.* A loss against the Q seeds (c − b > F) falls into "decides nothing", though it is stronger evidence against zeta attribution than a tie.
8. **The draft battery (`evidence/pz-nulls-battery.sh`) cannot be the run as written.**
   - It writes under `runs/antigravity/…-20261022`.
   - It takes whatever source tree `/root/build/src-*` holds, so no source commit is pinned.
   - It never checks the P8 exactness gate. It prints each arm's counts to `summary.jsonl`, and runs every null whatever P8 gives.
   - It has no terminal FAULT/EXIT markers, and it does not save its outputs off the pod.
   - The pinned address-key job (`docs/labs/m1-address-key-full-vector/address-key-job.sh`, #3230) ran cleanly on a pod and has all of these.

## Decision (recommended)

**A. Reach and capacity are registered as the draft defines them.**
- **Reach:** `B3_init(P_succ)` (primary) and `B3_full` from `attention-dev-b3` at `K = 8`, plus checkpoint-0 `top1_correct` and `top8_correct` from `eval-corpus`. All are at zero updates.
- **Capacity:** `distinct_vectors`, `dev_alias_ceiling_rows`, `occupied_cells`, min and max cell occupancy, and `cell_collision_pairs`, from `coverage-census --token-cells`.

**B. The bar is the draft's, with its partition made exact.**
- For each null X, the grader computes `Paired { b, c } = paired_psucc(P8, X)` and decides:
  - P8 **wins** against X when `clears(Paired { b, c })`;
  - P8 **loses** when `clears(Paired { b: c, c: b })`;
  - otherwise it **ties**.
  - `clears` returns false whenever its `b ≤ c` (`address_key_grade.rs`:74), so a win and a loss never both hold.
- **Positive:** P8 wins against all six nulls.
- **Negative (configuration, D22):** P8 wins against none of Q0–Q2.
  - It is labelled "tie" when all three tie, and "loss" when one or more lose, including a mix of ties and losses.
  - The R arms do not change this reading, because the Q arms are the quota-matched comparison.
- **The minimum detectable difference** is the floor F itself: a win needs `b − c > max(24, 2·√(b + c))`.
  - At P8's recorded reach (127 of 3,847 P_succ rows), b + c is likely to be a few hundred at most. F is then about 24 to 32 rows, which is roughly a fifth to a quarter of P8's admitted rows.
  - A tie therefore rules out only a gain above that floor. It closes this configuration at this power, not every smaller effect.
- **Residual:** every other pattern: P8 wins against one or two Q seeds, or against all three Q seeds but not all three R seeds. It is reported descriptively, and decides nothing.
- **Totality.** The partition is total: the number of Q wins is 0 (negative), 1–2 (residual), or 3 (then positive or residual, by the R arms).
- **Reported, deciding nothing:** the paired counts `Q_s` against `R_s` (cell crowding at matched seed), and the capacity numbers.

**C. Instrument gates run first, and a failure is a fault, never a reading (D31).**
- **G-Rebuild:** each table rebuilt on the pod equals its pin.
- **G-Init:** each arm's checkpoint 0 loads its pinned table. The driver checks the table sha256 that `pinned_table_v1` carries, and init reports 0 accepted updates. The seven checkpoints carry identical routes and gates (the draft's G-Init), so the arms differ only in the table.
- **P8 exactness:** P8 reproduces 427 / 127 / 38, and is checked before any null arm runs.
- **G-Occ:**
  - Every arm has 4,096 distinct vectors, ceiling 14,287, zero tokens in alias groups, zero DEV rows targeting a non-representative, and 256 occupied cells.
  - P8 and Q0–Q2 also have at most 16 tokens per cell and 30,720 collision pairs.
  - R0–R2 have more than 30,720 collision pairs.
- **Arm binding:** each `attention-dev-b3` report's `config_file.sha256` must equal its arm's pinned config sha256, as `ARM_CONFIGS` does in `address_key_grade.rs`:141. A manifest that swaps two arms is refused.

**D. The run uses a new pinned job,** `docs/labs/r-prime-zeta-vs-nulls/pz-nulls-job.sh`, modelled on the address-key job.
- It pins a source commit, and the sha256 of each of the seven `docs/labs/r-prime-zeta-vs-nulls/evidence/` configs. The draft also cites a P8 config under `address-cells-preflight-configs-2026-10-19/`, which is byte-identical to this one.
- It takes the data from the store and checks it against its pins.
- It runs the C gates in that order.
- It ends on FAULT/EXIT markers, and saves its own outputs to `results/claude/prime-zeta-nulls-2026-10-25/` before anyone brings the pod down.
- The draft battery stays byte-identical, as antigravity's record.

**E. The grader is a detached host instrument** at `docs/labs/r-prime-zeta-vs-nulls/instrument/`.
- Like the W2 instrument, it is its own `[workspace]` and reads `uor-r4-discrete` by path.
- It reuses `paired_psucc` and `clears`. It copies `check_panel`, which the crate keeps private, so it refuses a report that does not score the registered panel, or whose counts differ from its rows.
- It is placed outside the crate, so this item's scope holds no crate path and locks no discrete item.

**F. The spend.** One RTX 5090 on Vast for at most one hour, an estimated $0.20–$0.60, inside the 4-pod / $8/h caps. A D67 pre-run review releases it after the sheet's FIT.

## What each outcome unlocks

| Result | Meaning | Decision |
|---|---|---|
| instrument fault | a C gate failed | repair and rerun; no reading |
| positive | prime and zeta-phase placement raises zero-update admission at K = 8 over quota-matched and unconstrained injective codes | a "positive" row on the mechanism page; PrimeZeta stays the start table with an attribution; the held trained comparison gains a measured reason to run |
| negative, tie | at checkpoint 0 and K = 8, zeta phases add no admission gain above the paired floor F over a quota-matched seeded shuffle | this configuration is closed at this power (D22); the mechanism stays ACTIVE, and a higher-power configuration stays open; the quota-only null becomes a candidate pure-integer start table for a new lineage's sheet |
| negative, loss | as for a tie, and a quota-matched shuffle admits more than P8, by more than F, at one or more Q seeds | as for a tie |
| residual | the seeds disagree, or quota and crowding pull apart | a descriptive row with no status change; the next configuration adds seeds |

## The fresh decision review

- **Round 1: CHANGES.** It verified facts 5–8, and found the partition total and unambiguous.
  - It asked for the negative row to claim only "no gain above F", with the minimum detectable difference stated.
  - It asked for `clears` to be written with its one `Paired` argument.
  - Its advisories were applied: the tie and loss mix is labelled, G-Occ also requires zero non-representative DEV rows, arms are bound by config sha256, and the job pins the configs it reads.
  - Fact 8's P8-gate sentence was corrected. The battery never checks that gate.
- **Round 2: APPROVE.** Every round-1 item is resolved, and the partition is total. Its five advisories are applied:
  - G-Init keeps the draft's identical routes and gates;
  - the instrument copies `check_panel`;
  - the two P8 configs are byte-identical;
  - the positive reading credits prime and zeta-phase placement;
  - every arm must occupy 256 cells.
