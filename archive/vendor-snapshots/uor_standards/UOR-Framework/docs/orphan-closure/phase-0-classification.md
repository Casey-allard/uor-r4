# Phase 0: Classification table

## What it does

Classifies every class in `Ontology::full()` into exactly one `PathKind`.
The classification drives every subsequent phase's codegen.

## Decision procedure

Ordered — first match wins:

1. **`Skip`** — the class local name is in
   [`Ontology::enum_class_names()`](../../spec/src/model.rs), or is
   `Primitives`. These classes don't become traits in
   `foundation/src/` so they can't be orphans.

2. **`AlreadyImplemented`** — the class already has a hand-written
   concrete impl in `foundation/src/enforcement.rs` or elsewhere in
   the workspace. Explicit allow-list in
   [`codegen/src/classification.rs`](../../codegen/src/classification.rs);
   the list is cross-checked at test time against `impl Certificate`
   and `impl VerifiedMint` grep results.

3. **`Path4TheoryDeferred`** — explicit allow-list derived from the
   strategy doc §Path 4: cohomology machinery (`CochainComplex`,
   `CohomologyGroup`, `Sheaf`, `RestrictionMap`), operad/monoidal
   (`MonoidalProduct`), every class in `kernel/parallel` and
   `kernel/stream`. These are genuine research gaps; they become
   orphans by design until theory lands.

4. **`Path2TheoremWitness { entropy_bearing }`** — the class name
   matches `/Witness$|Obstruction$|Verification$|Bound$/`, OR the
   class has a property whose range is an `op:Identity` individual
   in the `PT_`/`ST_`/`CPT_`/`OB_P_` families. `entropy_bearing` is
   `true` iff any property has range `xsd:decimal` OR a name in
   `{bits, bitsDissipated, landauerCost, entropy, crossEntropy,
   freeEnergy}` (R7).

5. **`Path3PrimitiveBacked { primitive_name }`** — explicit allow-list
   of (class, primitive function name) pairs. R13: the primitive
   function must exist in `foundation/src/enforcement.rs` at
   classification time; if not, classification fails loud. Phase 0's
   initial allow-list is empty — Phase 4 populates it as blanket impls
   are added.

6. **`Path1HandleResolver`** — fallthrough. The R4 check runs here:
   every property's range must map to a known "absent sentinel"
   (EMPTY_DECIMAL / EMPTY_HOST_STRING / zero fingerprint / etc.).
   Classes with at least one property whose range has no absent
   sentinel fall through to `Path4TheoryDeferred` with rationale
   `"no-absent-semantics: {type}"`.

## Outputs

- **`PathKind`** enum in `codegen/src/classification.rs`.
- **`classify(class, ontology) -> ClassificationEntry`** — pure function,
  deterministic.
- **`classify_all(ontology) -> Vec<ClassificationEntry>`** — batch.
- **`write_report(entries, out_path)`** — emits
  `docs/orphan-closure/classification_report.md`, one row per class,
  sorted by namespace then class name. Columns: class IRI, namespace,
  PathKind, rationale, entropy_bearing, theorem_identity, primitive_name.

## Counts registered in `spec/src/counts.rs`

- `PATH1_COUNT`, `PATH2_COUNT`, `PATH3_COUNT`, `PATH4_COUNT`
- `ALREADY_IMPLEMENTED_COUNT`
- `SKIP_COUNT`
- `CLASSES == PATH1 + PATH2 + PATH3 + PATH4 + ALREADY_IMPLEMENTED + SKIP`
  (enforced by the `classification_counts` test).

## Tests

- **`classification_coverage`** — every class in `Ontology::full()`
  gets a non-`None` classification; total == 471; determinism
  (calling `classify()` twice returns the same result).
- **`classification_counts`** — per-variant counts match
  `spec/src/counts.rs`. Drift between ontology and classification
  fails this test.
- **`classification_spot_checks`** — hand-picked sentinels:
  - `Partition` → `Path1HandleResolver`
  - `PartitionProduct` → `AlreadyImplemented`
  - `CochainComplex` → `Path4TheoryDeferred`
  - `GluingObstruction` → `Path2TheoremWitness`
  - `WittLevel` → `Skip`

## Where Path-0 is called

`codegen/src/lib.rs::generate()` runs classification before any emission
and calls `classification::write_report(&entries, out_dir)` after all
other generators. The report lives at
`docs/orphan-closure/classification_report.md` and is `git
diff --exit-code`-gated in CI.
