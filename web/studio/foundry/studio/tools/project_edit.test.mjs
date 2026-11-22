import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import { canonicalEditPath } from "../shell/canonical-edits.mjs";
import { editStoredProject } from "../shell/project-edit.mjs";
import { prepareExistingProject } from "../shell/project-creation.mjs";
import { kappa } from "../../try/vendor/prism-code/src/blake3.mjs";
const parent = "blake3:" + "a".repeat(64);
test("existing snapshot delegates the exact parent to generated preimage", async () => {
  const calls = [];
  const core = { run: value => { calls.push(value); return { bytes: JSON.stringify(value) }; } };
  const prepared = await prepareExistingProject({ name: "app", files: { "x": "new" } }, { core, derive: kappa }, parent);
  assert.equal(calls[0].op, "preimage"); assert.equal(calls[0].parent, parent); assert.equal(prepared.files[0][1], "new");
});
test("invalid parents refuse before core or identity work", async () => {
  for (const value of ["", "wrong", null]) await assert.rejects(prepareExistingProject({ name: "x", files: {} }, { core: { run() { assert.fail("core called"); } }, derive() { assert.fail("hash called"); } }, value), e => e.code === "projectPreparationFailed");
});
test("invalid edit inputs refuse before opening storage", async () => {
  const db = { transaction() { assert.fail("storage touched"); } };
  for (const input of [null, { appId: 0, filePath: "x", content: "x" }, { appId: 1, filePath: "../x", content: "x" }, { appId: 1, filePath: "x", content: 4 }]) await assert.rejects(editStoredProject(db, input, {}), e => e.code === "invalidProjectEdit");
});
test("the actual host delegates edits without separate object or file writes", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = source.indexOf('handlers.set("edit-app-file",'); const end = source.indexOf('handlers.set("delete-app",', start);
  const calls = [], handlers = new Map();
  vm.runInNewContext(source.slice(start, end), { structuredClone, handlers, canonicalEditPath, coreReady: Promise.resolve(), dbReady: Promise.resolve({}), core: {}, kappa() {}, editStoredProject: async (...args) => { calls.push(args); return "oid"; }, emit: (...args) => calls.push(args) });
  const input = { appId: 1, filePath: "x", content: "new" };
  assert.deepEqual(JSON.parse(JSON.stringify(await handlers.get("edit-app-file")(input))), {});
  assert.deepEqual(calls[0][1], input); assert.notEqual(calls[0][1], input); assert.equal(calls[1][0], "version-created"); assert.ok(!source.includes("const rawEdit ="));
});
test("typed edit failures and parser inventory are canonical", () => {
  const source = fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex", import.meta.url), "utf8");
  for (const name of ["ProjectEditError", "invalidProjectEdit", "projectEditConflict", "projectEditStorageFailed", "projectEditStateInvalid", "projectEditBoundary"]) assert.ok(source.includes(`name: ${name},`));
  assert.ok(fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8").includes('"project-edit.mjs": "mjs"'));
});
