// What the bar needs only once the visitor moves: every panel's styles
// (frame/panels.css), the theme panel, and the immersive photos (the nine
// built in, any other found on Unsplash, each fetched once and kept).
// frame/publish.mjs publishes it beside the frame, named by its SHA-256,
// and writes that hash into the frame, which fetches it after the visitor's
// first move, checks it and imports it: the page's first load carries none
// of it.
const CSS = "/*PANELS_CSS*/";
export const sheet = typeof CSSStyleSheet === "function" ? new CSSStyleSheet() : null;
sheet?.replaceSync(CSS);

const UNSPLASH = "utm_source=prism&utm_medium=referral";
const UNSPLASH_PROXY = "https://prism-unsplash.trinity-03d.workers.dev/";
const FRAME_HOST = "https://uor-foundation.github.io/template/frame/";
const BACKDROPS = [
  {
    id: "E0AHdsENmDg",
    image: "https://images.unsplash.com/photo-1464802686167-b939a6910659?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHwxfHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0",
    color: "#0c2659",
    alt: "The Milky Way in blue and violet",
    name: "Jeremy Thomas",
    user: "jeremythomasphoto",
    page: "https://unsplash.com/photos/blue-and-purple-galaxy-digital-wallpaper-E0AHdsENmDg",
  },
  {
    id: "qtRF_RxCAo0",
    image: "https://images.unsplash.com/photo-1610296669228-602fa827fc1f?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw0fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0",
    color: "#262626",
    alt: "A nebula in blue, gold and rust",
    name: "Aldebaran S",
    user: "aldebarans",
    page: "https://unsplash.com/photos/blue-and-black-galaxy-digital-wallpaper-qtRF_RxCAo0",
  },
  {
    id: "RF4p4rTM-2s",
    image: "https://images.unsplash.com/photo-1506703719100-a0f3a48c0f86?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw1fHxtaWxreSUyMHdheXxlbnwxfDB8fHwxNzk0MDU2MjUxfDA&ixlib=rb-4.1.0",
    color: "#262626",
    alt: "The Andromeda galaxy among stars",
    name: "Bryan Goff",
    user: "bryangoffphoto",
    page: "https://unsplash.com/photos/galaxy-with-starry-night-RF4p4rTM-2s",
  },
  {
    id: "Q1p7bh3SHj8",
    image: "https://images.unsplash.com/photo-1451187580459-43490279c0fa?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw1fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0",
    color: "#0c2626",
    alt: "City lights across Earth at night, from orbit",
    name: "NASA",
    user: "nasa",
    page: "https://unsplash.com/photos/photo-of-outer-space-Q1p7bh3SHj8",
  },
  {
    id: "t7EL2iG3jMc",
    image: "https://images.unsplash.com/photo-1534996858221-380b92700493?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHw5fHxzcGFjZXxlbnwxfDB8fHwxNzk0MDU1OTU2fDA&ixlib=rb-4.1.0",
    color: "#59598c",
    alt: "Earth's blue horizon against space",
    name: "ActionVance",
    user: "actionvance",
    page: "https://unsplash.com/photos/outer-space-photography-of-earth-t7EL2iG3jMc",
  },
  {
    id: "9wH624ALFQA",
    image: "https://images.unsplash.com/photo-1515705576963-95cad62945b6?ixid=M3w4OTc5NTl8MHwxfHNlYXJjaHwxfHxtaWxreSUyMHdheXxlbnwxfDB8fHwxNzk0MDU2MjUxfDA&ixlib=rb-4.1.0",
    color: "#0c2640",
    alt: "The Milky Way rising over mountains",
    name: "Dns Dgn",
    user: "dnsdgn",
    page: "https://unsplash.com/photos/milky-way-on-mountains-9wH624ALFQA",
  },
  {
    id: "oMcTmNHclZI",
    image: "https://images.unsplash.com/photo-1628818144466-856f7d477125?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTV8&ixlib=rb-4.1.0",
    color: "#0c2626",
    alt: "Green and violet aurora over a starry sky",
    name: "Condor Wei",
    user: "cwcanchn",
    page: "https://unsplash.com/photos/aurora-borealis-in-starry-night-sky-oMcTmNHclZI",
  },
  {
    id: "cZgVpseeazg",
    image: "https://images.unsplash.com/photo-1545431766-45ff67845191?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTB8&ixlib=rb-4.1.0",
    color: "#26260c",
    alt: "A soft blur of rainbow light",
    name: "Sean Sinclair",
    user: "seanwsinclair",
    page: "https://unsplash.com/photos/abstract-rainbow-color-blur-cZgVpseeazg",
  },
  {
    id: "0wFwSZn6I-I",
    image: "https://images.unsplash.com/photo-1709625862266-014ef072fd93?ixid=M3w4OTc5NTl8MHwxfGFsbHx8fHx8fHx8fDE3OTQwNTIyMTJ8&ixlib=rb-4.1.0",
    color: "#262626",
    alt: "Dark curved forms in soft light",
    name: "Pawel Czerwinski",
    user: "pawel_czerwinski",
    page: "https://unsplash.com/photos/a-black-and-white-photo-of-a-curved-object-0wFwSZn6I-I",
  },
];
// A photo as a data: URL. Every page the frame runs on may show data:
// images, while most may show no image from elsewhere. A built-in photo is
// fetched from the frame's own host, which every app page's connect-src
// names, so it works in every Prism app and in Holo Search, unrebuilt; any
// other comes through the worker. Each is fetched once per page, and only
// once it is to be shown.
//
// Never with the page, though: a visitor who chose immersive gets it on every
// load, and an app's Lighthouse budget allows two third-party requests, which the
// frame's pointer and module take. A photo shown before comes from this
// origin's cache at once, with no request; a new one waits for the
// visitor's first move (a pointer, a touch, a key, a scroll), as the stars
// and the Prism index do, and its ground is the photo's colour until then.
// The cache is not named prism-*: those are the app's service worker's.
const SHOWN = "holo-backdrops";
const KEPT = 4;
const shownBefore = async (url) => {
  try {
    return (await (await caches.open(SHOWN)).match(url)) ?? null;
  } catch {
    return null; // No Cache API here (an insecure origin): fetched as new.
  }
};
// The last few photos shown, and no more: each is a few hundred kilobytes.
const keep = async (url, response) => {
  try {
    const cache = await caches.open(SHOWN);
    await cache.put(url, response);
    const kept = await cache.keys();
    for (const old of kept.slice(0, Math.max(0, kept.length - KEPT))) await cache.delete(old);
  } catch {
    // Kept for this page only.
  }
};
const pictures = new Map();
const picture = (url, { thumb = false } = {}) => {
  if (!pictures.has(url)) {
    const loading = (async () => {
      // A thumbnail is asked for only by the open panel, after a move.
      let response = thumb ? null : await shownBefore(url);
      if (!response) {
        response = await fetch(url);
        if (!response.ok) throw new Error(`backdrop: HTTP ${response.status}`);
        if (!thumb) keep(url, response.clone());
      }
      const blob = await response.blob();
      return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(blob);
      });
    })();
    // A failed fetch is tried again the next time the photo is wanted.
    loading.catch(() => pictures.delete(url));
    pictures.set(url, loading);
  }
  return pictures.get(url);
};
const builtIn = (photo) => BACKDROPS.some((p) => p.id === photo.id);
const pictureOf = (photo, thumb = false) =>
  picture(
    builtIn(photo)
      ? new URL(`backdrops/${photo.id}${thumb ? "-thumb" : ""}.jpg`, FRAME_HOST).href
      : `${UNSPLASH_PROXY}image?${new URLSearchParams(thumb ? { src: photo.image, w: "240", h: "150" } : { src: photo.image, w: "2400" })}`,
    { thumb },
  );
// Search is offered where the page lets the frame reach the worker: a page
// whose policy names it (every app page built from this template) or one
// with no policy (Holo Search).
const searchable = () => {
  const policy = document.querySelector('meta[http-equiv="Content-Security-Policy" i]')?.content ?? "";
  const rule = /(?:^|;)\s*connect-src\s([^;]*)/i.exec(policy) ?? /(?:^|;)\s*default-src\s([^;]*)/i.exec(policy);
  if (!rule) return true;
  const origin = new URL(UNSPLASH_PROXY).origin;
  return rule[1].trim().split(/\s+/).some((source) => source === "*" || source.replace(/\/$/, "") === origin);
};

// The photo in use: a built-in one, one found and chosen before, or, where
// the browser keeps nothing (a private window), the one chosen this visit.
export function current(frame, core) {
  const { photo: id, recent } = core.storedTheme();
  const wanted = id ?? frame.chosen?.id;
  return [...BACKDROPS, ...recent, frame.chosen].find((p) => p && p.id === wanted) ?? BACKDROPS[0];
}

// The photo behind the page, and the panel's pressed photo and credit.
export function paint(frame, core) {
  const mode = frame.mode();
  // A photo found through the worker shows only where the page lets the
  // frame reach it; elsewhere (an app built before) the first built-in one.
  const chosen = current(frame, core);
  const shown = builtIn(chosen) || searchable() ? chosen : BACKDROPS[0];
  const root = document.documentElement;
  if (mode === "immersive") {
    root.style.setProperty("--prism-backdrop-color", shown.color);
    if (frame.backdrop !== shown.id) {
      frame.backdrop = shown.id;
      // Until it arrives, or if it cannot, the photo's own colour is the ground.
      pictureOf(shown)
        .then((url) => {
          if (frame.backdrop === shown.id) root.style.setProperty("--prism-backdrop", `url("${url}")`);
        })
        .catch(() => {
          // Not shown, so the next paint (a click, a reload) asks again.
          if (frame.backdrop === shown.id) frame.backdrop = null;
        });
    }
  }
  for (const choice of frame.shadowRoot.querySelectorAll(".mode")) choice.setAttribute("aria-pressed", String(choice.dataset.mode === mode));
  for (const choice of frame.shadowRoot.querySelectorAll("button.photo-pick")) choice.setAttribute("aria-pressed", String(choice.dataset.photo === shown.id));
  const credit = frame.$("credit-name");
  if (!credit) return;
  credit.textContent = shown.name;
  credit.href = `https://unsplash.com/@${shown.user}?${UNSPLASH}`;
}

// The panel's photos and search, wired to this render of the frame.
export function panel(frame, core) {
  for (const choice of frame.$("theme-panel").querySelectorAll(".mode")) {
    choice.addEventListener("click", () => frame.setTheme(choice.dataset.mode));
  }
  const grid = frame.$("photos");
  const note = frame.$("photos-note");
  const more = frame.$("photos-more");
  const find = frame.$("find");
  const input = frame.$("find-input");
  find.hidden = !searchable();
  let query = "";
  let page = 1;
  let asking = null;
  let timer = 0;

  // A photo's tile: its colour at once, the photo fading in over it.
  const tile = (photo) => {
    const pick = Object.assign(document.createElement("button"), { type: "button", className: "photo-pick" });
    pick.dataset.photo = photo.id;
    const words = `${photo.alt || "Photo"}, by ${photo.name}`;
    pick.setAttribute("aria-label", words);
    pick.title = words;
    pick.setAttribute("aria-pressed", String(photo.id === current(frame, core).id));
    pick.style.backgroundColor = photo.color;
    const img = Object.assign(new Image(240, 150), { alt: "", decoding: "async" });
    img.addEventListener("load", () => pick.classList.add("ready"), { once: true });
    pictureOf(photo, true).then((url) => (img.src = url)).catch(() => {});
    pick.append(img);
    pick.addEventListener("click", () => {
      frame.setTheme("immersive", photo);
      // Unsplash is told once a photo it found is put to use.
      if (!builtIn(photo)) fetch(`${UNSPLASH_PROXY}use?id=${encodeURIComponent(photo.id)}`, { method: "POST" }).catch(() => {});
    });
    return pick;
  };
  const unique = (photos) => photos.filter((photo, i) => photos.findIndex((p) => p.id === photo.id) === i);
  const say = (words) => {
    note.textContent = words;
    note.hidden = !words;
  };
  // Nine, as a search shows: the recent ones first, then the built-in
  // ones, never leaving out the photo in use.
  const home = () => {
    const chosen = frame.chosen && !builtIn(frame.chosen) ? [frame.chosen] : [];
    const all = unique([...chosen, ...core.storedTheme().recent, ...BACKDROPS]);
    const shown = all.slice(0, 9);
    const inUse = current(frame, core);
    if (!shown.some((p) => p.id === inUse.id)) shown[shown.length - 1] = inUse;
    grid.replaceChildren(...shown.map(tile));
    more.hidden = true;
    say("");
  };
  const skeleton = () =>
    grid.replaceChildren(...Array.from({ length: 9 }, () => {
      const blank = document.createElement("span");
      blank.className = "photo-pick blank";
      return blank;
    }));
  const settle = (control) => {
    if (asking !== control) return;
    find.classList.remove("busy");
    grid.setAttribute("aria-busy", "false");
  };
  async function ask(words, next = 0) {
    asking?.abort();
    const control = (asking = new AbortController());
    find.classList.add("busy");
    grid.setAttribute("aria-busy", "true");
    if (!next) {
      skeleton();
      more.hidden = true;
      say("");
    }
    let status = 0;
    try {
      const response = await fetch(`${UNSPLASH_PROXY}search?${new URLSearchParams({ q: words, page: String(next || 1) })}`, { signal: control.signal });
      status = response.status;
      const found = await response.json();
      if (!response.ok) throw new Error(found.error ?? `HTTP ${status}`);
      const photos = (Array.isArray(found.photos) ? found.photos : []).map(core.photoOf).filter(Boolean);
      page = found.page ?? 1;
      if (next) grid.append(...photos.map(tile));
      else grid.replaceChildren(...photos.map(tile));
      more.hidden = !found.more || !photos.length;
      if (!next && !photos.length) say(`No photos for “${words}”`);
    } catch (error) {
      if (error.name === "AbortError") return;
      if (!next) home();
      say(status === 429 ? "Unsplash needs a short rest. Try again soon." : "Search isn’t available right now.");
    } finally {
      settle(control);
    }
  }

  input.addEventListener("input", () => {
    clearTimeout(timer);
    const words = input.value.trim();
    if (words === query) return;
    query = words;
    if (!words) {
      asking?.abort();
      settle(asking);
      home();
      return;
    }
    timer = setTimeout(() => ask(words), 280);
  });
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      clearTimeout(timer);
      if (query) ask(query);
    } else if (event.key === "Escape" && input.value) {
      // The first Escape clears the search; the next closes the panel.
      event.preventDefault();
      event.stopPropagation();
      input.value = "";
      input.dispatchEvent(new Event("input"));
    }
  });
  more.addEventListener("click", () => query && ask(query, page + 1));
  return { open: () => query || home() };
}

// Prism ids (prism/identity/4, identity/PROTOCOL.md). A visitor's Prism id
// is stitch's Prism id (afflom/stitch, pinned in identity/stitch.lock): the
// κ of its genesis, held by devices whose keys never leave them. Each app's
// origin keeps its own device of the id, made here with no prompt; another
// device or app joins the same id by stitch's link offer, confirmed with a
// six-digit code, and a recovery kit brings it back after every device is
// lost. Devices meet on stitch's substrate (its relay), so nothing here
// needs a server of Prism's own.
//
// Signing in is the device signing one action for this app,
// "prism/session/1", which lets a session key made here act for the id on
// this origin until it expires. verifySession checks that action against the
// id's log with Prism id's own verifyAction, here and for anyone the app
// shows a signature to. A failed check is an IdentityError naming which.
const SESSION_ACTION = "prism/session/1";
const MAX_SESSION_MS = 8 * 24 * 60 * 60 * 1000;
const b64u = (bytes) => btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");

export class IdentityError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "IdentityError";
    this.code = code;
  }
}
const deny = (code, message) => {
  throw new IdentityError(code, message);
};

// Prism id itself (identity/prism-id.mjs), published beside the frame and
// named by its SHA-256, which frame/publish.mjs writes here. It is fetched
// when the visitor reaches for the account icon, and checked as the frame
// checks this module. Unpublished (tests, a checkout), it is read from the
// repository.
const PRISM_ID = "/*PRISM_ID*/";
let prismModule = null;
export const prismId = () =>
  (prismModule ??= (async () => {
    if (!/^[0-9a-f]{64}$/.test(PRISM_ID)) return import("../identity/prism-id.mjs");
    const bytes = await (await fetch(new URL(`${PRISM_ID}/prism-id.mjs`, FRAME_HOST))).arrayBuffer();
    const hex = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)), (b) => b.toString(16).padStart(2, "0")).join("");
    if (hex !== PRISM_ID) throw new Error("Prism id is not the module this frame names");
    const url = URL.createObjectURL(new Blob([bytes], { type: "text/javascript" }));
    try {
      return await import(url);
    } finally {
      URL.revokeObjectURL(url);
    }
  })().catch((error) => {
    prismModule = null;
    throw error;
  }));

// A session: { app, envelope, records }. The envelope is the id's
// "prism/session/1" action for `app`, naming the session key and its expiry;
// records are the id's log as the signing device knew it. `fresh` is the
// window in which it must be checked after signing (Prism id's actionFresh);
// anyone checking a kept session later checks it as of its own signing
// (fresh: 0), and its expiry bounds it.
export async function verifySession(session, { now = Date.now(), fresh = 60000, hear = true } = {}) {
  const Prism = await prismId();
  const { app, envelope, records } = session ?? {};
  const body = envelope?.body;
  if (typeof app !== "string" || !/^https?:\/\/[^/]+$/.test(app)) deny("app", "The session names no app");
  if (body?.action !== SESSION_ACTION) deny("action", "The signature is not a session");
  const { sessionKey, expires } = body.payload ?? {};
  if (typeof sessionKey !== "string" || !/^[\w-]{87}$/.test(sessionKey)) deny("key", "The session key is malformed");
  if (!Number.isSafeInteger(expires) || expires <= now) deny("expired", "The session has expired");
  if (expires > body.at + MAX_SESSION_MS) deny("expired", "The session lasts too long");
  let log;
  try {
    log = await Prism.logFromRecords(records);
  } catch (error) {
    if (error instanceof Prism.PrismIdError) deny("log", "The log does not parse");
    throw error;
  }
  const at = fresh ? now : body.at;
  let verdict;
  try {
    // With the relay known, also hear removals the signing device did not
    // present (the id's public log). `hear: false` is for the device that
    // just made the id: nothing about it can be on the relay yet.
    verdict = await Prism.verifyAction(body.prism, log, envelope, { audience: app, now: at, window: fresh || 1, ...(relay && hear ? { bus: bus(Prism) } : {}) });
  } catch (error) {
    if (error instanceof Prism.PrismIdError) deny("log", `The log does not resolve: ${error.message}`);
    throw error;
  }
  if (!verdict.allowed) deny(verdict.reason === "scope" ? "scope" : "signature", `The session was refused (${verdict.reason})`);
  return { account: body.prism, key: envelope.signer, sessionKey, expires };
}

// ---------------------------------------------------------------- this origin's device

// The device and its view of the id, in this origin's IndexedDB. IndexedDB
// keeps CryptoKeys as they are: a non-extractable key stays one.
function deviceStore(mode, work) {
  return new Promise((done, fail) => {
    const open = indexedDB.open("prism-id", 1);
    open.onupgradeneeded = () => open.result.createObjectStore("prism");
    open.onerror = () => fail(open.error);
    open.onsuccess = () => {
      const tx = open.result.transaction("prism", mode);
      const request = work(tx.objectStore("prism"));
      tx.oncomplete = () => (open.result.close(), done(request?.result));
      tx.onerror = () => (open.result.close(), fail(tx.error));
    };
  });
}
export const loadDevice = () => deviceStore("readonly", (s) => s.get("self")).catch(() => null);
const saveDevice = (held, workspace) => deviceStore("readwrite", (s) => (workspace && s.put(workspace, "workspace"), s.put(held, "self")));
const forgetDevice = () => deviceStore("readwrite", (s) => s.delete("self"));
const loadKit = () => deviceStore("readonly", (s) => s.get("kit")).catch(() => null);
const saveKit = (kit) => deviceStore("readwrite", (s) => s.put(kit, "kit"));
const forgetKit = () => deviceStore("readwrite", (s) => s.delete("kit"));
// Shares this device keeps as someone's guardian, each wrapped to it alone.
const loadGuarding = () => deviceStore("readonly", (s) => s.get("guarding")).then((list) => list ?? [], () => []);
const saveGuarding = (list) => deviceStore("readwrite", (s) => s.put(list, "guarding"));
// The recovery device this device gave a guardian, so the panel stops offering one.
const loadGuarded = () => deviceStore("readonly", (s) => s.get("guarded")).catch(() => null);
const saveGuarded = (recovery) => deviceStore("readwrite", (s) => s.put(recovery, "guarded"));
// An app's workspace key: 32 random bytes sealed under the id's private
// boundary key, so only a device of the id that made it opens it. The app
// seals its own records with the key; the frame keeps only the sealed key.
const loadWorkspace = () => deviceStore("readonly", (s) => s.get("workspace")).catch(() => null);
export async function openWorkspace(Prism, held, kept) {
  if (kept && kept.prism !== held.prism) deny("removed", "This device was removed from the Prism ID that keeps this work");
  const value = kept ? await Prism.openPrivate(held, kept.sealed) : { key: Prism.toBase64Url(crypto.getRandomValues(new Uint8Array(32))) };
  const boundary = await Prism.currentBoundary(held).catch(() => (kept ? null : deny("stale-key", "This device is waiting for your Prism ID's newest key")));
  const record = boundary && kept?.sealed.key !== boundary.keyId ? { prism: held.prism, sealed: (await Prism.sealPrivate(held, value)).sealed } : null;
  return { key: Prism.fromBase64Url(value.key), record };
}
// An app's pages on every device of one id meet on a relay topic only those
// devices derive, from the id's private boundary key and the app's origin; a
// removed device holds no current key, so it is not among them (FR-05).
export async function appTopic(Prism, held, origin) {
  const { keyId } = await Prism.currentBoundary(held);
  const mac = await crypto.subtle.sign("HMAC", held.keyring.get(keyId).topic, new TextEncoder().encode(`prism/app-channel/1 ${origin}`));
  return "prism-app:" + Array.from(new Uint8Array(mac), (b) => b.toString(16).padStart(2, "0")).join("");
}
// Joining or recovering an id carries what the id this browser had kept; a
// removed device makes a new id and carries nothing, so its work stays locked.
export const carryWorkspace = async (Prism, kept, from, to) =>
  kept && from && kept.prism === from.prism ? { prism: to.prism, sealed: (await Prism.sealPrivate(to, await Prism.openPrivate(from, kept.sealed))).sealed } : null;

// The bus to the id's other devices: stitch's relay (the frame's
// PRISM_RELAY), which the page's connect-src must name; the page build reads
// it from the frame. A page that does not admit it keeps the id on this
// device: the id, its session and recovery work, linking and syncing wait.
let relay = null;
let relayBus = null;
const relayAdmitted = (origin) => {
  const policy = [...document.querySelectorAll('meta[http-equiv="Content-Security-Policy" i]')].map((meta) => meta.content).join(";");
  const connect = /(?:^|;)\s*connect-src([^;]*)/.exec(policy)?.[1] ?? /(?:^|;)\s*default-src([^;]*)/.exec(policy)?.[1];
  return connect === undefined || connect.includes(origin);
};
const bus = (Prism) => (relayBus ??= Prism.createRelayBus({ relay: relay && relayAdmitted(new URL(relay).origin) ? relay : null }));

// Share this device's log with the id's other devices, under every boundary
// key it holds, so a device that has not taken up a rotation still hears,
// and publish its public records where apps hear removals.
async function share(Prism, held) {
  for (const keyId of held.keyring.keys()) await Prism.publishSync(held, bus(Prism), { keyId }).catch(() => {});
  await Prism.publishLog(held, bus(Prism)).catch(() => {});
}
// Take in what the other devices shared. A message that does not open or
// resolve is someone else's or stale: it is skipped, never trusted.
async function pull(Prism, held) {
  for (const keyId of [...held.keyring.keys()]) {
    const topic = await Prism.syncTopic(held, keyId).catch(() => null);
    if (!topic) continue;
    for (const bytes of await bus(Prism).retained(topic).catch(() => [])) {
      try {
        held = await Prism.receiveSync(held, bytes);
      } catch (error) {
        if (!(error instanceof Prism.PrismIdError) && !(error instanceof DOMException) && !(error instanceof SyntaxError)) throw error;
      }
    }
  }
  return held;
}
// The id's log as apps see it: this device's, with what the public log holds.
// `own`: the log is this device's, so a heard record that would fork it (an
// old recovery kit replayed) is refused, as stitch's admitRecords does.
const heard = (Prism, held, own = true) => Prism.heardLog(held.prism, held.log, bus(Prism), { own }).catch(() => held.log);

// A recovery kit: the paper share of the id's recovery device (stitch's
// threshold recovery, one share of one) and the id's log. It is as secret as
// the id: whoever holds it can take the id over.
const KIT = "prism/recovery-kit/1";
const ONE_PAPER_SHARE = { threshold: 1, guardians: [{ paper: true }] };
const kitOf = (held, share) => ({ kind: KIT, prism: held.prism, share, records: [...held.log.values()] });

// A guardian: a device of someone else's Prism id that keeps one share of
// this id's recovery device (stitch's replaceRecovery, two shares of two:
// the paper kit and the guardian's). The kit alone then recovers nothing,
// and the guardian, who releases its share once and only after talking to
// the person, is the witness a copied kit never gets past. The two devices
// meet on a rendezvous topic named by a secret in the link one gives the
// other; the relay sees neither the secret nor what passes.
const GUARDIAN = "prism-guardian:";
const HELP = "prism-help:";
const MSG = {
  offer: "prism/guardian-offer/1",
  share: "prism/guardian-share/1",
  kept: "prism/guardian-kept/1",
  help: "prism/guardian-help/1",
  release: "prism/guardian-release/1",
};
const TWO_OF_PAPER_AND = (guardian) => ({ threshold: 2, guardians: [{ paper: true }, guardian] });
const WAIT_MS = 5 * 60000;
const newSecret = () => b64u(crypto.getRandomValues(new Uint8Array(32)));
const secretOf = (uri, scheme) => {
  const secret = uri.trim().startsWith(scheme) ? uri.trim().slice(scheme.length) : "";
  if (!/^[\w-]{43}$/.test(secret)) deny("link", "That is not the link they sent");
  return secret;
};
// The first message of `kind` on a topic, or a refusal once `ms` pass.
// Stopped (the person went back), it never settles.
function awaitOn(Prism, topic, kind, accept = () => true, ms = WAIT_MS) {
  let unsubscribe = () => {};
  let timer = 0;
  const stop = () => (clearTimeout(timer), unsubscribe());
  const found = new Promise((done, failed) => {
    unsubscribe = bus(Prism).subscribe(topic, (bytes) => {
      const message = parse(bytes);
      if (message?.kind === kind && accept(message)) done(message);
    });
    timer = setTimeout(() => failed(new IdentityError("wait", "The other device did not answer in time. Try again")), ms);
  });
  return { found: found.finally(stop), stop };
}

// A QR code as one SVG path, with its four-module quiet zone.
function drawQr(svg, { size, modules }) {
  let d = "";
  modules.forEach((row, y) => row.forEach((dark, x) => dark && (d += `M${x + 4} ${y + 4}h1v1h-1z`)));
  svg.setAttribute("viewBox", `0 0 ${size + 8} ${size + 8}`);
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", d);
  svg.replaceChildren(path);
}

const json = (value) => new TextEncoder().encode(JSON.stringify(value));
// A relay message as JSON, or null when it is not.
const parse = (bytes) => {
  try {
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    return null;
  }
};
const short = (id) => `${id.slice(0, 15)}…${id.slice(-4)}`;
const sentence = (text) => (text ? text[0].toUpperCase() + text.slice(1) + (/[.!?]$/.test(text) ? "" : ".") : text);
const KIND_LABEL = { browser: "Browser", native: "App", mobile: "Phone", embedded: "Device", recovery: "Recovery kit" };

// Lucide's outline icons, drawn at the text colour, so the panel reads as the kit does everywhere else.
const glyph = (d, cls = "") => `<svg class="glyph ${cls}" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${d}</svg>`;
const [USER, LOCK, COPY, SHIELD, CHEVRON, CHECK, BACK] = ['<circle cx="12" cy="12" r="10"/><circle cx="12" cy="10" r="4"/><path d="M18 20a6 6 0 0 0-12 0"/>', '<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>', '<rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>', '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>', '<path d="m9 18 6-6-6-6"/>', '<path d="M20 6 9 17l-5-5"/>', '<path d="m15 18-6-6 6-6"/>'];
const [LAPTOP, PLUS, LOGIN, USERS, BUOY, HEART, KEY] = ['<path d="M20 16V7a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9m16 0H4m16 0 1.28 2.55a1 1 0 0 1-.9 1.45H3.62a1 1 0 0 1-.9-1.45L4 16"/>', '<path d="M5 12h14M12 5v14"/>', '<path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M15 12H3"/>', '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>', '<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><path d="m4.93 4.93 4.24 4.24m5.66 0 4.24-4.24m-4.24 9.9 4.24 4.24m-9.9-4.24-4.24 4.24"/>', '<path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/>', '<path d="M2.59 17.41A2 2 0 0 0 2 18.83V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.17a2 2 0 0 0 1.41-.59l.82-.81a6.5 6.5 0 1 0-4-4z"/><circle cx="16.5" cy="7.5" r=".5"/>'];
// One choice: what it does, and in a line, why you would want it.
const row = (id, icon, what, why, hidden = false) => `<button id="account-${id}" class="row" type="button"${hidden ? " hidden" : ""}>${glyph(icon)}<span><b>${what}</b><small>${why}</small></span>${glyph(CHEVRON, "chevron")}</button>`;

const ACCOUNT_PANEL = `
<button class="account-close" type="button" popovertarget="account-panel" popovertargetaction="hide" aria-label="Close"><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false"><path d="M6 6l12 12M18 6 6 18" /></svg></button>
<h2 id="account-title" class="account-title">Prism ID</h2>
<p id="account-sub" class="account-sub"></p>
<div id="account-view-held" class="account-in" hidden>
  <div class="account-card">${glyph(USER, "account-avatar")}<span class="account-who"><code id="account-id"></code><small>${glyph(LOCK)}<span id="account-held"></span></small></span><button id="account-copy" class="account-icon" type="button" aria-label="Copy your Prism ID" title="Copy">${glyph(COPY)}</button></div>
  <div id="account-kit" class="account-kit" hidden>${glyph(SHIELD)}<span><b>Save your recovery kit</b><small>If you lose every device, it brings your ID back. Whoever holds it can take your ID, so keep it offline.</small><button id="account-kit-save" class="primary" type="button">Save recovery kit</button></span></div>
  <p class="account-group">Devices</p>
  ${row("devices", LAPTOP, "Your devices", "See where your ID is, or remove a device")}
  ${row("link", PLUS, "Add a device", "A phone, a computer or another Prism app")}
  ${row("join", LOGIN, "Use an ID you already have", "Bring your Prism ID here from another device")}
  <p class="account-group">Recovery</p>
  ${row("guard", USERS, "Add a guardian", "Someone you trust, who helps you recover", true)}
  ${row("recover", BUOY, "Recover with a kit", "Lost your devices? Start here")}
  ${row("guardian", HEART, "Be someone's guardian", "Keep a share of a friend's recovery")}
  ${row("help", KEY, "Help someone recover", "Release your share to a person you know", true)}
</div>
<div id="account-view-offer" class="account-in" hidden>
  <p>On a phone, scan this code. On a computer or in another Prism app, open Prism ID, choose <b>Use an ID you already have</b> and paste the link. It works once, for five minutes.</p>
  <svg id="account-qr" class="account-qr" role="img" aria-label="QR code of the link"></svg>
  <p class="account-id"><code id="account-offer"></code><button id="account-offer-copy" class="text" type="button">Copy</button></p>
  <label id="account-confirm" hidden>Type the code the new device shows <input id="account-code" inputmode="numeric" autocomplete="one-time-code" maxlength="6" pattern="[0-9]{6}" /></label>
  <button id="account-accept" class="primary" type="button" hidden>Add it</button>
</div>
<div id="account-view-join" class="account-in" hidden>
  <p>On a device that has your Prism ID, open Prism ID, choose <b>Add a device</b> and paste its link here.</p>
  <input id="account-uri" type="text" autocomplete="off" autocapitalize="none" spellcheck="false" aria-label="Link from your other device" />
  <button id="account-join-go" class="primary" type="button">Use this ID</button>
  <p id="account-join-code" class="account-code" hidden></p>
</div>
<div id="account-view-recover" class="account-in" hidden>
  <p>Choose the recovery kit you saved. This browser becomes your Prism ID's device, every other device is removed, and you get a new kit to save.</p>
  <input id="account-kit-file" type="file" accept=".json,application/json" aria-label="Recovery kit" />
  <div id="account-ask" hidden>
    <p>This kit also needs your guardian. Send them this link; they open Prism ID in any Prism app, choose <b>Help someone recover</b> and paste it. Then read them this code: <b id="account-ask-code"></b></p>
    <p class="account-id"><code id="account-ask-link"></code><button id="account-ask-copy" class="text" type="button">Copy</button></p>
  </div>
</div>
<div id="account-view-guard" class="account-in" hidden>
  <p>Ask someone you trust to open Prism ID in any Prism app, choose <b>Be someone's guardian</b>, and send you the link it shows. Paste it here.</p>
  <input id="account-guard-uri" type="text" autocomplete="off" autocapitalize="none" spellcheck="false" aria-label="Link from your guardian" />
  <label id="account-guard-kit-label" hidden>Your recovery kit <input id="account-guard-kit" type="file" accept=".json,application/json" /></label>
  <button id="account-guard-go" class="primary" type="button">Add guardian</button>
</div>
<div id="account-view-guardian" class="account-in" hidden>
  <p>Send this link to the person. Keep this open until it says you are their guardian. If they ever lose every device, they will ask you to help.</p>
  <p class="account-id"><code id="account-guardian-link"></code><button id="account-guardian-copy" class="text" type="button">Copy</button></p>
</div>
<div id="account-view-help" class="account-in" hidden>
  <p>Paste the link the person sent you.</p>
  <input id="account-help-uri" type="text" autocomplete="off" autocapitalize="none" spellcheck="false" aria-label="Link from the person" />
  <button id="account-help-go" class="primary" type="button">Find their request</button>
  <div id="account-help-check" hidden>
    <p>Release your share only after you have talked to them, by voice or in person, and they read you this code: <b id="account-help-code"></b></p>
    <button id="account-help-release" class="primary" type="button">Release my share</button>
  </div>
</div>
<div id="account-view-devices" class="account-in" hidden>
  <ul id="account-device-list" class="account-devices"></ul>
</div>
<p id="account-error" class="account-error" role="alert" hidden></p>
<button id="account-back" class="row" type="button" hidden>${glyph(BACK)}<span>Back</span></button>
`;

const TITLES = {
  held: ["Prism ID", "Every Prism app knows you by this ID. There is no password to remember and no account to make."],
  offer: ["Add a device", ""],
  join: ["Use an ID you already have", ""],
  recover: ["Recover with a kit", ""],
  devices: ["Your devices", "A device you remove can no longer act as you once it syncs."],
  guard: ["Add a guardian", "Your new kit then needs them too, so a lost or copied kit alone cannot bring your ID back."],
  guardian: ["Be someone's guardian", ""],
  help: ["Help someone recover", ""],
};

// The account panel, drawn in the frame. Every step is stitch's Prism id
// running in this page; the relay only carries what the devices exchange.
export function accounts(frame, core) {
  const { SESSION_DAYS, store, accountChannel, loadSession } = core;
  relay = (() => {
    try {
      return localStorage.getItem("uor.studio.enablePrismSync") === "1" ? (core.PRISM_RELAY ?? null) : null;
    } catch {
      return null;
    }
  })();
  frame.$("account-panel").innerHTML = ACCOUNT_PANEL;
  const $ = (id) => frame.$(`account-${id}`);
  const panel = frame.$("account-panel");
  let held = null;
  let view = "held";
  let kit = null;
  let link = null;
  let guarding = [];
  let guarded = false;
  let working = false;

  const say = (words) => {
    frame.$("bar-status").textContent = words;
    setTimeout(() => (frame.$("bar-status").textContent = ""), 1600);
  };
  const changed = () => {
    paint();
    frame.dispatchEvent(new CustomEvent("accountchange", { detail: frame.account, bubbles: true, composed: true }));
  };
  function paint() {
    core.paint();
    for (const name of ["held", "offer", "join", "recover", "devices", "guard", "guardian", "help"]) $(`view-${name}`).hidden = name !== view;
    const [title, sub] = TITLES[view];
    $("title").textContent = title;
    $("sub").textContent = sub;
    $("sub").hidden = !sub;
    $("kit").hidden = !kit;
    for (const name of ["link", "devices", "guardian"]) $(name).hidden = !held;
    // A kit of one paper share can gain a guardian; one that has one cannot change it without them.
    $("guard").hidden = !held || guarded;
    $("help").hidden = !held || !guarding.length;
    $("back").hidden = view === "held";
    $("id").textContent = held ? short(held.prism) : "Making your Prism ID";
    $("id").title = held?.prism ?? "";
    $("held").textContent = working ? "Your ID and your work, sealed on your devices" : "Held only on your devices";
  }
  const show = (next) => {
    view = next;
    $("error").hidden = true;
    paint();
  };
  const fail = (words) => {
    $("error").textContent = words;
    $("error").hidden = false;
  };
  const run = async (task) => {
    panel.classList.add("busy");
    frame.$("account").classList.add("busy");
    $("error").hidden = true;
    try {
      const Prism = await prismId();
      try {
        await task(Prism);
      } catch (error) {
        if (error instanceof Prism.PrismIdError || error instanceof IdentityError) fail(sentence(error.message));
        else throw error;
      }
    } catch {
      fail("That did not work. Try again.");
    } finally {
      panel.classList.remove("busy");
      frame.$("account").classList.remove("busy");
    }
  };
  const keep = async (Prism, next, workspace) => {
    held = next;
    await saveDevice(held, workspace);
    await share(Prism, held);
  };

  // This app's session, made without asking: take in what the other
  // devices shared first, so a device removed elsewhere stops here (it then
  // forgets the device and makes a new id), then sign this app's session.
  // An id made this moment has no other device and nothing to hear, so it
  // asks the relay nothing.
  const session = async (Prism, { fresh = false } = {}) => {
    if (!fresh) held = await pull(Prism, held);
    await saveDevice(held);
    if (!fresh && !(await Prism.canSign(held.prism, await heard(Prism, held), held.device.id))) {
      await forgetDevice();
      await forgetKit();
      held = null;
      kit = null;
      return ensure(Prism);
    }
    const keys = await crypto.subtle.generateKey({ name: "ECDSA", namedCurve: "P-256" }, false, ["sign", "verify"]);
    const sessionKey = b64u(await crypto.subtle.exportKey("raw", keys.publicKey));
    const expires = Date.now() + SESSION_DAYS * 864e5;
    const envelope = await Prism.signAction(held, SESSION_ACTION, { sessionKey, expires }, { now: Date.now(), audience: location.origin });
    const delegation = { app: location.origin, envelope, records: [...held.log.values()] };
    const who = await verifySession(delegation, { hear: !fresh });
    const saved = { ...who, delegation, privateKey: keys.privateKey };
    await store("readwrite", (s) => s.put(saved, "current"));
    accountChannel.session = saved;
    changed();
  };

  // The recovery kit of a new id or a recovery waits in this origin's
  // storage until it is saved; the panel offers it, nothing asks for it.
  const holdKit = async (share) => {
    kit = kitOf(held, share);
    await saveKit(kit);
  };

  // ---- the id itself: made on first use, with nothing to click. Every
  // app keeps its own device; linking joins them into one id.
  let ensuring = null;
  async function ensure(Prism) {
    held ??= await loadDevice();
    kit ??= await loadKit();
    guarding = await loadGuarding();
    if (held) guarded = (await loadGuarded()) === (await Prism.resolvePrism(held.prism, held.log).catch(() => null))?.recovery;
    // A new id stays on this device until it links or is recovered: there
    // is no other device to share with, so nothing goes to the relay.
    let fresh = false;
    if (!held) {
      const made = await Prism.createRecovery(await Prism.createPrism({ deviceClass: "browser", protection: "software" }), ONE_PAPER_SHARE);
      held = made.session;
      await saveDevice(held);
      await holdKit(made.shares[0]);
      fresh = true;
      checked = true; // nothing elsewhere can have removed a device made this moment
    }
    await loadSession();
    if (accountChannel.session) await stillHeld(Prism);
    if (!accountChannel.session) await session(Prism, { fresh });
    paint();
  }
  // Once per page: a device removed elsewhere drops its session here.
  let checked = false;
  async function stillHeld(Prism) {
    if (checked) return;
    checked = true;
    held = await pull(Prism, held);
    await saveDevice(held);
    if (await Prism.canSign(held.prism, await heard(Prism, held), held.device.id)) return;
    await store("readwrite", (s) => s.delete("current")).catch(() => {});
    accountChannel.session = null;
    changed();
  }
  const quietly = () => (ensuring ??= prismId().then(ensure).catch(() => {}).finally(() => (ensuring = null)));

  $("kit-save").addEventListener("click", async () => {
    if (!kit) return;
    const a = Object.assign(document.createElement("a"), { href: URL.createObjectURL(new Blob([JSON.stringify(kit, null, 1)], { type: "application/json" })), download: "prism-id-recovery-kit.json" });
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    kit = null;
    await forgetKit().catch(() => {});
    paint();
    say("Recovery kit saved");
  });

  // ---- this device offers a link; the new one answers on the rendezvous topic.
  const endLink = () => {
    link?.stop?.();
    link = null;
    for (const id of ["ask", "help-check", "confirm", "accept", "join-code"]) $(id).hidden = true;
    $("code").value = "";
  };
  $("link").addEventListener("click", () =>
    run(async (Prism) => {
      endLink();
      held = await pull(Prism, held);
      const { uri, pending } = await Prism.createLinkOffer(held, { now: Date.now() });
      const topic = await Prism.rendezvousTopic(pending.offer.secret);
      link = { pending, topic, request: null };
      link.stop = bus(Prism).subscribe(topic, (bytes) => {
        const request = parse(bytes);
        if (request?.kind !== Prism.KIND.request || !link || link.request) return;
        link.request = request;
        $("confirm").hidden = $("accept").hidden = false;
        $("code").focus();
      });
      $("offer").textContent = uri;
      $("offer").title = uri;
      drawQr($("qr"), Prism.linkQr(uri));
      show("offer");
    }),
  );
  $("offer-copy").addEventListener("click", () => copy($("offer-copy"), $("offer").textContent));
  $("accept").addEventListener("click", () =>
    run(async (Prism) => {
      if (!link?.request) return;
      const accepted = await Prism.acceptLinkRequest(held, link.pending, link.request, { now: Date.now(), confirmCode: $("code").value.trim() });
      await bus(Prism).publish(link.topic, json(accepted.response));
      await keep(Prism, accepted.session);
      endLink();
      show("held");
      panel.hidePopover();
      say("Device added");
    }),
  );

  $("join-go").addEventListener("click", () =>
    run(async (Prism) => {
      endLink();
      const offer = Prism.parseLinkUri($("uri").value.trim());
      if (offer.expires <= Date.now()) deny("offer", "That link has expired. Make a new one");
      const device = await Prism.createDevice({ deviceClass: "browser", protection: "software" });
      const topic = await Prism.rendezvousTopic(offer.secret);
      const joined = new Promise((done, failed) => {
        link = { topic };
        link.stop = bus(Prism).subscribe(topic, (bytes) => {
          const response = parse(bytes);
          if (response?.kind !== Prism.KIND.response) return;
          Prism.completeLink(device, offer, response).then(done, failed);
        });
        setTimeout(() => failed(new IdentityError("offer", "The other device did not answer. Make a new link and try again")), Math.max(0, offer.expires - Date.now()));
      });
      await bus(Prism).publish(topic, json(await Prism.requestLink(device, offer)));
      $("join-code").textContent = `Type ${await Prism.linkCode(offer.secret, device.record)} on your other device.`;
      $("join-code").hidden = false;
      panel.classList.remove("busy");
      try {
        const next = await joined;
        await keep(Prism, next, await carryWorkspace(Prism, await loadWorkspace(), held, next));
      } finally {
        endLink();
      }
      $("uri").value = "";
      kit = null;
      await forgetKit();
      await session(Prism);
      show("held");
      say("This app uses your Prism ID");
    }),
  );

  // ---- recovery: the kit's recovery device binds this one, removes every
  // other device and makes a new kit, which retires the old one.
  $("kit-file").addEventListener("change", () =>
    run(async (Prism) => {
      const file = $("kit-file").files[0];
      $("kit-file").value = "";
      let saved;
      try {
        saved = JSON.parse(await file.text());
      } catch {
        deny("kit", "That is not a recovery kit");
      }
      if (saved?.kind !== KIT || !Array.isArray(saved.records)) deny("kit", "That is not a recovery kit");
      const device = await Prism.createDevice({ deviceClass: "browser", protection: "software" });
      const shares = [saved.share];
      if (saved.share?.threshold === 2) {
        // A guarded kit: the guardian releases their share to this device.
        endLink();
        const secret = newSecret();
        const topic = await Prism.rendezvousTopic(secret);
        const waiting = awaitOn(Prism, topic, MSG.release, (m) => m.share?.guardian === device.id && m.share?.recovery === saved.share.recovery);
        link = { stop: waiting.stop };
        await bus(Prism).publish(topic, json({ kind: MSG.help, prism: saved.prism, recovery: saved.share.recovery, device: device.record }));
        $("ask-link").textContent = HELP + secret;
        $("ask-code").textContent = await Prism.recoveryCode(saved.prism, device.record);
        $("ask").hidden = false;
        panel.classList.remove("busy");
        shares.push((await waiting.found).share);
        endLink();
      }
      const recovered = await Prism.recoverPrism({ prism: saved.prism, records: saved.records, shares, device, rotateRecovery: ONE_PAPER_SHARE });
      // The kit's log may be older than the id's: what the other devices
      // shared since, and the id's public log, show a kit already used. Its
      // retirement never kept what this replay signed, so this device cannot
      // sign, or the two retirements fork the log. Either way nothing is
      // kept or published.
      const synced = await pull(Prism, recovered.session);
      const log = await heard(Prism, synced, false);
      const used = () => deny("kit", "That recovery kit was already used, and replaced by a newer one");
      const state = await Prism.resolvePrism(synced.prism, log).catch((error) => (error?.code === "fork" ? used() : Promise.reject(error)));
      const newer = state.active.filter((id) => state.roles[id] === "recovery" && id !== recovered.shares[0].recovery);
      if (newer.length || !(await Prism.canSign(synced.prism, log, synced.device.id))) used();
      await keep(Prism, synced, await carryWorkspace(Prism, await loadWorkspace(), held, synced));
      await holdKit(recovered.shares[0]);
      await store("readwrite", (s) => s.delete("current")).catch(() => {});
      accountChannel.session = null;
      await session(Prism);
      show("held");
      say("Prism ID recovered");
    }),
  );

  // ---- the id's devices, from its log; Remove is Prism id's revokeDevice,
  // which rotates the private boundary key.
  const listDevices = async (Prism) => {
    const state = await Prism.resolvePrism(held.prism, held.log);
    $("device-list").replaceChildren(
      ...state.active.map((id) => {
        const item = document.createElement("li");
        const record = state.devices[id];
        const label = document.createElement("span");
        const code = Object.assign(document.createElement("code"), { textContent: id.slice(7, 15), title: id });
        label.append(`${KIND_LABEL[record.class] ?? "Device"} `, code, id === held.device.id ? " (this one)" : "");
        item.append(label);
        if (id !== held.device.id && state.roles[id] !== "recovery") {
          const remove = Object.assign(document.createElement("button"), { type: "button", className: "text", textContent: "Remove" });
          remove.addEventListener("click", () =>
            run(async (P) => {
              await keep(P, await P.revokeDevice(await pull(P, held), id));
              await listDevices(P);
            }),
          );
          item.append(remove);
        }
        return item;
      }),
    );
  };
  $("devices").addEventListener("click", () =>
    run(async (Prism) => {
      held = await pull(Prism, held);
      await saveDevice(held);
      await listDevices(Prism);
      show("devices");
    }),
  );

  $("join").addEventListener("click", () => (show("join"), $("uri").focus()));
  $("recover").addEventListener("click", () => show("recover"));
  $("ask-copy").addEventListener("click", () => copy($("ask-copy"), $("ask-link").textContent));

  // ---- a guardian for this id: the guardian's device offers itself on the
  // topic, this device moves the recovery to two shares of two and hands the
  // guardian theirs, and keeps the change only once the guardian kept it.
  $("guard").addEventListener("click", () => {
    $("guard-kit-label").hidden = !!kit;
    show("guard");
    $("guard-uri").focus();
  });
  $("guard-go").addEventListener("click", () =>
    run(async (Prism) => {
      endLink();
      const secret = secretOf($("guard-uri").value, GUARDIAN);
      let current = kit;
      if (!current) {
        const file = $("guard-kit").files[0];
        if (!file) deny("kit", "Choose your recovery kit too");
        try {
          current = JSON.parse(await file.text());
        } catch {
          deny("kit", "That is not a recovery kit");
        }
      }
      if (current?.kind !== KIT || current.prism !== held.prism || current.share?.threshold !== 1) deny("kit", "That is not this id's recovery kit");
      const topic = await Prism.rendezvousTopic(secret);
      const offered = awaitOn(Prism, topic, MSG.offer);
      link = { stop: offered.stop };
      const guardian = (await offered.found).device;
      Prism.checkDeviceRecord(guardian);
      held = await pull(Prism, held);
      const replaced = await Prism.replaceRecovery(held, [current.share], TWO_OF_PAPER_AND(guardian));
      const share = replaced.shares.find((s) => s.guardian !== "paper");
      const kept = awaitOn(Prism, topic, MSG.kept, (m) => m.recovery === replaced.recovery);
      link = { stop: kept.stop };
      await bus(Prism).publish(topic, json({ kind: MSG.share, share }));
      await kept.found;
      endLink();
      await keep(Prism, replaced.session);
      await saveGuarded(replaced.recovery);
      guarded = true;
      await holdKit(replaced.shares.find((s) => s.guardian === "paper"));
      $("guard-uri").value = "";
      show("held");
      say("Guardian added. Save your new recovery kit");
    }),
  );

  // ---- being a guardian: offer this device, keep the share that comes back.
  $("guardian").addEventListener("click", () =>
    run(async (Prism) => {
      endLink();
      const secret = newSecret();
      const topic = await Prism.rendezvousTopic(secret);
      const shared = awaitOn(Prism, topic, MSG.share, (m) => m.share?.kind === Prism.KIND.recoveryShare && m.share.guardian === held.device.id);
      link = { stop: shared.stop };
      await bus(Prism).publish(topic, json({ kind: MSG.offer, device: held.device.record }));
      $("guardian-link").textContent = GUARDIAN + secret;
      show("guardian");
      panel.classList.remove("busy");
      const { share } = await shared.found;
      guarding = [...(await loadGuarding()).filter((s) => s.prism !== share.prism), share];
      await saveGuarding(guarding);
      await bus(Prism).publish(topic, json({ kind: MSG.kept, recovery: share.recovery }));
      endLink();
      show("held");
      say("You are their guardian now");
    }),
  );
  $("guardian-copy").addEventListener("click", () => copy($("guardian-copy"), $("guardian-link").textContent));

  // ---- helping: find the request, show its code, release once.
  let request = null;
  $("help").addEventListener("click", () => {
    request = null;
    show("help");
    $("help-uri").focus();
  });
  $("help-go").addEventListener("click", () =>
    run(async (Prism) => {
      endLink();
      const topic = await Prism.rendezvousTopic(secretOf($("help-uri").value, HELP));
      const asked = awaitOn(Prism, topic, MSG.help, () => true, 15000);
      link = { stop: asked.stop };
      const message = await asked.found;
      Prism.checkDeviceRecord(message.device);
      guarding = await loadGuarding();
      const share = guarding.find((s) => s.prism === message.prism && s.recovery === message.recovery);
      if (!share) deny("share", "You hold no share for that request: it was released already, or it is for someone else");
      request = { topic, share, device: message.device };
      $("help-code").textContent = await Prism.recoveryCode(message.prism, message.device);
      $("help-check").hidden = false;
    }),
  );
  $("help-release").addEventListener("click", () =>
    run(async (Prism) => {
      if (!request) return;
      const released = await Prism.releaseShare(held.device, request.share, request.device);
      // Released once: the share leaves this device, so a replay finds nothing here.
      guarding = (await loadGuarding()).filter((s) => !(s.prism === request.share.prism && s.recovery === request.share.recovery));
      await saveGuarding(guarding);
      await bus(Prism).publish(request.topic, json({ kind: MSG.release, share: released }));
      request = null;
      $("help-uri").value = "";
      endLink();
      show("held");
      say("Share released");
    }),
  );
  $("back").addEventListener("click", () => {
    endLink();
    show("held");
  });
  // An icon button confirms with a check and says so; a word button says it in place.
  const copy = async (button, text) => {
    const was = button.innerHTML;
    const done = await navigator.clipboard.writeText(text).then(() => "Copied", () => "Select it");
    if (button.classList.contains("account-icon")) (button.innerHTML = glyph(CHECK)), say(done);
    else button.textContent = done;
    setTimeout(() => (button.innerHTML = was), 1600);
  };
  $("copy").addEventListener("click", () => copy($("copy"), held?.prism ?? ""));

  const reach = () => {
    paint();
    return quietly();
  };
  panel.addEventListener("beforetoggle", (event) => {
    if (event.newState === "open") {
      endLink();
      view = "held";
      reach();
    }
  });

  // { replace: true } starts a new key, for an app whose device was removed.
  const workspace = async ({ replace = false } = {}) => {
    const Prism = await prismId();
    await quietly();
    if (!held) deny("unsupported", "There is no Prism ID here yet");
    const { key, record } = await openWorkspace(Prism, held, replace ? null : await loadWorkspace());
    if (record) await saveDevice(held, record);
    working = true;
    paint();
    return key;
  };
  // Each message is sealed under the id's private boundary key; one that does not open is dropped.
  const channel = async (listen) => {
    const Prism = await prismId();
    await quietly();
    if (!held) deny("unsupported", "There is no Prism ID here yet");
    const topic = await appTopic(Prism, held, location.origin);
    const stop = bus(Prism).subscribe(topic, (bytes) => Prism.openPrivate(held, parse(bytes)).then(listen, () => {}));
    return { send: async (value) => bus(Prism).publish(topic, json((await Prism.sealPrivate(held, value)).sealed)), stop };
  };
  reach();
  return { reach, workspace, channel };
}

// The theme panel's contents, which the frame puts in its empty panel.
export const THEME_PANEL = `
    <p id="theme-title" class="theme-title">Theme</p>
    <div class="modes" role="group" aria-labelledby="theme-title">
      <button class="mode" type="button" aria-pressed="false" data-mode="immersive"><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false"><rect x="2" y="3" width="12" height="10" rx="2" /><circle cx="5.75" cy="6.5" r="1" /><path d="m2.5 11.5 3.25-3 2.5 2 2-1.75 3.25 2.75" /></svg>Immersive</button>
      <button class="mode" type="button" aria-pressed="false" data-mode="dark"><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false"><path d="M13.5 9.6A5.6 5.6 0 1 1 6.4 2.5a4.5 4.5 0 0 0 7.1 7.1z" /></svg>Dark</button>
      <button class="mode" type="button" aria-pressed="false" data-mode="light"><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false"><circle cx="8" cy="8" r="2.75" /><path d="M8 1.5v1.25M8 13.25v1.25M1.5 8h1.25M13.25 8h1.25M3.4 3.4l.9.9M11.7 11.7l.9.9M3.4 12.6l.9-.9M11.7 4.3l.9-.9" /></svg>Light</button>
    </div>
    <label id="find" class="find" hidden><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" focusable="false"><circle cx="7" cy="7" r="4.5" /><path d="m10.5 10.5 3 3" /></svg><input id="find-input" type="search" placeholder="Search Unsplash" aria-label="Search Unsplash for a background" aria-controls="photos" autocomplete="off" autocapitalize="none" spellcheck="false" enterkeyhint="search" /><span id="find-busy" class="find-busy" aria-hidden="true"></span></label>
    <div id="photos" class="photos" role="group" aria-label="Immersive background" aria-busy="false"></div>
    <p id="photos-note" class="photos-note" role="status" hidden></p>
    <button id="photos-more" class="photos-more" type="button" hidden>More photos</button>
    <p class="credit">Photo by <a id="credit-name" target="_blank" rel="noopener noreferrer"></a> on <a href="https://unsplash.com/?${UNSPLASH}" target="_blank" rel="noopener noreferrer">Unsplash</a></p>
`;
