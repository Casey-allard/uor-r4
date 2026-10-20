//! UOR Foundation namespace modules.
//!
//! Each sub-module encodes one namespace of the UOR ontology as Rust static data.
//! Modules are listed in dependency order; see [`crate::Ontology::full`] for the
//! assembly sequence.

pub mod boundary;
pub mod carry;
pub mod cert;
pub mod cohomology;
pub mod conformance_;
pub mod convergence;
pub mod derivation;
pub mod division;
pub mod effect;
pub mod failure;
pub mod foundation;
pub mod homology;
pub mod interaction;
pub mod linear;
pub mod monoidal;
pub mod morphism;
pub mod observable;
pub mod op;
pub mod operad;
pub mod parallel;
pub mod partition;
pub mod predicate;
pub mod proof;
pub mod query;
pub mod recursion;
pub mod reduction;
pub mod region;
pub mod resolver;
pub mod schema;
pub mod state;
pub mod stream;
pub mod trace;
pub mod type_;
pub mod u;
