import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { normalizeProjectImport, prepareProjectImport } from "../shell/project-import.mjs";
import { prepareNewProject, prepareExistingProject } from "../shell/project-creation.mjs";
import { kappa } from "../../try/vendor/prism-code/src/blake3.mjs";
const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
const boundary = { core, derive: kappa, admit: () => true };
const doc = (files = { "a.txt": "\uFEFF😀\r\n" }) => ({ schema: "hologram-forge/export/1", apps: [{ app: { name: "Example" }, files, chats: [] }], objects: {} });
const sealed = async () => { const d = doc(), p = await prepareNewProject({ ...d.apps[0].app, files: d.apps[0].files }, boundary); d.apps[0].main = p.oid; d.objects = Object.fromEntries(p.objects.map(([key, bytes]) => [key, Buffer.from(bytes).toString("base64")])); return d; };
const refuse = (p, code) => assert.rejects(p, error => error.code === code && !error.message.includes("secret"));
test("normalization bounds shapes and excludes authority before asynchronous work", async () => {
  const d = doc(); Object.assign(d.apps[0].app, { grants: ["https://secret.test"], apiKey: "secret", provider: "secret" });
  d.apps[0].rules = { phase: "published" };
  d.apps[0].chats = [{ title: "Conversation", referencedApps: [8], modelSelection: { key: "secret" }, messages: [{ id: 1, role: "user", content: "hello" }] }];
  let unblock; const wait = new Promise(r => { unblock = r; });
  const preparing = prepareProjectImport(d, { ...boundary, derive: async bytes => { await wait; return kappa(bytes); } });
  d.apps[0].app.name = "changed"; d.apps[0].files["a.txt"] = "changed"; unblock();
  const p = await preparing; assert.equal(p.apps[0].app.name, "Example"); assert.equal(p.apps[0].files["a.txt"], "\uFEFF😀\r\n");
  assert.equal(p.apps[0].app.grants, undefined); assert.equal(p.apps[0].app.apiKey, undefined); assert.equal(p.apps[0].app.provider, undefined); assert.equal(p.apps[0].rules, undefined);
  assert.deepEqual(p.apps[0].chats[0].referencedApps, []); assert.equal(p.apps[0].chats[0].modelSelection, null);
  for (const input of ["bad", {}, { ...doc(), apps: [null] }, doc({ "../x": "secret" })]) await refuse(prepareProjectImport(input, boundary), "invalidProjectImport");
  assert.throws(() => normalizeProjectImport({ ...doc(), apps: Array(65).fill(doc().apps[0]) }), { code: "projectImportLimitExceeded" });
  await refuse(prepareProjectImport(doc({ a: "\ud800" }), boundary), "invalidProjectImport");
  await refuse(prepareProjectImport(doc({ a: "😀".repeat(4194305) }), boundary), "projectImportLimitExceeded");
});
test("valid current and historical closure survive exact-byte verification", async () => {
  const d = await sealed(), parent = d.apps[0].main;
  const p = await prepareExistingProject({ name: "Example", files: { "a.txt": "new" } }, boundary, parent);
  d.apps[0].files = { "a.txt": "new" }; d.apps[0].main = p.oid; d.apps[0].versions = [{ oid: parent }];
  for (const [key, bytes] of p.objects) d.objects[key] = Buffer.from(bytes).toString("base64");
  const result = await prepareProjectImport(d, boundary);
  assert.equal(result.apps[0].main, p.oid); assert.equal(result.apps[0].versions.length, 2);
  for (const [key, bytes] of result.objects) assert.equal(kappa(bytes), key);
  assert.ok(result.objects.has(parent));
});
test("corrupt missing mismatched source and invalid history refuse the entire import", async () => {
  let d = await sealed(); d.objects[d.apps[0].main] = Buffer.from("secret").toString("base64");
  await refuse(prepareProjectImport(d, boundary), "projectImportIdentityMismatch");
  d = await sealed(); delete d.objects[d.apps[0].main]; await refuse(prepareProjectImport(d, boundary), "projectImportMissingObject");
  d = await sealed(); d.apps[0].files["a.txt"] = "wrong"; await refuse(prepareProjectImport(d, boundary), "projectImportSourceMismatch");
  d = await sealed(); d.apps[0].app.name = "wrong"; await refuse(prepareProjectImport(d, boundary), "projectImportSourceMismatch");
  d = await sealed(); d.apps[0].versions = [{ oid: "blake3:" + "0".repeat(64) }]; await refuse(prepareProjectImport(d, boundary), "projectImportMissingObject");
  d = await sealed(); await refuse(prepareProjectImport(d, { ...boundary, admit: () => false }), "snapshotCoreRefused");
  // A deliberately dishonest identity oracle makes a parent cycle constructible.
  const cyclic = doc(), key = "blake3:" + "0".repeat(64); cyclic.apps[0].files = {}; cyclic.apps[0].main = key;
  cyclic.objects[key] = Buffer.from(JSON.stringify({ entries: [], name: "Example", parent: key })).toString("base64");
  await refuse(prepareProjectImport(cyclic, { ...boundary, derive: () => key }), "projectImportHistoryInvalid");
});
test("unsealed exports and collisions acquire coherent generated snapshots", async () => {
  const d = doc(); const p = await prepareProjectImport(d, boundary, { takenNames: ["Example", "Example (moved)"] });
  assert.equal(p.apps[0].app.name, "Example (moved 2)");
  const snapshot = JSON.parse(new TextDecoder().decode(p.objects.get(p.apps[0].main)));
  assert.equal(snapshot.name, p.apps[0].app.name); assert.ok(p.objects.has(snapshot.parent)); assert.equal(p.apps[0].versions[0].oid, p.apps[0].main);
  d.apps[0].app.name = "x".repeat(255); await refuse(prepareProjectImport(d, boundary, { takenNames: [d.apps[0].app.name] }), "projectImportLimitExceeded");
});
test("dependency metadata and message byte limits cannot bypass source declarations", async () => {
  const d = doc(); d.apps[0].app.extraDeps = ["react@19.0.0"]; await refuse(prepareProjectImport(d, boundary), "projectImportSourceMismatch");
  d.apps[0].files["prism/dependencies.json"] = JSON.stringify({ profile: "foundry/direct-dependency-requests/1", dependencies: [{ name: "react", version: "19.0.0" }] });
  assert.deepEqual((await prepareProjectImport(d, boundary)).apps[0].app.extraDeps, ["react@19.0.0"]);
  d.apps[0].chats = [{ messages: [{ id: 1, role: "assistant", content: "😀".repeat(524289) }] }];
  await refuse(prepareProjectImport(d, boundary), "projectImportLimitExceeded");
});
test("deadlines cancellation identity failures and actual host parser bindings are enforced", async () => {
  await refuse(prepareProjectImport(doc(), { ...boundary, derive: () => new Promise(() => {}) }, { timeoutMs: 5 }), "projectImportTimedOut");
  const controller = new AbortController(); controller.abort(); await refuse(prepareProjectImport(doc(), boundary, { signal: controller.signal }), "projectImportCancelled");
  const active = new AbortController(), pending = prepareProjectImport(doc(), { ...boundary, derive: () => new Promise(() => {}) }, { signal: active.signal }); active.abort(); await refuse(pending, "projectImportCancelled");
  await refuse(prepareProjectImport(doc(), { ...boundary, derive: () => { throw Error("secret"); } }), "projectImportIdentityMismatch");
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8"), parser = fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8");
  assert.match(host, /await prepareProjectImport\(input,/); assert.match(host, /from "\.\/project-import.mjs"/); assert.match(parser, /"project-import.mjs": "mjs"/);
});

test("history decoding charges repeated file references before materialization", async () => {
  const d = doc({ "shared.txt": "x".repeat(8 * 1024 * 1024) });
  const bytes = new TextEncoder().encode(d.apps[0].files["shared.txt"]), key = kappa(bytes);
  d.objects[key] = Buffer.from(bytes).toString("base64");
  let parent = "";
  for (let n = 0; n < 17; n++) {
    const body = new TextEncoder().encode(JSON.stringify({ entries: [["shared.txt", key, bytes.length]], name: "Example", parent }));
    parent = kappa(body); d.objects[parent] = Buffer.from(body).toString("base64");
  }
  d.apps[0].main = parent;
  let fileDerivations = 0;
  await refuse(prepareProjectImport(d, { ...boundary, derive: value => {
    if (value.length === bytes.length) fileDerivations++;
    return kappa(value);
  } }), "projectImportLimitExceeded");
  // One supplied object, one current source and fifteen decoded history files.
  // Snapshot bodies also consume the budget; file sixteen is refused before hashing/decoding.
  assert.equal(fileDerivations, 17);
});
test("imported maximum message IDs become bounded local sequences", async () => {
  const d = doc();
  d.apps[0].chats = [{ messages: [
    { id: Number.MAX_SAFE_INTEGER, role: "user", content: "first" },
    { id: 0, role: "assistant", content: "second" },
  ] }];
  const messages = (await prepareProjectImport(d, boundary)).apps[0].chats[0].messages;
  assert.deepEqual(messages.map(m => m.id), [1, 2]);
  assert.deepEqual(messages.map(m => m.content), ["first", "second"]);
  const userId = Math.max(...messages.map(m => m.id)) + 1, assistantId = userId + 1;
  assert.ok(Number.isSafeInteger(userId) && Number.isSafeInteger(assistantId));
  assert.notEqual(userId, assistantId);
  d.apps[0].chats[0].messages[1].id = Number.MAX_SAFE_INTEGER;
  await refuse(prepareProjectImport(d, boundary), "invalidProjectImport");
});
