// FoundrySpecification.rulesSourceBoundary: identity plus an atomic status update.
import {isStatefulModule, shapeFailure} from './application-shape.mjs';
const phases = new Set(["drafting", "checked", "proving", "proved", "published"]);
const address = value => typeof value === "string" && /^blake3:[0-9a-f]{64}$/.test(value);
const valid = state => state && typeof state === "object" && !Array.isArray(state) && address(state.specKappa) && phases.has(state.phase);
const fail = code => Object.assign(new Error(`Rule state refused: ${code}`), { code });
const encoder = new TextEncoder();

// Rewords can change source and presentation while one approval still owns the same logic.
export function sameRuleReservation(current, reservation) {
  return !!(current && reservation && current.phase === "proving" && reservation.phase === "proving"
    && current.approvalId === reservation.approvalId
    && (current.logicKappa || current.specKappa) === (reservation.logicKappa || reservation.specKappa));
}

function capture(db, appId) {
  return new Promise((resolve, reject) => {
    try {
      const t = db.transaction(["files", "refs"], "readonly");
      const source = t.objectStore("files").get(`${appId}:prism/spec.json`);
      const application = t.objectStore('files').get(`${appId}:prism/App.lex.tex`);
      const state = t.objectStore("refs").get(`prism:${appId}`);
      t.oncomplete = () => resolve({ source: source.result, state: state.result, application:application.result });
      t.onerror = t.onabort = () => reject(fail("rulesStorageFailed"));
    } catch { reject(fail("rulesStorageFailed")); }
  });
}

async function matches(source, state, derive) {
  if (typeof source !== "string" || !valid(state)) return false;
  try { return await derive(encoder.encode(source)) === state.specKappa; }
  catch { throw fail("rulesIdentityFailed"); }
}

export async function readBoundRulesSnapshot(db, appId, derive) {
  const captured = await capture(db, appId);
  return { state: !isStatefulModule(captured.application) && await matches(captured.source, captured.state, derive) ? captured.state : null, expected: captured.state, conflict:typeof captured.source === 'string' && isStatefulModule(captured.application) };
}
export async function readBoundRules(db, appId, derive) {
  const snapshot = await readBoundRulesSnapshot(db, appId, derive);
  if (snapshot.conflict) throw shapeFailure('applicationShapeConflict');
  return snapshot.state;
}

export async function saveBoundRules(db, appId, state, derive, expected, signal) {
  if (signal?.aborted) return false;
  // Freeze both caller-owned records before any asynchronous work.
  let next, prior;
  try { next = structuredClone(state); if (!valid(next)) throw fail("invalidRulesState"); prior = expected === undefined ? undefined : JSON.stringify(expected); }
  catch { throw fail("invalidRulesState"); }
  const captured = await capture(db, appId);
  if (isStatefulModule(captured.application) || !await matches(captured.source, next, derive) || signal?.aborted) return false;
  // The exact raw source comparison closes the race while hashing outside IndexedDB.
  return new Promise((resolve, reject) => {
    let t, written = false;
    const clean = () => signal?.removeEventListener("abort", cancel);
    const cancel = () => { if (t) { try { t.abort(); } catch {} } };
    const refuse = () => { clean(); if (signal?.aborted) resolve(false); else reject(fail("rulesStorageFailed")); };
    try {
      if (signal?.aborted) { resolve(false); return; }
      signal?.addEventListener("abort", cancel, { once: true });
      t = db.transaction(["files", "refs"], "readwrite");
      t.oncomplete = () => { clean(); resolve(written); };
      t.onerror = t.onabort = refuse;
      const source = t.objectStore("files").get(`${appId}:prism/spec.json`);
      const application = t.objectStore('files').get(`${appId}:prism/App.lex.tex`);
      const current = t.objectStore("refs").get(`prism:${appId}`);
      let finished = 0;
      const ready = () => {
        if (++finished !== 3) return;
        try {
          if (application.result !== captured.application || source.result !== captured.source || (prior !== undefined && JSON.stringify(current.result) !== prior)) return;
          t.objectStore("refs").put(next, `prism:${appId}`);
          written = true;
        } catch { t.abort(); }
      };
      source.onsuccess = current.onsuccess = application.onsuccess = ready;
    } catch {
      if (t) { try { t.abort(); } catch {} }
      refuse();
    }
  });
}
