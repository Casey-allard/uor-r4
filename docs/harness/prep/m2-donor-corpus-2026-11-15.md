# Prep sheet: D87's first sealed donor corpus, 1B kept tokens from DeepSeek V4.1 Flash with reasoning (m2-donor-corpus-generation, claude-400, 15 Nov 2026)

Lab: claude. Item: [m2-donor-corpus-generation](../../roadmap/items/m2-donor-corpus-generation.md) (#2848). Order: [claude-400](../../loop/tasks/m2-donor-corpus-generation/claude-400/ORDER.md). Code: branch `claude/claude-400-donor-corpus`, PR #4028. Release cited at dispatch: D87 (§3.10), once PR #4027 is on `main`.

TEST FITNESS: FIT. A fresh-context non-author reviewer passed amendment 6 (finish over DeepSeek's API, the `--api openai` client, `balance` and api-watch.sh, 4ad0c58d9) with one blocker, on wording only: the receipt must say that the pre-registered math criterion failed as measured (16/38 and 17/38) and that "consistent" rests on a post-hoc reading by the lab, as a decision review. That is fixed in the next commit, together with its advisories: the watcher fails closed after 10 unreadable balances and counts spend as the sum of balance falls, part B covers abandoned prompts, a turn can drain past 00:40Z into peak, and a 429 storm ends a segment as `server`. Its gates (26 tests, clippy, float scan, shellcheck, prep-check, key not in argv) passed. Before that, A fresh-context non-author reviewer passed amendment 5 (an 8×B200 and a 4×B200 beside pod 3, three weighted parts, 17a83e711) with no blocker, after running the three-part split, seed, join and refusal stub gates and shellcheck. Its five advisories are applied in the next commit: the bank's source mix per part, PART 0 on pod 3's next run, production only after this FIT, the head-start note, and three roots. Before that, A fresh-context non-author reviewer passed amendment 4 (the full 1B on up to four 4×B200, e43ca4c61) with no blocker, after running the bank-split, seed, refusal, serve-argument and unserve stub gates, shellcheck and prep-check. Its five advisories are applied in the next commit: an engine-counter reading beside the kept reading for tuning, a fresh hold before every restart, the cap arithmetic corrected (pod 3 at $550), unserve stopping vLLM by its process group instead of by pids from nvidia-smi, and new leases ordered after D88 and the tuning result. Before that, A fresh-context non-author reviewer passed amendment 3 (the 4×B200 measured hour, 721aba647) with no blocker, after running the serve-argument, offer-pin and prep-check gates. Its four advisories are applied in the next commit: a 2×B200 RAM filter and a dated fallback rate for projection (b), comments, the 256-sequence choice for TP4, and serve presets named on the command line so that nothing is quoted through ssh. Before that, A fresh-context non-author reviewer passed amendment 2 (the 8×H200 measured hour, 9d5df4086) with no blocker; it ran the offer pins, the 29 guard gates and prep-check, and its six advisories (back-to-back segments under the 15 min grace, a $57.81 price pin, a cheapest-offer recheck before `up`, the NVL bound on R/2, a 07:00Z latest lease, the list of overridden steps) are applied in the next commit. Before that, A fresh-context non-author reviewer passed the 4×H200 NVL amendment (370b0e16b) with three advisories, fixed in the next commit; another passed this sheet in round 4 on c8df04e34 with no blocker, after rounds 1-3 returned NOT FIT (round 1: the guard ended per segment, no restore, unmapped endings; round 2: an empty listing verified, the guard exited with the pod up; round 3: a cut copy kept under its final name). Its six advisories (caps during an unverified final sync, a failed restore left in place, a fail-open emptiness check, a shared staging folder, unchecked log copies, a cumulative unreachable count) and the author's own lease-renewal finding are fixed in the next commit, each with a stub gate that fails on c8df04e34.

## The question

At what measured rate and cost does DeepSeek V4.1 Flash on one rented 4×H200 pod (one tensor-parallel-4 replica) produce hard reasoning-plus-answer texts? If the rate fits D87's bounds, the unit produces and seals 1B kept tokens from it. If it does not, the unit stops the pod and reports the measured numbers with a fallback recommendation; it does not switch donors itself, because Casey chose DeepSeek V4.1 Flash and chose no fallback.

## Steps

1. **Prompt bank** (laptop, no model): `donor-corpus bank --sources local/m2-sources --panels data/panels --seed 20261116 --out bank.jsonl`, at the instrument commit named in the receipt. The sources are fetched at pinned revisions by `docs/labs/m2-donor-corpus/fetch-sources.sh`, which checks each file against Hugging Face's LFS sha256. The bank's sha256, size and per-source counts are fixed here before the FIT:
   - **Bank:** sha256 `787e5ab4b7e783d0aab1ae2ea7c891b78b55cdafc775b8096bfd644d99ed1a61`, 773,996,008 bytes, 692,364 prompts (646,527 single, 45,837 two-turn). It was built at instrument commit `f8b838fa3`, and `bank.rs` is unchanged at the run's commit. A second build from the same sources compares byte-identical.
   - **Kept per source:**
     - wildchat 454,231 (45,319 two-turn), below its 650,000 cap;
     - numina 120,000 (cap);
     - openthoughts 112,091;
     - oasst2 4,963 (518 two-turn);
     - gsm8k 1,079.
   - **Dropped by the panel guard:** 20,169 in all (wildchat 11,004, numina 7,895, openthoughts 1,259, oasst2 10, gsm8k 1). The guard matches sentence by sentence, so it errs toward dropping.
   - **Dropped as duplicates:** 6,393 of gsm8k's 7,473 duplicate NuminaMath-1.5's GSM8K problems. Dedup runs across sources.
2. **Lease** after D87 is on `main`: one Vast 4×H200 (SXM) offer at ≤ $35/h, with all labs' running pods counted: the cheapest with reliability ≥ 0.985, ≥ 1 TB of RAM, ≥ 1.2 TB of disk and ≥ 1 Gb/s. On 16 Nov the 15 Nov 8×H200 at $34.25/h was gone, and the cheapest 8×H200 was $46.16/h, over the cap. A 4×H200 is the recipe's per-replica shape, about 315 GB of backbone weights on 561 GB of GPU memory with the Engram tables in host RAM; 45686955 (Japan, $23.08/h, reliability 0.995) looked like the pick that day. **Amendment (15 Nov, at lease time):** `uor-pod up` takes verified datacenter hosts only, and 45686955 is a community host, so it was refused; no datacenter 4×H200 SXM was listed. A verified datacenter 4×H200 NVL is then taken instead, with `--gpu h200nvl` (added to `uor-pod`'s policy table, still forbidden tier), on the same filters plus an NVLink bridge across all four GPUs (Vast `bw_nvlink` ≥ 400 GB/s) for TP4. It has the same 141 GB per GPU, but 3.7 TB/s of memory bandwidth against 4.8, so R may be lower; the measured hour decides, and step 8 is unchanged. Offer 21829053 (Czechia, $20.96/h, reliability 0.986, 1 TB RAM, 4.8 TB disk, 8 Gb/s, NVLink 478 GB/s) qualified. `up` is not pinned to one driver: Vast's query returned no offer with `driver_version=620.70.06`, while `up`'s own filters matched only 21829053 at lease time. Right after `up`, before `note`, `drive.sh offer POD` reads the rented instance from Vast and saves it as `offer.json`. It checks 4 H200 GPUs, at least 1,000 GB of RAM and `bw_nvlink` of at least 400 GB/s, and the driver downs the pod at once on a mismatch (a matter of seconds, so no guard is needed yet). It then reads `nvidia-smi topo -m` on the pod and fails unless every GPU pair is NV#, because Vast's `bw_nvlink` is not proof. **Pod 1 (16 Nov 03:46–03:48 UTC, about $1):** vast-60875524 on offer 21829053 was listed with 478 GB/s of NVLink, but on the pod every GPU pair read SYS or NODE and `nvidia-smi nvlink -s` reported no NVLink on any GPU. It was downed at once under this rule; its topology is kept in `local/m2-run/pod1-vast-60875524-topology.txt`. After that no Vast offer met this step: no verified datacenter 4×H200 SXM, no datacenter 8×H100 SXM, and the one datacenter 8×H200 at $56.85/h is over the cap. Community hosts are refused by `uor-pod`, and Runpod by hand needs the owner (its `uor-pod` path also has a 60 GB container disk). `pod.sh setup` also records `nvidia-smi topo -m` in the receipt. An F REPORT names the NVL form factor and its measured memory bandwidth, so it is not read as a result about 4×H200 SXM, nor compared one-for-one with a fallback donor's hardware. An 8×H200 is not taken in this unit, even if one appears under the cap: its bounds were never set, so it would need a new sheet. The one exception is amendment 2 below: one named 8×H200, for the measured hour only. The lease uses `UOR_POD_VAST_BIN=~/uor-r4/local/pod-provider-target/debug/uor-r4-pod-provider UOR_POD_VAST_MIN_RELIABILITY=0.985 UOR_POD_MAX_RATE=35 UOR_POD_VAST_DISK_GB=1200 UOR_POD_VAST_MIN_INET_MBPS=1000 UOR_POD_IMAGE=vllm/vllm-openai:v0.33.0-x86_64 scripts/pod/uor-pod up --lab claude --session claude-400 --purpose "D87 donor corpus" --hours 24 --gpu h200nvl --count 4 --no-bootstrap --owner-approved` (the owner approval is D87). The pod's RAM holds one Engram host-table copy per replica (about 197 GB each). As soon as `up` returns, `drive.sh note POD RATE_CENTS OFFER` records the lease time and rate. RATE_CENTS must be whole cents per hour, and a re-lease carries the earlier pods' cents and the first lease time. The guard `drive.sh watch POD` is then started at once (L-004); it refuses to start, with an alert, unless `note` has recorded this pod. It runs detached and kept awake: `caffeinate -i nohup drive.sh watch POD >> local/m2-run/watch.log 2>&1 &`. The driver also waits on the guard's pid, so it learns when the guard exits. The guard outlives segments and enforces the hard stops itself:
   - Every 10 min it syncs. It retries an SSH probe three times before believing a state.
   - A listing counts only if it ends with an `END` line. An empty final listing after a run started is refused.
   - A finished shard already on the laptop is never overwritten: a differing copy is a mismatch.
   - No sync runs while `put` is restoring.
   - At $1,100 spent or 60 h since the first lease, counting every pod, it writes `STOP` and waits 30 min for the in-flight prompts. If the run is still alive, it sends TERM, then KILL. It then downs the pod after a verified final sync.
   - Once no run has been alive for 30 min (3 h before the first segment, which covers setup), it downs the pod after a verified final sync. A restart of vLLM is announced with `drive.sh hold POD`, which counts as busy for 2 h.
   - After 30 min unreachable, it downs the pod; the last verified sync holds the shards.
   - Every hour it renews the `uor-pod` lease for 24 h (leases last at most 24 h), so the reaper never takes the pod during a GPU-idle stretch such as an engine restart. A failed renew is alerted and retried.
   - Copies land in a per-sync staging folder: a shard is kept only when its sha256 matches the pod listing, and a log copy replaces the local one only when it is whole. `put` refuses unless the pod reports an empty `out/` (an unreadable answer refuses); it clears a failed restore itself; `pod.sh run` refuses while a restore marker exists. A restore marker that stays with no run for three tries downs the pod (its `out/` holds only laptop copies).

   The guard never exits while the pod is up. A final sync that does not verify, or a `down` that fails, is retried every 5 min with an alert, written to stdout, to `local/m2-run/ALERTS` and as a desktop notification. Each segment is followed by `drive.sh wait-segment`, which tells the driver when it ends. Every one of these paths passed a local stub test (29 checks) before the FIT re-check; the gates added in each FIT round fail on the previous round's guard.
   **Amendment 2 (16 Nov 2026): one 8×H200 for the measured hour only.** For this lease it replaces step 2's machine, step 6's values, step 8, the Stop rule's one re-lease and the B1 reading; every other step and guard rule stands.
   - **Owner's words.** Casey wrote in claude's chat at 2026-11-16T04:30Z, relayed word for word by the parent session: "approve the $56.85/h host for the one-hour measurement". The lab reads this narrowly:
     - it covers that one host only, Vast offer 60874142 on machine 34325: Iceland, 8×H200 SXM, verified datacenter, listed at $56.842/h before storage, reliability 0.9946, 2,063 GB of RAM, 2.9 TB of disk, 7.85 Gb/s, `bw_nvlink` 478;
     - it covers the measured hour alone, not the 1B run.
   - **No substitute.** If that offer is gone, or its listed price (before storage) is above $56.85/h, nothing is leased and the unit reports.
   - **Storage.** Vast adds a disk charge: at `UOR_POD_VAST_DISK_GB=700` the all-in price is $57.80/h. The 700 GB holds the 511 GB of weights, the image, the client build and `out/`.
     - So the lease sets `UOR_POD_MAX_RATE=58` for this lease only.
     - Casey confirmed the disk charge. Asked in claude's chat at about 2026-11-16T04:40Z, "Is the disk charge covered by your approval?", Casey answered, word for word: "Yes, that's fine". The parent session relayed this. It covers offer 60874142 at about $57.80/h all-in with 700 GB of disk, and `UOR_POD_MAX_RATE=58` for this one-hour lease only; a larger disk (1.2 TB) is not covered.
   - **Widened (05:00Z).** Offer 60874142 stopped being listed at 04:43Z, and nothing was leased. Asked in claude's chat at about 2026-11-16T05:00Z, "Allow any verified datacenter 8×H200 at up to $58/h for the hour?", Casey answered, word for word: "Yes, any such host". The parent session relayed this. Its reading:
     - any Vast verified datacenter 8×H200 (SXM or NVL; an NVL board's R/2 is an upper bound, as in reading M) at no more than $58/h all-in with the 700 GB disk;
     - `UOR_POD_MAX_RATE=58` for this one-hour lease only;
     - community hosts stay out.

     Machine 34325 below then stands for "the cheapest qualifying offer's machine": the pin becomes that offer's `machine_id` (read right before `up`), with `M2_MAX_DPH` at that offer's all-in price plus $0.01, never above 58. An NVL offer is leased with `--gpu h200nvl`. Every other check stands.
   - **All labs' pods.** Right before `up`, the driver reads `uor-pod status` (with the adapter) and `vastai show instances`. The lease goes ahead only if no other pod of any lab is running, because any other pod would take the total past $58/h. It also re-runs `up`'s offer query (`vastai search offers` with the same filters and `--storage 700`) and goes ahead only if 60874142 is the cheapest match. `up` takes the cheapest matching offer and has no offer-id pin, so if another offer is cheaper, nothing is leased. The lease starts no later than 07:00Z, so that setup and the hour fit inside the 8 h cap below.
   - **Lease command:**
     `UOR_POD_VAST_BIN=~/uor-r4/local/pod-provider-target/debug/uor-r4-pod-provider UOR_POD_VAST_MIN_RELIABILITY=0.985 UOR_POD_MAX_RATE=58 UOR_POD_VAST_DISK_GB=700 UOR_POD_VAST_MIN_INET_MBPS=1000 UOR_POD_IMAGE=vllm/vllm-openai:v0.33.0-x86_64 scripts/pod/uor-pod up --lab claude --session claude-400 --purpose "D87 donor corpus: 8xH200 measured hour (Casey 04:30Z)" --hours 24 --gpu h200 --count 8 --no-bootstrap --owner-approved`
   - **Right after `up`:** `M2_EXPECT_MACHINE=34325 M2_MAX_DPH=57.81 drive.sh offer POD` checks:
     - 8 H200s and at least 2,000 GB of RAM, which is 1,000 GB per replica;
     - `bw_nvlink` of at least 400;
     - that the rented machine is 34325 and its all-in price is at most $57.81/h (the approved $57.80);
     - NV# between every pair of the 8 GPUs in the pod's own `nvidia-smi topo -m` (SXM boards bridge through NVSwitch).

     On any mismatch the pod goes down at once. Missing fields fail closed. Then `note POD 5780 60874142` runs; it carries pod 1's cents and lease time.
   - **Serve:** data-parallel 2 × tensor-parallel 4, which `pod.sh serve` sets from the GPU count, with `--max-num-seqs 256` per replica.
   - **Tuning (step 6):** client `--parallel` 512, then 1024, 8 min each: step 6's 256 and 512 per replica, doubled for two replicas.
   - **Measured hour (step 7):** as written, at the kept setting (`--max-seconds 3600`).
   - **After the hour the run ends, whatever the projection.** Under this approval nothing continues toward 1B. Fanning out for the corpus returns to D87's ≤ $35/h, and anything above that needs Casey's word again.
   - **Guard settings for this lease:**
     - `M2_CAP_CENTS=30000`: $300 across all pods, pod 1's $1 included;
     - `M2_CAP_SECONDS=28800`: 8 h from pod 1's lease at 03:46Z;
     - `M2_GRACE_FIRST=9000`: 2.5 h of setup before the first segment;
     - `M2_GRACE=900`: once the hour's segment ends, the pod goes down after a verified final sync within about 15 min, not 30. The grace counts from every segment's end, not only the hour's. So the two tuning segments and the hour run back to back: each starts as soon as `wait-segment` reports the last one has ended. Any gap that could pass 15 min (an engine restart) is covered by `drive.sh hold POD`, and the hold is cleared before step 7 so that it cannot bill idle time afterwards.

     Expected spend: about 1–1.5 h of setup, 16 min of tuning, the hour and the final sync. That is about 2.5–3 h at $57.80/h, or $145–175. The $300 cap is the backstop.
   - **Pod failure:** no re-lease under this approval, which named one host. The shards already off the pod are sealed as they stand.
   - **Reading M (measurement):**
     - The records from tuning and the hour are sealed and verified (B2 and B4 as written; either failing reads I). They are stored as a partial set marked "8×H200 measured hour".
     - The REPORT gives R (the hour's mean kept tokens/s), R per replica (R/2), generated tokens/s, P_yield and the hour's cost.
     - It gives two projections for 1B: (a) on this host at $57.80/h, P_cost and P_time; (b) under D87's $35/h, one replica at R/2 on a 4×H200 at the cheapest qualifying rate listed at report time, with the rate and its source named, stating whether both bounds ($1,000, 48 h) are met by one pod, and by the N such pods the cap allows. R/2 comes from an SXM board. A 4×H200 NVL, the only kind listed under $35/h so far, has 3.7 TB/s of memory bandwidth against 4.8, so for NVL, R/2 is an upper bound. The bounds in (b) are for the 1B run alone, on new pods; spend so far (pod 1 and this lease) is given beside them, not inside them.
     - B1 is not read. B3 and B5 are recorded. Then the unit stops: Casey's choice comes next.
   - **Pre-run review for the over-cap lease (the lab's, §5.10):**
     1. *Chain:* D87 (PR #4027 on `main`) releases this corpus for M2's distillation item #2848. The measured R is the one number missing from steps 7–8, and every choice left (cap, shape, time bound, fallback) turns on it.
     2. *Tools and inputs present:* the bank (sha256 `787e5ab4…`, in `local/m2-run/`); the model, engine and image pins; the instrument at its commit; `drive.sh`/`pod.sh` (29 stub gates, plus the offer checks above, stub-tested); the Vast adapter in the main checkout, which the guard checks before it starts.
     3. *What each outcome unlocks:* a pass (M) gives Casey the R to choose by; an I reading stops further spend until the instrument is repaired; an offer mismatch costs seconds.
     4. *FIT:* the delta FIT on this amendment, named in the TEST FITNESS line.
     5. *Lease:* one pod, all-in $57.80/h under the $58 set for this lease only, with no other pod running (checked at lease). The guard is armed before setup (L-004), and results are copied off before `down` (the guard's verified final sync).
   **Amendment 3 (16 Nov 2026): one 4×B200 for the measured hour only.** At 05:07Z no Vast verified datacenter 8×H200 was listed at any price. Asked in claude's chat at about 2026-11-16T05:10Z, "Allow a Vast datacenter 4×B200 at up to $44/h for the hour?", Casey answered, word for word: "Yes, use 4×B200". The parent session relayed this and reads it as: any Vast verified datacenter 4×B200, at no more than $44/h all-in with the 700 GB disk, for the measured hour only, with `UOR_POD_MAX_RATE=44` for that lease alone. Only one lease is taken, either this or amendment 2's, whichever is first. Everything in amendment 2 applies except what follows. **The go:** when this unit's session ran `up`, its own permission check refused the paid lease. In claude's chat Casey then wrote, word for word, "yes approved go please" (about 05:3xZ) and "go i said go" (about 05:40Z). The parent session relayed both, and the lease was run from the parent session under its own permission check, with this sheet's scripts. Where two offers tie at the cheapest price, the pin is the machine `up` actually rented, if it is one of the tied qualifying machines.
   - **Host:** at 05:09Z two offers were listed, both at $43.41/h all-in with 700 GB. 58523928 (machine 137780, US, reliability 0.995, 1,022 GB RAM, 10.4 TB disk, 14 Gb/s, `bw_nvlink` 900, CUDA 13.2, driver 595.91.07) qualifies. 51061479 (machine 67795, reliability 0.983) is below the 0.985 bar, and `UOR_POD_VAST_MIN_RELIABILITY=0.985` keeps `up` off it. The cheapest-match check right before `up` and the 07:00Z latest start stand.
   - **Lease command:** amendment 2's, with `UOR_POD_MAX_RATE=44 ... --purpose "D87 donor corpus: 4xB200 measured hour (Casey 05:10Z)" --gpu b200 --count 4` (`b200` is in `uor-pod`'s table, forbidden tier, so `--owner-approved` stays). Right after `up`, `M2_EXPECT_MACHINE=<the cheapest match's machine_id> M2_MAX_DPH=<its all-in + 0.01> drive.sh offer POD` checks 4 B200s, at least 1,000 GB of RAM, `bw_nvlink` of at least 400, the machine and price, and NV# between every GPU pair. Then `note POD 4341 <offer>` runs.
   - **Serve, from vLLM's B200 recipe (Blackwell TP):** TP2 per replica. `drive.sh pod POD serve b200` (a preset named in `pod.sh`, so no setting travels through ssh) gives data-parallel 2 × tensor-parallel 2 on four GPUs, `--attention-config {"backend":"FLASHINFER_MLA_SPARSE_DSV41","indexer_kv_dtype":"mxfp4","indexer_sparse_logits":true}` and `--max-num-seqs 128` (the recipe's) per replica. These go with the unchanged `--kv-cache-dtype fp8`, `--engram-config '{"cpu_offload":true}'` and `--reasoning-parser deepseek_v41`, and again no speculative decoding.
     - The recipe needs vLLM 0.31.0 or later; the pinned v0.33.0 image is newer. `smoke` proves sm_100 on the pod before any corpus record.
     - Memory: each TP2 replica holds about 315 GB of backbone on 2 × 183 GB, which leaves KV room the recipe sizes at 128 sequences. Two Engram host copies (about 394 GB) fit in 1,022 GB of RAM.
     - If DP2×TP2 does not start (out of memory or no KV blocks), there is one restart under `drive.sh hold` with `serve b200-tp4`: TP4 (one replica), the same attention config, and 256 sequences. That is the H200 recipe's value, chosen deliberately for a single replica; the B200 recipe pins 128 only for TP2. Tuning is then 256/512. The REPORT names the shape that ran.
   - **Tuning (step 6):** client `--parallel` 256, then 512 (2 replicas × 128 × {1, 2}), 8 min each.
   - **Guard:** amendment 2's settings. Expected spend: about 2.5–3 h at $43.41/h, about $110–130.
   - **Reading M:** as in amendment 2, with R per replica = R / DP.
     - Projection (b) is one TP2 replica on a 2×B200 at the cheapest verified datacenter rate listed at report time (the 2×B200 is Blackwell's per-replica shape, and was $24.79/h on 15 Nov), plus the N such pods the $35/h cap allows. The 2×B200 must have at least 250 GB of RAM for one Engram copy. If none is listed at report time, the REPORT says so and uses the 15 Nov $24.79/h figure, with its date.
     - If the run fell back to TP4, (b) is a 4×B200 at its rate, which is over $35/h; the REPORT says so.

   **Amendment 4 (16 Nov 2026): the full 1B on up to four 4×B200 pods.** In claude's chat at about 2026-11-16T06:50Z Casey wrote, word for word: "approve 4 pods at $43.41/h for the full 1B - make sure you do everything possible to make these run as quickly as possible for as cheap as possible because 1270$ sped is ridiculous". The parent session relayed it and reads it as: up to 4 pods, each at no more than $43.41/h all-in ($173.64/h in all), for the full 1B. It records this as D88 on `main` in its own PR. The measured hour still runs and is read as M; the REPORT gives R over the hour and over its last 50 minutes. The run then continues to 1B. Everything in amendments 2 and 3 applies except what follows. Leases are run from the parent session, because this session's permission check refuses `up`. They use this sheet's commands, the machine and price pin, and the `offer`, `note` and guard steps.
   - **Overrides:**
     - amendment 2's measured-hour-only scope and its step 8;
     - the Stop rule's $1,100 and 60 h stops and its one re-lease, replaced by the per-pod caps below;
     - the Cost section's "fan-out is not part of this sheet".
   - **Hosts:** Vast verified datacenter 4×B200, reliability ≥ 0.985, at no more than $43.41/h all-in with 700 GB.
     - Each lease uses `UOR_POD_MAX_RATE=44 ... --hours 24 --purpose "D87 donor corpus: full 1B, part K (Casey 06:50Z)" --gpu b200 --count 4`. Then `M2_EXPECT_MACHINE=<the machine up rented> M2_MAX_DPH=43.42 drive.sh offer POD` runs amendment 3's checks.
     - At most 4 pods, pod 3 included; all labs' pods count against the 4-pod cap at lease time. At 06:50Z one more machine qualified: 137780 (offer 58523928).
     - An 8×B200 ($85.01/h) is not taken under this amendment; it needs Casey's word.
     - Interruptible (bid) pricing is not used: `uor-pod` has no bid path, and none is built here.
   - **Cost per kept token is the target.** Tuning on pod 3 starts right after the measured hour, under `drive.sh hold`, with segments back to back. Each segment is 8 min and is read over its last 5 minutes, as in step 6.
     - **Two readings:**
       - *Primary (FIT advisory 1):* vLLM's own `vllm:generation_tokens_total` counter, summed over both engines, read from the server's `/metrics` at about minute 3 and minute 8, and given as generated tokens/s. Kept tokens count only when a record ends, so an 8-minute kept reading comes in low for the long records at higher concurrency. The engine counter has no such lag.
       - *Check:* the kept reading over the same 5 minutes may fall at most 15% below the baseline's. This catches a server that is busy but finishes nothing.
       - *Baseline,* taken during the measured hour at 256 in flight: 11,015 generated tokens/s (07:01:49–07:06:04Z, 2,808,835 tokens in 255 s), and segment A's 2,840,610 kept tokens.
     - **Restarts:** `drive.sh hold` runs again before every `unserve`, because a hold lasts 120 minutes from its touch (FIT advisory 2). `$W/HOLD` is removed before production.
     1. `pod.sh unserve`, then `serve b200 512` (512 sequences per replica), `wait-ready`, and a run at `--parallel 1024`. KV room: each replica's cache holds 43.6M tokens, and 512 × about 17k is about 8.7M.
     2. If step 1 beats the reading by at least 5%, repeat with `serve b200 1024` and `--parallel 2048`.
     3. Only if step 1 does not beat it: TP4 (`serve b200-tp4 512`, `--parallel 512`). TP4 halves the replica count, so it can only win where TP2 cannot use more sequences.
     4. Production runs at the setting with the best reading. If that is not the one being served, one more restart. Each restart costs about 14 min of engine start (06:11–06:25Z on pod 3), about $10. New pods serve at the chosen setting from the start.
   - **Wasted tokens:** across segments A and B, 5,557 records were kept and 125 dropped (121 for length, 4 empty). Length-dropped records hold about 16% of the generated tokens. A tighter `max_tokens` loses more kept text than it saves:
     - at 12,288 it cuts 64 kept records (902k kept tokens) to save about 496k generated;
     - at 8,192 it cuts 182 kept records (2.07M) to save about 991k.

     So `max_tokens` stays at 16,384. Prefix caching is already on (`enable_prefix_caching=True` in the engine log).
   - **Bank split:** `pod.sh run COMMIT PAR SECS PART`, with PART 0–3. Q = 173,091, a quarter of the bank.
     - Each part is bank lines PART×Q + 1 to (PART+1)×Q, with base seed 20261116 + 2×PART×Q, so every prompt keeps the seed of its full-bank index.
     - The budget is 250M kept tokens per part. Parts are disjoint, so no prompt is generated twice.
     - Pod 3 is part 0. It dispatches in bank order and had dispatched 5,682 prompts by 06:53Z (about 16k an hour at 256 in flight), all below index Q.
     - Yield so far is about 1,875 kept tokens per dispatched prompt (10.65M over 5,682), about 325M per part. A part that runs out first stops at its total, and the REPORT gives it. Any shortfall from 1B is the Stop rule's top-up bank, a new sheet.
   - **Local folders:** pod 3 keeps `local/m2-run/`. Pod K's folder is `local/m2-run/part-K/` (`M2_LOCAL`): its own lease, guard state and `out/`, plus copies of `bank.jsonl`, its manifest, `model.json` and `engine.json`. Shard names repeat across pods, so folders are never shared.
   - **New pods** run `put`, `setup`, `machine`, `serve` at the chosen setting, `wait-ready`, then production. The smoke is not repeated: the image, instrument commit and attention config are the ones pod 3's smoke proved. `verify` against the full bank runs on each part's first synced shard.
   - **Production:** one segment per pod, `run COMMIT <best PAR> 57600 PART`. It stops at the part's budget. The guard then makes its final verified sync and downs the pod after the 15 min grace.
   - **Guard per pod:** `M2_CAP_SECONDS=57600 M2_GRACE=900 M2_GRACE_FIRST=9000`, with `M2_CAP_CENTS=55000` for pod 3 and `45000` for a new pod. The guard counts spend as earlier cents plus this pod's own lease time × its rate. It counts time from the folder's first lease.
     - Pod 3's guard is restarted with these caps before amendment 2's 11:47Z stop. The parent stopped and restarted it before, by pid, and does so again.
     - **Pod 3** (lease 05:58Z, 52 earlier cents) reaches $550 after 12.65 h, at about 18:37Z, before its time cap at 19:47Z (16 h from 03:47Z). That leaves part 0 about 9 h of production after tuning, against about 6.7 h at 9.5k kept tokens/s (FIT advisory 3). Pod 2's ~$11 is outside the guard and is added by hand in the REPORT.
     - **A new pod** reaches $450 after 10.4 h of its own lease, against about 7.3 h for a part at 9.5k kept tokens/s plus about 0.5 h of setup.
     - One re-lease per part, through `put`'s restore into a new pod. A re-lease carries the cents already spent, so a late failure can stop a part on its cap. The REPORT reads a cap stop as a cap stop (S), never as a bank that ran out.
     - The caps are a backstop: $1,900 at most over 4 pods. The projection after tuning governs, and the REPORT gives it.
   - **Order:** the first new lease waits for D88 on `main` (the parent's PR), for this amendment's FIT, and for the tuning result, so every new pod serves at the chosen setting.
   - **Seal:** one sealed set per part, each verified against the full `bank.jsonl`. `verify` checks records by id, so the full bank holds for every part. Each set is stored at `results/claude/m2-donor-corpus-2026-11-16/part-K/` and read back. The REPORT lists the four roots and the summed kept tokens. Each set's `engine.json` names every serve setting it ran with and the UTC from which each applied.
   - **Expected:** if tuning gains nothing, one pod at about 9.5k kept tokens/s needs about 29 h for 1B, about $1,270. Four pods cut the wall time to about a quarter, for about $20 more setup per new pod. Each doubling of R halves the cost.

   **Amendment 5 (16 Nov 2026): one 8×B200 and one 4×B200 beside pod 3, and three weighted parts.** D88 merged on `main` as 4b770a92b. The parent offered Casey one 8×B200 at about $85/h in place of two 4×B200 pods. At about 07:40Z Casey answered, word for word: "ok i mean yeah go for it just get it done please as quickly and cheaply as possible". The parent leased both pods at about 07:42Z without waiting for the tuning result, because hosts were scarce and setup takes about 40 min. Amendment 4 applies except what follows.
   - **Pods:** pod 3 (part 0, 4×B200); part 1, an 8×B200 on machine 137783 (offer 57888900, about $85/h plus disk, `M2_MAX_DPH=86.80`); part 2, a 4×B200 on machine 137774 or 137780 (`M2_MAX_DPH=43.42`). That is three pods and 16 GPUs, about $172/h. There is no fourth pod.
     - The 8×B200 passes amendment 3's `offer` checks with its own GPU count: 8 B200s, at least 2,000 GB of RAM, `bw_nvlink` of at least 400, and NV# between every pair. `serve b200` gives it data-parallel 4 × tensor-parallel 2; four Engram host copies take about 790 GB of RAM.
     - Guard caps for part 1: `M2_CAP_CENTS=90000`, which is reached after about 10.4 h of its own lease. Part 2 keeps amendment 4's new-pod caps.
   - **Three weighted parts** replace amendment 4's quarters. `pod.sh run COMMIT PAR SECS PART`, PART 0–2:

     | Part | Pod | Bank lines | Lines | Budget (kept tokens) | Base seed |
     |---|---|---|---|---|---|
     | 0 | pod 3 | 1–200,786 | 200,786 | 290M | 20261116 |
     | 1 | 8×B200 | 200,787–526,197 | 325,411 | 470M | 20662688 |
     | 2 | 4×B200 | 526,198–692,364 | 166,167 | 240M | 21313510 |

     - **Budgets:** in proportion to GPUs (4 : 8 : 4) over the 1B still to make once the new pods produce, with pod 3's head start (an estimated 50–60M kept tokens by then) added to part 0. Parts should then end at about the same time if every GPU runs at the same rate.
     - **Line ranges:** in proportion to the budgets, so every part holds about 1.3 times the prompts its budget needs at about 1,875 kept tokens per prompt.
     - **Seeds:** each base seed is 20261116 + 2 × (first line − 1), so every prompt keeps its full-bank seed.
     - **Pod 3** has dispatched only lines below 200,786, so it resumes into part 0 unchanged.
     - An unequal rate across pods costs wall time, not dollars: a pod that ends first is downed, and the cost per kept token is unchanged.
   - **Serving on the new pods:** each starts production at the best setting so far (`serve b200`, 128 sequences per replica, `--parallel` equal to the replica count × 128, the server ceiling that won step 6: 512 on the 8×B200, 256 on the 4×B200) as soon as `wait-ready` passes. Each then moves to the tuned setting with one restart under `hold`, which `unserve` makes. Kept records from both settings stay in the part. The part's `engine.json` names each setting and the UTC from which it applied.
   - **Local folders:** `local/m2-run/part-1/` and `local/m2-run/part-2/`. The unused `part-3/` folder is removed before sealing.
   - **After the FIT (advisories applied):**
     - The bank is shuffled. Each part has the same source mix, about 65.5% WildChat, 17.4% NuminaMath, 16.2% OpenThoughts, 0.7% oasst2 and 0.15% gsm8k by line count. So the yield measured on part 0 holds for parts 1 and 2. The REPORT also gives each part's first-hour yield.
     - Pod 3's next `run` names PART 0. Until then it runs the full bank with the 1B budget, and only its guard would stop it.
     - Production on the new pods starts only after this amendment's FIT. Setup and `serve` may run before it. This replaces amendment 4's rule that leases come after the tuning result.
     - If tuning on pod 3 takes several restarts, its head start shrinks and part 0 ends last. That costs wall time, not dollars.
     - Sealing yields three sets and three roots, not amendment 4's four.

   **Amendment 6 (16 Nov 2026): finish over DeepSeek's API; pod 3 runs part 0 until the API path works.** Release cited: D89 (on `main` as 38c4a1994). For the API path, D89 makes four changes to D87's lab conditions. The API receipt names the endpoint, the model-version string, the request dates and the off-peak window, in place of a pinned checkpoint, engine and GPUs. Pod 3's shards and the API's are tagged apart, here by separate sets and `engine.json`. DeepSeek is the owner's term, and the OpenAI note is not a donor switch. The key comes from a file or env var and is never read, printed or logged by the lab. Casey wrote in the parent's chat, word for word: "i can just stream deepseek from api..." and then "go with your recommendation". The parent's recommendation was: down the two new pods, build an API path, keep pod 3 generating until the API path works, then down pod 3 and finish over the API. The two amendment 5 pods (vast-60895844, vast-60895824) were deleted before they had SSH. No file was ever put on them, and `part-1/` and `part-2/` hold no lease. Amendment 4's tuning is cancelled: it no longer pays off.
   - **Pod 3:** right after the measured hour, `run COMMIT 256 57600 0` (part 0, 128 sequences per replica, as measured). It runs under its guard (caps $550 and 16 h) until the API path is proven by the probe below. Then the driver writes `STOP`, the run finishes its in-flight prompts, the guard makes its final verified sync, and it downs the pod after the 15 min grace.
   - **API client:** `generate --api openai` (instrument change on this branch, reviewed before use).
     - It sends no `seed` and no `chat_template_kwargs`. The provider's thinking and effort keys go in an `--extra-body` file kept beside the run.
     - It sends `Authorization: Bearer` with the key, read from `--api-key-env` or `--api-key-file`. The lab never reads or prints the key, and no file holds it.
     - It stops (`refused`) on 401, 402 or 403, and on 32 refusals in a row. It backs off on 429.
     - Base URL and model come from the command line, so the provider is not fixed: `https://api.deepseek.com`, model `deepseek-flash`. DeepSeek's page names it DeepSeek-V4.1-Flash-1017 and allows at most 2,500 requests in flight.
     - The client runs on the laptop. It is an HTTP client with no model code, and nothing is trained or evaluated there.
   - **Sampling (DeepSeek's thinking-mode guide, https://api-docs.deepseek.com/guides/thinking_mode, read by the parent 16 Nov):**
     - The probe's first `--extra-body` file is `{"thinking": {"type": "enabled"}, "reasoning_effort": "high"}`; the probe result below sets the production level. The response carries the reasoning in `choices[0].message.reasoning_content`, which the client already reads.
     - In thinking mode the API ignores `temperature`, and on `deepseek-flash` it clamps `top_p` to 0.95–1.0. The body still sends temperature 1, top_p 1 and max_tokens 16,384, as on pod 3, but the API samples at its own settings.
     - Pod 3 samples at temperature 1, top_p 1, effort 50, and these API settings cannot match that exactly. The receipt records what was sent and the documented limits, never a claim that the sampling matched.
     - The effort level is chosen by the probe. `high` is tried first; if the mean length is outside ±30% of pod 3's, one more probe runs at the neighbouring level in that direction (`low` or `max`). The level whose length is closer is used, and recorded.
     - Multi-turn prompts send only the previous answer back, never the reasoning, as on pod 3.
   - **Seed rule: recorded, not replayable.** API records carry no seed; their segment says `"api": "openai", "base_seed": null`. The receipt and the REPORT say that these records cannot be regenerated from a seed. Pod 3's records keep their seeds, though batched vLLM decoding is not bit-replayable either.
   - **Model-version check (the probe), before any API corpus record:**
     - 64 prompts from part 0 that pod 3 has already answered, sent through the API into `local/m2-run/api-probe/`. That folder is not part of the corpus.
     - Read: the response's `model` field and any `system_fingerprint`; reasoning present in every turn; the final answers of the NuminaMath, OpenThoughts and gsm8k prompts compared with pod 3's (the last `\boxed{}` or the last number); and mean completion tokens compared with pod 3's on the same prompts.
     - **Consistent** if the `model` field names the Flash model, every turn has reasoning, math answers agree on at least 70% of the prompts where pod 3's answer is extractable, and the mean length is within ±30%. Sampling at temperature 1 makes exact agreement impossible.
     - **Inconsistent** otherwise: no API corpus is made, pod 3 keeps running, and the parent puts the numbers to Casey.
     - The weights behind an API cannot be verified, so the receipt records the provider, the model id and the `model` field returned, never a weights hash.
   - **Probe result (16 Nov, 08:08–08:12Z, peak hours, about $0.40; instrument 157a1bcde; files in `local/m2-run/api-probe/`).** The probe ran at `high`, then at `low` because `high` was outside the length band.

     | Effort | Kept / sent | Dropped | Length vs pod 3 (same 62 prompts) | Turns without reasoning | Math answers, exact string after normalising | Math answers, read |
     |---|---|---|---|---|---|---|
     | `high` | 62 / 64 | 2, both for length | 2,776 vs 2,094 (1.325×, outside ±30%) | 0 of 63 | 16 / 38 | 38 / 38 |
     | `low` | 62 / 64 | the same 2, for length | 1,826 vs 2,094 (0.872×, inside) | 0 of 63 | 17 / 38 | 38 / 38 |

     - The `model` field says `deepseek-flash` in every response, and no `system_fingerprint` was returned. That field echoes the requested alias, so it is weak evidence; the receipt records it and makes no further claim.
     - **The exact-string comparison is an instrument failure, not a model result** (§4.2, L-007). The extractor took the last `\boxed{}`, else the last number. Many of the math prompts are proofs, constructions or essays, so the last number is an arbitrary digit. The normaliser also misses equivalent forms: `3/4165` against `0.0007203`, `1:2` against `1/2`, `k<-4/7` against the interval, and `\ ` inside `\boxed{}`, which turned pod 3's `700\ \text{rubles}` into "0rubles". The lab read the extracted pair of each mismatch at both levels (22 at `high`, 21 at `low`). It read the closing lines of the final answers of the six where the pairs differ in substance (numina-367a8f982cdb88e7, numina-c7e1ca1096dda844, numina-04e782ebe306073f, openthoughts-9d1ef9b5822f9629, openthoughts-f1e5f448b124686e, openthoughts-1d1ea7dd80c1548c). None gives a different final result. The tables of every id are `high-compare.json` and `low-compare.json`.
     - **As measured, the pre-registered math criterion failed:** 16/38 (42%) at `high` and 17/38 (45%) at `low`, against a bar of 70%. The lab classified that failure as an instrument failure after the fact. "Consistent" rests on the lab's own unblinded reading, not on the registered measure. This is the lab's decision review (§5.1), logged for Casey, who can reverse it. The REPORT says so in these words.
     - **Verdict (decision review): consistent, at `low`.** The `--extra-body` file for every API segment is `{"thinking": {"type": "enabled"}, "reasoning_effort": "low"}`. `low` is also the card's name for pod 3's effort 50.
   - **Terms (reported by the parent, 16 Nov, quoted from the source pages):**
     - DeepSeek Open Platform Terms §4.2 assigns the Output rights to the user. It says you "may apply the Inputs and Outputs … to a wide range of use cases, including … training other models (such as model distillation)" (https://cdn.deepseek.com/policies/en-US/deepseek-open-platform-terms-of-service.html).
     - OpenAI Services Agreement §3.3(e), by contrast: the customer will not, "except for a Permitted Exception, use Output to develop artificial intelligence models that compete with OpenAI's products and services" (https://openai.com/policies/services-agreement/).
     - So the API provider is DeepSeek (`deepseek-flash`), as D87 chose. The base URL and model stay on the command line. The parent records Casey's switch to the API as D89 in its own PR. The REPORT carries both quotes.
   - **Off-peak only:** off-peak is outside 01:00–04:00Z and 06:00–10:00Z on weekdays, at $0.60 per 1M output tokens and $0.15 per 1M cache-miss input tokens. Peak is double.
     - Each API segment's `--max-seconds` ends it at least 20 min before the next peak window, so prompts still in flight usually finish off-peak. A 16,384-token turn takes about 12.4 min at 22 tok/s, so a multi-turn prompt in flight at 00:40Z can bill a few requests at peak. The receipt does not claim that every token was billed off-peak.
     - A 429 storm that outlasts `--server-wait-seconds` ends a segment as `server`, the same as an outage. The rate rows and the log tell the two apart.
     - The first segment starts no earlier than 10:00Z on 16 Nov.
   - **What the API covers:**
     - **Part A:** bank lines 200,787–692,364, the former parts 1 and 2 (491,578 prompts), into `local/m2-run/api/out/`. Its budget is 1B minus pod 3's final part 0 kept total, set when pod 3 stops.
     - **Part B,** only if A runs out first: part 0's prompts whose ids are neither among pod 3's records nor among its drops, so prompts that pod 3 abandoned are included. It goes into `local/m2-run/api-b/out/`.
   - **Concurrency:** start at `--parallel 256` for 5 min. Then 1,000, then 2,000 (below the 2,500 limit), each after a clean 5-minute rate row: no `refused` and errors under 1% of requests. 429s mean step back to the last clean setting.
   - **Segments.** `--parallel` is fixed for a segment, so the ramp is three segments into the same `out/`. Each one resumes from the final shards, and each finishes its in-flight prompts when its time ends. All run with `--api openai --api-key-file ~/.config/uor-r4/deepseek.key --server https://api.deepseek.com --model-name deepseek-flash --max-tokens 16384`, the `low` extra-body file and the bank of part A.
     - **A1:** 10:00Z, `--parallel 256 --max-seconds 300 --budget 900000000`.
     - **A2:** `--parallel 1000 --max-seconds 1500`, same budget. When it ends, `verify` runs on a copy of `out/` against part A's bank. If it passes, pod 3 gets `STOP`, then its final verified sync and down. That gives part 0's final kept total K0.
     - **A3:** `--parallel 2000 --budget (1,000,000,000 − K0)`. Its `--max-seconds` ends it at 00:40Z, 20 min before the 01:00Z peak. If A runs out of prompts first, B runs the same way into `api-b/out/` with the budget still left.
     - At 2,000 in flight, the probe's 22 tokens/s per stream gives about 44k generated tokens/s, so A3 should finish inside one off-peak window. If it does not, it resumes after 10:00Z on 17 Nov.
     - Before each segment, the driver checks that `out/STOP` is absent. A STOP left by the watcher ends a new segment at once, which is the safe direction.
   - **Cost and watch:**
     - Cost is computed from the recorded usage: output tokens (kept and dropped) × $0.60/M, plus prompt tokens × $0.15/M, counted as cache misses.
     - At pod 3's measured 84% kept fraction, 1B kept tokens is about 1.19B output, about $715 plus about $20 of input.
     - The probe at `low` kept 77.6% of what it generated, lower than pod 3's 84%, because its two length drops are 2 of only 64 prompts. Bounds for 1B kept are therefore 1.19B to 1.29B output, about $715 to $775, plus about $20 of input.
     - **Watcher:** `docs/labs/m2-donor-corpus/api-watch.sh local/m2-run/api-watch 85000 local/m2-run/api/out local/m2-run/api-b/out`, started before A1.
       - It reads the provider's balance once a minute with `donor-corpus balance`, which reads the key the same way `generate` does. It counts spend as the sum of every fall in the balance, so peak pricing, dropped tokens and re-billed retries are counted, and a top-up never hides spend.
       - **The balance is the account's, not the corpus's.** Casey has OpenCode on wick using DeepSeek (`deepseek/deepseek-flash`, `deepseek-v4-pro`); if that bills the same account, the watcher counts it as spend too. That can bring the $850 stop forward, but it never lets the run overspend. So the receipt reports two figures:
         - the balance-based spend, marked as possibly including other use of the account;
         - the corpus-only cost, computed from our own usage counts at the price list.
       - Segment budgets are set from our own kept-token counts, never from the balance. A watcher STOP that fires while the corpus-only cost is clearly under $850 is a measurement STOP, not a result about the run, and it is reported as such.
       - At $850 it writes `STOP` into both out folders. It alerts on a segment that ended `refused`, on a balance that rises, and at 5 failed readings in a row. At 10 failed readings in a row it writes `STOP` (fail closed).
       - The balance read at 08:14Z was $1,168.17, so the provider's prepaid balance (402) is the outer stop above the watcher.
       - Stub tests checked:
         - no STOP at 9,999 cents and STOP in both folders at 10,000, with a top-up in between;
         - one alert for the rise and one per refused segment;
         - an alert at the fifth failed reading, and STOP at the tenth.
         shellcheck passes.
     - The `drive.sh` pod guard does not apply: there is no pod.
   - **Seal:** part 0 (pod 3), A, and B if made, each verified against the full `bank.jsonl`. They are stored at `results/claude/m2-donor-corpus-2026-11-16/{part-0,api-a,api-b}/` and read back. Each receipt's `engine.json` names its engine: vLLM on pod 3, or the API provider, base URL, model id and returned `model` field.
   - **Storage (Casey, relayed 16 Nov):** "be conscious of wick's hard drive space and the hard drive space on this local laptop as well".
     - The laptop had 22 GiB free at 08:20Z. The raw 1B is about 5 GB (195 MB per 40M kept).
     - Each sealed shard is compressed with zstd. The manifest keeps both the raw and the compressed sha256. The raw copy is deleted only after the compressed one decompresses to the raw sha256.
     - Copies:
       - The HF store at the path above.
       - A third copy on wick, at `/mnt/plexpool/uor-r4-corpus/` only: ZFS dataset `plexpool/uor-r4-corpus`, owned by callard, zstd. It gets `rsync -a --partial` over `ssh wick`, retried with backoff, then `sha256sum` on wick against the manifest. A shard counts as backed up only when the hashes match.
     - Nothing is written to wick's root disk. Copies never block generation.
3. **Setup** (`docs/labs/m2-donor-corpus/pod.sh setup`): download `deepseek-ai/DeepSeek-V4.1-Flash-1017` at revision `08c8657f02291e11b6f6b63e0e16014f8e748518` (MIT, not gated, about 511 GB in 48 shards). Check every LFS file against Hugging Face's sha256. Build the client on the pod at the receipt's instrument commit. Then `drive.sh machine POD` writes `machine.json` from the lease (offer, rate, lease time) and the pod's `gpus.csv`, `host-mem.txt` and vLLM version, and pushes it to the pod. `model.json` and `engine.json` are written on the laptop before `put` and hold the pins of Base artifact.
4. **Serve:** one vLLM server, `--tensor-parallel-size 4` with `--data-parallel-size` set to the GPU count over 4 (1 on a 4×H200; `pod.sh serve` refuses a count that is not a multiple of TP, which is 4 unless a preset of amendment 3 sets it), with the recipe's H200 settings: `--tokenizer-mode deepseek_v41`, `--kv-cache-dtype fp8`, `--engram-config '{"cpu_offload":true}'`, `--reasoning-parser deepseek_v41`, `--language-model-only`, no speculative decoding (the recipe marks it for small batches), `--max-num-seqs 256 --max-num-batched-tokens 8192`.
5. **Smoke** (instrument check, not corpus): `generate --limit 32 --parallel 8` into `smoke/`. It checks that `reasoning` and `answer` arrive in separate fields with no markup, that two-turn prompts run, and that `seal`/`verify` pass on the smoke directory. Smoke refuses to start if `model.json`, `machine.json` or `engine.json` is missing. It fails unless the recorded segment shows effort 50, max_tokens 16,384, base seed 20261116 and model name `donor`; temperature 1 and top-p 1 are fixed in the request body. The smoke output is never part of the corpus.
6. **Tuning, at most 20 minutes:** client `--parallel` 256, then 512, each for 8 minutes from the start of the bank. The setting with the higher kept tokens per minute over its last 5 minutes is kept. These records stay in the corpus, because they are the bank's first prompts at the same sampling.
7. **The measured hour:** 60 minutes at the kept setting, read from `rate.jsonl` (kept tokens in donor tokens per minute). The projection is P_cost = spent + (1B − kept) / R / 3600 × rate_per_h, and P_time = elapsed + (1B − kept) / R / 3600, where R is the hour's mean kept tokens/s. Because the bank is globally shuffled, the hour also gives the bank's yield, P_yield = (kept tokens / prompts dispatched) × 692,364. The yield is recorded and reported, with no stop on it. If P_yield < 1B, the run continues until the bank runs out (an S reading), and a top-up bank from WildChat's other 43 files is a new sheet.
8. **Continue or stop:** if P_cost ≤ $1,000 and P_time ≤ 48 h, the run continues to the 1B kept-token budget. Otherwise the DeepSeek run stops, its shards are copied off, sealed and verified as a separate partial set, the pod goes `down`, and the unit reports P_cost, P_time, R and the fallback recommendation below as its result. The parent session puts the choice to Casey; nothing more is leased under this sheet until Casey answers.
9. **Offload:** every 10 minutes, finished shards and their counts files are copied off the pod to the laptop and checked by sha256. That copy is the guard's main job. A final sync also checks the rate, segment, dropped and metrics logs by sha256, and `down` comes only after it is verified.
10. **Seal and verify:** `seal` writes `receipt.json` (shard sha256s and Merkle root, the bank's sha256, the counts, rate and segment logs, the instrument commits, model, engine image and digest, GPUs). `verify` runs on the laptop, and again after a fetch back from the HF store (`results/claude/m2-donor-corpus-2026-11-16/`).
11. **Project tokenizer:** kept texts are counted in the 4,096-token tokenizer (sha256 `d36d3e87…`) on the laptop, if its encoder runs at that scale. This is a count only, with no bar.

**Fallback recommendation (the lab's, not an owner term; it does not run under this sheet):** `openai/gpt-oss-120b` at revision `b5c939de8f754692c1647ca79fbf85e8c1e70f8a` (Apache-2.0, not gated), on the cheapest verified B200 offer that fits the cap (2×B200 was $24.79/h on 15 Nov). It would use the same vLLM image, client, bank (continuing after the DeepSeek prompts), sampling and stop rules, with reasoning from the analysis channel through vLLM's gpt-oss reasoning parser and `reasoning_effort` "medium" passed through `--extra-body`. If Casey chooses it, it runs under a sheet amendment with its own FIT, smoke, measured hour and bounds.

## Bar

- **B1 size:** kept tokens ≥ 1,000,000,000 in the donor's tokenizer, so the budget stop was reached.
- **B2 receipt:** `verify` passes after the round trip through the store. It covers shard sha256s, the Merkle root, the bank hash, every record's messages against the bank, and the structural check: a record has exactly `id`, `kind`, `source`, `messages` and `turns`, and a turn has exactly `reasoning` and `answer`, with no markup and a non-empty answer.
- **B3 quality floor:** dropped prompts (length, empty, markup, http) ≤ 10% of those dispatched; `http` drops ≤ 1%.
- **B4 panels:** the bank holds no panel string (dropped by the guard; the count is reported).
- **B5 bounds:** dollars spent ≤ $1,100, which is the $1,000 bound plus at most one tuning or setup overrun; wall time from lease to `down` ≤ 60 h.

**Recorded, with no bar:** kept and generated tokens/s per minute; P_yield; segments that stopped with `stopped_by: "server"` (the client waits up to 30 min for `/v1/models`, three times per prompt, and never drops a prompt because the server was down; only a 4xx other than 429 counts as an `http` drop); the share of empty reasoning; tokens per record by source and kind; reasoning versus answer bytes; request p50/p95; GPU and host-memory use; and the count in the project tokenizer.

**Reading map** (fixed before the run):
- **P:** B1–B5 hold. The corpus is sealed and stored. The REPORT gives tok/s, wall time and dollars.
- **F (over bound):** the measured hour projects > $1,000 or > 48 h. The pod goes down, the DeepSeek partial set is sealed, stored and marked partial, and the unit reports the numbers and its fallback recommendation for Casey's choice. No donor switch happens under this sheet.
- **M (measurement, amendment 2 only):** the 8×H200 hour ran and its records verify. The partial set is sealed, stored and marked, and the REPORT gives R and both projections, then stops. No 1B run happens under that lease.
- **I (instrument):** B2 or B4 fails. The instrument is repaired, and nothing is used until a re-verify passes.
- **Q (quality):** only B3 fails. The corpus is sealed and stored with the failure named. Use waits on the cause.
- **S (short):** B1 fails for any of these reasons:
  - a hard stop or the money bound;
  - the bank running out;
  - a second server stop;
  - a second pod failure.

  The partial corpus is sealed and stored, and the REPORT gives the cause and the measured numbers.
- **Server stop:** a segment ending with `stopped_by: "server"` (vLLM down or hung for 30 min, three times) is not yet a reading. `serve` is restarted once under `drive.sh hold`, and a new segment resumes. If the guard downs the pod during that restart anyway (the hold expired), it reads as a pod failure. A second server stop reads as S, with the vLLM log named.
- **Several bars fail together:** I wins over everything, then S, and Q is named alongside. Example: B1 and B3 read as S, with the B3 failure also named. F (over bound) is checked at the measured hour, before any of these can arise.

## Base artifact

- **Donor:** `deepseek-ai/DeepSeek-V4.1-Flash-1017` at `08c8657f02291e11b6f6b63e0e16014f8e748518`, MIT (card and `LICENSE` read 15 Nov 2026). Recommended sampling is temperature 1.0 and top-p 1.0. Thinking mode is on, with reasoning effort 50 (the card's "low").
- **Engine:** vLLM `v0.33.0-x86_64`, image digest `sha256:91ee972ab751fb7180bc71c175865e36b091686600dca783d52b2511399810b3` on Docker Hub (2 Nov 2026). It runs on the pod, outside every compiled module (§3.4).
- **Instrument:** `docs/labs/m2-donor-corpus/instrument` (Rust, a client with no model code) at the commit each segment records.
- **Machine:** one Vast 4×H200 NVL in a verified datacenter (the amendment in step 2). The offer id, host, GPU UUIDs and driver are recorded in the receipt.

## Data split

- **The prompt sources** are user turns only, at pinned revisions:
  - `allenai/WildChat-4.8M` at `c827c6df` (ODC-BY; English, non-toxic, not redacted, first user turn; 43 of 86 files);
  - `project-numina/NuminaMath-1.5` at `1b05109f` (Apache-2.0);
  - `open-thoughts/OpenThoughts-114k` at `bd093c39` (Apache-2.0);
  - `OpenAssistant/oasst2` at `179dd21f` (Apache-2.0; English root prompts, train file only);
  - `openai/gsm8k` at `740312ad` (MIT; main/train only, no test split).
  Every assistant text in them is ignored. ODC-BY asks for attribution, which the receipt and REPORT give.
- **Two-turn exchanges** (about 10% of the WildChat and oasst2 prompts) add one generic follow-up from a fixed list of 40 written in `bank.rs`. Both assistant turns come from the teacher.
- **Panel guard:** every prompt, every one of its sentences and every follow-up is checked against every string in `data/panels/` after lower-casing and dropping spacing and punctuation (the pilot's guard). A match is dropped.
- **Use:** the corpus is training data only, never a panel, a grader or a comparator (D41 (c)). Public benchmark problems may occur in the NuminaMath and OpenThoughts prompts. They are not the project's evaluation panels, and this is stated in the REPORT.

## Minimum test power

- This is a data-production run, not a mechanism claim. There is no seed requirement.
- The rate is the mean over 60 one-minute windows, reported with the minimum and maximum window, so a warm-up or stall is visible.

## Cost

- **DeepSeek path:** one 4×H200 was $23.08/h on 16 Nov (offer 45686955). At R kept tokens/s, 1B takes 277,778 / R hours. Both bounds are on one basis: about 1.5 h of setup produces nothing, while tuning and the measured hour produce corpus records.
  - The $1,000 bound leaves $965 for 41.8 h of generation, so it needs R ≥ about 6,650.
  - The 48 h bound leaves 46.5 h, so it needs R ≥ about 5,970.

  At another rate p, the cost bound needs R ≥ 277,778 / ((1,000 − 1.5p) / p). At the amended $20.96/h: the $1,000 bound leaves 46.2 h, so R ≥ about 6,010; the 48 h bound still needs R ≥ about 5,970. The F path costs about $52–63. Step 8's P_cost and P_time govern; these figures are a guide only.
- **Unmeasured guess:** one replica gives about half of the two-replica guess, so 20–60 h and $460–1,390. Weights download at about 1 GB/s; setup is estimated at 1–1.5 h.
- **Fallback recommendation (not run here):** about 5–10 h on 2×B200 at about $25/h, if the published per-B200 rate holds. That is a hypothesis, given to Casey with the measured hour if F applies.
- **F path cost:** setup plus the measured hour, about 2.5–3 h at the amended $20.96/h (about $52–63).
- **Caps:**
  - Per replica, the price is the same as the 8×H200's, about $17–23/h against $17/h. So D87's $35/h buys about one replica, not the four pods D87 allowed.
  - On 16 Nov no second 4×H200 fitted beside the first under the cap ($23.08 + $17.36 = $40.44). Fan-out is therefore not part of this sheet.
  - If F applies, the REPORT adds the fleet projection: the pods of the measured shape that the cap would need to meet both bounds. The parent session can then put the choice to Casey: the cap, the time bound, or the fallback.
  - Other labs' pods are counted at lease time, and the run waits if they would breach the cap.

## Stop rule

- Dispatch stops at 1B kept tokens, and the prompts in flight finish.
- After the measured hour, step 8's continue-or-stop rule applies.
- The guard enforces the $1,100 and 60 h hard stops itself, with one named exception: a final sync that keeps failing to verify (a mismatch, or no trustworthy listing from a reachable pod) keeps the pod up past the caps, with an alert every 5 min, because downing it would lose unverified shards. The driver reads `ALERTS` at every check and settles that state by hand; it is reported as S.
- Hard stops are reported as S:
  - dollars spent reach $1,100;
  - wall time from lease reaches 60 h;
  - the bank runs out before 1B, in which case the REPORT gives the kept total, and a top-up bank is a new sheet.
- **Pod failure:** shards are already off the pod. One re-lease is allowed under this sheet. `drive.sh put` restores every finished file of the laptop's `out/` to the new pod: shards, counts, dropped, rate and segment logs. It checks each one by sha256 readback and refuses a pod whose `out/` is not empty. `generate` then resumes from those files, so no prompt id is generated twice. The cost bounds count both pods: the new `note` carries the first pod's cents and lease time, so the guard's $1,100 and 60 h stops cover the whole run. A second pod failure reads as S.
- One production pass only (D41 (f)). The corpus is not regenerated under this sheet.
