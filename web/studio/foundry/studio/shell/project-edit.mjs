import {capturePreviewState} from "./version-preview-codec.mjs";
// FoundrySpecification.projectEditBoundary: prepare, compare, atomically commit.
import { allocateIdentifier, prepareExistingProject } from "./project-creation.mjs";
const failure = code => Object.assign(new Error(code === "projectPreviewActive" ? "Close version preview before changing project source" : code === "projectMutationCancelled" ? "Project mutation cancelled" : code === "projectEditConflict" ? "Project changed during edit" : code === "invalidProjectEdit" ? "Invalid project edit" : code === "projectEditStateInvalid" ? "Project state is invalid" : "Project edit storage failed"), { code });
const validPath = path => typeof path === "string" && path.length > 0 && !/[\\\u0000-\u001f:]/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
const dependencyShape = value => Array.isArray(value) && value.length <= 128 && value.every(item => typeof item === "string" && item.length <= 256);
const range = id => IDBKeyRange.bound(`${id}:`, `${id};`, false, true);
export function captureProjectSource(db, appId) {
  return new Promise((resolve, reject) => {
    let t, app, reference, applicationState; const files = [];
    const refuse = code => { if (t) { try { t.abort(); } catch {} } reject(failure(code)); };
    try {
      t = db.transaction(["apps", "files", "refs"], "readonly");
      t.onabort = () => reject(failure("projectEditStorageFailed")); t.onerror = () => refuse("projectEditStorageFailed");
      t.oncomplete = () => {
        if (!app || typeof app.name !== "string" || typeof reference !== "string" || !/^blake3:[0-9a-f]{64}$/.test(reference) || !dependencyShape(app.extraDeps || [])) reject(failure("projectEditStateInvalid"));
        else resolve({ name: app.name, reference, files, dependencies: structuredClone(app.extraDeps || []), ...(applicationState === undefined ? {} : {applicationState}) });
      };
      const aq = t.objectStore("apps").get(appId), rq = t.objectStore("refs").get(`${appId}:main`);
      aq.onsuccess = () => { app = aq.result; }; rq.onsuccess = () => { reference = rq.result; };
      const sq = t.objectStore('refs').get(`stateful:${appId}`); sq.onsuccess = () => {applicationState = sq.result;};
      const q = t.objectStore("files").openCursor(range(appId));
      q.onsuccess = () => {
        try {
          const cursor = q.result; if (!cursor) return;
          files.push([cursor.key.slice(String(appId).length + 1), cursor.value]);
          if (files.length > 1000) { refuse("projectEditStateInvalid"); return; }
          cursor.continue();
        } catch { refuse("projectEditStorageFailed"); }
      };
    } catch { refuse("projectEditStorageFailed"); }
  });
}
export function commitPreparedProject(db, appId, changedPaths, captured, prepared, { signal, dependencies, message, name, recordVersion = true, rulesUpdate, appUpdate, eventUpdate, interactionUpdate, workUpdate, previewUpdate, chatUpdate } = {}) {
  return new Promise((resolve, reject) => {
    let t, app, reference, currentPreview, previewRead = false, currentWork, workRead = !workUpdate, currentInteraction, interactionRead = !interactionUpdate, currentRules, currentEvents, eventsRead = !eventUpdate, appRead = false, refRead = false, rulesRead = !rulesUpdate, filesRead = false, failed = false;
    const chatChange=chatUpdate?structuredClone(chatUpdate):null;let chatsRead=!chatChange,queuesRead=!chatChange,chatAllocated=!chatChange,createdChatId=null;
    const preview = previewUpdate ? structuredClone(previewUpdate) : null;
    const sourceMutation = prepared.oid !== captured.reference || changedPaths.length > 0 || name !== undefined || dependencies !== undefined;
    const work = workUpdate ? structuredClone(workUpdate) : null;
    const interaction = interactionUpdate ? structuredClone(interactionUpdate) : null;
    const events = eventUpdate ? structuredClone(eventUpdate) : null;
    const rules = rulesUpdate ? structuredClone(rulesUpdate) : null;
    const application = appUpdate ? structuredClone(appUpdate) : null;
    let currentApplication, applicationRead = !application;
    const original = new Map(captured.files); let seen = 0;
    const clean = () => signal?.removeEventListener("abort", cancelled);
    const cancelled = () => refuse("projectMutationCancelled");
    const refuse = code => { failed = true; clean(); if (t) { try { t.abort(); } catch {} } reject(failure(code)); };
    try {
      if (preview) { try { if(preview.state?.profile!=="foundry/version-preview/1"||preview.state.appId!==appId||!Array.isArray(preview.state.receipts)||preview.state.receipts.length>128)throw Error();capturePreviewState(preview.state.state); } catch { refuse("projectEditStateInvalid");return; } }
      if (signal?.aborted) { refuse("projectMutationCancelled"); return; }
      signal?.addEventListener("abort", cancelled, { once: true });
      t = db.transaction(["apps", "files", "objects", "refs", "versions",...(chatChange?["chats"]:[])], "readwrite");
      t.onabort = () => { clean(); reject(failure(signal?.aborted ? "projectMutationCancelled" : "projectEditStorageFailed")); }; t.onerror = () => refuse("projectEditStorageFailed");
      t.oncomplete = () => { clean(); if (failed) reject(failure("projectEditStorageFailed")); else resolve(chatChange?{oid:prepared.oid,createdChatId,affectedChatId:chatChange.mode==="truncate"?chatChange.chatId:null}:prepared.oid); };
      const write = () => {
        if (failed || !appRead || !refRead || !rulesRead || !applicationRead || !eventsRead || !interactionRead || !workRead || !previewRead || !filesRead || !chatsRead || !queuesRead || !chatAllocated) return;
        if (!app || app.name !== captured.name || reference !== captured.reference || seen !== original.size || (!dependencyShape(app.extraDeps || []) || JSON.stringify(app.extraDeps || []) !== JSON.stringify(captured.dependencies))) { refuse("projectEditConflict"); return; }
        if (currentPreview !== undefined) { try { if (currentPreview?.profile !== "foundry/version-preview/1" || currentPreview.appId !== appId) throw Error(); capturePreviewState(currentPreview.state); } catch { refuse("projectEditStateInvalid"); return; } }
        if (preview && JSON.stringify(currentPreview) !== JSON.stringify(preview.expected)) { refuse("projectEditConflict"); return; }
        if (sourceMutation && !preview && currentPreview?.state.active === 1) { refuse("projectPreviewActive"); return; }
        if (work && JSON.stringify(currentWork) !== JSON.stringify(work.expected)) { refuse("projectEditConflict"); return; }
        if (rules && JSON.stringify(currentRules) !== JSON.stringify(rules.expected)) { refuse("projectEditConflict"); return; }
        if (application && JSON.stringify(currentApplication) !== JSON.stringify(application.expected)) { refuse('projectEditConflict'); return; }
        if (interaction && JSON.stringify(currentInteraction) !== JSON.stringify(interaction.expected)) { refuse("projectEditConflict"); return; }
        if (events && JSON.stringify(currentEvents) !== JSON.stringify(events.expected)) { refuse("projectEditConflict"); return; }
        try {
          const q = t.objectStore("versions").get(appId);
          q.onsuccess = () => {
            try {
              if (!Array.isArray(q.result)) { refuse("projectEditStateInvalid"); return; }
              const at = new Date().toISOString();
              if(chatChange){const next=structuredClone(chatChange.next);if(chatChange.mode==="fork"){next.id=createdChatId;next.initialCommitHash=prepared.oid;t.objectStore("chats").add(next,createdChatId);}else t.objectStore("chats").put(next,chatChange.chatId);}
              const files = new Map(prepared.files);
              for (const path of changedPaths) {
                if (files.has(path)) t.objectStore("files").put(files.get(path), `${appId}:${path}`);
                else t.objectStore("files").delete(`${appId}:${path}`);
              }
              for (const [key, bytes] of prepared.objects) t.objectStore("objects").put(bytes, key);
              t.objectStore("refs").put(prepared.oid, `${appId}:main`);
              if (preview) {if(chatChange&&preview.state.receipts.length){preview.state.receipts[preview.state.receipts.length-1].result={createdChatId,affectedChatId:chatChange.mode==="truncate"?chatChange.chatId:null};}t.objectStore("refs").put(preview.state, `preview:${appId}`);}
              if (work) t.objectStore("refs").put(work.state, `work:${appId}`);
              if (interaction) t.objectStore("refs").put(interaction.state, `interaction:${appId}`);
              if (events) t.objectStore("refs").put(events.state, `events:${appId}`);
              if (application) { if (application.state === null) t.objectStore('refs').delete(`stateful:${appId}`); else t.objectStore('refs').put(application.state, `stateful:${appId}`); }
              else if (changedPaths.includes('prism/App.lex.tex') || changedPaths.includes('prism/spec.json')) t.objectStore('refs').delete(`stateful:${appId}`);
              if (rules) { if (rules.state === null) t.objectStore('refs').delete(`prism:${appId}`); else t.objectStore("refs").put(rules.state, `prism:${appId}`); }
              else if (changedPaths.includes("prism/spec.json")) t.objectStore("refs").delete(`prism:${appId}`);
              if (recordVersion) t.objectStore("versions").put([{ oid: prepared.oid, message: message || `Edit ${changedPaths.join(", ")}`, timestamp: Math.floor(Date.parse(at) / 1000), dbTimestamp: null, isFavorite: false, note: null }, ...q.result], appId);
              t.objectStore("apps").put({ ...app, ...(name === undefined ? {} : { name, path: name }), ...(dependencies ? { extraDeps: dependencies } : {}), updatedAt: at }, appId);
            } catch { refuse("projectEditStorageFailed"); }
          };
        } catch { refuse("projectEditStorageFailed"); }
      };
      const previewStatus = t.objectStore("refs").get(`preview:${appId}`);
      previewStatus.onsuccess = () => { currentPreview = previewStatus.result; previewRead = true; write(); };
      if (work) {
        const status = t.objectStore("refs").get(`work:${appId}`);
        status.onsuccess = () => { currentWork = status.result; workRead = true; write(); };
      }
      if (interaction) {
        const status = t.objectStore("refs").get(`interaction:${appId}`);
        status.onsuccess = () => { currentInteraction = status.result; interactionRead = true; write(); };
      }
      if (events) {
        const status = t.objectStore("refs").get(`events:${appId}`);
        status.onsuccess = () => { currentEvents = status.result; eventsRead = true; write(); };
      }
      if (rules) {
        const status = t.objectStore("refs").get(`prism:${appId}`);
        status.onsuccess = () => { currentRules = status.result; rulesRead = true; write(); };
      }
      if (application) {
        const status = t.objectStore('refs').get(`stateful:${appId}`);
        status.onsuccess = () => {currentApplication = status.result; applicationRead = true; write();};
      }
      if(chatChange){
        if(chatChange.profile!=="foundry/message-restore/1"||chatChange.appId!==appId||!["fork","truncate"].includes(chatChange.mode)||!Array.isArray(chatChange.expectedChats)||!Array.isArray(chatChange.expectedQueues)||!chatChange.next||JSON.stringify(chatChange.expectedQueues.map(([id])=>id))!==JSON.stringify(chatChange.expectedChats.map(c=>c.id))||!chatChange.expectedChats.some(c=>c.id===chatChange.chatId&&c.appId===appId)||chatChange.next.appId!==appId||(chatChange.mode==="truncate"&&chatChange.next.id!==chatChange.chatId)){refuse("projectEditStateInvalid");return;}
        const cq=t.objectStore("chats").getAll();cq.onsuccess=()=>{try{const current=cq.result.filter(c=>c.appId===appId).sort((a,b)=>a.id-b.id);if(JSON.stringify(current)!==JSON.stringify(chatChange.expectedChats)){refuse("projectEditConflict");return;}chatsRead=true;write();}catch{refuse("projectEditStateInvalid");}};
        let remaining=chatChange.expectedQueues.length;queuesRead=remaining===0;
        for(const [chatId,expected] of chatChange.expectedQueues){const q=t.objectStore("refs").get(`chat-queue:${chatId}`);q.onsuccess=()=>{if(JSON.stringify(q.result)!==JSON.stringify(expected)||q.result?.state.active||q.result?.state.items?.length){refuse("projectEditConflict");return;}if(--remaining===0){queuesRead=true;write();}};}
        if(chatChange.mode==="fork")allocateIdentifier(t,"chats",id=>{createdChatId=id;chatAllocated=true;write();},()=>refuse("projectEditStorageFailed"));else chatAllocated=true;
      }
      const aq = t.objectStore("apps").get(appId), rq = t.objectStore("refs").get(`${appId}:main`);
      aq.onsuccess = () => { app = aq.result; appRead = true; write(); };
      rq.onsuccess = () => { reference = rq.result; refRead = true; write(); };
      const q = t.objectStore("files").openCursor(range(appId));
      q.onsuccess = () => {
        try {
          const cursor = q.result;
          if (!cursor) { filesRead = true; write(); return; }
          const path = cursor.key.slice(String(appId).length + 1);
          if (!original.has(path) || original.get(path) !== cursor.value) { refuse("projectEditConflict"); return; }
          seen++; cursor.continue();
        } catch { refuse("projectEditStorageFailed"); }
      };
    } catch { refuse("projectEditStorageFailed"); }
  });
}
export async function editStoredProject(db, input, boundary) {
  let edit;
  try { edit = structuredClone(input); } catch { throw failure("invalidProjectEdit"); }
  if (!edit || !Number.isSafeInteger(edit.appId) || edit.appId < 1 || !validPath(edit.filePath) || typeof edit.content !== "string") throw failure("invalidProjectEdit");
  const captured = await captureProjectSource(db, edit.appId);
  const files = Object.fromEntries(captured.files); Object.defineProperty(files, edit.filePath, { value: edit.content, enumerable: true, configurable: true, writable: true });
  const prepared = await prepareExistingProject({ name: captured.name, files }, boundary, captured.reference);
  return commitPreparedProject(db, edit.appId, [edit.filePath], captured, prepared);
}
