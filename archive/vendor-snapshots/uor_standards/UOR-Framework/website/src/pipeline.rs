//! PRISM pipeline stage definitions and concept SVG hook registry.

use uor_ontology::Ontology;

/// PRISM pipeline stage definitions — re-exported from `uor_docs::nav` (single source of truth).
pub use uor_docs::nav::PRISM_STAGES;

/// SVG hook function type: takes an ontology reference, returns an SVG string.
pub type SvgHookFn = fn(&Ontology) -> String;

/// Maps concept page slugs to SVG generator functions.
///
/// When generating a concept page, if a `(slug, fn)` entry exists for the concept's
/// slug, the generator is called and its SVG is injected after the page `<h1>`.
/// This eliminates `match` expressions on hard-coded slug strings in `generate()`.
pub const CONCEPT_SVG_HOOKS: &[(&str, SvgHookFn)] =
    &[("witt-levels", crate::svg::render_witt_levels_svg)];
