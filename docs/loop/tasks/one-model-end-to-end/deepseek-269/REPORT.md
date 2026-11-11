# REPORT: one-model-end-to-end, deepseek-269 (reply-control trace)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model training, pod or
spend.

**In one line:** `raw-id-replies` can now write an observation-only control row
for every generated token, and its traced run must match an untraced product
replay id for id.

## What did not work, first

- **The first EOS-rank unit had the wrong expectation.** Query `[1,1]` against
  rows `[0,0]`, `[1,1]`, `[2,2]`, `[1,1]` makes EOS rank 1, not 2. The test was
  corrected and now pins both the exact hit and the smaller-id tie.
- **The first synthetic replay exceeded its context.** The 8-position trunk
  could not hold a 4-token prompt and 8 reply steps. The test now uses a
  2-token prompt and a 4-token cap; it exercises the same trace path.
- **The first local PR gate found four new D32 sites.** Each was a tuple-index
  `.0` on `serve_step`'s `(token, margin)` result, which the source scanner
  treats as a floating-point literal. The four sites are now destructured; no
  serving behavior changed.
- **A recursive `rustfmt` invocation reformatted unrelated integer modules.**
  Those formatter-only changes were removed, and the four intended integer
  edits were reapplied. `rustfmt --check` remains non-clean over the entire
  integer source tree because of pre-existing formatting drift; the changed
  instrument file is clean, and `git diff --check` passes.
- **The full real panel is capped at eight reply tokens here.** The real pass
  proves observation and token identity on 1,141 prompts; it is not a new
  complete-reply quality score.

## What worked

`raw-id-replies --trace-control 1 --trace-out FILE` adds one JSONL row per
generated token. Each row records:

- nearest token, distance, runner-up token/distance and margin;
- the EOS rank and distance;
- turn-memory key/stored token/opening row, turn-memory presence and point-key
  token/distance;
- every nonempty Hamming head as copy or type, with its kept positions;
- reply position and short-cycle period;
- `opaque`, `reflective` or `permeable` class.

The classes are deliberately narrow observations:

- `opaque`: no memory/point move and no copy/type kept set;
- `reflective`: a copy or type head kept an earlier position;
- `permeable`: a turn-memory move named a token or the point key fired.

Permeable takes priority over reflective. The component fields remain present,
so the label never hides which read fired.

The probe rebuilds the selected token as the point-key token when present and
the whole-table nearest token otherwise. It refuses a run if that probe token
differs from the product token. After each reply it also runs a fresh untraced
session and refuses any id difference from the traced run.

## What changed

- `generation::short_cycle` is public so the instrument calls the same serving
  stop rule instead of copying it.
- `IntegerStackSession::last_memory_move_state` returns a copied, read-only
  view of the last turn-memory move. Serving never reads this accessor.
- The raw-id instrument adds the control-row schema, trace writer, selection
  guard, untraced replay guard, receipt tally, CLI flags and seven tests.
- The item scope includes the two integer files exposed for observation.
- No serving token, session state transition, artifact byte, trainer,
  exporter, store object or milestone bar changed.

## Measurements

Focused synthetic `/7` artifact:

- artifact SHA-256:
  `d84630417bfcc2c2d6194c7ff4d9b9bc7f73d66cd9b24ba7ea09079f64951beb`
- samples SHA-256:
  `7504c3173c3a3d3775bd35efe8e3e9af0e63fcdc1bfc0a2931233ed357d818f3`
- 1 prompt, 4 generated tokens, 0 EOS stops;
- control rows: **4**;
- classes: **1 opaque, 3 reflective, 0 permeable**;
- short-cycle rows: **2**;
- probe-selection mismatches: **0**;
- traced-versus-untraced reply mismatches: **0**;
- traced and untraced reply JSONL files were byte-identical.

The fixture includes copy, type and turn-memory tables. Its generated reply did
not produce a permeable row, so the production path for `permeable` priority
is covered by a unit case rather than this synthetic reply.

Real sealed arm-r seed-2 artifact
`a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`
on `samples-p-greedy-memory.json`
(`53d966dcfd63ff71e4b2d30f9af0fdbdc0607ca6c6d96c7d09aa4fb37e99a2e2`):

- 1,141 prompts, 8,459 generated-token rows, 462 EOS stops;
- **2,353 permeable**, **6,106 reflective**, **0 opaque** rows;
- 2,353 turn-memory moves stored a token; all 8,459 rows had a memory move;
- 41 rows entered a short cycle;
- EOS was top-1 on 462 rows and top-8 on 566;
- 0 point-key overrides and 0 zero nearest margins;
- trace SHA-256:
  `163fbbf00775b8a9e80656d09db8917208a280aab2edd7671852cea6593e41fb`;
- traced and independent untraced reply files both hash to
  `5a1844d10d12e6347be88993ca5b3f588d19d61dd722fbed1a8d8c28c11f1c16`
  and compare byte-identically.

## Checks run at the head

- Instrument tests with `integer-reader`: 7 passed, 0 failed.
- Full `uor-r4-integer` library with `serving-wasm`: 523 passed, 0 failed,
  1 ignored.
- The synthetic traced-versus-untraced reply is id-for-id and file-for-file
  identical.
- The real arm-r traced/untraced reply files are byte-identical and share the
  hash above.
- Instrument Clippy with `-D warnings`: pass.
- Integer Clippy exits 0 and reports 61 pre-existing warnings, none in the
  changed session/generation/mod lines.
- `git diff --check`: pass.
- Report check and local PR check run after the final committed head.

## Skills used

`working-discipline` kept the trace observation-only and used the shared
serving stop rule. `solution-integrity` required a product-token selection
guard, an untraced replay guard and a real synthetic artifact path, not only
field-format tests.

## Still open

- Use the real trace only to choose a later bake-off controller; this REPORT
  makes no claim that any margin, EOS rank or class predicts a complete reply.
- Add the pre-registered controller comparison in the separate bake-off unit.

## Plain summary

The reply instrument can now explain each generated token: how close the
nearest and runner-up writer rows were, where EOS ranked, whether memory or a
copy/type read fired, whether a short cycle had formed, and where the token sat
in the reply. It labels the route as opaque, reflective or permeable. The
instrument refuses its own result if the reconstructed choice differs from the
product's choice or if tracing changes any reply id.
