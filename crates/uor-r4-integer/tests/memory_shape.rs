//! The served format can **carry** a product-key memory — an operator that
//! replaces a layer's **MLP** with a lookup — and it is refused at validation until the D11
//! kernel serves it, so that no container passes validation and is then
//! served as if the memory were not there.
//!
//! Why this test exists at this exact moment: the product's serving contract
//! (D11: no floating point, no multiplier instruction) is where an address's
//! arithmetic can beat a matrix multiply, and the owner's metric direction
//! (tokens/s, measured on GPU) is what decides whether it does. The exporter
//! refuses memory models today — `integer export of product-key memories is
//! not implemented` — so the mechanism **cannot be measured on the product
//! path at all**. This is the first landed slice of the port: the format
//! records the geometry, the shape rules reject nonsense, and the container
//! refuses to claim a capability the engine does not have yet.

use uor_r4_integer::stack::{StackError, StackMemory, StackShape};

fn shape(memory: Option<StackMemory>) -> StackShape {
    StackShape {
        vocab: 40,
        width: 32,
        heads: 1,
        mlp: 32,
        pattern: "ra".to_owned(),
        read: "dot".to_owned(),
        rotation: false,
        context: 16,
        pointer: None,
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory,
        rec_gates: None,
        gate: None,
    }
}

fn memory(layers: Vec<usize>, key_dim: usize) -> StackMemory {
    StackMemory {
        layers,
        heads: 1,
        key_dim,
        sub_keys: 4,
        top_k: 2,
        score: "dot".to_owned(),
    }
}

#[test]
fn a_carried_memory_is_refused_until_the_kernel_serves_it() {
    // Without one, the shape validates.
    assert!(shape(None).validate().is_ok());
    // With one, it is refused — and for that reason, not another.
    assert!(matches!(
        shape(Some(memory(vec![1], 8))).validate(),
        Err(StackError::Shape(
            "a product-key memory has no served kernel yet"
        ))
    ));
}

#[test]
fn the_geometry_rules_reject_a_memory_that_could_not_be_served() {
    // A layer index outside the pattern is refused.
    assert!(matches!(
        shape(Some(memory(vec![2], 8))).validate(),
        Err(StackError::Shape(
            "the product-key memory's layers or geometry are unsupported"
        ))
    ));
    // Layers ascend without repetition.
    assert!(matches!(
        shape(Some(memory(vec![1, 1], 8))).validate(),
        Err(StackError::Shape(
            "the product-key memory's layers or geometry are unsupported"
        ))
    ));
    // Each head's query block splits into two equal sides.
    assert!(matches!(
        shape(Some(memory(vec![1], 7))).validate(),
        Err(StackError::Shape(
            "the product-key memory's layers or geometry are unsupported"
        ))
    ));
}

#[test]
fn the_record_deserialises_from_a_container_shape() {
    let carried: StackShape = serde_json::from_value(serde_json::json!({
        "vocab": 40, "width": 32, "heads": 1, "mlp": 32, "pattern": "ra",
        "read": "dot", "rotation": false, "context": 16,
        "memory": {"layers": [1], "heads": 1, "key_dim": 8, "sub_keys": 4,
                   "top_k": 2, "score": "dot"}
    }))
    .expect("a shape carrying a memory deserialises");
    assert_eq!(carried.memory, Some(memory(vec![1], 8)));
    // And an older shape without the section still parses, absent.
    let older: StackShape = serde_json::from_value(serde_json::json!({
        "vocab": 40, "width": 32, "heads": 1, "mlp": 32, "pattern": "ra",
        "read": "dot", "rotation": false, "context": 16
    }))
    .expect("a shape without the section deserialises");
    assert_eq!(older.memory, None);
    assert!(older.validate().is_ok());
}
