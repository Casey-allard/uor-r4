// The working look: every screen but Home (the apps, the library, the build screen) opens Dark until a look
// is chosen on one of them, keeps that choice, and Home keeps the home look, Earth from orbit for a new visitor. Both places that decide
// the look are run here, as the browser runs them, against a small stand-in page:
//   the pre paint script in shell/index.html (a direct load or refresh, before any module), and
//   shell/appearance.js (moving between screens with pushState and popstate, and choosing a look).
//   node tools/build_look.mjs
import fs from "node:fs";
import vm from "node:vm";

const here = new URL("../shell/", import.meta.url);
const page = fs.readFileSync(new URL("index.html", here), "utf8");
const prepaint = [...page.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]).find((s) => s.includes("holo.theme.v1"));
const walls = page.match(/<script type="application\/json" id="wallpapers">([\s\S]*?)<\/script>/)[1];
const appearance = fs.readFileSync(new URL("appearance.js", here), "utf8");

// A stand-in page: <html>'s classes and attributes, localStorage, location, history, and inert elements.
function world(path, stored) {
  const store = new Map(Object.entries(stored).map(([k, v]) => [k, JSON.stringify(v)]));
  const classes = new Set(["notranslate"]);
  const attrs = new Map();
  const listeners = {};
  const element = () => ({ style: { setProperty() {} }, hidden: false, innerHTML: "", textContent: "", children: [], dataset: {}, append() {}, setAttribute() {}, getAttribute() { return null; }, addEventListener() {}, contains: () => false });
  const elements = { wallpapers: { ...element(), textContent: walls } };
  const modes = ["immersive", "dark", "light"].map((mode) => ({ ...element(), dataset: { mode } }));
  const root = {
    classList: { contains: (c) => classes.has(c), add: (c) => classes.add(c), remove: (...cs) => cs.forEach((c) => classes.delete(c)) },
    setAttribute: (k, v) => attrs.set(k, v), getAttribute: (k) => attrs.get(k) ?? null, style: { setProperty() {} },
  };
  const location = { pathname: path };
  const to = (url) => { location.pathname = new URL(url, "http://forge.test" + location.pathname).pathname; };
  const history = { pushState(_s, _t, url) { to(url); }, replaceState(_s, _t, url) { to(url); } };
  const g = {
    localStorage: { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, String(v)) },
    location, history, JSON, URL, URLSearchParams, setTimeout, clearTimeout,
    addEventListener: (type, fn) => (listeners[type] ||= []).push(fn),
    MutationObserver: class { observe() {} },
    fetch: () => Promise.reject(new Error("offline")),
    document: {
      documentElement: root,
      getElementById: (id) => (elements[id] ||= element()),
      querySelector: (sel) => (sel === 'link[href="appearance.css"]' ? { href: "http://forge.test/foundry/appearance.css" } : null),
      querySelectorAll: (sel) => (sel === "#popover .mode" ? modes : []),
      addEventListener() {},
    },
  };
  g.window = g;
  vm.createContext(g);
  return {
    g,
    look: () => ({ palette: classes.has("dark") ? "dark" : classes.has("light") ? "light" : "none", immersive: attrs.get("data-holo-immersive") }),
    build: () => g.localStorage.getItem("holo.theme.build.v1"),
    go: (url) => g.history.pushState(null, "", url),
    back: (url) => { location.pathname = url; (listeners.popstate || []).forEach((fn) => fn({})); },
    choose: (mode) => modes.find((b) => b.dataset.mode === mode).onclick(),
  };
}

let failed = 0;
const want = (name, got, expected) => {
  const ok = Object.entries(expected).every(([k, v]) => got[k] === v);
  if (!ok) failed++;
  console.log(`${ok ? "ok    " : "WRONG "}${name.padEnd(58)} ${JSON.stringify(got)}${ok ? "" : "  wanted " + JSON.stringify(expected)}`);
};
const HOME = "/foundry/", BARE = "/foundry", CHAT = "/foundry/chat", DETAILS = "/foundry/app-details", APPS = "/foundry/apps";
const light = { look: 3, palette: "light", immersive: false };
const immersive = { look: 3, palette: "dark", immersive: true, wallpaper: "wallpapers/alps.jpg" };

// Direct loads and refreshes: the pre paint script alone.
for (const [name, path, stored, expected] of [
  ["first run, home: Immersive", HOME, {}, { palette: "dark", immersive: "on" }],
  ["first run, build screen: Dark", CHAT, {}, { palette: "dark", immersive: "off" }],
  ["first run, apps: Dark", APPS, {}, { palette: "dark", immersive: "off" }],
  ["first run, home without its slash: Immersive", BARE, {}, { palette: "dark", immersive: "on" }],
  ["home Immersive, apps not chosen: Dark", APPS, { "holo.theme.v1": immersive }, { palette: "dark", immersive: "off" }],
  ["home Light, build screen not chosen: Dark", CHAT, { "holo.theme.v1": light }, { palette: "dark", immersive: "off" }],
  ["home Immersive, build screen not chosen: Dark", CHAT, { "holo.theme.v1": immersive }, { palette: "dark", immersive: "off" }],
  ["build screen chose Light: Light", CHAT, { "holo.theme.v1": immersive, "holo.theme.build.v1": { palette: "light", immersive: false } }, { palette: "light", immersive: "off" }],
  ["build screen chose Immersive: Immersive", CHAT, { "holo.theme.v1": light, "holo.theme.build.v1": { palette: "dark", immersive: true } }, { palette: "dark", immersive: "on" }],
  ["home with a build choice: home's own Light", HOME, { "holo.theme.v1": light, "holo.theme.build.v1": { palette: "dark", immersive: true } }, { palette: "light", immersive: "off" }],
]) {
  const w = world(path, stored);
  vm.runInContext(prepaint, w.g);
  want("pre paint, " + name, w.look(), expected);
}

// Moving between screens without a page load, and choosing: appearance.js.
{
  const w = world(HOME, { "holo.theme.v1": light });
  vm.runInContext(appearance, w.g);
  want("home: Light", w.look(), { palette: "light", immersive: "off" });
  w.go(DETAILS);
  want("pushState to another screen: Dark, nothing stored", { ...w.look(), build: w.build() }, { palette: "dark", immersive: "off", build: null });
  w.go(CHAT + "?id=1&appId=1");
  want("pushState to the build screen: Dark, nothing stored", { ...w.look(), build: w.build() }, { palette: "dark", immersive: "off", build: null });
  w.back(HOME);
  want("back (popstate) to home: home look", w.look(), { palette: "light", immersive: "off" });
  w.back(CHAT);
  want("forward (popstate) to the build screen: Dark", w.look(), { palette: "dark", immersive: "off" });
  w.choose("immersive");
  want("Immersive chosen on the build screen", { ...w.look(), build: w.build() }, { palette: "dark", immersive: "on", build: '{"palette":"dark","immersive":true}' });
  w.go(APPS);
  want("apps share the working look: Immersive", w.look(), { palette: "dark", immersive: "on" });
  w.go(HOME);
  want("home again: its Light, untouched", w.look(), { palette: "light", immersive: "off" });
  w.choose("dark");
  w.go(CHAT);
  want("home chose Dark; the build screen keeps its Immersive", w.look(), { palette: "dark", immersive: "on" });
  w.choose("light");
  w.go(HOME);
  want("build screen chose Light; home keeps its Dark", w.look(), { palette: "dark", immersive: "off" });
}

if (failed) { console.error(`build look: ${failed} wrong`); process.exit(1); }
console.log("build look: every screen but Home opens Dark, keeps one shared choice, and leaves the home look alone");
