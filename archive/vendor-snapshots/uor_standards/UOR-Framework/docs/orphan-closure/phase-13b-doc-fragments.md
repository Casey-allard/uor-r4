<!-- @generated-marker: this file hosts Phase 13b doc-fragment markers -->

# Phase 13b — doc-fragment library

This Markdown file hosts Phase-doc rustdoc fragments that the codegen's
[`emit::load_doc_fragment`](../../codegen/src/emit.rs) helper resolves
into emitted rustdoc strings. The marker format is:

```text
<!-- doc-key: {kind}:{name} -->
The Markdown paragraph(s) of rustdoc content. Plain Markdown — no
`//!` / `///` prefixes. Whitespace at start/end is trimmed when
the fragment is read.
<!-- /doc-key -->
```

Each fragment ends at the **first** of: the next `<!-- doc-key: ... -->`
marker, a Markdown heading (`##` or `###`), the explicit
`<!-- /doc-key -->` terminator, or end-of-file.

The Phase 13b migration moves codegen call sites to
`load_doc_fragment` incrementally; new emissions added after Phase 13b
ship use this helper directly.

## Reference fragments — verified by `phase13b_doc_fragments.rs`

<!-- doc-key: phase-13b:hello -->
This fragment is referenced by the Phase 13b helper test. The test
loads `phase-13b:hello` and asserts equality against the literal
`hello world` content recorded below.

hello world
<!-- /doc-key -->

<!-- doc-key: phase-13b:multiline -->
Multiline fragments preserve internal blank lines and Markdown
formatting. The terminator below is explicit so the fragment can
include heading-shaped content (`#` not at column 0) without
prematurely ending.

```
fenced code preserved verbatim
```

end-of-multiline
<!-- /doc-key -->

## Migration registry — phase-doc keys per emission kind

The Phase 13b spec lists eleven emission kinds, each addressed by a
`{kind}:{name}` key. The migration adds fragments incrementally as
emission sites adopt `load_doc_fragment`:

| Kind | Phase | Example key |
|---|---|---|
| `class` | 2 | `class:CarryChain` |
| `trait` | 2 | alias for `class` |
| `handle` | 8 | `handle:CarryChain` |
| `resolver` | 8 | `resolver:CarryChain` |
| `record` | 8 | `record:CarryChain` |
| `resolved` | 8 | `resolved:CarryChain` |
| `null-stub` | 7 / 7d | `null-stub:CarryChain` |
| `witness` | 10 | `witness:BornRuleVerification` |
| `mint-inputs` | 10 | `mint-inputs:BornRuleVerification` |
| `blanket-impl` | 11 | `blanket-impl:LandauerBudget` |
| `primitive` | 12 | `primitive:QM_5` |

The migration's red test
[codegen/tests/no_hand_written_rustdoc.rs](../../codegen/tests/no_hand_written_rustdoc.rs)
will gate hand-written rustdoc out of new codegen emissions once every
existing site has moved.
