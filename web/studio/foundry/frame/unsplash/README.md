# prism-unsplash

The Cloudflare Worker behind the theme panel's Unsplash search. It holds the
Unsplash key, so no page carries it; the frame (`frame/frame.mjs`,
`UNSPLASH_PROXY`) is its only client.

| Route | What it does |
| --- | --- |
| `GET /search?q=<words>&page=<n>` | Nine landscape photos; with no words, Unsplash's wallpapers. Cached an hour. |
| `GET /image?src=<images.unsplash.com URL>&w=<px>[&h=<px>][&q=<40-95>][&fm=jpg\|webp\|avif]` | The photo's bytes, resized by Unsplash, cached a month. Any other host is refused. Up to 5120 px; a width alone scales without ever enlarging past the original, a height too crops to the box. Quality defaults to 72 (60 for a box), format to jpg. |
| `POST /use?id=<photo id>` | Unsplash's download event, sent when a visitor sets a photo. |

## Deploy

```bash
cd frame/unsplash
npx wrangler login
npx wrangler secret put UNSPLASH_ACCESS_KEY
npx wrangler deploy
```

`wrangler deploy` prints the worker's address,
`https://prism-unsplash.<subdomain>.workers.dev/`; today it is
`https://prism-unsplash.trinity-03d.workers.dev/`. That address is
`UNSPLASH_PROXY` in `frame/frame.mjs`. The page build reads its origin from
the published frame into every app page's `connect-src`, so an app offers
search once it is next built.

## Limits

An Unsplash app in demo mode may make 50 API requests an hour (searches and
download events; images do not count). Searches are cached, but a busy week
needs Unsplash's production approval, which raises it to 5,000.
