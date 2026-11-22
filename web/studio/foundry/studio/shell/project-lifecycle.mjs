// FoundrySpecification.projectLifecycleBoundary: validated lifecycle transactions.
import { prepareExistingProject } from "./project-creation.mjs";
import { captureProjectSource, commitPreparedProject } from "./project-edit.mjs";
import { prepareSnapshot } from "./snapshot-restore.mjs";
import { stageGenerationPatch } from "./generation-patch.mjs";
const fail = code => Object.assign(new Error(code === "projectLifecycleMissing" ? "Project no longer exists" : code === "invalidProjectLifecycle" ? "Invalid project lifecycle request" : "Project lifecycle storage failed"), { code });
const idOf = id => { if (!Number.isSafeInteger(id) || id < 1) throw fail("invalidProjectLifecycle"); return id; };
export function snapshotDependencies(files) {
  const text = new Map(files).get("prism/dependencies.json");
  if (text === undefined) return [];
  try {
    const data = JSON.parse(text);
    if (data.profile !== "foundry/direct-dependency-requests/1" || Object.keys(data).sort().join() !== "dependencies,profile" || !Array.isArray(data.dependencies)) throw Error();
    const requested = data.dependencies.map(d => {
      if (!d || Object.keys(d).sort().join() !== "name,version" || typeof d.name !== "string" || typeof d.version !== "string") throw Error();
      return `${d.name}@${d.version}`;
    });
    if (new Set(requested).size !== requested.length) throw Error();
    if (!requested.length) return [];
    return stageGenerationPatch({ name: "snapshot", reference: "blake3:" + "0".repeat(64), files: [], dependencies: [] }, { writes: [], deletes: [], renames: [], deps: requested }).dependencies;
  } catch { throw fail("invalidProjectLifecycle"); }
}
export async function renameStoredProject(db, input, boundary) {
  const appId = idOf(input?.appId), name = input?.appName;
  if (typeof name !== "string" || !name.trim() || name.length > 255) throw fail("invalidProjectLifecycle");
  const source = await captureProjectSource(db, appId);
  const prepared = await prepareExistingProject({ name, files: Object.fromEntries(source.files) }, boundary, source.reference);
  await commitPreparedProject(db, appId, [], source, prepared, { name, message: "Rename project" });
  boundary.onCommit?.(prepared.oid);
  return prepared.oid;
}
export async function restoreStoredProject(db, appId, oid, boundary, revert = false) {
  idOf(appId);
  const source = await captureProjectSource(db, appId);
  const target = await prepareSnapshot(oid, boundary), dependencies = snapshotDependencies(target.files);
  if (!target.name.trim() || target.name.length > 255) throw fail("invalidProjectLifecycle");
  const prepared = revert ? await prepareExistingProject({ name: target.name, files: Object.fromEntries(target.files) }, boundary, source.reference) : { ...target, objects: [] };
  const changedPaths = [...new Set([...source.files.map(([path]) => path), ...target.files.map(([path]) => path), "prism/spec.json"])];
  await commitPreparedProject(db, appId, changedPaths, source, prepared, { name: target.name, dependencies, recordVersion: revert, message: "Revert to " + oid.slice(0, 22) });
  if (revert) boundary.onCommit?.(prepared.oid);
  return prepared.oid;
}
export async function deleteStoredProject(db, appId) {
  idOf(appId);
  return new Promise((resolve, reject) => {
    let t;
    const refuse = code => { if (t) { try { t.abort(); } catch {} } reject(fail(code || "projectLifecycleStorageFailed")); };
    try {
      t = db.transaction(["apps", "chats", "files", "refs", "versions"], "readwrite");
      t.oncomplete = () => resolve(); t.onabort = () => reject(fail("projectLifecycleStorageFailed")); t.onerror = () => refuse();
      const aq = t.objectStore("apps").get(appId);
      aq.onsuccess = () => {
        if (!aq.result) { refuse("projectLifecycleMissing"); return; }
        try {
          const refs = t.objectStore("refs");
          const mark = (store, value) => {
            const q = refs.get(`allocation:${store}`);
            q.onsuccess = () => { try { if (q.result !== undefined && (!Number.isSafeInteger(q.result) || q.result < 0)) throw Error(); refs.put(Math.max(q.result || 0, value), `allocation:${store}`); } catch { refuse(); } };
          };
          mark("apps", appId);
          t.objectStore("apps").delete(appId); t.objectStore("versions").delete(appId);
          refs.delete(`${appId}:main`); refs.delete(`prism:${appId}`); refs.delete(`stateful:${appId}`); refs.delete(`events:${appId}`); refs.delete(`interaction:${appId}`); refs.delete(`work:${appId}`); refs.delete(`preview:${appId}`); refs.delete(`gallery:${appId}`);
          const files = t.objectStore("files").openCursor(IDBKeyRange.bound(`${appId}:`, `${appId};`, false, true));
          files.onsuccess = () => { try { if (files.result) { files.result.delete(); files.result.continue(); } } catch { refuse(); } };
          let maximumChat = 0;
          const chats = t.objectStore("chats").openCursor();
          chats.onsuccess = () => {
            try {
              const cursor = chats.result;
              if (!cursor) { mark("chats", maximumChat); return; }
              if (!Number.isSafeInteger(cursor.key) || cursor.key < 1) throw Error();
              maximumChat = Math.max(maximumChat, cursor.key);
              if (cursor.value.appId === appId) { refs.delete(`chat-queue:${cursor.key}`); cursor.delete(); }
              cursor.continue();
            } catch { refuse(); }
          };
        } catch { refuse(); }
      };
    } catch { refuse(); }
  });
}
export async function deleteStoredChat(db, chatId) {
  idOf(chatId);
  return new Promise((resolve, reject) => {
    let t;
    const refuse = () => { if (t) { try { t.abort(); } catch {} } reject(fail("projectLifecycleStorageFailed")); };
    try {
      t = db.transaction(["chats", "refs"], "readwrite");
      t.oncomplete = () => resolve(); t.onabort = refuse; t.onerror = refuse;
      const refs = t.objectStore("refs"), q = refs.get("allocation:chats");
      q.onsuccess = () => {
        try {
          if (q.result !== undefined && (!Number.isSafeInteger(q.result) || q.result < 0)) throw Error();
          refs.put(Math.max(q.result || 0, chatId), "allocation:chats"); t.objectStore("chats").delete(chatId); refs.delete(`chat-queue:${chatId}`);
        } catch { refuse(); }
      };
    } catch { refuse(); }
  });
}
export async function updateStoredChat(db, chatId, patch) {
  idOf(chatId);
  let copied;
  try { copied = structuredClone(patch); } catch { throw fail("invalidProjectLifecycle"); }
  if (!copied || Object.keys(copied).some(k => !["title", "messages"].includes(k)) || ("title" in copied && typeof copied.title !== "string") || ("messages" in copied && !Array.isArray(copied.messages))) throw fail("invalidProjectLifecycle");
  return new Promise((resolve, reject) => {
    let t, updated = false;
    const refuse = () => { if (t) { try { t.abort(); } catch {} } reject(fail("projectLifecycleStorageFailed")); };
    try {
      t = db.transaction(["apps", "chats"], "readwrite");
      t.oncomplete = () => resolve(updated); t.onabort = refuse; t.onerror = refuse;
      const q = t.objectStore("chats").get(chatId);
      q.onsuccess = () => {
        if (!q.result) return;
        try {
          const chat = q.result, app = t.objectStore("apps").get(chat.appId);
          app.onsuccess = () => { try { if (app.result) { t.objectStore("chats").put({ ...chat, ...copied }, chatId); updated = true; } } catch { refuse(); } };
        } catch { refuse(); }
      };
    } catch { refuse(); }
  });
}
