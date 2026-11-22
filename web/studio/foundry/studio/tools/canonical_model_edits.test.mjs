import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import * as lexlean from "../shell/live/lexlean.mjs";
import * as prism from "../shell/prism.js";
import * as live from "../shell/live/project.mjs";
import { canonicalEditPath, planCanonicalEdit } from "../shell/canonical-edits.mjs";
import { invokeHost } from "../shell/host-boundary.mjs";
import {retiredApplicationPaths, isStatefulModule} from '../shell/application-shape.mjs';
const fixture = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json", import.meta.url)));
const canonical = lexlean.canonicalSource(fixture), modulePath = `prism/${canonical.name}.lex.tex`;
const source = { reference: "snapshot", files: [["prism/spec.json", JSON.stringify(fixture)], [modulePath, canonical.source], ["src/lib/prism.ts", "runtime"]] };
const boundary = { lexlean, prism };
const edit = (filePath, content) => ({ appId: 1, filePath, content });
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
test("module edits preserve presentation and reconstruct the changed semantics", () => {
  const changed = structuredClone(fixture); changed.fields.find(f => f.name === "seats").max = 501;
  const before = JSON.stringify(source);
  const plan = planCanonicalEdit(edit(modulePath, lexlean.canonicalSource(changed).source), source, boundary);
  const parsed = prism.parseSpecTag(plan.text).spec;
  assert.equal(parsed.fields.find(f => f.name === "seats").max, 501);
  assert.deepEqual(parsed.view, fixture.view); assert.equal(parsed.purpose, fixture.purpose);
  assert.equal(JSON.stringify(source), before); assert.equal(plan.warning, null);
});
test("JSON and first-module authoring require checked canonical semantics", () => {
  const plan = planCanonicalEdit(edit("prism/spec.json", JSON.stringify(fixture)), { files: [] }, boundary);
  assert.ok(prism.parseSpecTag(plan.text).spec);
  const text = "a </prism-spec> b";
  const escaped = planCanonicalEdit(edit("prism/spec.json", JSON.stringify({...fixture, purpose:text})), source, boundary);
  assert.equal(prism.parseSpecTag(escaped.text).spec?.purpose, text);
  assert.ok(planCanonicalEdit(edit(modulePath, canonical.source), { files: [] }, boundary));
  assert.throws(() => planCanonicalEdit(edit("prism/Wrong.lex.tex", canonical.source), { files: [] }, boundary), e => e.code === "invalidCanonicalEdit");
});
test("malformed oversized or invalid canonical edits refuse privately", () => {
  const broken = structuredClone(fixture); broken.rules[0].then = "confirmed";
  for (const [path, content] of [["prism/spec.json", JSON.stringify(broken)], ["prism/spec.json", "secret-broken-json"], ["prism/spec.json", "null"], ["prism/spec.json", JSON.stringify(fixture) + " ".repeat(1024 * 1024)], [modulePath, "private-module"], ["prism/spec.json", JSON.stringify({ ...fixture, outcomes: [] })]]) {
    assert.throws(() => planCanonicalEdit(edit(path, content), source, boundary), e => e.code === "invalidCanonicalEdit" && e.message === "The canonical model edit is invalid or unsupported");
  }
});
test("generated runtime edits refuse only for model-owned projects", async () => {
  assert.ok(canonicalEditPath(modulePath)); assert.ok(canonicalEditPath("src/lib/prism.ts")); assert.equal(canonicalEditPath("src/pages/Index.tsx"), false);
  assert.throws(() => planCanonicalEdit(edit("src/lib/prism.ts", "changed"), source, boundary), e => e.code === "canonicalProjectionReadOnly");
  assert.equal(planCanonicalEdit(edit("src/lib/prism.ts", "changed"), { files: [] }, boundary), null);
  assert.equal(planCanonicalEdit(edit("prism/Inactive.lex.tex", "changed"), source, boundary), null);
  for (const code of ["invalidCanonicalEdit", "canonicalProjectionReadOnly", "projectEditConflict"]) {
    const result = await invokeHost("edit-app-file", [], () => { throw Object.assign(Error("private-source"), { code }); }, null);
    assert.equal(result.error.code, code); assert.equal(result.error.kind, "validation"); assert.ok(!JSON.stringify(result).includes("private-source"));
  }
});
test("the actual editor delegates canonical writes to one captured rules transaction", async () => {
  const handlers = new Map(), events = []; let writes = 0, refreshes = 0;
  const start = host.indexOf('handlers.set("edit-app-file",'), end = host.indexOf('handlers.set("delete-app",', start);
  vm.runInNewContext(host.slice(start, end), { structuredClone, handlers, coreReady: Promise.resolve(), dbReady: Promise.resolve({}), canonicalEditPath, planCanonicalEdit, lexlean, prism, captureProjectSource: async () => source,
    rulesAfterTurn: async (id, text, paths, options) => { writes++; assert.equal(options.source, source); assert.ok(prism.parseSpecTag(text).spec); options.onCommit("new-reference"); return { changed: 3 }; }, refreshLive: async () => refreshes++,
    editStoredProject: () => assert.fail("generic writer bypassed canonical transaction"), emit: (...args) => events.push(args) });
  const handler = handlers.get("edit-app-file");
  assert.deepEqual(JSON.parse(JSON.stringify(await handler(edit(modulePath, canonical.source)))), {});
  assert.equal(writes, 1); assert.equal(refreshes, 1); assert.equal(events.length, 1);
  await assert.rejects(handler(edit(modulePath, "invalid")), e => e.code === "invalidCanonicalEdit"); assert.equal(writes, 1);
});
test("model renames delete only the previous active module in the atomic batch", async () => {
  const start = host.indexOf("async function rulesAfterTurn("), end = host.indexOf("// Lean proves", start);
  let batch, options;
  const run = vm.runInNewContext(host.slice(start, end) + "\nrulesAfterTurn", { assertGenerationActive() {}, coreReady: Promise.resolve(), core: {}, dbReady: Promise.resolve({}), captureProjectSource: async () => source,
    prism, lexlean, live, appShape:{SOURCE_PATH:'prism/App.lex.tex',isAppShape:isStatefulModule},retiredApplicationPaths,forgetApplicationFrames(){}, enc: new TextEncoder(), kappa: async bytes => live.kappaOfValue([...bytes]), rulesSnapshotOf: async () => ({state:null,expected:undefined}), nextPhase: async () => "checked", now: () => "now",
    applyStoredProjectPatch: async (_, id, patch, opts) => { batch = patch; options = opts; return { changed: 4, source }; } });
  await run(1, "<prism-spec>" + JSON.stringify({...fixture, name:"renamed-seat"}) + "</prism-spec>");
  assert.deepEqual(JSON.parse(JSON.stringify(batch.deletes)), [{path:modulePath}]); assert.ok(batch.writes.some(w => w.path === "prism/RenamedSeat.lex.tex"));
  assert.ok(batch.writes.some(w => w.path === "prism/spec.json")); assert.ok(batch.writes.some(w => w.path === "src/lib/prism.ts"));
  assert.equal(options.source, source); assert.equal(options.rulesUpdate.state.phase, "checked");
});
