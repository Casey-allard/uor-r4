//! `uor-addr` — Code-module AST realization comprehensive example.
//!
//! Demonstrates [`uor_addr::codemodule::address`] over the
//! Canonical Code-Module AST Serialization (CCMAS) grammar:
//! Module, Function, Type/Const, atom literals/identifiers. Shows
//! basic minting, determinism, structural typed-distinction, and
//! CCMAS-as-Rivest-canonical-S-expression-subset.
//!
//! Run with `cargo run -p uor-addr --example codemodule_realization`.

use uor_addr::codemodule::{address, AddressFailure, CodeModuleValue};

fn main() {
    println!("uor-addr — code-module AST realization (CCMAS)\n");

    // 1. Empty module.
    let empty_mod = CodeModuleValue::module("empty", &[]);
    let outcome = address(empty_mod.tagged_bytes()).expect("κ-label");
    println!("1. Empty Module");
    println!(
        "   surface:  {}",
        core::str::from_utf8(empty_mod.tagged_bytes()).unwrap_or("<binary>")
    );
    println!("   κ-label:  {}\n", outcome.address);

    // 2. Module with a function and atom literals.
    let body = CodeModuleValue::atom("42");
    let ret_ty = CodeModuleValue::atom("u32");
    let f = CodeModuleValue::function("greet", &[], &ret_ty, &body);
    let m = CodeModuleValue::module("demo", &[f]);
    let outcome = address(m.tagged_bytes()).expect("κ-label");
    println!("2. Module with Function");
    println!(
        "   surface:  {}",
        core::str::from_utf8(m.tagged_bytes()).unwrap_or("<binary>")
    );
    println!("   κ-label:  {}\n", outcome.address);

    // 3. Determinism.
    let a = address(m.tagged_bytes()).expect("κ-label").address;
    let b = address(m.tagged_bytes()).expect("κ-label").address;
    assert_eq!(a, b);
    println!("3. Determinism");
    println!("   run 1: {a}");
    println!("   run 2: {b}");
    println!("   match: {} ✓\n", a == b);

    // 4. CCMAS-as-Rivest-S-expression-subset: the κ-label produced
    //    by the codemodule realization differs from the sexp
    //    realization's κ-label for the same canonical bytes,
    //    because the typed-input IRI differs (CodeModuleValue vs
    //    SExprValue) and the AddressInput trait disambiguates
    //    canonicalization-output by V::IRI.
    //
    //    However, the surface canonical bytes ARE valid Rivest
    //    canonical S-expressions and a Rivest canonicalize round-
    //    trip is the identity on them — the underlying byte layer
    //    is shared.
    let rivest_round_trip = uor_addr::sexp::canonicalize(m.tagged_bytes()).expect("valid sexp");
    assert_eq!(rivest_round_trip, m.tagged_bytes());
    println!("4. CCMAS bytes are Rivest canonical S-expressions");
    println!("   sexp::canonicalize(codemodule bytes) == codemodule bytes ✓\n");

    // 5. Typed distinction — different AST shapes yield different κ-labels.
    let m0 = CodeModuleValue::module("a", &[]);
    let m1 = CodeModuleValue::module("b", &[]);
    let atom_a = CodeModuleValue::atom("a");
    let l0 = address(m0.tagged_bytes()).expect("κ-label").address;
    let l1 = address(m1.tagged_bytes()).expect("κ-label").address;
    let la = address(atom_a.tagged_bytes()).expect("κ-label").address;
    assert_ne!(l0, l1);
    assert_ne!(l0, la);
    assert_ne!(l1, la);
    println!("5. Typed distinction");
    println!("   Module \"a\":           {l0}");
    println!("   Module \"b\":           {l1}");
    println!("   Atom \"a\":             {la}");
    println!();

    // 6. Failure modes.
    println!("6. Failure modes");
    match address(b"not ccmas") {
        Err(AddressFailure::InvalidAst) => println!("   non-CCMAS input rejected ✓"),
        other => panic!("expected InvalidAst: {other:?}"),
    }
    // ADR-060 removed the fixed name-width cap: code-module names are
    // now unbounded. A very long atom name is admitted and yields a
    // valid 71-byte κ-label.
    let very_long = "a".repeat(100_000);
    let big_atom = CodeModuleValue::atom(&very_long);
    let outcome = address(big_atom.tagged_bytes()).expect("κ-label");
    assert_eq!(outcome.address.len(), 71);
    println!("   unbounded long-name atom admitted ✓");

    println!("\nOK — CCMAS realization shipped; Rivest-canonical byte layer shared.");
}
