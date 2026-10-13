//! CCMAS conformance suite for the code-module AST realization.
//!
//! Pins [`uor_addr::codemodule::CodeModuleValue`] against the
//! Canonical Code-Module AST Serialization grammar documented in
//! [`uor_addr::codemodule`] — an AST-shaped extension of Rivest
//! canonical S-expressions over the AST's `mod` / `fun` / atom
//! constructors.

use uor_addr::codemodule::{address, canonicalize, AddressFailure, CodeModuleValue};

#[test]
fn empty_module_canonical_form() {
    let m = CodeModuleValue::module("empty", &[]);
    assert_eq!(m.tagged_bytes(), b"(3:mod 5:empty)");
}

#[test]
fn module_with_atoms_canonical_form() {
    let body = CodeModuleValue::atom("value");
    let m = CodeModuleValue::module("demo", &[body]);
    let canon = canonicalize(m.tagged_bytes()).expect("canonicalize");
    assert_eq!(canon, b"(3:mod 4:demo 5:value)");
}

#[test]
fn function_canonical_form() {
    let body = CodeModuleValue::atom("42");
    let ret = CodeModuleValue::atom("u32");
    let f = CodeModuleValue::function("hello", &[], &ret, &body);
    // (3:fun 5:hello () 3:u32 2:42)
    assert_eq!(f.tagged_bytes(), b"(3:fun 5:hello () 3:u32 2:42)");
}

#[test]
fn function_with_parameters_canonical_form() {
    let body = CodeModuleValue::atom("body");
    let ret = CodeModuleValue::atom("unit");
    let p1 = CodeModuleValue::atom("x");
    let p2 = CodeModuleValue::atom("y");
    let f = CodeModuleValue::function("add", &[p1, p2], &ret, &body);
    assert_eq!(f.tagged_bytes(), b"(3:fun 3:add (1:x 1:y) 4:unit 4:body)");
}

#[test]
fn nested_module_canonical_form() {
    // Module containing a Module — exercises the recursive grammar
    // case admitted by AstWalker.
    let inner = CodeModuleValue::module("inner", &[]);
    let outer = CodeModuleValue::module("outer", &[inner]);
    assert_eq!(outer.tagged_bytes(), b"(3:mod 5:outer (3:mod 5:inner))");
}

#[test]
fn round_trip_preserves_bytes() {
    let body = CodeModuleValue::atom("body");
    let ret = CodeModuleValue::atom("u32");
    let f = CodeModuleValue::function("compute", &[], &ret, &body);
    let m = CodeModuleValue::module("library", &[f]);
    let bytes = m.tagged_bytes().to_vec();
    let parsed = CodeModuleValue::parse(&bytes).expect("parse");
    assert_eq!(parsed.tagged_bytes(), bytes.as_slice());
}

#[test]
fn ccmas_is_subset_of_rivest_canonical() {
    // CCMAS bytes are valid Rivest canonical S-expressions; the
    // sexp realization's canonicalizer is the identity on them.
    let m = CodeModuleValue::module("demo", &[]);
    let bytes = m.tagged_bytes();
    let sexp_canon = uor_addr::sexp::canonicalize(bytes).expect("sexp accepts CCMAS bytes");
    assert_eq!(sexp_canon, bytes);
}

#[test]
fn typed_distinction_between_module_and_function() {
    let m = CodeModuleValue::module("a", &[]);
    let body = CodeModuleValue::atom("x");
    let ret = CodeModuleValue::atom("u32");
    let f = CodeModuleValue::function("a", &[], &ret, &body);
    let m_label = address(m.tagged_bytes()).expect("κ-label").address;
    let f_label = address(f.tagged_bytes()).expect("κ-label").address;
    assert_ne!(m_label, f_label);
}

#[test]
fn typed_distinction_between_atoms() {
    let a = CodeModuleValue::atom("a");
    let b = CodeModuleValue::atom("b");
    let la = address(a.tagged_bytes()).expect("κ-label").address;
    let lb = address(b.tagged_bytes()).expect("κ-label").address;
    assert_ne!(la, lb);
}

#[test]
fn rejects_invalid_ccmas() {
    let cases: &[&[u8]] = &[b"not ccmas", b"((((", b"(99:short", b"(unbalanced"];
    for raw in cases {
        match address(raw) {
            Err(AddressFailure::InvalidAst) => {}
            other => panic!("expected rejection for {raw:?}: {other:?}"),
        }
    }
}

#[test]
fn admits_unbounded_atom_name() {
    // ADR-060 removed the name-width cap; long atom names are admitted.
    let long = "a".repeat(10_000);
    let atom = CodeModuleValue::atom(&long);
    address(atom.tagged_bytes()).expect("long atom name admits");
}

#[test]
fn admits_unbounded_module_name() {
    // ADR-060 removed the name-width cap; long module names are admitted.
    let long = "a".repeat(10_000);
    let m = CodeModuleValue::module(&long, &[]);
    address(m.tagged_bytes()).expect("long module name admits");
}

#[test]
fn deeply_nested_modules_admit_within_bound() {
    // Build nesting comfortably within both the codemodule and the
    // sexp depth bounds (the CCMAS parser walks through sexp first).
    let mut value = CodeModuleValue::atom("leaf");
    for i in 0..8 {
        value = CodeModuleValue::module(&alloc::format!("m{i}"), core::slice::from_ref(&value));
    }
    address(value.tagged_bytes()).expect("within-bound depth admits");
}

#[test]
fn admits_module_with_multiple_items() {
    // Module carrying multiple atom items.
    let items: alloc::vec::Vec<CodeModuleValue> = (0..8)
        .map(|i| CodeModuleValue::atom(&alloc::format!("item{i}")))
        .collect();
    let m = CodeModuleValue::module("big", &items);
    address(m.tagged_bytes()).expect("admits");
}

extern crate alloc;
