# JSON-LD 1.1 Standards

## Overview

The UOR Foundation ontology is published as a JSON-LD 1.1 document
(`public/uor.foundation.jsonld`). This document must conform to the
W3C JSON-LD 1.1 specification.

## Required Structure

```json
{
  "@context": {
    "@version": 1.1,
    "owl":  "http://www.w3.org/2002/07/owl#",
    "rdf":  "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
    "xsd":  "http://www.w3.org/2001/XMLSchema#",
    "sh":   "http://www.w3.org/ns/shacl#",
    "uor":  "https://uor.foundation/",
    "u":    "https://uor.foundation/u/",
    "schema": "https://uor.foundation/schema/",
    "op":   "https://uor.foundation/op/",
    ...
  },
  "@graph": [...]
}
```

## Context Requirements

- `@version` must be `1.1`
- All 33 UOR namespace prefixes must be declared
- Standard prefixes (`owl`, `rdf`, `rdfs`, `xsd`, `sh`) must be declared
- All prefix IRIs must end with `/` or `#`

## Graph Requirements

- `@graph` must be an array
- Every node must have `@id`
- `@id` values must be IRIs (contain `:`)
- OWL class nodes must have `@type: "owl:Class"`
- OWL property nodes must have `@type` of `owl:DatatypeProperty`, `owl:ObjectProperty`, or `owl:AnnotationProperty`
- Named individual nodes must have `@type: "owl:NamedIndividual"`

## Node Count Minimums

| Type | Minimum |
|------|---------|
| owl:Class | 441 |
| owl:DatatypeProperty + owl:ObjectProperty + owl:AnnotationProperty | 891 |
| owl:NamedIndividual | 3356 |

## References

- [JSON-LD 1.1 W3C Specification](https://www.w3.org/TR/json-ld11/)
- [JSON-LD 1.1 Processing Algorithms](https://www.w3.org/TR/json-ld11-api/)
