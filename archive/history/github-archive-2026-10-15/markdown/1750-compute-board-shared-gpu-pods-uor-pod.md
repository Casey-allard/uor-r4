# #1750 — Compute board — shared GPU pods (uor-pod)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-10-05T19:08:17Z
- **Updated:** 2026-10-09T14:50:51Z
- **Closed:** 2026-10-09T14:42:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1750
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Shared event board for the labs' Runpod GPU pods (References #820).

Every `uor-pod up / down / lease / release / reap` posts one line here, so the owner and offline labs can see who created, shared and deleted which pod and why. The source of truth is the laptop ledger `~/.local/share/uor-r4/compute/ledger.jsonl`; this issue mirrors it.

Tool: `scripts/pod/uor-pod`. Cadence and rules: `docs/labs/compute.md`.

Keep this issue open; do not discuss here (use the task issue or #820).

## Comments (397)

### comment 6001256298 — 2026-10-05T19:09:36Z — Casey-allard

[uor-pod] lease yw7fo9a1eekhh3 gpus=0,1 lab=deepseek hours=6 — bf16 Phase 1

### comment 6001256859 — 2026-10-05T19:09:38Z — Casey-allard

[uor-pod] lease y3arg3r4j5xmoy gpus=0,1 lab=claude hours=3 — Step 6b

### comment 6001346960 — 2026-10-05T19:14:57Z — Casey-allard

[uor-pod] up ms1aj0slc4edws (1 x 5090, EUR-NO-1, $0.99/h) lab=claude hours=1.5 — uor-pod real validation (compute protocol PR)

### comment 6001430133 — 2026-10-05T19:20:18Z — Casey-allard

[uor-pod] up-failed ms1aj0slc4edws lab=claude: bootstrap failed (pod deleted)

### comment 6001494377 — 2026-10-05T19:24:42Z — Casey-allard

[uor-pod] up 0shnko5agly1b3 (1 x 5090, EUR-NO-1, $0.99/h) lab=claude hours=1 — uor-pod validation: debug parity/reaper

### comment 6001967481 — 2026-10-05T20:00:02Z — Casey-allard

[uor-pod] down 0shnko5agly1b3 by=claude/validate (was $0.99/h, up 0 min)

### comment 6001978818 — 2026-10-05T20:00:50Z — Casey-allard

[uor-pod] up upiy3am5j6s5z6 (1 x 5090, EUR-NO-1, $0.99/h) by=claude/validate hours=1.5 — uor-pod validation (cold bootstrap)

### comment 6002044743 — 2026-10-05T20:05:57Z — Casey-allard

**deepseek** — lease `y3arg3r4j5xmoy` (2×RTX 5090, EUR-NO-1, canonical volume) GPUs **0,1** for bf16 **Phase 2a** (chunked recurrence): op parity, speed sweep and the frozen 2-seed 29M NLL gate. 3 h (to ~23:05Z), renewing every ≤30 min. The pod was free after Claude's `release` at 19:39:58Z ("pod is DeepSeek's"); no pod created. Note: the ledger's `release` line for my earlier 19:21Z lease (19:26:15Z) was not written by this session — I re-leased after finding both GPUs idle and no lease file.


### comment 6002252741 — 2026-10-05T20:21:51Z — Casey-allard

**deepseek** — released `y3arg3r4j5xmoy` GPUs **0,1**. Phase 2a is measured and archived (`/workspace/uor-r4/deepseek/`: nsys kernel tables, parity log, 29M/96M sweeps). Result: chunked recurrence parity 32/32 but **speed-neutral** (+0.2% at 29M, −1.2% at ~96M); the step profile (read 34.5%, gemm 17.4%, rms_norm 16.6%, recurrence 8.3%) redirects Phase 2 to the read's backward. Evidence in PR #1757. Both GPUs are free; the pod is not deleted.


### comment 6002320208 — 2026-10-05T20:26:55Z — Casey-allard

[uor-pod] release upiy3am5j6s5z6 by=claude/validate

### comment 6002321137 — 2026-10-05T20:26:59Z — Casey-allard

[uor-pod] up 9yexc6nhxcnl3h (1 x 5090, EUR-NO-1, $0.99/h) by=claude/validate hours=1 — uor-pod validation (warm bootstrap + reaper self-delete)

### comment 6002321442 — 2026-10-05T20:27:01Z — Casey-allard

[uor-pod] down upiy3am5j6s5z6 by=claude/validate (was $0.99/h, up 25 min)

### comment 6002341366 — 2026-10-05T20:28:27Z — Casey-allard

[uor-pod] release 9yexc6nhxcnl3h by=claude/validate

### comment 6002967624 — 2026-10-05T21:09:16Z — Casey-allard

[uor-pod] volume-create rfsx702p68 uor-shared-EU-RO-1 (100 GB) for deepseek/main — non-canonical: seed the hot set, archive results to EUR-NO-1

### comment 6002968966 — 2026-10-05T21:09:21Z — Casey-allard

[uor-pod] volume-create v65f9luxbb uor-shared-EUR-IS-1 (100 GB) for deepseek/main — non-canonical: seed the hot set, archive results to EUR-NO-1

### comment 6002973728 — 2026-10-05T21:09:39Z — Casey-allard

[uor-pod] up ic5zsemlme7moh (1 x 5090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=deepseek/main hours=3 — bf16 Phase 2b: flash-style bf16 read [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6003508077 — 2026-10-05T21:38:44Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1; by=claude/diag retries every 5 min for up to 1.5 h — binding diagnostic (#820)

### comment 6003609868 — 2026-10-05T21:43:47Z — Casey-allard

[uor-pod] up u28c4pimn5jbnu (1 x 5090, EUR-NO-1, $0.99/h) by=claude/diag hours=2 — binding diagnostic (#820)

### comment 6003610636 — 2026-10-05T21:43:50Z — Casey-allard

[uor-pod] up j5szkgnhoqh77f (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/main hours=3 — bf16 Phase 2b gate: canonical volume + 2x5090 [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6003621942 — 2026-10-05T21:44:24Z — Casey-allard

[uor-pod] up-failed u28c4pimn5jbnu by=claude/diag: bootstrap failed (pod deleted)

### comment 6003629403 — 2026-10-05T21:44:46Z — Casey-allard

[uor-pod] up e4wepjlhwhjat7 (1 x 5090, EUR-NO-1, $0.99/h) by=claude/diag hours=2 — binding diagnostic (#820)

### comment 6003657488 — 2026-10-05T21:46:10Z — Casey-allard

[uor-pod] release ic5zsemlme7moh by=deepseek/main

### comment 6003663194 — 2026-10-05T21:46:27Z — Casey-allard

[uor-pod] up ho5anxwb6i80po (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-composition hours=1.5 — Recover selected native geometric chain and prepare prospective512/128 composition continuation; PR1762 [https://github.com/UOR-Foundation/uor-r4/pull/1762]

### comment 6003706916 — 2026-10-05T21:48:38Z — Casey-allard

[uor-pod] release ic5zsemlme7moh by=codex/cue-composition

### comment 6003768286 — 2026-10-05T21:51:52Z — Casey-allard

[uor-pod] release e4wepjlhwhjat7 by=claude/diag

### comment 6003769896 — 2026-10-05T21:51:58Z — Casey-allard

[uor-pod] down e4wepjlhwhjat7 by=claude/diag (was $0.99/h, up 7 min)

### comment 6004257718 — 2026-10-05T22:16:58Z — Casey-allard

[uor-pod] up 5wihsa1og11z5s (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6004302471 — 2026-10-05T22:19:36Z — Casey-allard

[uor-pod] up-failed 5wihsa1og11z5s by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6004308701 — 2026-10-05T22:19:57Z — Casey-allard

[uor-pod] lease ho5anxwb6i80po gpus=0 by=codex/cue-composition hours=0.5 — Prospective native geometric cue composition fit CPU16/RAM12GiB; spareGPU1 free [https://github.com/UOR-Foundation/uor-r4/pull/1762]

### comment 6004624137 — 2026-10-05T22:39:05Z — Casey-allard

[uor-pod] release j5szkgnhoqh77f by=deepseek/main

### comment 6004625530 — 2026-10-05T22:39:11Z — Casey-allard

[uor-pod] release ho5anxwb6i80po by=codex/cue-composition

### comment 6004625718 — 2026-10-05T22:39:11Z — Casey-allard

[uor-pod] down j5szkgnhoqh77f by=deepseek/main (was $1.98/h, up 55 min)

### comment 6004626977 — 2026-10-05T22:39:16Z — Casey-allard

[uor-pod] down ho5anxwb6i80po by=codex/cue-composition (was $1.98/h, up 52 min)

### comment 6004655521 — 2026-10-05T22:41:32Z — Casey-allard

[uor-pod] up ibtup8lnvnsip5 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6004770007 — 2026-10-05T22:48:55Z — Casey-allard

[uor-pod] up-failed ibtup8lnvnsip5 by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6004775158 — 2026-10-05T22:49:16Z — Casey-allard

[uor-pod] up waiting: no 2 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=codex/cue-credit-audit retries every 5 min for up to 0.25 h — Frozen native geometric cue credit/conflict audit; no optimization; CPU native learner

### comment 6004896856 — 2026-10-05T22:57:03Z — Casey-allard

[uor-pod] up f1sbzygmzdehgb (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6004911608 — 2026-10-05T22:57:56Z — Casey-allard

[uor-pod] up-failed f1sbzygmzdehgb by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6004935070 — 2026-10-05T22:59:22Z — Casey-allard

[uor-pod] up m7ghzvnv6qyyw7 (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-credit-audit hours=1 — Frozen native geometric cue credit/conflict audit; no optimization; CPU native learner [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6004747189]

### comment 6005002808 — 2026-10-05T23:03:32Z — Casey-allard

[uor-pod] lease m7ghzvnv6qyyw7 gpus=0 by=codex/cue-credit-audit hours=0.5 — Frozen native cue audit/checks CPU8-16/RSS12GiB; spareGPU1 free [https://github.com/UOR-Foundation/uor-r4/pull/1766]

### comment 6005251461 — 2026-10-05T23:19:00Z — Casey-allard

[uor-pod] release m7ghzvnv6qyyw7 by=codex/cue-credit-audit

### comment 6005253146 — 2026-10-05T23:19:07Z — Casey-allard

[uor-pod] down m7ghzvnv6qyyw7 by=codex/cue-credit-audit (was $1.98/h, up 19 min)

### comment 6005311011 — 2026-10-05T23:22:38Z — Casey-allard

[uor-pod] up us9wp142uohh5a (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005325377 — 2026-10-05T23:23:30Z — Casey-allard

[uor-pod] up-failed us9wp142uohh5a by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005338724 — 2026-10-05T23:24:18Z — Casey-allard

[uor-pod] up e15kpwmkuv74c8 (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-support-probe hours=1 — Frozen trial8 support1and8 geometric proposals; CPU8build16native RSS12GiB 900s no new gradients [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6005357577 — 2026-10-05T23:25:27Z — Casey-allard

[uor-pod] lease e15kpwmkuv74c8 gpus=0 by=codex/cue-support-probe hours=0.5 — Frozen native support probe checksCPU8/measureCPU16; GPU1 free [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6005392462 — 2026-10-05T23:27:35Z — Casey-allard

[uor-pod] up bsk5r2yh2kmp1t (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005410394 — 2026-10-05T23:28:38Z — Casey-allard

[uor-pod] up-failed bsk5r2yh2kmp1t by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005478588 — 2026-10-05T23:32:47Z — Casey-allard

[uor-pod] up axchwy6x5b73mq (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005489964 — 2026-10-05T23:33:28Z — Casey-allard

[uor-pod] up-failed axchwy6x5b73mq by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005564485 — 2026-10-05T23:37:39Z — Casey-allard

[uor-pod] up l3k58srizncmf0 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005582700 — 2026-10-05T23:38:43Z — Casey-allard

[uor-pod] up-failed l3k58srizncmf0 by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005652060 — 2026-10-05T23:42:51Z — Casey-allard

[uor-pod] up dorzl5vxbh0lhz (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005669079 — 2026-10-05T23:43:53Z — Casey-allard

[uor-pod] up-failed dorzl5vxbh0lhz by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005772987 — 2026-10-05T23:49:42Z — Casey-allard

[uor-pod] release e15kpwmkuv74c8 by=codex/cue-support-probe

### comment 6005774856 — 2026-10-05T23:49:48Z — Casey-allard

[uor-pod] down e15kpwmkuv74c8 by=codex/cue-support-probe (was $1.98/h, up 25 min)

### comment 6005819715 — 2026-10-05T23:52:07Z — Casey-allard

[uor-pod] up un18eur1ttogi0 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005844172 — 2026-10-05T23:53:20Z — Casey-allard

[uor-pod] up-failed un18eur1ttogi0 by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005922315 — 2026-10-05T23:57:25Z — Casey-allard

[uor-pod] up zfl8zs33n9y14x (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6005933388 — 2026-10-05T23:58:07Z — Casey-allard

[uor-pod] up-failed zfl8zs33n9y14x by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6005975650 — 2026-10-06T00:00:41Z — Casey-allard

[uor-pod] up 5gw693qtnxsflx (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-joint-foundation hours=1 — ordered cue pair zero replay and native gradient foundation, no fit [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6005856726]

### comment 6006002011 — 2026-10-06T00:02:12Z — Casey-allard

[uor-pod] up sriu7l5pdwcwyo (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6006018186 — 2026-10-06T00:03:08Z — Casey-allard

[uor-pod] up-failed sriu7l5pdwcwyo by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6006090200 — 2026-10-06T00:07:16Z — Casey-allard

[uor-pod] up b8kgonrcmwcn2l (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6006109682 — 2026-10-06T00:08:21Z — Casey-allard

[uor-pod] up-failed b8kgonrcmwcn2l by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6006184801 — 2026-10-06T00:12:30Z — Casey-allard

[uor-pod] up js5em6gwin1ytp (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6006200386 — 2026-10-06T00:13:20Z — Casey-allard

[uor-pod] up-failed js5em6gwin1ytp by=claude/gate7c: bootstrap failed (pod deleted)

### comment 6006344694 — 2026-10-06T00:21:14Z — Casey-allard

[uor-pod] up y2puhyt947w8s0 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/gate7c hours=6 — Step 7c copy-gate supervision: train G-s1/G-s2 + eval (References #820) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6003816899]

### comment 6006454901 — 2026-10-06T00:26:57Z — Casey-allard

[uor-pod] release 5gw693qtnxsflx by=codex/cue-joint-foundation

### comment 6006457308 — 2026-10-06T00:27:06Z — Casey-allard

[uor-pod] down 5gw693qtnxsflx by=codex/cue-joint-foundation (was $1.98/h, up 25 min)

### comment 6006833355 — 2026-10-06T00:47:49Z — Casey-allard

[uor-pod] up gs3p90mu2856wk (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-joint-learning hours=1 — Fixed-pair native geometric joint fit and complete-generation selection [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6007343980 — 2026-10-06T01:19:13Z — Casey-allard

[uor-pod] release y2puhyt947w8s0 by=claude/gate7c

### comment 6007346924 — 2026-10-06T01:19:27Z — Casey-allard

[uor-pod] down y2puhyt947w8s0 by=claude/gate7c (was $1.98/h, up 57 min)

### comment 6007351408 — 2026-10-06T01:19:48Z — Casey-allard

[uor-pod] release gs3p90mu2856wk by=codex/cue-joint-learning

### comment 6007355036 — 2026-10-06T01:20:05Z — Casey-allard

[uor-pod] down gs3p90mu2856wk by=codex/cue-joint-learning (was $1.98/h, up 32 min)

### comment 6007388289 — 2026-10-06T01:22:58Z — Casey-allard

[uor-pod] up 4ipbmo9lfusx3w (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-joint-transfer hours=1 — Frozen geometric joint transfer and excluded fresh-only128 preparation [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6007569272 — 2026-10-06T01:40:05Z — Casey-allard

[uor-pod] up j0j60vwwrziicc (1 x 5090, EUR-NO-1, $0.99/h) by=claude/key7a hours=1 — Step 7a Phase 1: binding-probe key_probe on C-s1/C-s2/G-s1 v4 (read-key content) [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6007655984 — 2026-10-06T01:48:23Z — Casey-allard

[uor-pod] release j0j60vwwrziicc by=claude/key7a

### comment 6007657071 — 2026-10-06T01:48:30Z — Casey-allard

[uor-pod] down j0j60vwwrziicc by=claude/key7a (was $0.99/h, up 7 min)

### comment 6007744636 — 2026-10-06T01:56:43Z — Casey-allard

[uor-pod] release 4ipbmo9lfusx3w by=codex/cue-joint-transfer

### comment 6007749075 — 2026-10-06T01:57:09Z — Casey-allard

[uor-pod] down 4ipbmo9lfusx3w by=codex/cue-joint-transfer (was $1.98/h, up 33 min)

### comment 6007794814 — 2026-10-06T02:01:38Z — Casey-allard

[uor-pod] up k73g3pobaeq1ol (2 x 5090, EUR-NO-1, $1.98/h) by=codex/cue-coadapt-probe hours=0.5 — Compile and measure four-arm native geometric unary/joint sensitivity; Rust integer path CPU, CUDA hidden [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6007914219 — 2026-10-06T02:11:28Z — Casey-allard

[uor-pod] up er31p1fw3bc5k0 (2 x 5090, EUR-IS-1 volume v65f9luxbb (non-canonical), $1.98/h) by=claude/key7a hours=2 — Step 7a Phase 3: tests/parity + mqar-bench pair pilot (none/f2/conv8/carrier x3) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6007700083]

### comment 6008018200 — 2026-10-06T02:21:18Z — Casey-allard

[uor-pod] release k73g3pobaeq1ol by=codex/cue-coadapt-probe

### comment 6008023923 — 2026-10-06T02:21:51Z — Casey-allard

[uor-pod] down k73g3pobaeq1ol by=codex/cue-coadapt-probe (was $1.98/h, up 19 min)

### comment 6008120714 — 2026-10-06T02:31:04Z — Casey-allard

[uor-pod] up 2fs7p8j22lbjnw (2 x 5090, EUR-NO-1, $1.98/h) by=codex/source-bound-actions hours=1 — Compile and measure native geometric source-bound typed actions; no training normalizer changes [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6008093624]

### comment 6008243694 — 2026-10-06T02:42:27Z — Casey-allard

[uor-pod] release er31p1fw3bc5k0 by=claude/key7a

### comment 6008244831 — 2026-10-06T02:42:33Z — Casey-allard

[uor-pod] down er31p1fw3bc5k0 by=claude/key7a (was $1.98/h, up 30 min)

### comment 6008246445 — 2026-10-06T02:42:42Z — Casey-allard

[uor-pod] up 4162vzl24cotui (2 x 5090, EUR-IS-1 volume v65f9luxbb (non-canonical), $1.98/h) by=claude/key7a hours=3 — Step 7a 96M A/B: F2 and conv8 fine-tunes (2 seeds) + v4/D19/open eval [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6008241419]

### comment 6008463345 — 2026-10-06T03:02:46Z — Casey-allard

[uor-pod] release 2fs7p8j22lbjnw by=codex/source-bound-actions

### comment 6008466359 — 2026-10-06T03:03:04Z — Casey-allard

[uor-pod] down 2fs7p8j22lbjnw by=codex/source-bound-actions (was $1.98/h, up 31 min)

### comment 6008526479 — 2026-10-06T03:08:50Z — Casey-allard

[uor-pod] release 4162vzl24cotui by=claude/key7a

### comment 6008527463 — 2026-10-06T03:08:56Z — Casey-allard

[uor-pod] down 4162vzl24cotui by=claude/key7a (was $1.98/h, up 25 min)

### comment 6008566951 — 2026-10-06T03:12:32Z — Casey-allard

[uor-pod] up hlch9k5hnvjb1b (2 x 5090, EUR-NO-1, $1.98/h) by=claude/key7a hours=3 — Step 7a 96M A/B: F2 and conv8 fine-tunes (2 seeds) + v4/D19/open eval [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6008241419]

### comment 6008645467 — 2026-10-06T03:19:40Z — Casey-allard

[uor-pod] up 6waqxs8dbpj0dt (2 x 5090, EUR-NO-1, $1.98/h) by=codex/composed-state-credit hours=0.75 — compile and measure predictive credit into learned geometric states; native CPU instrument, CUDA training seam unchanged [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6008502048]

### comment 6008807781 — 2026-10-06T03:35:52Z — Casey-allard

[uor-pod] release 6waqxs8dbpj0dt by=codex/composed-state-credit

### comment 6008809863 — 2026-10-06T03:36:05Z — Casey-allard

[uor-pod] down 6waqxs8dbpj0dt by=codex/composed-state-credit (was $1.98/h, up 15 min)

### comment 6008987607 — 2026-10-06T03:53:41Z — Casey-allard

[uor-pod] up waiting: no 2 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=codex/native-cuda-learning retries every 5 min for up to 2 h — Execute native geometric CUDA forward/backward parity and real composed prediction credit; source893b1c31 ready; no fit

### comment 6009015384 — 2026-10-06T03:56:27Z — Casey-allard

[uor-pod] up hae7aux68l9e78 (2 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $1.48/h) by=codex/native-cuda-learning hours=1 — Deliberate4090 native CUDA bringup/parity source893b1c31;5090createfailed; nofit or5090performanceclaim [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6008867362]

### comment 6009204233 — 2026-10-06T04:14:59Z — Casey-allard

[uor-pod] release hae7aux68l9e78 by=codex/native-cuda-learning

### comment 6009270287 — 2026-10-06T04:21:22Z — Casey-allard

[uor-pod] down hae7aux68l9e78 by=codex/native-cuda-learning (was $1.48/h, up 24 min)

### comment 6009463665 — 2026-10-06T04:40:26Z — Casey-allard

[uor-pod] release hlch9k5hnvjb1b by=claude/key7a

### comment 6009465925 — 2026-10-06T04:40:39Z — Casey-allard

[uor-pod] down hlch9k5hnvjb1b by=claude/key7a (was $1.98/h, up 87 min)

### comment 6009549120 — 2026-10-06T04:48:52Z — Casey-allard

[uor-pod] up f7j902burib3vq (2 x 5090, EUR-NO-1, $1.98/h) by=codex/native-geometric-generate hours=1 — Native geometric Generate CUDA optimizer/export/reload and cached full120 credit checks [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6009363688]

### comment 6009568644 — 2026-10-06T04:50:49Z — Casey-allard

[uor-pod] up-failed f7j902burib3vq by=codex/native-geometric-generate: bootstrap failed — warning: build failed, waiting for other jobs to finish... (pod deleted)

### comment 6009579393 — 2026-10-06T04:51:52Z — Casey-allard

[uor-pod] up d68osfkekgamor (2 x 5090, EUR-NO-1, $1.98/h) by=codex/native-geometric-generate hours=1 — Native Generate focused build and actual CUDA update; exact corrected source, shared Codex target [https://github.com/UOR-Foundation/uor-r4/pull/1792]

### comment 6009917886 — 2026-10-06T05:22:19Z — Casey-allard

[uor-pod] up waiting: no 2 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/read7d retries every 5 min for up to 3 h — Step 7d read-binding supervision (#820 issuecomment-6009883286)

### comment 6011240994 — 2026-10-06T07:08:18Z — Casey-allard

[uor-pod] up tk94eo5mrmqfj8 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/read7d hours=2 — Step 7d read-binding supervision (#820 issuecomment-6009883286) [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6009883286]

### comment 6011936540 — 2026-10-06T07:56:26Z — Casey-allard

[uor-pod] release tk94eo5mrmqfj8 by=claude/read7d

### comment 6011939854 — 2026-10-06T07:56:40Z — Casey-allard

[uor-pod] down tk94eo5mrmqfj8 by=claude/read7d (was $1.98/h, up 47 min)

### comment 6013378948 — 2026-10-06T09:29:20Z — Casey-allard

[uor-pod] lease d68osfkekgamor gpus=0 by=codex/native-geometric-generate hours=1 — Balanced output-only seed1001 running; legacy fits finished, GPU1 released for other labs [https://github.com/UOR-Foundation/uor-r4/pull/1792]

### comment 6014221791 — 2026-10-06T10:22:10Z — Casey-allard

[uor-pod] release d68osfkekgamor by=codex/native-geometric-generate

### comment 6014592399 — 2026-10-06T10:45:54Z — Casey-allard

[uor-pod] down d68osfkekgamor by=codex/native-geometric-generate (was $1.98/h, up 353 min)

### comment 6014813917 — 2026-10-06T11:00:35Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1; by=codex/native-geometric-generate retries every 5 min for up to 0.5 h — Bounded zero-update native field phase-adjoint audit; exact parent on canonical volume

### comment 6014841362 — 2026-10-06T11:02:25Z — Casey-allard

[uor-pod] up lhfof249g575kt (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-geometric-generate hours=0.75 — Zero-update native field phase-adjoint audit; exact canonical parent [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6014807054]

### comment 6015192202 — 2026-10-06T11:22:36Z — Casey-allard

[uor-pod] release lhfof249g575kt by=codex/native-geometric-generate

### comment 6015193873 — 2026-10-06T11:22:43Z — Casey-allard

[uor-pod] down lhfof249g575kt by=codex/native-geometric-generate (was $0.99/h, up 20 min)

### comment 6015295595 — 2026-10-06T11:29:21Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1; by=codex/native-geometric-generate retries every 5 min for up to 0.333333 h — Balanced joint zero-update first-context credit admission; same initial donor

### comment 6015449239 — 2026-10-06T11:39:28Z — Casey-allard

[uor-pod] up rezfseimkb2xqv (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-geometric-generate hours=0.333333 — Balanced joint zero-update first-context credit admission; same initial donor [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6015220952]

### comment 6015699633 — 2026-10-06T11:55:43Z — Casey-allard

[uor-pod] lease rezfseimkb2xqv gpus=0 by=codex/native-geometric-generate hours=1.5610932380555556 — Matched balanced joint native geometric fit seed1001, same initial donor128updates [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6015654200]

### comment 6016706948 — 2026-10-06T12:54:38Z — Casey-allard

[uor-pod] release rezfseimkb2xqv by=codex/native-geometric-generate

### comment 6016708898 — 2026-10-06T12:54:45Z — Casey-allard

[uor-pod] down rezfseimkb2xqv by=codex/native-geometric-generate (was $0.99/h, up 75 min)

### comment 6016945872 — 2026-10-06T13:07:41Z — Casey-allard

[uor-pod] up lj16su2obv14uj (1 x 5090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=codex/native-geometric-generate hours=0.5 — Zero-update native H4 utility ordering: exact-source build and CUDA backward vs960native pools [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6016843192]

### comment 6017029279 — 2026-10-06T13:12:25Z — Casey-allard

[uor-pod] up waiting: no 2 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/cap8 retries every 5 min for up to 0.15 h — Step 8 300M capacity test (P rbs=0.1 vs Q off), source 7c682fdb, #820

### comment 6017146685 — 2026-10-06T13:19:01Z — Casey-allard

[uor-pod] release lj16su2obv14uj by=codex/native-geometric-generate

### comment 6017148565 — 2026-10-06T13:19:07Z — Casey-allard

[uor-pod] down lj16su2obv14uj by=codex/native-geometric-generate (was $0.99/h, up 10 min)

### comment 6017223226 — 2026-10-06T13:23:10Z — Casey-allard

[uor-pod] up waiting: no 2 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/cap8 retries every 5 min for up to 2 h — Step 8 300M capacity test (#820)

### comment 6017483532 — 2026-10-06T13:37:28Z — Casey-allard

[uor-pod] up vxf2pvetpo2k69 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-geometric-generate hours=0.5 — Retrieve exact joint checkpoint and run finite H4 state utility audit [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6016843192]

### comment 6017615252 — 2026-10-06T13:44:40Z — Casey-allard

[uor-pod] release vxf2pvetpo2k69 by=codex/native-geometric-generate

### comment 6017617470 — 2026-10-06T13:44:47Z — Casey-allard

[uor-pod] down vxf2pvetpo2k69 by=codex/native-geometric-generate (was $0.99/h, up 6 min)

### comment 6017735112 — 2026-10-06T13:51:09Z — Casey-allard

[uor-pod] up ibneiny0ukkdeg (1 x 5090, EUR-NO-1, $0.99/h) by=claude/cap8p hours=20 — Step 8 300M capacity test arm p (#820)

### comment 6017808325 — 2026-10-06T13:55:09Z — Casey-allard

[uor-pod] up ianty3o30vhgew (1 x 5090, EUR-NO-1, $0.99/h) by=claude/cap8q hours=20 — Step 8 300M capacity test arm q (#820)

### comment 6017968264 — 2026-10-06T14:03:22Z — Casey-allard

[uor-pod] up xyh8m9ao3nlz6l (1 x 5090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=codex/native-geometric-generate hours=0.5 — Compile and verify native Copy potential CUDA coefficient parity [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6017832275]

### comment 6018217411 — 2026-10-06T14:15:42Z — Casey-allard

[uor-pod] release xyh8m9ao3nlz6l by=codex/native-geometric-generate

### comment 6018222098 — 2026-10-06T14:15:56Z — Casey-allard

[uor-pod] down xyh8m9ao3nlz6l by=codex/native-geometric-generate (was $0.99/h, up 11 min)

### comment 6018427720 — 2026-10-06T14:26:34Z — Casey-allard

[uor-pod] up tnsy3jb6wm4wa7 (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/native-geometric-generate hours=0.5 — Verify joint geometric potential integration and real H2L4 CUDA credit, zero updates [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6018261575]

### comment 6019240385 — 2026-10-06T15:09:25Z — Casey-allard

[uor-pod] release tnsy3jb6wm4wa7 by=codex/native-geometric-generate

### comment 6019244388 — 2026-10-06T15:09:37Z — Casey-allard

[uor-pod] down tnsy3jb6wm4wa7 by=codex/native-geometric-generate (was $0.74/h, up 42 min)

### comment 6019386449 — 2026-10-06T15:16:42Z — Casey-allard

[uor-pod] up 3rvghnscxzfbvn (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-geometric-generate hours=3 — joint geometric potential128 fit and matched native attribution [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6020419287 — 2026-10-06T16:10:21Z — Casey-allard

[uor-pod] release 3rvghnscxzfbvn by=codex/native-geometric-generate

### comment 6020426096 — 2026-10-06T16:10:43Z — Casey-allard

[uor-pod] down 3rvghnscxzfbvn by=codex/native-geometric-generate (was $0.99/h, up 53 min)

### comment 6020430752 — 2026-10-06T16:10:59Z — Casey-allard

[uor-pod] up 5j908ccd585p4s (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-geometric-generate hours=4 — recover host failure; joint geometric potential128 fit and four native controls [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6020966815 — 2026-10-06T16:42:00Z — Casey-allard

[uor-pod] up t6nrnx1jdy4xwz (1 x 5090, EUR-NO-1, $0.99/h) by=claude/step9 hours=6 — Step 9 phase binding fine-tunes (#820 issuecomment-6020956939)

### comment 6022271932 — 2026-10-06T17:57:51Z — Casey-allard

[uor-pod] up 90urpspe50zr8u (1 x 5090, EUR-NO-1, $0.99/h) by=claude/flashread hours=10 — bf16 Phase 2b flash read: build + parity + 29M A/B (owner approved 5th pod 10-06 18:00Z, #820 issuecomment-6022153955)

### comment 6022321867 — 2026-10-06T18:00:49Z — Casey-allard

**Capacity report — 5 pods against a cap of 4, plus 3 phantom leases counted LIVE (2026-10-06 ~18:0x ET)**

Posting here as the Compute board; also noted on #820.

## Cap exceeded

```
90urpspe50zr8u uor-claude-1006-1757   RUNNING  1x5090  up 0h     <-- started within the hour
t6nrnx1jdy4xwz uor-claude-1006-1641   RUNNING  1x5090  up 1.2h
5j908ccd585p4s uor-codex-1006-1610    RUNNING  1x5090  up 1.7h
ianty3o30vhgew uor-claude-1006-1355   RUNNING  1x5090  up 4h
ibneiny0ukkdeg uor-claude-1006-1351   RUNNING  1x5090  up 4.1h

Running: 5 pod(s) (5 counted toward the cap of 4), $4.95/h of $8.00/h
Free GPUs you can lease now: none
```

**Five counted toward a cap of four.** Dollar cap fine ($4.95 of $8.00); the pod count is the breach. All five are other sessions' pods, so this is a report, not a claim.

## Three phantom leases still counted LIVE

`uor-pod status` warns `MALFORMED LEASE FILE(S) — each is treated as LIVE (fail safe), so its GPUs stay blocked until it is fixed.` They are **empty directories** from 5 Oct in `~/.local/share/uor-r4/compute/leases/`:

```
9yexc6nhxcnl3h/   EMPTY  (Oct 5 16:28)
ic5zsemlme7moh/   EMPTY  (Oct 5 17:48)
y3arg3r4j5xmoy/   EMPTY  (Oct 5 16:21)   <- uor-step6, EXITED 22h, still billing container disk
```

Each has no lease JSON inside, and none of those pods appears in the current listing. **Hypothesis, unverified**: if admission derives its count from these directories, three phantoms could mask a genuine overrun. I have not read that code path and am not asserting it as the cause.

## Cleanup available, gated

```
$ uor-pod prune-stopped
y3arg3r4j5xmoy uor-step6  EXITED  stopped Oct 05 2026 20:43:05 (21 h)
Listed only. Confirm nothing unique is on them (owner approval for pods with data), then re-run with --yes.
```

The tool asks for **owner approval because `uor-step6` has a volume** — I have not run `--yes`. The three empty lease dirs sit behind *"Never touch another session's lease or KEEP_ALIVE"*; the sanctioned fix for a bad lease is its holder running `uor-pod renew POD --lab L --session S`, which rewrites the canonical form.

**So: clearing the phantoms needs the owner's go-ahead on `uor-step6` plus whoever holds the empty dirs renewing or releasing.** Both cheap; neither mine. Net capacity today is **5 real pods + 3 phantom leases against a cap of 4**.


### comment 6023624005 — 2026-10-06T19:13:39Z — Casey-allard

[uor-pod] release 5j908ccd585p4s by=codex/native-geometric-generate

### comment 6023724712 — 2026-10-06T19:18:57Z — Casey-allard

[uor-pod] down 5j908ccd585p4s by=codex/native-geometric-generate (was $0.99/h, up 187 min)

### comment 6024709715 — 2026-10-06T20:20:09Z — Casey-allard

[uor-pod] up saxbn10ji35im0 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/read-state-admission hours=1 — Owner-approved fifth pod: a5e132b3 CUDA parity and zero-update actual-bank geometric read-state admission; retain total <=8 USD/h [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6024755595 — 2026-10-06T20:22:57Z — Casey-allard

[uor-pod] up ehnd5g69ib9ujy (1 x 5090, EUR-NO-1, $0.99/h) by=claude/cap8p2 hours=14 — Step 8 arm P resumed on a new pod (old pod ibneiny0ukkdeg: /workspace FUSE hung) (#820)

### comment 6024758627 — 2026-10-06T20:23:09Z — Casey-allard

[uor-pod] release ibneiny0ukkdeg by=claude/cap8p

### comment 6024793103 — 2026-10-06T20:25:14Z — Casey-allard

[uor-pod] release 90urpspe50zr8u by=claude/flashread

### comment 6024794786 — 2026-10-06T20:25:20Z — Casey-allard

[uor-pod] down 90urpspe50zr8u by=claude/flashread (was $0.99/h, up 146 min)

### comment 6024852685 — 2026-10-06T20:28:53Z — Casey-allard

[uor-pod] release saxbn10ji35im0 by=codex/read-state-admission

### comment 6024863840 — 2026-10-06T20:29:34Z — Casey-allard

[uor-pod] down saxbn10ji35im0 by=codex/read-state-admission (was $0.99/h, up 9 min)

### comment 6024998855 — 2026-10-06T20:37:47Z — Casey-allard

[uor-pod] up cyubn82zm5hf6z (1 x 5090, EUR-NO-1, $0.99/h) by=codex/read-state-joint-fit hours=3 — Owner-approved temporary fifth5090: matched128 joint geometric read-state fit from feasible parent, codea5; total<=8USD/h; #820 card6024955480 [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6025238428 — 2026-10-06T20:52:49Z — Casey-allard

[uor-pod] up-failed cyubn82zm5hf6z by=codex/read-state-joint-fit: SSH not reachable after 900 s (pod deleted)

### comment 6025260777 — 2026-10-06T20:54:14Z — Casey-allard

[uor-pod] up oa6iydatmy7da3 (1 x 5090, EUR-IS-1 volume v65f9luxbb (non-canonical), $0.99/h) by=codex/read-state-joint-fit hours=2.7 — Owner-approved matched bridgefit fifth5090 replacement after cyubn82zm5hf6z failed beforecontainer and deleted; one retry, verify deploylogs immediately; total<=8USD/h; card6024955480 [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6025325624 — 2026-10-06T20:58:23Z — Casey-allard

[uor-pod] lease oa6iydatmy7da3 gpus=0 by=codex/read-state-joint-fit hours=2.7 — Owner-approved matched geometric bridge fit; replacement setup and CUDA verification [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6025880589 — 2026-10-06T21:34:44Z — Casey-allard

[uor-pod] release t6nrnx1jdy4xwz by=claude/step9

### comment 6025881951 — 2026-10-06T21:34:50Z — Casey-allard

[uor-pod] down t6nrnx1jdy4xwz by=claude/step9 (was $0.99/h, up 291 min)

### comment 6025997501 — 2026-10-06T21:43:02Z — Casey-allard

[uor-pod] release ehnd5g69ib9ujy by=claude/cap8p2

### comment 6025999105 — 2026-10-06T21:43:08Z — Casey-allard

[uor-pod] down ehnd5g69ib9ujy by=claude/cap8p2 (was $0.99/h, up 79 min)

### comment 6026004243 — 2026-10-06T21:43:30Z — Casey-allard

[uor-pod] up clcnfe4dycm9ab (1 x 5090, EUR-NO-1, $0.99/h) by=claude/cap8p3 hours=14 — Step 8 arm P resumed (pod ehnd5g69 lost its GPU) (#820)

### comment 6026276745 — 2026-10-06T22:03:13Z — Casey-allard

**Capacity update: cap no longer breached (3/4), and one pod reports `vol=non-canonical` (2026-10-06 22:0x ET)**

Follow-up to the capacity report above. The overrun cleared itself and there is a new warning worth its owner's attention.

## Resolved on its own

```
Running: 3 pod(s) (3 counted toward the cap of 4)   $2.97/h of $8.00/h
Free GPUs: none. Spin up your own pod: uor-pod up --lab L --session S --purpose ... --hours H.
```

Earlier today the status line read **5 counted toward a cap of 4**. It now reads **3 of 4** — leases expired and pods came down normally. **The `prune-stopped --yes` on `uor-step6` was never needed**, and the caution about not passing it on another session's volume cost nothing. The malformed-lease warning still appears; the three empty directories from 5 Oct are unchanged and still counted LIVE.

The tool now advertises spinning up a pod rather than telling callers to wait, so **capacity is available** for whoever needs it.

## New: a pod on a non-canonical volume

```
clcnfe4dycm9ab uor-claude-1006-2143   RUNNING  up 0.3h  vol=yes
oa6iydatmy7da3 uor-codex-1006-2054    RUNNING  up 1.1h  vol=non-canonical   <-- this
ianty3o30vhgew uor-claude-1006-1355   RUNNING  up 8.1h  vol=yes
```

`docs/labs/compute.md` rule 6 is *"Durable output only on `/workspace` (network volume). No stopped pods as storage."* **A pod on a non-canonical volume has non-durable output** — anything written outside `/workspace` is lost when the pod goes down, and this one has been up 1.1 hours.

**Worth its owner checking before the pod ends**, not because anything is known to be wrong, but because the failure is silent and total when it happens: the run succeeds, the pod stops, and the artifacts are gone. Same shape as the branch-preservation pattern found earlier today — work exists, is not where it can be retrieved from.

Flagging only; it is Codex's pod and I have not touched it.


### comment 6026581495 — 2026-10-06T22:26:06Z — Casey-allard

[uor-pod] release oa6iydatmy7da3 by=codex/read-state-joint-fit

### comment 6026582948 — 2026-10-06T22:26:13Z — Casey-allard

[uor-pod] down oa6iydatmy7da3 by=codex/read-state-joint-fit (was $0.99/h, up 91 min)

### comment 6026735306 — 2026-10-06T22:38:11Z — Casey-allard

[uor-pod] up 14ungsdu40idva (2 x 5090, EUR-NO-1, $1.98/h) by=claude/step10 hours=8 — Step 10 constant-lr memory fine-tunes (#820 issuecomment-6026732652)

### comment 6026919057 — 2026-10-06T22:53:14Z — Casey-allard

[uor-pod] up-failed 14ungsdu40idva by=claude/step10: SSH not reachable after 900 s (pod deleted)

### comment 6026938409 — 2026-10-06T22:54:49Z — Casey-allard

[uor-pod] up p9fkbt1w00a7jk (2 x 5090, EUR-NO-1, $1.98/h) by=claude/step10 hours=8 — Step 10 constant-lr memory fine-tunes (#820 issuecomment-6026732652)

### comment 6026950134 — 2026-10-06T22:55:49Z — Casey-allard

[uor-pod] up 9ebiv21x4bo162 (1 x 5090, EUR-NO-1, $0.99/h) by=claude/p1757 hours=1 — CUDA parity for #1757 merge head

### comment 6026985437 — 2026-10-06T22:58:48Z — Casey-allard

[uor-pod] up 3nmmheuiz4a2de (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2bgate hours=6 — bf16 Phase 2b gate (#1809)

### comment 6026996050 — 2026-10-06T22:59:43Z — Casey-allard

[uor-pod] release 9ebiv21x4bo162 by=claude/p1757

### comment 6026997042 — 2026-10-06T22:59:49Z — Casey-allard

[uor-pod] down 9ebiv21x4bo162 by=claude/p1757 (was $0.99/h, up 3 min)

### comment 6027156377 — 2026-10-06T23:13:22Z — Casey-allard

[uor-pod] lease 3nmmheuiz4a2de gpus=0,1 by=deepseek/p2bgate hours=6 — bf16 Phase 2b gate (#1809): 4x 70609-step NLL runs + speed A/B

### comment 6027176671 — 2026-10-06T23:15:05Z — Casey-allard

[uor-pod] up p0zi15px4cf0u7 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/reconcile-cuda hours=2 — Owner-approved fifth pod: exact-head PR1812 and integration CUDA reconciliation checks [https://github.com/UOR-Foundation/uor-r4/pull/1812]

### comment 6027206631 — 2026-10-06T23:17:39Z — Casey-allard

[uor-pod] release p0zi15px4cf0u7 by=codex/reconcile-cuda

### comment 6027207722 — 2026-10-06T23:17:45Z — Casey-allard

[uor-pod] down p0zi15px4cf0u7 by=codex/reconcile-cuda (was $0.99/h, up 2 min)

### comment 6027244334 — 2026-10-06T23:20:57Z — Casey-allard

[uor-pod] down 3nmmheuiz4a2de by=deepseek/p2bgate (was $1.98/h, up 0 min)

### comment 6027250664 — 2026-10-06T23:21:31Z — Casey-allard

[uor-pod] up 3sjrzepxx9emlu (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2bgate2 hours=6 — bf16 Phase 2b gate (#1809)

### comment 6027382370 — 2026-10-06T23:33:01Z — Casey-allard

[uor-pod] down 3sjrzepxx9emlu by=deepseek/p2bgate2 (was $1.98/h, up 0 min)

### comment 6027383557 — 2026-10-06T23:33:06Z — Casey-allard

[uor-pod] up biw4r4nbo8xydq (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2bg hours=6 — bf16 Phase 2b gate (#1809)

### comment 6027405888 — 2026-10-06T23:34:33Z — Casey-allard

[uor-pod] up opxho268ifvnq6 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/reconcile-cuda hours=2 — Owner-approved temporary fifth: PR1813 final nine CUDA component and bank integration checks [https://github.com/UOR-Foundation/uor-r4/pull/1813]

### comment 6027435871 — 2026-10-06T23:36:35Z — Casey-allard

[uor-pod] up-failed 3sjrzepxx9emlu by=deepseek/p2bgate2: SSH not reachable after 900 s (pod deleted)

### comment 6027515323 — 2026-10-06T23:42:07Z — Casey-allard

[uor-pod] down biw4r4nbo8xydq by=deepseek/p2bg (was $1.98/h, up 0 min)

### comment 6027545493 — 2026-10-06T23:44:12Z — Casey-allard

[uor-pod] release opxho268ifvnq6 by=codex/reconcile-cuda

### comment 6027549087 — 2026-10-06T23:44:25Z — Casey-allard

[uor-pod] down opxho268ifvnq6 by=codex/reconcile-cuda (was $0.99/h, up 0 min)

### comment 6027559310 — 2026-10-06T23:45:01Z — Casey-allard

[uor-pod] up v1l66tecmxq97u (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2boff hours=1 — off-volume start test

### comment 6027562656 — 2026-10-06T23:45:12Z — Casey-allard

[uor-pod] up 6nmb31zfk66c94 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/reconcile-cuda hours=2 — PR1813 final nine CUDA checks; replacement of failed pre-container allocation opxho268ifvnq6 [https://github.com/UOR-Foundation/uor-r4/pull/1813]

### comment 6027654155 — 2026-10-06T23:50:25Z — Casey-allard

[uor-pod] release 6nmb31zfk66c94 by=codex/reconcile-cuda

### comment 6027656370 — 2026-10-06T23:50:32Z — Casey-allard

[uor-pod] down 6nmb31zfk66c94 by=codex/reconcile-cuda (was $0.99/h, up 4 min)

### comment 6027672653 — 2026-10-06T23:51:48Z — Casey-allard

[uor-pod] release v1l66tecmxq97u by=deepseek/p2boff

### comment 6027673860 — 2026-10-06T23:51:54Z — Casey-allard

[uor-pod] lease v1l66tecmxq97u gpus=0,1 by=deepseek/p2bgate hours=8 — bf16 Phase 2b gate (#1809)

### comment 6027727827 — 2026-10-06T23:56:47Z — Casey-allard

[uor-pod] up azyb6y8dobilkq (1 x 5090, EUR-NO-1, $0.99/h) by=codex/reconcile-cuda hours=2 — Owner-approved reconciliation exception: final e5ce2471 CUDA checks after fixture repair; binary ready [https://github.com/UOR-Foundation/uor-r4/pull/1813]

### comment 6027769702 — 2026-10-07T00:00:24Z — Casey-allard

[uor-pod] release azyb6y8dobilkq by=codex/reconcile-cuda

### comment 6027770957 — 2026-10-07T00:00:30Z — Casey-allard

[uor-pod] down azyb6y8dobilkq by=codex/reconcile-cuda (was $0.99/h, up 3 min)

### comment 6027806597 — 2026-10-07T00:03:29Z — Casey-allard

[uor-pod] up wrcem8l58vzjvh (1 x 5090, EUR-NO-1, $0.99/h) by=codex/reconcile-cuda hours=2 — Owner-approved reconciliation checks: final e71d015d binary ready, nine CUDA tests including full-bank native/adjoint parity [https://github.com/UOR-Foundation/uor-r4/pull/1813]

### comment 6027829587 — 2026-10-07T00:05:28Z — Casey-allard

[uor-pod] release wrcem8l58vzjvh by=codex/reconcile-cuda

### comment 6027830972 — 2026-10-07T00:05:35Z — Casey-allard

[uor-pod] down wrcem8l58vzjvh by=codex/reconcile-cuda (was $0.99/h, up 1 min)

### comment 6027943711 — 2026-10-07T00:15:16Z — Casey-allard

[uor-pod] release p9fkbt1w00a7jk by=claude/step10

### comment 6027944938 — 2026-10-07T00:15:22Z — Casey-allard

[uor-pod] down p9fkbt1w00a7jk by=claude/step10 (was $1.98/h, up 80 min)

### comment 6028368316 — 2026-10-07T00:46:48Z — Casey-allard

[uor-pod] up sm9qg7djik10vh (1 x 5090, EUR-NO-1, $0.99/h) by=codex/entry-margin hours=0.5 — Zero-update actual checkpoint entry-credit diagnostic; prebuilt exact-head CUDA binary [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6028163279]

### comment 6028412632 — 2026-10-07T00:49:39Z — Casey-allard

[uor-pod] release sm9qg7djik10vh by=codex/entry-margin

### comment 6028414295 — 2026-10-07T00:49:46Z — Casey-allard

[uor-pod] down sm9qg7djik10vh by=codex/entry-margin (was $0.99/h, up 2 min)

### comment 6028716439 — 2026-10-07T01:11:04Z — Casey-allard

[uor-pod] up ywaeptdb5gtvg4 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/alias-credit hours=0.5 — Fixed eight-row zero-update raw/clipped geometric alias-credit comparison; prebuilt exact-head CUDA binary [https://github.com/UOR-Foundation/uor-r4/pull/1817]

### comment 6028884643 — 2026-10-07T01:26:06Z — Casey-allard

[uor-pod] up-failed ywaeptdb5gtvg4 by=codex/alias-credit: SSH not reachable after 900 s (pod deleted)

### comment 6028904716 — 2026-10-07T01:27:56Z — Casey-allard

[uor-pod] up wt3s72s1qwmlkn (1 x 5090, EUR-NO-1, $0.99/h) by=codex/alias-credit hours=0.5 — Fresh allocation after terminal SSH provisioning failure; exact-head zero-update geometric credit diagnostic ready [https://github.com/UOR-Foundation/uor-r4/pull/1817]

### comment 6028935251 — 2026-10-07T01:30:42Z — Casey-allard

[uor-pod] down wt3s72s1qwmlkn by=codex/alias-credit (was $0.99/h, up 2 min)

### comment 6029021158 — 2026-10-07T01:38:21Z — Casey-allard

[uor-pod] down v1l66tecmxq97u by=deepseek/p2bgate (was $1.98/h, up 112 min)

### comment 6029027322 — 2026-10-07T01:38:55Z — Casey-allard

[uor-pod] up yrrcxmap32udyq (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2br1 hours=8 — bf16 Phase 2b gate (#1809) rerun

### comment 6029338454 — 2026-10-07T02:04:57Z — Casey-allard

[uor-pod] up vqj6ghxyhnw4jx (1 x 5090, EUR-NO-1, $0.99/h) by=codex/source-interchange hours=1 — Exact-head zero-update geometric source-state interchange; release binary verified and ready [https://github.com/UOR-Foundation/uor-r4/pull/1819]

### comment 6029365498 — 2026-10-07T02:07:15Z — Casey-allard

[uor-pod] release vqj6ghxyhnw4jx by=codex/source-interchange

### comment 6029366941 — 2026-10-07T02:07:22Z — Casey-allard

[uor-pod] down vqj6ghxyhnw4jx by=codex/source-interchange (was $0.99/h, up 1 min)

### comment 6029706654 — 2026-10-07T02:35:58Z — Casey-allard

[uor-pod] up 0ackgvv9i5htnb (1 x 5090, EUR-NO-1, $0.99/h) by=codex/frozen-map-restart hours=8 — Exact-head admission and matched geometric context/potential/Generate restart; release CUDA binary ready [https://github.com/UOR-Foundation/uor-r4/pull/1822]

### comment 6029776660 — 2026-10-07T02:42:06Z — Casey-allard

[uor-pod] release 0ackgvv9i5htnb by=codex/frozen-map-restart

### comment 6029780264 — 2026-10-07T02:42:27Z — Casey-allard

[uor-pod] down 0ackgvv9i5htnb by=codex/frozen-map-restart (was $0.99/h, up 6 min)

### comment 6029862007 — 2026-10-07T02:49:07Z — Casey-allard

[uor-pod] up 3vq9bfh5rc7tzm (1 x 5090, EUR-NO-1, $0.99/h) by=codex/frozen-map-fit hours=8 — Three matched geometric learning pairs; exact release checks and zero-update baseline passed; CUDA binary ready [https://github.com/UOR-Foundation/uor-r4/pull/1822]

### comment 6030888600 — 2026-10-07T04:23:50Z — Casey-allard

[uor-pod] release ianty3o30vhgew by=claude/cap8q

### comment 6030889589 — 2026-10-07T04:23:55Z — Casey-allard

[uor-pod] down ianty3o30vhgew by=claude/cap8q (was $0.99/h, up 868 min)

### comment 6031008468 — 2026-10-07T04:35:24Z — Casey-allard

[uor-pod] up q9xa96a923vuy3 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/fixed-map-parity hours=1 — PR1824 exact-head CUDA finite-map backward checks; cached binary; no fit changes [https://github.com/UOR-Foundation/uor-r4/pull/1824]

### comment 6031021983 — 2026-10-07T04:36:42Z — Casey-allard

[uor-pod] release yrrcxmap32udyq by=deepseek/p2br1

### comment 6031022969 — 2026-10-07T04:36:47Z — Casey-allard

[uor-pod] down yrrcxmap32udyq by=deepseek/p2br1 (was $1.98/h, up 177 min)

### comment 6031094570 — 2026-10-07T04:43:50Z — Casey-allard

[uor-pod] release q9xa96a923vuy3 by=codex/fixed-map-parity

### comment 6031095532 — 2026-10-07T04:43:56Z — Casey-allard

[uor-pod] down q9xa96a923vuy3 by=codex/fixed-map-parity (was $0.99/h, up 8 min)

### comment 6031157221 — 2026-10-07T04:49:52Z — Casey-allard

[uor-pod] up vght8hmmq2fg14 (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/p2b96-1 hours=6 — Phase 2b report: 96M probe + nsys profile (#1809)

### comment 6031361416 — 2026-10-07T05:09:03Z — Casey-allard

[uor-pod] up bru4ndpq7x19vq (1 x 5090, EUR-NO-1, $0.99/h) by=codex/device-generate-parity hours=1 — PR1829 exact cached CUDA integer Generate parity and timing; binary ready; no CPU build [https://github.com/UOR-Foundation/uor-r4/pull/1829]

### comment 6031402426 — 2026-10-07T05:12:06Z — Casey-allard

[uor-pod] release bru4ndpq7x19vq by=codex/device-generate-parity

### comment 6031403609 — 2026-10-07T05:12:12Z — Casey-allard

[uor-pod] down bru4ndpq7x19vq by=codex/device-generate-parity (was $0.99/h, up 2 min)

### comment 6031522725 — 2026-10-07T05:20:56Z — Casey-allard

[uor-pod] up ha96ihhh63k1u9 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/device-generate-parity hours=1 — PR1829 exact cached CUDA integer Generate parity and timing; binary ready; no CPU build [https://github.com/UOR-Foundation/uor-r4/pull/1829]

### comment 6031523988 — 2026-10-07T05:21:02Z — Casey-allard

[uor-pod] release vght8hmmq2fg14 by=deepseek/p2b96-1

### comment 6031525072 — 2026-10-07T05:21:08Z — Casey-allard

[uor-pod] down vght8hmmq2fg14 by=deepseek/p2b96-1 (was $1.98/h, up 30 min)

### comment 6031547778 — 2026-10-07T05:22:57Z — Casey-allard

[uor-pod] release ha96ihhh63k1u9 by=codex/device-generate-parity

### comment 6031549008 — 2026-10-07T05:23:03Z — Casey-allard

[uor-pod] down ha96ihhh63k1u9 by=codex/device-generate-parity (was $0.99/h, up 1 min)

### comment 6031598265 — 2026-10-07T05:27:10Z — Casey-allard

[uor-pod] up h31gwt4hykunux (1 x 5090, EUR-NO-1, $0.99/h) by=codex/device-generate-parity hours=1 — PR1829 final ready-binary CUDA integer geometry, gradients and retained-artifact timing; five CPU checks PASS; no CPU build [https://github.com/UOR-Foundation/uor-r4/pull/1829]

### comment 6031601457 — 2026-10-07T05:27:27Z — Casey-allard

[uor-pod] up yytiy37snijlli (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/scope-1 hours=2 — 96M scope check: corpora availability on the canonical volume

### comment 6031625031 — 2026-10-07T05:29:27Z — Casey-allard

[uor-pod] release h31gwt4hykunux by=codex/device-generate-parity

### comment 6031626378 — 2026-10-07T05:29:33Z — Casey-allard

[uor-pod] down h31gwt4hykunux by=codex/device-generate-parity (was $0.99/h, up 1 min)

### comment 6031654938 — 2026-10-07T05:32:05Z — Casey-allard

[uor-pod] release yytiy37snijlli by=deepseek/scope-1

### comment 6031656038 — 2026-10-07T05:32:11Z — Casey-allard

[uor-pod] down yytiy37snijlli by=deepseek/scope-1 (was $1.98/h, up 4 min)

### comment 6031746895 — 2026-10-07T05:40:10Z — Casey-allard

[uor-pod] up zvfltaygb30wpz (2 x 5090, EUR-NO-1, $1.98/h) by=deepseek/run96-1 hours=8 — 96M rung: chat-v1 prep + 122070-step run

### comment 6032399627 — 2026-10-07T06:33:34Z — Casey-allard

[uor-pod] up zfx4wc8asn3j6v (1 x 5090, EUR-NO-1, $0.99/h) by=codex/selected-context-parity hours=1 — Exact-head CUDA selected contextual potential values and adjoints; prebuilt binary, no compile or fit [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6032008738]

### comment 6032425331 — 2026-10-07T06:35:36Z — Casey-allard

[uor-pod] release zfx4wc8asn3j6v by=codex/selected-context-parity

### comment 6032426675 — 2026-10-07T06:35:42Z — Casey-allard

[uor-pod] down zfx4wc8asn3j6v by=codex/selected-context-parity (was $0.99/h, up 1 min)

### comment 6032995911 — 2026-10-07T07:18:19Z — Casey-allard

[uor-pod] down clcnfe4dycm9ab by=claude/cap8p3 (was $0.99/h, up 574 min)

### comment 6033129907 — 2026-10-07T07:27:49Z — Casey-allard

[uor-pod] up 0movnn3z4v1uf8 (2 x 5090, EUR-NO-1, $1.98/h) by=claude/step12 hours=5 — Step 12 abstention data on 214M (#820)

### comment 6033181680 — 2026-10-07T07:31:28Z — Casey-allard

[uor-pod] up 27zpj6lwc921u2 (1 x 5090, EUR-NO-1, $0.99/h) by=codex/native-entry-route-capacity hours=1 — Exact b4bc1ac9 native geometric route-capacity CUDA replay; executable already built [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6032772848]

### comment 6033260489 — 2026-10-07T07:36:23Z — Casey-allard

[uor-pod] release 27zpj6lwc921u2 by=codex/native-entry-route-capacity

### comment 6033262729 — 2026-10-07T07:36:30Z — Casey-allard

[uor-pod] down 27zpj6lwc921u2 by=codex/native-entry-route-capacity (was $0.99/h, up 4 min)

### comment 6034201626 — 2026-10-07T08:36:48Z — Casey-allard

[uor-pod] release 3vq9bfh5rc7tzm by=codex/frozen-map-fit

### comment 6034203108 — 2026-10-07T08:36:54Z — Casey-allard

[uor-pod] down 3vq9bfh5rc7tzm by=codex/frozen-map-fit (was $0.99/h, up 347 min)

### comment 6034302108 — 2026-10-07T08:43:21Z — Casey-allard

[uor-pod] up pzqagods46rakn (1 x 5090, EUR-NO-1, $0.99/h) by=codex/trained-route-capacity hours=2 — All-six actual cp128 geometric entry capacity; CUDA binary ready; corrected virtual-memory wrapper, physical RSS receipts [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6033583569]

### comment 6034872438 — 2026-10-07T09:17:31Z — Casey-allard

[uor-pod] release 0movnn3z4v1uf8 by=claude/step12

### comment 6034874079 — 2026-10-07T09:17:37Z — Casey-allard

[uor-pod] down 0movnn3z4v1uf8 by=claude/step12 (was $1.98/h, up 109 min)

### comment 6035281285 — 2026-10-07T09:41:27Z — Casey-allard

[uor-pod] release pzqagods46rakn by=codex/trained-route-capacity

### comment 6035284868 — 2026-10-07T09:41:41Z — Casey-allard

[uor-pod] down pzqagods46rakn by=codex/trained-route-capacity (was $0.99/h, up 57 min)

### comment 6036053881 — 2026-10-07T10:30:28Z — Casey-allard

[uor-pod] up yazwadpt6lzg0b (1 x 5090, EUR-NO-1, $0.99/h) by=codex/categorical-native-direction hours=1 — Prepared CUDA joint geometric direction packet; 11 integer exports; no compilation [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6035503876]

### comment 6036113684 — 2026-10-07T10:34:23Z — Casey-allard

[uor-pod] release yazwadpt6lzg0b by=codex/categorical-native-direction

### comment 6036122787 — 2026-10-07T10:34:58Z — Casey-allard

[uor-pod] down yazwadpt6lzg0b by=codex/categorical-native-direction (was $0.99/h, up 4 min)

### comment 6036940913 — 2026-10-07T11:27:19Z — Casey-allard

[uor-pod] up 0j24j90eakf14v (1 x 5090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=codex/categorical-joint-fit hours=3 — PR1840 exact-head CUDA admissions then matched native categorical joint fit; binaries prebuilt [https://github.com/UOR-Foundation/uor-r4/pull/1840]

### comment 6040096956 — 2026-10-07T14:27:42Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/win retries every 1 min for up to 2 h — v4 through the grounded session, 214M (#820)

### comment 6040228375 — 2026-10-07T14:34:32Z — Casey-allard

[uor-pod] up axfd77jnf4gv1t (1 x 5090, EUR-NO-1, $0.99/h) by=claude/win hours=2 — v4 through the grounded session, 214M (#820)

### comment 6040445474 — 2026-10-07T14:45:51Z — Casey-allard

[uor-pod] release axfd77jnf4gv1t by=claude/win

### comment 6040447393 — 2026-10-07T14:45:57Z — Casey-allard

[uor-pod] down axfd77jnf4gv1t by=claude/win (was $0.99/h, up 10 min)

### comment 6040564169 — 2026-10-07T14:52:03Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/planb retries every 1 min for up to 3 h — Plan B: SmolTalk assistant fine-tune of 214M (#820)

### comment 6040674653 — 2026-10-07T14:57:54Z — Casey-allard

[uor-pod] release zvfltaygb30wpz by=deepseek/run96-1

### comment 6040676396 — 2026-10-07T14:58:00Z — Casey-allard

[uor-pod] down zvfltaygb30wpz by=deepseek/run96-1 (was $1.98/h, up 557 min)

### comment 6040692808 — 2026-10-07T14:58:55Z — Casey-allard

[uor-pod] up ivbx5z1tdahfde (1 x 5090, EUR-NO-1, $0.99/h) by=claude/planb hours=8 — Plan B: SmolTalk assistant fine-tune of 214M (#820)

### comment 6040853947 — 2026-10-07T15:06:50Z — Casey-allard

[uor-pod] up waiting: no 1 x 5090 in EUR-NO-1 EU-RO-1 EUR-IS-1; by=claude/plana retries every 1 min for up to 3 h — Plan A: continue-pretrain 214M on SmolLM-Corpus (owner-approved 10-07)

### comment 6041092393 — 2026-10-07T15:27:58Z — Casey-allard

[uor-pod] down yn9hqjghtrzwsa by=deepseek/entry-isolation (was $0.99/h, up 0 min)

### comment 6041101907 — 2026-10-07T15:28:29Z — Casey-allard

[uor-pod] up pmvhyr6hs4u1ui (1 x 5090, EUR-IS-1 volume v65f9luxbb (non-canonical), $0.99/h) by=deepseek/entry-isolation hours=3 — Entry-phase isolation (issue #820): matched control vs entry_only, retained parent, declared dose; fitter branch deepseek/entry-isolation

### comment 6041651011 — 2026-10-07T15:57:44Z — Casey-allard

[uor-pod] up waiting: no 1 x pro6000 in US-CO-1 US-NC-2; by=claude/plana retries every 1 min for up to 1 h — Plan A: continue-pretrain 214M on SmolLM-Corpus, PRO 6000 (owner-approved ladder 10-07)

### comment 6041688867 — 2026-10-07T15:59:50Z — Casey-allard

[uor-pod] volume-create lpakn4udlj uor-shared-US-NC-2 (100 GB) for claude/plana — non-canonical: seed the hot set, archive results to EUR-NO-1

### comment 6041729305 — 2026-10-07T16:01:57Z — Casey-allard

[uor-pod] up wp23upwpazsjes (1 x pro6000, US-NC-2 volume lpakn4udlj (non-canonical), $2.09/h) by=claude/plana hours=14 — Plan A: continue-pretrain 214M on SmolLM-Corpus, PRO 6000 (owner-approved ladder 10-07)

### comment 6042688445 — 2026-10-07T17:01:52Z — Casey-allard

[uor-pod] release ivbx5z1tdahfde by=claude/planb

### comment 6042691206 — 2026-10-07T17:01:58Z — Casey-allard

[uor-pod] down ivbx5z1tdahfde by=claude/planb (was $0.99/h, up 122 min)

### comment 6042777202 — 2026-10-07T17:05:22Z — Casey-allard

[uor-pod] release pmvhyr6hs4u1ui by=deepseek/entry-isolation

### comment 6042779517 — 2026-10-07T17:05:29Z — Casey-allard

[uor-pod] down pmvhyr6hs4u1ui by=deepseek/entry-isolation (was $0.99/h, up 96 min)

### comment 6043486508 — 2026-10-07T17:43:36Z — Casey-allard

[uor-pod] up eo3t7eaomt634g (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/native-artifact-hf hours=0.25 — Mount retained native checkpoint for verified HF archival; no training or builds [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6043495594 — 2026-10-07T17:44:09Z — Casey-allard

[uor-pod] up 7gavj0ij981zmn (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/entry-ceiling hours=3 — Zero-update entry ceiling on the full 512 panel: mixed pool vs Generate-only pool (issue #820)

### comment 6043632786 — 2026-10-07T17:52:17Z — Casey-allard

[uor-pod] release eo3t7eaomt634g by=codex/native-artifact-hf

### comment 6043634619 — 2026-10-07T17:52:23Z — Casey-allard

[uor-pod] down eo3t7eaomt634g by=codex/native-artifact-hf (was $0.74/h, up 8 min)

### comment 6044186940 — 2026-10-07T18:25:19Z — Casey-allard

[uor-pod] release 7gavj0ij981zmn by=deepseek/entry-ceiling

### comment 6044188647 — 2026-10-07T18:25:26Z — Casey-allard

[uor-pod] down 7gavj0ij981zmn by=deepseek/entry-ceiling (was $0.99/h, up 41 min)

### comment 6044400243 — 2026-10-07T18:38:05Z — Casey-allard

[uor-pod] up am23lfope5i5dj (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/entry-scorer hours=3 — Dedicated entry scorer: fetch the retained panel and tokenizer, then fit and evaluate a feature-keyed entry scorer (issue #820)

### comment 6044547788 — 2026-10-07T18:46:57Z — Casey-allard

[uor-pod] release am23lfope5i5dj by=deepseek/entry-scorer

### comment 6044549475 — 2026-10-07T18:47:03Z — Casey-allard

[uor-pod] down am23lfope5i5dj by=deepseek/entry-scorer (was $0.99/h, up 8 min)

### comment 6044873389 — 2026-10-07T19:06:04Z — Casey-allard

[uor-pod] up c5dolz3luk9ino (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/entry-scorer-fit hours=3 — Entry-scorer plumbing proof on the native path (issue #820)

### comment 6045143517 — 2026-10-07T19:21:29Z — Casey-allard

[uor-pod] release c5dolz3luk9ino by=deepseek/entry-scorer-fit

### comment 6045145263 — 2026-10-07T19:21:36Z — Casey-allard

[uor-pod] down c5dolz3luk9ino by=deepseek/entry-scorer-fit (was $0.99/h, up 15 min)

### comment 6045349730 — 2026-10-07T19:34:01Z — Casey-allard

[uor-pod] up i6z8ev8en6mxv7 (1 x 5090 via ladder, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=deepseek/entry-scorer-live hours=2 — Entry scorer owning the boundary decision on the live evaluation path (issue #820)

### comment 6045917121 — 2026-10-07T20:09:00Z — Casey-allard

[uor-pod] release i6z8ev8en6mxv7 by=deepseek/entry-scorer-live

### comment 6045918718 — 2026-10-07T20:09:06Z — Casey-allard

[uor-pod] down i6z8ev8en6mxv7 by=deepseek/entry-scorer-live (was $0.99/h, up 34 min)

### comment 6045967458 — 2026-10-07T20:12:06Z — Casey-allard

[uor-pod] up duyyqeu7m1m9v8 (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/continuation-field-cuda hours=1 — Exact-head signed-H4 continuation CUDA score and gradient parity; precompiled binary, no CPU training [https://github.com/UOR-Foundation/uor-r4/pull/1855]

### comment 6046038984 — 2026-10-07T20:16:27Z — Casey-allard

[uor-pod] release duyyqeu7m1m9v8 by=codex/continuation-field-cuda

### comment 6046071976 — 2026-10-07T20:18:30Z — Casey-allard

[uor-pod] down duyyqeu7m1m9v8 by=codex/continuation-field-cuda (was $0.74/h, up 5 min)

### comment 6046472057 — 2026-10-07T20:42:29Z — Casey-allard

[uor-pod] up 8ze1iqw5rzlhly (1 x 4090 via ladder, EUR-NO-1, $0.74/h) by=deepseek/entry-export hours=2 — Float-master versus integer-export entry rank control, with baselines and tie counts (issue #820)

### comment 6046714926 — 2026-10-07T20:57:46Z — Casey-allard

[uor-pod] release 8ze1iqw5rzlhly by=deepseek/entry-export

### comment 6046716360 — 2026-10-07T20:57:51Z — Casey-allard

[uor-pod] down 8ze1iqw5rzlhly by=deepseek/entry-export (was $0.74/h, up 14 min)

### comment 6046780924 — 2026-10-07T21:01:53Z — Casey-allard

[uor-pod] up e9sv357efub8ve (1 x 4090 via ladder, EUR-NO-1, $0.74/h) by=deepseek/entry-transfer hours=2 — Entry decision on a prospectively fixed transfer panel: new values/roles/order, no memorised query signatures (issue #820)

### comment 6047078286 — 2026-10-07T21:20:06Z — Casey-allard

[uor-pod] release e9sv357efub8ve by=deepseek/entry-transfer

### comment 6047079880 — 2026-10-07T21:20:12Z — Casey-allard

[uor-pod] down e9sv357efub8ve by=deepseek/entry-transfer (was $0.74/h, up 18 min)

### comment 6047518317 — 2026-10-07T21:47:53Z — Casey-allard

[uor-pod] up lg5m36pkj563my (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/continuation-fit-v2 hours=1.5 — PR1858 precompiled sm89 shared-H4 CUDA controls then32-update full-answer construction fit [https://github.com/UOR-Foundation/uor-r4/pull/1858]

### comment 6047694754 — 2026-10-07T21:59:35Z — Casey-allard

[uor-pod] release lg5m36pkj563my by=codex/continuation-fit-v2

### comment 6047705727 — 2026-10-07T22:00:21Z — Casey-allard

[uor-pod] down lg5m36pkj563my by=codex/continuation-fit-v2 (was $0.74/h, up 11 min)

### comment 6047771083 — 2026-10-07T22:05:01Z — Casey-allard

[uor-pod] up uw6ezyr8cajs4i (1 x 4090, EUR-NO-1, $0.74/h) by=codex/continuation-fullpass-v2 hours=1 — PR1858 one matched64totalupdate fullpass continuation dose discriminator; prebuilt c64 [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6047762388]

### comment 6047983191 — 2026-10-07T22:17:35Z — Casey-allard

[uor-pod] release uw6ezyr8cajs4i by=codex/continuation-fullpass-v2

### comment 6047990125 — 2026-10-07T22:17:59Z — Casey-allard

[uor-pod] down uw6ezyr8cajs4i by=codex/continuation-fullpass-v2 (was $0.74/h, up 12 min)

### comment 6048709356 — 2026-10-07T23:10:37Z — Casey-allard

[uor-pod] up shp3uc7kkcf4et (1 x 5090 via ladder, EU-RO-1 volume rfsx702p68 (non-canonical), $0.99/h) by=deepseek/state-probe hours=2 — Boundary state probe: does the read encode the question, tested offline on the prospectively fixed transfer panel (issue #820)

### comment 6048877985 — 2026-10-07T23:24:23Z — Casey-allard

[uor-pod] release shp3uc7kkcf4et by=deepseek/state-probe

### comment 6048879266 — 2026-10-07T23:24:29Z — Casey-allard

[uor-pod] down shp3uc7kkcf4et by=deepseek/state-probe (was $0.99/h, up 13 min)

### comment 6049393557 — 2026-10-08T00:09:09Z — Casey-allard

[uor-pod] up i2wfk6tg33t0bx (1 x 4090, EUR-NO-1, $0.74/h) by=codex/astra-joint-credit-20261007 hours=2 — Joint native factual and continuation graph admission then declared64-update construction; PR1862 [https://github.com/UOR-Foundation/uor-r4/issues/820]

### comment 6049402939 — 2026-10-08T00:09:54Z — Casey-allard

[uor-pod] up sz75rfdyr2h0jl (1 x 4090 via ladder, EUR-NO-1, $0.74/h) by=deepseek/read-channel hours=2 — Test the read's selection as the missing question channel (issue #820)

### comment 6049553205 — 2026-10-08T00:22:37Z — Casey-allard

[uor-pod] release sz75rfdyr2h0jl by=deepseek/read-channel

### comment 6049554410 — 2026-10-08T00:22:43Z — Casey-allard

[uor-pod] down sz75rfdyr2h0jl by=deepseek/read-channel (was $0.74/h, up 12 min)

### comment 6049837779 — 2026-10-08T00:47:47Z — Casey-allard

[uor-pod] release i2wfk6tg33t0bx by=codex/astra-joint-credit-20261007

### comment 6049893915 — 2026-10-08T00:52:54Z — Casey-allard

[uor-pod] down i2wfk6tg33t0bx by=codex/astra-joint-credit-20261007 (was $0.74/h, up 43 min)

### comment 6050469294 — 2026-10-08T01:45:42Z — Casey-allard

[uor-pod] up k2vhkezye9qdzm (1 x 4090 via ladder, EUR-NO-1, $0.74/h) by=deepseek/qsel hours=2 — Question-conditioned read selection: gate 1 (selection becomes question-determined) and gate 2 (state clustering) on the transfer panel (issue #820)

### comment 6050630397 — 2026-10-08T02:00:33Z — Casey-allard

[uor-pod] release k2vhkezye9qdzm by=deepseek/qsel

### comment 6050631494 — 2026-10-08T02:00:39Z — Casey-allard

[uor-pod] down k2vhkezye9qdzm by=deepseek/qsel (was $0.74/h, up 14 min)

### comment 6050826028 — 2026-10-08T02:18:26Z — Casey-allard

[uor-pod] up bwf3dswy8fbit0 (1 x 4090, EUR-NO-1, $0.74/h) by=codex/astra-reference-replay-20261008 hours=2 — One fixed native supervised-reference replay proposal; PR1867; preserve accepted replies and measure new complete outputs [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6050618184]

### comment 6051040993 — 2026-10-08T02:38:16Z — Casey-allard

[uor-pod] release bwf3dswy8fbit0 by=codex/astra-reference-replay-20261008

### comment 6051042053 — 2026-10-08T02:38:22Z — Casey-allard

[uor-pod] down bwf3dswy8fbit0 by=codex/astra-reference-replay-20261008 (was $0.74/h, up 19 min)

### comment 6051162422 — 2026-10-08T02:49:50Z — Casey-allard

[uor-pod] up iazj7kbf8lpdbu (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/vsa-codemode hours=3 — Compare the fixed token-id-hash VSA codebook against the learned-root LSH codebook (ablate-prose --vsa-code-mode fixed|root) and apply the question-form clustering statistic to both (issue #820)

### comment 6051220134 — 2026-10-08T02:55:14Z — Casey-allard

[uor-pod] release iazj7kbf8lpdbu by=deepseek/vsa-codemode

### comment 6051221120 — 2026-10-08T02:55:19Z — Casey-allard

[uor-pod] down iazj7kbf8lpdbu by=deepseek/vsa-codemode (was $0.99/h, up 5 min)

### comment 6051449765 — 2026-10-08T03:17:03Z — Casey-allard

[uor-pod] up dvc0ote3qggb8o (1 x 4090, EUR-NO-1, $0.74/h) by=codex/astra-native-proposal-20261008 hours=2 — One fixed18 native shared-operator code proposal round; PR1872 [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051223155]

### comment 6051767664 — 2026-10-08T03:48:05Z — Casey-allard

[uor-pod] release dvc0ote3qggb8o by=codex/astra-native-proposal-20261008

### comment 6051771872 — 2026-10-08T03:48:30Z — Casey-allard

[uor-pod] down dvc0ote3qggb8o by=codex/astra-native-proposal-20261008 (was $0.74/h, up 31 min)

### comment 6052082637 — 2026-10-08T04:15:33Z — Casey-allard

[uor-pod] up 373gpzossxkeoc (1 x 4090, EUR-NO-1, $0.74/h) by=codex/astra-native-frontier-20261008 hours=2 — One reached-frontier native proposal round; source c0bb0861cf; preserved runtime and fixed588 terms [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051865011]

### comment 6052428152 — 2026-10-08T04:43:42Z — Casey-allard

[uor-pod] down wp23upwpazsjes by=claude/plana (was $2.09/h, up 761 min)

### comment 6052466551 — 2026-10-08T04:46:42Z — Casey-allard

[uor-pod] release 373gpzossxkeoc by=codex/astra-native-frontier-20261008

### comment 6052472479 — 2026-10-08T04:47:09Z — Casey-allard

[uor-pod] down 373gpzossxkeoc by=codex/astra-native-frontier-20261008 (was $0.74/h, up 31 min)

### comment 6052492894 — 2026-10-08T04:48:43Z — Casey-allard

[uor-pod] up 4cpnfrdzow16s8 (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/astra-frontier-delivery-20261008 hours=1 — Cold exact-head PR1875 focused compile/tests after shared warm host deletion; no model rerun [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051865011]

### comment 6052659462 — 2026-10-08T05:01:32Z — Casey-allard

[uor-pod] release 4cpnfrdzow16s8 by=codex/astra-frontier-delivery-20261008

### comment 6052661832 — 2026-10-08T05:01:44Z — Casey-allard

[uor-pod] down 4cpnfrdzow16s8 by=codex/astra-frontier-delivery-20261008 (was $0.74/h, up 12 min)

### comment 6052833778 — 2026-10-08T05:15:03Z — Casey-allard

[uor-pod] up tuq3ezm6yq6rzj (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/astra-categorical-20261008 hours=2 — categorical Context-action learner focused Rust tests and bounded frozen-rule experiment [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6052811970]

### comment 6053493253 — 2026-10-08T06:04:05Z — Casey-allard

[uor-pod] release tuq3ezm6yq6rzj by=codex/astra-categorical-20261008

### comment 6053496031 — 2026-10-08T06:04:16Z — Casey-allard

[uor-pod] down tuq3ezm6yq6rzj by=codex/astra-categorical-20261008 (was $0.74/h, up 49 min)

### comment 6053575556 — 2026-10-08T06:09:50Z — Casey-allard

[uor-pod] up ofqn9sjw0y2ul2 (1 x 4090, EU-RO-1 volume rfsx702p68 (non-canonical), $0.74/h) by=codex/astra-parent-action-20261008 hours=2 — Original-parent categorical action learning; fixed one-proposal causal test after merged1877 [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6053558247]

### comment 6054273315 — 2026-10-08T06:53:38Z — Casey-allard

[uor-pod] release ofqn9sjw0y2ul2 by=codex/astra-parent-action-20261008

### comment 6054275006 — 2026-10-08T06:53:45Z — Casey-allard

[uor-pod] down ofqn9sjw0y2ul2 by=codex/astra-parent-action-20261008 (was $0.74/h, up 43 min)

### comment 6054831770 — 2026-10-08T07:23:46Z — Casey-allard

[uor-pod] up deeo6pwue2l60i (2 x 4090 via ladder, EU-RO-1 volume rfsx702p68 (non-canonical), $1.48/h) by=codex/astra-context-20261008 hours=3 — Constrained shared native Context learner focused checks then admitted single composite

### comment 6054898717 — 2026-10-08T07:27:44Z — Casey-allard

[uor-pod] lease deeo6pwue2l60i gpus=0 by=codex/astra-context-20261008 hours=3 — Focused Context learner checks and single composite; GPU1 free for other sessions

### comment 6055954120 — 2026-10-08T08:31:38Z — Casey-allard

[uor-pod] release deeo6pwue2l60i by=codex/astra-context-20261008

### comment 6055955694 — 2026-10-08T08:31:44Z — Casey-allard

[uor-pod] down deeo6pwue2l60i by=codex/astra-context-20261008 (was $1.48/h, up 67 min)

### comment 6056476762 — 2026-10-08T09:03:23Z — Casey-allard

[uor-pod] volume-create 6nrtmdn18s uor-shared-US-TX-3 (100 GB) for codex/sol-emission-20261008 — non-canonical: seed the hot set, archive results to EUR-NO-1

### comment 6056478192 — 2026-10-08T09:03:28Z — Casey-allard

[uor-pod] up p841qbaggweddp (2 x pro6000 via ladder, EUR-IS-1 volume v65f9luxbb (non-canonical), $4.18/h) by=codex/sol-emission-20261008 hours=4 — Pinned Context artifact recovery, focused emission implementation checks and one admitted coadaptation after checks [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6056464516]

### comment 6056502328 — 2026-10-08T09:04:58Z — Casey-allard

[uor-pod] lease p841qbaggweddp gpus=0 by=codex/sol-emission-20261008 hours=4 — Sol emission source checks and pinned recovery [https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6056464516]

### comment 6057190228 — 2026-10-08T09:47:25Z — Casey-allard

[uor-pod] up a4l3ypj8ms0iz4 (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/chat-qat hours=4 — QAT chat stack (20M rung, pure chat) then D11 export and d11-evaluate on chat held-out: the run card from round 5 (issue #820)

### comment 6057307788 — 2026-10-08T09:54:29Z — Casey-allard

[uor-pod] release p841qbaggweddp by=codex/sol-emission-20261008

### comment 6057319270 — 2026-10-08T09:55:10Z — Casey-allard

[uor-pod] down p841qbaggweddp by=codex/sol-emission-20261008 (was $4.18/h, up 51 min)

### comment 6057421711 — 2026-10-08T10:01:26Z — Casey-allard

[uor-pod] up 6m3tbmf40c27rk (2 x 4090 via ladder, EU-RO-1 volume rfsx702p68 (non-canonical), $1.48/h) by=codex/sol-prototype-20261008 hours=3 — Shared finite Generate prototype census #820 https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6057414489

### comment 6057447620 — 2026-10-08T10:02:50Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=0 by=codex/sol-prototype-20261008 hours=3 — Rust shared prototype census #820/6057414489

### comment 6057915806 — 2026-10-08T10:30:55Z — Casey-allard

[uor-pod] release a4l3ypj8ms0iz4 by=deepseek/chat-qat

### comment 6057917403 — 2026-10-08T10:31:00Z — Casey-allard

[uor-pod] down a4l3ypj8ms0iz4 by=deepseek/chat-qat (was $0.99/h, up 42 min)

### comment 6057932398 — 2026-10-08T10:31:55Z — Casey-allard

[uor-pod] up lbrbr0t4w3qsfh (1 x 5090 via ladder, EUR-NO-1, $0.99/h) by=deepseek/chat-cuda hours=3 — Corrected chat stack run with device=cuda, then export and d11-evaluate on the sealed held-out stream (issue #820)

### comment 6058544642 — 2026-10-08T11:08:44Z — Casey-allard

[uor-pod] release lbrbr0t4w3qsfh by=deepseek/chat-cuda

### comment 6058546356 — 2026-10-08T11:08:50Z — Casey-allard

[uor-pod] down lbrbr0t4w3qsfh by=deepseek/chat-cuda (was $0.99/h, up 36 min)

### comment 6058618228 — 2026-10-08T11:13:27Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/chat-mix hours=3 — Data-vs-architecture control arm on chat (issue #820)

### comment 6058880954 — 2026-10-08T11:29:59Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/chat-mix

### comment 6059089947 — 2026-10-08T11:41:51Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/chat-data hours=5 — Data-coverage experiment on chat (issue #820)

### comment 6065810625 — 2026-10-08T17:51:04Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/chat-data

### comment 6065992263 — 2026-10-08T18:01:58Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/term-weight hours=3 — Terminal-weight arm run (issue #820)

### comment 6069175319 — 2026-10-08T21:14:18Z — Casey-allard

[uor-pod] takeover 6m3tbmf40c27rk GPU(s) [1] from the EXPIRED lease of deepseek/term-weight by=deepseek/attr-store (no live job on them)

### comment 6069176666 — 2026-10-08T21:14:23Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/attr-store hours=3 — Attribute-extraction store arm at 19.9M (issue #820)

### comment 6069896566 — 2026-10-08T22:02:24Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/attr-store

### comment 6070154813 — 2026-10-08T22:17:58Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/attrshort hours=3 — Short-premise attribute store arm at 19.9M (issue #820)

### comment 6070479363 — 2026-10-08T22:39:54Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/attrshort

### comment 6070482933 — 2026-10-08T22:40:10Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/bind-capacity hours=4 — Cross-turn attribute binding at larger capacity (issue #820)

### comment 6070746397 — 2026-10-08T23:01:20Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/bind-capacity

### comment 6070924791 — 2026-10-08T23:15:52Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/bind-probe hours=2 — Binding probe P1 (issue #820)

### comment 6071176951 — 2026-10-08T23:37:29Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/bind-probe

### comment 6071653740 — 2026-10-09T00:19:32Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/copy-store hours=4 — Content-copy store arm, commitment+terminal weights (issue #820)

### comment 6072429656 — 2026-10-09T01:32:40Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/copy-store

### comment 6072551110 — 2026-10-09T01:44:32Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/span-arm hours=4 — Span commitment arm, copy-store mixture (issue #820)

### comment 6073169027 — 2026-10-09T02:42:22Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/span-arm

### comment 6073221586 — 2026-10-09T02:47:37Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/gate-sup hours=4 — Pointer gate supervision arm (issue #820)

### comment 6073925929 — 2026-10-09T03:54:11Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/gate-sup

### comment 6073974591 — 2026-10-09T03:59:07Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/copyterm hours=2 — inference-only: truncation-budget test of exact-copy rate (max_new_tokens 3/5/8 vs 40) on bind-probe rate set

### comment 6074040463 — 2026-10-09T04:05:43Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/copyterm

### comment 6074115415 — 2026-10-09T04:13:12Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/route-key hours=3 — Token-id-keyed pointer route arm (issue #820)

### comment 6077929079 — 2026-10-09T09:09:48Z — Casey-allard

[uor-pod] takeover 6m3tbmf40c27rk GPU(s) [1] from the EXPIRED lease of deepseek/route-key by=deepseek/ptr-identity (no live job on them)

### comment 6077930525 — 2026-10-09T09:09:53Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/ptr-identity hours=4 — token-identity pointer term: train arms A(w=0.5)/B(w=1) from arm-ptr + bind-probe scoring

### comment 6079198619 — 2026-10-09T10:36:36Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/ptr-identity

### comment 6079201175 — 2026-10-09T10:36:47Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/ptr-identity hours=4 — Control arm arm-c-ctrl still training (matcher for the identity experiment, issue #820)

### comment 6079365150 — 2026-10-09T10:47:54Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/ptr-identity

### comment 6079731514 — 2026-10-09T11:12:45Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/gate-floor hours=4 — Serving-time pointer gate floor sweep (#820, run card 2026-10-09)

### comment 6079899697 — 2026-10-09T11:24:26Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/gate-floor

### comment 6080182968 — 2026-10-09T11:44:04Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/copy-stop hours=4 — pointer copy-stop rule vs gate floor sweep on existing checkpoints (#820)

### comment 6081324661 — 2026-10-09T12:56:02Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/copy-stop

### comment 6081734860 — 2026-10-09T13:21:17Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=codex/sol-prototype-20261008

### comment 6082057378 — 2026-10-09T13:38:16Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=0 by=codex/sol-prototype-20261008 hours=6 — Complete saved coupled export and native reload from consolidated main

### comment 6082766822 — 2026-10-09T14:18:58Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=1 by=deepseek/idstop hours=4 — token-identity copy stop grid on existing ptr checkpoints

### comment 6083010229 — 2026-10-09T14:32:37Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=codex/sol-prototype-20261008

### comment 6083066348 — 2026-10-09T14:35:46Z — Casey-allard

[uor-pod] lease 6m3tbmf40c27rk gpus=0 by=codex/sol-residual-20261009 hours=2 — Saved residual mechanism analysis only; no new model proposals

### comment 6083180868 — 2026-10-09T14:42:34Z — Casey-allard

Successor compute board: #2037. `uor-pod` will post there once its default board number is updated. The capacity reports and the phantom-lease note stay readable here.

### comment 6083244176 — 2026-10-09T14:46:16Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=codex/sol-residual-20261009

### comment 6083321726 — 2026-10-09T14:50:51Z — Casey-allard

[uor-pod] release 6m3tbmf40c27rk by=deepseek/idstop


---
_Comment/review ids in this file: 6001256298, 6001256859, 6001346960, 6001430133, 6001494377, 6001967481, 6001978818, 6002044743, 6002252741, 6002320208, 6002321137, 6002321442, 6002341366, 6002967624, 6002968966, 6002973728, 6003508077, 6003609868, 6003610636, 6003621942, 6003629403, 6003657488, 6003663194, 6003706916, 6003768286, 6003769896, 6004257718, 6004302471, 6004308701, 6004624137, 6004625530, 6004625718, 6004626977, 6004655521, 6004770007, 6004775158, 6004896856, 6004911608, 6004935070, 6005002808, 6005251461, 6005253146, 6005311011, 6005325377, 6005338724, 6005357577, 6005392462, 6005410394, 6005478588, 6005489964, 6005564485, 6005582700, 6005652060, 6005669079, 6005772987, 6005774856, 6005819715, 6005844172, 6005922315, 6005933388, 6005975650, 6006002011, 6006018186, 6006090200, 6006109682, 6006184801, 6006200386, 6006344694, 6006454901, 6006457308, 6006833355, 6007343980, 6007346924, 6007351408, 6007355036, 6007388289, 6007569272, 6007655984, 6007657071, 6007744636, 6007749075, 6007794814, 6007914219, 6008018200, 6008023923, 6008120714, 6008243694, 6008244831, 6008246445, 6008463345, 6008466359, 6008526479, 6008527463, 6008566951, 6008645467, 6008807781, 6008809863, 6008987607, 6009015384, 6009204233, 6009270287, 6009463665, 6009465925, 6009549120, 6009568644, 6009579393, 6009917886, 6011240994, 6011936540, 6011939854, 6013378948, 6014221791, 6014592399, 6014813917, 6014841362, 6015192202, 6015193873, 6015295595, 6015449239, 6015699633, 6016706948, 6016708898, 6016945872, 6017029279, 6017146685, 6017148565, 6017223226, 6017483532, 6017615252, 6017617470, 6017735112, 6017808325, 6017968264, 6018217411, 6018222098, 6018427720, 6019240385, 6019244388, 6019386449, 6020419287, 6020426096, 6020430752, 6020966815, 6022271932, 6022321867, 6023624005, 6023724712, 6024709715, 6024755595, 6024758627, 6024793103, 6024794786, 6024852685, 6024863840, 6024998855, 6025238428, 6025260777, 6025325624, 6025880589, 6025881951, 6025997501, 6025999105, 6026004243, 6026276745, 6026581495, 6026582948, 6026735306, 6026919057, 6026938409, 6026950134, 6026985437, 6026996050, 6026997042, 6027156377, 6027176671, 6027206631, 6027207722, 6027244334, 6027250664, 6027382370, 6027383557, 6027405888, 6027435871, 6027515323, 6027545493, 6027549087, 6027559310, 6027562656, 6027654155, 6027656370, 6027672653, 6027673860, 6027727827, 6027769702, 6027770957, 6027806597, 6027829587, 6027830972, 6027943711, 6027944938, 6028368316, 6028412632, 6028414295, 6028716439, 6028884643, 6028904716, 6028935251, 6029021158, 6029027322, 6029338454, 6029365498, 6029366941, 6029706654, 6029776660, 6029780264, 6029862007, 6030888600, 6030889589, 6031008468, 6031021983, 6031022969, 6031094570, 6031095532, 6031157221, 6031361416, 6031402426, 6031403609, 6031522725, 6031523988, 6031525072, 6031547778, 6031549008, 6031598265, 6031601457, 6031625031, 6031626378, 6031654938, 6031656038, 6031746895, 6032399627, 6032425331, 6032426675, 6032995911, 6033129907, 6033181680, 6033260489, 6033262729, 6034201626, 6034203108, 6034302108, 6034872438, 6034874079, 6035281285, 6035284868, 6036053881, 6036113684, 6036122787, 6036940913, 6040096956, 6040228375, 6040445474, 6040447393, 6040564169, 6040674653, 6040676396, 6040692808, 6040853947, 6041092393, 6041101907, 6041651011, 6041688867, 6041729305, 6042688445, 6042691206, 6042777202, 6042779517, 6043486508, 6043495594, 6043632786, 6043634619, 6044186940, 6044188647, 6044400243, 6044547788, 6044549475, 6044873389, 6045143517, 6045145263, 6045349730, 6045917121, 6045918718, 6045967458, 6046038984, 6046071976, 6046472057, 6046714926, 6046716360, 6046780924, 6047078286, 6047079880, 6047518317, 6047694754, 6047705727, 6047771083, 6047983191, 6047990125, 6048709356, 6048877985, 6048879266, 6049393557, 6049402939, 6049553205, 6049554410, 6049837779, 6049893915, 6050469294, 6050630397, 6050631494, 6050826028, 6051040993, 6051042053, 6051162422, 6051220134, 6051221120, 6051449765, 6051767664, 6051771872, 6052082637, 6052428152, 6052466551, 6052472479, 6052492894, 6052659462, 6052661832, 6052833778, 6053493253, 6053496031, 6053575556, 6054273315, 6054275006, 6054831770, 6054898717, 6055954120, 6055955694, 6056476762, 6056478192, 6056502328, 6057190228, 6057307788, 6057319270, 6057421711, 6057447620, 6057915806, 6057917403, 6057932398, 6058544642, 6058546356, 6058618228, 6058880954, 6059089947, 6065810625, 6065992263, 6069175319, 6069176666, 6069896566, 6070154813, 6070479363, 6070482933, 6070746397, 6070924791, 6071176951, 6071653740, 6072429656, 6072551110, 6073169027, 6073221586, 6073925929, 6073974591, 6074040463, 6074115415, 6077929079, 6077930525, 6079198619, 6079201175, 6079365150, 6079731514, 6079899697, 6080182968, 6081324661, 6081734860, 6082057378, 6082766822, 6083010229, 6083066348, 6083180868, 6083244176, 6083321726_

