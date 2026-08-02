# Compact one-term dot layout (#330)

The deployed TLA6/TLA7 artifact keeps two packed power-of-two term bytes per
dot entry for ABI compatibility. Since the Phase C compiler emits one active
term, the old SIMD load-time expansion retained an inactive term plus its
active mask for every dimension/class group.

The runtime load layout now has two representations:

- current one-term tables use a 64-byte `DotSingleVector` containing only four
  decoded shifts and four sign masks; the active term is selected from either
  packed byte, matching the compiler's high-byte-first encoding;
- legacy two-term tables retain the 192-byte `DotVector` representation and
  both decoded terms.

Both layouts remain dimension-major (`D × K/4`) so SIMD lanes still represent
adjacent classes. The on-disk `u16` dot-table ABI is unchanged, and the scalar
path remains the reference for exact output equality.

The improvement comes from selecting the compact representation for the actual
high-byte-first one-term artifact; the inactive term and mask traffic had
previously kept the legacy 192-byte vector layout on the hot path.

## Measurement

Isolated scans are timed through a measurement-only surface behind the
`bench-internals` feature (`simd::bench`), which exposes the built dot tables
read-only and reports which layout load-time selection chose. Nothing in it
participates in the deployed prediction path.

```
cargo bench --features bench-internals --bench transformerless_dot -p uor-r4-core \
  -- <artifact> 20000
```

Local arm64 (M4 Max), release, min-floor over 8 runs at 20,000 iterations on an
otherwise-idle machine (per-metric spread ≤2%), TLA6 fixture
`smollm2-135m-instruct-tla6`. All three scans cover identical work — one full
K×D scan per stage, 1024 class rows — in a single process, so they share
fixture, build, and machine state:

| isolated dot scan | ns/scan | ns/row | vs scalar |
| --- | ---: | ---: | ---: |
| scalar reference (`dot_score_plain`) | 211,712 | 206.7 | 1.00× |
| legacy two-term SIMD layout | 142,213 | 138.9 | 1.49× |
| compact one-term SIMD layout | 55,397 | 54.1 | **3.82×** |

Complete assignment (`assign_window`, compact layout): **57,425 ns/token**.

The checked-in fixture (`crates/uor-r4-core/tests/fixtures/tless_artifacts.bin`,
no external path required) reproduces this independently — min-floor over 6 runs
at 20,000 iterations, `simd_dot_layout=compact`:

| isolated dot scan | ns/scan | vs scalar |
| --- | ---: | ---: |
| scalar reference | 213,546 | 1.00× |
| legacy two-term SIMD layout | 144,770 | 1.48× |
| compact one-term SIMD layout | 56,284 | **3.79×** |

Complete assignment: 59,032 ns/token. Layout delta 2.57×, matching the
`smollm2-135m-instruct-tla6` figure to two significant figures.

### Against the #330 criteria

- **≥3× isolated: met — 3.82× (3.79× on the checked-in fixture).** The
  baseline is `dot_score_plain`, the uninstrumented scalar reference —
  deliberately *not* the scalar fallback in `Runtime::dot_argmax`, which routes
  every table entry through the op-census kernel (~295k counter increments per
  token) and would inflate the baseline. The SIMD side additionally performs
  the argmax compare that the scalar loop omits, so the ratio is conservative.
- **≥2× end-to-end: met, ≥3.69×.** A directly-measured scalar *serving*
  baseline is unavailable for the reason above, so this is stated as a bound:
  the dot scan is ~95% of steady-state serving cost (op-census figure from
  #330), so scalar serving is at least the 211,712 ns scalar scan, against
  57,425 ns/token measured here. Taking the 95% figure at face value gives
  3.88×; the 3.69× floor assumes the remaining ~5% is free.
- **Layout delta: 2.57×** (legacy 142,213 → compact 55,397) — the portion
  attributable to the compact layout rather than to #334's adapter. Both
  layouts are built from the same packed artifact in one process, and the
  benchmark asserts they agree class-for-class before timing either.

Two corroborations: the legacy layout's 1.49× isolated ratio independently
lands near #334's reported ~1.5×, and compact end-to-end minus compact isolated
leaves ~2.0µs of per-token bundling. An earlier single 5,000-iteration sample
reported 159,189 ns/token for the previous layout and 59,556 for the compact
one; those figures are superseded — the first implies ~17µs of bundling for the
same work and was taken on a contended machine.

Existing scalar/SIMD assignment witnesses remain the semantic-equality check;
the scalar path stays the reference for exact output equality.

No multiply, divide, float, allocation, or GPU operation enters the deployed
prediction path.
