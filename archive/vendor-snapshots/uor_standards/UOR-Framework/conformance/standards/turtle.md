# Turtle 1.1 Standards

See [rdf.md](rdf.md) for full RDF 1.1 and Turtle 1.1 requirements.

## Turtle-Specific Rules

### Prefix Declarations

All `@prefix` declarations appear at the top of the file before any triples.

### List Syntax

Ordered collections (e.g., `op:composedOf` for composition order) use Turtle list syntax:

```turtle
op:succ  op:composedOf  ( op:neg  op:bnot ) .
```

This preserves application order (neg is applied first, then bnot).

### Literal Datatypes

All literal values include explicit datatype annotations:

```turtle
op:neg  op:arity  "1"^^xsd:nonNegativeInteger .
schema:pi1  schema:value  "1"^^xsd:integer .
```

### Space Annotation

Every namespace ontology declaration includes the `uor:space` annotation (Amendment 8):

```turtle
<https://uor.foundation/u/>  uor:space  "kernel" .
<https://uor.foundation/state/>  uor:space  "user" .
<https://uor.foundation/partition/>  uor:space  "bridge" .
```

### Reference

- [Turtle 1.1 W3C Specification](https://www.w3.org/TR/turtle/)
