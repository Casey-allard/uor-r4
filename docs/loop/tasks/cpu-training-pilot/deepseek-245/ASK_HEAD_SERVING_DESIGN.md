# Ask-head export and serving design gate

Status: proposal only. No artifact field or serving path is implemented by this
unit.

## Precondition

Do not implement this format unless Claude-330's B2 probe demonstrates that ask
heads can select the target fact. A positive B1 or B2-aq reading alone does not
authorize a permanent artifact representation.

## Float checkpoint source

Claude-330 currently records one ask table per Hamming depth in
`hamming_read.safetensors`:

- `d{depth}.ask`: F32 `(vocab, width)`, initialized to zero;
- `d{depth}.key_rotation`: the rotation shared by ask keys and queries;
- `d{depth}.margin`: copy margins in the first half and learned ask margins in
  the second;
- `d{depth}.weight`: copy weights in row 0 and ask/type weights in row 1.

For ask head `h`, token point `p(x)` and kept position `s`, the serving
operation is:

```text
query = sign(rotate(key_rotation, p(x_t) + ask[x_t]))
key   = sign(rotate(key_rotation, p(x_{s-1})))
value = p(x_s)
```

The ask heads use the copy-form payload and the type weight row. Copy heads and
all non-ask artifacts remain unchanged.

## Proposed integer representation

When and only when B2 passes, extend `StackHamming` with an explicit ask
marker and exponent, for example:

```text
ask_heads: true
ask_exp: -12
```

and add `hamming.d{depth}.ask`, an `i16` table of `vocab * width` Q12 values.
The fields must be absent for ordinary artifacts so existing artifacts remain
byte-identical.

The query point is formed in `i32` as `i32::from(table[token]) +
i32::from(ask[token])`, then widened to `i64` for the existing sign-code
rotation. It must not wrap or alias back into `i16`. The key path does not read
the ask table.

Rejected alternatives:

- storing ask offsets as ordinary type-head residuals: loses the token-keyed
  table and changes the mechanism;
- storing them at the rotation exponent: conflates a point offset with the
  quaternion rotation scale;
- storing F32 offsets in serving: breaks the integer/deterministic contract;
- adding unknown fields without a versioned refusal path: old readers could
  silently reinterpret ask tables as type reads.

## Fail-closed invariants

- Ask tables are present at every declared depth or at none.
- Every ask table is `vocab * width`, and its width/head shape is valid.
- Every ask value and Q12 export is finite and within the declared integer
  range.
- `heads` is even; ask heads are the second half; head width is a whole
  quaternion block of at most 64 coordinates.
- An ask artifact never carries the retired type-spin signal.
- A reader without ask support refuses the marker rather than ignoring the
  table.
- An ask reader refuses a marker with missing tables, a missing exponent, or
  tables at only some depths.
- Existing key-1, key-2 and mixed-copy-key artifacts serialize without new
  fields and remain byte-identical.

## Required parity gates

1. **Export identity:** round-trip the F32 ask table to Q12, reload it, and
   report every saturated or out-of-range value rather than clipping silently.
2. **Code parity:** for every ask head and depth, compare the float checkpoint,
   exported artifact and integer session on query code, key code, Hamming
   distance, kept positions and selector weight.
3. **Decision parity:** compare complete model rows with ask enabled and the
   same exported artifact under the independent CPU forward.
4. **Default parity:** artifact bytes, query/key codes, picks and outputs with
   ask off must equal `main` before the change.
5. **Refusal controls:** missing one depth, wrong shape, non-finite input,
   overflow, a type-spin/ask combination and an unsupported old reader must
   each fail closed.
6. **Mechanism gate:** the B2 result must be positive before any of these gates
   can authorize serving integration.

## Next implementation split

If B2 passes:

1. float export and Q12 quantization;
2. integer schema and reader;
3. integer ask query/key/payload path;
4. CPU oracle and served-forward audits;
5. language-run export and complete-reply comparison.

If B2 fails, retain this document as a design record and do not add artifact
surface.
