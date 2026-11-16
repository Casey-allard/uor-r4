# DECISION: the written-state geometric count model track (`CHARTER.md` §5.1, D68)

- **Lab:** `claude` (top session), 16 Nov 2026.
- **Order:** [ORDER.md](ORDER.md). **Native-space items:** [NATIVE-SPACE.md](NATIVE-SPACE.md).
- **Decision review:** the PI review and two red teams, both SURVIVES-WITH-FIXES: [docs/research/reviews/pi-review-2026-11-16/](../../../../research/reviews/pi-review-2026-11-16/README.md), final text [DECISION.md](../../../../research/reviews/pi-review-2026-11-16/DECISION.md).
- **Authority:** §5.1 and D68: a route, scope or stop-rule change goes ahead when the lab's own whole-project decision review passes. It does not wait on the owner. It is logged for Casey, who can reverse it (§5.1). Nothing in §3 or the caps changes here. [claude]

## The decision

16 Nov: the lab's top session (claude), after its whole-project decision review (PI review + two red teams, linked), adopts the written-state geometric count model track. [claude]

The model's state is written by counting and by an integer Brier error-write. It is not found by scoring 1e9 coordinates. It holds n-let rows under positional-prime route keys, state rows under E8-snapped transport codes, pointer-valued conjunction rows and the existing Pick read. A lawful bucketed linear mixer combines them. All of it is new table families inside the D11 trunk's `TrunkTables`: one D85 state, S0–S6. The phases P0–P6 and their gates are in the review's DECISION §4 and in PLAN.md's "Approved sequences". [claude]

## What it supersedes

Steps (3)–(6) of the owner's 16 Nov gap order (PLAN.md, owner directions, 16 Nov), and the scored 1B run that step (6) points to. [claude]

| Step | Disposition |
|---|---|
| (1) the Pick read | done (claude-408, #4045); now S5, the output expert and the pointer copy |
| (2) conjunction memory | kept, as P3's pointer-valued conjunction rows: prime-product keyed, and "written by the output error" kept literally as the Brier error-write |
| (3) readout from the pick | kept, as the Pick expert and the S4 pointer readout; no 4,096-wide scan |
| (4) state across windows | becomes the transport-determinism item of the P0 contract |
| (5) `native-space-state-contract` over this line | P0 item 2: [NATIVE-SPACE.md](NATIVE-SPACE.md) |
| (6) corpus trainer, D87 ingest, 1B run | the P1–P3 counting and error-write runs on wick and a P5 core of ≤ 1e5 scored coordinates; any scored run over 1e6 coordinates stops |

## The bar, verbatim from DECISION §2

> The label is **"count-class, 1B-cell"** until bars A, R and B all pass at once. Only then is it "1B-equivalent (lawful-count reading)". The label never implies a neural comparison, because §3.10 bans them [checked CHARTER.md:38].
>
> **A. Absolute quality (the owner's bar, unchanged).** The model scores ≤ **0.90 BPB** on the owner-pinned `chat-v0-p2` held-out stream, through PRLL. The bar is the owner's (ROADMAP.md:17, via the m4 sheet §5 [checked docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md:1-20, §5]; the stream pin is [red R2, sheet:47]). The same number is also reported on sealed D87 DEV-test.
>
> **Q. Relative quality.** BPB(model) ≤ **0.80 x BPB(R\*)** on sealed D87 DEV-test bytes.
> - **R\***: interpolated modified Kneser-Ney with the standard three discounts per order and full N1/N2/N3+ type counts. It is unpruned, orders 1..N, with N chosen on DEV-tune (8-12 expected at about 2 B/token [red R1]). It is built in Rust from exact suffix-array counts on wick, and its hash and BPB are sealed in P0 before any model work.
> - **Anchor:** PPMd (var.H-class: counts + SEE + exclusion, no logistic), a Rust port validated byte-for-byte against the reference coder on a public fixture, then primed on TRAIN and scored on DEV-test. It is reported and does not gate.
> - **Stepping stones:** 0.95 x R\* and 0.90 x R\* are recorded checkpoints, not bars.
> - **Honesty note:** the lawful literature predicts 0.90-0.95 for counts plus linear mixing [red R1-M3, est]. The gap from 0.90 to 0.80 must come from the geometric components (S3, the P5 core). If it does not, the model keeps the count-class label.
>
> **R. Replies.** Each one exceeds the **no-memory control** (S4 and S5 off) and a **shuffled-role control**:
> - **R1:** ≥ 52/128 on the M2 fresh panel (the existing bar; 0/128 now [survey A1b]).
> - **R2:** ≥ 200/256 in free generation on the CPU-pilot reply panel (the forced-value ceiling [red R1-M4: cpu-training.md:27-28]).
> - **R4:** a fresh held-out-phrasing panel of ≥ 200 rows (§4.3 transfer), above both controls by the paired band.
> - **R3 floor:** ft/dev ≥ 7,002/14,287 (the D83 model [checked DECISIONS.md:287]). It is a floor, not part of the definition (R1-m3).
>
> **B. Budget and speed.**
> - State: ≥ **1.0e9 integer cells**, each written or refined by training, ≤ **4 GB resident**, with a lossless artifact. Report D5-a and D5-b (§3.9).
> - Speed: ≥ **1,000 tok/s** warm at 2k context on one M1 performance core, reported first. Cold-cache tok/s is reported beside it.
> - D85: replay is bit-identical between wick (CUDA) and the M1 (CPU) on ≥ 64 prompts.
>
> **Statistics.**
> - Count models are deterministic, so they have no seeds. Every gain needs a fresh panel or the sealed DEV-test (§4.3).
> - BPB comparisons use a paired prompt-block bootstrap, and its 95% interval must exclude 0.
> - Reply comparisons use the paired band, about 2·sqrt(discordant).
>
> **What "1B" counts.** It counts the written and refined cells, ~1.2e9 [est §3.5]. They are not neural parameters, and the label says so.

The lab's reading, not owner terms: the label sentence above names bars A, R and B, while DECISION §0 and §7 require Q as well. This card reads the label as needing all of A, Q, R and B. [claude]

## Reason, alternatives, cost to undo

- **Reason.** One 4090 cannot score 1e9 coordinates (900–2,000 GPU-days by survey B2.3), but it can count and error-write 1e9 integer cells in hours per pass (DECISION §0, §3.7, estimates). The M1 model is already a count model keyed by the trunk's read codes. DECISION §1 rules on every red-team blocker and major. [claude]
- **Turned down.** The gap order as written, leading to a scored 1e9-coordinate run; a hashed n-gram or PPM lane beside the trunk with geometry optional (R2-B1); a count table outside `TrunkTables` (R2-B2); squarefree prime products as n-let keys (R1-B3); a self-set weak denominator (R1-B1); logistic, stretch or exponentiated mixing (§3.2). [claude]
- **Cost to undo.** Medium: no code or run depends on it yet. Reversing it restores steps (3)–(6), and the review record stays as history. [claude]

## The owner queue (proposals; nothing waits on them)

DECISION §7's four items: P6 donor data; a linear-domain APM reading of §3.2 (off by default); the meaning of the label; a conditional §3.1 proposal if every P2 address arm loses. [claude]

## Index row: not written

`docs/integration/DECISIONS.md` is an owner path (`docs/harness/owner-paths.json`). `loop pr-check` refuses a change to it without an `## Owner approval` section quoting Casey with an ISO UTC time, and this unit has none. So the card lives here, as antigravity-114's did. The next free index number at `ef1f4a641` is D90. If Casey approves the index row, it reads: "D90 | 2026-11-16 | The written-state geometric count model track, a lab decision under D68 (claude-409's card), superseding steps (3)–(6) of the 16 Nov gap order." [claude]
