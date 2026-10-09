# Criterion 2: the served-BPB gap is quantisation, not protocol — and the residual is the model — October 9

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: **CPU-only
integer/LUT evaluation of existing artifacts. No training, no pod, no local float generation.** Result:
**the recorded "float 0.877 / served 0.933" pair is a cross-position-set comparison; on identical
positions the serving path costs 0.004–0.009 BPB, the engine costs ~1e-6 BPB, and the remaining gap to
0.90 is the float model.** criterion 2 is not claimed and no served number moved.

## The question

M1 criterion 2 requires the model to export and serve under D11 at **≤ 0.90 BPB** on the chat held-out
stream. The recorded position is **float 0.877 BPB, served 0.933 BPB**, with quantisation cost
**0.00883 BPB** already measured on identical positions. The gap to target is **0.0330 BPB**, of which
quantisation is at most 27 %. The residual (~0.024) was recorded as **float model quality + protocol,
not separated**. That split decides whether effort goes to the model or to the export path. The round's
job was to separate it, or to show the recorded numbers cannot be separated.

The context that makes this worth a round: an earlier artifact showed an apparent 3.27 BPB gap that was
**2.5683 of domain mismatch plus a cross-protocol artefact** ([served gap
settled](../evidence/served_gap_settled_2026-10-08.txt)), so protocol effects on this stream are real
and can be large.

## What was measured, and on what

Everything below ran **on the laptop CPU**, on **existing artifacts**, on **identical position sets**.
No training. No pod. No float reply generation — float *scoring* of an existing checkpoint through its
own forward pass is evaluation, and it is the `float` column of the same tool that serves the integer
column, not a model run.

| identity | bytes | sha256 |
|---|---|---|
| artifact `tw-export-big2/model.lut` (GPTQ, 64 calib. windows, damp 0.01, `transport_snap=None`) | 11,891,588 | `10d8d60be3eed25f50d1668a159b8f93e73c75426ea2fb3393327f0cb4a5b966` |
| model `tw-arm-big2/model/model.safetensors` (19,929,136 params, w512 h8 ctx384 `rrarrarr`, read l2, seed 1) | 79,726,120 | `4ef5f38feb53cda11b53ac812b45fc804908997c14d0cde3ebe60a691972b43b` |
| valid `chat-v0-p2/heldout/tokens.u16` (6,194,589 tokens) | 12,389,242 | `e5f400b0e676791fd0458ce810163c5d2542adda80dccc8282f37d9ca8904cef` |
| tokenizer (4096 vocab, 3 added) | 109,457 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

The held-out stream's identity is the one both rounds of the record scored; the byte basis reproduces
exactly — **6,194,589 tokens, 17,576,697 bytes, 0 out-of-vocabulary, 2.837427 bytes/token**, the
sealed-league basis. All four files were reached from the laptop: the artifact and model from iCloud
(`UOR-R4/results/deepseek/tw-export-big2.tar`, `tw-arm-big2.tar`, MD5-verified against the cloud-store
index), the stream from `chat-v0-p2.tar`, the tokenizer from a hash-identical local copy. **The record's
own model is not in the index** — see Limitations.

The `lens.u16` byte-length table is derived from the tokenizer exactly as `uor-r4-tokenizer`'s
`token_byte_lengths` does (raw GPT-2 byte-decoder lengths, added tokens counted as their literal bytes);
[`make_lens.py`](make_lens.py) rebuilds it and prints the byte-basis check, which is the 2.837427 above.

Commands, verbatim:

```text
geometric-stack lut-evaluate artifact=/tmp/bpb-split/x/tw-export-big2/export-big2/model.lut \
  model=/tmp/bpb-split/x/tw-arm-big2/arm-big2/model \
  valid=/tmp/bpb-split/chat-v0-p2/chat-v0-p2/heldout/tokens.u16 \
  lens=/tmp/bpb-split/lens.u16 windows=64 threads=6 reference=true out=.../lut-ref-64

geometric-stack d11-evaluate artifact=.../model.lut model=.../model \
  valid=.../tokens.u16 lens=.../lens.u16 windows=64 threads=6 out=.../d11-64

geometric-stack lut-evaluate ... windows=512 threads=6 reference=true out=.../lut-ref-512
```

## The three-way attribution, on identical positions

Built from the same source as the record (main `eff6516d4`), `--release`, in a scratch target directory.

**64 windows, 24,576 targets** (the record's own comparison size):

| column | nll | BPB |
|---|---|---|
| float (the checkpoint's own forward) | 3.5340307 | 1.796884 |
| reference (the artifact dequantized, in f32) | 3.5420724 | 1.800973 |
| integer (the multiplier-free served engine) | 3.5420708 | 1.800973 |

**quantisation 0.00804 nats = 0.004089 BPB; engine −0.0000015 nats = −0.00000077 BPB; total
0.004088 BPB.** Top-1 agreement engine vs float 0.88265.

**512 windows, 196,608 targets** (the size the recorded float cell used): float 3.4666163
(1.767493 BPB), reference 3.4741697 (1.771344), integer 3.4741654 (1.771342); **quantisation
0.0075534 nats = 0.003841 BPB, engine −0.0000043 nats.** Top-1 agreement 0.88000.

Independent checks on the same artifact: `d11-evaluate` gives `d11 = d10 = 3.5420708435`,
`d11_minus_d10_nll 0.0`, `top1_agreement 1.0`, `max_abs_logit_difference 0`. The `lut-evaluate`
integer column reproduces that nll to ten decimals, and the 512-window integer column is
`3.4741654130` whether or not the float model is loaded. The served number is a property of the
artifact.

## Why the absolute number moves and the quantisation term does not

The float model's per-window score spread is **0.98 nats at 64 windows (0.50 BPB per window)** and
**0.87 nats at 512 windows (0.44 BPB)**; the re-sampling standard error of the 64-window mean is
**0.122 nats = 0.063 BPB**. The two window sets here share only their first window (strides 96,784 and
12,074), and they differ by **0.029 BPB** on the same model — inside that error. The quantisation term
is ~50× less sensitive: **0.00409 ± 0.00141 BPB** (64 w) and **0.00384 ± 0.00049 BPB** (512 w).

So the honest reading of the record's own two headline numbers is: **0.0552 BPB is 0.0088 of
quantisation and 0.0464 of position-set difference**, and the 0.0464 is not a serving cost. The
"0.0464 un-attributed integer arithmetic" that [the serving-mode
audit](../evidence/serving_mode_audit_2026-10-08.txt) inferred from the cross-set subtraction was
already refuted by [the matched measurement](../evidence/served_gap_settled_2026-10-08.txt), which
measured the engine at 1e-6 nats; this round reproduces that independently on another artifact and at
both window sizes.

## The split, stated as the round asked for it

- **(a) quantisation — measured.** 0.00883 BPB on the recorded artifact's identical positions; 0.0041
  and 0.0038 BPB on the two matched cells measured here. It is small and it is stable.
- **(b) protocol / protocol-artefact — measured, and it is not in the serving path.** The engine
  contributes **≤ 1e-5 BPB** in every cell (bit-exact D10 = D11 logits). What is large is the
  **position-set sensitivity of the absolute score**, measured at 0.44–0.50 BPB per window and 0.029
  BPB between the two window sizes tried here. That is a property of the *measurement*, not of serving.
- **(c) float model quality — measured by elimination on identical positions, and it is the residual.**
  On the record's own matched table the float model is 0.92394 against a 0.90 target, so even a
  lossless export misses by ~0.024 BPB. Perfecting the quantiser buys at most 0.0088 and cannot close
  the gap alone — the arithmetic the round set out to check survives, with the residual now attributed
  to the model rather than to protocol.

## Decision

Criterion 2 is **not met** and cannot be met by export work. The serving path is within 0.4 % of its own
float model on matched positions, and the engine is exact, so **the remaining lever is the float model
at this scale or a larger one within the laptop budget** — the same conclusion the criterion-2
arithmetic reached, now with protocol excluded as the cause rather than left in the residual. **A second,
cheaper decision follows from the spread:** criterion 2 as written does not pin its protocol (window
count, position set, byte basis), and on this stream the same model moves further between window sets
than the whole gap to target. Any future criterion-2 claim should name the window count and compute the
BPB from the stream's own byte basis, or it will not be reproducible.

## Limitations

1. **The artifact measured is not the artifact that produced 0.877/0.933.** The record's model
   (`batch 32`, `lr 4e-4`, `warmup 500`, `chat-v0-p2` train only) is not in the cloud-store index and
   is not on this laptop; the search covered the full 466-entry index, the local checkouts and the
   filesystem. What is available is the term-weight 19.9 M family — same architecture, same
   parameter count, same 12,207 steps, no transport snap — but `batch 16`, `lr 6e-4`, a `mixed-t2`
   corpus, a round-to-nearest export and a different GPTQ calibration set. Its absolute level
   (1.77–1.80 BPB) is ~0.87 BPB worse than the recorded model's float cell (0.87755), so this is a **protocol and structure**
   measurement, not a re-measurement of that model. The three-way split it gives is specific to its own
   weights; the record's 0.00883 remains the comparable quantisation cell for the recorded artifact.
2. **The 0.877 float cell's own validity is unresolved.** One model here gives 1.7969 BPB at 64 windows
   and 1.7675 at 512, a 0.029 BPB swing, and the record's own matched table gives 0.92394 for the
   recorded pair's float side while the headline quotes 0.877. A 0.047 BPB swing from window draw is
   larger than the 0.033 gap; settling it needs the recorded artifact.
3. **Not a served-model result.** No D11 export or serving claim is made for any candidate, criterion 2
   is not claimed, and STATUS/ROADMAP/#2028 are unchanged because no BPB headline, served model or
   milestone moved. The recorded 0.93277 served number also has no D11 export for the *pointer* lineage
   ([#2075](https://github.com/UOR-Foundation/uor-r4/pull/2075)); this record is about the
   non-pointer chat stack that does export.

## Cost

Zero pod spend, zero external compute. Laptop wall time **≈ 60 min**: release build 4 m 31 s
(measured), 64-window matched run 1 m 40 s (measured), `d11-evaluate` 64 windows and the two
512-window runs ≈ 35 min together (one pair concurrent), analysis and write-up the remainder. Peak
RSS below 2 GB; no artifact was left outside `/tmp`.

## Evidence

[`docs/evidence/chat_served_gap_attribution_2026-10-09.txt`](../evidence/chat_served_gap_attribution_2026-10-09.txt)
carries the artifact identities, the four command forms, all four tables, the per-window spread and the
cross-mode checks.
