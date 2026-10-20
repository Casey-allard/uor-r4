//! Code-module AST typed input under the Canonical Code-Module AST
//! Serialization (CCMAS) form (ADR-023 amended by ADR-060).
//!
//! CCMAS is Rivest canonical S-expressions over the AST grammar cases:
//! the canonical byte output is a Rivest `(s₁ s₂ … sₙ)` flat list
//! (Sexp.txt §4.3) with `<length>:<bytes>` atoms (§4.2). The canonical
//! form is therefore **identical** to the [`crate::sexp`] realization's,
//! so the pipeline reuses sexp's no_alloc streaming canonicalizer
//! ([`SExprCanon`]) — under the `CodeModuleValue` typed-input IRI. There
//! is no size, name-width, item-count, or nesting-depth ceiling.
//!
//! [`CodeModuleValue`] (the owned AST **builder**, `alloc`-gated)
//! constructs canonical CCMAS bytes programmatically (`module`,
//! `function`, `atom`) for reference and testing; [`CodeModuleCarrier`]
//! is the borrowed model-input handle the pipeline binds.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};
// `ShapeViolation` is consumed only by the `alloc`-gated builder / parser.
#[cfg(feature = "alloc")]
use prism::pipeline::ShapeViolation;
#[cfg(feature = "alloc")]
use prism::uor_foundation::pipeline::ChunkSource;

use crate::sexp::SExprCanon;

/// The CCMAS typed-input IRI.
pub(crate) const CODEMODULE_IRI: &str = "https://uor.foundation/addr/CodeModuleValue";

// ─── CodeModuleCarrier — the borrowed model-input handle (no_alloc) ─────

/// Borrowed CCMAS input handle (ADR-060 stream carrier). A thin, `Copy`
/// borrow of a [`SExprCanon`]; `as_binding_value` returns the `Stream`
/// carrier zero-copy under the `CodeModuleValue` IRI.
#[derive(Clone, Copy, Debug)]
pub struct CodeModuleCarrier<'a>(&'a SExprCanon<'a>);

impl<'a> CodeModuleCarrier<'a> {
    /// Wrap a validated canonical-form carrier as a model input handle.
    #[must_use]
    pub fn new(canon: &'a SExprCanon<'a>) -> Self {
        Self(canon)
    }
}

impl ConstrainedTypeShape for CodeModuleCarrier<'_> {
    const IRI: &'static str = CODEMODULE_IRI;
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for CodeModuleCarrier<'_> {}

impl<'a> IntoBindingValue<'a> for CodeModuleCarrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::stream(self.0)
    }
}

impl PartitionProductFields for CodeModuleCarrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ─── CodeModuleValue — the owned AST builder (alloc) ────────────────────

/// Owned CCMAS value + AST builder. Constructs canonical CCMAS bytes
/// programmatically for reference and testing. **`alloc`-gated** — the
/// pipeline binds the borrowed [`CodeModuleCarrier`] handle, which needs
/// no allocator. There is no width / count ceiling.
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq, Eq)]
pub struct CodeModuleValue {
    bytes: alloc::vec::Vec<u8>,
}

#[cfg(feature = "alloc")]
impl core::fmt::Debug for CodeModuleValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CodeModuleValue")
            .field("len", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "alloc")]
impl CodeModuleValue {
    /// Parse + canonicalize raw CCMAS bytes (Rivest canonical or
    /// token-list form), retaining the canonical bytes.
    ///
    /// # Errors
    ///
    /// A `validCcmas`/`validUtf8SExpr` [`ShapeViolation`] if `raw` is not
    /// a well-formed S-expression.
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        SExprCanon::validate(raw)?;
        let canon = SExprCanon::new(raw);
        let mut bytes = alloc::vec::Vec::new();
        canon.for_each_chunk(&mut |chunk| bytes.extend_from_slice(chunk));
        Ok(Self { bytes })
    }

    /// Build a Module AST node: `(3:mod <name> <item>…)`.
    #[must_use]
    pub fn module(name: &str, items: &[CodeModuleValue]) -> Self {
        Self::ast_call("mod", name, items)
    }

    /// Build a Function AST node: `(3:fun <name> (<param>…) <ret> <body>)`.
    #[must_use]
    pub fn function(
        name: &str,
        parameters: &[CodeModuleValue],
        return_type: &CodeModuleValue,
        body: &CodeModuleValue,
    ) -> Self {
        let mut out = alloc::vec::Vec::new();
        out.extend_from_slice(b"(3:fun ");
        write_atom(&mut out, name.as_bytes());
        out.extend_from_slice(b" (");
        for (i, p) in parameters.iter().enumerate() {
            if i > 0 {
                out.push(b' ');
            }
            out.extend_from_slice(&p.bytes);
        }
        out.push(b')');
        out.push(b' ');
        out.extend_from_slice(&return_type.bytes);
        out.push(b' ');
        out.extend_from_slice(&body.bytes);
        out.push(b')');
        Self { bytes: out }
    }

    /// Build an Atom AST node (Identifier, Literal, etc.): `<len>:<text>`.
    #[must_use]
    pub fn atom(text: &str) -> Self {
        let mut out = alloc::vec::Vec::new();
        write_atom(&mut out, text.as_bytes());
        Self { bytes: out }
    }

    fn ast_call(tag: &str, name: &str, items: &[CodeModuleValue]) -> Self {
        let mut out = alloc::vec::Vec::new();
        out.push(b'(');
        write_atom(&mut out, tag.as_bytes());
        out.push(b' ');
        write_atom(&mut out, name.as_bytes());
        for item in items {
            out.push(b' ');
            out.extend_from_slice(&item.bytes);
        }
        out.push(b')');
        Self { bytes: out }
    }

    /// Borrow the CCMAS canonical bytes.
    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Append a Rivest canonical atom `<len>:<bytes>` to `out`.
#[cfg(feature = "alloc")]
fn write_atom(out: &mut alloc::vec::Vec<u8>, bytes: &[u8]) {
    let mut buf = [0u8; 20];
    let mut n = bytes.len();
    let s = if n == 0 {
        buf[0] = b'0';
        &buf[..1]
    } else {
        let mut idx = buf.len();
        while n > 0 {
            idx -= 1;
            buf[idx] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        &buf[idx..]
    };
    out.extend_from_slice(s);
    out.push(b':');
    out.extend_from_slice(bytes);
}

/// Validate + materialize the canonical CCMAS bytes.
///
/// **Available only under the `alloc` feature.**
///
/// # Errors
///
/// Surfaces the [`ShapeViolation`] [`SExprCanon::validate`] would raise.
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, ShapeViolation> {
    Ok(CodeModuleValue::parse(raw)?.bytes)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn module_canonical_bytes() {
        let body = CodeModuleValue::atom("value");
        let m = CodeModuleValue::module("demo", &[body]);
        assert_eq!(m.tagged_bytes(), b"(3:mod 4:demo 5:value)");
    }

    #[test]
    fn function_canonical_bytes() {
        let body = CodeModuleValue::atom("body");
        let ret = CodeModuleValue::atom("u32");
        let p = CodeModuleValue::atom("x");
        let f = CodeModuleValue::function("add", &[p], &ret, &body);
        assert_eq!(f.tagged_bytes(), b"(3:fun 3:add (1:x) 3:u32 4:body)");
    }

    #[test]
    fn parse_is_idempotent_on_canonical_bytes() {
        let m = CodeModuleValue::module("library", &[]);
        let parsed = CodeModuleValue::parse(m.tagged_bytes()).expect("parse");
        assert_eq!(parsed.tagged_bytes(), m.tagged_bytes());
    }

    #[test]
    fn canonical_form_matches_sexp_realization() {
        // CCMAS canonical bytes are Rivest canonical S-expressions, so the
        // sexp realization's canonicalizer is the identity on them.
        let m = CodeModuleValue::module("demo", &[]);
        let sexp_canon = crate::sexp::canonicalize(m.tagged_bytes()).expect("sexp accepts CCMAS");
        assert_eq!(sexp_canon, m.tagged_bytes());
    }
}
