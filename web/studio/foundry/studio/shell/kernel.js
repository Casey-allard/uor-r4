// FoundrySpecification.compositionHostBoundary. Classic script shared by page
// and worker. tools/kernel.mjs binds this guest and metadata to KERNEL_REV.
(function (scope) {
  "use strict";
  const encoder = new TextEncoder(), decoder = new TextDecoder();
  const fail = code => Object.assign(new Error(`Kernel refused: ${code}`), { code });
  const toHex = buffer => Array.from(new Uint8Array(buffer), b => b.toString(16).padStart(2, "0")).join("");
  const same = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);
  const byteLength = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype), "byteLength").get;
  const bounded = (value, maximum) => Number.isSafeInteger(value) && value > 0 && value <= maximum;
  const fromBase64 = text => {
    if (typeof text !== "string" || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(text)) throw fail("invalidKernelMetadata");
    try { return Uint8Array.from(atob(text), c => c.charCodeAt(0)); }
    catch { throw fail("invalidKernelMetadata"); }
  };
  function metadata(meta) {
    if (meta?.spec !== "hologram-forge/kernel/1" || typeof meta.digest !== "string" || !/^blake3:[0-9a-f]{64}$/.test(meta.digest || "") || meta.guest?.path !== "kernel/guest.wasm" || typeof meta.guest.sha256 !== "string" || !/^[0-9a-f]{64}$/.test(meta.guest.sha256 || "") || !bounded(meta.guest.bytes, 16 * 1024 * 1024)) throw fail("invalidKernelMetadata");
    const limits = meta.limits;
    if (!limits || ![limits.request, limits.response, limits.guestAllocation].every(n => bounded(n, 1024 * 1024)) || limits.request > limits.guestAllocation || !Array.isArray(meta.vectors) || !bounded(meta.vectors.length, 4096)) throw fail("invalidKernelMetadata");
    let total = 0;
    const vectors = meta.vectors.map(row => {
      if (!Array.isArray(row) || row.length !== 2 || row.some(text => typeof text !== "string" || text.length > 1400000)) throw fail("invalidKernelMetadata");
      const request = fromBase64(row[0]), response = fromBase64(row[1]);
      total += request.length + response.length;
      if (request.length > limits.guestAllocation || response.length > limits.response || total > 64 * 1024 * 1024) throw fail("invalidKernelMetadata");
      return [request, response];
    });
    return { vectors, limits: { ...limits } };
  }
  async function run(module, request, maximum) {
    let result;
    try {
      const instance = await WebAssembly.instantiate(module, {});
      const { memory, holo_alloc, holo_run } = instance.exports;
      const at = holo_alloc(request.length);
      new Uint8Array(memory.buffer, at, request.length).set(request);
      const packed = BigInt.asUintN(64, BigInt(holo_run(at, request.length)));
      const length = Number(packed & 0xffffffffn);
      if (length > maximum) throw fail("kernelResponseTooLarge");
      result = new Uint8Array(memory.buffer, Number(packed >> 32n), length).slice();
    } catch (error) {
      if (error?.code === "kernelResponseTooLarge") throw error;
      throw fail("kernelExecutionFailed");
    }
    return result;
  }
  // A small number of instances in a worker bounds V8's virtual Wasm reservations.
  // Terminating the worker releases its isolate; every run still instantiates the exact module afresh.
  function browserExecutor(module) {
    let worker = null, workerURL = null, uses = 0, pending = 0, queuedBytes = 0, sequence = 0, tail = Promise.resolve();
    const retire = () => { if (worker) worker.terminate(); if (workerURL) URL.revokeObjectURL(workerURL); worker = null; workerURL = null; uses = 0; };
    const workerMain = function () {
      let module;
      self.onmessage = async ({ data }) => {
        try {
          if (data.module) module = data.module;
          const result = await run(module, data.request, data.maximum);
          self.postMessage({ id: data.id, ok: true, result }, [result.buffer]);
        } catch (error) { self.postMessage({ id: data.id, ok: false, code: error?.code === "kernelResponseTooLarge" ? error.code : "kernelExecutionFailed" }); }
      };
    };
    const execute = (request, maximum) => {
      if (pending >= 64 || queuedBytes + request.length > 64 * 1024 * 1024) return Promise.reject(fail("kernelExecutionFailed"));
      pending++; queuedBytes += request.length;
      const next = tail.then(() => new Promise((resolve, reject) => {
        let timer;
        const refuse = code => { clearTimeout(timer); retire(); reject(fail(code)); };
        try {
          if (uses >= 16) retire();
          if (!worker) {
            workerURL = URL.createObjectURL(new Blob([`const fail = ${fail.toString()}; const run = ${run.toString()}; (${workerMain.toString()})();`], { type: "text/javascript" }));
            worker = new Worker(workerURL);
          }
          const active = worker, id = ++sequence, first = uses++ === 0;
          timer = setTimeout(() => refuse("kernelExecutionFailed"), 30000);
          active.onerror = event => { event.preventDefault?.(); refuse("kernelExecutionFailed"); };
          active.onmessage = ({ data }) => {
            if (!data || data.id !== id || typeof data.ok !== "boolean") { refuse("kernelExecutionFailed"); return; }
            clearTimeout(timer);
            if (!data.ok) { refuse(data.code === "kernelResponseTooLarge" ? data.code : "kernelExecutionFailed"); return; }
            if (!(data.result instanceof Uint8Array) || data.result.length > maximum) { refuse(data.result?.length > maximum ? "kernelResponseTooLarge" : "kernelExecutionFailed"); return; }
            resolve(data.result.slice());
          };
          active.postMessage({ id, ...(first ? { module } : {}), request, maximum });
        } catch { refuse("kernelExecutionFailed"); }
      })).finally(() => { pending--; queuedBytes -= request.length; });
      tail = next.catch(() => {});
      return next;
    };
    return { execute, retire };
  }
  const browserKernels = new Map(), browserLoads = new Set();
  scope.navigator?.serviceWorker?.addEventListener("message", async event => {
    const data = event.data, port = event.ports?.[0];
    if (data?.kind !== "foundry:kernel-execute/1" || !port || event.source !== scope.navigator.serviceWorker.controller) {port?.close();return;}
    try {
      let entry = browserKernels.get(data.key);
      if (!entry && browserLoads.size) {
        let timer;
        try {
          await Promise.race([Promise.allSettled([...browserLoads]), new Promise((_, reject) => {
            timer = setTimeout(() => reject(fail("kernelExecutionFailed")), 30000);
          })]);
        } finally { clearTimeout(timer); }
        entry = browserKernels.get(data.key);
      }
      if (!entry) throw fail("kernelExecutionFailed");
      const result = await entry(data.request);
      port.postMessage({ ok: true, result }, [result.buffer]);
    } catch (error) { port.postMessage({ ok: false, code: error?.code === "kernelResponseTooLarge" ? error.code : "kernelExecutionFailed" }); }
    finally { port.close(); }
  });
  // Service workers cannot construct dedicated workers. Controlled document owners execute
  // only the identical verified guest; this transport grants no effects or semantic authority.
  function serviceExecutor(key) {
    let pending = 0, queuedBytes = 0;
    return { retire() {}, async execute(request, maximum) {
      if (pending >= 64 || queuedBytes + request.length > 64 * 1024 * 1024) throw fail("kernelExecutionFailed");
      pending++; queuedBytes += request.length; const ports = []; let timer;
      try {
        let clients = (await scope.clients.matchAll({type:"window",includeUncontrolled:false})).slice(0,8);
        if (!clients.length) throw fail("kernelExecutionFailed");
        const deadline = new Promise((_, reject) => {timer=setTimeout(()=>reject(fail("kernelExecutionFailed")),30000);});
        // FoundrySpecification.kernelOwnerHandoffBoundary: a navigation can destroy
        // an owner mid-replay. Keep the same request and deadline across replacements.
        for (let batch=0;batch<8;batch++) {
          let observer, readyListener, closed=false; const selected=new Set(clients.map(client=>client.id)), batchPorts=[];
          try {
            const answer = Promise.any(clients.map(client=>new Promise((resolve,reject)=>{
              const pair = new MessageChannel(); batchPorts.push(pair.port1,pair.port2); ports.push(pair.port1,pair.port2);
              pair.port1.onmessage = ({data}) => {
                selected.delete(client.id);
                if (!data?.ok || !(data.result instanceof Uint8Array) || data.result.length>maximum) {reject(fail(data?.code === "kernelResponseTooLarge" || data?.result?.length>maximum ? "kernelResponseTooLarge" : "kernelExecutionFailed"));return;}
                resolve({result:data.result.slice()});
              };
              try {client.postMessage({kind:"foundry:kernel-execute/1",key,request},[pair.port2]);} catch {reject(fail("kernelExecutionFailed"));}
            })));
            const handoff = new Promise((resolve,reject)=>{
              readyListener=async event=>{
                if(event.data?.kind!=="foundry:kernel-owner-ready/1"||event.data.key!==key||!event.source?.id)return;
                try {
                  const current=(await scope.clients.matchAll({type:"window",includeUncontrolled:false})).slice(0,8);
                  if(!closed&&current.some(client=>client.id===event.source.id))resolve({clients:current});
                } catch {if(!closed)reject(fail("kernelExecutionFailed"));}
              };
              scope.addEventListener?.("message",readyListener);
              const observe=async()=>{
                try {
                  const current=(await scope.clients.matchAll({type:"window",includeUncontrolled:false})).slice(0,8);
                  if(closed)return;
                  if(current.length&&!current.some(client=>selected.has(client.id))) {resolve({clients:current});return;}
                  observer=setTimeout(observe,50);
                } catch {if(!closed)reject(fail("kernelExecutionFailed"));}
              };
              observer=setTimeout(observe,50);
            });
            const next=await Promise.race([answer,handoff,deadline]);
            if(next.result)return next.result;
            clients=next.clients;
          } catch (error) {throw fail(error?.errors?.some(e=>e?.code==="kernelResponseTooLarge") ? "kernelResponseTooLarge" : "kernelExecutionFailed");}
          finally {closed=true;clearTimeout(observer);scope.removeEventListener?.("message",readyListener);for(const port of batchPorts)port.close();}
        }
        throw fail("kernelExecutionFailed");
      } finally {clearTimeout(timer);for(const port of ports)port.close();pending--;queuedBytes-=request.length;}
    }};
  }
  // FoundrySpecification.kernelOwnerReadinessBoundary: readiness is not verification.
  function load(get) {
    const documentOwner = typeof Worker === "function" && scope.navigator?.serviceWorker;
    if (documentOwner && browserLoads.size >= 8) return Promise.reject(fail("kernelExecutionFailed"));
    const pending = loadVerified(get);
    if (documentOwner) {
      browserLoads.add(pending);
      void pending.then(() => browserLoads.delete(pending), () => browserLoads.delete(pending));
    }
    return pending;
  }
  async function loadVerified(get) {
    let meta, bytes;
    try {
      const response = await get("kernel/kernel.json");
      if (!response.ok) throw Error();
      meta = await response.json();
    } catch { throw fail("kernelLoadFailed"); }
    const { vectors, limits } = metadata(meta);
    try {
      const response = await get(meta.guest.path);
      if (!response.ok) throw Error();
      bytes = await response.arrayBuffer();
    } catch { throw fail("kernelLoadFailed"); }
    let digest;
    try { digest = toHex(await crypto.subtle.digest("SHA-256", bytes)); }
    catch { throw fail("kernelLoadFailed"); }
    if (bytes.byteLength !== meta.guest.bytes || digest !== meta.guest.sha256) throw fail("kernelIntegrityMismatch");
    let module;
    try { module = await WebAssembly.compile(bytes); }
    catch { throw fail("kernelLoadFailed"); }
    const names = WebAssembly.Module.exports(module).map(e => e.name).sort().join(",");
    if (WebAssembly.Module.imports(module).length || names !== "holo_alloc,holo_run,memory") throw fail("kernelExportsMismatch");
    // Headless Node diagnostics have no browser location. Browsers must use a worker.
    const key = meta.digest + ":" + meta.guest.sha256;
    const service = typeof Worker !== "function" && scope.clients?.matchAll && typeof MessageChannel === "function";
    if (typeof location !== "undefined" && typeof Worker !== "function" && !service) throw fail("kernelExecutionFailed");
    const executor = typeof Worker === "function" ? browserExecutor(module) : service ? serviceExecutor(key) : { execute: (request, maximum) => run(module, request, maximum), retire() {} };
    try { for (const [request, response] of vectors) {
      try { if (!same(await executor.execute(request, limits.response), response)) throw Error(); }
      catch { throw fail("kernelAcceptanceFailed"); }
    }
    } finally { executor.retire(); }
    const executeBounded = async (request, bound) => {
      if (!(request instanceof Uint8Array)) throw fail("invalidKernelRequest");
      let length, copy;
      try { length = byteLength.call(request); }
      catch { throw fail("invalidKernelRequest"); }
      if (length > bound) throw fail("kernelRequestTooLarge");
      try { copy = new Uint8Array(length); Uint8Array.prototype.set.call(copy, request); }
      catch { throw fail("invalidKernelRequest"); }
      return executor.execute(copy, limits.response);
    };
    const dispatch = request => executeBounded(request,limits.request);
    if (typeof Worker === "function" && scope.navigator?.serviceWorker) {
      if (browserKernels.size>=8&&!browserKernels.has(key)) browserKernels.delete(browserKernels.keys().next().value);
      browserKernels.set(key,request=>executeBounded(request,limits.guestAllocation));
      // A replacement client can be discoverable before its listener starts.
      // Readiness only wakes transport; the receiving document still checks identity.
      try {scope.navigator.serviceWorker.controller?.postMessage?.({kind:"foundry:kernel-owner-ready/1",key});} catch {}
    }
    return Object.freeze({
      digest: meta.digest, vectors: vectors.length, dispatch,
      ask: async line => {
        if (typeof line !== "string") throw fail("invalidKernelRequest");
        return decoder.decode(await dispatch(encoder.encode(line)));
      },
    });
  }
  scope.ForgeKernel = Object.freeze({ load });
})(self);
