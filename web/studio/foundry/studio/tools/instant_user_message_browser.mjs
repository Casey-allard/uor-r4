// A sent prompt in the actual shell: on screen and out of the composer before the host has stored anything,
// replaced by its stored message without ever showing twice, withdrawn when the send is refused, and left
// to the queue when a turn is already running.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { DEVICE_MODEL, deviceAsked, deviceHold, deviceRelease, deviceSeam } from "./device_fixture.mjs";
const root = fileURLToPath(new URL("../shell/", import.meta.url));
for (const name of ["core.wasm", "assets/index.js", "manifest.json", "sw.js", "provenance.json"]) assert.ok(fs.existsSync(path.join(root, name)), `Build and project the actual shell first: ${name}`);
const types = { ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm", ".svg": "image/svg+xml", ".woff2": "font/woff2", ".html": "text/html" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost"), relative = decodeURIComponent(url.pathname).replace(/^\/foundry\//, "");
  let file = path.resolve(root, relative);
  if ((file !== path.resolve(root) && !file.startsWith(root)) || relative.split("/").includes("..")) { res.writeHead(404); res.end(); return; }
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
  if (!fs.existsSync(file) && !path.extname(file) && !relative.startsWith("p/")) file = path.join(root, "index.html");
  if (!fs.existsSync(file)) { res.writeHead(404); res.end(); return; }
  res.setHeader("content-type", types[path.extname(file)] || "application/octet-stream"); res.setHeader("cache-control", "no-store"); res.end(fs.readFileSync(file));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}/foundry/`, origin = new URL(base).origin;
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find((p) => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "block", viewport: { width: 1280, height: 844 } }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  // Foundry answers with its own model only (Prism R4): any request off the origin is refused.
  await context.route("**/*", (route) => (new URL(route.request().url()).origin === origin ? route.continue() : route.abort()));
  // The preview's permission dialog is the preview's own (tools/preview_auto_start_browser.mjs): it is answered when it appears.
  await page.addInitScript(() => { setInterval(() => { const dialog = document.querySelector('dialog[aria-label="Review preview permissions"]'); const deny = dialog && [...dialog.querySelectorAll("button")].find((b) => b.textContent === "Don't allow"); if (deny) deny.click(); }, 50); });
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const call = (channel, input) => page.evaluate(({ channel, input }) => window.electron.ipcRenderer.invoke(channel, input), { channel, input });
  await call("set-user-settings", { selectedModel: DEVICE_MODEL });
  const made = await call("create-app", { name: "instant-message", initialChatMode: "ask" }), chatId = made.chatId, appId = made.app.id;
  await page.goto(`${base}chat?id=${chatId}&appId=${appId}`);
  const input = page.getByTestId("chat-input-container").getByRole("textbox"), list = page.getByTestId("messages-list"); await input.waitFor();
  // Each page load starts the preview once and asks for its permission; the answer above declines it. The test goes on only once that has settled, so the dialog never opens under a key press.
  const previewSettled = () => page.waitForFunction(async (appId) => (await window.electron.ipcRenderer.invoke("distributed-machine:subscribe", { protocolVersion: 1, machineId: "app_run", encodedKey: { appId } })).encodedState.phase === "errored" && !document.querySelector('dialog[aria-label="Review preview permissions"]'), appId, { timeout: 30000 });
  await previewSettled();
  // The host admits a turn under the chat queue's own lock: holding that lock from here holds the admission,
  // and refusing it once refuses the send. What is on screen meanwhile is the renderer's alone.
  const hold = () => page.evaluate((name) => { window.__held = new Promise((held) => { void navigator.locks.request(name, () => new Promise((release) => { window.__release = release; held(); })); }); return window.__held; }, `foundry:chat-queue:${chatId}`);
  const release = () => page.evaluate(() => window.__release());
  // Every request for the queue's lock is refused while this is on, whoever asks: a background refresh must not use up the refusal meant for the send.
  const refusing = (on) => page.evaluate((on) => { const manager = LockManager.prototype; manager.__real ??= manager.request; manager.request = on ? function (name, ...rest) { return String(name).startsWith("foundry:chat-queue:") ? Promise.reject(new Error("refused by the test")) : manager.__real.call(this, name, ...rest); } : manager.__real; }, on);
  const stored = async () => (await call("get-chat", chatId)).messages.map((m) => m.content);
  const view = (prompt) => page.evaluate((prompt) => { const text = document.querySelector('[data-testid="messages-list"]')?.textContent || ""; return { copies: text.split(prompt).length - 1, pending: document.querySelectorAll('[data-slot="pending-user-message"]').length, composer: document.querySelector('[data-testid="chat-input-container"] [role=textbox]').textContent }; }, prompt);
  // Every later check also holds over time: the prompt is never on screen twice.
  const watch = (prompt) => page.evaluate((prompt) => { window.__most = 0; window.__watch?.disconnect(); window.__watch = new MutationObserver(() => { const text = document.querySelector('[data-testid="messages-list"]')?.textContent || ""; window.__most = Math.max(window.__most, text.split(prompt).length - 1); }); window.__watch.observe(document.body, { subtree: true, childList: true, characterData: true }); }, prompt);
  const most = () => page.evaluate(() => window.__most);
  const frames = () => page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  // A turn is over for the host (nothing active or queued) and for the renderer (the send button is back): a prompt sent before that is queued, by design.
  const atRest = () => page.waitForFunction(async (chatId) => { const journal = await window.__host.get("refs", `chat-queue:${chatId}`); return !!journal && !journal.state.active && journal.state.items.length === 0 && !!document.querySelector('[data-testid="chat-input-container"] button[aria-label="Send message"]'); }, chatId);
  // The words are typed and the composer has them (its send button is enabled) before Enter: a key pressed sooner finds an empty composer.
  const write = async (text) => { await input.fill(text); await page.waitForFunction(() => { const send = document.querySelector('[data-testid="chat-input-container"] button[aria-label="Send message"]'); return !send || !send.disabled; }); await frames(); };

  // 1. The first message of an empty conversation, with the host's admission held: shown, composer empty, nothing stored.
  const first = "Is the first message shown at once?";
  await hold(); await watch(first);
  await write(first); await input.press("Enter"); await frames();
  assert.deepEqual(await view(first), { copies: 1, pending: 1, composer: "" });
  await page.waitForTimeout(400);
  assert.deepEqual(await stored(), [], "the host has stored nothing yet");
  assert.equal(await page.locator('[data-slot="pending-user-message"][inert]').count(), 1, "not a stored message: it cannot be acted on");
  assert.deepEqual(await view(first), { copies: 1, pending: 1, composer: "" }, "still standing in while the host is held");
  await release();
  await list.getByText("Open questions:").first().waitFor();
  assert.deepEqual(await view(first), { copies: 1, pending: 0, composer: "" }); assert.equal(await most(), 1, "never beside its stored message");
  assert.deepEqual((await stored()).slice(0, 1), [first]);
  await atRest();

  // 2. A send with the reader held: shown at once; while that turn runs, another prompt is queued, not shown as a message.
  const second = "Is a second message shown at once?";
  await deviceHold(page);
  await watch(second); await write(second); await input.press("Enter"); await frames();
  // Nothing holds the host here, so the stored message may already have taken the stand-in's place: either way it is on screen once.
  { const now = await view(second); assert.deepEqual({ copies: now.copies, composer: now.composer }, { copies: 1, composer: "" }); }
  await page.waitForFunction(() => document.querySelectorAll('[data-slot="pending-user-message"]').length === 0);
  const queued = "Is a queued prompt left to the queue?";
  await write(queued); await input.press("Enter"); await frames();
  assert.equal((await view(queued)).pending, 0); assert.equal((await view(queued)).copies, 0, "a queued prompt is not a message yet");
  await page.getByText(queued).first().waitFor();
  await deviceRelease(page);
  await list.getByText("Open questions:").nth(1).waitFor();
  assert.equal(await most(), 1); assert.equal((await view(second)).copies, 1);
  // The queued prompt runs as its own turn; the conversation is then at rest.
  await list.getByText("Open questions:").nth(2).waitFor();
  await atRest();
  const sends = (await deviceAsked(page)).length; assert.equal(sends, 3);

  // 3. A refused send, last because it breaks the queue's lock on purpose: the message is withdrawn and the composer has its words back.
  const refused = "Is a refused message taken back?";
  await refusing(true); await watch(refused);
  await write(refused); await input.press("Enter");
  await page.waitForFunction((refused) => document.querySelectorAll('[data-slot="pending-user-message"]').length === 0 && document.querySelector('[data-testid="chat-input-container"] [role=textbox]').textContent === refused, refused);
  await refusing(false);
  assert.equal((await view(refused)).copies, 0); assert.equal(await most(), 1, "it was shown, once, while the send was in flight");
  assert.ok(!(await stored()).includes(refused)); assert.equal((await deviceAsked(page)).length, sends, "a refused send reaches no model");
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ profile: "foundry/instant-user-message/1", shownBeforeStored: true, composerCleared: true, neverTwice: true, refusedWithdrawn: true, draftRestored: true, queuedLeftToQueue: true, pageErrors: errors }));
} finally { await browser?.close(); await new Promise((resolve) => server.close(resolve)); }
