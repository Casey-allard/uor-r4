// App covers: the gallery card of every app shows the app itself, captured from its own preview, with one
// sentence and one tag the visitor's model writes about it. Nothing here decides what an app is; it
// captures, checks and stores. The rules a card's words must meet are below, as data, so the host, the
// renderer and the tests read the same ones.
//
// The preview is served in a CSP sandbox with an opaque origin (sw.template.js), so this page cannot read
// its document. The capture therefore runs inside the preview: the host adds the locked html-to-image
// library and a small responder to the preview's HTML, loads the preview once more in a hidden frame of
// cover size, asks it for a picture, and accepts only an answer from that frame's own window.

export const COVER = Object.freeze({ width: 1280, height: 640, scale: 2, outWidth: 1600, outHeight: 800 });
export const CACHE = "covers";
export const EVENT = "hologram:covers";

// One sentence a first-time visitor understands: what they get first, then how the app gives it.
export const DESCRIPTION = Object.freeze({ min: 84, max: 92 });
export const TAGS = Object.freeze(["Productivity", "Finance", "Health", "Learning", "Games", "Social", "Shopping", "Travel", "Food", "Creative", "Business", "Tools"]);
const JARGON = /\b(api|apis|react|javascript|typescript|tailwind|component|components|codec|codecs|framework|webgl|webassembly|wasm|backend|frontend|server|database|llm|sdk|json|cloud|ai-powered|ultimate|seamless|seamlessly|leverage|leverages|powerful|revolutionary|cutting-edge)\b/i;

export function cardMessages(name, pageText, issues = []) {
  const brief = [
    "Write the gallery card for one app.",
    `Answer with JSON only: {"description": "...", "tag": "..."}.`,
    `description: one sentence of ${DESCRIPTION.min} to ${DESCRIPTION.max} characters, ending with a period.`,
    "Say first what the person gets from the app, then how the app gives it to them.",
    "Open with a plain verb. Use everyday words; no technical terms, no app name, no praise.",
    `tag: exactly one of ${TAGS.join(", ")}.`,
  ].join("\n");
  const seen = String(pageText || "").replace(/\s+/g, " ").trim().slice(0, 1500);
  const ask = `App name: ${name}\nText on its first screen: ${seen || "(none)"}` + (issues.length ? `\n\nYour last answer was refused: ${issues.join("; ")}. Write it again.` : "");
  return [{ role: "system", content: brief }, { role: "user", content: ask }];
}

// The model's answer, read leniently (a reasoning model may wrap the JSON in prose or a fence) and
// checked strictly. Returns { card, issues }: a card only when there are no issues.
export function readCard(answer) {
  const text = String(answer || "").replace(/<think>[\s\S]*?<\/think>/g, "");
  const m = text.match(/\{[\s\S]*\}/);
  let raw = null;
  try { raw = m ? JSON.parse(m[0]) : null; } catch { raw = null; }
  if (!raw || typeof raw !== "object") return { card: null, issues: ["no JSON object"] };
  const description = typeof raw.description === "string" ? raw.description.replace(/\s+/g, " ").trim() : "";
  const tag = TAGS.find((t) => typeof raw.tag === "string" && t.toLowerCase() === raw.tag.trim().toLowerCase()) || null;
  const issues = checkCard({ description, tag });
  return { card: issues.length ? null : { description, tag }, issues };
}

export function checkCard({ description, tag }) {
  const issues = [];
  const d = String(description || "");
  if (d.length < DESCRIPTION.min || d.length > DESCRIPTION.max) issues.push(`description is ${d.length} characters, not ${DESCRIPTION.min} to ${DESCRIPTION.max}`);
  if (!/\.$/.test(d)) issues.push("description does not end with a period");
  const j = d.match(JARGON);
  if (j) issues.push(`description uses "${j[0]}"`);
  if (!TAGS.includes(tag)) issues.push(`tag is not one of ${TAGS.join(", ")}`);
  return issues;
}

// A blank or still loading frame is near uniform: the spread of its luminance is tiny.
export function isBlank(rgba) {
  let s = 0, s2 = 0, n = 0;
  for (let i = 0; i < rgba.length; i += 16) {
    const l = 0.2126 * rgba[i] + 0.7152 * rgba[i + 1] + 0.0722 * rgba[i + 2];
    s += l; s2 += l * l; n++;
  }
  if (!n) return true;
  const mean = s / n;
  return Math.sqrt(Math.max(0, s2 / n - mean * mean)) < 6;
}

// Runs inside the preview. It answers only its parent, hides what is passing (toasts, cookie bars, chat
// bubbles: small fixed elements low on the screen) and draws the top of the page at cover size.
const RESPONDER = `(() => {
  const hide = () => {
    const vw = innerWidth, vh = innerHeight;
    for (const el of document.querySelectorAll("body *")) {
      const cs = getComputedStyle(el);
      if (cs.position !== "fixed" && cs.position !== "sticky") continue;
      const r = el.getBoundingClientRect(), area = (r.width * r.height) / (vw * vh);
      const passing = el.matches('[role="alert"],[role="status"],[class*="toast" i],[class*="snack" i],[class*="cookie" i],[id*="cookie" i],[class*="consent" i]');
      if (passing || (area > 0 && area < 0.2 && r.top > vh * 0.6)) el.style.setProperty("visibility", "hidden", "important");
    }
  };
  addEventListener("message", async (e) => {
    const m = e.data;
    if (e.source !== parent || !m || m.type !== "hologram/cover-request") return;
    try {
      if (document.fonts) await document.fonts.ready;
      hide();
      // A browser paints the body's background over the whole viewport; a drawing of the root element does
      // not, so the ground is named explicitly.
      const clear = (c) => !c || c === "transparent" || /rgba\\([^)]*,\\s*0\\)$/.test(c);
      const rootBg = getComputedStyle(document.documentElement).backgroundColor, bodyBg = document.body ? getComputedStyle(document.body).backgroundColor : "";
      const backgroundColor = !clear(rootBg) ? rootBg : !clear(bodyBg) ? bodyBg : "#ffffff";
      const canvas = await htmlToImage.toCanvas(document.documentElement, { width: m.width, height: m.height, pixelRatio: m.scale, skipAutoScale: true, backgroundColor, style: { overflow: "hidden" } });
      parent.postMessage({ type: "hologram/cover", id: m.id, png: canvas.toDataURL("image/png"), text: (document.body.innerText || "").slice(0, 4000), title: document.title }, "*");
    } catch (error) {
      parent.postMessage({ type: "hologram/cover", id: m.id, error: String(error && error.message || error) }, "*");
    }
  });
})();`;

const scriptSafe = (s) => String(s).replace(/<\/script/gi, "<\\/script");

// The preview's HTML with the library and the responder, placed last so the app is unchanged before them.
export function withResponder(html, library) {
  const tail = `<script>${scriptSafe(library)}</script><script>${scriptSafe(RESPONDER)}</script>`;
  const at = html.lastIndexOf("</body>");
  return at < 0 ? html + tail : html.slice(0, at) + tail + html.slice(at);
}

// ---- browser only: the capture and the store.

// Loads `url` in a hidden frame of cover size and asks it for one picture. Resolves { png, text, title }
// or null when the frame never answers, answers with an error, or shows only a blank screen twice.
export async function capture(url, { settle = 1200, retry = 2500, timeout = 20000 } = {}) {
  const frame = document.createElement("iframe");
  frame.setAttribute("sandbox", "allow-scripts allow-forms");
  frame.setAttribute("aria-hidden", "true");
  frame.tabIndex = -1;
  // Inside the viewport and fully transparent: Chromium stops rendering a frame placed off screen, and a
  // frame that does not render never answers.
  frame.style.cssText = `position:fixed;left:0;top:0;width:${COVER.width}px;height:${COVER.height}px;border:0;opacity:0;z-index:-1;pointer-events:none;`;
  const loaded = new Promise((resolve) => frame.addEventListener("load", resolve, { once: true }));
  frame.src = url;
  document.body.appendChild(frame);
  try {
    await Promise.race([loaded, delay(timeout)]);
    for (const wait of [settle, retry]) {
      await delay(wait);
      const shot = await ask(frame, timeout);
      if (!shot || !shot.png) return null;
      if (!(await blankPng(shot.png))) return shot;
    }
    return null;
  } finally {
    frame.remove();
  }
}

let asked = 0;
function ask(frame, timeout) {
  const id = ++asked;
  return new Promise((resolve) => {
    const done = (value) => { removeEventListener("message", on); clearTimeout(timer); resolve(value); };
    const on = (e) => {
      if (e.source !== frame.contentWindow || !e.data || e.data.type !== "hologram/cover" || e.data.id !== id) return;
      done(e.data.error || typeof e.data.png !== "string" || !e.data.png.startsWith("data:image/png;base64,") ? null : e.data);
    };
    const timer = setTimeout(() => done(null), timeout);
    addEventListener("message", on);
    frame.contentWindow.postMessage({ type: "hologram/cover-request", id, width: COVER.width, height: COVER.height, scale: COVER.scale }, "*");
  });
}

const delay = (ms) => new Promise((r) => setTimeout(r, ms));
async function bitmapOf(png) { return createImageBitmap(await (await fetch(png)).blob()); }
async function blankPng(png) {
  const bmp = await bitmapOf(png);
  const c = new OffscreenCanvas(96, 48);
  c.getContext("2d").drawImage(bmp, 0, 0, 96, 48);
  return isBlank(c.getContext("2d").getImageData(0, 0, 96, 48).data);
}

// The stored cover: WebP at the card's double size, cropped from the top of the capture.
export async function encodeCover(png) {
  const bmp = await bitmapOf(png);
  const c = new OffscreenCanvas(COVER.outWidth, COVER.outHeight);
  const g = c.getContext("2d");
  g.imageSmoothingQuality = "high";
  const sh = Math.min(bmp.height, (bmp.width * COVER.outHeight) / COVER.outWidth);
  g.drawImage(bmp, 0, 0, bmp.width, sh, 0, 0, COVER.outWidth, COVER.outHeight);
  return c.convertToBlob({ type: "image/webp", quality: 0.9 });
}

const keyOf = (base, appId, part) => new URL(`${CACHE}/${encodeURIComponent(appId)}.${part}`, base).href;

export async function forget(base, appId) {
  const cache = await caches.open(CACHE);
  await cache.delete(keyOf(base, appId, "webp"));
  await cache.delete(keyOf(base, appId, "json"));
  dispatchEvent(new CustomEvent(EVENT, { detail: { appId } }));
}
