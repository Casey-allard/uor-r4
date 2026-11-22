// Production batch adapters against real IndexedDB, including mid-write abort.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
const apply = host.slice(host.indexOf("async function applyPrismTags("), host.indexOf("window.__chat ="));
const rules = host.slice(host.indexOf("async function rulesAfterTurn("), host.indexOf("// Lean proves"));
const proof = host.slice(host.indexOf("async function proveRules("), host.indexOf('// "approve"'));
const hostModule = `import { sameRuleReservation } from "/rules-source.mjs";\nimport { retiredApplicationPaths, isStatefulModule } from "/application-shape.mjs";\nconst appShape = {SOURCE_PATH:"prism/App.lex.tex",isAppShape:isStatefulModule};\nimport { copyGenerationInput, assertCanonicalGeneration } from "/canonical-generation.mjs";\nimport * as lexlean from "/live/lexlean.mjs";\nexport function hostAdapters(env) { const { coreReady, kernelReady, dbReady, core, kappa, applyStoredProjectPatch, assertGenerationActive, captureProjectSource, prism, live, rulesOf, rulesSnapshotOf, saveRules, nextPhase, now, enc, put, refreshLive, forgetApplicationFrames } = env; ${apply}\n${rules}\n${proof}\nreturn { applyPrismTags, rulesAfterTurn, proveRules }; }`;
const sources = new Map([
  ["/browser-chat-modes.mjs", fs.readFileSync(new URL("../shell/browser-chat-modes.mjs", import.meta.url))],
  ["/chat-mode-catalog.mjs", fs.readFileSync(new URL("../shell/chat-mode-catalog.mjs", import.meta.url))],
  ["/chat-contracts.mjs", fs.readFileSync(new URL("../shell/chat-contracts.mjs", import.meta.url))],

  ["/theme-library.mjs", fs.readFileSync(new URL("../shell/theme-library.mjs", import.meta.url))],
  ["/version-preview-codec.mjs", fs.readFileSync(new URL("../shell/version-preview-codec.mjs", import.meta.url))],["/host-adapters.mjs", hostModule], ["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))]]);
for (const name of ["application-shape.mjs", "canonical-generation.mjs", "generation-patch.mjs", "project-edit.mjs", "project-creation.mjs", "storage-request.mjs", "rules-source.mjs"]) sources.set("/" + name, fs.readFileSync(new URL("../shell/" + name, import.meta.url)));
for (const name of ["live/lexlean.mjs", "prism/render.mjs", "prism/spec.mjs"]) sources.set("/" + name, fs.readFileSync(new URL("../shell/" + name, import.meta.url)));
const server = http.createServer((request, response) => { const source = sources.get(request.url); response.setHeader("content-type", source ? "text/javascript" : "text/html"); response.end(source || "<!doctype html><title>Generation patch acceptance</title>"); });
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { applyStoredProjectPatch, assertGenerationActive } = await import("/generation-patch.mjs");
    const { captureProjectSource } = await import("/project-edit.mjs"); const { prepareNewProject, createStoredProject } = await import("/project-creation.mjs");
    const { storageRequest } = await import("/storage-request.mjs"); const { kappa } = await import("/blake3.mjs"); const { hostAdapters } = await import("/host-adapters.mjs");
    const check = (v, m) => { if (!v) throw Error(m); };
    const stores = ["apps", "chats", "files", "objects", "refs", "versions"];
    const db = await new Promise((resolve, reject) => { const q = indexedDB.open("generation-patch", 1); q.onupgradeneeded = () => stores.forEach(s => q.result.createObjectStore(s)); q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error); });
    const get = (store, key) => storageRequest(db, store, "readonly", s => s.get(key));
    const put = (store, key, value) => storageRequest(db, store, "readwrite", s => s.put(value, key));
    const snapshot = async () => JSON.stringify(await Promise.all(stores.map(store => storageRequest(db, store, "readonly", s => s.getAll()))));
    const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
    const boundary = { core, derive: kappa };
    const seed = async () => createStoredProject(db, await prepareNewProject({ name: "app", files: { a: "old", b: "other" } }, boundary));
    const tags = extra => ({ writes: [], deletes: [], renames: [], deps: [], ...extra });
    const first = await seed(); const commits = [];
    const applied = await applyStoredProjectPatch(db, first.app.id, tags({ writes: [{ path: "c", content: "new" }], renames: [{ from: "a", to: "d" }], deletes: [{ path: "b" }], deps: ["react@19.1.0"] }), { ...boundary, onCommit: id => commits.push(id) });
    check(applied.changed === 4 && commits.length === 1 && commits[0] === applied.oid, "batch commit not reported once");
    check(await get("files", `${first.app.id}:a`) === undefined && await get("files", `${first.app.id}:b`) === undefined && await get("files", `${first.app.id}:d`) === "old" && await get("files", `${first.app.id}:c`) === "new", "batch files incorrect");
    check((await get("apps", first.app.id)).extraDeps[0] === "react@19.1.0" && JSON.parse(await get("files", `${first.app.id}:prism/dependencies.json`)).dependencies[0].version === "19.1.0", "dependency declaration not bound");
    check((await get("versions", first.app.id)).length === 2, "batch did not create one version");
    const firstBody = await get("objects", applied.oid); check(kappa(firstBody) === applied.oid, "snapshot identity wrong");
    const second = await seed(), source = await captureProjectSource(db, second.app.id);
    const adapters = hostAdapters({ forgetApplicationFrames(){}, coreReady: Promise.resolve(), dbReady: Promise.resolve(db), core, kappa, applyStoredProjectPatch, assertGenerationActive, captureProjectSource, live: { logicOf: spec => spec, kappaOfValue: spec => JSON.stringify(spec) }, rulesOf: async () => null, rulesSnapshotOf: async () => ({ state: null, expected: undefined }), saveRules: async () => true, nextPhase: async () => "checked", now: () => new Date().toISOString(), enc: new TextEncoder(), prism: { parseSpecTag: () => ({ spec: { example: true } }), parsePromptTag: () => null, facetsOf: async () => null, checkSpec: () => ({ ok: true, module: "checked module", spec: {}, issues: [], vectors: 1 }), screenSource: () => "screen", card: () => "checked" } });
    const empty = await adapters.applyPrismTags(second.app.id, tags({}), { source });
    await put("files", `${second.app.id}:b`, "intervening"); const beforeStale = await snapshot();
    let refused = false; try { await adapters.rulesAfterTurn(second.app.id, "spec", [], { source: empty.source }); } catch (e) { refused = e.code === "projectEditConflict"; }
    check(refused && await snapshot() === beforeStale, "no-op batch lost source binding before rules");
    const copying = await seed(), mutableSource = await captureProjectSource(db, copying.app.id);
    let resumeCopy, beganCopy;
    const copyGate = new Promise(r => { resumeCopy = r; }), copyStarted = new Promise(r => { beganCopy = r; });
    const copyingPatch = applyStoredProjectPatch(db, copying.app.id, tags({ writes: [{ path: "a", content: "candidate" }] }), { core, source: mutableSource, derive: async bytes => { beganCopy(); await copyGate; return kappa(bytes); } });
    await copyStarted; await put("files", `${copying.app.id}:b`, "intervening"); mutableSource.files.find(row => row[0] === "b")[1] = "intervening";
    const beforeCopy = await snapshot(); resumeCopy(); refused = false;
    try { await copyingPatch; } catch (e) { refused = e.code === "projectEditConflict"; }
    check(refused && await snapshot() === beforeCopy, "caller mutation changed the captured comparison source");
    const { saveBoundRules } = await import("/rules-source.mjs");
    const statusOwner = await seed(), statusSource = "rules";
    await put("files", `${statusOwner.app.id}:prism/spec.json`, statusSource);
    const statusBefore = await snapshot(), statusAbort = new AbortController();
    const statusDb = { transaction(names, mode) {
      const t = db.transaction(names, mode); if (mode === "readonly") return t;
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => {
          const store = target.objectStore(name); if (name !== "refs") return store;
          return { get: store.get.bind(store), put(...args) { const q = store.put(...args); q.addEventListener("success", () => statusAbort.abort(), { once: true }); return q; } };
        };
        const value = target[key]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, key, value) { target[key] = value; return true; } });
    } };
    const saved = await saveBoundRules(statusDb, statusOwner.app.id, { phase: "checked", specKappa: kappa(new TextEncoder().encode(statusSource)) }, kappa, undefined, statusAbort.signal);
    check(saved === false && await snapshot() === statusBefore, "cancelled rule status committed");
    const third = await seed(); const beforeCancel = await snapshot(); let release, started;
    const gate = new Promise(r => { release = r; }), began = new Promise(r => { started = r; }); const controller = new AbortController();
    const preparing = applyStoredProjectPatch(db, third.app.id, tags({ writes: [{ path: "a", content: "cancelled" }] }), { core, signal: controller.signal, derive: async bytes => { started(); await gate; return kappa(bytes); } });
    await began; controller.abort(); release(); refused = false; try { await preparing; } catch (e) { refused = e.code === "generationCancelled"; }
    check(refused && await snapshot() === beforeCancel, "cancelled preparation changed storage");
    const active = new AbortController(); let committedAfterAbort = false;
    const abortingDb = { transaction(names, mode) {
      const t = db.transaction(names, mode); if (mode === "readonly") return t;
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => {
          const store = target.objectStore(name); if (name !== "files") return store;
          return { openCursor: store.openCursor.bind(store), delete: store.delete.bind(store), put(...args) { const q = store.put(...args); q.addEventListener("success", () => active.abort(), { once: true }); return q; } };
        };
        const value = target[key]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, key, value) { target[key] = value; return true; } });
    } };
    refused = false; try { await applyStoredProjectPatch(abortingDb, third.app.id, tags({ writes: [{ path: "a", content: "mid-write" }] }), { ...boundary, signal: active.signal, onCommit: () => { committedAfterAbort = true; } }); } catch (e) { refused = e.code === "projectMutationCancelled"; }
    check(refused && !committedAfterAbort && await snapshot() === beforeCancel, "active transaction cancellation did not roll back");
    const afterCommit = new AbortController(); const completed = [];
    const retained = await applyStoredProjectPatch(db, third.app.id, tags({ writes: [{ path: "a", content: "committed" }] }), { ...boundary, signal: afterCommit.signal, onCommit: id => { completed.push(id); afterCommit.abort(); } });
    check(completed[0] === retained.oid && await get("files", `${third.app.id}:a`) === "committed", "completed work lost on later cancellation");
    refused = false; try { await adapters.rulesAfterTurn(third.app.id, "spec", [], { signal: afterCommit.signal, source: retained.source }); } catch (e) { refused = e.code === "generationCancelled"; }
    check(refused, "later cancelled rules still ran");
    // Execute the exact host lifecycle functions over real IndexedDB.
    const { readBoundRules, readBoundRulesSnapshot } = await import("/rules-source.mjs");
    const makeRules = async () => {
      const spec = { name: "demo", title: "old" }, raw = JSON.stringify(spec, null, 2) + "\n";
      const owner = await createStoredProject(db, await prepareNewProject({ name: "rules", files: { "prism/spec.json": raw, "src/lib/prism.ts": "module", "src/pages/Index.tsx": "import '@/lib/prism'" } }, boundary));
      const id = owner.app.id, state = { phase: "proving", specKappa: kappa(new TextEncoder().encode(raw)), logicKappa: "demo", spec, proof: null };
      await put("refs", `prism:${id}`, state);
      return { id, state };
    };
    const environment = (id, overrides = {}) => ({
      coreReady: Promise.resolve(), dbReady: Promise.resolve(db), core, kappa,
      applyStoredProjectPatch, assertGenerationActive, captureProjectSource,
      live: { logicOf: () => ({}), kappaOfValue: value => value.name },
      rulesOf: app => readBoundRules(db, app, kappa),
      rulesSnapshotOf: app => readBoundRulesSnapshot(db, app, kappa),
      saveRules: (app, state, expected) => saveBoundRules(db, app, state, kappa, expected),
      nextPhase: async (phase, event) => event === "reword" ? phase : event === "attest" ? "proved" : event === "check" ? "checked" : "drafting",
      now: () => "now", enc: new TextEncoder(), put: async () => {}, refreshLive: async () => {},
      prism: {
        parseSpecTag: () => ({ spec: { name: "demo", title: "new" } }), parsePromptTag: () => null, facetsOf: async () => null,
        checkSpec: spec => ({ ok: true, module: "module", spec, issues: [] }), card: state => state.phase,
        prove: async () => ({ ok: true, record: { id: "settled" } }),
      }, ...overrides,
    });
    const ahead = await makeRules();
    let releasePrepare, enteredPrepare;
    const prepareGate = new Promise(r => { releasePrepare = r; }), prepareEntered = new Promise(r => { enteredPrepare = r; });
    const aheadHost = hostAdapters(environment(ahead.id, {
      applyStoredProjectPatch: (database, id, patch, options) => applyStoredProjectPatch(database, id, patch, {
        ...options, derive: async bytes => { enteredPrepare(); await prepareGate; return kappa(bytes); },
      }),
    }));
    const staleReword = aheadHost.rulesAfterTurn(ahead.id, "reword").then(() => false, e => e.code === "projectEditConflict");
    await prepareEntered;
    check((await aheadHost.proveRules(ahead.id, "demo")).ok, "pre-commit proof did not complete");
    const completedSnapshot = await snapshot(); releasePrepare();
    check(await staleReword && await snapshot() === completedSnapshot, "stale reword overwrote a completed proof");

    const behind = await makeRules();
    let releaseSave, enteredSave, saves = 0, observedAfterCommit;
    const saveGate = new Promise(r => { releaseSave = r; }), saveEntered = new Promise(r => { enteredSave = r; });
    const behindHost = hostAdapters(environment(behind.id, {
      saveRules: async (id, state, expected) => {
        if (++saves === 1) { enteredSave(); await saveGate; }
        return saveBoundRules(db, id, state, kappa, expected);
      },
    }));
    const finishing = behindHost.proveRules(behind.id, "demo"); await saveEntered;
    await behindHost.rulesAfterTurn(behind.id, "reword", [], { onCommit: () => {
      observedAfterCommit = readBoundRules(db, behind.id, kappa);
    } });
    const visible = await observedAfterCommit;
    check(visible?.phase === "proving" && visible.spec.title === "new", "source commit exposed a missing or stale rules cache");
    releaseSave();
    check((await finishing).ok && saves === 2, "proof did not retry the committed reword predecessor");
    const final = await readBoundRules(db, behind.id, kappa);
    check(final.phase === "proved" && final.spec.title === "new" && final.proof.id === "settled", "post-commit proof was lost");
    db.close(); return { completeBatch: true, directDependencies: "snapshot-bound", noOpSource: "preserved", preparationCancel: "unchanged", activeCancel: "rolled-back", committedWork: "retained", rewordRace: "atomic-before-and-after" };
  });
  assert.deepEqual(report, { completeBatch: true, directDependencies: "snapshot-bound", noOpSource: "preserved", preparationCancel: "unchanged", activeCancel: "rolled-back", committedWork: "retained", rewordRace: "atomic-before-and-after" }); console.log(JSON.stringify(report));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
