//! Grammar constant for the XML realization. The capacity profile is the
//! shared [`crate::bounds::AddrBounds`]; only the recursion stack-safety
//! bound lives here.

pub mod bounds;

pub use bounds::MAX_XML_DEPTH;
