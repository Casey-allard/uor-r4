// <prism-frame>: the canonical frame every Prism app opens in, as one web
// component. UOR-Foundation/template publishes it, content-addressed, on
// every push to main (frame/publish.mjs); every app page, and Holo Search,
// loads the current one at run time and checks its SHA-256 before running it.
// Change the frame in the template and every Prism app shows the change on
// its next load: no app is rebuilt.
//
// The contract (prism/frame/1):
//   <prism-frame name address live repo home purpose card identity
//                verdict verdict-label fingerprint proof proof-url examples>
//     <span slot="status">…</span>     optional: shown in the bar, e.g. "Update available"
//   </prism-frame>
// name      the app's name ("Luhn check")
// address   its holo:// name ("luhn-check"); derived from name when absent
// live      the app's canonical URL, which every share uses
// repo      its source; the GitHub icon hides when absent
// home      Holo Search: the home icon, and where the address bar resolves
// purpose   one or more sentences; the first is the share text
// card      the 1200 x 630 social card; live + "og.png" when absent
// identity  the page's full verification details (default "#identity-title")
// verdict   wait | good | bad: whether the holo's bytes match its fingerprint;
//           verdict-label says it in words, fingerprint is the holo's (shown
//           in the shield's panel, matched by the address bar)
// proof     optional: what the app's theorem guarantees, in one sentence
// proof-url optional: the proof's attestation
// examples  optional: "passed/total" once the page has replayed the holo's
//           examples, "running" while it does, "error" when it could not
// The element's account is the visitor's Prism id, made without asking on
// their first touch of the page (null until then); sign(bytes) signs with
// this browser's session key and returns the signature with the id's
// delegation; "accountchange" fires when the id is ready or changes. There is
// no sign-in. identity/PROTOCOL.md is the account contract.
// The shield sums the three: green only when the bytes are authentic and
// every example gives its proved answer; its panel shows each check, as a
// browser shows a page's security.
// The frame owns the theme: light, dark or immersive, plain dark by default.
// It sets the dark and immersive classes on <html> and remembers the choice
// as "prism-theme" (and the photo as "prism-backdrop"), as the page's theme
// bootstrap reads.
// The window and the bar end --prism-aside from the right edge (0 unless the
// page sets it on <html>), so a page can open a panel beside its window.

const CSS = "/*CSS*/";
const BRAND = "/*BRAND*/";
const HOME = "https://uor-foundation.github.io/holo-search/";
// The Prism registry: the canonical kappa registry. Its index lists every
// Prism app by its self-verifying address; every address bar reads it.
const REGISTRY = "https://registry.uor.foundation/";

// Immersive: the dark ground over one of nine Unsplash photos, chosen once
// (image is each one's Unsplash original) and published beside the frame
// (frame/publish.mjs), so no key ships; each is credited to its
// photographer in the theme panel.
const UNSPLASH = "utm_source=prism&utm_medium=referral";
// The built-in photos by id, with the colour each shows until it arrives.
// Who took them and where they are is in frame/theme.mjs, loaded only
// when wanted.
const BUILT_IN = { "E0AHdsENmDg": "#0c2659", "qtRF_RxCAo0": "#262626", "RF4p4rTM-2s": "#262626", "Q1p7bh3SHj8": "#0c2626", "t7EL2iG3jMc": "#59598c", "9wH624ALFQA": "#0c2640", "oMcTmNHclZI": "#0c2626", "cZgVpseeazg": "#26260c", "0wFwSZn6I-I": "#262626" };
const FIRST_PHOTO = "E0AHdsENmDg";
const MODES = ["immersive", "dark", "light"];
// Drawn on the page, outside the frame's shadow root: the photo fixed
// behind the app under a scrim, so the page's text stays legible on any of
// the three. It fills the window, exactly its edges and corners, and never
// the chrome around it. Only the ground goes; the page's surfaces keep
// their colour.
const BACKDROP_CSS = `
html.immersive { background: var(--prism-backdrop-color, #151312); }
html.immersive body { background: transparent; }
html.immersive body::before {
  content: "";
  position: fixed;
  top: var(--prism-window-top, 0px);
  right: var(--prism-window-right, 0px);
  bottom: var(--prism-window-bottom, 0px);
  left: var(--prism-window-left, 0px);
  border-radius: var(--prism-window-radius, 0px);
  z-index: -1;
  pointer-events: none;
  background: linear-gradient(rgb(12 10 9 / 0.5), rgb(12 10 9 / 0.62)), var(--prism-backdrop, none) center / cover no-repeat, var(--prism-backdrop-color, #151312);
}`;
let backdropSheet = null;

// Any other Unsplash photo, found from the theme panel, comes through the
// frame's Unsplash worker (frame/unsplash/worker.mjs), which holds the key.
const UNSPLASH_PROXY = "https://prism-unsplash.trinity-03d.workers.dev/";
// The page build (bootstrap/page.mjs) reads this into the page's connect-src.
export const UNSPLASH_ORIGIN = new URL(UNSPLASH_PROXY).origin;
// A photo read back from storage is trusted only in the shape the worker gives.
const photoOf = (value) => {
  const p = value && typeof value === "object" ? value : {};
  const ok =
    /^[\w-]{6,20}$/.test(p.id ?? "") &&
    typeof p.image === "string" && p.image.startsWith("https://images.unsplash.com/") &&
    /^#[0-9a-f]{6}$/i.test(p.color ?? "") &&
    /^[\w.-]{1,64}$/.test(p.user ?? "") &&
    typeof p.name === "string";
  return ok ? { id: p.id, image: p.image, color: p.color, alt: String(p.alt ?? "").slice(0, 200), name: p.name.slice(0, 80), user: p.user, page: String(p.page ?? "") } : null;
};
// The theme, the photo in use, and the last photos found and chosen (up to
// three), which the panel shows before the built-in ones.
const storedTheme = () => {
  try {
    const recent = JSON.parse(localStorage.getItem("prism-backdrop-recent") ?? "[]");
    return {
      mode: localStorage.getItem("prism-theme"),
      photo: localStorage.getItem("prism-backdrop"),
      recent: (Array.isArray(recent) ? recent : []).map(photoOf).filter(Boolean).slice(0, 3),
    };
  } catch {
    return { mode: null, photo: null, recent: [] };
  }
};

// Where the frame is published, and its theme module beside it.
const FRAME_HOST = "https://uor-foundation.github.io/template/frame/";
// The visitor's first move (a pointer, a touch, a key, a scroll): what the
// panels module and the photo wait for, so neither is part of the page's
// first load.
let moved = null;
const firstMove = () =>
  (moved ??= new Promise((resolve) => {
    const kinds = ["pointermove", "pointerdown", "touchstart", "keydown", "wheel", "scroll"];
    const go = () => {
      for (const kind of kinds) removeEventListener(kind, go, true);
      resolve();
    };
    for (const kind of kinds) addEventListener(kind, go, { capture: true, passive: true });
  }));
// Every panel's styles, the theme panel and the immersive photos:
// frame/panels.mjs, published beside the frame and named by its SHA-256,
// which publish.mjs writes here.
// It is fetched after the first move, checked against that hash as the page
// checks the frame, and imported; a failure is tried again the next time.
// Unpublished (a checkout, or an app that bundles its locked copy of the
// frame), it is imported from beside this file, as panels.mjs imports Prism id.
const PANELS = "/*PANELS*/";
let panelsModule = null;
let panelsSheet = null;
const loadPanels = () =>
  (panelsModule ??= (async () => {
    if (!/^[0-9a-f]{64}$/.test(PANELS)) return import("./panels.mjs");
    const bytes = await (await fetch(new URL(`${PANELS}/prism-panels.mjs`, FRAME_HOST))).arrayBuffer();
    const hex = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)), (b) => b.toString(16).padStart(2, "0")).join("");
    if (hex !== PANELS) throw new Error("the panels module is not the one this frame names");
    const url = URL.createObjectURL(new Blob([bytes], { type: "text/javascript" }));
    try {
      return await import(url);
    } finally {
      URL.revokeObjectURL(url);
    }
  })().catch((error) => {
    panelsModule = null;
    throw error;
  }));
const escape = (text) =>
  String(text ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const slugOf = (name) =>
  name.toLowerCase().normalize("NFKD").replace(/\p{M}/gu, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "app";
const sheetOf = (text) => {
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(text);
  return sheet;
};
let frameSheet = null;

// What the theme module borrows from the frame.
const THEME_CORE = { storedTheme, photoOf };

// The frame's markup. Every value in it is escaped by render().
const markup = (e) => `
  <div class="canvas" aria-hidden="true"></div>
  <div class="bar"><div class="inner">
    <div class="start">
      <a id="home" class="ghost icon" href="${e.home}" aria-label="Hologram: home" title="Hologram"><svg class="mark fill" viewBox="-104 -104 208 208" width="20" height="20" aria-hidden="true" focusable="false"><circle cx="0.20" cy="-97.39" r="2.61"/><circle cx="-22.86" cy="-86.55" r="2.71"/><circle cx="22.54" cy="-86.32" r="2.81"/><circle cx="-0.03" cy="-76.01" r="2.71"/><circle cx="45.26" cy="-75.92" r="7.80"/><circle cx="-45.82" cy="-75.86" r="2.61"/><circle cx="68.34" cy="-65.13" r="7.70"/><circle cx="-68.83" cy="-65.00" r="2.61"/><circle cx="-22.91" cy="-64.90" r="2.61"/><circle cx="22.71" cy="-64.88" r="2.51"/><circle cx="91.24" cy="-54.34" r="2.61"/><circle cx="-45.94" cy="-54.25" r="7.83"/><circle cx="-91.17" cy="-54.19" r="2.71"/><circle cx="-0.03" cy="-54.19" r="2.71"/><circle cx="45.35" cy="-54.19" r="7.80"/><circle cx="-22.86" cy="-43.64" r="2.71"/><circle cx="22.71" cy="-43.49" r="2.51"/><circle cx="68.29" cy="-43.47" r="7.73"/><circle cx="-68.60" cy="-43.37" r="7.73"/><circle cx="-45.85" cy="-32.63" r="7.77"/><circle cx="45.36" cy="-32.60" r="7.80"/><circle cx="-91.26" cy="-32.55" r="2.71"/><circle cx="0.10" cy="-32.51" r="2.61"/><circle cx="91.24" cy="-32.51" r="2.61"/><circle cx="68.22" cy="-21.95" r="7.83"/><circle cx="22.67" cy="-21.84" r="7.87"/><circle cx="-22.86" cy="-21.82" r="2.71"/><circle cx="-68.57" cy="-21.80" r="7.80"/><circle cx="45.45" cy="-11.06" r="7.73"/><circle cx="-0.19" cy="-11.04" r="7.87"/><circle cx="91.35" cy="-11.01" r="2.81"/><circle cx="-91.54" cy="-10.97" r="2.51"/><circle cx="-45.87" cy="-10.87" r="7.73"/><circle cx="22.71" cy="-0.27" r="8.06"/><circle cx="-22.89" cy="-0.21" r="7.90"/><circle cx="68.28" cy="-0.15" r="7.87"/><circle cx="-68.62" cy="-0.11" r="8.00"/><circle cx="-0.06" cy="10.98" r="7.87"/><circle cx="45.54" cy="11.00" r="7.83"/><circle cx="-45.85" cy="11.02" r="7.77"/><circle cx="-91.26" cy="11.10" r="2.71"/><circle cx="91.24" cy="11.13" r="2.61"/><circle cx="22.71" cy="21.64" r="2.71"/><circle cx="-68.74" cy="21.66" r="7.87"/><circle cx="-22.86" cy="21.67" r="7.87"/><circle cx="68.28" cy="21.67" r="7.87"/><circle cx="-91.54" cy="32.46" r="2.61"/><circle cx="0.15" cy="32.46" r="2.71"/><circle cx="-45.72" cy="32.50" r="7.73"/><circle cx="45.54" cy="32.59" r="7.83"/><circle cx="91.35" cy="32.63" r="2.81"/><circle cx="-23.01" cy="43.23" r="2.61"/><circle cx="68.25" cy="43.31" r="7.83"/><circle cx="-68.71" cy="43.34" r="7.83"/><circle cx="22.71" cy="43.37" r="2.90"/><circle cx="91.39" cy="53.92" r="2.71"/><circle cx="45.48" cy="53.95" r="7.87"/><circle cx="-45.86" cy="53.97" r="7.80"/><circle cx="-91.34" cy="53.99" r="2.61"/><circle cx="0.20" cy="54.09" r="2.61"/><circle cx="-68.57" cy="64.90" r="7.80"/><circle cx="-22.86" cy="64.92" r="2.71"/><circle cx="68.28" cy="64.92" r="2.71"/><circle cx="22.54" cy="65.15" r="2.81"/><circle cx="-45.88" cy="75.56" r="7.80"/><circle cx="0.10" cy="75.62" r="2.61"/><circle cx="45.32" cy="75.62" r="2.61"/><circle cx="22.53" cy="86.47" r="2.71"/><circle cx="-22.86" cy="86.75" r="2.71"/><circle cx="-0.03" cy="97.29" r="2.71"/></svg></a>
    </div>
    <form id="address" class="address" role="search" action="${e.home}" method="get" autocomplete="off">
      <button id="trust" class="ghost icon inset trust wait" type="button" popovertarget="trust-panel" aria-label="Verifying this app" title="Verifying this app"><svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path class="body" d="M12 3 4 6v6c0 4.6 3.4 8.4 8 9 4.6-.6 8-4.4 8-9V6z" /><path class="wait" d="M8.5 12h.01M12 12h.01M15.5 12h.01" /><path class="ok" d="m8.5 12 2.5 2.5 4.5-5" /><path class="bad" d="M12 8v4.5M12 16h.01" /></svg></button>
      <span class="field">
        <span class="url" aria-hidden="true"><span class="scheme">holo://</span><span class="host">${e.slug}</span></span>
        <input id="omnibox" class="omnibox" name="r" type="text" value="holo://${e.slug}" role="combobox" aria-label="Search Prism apps, or paste a holo address or hash" aria-autocomplete="list" aria-expanded="false" aria-controls="omnibox-list" autocomplete="off" autocapitalize="none" autocorrect="off" spellcheck="false" enterkeyhint="go" />
      </span>
      <a id="github" class="ghost icon inset${e.starred}" href="${e.repo}" aria-label="${e.githubLabel}" title="${e.githubLabel}"><svg class="gh" viewBox="0 0 16 16" width="20" height="20" aria-hidden="true" focusable="false"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" /></svg><span class="stars"><svg class="star" viewBox="0 0 16 16" width="12" height="12" aria-hidden="true" focusable="false"><path d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.751.751 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z" /></svg><span id="stars">${e.starsText}</span></span></a>
      <span class="sep" aria-hidden="true"></span>
      <button id="share" class="ghost icon inset" type="button" popovertarget="share-panel" aria-label="Share this app" title="Share"><svg class="share" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><circle cx="17.5" cy="5.5" r="2.5" /><circle cx="6.5" cy="12" r="2.5" /><circle cx="17.5" cy="18.5" r="2.5" /><path d="m8.7 13.3 6.6 3.9M15.3 6.8l-6.6 3.9" /></svg><svg class="done" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="m5 12.5 4.5 4.5L19 7.5" /></svg></button>
      <ul id="omnibox-list" class="omnibox-list" role="listbox" aria-label="Prism apps" hidden></ul>
    </form>
    <div class="end">
    <slot name="status"></slot>
    <span id="bar-status" class="sr" aria-live="polite"></span>
    <button id="theme" class="ghost icon" type="button" popovertarget="theme-panel" aria-label="Theme" title="Theme"><svg class="moon" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z" /></svg><svg class="sun" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" /></svg><svg class="photo" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><rect x="3" y="4.5" width="18" height="15" rx="2.5" /><circle cx="9" cy="10" r="1.75" /><path d="m3.5 17 5-4.5 3.5 3 3-2.5 5.5 4.5" /></svg></button>
    <button id="fullscreen" class="ghost icon" type="button" aria-label="Enter full screen" title="Full screen"><svg class="expand" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M4 9V5.5A1.5 1.5 0 0 1 5.5 4H9M15 4h3.5A1.5 1.5 0 0 1 20 5.5V9M20 15v3.5a1.5 1.5 0 0 1-1.5 1.5H15M9 20H5.5A1.5 1.5 0 0 1 4 18.5V15" /></svg><svg class="shrink" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M9 4v3.5A1.5 1.5 0 0 1 7.5 9H4M20 9h-3.5A1.5 1.5 0 0 1 15 7.5V4M15 20v-3.5a1.5 1.5 0 0 1 1.5-1.5H20M4 15h3.5A1.5 1.5 0 0 1 9 16.5V20" /></svg></button>
    <button id="account" class="ghost icon" type="button" popovertarget="account-panel" aria-label="Prism ID" title="Prism ID"><svg class="person" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><circle cx="12" cy="8" r="3.5" /><path d="M5 20c0-3.6 3.1-6 7-6s7 2.4 7 6" /></svg></button>
    </div>
  </div></div>
  <div id="trust-panel" class="trust-panel" popover aria-labelledby="trust-title">
    <div class="trust-head">
      <span id="trust-badge" class="trust-badge wait"><svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path class="body" d="M12 3 4 6v6c0 4.6 3.4 8.4 8 9 4.6-.6 8-4.4 8-9V6z" /><path class="wait" d="M8.5 12h.01M12 12h.01M15.5 12h.01" /><path class="ok" d="m8.5 12 2.5 2.5 4.5-5" /><path class="bad" d="M12 8v4.5M12 16h.01" /></svg></span>
      <div class="trust-words">
        <p id="trust-title" class="trust-title">Verifying this app</p>
        <p id="trust-sub" class="trust-sub">Checking it in your browser</p>
      </div>
    </div>
    <ol class="checks">
      <li id="check-authentic" class="check wait">
        <span class="mark"><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false"><circle class="ring" cx="12" cy="12" r="9" /><path class="wait" d="M8 12h.01M12 12h.01M16 12h.01" /><path class="ok" d="m8 12.5 2.8 2.8L16.5 9.5" /><path class="bad" d="M12 7.5v5M12 16.2h.01" /></svg></span>
        <div class="check-words">
          <p class="check-title">Authentic</p>
          <p id="authentic-detail" class="check-detail">Checking every byte against its fingerprint</p>
          <p id="authentic-print" class="check-meta" hidden><code id="trust-print"></code><button id="trust-copy" class="text" type="button">Copy</button></p>
        </div>
      </li>
      <li id="check-proved" class="check wait" hidden>
        <span class="mark"><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false"><circle class="ring" cx="12" cy="12" r="9" /><path class="wait" d="M8 12h.01M12 12h.01M16 12h.01" /><path class="ok" d="m8 12.5 2.8 2.8L16.5 9.5" /><path class="bad" d="M12 7.5v5M12 16.2h.01" /></svg></span>
        <div class="check-words">
          <p class="check-title">Formally proved</p>
          <p id="proved-detail" class="check-detail"></p>
          <p class="check-meta"><span>Proved in Lean when this app was built</span><a id="trust-proof" class="text" target="_blank" rel="noopener noreferrer" hidden>Attestation</a></p>
        </div>
      </li>
      <li id="check-replayed" class="check wait" hidden>
        <span class="mark"><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false"><circle class="ring" cx="12" cy="12" r="9" /><path class="wait" d="M8 12h.01M12 12h.01M16 12h.01" /><path class="ok" d="m8 12.5 2.8 2.8L16.5 9.5" /><path class="bad" d="M12 7.5v5M12 16.2h.01" /></svg></span>
        <div class="check-words">
          <p class="check-title">Behaves as proved</p>
          <p id="replayed-detail" class="check-detail">Replaying its examples</p>
        </div>
      </li>
    </ol>
    <a id="trust-details" class="trust-details" href="${e.identity}">Full details<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false"><path d="M5 12h14M13 6l6 6-6 6" /></svg></a>
  </div>
  <div id="share-panel" class="share-panel" popover aria-labelledby="share-title">
    <p id="share-title" class="share-title">Share ${e.name}</p>
    <div id="share-image" class="share-image"></div>
    <div class="share-link">
      <code class="share-url">${e.host}</code>
      <button id="share-copy" class="primary" type="button" autofocus><svg class="link" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M10 13.5a4.5 4.5 0 0 0 6.4.4l2.8-2.8a4.5 4.5 0 0 0-6.4-6.4l-1.3 1.3" /><path d="M14 10.5a4.5 4.5 0 0 0-6.4-.4l-2.8 2.8a4.5 4.5 0 0 0 6.4 6.4l1.3-1.3" /></svg><svg class="done" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="m5 12.5 4.5 4.5L19 7.5" /></svg><span id="share-copy-label">Copy link</span></button>
    </div>
    <ul class="share-targets">
      <li><a class="share-target" href="${e.x}" target="_blank" rel="noopener noreferrer"><span class="share-glyph"><svg class="fill" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M17.8 3h3.1l-6.8 7.7L22 21h-6.2l-4.9-6.3L5.3 21H2.2l7.2-8.3L1.8 3h6.4l4.4 5.8zm-1.1 16.2h1.7L7.4 4.7H5.6z" /></svg></span>X</a></li>
      <li><a class="share-target" href="${e.linkedin}" target="_blank" rel="noopener noreferrer"><span class="share-glyph"><svg class="fill" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M20.45 20.45h-3.56v-5.57c0-1.33-.02-3.04-1.85-3.04-1.85 0-2.14 1.45-2.14 2.94v5.67H9.35V9h3.41v1.56h.05c.48-.9 1.64-1.85 3.37-1.85 3.6 0 4.27 2.37 4.27 5.46zM5.34 7.43a2.06 2.06 0 1 1 0-4.13 2.06 2.06 0 0 1 0 4.13M7.12 20.45H3.56V9h3.56zM22.22 0H1.77C.79 0 0 .77 0 1.73v20.54C0 23.23.79 24 1.77 24h20.45c.98 0 1.78-.77 1.78-1.73V1.73C24 .77 23.2 0 22.22 0" /></svg></span>LinkedIn</a></li>
      <li><a class="share-target" href="${e.facebook}" target="_blank" rel="noopener noreferrer"><span class="share-glyph"><svg class="fill" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><path d="M24 12.07C24 5.41 18.63 0 12 0S0 5.4 0 12.07C0 18.1 4.39 23.1 10.13 24v-8.44H7.08v-3.49h3.04V9.41c0-3.02 1.8-4.7 4.54-4.7 1.31 0 2.68.24 2.68.24v2.97h-1.5c-1.5 0-1.96.93-1.96 1.89v2.26h3.32l-.53 3.5h-2.8V24C19.62 23.1 24 18.1 24 12.07" /></svg></span>Facebook</a></li>
      <li><a class="share-target" href="${e.email}"><span class="share-glyph"><svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><rect x="3" y="5" width="18" height="14" rx="2" /><path d="m3.5 6.5 8.5 6.5 8.5-6.5" /></svg></span>Email</a></li>
      <li id="share-more-item" hidden><button id="share-more" class="share-target" type="button"><span class="share-glyph"><svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false"><circle cx="5" cy="12" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /></svg></span>More</button></li>
    </ul>
  </div>
  <div id="account-panel" class="account-panel" popover aria-labelledby="account-title"></div>
  <div id="theme-panel" class="theme-panel" popover aria-labelledby="theme-title"></div>
`;

// Speculation: the app a visitor shows they are about to open (home under
// the pointer, a suggestion chosen in the address bar) is loaded before they
// open it. A Prism app is named by its content, so what is loaded early is
// never stale and never another app's; on its own origin it is prerendered,
// so it also verifies its bytes and replays its examples before the visitor
// arrives, and opening it is showing a page already checked. Another origin
// is only fetched.
// The page carries one fixed rule (bootstrap/page.mjs hashes it into its
// Content-Security-Policy): prerender, or for another origin prefetch, any
// link marked data-prism-ahead. The frame marks the pages to load: up to
// three links, the latest first, rendered but out of sight and out of the
// tab order, since the browser only acts on rendered links. A page without
// the rule gets a plain prefetch of a same-origin page instead.
const speculation = { box: null, prefetched: new Set() };
function speculate(href) {
  if (!href) return;
  let url;
  try {
    url = new URL(href, location.href);
  } catch {
    return;
  }
  url.hash = "";
  const here = new URL(location.href);
  here.hash = "";
  if (!/^https?:$/.test(url.protocol) || url.href === here.href) return;
  if (document.querySelector('script[type="speculationrules"][data-prism-ahead]')) {
    if (!speculation.box?.isConnected) {
      speculation.box = document.createElement("div");
      speculation.box.setAttribute("aria-hidden", "true");
      Object.assign(speculation.box.style, { position: "absolute", width: "1px", height: "1px", overflow: "hidden", clipPath: "inset(50%)", pointerEvents: "none" });
      document.body.append(speculation.box);
    }
    const held = [...speculation.box.children].find((link) => link.href === url.href);
    if (held) {
      speculation.box.prepend(held);
      return;
    }
    const link = Object.assign(document.createElement("a"), { href: url.href, tabIndex: -1 });
    link.dataset.prismAhead = "";
    speculation.box.prepend(link);
    while (speculation.box.children.length > 3) speculation.box.lastElementChild.remove();
    return;
  }
  if (url.origin !== location.origin || speculation.prefetched.has(url.href)) return;
  speculation.prefetched.add(url.href);
  const link = Object.assign(document.createElement("link"), { rel: "prefetch", href: url.href });
  document.head.append(link);
}

// GitHub stars, as a short count: 128, 1.2k, 12k, 1.2m. None, or none
// known, shows the GitHub mark alone.
const shortCount = (n) =>
  n < 1000 ? String(n) : n < 1e6 ? `${(n / 1000).toFixed(n < 10000 ? 1 : 0).replace(/\.0$/, "")}k` : `${(n / 1e6).toFixed(1).replace(/\.0$/, "")}m`;
const starsOf = (n) => {
  const stars = Number.isInteger(n) && n > 0 ? n : 0;
  return {
    stars,
    starsText: stars ? shortCount(stars) : "",
    starred: stars ? " starred" : "",
    githubLabel: stars ? `Source on GitHub, ${stars} star${stars === 1 ? "" : "s"}` : "Source on GitHub",
  };
};

// The Prism index: every Prism app that verifies, with its GitHub stars, from
// the Prism registry's /index/ (its indexer, beside the canonical kappa
// registry, re-verifies every app every ten minutes and on each registration,
// and publishes a content-addressed snapshot). The frame reads it once per
// page: the pointer fresh
// (no-cache, so a new app or star shows on the next load), then the snapshot
// it names, whose SHA-256 is checked before anything in it is shown. It is
// read only where the page's Content-Security-Policy admits the registry; any
// failure leaves an empty index and the counts the page was built with. No
// visitor's browser ever calls GitHub.
const repoOf = (link) => (String(link ?? "").match(/^https:\/\/github\.com\/([\w.-]+\/[\w.-]+?)(?:\.git)?\/?$/) ?? [])[1] ?? "";
const NO_INDEX = Object.freeze({ apps: [], stars: {} });
let indexAsked = null;
function holoIndex() {
  if (indexAsked) return indexAsked;
  const pointer = new URL("index/current.json", REGISTRY);
  const policy = [...document.querySelectorAll('meta[http-equiv="Content-Security-Policy" i]')].map((meta) => meta.content).join(";");
  const connect = /(?:^|;)\s*connect-src([^;]*)/.exec(policy)?.[1] ?? /(?:^|;)\s*default-src([^;]*)/.exec(policy)?.[1];
  const admitted = connect === undefined || (pointer.origin === location.origin && connect.includes("'self'")) || connect.includes(pointer.origin);
  if (!admitted) return (indexAsked = Promise.resolve(NO_INDEX));
  const hex = (buffer) => Array.from(new Uint8Array(buffer), (b) => b.toString(16).padStart(2, "0")).join("");
  return (indexAsked = (async () => {
    const named = await (await fetch(pointer, { cache: "no-cache" })).json();
    if (!/^[0-9a-f]{64}$/.test(named?.sha256 ?? "")) return NO_INDEX;
    const bytes = await (await fetch(new URL(`${named.sha256}.json`, pointer))).arrayBuffer();
    if (hex(await crypto.subtle.digest("SHA-256", bytes)) !== named.sha256) return NO_INDEX;
    const apps = (JSON.parse(new TextDecoder().decode(bytes)).apps ?? []).filter((app) => typeof app?.name === "string" && /^https:\/\//.test(app.live ?? ""));
    return { apps, stars: Object.fromEntries(apps.map((app) => [repoOf(app.repository), app.stars])) };
  })().catch(() => NO_INDEX));
}
// The star count as a row shows it: GitHub's gold star and the short count.
const githubMark = '<svg class="gh" viewBox="0 0 16 16" width="12" height="12" aria-hidden="true" focusable="false"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" /></svg>';
const starMark = '<svg class="star" viewBox="0 0 16 16" width="12" height="12" aria-hidden="true" focusable="false"><path d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.751.751 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z" /></svg>';

// Prism ids (prism/identity/4, identity/PROTOCOL.md): stitch's Prism id,
// named by the κ of its genesis and held by devices whose keys never leave
// them, with a recovery kit its owner keeps, so losing a device never loses
// the id. The frame keeps this app's session and signs with it; the panels
// module holds this origin's device and makes the id and its session, in the visitor's
// browser. Devices of one id meet on stitch's relay, which the page build
// reads from here into the page's connect-src (with its WebSocket origin).
export const PRISM_RELAY = "https://ntfy.sh";
const SESSION_DAYS = 7;
const b64u = (bytes) => btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
// The session, in this app's IndexedDB: the browser key cannot be read out
// of it by any script, only used. Nothing goes to localStorage.
function store(mode, work) {
  return new Promise((done, fail) => {
    const open = indexedDB.open("prism-identity", 1);
    open.onupgradeneeded = () => open.result.createObjectStore("session");
    open.onerror = () => fail(open.error);
    open.onsuccess = () => {
      const db = open.result;
      const tx = db.transaction("session", mode);
      const request = work(tx.objectStore("session"));
      tx.oncomplete = () => (db.close(), done(request?.result));
      tx.onerror = () => (db.close(), fail(tx.error));
    };
  });
}
const accountChannel = { session: null, loaded: null };
function loadSession() {
  return (accountChannel.loaded ??= store("readonly", (s) => s.get("current"))
    .then((saved) => (accountChannel.session = saved?.delegation?.envelope && saved.expires > Date.now() ? saved : null))
    .catch(() => null));
}
// What the account panel, in the panels module, borrows from the frame.
const ACCOUNT_CORE = { SESSION_DAYS, b64u, store, accountChannel, loadSession, PRISM_RELAY };

// Everything the markup needs, from the attributes (get), the page's title
// and its address: one function, for the element and for the shell.
function modelOf(get, title, base) {
  const name = get("name") ?? title;
  const slug = get("address") ?? slugOf(name);
  const home = get("home") ?? HOME;
  const live = get("live") ?? base;
  const purpose = get("purpose") ?? "";
  const text = purpose ? `${name}: ${purpose.split(/(?<=\.)\s/)[0]}` : name;
  const url = new URL(live, base);
  const query = (params) => new URLSearchParams(params).toString();
  return {
    name,
    slug,
    home,
    live: url.href,
    text,
    repo: get("repo") ?? "",
    ...starsOf(Number(get("stars"))),
    card: get("card") ?? new URL("og.png", url).href,
    identity: get("identity") ?? "#identity-title",
    host: (url.host + url.pathname).replace(/\/$/, ""),
    x: `https://x.com/intent/post?${query({ text, url: url.href })}`,
    linkedin: `https://www.linkedin.com/sharing/share-offsite/?${query({ url: url.href })}`,
    facebook: `https://www.facebook.com/sharer/sharer.php?${query({ u: url.href })}`,
    email: `mailto:?subject=${encodeURIComponent(name)}&body=${encodeURIComponent(`${text}\n\n${url.href}`)}`,
  };
}

// The shell: the frame's first paint, which a page's build writes into the
// page (bootstrap/page.mjs) as a declarative shadow root holding this
// frame's own styles and markup for its attributes. The browser paints the
// real bar with the page's first frame; when this module arrives the element
// takes that root over and draws the same thing, so nothing moves and
// nothing flickers between apps. The verdict starts as the page starts
// (checking), and the theme's icon and colours follow the page's colour
// scheme, which the page sets before its first paint.
//
// The shell is the bar alone: the trust, share and theme panels open only on a
// click, after this module has loaded and drawn the whole root, so neither
// they nor their styles are written into the page, which keeps every app's
// first load inside its Lighthouse budget.
const PANEL_RULE = /trust-panel|share-|theme-|check-|omnibox|opt-|\.trust-/;
function barCss(css) {
  const rules = [];
  for (let depth = 0, start = 0, at = 0; at < css.length; at++) {
    if (css[at] === "{") depth++;
    else if (css[at] === "}" && --depth === 0) {
      rules.push(css.slice(start, at + 1));
      start = at + 1;
    }
  }
  return rules.filter((rule) => rule.trimStart().startsWith("@") || !PANEL_RULE.test(rule.slice(0, rule.indexOf("{")))).join("");
}
export function shell(attributes, base) {
  const m = modelOf((name) => attributes[name] ?? null, attributes.name ?? "", base);
  const e = Object.fromEntries(Object.entries(m).map(([key, value]) => [key, escape(value)]));
  let html = markup(e);
  html = html.slice(0, html.indexOf('  <div id="trust-panel"'));
  if (!m.repo) html = html.replace('<a id="github" ', '<a id="github" hidden ');
  const css = barCss(CSS);
  return { css, html: `<template shadowrootmode="open"><style>${css}</style>${html}</template>` };
}

// The element runs in a browser; the module also loads at build time, where
// there is none, to write the shell.
// A bar panel opens under its button, `rem` wide at most, `gap` below the bar,
// and never closer than 16 px to either side of the window: on the left, a
// little before the button's centre; on the right, edge to edge with it.
function placeUnder(frame, button, panel, name, rem, gap, side) {
  const bar = frame.shadowRoot.querySelector(".bar").getBoundingClientRect();
  const edge = button.getBoundingClientRect();
  const view = document.documentElement.clientWidth;
  const width = Math.min(rem * parseFloat(getComputedStyle(document.documentElement).fontSize), view - 32);
  const at = side === "left" ? edge.left + edge.width / 2 - 24 : view - edge.right;
  panel.style.setProperty(`--${name}-top`, `${Math.round(bar.bottom + gap)}px`);
  panel.style.setProperty(`--${name}-${side}`, `${Math.round(Math.max(16, Math.min(at, view - width - 16)))}px`);
}

const Element = globalThis.HTMLElement ?? class {};
class PrismFrame extends Element {
  static observedAttributes = ["name", "address", "live", "repo", "home", "purpose", "card", "identity", "verdict", "verdict-label", "fingerprint", "proof", "proof-url", "examples", "stars"];

  constructor() {
    super();
    this.attachShadow({ mode: "open" });
    frameSheet ??= sheetOf(CSS);
    this.shadowRoot.adoptedStyleSheets = [frameSheet];
    this.$ = (id) => this.shadowRoot.getElementById(id);
  }

  connectedCallback() {
    // A page without the brand's tokens gets them, so the frame never draws unstyled.
    if (!getComputedStyle(document.documentElement).getPropertyValue("--background").trim()) {
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheetOf(BRAND)];
    }
    backdropSheet ??= sheetOf(BACKDROP_CSS);
    if (!document.adoptedStyleSheets.includes(backdropSheet)) {
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, backdropSheet];
    }
    // Plain dark is the default: a visitor who has chosen nothing gets the
    // warm dark ground, and immersive is one choice away in the theme panel.
    // A page built when immersive was the default sets it before first paint;
    // the frame corrects it, once, as it arrives.
    const chosen = storedTheme().mode;
    if (chosen !== "light") document.documentElement.classList.add("dark");
    if (chosen !== "immersive") document.documentElement.classList.remove("immersive");
    this.render();
    this.watchTheme();
    this.watchHeight();
  }

  disconnectedCallback() {
    this.heightWatch?.disconnect();
    this.windowWatch?.disconnect();
    this.fullscreenWatch?.abort();
    this.themeWatch?.disconnect();
    this.accountWatch?.abort();
  }

  attributeChangedCallback(name, before, after) {
    if (!this.shadowRoot.firstChild || before === after) return;
    if (["verdict", "verdict-label", "fingerprint", "proof", "proof-url", "examples"].includes(name)) this.paintTrust();
    else if (name === "stars") this.paintStars();
    else this.render();
  }

  // Everything the markup needs, from the attributes.
  model() {
    return modelOf((name) => this.getAttribute(name), document.title, location.href);
  }

  render() {
    const m = this.model();
    const e = Object.fromEntries(Object.entries(m).map(([key, value]) => [key, escape(value)]));
    this.data = m;
    this.shadowRoot.innerHTML = markup(e);
    if (!m.repo) this.$("github").hidden = true;
    this.liveStars();
    this.paintTrust();
    this.paintTheme();
    this.styled(this.$("trust-panel"));
    this.styled(this.$("share-panel"));
    this.$("omnibox").addEventListener("focus", () => loadPanels().then((panels) => this.adoptPanels(panels), () => {}), { once: true });
    this.trust();
    this.share();
    this.omnibox();
    this.themePanel();
    this.fullScreen();
    this.accountPanel();
    this.watchWindow();
    const home = this.$("home");
    for (const type of ["pointerenter", "focus", "touchstart"]) home.addEventListener(type, () => speculate(this.data.home), { passive: true });
  }

  // The shield and its panel: three checks, each from what the page found.
  // Authentic: the holo's bytes match its fingerprint. Formally proved: the
  // theorem those bytes were built from, proved in Lean when they were
  // built, so it holds for this app only when they are authentic. Behaves as
  // proved: every example the holo carries gives its proved answer here.
  // The shield is the sum: green only when every check it has passed.
  checks() {
    const verdict = this.getAttribute("verdict") ?? "wait";
    const label = this.getAttribute("verdict-label") ?? "";
    const proof = this.getAttribute("proof") ?? "";
    const examples = this.getAttribute("examples");
    const authentic = ["good", "bad"].includes(verdict) ? verdict : "wait";
    const failed = label && !/^not verified$/i.test(label) ? label : "Its bytes do not match its fingerprint";
    const checks = {
      authentic: {
        state: authentic,
        detail: { wait: "Checking every byte against its fingerprint", good: "Every byte matches its fingerprint", bad: failed }[authentic],
      },
    };
    if (proof) checks.proved = { state: authentic, detail: proof };
    if (examples !== null) {
      const [passed, total] = examples.split("/").map(Number);
      const counted = Number.isInteger(passed) && Number.isInteger(total) && total > 0;
      let state = "wait";
      let detail = "Replaying its examples in your browser";
      if (authentic === "bad") [state, detail] = ["bad", "Not run: the app is not authentic"];
      else if (examples === "error") [state, detail] = ["bad", "Its examples could not be replayed"];
      else if (counted && authentic === "good" && passed === total) [state, detail] = ["good", `${total} of ${total} examples give their proved answers in your browser`];
      else if (counted && authentic === "good") [state, detail] = ["bad", `${total - passed} of ${total} examples gave a wrong answer`];
      checks.replayed = { state, detail };
    }
    const states = Object.values(checks).map((check) => check.state);
    const state = states.includes("bad") ? "bad" : states.includes("wait") ? "wait" : "good";
    return { state, checks };
  }

  paintTrust() {
    const { state, checks } = this.checks();
    const fingerprint = this.getAttribute("fingerprint") ?? "";
    const title = { wait: "Verifying this app", good: "Verified app", bad: "Not verified" }[state];
    const sub = {
      wait: "Checking it in your browser",
      good: checks.proved ? "Proved in Lean. Checked in your browser." : "Checked in your browser.",
      bad: "A check failed. Do not trust its answers.",
    }[state];
    const shield = this.$("trust");
    shield.classList.remove("wait", "good", "bad");
    shield.classList.add(state);
    const words = `${title}. ${sub}`;
    shield.setAttribute("aria-label", words);
    shield.title = words;
    this.$("trust-badge").className = `trust-badge ${state}`;
    this.$("trust-title").textContent = title;
    this.$("trust-sub").textContent = sub;
    for (const id of ["authentic", "proved", "replayed"]) {
      const row = this.$(`check-${id}`);
      const check = checks[id];
      row.hidden = !check;
      if (!check) continue;
      row.className = `check ${check.state}`;
      this.$(`${id}-detail`).textContent = check.detail;
    }
    this.$("authentic-print").hidden = !fingerprint;
    this.$("trust-print").textContent = fingerprint ? `blake3:${fingerprint.slice(0, 8)}…${fingerprint.slice(-4)}` : "";
    this.$("trust-print").title = fingerprint;
    const url = this.getAttribute("proof-url") ?? "";
    const link = this.$("trust-proof");
    link.hidden = !/^https:\/\//.test(url);
    if (!link.hidden) link.href = url;
  }

  // The panel opens under the shield, as a browser's page information does,
  // its left edge near the shield's and never closer than 16 px to either
  // side of the window. Without popovers the shield goes to the details.
  trust() {
    const frame = this;
    const button = frame.$("trust");
    const panel = frame.$("trust-panel");
    if (typeof panel.showPopover !== "function") {
      button.removeAttribute("popovertarget");
      button.addEventListener("click", () => (location.hash = frame.data.identity));
      return;
    }
    panel.addEventListener("beforetoggle", (event) => {
      if (event.newState !== "open") return;
      placeUnder(frame, button, panel, "trust", 22, 6, "left");
    });
    frame.$("trust-copy").addEventListener("click", async () => {
      const copy = frame.$("trust-copy");
      try {
        await navigator.clipboard.writeText(frame.getAttribute("fingerprint") ?? "");
        copy.textContent = "Copied";
      } catch {
        copy.textContent = "Select it";
      }
      setTimeout(() => (copy.textContent = "Copy"), 1600);
    });
    frame.$("trust-details").addEventListener("click", () => panel.hidePopover());
  }

  // The star count the page was built with (bootstrap/page.mjs), brought up
  // to date from the Prism index (holoIndex) once the visitor reaches for the
  // bar: the GitHub pill under the pointer or in focus, or the address bar
  // opened (omnibox). Never on load: the index is two requests to Holo
  // Search, and an app's Lighthouse budget allows two third-party requests,
  // which the frame's own pointer and module already take. From no count to
  // one star and on, the bar follows the repository. A changed count is set
  // as the stars attribute, which repaints only the GitHub pill, so a visitor
  // typing in the address bar is never interrupted.
  liveStars() {
    const github = this.$("github");
    if (!repoOf(this.data.repo) || !github) return;
    const refresh = () => holoIndex().then((known) => this.applyStars(known));
    for (const type of ["pointerenter", "focus", "touchstart"]) github.addEventListener(type, refresh, { once: true, passive: true });
  }

  applyStars({ stars }) {
    const n = stars[repoOf(this.data.repo)];
    if (Number.isInteger(n) && String(n) !== (this.getAttribute("stars") ?? "")) this.setAttribute("stars", String(n));
  }

  paintStars() {
    const github = this.$("github");
    if (!github) return;
    const { starsText, starred, githubLabel } = starsOf(Number(this.getAttribute("stars")));
    github.classList.toggle("starred", Boolean(starred));
    this.$("stars").textContent = starsText;
    github.setAttribute("aria-label", githubLabel);
    github.title = githubLabel;
  }

  // The theme lives on <html>, where the page's own bootstrap set it before
  // first paint; the frame mirrors it and sets it. Immersive is the dark
  // ground with the chosen photo behind the page.
  watchTheme() {
    this.themeWatch = new MutationObserver(() => this.paintTheme());
    this.themeWatch.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
  }
  mode() {
    const root = document.documentElement.classList;
    return root.contains("immersive") ? "immersive" : root.contains("dark") ? "dark" : "light";
  }
  // The photo in use, as far as the ground needs it (its id and colour): a
  // built-in one, one found and chosen before, or, where the browser keeps
  // nothing (a private window), the one chosen this visit.
  photoRef() {
    const { photo: id, recent } = storedTheme();
    const wanted = id ?? this.chosen?.id;
    if (wanted && BUILT_IN[wanted]) return { id: wanted, color: BUILT_IN[wanted] };
    const found = [...recent, this.chosen].find((photo) => photo && photo.id === wanted);
    return found ? { id: found.id, color: found.color } : { id: FIRST_PHOTO, color: BUILT_IN[FIRST_PHOTO] };
  }
  // The panels module, after the visitor's first move.
  panels() {
    return firstMove().then(loadPanels);
  }
  // Its styles, in this frame's root.
  adoptPanels(panels) {
    panelsSheet = panels.sheet;
    if (!this.shadowRoot.adoptedStyleSheets.includes(panelsSheet)) this.shadowRoot.adoptedStyleSheets = [...this.shadowRoot.adoptedStyleSheets, panelsSheet];
  }
  // A panel opened before its styles came (a click that moved nothing, as
  // some assistive technology sends) waits for them, then opens.
  styled(panel) {
    panel.addEventListener("beforetoggle", (event) => {
      if (event.newState !== "open" || (panelsSheet && this.shadowRoot.adoptedStyleSheets.includes(panelsSheet))) return;
      event.preventDefault();
      loadPanels().then((panels) => {
        this.adoptPanels(panels);
        panel.showPopover();
      }, () => {});
    });
  }
  paintTheme() {
    const mode = this.mode();
    const root = document.documentElement;
    this.toggleAttribute("dark", mode !== "light");
    this.toggleAttribute("immersive", mode === "immersive");
    if (mode === "immersive") {
      // The photo's own colour is the ground until the photo arrives.
      root.style.setProperty("--prism-backdrop-color", this.photoRef().color);
    } else {
      this.backdrop = null;
      root.style.removeProperty("--prism-backdrop");
      root.style.removeProperty("--prism-backdrop-color");
    }
    // The photo, and the panel's photos and credit, come with the panels
    // module: at once once it is here; before, after the visitor's first move.
    if (mode === "immersive" || this.themed) (panelsModule ?? this.panels()).then((theme) => theme.paint(this, THEME_CORE), () => {});
    const button = this.$("theme");
    if (!button) return;
    const label = `Theme: ${mode[0].toUpperCase()}${mode.slice(1)}`;
    button.setAttribute("aria-label", label);
    button.title = label;
    for (const choice of this.shadowRoot.querySelectorAll(".mode")) choice.setAttribute("aria-pressed", String(choice.dataset.mode === mode));
  }
  setTheme(mode, photo) {
    const root = document.documentElement;
    if (photo) this.chosen = photo;
    try {
      localStorage.setItem("prism-theme", mode);
      if (photo) {
        localStorage.setItem("prism-backdrop", photo.id);
        if (!BUILT_IN[photo.id]) {
          const recent = [photo, ...storedTheme().recent.filter((p) => p.id !== photo.id)].slice(0, 3);
          localStorage.setItem("prism-backdrop-recent", JSON.stringify(recent));
        }
      }
    } catch {
      // A private window keeps no preference; the page still switches.
    }
    root.classList.toggle("dark", mode !== "light");
    root.classList.toggle("immersive", mode === "immersive");
    // A new photo in the same mode changes no class, so nothing else repaints.
    this.paintTheme();
  }

  // The theme panel: the three modes, a search of Unsplash, and the photos:
  // with nothing typed, the last ones found and chosen, then the nine built
  // in, space first; with words, what Unsplash finds, nine at a time. Choosing a
  // photo is also the way into immersive. It opens under the theme button,
  // its right edge on the button's. Photos are fetched the first time the
  // panel opens, never with the page. Without popovers the button steps
  // through the modes.
  themePanel() {
    const frame = this;
    const button = frame.$("theme");
    const panel = frame.$("theme-panel");
    if (typeof panel.showPopover !== "function") {
      button.removeAttribute("popovertarget");
      button.addEventListener("click", () => frame.setTheme(MODES[(MODES.indexOf(frame.mode()) + 1) % MODES.length]));
      return;
    }
    // Its modes, photos, search and styles arrive with the panels module;
    // the panel waits for them rather than open empty.
    let ui = null;
    const ready = (theme) => {
      // A render since this one (an attribute changed, as when Holo Search
      // opens an app) drew a new panel, which its own render wires.
      if (ui || !panel.isConnected) return;
      frame.adoptPanels(theme);
      panel.innerHTML = theme.THEME_PANEL;
      frame.themed = true;
      ui = theme.panel(frame, THEME_CORE);
      theme.paint(frame, THEME_CORE);
    };
    this.panels().then(ready, () => {});
    for (const type of ["pointerenter", "focus"]) button.addEventListener(type, () => this.panels().catch(() => {}), { passive: true });
    panel.addEventListener("beforetoggle", (event) => {
      if (event.newState !== "open") return;
      // Opened before the module came (a click that moved nothing, as some
      // assistive technology sends): fetch it now, then open.
      if (!ui) {
        event.preventDefault();
        loadPanels().then((theme) => {
          ready(theme);
          panel.showPopover();
        }, () => {});
        return;
      }
      placeUnder(frame, button, panel, "theme", 20, 6, "right");
      ui.open();
    });
  }
  // Full screen: the whole page, where the browser allows it; the button
  // hides where it does not (an iPhone's Safari). Escape, or the button,
  // leaves it, and the icon follows either way.
  fullScreen() {
    const button = this.$("fullscreen");
    const root = document.documentElement;
    const enter = root.requestFullscreen ?? root.webkitRequestFullscreen;
    const leave = document.exitFullscreen ?? document.webkitExitFullscreen;
    const current = () => document.fullscreenElement ?? document.webkitFullscreenElement ?? null;
    if (!(document.fullscreenEnabled ?? document.webkitFullscreenEnabled) || !enter || !leave) {
      button.hidden = true;
      return;
    }
    const paint = () => {
      const on = Boolean(current());
      this.toggleAttribute("fullscreen", on);
      const label = on ? "Exit full screen" : "Enter full screen";
      button.setAttribute("aria-label", label);
      button.title = on ? "Exit full screen" : "Full screen";
    };
    button.addEventListener("click", () => {
      const done = current() ? leave.call(document) : enter.call(root);
      done?.catch?.(() => {});
    });
    this.fullscreenWatch?.abort();
    this.fullscreenWatch = new AbortController();
    for (const type of ["fullscreenchange", "webkitfullscreenchange"]) {
      document.addEventListener(type, paint, { signal: this.fullscreenWatch.signal });
    }
    paint();
  }

  // The visitor's Prism id, for the app: null until it is made.
  get account() {
    const saved = accountChannel.session;
    return saved ? { id: saved.account, key: saved.key, expires: saved.expires } : null;
  }

  // Signs bytes with this browser's session key. The delegation proves the
  // Prism id let that key act for it here; verifySession checks it.
  async sign(data) {
    const saved = accountChannel.session;
    if (!saved || saved.expires <= Date.now()) throw Object.assign(new Error("The Prism id is not ready yet"), { name: "IdentityError", code: "signed-out" });
    const signature = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, saved.privateKey, data);
    return { signature: b64u(signature), delegation: saved.delegation };
  }

  // What the account in the panels module (panels.mjs) gives this app: its
  // workspace key, 32 bytes only a device of the visitor's Prism ID opens
  // (openWorkspace; { replace: true } starts a new one), and this app on the
  // visitor's other devices, where send(value) reaches them and listen(value)
  // hears them, sealed to the Prism ID (appTopic).
  viaAccount(work) {
    if (!this.accountsReady) return Promise.reject(Object.assign(new Error("A Prism ID cannot work here"), { name: "IdentityError", code: "unsupported" }));
    return this.accountsReady().then(work);
  }
  workspaceKey(options) {
    return this.viaAccount((accounts) => accounts.workspace(options));
  }
  appChannel(listen) {
    return this.viaAccount((accounts) => accounts.channel(listen));
  }

  // The Prism id icon, after full screen: a line drawing until the id is
  // ready, filled once it is. The id is made, and this app's session signed,
  // with nothing to click and nothing in the way, by the panels module
  // (accounts() there), fetched on the visitor's first touch of the page or
  // reach for the icon, so an app's first load carries none of it. Where a Prism id cannot work (no secure context, no
  // WebCrypto, no IndexedDB, no popover) the icon is not shown, so it never
  // offers what would fail.
  accountPanel() {
    const button = this.$("account");
    const panel = this.$("account-panel");
    this.accountWatch?.abort();
    this.accountReady = this.accountsReady = null;
    let wired = false;
    if (typeof panel.showPopover !== "function" || !window.isSecureContext || !globalThis.crypto?.subtle || !globalThis.indexedDB) {
      button.hidden = true;
      return;
    }
    const paint = () => {
      const saved = accountChannel.session;
      const label = "Prism ID";
      button.classList.toggle("signed-in", Boolean(saved));
      button.setAttribute("aria-label", label);
      button.title = label;
    };
    // Reaching for the icon is intent enough: no wait for a first move. An
    // app sets the frame's attributes as it loads, and each redraws the bar,
    // so this render's panel may be gone by the time the module arrives: it
    // then wires nothing, and the panel now drawn wires itself.
    const current = () => this.$("account-panel") === panel;
    const ready = () => {
      if (!this.accountReady) {
        const mine = loadPanels().then((panels) => {
          if (!current()) throw new Error("the bar was redrawn");
          this.adoptPanels(panels);
          const accounts = panels.accounts(this, { ...ACCOUNT_CORE, paint });
          wired = true;
          return accounts;
        });
        mine.catch(() => {
          if (this.accountReady === mine) this.accountReady = null;
        });
        this.accountReady = mine;
      }
      return this.accountReady;
    };
    this.accountsReady = ready;
    const reach = () => ready().then((accounts) => accounts.reach(), () => {});
    for (const type of ["pointerenter", "focus", "touchstart"]) button.addEventListener(type, reach, { passive: true });
    // First use: the visitor's first touch, click or key anywhere makes the id.
    this.accountWatch = new AbortController();
    for (const type of ["pointerdown", "keydown"]) document.addEventListener(type, reach, { passive: true, once: true, signal: this.accountWatch.signal });
    // A panel opened before its part of the panels module came waits for it,
    // then opens: this one, or the one drawn since, so no click is lost.
    panel.addEventListener("beforetoggle", (event) => {
      if (event.newState !== "open" || wired) return;
      event.preventDefault();
      const open = () => {
        const shown = this.$("account-panel");
        if (!shown.matches(":popover-open")) shown.showPopover();
      };
      ready().then(open, () => current() || open());
    });
    paint();
    loadSession().then(paint);
  }

  // The window's top edge is the bar's bottom edge: the height the page
  // reserves, until the bar wraps at large text and grows.
  watchHeight() {
    if (!("ResizeObserver" in window)) return;
    this.heightWatch = new ResizeObserver(([entry]) => {
      const height = entry.borderBoxSize?.[0]?.blockSize ?? entry.target.getBoundingClientRect().height;
      this.style.setProperty("--frame-height", `${height}px`);
    });
    this.heightWatch.observe(this);
  }

  // Immersive's photo is the app's, so it fills the window and stops at its
  // edges. The window is the frame's to size (the bar's height, the gutters,
  // the safe areas, the corners, none at all on a phone in landscape), so
  // the frame measures it as drawn and publishes its edges on <html> for
  // BACKDROP_CSS. It follows every resize of the window, which is fixed to
  // the viewport. render() redraws the window, so it watches the new one.
  watchWindow() {
    this.windowWatch?.disconnect();
    const canvas = this.shadowRoot.querySelector(".canvas");
    const place = () => {
      const view = document.documentElement;
      const shown = getComputedStyle(canvas).display !== "none";
      const box = canvas.getBoundingClientRect();
      const edges = shown
        ? { top: box.top, right: view.clientWidth - box.right, bottom: view.clientHeight - box.bottom, left: box.left, radius: parseFloat(getComputedStyle(canvas).borderTopLeftRadius) || 0 }
        : { top: 0, right: 0, bottom: 0, left: 0, radius: 0 };
      for (const [edge, px] of Object.entries(edges)) view.style.setProperty(`--prism-window-${edge}`, `${Math.max(0, Math.round(px * 100) / 100)}px`);
    };
    if ("ResizeObserver" in window) {
      this.windowWatch = new ResizeObserver(place);
      this.windowWatch.observe(canvas);
    }
    place();
  }

  share() {
    const frame = this;
    const { live: url, name: title, text, card } = frame.data;
    const button = frame.$("share");
    const panel = frame.$("share-panel");
    const native = typeof navigator.share === "function";
    const popover = typeof panel.showPopover === "function";
    const phone = matchMedia("(pointer: coarse)").matches;

    const say = (words) => {
      frame.$("bar-status").textContent = words;
      setTimeout(() => (frame.$("bar-status").textContent = ""), 1600);
    };
    // True when the sheet shared or the visitor dismissed it; false when it failed.
    const sheet = async () => {
      try {
        await navigator.share({ title, text, url });
        return true;
      } catch (error) {
        return error?.name === "AbortError";
      }
    };
    const copy = async () => {
      try {
        await navigator.clipboard.writeText(url);
        return true;
      } catch {
        return false;
      }
    };
    // The tick on the share icon, when there is no panel to show it in.
    const flash = async () => {
      const copied = await copy();
      button.classList.toggle("copied", copied);
      button.title = copied ? "Link copied" : `Copy the link: ${url}`;
      say(button.title);
      setTimeout(() => {
        button.classList.remove("copied");
        button.title = "Share";
      }, 1600);
    };

    if ((phone && native) || !popover) {
      // The button no longer opens the panel; the sheet, or the clipboard, does it all.
      button.removeAttribute("popovertarget");
      button.addEventListener("click", async () => {
        if (native && (await sheet())) return;
        await flash();
      });
      return;
    }

    // The panel opens under the share button, its right edge on the button's,
    // and never closer than 16 px to either side of the window.
    panel.addEventListener("beforetoggle", (event) => {
      if (event.newState !== "open") return;
      placeUnder(frame, button, panel, "share", 23, 8, "right");
      // The card is fetched the first time the panel opens, never with the page.
      if (!frame.$("share-image").firstChild) {
        const image = Object.assign(new Image(1200, 630), { src: card, alt: "", decoding: "async" });
        frame.$("share-image").append(image);
      }
    });

    frame.$("share-copy").addEventListener("click", async () => {
      const copied = await copy();
      const control = frame.$("share-copy");
      control.classList.toggle("copied", copied);
      frame.$("share-copy-label").textContent = copied ? "Copied" : "Select the link";
      say(copied ? "Link copied" : "Copy failed; select the link");
      setTimeout(() => {
        control.classList.remove("copied");
        frame.$("share-copy-label").textContent = "Copy link";
      }, 1600);
    });

    // A platform opens in a new tab; the panel closes behind it.
    for (const target of panel.querySelectorAll("a.share-target")) {
      target.addEventListener("click", () => panel.hidePopover());
    }

    if (native) {
      frame.$("share-more-item").hidden = false;
      frame.$("share-more").addEventListener("click", async () => {
        panel.hidePopover();
        if (!(await sheet())) await flash();
      });
    }
  }

  omnibox() {
    const frame = this;
    const form = frame.$("address");
    const input = frame.$("omnibox");
    const list = frame.$("omnibox-list");
    const bar = frame.shadowRoot.querySelector(".inner");
    const shown = input.value;
    const slug = shown.replace(/^holo:\/\//, "");
    const name = frame.data.name;
    const search = frame.data.home;
    // The fingerprint is read when needed: the page sets it once the holo verifies.
    const here = { get fingerprint() { return frame.getAttribute("fingerprint") ?? ""; } };
    let index = null;
    let loading = null;
    let rows = [];
    let active = 0;

    const short = (hex) => (hex.length > 12 ? `${hex.slice(0, 8)}…${hex.slice(-4)}` : hex);
    // This app's icon is the page's own; a registry entry's is the one it
    // names, or its live page's, where Prism apps keep icons/icon-192.png.
    const selfIcon = document.querySelector('link[rel="icon"]')?.href ?? "";
    const iconOf = (entry) => {
      try {
        if (typeof entry.icon === "string") return new URL(entry.icon, search).href;
        if (typeof entry.live === "string") return new URL("icons/icon-192.png", new URL(entry.live)).href;
      } catch {
        // No usable link: the tile stands in.
      }
      return "";
    };
    const fold = (text) => text.toLowerCase().replace(/[^a-z0-9]/g, "");
    // An icon is loaded only where the page's Content-Security-Policy admits
    // it, so a suggestion never logs a refusal; elsewhere its tile stands in.
    const imageAdmitted = (link) => {
      const policy = [...document.querySelectorAll('meta[http-equiv="Content-Security-Policy" i]')].map((meta) => meta.content).join(";");
      const images = /(?:^|;)\s*img-src([^;]*)/.exec(policy)?.[1] ?? /(?:^|;)\s*default-src([^;]*)/.exec(policy)?.[1];
      if (images === undefined) return true;
      const origin = new URL(link).origin;
      return (origin === location.origin && images.includes("'self'")) || images.includes(origin) || /(^|\s)(\*|https:)(\s|$)/.test(images);
    };
    // Holo Search's registry (models, holos, images, by hash), fetched the
    // first time the bar is focused, and the Prism index (holoIndex): every app
    // that verifies, with its stars.
    let apps = [];
    const list_ = (path) =>
      fetch(new URL(path, search), { cache: "no-cache" })
        .then((response) => (response.ok ? response.json() : []))
        .then((entries) => (Array.isArray(entries) ? entries : []))
        .catch(() => []);
    const load = () =>
      (loading ??= Promise.all([list_("registry/index.json"), holoIndex()]).then(([entries, known]) => {
        apps = known.apps;
        index = entries;
        frame.applyStars(known);
      }));

    // What was typed, without holo:// or the Holo Search link around it.
    const clean = (text) => {
      let query = text.trim();
      try {
        const link = new URL(query);
        if (link.href.startsWith(search) && link.searchParams.get("r")) query = link.searchParams.get("r");
      } catch {
        // Not a link.
      }
      return query.replace(/^holo:\/\//i, "").trim();
    };
    const address = (query) =>
      /^(sha256:|blake3:)?[0-9a-f]{8,64}$/i.test(query) || /^baf[a-z2-7]{20,}$/i.test(query) || /^https?:\/\//i.test(query);

    // The rows for what was typed: this app, the registry's apps that match by
    // name or hash prefix, and what was typed itself, as an address to open or
    // a search to run.
    function suggest(raw) {
      const query = raw === shown ? "" : clean(raw);
      const key = fold(query);
      const hex = query.replace(/^(sha256|blake3):/i, "").toLowerCase();
      const hashy = /^[0-9a-f]{4,64}$/.test(hex);
      const named = (text) => !key || fold(text).includes(key);
      const hashed = (digest) => hashy && Boolean(digest) && digest.startsWith(hex);
      const out = [];
      if (named(name) || named(slug) || hashed(here.fingerprint)) {
        out.push({ kind: "self", title: name, sub: `holo://${slug}`, tag: "This app", icon: selfIcon, print: here.fingerprint, repo: repoOf(frame.data.repo), stars: Number(frame.getAttribute("stars")), match: query });
      }
      // The Prism apps the index lists, by name or holo:// address, in its
      // order (most stars first); each opens its own live page. This app is
      // listed once, as this app.
      const listed = new Set([fold(name)]);
      for (const app of apps) {
        const address = app.address ?? app.name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
        if (fold(app.name) === fold(name) || address === slug) continue;
        if (query && !named(app.name) && !named(address)) continue;
        listed.add(fold(app.name));
        out.push({ kind: "app", title: app.name, sub: `holo://${address}`, tag: "App", icon: iconOf(app), print: app.fingerprint ?? "", repo: repoOf(app.repository), stars: app.stars, live: app.live, match: query });
      }
      // One row per app when matched by name, as Chrome shows a site once: its
      // holo first, then its model, then its image. A hash names one entry, so
      // a hash match shows exactly that one.
      const rank = { holo: 0, model: 1, image: 2 };
      const seen = new Map();
      let known = false;
      for (const entry of index ?? []) {
        if (entry.blake3 && entry.blake3 === here.fingerprint) continue;
        // An app the list already shows is not shown again by name.
        if (!hashed(entry.sha256) && !hashed(entry.blake3) && listed.has(fold(entry.module ?? ""))) continue;
        const byName = named(entry.module ?? "");
        const byHash = hashed(entry.sha256) || hashed(entry.blake3);
        if (query && !byName && !byHash) continue;
        const holo = Boolean(entry.blake3);
        const kind = holo ? "holo" : entry.kind === "image" ? "image" : "model";
        known ||= byHash;
        if (!byHash) {
          const kept = seen.get(fold(entry.module ?? ""));
          if (kept && rank[kept.kind] <= rank[kind]) continue;
          if (kept) out.splice(out.indexOf(kept), 1);
        }
        const row = {
          kind,
          title: entry.module ?? "Untitled",
          sub: holo ? `blake3:${short(entry.blake3)}` : `sha256:${short(entry.sha256)}`,
          tag: { holo: "App", model: "Model", image: "Image" }[kind],
          icon: iconOf(entry),
          repo: repoOf(entry.repository),
          print: entry.blake3 ?? entry.sha256 ?? "",
          target: holo ? `blake3:${entry.blake3}` : entry.sha256,
          match: byName ? query : "",
        };
        out.push(row);
        if (!byHash) seen.set(fold(entry.module ?? ""), row);
      }
      const top = out.slice(0, 6);
      // A digest keeps its algorithm and loses its middle; a long link its tail.
      const label = (text) => {
        const digest = text.match(/^((?:sha256|blake3):)?([0-9a-f]{13,})$/i);
        if (digest) return `${digest[1] ?? ""}${short(digest[2])}`;
        return text.length > 48 ? `${text.slice(0, 47)}…` : text;
      };
      const exact = hex.length === 64 && hex === here.fingerprint;
      // An address Holo Search must look up gets its own row; one the registry
      // already knows is that app's row.
      if (query && address(query) && !exact && !known) {
        top.unshift({ kind: "open", title: `Open ${label(query)}`, sub: "Verified in Holo Search", target: query });
      } else if (query && !exact && !known) {
        top.push({ kind: "search", title: query, sub: "Search Prism apps", target: query });
      }
      return top;
    }

    const icons = {
      open: '<path d="M5 12h14M13 6l6 6-6 6" />',
      search: '<circle cx="11" cy="11" r="6.5" /><path d="m20 20-4.2-4.2" />',
    };
    // A tile: the name's first letter on a hue taken from the fingerprint, so
    // an app keeps its colour wherever it is listed.
    const tile = (row) => {
      const span = Object.assign(document.createElement("span"), { className: "opt-tile", textContent: (row.title.match(/[\p{L}\p{N}]/u)?.[0] ?? "?").toUpperCase() });
      const seed = /^[0-9a-f]{4}/i.test(row.print) ? parseInt(row.print.slice(0, 4), 16) : [...row.title].reduce((sum, c) => sum * 31 + c.codePointAt(0), 7);
      span.style.setProperty("--tile-hue", String(Math.abs(seed) % 360));
      return span;
    };
    const iconFor = (row) => {
      const box = document.createElement("span");
      box.className = "opt-icon";
      if (icons[row.kind]) {
        box.innerHTML = `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">${icons[row.kind]}</svg>`;
        return box;
      }
      if (!row.icon || !imageAdmitted(row.icon)) {
        box.append(tile(row));
        return box;
      }
      const image = Object.assign(new Image(24, 24), { alt: "", decoding: "async" });
      image.addEventListener("error", () => image.replaceWith(tile(row)), { once: true });
      image.src = row.icon;
      box.append(image);
      return box;
    };

    // The title with what was typed in plain weight and the rest in bold, as
    // Chrome shows a completion. Every string from the registry goes in as text.
    const title = (text, match) => {
      const span = document.createElement("span");
      span.className = "opt-title";
      const bold = (part) => {
        const b = document.createElement("b");
        b.textContent = part;
        return b;
      };
      const at = match ? text.toLowerCase().indexOf(match.toLowerCase()) : -1;
      if (at < 0) span.append(bold(text));
      else span.append(bold(text.slice(0, at)), text.slice(at, at + match.length), bold(text.slice(at + match.length)));
      return span;
    };

    function render() {
      rows = suggest(input.value);
      active = Math.min(active, Math.max(rows.length - 1, 0));
      const items = rows.map((row, i) => {
        const li = Object.assign(document.createElement("li"), { id: `omnibox-${i}`, className: `opt opt-${row.kind}` });
        li.setAttribute("role", "option");
        li.setAttribute("aria-selected", String(i === active));
        const sub = Object.assign(document.createElement("span"), { className: "opt-sub", textContent: row.sub });
        const fill = document.createElement("span");
        fill.className = "opt-fill";
        li.append(iconFor(row), title(row.title, row.match), sub, fill);
        // Who keeps it on GitHub, and its stars, 0 included, so the column lines up.
        const owner = row.repo ? row.repo.split("/")[0] : "";
        if (owner) {
          const by = Object.assign(document.createElement("span"), { className: "opt-owner", textContent: owner, title: `github.com/${row.repo}` });
          li.append(by);
          const stars = document.createElement("span");
          stars.className = "opt-stars";
          const paint = (n) => {
            stars.classList.toggle("unknown", !Number.isInteger(n));
            if (!Number.isInteger(n)) return;
            stars.innerHTML = githubMark + starMark;
            stars.append(shortCount(n));
            stars.title = `${n} star${n === 1 ? "" : "s"} on GitHub`;
          };
          paint(Number.isInteger(row.stars) ? row.stars : null);
          li.append(stars);
        }
        li.setAttribute("aria-label", [row.title, row.sub, row.repo && `by ${row.repo.split("/")[0]}`, row.tag].filter(Boolean).join(", "));
        li.addEventListener("pointerdown", (event) => event.preventDefault());
        li.addEventListener("pointermove", () => select(i));
        li.addEventListener("click", () => choose(row));
        return li;
      });
      if (!index) {
        const note = document.createElement("li");
        note.className = "opt-note";
        note.setAttribute("role", "presentation");
        note.textContent = "Loading Prism apps";
        items.push(note);
      }
      list.replaceChildren(...items);
      list.hidden = items.length === 0;
      input.setAttribute("aria-expanded", String(!list.hidden));
      if (rows.length) input.setAttribute("aria-activedescendant", `omnibox-${active}`);
      else input.removeAttribute("aria-activedescendant");
      lookAhead();
    }

    function select(i) {
      if (!rows.length) return;
      active = (i + rows.length) % rows.length;
      for (const li of list.querySelectorAll(".opt")) li.setAttribute("aria-selected", String(li.id === `omnibox-${active}`));
      input.setAttribute("aria-activedescendant", `omnibox-${active}`);
      frame.$(`omnibox-${active}`)?.scrollIntoView({ block: "nearest" });
      lookAhead();
    }

    function close() {
      bar.classList.remove("editing");
      list.hidden = true;
      input.setAttribute("aria-expanded", "false");
      input.removeAttribute("aria-activedescendant");
      input.value = shown;
      active = 0;
    }

    // Where a row goes: this app stays here; a Prism app opens its own live
    // page; anything else opens through Holo Search, which verifies it.
    const destination = (row) => (!row || row.kind === "self" ? "" : row.kind === "app" ? row.live : `${search}?r=${encodeURIComponent(row.target)}`);
    // The active row is loaded ahead, a beat after it settles, so arrowing
    // through the list does not start a page for every row passed.
    let ahead = 0;
    const lookAhead = () => {
      clearTimeout(ahead);
      ahead = setTimeout(() => speculate(destination(rows[active])), 80);
    };

    function choose(row) {
      if (row.kind === "self") {
        input.blur();
        close();
        const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
        scrollTo({ top: 0, behavior: still ? "auto" : "smooth" });
        return;
      }
      list.hidden = true;
      location.assign(destination(row));
    }

    input.addEventListener("focus", () => {
      bar.classList.add("editing");
      input.select();
      active = 0;
      render();
      load().then(() => frame.shadowRoot.activeElement === input && render());
    });
    input.addEventListener("input", () => {
      active = 0;
      render();
    });
    input.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        if (list.hidden) render();
        select(active + (event.key === "ArrowDown" ? 1 : -1));
      } else if (event.key === "Tab" && !event.shiftKey) { close(); } else if (event.key === "Escape") {
        event.preventDefault();
        if (input.value !== shown) {
          input.value = shown;
          input.select();
          active = 0;
          render();
        } else {
          input.blur();
          close();
        }
      }
    });
    input.addEventListener("blur", close);
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const row = rows[active] ?? rows[0];
      if (row) choose(row);
      else if (clean(input.value)) choose({ kind: "search", target: clean(input.value) });
    });
  }
}

if (globalThis.customElements && !customElements.get("prism-frame")) customElements.define("prism-frame", PrismFrame);
