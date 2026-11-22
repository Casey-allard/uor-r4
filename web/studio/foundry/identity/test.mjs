// Prism ids: the frame's checks, each refused case failing closed, and the
// whole flow in Chromium across two apps (two origins): a new id with its
// recovery kit, a session, a second app joining the same id by a link and a
// six-digit code, a device removed, and every device lost and the id brought
// back from the kit. Devices meet on stitch's relay, which a relay in memory
// stands in for here, answering the browser as the relay does.
//   node --test identity/test.mjs
import assert from "node:assert/strict";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, describe, test } from "node:test";
import { IdentityError, verifySession } from "../frame/panels.mjs";
import * as Prism from "./prism-id.mjs";
import { start } from "./dev.mjs";

const APP = "https://app.example";
const DAY = 864e5;
const key = () => Buffer.from(crypto.getRandomValues(new Uint8Array(65))).toString("base64url");

// A session as the frame signs one.
async function session(signer, { app = APP, action = "prism/session/1", expires = Date.now() + DAY, now = Date.now(), sessionKey = key() } = {}) {
  const envelope = await Prism.signAction(signer, action, { sessionKey, expires }, { now, audience: app });
  return { app, envelope, records: [...signer.log.values()] };
}
const refuses = (code) => (error) => error instanceof IdentityError && error.code === code;

// Link a second device the way the panel does, without a relay.
async function linked(id) {
  const now = Date.now();
  const { uri, pending } = await Prism.createLinkOffer(id, { now });
  const offer = Prism.parseLinkUri(uri);
  const device = await Prism.createDevice();
  const request = await Prism.requestLink(device, offer);
  const accepted = await Prism.acceptLinkRequest(id, pending, request, { now, confirmCode: await Prism.linkCode(offer.secret, device.record) });
  return { first: accepted.session, second: await Prism.completeLink(device, offer, accepted.response) };
}

describe("the frame's checks", () => {
  test("a new id's session verifies and names the id", async () => {
    const id = await Prism.createPrism();
    const who = await verifySession(await session(id));
    assert.equal(who.account, id.prism);
    assert.equal(who.key, id.device.id);
    assert.match(who.account, /^sha256:[0-9a-f]{64}$/);
  });

  test("a session for another app is refused", async () => {
    const id = await Prism.createPrism();
    const s = await session(id, { app: "https://other.example" });
    await assert.rejects(verifySession({ ...s, app: APP }), refuses("signature"));
  });

  test("a changed session key or expiry breaks the signature", async () => {
    const id = await Prism.createPrism();
    const s = await session(id);
    const swapped = structuredClone(s);
    swapped.envelope.body.payload.sessionKey = key();
    await assert.rejects(verifySession(swapped), refuses("signature"));
    const longer = structuredClone(s);
    longer.envelope.body.payload.expires += DAY;
    await assert.rejects(verifySession(longer), refuses("signature"));
  });

  test("an expired, overlong or stale session is refused", async () => {
    const id = await Prism.createPrism();
    await assert.rejects(verifySession(await session(id, { expires: Date.now() - 1 })), refuses("expired"));
    await assert.rejects(verifySession(await session(id, { expires: Date.now() + 9 * DAY })), refuses("expired"));
    await assert.rejects(verifySession(await session(id, { now: Date.now() - 5 * 60000 })), refuses("signature"));
  });

  test("a kept session checks later, as of its signing", async () => {
    const id = await Prism.createPrism();
    const s = await session(id, { now: Date.now() - 3 * DAY });
    await assert.rejects(verifySession(s), refuses("signature"));
    assert.equal((await verifySession(s, { fresh: 0 })).account, id.prism);
  });

  test("an action that is not a session is refused", async () => {
    const id = await Prism.createPrism();
    await assert.rejects(verifySession(await session(id, { action: "prism/pay/1" })), refuses("action"));
  });

  test("a device removed from the id signs no session", async () => {
    const { first, second } = await linked(await Prism.createPrism());
    const removed = await Prism.revokeDevice(first, second.device.id);
    const s = await session(second);
    assert.equal((await verifySession(s)).account, first.prism);
    await assert.rejects(verifySession({ ...s, records: [...removed.log.values()] }), refuses("signature"));
  });

  test("a device of another id cannot sign for this one", async () => {
    const id = await Prism.createPrism();
    const other = await Prism.createPrism();
    const s = await session(other);
    s.envelope.body.prism = id.prism;
    await assert.rejects(verifySession({ ...s, records: [...id.log.values(), ...other.log.values()] }), refuses("signature"));
    await assert.rejects(verifySession({ ...(await session(other)), records: [...id.log.values()] }), refuses("log"));
  });
});

// The relay in memory, as the browser sees ntfy: POST /<topic> keeps a
// message, GET /<topic>/json?poll=1 lists them, a WebSocket on /<topic>/ws
// hears new ones.
function relayStandIn(origin) {
  const topics = new Map();
  const sockets = new Set();
  const nameOf = (url) => new URL(url).pathname.split("/")[1];
  const relay = {
    topics,
    requests: 0,
    async attach(context) {
      await context.route(`${origin}/**`, async (route) => {
        relay.requests += 1;
        const request = route.request();
        const name = nameOf(request.url());
        const headers = { "access-control-allow-origin": "*", "access-control-allow-headers": "*", "access-control-allow-methods": "GET, POST" };
        if (request.method() === "OPTIONS") return route.fulfill({ status: 204, headers });
        if (request.method() === "POST") {
          const message = request.postData();
          topics.set(name, [...(topics.get(name) ?? []), message]);
          for (const socket of sockets) if (socket.name === name) socket.ws.send(JSON.stringify({ event: "message", message }));
          return route.fulfill({ status: 200, headers, contentType: "application/json", body: "{}" });
        }
        const body = (topics.get(name) ?? []).map((message) => JSON.stringify({ event: "message", message })).join("\n");
        return route.fulfill({ status: 200, headers, contentType: "application/x-ndjson", body });
      });
      await context.routeWebSocket(`${origin.replace(/^https:/, "wss:")}/**`, (ws) => {
        const socket = { name: nameOf(ws.url().replace(/^wss:/, "https:")), ws };
        sockets.add(socket);
        ws.onClose(() => sockets.delete(socket));
      });
    },
  };
  return relay;
}

describe("the flow in Chromium", () => {
  let browser;
  let running;
  let relay;
  let context;
  let first;
  let second;
  let firstId = null;
  let kitPath = null;

  const account = (p) => p.locator("prism-frame").evaluate((frame) => frame.account);
  // The app's workspace key as bytes, or the refusal's code (FR-04).
  const workspaceKey = (p, options) => p.locator("prism-frame").evaluate((frame, o) => frame.workspaceKey(o).then((key) => [...key], (error) => error.code), options);
  let secondKey = null;
  const ready = (p) => p.locator("#account.signed-in").waitFor({ timeout: 15000 });
  const newContext = async () => {
    const made = await browser.newContext({ acceptDownloads: true });
    await relay.attach(made);
    return made;
  };

  before(async () => {
    running = await start({ ports: [8793, 8794] });
    relay = relayStandIn(running.relay);
    const { chromium } = await import("playwright");
    browser = await chromium.launch();
    context = await newContext();
    first = await context.newPage();
    second = await context.newPage();
    await first.goto(running.apps[0]);
    await second.goto(running.apps[1]);
  });

  after(async () => {
    await browser?.close();
    await running?.close();
  });

  test("the icon sits after full screen, and nothing asks to sign in", async () => {
    const order = await first.locator("prism-frame").evaluate((frame) => [...frame.shadowRoot.querySelector(".end").querySelectorAll("button")].map((b) => b.id));
    assert.deepEqual(order.slice(-2), ["fullscreen", "account"]);
    assert.equal(await first.locator("#account").getAttribute("aria-label"), "Prism ID");
    assert.equal(await first.locator("#account-panel").evaluate((panel) => panel.matches(":popover-open")), false);
    assert.equal(await account(first), null);
  });

  test("first use makes the Prism id, with nothing to click; the kit waits in the panel", async () => {
    const asked = relay.requests;
    await first.mouse.click(5, 200);
    await ready(first);
    // A new id has no other device: nothing goes to the relay, nothing is asked of it.
    assert.equal(relay.requests, asked);
    assert.equal(await first.locator("#account-panel").evaluate((panel) => panel.matches(":popover-open")), false);
    firstId = (await account(first)).id;
    const key = await workspaceKey(first);
    assert.equal(key.length, 32);
    assert.deepEqual(await workspaceKey(first), key);
    await first.locator("#account").click();
    assert.equal(await first.locator("#account-held").textContent(), "Your ID and your work, sealed on your devices");
    assert.equal(await first.locator("#account-title").textContent(), "Prism ID");
    assert.equal(await first.getByText(/sign in/i).count(), 0);
    assert.equal(relay.requests, asked);
    const [download] = await Promise.all([first.waitForEvent("download"), first.locator("#account-kit-save").click()]);
    kitPath = await download.path();
    const kit = JSON.parse(await readFile(kitPath, "utf8"));
    assert.equal(kit.kind, "prism/recovery-kit/1");
    assert.equal(kit.share.guardian, "paper");
    assert.equal(kit.prism, firstId);
    await first.locator("#account-kit-save").waitFor({ state: "hidden", timeout: 5000 });
    await first.keyboard.press("Escape");
  });

  test("the session survives a reload and signs for the id", async () => {
    await first.reload();
    await ready(first);
    const checked = await first.evaluate(async () => {
      const frame = document.querySelector("prism-frame");
      const data = new TextEncoder().encode("hello");
      const { signature, delegation } = await frame.sign(data);
      const raw = Uint8Array.from(atob(delegation.envelope.body.payload.sessionKey.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));
      const key = await crypto.subtle.importKey("raw", raw, { name: "ECDSA", namedCurve: "P-256" }, false, ["verify"]);
      const sig = Uint8Array.from(atob(signature.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));
      return { delegation, id: frame.account.id, valid: await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, key, sig, data) };
    });
    assert.equal(checked.valid, true);
    assert.equal(checked.id, firstId);
    assert.equal(checked.delegation.app, new URL(running.apps[0]).origin);
    assert.equal((await verifySession(checked.delegation, { fresh: 0 })).account, checked.id);
  });

  test("an app's pages hear each other on the id's channel, sealed (FR-05)", async () => {
    const other = await context.newPage();
    await other.goto(running.apps[0]);
    await first.evaluate(() => document.querySelector("prism-frame").appChannel((value) => (window.heard = value)));
    await other.evaluate(async () => (await document.querySelector("prism-frame").appChannel(() => {})).send({ hello: "from the other page" }));
    await first.waitForFunction(() => window.heard?.hello === "from the other page", null, { timeout: 10000 });
    await other.close();
  });

  test("the session key and the device keys cannot be read out by script", async () => {
    const keys = await first.evaluate(async () => {
      const read = (name, store, id) =>
        new Promise((done) => {
          const open = indexedDB.open(name);
          open.onsuccess = () => {
            const get = open.result.transaction(store).objectStore(store).get(id);
            get.onsuccess = () => done(get.result);
          };
        });
      const kept = await read("prism-identity", "session", "current");
      const device = await read("prism-id", "prism", "self");
      return [kept.privateKey.extractable, device.device.signingKey.extractable, device.device.agreementKey.extractable];
    });
    assert.deepEqual(keys, [false, false, false]);
  });

  test("another app joins the same id: a link, then the six-digit code", async () => {
    secondKey = await workspaceKey(second);
    await second.locator("#account").click();
    await second.locator("#account-join").click();
    // On the first app: a link offer.
    await first.locator("#account").click();
    await first.locator("#account-link").click();
    await first.locator("#account-offer").waitFor({ timeout: 10000 });
    const uri = await first.locator("#account-offer").textContent();
    assert.match(await first.locator("#account-qr path").getAttribute("d"), /^M\d+ \d+h1v1h-1z/, "the offer shows its QR code");
    assert.match(uri, /^prism-link:/);
    await second.locator("#account-uri").fill(uri);
    await second.locator("#account-join-go").click();
    await second.locator("#account-join-code").waitFor({ timeout: 10000 });
    const code = /(\d{6})/.exec(await second.locator("#account-join-code").textContent())[1];
    await first.locator("#account-code").waitFor({ timeout: 10000 });
    // A wrong code is refused, and the right one still works.
    await first.locator("#account-code").fill(code === "000000" ? "111111" : "000000");
    await first.locator("#account-accept").click();
    await first.locator("#account-error", { hasText: "refused" }).waitFor({ timeout: 10000 });
    await first.locator("#account-code").fill(code);
    await first.locator("#account-accept").click();
    await second.waitForFunction((id) => document.querySelector("prism-frame").account?.id === id, firstId, { timeout: 15000 });
    assert.notEqual((await account(second)).key, (await account(first)).key);
    // The work the second app kept under the id it had now opens under the joined id.
    assert.deepEqual(await workspaceKey(second), secondKey);
  });

  test("a removed app hears it on its next load and stops signing for the id", async () => {
    await first.locator("#account").click();
    await first.locator("#account-devices").click();
    const rows = first.locator("#account-device-list li");
    await rows.first().waitFor({ timeout: 10000 });
    assert.equal(await rows.count(), 3);
    assert.equal(await first.locator("#account-device-list li", { hasText: "(this one)" }).count(), 1);
    assert.equal(await first.locator("#account-device-list li", { hasText: "Recovery kit" }).getByRole("button").count(), 0);
    await first.locator("#account-device-list li", { hasText: "Browser" }).filter({ hasNotText: "(this one)" }).getByRole("button", { name: "Remove" }).click();
    await first.locator("#account-device-list li").nth(2).waitFor({ state: "detached", timeout: 10000 });
    await first.keyboard.press("Escape");

    await second.reload();
    await second.mouse.click(5, 200);
    await second.waitForFunction((id) => { const a = document.querySelector("prism-frame").account; return a && a.id !== id; }, firstId, { timeout: 15000 });
    // A removed device keeps its work locked, and may start again with a new key.
    assert.equal(await workspaceKey(second), "removed");
    assert.notDeepEqual(await workspaceKey(second, { replace: true }), secondKey);
  });

  test("every device lost: the recovery kit brings the id back, and the kit is replaced", async () => {
    const elsewhere = await newContext();
    const fresh = await elsewhere.newPage();
    await fresh.goto(running.apps[1]);
    await fresh.locator("#account").click();
    await fresh.locator("#account-recover").click();
    await fresh.locator("#account-kit-file").setInputFiles(kitPath);
    await fresh.waitForFunction((id) => document.querySelector("prism-frame").account?.id === id, firstId, { timeout: 15000 });
    await fresh.locator("#account-kit-save").waitFor({ timeout: 15000 });
    const [download] = await Promise.all([fresh.waitForEvent("download"), fresh.locator("#account-kit-save").click()]);
    const newKit = JSON.parse(await readFile(await download.path(), "utf8"));
    assert.equal(newKit.prism, firstId);
    assert.notEqual(newKit.share.recovery, JSON.parse(await readFile(kitPath, "utf8")).share.recovery);

    // Recovery removed every other device: the first app hears it on its next load.
    await first.reload();
    await first.mouse.click(5, 200);
    await first.waitForFunction((id) => { const a = document.querySelector("prism-frame").account; return a && a.id !== id; }, firstId, { timeout: 15000 });

    // The old kit brings nothing back.
    const third = await (await newContext()).newPage();
    await third.goto(running.apps[0]);
    await third.locator("#account").click();
    await third.locator("#account-recover").click();
    await third.locator("#account-kit-file").setInputFiles(kitPath);
    await third.locator("#account-error", { hasText: "already used" }).waitFor({ timeout: 15000 });
    assert.notEqual((await account(third))?.id, firstId);
    await third.context().close();
    await elsewhere.close();
  });

  test("a guardian: the new kit needs them, they release their share once, and a copied kit gets nothing", async () => {
    const holder = await (await newContext()).newPage();
    const guardian = await (await newContext()).newPage();
    await holder.goto(running.apps[0]);
    await guardian.goto(running.apps[1]);
    await holder.mouse.click(5, 200);
    await guardian.mouse.click(5, 200);
    await ready(holder);
    await ready(guardian);
    const holderId = (await account(holder)).id;

    // The guardian offers a link; the holder pastes it, from the kit still waiting in the panel.
    await guardian.locator("#account").click();
    await guardian.locator("#account-guardian").click();
    await guardian.locator("#account-guardian-link", { hasText: "prism-guardian:" }).waitFor({ timeout: 10000 });
    const offer = await guardian.locator("#account-guardian-link").textContent();
    await holder.locator("#account").click();
    await holder.locator("#account-guard").click();
    assert.equal(await holder.locator("#account-guard-kit-label").isHidden(), true, "the unsaved kit is used, nothing to choose");
    await holder.locator("#account-guard-uri").fill(offer);
    await holder.locator("#account-guard-go").click();
    await guardian.locator("#account-help").waitFor({ timeout: 15000 });
    await holder.locator("#account-kit-save").waitFor({ timeout: 15000 });
    assert.equal(await holder.locator("#account-guard").isHidden(), true);
    const [download] = await Promise.all([holder.waitForEvent("download"), holder.locator("#account-kit-save").click()]);
    const guardedKit = join(await mkdtemp(join(tmpdir(), "prism-kit-")), "kit.json");
    await download.saveAs(guardedKit);
    const saved = JSON.parse(await readFile(guardedKit, "utf8"));
    assert.equal(saved.share.guardian, "paper");
    assert.equal(saved.share.threshold, 2);
    await holder.context().close();

    // Every device lost: the kit asks for the guardian, who checks the code and releases.
    const helped = async () => {
      const page = await (await newContext()).newPage();
      await page.goto(running.apps[0]);
      await page.locator("#account").click();
      await page.locator("#account-recover").click();
      await page.locator("#account-kit-file").setInputFiles(guardedKit);
      await page.locator("#account-ask-link", { hasText: "prism-help:" }).waitFor({ timeout: 15000 });
      return { page, link: await page.locator("#account-ask-link").textContent(), code: await page.locator("#account-ask-code").textContent() };
    };
    const lost = await helped();
    if (!(await guardian.locator("#account-help").isVisible())) await guardian.locator("#account").click();
    await guardian.locator("#account-help").click();
    await guardian.locator("#account-help-uri").fill(lost.link);
    await guardian.locator("#account-help-go").click();
    await guardian.locator("#account-help-code").waitFor({ timeout: 15000 });
    assert.equal(await guardian.locator("#account-help-code").textContent(), lost.code);
    await guardian.locator("#account-help-release").click();
    await lost.page.waitForFunction((id) => document.querySelector("prism-frame").account?.id === id, holderId, { timeout: 15000 });
    await lost.page.locator("#account-kit-save").waitFor({ timeout: 15000 });
    await guardian.locator("#account-help").waitFor({ state: "hidden", timeout: 10000 });

    // A copy of the same kit: the guardian has nothing left to release.
    const copied = await helped();
    assert.equal(await guardian.locator("#account-help").isHidden(), true);
    const store = await guardian.evaluate(
      () =>
        new Promise((done) => {
          const open = indexedDB.open("prism-id");
          open.onsuccess = () => {
            const get = open.result.transaction("prism").objectStore("prism").get("guarding");
            get.onsuccess = () => done(get.result);
          };
        }),
    );
    assert.deepEqual(store, []);
    assert.notEqual((await account(copied.page))?.id, holderId);
    await copied.page.context().close();
    await lost.page.context().close();
    await guardian.context().close();
  });
});
