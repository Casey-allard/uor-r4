// The publish commit is a full Prism model: for every fixture spec, the commit the studio makes on a
// template instance (prism.js templateCommit, then prism/builder/commit.mjs facetCommit) carries every
// facet module, lists each as an entrypoint, and registers the claims the facet theorems make. The
// builder is prism-devrel's, brought into the studio; prismpm verify in the instance's own gate proves it.
//
//   node --test tools/builder_commit.test.mjs
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { TEMPLATE } from "../shell/prism/builder/assets.mjs";
import { facetCommit, promptOf } from "../shell/prism/builder/commit.mjs";
import { FACET_CLAIMS } from "../shell/prism/builder/build.mjs";
import { validateSpec } from "../shell/prism/spec.mjs";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const prism = await import(pathToFileURL(path.join(root, "shell/prism.js")).href);
const load = async (p) => fs.readFileSync(new URL(p, pathToFileURL(path.join(root, "shell/prism.js"))), "utf8").replace(/\r\n/g, "\n");
const FACETS = ["Architecture", "Compliance", "Design", "Features", "Quality", "Security"];
const specs = fs.readdirSync(path.join(root, "tools/fixtures/specs")).filter((f) => f.endsWith(".json"));

async function commitFor(file) {
  const fixture = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/specs", file), "utf8"));
  const spec = validateSpec(fixture.spec);
  const read = async (p) => TEMPLATE[p] ?? (p === "app/lean-toolchain" ? "leanprover/lean4:v4.32.1\n" : "");
  const args = { spec, axioms: fixture.axioms || {}, attestation: "a".repeat(64), repository: `owner/prism-${spec.name}`, tree: Object.keys(TEMPLATE), read, load };
  const behavior = await prism.templateCommit(args);
  return { spec, behavior, full: await facetCommit(behavior, { ...args, lastModified: "2026-11-13T00:00:00Z" }) };
}

for (const file of specs) {
  test(`${file}: the publish commit carries every facet of the app`, async () => {
    const { spec, behavior, full } = await commitFor(file);
    const files = new Map(full.files.map((f) => [f.path, f.content]));
    const toml = files.get("app/lexlean.toml");
    for (const facet of FACETS) {
      assert.ok(files.has(`app/src/${facet}.lex.tex`), `${facet} is written`);
      assert.match(toml, new RegExp(`"src/${facet}\\.lex\\.tex"`), `${facet} is an entrypoint`);
    }
    for (const pack of ["prism.arch", "prism.qual", "prism.sec"]) assert.match(toml, new RegExp(`package = "${pack}"`));
    // The behavior is templateCommit's, unchanged; the spec document and the page keep its attestation.
    const own = new Map(behavior.files.map((f) => [f.path, f.content]));
    for (const p of [...own.keys()].filter((p) => /^app\/src\/[^/]+\.lex\.tex$/.test(p))) assert.equal(files.get(p), own.get(p), `${p} is the behavior templateCommit wrote`);
    for (const p of [`openspec/specs/${spec.name}/spec.md`, "site/app.json"]) assert.equal(files.get(p), own.get(p), `${p} keeps the attestation`);
    // Every facet theorem is a registered claim with a scenario and a test.
    const facetClaims = full.claims.filter((c) => c.kind === "facet");
    assert.deepEqual(facetClaims.map((c) => c.module), FACET_CLAIMS.map((c) => c.module));
    for (const c of facetClaims) {
      assert.match(files.get("model/ids.toml"), new RegExp(`id = "${c.id}"`));
      assert.ok([...files.values()].some((t) => t.includes(`fn ${c.test}()`)), `${c.id} has its test`);
    }
    assert.equal(full.remove.filter((p) => files.has(p)).length, 0, "nothing written is removed");
  });
}

test("a prompt keeps the axioms the studio observed, as lists", () => {
  const p = promptOf({ name: "x" }, { axioms: { a: "[propext, Quot.sound]", b: "[]" } });
  assert.deepEqual(p.axioms, { a: ["propext", "Quot.sound"], b: [] });
  assert.equal(p.schema, "prism/prompt/1");
});

// ---- the idea routed onto every facet (prism/builder/route.mjs)
const route = await import("../shell/prism/builder/route.mjs");
const parking = () => validateSpec(JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/specs/parking-fee.json"), "utf8")).spec);

test("the example the model is shown projects into every facet", async () => {
  const example = /Example: (<prism-prompt>[\s\S]*<\/prism-prompt>)$/.exec(route.PROMPT_RULES)[1];
  const { parts } = route.parsePromptTag(example);
  const facets = await route.facetsOf(parts, parking());
  assert.ok(facets.ok, facets.issue);
  assert.ok(facets.summary.stakeholders >= 2, "the driver and the Verifier");
  assert.match(route.facetLine(facets), /^Full model \(Decision table\): /);
});

test("an unsound prompt is refused with the builder's reason", async () => {
  const facets = await route.facetsOf({ threats: [{ name: "Leak", text: "x", stride: "tampering", asset: "answer" }] }, parking());
  assert.equal(facets.ok, false);
  assert.match(facets.issue, /needs a control with evidence, or an accepted risk/);
  assert.deepEqual(route.parsePromptTag("<prism-prompt>{nope</prism-prompt>").error.slice(0, 33), "The app's prompt is not valid JSO");
  assert.equal(route.parsePromptTag("no block"), null);
  assert.deepEqual(Object.keys(route.parsePromptTag('<prism-prompt>{"goal":"g","app":{},"axioms":{}}</prism-prompt>').parts), ["goal"], "only prompt parts, never the app");
});

test("the published commit carries the stakeholders the chat routed", async () => {
  const spec = parking();
  const read = async (p) => TEMPLATE[p] ?? (p === "app/lean-toolchain" ? "leanprover/lean4:v4.32.1\n" : "");
  const args = { spec, axioms: {}, attestation: "a".repeat(64), repository: "owner/prism-parking-fee", tree: Object.keys(TEMPLATE), read, load };
  const parts = { stakeholders: [{ name: "Garage owner", text: "Sets the tiers.", concerns: [] }] };
  const full = await facetCommit(await prism.templateCommit(args), { ...args, parts, lastModified: "2026-11-13T00:00:00Z" });
  const arch = full.files.find((f) => f.path === "app/src/Architecture.lex.tex").content;
  assert.match(arch, /"Garage owner"/);
  assert.match(full.files.find((f) => f.path === "app/prompt.json").content, /Garage owner/);
});
