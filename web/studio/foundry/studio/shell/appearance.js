// The appearance switch on the renderer's UI: Immersive, Dark, Light, the same canonical state and hooks
// Hologram OS keeps (holo.theme.v1 on <html>: data-holo-palette, data-holo-immersive, --holo-wallpaper),
// mirrored into the renderer's own theme (the dark or light class on <html>, and its localStorage "theme"),
// and followed the other way when the renderer's settings page switches its theme. Adapters only.
//
// The panel is the Prism frame's theme panel (UOR-Foundation/template frame/panels.mjs): the three looks as
// one segmented control, Unsplash search, nine photos (the visitor's recent picks, then the frame's nine
// built-in ones) and the photographer's credit. The built-in photos are the frame's own files on its host;
// search and every found photo go through the frame's Unsplash worker (frame/unsplash), which holds the one
// Unsplash key every Prism app and every visitor shares, so no page carries a key. The markup is built here,
// into the popover the projected page already has, so the page itself is unchanged.
const KEY = "holo.theme.v1";
const $ = (id) => document.getElementById(id);
const WALLS = JSON.parse($("wallpapers").textContent);
const root = document.documentElement;

const FRAME_HOST = "https://uor-foundation.github.io/template/frame/";
const PROXY = "https://prism-unsplash.trinity-03d.workers.dev/";
const UTM = "utm_source=prism&utm_medium=referral";
// The frame's nine built-in photos (frame/panels.mjs BACKDROPS), in its order. Their placeholder colours are
// left out: no colour is typed in the shell (the Brand gate); a found photo brings its own from Unsplash.
const BUILT_IN = [
  {"id":"E0AHdsENmDg","image":"https://images.unsplash.com/photo-1464802686167-b939a6910659?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHwxfHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0","alt":"The Milky Way in blue and violet","name":"Jeremy Thomas","user":"jeremythomasphoto"},
  {"id":"qtRF_RxCAo0","image":"https://images.unsplash.com/photo-1610296669228-602fa827fc1f?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw0fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0","alt":"A nebula in blue, gold and rust","name":"Aldebaran S","user":"aldebarans"},
  {"id":"RF4p4rTM-2s","image":"https://images.unsplash.com/photo-1506703719100-a0f3a48c0f86?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw1fHxtaWxreSUyMHdheXxlbnwxfDB8fHwxNzk0MDU2MjUxfDA&ixlib=rb-4.1.0","alt":"The Andromeda galaxy among stars","name":"Bryan Goff","user":"bryangoffphoto"},
  {"id":"Q1p7bh3SHj8","image":"https://images.unsplash.com/photo-1451187580459-43490279c0fa?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw1fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0","alt":"City lights across Earth at night, from orbit","name":"NASA","user":"nasa"},
  {"id":"t7EL2iG3jMc","image":"https://images.unsplash.com/photo-1534996858221-380b92700493?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw5fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0","alt":"Earth's blue horizon against space","name":"ActionVance","user":"actionvance"},
  {"id":"9wH624ALFQA","image":"https://images.unsplash.com/photo-1515705576963-95cad62945b6?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHwxfHxtaWxreSUyMHdheXxlbnwxfDB8fHwxNzk0MDU2MjUxfDA&ixlib=rb-4.1.0","alt":"The Milky Way rising over mountains","name":"Dns Dgn","user":"dnsdgn"},
  {"id":"oMcTmNHclZI","image":"https://images.unsplash.com/photo-1628818144466-856f7d477125?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTV8&ixlib=rb-4.1.0","alt":"Green and violet aurora over a starry sky","name":"Condor Wei","user":"cwcanchn"},
  {"id":"cZgVpseeazg","image":"https://images.unsplash.com/photo-1545431766-45ff67845191?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTB8&ixlib=rb-4.1.0","alt":"A soft blur of rainbow light","name":"Sean Sinclair","user":"seanwsinclair"},
  {"id":"0wFwSZn6I-I","image":"https://images.unsplash.com/photo-1709625862266-014ef072fd93?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTJ8&ixlib=rb-4.1.0","alt":"Dark curved forms in soft light","name":"Pawel Czerwinski","user":"pawel_czerwinski"},
];
const isBuiltIn = (photo) => BUILT_IN.some((p) => p.id === photo.id);
// Every photo is shown at this screen's own pixels, the most it can display: the worker asks Unsplash for the
// photo at that width (never past the original's own), at high quality, in the smallest format this browser
// decodes. A built-in photo's frame file is only its quick first paint; a sharp copy replaces it once decoded.
const SIDES = [1280, 1920, 2560, 3200, 3840, 5120]; // a few widths, so screens share the worker's cached copies
const ASPECT = 1.5; // the photos are about 3:2; a tall screen covered by one crops its sides, so needs it wider
function screenSide() {
  const sc = globalThis.screen || { width: 1920, height: 1080 };
  const dpr = Math.min(Math.max(globalThis.devicePixelRatio || 1, 1), 4);
  const want = Math.max(sc.width, sc.height * ASPECT) * dpr;
  return SIDES.find((w) => w >= want) || SIDES[SIDES.length - 1];
}
// AVIF where the browser decodes it (about half of WebP's bytes at the same look), else WebP. The answer is
// kept, so every later visit knows it before anything loads.
const FORMAT_KEY = "holo.image.format";
const knownFormat = (() => { try { return localStorage.getItem(FORMAT_KEY); } catch (e) { return "webp"; } })();
let format = knownFormat || "webp";
const AVIF_PROBE = "data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADrbWV0YQAAAAAAAAAhaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAAAAAAAOcGl0bQAAAAAAAQAAAB5pbG9jAAAAAEQAAAEAAQAAAAEAAAETAAAAIQAAAChpaW5mAAAAAAABAAAAGmluZmUCAAAAAAEAAGF2MDFDb2xvcgAAAABqaXBycAAAAEtpcGNvAAAAFGlzcGUAAAAAAAAAAQAAAAEAAAAQcGl4aQAAAAADCAgIAAAADGF2MUOBAAwAAAAAE2NvbHJuY2x4AAEADQAGgAAAABdpcG1hAAAAAAAAAAEAAQQBAoMEAAAAKW1kYXQSAAoIGAAGiAhoNCAyExlHh4Yhh5555oAAAJBAyRxhQr4=";
// A one pixel AVIF decodes at once from memory; the first visit's sharp copy waits for the answer.
const formatReady = knownFormat || typeof Image !== "function" ? Promise.resolve() : (() => {
  const probe = new Image();
  probe.src = AVIF_PROBE;
  return probe.decode().then(() => "avif", () => "webp").then((found) => {
    format = found;
    try { localStorage.setItem(FORMAT_KEY, found); } catch (e) {}
  });
})();
const proxied = (photo, size) => `${PROXY}image?${new URLSearchParams({ src: photo.image, ...size, fm: format })}`;
const sharpOf = (photo) => proxied(photo, { w: String(screenSide()), q: "85" });
// The frame's built-in file paints at once; a found photo's quickest first paint is its tile, already loaded.
const quickOf = (photo) => (isBuiltIn(photo) ? `${FRAME_HOST}backdrops/${photo.id}.jpg` : thumbOf(photo));
// Tiles: twice the frame's 240 by 150, sharp on a phone's 3x screen; under it the built-in tile's frame file,
// shown until the sharp one arrives (two background layers, no script).
const thumbOf = (photo) => proxied(photo, { w: "480", h: "300", q: "75" });
const tileOf = (photo) => [thumbOf(photo), ...(isBuiltIn(photo) ? [`${FRAME_HOST}backdrops/${photo.id}-thumb.jpg`] : [])].map((u) => `url(${JSON.stringify(u)})`).join(", ");
// Decoded sharp copies, by address: one asked for again (hovered, then chosen) is not fetched twice.
const decoded = new Map();
function warm(src) {
  if (!decoded.has(src)) {
    if (typeof Image !== "function") return Promise.reject(new Error("no images here"));
    const img = new Image();
    img.decoding = "async";
    img.src = src;
    const done = img.decode().then(() => { done.ready = true; return src; }, (e) => { decoded.delete(src); throw e; });
    decoded.set(src, done);
  }
  return decoded.get(src);
}
const widthOf = (src) => { const m = /[?&]w=(\d+)/.exec(src || ""); return m && String(src).startsWith(PROXY) && !/[?&]h=/.test(src) ? Number(m[1]) : 0; };
// The photo behind the page, made sharp: kept as the wallpaper once decoded, so the pre paint script shows it
// from the browser's cache on every later load. A copy already as wide as this screen needs is kept.
function sharpen() { formatReady.then(sharpenNow); }
function sharpenNow() {
  const s = readTheme();
  if (!s.immersive || !s.photo || !s.photo.image || isBuiltIn(s.photo)) return; // a built-in photo stays the frame's own file: no worker on load
  const src = sharpOf(s.photo);
  if (s.wallpaper === src || widthOf(s.wallpaper) >= screenSide()) return;
  warm(src).then(() => {
    const now = readJson(KEY) || {};
    if (!now.photo || now.photo.id !== s.photo.id) return; // another photo was chosen meanwhile
    try { localStorage.setItem(KEY, JSON.stringify({ ...now, wallpaper: src })); } catch (e) {}
    if (readTheme().immersive) root.style.setProperty("--holo-wallpaper", `url(${JSON.stringify(src)})`);
  }, () => {});
}

// Home keeps the home look under KEY: a new visitor sees Earth from orbit there. Every other screen (apps,
// library, theater, the build screen, settings) is a place to read and work, so it keeps one shared look of its
// own, Dark until one is chosen on any of them, under BUILD_KEY (its name is kept, so a look chosen on the build
// screen before carries over). The photo, wallpaper and recent picks are one set, shared by both. Home is the
// shell's own folder, read from the stylesheet link that precedes every script; the pre paint script in
// index.html decides the same way.
const BUILD_KEY = "holo.theme.build.v1";
const shellBase = (() => { const l = document.querySelector('link[href="appearance.css"]'); try { return l ? new URL(".", l.href).pathname : "/"; } catch (e) { return "/"; } })();
const isHomeHash = () => { const h = location.hash; return !h || h === "#" || h === "#/" || h === "#telemetry" || h.startsWith("#/?"); };
const onBuild = () => { const p = location.pathname; return !(p === shellBase || p + "/" === shellBase || p === shellBase + "index.html") || !isHomeHash(); };
function readJson(key) { try { return JSON.parse(localStorage.getItem(key) || "null"); } catch (e) { return null; } }
function readTheme() {
  const home = readJson(KEY) || {};
  if (!onBuild()) return home;
  const b = readJson(BUILD_KEY) || { palette: "dark", immersive: false };
  return { ...home, palette: b.palette === "light" ? "light" : "dark", immersive: !!b.immersive };
}
// A choice: kept for the screen it was made on, then shown.
function applyTheme(s) {
  try {
    if (onBuild()) {
      localStorage.setItem(KEY, JSON.stringify({ ...(readJson(KEY) || {}), look: 3, wallpaper: s.wallpaper, photo: s.photo, recent: s.recent }));
      localStorage.setItem(BUILD_KEY, JSON.stringify({ palette: s.palette === "light" ? "light" : "dark", immersive: !!s.immersive }));
    } else localStorage.setItem(KEY, JSON.stringify({ look: 3, ...s }));
  } catch (e) {}
  showTheme(s);
}
function showTheme(s) {
  const palette = s.palette === "light" ? "light" : "dark";
  root.setAttribute("data-holo-palette", palette);
  root.setAttribute("data-holo-immersive", s.immersive ? "on" : "off");
  root.style.setProperty("color-scheme", palette);
  if (s.wallpaper) root.style.setProperty("--holo-wallpaper", `url(${JSON.stringify(s.wallpaper)})`);
  // the renderer's theme follows the palette: its provider reads the class it set and its stored choice.
  if (!root.classList.contains(palette)) { root.classList.remove("light", "dark"); root.classList.add(palette); }
  try { localStorage.setItem("theme", palette); } catch (e) {}
  paint(s);
  sharpen();
}

// ---- the panel
const icon = (mode) => { const b = document.querySelector(`#popover .mode[data-mode="${mode}"] svg`); return b ? b.outerHTML : ""; };
const LENS = '<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><circle cx="7" cy="7" r="4.5"></circle><path d="m10.5 10.5 3 3"></path></svg>';
const icons = { immersive: icon("immersive"), dark: icon("dark"), light: icon("light") };
$("popover").innerHTML = `
  <div class="theme-head">Theme</div>
  <div class="seg" role="radiogroup" aria-label="Look">
    <button class="mode" type="button" data-mode="immersive" role="radio">${icons.immersive}<span>Immersive</span></button>
    <button class="mode" type="button" data-mode="dark" role="radio">${icons.dark}<span>Dark</span></button>
    <button class="mode" type="button" data-mode="light" role="radio">${icons.light}<span>Light</span></button>
  </div>
  <div class="photos-part" id="photos-part">
    <label class="find">${LENS}<input id="find" type="search" placeholder="Search Unsplash" autocomplete="off" spellcheck="false" enterkeyhint="search" aria-label="Search Unsplash" /></label>
    <div class="photos" id="photos" role="listbox" aria-label="Photos"></div>
    <p class="credit" id="credit">Photo by <a id="credit-name" target="_blank" rel="noopener"></a> on <a href="https://unsplash.com/?${UTM}" target="_blank" rel="noopener">Unsplash</a></p>
  </div>`;

let shown = [];
function grid(photos) {
  shown = photos.slice(0, 9);
  const s = readTheme();
  $("photos").innerHTML = "";
  for (const photo of shown) {
    const b = document.createElement("button");
    b.type = "button"; b.className = "photo"; b.title = photo.alt || photo.name; b.setAttribute("aria-label", photo.alt || photo.name);
    b.style.backgroundColor = photo.color || "";
    b.style.backgroundImage = tileOf(photo);
    b.setAttribute("aria-pressed", String(!!s.immersive && s.photo && s.photo.id === photo.id));
    b.onclick = () => choose(photo);
    // Pointing at a photo, or touching it, starts its sharp copy, so it is often ready by the click.
    const ready = () => { warm(sharpOf(photo)).catch(() => {}); };
    b.addEventListener("pointerenter", ready); b.addEventListener("pointerdown", ready); b.addEventListener("focus", ready);
    $("photos").append(b);
  }
}
function defaults() {
  const recent = readTheme().recent || [];
  return [...recent, ...BUILT_IN.filter((p) => !recent.some((r) => r.id === p.id))];
}
function choose(photo) {
  const s = readTheme();
  // A found photo counts as used once set, as Unsplash asks; the built-in ones are the frame's files.
  if (!isBuiltIn(photo)) {
    fetch(`${PROXY}use?id=${encodeURIComponent(photo.id)}`, { method: "POST" }).catch(() => {});
  }
  const recent = isBuiltIn(photo) ? s.recent || [] : [photo, ...(s.recent || []).filter((p) => p.id !== photo.id)].slice(0, 3);
  // The sharp copy at once when it is already decoded; otherwise the quick one now and the sharp one after.
  const sharp = sharpOf(photo);
  const ready = decoded.has(sharp) && decoded.get(sharp).ready;
  applyTheme({ ...s, palette: "dark", immersive: true, wallpaper: ready ? sharp : quickOf(photo), photo, recent });
}
function paint(s) {
  const mode = s.immersive ? "immersive" : s.palette === "light" ? "light" : "dark";
  for (const b of document.querySelectorAll("#popover .mode")) b.setAttribute("aria-checked", String(b.dataset.mode === mode));
  $("photos-part").hidden = mode !== "immersive";
  for (const [i, b] of [...document.querySelectorAll("#photos .photo")].entries()) b.setAttribute("aria-pressed", String(!!s.immersive && !!s.photo && shown[i] && s.photo.id === shown[i].id));
  // The credit: the chosen photo's photographer, or the shell's own wallpaper's.
  const photo = s.photo;
  const wall = WALLS.find((w) => w.file === s.wallpaper);
  // A wallpaper credited only to Unsplash names no photographer, so it gets no line.
  const name = photo ? photo.name : wall && wall.by !== "Unsplash" ? wall.by : "";
  const href = photo ? `https://unsplash.com/@${photo.user}?${UTM}` : wall ? wall.byUrl : "";
  $("credit").hidden = !name;
  $("credit-name").textContent = name;
  $("credit-name").href = href || `https://unsplash.com/?${UTM}`;
}
function setMode(mode) {
  const s = readTheme();
  if (mode === "immersive") applyTheme({ ...s, palette: "dark", immersive: true, wallpaper: s.wallpaper || WALLS[0].file });
  else applyTheme({ ...s, palette: mode, immersive: false, wallpaper: s.wallpaper || WALLS[0].file });
}

// Search: nine landscape photos for the words, through the worker; no words, the default nine.
let searching = null, typing = null;
async function search(words) {
  if (searching) searching.abort();
  if (!words.trim()) { grid(defaults()); return; }
  const control = (searching = new AbortController());
  try {
    const r = await fetch(`${PROXY}search?${new URLSearchParams({ q: words.trim(), page: "1" })}`, { signal: control.signal });
    if (!r.ok) throw new Error(String(r.status));
    const found = await r.json();
    if (searching === control) grid(found.photos || []);
  } catch (e) {
    if (e.name !== "AbortError" && searching === control) $("photos").innerHTML = '<p class="none">Search is unavailable right now.</p>';
  }
}
$("find").addEventListener("input", (e) => { clearTimeout(typing); typing = setTimeout(() => search(e.target.value), 350); });
$("find").addEventListener("keydown", (e) => {
  if (e.key === "Enter") { clearTimeout(typing); search(e.target.value); }
  // The first Escape clears the search; the next closes the panel.
  if (e.key === "Escape") { if (e.target.value) { e.target.value = ""; search(""); e.stopPropagation(); } else close(); }
});

const close = () => { $("popover").hidden = true; $("appearance").setAttribute("aria-expanded", "false"); };
$("appearance").onclick = (e) => {
  e.stopPropagation();
  const open = $("popover").hidden;
  $("popover").hidden = !open;
  $("appearance").setAttribute("aria-expanded", String(open));
  if (open && !$("photos").children.length) grid(defaults());
};
document.addEventListener("click", (e) => { if (!$("popover").contains(e.target)) close(); });
document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !$("popover").hidden) close(); });
for (const b of document.querySelectorAll("#popover .mode")) b.onclick = () => setMode(b.dataset.mode);
showTheme(readTheme());
// Moved to a sharper screen (a window dragged to a 4K display): the photo follows.
let resized = null;
addEventListener("resize", () => { clearTimeout(resized); resized = setTimeout(sharpen, 400); });

// The renderer moves between screens without loading a page (pushState, replaceState, back and forward), so
// the look follows the route: leaving Home or coming back to it shows that screen's look.
let wasBuild = onBuild();
function routed() {
  if (onBuild() === wasBuild) return;
  wasBuild = onBuild();
  showTheme(readTheme());
}
for (const name of ["pushState", "replaceState"]) {
  const move = history[name];
  history[name] = function (...args) { const r = move.apply(this, args); routed(); watchHero(); return r; };
}
addEventListener("popstate", () => { routed(); watchHero(); });
addEventListener("hashchange", () => { routed(); watchHero(); });

// When the renderer's own settings switch its theme class, the palette follows (and Immersive, which is a
// dark look, steps aside for Light).
new MutationObserver(() => {
  const s = readTheme();
  const rtheme = root.classList.contains("light") ? "light" : root.classList.contains("dark") ? "dark" : null;
  if (!rtheme || rtheme === (s.palette === "light" ? "light" : "dark")) return;
  applyTheme({ ...s, palette: rtheme, immersive: rtheme === "light" ? false : s.immersive });
}).observe(root, { attributes: true, attributeFilter: ["class"] });

// Full screen: a second round button, to the right of the sun. It is drawn only where the browser lets a page
// fill the screen (desktop browsers and Android; Safari on iPhone does not), and its icon and label follow the
// state, however full screen was entered or left (the button, Escape, or the browser's own control).
const FULL_ON = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3"></path></svg>';
const FULL_OFF = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 3v3a2 2 0 0 1-2 2H3M21 8h-3a2 2 0 0 1-2-2V3M3 16h3a2 2 0 0 1 2 2v3M16 21v-3a2 2 0 0 1 2-2h3"></path></svg>';
const fullElement = () => document.fullscreenElement || document.webkitFullscreenElement || null;
if (document.fullscreenEnabled || document.webkitFullscreenEnabled) {
  const full = document.createElement("button");
  full.type = "button"; full.id = "fullscreen"; full.className = "appearance fullscreen";
  const show = () => {
    const on = !!fullElement();
    full.innerHTML = on ? FULL_OFF : FULL_ON;
    full.setAttribute("aria-label", on ? "Exit full screen" : "Full screen");
    full.title = on ? "Exit full screen" : "Full screen";
    full.setAttribute("aria-pressed", String(on));
  };
  full.onclick = async (e) => {
    e.stopPropagation();
    try {
      if (fullElement()) await (document.exitFullscreen || document.webkitExitFullscreen).call(document);
      else await (root.requestFullscreen || root.webkitRequestFullscreen).call(root, { navigationUI: "hide" });
    } catch { /* refused by the browser (for example inside a frame); the button keeps its state */ }
    show();
  };
  document.addEventListener("fullscreenchange", show);
  document.addEventListener("webkitfullscreenchange", show);
  show();
  $("appearance").after(full);
  root.classList.add("has-fullscreen");
}

// Home's hero keeps its proportions on every screen. Its measures (the headline, the 610 px box, the chips,
// their spaces) were set for a room of 1060 by 791 px above the floor, 791 being 1280 over the ratio; in a larger
// room the hero grows with it as one piece, by the smaller of the two proportions so it never crowds either edge,
// up to the ratio squared. In a smaller room it keeps those measures, which are already set for it. The scale is
// one number on <html>, which appearance.css applies to the hero alone (zoom), so nothing else moves.
const HERO_ROOM = { width: 1060, height: 791 };
function fitHero(room) {
  const scale = Math.min(Math.max(Math.min(room.clientWidth / HERO_ROOM.width, room.clientHeight / HERO_ROOM.height), 1), 2.618);
  root.style.setProperty("--hero-scale", scale.toFixed(3));
}
// Home is looked for only when the route changes, and only on Home, every 34 ms for up to two seconds while its page mounts:
// other screens (the build screen above all, whose page changes on every edit) pay nothing for it.
let heroRoom = null;
const heroSizes = typeof ResizeObserver === "function" ? new ResizeObserver(() => { if (heroRoom) fitHero(heroRoom); }) : null;
function watchHero(frames = 55) {
  if (!heroSizes) return;
  const room = onBuild() ? null : document.querySelector('[data-slot="home-page"]');
  if (room !== heroRoom) {
    if (heroRoom) heroSizes.unobserve(heroRoom);
    heroRoom = room;
    if (room) { heroSizes.observe(room); fitHero(room); }
  }
  if (!room && !onBuild() && frames > 0) setTimeout(() => watchHero(frames - 1), 34);
}
watchHero();
