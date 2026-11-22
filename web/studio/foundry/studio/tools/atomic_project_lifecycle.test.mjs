import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { snapshotDependencies, renameStoredProject, restoreStoredProject, deleteStoredProject } from "../shell/project-lifecycle.mjs";
const manifest = dependencies => [["prism/dependencies.json", JSON.stringify({ profile: "foundry/direct-dependency-requests/1", dependencies })]];
test("snapshot dependency metadata is empty only when absent or explicitly empty", () => {
  assert.deepEqual(snapshotDependencies([]), []); assert.deepEqual(snapshotDependencies(manifest([])), []);
});
test("snapshot direct dependencies restore exact stable versions in canonical order", () => {
  assert.deepEqual(snapshotDependencies(manifest([{ name: "react", version: "19.1.0" }, { name: "@scope/pkg", version: "1.2.3" }])), ["@scope/pkg@1.2.3", "react@19.1.0"]);
});
test("malformed dependency snapshots cannot clear or substitute metadata", () => {
  for (const files of [[["prism/dependencies.json", "{"]], manifest([{ name: "react", version: "latest" }]), manifest([{ name: "react", version: "1.0.0" }, { name: "react", version: "2.0.0" }]), manifest([{ name: "react", version: "1.0.0", url: "elsewhere" }]), manifest([{ name: "react", version: "1.0.0" }, { name: "react", version: "1.0.0" }])]) assert.throws(() => snapshotDependencies(files), e => e.code === "invalidProjectLifecycle");
});
test("invalid lifecycle identifiers refuse before touching storage", async () => {
  const db = { transaction() { assert.fail("invalid request touched storage"); } };
  for (const appId of [0, -1, 1.5, "1", Number.MAX_SAFE_INTEGER + 1]) {
    await assert.rejects(deleteStoredProject(db, appId), e => e.code === "invalidProjectLifecycle");
    await assert.rejects(restoreStoredProject(db, appId, "", {}), e => e.code === "invalidProjectLifecycle");
    await assert.rejects(renameStoredProject(db, { appId, appName: "ok" }, {}), e => e.code === "invalidProjectLifecycle");
  }
});
test("invalid rename and failed deletion expose fixed public errors", async () => {
  for (const appName of [null, "", "  ", "a".repeat(256)]) await assert.rejects(renameStoredProject({}, { appId: 1, appName }, {}), e => e.code === "invalidProjectLifecycle");
  await assert.rejects(deleteStoredProject({ transaction() { throw Error("private database values"); } }, 1), e => e.code === "projectLifecycleStorageFailed" && !e.message.includes("private"));
});
test("host lifecycle uses coherent adapters without a second snapshot seal", () => {
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  for (const name of ["renameStoredProject", "restoreStoredProject", "deleteStoredProject"]) assert.ok(host.includes(name + "(await dbReady,"));
  assert.ok(host.includes('versionPreview.dispatch(appId,{type:"RESTORE",versionId:previousVersionId'));
  assert.ok(!host.includes("async function seal("));
  assert.ok(fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8").includes('"project-lifecycle.mjs": "mjs"'));
});
