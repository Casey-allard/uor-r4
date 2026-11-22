// Real IndexedDB compare-and-commit tests; preimage oracle scopes this to adapters.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
const helloStart = host.indexOf('window.addEventListener("message", (e) => {\n  const m = e.data;'), helloEnd = host.indexOf('new MutationObserver', helloStart);
const identityStart = host.indexOf('const isLiveFrame ='), identityEnd = host.indexOf('\n', identityStart);
assert.ok(helloStart >= 0 && helloEnd > helloStart && identityStart >= 0);
const helloModule = `export function installHello(env) { const {liveFrameUrl,liveWindows,appApps,liveLatest,appFramesReady,refreshPreview,sendLive,refuse} = env; ${host.slice(identityStart,identityEnd)}\n${host.slice(helloStart,helloEnd)} }`;
const sources = new Map([
  ["/live-hello.mjs", helloModule],
  ["/browser-chat-modes.mjs", fs.readFileSync(new URL("../shell/browser-chat-modes.mjs", import.meta.url))],
  ["/chat-mode-catalog.mjs", fs.readFileSync(new URL("../shell/chat-mode-catalog.mjs", import.meta.url))],
  ["/chat-contracts.mjs", fs.readFileSync(new URL("../shell/chat-contracts.mjs", import.meta.url))],

  ["/theme-library.mjs", fs.readFileSync(new URL("../shell/theme-library.mjs", import.meta.url))],
  ["/version-preview-codec.mjs", fs.readFileSync(new URL("../shell/version-preview-codec.mjs", import.meta.url))],
  ["/project-edit.mjs", fs.readFileSync(new URL("../shell/project-edit.mjs", import.meta.url))],
  ["/project-creation.mjs", fs.readFileSync(new URL("../shell/project-creation.mjs", import.meta.url))],
  ["/storage.mjs", fs.readFileSync(new URL("../shell/storage-request.mjs", import.meta.url))],
  ["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))],
]);
const server = http.createServer((request, response) => {
  if(request.url === '/hello-frame.html') {response.setHeader('content-type','text/html');response.end('<!doctype html><script>parent.postMessage({type:"live/hello",app:decodeURIComponent(location.hash.slice(1))},"*")</script>');return;}
  const source = sources.get(request.url); response.setHeader("content-type", source ? "text/javascript" : "text/html"); response.end(source || "<!doctype html><title>Atomic edit acceptance</title>"); });
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(), pageErrors = [];page.on("pageerror",error=>pageErrors.push(error.message)); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { editStoredProject, captureProjectSource, commitPreparedProject } = await import("/project-edit.mjs");
    const { prepareNewProject, prepareExistingProject, createStoredProject } = await import("/project-creation.mjs");
    const { storageRequest } = await import("/storage.mjs"); const { kappa } = await import("/blake3.mjs");
    const check = (value, message) => { if (!value) throw Error(message); };
    const stores = ["apps", "chats", "files", "objects", "refs", "versions"];
    const db = await new Promise((resolve, reject) => { const q = indexedDB.open("atomic-edit", 1); q.onupgradeneeded = () => stores.forEach(s => q.result.createObjectStore(s)); q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error); });
    const get = (store, key) => storageRequest(db, store, "readonly", s => s.get(key));
    const put = (store, key, value) => storageRequest(db, store, "readwrite", s => s.put(value, key));
    const snapshot = async () => JSON.stringify(await Promise.all(stores.map(store => storageRequest(db, store, "readonly", s => s.getAll()))));
    const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
    const boundary = { core, derive: kappa };
    const seed = async () => createStoredProject(db, await prepareNewProject({ name: "app", files: { "a.txt": "original", "b.txt": "other", "prism/spec.json": "old rules" } }, boundary));
    const first = await seed(); const appId = first.app.id;
    await put("refs", `prism:${appId}`, { phase: "proved" });
    const oid = await editStoredProject(db, { appId, filePath: "prism/spec.json", content: "new rules" }, boundary);
    check(await get("refs", `prism:${appId}`) === undefined, "stale rule state survived");
    check(await get("files", `${appId}:prism/spec.json`) === "new rules", "edit missing");
    const body = await get("objects", oid); check(kappa(body) === oid && JSON.parse(new TextDecoder().decode(body)).parent === first.oid, "snapshot parent or identity wrong");
    const versions = await get("versions", appId); check(versions.length === 2 && versions[0].oid === oid && versions[1].oid === first.oid, "history not preserved");
    for (const kind of ["changed", "added", "removed", "reference", "name"]) {
      const p = await seed(); let release, started;
      const gate = new Promise(resolve => { release = resolve; }), began = new Promise(resolve => { started = resolve; });
      const editing = editStoredProject(db, { appId: p.app.id, filePath: "a.txt", content: "candidate" }, { core, derive: async bytes => { started(); await gate; return kappa(bytes); } });
      await began;
      if (kind === "changed") await put("files", `${p.app.id}:b.txt`, "intervening");
      if (kind === "added") await put("files", `${p.app.id}:added.txt`, "intervening");
      if (kind === "removed") await storageRequest(db, "files", "readwrite", s => s.delete(`${p.app.id}:b.txt`));
      if (kind === "reference") await put("refs", `${p.app.id}:main`, "blake3:" + "f".repeat(64));
      if (kind === "name") await put("apps", p.app.id, { ...(await get("apps", p.app.id)), name: "renamed" });
      const before = await snapshot(); release();
      let refused = false; try { await editing; } catch (e) { refused = e.code === "projectEditConflict"; }
      check(refused && await snapshot() === before, `${kind} source race accepted or modified state`);
    }
    const concurrent = await seed(); let release, starts = 0, started;
    const gate = new Promise(resolve => { release = resolve; }), began = new Promise(resolve => { started = resolve; });
    const racingBoundary = { core, derive: async bytes => { if (++starts === 2) started(); await gate; return kappa(bytes); } };
    const edits = ["left", "right"].map(content => editStoredProject(db, { appId: concurrent.app.id, filePath: "a.txt", content }, racingBoundary));
    await began; release(); const outcomes = await Promise.allSettled(edits);
    check(outcomes.filter(x => x.status === "fulfilled").length === 1 && outcomes.filter(x => x.status === "rejected" && x.reason.code === "projectEditConflict").length === 1, "concurrent edits did not reject stale source");
    check((await get("versions", concurrent.app.id)).length === 2, "concurrent history corrupted");
    const before = await snapshot();
    const failingDb = { transaction(names, mode) {
      const t = db.transaction(names, mode); if (mode === "readonly") return t;
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => {
          const store = target.objectStore(name); if (name !== "refs") return store;
          return { get: store.get.bind(store), put() { throw Error("injected reference write failure"); } };
        };
        const value = target[key]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, key, value) { target[key] = value; return true; } });
    } };
    let refused = false; try { await editStoredProject(failingDb, { appId, filePath: "a.txt", content: "aborted" }, boundary); } catch (e) { refused = e.code === "projectEditStorageFailed"; }
    check(refused && await snapshot() === before, "failed edit left partial writes");
    const recovery = await editStoredProject(db, { appId, filePath: "a.txt", content: "recovered" }, boundary);
    check(await get("refs", `${appId}:main`) === recovery && await get("files", `${appId}:a.txt`) === "recovered", "recovery failed");
    // Protected lifecycle is captured with source and changed only by its exact predecessor.
    // These are transaction fixtures, not evidence of checked/proved application semantics.
    const stateful = await createStoredProject(db, await prepareNewProject({name:'stateful',files:{'prism/App.lex.tex':'structure{ name: State }','prism/app.json':'{"phase":"proved"}'}},boundary));
    const statefulId=stateful.app.id, predecessor={phase:'checked',sourceKappa:'blake3:'+'a'.repeat(64)},successor={...predecessor,phase:'proving',approvalId:'one'};
    await put('refs',`stateful:${statefulId}`,predecessor);
    const captured=await captureProjectSource(db,statefulId);
    check(JSON.stringify(captured.applicationState)===JSON.stringify(predecessor),'protected state not captured with source');
    const prepared=await prepareExistingProject({name:captured.name,files:{...Object.fromEntries(captured.files),'prism/App.lex.tex':'structure{ name: State }\n'}},boundary,captured.reference);
    await commitPreparedProject(db,statefulId,['prism/App.lex.tex'],captured,prepared,{appUpdate:{expected:predecessor,state:successor}});
    check(JSON.stringify(await get('refs',`stateful:${statefulId}`))===JSON.stringify(successor),'source and lifecycle did not commit together');
    const stale=await captureProjectSource(db,statefulId);
    await put('refs',`stateful:${statefulId}`,{...successor,approvalId:'replacement'});
    const beforeLifecycle=await snapshot();let lifecycleRefused=false;
    try{await commitPreparedProject(db,statefulId,[],stale,{oid:stale.reference,files:stale.files,objects:[]},{recordVersion:false,appUpdate:{expected:successor,state:predecessor}});}catch(e){lifecycleRefused=e.code==='projectEditConflict';}
    check(lifecycleRefused&&await snapshot()===beforeLifecycle,'stale lifecycle overwrote a replacement approval');
    await editStoredProject(db,{appId:statefulId,filePath:'prism/App.lex.tex',content:'structure{ name: State }\n\n'},boundary);
    check(await get('refs',`stateful:${statefulId}`)===undefined,'ordinary source mutation retained protected standing');
    // The production frame listener receives warm-up's empty hash before any project exists.
    // Keep strict project reads; reject non-project identities at the frame boundary instead.
    const {installHello}=await import('/live-hello.mjs');
    const liveFrameUrl=new URL('/hello-frame.html',location.href).href, liveWindows=new Map(), reads=[], refreshed=[], refusals=[];
    installHello({liveFrameUrl,liveWindows,appApps:new Set(),liveLatest:new Map(),appFramesReady:()=>{throw Error('unexpected stateful path');},
      refreshPreview:async id=>{reads.push(id);refreshed.push(await captureProjectSource(db,id));},
      sendLive:()=>{throw Error('unexpected cached projection');},refuse:(...args)=>refusals.push(args)});
    const pause=()=>new Promise(resolve=>setTimeout(resolve,10));
    const waitFor=async predicate=>{for(let i=0;i<100;i++){if(predicate())return;await pause();}throw Error('live hello did not settle');};
    const hello=async id=>{
      const frame=document.createElement('iframe');frame.src=liveFrameUrl+'#'+encodeURIComponent(id);
      const said=new Promise(resolve=>{const on=e=>{if(e.source===frame.contentWindow&&e.data?.type==='live/hello'){removeEventListener('message',on);resolve();}};addEventListener('message',on);});
      document.body.append(frame);await said;await pause();return frame;
    };
    for(const id of ['', '0', 'not-an-app', '9007199254740992']){const frame=await hello(id);frame.remove();}
    check(reads.length===0&&liveWindows.size===0,'warmup or invalid frame read project state');
    let missingRefused=false;try{await captureProjectSource(db,'');}catch(error){missingRefused=error.code==='projectEditStateInvalid';}
    check(missingRefused,'missing source was admitted to hide preview failure');
    const frame=await hello(String(appId));await waitFor(()=>refreshed.length===1);
    check(reads[0]===appId&&refreshed[0].reference===recovery,'valid frame failed its captured source read');frame.remove();
    const absent=await hello('9007199254740991');await waitFor(()=>refusals.length===1);
    check(refusals[0][0]===9007199254740991&&/preview.*could not be refreshed/i.test(refusals[0][2]),'missing app refresh did not report refusal');absent.remove();
    db.close();
    return { exactParent: true, rulesInvalidated: true, raceKindsRefused: 5, concurrentWinner: 1, failedWrite: "rolled-back", recovery: "committed", statefulLifecycle:'source-bound-cas', liveWarmup:'no-project-read', invalidRefresh:'private-refusal' };
  });
  assert.deepEqual(report, { exactParent: true, rulesInvalidated: true, raceKindsRefused: 5, concurrentWinner: 1, failedWrite: "rolled-back", recovery: "committed", statefulLifecycle:'source-bound-cas', liveWarmup:'no-project-read', invalidRefresh:'private-refusal' }); assert.deepEqual(pageErrors,[]);console.log(JSON.stringify(report));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
