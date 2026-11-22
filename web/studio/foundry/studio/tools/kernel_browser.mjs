// Execute the shipped classic loader in both of its browser contexts.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const files = new Map([
  ["/kernel.js", "../shell/kernel.js"],
  ["/preview-vectors.json", "../../verification/version-preview-vectors.json"],
  ["/queue-vectors.json", "../../verification/chat-queue-vectors.json"],
  ["/routing-vectors.json", "../../verification/routing-vectors.json"],
  ["/kernel/kernel.json", "../shell/kernel/kernel.json"],
  ["/kernel/guest.wasm", "../shell/kernel/guest.wasm"],
  ["/composition-codec.mjs", "../shell/composition-codec.mjs"],
  ["/vectors.json", "../../verification/composition-wire-vectors.json"],
  ["/work-vectors.json", "../../verification/work-wire-vectors.json"],
  ["/work-codec.mjs", "../shell/work-codec.mjs"],
  ["/event-vectors.json", "../../verification/event-wire-vectors.json"],
  ["/event-codec.mjs", "../shell/event-codec.mjs"],
  ["/interaction-codec.mjs", "../shell/interaction-codec.mjs"],
  ["/interaction-vectors.json", "../../verification/interaction-vectors.json"],
].map(([url, file]) => [url, fs.readFileSync(new URL(file, import.meta.url))]));
const exercise = async () => {
  const { admitWithGuest } = await import("/composition-codec.mjs");
  const vectors = await (await fetch("/vectors.json")).json();
  const guest = await self.ForgeKernel.load(path => fetch("/" + path));
  const decoder = new TextDecoder();
  for (const vector of vectors) {
    const request = Uint8Array.from(vector.request.match(/../g) ?? [], pair => parseInt(pair, 16));
    if (request.length > 32776) {
      try { await guest.dispatch(request); throw Error("oversized public request accepted"); }
      catch (error) { if (error.code !== "kernelRequestTooLarge") throw error; }
    } else if (decoder.decode(await guest.dispatch(request)) !== vector.response) throw Error("wire: " + vector.name);
    if (vector.policy) {
      const result = await admitWithGuest(guest, vector.policy, vector.features);
      if (result.ok !== (vector.response === "accepted") || (!result.ok && result.code !== vector.response)) throw Error("composition: " + vector.name);
    }
  }
  const { executeWorkWithGuest } = await import("/work-codec.mjs");
  const work = (await (await fetch("/work-vectors.json")).json()).vectors;
  for (const vector of work) if (await guest.ask(vector.request) !== vector.response) throw Error("work: " + vector.name);
  const initialized = await executeWorkWithGuest(guest, { kind: "initialize", owner: 7, capacity: 20, remaining: 2 });
  if (!initialized.ok || initialized.state.owner !== 7 || initialized.state.remaining !== 2) throw Error("work adapter");
  const { admitEventWithGuest } = await import("/event-codec.mjs");
  const events = (await (await fetch("/event-vectors.json")).json()).vectors;
  for (const vector of events) {
    if (await guest.ask(vector.request) !== vector.response) throw Error("event: " + vector.name);
    if (vector.input) {
      const reply = await admitEventWithGuest(guest, vector.input);
      if (vector.response.startsWith("ok ")) {
        const next = vector.name === "lastSequence" ? 0xffffffff : 3;
        if (!reply.ok || reply.nextSequence !== next) throw Error("event adapter: " + vector.name);
      } else if (reply.ok || reply.code !== vector.response) throw Error("event adapter: " + vector.name);
    }
  }
  const { executeInteraction } = await import("/interaction-codec.mjs");
  const interactions = (await (await fetch("/interaction-vectors.json")).json()).vectors;
  for (const vector of interactions) {
    const result = decoder.decode(await guest.dispatch(new TextEncoder().encode(vector.request)));
    if (result !== vector.response) throw Error("interaction: " + vector.name + ": " + result);
    if (vector.input) {
      const encoded = await executeInteraction(guest,vector.input);
      if (encoded.ok !== vector.response.startsWith("ok ") || (!encoded.ok && encoded.code !== vector.response)) throw Error("interaction adapter: " + vector.name);
    }
  }
  const routing = (await (await fetch("/routing-vectors.json")).json()).vectors;
  for (const vector of routing) if (decoder.decode(await guest.dispatch(new TextEncoder().encode(vector.request))) !== vector.response) throw Error("routing: " + vector.name);
  const queue = (await (await fetch("/queue-vectors.json")).json()).vectors;
  for (const vector of queue) if (decoder.decode(await guest.dispatch(new TextEncoder().encode(vector.request))) !== vector.response) throw Error("queue: " + vector.name);
  const previews=(await (await fetch("/preview-vectors.json")).json()).vectors;
  for(const vector of previews)if(decoder.decode(await guest.dispatch(new TextEncoder().encode(vector.request)))!==vector.response)throw Error("preview: "+vector.name);
  if (await guest.ask("drafting check") !== "checked") throw Error("legacy phase");
  for (const phase of ["drafting", "checked", "proving", "proved", "published"]) if (await guest.ask(phase + " reword") !== phase) throw Error("reword phase");
  return { previews: previews.length, queue: queue.length, routing: routing.length, vectors: vectors.length, work: work.length, events: events.length, interactions: interactions.length, embedded: guest.vectors, digest: guest.digest };
};
const workerSource = `importScripts('/kernel.js'); (${exercise.toString()})().then(result => postMessage({result}), error => postMessage({error: String(error)}));`;
const server = http.createServer((request, response) => {
  if (request.url === "/") { response.setHeader("content-type", "text/html"); response.end('<!doctype html><title>Kernel browser acceptance</title><script src="/kernel.js"></script>'); return; }
  if (request.url === "/worker.js") { response.setHeader("content-type", "text/javascript"); response.end(workerSource); return; }
  const bytes = files.get(request.url);
  response.statusCode = bytes ? 200 : 404;
  response.setHeader("content-type", request.url.endsWith(".wasm") ? "application/wasm" : request.url.endsWith(".json") ? "application/json" : "text/javascript");
  response.end(bytes ?? "");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(path => fs.existsSync(path));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const documentResult = await page.evaluate(exercise);
  const workerResult = await page.evaluate(() => new Promise((resolve, reject) => {
    const worker = new Worker("/worker.js");
    const timeout = setTimeout(() => { worker.terminate(); reject(Error("worker timeout")); }, 60000);
    worker.onmessage = ({ data }) => { clearTimeout(timeout); worker.terminate(); if (data.error) reject(Error(data.error)); else resolve(data.result); };
    worker.onerror = event => { clearTimeout(timeout); worker.terminate(); reject(Error(event.message)); };
  }));
  assert.deepEqual(workerResult, documentResult);
  assert.equal(documentResult.vectors, 26);
  assert.equal(documentResult.work, 30);
  assert.equal(documentResult.events, 30);
  assert.equal(documentResult.interactions, 21);
  assert.equal(documentResult.routing, 32);
  assert.equal(documentResult.queue, 38);
  assert.equal(documentResult.previews, 30);
  assert.equal(documentResult.embedded, JSON.parse(fs.readFileSync(new URL('../shell/kernel/kernel.json',import.meta.url))).vectors.length);
  console.log(JSON.stringify({ page: documentResult, worker: workerResult }));
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
