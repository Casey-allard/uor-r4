//! Output-supervised word alignment over short raw language relations.
//! Generic word segmentation supplies occurrences, never semantic roles or keys.
pub mod data;
pub mod learning;
#[cfg(test)]
mod report;
pub mod runtime;
