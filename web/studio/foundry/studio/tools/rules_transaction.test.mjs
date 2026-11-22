import assert from "node:assert/strict";
import { test } from "node:test";
import { commitPreparedProject } from "../shell/project-edit.mjs";
import { applyStoredProjectPatch } from "../shell/generation-patch.mjs";
const oid = "blake3:" + "a".repeat(64), nextOid = "blake3:" + "b".repeat(64);
const before = { phase: "proving", specKappa: oid, logicKappa: "same", proof: null };
const source = { name: "app", reference: oid, files: [["prism/spec.json", "old"]], dependencies: [] };
// Deterministic transaction scheduler, not a substitute for the real IndexedDB CI controls.
function database(state) {
  const tables = new Map(Object.entries({
    apps: new Map([[1, { name: "app" }]]), refs: new Map([["1:main", oid], ["prism:1", state]]),
    files: new Map([["1:prism/spec.json", "old"]]), objects: new Map(), versions: new Map([[1, []]]),
  }));
  const trace = [];
  return { tables, trace, transaction() {
    const copy = structuredClone(tables); let pending = 0, aborted = false;
    const t = { abort() { aborted = true; queueMicrotask(() => t.onabort?.()); }, objectStore(name) {
      const data = copy.get(name);
      const request = action => {
        const q = {}; pending++;
        queueMicrotask(() => {
          if (!aborted) { q.result = action(); q.onsuccess?.(); }
          if (--pending === 0 && !aborted) { for (const [key, value] of copy) tables.set(key, value); t.oncomplete?.(); }
        });
        return q;
      };
      return {
        get: key => request(() => structuredClone(data.get(key))),
        put: (value, key) => request(() => { trace.push(["put", name, key]); data.set(key, structuredClone(value)); }),
        delete: key => request(() => { trace.push(["delete", name, key]); data.delete(key); }),
        openCursor() {
          const rows = [...data], q = {}; let at = 0;
          const advance = () => {
            pending++;
            queueMicrotask(() => {
              if (!aborted) { q.result = at < rows.length ? { key: rows[at][0], value: rows[at++][1], continue: advance } : null; q.onsuccess?.(); }
              if (--pending === 0 && !aborted) { for (const [key, value] of copy) tables.set(key, value); t.oncomplete?.(); }
            });
          };
          advance(); return q;
        },
      };
    } };
    return t;
  } };
}
globalThis.IDBKeyRange = { bound() { return {}; } };
const prepared = { oid: nextOid, files: [["prism/spec.json", "new"]], objects: [[nextOid, new Uint8Array([1])]] };
test("proof completed before source commit wins; stale reword rolls back completely", async () => {
  const completed = { ...before, phase: "proved", proof: { id: "completed" } };
  const db = database(completed), original = structuredClone(db.tables);
  await assert.rejects(commitPreparedProject(db, 1, ["prism/spec.json"], source, prepared, {
    rulesUpdate: { expected: before, state: { ...before, specKappa: nextOid } },
  }), { code: "projectEditConflict" });
  assert.deepEqual(db.tables, original); assert.deepEqual(db.trace, []);
});
test("reword source and matching status commit together with no status deletion", async () => {
  const db = database(before), state = { ...before, specKappa: nextOid };
  await commitPreparedProject(db, 1, ["prism/spec.json"], source, prepared, { rulesUpdate: { expected: before, state } });
  assert.deepEqual(db.tables.get("refs").get("prism:1"), state);
  assert.equal(db.tables.get("files").get("1:prism/spec.json"), "new");
  assert.equal(db.trace.some(([op, store, key]) => op === "delete" && store === "refs" && key === "prism:1"), false);
});
test("unchanged files still compare lifecycle without creating a source version", async () => {
  const db = database(before), state = { ...before, phase: "proved", proof: { id: "done" } };
  const result = await applyStoredProjectPatch(db, 1, { writes: [], deletes: [], renames: [], deps: [] }, {
    source, rulesUpdate: { expected: before, state }, onCommit() { assert.fail("no source commit"); },
  });
  assert.equal(result.changed, 0); assert.equal(result.oid, oid);
  assert.deepEqual(db.tables.get("refs").get("prism:1"), state);
  assert.deepEqual(db.tables.get("versions").get(1), []);
});
