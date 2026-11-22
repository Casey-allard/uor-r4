// Native keyboard activation against the built production renderer, not a component fixture.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { featuredApps, featuredOnHome } from "../shell/featured-apps.mjs";

const root = fileURLToPath(new URL("../shell/", import.meta.url));
for (const name of ["core.wasm", "assets/index.js", "manifest.json", "sw.js", "provenance.json"]) {
  assert.ok(
    fs.existsSync(path.join(root, name)),
    `Build and project the actual shell first: ${name}`,
  );
}
const lock = JSON.parse(
  fs.readFileSync(new URL("../vendor/prism/package-lock.json", import.meta.url), "utf8"),
);
assert.equal(
  JSON.parse(
    fs.readFileSync(
      new URL("../vendor/prism/node_modules/playwright-core/package.json", import.meta.url),
      "utf8",
    ),
  ).version,
  lock.packages["node_modules/playwright-core"].version,
);
const app = featuredApps.slice(0, featuredOnHome).find((candidate) => !candidate.url);
assert.ok(app, "the Home keyboard journey requires an offered idea card");
const types = {
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
  ".html": "text/html",
};
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost");
  const relative = decodeURIComponent(url.pathname).replace(/^\/foundry\//, "");
  let file = path.resolve(root, relative);
  if (
    (file !== path.resolve(root) && !file.startsWith(root)) ||
    relative.split("/").includes("..")
  ) {
    res.writeHead(404);
    res.end();
    return;
  }
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
  if (!fs.existsSync(file) && !path.extname(file)) file = path.join(root, "index.html");
  if (!fs.existsSync(file)) {
    res.writeHead(404);
    res.end();
    return;
  }
  res.setHeader("content-type", types[path.extname(file)] || "application/octet-stream");
  res.setHeader("cache-control", "no-store");
  res.end(fs.readFileSync(file));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}/foundry/`,
  origin = new URL(base).origin;
const executablePath =
  process.env.CHROME_BIN ||
  ["/usr/bin/chromium", "/usr/bin/google-chrome"].find((file) => fs.existsSync(file));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "block" });
  await context.route("**/*", (route) =>
    new URL(route.request().url()).origin === origin ? route.continue() : route.abort(),
  );
  const page = await context.newPage(),
    errors = [],
    popups = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("popup", (popup) => popups.push(popup.url()));
  await page.addInitScript(() => {
    window.homeIdeaOpens = [];
    window.addEventListener("foundry:idea", (event) => window.homeIdeaOpens.push(event.detail));
  });
  await page.goto(base);
  const card = page.getByTestId(`app-showcase-card-${app.name}`);
  const keep = card.getByRole("button", { name: `Bookmark ${app.name}`, exact: true });
  const composer = page.locator('[data-testid="home-chat-input-container"] [role="textbox"]');
  await card.waitFor();
  await composer.waitFor();
  const before = await composer.innerText();
  const bookmarkKey = `featured:${app.id}`;
  const bookmarks = () =>
    page.evaluate(() => JSON.parse(localStorage.getItem("foundry:bookmarks") ?? "[]"));
  let bookmarkOpens = 0,
    cardOpens = 0;
  for (const key of ["Enter", "Space"]) {
    for (const chosen of [true, false]) {
      await keep.focus();
      assert.equal(await keep.evaluate((element) => document.activeElement === element), true);
      await keep.press(key);
      bookmarkOpens += await page.evaluate(() => window.homeIdeaOpens.length);
      assert.equal(bookmarkOpens, 0, `Bookmark ${key} opened the card`);
      await page.waitForFunction(
        ({ name, chosen }) =>
          document
            .querySelector(`button[aria-label="Bookmark ${name}"]`)
            ?.getAttribute("aria-pressed") === String(chosen),
        { name: app.name, chosen },
      );
      assert.equal(await keep.getAttribute("aria-pressed"), String(chosen));
      assert.deepEqual(await bookmarks(), chosen ? [bookmarkKey] : []);
      assert.equal(await composer.innerText(), before, "Bookmark changed the prompt");
      assert.equal(page.url(), base);
    }
  }
  for (const key of ["Enter", "Space"]) {
    await card.focus();
    assert.equal(await card.evaluate((element) => document.activeElement === element), true);
    await card.press(key);
    cardOpens++;
    assert.deepEqual(
      await page.evaluate(() => window.homeIdeaOpens),
      Array(cardOpens).fill(app.idea),
      `card ${key} must open exactly once`,
    );
    await page.waitForFunction(
      (idea) =>
        document
          .querySelector('[data-testid="home-chat-input-container"] [role="textbox"]')
          ?.innerText.trim() === idea,
      app.idea,
    );
    assert.equal((await composer.innerText()).trim(), app.idea);
    assert.deepEqual(await bookmarks(), []);
  }
  await keep.press("Enter");
  assert.deepEqual(await bookmarks(), [bookmarkKey]);
  assert.equal(await page.evaluate(() => window.homeIdeaOpens.length), cardOpens);
  await page.reload();
  await keep.waitFor();
  await page.waitForFunction(
    (name) =>
      document
        .querySelector(`button[aria-label="Bookmark ${name}"]`)
        ?.getAttribute("aria-pressed") === "true",
    app.name,
  );
  assert.equal(await keep.getAttribute("aria-pressed"), "true");
  await page.getByRole("button", { name: "Bookmarked", exact: true }).click();
  await keep.press("Space");
  assert.deepEqual(await bookmarks(), []);
  assert.equal(await page.evaluate(() => window.homeIdeaOpens.length), 0);
  await card.waitFor({ state: "hidden" });
  assert.deepEqual(popups, []);
  assert.deepEqual(errors, []);
  console.log(
    JSON.stringify({
      actualRenderer: true,
      nativeKeyboard: true,
      keys: ["Enter", "Space"],
      bookmarkOpens,
      cardOpens,
      repeatedToggle: true,
      persistedBookmark: true,
      savedCardRemoval: true,
      popups: 0,
      pageExceptions: 0,
    }),
  );
} finally {
  if (browser) await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
