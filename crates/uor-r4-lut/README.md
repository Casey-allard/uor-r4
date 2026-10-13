# UOR-R4 LUT artifact metadata

The active crate is a **metadata-only facade under D29**. It preserves the existing
artifact schemas/container read-write API, GROUP32, retained exponent constants
and exact grid/arcosh metadata helpers. It compiles no historical engine,
sampling, normalized-exponential table law or dense SIMD projection, and
registers no `lut-chat` executable. No feature reactivates those paths.

The complete original source and README are retained under [historical](../../docs/history/d29-excluded-lut-2026-10-13/)
as inactive evidence. The exact boundary is documented in [UOR-PATCH.md](UOR-PATCH.md).
This source exclusion is not a serving qualification, model result or training
release. The global training and separate M2 holds remain.
