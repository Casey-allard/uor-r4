/// SHACL test 286: TagSite with tagValue (Product/Coproduct Completion
/// Amendment, Gap 4).
///
/// Validates that an instance of `partition:TagSite` with the required
/// `tagValue` boolean assertion satisfies the `TagSiteShape` from
/// `conformance/shapes/uor-shapes.ttl`. Also asserts the SiteIndex
/// supertrait properties (`sitePosition`, `siteState`) since TagSite
/// extends SiteIndex.
pub const TEST286_TAG_SITE: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix partition: <https://uor.foundation/partition/> .

partition:ex_tag_286 a owl:NamedIndividual, partition:TagSite ;
    partition:sitePosition 7 ;
    partition:siteState 1 ;
    partition:tagValue "false"^^xsd:boolean .
"#;
