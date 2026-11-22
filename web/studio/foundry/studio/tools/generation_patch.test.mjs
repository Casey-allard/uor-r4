import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import * as lexlean from "../shell/live/lexlean.mjs";
import { stageGenerationPatch, assertGenerationActive } from "../shell/generation-patch.mjs";
const source = { name: "app", reference: "blake3:" + "a".repeat(64), files: [["a", "old"], ["b", "other"]], dependencies: [] };
const tags = (extra = {}) => ({ writes: [], deletes: [], renames: [], deps: [], ...extra });
test("stage applies complete distinct writes deletes and renames without mutating source", () => {
  const result = stageGenerationPatch(source, tags({ writes: [{ path: "c", content: "new" }], renames: [{ from: "a", to: "d" }], deletes: [{ path: "b" }] }));
  assert.deepEqual([...result.files].sort(), [["c", "new"], ["d", "old"]]); assert.deepEqual(source.files, [["a", "old"], ["b", "other"]]); assert.equal(result.changed, 3);
});
test("ambiguous escaping missing and overwriting operations refuse", () => {
  for (const patch of [tags({ writes: [{ path: "../x", content: "x" }] }), tags({ deletes: [{ path: "missing" }] }), tags({ renames: [{ from: "a", to: "b" }] }), tags({ writes: [{ path: "a", content: "x" }], deletes: [{ path: "a" }] }), tags({ writes: [{ path: "x", content: "x" }, { path: "x", content: "y" }] })]) assert.throws(() => stageGenerationPatch(source, patch), e => e.code === "invalidGenerationPatch");
});
test("direct dependencies require exact versions and become snapshot bytes", () => {
  const p = stageGenerationPatch(source, tags({ deps: ["react@19.1.0", "@scope/pkg@1.2.3"] }));
  assert.deepEqual(p.dependencies, ["@scope/pkg@1.2.3", "react@19.1.0"]);
  const declaration = JSON.parse(p.files.get("prism/dependencies.json")); assert.equal(declaration.profile, "foundry/direct-dependency-requests/1"); assert.deepEqual(declaration.dependencies, [{ name: "@scope/pkg", version: "1.2.3" }, { name: "react", version: "19.1.0" }]);
  for (const dep of ["react", "react@^19.1.0", "react@latest", "file:local", "https://example.test/x", "react@01.2.3"]) assert.throws(() => stageGenerationPatch(source, tags({ deps: [dep] })), e => e.code === "unlockedDependencyRequest");
});
test("conflicting versions reserved manifest and oversized batches refuse", () => {
  assert.throws(() => stageGenerationPatch(source, tags({ deps: ["react@19.1.0", "react@19.2.0"] })), e => e.code === "invalidGenerationPatch");
  assert.throws(() => stageGenerationPatch(source, tags({ deps: ["react@19.1.0"], writes: [{ path: "prism/dependencies.json", content: "wrong" }] })), e => e.code === "invalidGenerationPatch");
  assert.throws(() => stageGenerationPatch(source, tags({ writes: Array.from({ length: 1001 }, (_, i) => ({ path: String(i), content: "" })) })), e => e.code === "invalidGenerationPatch");
});
test("identical writes are no-ops and pre-cancellation is typed", () => {
  assert.equal(stageGenerationPatch(source, tags({ writes: [{ path: "a", content: "old" }] })).changed, 0);
  const controller = new AbortController(); controller.abort("secret"); assert.throws(() => assertGenerationActive(controller.signal), e => e.code === "generationCancelled" && !e.message.includes("secret"));
});
test("the host binds generation source signal and committed results", () => {
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  assert.ok(host.includes("applyPrismTags(appId, tags, { signal: abort.signal, source: projectSource, onCommit })"));
  // Code a model writes is never applied; the rules are read against the applied source.
  assert.ok(host.includes("const tags = { ...wrote, writes: [], deletes: [], renames: [], deps: [] };"));
  // What is applied is the reader's turn as read: its lines are the app's notation.
  assert.ok(host.includes("const turn = full;"));
  assert.ok(host.includes("rulesAfterTurn(appId, turn, [], { signal: abort.signal, source: applied.source, onCommit })"));
  assert.ok(!host.includes("lexlean.partialFromTurn(full)") && !host.includes("pushApp(appId, writing.text"));
  assert.ok(host.includes("Committed project changes were retained."));
  // A first turn opens the live frame before the app has rules; the renderer's start for that app must
  // settle on the frame, never wait on a build that yields to it (seen on the live site: "Preparing
  // preview" for good, from the home composer).
  // An app still made of its scaffold is projected too, so a new app's start never builds code (or asks to
  // reach other sites for it) before its rules arrive.
  assert.ok(host.includes("if (liveShown.has(appId) || await projected(appId)) {"));
  assert.ok(host.includes("const projected = async (appId) => (await hasRules(appId)) || (await scaffoldOnly(appId));"));
  assert.ok(host.includes('if (liveShown.has(appId)) { await showLive(appId).catch(() => {}); publish(appId, { phase: "ready", operation: null, lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "run", outcome: "succeeded" } }); return; }'));
  assert.ok(!host.includes('if (updatedFiles) commit = await seal('));
});
