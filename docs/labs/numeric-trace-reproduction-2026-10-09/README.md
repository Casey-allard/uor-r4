# The missing control is FIXED: 13 of 13 byte for byte — and the split is still VOID

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, $0** (two warm builds,
two ~12 s runs). No v5 re-run.

## 0. The structural qualifier stays visible

**The artifact that answered the v5 "memory" panel carries NO MEMORY OPERATOR — only a copy pointer.**
So **"10 of 40 on the memory category" is 10 of 40 for a copy-pointer dialogue stack on requests
*named* memory rows**, not for an addressed-memory path.

## 1. THE MISSING CONTROL IS FIXED: 13 of 13 BYTE FOR BYTE

**Root cause, found by reading `reply_panel` rather than guessing: the panel calls the reply closure
ONCE PER TURN, NOT ONCE PER ROW.**

```rust
for request in requests {
    for (turn, user) in request.user_turns.iter().enumerate() {
        let generated = reply(&history, max_new_tokens)?;   // once per turn
```

With 13 three-turn rows the previous probe took the **first 13 calls** — the first few rows' *turns* —
as if they were the 13 rows' replies. **That is exactly why `mem-040` and `mem-006` appeared to share a
byte-identical sequence: they were not those rows' replies at all.** The fix groups calls by request and
takes each row's **last turn** — the reply the check grades — and asserts the call count equals the
total turn count, so a mis-grouping fails loudly instead of silently.

**Acceptance condition, stated before the build and met exactly: ALL THIRTEEN rows reproduce their
sealed replies byte for byte — 13 of 13 — and `mem-040`'s is
`[2997, 1700, 386, 2605, 284, 2471, 435, 223, 26, 22, 16, 1]`, the reply containing the stored `84`.**
Verified against the sealed `replies-29m.json`. **So the sealed replies ARE reproducible from the
artifact by this path**, and the previous piece's "history-insensitive replies" are explained and gone.

## 2. THE SPLIT IS STILL VOID, AND NOT REPORTED

The fixed run produced `READER: 10, PARTIAL: 3` — **not a result either.** A second defect was found in
the same probe: the values file is tab-separated (`id`, `value`, `forbid`, `keys`) and the probe took
**everything after the first tab** as the value, so `digit_ids` came out as **12-token sequences for
two-digit values** (`[22, 19, 200, 22, 23, 200, 1986, 502, 94, 1986, 502, 386]` for `41`). Every class
assignment is keyed on the wrong token set and **means nothing**.

The extraction fix is written (`columns[0]` / `columns[1]`) but **the reported numbers come from the
pre-fix binary**, so they are void and are not reported as a split. **No reader-versus-emitter
classification is claimed by this piece, and nothing about pointer selection may be read out of the
void run.**

## 3. The four loud-failure conditions

| condition | status |
|---|---|
| 1. Both classes non-empty or INCONCLUSIVE | **not reached** — the deciding run is void |
| 2. `mem-040` positive control | **satisfied at the generation level** (reproduces byte for byte, contains `84`); unreadable at the *selection* level from the void run |
| 3. Per-step records for two rows | **not reported** — the audit steps are keyed on the wrong token set |
| 4. Token ids, never strings | **held** throughout |

## 4. The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The learned read/emit path | **OPEN** — no memory reader on this artifact, so the question is the pointer's; the instrument now **reproduces the sealed replies byte for byte (13 of 13)**, with one defect in its value extraction remaining |

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**One rebuild and one 12-second run** with the value extraction fixed, then report the split under the
four conditions — both classes non-empty or INCONCLUSIVE, `mem-040` as the positive control at the
*selection* level, per-step records for two rows, token ids throughout. The probe's generation path is
now trustworthy; only its key extraction is not.
