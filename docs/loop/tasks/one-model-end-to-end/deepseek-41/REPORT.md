# REPORT: one-model-end-to-end, deepseek-41

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

**In one line:** `served-reply-panel --engine integer` now generates through
`IntegerStackSession::serve_step`, the same path as `uor-chat`, and can attach
the same artifact-bound content-readout sidecar. On a sealed 256-row panel
without a content readout, the CPU and integer reports remain equal after their
timing and engine fields are removed.

## What did not work, first

- **No real asking artifact with a content readout was available.** The 256-row
  parity run uses a sealed nearest artifact with no content readout. The new
  content-readout and turn-memory product paths are pinned by synthetic tests,
  not by a trained asking model.
- **Teacher-forced and paired-first ranking remain unavailable on the integer
  engine.** They still need the CPU port's full Q12 query and nearest ranking,
  which `serve_step` does not return. This unit keeps their refusal rather than
  silently scoring a different path.
- **The first content-readout test fixture had width 64.** The serving
  signature requires a 256-coordinate query, so the test was refused before
  decoding. The fixture now uses width 256; this was an instrument repair, not
  a mechanism result.
- **A debug-build parity run was stopped after two minutes.** The release build
  finished the same CPU side in 12.1 s. The stopped run produced no result.

## What worked

- **Integer generation is the product path.** The `Decoder` interface now asks
  only for the served token. The CPU engine returns its nearest-neighbour token;
  the integer engine calls `serve_step`, which applies the Hamming ask/carry,
  turn memory, point key and content readout in the product order.
- **Content readouts can be supplied and identified.** `--content-readout PATH`
  validates the sidecar against the artifact SHA-256 and vocabulary, attaches
  it before any session, and records its SHA-256 under
  `inputs.content_readout_sha256`. The CPU engine refuses the flag because it
  would ignore the readout.
- **The product pick is tested.** An attached readout whose 2,048-bit signature
  matches the first reply query changes the panel decoder's token from the
  native pick to the sidecar target. Restoring `step_native` or omitting the
  attachment makes the test fail.
- **Turn memory is tested.** The panel primes the prompt exactly as `StackChat`
  does, marking the reply start before the prompt's final token. On the
  point-key fixture, removing that mark changes the first served token from 99
  to 142 and fails the test.
- **Existing parity is unchanged.** On the sealed 256-row panel, the CPU and
  integer reports both have 0 complete and 0 complete-distinct pairs, with all
  256 replies ending at the 64-token cap. Their stripped reports are equal.

## What changed

One code file,
`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- generation uses a served-token `Decoder` method;
- the integer implementation calls `serve_step`;
- teacher-forced ranking keeps a separate CPU priming helper;
- `--content-readout` is parsed, validated, attached and identified;
- the CPU engine refuses a content readout;
- tests cover served content selection and turn-memory priming.

The ORDER, this REPORT and one notebook row are the remaining changes. No
bar, status, item scope or integer-library behavior changed.

## Measurements

- **Artifact:** `b64L-s1.uorlut`,
  `9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9`.
- **Rows:** 256. Complete 0, EOS stops 0, cap stops 256, context stops 0; pairs
  128, complete-distinct pairs 0.
- **Stripped report:** CPU and integer both
  `0081f5f37786c323174f0d2b387f8e3bd0482228d010cdf261eb566dcfbde020`.
- **Observed decode:** 1,351 tokens/s CPU port and 823 tokens/s integer engine
  in this release run. These are instrument timings, not a product-throughput
  claim.
- **Release binary:**
  `955efd78ea0bc90f68ccdc473a1e89d377021a58122a102203ec89e416345a4c`.
- **Local reports:** `local/deepseek/served-product-parity/final-parity/`
  (`cpu.json` `7d5bbf89…`, `integer.json` `32c389df…`, stripped
  `0081f5f3…`). They stay local.

## Checks run at the head

- `served-reply-panel` tests with `cpu-parity`: **25 passed, 0 failed**.
- `rustfmt --check` on the changed file: pass.
- Scoped Clippy for the binary with `-D warnings`: exit 0. Dependency crates
  retain their pre-existing warnings.
- Release build of `served-reply-panel`: pass.
- Sealed CPU/integer parity over 256 rows: stripped outputs byte-identical.
- Four mutations, each applied and restored:
  - use `step_native` instead of `serve_step`: content-readout test failed;
  - load but do not attach the readout: content-readout test failed;
  - do not return the sidecar hash: identity assertion failed;
  - omit the reply-start mark: turn-memory test changed 99 to 142 and failed.

## Skills used

None beyond the loop itself: the ORDER, focused host tests, sealed-panel
parity and mutation checks.

## Still open

- Train and score a real asking artifact through this path. That is claude-339,
  which remains Claude's run.
- Add integer-session teacher-forced ranking if later diagnostics require it.
- Implement ask/carry in the independent `uor-r4-cpu-train` Hamming port, or
  retain its named refusal. This unit does not edit that crate.
- Re-run the product smoke with the first successful asking artifact through
  both `served-reply-panel --engine integer` and `uor-chat`.

## Plain summary

The grading program had been reading an integer model one step before the
actual chat program. It now takes the same final choice as `uor-chat`, including
a content sidecar when one is supplied. On the latest sealed model, which has
no content sidecar, both paths still produce exactly the same replies and
stops. The same model is still too weak to answer the panel: every reply
reaches the length cap and none is complete. This change makes the next model's
score trustworthy; it does not make that model capable.
