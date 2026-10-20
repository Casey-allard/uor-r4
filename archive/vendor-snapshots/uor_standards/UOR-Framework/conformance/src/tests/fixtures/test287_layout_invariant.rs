/// SHACL test 287: LayoutInvariant with layoutRule (Product/Coproduct
/// Completion Amendment, Gap 3 — foundation namespace).
///
/// Validates that an instance of `foundation:LayoutInvariant` with the
/// required `layoutRule` string assertion satisfies the
/// `LayoutInvariantShape` from `conformance/shapes/uor-shapes.ttl`.
/// The instance corresponds to the `ProductLayoutWidth` invariant the
/// amendment introduces (one of four: ProductLayoutWidth,
/// CartesianLayoutWidth, CoproductLayoutWidth, CoproductTagEncoding).
pub const TEST287_LAYOUT_INVARIANT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix foundation: <https://uor.foundation/foundation/> .

foundation:ex_layout_287 a owl:NamedIndividual, foundation:LayoutInvariant ;
    foundation:layoutRule "SITE_COUNT(A × B) = SITE_COUNT(A) + SITE_COUNT(B)" .
"#;
