//! ONNX typed input (IR ≤ v13) (ADR-023 amended by ADR-060).
//!
//! Protobuf v3 admits many byte-representations of the same logical
//! message; this realization defines a canonical form — a **flat
//! skeleton** — that collapses that freedom. Two ONNX models that decode
//! to the same logical content (regardless of protobuf field order, node
//! ordering among valid topological orderings, or whether tensor data is
//! stored in `raw_data` or the typed-data fields) canonicalize to
//! byte-identical skeletons and therefore to the same κ-label.
//!
//! ```text
//! LE_i64(ir_version)
//! ── opset imports, sorted by (domain, version) ──
//!   for op: sha256(domain) || LE_i64(version)
//! ── graph (recursive) ──
//!   sha256(graph_name)
//!   nodes in Kahn-topological order (lex (name, op_type, domain) tie-break):
//!     sha256(name) || sha256(op_type) || sha256(domain) || sha256(overload)
//!       || LE_u32(n_in)  || (sha256(input_name)  × n_in)
//!       || LE_u32(n_out) || (sha256(output_name) × n_out)
//!       || attributes, sorted by name (GRAPH/GRAPHS recurse inline)
//!   initializers (#5), sorted by name, each a canonical TensorProto record
//!   graph input (#11) / output (#12) / value_info (#13), sorted by name
//! ── model metadata ──
//!   sha256(producer_name) || sha256(producer_version) || sha256(domain)
//!     || LE_i64(model_version) || metadata_props sorted by key
//! ```
//!
//! Under ADR-060 the **full skeleton** flows through the pipeline as a
//! [`TermValue::Borrowed`] carrier and ψ₉ folds it through the σ-axis —
//! there is no two-level commitment, no carrier ceiling, and no node /
//! attribute / initializer / IO count cap. Variable-length leaves (tensor
//! data bytes, strings, opaque sub-message payloads) are still replaced by
//! their `sha256(...)` digest so the skeleton stays bounded by structure
//! size, not data size, while still binding every weight byte into the
//! κ-label.
//!
//! [`OnnxValue`] (the owned parsed value, `alloc`-gated) holds the
//! skeleton; [`OnnxCarrier`] is the borrowed model-input handle the
//! pipeline binds.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};

// ─── OnnxCarrier — the borrowed model-input handle (no_alloc) ───────────

/// Borrowed canonical-skeleton input handle (ADR-060 borrowed carrier). A
/// thin, `Copy` borrow of the skeleton bytes produced by [`canonicalize`];
/// `as_binding_value` returns the `Borrowed` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct OnnxCarrier<'a>(&'a [u8]);

impl<'a> OnnxCarrier<'a> {
    /// Wrap a canonical-skeleton byte slice as a model input handle.
    #[must_use]
    pub fn new(skeleton: &'a [u8]) -> Self {
        Self(skeleton)
    }

    /// Borrow the canonical-skeleton bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for OnnxCarrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/OnnxValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for OnnxCarrier<'_> {}

impl<'a> IntoBindingValue<'a> for OnnxCarrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for OnnxCarrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ═════════════════════════════════════════════════════════════════════
// alloc-gated parser + owned value
// ═════════════════════════════════════════════════════════════════════

#[cfg(feature = "alloc")]
pub use alloc_impl::{canonicalize, OnnxValue};

#[cfg(feature = "alloc")]
mod alloc_impl {
    use alloc::vec::Vec;

    use prism::crypto::Sha256Hasher;
    use prism::pipeline::{ShapeViolation, ViolationKind};
    use prism::vocabulary::Hasher;

    use crate::onnx::dtype::OnnxDataType;
    use crate::onnx::protobuf::{read_varint, FieldValue, MessageReader};
    use crate::onnx::shapes::bounds::{
        ONNX_IR_VERSION_MAX, ONNX_OPSET_VERSION_MIN, ONNX_SUBGRAPH_DEPTH_MAX,
    };

    // ─── ShapeViolation IRIs ─────────────────────────────────────────────

    macro_rules! violation {
        ($name:ident, $constraint:literal, $kind:expr) => {
            const $name: ShapeViolation = ShapeViolation {
                shape_iri: "https://uor.foundation/addr/OnnxValue",
                constraint_iri: concat!("https://uor.foundation/addr/OnnxValue/", $constraint),
                property_iri: concat!("https://uor.foundation/addr/OnnxValue/", $constraint),
                expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
                min_count: 0,
                max_count: 1,
                kind: $kind,
            };
        };
    }

    violation!(PROTOBUF_FAILURE, "validProtobuf", ViolationKind::ValueCheck);
    violation!(
        UNSUPPORTED_IR,
        "supportedIrVersion",
        ViolationKind::ValueCheck
    );
    violation!(OPSET_TOO_OLD, "opsetVersionMin", ViolationKind::ValueCheck);
    violation!(MISSING_GRAPH, "graphPresent", ViolationKind::ValueCheck);
    violation!(
        SUBGRAPH_DEPTH,
        "subgraphDepthBound",
        ViolationKind::CardinalityViolation
    );
    violation!(GRAPH_CYCLE, "acyclicGraph", ViolationKind::ValueCheck);
    violation!(
        UNKNOWN_DTYPE,
        "knownTensorDataType",
        ViolationKind::ValueCheck
    );

    fn from_wire(_e: crate::onnx::protobuf::WireError) -> ShapeViolation {
        PROTOBUF_FAILURE
    }

    #[inline]
    fn sha256(bytes: &[u8]) -> [u8; 32] {
        Sha256Hasher::initial().fold_bytes(bytes).finalize()
    }

    /// Recursion ceiling for the opaque-message field-order canonicalizer.
    const CANON_PROTO_DEPTH_MAX: usize = 32;

    /// Field-order-canonical digest of an opaque protobuf message — folds
    /// its fields in ascending field-number order (stable within a number,
    /// so repeated-field order is preserved), recursing into
    /// length-delimited fields. This applies canonicalization rule 1
    /// (field-number ordering) to sub-messages the realization otherwise
    /// treats opaquely (`TypeProto`, `SparseTensorProto`), so two
    /// serializations of the same logical value canonicalize identically.
    /// Returns a 32-byte **leaf digest** (an opaque sub-message is a leaf
    /// — its digest is appended inline, never expanded into the skeleton).
    ///
    /// A length-delimited field that is genuinely a string / bytes leaf
    /// (e.g. `dim_param`) generally fails to re-parse as a well-formed
    /// message; that case falls back to a digest of the raw payload. The
    /// transform is deterministic either way: identical bytes always take
    /// the same path.
    fn canonical_proto_digest(body: &[u8], depth: usize) -> Result<[u8; 32], ShapeViolation> {
        #[derive(Clone, Copy)]
        struct F {
            number: u64,
            wt: u8,
            off: usize,
            len: usize,
            val: u64,
        }
        let mut fs: Vec<F> = Vec::new();
        let mut r = MessageReader::new(body);
        while let Some(f) = r.next_field().map_err(from_wire)? {
            fs.push(match f.value {
                FieldValue::Varint(v) => F {
                    number: f.number,
                    wt: 0,
                    off: 0,
                    len: 0,
                    val: v,
                },
                FieldValue::Fixed64(v) => F {
                    number: f.number,
                    wt: 1,
                    off: 0,
                    len: 0,
                    val: v,
                },
                FieldValue::Fixed32(v) => F {
                    number: f.number,
                    wt: 5,
                    off: 0,
                    len: 0,
                    val: u64::from(v),
                },
                FieldValue::Bytes(b) => F {
                    number: f.number,
                    wt: 2,
                    off: b.as_ptr() as usize - body.as_ptr() as usize,
                    len: b.len(),
                    val: 0,
                },
            });
        }
        // Stable sort by field number (preserves repeated-field order).
        fs.sort_by_key(|f| f.number);

        let mut h = Sha256Hasher::initial();
        for f in fs.iter() {
            fold(&mut h, &f.number.to_le_bytes());
            fold(&mut h, &[f.wt]);
            match f.wt {
                0 | 1 => fold(&mut h, &f.val.to_le_bytes()),
                5 => fold(&mut h, &(f.val as u32).to_le_bytes()),
                _ => {
                    let payload = &body[f.off..f.off + f.len];
                    let sub = if depth < CANON_PROTO_DEPTH_MAX && !payload.is_empty() {
                        canonical_proto_digest(payload, depth + 1)
                            .unwrap_or_else(|_| sha256(payload))
                    } else {
                        sha256(payload)
                    };
                    fold(&mut h, &sub);
                }
            }
        }
        Ok(h.finalize())
    }

    /// Fold `bytes` into the running hasher behind a mutable reference (the
    /// `Hasher::fold_bytes` consume-and-return API is awkward inside
    /// `FnMut` closures; this wraps the take-replace dance once).
    #[inline]
    fn fold(h: &mut Sha256Hasher, bytes: &[u8]) {
        let cur = core::mem::replace(h, Sha256Hasher::initial());
        *h = cur.fold_bytes(bytes);
    }

    // ─── Protobuf field accessors over a message body ──────────────────

    /// First occurrence of `field_no` in `body`, or `None`.
    fn first_field(body: &[u8], field_no: u64) -> Result<Option<FieldValue<'_>>, ShapeViolation> {
        let mut r = MessageReader::new(body);
        while let Some(f) = r.next_field().map_err(from_wire)? {
            if f.number == field_no {
                return Ok(Some(f.value));
            }
        }
        Ok(None)
    }

    fn first_varint(body: &[u8], field_no: u64) -> Result<Option<u64>, ShapeViolation> {
        Ok(match first_field(body, field_no)? {
            Some(FieldValue::Varint(v)) => Some(v),
            _ => None,
        })
    }

    fn first_bytes(body: &[u8], field_no: u64) -> Result<&[u8], ShapeViolation> {
        Ok(match first_field(body, field_no)? {
            Some(FieldValue::Bytes(b)) => b,
            _ => &[],
        })
    }

    /// Invoke `f` for every occurrence of `field_no` (the repeated-field
    /// iterator). Stops and propagates the first error `f` returns.
    fn for_each_field(
        body: &[u8],
        field_no: u64,
        mut f: impl FnMut(FieldValue<'_>) -> Result<(), ShapeViolation>,
    ) -> Result<(), ShapeViolation> {
        let mut r = MessageReader::new(body);
        while let Some(field) = r.next_field().map_err(from_wire)? {
            if field.number == field_no {
                f(field.value)?;
            }
        }
        Ok(())
    }

    fn count_field(body: &[u8], field_no: u64) -> Result<usize, ShapeViolation> {
        let mut n = 0;
        for_each_field(body, field_no, |_| {
            n += 1;
            Ok(())
        })?;
        Ok(n)
    }

    /// A `(offset, len)` span into a parent buffer.
    #[derive(Clone, Copy)]
    struct Span {
        off: usize,
        len: usize,
    }

    /// Collect every occurrence of `field_no` (length-delimited) in `body`
    /// as a `(offset, len)` span.
    fn collect_spans(body: &[u8], field_no: u64) -> Result<Vec<Span>, ShapeViolation> {
        let mut spans: Vec<Span> = Vec::new();
        let mut r = MessageReader::new(body);
        while let Some(f) = r.next_field().map_err(from_wire)? {
            if f.number == field_no {
                if let FieldValue::Bytes(b) = f.value {
                    spans.push(Span {
                        off: b.as_ptr() as usize - body.as_ptr() as usize,
                        len: b.len(),
                    });
                }
            }
        }
        Ok(spans)
    }

    /// A parsed, canonicalized ONNX `ModelProto`. The stored bytes are the
    /// flat canonical skeleton (see [module docs](super)). **`alloc`-gated**
    /// — the pipeline binds the borrowed [`OnnxCarrier`](super::OnnxCarrier).
    #[derive(Clone, PartialEq, Eq)]
    pub struct OnnxValue {
        bytes: Vec<u8>,
    }

    impl core::fmt::Debug for OnnxValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("OnnxValue")
                .field("canonical_len", &self.bytes.len())
                .finish_non_exhaustive()
        }
    }

    impl OnnxValue {
        /// Borrow the canonical-skeleton bytes.
        #[must_use]
        pub fn canonical_bytes(&self) -> &[u8] {
            &self.bytes
        }

        /// Parse an ONNX `ModelProto` wire buffer into a canonicalized
        /// skeleton.
        ///
        /// # Errors
        ///
        /// A [`ShapeViolation`] whose `constraint_iri` names the violated
        /// invariant (protobuf decode failure, unsupported IR version,
        /// opset below the minimum, missing graph, a subgraph cycle, an
        /// over-deep subgraph nesting, or an unknown tensor data type).
        pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
            let mut out: Vec<u8> = Vec::new();

            // ── ir_version (ModelProto #1) ──
            // Accept any known IR revision (1..=ONNX_IR_VERSION_MAX); the
            // canonical skeleton is IR-version-agnostic and binds the
            // `ir_version` value, so distinct revisions canonicalize
            // distinctly. Reject absent / 0 / a future unknown revision.
            let ir_version = first_varint(raw, 1)?.ok_or(UNSUPPORTED_IR)? as i64;
            if !(1..=ONNX_IR_VERSION_MAX).contains(&ir_version) {
                return Err(UNSUPPORTED_IR);
            }
            out.extend_from_slice(&ir_version.to_le_bytes());

            // ── opset imports (ModelProto #8, repeated OperatorSetIdProto) ──
            emit_opsets(&mut out, raw)?;

            // ── graph (ModelProto #7) ──
            let graph = first_bytes(raw, 7)?;
            if graph.is_empty() {
                return Err(MISSING_GRAPH);
            }
            emit_canonical_graph(&mut out, graph, 0)?;

            // ── model metadata ──
            emit_model_meta(&mut out, raw)?;

            Ok(Self { bytes: out })
        }
    }

    /// Emit opset imports sorted by `(domain, version)`. Enforces at least
    /// one default-domain (`""`) import at or above
    /// [`ONNX_OPSET_VERSION_MIN`].
    fn emit_opsets(out: &mut Vec<u8>, model: &[u8]) -> Result<(), ShapeViolation> {
        let entries = collect_spans(model, 8)?;

        // Default-domain minimum-version check.
        let mut ok_min = false;
        for e in &entries {
            let body = &model[e.off..e.off + e.len];
            let domain = first_bytes(body, 1)?;
            let version = first_varint(body, 2)?.unwrap_or(0) as i64;
            if domain.is_empty() && version >= ONNX_OPSET_VERSION_MIN {
                ok_min = true;
            }
        }
        if !ok_min && !entries.is_empty() {
            return Err(OPSET_TOO_OLD);
        }

        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| {
            let ea = &model[entries[a].off..entries[a].off + entries[a].len];
            let eb = &model[entries[b].off..entries[b].off + entries[b].len];
            let ka = (
                first_bytes(ea, 1).unwrap_or(&[]),
                first_varint(ea, 2).ok().flatten().unwrap_or(0),
            );
            let kb = (
                first_bytes(eb, 1).unwrap_or(&[]),
                first_varint(eb, 2).ok().flatten().unwrap_or(0),
            );
            ka.cmp(&kb)
        });

        for &idx in &order {
            let body = &model[entries[idx].off..entries[idx].off + entries[idx].len];
            let domain = first_bytes(body, 1)?;
            let version = first_varint(body, 2)?.unwrap_or(0) as i64;
            out.extend_from_slice(&sha256(domain));
            out.extend_from_slice(&version.to_le_bytes());
        }
        Ok(())
    }

    /// Emit producer / domain / model_version + sorted `metadata_props`.
    fn emit_model_meta(out: &mut Vec<u8>, model: &[u8]) -> Result<(), ShapeViolation> {
        out.extend_from_slice(&sha256(first_bytes(model, 2)?)); // producer_name
        out.extend_from_slice(&sha256(first_bytes(model, 3)?)); // producer_version
        out.extend_from_slice(&sha256(first_bytes(model, 4)?)); // domain
        out.extend_from_slice(&(first_varint(model, 5)?.unwrap_or(0) as i64).to_le_bytes()); // model_version
        emit_string_string(out, model, 14) // metadata_props
    }

    /// Emit a repeated `StringStringEntryProto` map (`field_no`), sorted by
    /// key, inline: for each entry `sha256(key) || sha256(value)`.
    fn emit_string_string(
        out: &mut Vec<u8>,
        body: &[u8],
        field_no: u64,
    ) -> Result<(), ShapeViolation> {
        let entries = collect_spans(body, field_no)?;
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| {
            let ka = first_bytes(&body[entries[a].off..entries[a].off + entries[a].len], 1)
                .unwrap_or(&[]);
            let kb = first_bytes(&body[entries[b].off..entries[b].off + entries[b].len], 1)
                .unwrap_or(&[]);
            ka.cmp(kb)
        });
        out.extend_from_slice(&(order.len() as u32).to_le_bytes());
        for &idx in &order {
            let e = &body[entries[idx].off..entries[idx].off + entries[idx].len];
            out.extend_from_slice(&sha256(first_bytes(e, 1)?));
            out.extend_from_slice(&sha256(first_bytes(e, 2)?));
        }
        Ok(())
    }

    /// Emit a `GraphProto` body inline, recursing into subgraphs bounded by
    /// [`ONNX_SUBGRAPH_DEPTH_MAX`].
    fn emit_canonical_graph(
        out: &mut Vec<u8>,
        graph: &[u8],
        depth: usize,
    ) -> Result<(), ShapeViolation> {
        if depth > ONNX_SUBGRAPH_DEPTH_MAX {
            return Err(SUBGRAPH_DEPTH);
        }

        out.extend_from_slice(&sha256(first_bytes(graph, 2)?)); // graph name

        // ── Nodes in Kahn-topological order (lex tie-break) ──
        let nodes = collect_spans(graph, 1)?;
        let node_count = nodes.len();
        out.extend_from_slice(&(node_count as u32).to_le_bytes());

        let mut emitted: Vec<bool> = alloc::vec![false; node_count];
        for _ in 0..node_count {
            // Find the lex-min ready (all producers emitted), unemitted node.
            let mut best: Option<usize> = None;
            for (cand, node) in nodes.iter().enumerate() {
                if emitted[cand] {
                    continue;
                }
                if !node_ready(graph, &nodes, &emitted, node)? {
                    continue;
                }
                best = Some(match best {
                    None => cand,
                    Some(b) => {
                        if node_lex_le(graph, &nodes[cand], &nodes[b])? {
                            cand
                        } else {
                            b
                        }
                    }
                });
            }
            let pick = best.ok_or(GRAPH_CYCLE)?; // no ready node ⇒ cycle
            let body = &graph[nodes[pick].off..nodes[pick].off + nodes[pick].len];
            emit_node(out, body, depth)?;
            emitted[pick] = true;
        }

        // ── Initializers (#5), sorted by name, with tensor-data digests ──
        emit_tensor_section(out, graph, 5)?;

        // ── Graph IO: inputs (#11), outputs (#12), value_info (#13) ──
        emit_value_info(out, graph, 11)?;
        emit_value_info(out, graph, 12)?;
        emit_value_info(out, graph, 13)?;

        Ok(())
    }

    /// A node is ready when every input name that is *produced by another
    /// node in this graph* has had its producer emitted.
    fn node_ready(
        graph: &[u8],
        nodes: &[Span],
        emitted: &[bool],
        node: &Span,
    ) -> Result<bool, ShapeViolation> {
        let body = &graph[node.off..node.off + node.len];
        let mut ready = true;
        for_each_field(body, 1, |v| {
            if let FieldValue::Bytes(name) = v {
                if !name.is_empty() {
                    for (k, prod) in nodes.iter().enumerate() {
                        let pbody = &graph[prod.off..prod.off + prod.len];
                        let mut produces = false;
                        for_each_field(pbody, 2, |ov| {
                            if let FieldValue::Bytes(on) = ov {
                                if on == name {
                                    produces = true;
                                }
                            }
                            Ok(())
                        })?;
                        if produces && !emitted[k] {
                            ready = false;
                        }
                    }
                }
            }
            Ok(())
        })?;
        Ok(ready)
    }

    /// Lexicographic order on `(name, op_type, domain)`.
    fn node_lex_le(graph: &[u8], a: &Span, b: &Span) -> Result<bool, ShapeViolation> {
        let ba = &graph[a.off..a.off + a.len];
        let bb = &graph[b.off..b.off + b.len];
        let ka = (
            first_bytes(ba, 3)?,
            first_bytes(ba, 4)?,
            first_bytes(ba, 7)?,
        );
        let kb = (
            first_bytes(bb, 3)?,
            first_bytes(bb, 4)?,
            first_bytes(bb, 7)?,
        );
        Ok(ka <= kb)
    }

    /// Emit a `NodeProto` inline: identity fields, positional inputs /
    /// outputs, then the name-sorted attributes (which recurse into
    /// subgraphs inline).
    fn emit_node(out: &mut Vec<u8>, node: &[u8], depth: usize) -> Result<(), ShapeViolation> {
        out.extend_from_slice(&sha256(first_bytes(node, 3)?)); // name
        out.extend_from_slice(&sha256(first_bytes(node, 4)?)); // op_type
        out.extend_from_slice(&sha256(first_bytes(node, 7)?)); // domain
        out.extend_from_slice(&sha256(first_bytes(node, 8)?)); // overload (IR v10+)

        let n_in = count_field(node, 1)?;
        out.extend_from_slice(&(n_in as u32).to_le_bytes());
        for_each_field(node, 1, |v| {
            if let FieldValue::Bytes(name) = v {
                out.extend_from_slice(&sha256(name));
            }
            Ok(())
        })?;

        let n_out = count_field(node, 2)?;
        out.extend_from_slice(&(n_out as u32).to_le_bytes());
        for_each_field(node, 2, |v| {
            if let FieldValue::Bytes(name) = v {
                out.extend_from_slice(&sha256(name));
            }
            Ok(())
        })?;

        emit_attributes(out, node, depth)
    }

    /// Emit a node's `attribute` field (#5), sorted by name, inline.
    fn emit_attributes(out: &mut Vec<u8>, node: &[u8], depth: usize) -> Result<(), ShapeViolation> {
        let attrs = collect_spans(node, 5)?;
        let mut order: Vec<usize> = (0..attrs.len()).collect();
        order.sort_by(|&a, &b| {
            let na =
                first_bytes(&node[attrs[a].off..attrs[a].off + attrs[a].len], 1).unwrap_or(&[]);
            let nb =
                first_bytes(&node[attrs[b].off..attrs[b].off + attrs[b].len], 1).unwrap_or(&[]);
            na.cmp(nb)
        });
        out.extend_from_slice(&(order.len() as u32).to_le_bytes());
        for &idx in &order {
            let a = &node[attrs[idx].off..attrs[idx].off + attrs[idx].len];
            out.extend_from_slice(&sha256(first_bytes(a, 1)?)); // name
            let atype = first_varint(a, 20)?.unwrap_or(0) as i32;
            out.extend_from_slice(&atype.to_le_bytes());
            emit_attribute_value(out, a, atype, depth)?;
        }
        Ok(())
    }

    /// Emit an attribute's value inline, dispatched on its `AttributeType`.
    fn emit_attribute_value(
        out: &mut Vec<u8>,
        a: &[u8],
        atype: i32,
        depth: usize,
    ) -> Result<(), ShapeViolation> {
        match atype {
            1 => {
                // FLOAT (#2, fixed32)
                if let Some(FieldValue::Fixed32(bits)) = first_field(a, 2)? {
                    out.extend_from_slice(&bits.to_le_bytes());
                }
            }
            2 => {
                // INT (#3, varint)
                out.extend_from_slice(&(first_varint(a, 3)?.unwrap_or(0) as i64).to_le_bytes());
            }
            3 => {
                // STRING (#4, bytes)
                out.extend_from_slice(&sha256(first_bytes(a, 4)?));
            }
            4 => {
                // TENSOR (#5)
                emit_tensor(out, first_bytes(a, 5)?)?;
            }
            5 => {
                // GRAPH (#6) — recurse inline
                emit_canonical_graph(out, first_bytes(a, 6)?, depth + 1)?;
            }
            6 => {
                // FLOATS (#7, packed fixed32)
                for_each_field(a, 7, |v| {
                    if let FieldValue::Bytes(p) = v {
                        out.extend_from_slice(&sha256(p));
                    } else if let FieldValue::Fixed32(b) = v {
                        out.extend_from_slice(&b.to_le_bytes());
                    }
                    Ok(())
                })?;
            }
            7 => {
                // INTS (#8, packed varint)
                emit_packed_varints(out, a, 8)?;
            }
            8 => {
                // STRINGS (#9, repeated bytes)
                for_each_field(a, 9, |v| {
                    if let FieldValue::Bytes(s) = v {
                        out.extend_from_slice(&sha256(s));
                    }
                    Ok(())
                })?;
            }
            9 => {
                // TENSORS (#10)
                let spans = collect_spans(a, 10)?;
                for s in &spans {
                    emit_tensor(out, &a[s.off..s.off + s.len])?;
                }
            }
            10 => {
                // GRAPHS (#11) — recurse inline
                let spans = collect_spans(a, 11)?;
                for s in &spans {
                    emit_canonical_graph(out, &a[s.off..s.off + s.len], depth + 1)?;
                }
            }
            11 => out.extend_from_slice(&canonical_proto_digest(first_bytes(a, 22)?, 0)?), // SPARSE_TENSOR
            12 => {
                // SPARSE_TENSORS (#23)
                for_each_field(a, 23, |v| {
                    if let FieldValue::Bytes(s) = v {
                        out.extend_from_slice(&canonical_proto_digest(s, 0)?);
                    }
                    Ok(())
                })?;
            }
            13 => out.extend_from_slice(&canonical_proto_digest(first_bytes(a, 14)?, 0)?), // TYPE_PROTO
            14 => {
                // TYPE_PROTOS (#15)
                for_each_field(a, 15, |v| {
                    if let FieldValue::Bytes(s) = v {
                        out.extend_from_slice(&canonical_proto_digest(s, 0)?);
                    }
                    Ok(())
                })?;
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_packed_varints(
        out: &mut Vec<u8>,
        body: &[u8],
        field_no: u64,
    ) -> Result<(), ShapeViolation> {
        for_each_field(body, field_no, |v| {
            match v {
                FieldValue::Bytes(p) => {
                    let mut pos = 0;
                    while pos < p.len() {
                        let (val, np) = read_varint(p, pos).map_err(from_wire)?;
                        out.extend_from_slice(&(val as i64).to_le_bytes());
                        pos = np;
                    }
                }
                FieldValue::Varint(val) => out.extend_from_slice(&(val as i64).to_le_bytes()),
                _ => {}
            }
            Ok(())
        })
    }

    /// Emit a name-sorted section of repeated `TensorProto` (initializers).
    fn emit_tensor_section(
        out: &mut Vec<u8>,
        graph: &[u8],
        field_no: u64,
    ) -> Result<(), ShapeViolation> {
        let spans = collect_spans(graph, field_no)?;
        let mut order: Vec<usize> = (0..spans.len()).collect();
        order.sort_by(|&a, &b| {
            let na =
                first_bytes(&graph[spans[a].off..spans[a].off + spans[a].len], 8).unwrap_or(&[]);
            let nb =
                first_bytes(&graph[spans[b].off..spans[b].off + spans[b].len], 8).unwrap_or(&[]);
            na.cmp(nb)
        });
        out.extend_from_slice(&(order.len() as u32).to_le_bytes());
        for &idx in &order {
            let body = &graph[spans[idx].off..spans[idx].off + spans[idx].len];
            emit_tensor(out, body)?;
        }
        Ok(())
    }

    /// Emit a canonical `TensorProto` record inline: `sha256(name) ||
    /// LE_i32(dtype) || LE_u32(rank) || (LE_i64 dim …) || tensor_data_digest`,
    /// where `tensor_data_digest` is a 32-byte leaf digest streaming
    /// `raw_data` if present, else the typed-data field re-encoded to the
    /// canonical little-endian `raw_data` layout (so the two storage forms
    /// canonicalize identically).
    fn emit_tensor(out: &mut Vec<u8>, t: &[u8]) -> Result<(), ShapeViolation> {
        let dtype_id = first_varint(t, 2)?.unwrap_or(0) as i32;
        let dtype = OnnxDataType::from_i32(dtype_id).ok_or(UNKNOWN_DTYPE)?;

        out.extend_from_slice(&sha256(first_bytes(t, 8)?)); // name
        out.extend_from_slice(&dtype_id.to_le_bytes());

        // dims (#1, repeated int64; packed or unpacked).
        let rank = count_dims(t)?;
        out.extend_from_slice(&(rank as u32).to_le_bytes());
        emit_packed_varints(out, t, 1)?;

        // data digest (a leaf — appended inline as 32 bytes).
        out.extend_from_slice(&tensor_data_digest(t, dtype)?);
        Ok(())
    }

    fn count_dims(t: &[u8]) -> Result<usize, ShapeViolation> {
        let mut n = 0;
        for_each_field(t, 1, |v| {
            match v {
                FieldValue::Bytes(p) => {
                    let mut pos = 0;
                    while pos < p.len() {
                        let (_, np) = read_varint(p, pos).map_err(from_wire)?;
                        n += 1;
                        pos = np;
                    }
                }
                FieldValue::Varint(_) => n += 1,
                _ => {}
            }
            Ok(())
        })?;
        Ok(n)
    }

    /// Stream the tensor's data through SHA-256 in canonical `raw_data`
    /// layout, returning the 32-byte leaf digest.
    fn tensor_data_digest(t: &[u8], dtype: OnnxDataType) -> Result<[u8; 32], ShapeViolation> {
        // External data (`data_location` #14 == EXTERNAL = 1): the core
        // cannot open the referenced sibling file, so the κ-label binds the
        // external *reference* (`external_data` #13 — location / offset /
        // length / checksum, sorted by key) rather than the dereferenced
        // bytes. A domain tag keeps external digests disjoint from inline
        // ones. Hosts requiring inline≡external equivalence dereference
        // before calling.
        if first_varint(t, 14)?.unwrap_or(0) == 1 {
            let mut h = Sha256Hasher::initial();
            fold(&mut h, b"onnx:external-data:v1");
            // metadata-style sorted digest of external_data (#13).
            let mut sub: Vec<u8> = Vec::new();
            emit_string_string(&mut sub, t, 13)?;
            fold(&mut h, &sub);
            return Ok(h.finalize());
        }
        // raw_data (#9) takes precedence and is already canonical.
        if let Some(FieldValue::Bytes(raw)) = first_field(t, 9)? {
            if !raw.is_empty() {
                return Ok(sha256(raw));
            }
        }
        let mut h = Sha256Hasher::initial();
        match dtype {
            // float_data (#4) / double_data (#10): packed fixed-width — the
            // packed payload IS the canonical raw layout.
            OnnxDataType::Float => fold_fixed_payload(t, 4, &mut h)?,
            OnnxDataType::Double | OnnxDataType::Complex128 => fold_fixed_payload(t, 10, &mut h)?,
            OnnxDataType::Complex64 => fold_fixed_payload(t, 4, &mut h)?,
            // int64_data (#7): re-encode each varint to 8-byte LE.
            OnnxDataType::Int64 => fold_typed_varints(t, 7, 8, &mut h)?,
            // uint64_data (#11): UINT64 → 8-byte LE; UINT32 → 4-byte LE.
            OnnxDataType::Uint64 => fold_typed_varints(t, 11, 8, &mut h)?,
            OnnxDataType::Uint32 => fold_typed_varints(t, 11, 4, &mut h)?,
            // int32_data (#5) carries INT32/INT16/INT8/UINT16/UINT8/BOOL and
            // the bit-packed small floats — re-encode to the dtype's width.
            OnnxDataType::Int32 => fold_typed_varints(t, 5, 4, &mut h)?,
            OnnxDataType::Int16
            | OnnxDataType::Uint16
            | OnnxDataType::Float16
            | OnnxDataType::Bfloat16 => fold_typed_varints(t, 5, 2, &mut h)?,
            OnnxDataType::Int8
            | OnnxDataType::Uint8
            | OnnxDataType::Bool
            | OnnxDataType::Float8E4M3Fn
            | OnnxDataType::Float8E4M3Fnuz
            | OnnxDataType::Float8E5M2
            | OnnxDataType::Float8E5M2Fnuz
            | OnnxDataType::Int4
            | OnnxDataType::Uint4
            | OnnxDataType::Float4E2M1 => fold_typed_varints(t, 5, 1, &mut h)?,
            // string_data (#6): fold each element's digest.
            OnnxDataType::String => {
                for_each_field(t, 6, |v| {
                    if let FieldValue::Bytes(s) = v {
                        fold(&mut h, &sha256(s));
                    }
                    Ok(())
                })?;
            }
        }
        Ok(h.finalize())
    }

    /// Fold the (already-canonical) packed payload of a fixed-width
    /// repeated field directly.
    fn fold_fixed_payload(
        body: &[u8],
        field_no: u64,
        h: &mut Sha256Hasher,
    ) -> Result<(), ShapeViolation> {
        for_each_field(body, field_no, |v| {
            match v {
                FieldValue::Bytes(p) => fold(h, p),
                FieldValue::Fixed32(b) => fold(h, &b.to_le_bytes()),
                FieldValue::Fixed64(b) => fold(h, &b.to_le_bytes()),
                _ => {}
            }
            Ok(())
        })
    }

    /// Re-encode each varint of a packed/unpacked repeated field to `width`
    /// little-endian bytes (the canonical `raw_data` element layout).
    fn fold_typed_varints(
        body: &[u8],
        field_no: u64,
        width: usize,
        h: &mut Sha256Hasher,
    ) -> Result<(), ShapeViolation> {
        for_each_field(body, field_no, |v| {
            match v {
                FieldValue::Bytes(p) => {
                    let mut pos = 0;
                    while pos < p.len() {
                        let (val, np) = read_varint(p, pos).map_err(from_wire)?;
                        fold(h, &val.to_le_bytes()[..width]);
                        pos = np;
                    }
                }
                FieldValue::Varint(val) => fold(h, &val.to_le_bytes()[..width]),
                _ => {}
            }
            Ok(())
        })
    }

    /// Emit a name-sorted section of repeated `ValueInfoProto` (graph
    /// input / output / value_info). Binds the name plus a field-order-
    /// canonical leaf digest of the `TypeProto`.
    fn emit_value_info(
        out: &mut Vec<u8>,
        graph: &[u8],
        field_no: u64,
    ) -> Result<(), ShapeViolation> {
        let spans = collect_spans(graph, field_no)?;
        let mut order: Vec<usize> = (0..spans.len()).collect();
        order.sort_by(|&a, &b| {
            let na =
                first_bytes(&graph[spans[a].off..spans[a].off + spans[a].len], 1).unwrap_or(&[]);
            let nb =
                first_bytes(&graph[spans[b].off..spans[b].off + spans[b].len], 1).unwrap_or(&[]);
            na.cmp(nb)
        });
        out.extend_from_slice(&(order.len() as u32).to_le_bytes());
        for &idx in &order {
            let body = &graph[spans[idx].off..spans[idx].off + spans[idx].len];
            out.extend_from_slice(&sha256(first_bytes(body, 1)?)); // name
            out.extend_from_slice(&canonical_proto_digest(first_bytes(body, 2)?, 0)?);
            // type (TypeProto)
        }
        Ok(())
    }

    /// Canonical skeleton as an owned `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Surfaces the [`ShapeViolation`] [`OnnxValue::parse`] would raise.
    pub fn canonicalize(raw: &[u8]) -> Result<Vec<u8>, ShapeViolation> {
        Ok(OnnxValue::parse(raw)?.bytes)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // ── Minimal protobuf encoders for building test `ModelProto`s ──

        fn put_varint(out: &mut Vec<u8>, mut v: u64) {
            loop {
                let mut byte = (v & 0x7f) as u8;
                v >>= 7;
                if v != 0 {
                    byte |= 0x80;
                }
                out.push(byte);
                if v == 0 {
                    break;
                }
            }
        }

        fn tag(out: &mut Vec<u8>, field_no: u64, wire: u64) {
            put_varint(out, (field_no << 3) | wire);
        }

        fn field_varint(out: &mut Vec<u8>, field_no: u64, v: u64) {
            tag(out, field_no, 0);
            put_varint(out, v);
        }

        fn field_bytes(out: &mut Vec<u8>, field_no: u64, b: &[u8]) {
            tag(out, field_no, 2);
            put_varint(out, b.len() as u64);
            out.extend_from_slice(b);
        }

        /// Smallest valid ONNX `ModelProto`: ir_version=13, one
        /// default-domain opset import (version 1), and a non-empty graph.
        fn minimal_onnx() -> Vec<u8> {
            // OperatorSetIdProto { domain = "", version = 1 }
            let mut opset = Vec::new();
            field_bytes(&mut opset, 1, b""); // domain
            field_varint(&mut opset, 2, 1); // version

            // GraphProto { name = "g" }
            let mut graph = Vec::new();
            field_bytes(&mut graph, 2, b"g");

            // ModelProto
            let mut model = Vec::new();
            field_varint(&mut model, 1, ONNX_IR_VERSION_MAX as u64); // ir_version
            field_bytes(&mut model, 7, &graph); // graph
            field_bytes(&mut model, 8, &opset); // opset_import
            model
        }

        #[test]
        fn parses_minimal_model() {
            let canon = canonicalize(&minimal_onnx()).expect("valid");
            // ir_version(8) + opset(domain digest 32 + version 8)
            //   + graph: name(32) + node_count(4) + init_count(4)
            //     + 3× IO counts(4 each)
            //   + meta: producer(32) + producer_ver(32) + domain(32)
            //     + model_ver(8) + metadata_props count(4)
            assert_eq!(
                canon.len(),
                8 + 40 + (32 + 4 + 4 + 12) + (32 + 32 + 32 + 8 + 4)
            );
        }

        #[test]
        fn rejects_out_of_range_ir() {
            // IR 7 is in range (1..=13) → accepted (would reach MISSING_GRAPH);
            // 14 is a future/unknown revision → rejected at the IR gate.
            let mut model = Vec::new();
            field_varint(&mut model, 1, (ONNX_IR_VERSION_MAX + 1) as u64);
            let err = OnnxValue::parse(&model).expect_err("unsupported ir");
            assert_eq!(err.constraint_iri, UNSUPPORTED_IR.constraint_iri);
        }

        #[test]
        fn rejects_missing_graph() {
            let mut opset = Vec::new();
            field_bytes(&mut opset, 1, b"");
            field_varint(&mut opset, 2, 1);
            let mut model = Vec::new();
            field_varint(&mut model, 1, ONNX_IR_VERSION_MAX as u64);
            field_bytes(&mut model, 8, &opset);
            let err = OnnxValue::parse(&model).expect_err("no graph");
            assert_eq!(err.constraint_iri, MISSING_GRAPH.constraint_iri);
        }

        #[test]
        fn deterministic() {
            let a = canonicalize(&minimal_onnx()).expect("valid");
            let b = canonicalize(&minimal_onnx()).expect("valid");
            assert_eq!(a, b);
        }
    }
}
