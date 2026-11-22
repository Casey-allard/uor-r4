// FoundrySpecification.snapshotRestoreBoundary: checked browser storage adapter.
const MiB = 1024 * 1024;
const address = value => typeof value === "string" && /^blake3:[0-9a-f]{64}$/.test(value);
const fail = code => Object.assign(new Error(`Snapshot restoration refused: ${code}`), { code });
const exact = (value, fields) => value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).sort().join(",") === fields;
const portable = path => typeof path === "string" && path.length > 0 && !/[\\\u0000-\u001f:]/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const encoder = new TextEncoder();

export async function prepareSnapshot(oid, { readObject, derive, admit }) {
  if (!address(oid)) throw fail("invalidSnapshot");
  const read = async key => {
    try { return await readObject(key); }
    catch { throw fail("snapshotStorageFailed"); }
  };
  const identity = async bytes => {
    try { return await derive(bytes); }
    catch { throw fail("snapshotIdentityMismatch"); }
  };
  const stored = await read(oid);
  if (stored == null) throw fail("missingSnapshotObject");
  if (!(stored instanceof Uint8Array)) throw fail("invalidSnapshot");
  if (stored.length > 2 * MiB) throw fail("snapshotLimitExceeded");
  const body = stored.slice();
  const derived = await identity(body);
  if (derived !== oid) throw fail("snapshotIdentityMismatch");
  try { if (await admit(derived, oid) !== true) throw fail("snapshotCoreRefused"); }
  catch { throw fail("snapshotCoreRefused"); }
  let state;
  try { state = JSON.parse(decoder.decode(body)); }
  catch { throw fail("invalidSnapshot"); }
  if (!exact(state, "entries,name,parent") || typeof state.name !== "string" || !(state.parent === "" || address(state.parent)) || !Array.isArray(state.entries)) throw fail("invalidSnapshot");
  if (state.entries.length > 4096) throw fail("snapshotLimitExceeded");
  const seen = new Set(), files = [];
  let total = 0;
  for (const entry of state.entries) {
    if (!Array.isArray(entry) || entry.length !== 3) throw fail("invalidSnapshot");
    const [path, key, length] = entry;
    if (!portable(path) || seen.has(path) || !address(key) || !Number.isSafeInteger(length) || length < 0) throw fail("invalidSnapshot");
    seen.add(path);
    total += length;
    if (length > 16 * MiB || total > 64 * MiB) throw fail("snapshotLimitExceeded");
  }
  for (const [path, key, length] of state.entries) {
    const value = await read(key);
    if (value == null) throw fail("missingFileObject");
    if (!(value instanceof Uint8Array) || value.length !== length) throw fail("fileIdentityMismatch");
    const bytes = value.slice();
    if (await identity(bytes) !== key) throw fail("fileIdentityMismatch");
    let text;
    try { text = decoder.decode(bytes); }
    catch { throw fail("invalidFileEncoding"); }
    const roundtrip = encoder.encode(text);
    if (roundtrip.length !== bytes.length || roundtrip.some((byte, index) => byte !== bytes[index])) throw fail("invalidFileEncoding");
    files.push([path, text]);
  }
  return { oid, files, name: state.name, parent: state.parent };
}

export async function restoreSnapshot(db, appId, oid, boundary) {
  if (!Number.isSafeInteger(appId) || appId < 1) throw fail("invalidSnapshot");
  const prepared = await prepareSnapshot(oid, boundary);
  await new Promise((resolve, reject) => {
    let transaction;
    const refuse = () => reject(fail("snapshotStorageFailed"));
    try {
      transaction = db.transaction(["files", "refs"], "readwrite");
      transaction.oncomplete = resolve;
      transaction.onerror = refuse;
      transaction.onabort = refuse;
      const store = transaction.objectStore("files"), prefix = `${appId}:`;
      const cursor = store.openCursor();
      cursor.onsuccess = () => {
        try {
          if (cursor.result) {
            if (typeof cursor.result.key === "string" && cursor.result.key.startsWith(prefix)) cursor.result.delete();
            cursor.result.continue();
          } else {
            for (const [path, text] of prepared.files) store.put(text, prefix + path);
            transaction.objectStore("refs").put(prepared.oid, prefix + "main");
            transaction.objectStore("refs").delete(`prism:${appId}`);
          }
        } catch { transaction.abort(); }
      };
    } catch {
      if (transaction) { try { transaction.abort(); } catch {} }
      refuse();
    }
  });
  return oid;
}
