// This directory is prism-devrel's builder/ (afflom/prism-devrel 7b6c54b), brought into the studio and
// owned here now. Its imports read the studio's own prism-code journey (shell/prism, held by
// tools/prism_journey.mjs); facets.mjs also projects a product with several behavior modules, such as
// Foundry itself (tools/foundry_model.mjs). tools/builder/pack.mjs writes assets.mjs.
//
// Every app the studio publishes is a full Prism model. The commit prism-code makes on a template
// instance (prism.js templateCommit) carries the behavior: the spec's model, its proofs and claims. The
// builder (this directory, prism-devrel's builder brought into the studio) writes the same app from one
// prism/prompt/1 whose app is that spec, and adds every other facet: architecture, design, features,
// security and the threat model, quality and compliance, with the claims their theorems make. What the
// prompt leaves out is derived from the spec, never invented.
//
// The builder's files take the place of templateCommit's under app/ and in the register, scenarios,
// tests and CONFORMANCE.md, which now also carry the facet claims. The spec document and site/app.json
// stay templateCommit's: they record the attestation Lean returned, which the builder does not see.
import { buildApp } from "./build.mjs";
import { suiteOf } from "../template.mjs";
import { readSpecFile } from "../spec-file.mjs";

/** The prompt the builder takes for a spec the studio checked and Lean proved. */
export function promptOf(spec, { axioms = {}, repository, parts = null } = {}) {
  // The studio keeps an observed axiom set as the text "[a, b]"; the prompt keeps it as a list.
  const sets = Object.fromEntries(Object.entries(axioms).map(([name, set]) => [name, String(set).replace(/^\[|\]$/g, "").split(",").map((a) => a.trim()).filter(Boolean)]));
  // The parts the chat routed the idea onto (prism/builder/route.mjs) come first; the spec, the observed
  // axioms and the repository are the studio's own and always win.
  return { ...(parts || {}), schema: "prism/prompt/1", app: spec, axioms: sets, ...(repository ? { repository } : {}) };
}

/** `commit` is templateCommit's { files, remove }; the result is the same commit with every facet in it. */
export async function facetCommit(commit, { spec, axioms = {}, repository, parts = null, lastModified = new Date().toISOString().replace(/\.\d{3}Z$/, "Z") }) {
  // The model carries the spec's seal, as templateCommit's does, so the app is held to its JSON.
  const built = await buildApp(promptOf(spec, { axioms, repository, parts }), { lastModified, specAddress: readSpecFile(spec).specAddress });
  const suite = suiteOf(spec);
  const owned = (p) => p.startsWith("app/") || ["model/ids.toml", "CONFORMANCE.md", `features/suites/${suite}.feature`, `crates/conformance/tests/${suite}.rs`].includes(p);
  const files = new Map(commit.files.map((f) => [f.path, f.content]));
  for (const [p, content] of Object.entries(built.files)) if (owned(p)) files.set(p, content);
  const remove = [...new Set([...commit.remove, ...built.remove])].filter((p) => !files.has(p));
  return { files: [...files].map(([path, content]) => ({ path, content })), remove, claims: built.claims };
}
