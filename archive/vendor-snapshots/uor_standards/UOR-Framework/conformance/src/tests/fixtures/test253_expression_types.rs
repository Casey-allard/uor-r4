//! SHACL test 253: `schema` expression and declaration types.

/// Instance graph for Test 253: Schema expression and declaration types.
pub const TEST253_EXPRESSION_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:ex_term_expr_253 a owl:NamedIndividual, schema:TermExpression .
schema:ex_literal_expr_253 a owl:NamedIndividual, schema:LiteralExpression .
schema:ex_app_expr_253 a owl:NamedIndividual, schema:ApplicationExpression .
schema:ex_infix_expr_253 a owl:NamedIndividual, schema:InfixExpression .
schema:ex_set_expr_253 a owl:NamedIndividual, schema:SetExpression .
schema:ex_comp_expr_253 a owl:NamedIndividual, schema:CompositionExpression .
schema:ex_forall_decl_253 a owl:NamedIndividual, schema:ForAllDeclaration .
schema:ex_var_binding_253 a owl:NamedIndividual, schema:VariableBinding .
schema:ex_quant_kind_253 a owl:NamedIndividual, schema:QuantifierKind .
"#;
