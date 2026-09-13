//! First learned relational read over explicitly supplied exact records.
//! Typed record boundaries and canonical query/key encoding are fixed scaffolds;
//! this is not a raw-text parser, trained query encoder or full language model.
pub mod circuit;
pub mod data;
pub mod learning;
#[cfg(test)]
mod report;
pub mod runtime;
