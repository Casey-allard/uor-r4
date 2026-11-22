// Unsplash for the Prism frame's theme panel, as a Cloudflare Worker.
//
// The frame runs in every Prism app's page, where no secret can live and the
// page's Content-Security-Policy lets it reach only a few hosts. This worker
// is one of them: it holds the Unsplash key, searches, tells Unsplash when a
// photo is used, and passes the photos through, so the frame can show them
// as data: images on any page. Nothing here is a secret but the key.
//
//   GET  /search?q=<words>&page=<n>   nine landscape photos; no words: wallpapers
//   GET  /image?src=<images.unsplash.com URL>&w=<px>[&h=<px>][&q=<40-95>][&fm=jpg|webp|avif]   the photo's bytes
//   POST /use?id=<photo id>           Unsplash's download event, when one is chosen
//
// Deploy (frame/unsplash/README.md): wrangler secret put UNSPLASH_ACCESS_KEY,
// then wrangler deploy. No dependencies.

const API = "https://api.unsplash.com";
const IMAGES = "images.unsplash.com";
const PER_PAGE = 9;
const CORS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, POST, OPTIONS",
  "access-control-max-age": "86400",
};

const json = (body, status = 200, headers = {}) =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json; charset=utf-8", ...CORS, ...headers } });
// The widest side a photo is served at: a 5K screen's width, so any screen gets one image pixel per pixel.
const MAX_SIDE = 5120;
// The formats Unsplash renders; the caller picks one its browser decodes (jpg when it says nothing).
const FORMATS = new Set(["jpg", "webp", "avif"]);
const clamp = (text, low, high, fallback) => Math.min(Math.max(Number.parseInt(text ?? "", 10) || fallback, low), high);

// What the frame needs of a photo, and nothing else: it stores this, so the
// shape is the frame's (BACKDROPS in frame/frame.mjs).
const shape = (photo) => ({
  id: photo.id,
  image: photo.urls.raw,
  color: /^#[0-9a-f]{6}$/i.test(photo.color ?? "") ? photo.color : "#262626",
  alt: (photo.alt_description ?? photo.description ?? "").slice(0, 200),
  name: photo.user.name,
  user: photo.user.username,
  page: photo.links.html,
});

class Failure extends Error {
  constructor(message, status) {
    super(message);
    this.status = status;
  }
}

async function unsplash(env, path) {
  const response = await fetch(API + path, {
    headers: { authorization: `Client-ID ${env.UNSPLASH_ACCESS_KEY}`, "accept-version": "v1" },
  });
  // Unsplash says 403 when the hour's requests are spent.
  if (response.status === 403) throw new Failure("Unsplash's hourly limit is reached", 429);
  if (!response.ok) throw new Failure(`Unsplash answered ${response.status}`, 502);
  return response.json();
}

// A search is cached for an hour, so a popular word costs Unsplash one request.
async function search(url, env, ctx) {
  const query = (url.searchParams.get("q") ?? "").trim().replace(/\s+/g, " ").slice(0, 80);
  const page = clamp(url.searchParams.get("page"), 1, 30, 1);
  const cache = globalThis.caches?.default;
  const key = new Request(`https://prism-unsplash.cache/search?q=${encodeURIComponent(query.toLowerCase())}&page=${page}`);
  const hit = await cache?.match(key);
  if (hit) return hit;
  const found = query
    ? await unsplash(env, `/search/photos?query=${encodeURIComponent(query)}&page=${page}&per_page=${PER_PAGE}&orientation=landscape&content_filter=high`)
    : { results: await unsplash(env, `/topics/wallpapers/photos?page=${page}&per_page=${PER_PAGE}&orientation=landscape`), total_pages: 30 };
  const response = json(
    { query, page, more: page < (found.total_pages ?? 1), photos: found.results.map(shape) },
    200,
    { "cache-control": "public, max-age=3600" },
  );
  if (cache) ctx.waitUntil(cache.put(key, response.clone()));
  return response;
}

// Only Unsplash's own image host, resized there; cached at the edge for a month.
async function image(url) {
  let source;
  try {
    source = new URL(url.searchParams.get("src") ?? "");
  } catch {
    throw new Failure("src is not a URL", 400);
  }
  if (source.protocol !== "https:" || source.host !== IMAGES) throw new Failure("src is not an Unsplash image", 400);
  const width = clamp(url.searchParams.get("w"), 16, MAX_SIDE, 2400);
  const height = url.searchParams.has("h") ? clamp(url.searchParams.get("h"), 16, MAX_SIDE, 150) : null;
  const format = FORMATS.has(url.searchParams.get("fm")) ? url.searchParams.get("fm") : "jpg";
  const quality = url.searchParams.has("q") ? clamp(url.searchParams.get("q"), 40, 95, 72) : height ? 60 : 72;
  source.searchParams.set("fm", format);
  // With a height the photo is cropped to the box (a thumbnail); with a width alone it is scaled, never past
  // its own size, so asking for a screen wider than the original returns the original's every pixel.
  source.searchParams.set("fit", height ? "crop" : "max");
  source.searchParams.set("q", String(quality));
  source.searchParams.set("w", String(width));
  if (height) source.searchParams.set("h", String(height));
  const response = await fetch(source, { cf: { cacheEverything: true, cacheTtl: 2592000 } });
  if (!response.ok) throw new Failure(`the image answered ${response.status}`, 502);
  return new Response(response.body, {
    headers: { "content-type": response.headers.get("content-type") ?? "image/jpeg", "cache-control": "public, max-age=2592000, immutable", ...CORS },
  });
}

// Unsplash asks to be told when a photo is used; the frame calls this once,
// when a visitor sets a photo behind their apps.
async function use(url, env) {
  const id = url.searchParams.get("id") ?? "";
  if (!/^[\w-]{6,20}$/.test(id)) throw new Failure("id is not an Unsplash photo id", 400);
  await unsplash(env, `/photos/${id}/download`);
  return json({ ok: true });
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: CORS });
    try {
      if (request.method === "GET" && url.pathname === "/search") return await search(url, env, ctx);
      if (request.method === "GET" && url.pathname === "/image") return await image(url);
      if (request.method === "POST" && url.pathname === "/use") return await use(url, env);
      return json({ error: "not found" }, 404);
    } catch (error) {
      return json({ error: error.message }, error instanceof Failure ? error.status : 500);
    }
  },
};
