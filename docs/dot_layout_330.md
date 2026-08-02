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

On the local arm64 TLA6 fixture (`smollm2-135m-instruct-tla6`, 5,000 release
iterations), the complete assignment benchmark measured:

| layout | assignment ns/token |
| --- | ---: |
| previous two-term load layout | 159,189 |
| compact one-term load layout | 59,556 |

That is approximately a 2.67× wall-clock improvement on this fixture, clearing
the original ≥2× end-to-end criterion. The improvement comes from selecting
the compact representation for the actual high-byte-first one-term artifact;
the inactive term and mask traffic had previously kept the legacy 192-byte
vector layout on the hot path. The ≥3× isolated criterion still needs a
separate measurement/design decision.

No multiply, divide, float, allocation, or GPU operation enters the deployed
prediction path.
