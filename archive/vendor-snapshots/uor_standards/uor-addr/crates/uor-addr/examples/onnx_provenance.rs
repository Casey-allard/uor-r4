//! `uor-addr` — ONNX model provenance (supply-chain attestation).
//!
//! Real-world use case: a model-card / SLSA-style provenance system that
//! identifies an ONNX `ModelProto` by a **typed content address** of its
//! protobuf-canonical structural form (topological node ordering +
//! initializer-weight digests, ADR-046). This gives:
//!
//! - **build-tool invariance** — exporters and graph optimizers emit
//!   nodes and initializers in different byte orders; the canonical form
//!   sorts them topologically, so the same graph yields the same
//!   provenance ID regardless of emitter;
//! - **weight-tamper detection** — perturbing an initializer changes the
//!   ID, so a poisoned checkpoint is caught at verification;
//! - **ensemble binding** — two member-model IDs combine under the
//!   commutative CS-G2 product (ADR-061) into one ensemble ID that does
//!   not depend on member order;
//! - **σ-axis selection** — the same graph can be attested under any
//!   shipped hash axis.
//!
//! The ONNX bytes are synthesized in-process (a minimal-but-valid IR v13
//! `ModelProto`) so the example is self-contained.
//!
//! Run with `cargo run -p uor-addr --example onnx_provenance`.

// ── Minimal protobuf wire-format helpers (ONNX is protobuf). ──

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

fn value_info(name: &str) -> Vec<u8> {
    lfield(1, name.as_bytes())
}

/// A GraphProto from nodes (field 1) + initializers (field 5), emitted in
/// the given order so we can model two byte-different exports.
fn graph(nodes: &[&[u8]], initializers: &[&[u8]]) -> Vec<u8> {
    let mut g = Vec::new();
    for n in nodes {
        g.extend(lfield(1, n));
    }
    g.extend(lfield(2, b"demo-graph")); // name
    for init in initializers {
        g.extend(lfield(5, init));
    }
    g.extend(lfield(11, &value_info("x"))); // input
    g.extend(lfield(12, &value_info("z"))); // output
    g
}

fn model(graph_body: &[u8]) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend(vfield(1, 13)); // ir_version = 13
    let mut opset = Vec::new();
    opset.extend(vfield(2, 17)); // opset version 17
    m.extend(lfield(8, &opset)); // opset_import
    m.extend(lfield(7, graph_body)); // graph
    m
}

fn provenance_id(onnx: &[u8]) -> String {
    uor_addr::onnx::address(onnx)
        .expect("valid ONNX ModelProto")
        .address
        .as_str()
        .to_string()
}

fn main() {
    println!("uor-addr — ONNX model provenance (supply-chain attestation)\n");

    // ── The graph: x ─Relu(a)→ y ; (y,w) ─Add(b)→ z, with weight w. ──
    let a = node("a", "Relu", &["x"], &["y"]);
    let b = node("b", "Add", &["y", "w"], &["z"]);
    let w = init_raw("w", &[1.0, 2.0]);

    let model_a = model(&graph(&[&a, &b], &[&w]));
    let id = provenance_id(&model_a);
    println!("  provenance id:  {id}\n");

    // ── Build-tool invariance: a second exporter emits the two nodes in
    //    the opposite order. Byte-different file, same provenance id. ──
    let model_b = model(&graph(&[&b, &a], &[&w]));
    assert_ne!(model_a, model_b, "the two exports are byte-different");
    let id_b = provenance_id(&model_b);
    assert_eq!(id, id_b, "topological canonicalization → same id");
    println!("  re-exported with nodes in reverse order → same id:");
    println!("                  {id_b}\n");

    // ── Weight-tamper detection: perturb the initializer. ──
    let w_poisoned = init_raw("w", &[1.0, 2.0001]);
    let model_poisoned = model(&graph(&[&a, &b], &[&w_poisoned]));
    let id_poisoned = provenance_id(&model_poisoned);
    assert_ne!(id, id_poisoned, "a perturbed weight must change the id");
    println!("  weight 2.0 → 2.0001 (poisoned checkpoint) → different id:");
    println!("                  {id_poisoned}\n");

    // ── Ensemble binding: a second member model, then bind both ids with
    //    the commutative CS-G2 product into one order-independent
    //    ensemble id (ADR-061). ──
    let c = node("c", "Sigmoid", &["x"], &["z"]);
    let member_2 = model(&graph(&[&c], &[]));
    let id_2 = provenance_id(&member_2);

    let left = uor_addr::onnx::address(&model_a).unwrap().address;
    let right = uor_addr::onnx::address(&member_2).unwrap().address;
    let ensemble = uor_addr::composition::compose_g2_product(&left, &right)
        .expect("g2 binds the two member ids")
        .address;
    let ensemble_swapped = uor_addr::composition::compose_g2_product(&right, &left)
        .expect("g2 is commutative")
        .address;
    assert_eq!(
        ensemble, ensemble_swapped,
        "ensemble id is order-independent"
    );
    println!("  member ids:     {id}");
    println!("                  {id_2}");
    println!("  ensemble id (CS-G2, commutative):");
    println!("                  {ensemble}\n");

    // ── σ-axis: attest the same graph under sha3-256 (e.g. a FIPS-202
    //    deployment). ──
    let fips = uor_addr::onnx::address_sha3_256(&model_a)
        .expect("valid ONNX")
        .address;
    assert!(fips.as_str().starts_with("sha3-256:"));
    println!("  FIPS-202 attestation (sha3-256 σ-axis):");
    println!("                  {fips}");

    println!("\nOK — provenance id is exporter-invariant, tamper-sensitive, and composable.");
}
