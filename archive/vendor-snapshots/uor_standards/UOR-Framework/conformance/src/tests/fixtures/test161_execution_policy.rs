/// SHACL fixture for resolver:ExecutionPolicy with ExecutionPolicyKind — Amendment 48.
pub const TEST161_EXECUTION_POLICY: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix resolver: <https://uor.foundation/resolver/> .

<urn:test:policy_161> a owl:NamedIndividual , resolver:ExecutionPolicy .

<urn:test:resolver_161> a owl:NamedIndividual , resolver:SessionResolver ;
    resolver:executionPolicy <urn:test:policy_161> .

resolver:MinFreeCountFirst a owl:NamedIndividual , resolver:ExecutionPolicyKind .
"#;
