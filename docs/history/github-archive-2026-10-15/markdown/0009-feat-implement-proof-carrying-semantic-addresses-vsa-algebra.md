# #9 — feat: implement proof-carrying semantic addresses, VSA algebra, and SemanticGeometry trait

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-20T21:13:50Z
- **Updated:** 2026-07-20T22:05:52Z
- **Closed:** 2026-07-20T22:05:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/9
- **Merged:** 2026-07-20T22:05:52Z
- **Merge commit:** 953fa7fe841c2bb72d1337eca1b8dd660da67596
- **Base / Head:** main / UOR-Foundation:feature/proof-carrying-semantic-routing
- **Reviews / inline review comments:** 1 / 1
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR implements Phase 1 and Phase 4 of the technical proposal: 1. Core structures (SemanticRouteReferenceV1, FacetRoute, SemanticSpaceManifestV1, ReasoningPlanV1, SemanticInferenceWitnessV1). 2. 1024-bit VSA binary hypervector operations (bind, unbind, bundle, permute, similarity, expand_atom). 3. Pluggable SemanticGeometry trait in the router crate. 4. Unit test coverage for CBOR types, VSA algebra, and selective backoff.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 4739069212 — 2026-07-20T21:18:05Z — chatgpt-codex-connector[bot] — COMMENTED


### 💡 Codex Review

Here are some automated review suggestions for this pull request.

**Reviewed commit:** `472b5d4101`
    

<details> <summary>ℹ️ About Codex in GitHub</summary>
<br/>

[Your team has set up Codex to review pull requests in this repo](https://chatgpt.com/codex/cloud/settings/general). Reviews are triggered when you
- Open a pull request for review
- Mark a draft as ready
- Comment "@codex review".

If Codex has suggestions, it will comment; otherwise it will react with 👍.




Codex can also answer questions or update the PR. Try commenting "@codex address that feedback".
            
</details>

### review-comment 3617638270 — 2026-07-20T21:18:05Z — chatgpt-codex-connector[bot]

_on crates/uor-r4-core/src/semantic/vsa.rs:95_

**<sub><sub>![P2 Badge](https://img.shields.io/badge/P2-yellow?style=flat)</sub></sub>  Add boundaries between atom fields before hashing**

Because these `update` calls stream raw bytes without any delimiter or length prefix, distinct atom tuples can hash identically; for example, `expand_atom("ab", "c", "d")` and `expand_atom("a", "bc", "d")` both feed the same byte sequence. When variable-length prefixes/CIDs/spaces overlap this way, unrelated atoms get the exact same hypervector, defeating the pseudo-orthogonality assumption and causing VSA bindings/routes to alias. Length-prefix or otherwise domain-separate each component before hashing.

Useful? React with 👍 / 👎.


---
_Comment/review ids in this file: 4739069212, 3617638270_

