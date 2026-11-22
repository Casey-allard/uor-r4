// Chat and Plan in the actual shell: the build screen with its preview collapsed (mounted, as in Build),
// suggestions above the composer, one chat panel across every change of mode, and nothing written to the app.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { DEVICE_MODEL, deviceAsked, deviceHold, deviceRelease, deviceSeam } from "./device_fixture.mjs";
import { plainChatSuggestions, PLAIN_CHAT_WORDS } from "../vendor/prism/hologram.plain-chat-data.mjs";
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
const labels = (mode, hasSpec) => plainChatSuggestions(mode, hasSpec).map((chip) => chip.label);
const fixture = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json", import.meta.url), "utf8"));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "block", viewport: { width: 1280, height: 844 } }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", (route) => (new URL(route.request().url()).origin === origin ? route.continue() : route.abort()));
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const call = (channel, input) => page.evaluate(({ channel, input }) => window.electron.ipcRenderer.invoke(channel, input), { channel, input });
  await call("set-user-settings", { selectedModel: DEVICE_MODEL });
  const made = await call("create-app", { name: "plain-chat", initialChatMode: "build" }), chatId = made.chatId, appId = made.app.id;
  // Each page load starts the preview once and asks for its permission; the answer above declines it. The test goes on only once that has settled, so the dialog never opens under a key press.
  const previewSettled = () => page.waitForFunction(async (appId) => (await window.electron.ipcRenderer.invoke("distributed-machine:subscribe", { protocolVersion: 1, machineId: "app_run", encodedKey: { appId } })).encodedState.phase === "errored" && !document.querySelector('dialog[aria-label="Review preview permissions"]'), appId, { timeout: 30000 });
  const open = async () => { await page.goto(`${base}chat?id=${chatId}&appId=${appId}`); await page.getByTestId("chat-input-container").getByRole("textbox").waitFor(); await previewSettled(); };
  const mode = page.getByTestId("chat-mode-selector"), input = page.getByTestId("chat-input-container").getByRole("textbox");
  const chips = page.locator('[data-slot="chat-suggestions"] button');
  // The preview dialog is the preview's own (tools/preview_auto_start_browser.mjs): it is answered when it appears.
  await page.addInitScript(() => { setInterval(() => { const dialog = document.querySelector('dialog[aria-label="Review preview permissions"]'); const deny = dialog && [...dialog.querySelectorAll("button")].find((b) => b.textContent === "Don't allow"); if (deny) deny.click(); }, 50); });
  const state = () => page.evaluate(() => {
    const preview = document.getElementById("preview-panel"), chat = document.getElementById("chat-panel");
    return { layout: chat?.querySelector("[data-chat-layout]")?.getAttribute("data-chat-layout") ?? null, previewWidth: preview ? Math.round(preview.getBoundingClientRect().width) : null, previewNodes: preview ? preview.querySelectorAll("*").length : null, chips: [...document.querySelectorAll('[data-slot="chat-suggestions"] button')].map((b) => b.textContent), row: !!document.querySelector('[data-slot="chat-suggestions"]'), placeholder: document.querySelector('[data-testid="chat-input-container"] [role=textbox]')?.getAttribute("aria-placeholder") };
  });
  const until = async (check, what) => { const deadline = Date.now() + 15000; let last; while (Date.now() < deadline) { last = await state(); if (check(last)) return last; await page.waitForTimeout(25); } assert.fail(`${what}: ${JSON.stringify(last)}`); };
  const choose = async (value) => { await mode.selectOption(value); await page.waitForFunction(async ({ value, chatId }) => document.querySelector('[data-testid="chat-mode-selector"]').value === value && (await window.__host.get("chats", chatId))?.chatMode === value, { value, chatId }); };

  // Build is the build screen: the preview beside the chat, no suggestions, upstream's placeholder.
  await open();
  const build = await until((s) => s.layout === "build" && s.previewWidth > 0 && s.previewNodes > 0, "Build shows the preview");
  assert.equal(build.row, false); assert.deepEqual(build.chips, []);
  const buildPlaceholder = build.placeholder; assert.ok(buildPlaceholder && !Object.values(PLAIN_CHAT_WORDS).some((w) => w.placeholder === buildPlaceholder));

  // Chat: the same chat panel, the preview collapsed, the draft kept, four suggestions for an app without rules.
  await input.fill("a draft kept across modes");
  await page.evaluate(() => { document.querySelector('[data-testid="messages-list"]').__kept = true; document.getElementById("chat-panel").__kept = true; });
  await choose("ask");
  const ask = await until((s) => s.layout === "plain" && s.previewWidth === 0 && s.chips.length === 4, "Chat collapses the preview and shows suggestions");
  assert.deepEqual(ask.chips, labels("ask", false)); assert.equal(ask.placeholder, PLAIN_CHAT_WORDS.ask.placeholder);
  assert.equal(await input.textContent(), "a draft kept across modes");
  assert.deepEqual(await page.evaluate(() => [document.querySelector('[data-testid="messages-list"]').__kept === true, document.getElementById("chat-panel").__kept === true]), [true, true], "the chat section is not remounted by a change of mode");
  assert.equal(await page.getByText(PLAIN_CHAT_WORDS.ask.heading, { exact: true }).count(), 1);

  // Plan has its own; a chip writes its prompt into the composer and sends nothing.
  await choose("plan");
  const plan = await until((s) => s.layout === "plain" && s.chips[0] === labels("plan", false)[0], "Plan shows its own suggestions");
  assert.deepEqual(plan.chips, labels("plan", false)); assert.notDeepEqual(plan.chips, ask.chips); assert.equal(plan.placeholder, PLAIN_CHAT_WORDS.plan.placeholder);
  await chips.nth(1).focus(); await page.keyboard.press("Enter");
  await page.waitForFunction((text) => document.querySelector('[data-testid="chat-input-container"] [role=textbox]')?.textContent === text, plainChatSuggestions("plan", false)[1].prompt);
  assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("role")), "textbox", "the composer has the focus after a suggestion");
  assert.equal((await deviceAsked(page)).length, 0);
  assert.equal(await page.getByRole("group", { name: "Suggestions", exact: true }).count(), 1);
  await input.fill("");

  // An app with rules is asked about itself; a chat that opens in Chat opens collapsed, its preview mounted as ever.
  await page.evaluate(({ appId, spec }) => window.__prism.rulesAfterTurn(appId, "<prism-spec>" + JSON.stringify(spec) + "</prism-spec>"), { appId, spec: fixture.spec ?? fixture });
  await choose("ask");
  // Opened as a chat this browser has never seen: its mode is read before the preview is given anything.
  await page.evaluate((chatId) => localStorage.removeItem("holo.chat.mode." + chatId), chatId); await open();
  const reopened = await until((s) => s.layout === "plain" && s.chips.length === 4 && s.chips[0] === labels("ask", true)[0], "a Chat chat reopens as chat alone, with the app's suggestions");
  assert.deepEqual(reopened.chips, labels("ask", true)); assert.equal(reopened.previewWidth, 0); assert.ok(reopened.previewNodes > 0, "the preview is mounted, as in Build");

  // A read-only turn from this screen changes nothing in the app, and the suggestions step aside while it runs.
  const snapshot = () => page.evaluate(async (appId) => ({ files: await window.__host.all("files"), head: await window.__host.get("refs", `${appId}:main`), rules: await window.__prism.rulesOf(appId), versions: await window.__host.get("versions", appId) }), appId), before = await snapshot();
  await deviceHold(page);
  await chips.first().click(); await input.press("Enter");
  for (const deadline = Date.now() + 60000; !(await deviceAsked(page)).length; await new Promise((r) => setTimeout(r, 50))) assert.ok(Date.now() < deadline, "the reader is asked the turn");
  assert.deepEqual((await state()).chips, [], "no suggestions while a turn is in flight");
  await deviceRelease(page);
  await page.getByTestId("messages-list").getByText("foundry:reader").first().waitFor();
  const after = await until((s) => s.chips.length === 4, "the suggestions return after the reply");
  assert.equal(after.previewWidth, 0); assert.equal((await deviceAsked(page)).length, 1); assert.deepEqual(await snapshot(), before, "a Chat turn writes nothing to the app");
  assert.equal(await page.getByTestId("messages-list").locator("prism-write").count(), 0);
  const settled = await page.evaluate(() => { const row = document.querySelector('[data-slot="chat-suggestions"]').getBoundingClientRect(), box = document.querySelector('[data-testid="chat-input-container"]').getBoundingClientRect(); return { rowHeight: Math.round(row.height), above: row.bottom <= box.top + 1, inside: box.bottom <= innerHeight + 1 }; });
  assert.deepEqual(settled, { rowHeight: 48, above: true, inside: true });
  await input.fill("typing hides them"); await until((s) => s.chips.length === 0 && s.row, "typing over a conversation hides the chips and keeps their row");
  await input.fill("");

  // The header's own toggle still opens the preview from Chat; Build opens it, and going back collapses it mounted.
  await page.getByTestId("toggle-preview-panel-button").click();
  await until((s) => s.layout === "plain" && s.previewWidth > 0 && s.previewNodes > 0, "the toggle opens the preview in Chat");
  assert.equal(await mode.inputValue(), "ask");
  await page.getByTestId("toggle-preview-panel-button").click(); await until((s) => s.previewWidth === 0, "the toggle closes it again");
  await choose("build"); const again = await until((s) => s.layout === "build" && s.previewWidth > 0 && s.previewNodes > 0 && !s.row, "Build opens the preview");
  await choose("plan"); const kept = await until((s) => s.layout === "plain" && s.previewWidth === 0, "Plan collapses it");
  assert.ok(kept.previewNodes > 0 && again.previewNodes > 0, "the preview stays mounted across modes");

  // On a phone the row scrolls sideways inside the screen and the composer stays on it.
  await page.setViewportSize({ width: 375, height: 760 }); await open();
  await until((s) => s.layout === "plain" && s.chips.length === 4, "Plan on a phone");
  const phone = await page.evaluate(() => { const row = document.querySelector('[data-slot="chat-suggestions"]'), r = row.getBoundingClientRect(), box = document.querySelector('[data-testid="chat-input-container"]').getBoundingClientRect(); return { rowHeight: Math.round(r.height), rowInside: r.left >= 0 && r.right <= innerWidth + 1, scrolls: getComputedStyle(row).overflowX === "auto", composerInside: box.top >= 0 && box.bottom <= innerHeight + 1 && box.right <= innerWidth + 1, pageScrolls: document.documentElement.scrollWidth > innerWidth + 1 }; });
  assert.deepEqual(phone, { rowHeight: 48, rowInside: true, scrolls: true, composerInside: true, pageScrolls: false });
  await chips.last().focus(); assert.ok(await chips.last().evaluate((b) => { const r = b.getBoundingClientRect(); return r.left >= 0 && r.right <= innerWidth + 1; }), "a focused chip is scrolled into view");
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ profile: "foundry/plain-chat-layout/1", layoutByMode: true, chatPanelKept: true, draftKept: true, suggestionsByMode: true, suggestionsByApp: true, suggestionFillsComposer: true, hiddenWhileStreaming: true, previewCollapsedMounted: true, previewToggle: true, readOnly: true, phone: true, pageErrors: errors }));
} finally { await browser?.close(); await new Promise((resolve) => server.close(resolve)); }
