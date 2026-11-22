// FoundrySpecification.buildModuleBoundary: bounded retrieval and exact byte receipts.
const messages = {
  buildModuleOriginRefused: "Build module origin is not approved",
  buildModuleResponseInvalid: "Build module response is invalid",
  buildModuleUnavailable: "Build module is unavailable",
  buildModuleTooLarge: "Build module exceeds its byte limit",
  buildModuleLimitExceeded: "Build module closure exceeds its limit",
  buildModuleTimedOut: "Build module operation exceeded its deadline",
  buildModuleCancelled: "Build module operation was cancelled",
  buildModuleNetworkFailed: "Build module request failed",
  buildModuleEncodingInvalid: "Build module is not exact UTF8 text",
  buildModuleIdentityFailed: "Build module identity could not be computed",
};
const fail = code => Object.assign(new Error(messages[code]), { code });
const MiB = 1024 * 1024;
const media = new Set(["application/javascript", "text/javascript", "application/ecmascript", "text/ecmascript", "text/css"]);
export function approvedModuleURL(input) {
  let url;
  try { if (typeof input !== "string" || input.length > 8192) throw Error(); url = new URL(input); } catch { throw fail("buildModuleOriginRefused"); }
  if (url.origin !== "https://esm.sh" || url.username || url.password || url.href.includes("#")) throw fail("buildModuleOriginRefused");
  return url.href;
}
export async function withBuildModules(operation, { fetch = globalThis.fetch, signal, timeoutMs = 120000, digest = bytes => globalThis.crypto.subtle.digest("SHA-256", bytes) } = {}) {
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 120000 || typeof operation !== "function") throw fail("buildModuleResponseInvalid");
  const controller = new AbortController(), requests = new Map(), records = new Map(), readers = new Set();
  let total = 0, timer, stopped, rejectStop, closed = false;
  const interrupted = new Promise((_, reject) => { rejectStop = reject; });
  const stop = code => { if (!stopped) { stopped = fail(code); controller.abort(); rejectStop(stopped); } };
  const abort = () => stop("buildModuleCancelled");
  const active = () => { if (stopped) throw stopped; if (closed) throw fail("buildModuleCancelled"); };
  const load = async url => {
    active();
    let response;
    try { response = await fetch(url, { method: "GET", redirect: "error", credentials: "omit", cache: "no-store", signal: controller.signal }); }
    catch { active(); throw fail("buildModuleNetworkFailed"); }
    if (stopped || closed) { void response.body?.cancel().catch(() => {}); active(); }
    const contentType = (response.headers.get("content-type") || "").split(";")[0].trim().toLowerCase();
    if (response.status !== 200 || !media.has(contentType) || !response.body) {
      void response.body?.cancel().catch(() => {});
      throw fail(response.status !== 200 ? "buildModuleUnavailable" : "buildModuleResponseInvalid");
    }
    const reader = response.body.getReader(); readers.add(reader); const chunks = []; let length = 0;
    try {
      for (;;) {
        const next = await reader.read(); active(); if (next.done) break;
        length += next.value.length; total += next.value.length;
        if (length > 8 * MiB) throw fail("buildModuleTooLarge");
        if (total > 64 * MiB) throw fail("buildModuleLimitExceeded");
        chunks.push(Uint8Array.from(next.value));
      }
    } catch (error) { active(); if (error?.code && Object.hasOwn(messages, error.code)) throw error; throw fail("buildModuleNetworkFailed"); }
    finally { readers.delete(reader); void reader.cancel().catch(() => {}); reader.releaseLock(); }
    const bytes = new Uint8Array(length); let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
    let text;
    try {
      text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
      const roundtrip = new TextEncoder().encode(text);
      if (roundtrip.length !== bytes.length || roundtrip.some((b, i) => b !== bytes[i])) throw Error();
    } catch { throw fail("buildModuleEncodingInvalid"); }
    let hash;
    try { const raw = await digest(bytes.slice()); if (!(raw instanceof ArrayBuffer) || raw.byteLength !== 32) throw Error(); hash = [...new Uint8Array(raw)].map(b => b.toString(16).padStart(2, "0")).join(""); }
    catch { active(); throw fail("buildModuleIdentityFailed"); }
    active(); records.set(url, { url, contentType, sha256: hash, bytes: length, path: `build/modules/${hash}.txt`, content: bytes });
    return text;
  };
  const fetchText = input => {
    active(); const url = approvedModuleURL(input);
    if (!requests.has(url)) {
      if (requests.size >= 512) throw fail("buildModuleLimitExceeded");
      const pending = load(url); pending.catch(() => {}); requests.set(url, pending);
    }
    return requests.get(url);
  };
  try {
    signal?.addEventListener("abort", abort, { once: true }); timer = setTimeout(() => stop("buildModuleTimedOut"), timeoutMs);
    if (signal?.aborted) abort();
    const work = async () => {
      active(); const value = await operation({ fetchText, get count() { return requests.size; } }); active();
      await Promise.all(requests.values()); active();
      const modules = [...records.values()].sort((a, b) => a.url < b.url ? -1 : a.url > b.url ? 1 : 0);
      const files = new Map(modules.map(row => [row.path, row.content]));
      return { ...value, moduleClosure: { profile: "foundry/build-module-closure/1", modules: modules.map(({ content, ...row }) => row) }, moduleFiles: [...files].map(([path, bytes]) => ({ path, bytes: bytes.slice() })) };
    };
    return await Promise.race([interrupted, work()]);
  } finally {
    closed = true; clearTimeout(timer); signal?.removeEventListener("abort", abort); controller.abort();
    for (const reader of readers) void reader.cancel().catch(() => {});
  }
}
