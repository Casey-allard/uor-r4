// FoundrySpecification.shellClosureBoundary. Classic worker/Node-compatible adapter.
(() => {
  "use strict";
  const MiB = 1024 * 1024, HASH = /^[0-9a-f]{64}$/;
  const codes = new Set(["invalidShellManifest", "shellClosureMismatch", "shellFileMismatch", "shellResponseInvalid", "shellClosureLimitExceeded", "shellClosureTimedOut", "shellClosureCancelled", "shellClosureUnavailable"]);
  const fail = code => Object.assign(new Error(`Shell installation refused: ${code}`), { code });
  const exact = (value, keys) => value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).sort().join() === keys;
  const excluded = new Set(["manifest.json", "provenance.json", "warmup.json", "sw.js", "sw.template.js"]);
  const required = ["index.html", "host.js", "core.wasm", "assets/index.js", "assets/index.css", "scaffold/files.json", "editor/vs/loader.js", "editor/package.json", "compiler/package.json", "compiler/esm/browser.min.js", "compiler/esbuild.wasm"];
  const pathOK = path => typeof path === "string" && path.length <= 1024 && /^[A-Za-z0-9_.@+ /-]+$/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
  async function prepare(base, expected, { fetchFn = globalThis.fetch, digest = bytes => crypto.subtle.digest("SHA-256", bytes), signal, timeoutMs = 120000 } = {}) {
    let origin;
    try { origin = new URL(base); if (!["http:", "https:"].includes(origin.protocol) || origin.username || origin.password || origin.search || origin.hash || !origin.pathname.endsWith("/") || !HASH.test(expected)) throw Error(); }
    catch { throw fail("invalidShellManifest"); }
    const duration = Number.isFinite(timeoutMs) ? Math.min(120000, Math.max(1, timeoutMs)) : 120000, end = Date.now() + duration;
    const controller = new AbortController(); let stopped, timer, rejectStop;
    const deadline = new Promise((_, reject) => { rejectStop = reject; }); deadline.catch(() => {});
    const stop = code => { stopped ||= code; controller.abort(); rejectStop(fail(stopped)); };
    const cancel = () => stop("shellClosureCancelled");
    timer = setTimeout(() => stop("shellClosureTimedOut"), duration); signal?.addEventListener("abort", cancel, { once: true });
    const check = () => { if (signal?.aborted) throw fail("shellClosureCancelled"); if (stopped || Date.now() >= end) throw fail(stopped || "shellClosureTimedOut"); };
    const timed = async fn => { check(); const value = await Promise.race([Promise.resolve().then(fn), deadline]); check(); return value; };
    const sha = async bytes => { let hash; try { hash = await timed(() => digest(bytes)); } catch (e) { check(); throw fail("shellClosureUnavailable"); } if (!(hash instanceof ArrayBuffer) || hash.byteLength !== 32) throw fail("shellClosureUnavailable"); return Array.from(new Uint8Array(hash), x => x.toString(16).padStart(2, "0")).join(""); };
    const read = async (path, max) => {
      const url = new URL(path.split("/").map(encodeURIComponent).join("/"), origin);
      let response;
      try { response = await timed(async () => {
        const answer = await fetchFn(url, { method: "GET", redirect: "error", credentials: "omit", cache: "no-store", signal: controller.signal });
        if (controller.signal.aborted) { void answer.body?.cancel().catch(() => {}); check(); }
        return answer;
      }); } catch { check(); throw fail("shellClosureUnavailable"); }
      if (response.status !== 200 || response.redirected || !response.body || (response.url && response.url !== url.href)) { void response.body?.cancel().catch(() => {}); throw fail("shellResponseInvalid"); }
      const length = response.headers.get("content-length");
      if (length !== null && (!/^\d+$/.test(length) || Number(length) > max)) { void response.body.cancel().catch(() => {}); throw fail("shellClosureLimitExceeded"); }
      const reader = response.body.getReader(); let count = 0; const chunks = [];
      try {
        for (;;) { const { done, value } = await timed(() => reader.read()); if (done) break; count += value.byteLength; if (count > max) throw fail("shellClosureLimitExceeded"); chunks.push(value); }
      } finally { void reader.cancel().catch(() => {}); reader.releaseLock(); }
      const bytes = new Uint8Array(count); let offset = 0; for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
      return { url: url.href, bytes, headers: response.headers };
    };
    const decode = bytes => { try { return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)); } catch { throw fail("invalidShellManifest"); } };
    try {
      const manifest = await read("manifest.json", 2 * MiB), data = decode(manifest.bytes);
      if (!exact(data, "closure,files,spec") || data.spec !== "hologram-forge/shell/1" || data.closure !== expected || !Array.isArray(data.files)) throw fail("invalidShellManifest");
      if (!data.files.length || data.files.length > 8192) throw fail("shellClosureLimitExceeded");
      let total = 0, previous = "";
      for (const row of data.files) {
        if (!exact(row, "bytes,path,sha256") || !pathOK(row.path) || row.path <= previous || excluded.has(row.path) || !HASH.test(row.sha256) || !Number.isSafeInteger(row.bytes) || row.bytes < 0) throw fail("invalidShellManifest");
        previous = row.path; total += row.bytes;
        if (row.bytes > 32 * MiB || total > 128 * MiB) throw fail("shellClosureLimitExceeded");
      }
      if (required.some(path => !data.files.some(row => row.path === path))) throw fail("invalidShellManifest");
      const canonical = JSON.stringify(data.files.map(row => ({ bytes: row.bytes, path: row.path, sha256: row.sha256 })));
      if (await sha(new TextEncoder().encode(canonical)) !== expected) throw fail("shellClosureMismatch");
      const entries = new Array(data.files.length); let next = 0;
      await Promise.all(Array.from({ length: Math.min(8, data.files.length) }, async () => {
        for (;;) {
          check(); const index = next++; if (index >= data.files.length) return;
          const row = data.files[index], file = await read(row.path, row.bytes);
          if (file.bytes.length !== row.bytes || await sha(file.bytes) !== row.sha256) throw fail("shellFileMismatch");
          entries[index] = [file.url, new Response(file.bytes, { headers: file.headers })];
        }
      }));
      const provenance = await read("provenance.json", 2 * MiB);
      if (decode(provenance.bytes)?.browser_projection_sha256 !== expected) throw fail("shellClosureMismatch");
      entries.push([manifest.url, new Response(manifest.bytes, { headers: manifest.headers })], [provenance.url, new Response(provenance.bytes, { headers: provenance.headers })]);
      check(); return entries;
    } catch (error) { if (codes.has(error?.code)) throw fail(error.code); check(); throw fail("shellClosureUnavailable"); }
    finally { stopped ||= "shellClosureCancelled"; controller.abort(); clearTimeout(timer); signal?.removeEventListener("abort", cancel); }
  }
  globalThis.ShellClosure = Object.freeze({ prepare });
})();
