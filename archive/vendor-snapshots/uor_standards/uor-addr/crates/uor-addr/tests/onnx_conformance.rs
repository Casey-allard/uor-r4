//! Closed-loop (CL-ONNX) conformance for the ONNX realization (IR ≤ v13).
//!
//! Synthesizes minimal `ModelProto` wire buffers in-process and asserts
//! the canonical-form invariants: a 71-byte κ-label, determinism,
//! invariance under node reordering (topological canonicalization) and
//! under `raw_data` vs typed-`float_data` storage, sensitivity to weights
//! and op types, and rejection of malformed input.

#![cfg(feature = "onnx")]

use uor_addr::onnx::{self, AddressFailure};

// ── protobuf wire encoders ──

fn varint(mut v: u64) -> Vec<u8> {
    let mut o = Vec::new();
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            o.push(b | 0x80);
        } else {
            o.push(b);
            break;
        }
    }
    o
}
fn tag(f: u64, w: u8) -> Vec<u8> {
    varint((f << 3) | u64::from(w))
}
fn vfield(f: u64, val: u64) -> Vec<u8> {
    let mut o = tag(f, 0);
    o.extend(varint(val));
    o
}
fn lfield(f: u64, bytes: &[u8]) -> Vec<u8> {
    let mut o = tag(f, 2);
    o.extend(varint(bytes.len() as u64));
    o.extend_from_slice(bytes);
    o
}

fn node(name: &str, op_type: &str, inputs: &[&str], outputs: &[&str]) -> Vec<u8> {
    let mut n = Vec::new();
    for i in inputs {
        n.extend(lfield(1, i.as_bytes()));
    }
    for o in outputs {
        n.extend(lfield(2, o.as_bytes()));
    }
    n.extend(lfield(3, name.as_bytes()));
    n.extend(lfield(4, op_type.as_bytes()));
    n
}

/// A FLOAT initializer storing `vals` in `raw_data`.
fn init_raw(name: &str, vals: &[f32]) -> Vec<u8> {
    let mut data = Vec::new();
    for v in vals {
        data.extend_from_slice(&v.to_le_bytes());
    }
    let mut t = Vec::new();
    t.extend(vfield(2, 1)); // data_type = FLOAT
    t.extend(vfield(1, vals.len() as u64)); // dims = [n]
    t.extend(lfield(8, name.as_bytes())); // name
    t.extend(lfield(9, &data)); // raw_data
    t
}

/// A FLOAT initializer storing `vals` in the packed `float_data` field.
fn init_float_data(name: &str, vals: &[f32]) -> Vec<u8> {
    let mut packed = Vec::new();
    for v in vals {
        packed.extend_from_slice(&v.to_le_bytes());
    }
    let mut t = Vec::new();
    t.extend(vfield(2, 1));
    t.extend(vfield(1, vals.len() as u64));
    t.extend(lfield(8, name.as_bytes()));
    t.extend(lfield(4, &packed)); // float_data (packed fixed32)
    t
}

fn value_info(name: &str) -> Vec<u8> {
    lfield(1, name.as_bytes())
}

fn graph(nodes: &[&[u8]], initializers: &[&[u8]]) -> Vec<u8> {
    let mut g = Vec::new();
    for n in nodes {
        g.extend(lfield(1, n));
    }
    g.extend(lfield(2, b"g")); // name
    for init in initializers {
        g.extend(lfield(5, init));
    }
    g.extend(lfield(11, &value_info("x"))); // input
    g.extend(lfield(12, &value_info("z"))); // output
    g
}

fn model_with(ir_version: u64, graph_body: &[u8]) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend(vfield(1, ir_version)); // ir_version
    let mut opset = Vec::new();
    opset.extend(vfield(2, 17)); // version = 17, default domain ("") omitted
    m.extend(lfield(8, &opset)); // opset_import
    m.extend(lfield(7, graph_body)); // graph
    m
}

// ── sample model: x ─Relu(a)→ y ; (y,w) ─Add(b)→ z ──

fn sample_nodes() -> (Vec<u8>, Vec<u8>) {
    (
        node("a", "Relu", &["x"], &["y"]),
        node("b", "Add", &["y", "w"], &["z"]),
    )
}

fn sample_model() -> Vec<u8> {
    let (a, b) = sample_nodes();
    let w = init_raw("w", &[1.0, 2.0]);
    model_with(13, &graph(&[&a, &b], &[&w]))
}

fn label(bytes: &[u8]) -> String {
    onnx::address(bytes)
        .expect("valid onnx")
        .address
        .as_str()
        .to_string()
}
fn is_kappa(s: &str) -> bool {
    s.len() == 71
        && s.starts_with("sha256:")
        && s[7..]
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

#[test]
fn well_formed_yields_kappa_label() {
    assert!(is_kappa(&label(&sample_model())));
}

#[test]
fn label_is_deterministic() {
    assert_eq!(label(&sample_model()), label(&sample_model()));
}

#[test]
fn invariant_under_node_reorder() {
    // Same graph, nodes serialized in the opposite (still-valid-input)
    // order; topological canonicalization must collapse them.
    let (a, b) = sample_nodes();
    let w = init_raw("w", &[1.0, 2.0]);
    let forward = model_with(13, &graph(&[&a, &b], &[&w]));
    let reversed = model_with(13, &graph(&[&b, &a], &[&w]));
    assert_eq!(
        label(&forward),
        label(&reversed),
        "node order must not affect κ"
    );
}

#[test]
fn raw_data_and_typed_data_equivalent() {
    let (a, b) = sample_nodes();
    let raw = init_raw("w", &[1.0, 2.0]);
    let typed = init_float_data("w", &[1.0, 2.0]);
    let m_raw = model_with(13, &graph(&[&a, &b], &[&raw]));
    let m_typed = model_with(13, &graph(&[&a, &b], &[&typed]));
    assert_eq!(
        label(&m_raw),
        label(&m_typed),
        "raw_data and float_data storage must canonicalize identically"
    );
}

#[test]
fn sensitive_to_weight() {
    let base = label(&sample_model());
    let (a, b) = sample_nodes();
    let w = init_raw("w", &[1.0, 2.5]); // changed weight
    let other = label(&model_with(13, &graph(&[&a, &b], &[&w])));
    assert_ne!(base, other);
}

#[test]
fn sensitive_to_op_type() {
    let base = label(&sample_model());
    let a = node("a", "Sigmoid", &["x"], &["y"]); // Relu → Sigmoid
    let b = node("b", "Add", &["y", "w"], &["z"]);
    let w = init_raw("w", &[1.0, 2.0]);
    let other = label(&model_with(13, &graph(&[&a, &b], &[&w])));
    assert_ne!(base, other);
}

#[test]
fn rejects_out_of_range_ir_version() {
    // ADR-060 / V&V: the realization admits any known IR revision
    // (1..=ONNX_IR_VERSION_MAX = 13). A future / unknown revision (14) is
    // rejected; so is the absent/0 sentinel.
    let (a, b) = sample_nodes();
    let w = init_raw("w", &[1.0, 2.0]);
    let g = graph(&[&a, &b], &[&w]);
    assert_eq!(
        onnx::address(&model_with(14, &g)).unwrap_err(),
        AddressFailure::InvalidOnnx
    );
    assert_eq!(
        onnx::address(&model_with(0, &g)).unwrap_err(),
        AddressFailure::InvalidOnnx
    );
}

#[test]
fn accepts_in_range_ir_versions_distinctly() {
    // Older IR revisions (real-world exports are IR 6–10) are admitted;
    // the ir_version is bound into the skeleton, so distinct revisions of
    // the same logical graph canonicalize to distinct κ-labels.
    let (a, b) = sample_nodes();
    let w = init_raw("w", &[1.0, 2.0]);
    let g = graph(&[&a, &b], &[&w]);
    let v7 = onnx::address(&model_with(7, &g))
        .expect("IR 7 admitted")
        .address;
    let v13 = onnx::address(&model_with(13, &g))
        .expect("IR 13 admitted")
        .address;
    assert_ne!(v7, v13, "ir_version must bind into the κ-label");
}

#[test]
fn rejects_unknown_dtype() {
    let mut t = Vec::new();
    t.extend(vfield(2, 99)); // data_type = 99 (out of range)
    t.extend(vfield(1, 1));
    t.extend(lfield(8, b"w"));
    t.extend(lfield(9, &[0u8; 4]));
    let a = node("a", "Relu", &["x"], &["y"]);
    let m = model_with(13, &graph(&[&a], &[&t]));
    assert_eq!(onnx::address(&m).unwrap_err(), AddressFailure::InvalidOnnx);
}

#[test]
fn value_info_type_field_order_invariant() {
    // A TypeProto whose Tensor sub-message lists (elem_type, shape) in
    // either field order must canonicalize identically (canonicalization
    // rule 1, applied to opaque sub-messages via canonical_proto_digest).
    let dim = vfield(1, 2); // Dimension.dim_value = 2
    let shape = lfield(1, &dim); // TensorShapeProto.dim
    let tensor_fwd = [vfield(1, 1), lfield(2, &shape)].concat(); // elem_type, shape
    let tensor_rev = [lfield(2, &shape), vfield(1, 1)].concat(); // shape, elem_type
    let model = |tensor: &[u8]| {
        let typ = lfield(1, tensor); // TypeProto.tensor_type
        let vi = [lfield(1, b"x"), lfield(2, &typ)].concat(); // ValueInfoProto
        let (a, b) = sample_nodes();
        let w = init_raw("w", &[1.0, 2.0]);
        let mut g = Vec::new();
        g.extend(lfield(1, &a));
        g.extend(lfield(1, &b));
        g.extend(lfield(2, b"g"));
        g.extend(lfield(5, &w));
        g.extend(lfield(11, &vi)); // input with type
        g.extend(lfield(12, &value_info("z")));
        model_with(13, &g)
    };
    assert_eq!(
        label(&model(&tensor_fwd)),
        label(&model(&tensor_rev)),
        "TypeProto field order must not affect the κ-label"
    );
}

#[test]
fn rejects_cycle() {
    // a: u→v ; b: v→u  (mutual dependency, no topological order)
    let a = node("a", "Relu", &["u"], &["v"]);
    let b = node("b", "Relu", &["v"], &["u"]);
    let m = model_with(13, &graph(&[&a, &b], &[]));
    assert_eq!(onnx::address(&m).unwrap_err(), AddressFailure::InvalidOnnx);
}
