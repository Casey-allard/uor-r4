# Criterion 2's protocol, pinned: the published float/served pair is a cross-window-count comparison — October 9

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: **CPU-only
integer/LUT evaluation of existing artifacts. No training, no export, no quantiser change, no float
generation.** Result: the recorded artifact's LUT (`2ac4c306…`) is **gone — established exhaustively,
not assumed**; the volume holds the **independent retrain** of the same recipe, and on that retrain a
**matched** triple at a declared 512 windows / 196,608 positions on the stream's own byte basis reads
**float 0.877550 / reference 0.886838 / integer 0.886838 BPB**, quantisation **0.009288 BPB**, engine
**2.7e-7 BPB**. The published "0.87755 float / 0.93277 served" pairing was **never like-for-like**: the
float cell is a 512-window number and the served cell is a 64-window number, and the same artifact's
served column moves **0.045935 BPB** between those two counts on identical bytes. criterion 2 is
**not met and not claimable**.

## The question

The previous round
([chat-served-gap-attribution](../chat-served-gap-attribution-2026-10-09/README.md)) separated
quantisation from engine and model on one existing artifact, and closed with a named next step: replace
the published float cell with a **matched float/served pair on identical positions at a fixed window
count**, so criterion 2 has a protocol-pinned baseline, and restore the recorded artifact from the
EU-RO-1 volume `rfsx702p68` (`uor-shared-EU-RO-1`) to do it.

## Artifact identity: what was expected, and what exists

**The recorded artifact is not recoverable, and the volume was searched to the end rather than
sampled.** On `rfsx702p68`, read-only:

| search | result |
|---|---|
| every `*.lut` on `/workspace` hashed (~35 files) | none is `2ac4c3068551e56e3883880419e12f06bef8bc65cbabde59d00136953fe111af` |
| every file on `/workspace` of the recorded size `11891652` bytes | 4 files, none with that digest (`721d4bdb…`, `cce47c83…`, `chat-objective` `r6`/`r7`) |
| text grep of the whole `/workspace` tree for the recorded fused-float nll `1.7250754982233047` | only the checked-in docs copy, `term-weight/src/docs/evidence/chat_stack_served_d11_2026-10-08.txt` |
| text grep for the digest string `2ac4c306` | only three checked-in docs copies under the same `src/` tree |
| the 604-entry cloud-store index (all entries) | no `chat-served*`, no `chat-objective*`, no 11,891,652-byte object |

**What does exist is the retrain of the recorded recipe**, on the same volume, with its own provenance
chain (`attempt.json`, `report.json`, `export.json`, `snap.json`, `evaluation.json`, all read):

| identity | path on `rfsx702p68` | bytes | sha256 |
|---|---|---|---|
| checkpoint (retrain, `batch 32`, `lr 4e-4`, `warmup 500`, `steps 12207`, `chat-train.u16`) | `deepseek/chat-served2-20261008/model-a1/model/model.safetensors` | 79,726,120 | `cbef6906c11cb260d4316c25ef99fd672ee9c147bffb5d644c5013e4b9ab0d8c` |
| its GPTQ export (64 calibration windows, damp 0.01, `transport_snap=None`) | `deepseek/chat-served2-20261008/export-a1/model.lut` | 11,891,652 | `721d4bdbd193c64edbb26069e15ed49c06844c4bd9e4557b83ea2b942b8a2a8a` |
| held-out stream | `deepseek/chat-data-control-20261008/data/heldout.u16` | 12,389,242 | `e5f400b0e676791fd0458ce810163c5d2542adda80dccc8282f37d9ca8904cef` |
| tokenizer (4096 vocab, 3 added) | `deepseek/chat-data-control-20261008/data/tokenizer.json` | 109,457 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

All four digests were verified **on the pod and again after transfer** on the laptop. The checkpoint is
the same declared recipe as the recorded run (`batch 32`, `lr 4e-4`, `warmup 500`, 12,207 steps,
`chat-v0-p2` train only, w512 h8 ctx384 `rrarrarr`, read l2, seed 1, 19,929,136 parameters), trained on a
different pod at 126,945 tokens/s versus the recorded run's 164,277 — an independent retrain, exactly as
the earlier current-state note recorded ("an independent retrain reproduces the served nll to 2.1e-6").

**Which run produced which published cell — read off the numbers, not assumed:**

| published cell | value | producing artifact | nll | status |
|---|---|---|---|---|
| round-10 fused float (`chat_stack_served_d11_2026-10-08.txt`) | 0.8770 | original run, LUT `2ac4c306…` | 1.7250754982233047 @196,608 | **artifact gone** |
| round-23 fused float (`quantiser_arm_verdict_2026-10-08.txt`) | 0.87755 | **retrain**, `cbef6906…` / `721d4bdb…` | 1.7259248122572899 @196,608 | restored, measured here |
| served (published as 0.93277) | 0.93277 | both runs, 64 windows | 1.8345338586523747 (original) / 1.8345359993679817 (retrain) | retrain restored |

So the published "pair" mixes two runs **and** two window counts, and the float cell this round was sent
to match (0.87755) belongs to the retrain. The retrain's float forward reproduces it: 1.7259248087 nats at
512 windows → **0.877550 BPB**.

## Method, and the byte basis

Built from `origin/main` `36322f365`, release, in a gitignored `local/target` inside this worktree (no
`/tmp`). Commands, verbatim (paths elided to `$C2P`):

```text
geometric-stack lut-evaluate artifact=$C2P/export-a1/model.lut model=$C2P/model-a1 \
  valid=$C2P/data/heldout.u16 lens=$C2P/lens.u16 windows=512 threads=6 reference=true out=$C2P/out/lut-ref-512
geometric-stack d11-evaluate artifact=$C2P/export-a1/model.lut model=$C2P/model-a1 \
  valid=$C2P/data/heldout.u16 lens=$C2P/lens.u16 windows=512 threads=6 out=$C2P/out/d11-512
```

`lens.u16` is rebuilt from the transferred tokenizer by
[`make_lens.py`](../chat-served-gap-attribution-2026-10-09/make_lens.py). **The stream's own byte basis
was verified on this copy, not assumed:** 6,194,589 tokens after the 64-byte header, 17,576,697 bytes,
0 out-of-vocabulary, max id 4095, **2.837427 bytes/token** — the sealed-league basis, reproduced exactly.

**The tool's own `bits_per_byte` is a different basis, and every number below names which one it is.**
`lut-evaluate` divides by the byte lengths of the *scored targets* (`geometric-stack.rs:3617,3670`), which
on this 512-window sample is 556,319 bytes over 196,608 targets = **2.829585 B/token**. The published
0.877/0.933 figures use the stream constant **2.837427 B/token**. The two differ by 0.28 % here
(0.879982 tool vs 0.877550 stream basis on the same run). Quoting one while computing the other is a
second way to make a BPB number unreproducible; both are reported.

The window rule is deterministic and came from the same source as the recorded 64-window runs:
`stride = (tokens − context − 1) / windows`, starts `k × stride`, 384 targets per window
(`geometric-stack.rs:3614`). At 512 windows the starts are `0 + k × 12,098`, `k = 0..511`, last start
6,182,078 — 196,608 targets.

## The matched triple, on identical positions

**512 windows, 196,608 targets, stream `e5f400b0…`, one `lut-evaluate reference=true` run:**

| column | nll (nats/token) | BPB on stream basis (2.837427 B/token) | tool `bits_per_byte` (sample 2.829585) |
|---|---:|---:|---:|
| float (checkpoint's own forward) | 1.7259248087 | **0.877550** | 0.879982 |
| reference (artifact dequantized, f32) | 1.7441922849 | **0.886838** | 0.889296 |
| integer (multiplier-free served engine) | 1.7441928232 | **0.886838** | 0.889296 |

- **quantisation (float → reference): 0.0182674762 nats = 0.009288 BPB** (0.009314 on the tool's sample
  basis); per-window mean 0.0182675 nats, se 0.000568 nats = 0.000289 BPB;
- **engine (reference → integer): 5.3836293e-7 nats = 2.74e-7 BPB** — the served path is exact to
  seven decimal places in nats;
- total float → integer 0.0182680145 nats = **0.009288 BPB**; top-1 agreement engine vs float 0.92820.

## The finding: the published pair is cross-window-count

The same artifact's own recorded `d11-evaluate` at the recorded protocol — read from
`chat-served2-20261008/d11-a1/evaluation.json` on the volume — gives `d10 = d11 = 1.8345359993679817` on
**24,576 targets (64 windows)**, `d11_minus_d10_nll 0.0`, `top1_agreement 1.0`,
`max_abs_logit_difference 0`, first difference null → **0.932773 BPB** on the stream basis (published as
0.93277). Against this round's **0.886838 BPB** on the same artifact, same stream, same engine:

**the served column moves 0.045935 BPB between 64 windows and 512 windows on identical bytes.**

Per-window float spread at 512 windows is sd **0.643633 nats = 0.3273 BPB**, and the re-sampling standard
error of the 512-window mean is **0.028445 nats = 0.014525 BPB**. So the published float/served pair
subtracts a 512-window number from a 64-window number, and the window-count term it ignores is larger
than the entire 0.033 gap it was used to measure. This is the same class of error as the cross-position-set
artefact annotated on 2026-10-08 ([serving-mode audit](../../evidence/serving_mode_audit_2026-10-08.txt)),
now measured on criterion 2's own headline.

## The honest verdict on criterion 2

| reading | value | against the 0.90 target |
|---|---:|---|
| recorded protocol, 64 windows / 24,576 targets | 0.932773 | **above by 0.032773** (3.52 % relative; 0.41 % of the 8-bit/byte ceiling; 9.4 % of the distance to the KN-5 gate at 1.280275) |
| pinned protocol, 512 windows / 196,608 targets | 0.886838 | below by 0.013162 — **but the margin is smaller than the 512-window re-sampling se of 0.014525 BPB** |

criterion 2 is **not met, not claimable, and not measurable at the precision its threshold implies**: the
matched served number is under 0.90, the margin is inside the noise, and the *same artifact* is above
0.90 at the protocol the criterion's own served figure was recorded at. Declaring it met by choosing the
favourable window count is exactly the protocol-shopping this round exists to prevent. Independently of
the protocol question, criterion 2's original artifact is gone; what is measured here is its retrain.

**The lever, restated for the record:** the serving path is exact to 2.7e-7 BPB and the whole quantiser
contributes 0.0093 BPB on identical positions, so export work cannot move this number to any target that
matters. The 0.90 threshold is a published round number (3.5 % of the current served figure, 0.41 % of
the 8-bit/byte ceiling) and the lever behind it — and behind the 43/232 open reply panel — is the float
model, not the export path.

## The pinned protocol, proposed for criterion 2

Every BPB claim must name, in the claim itself:

1. **artifact identity** — checkpoint and LUT path + sha256;
2. **stream identity** — path + sha256, tokens and bytes after the header;
3. **window count** — 512 windows × 384 context = 196,608 targets, evenly spaced by
   `stride = (tokens − context − 1)/windows`, one fresh integer session per window;
4. **byte basis** — the stream's own 2.837427 B/token when comparing to published figures, and the
   tool's per-target lens basis when quoting `bits_per_byte`; say which;
5. **both columns at the same window count** — `lut-evaluate reference=true` reports float, reference and
   integer on identical positions in one run, and `d11-evaluate` at the same count carries the engine
   check;
6. **position count and re-sampling se** — a margin smaller than the se is not a result.

Proposed fixed protocol for criterion 2: **512 windows / 196,608 targets on chat-v0-p2 held-out
`e5f400b0…`, byte basis 2.837427 B/token, float and served columns from the same run.** On that protocol
this retrain reads 0.877550 float / 0.886838 served. The criterion itself should be restated to fix this
before any candidate is measured against 0.90.

## Decision

KEEP as a **protocol correction and a pinned baseline**, not as a capability gain: no model was trained,
exported, tuned or generated; the served number at the recorded protocol is unchanged. The published
pair "0.877 float / 0.933 served" should no longer be quoted as a pair; the matched pair at the pinned
protocol is **0.877550 float / 0.886838 served**. criterion 2 remains **not met**.

## Limitations

1. **The recorded artifact `2ac4c306…` is not recoverable** (search table above). Everything measured
   here is the independent retrain of the same declared recipe, 2.1e-6 nats from the recorded served nll
   and reproducing the published 0.87755 float cell to five decimals. The original's own matched pair,
   and its 512-window float, remain unmeasured and are **not** inferred from this one.
2. **One unverified cell: `d11-evaluate` at 512 windows.** It was started on the correct artifact and
   stream and killed when a hygiene pass removed this worktree mid-run (the branch
   `deepseek/criterion2-pin-20261009` was pushed before the run, which is why the numbers survived). The
   re-run was declined: the pod cap was full (3 pods, $7.36/h of $8.00/h) and the check is a
   consistency confirmation, not the finding. What carries D11-exactness for this artifact instead is
   its own recorded 64-window `d11-evaluate` (`d10 = d11`, diff 0.0, top-one 1.0, max abs logit
   difference 0) plus the 5.4e-7-nat integer-minus-reference term in the completed run above.
3. **One window draw.** The 512-window number has se 0.0145 BPB; a claim finer than that, or a comparison
   against 0.90, needs more windows or the document-clustered protocol. No second window count was run
   (deliberately: the 64-window figure quoted is the artifact's own recorded run, not a new one).
4. **No artifact preserved outside the volume.** The retrain pair is not in the 604-entry cloud-store
   index and exists only on the non-canonical EU-RO-1 volume. A `cloud-store put` of the pair is the named
   next step; this round did not spend on it.
5. No float reply generation, no training, no quantiser tuning, no re-export. The pod was used for
   read-only volume access only.

## Cost

Pod **$0.4336**: `1t3jmb6m4c9q9s`, 1×RTX 4090, EU-RO-1, `UOR_POD_VOLUME_DCS=EU-RO-1`, volume
`rfsx702p68`, up 2026-10-09T20:28:47Z → released 20:58:07Z → deleted 20:58:25Z, uptime 1754 s at
$0.89/h, read-only (no job ran on it, `ps` clean at release). A second pod attempt for the re-fetch was
**refused by the cap and cost nothing**. Laptop: release build 6 m 13 s; `lut-evaluate` 512 windows
16 m 30 s at 6 threads (engine step 623.7 s); the killed `d11-evaluate` contributed no number.

## Evidence

[`docs/evidence/criterion2_protocol_pin_2026-10-09.txt`](../../evidence/criterion2_protocol_pin_2026-10-09.txt)
carries the identities, the search transcript, the four command forms, the matched table on both byte
bases, the per-window spread, the recorded 64-window d11 record and the cost.
