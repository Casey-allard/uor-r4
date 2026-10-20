# Website Standards

## Information Architecture

The website follows a clear hierarchy:

```
/ (homepage)
/namespaces/<prefix>/   ← 33 namespace landing pages
/search.html            ← full-text search
/docs/                  ← documentation (from uor-docs)
/css/style.css
/js/search.js
/search-index.json
/sitemap.xml
/uor.foundation.jsonld    ← from uor-build
/uor.foundation.ttl     ← from uor-build
/uor.foundation.nt      ← from uor-build
/uor.term.ebnf          ← from uor-build (Amendment 42)
```

## Navigation

- Primary navigation: namespace grid on homepage, sidebar on namespace pages.
- Every page links back to the homepage.
- Search is reachable from every page.
- Breadcrumb navigation on deep pages.

## No External Dependencies

The website must function without any external network requests:
- No CDN-hosted CSS frameworks (Bootstrap, Tailwind, etc.)
- No CDN-hosted JavaScript libraries (jQuery, etc.)
- No external fonts (use system font stack)
- No analytics or tracking scripts
- No third-party embeds

## Performance

- Total page weight (HTML + CSS + JS) must be < 150 KB uncompressed.
- No render-blocking scripts in `<head>` (use `defer` or `async`).
- CSS is a single file: `css/style.css`.
- JavaScript is a single file: `js/search.js`.

## Search

- `search-index.json` contains all 474 class labels, 948 property labels, 3601 individual labels.
- Client-side search parses `search-index.json` and filters on input.
- Search results link to the relevant namespace page.
- Search works without JavaScript disabled (falls back to sitemap).

## Namespace Pages

- Every namespace page is 100% auto-generated from `uor_ontology::Ontology::full()`.
- Content is never hand-edited — always regenerated from spec.
- Pages include: namespace IRI, label, comment, space classification, imports, classes table, properties table, individuals table.
- Namespace pages include a link to the corresponding documentation reference page.

## Sitemap

- `sitemap.xml` lists all pages in the website.
- Format: XML Sitemap Protocol 0.9.
- All URLs use the production base URL `https://uor.foundation/`.

## CSS Requirements

- Single stylesheet: `css/style.css`
- No `!important` except where technically necessary (max 5 occurrences)
- Responsive layout with `@media` breakpoints for mobile (≤768px), tablet (≤1024px), desktop
- Syntax: parseable by `cssparser`

## References

- [XML Sitemap Protocol](https://www.sitemaps.org/protocol.html)
- [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/)
