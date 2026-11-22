// FoundrySpecification.projectImportBoundary: normalize, verify closure, then commit.
import { prepareSnapshot } from "./snapshot-restore.mjs";
import { prepareNewProject, prepareExistingProject } from "./project-creation.mjs";
import { snapshotDependencies } from "./project-lifecycle.mjs";
const MiB = 1024 * 1024, ADDRESS = /^blake3:[0-9a-f]{64}$/;
const messages = {
  invalidProjectImport: "Invalid project export",
  projectImportLimitExceeded: "Project export exceeds its import limit",
  projectImportIdentityMismatch: "Imported object bytes do not match their identity",
  projectImportMissingObject: "Project export is missing a required object",
  projectImportSourceMismatch: "Exported files or metadata do not match the committed snapshot",
  projectImportHistoryInvalid: "Project export history is invalid",
  projectImportTimedOut: "Project import preparation exceeded its deadline",
  projectImportCancelled: "Project import preparation was cancelled",
};
const fail = code => Object.assign(new Error(messages[code]), { code });
const object = value => value && typeof value === "object" && !Array.isArray(value);
const address = value => typeof value === "string" && ADDRESS.test(value);
const portable = path => typeof path === "string" && path.length > 0 && !/[\\\u0000-\u001f:]/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
const rows = files => JSON.stringify([...files].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
const date = (value, fallback) => { if (value === undefined || value === null) return fallback; if (typeof value !== "string" || value.length > 64 || !Number.isFinite(Date.parse(value))) throw fail("invalidProjectImport"); return new Date(value).toISOString(); };
export function normalizeProjectImport(input, at = new Date().toISOString()) {
  let doc;
  try { if (typeof input === "string" && input.length > 192 * MiB) throw fail("projectImportLimitExceeded"); doc = typeof input === "string" ? JSON.parse(input) : input; }
  catch (e) { if (e?.code === "projectImportLimitExceeded") throw e; throw fail("invalidProjectImport"); }
  if (!object(doc) || doc.schema !== "hologram-forge/export/1" || !Array.isArray(doc.apps) || !object(doc.objects || {})) throw fail("invalidProjectImport");
  if (doc.apps.length > 64) throw fail("projectImportLimitExceeded");
  const objects = Object.entries(doc.objects || {}); let encoded = 0, fileChars = 0, messageChars = 0, chats = 0, messageCount = 0, versions = 0;
  if (objects.length > 8192) throw fail("projectImportLimitExceeded");
  for (const [key, value] of objects) {
    if (typeof value === "string" && value.length > Math.ceil(16 * MiB / 3) * 4) throw fail("projectImportLimitExceeded");
    if (!address(key) || typeof value !== "string" || (value.length % 4 !== 0 || !/^[A-Za-z0-9+/]*={0,2}$/.test(value))) throw fail("invalidProjectImport");
    encoded += value.length; if (value.length > Math.ceil(16 * MiB / 3) * 4 || encoded > 176 * MiB) throw fail("projectImportLimitExceeded");
  }
  const apps = doc.apps.map(entry => {
    if (!object(entry) || !object(entry.app) || typeof entry.app.name !== "string" || !entry.app.name.trim() || entry.app.name.length > 255 || !object(entry.files || {})) throw fail("invalidProjectImport");
    const files = Object.entries(entry.files || {}); if (files.length > 1000) throw fail("projectImportLimitExceeded");
    for (const [path, text] of files) { if (!portable(path) || typeof text !== "string") throw fail("invalidProjectImport"); fileChars += text.length; if (text.length > 16 * MiB || fileChars > 128 * MiB) throw fail("projectImportLimitExceeded"); }
    const main = entry.main ?? null; if (main !== null && !address(main)) throw fail("invalidProjectImport");
    if (entry.versions !== undefined && !Array.isArray(entry.versions)) throw fail("invalidProjectImport");
    const history = (entry.versions || []).map(v => {
      versions++; if (versions > 4096) throw fail("projectImportLimitExceeded");
      if (!object(v) || !address(v.oid) || (v.message !== undefined && (typeof v.message !== "string" || v.message.length > 1024)) || (v.note != null && (typeof v.note !== "string" || v.note.length > 8192)) || (v.timestamp !== undefined && (!Number.isSafeInteger(v.timestamp) || v.timestamp < 0))) throw fail("invalidProjectImport");
      return { oid: v.oid, message: v.message || "Imported snapshot", timestamp: v.timestamp ?? Math.floor(Date.parse(at) / 1000), dbTimestamp: null, isFavorite: v.isFavorite === true, note: v.note || null };
    });
    if (entry.chats !== undefined && !Array.isArray(entry.chats)) throw fail("invalidProjectImport");
    const copiedChats = (entry.chats || []).map(chat => {
      chats++; if (chats > 4096) throw fail("projectImportLimitExceeded");
      if (!object(chat) || (chat.title !== undefined && (typeof chat.title !== "string" || chat.title.length > 1024)) || !Array.isArray(chat.messages || [])) throw fail("invalidProjectImport");
      const ids = new Set();
      const copiedMessages = (chat.messages || []).map((m, index) => {
        messageCount++; if (messageCount > 16384) throw fail("projectImportLimitExceeded");
        if (!object(m) || !Number.isSafeInteger(m.id) || m.id < 0 || ids.has(m.id) || !["user", "assistant"].includes(m.role) || typeof m.content !== "string") throw fail("invalidProjectImport"); ids.add(m.id);
        messageChars += m.content.length; if (m.content.length > 2 * MiB || messageChars > 32 * MiB) throw fail("projectImportLimitExceeded");
        const out = { id: index + 1, role: m.role, content: m.content, createdAt: date(m.createdAt, at), approvalState: null, commitHash: null, sourceCommitHash: null, dbTimestamp: null, requestId: null, totalTokens: null, model: null };
        if (m.approvalState != null) { if (!["approved", "rejected"].includes(m.approvalState)) throw fail("invalidProjectImport"); out.approvalState = m.approvalState; }
        for (const field of ["commitHash", "sourceCommitHash"]) { if (m[field] != null && !address(m[field])) throw fail("invalidProjectImport"); out[field] = m[field] || null; }
        if (m.model != null) { if (typeof m.model !== "string" || m.model.length > 512) throw fail("invalidProjectImport"); out.model = m.model; }
        if (m.totalTokens != null) { if (!Number.isSafeInteger(m.totalTokens) || m.totalTokens < 0) throw fail("invalidProjectImport"); out.totalTokens = m.totalTokens; }
        return out;
      });
      if (chat.initialCommitHash != null && !address(chat.initialCommitHash)) throw fail("invalidProjectImport");
      return { title: chat.title || "Imported chat", createdAt: date(chat.createdAt, at), messages: copiedMessages, initialCommitHash: chat.initialCommitHash || null, dbTimestamp: null, chatMode: "build", modelSelection: null, referencedApps: [] };
    });
    const extraDeps = entry.app.extraDeps || [];
    if (!Array.isArray(extraDeps) || extraDeps.length > 128 || extraDeps.some(d => typeof d !== "string" || d.length > 256)) throw fail("invalidProjectImport");
    return { app: { name: entry.app.name, createdAt: date(entry.app.createdAt, at), updatedAt: date(entry.app.updatedAt, at), extraDeps: [...extraDeps] }, files: Object.fromEntries(files), main, versions: history, chats: copiedChats };
  });
  return { schema: "hologram-forge/export/1", apps, objects: objects.map(row => [...row]) };
}

export async function prepareProjectImport(input, boundary, { takenNames = [], signal, timeoutMs = 120000 } = {}) {
  // Copy caller-owned primitives before yielding to identity derivation or storage.
  const doc = normalizeProjectImport(input), taken = new Set(takenNames);
  const duration = Number.isFinite(timeoutMs) ? Math.max(1, Math.min(120000, timeoutMs)) : 120000;
  const end = Date.now() + duration;
  let timer, stopped = null, rejectStop;
  const stop = code => { stopped ||= code; rejectStop(fail(stopped)); };
  const deadline = new Promise((_, reject) => { rejectStop = reject; timer = setTimeout(() => stop("projectImportTimedOut"), duration); });
  // Observe cancellation even if it arrives between asynchronous steps.
  deadline.catch(() => {});
  const cancel = () => stop("projectImportCancelled");
  const check = () => { if (signal?.aborted) throw fail("projectImportCancelled"); if (stopped || Date.now() >= end) throw fail(stopped || "projectImportTimedOut"); };
  const timed = async fn => { check(); const value = await Promise.race([Promise.resolve().then(fn), deadline]); check(); return value; };
  signal?.addEventListener("abort", cancel, { once: true });
  const encoder = new TextEncoder(), decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
  const objects = new Map(); let total = 0, messageBytes = 0;
  const add = (key, bytes) => {
    check(); if (bytes.length > 16 * MiB) throw fail("projectImportLimitExceeded");
    if (!objects.has(key)) { total += bytes.length; if (objects.size >= 8192 || total > 128 * MiB) throw fail("projectImportLimitExceeded"); objects.set(key, bytes); }
  };
  const derive = bytes => timed(async () => { let key; try { key = await boundary.derive(bytes); } catch { throw fail("projectImportIdentityMismatch"); } if (!address(key)) throw fail("projectImportIdentityMismatch"); return key; });
  const creation = { core: boundary.core, derive };
  const checked = new Map(); let materializedBytes = 0;
  const snapshot = async oid => {
    const chain = new Set(); let cursor = oid;
    while (cursor && !checked.has(cursor)) {
      check(); if (chain.has(cursor)) throw fail("projectImportHistoryInvalid");
      chain.add(cursor); if (checked.size >= 4096) throw fail("projectImportLimitExceeded");
      if (!objects.has(cursor)) throw fail("projectImportMissingObject");
      let prepared, materializationLimitExceeded = false;
      try { prepared = await timed(() => prepareSnapshot(cursor, { readObject: key => {
        check(); if (!objects.has(key)) throw fail("projectImportMissingObject");
        const bytes = objects.get(key);
        // History may repeatedly decode the same object into distinct retained strings.
        materializedBytes += bytes.length;
        if (materializedBytes > 128 * MiB) { materializationLimitExceeded = true; throw fail("projectImportLimitExceeded"); }
        return bytes;
      }, derive, admit: boundary.admit })); }
      catch (error) { check(); if (materializationLimitExceeded) throw fail("projectImportLimitExceeded"); if (["missingSnapshotObject", "missingFileObject", "snapshotStorageFailed"].includes(error.code)) throw fail("projectImportMissingObject"); throw error; }
      if (prepared.files.length > 1000 || !prepared.name.trim() || prepared.name.length > 255) throw fail("invalidProjectImport");
      // prepareSnapshot has checked this exact immutable body's complete schema.
      const parent = JSON.parse(decoder.decode(objects.get(cursor))).parent;
      checked.set(cursor, prepared); cursor = parent;
      if (chain.has(cursor)) throw fail("projectImportHistoryInvalid");
    }
    return checked.get(oid);
  };
  try {
    check();
    for (const [key, encoded] of doc.objects) {
      const raw = atob(encoded);
      if (btoa(raw) !== encoded) throw fail("invalidProjectImport");
      const bytes = Uint8Array.from(raw, char => char.charCodeAt(0));
      if (await derive(bytes) !== key) throw fail("projectImportIdentityMismatch"); add(key, bytes);
    }
    for (const entry of doc.apps) {
      for (const [, text] of Object.entries(entry.files)) { const bytes = encoder.encode(text); if (decoder.decode(bytes) !== text) throw fail("invalidProjectImport"); add(await derive(bytes), bytes); }
      for (const chat of entry.chats) for (const message of chat.messages) {
        const bytes = encoder.encode(message.content); messageBytes += bytes.length;
        if (bytes.length > 2 * MiB || messageBytes > 32 * MiB) throw fail("projectImportLimitExceeded");
        if (decoder.decode(bytes) !== message.content) throw fail("invalidProjectImport");
      }
      if (entry.main) {
        const current = await snapshot(entry.main);
        if (current.name !== entry.app.name || rows(current.files) !== rows(Object.entries(entry.files))) throw fail("projectImportSourceMismatch");
      } else {
        const initial = await timed(() => prepareNewProject({ name: entry.app.name, files: entry.files }, creation));
        for (const [key, bytes] of initial.objects) add(key, bytes); entry.main = initial.oid;
        await snapshot(entry.main);
      }
      for (const version of entry.versions) await snapshot(version.oid);
      for (const chat of entry.chats) {
        if (chat.initialCommitHash) await snapshot(chat.initialCommitHash);
        for (const message of chat.messages) for (const key of [message.commitHash, message.sourceCommitHash]) if (key) await snapshot(key);
      }
      let deps; try { deps = snapshotDependencies(Object.entries(entry.files)); } catch { throw fail("projectImportSourceMismatch"); }
      if (entry.app.extraDeps.length && JSON.stringify([...entry.app.extraDeps].sort()) !== JSON.stringify([...deps].sort())) throw fail("projectImportSourceMismatch");
      entry.app.extraDeps = deps; entry.from = entry.app.name;
      if (taken.has(entry.app.name)) {
        let name = entry.from + " (moved)";
        for (let n = 2; taken.has(name); n++) name = entry.from + ` (moved ${n})`;
        if (name.length > 255) throw fail("projectImportLimitExceeded");
        const renamed = await timed(() => prepareExistingProject({ name, files: entry.files }, creation, entry.main));
        for (const [key, bytes] of renamed.objects) add(key, bytes);
        entry.app.name = name; entry.main = renamed.oid; await snapshot(entry.main);
      }
      taken.add(entry.app.name);
      if (!entry.versions.some(version => version.oid === entry.main)) entry.versions.unshift({ oid: entry.main, message: "Imported project", timestamp: Math.floor(Date.now() / 1000), dbTimestamp: null, isFavorite: false, note: null });
    }
    check(); return { apps: doc.apps, objects };
  } finally { clearTimeout(timer); signal?.removeEventListener("abort", cancel); }
}
