# Prep sheet: R\*, the sealed reference of bar Q, second sheet (written-state-count-model, claude-422, 17 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [claude-422](../../loop/tasks/written-state-count-model/claude-422/ORDER.md). Code: branch `claude/kn-sealed-422` at `c63510aa2`. Release cited at dispatch: D86 (§3.10), wick is the owner's GPU.

TEST FITNESS: FIT - round 1 (fresh-context non-author, at `2685f0f03`) found no blocker. Its two should-fix findings and four notes are folded in below:
  - (1) The 2^64 branch of crash 2's message is ruled out by a bound on the mass (see "The cause of crash 2").
  - (2) The memory headroom cites the phases measured on DEV-tune: counting, and scoring with the host check. A sampler logs the sealed run's resident memory.
  - (Notes) The 4,096-unit bound states its premise, the 12-bit vocabulary. `masses_at_floor` is described as an upper bound. The oracle's floor is not exercised on an underflow. The reading map points an overflow or any other crash to the Stop rule.

This sheet follows [the first sheet](r-star-sealed-2026-11-17.md), whose stop rule sends a second sealed crash to a new sheet. Everything the first sheet says holds here unless this sheet changes it.

## The question

The question is the first sheet's: R\*'s BPB on sealed D87 DEV-test, on all targets and on the novel subset, as bar Q's reference. This sheet asks it again, once, after two sealed crashes and two repairs.

### What happened under the first sheet

DEV-tune steps 1–5 passed at budget 2^24: N\* = 10, the coverage, oracle and negative checks, and a `cmp`-identical repeat (job `c422-sweep5`). The sealed run was then started four times. None wrote a JSON, and no mass, bit count or BPB of DEV-test was printed by any of them.
- `c422-sealed1`: refused by the 12 GiB rule (10,390 MiB free). The instrument did not start.
- `c422-sealed2`: the kit's `put` takes a folder, so the script's path was a folder (rc 126). The instrument did not start.
- **Crash 1**, `c422-sealed3`: killed by the 16 GiB container limit (rc 137) at 590 s, while counting order 6, before the "gpu scored" line.
  - Cause: `KnQuery` listed every order's context and pair keys before counting, about 48 bytes per query position per order. That is about 11 GB for DEV-test's 21.35 M positions at N = 10, and half that on DEV-tune.
  - Repair `347f0122e`: each order's filter lists and owns only its own keys. It is a memory-only change, verified by job `c422-fix2`:
    - CUDA tests passed, 12 / 12;
    - DEV-tune at N = 10 is `cmp`-identical to the passed `sweep-10.json`;
    - peak resident memory fell from 15.2 GB to 9.54 GB.
- **Crash 2**, the repeat the first sheet allows, in `c422-fix2`: all eleven orders counted, then the GPU scorer stopped (rc 1). Its message was "target 8880153 has a KN mass of 0 or of 2^64 or more". That is before the "gpu scored" line, so it is a second crash.

**What crash 2 tells about DEV-test.** At least one DEV-test target, at stream position 8,880,153, has a Q48 mass that floors to 0. Nothing else about DEV-test is known. N\* was fixed on DEV-tune before it, so nothing feeds back into the choice of N.

### The cause of crash 2, from the code alone

The scorer's rule is `P_k = floor((a M - D(a)) / A) + floor(gamma P_{k-1} / M)`.
- A target unseen after its context at an order gets only `floor(gamma P_{k-1} / M)` there. `gamma / M` is about `D × types / total`, which is small for a context seen often with few followers.
- A novel word inside a long, often-repeated span backs off through about ten such factors, and the floors can reach 0. D87 holds long template spans (claude-411 / 414).
- Both the host scorer (`score_target`) and the kernel refused a zero mass by design. DEV-tune holds no such target, since every DEV-tune run passed both checks.
- **The message's other branch, a mass of 2^64 or more, cannot happen.** At each order the kept term is at most M, because `a ≤ A` and `D(a) ≥ 0`. Every accepted discount satisfies `Dq_i ≤ i M`, and `n1 + 2 n2 + 3 n3+ ≤ A`, so `gamma ≤ M` and the backoff term is at most `P_{k-1}`. So `P_k ≤ M + P_{k-1}`, and `P_N ≤ (N + 1) M + M / 4096`. That is less than 12 × 2^48, which is below 2^52, for N ≤ 10. The mass was 0.

### Decision review: the one-unit floor

**Decision** (lab, under §5.1, logged for Casey): R\*'s interpolated mass is floored at one Q48 unit, `max(P_N, 1)`, that is 2^-48.
- It applies in `KnTables::mass`, in the kernel and in `KnOracle::mass` (`c63510aa2`).
- `kn-reference` records `masses_at_floor`, the number of scored masses equal to 1. It is an upper bound on the floored masses, since a mass could compute to exactly 1 without the floor.

This is part of R\*'s definition, as claude-421's fallback rule is. R\* has not been sealed, so this defines the reference and does not change an established one.

**Why the floor.**
- Real-arithmetic KN gives an underflowing target a mass below 2^-48. The floor gives it exactly 48 bits, which is fewer than it should cost.
- So the floor can only make R\* look better. That makes bar Q (0.80 × R\*) harder, never easier: the floor is conservative for the model.
- It adds at most 4,096 units (2^-36) to any history's sum, one per id of the 12-bit vocabulary (`NGRAM_VOCABULARY`; `check_tokens` refuses any larger id).
- A mass that never reached 0 is unchanged. So every DEV-tune mass is unchanged, and N\* and the DEV-tune checks stand. Step 2 checks this byte for byte.

**Alternatives weighed.**
- **(b) A wider or scaled fixed point** (a mantissa and an exponent per mass). It would give underflowing targets their true cost, but it changes every floor and so every mass. The whole DEV-tune sweep and choice would rerun, and the kernel would be rewritten. It is the right shape for a later R\* with a wider key, reported beside the sealed one, as the first sheet's Edge reading already allows.
- **(c) Leaving R\* unsealed** blocks bar Q, and every model unit is judged against bar Q.
- **(d) Dropping such targets** biases the reference toward easy targets and changes the pre-stated totals.

**Reversal.** The rule is one `max` in three places. The JSON's `masses_at_floor` shows how far it reaches.

## Steps

Everything runs on wick's RTX 4090, from a source tarball of `c63510aa2` (or a later commit of this branch that changes only documents). The streams, paths, budget (2^24) and N\* = 10 are the first sheet's and job `c422-sweep5`'s.
1. **Build and CUDA tests** at this head: `cargo test --release --features cuda --lib -- ngram_count_cuda kn_score_cuda kn_filter`. All pass, including the new `kn_score_cuda_floors_an_underflowed_chain_as_the_host_does`.
2. **DEV-tune at N\* = 10 again**, `--check-host`, into `fix2/sweep-10.json`. It passes when all of these hold:
   - B1;
   - `masses_at_floor` is 0;
   - `jq -S 'del(.masses_at_floor)' fix2/sweep-10.json` equals `jq -S . sweep-10.json` (the run of `c422-sweep5`) byte for byte;
   - peak resident memory is at most 12 GiB.

   **Headroom.** `c422-fix2`'s DEV-tune run at N = 10, under the repaired filter, sampled resident memory once a second. It peaked at 9.54 GB while counting, and at 2.12 GB from the "gpu scored" line to the end, which covers scoring and the host check. On DEV-test, the query-sized terms (one order's key lists and the kept tables) are about twice DEV-tune's. That gives about 2.2 GB more while counting (about 11.8 GB) and at most about 4.3 GB while scoring, against the 16 GiB limit.
3. **Oracle at N\* = 10**, the first sheet's step 4b as written: the 2‰ slice, the stream sha256 checks first, and its peak memory and time logged. It runs because `KnOracle` changed, and it must pass.
   - The slice holds no floored mass. `KnOracle` builds from tokens, not tables, so it cannot read the underflow fixture. The oracle's floor is therefore not exercised on an underflow. The sealed number does not depend on it: the sealed run compares GPU with host only. The two floor tests pin the host and the kernel.
4. **The crash logs are kept.** The second crash's `sealed.log`, `smi-sealed.log` and `gpu-users-sealed.log` are renamed `crash2-*` before the sealed run writes new ones. The first crash's logs are already `crash1-*`.
5. **The sealed run**, once, by the first sheet's script `c422-sealed.sh` (sha256 `95e2567c…`), unchanged:

   `kn-reference train.tokens.u16 dev-test.tokens.u16 id-bytes.u8 dev-test.novel.u8 10 16777216 sealed.json --check-host --sealed-run`

   The job starts it only after steps 1–4 pass, and when at least 12 GiB of device memory are free. A sampler beside the unchanged script logs the instrument's resident memory once a second (`logs/sealed.rss`).
6. **Storage** is the first sheet's step 7, plus the `fix1/`, `fix2/` and `crash*` logs.

## Bar

- **B1 and B3** are the first sheet's. B2 is step 3 here. Step 2 here stands for the first sheet's B3 on the repaired code.
- **Recorded, with no bar:** `masses_at_floor` on DEV-test, its share of DEV-test's 21,349,576 targets, and the peak resident memory of steps 2 and 5.

**Reading map** (fixed before the run):
- **P (sealed):** steps 1–4 pass and the sealed run passes B1. R\*'s record is the first sheet's P list, plus `masses_at_floor`. The REPORT gives the bar Q lines as the first sheet says.
  - The REPORT names R\*'s definition with the floor rule in it.
  - It names N\* = 10 as "edge-limited" (the first sheet's Edge reading), with DEV-tune's gain from N = 9 to 10.
- **Floor-affected:** `masses_at_floor` above 1‰ of DEV-test's targets (21,349). R\* is still sealed, since the floor only lowers its bits, and is labelled "floor-affected". The REPORT then recommends the scaled fixed point (b) as the first later R\*, reported beside the sealed one.
- **I (instrument):** step 1, 2 or 3 fails. The repair is made under this sheet, and DEV-test is not touched until all three pass.
- **A sealed crash**, from memory, an overflow (ruled out above) or anything else before the "gpu scored" line, takes the Stop rule's crash line.

## Base artifact

- **Code:** `c63510aa2`, which is `main` at `9465b92d6` plus claude-422's commits. Three of those are memory-only repairs (`53cb85ee2`, `a15bb17b1`, `347f0122e`), and one is the floor rule (`c63510aa2`). The job writes the tarball's commit to `head.txt`.
- **Streams:** claude-420's `/workspace/out/c420/run1/` on wick, read only, the first sheet's list.
- **Machine:** wick, NVIDIA GeForce RTX 4090, in Claude's container with a 16 GiB memory limit.

## Data split

This is the first sheet's split. DEV-test is read only by step 5. Steps 2 and 3 use DEV-tune and TRAIN, and step 3 checks their sha256 first.

## Minimum test power

This is the first sheet's. Step 2's byte-for-byte check is exact. A changed mass anywhere on DEV-tune would change `masses_sha256` and fail it.

## Cost

Wick is the owner's GPU, so there is no rental (D86). The steps take about 35 minutes (step 2), a few minutes (step 3) and about 35 minutes (step 5), plus any wait for 12 GiB of free device memory. Another user shares the GPU, and the result is exact, so sharing changes only the times.

## Stop rule

- The stall rule is the first sheet's.
- A failure of step 1, 2 or 3 ends the job before DEV-test is read. It is repaired under this sheet.
- **The sealed run is run once under this sheet.** Its first written JSON is the record. The failure cases are the first sheet's:
  - **A crash**, an exit before the "gpu scored" line, is repeated once. A second crash needs a new sheet.
  - **A `--check-host` mismatch** goes to a repair and a new sheet.
  - **A JSON that fails B1** stays on record.
- No number seen on DEV-test feeds back into the choice of N or into the floor rule.
