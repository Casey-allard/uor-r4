# REPORT: written-state-count-model, claude-427 (time what is left of push)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Threaded hashing did not take hashing off the caller's path** [claude]. Of push's 150.4 s, the
  caller spent **73.2 s (48.6%) in `send` on the hash channels**, which means blocked on a full
  queue plus the call itself: 29.4 s on the triple streams and
  43.7 s on the context streams. A partition's encoded records are larger than a stream's queue
  (four 4 MiB chunks), so the caller waits for its hasher while each partition is fed. Across
  the sweep, the 22 hasher threads were busy 134.8 s, 0.69% of their combined wall time. They
  sit idle between bursts and are behind during them. This is why claude-426's push fell
  104.7 s, not the ~206 s the scout estimated (from claude-423's 241 s of push to about 35 s).
- **Push did not get faster, and was not meant to.** It took 150.4 s, against claude-426's
  148.2 s. The +2.2 s includes the timers and run-to-run noise; this unit adds diagnostics only.
- **Host phases this unit did not touch moved again, by as much as 19 s each.** Against
  claude-426, `runs` fell 19.0 s while `adjust` rose 4.9 s, `records` 4.0 s, `own count` 6.9 s
  and `download` 2.4 s. So some of claude-426's unexplained falls in other phases (68.6 s in
  all) may be run-to-run variance on the shared wick, not an effect of the cursor. Two runs are
  not enough to say how much.
- **The ORDER's test wording was too strong for the channel waits.** A wait can rightly be zero
  when the hasher keeps up, so the test checks that each `send_wait` is no more than its feed. The
  ORDER now says so, in the same commit as the code (`20ee2f665`).

## What changed

1. **`NgramAssembler::enable_timing`** (`crates/uor-r4-discrete/src/ngram_count.rs`). It is off
   by default. When on, `push` is read by one stopwatch in five consecutive splits:
   - triple checks;
   - context checks;
   - census (`check_census`, the count-of-counts compare, the seam check);
   - triple feed;
   - context feed.

   Each `StreamHasher` adds the time its `send` blocked, and its thread adds the time spent in
   `Sha256::update`. `finish_with_timing` returns the totals as an `AssemblerTiming`. It reads
   the waits before the last pending context and the tail are sent, so a wait holds only sends
   made inside `push`. `finish` keeps its signature.
2. **`kn-reference`**, under `--profile-phases`, prints one `push parts ms` line per order and one
   for all orders, and writes `unit: claude-427`.
3. **Test:** `ngram_count_push_timing_is_recorded_only_when_enabled`, at orders 1–3 with 7-byte
   chunks.
   - The digests with timing on equal those with timing off.
   - With timing on, every sub-phase and both hashers record time, and each wait is ≤ its feed.
   - With timing off, every total is zero.

## Measurements

G2, wick 4090, job `c427`, code `20ee2f665`. The run used all of TRAIN (2,091,560,001 tokens)
against DEV-tune's keys, with N = 10, budget 2^24, `--count-only --profile-phases`.
Packet: [`run1/c427/`](run1/c427/).

- **Wall time.** The process took 900 s (claude-426: 904 s), and the count 887.5 s (889.3 s),
  over 128 shared ranges and 1,408 partitions.
- **Output unchanged.** `count-10.json` with `unit` deleted (`jq -S 'del(.unit)'`) is
  byte-identical to claude-426's (`cmp`, on wick and again on the laptop). The file's sha256 is
  `a5f0912684c5330b4395e9c29eea1109bac8ac018955c162df4fe51e7ae18064`.
- **Memory.** Peak RSS was 11,643,568 KiB (+6,416 KiB on claude-426). The whole-device GPU peak
  was 20,049 MiB on a shared device; 3,498 MiB were in use before the run.

Push, all orders, in ms. The five sub-phases are consecutive splits of one stopwatch.

| sub-phase | ms | share of push |
|---|---|---|
| triple checks | 5,976 | 4.0% |
| context checks | 5,520 | 3.7% |
| census | 17,738 | 11.8% |
| triple feed | 47,495 | 31.6% |
| … of which send wait | 29,428 | 19.6% |
| context feed | 73,671 | 49.0% |
| … of which send wait | 43,731 | 29.1% |
| **sum of the five** | **150,400** | push total 150,403 |
| hasher busy, triple streams (other threads) | 54,676 | |
| hasher busy, context streams (other threads) | 80,134 | |

Order 10 alone took 32,037 ms of push. Its sub-phases sum to 32,036 ms, of which:
- 16,273 ms were channel waits;
- its context hasher was busy 17,713 ms, against a context feed of 16,845 ms.

The waits grow with the order, as the streams do: 5 ms at k = 2, 5,598 ms at k = 5 and
16,273 ms at k = 10.

Count phases in seconds, from the raw ms in each `count-10.log`:

| phase | claude-426 | claude-427 | change |
|---|---|---|---|
| sweep wall | 889.3 | 887.5 | −1.9 |
| own count (device) | 290.6 | 297.5 | +6.9 |
| select | 29.2 | 28.9 | −0.2 |
| sort | 111.1 | 111.6 | +0.5 |
| emit | 18.8 | 18.9 | +0.1 |
| download | 40.6 | 43.0 | +2.4 |
| records | 89.1 | 93.1 | +4.0 |
| runs | 287.3 | 268.3 | −19.0 |
| adjust | 119.6 | 124.5 | +4.9 |
| push | 148.2 | 150.4 | +2.2 |
| keep | 15.8 | 16.4 | +0.6 |

Host work over every gram (download, records, runs, adjust, push, keep) is 695.6 s, 78.4% of
the count.

## What the numbers say

- **The digest is most of push.** The two feeds take 121.2 s, 80.6% of push: 73.2 s waiting
  on the hashers and 48.0 s encoding records and merging contexts on the caller. Validation
  takes the other 29.2 s (19.4%): the per-record checks and the census.
- **The hashers are the bottleneck during a feed.** The evidence is the 73.2 s the caller
  spent in `send` on full queues, and order 10's context stream: its hasher was busy 17.7 s
  against a feed of 16.8 s, and 10.3 s of that feed were waits. (The 134.8 s of busy time
  summed over 22 threads is no evidence on its own, since the hashers overlap and keep draining
  after a push returns.) The caller produces bytes faster than one SHA-256 thread consumes them,
  and a stream must be hashed in order. So each stream's serial hashing time is roughly a floor
  for the sequential digest. The largest is order 10's context stream, at about 17.7 s, measured
  beside 21 other hasher threads and with the clock read on every chunk.
- **What D3 has to take with it.** If D3 keeps every partition on the device, it either moves
  the digest there too, or still sends every gram's bytes to the host to hash in order.
  - **Digest on the device:** a parallel digest, which changes the replay identity. This
    removes the 121.2 s of feeds and the 136.1 s of download and records.
  - **Sequential digest kept:** the hashing floor stays, along with download and records.
    Push could still fall toward that floor without an identity change, if the caller stopped
    blocking. Each order's stream would be encoded and hashed off the caller's thread, with
    the partition held until it is done. That costs memory and is not measured here.
  - **Validation** (29.2 s) would have to be redone on the device or argued away. The census
    and the checks are what refuse a wrong partition today.

## Checks run at the head

- **G1, laptop** at `20ee2f665`:
  - host `kn_` and `ngram_count` tests: 36/36 (35 before, plus the new one);
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests`: clean;
  - the instrument's `cargo check --features cuda`: clean, and its tests 7/7;
  - rustfmt hunk counts unchanged from main: 58 in `ngram_count.rs`, 2 in the instrument;
  - no decimal literal or banned pattern under D32.
- **G1 negative:** with the `push` totals never written (`if let Some(timing) = None`), the new
  test fails with "order 1: triple checks recorded no time". With the file restored and rebuilt,
  it passes again. The test does not pin the `send_wait` totals: a wait may rightly be zero, so
  a wait that is never written would still pass it. The waits rest on the wick log, where they
  are non-zero and below their feeds.
- **G2, wick** at `20ee2f665`: PASS (above).
- **G3:** this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **D3, device residency,** with its own ORDER. Before choosing the digest, it should weigh
  the numbers above: a parallel digest on the device against the sequential one. The sequential
  digest has a hashing floor of about 18 s per stream, and still needs download and records.
- **A cheaper step that keeps the identity:** take the hashing waits off the caller by handing
  each order's partition to its own encode-and-hash thread. It may save up to ~73 s of push,
  at a memory cost that has to be measured first. Whether it is worth doing before D3 depends
  on whether D3 keeps the sequential digest.
- **Run-to-run variance on wick** is now visible: up to 19 s in one host phase between two
  runs of nearly the same code. A gain in a later unit smaller than that needs repeat runs.

## Plain summary

The step that hands each order's counts to the fingerprinting code takes about 2.5 minutes.
This unit split it into its parts. Nearly half of it, 73 seconds, is the main thread waiting
for the fingerprinting threads to catch up, because each order's fingerprint has to be computed
in order on one thread. About a third is preparing the bytes, and a fifth is checking the
counts. So moving this step to the GPU means deciding how the fingerprint is computed there.
Every output byte is unchanged.
