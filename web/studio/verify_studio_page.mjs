#!/usr/bin/env node
// Page-load zero-external-request verification for the Studio pages.
//
// Static companion to verify_studio_worker.mjs (whose traps cover worker network
// APIs). This scans load-time references in the Studio HTML pages: load attributes,
// loading <link rel> tokens (including preconnect/dns-prefetch), <base href>,
// inline <style> url()/@import, inline <script> request literals, and one level of
// on-origin linked stylesheets. It is a static check: it cannot see requests a
// script computes at runtime, so the runtime page-load trap remains owed (recorded
// in docs/labs/m7-studio-self-host-page-assets/README.md).
//
// Usage: node verify_studio_page.mjs [--strict-router]

import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const LOAD_ATTRS = ['src', 'srcset', 'poster', 'data'];
const LOADING_REL_TOKENS = new Set([
    'stylesheet', 'icon', 'shortcut', 'apple-touch-icon', 'manifest',
    'preload', 'prefetch', 'modulepreload', 'apple-touch-startup-image',
    'preconnect', 'dns-prefetch',
]);
const SCRIPT_URL_LITERAL_RE = /(['"])((?:https?:)?\/\/[^'"]+)\1/g;
const STYLE_URL_RE = /url\(\s*['"]?([^'")]+)['"]?\s*\)/g;
const STYLE_IMPORT_STRING_RE = /@import\s+(['"])([^'"]+)\1/g;

export function loadReferences(html, { baseDir = null, followStylesheets = false } = {}) {
    const refs = [];
    const baseRe = /<base\b([^>]*)>/gi;
    let bm;
    while ((bm = baseRe.exec(html)) !== null) {
        const href = /href\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'>]+))/i.exec(bm[1]);
        if (href) refs.push({ tag: 'base', attr: 'href', value: href[2] ?? href[3] ?? href[4] });
    }
    const tagRe = /<([a-zA-Z][a-zA-Z0-9-]*)\b([^>]*)>/g;
    let m;
    while ((m = tagRe.exec(html)) !== null) {
        const tag = m[1].toLowerCase();
        const attrs = m[2];
        const attrRe = /([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'>]+))/g;
        const parsed = {};
        let a;
        while ((a = attrRe.exec(attrs)) !== null) parsed[a[1].toLowerCase()] = a[3] ?? a[4] ?? a[5] ?? '';
        if (tag === 'link') {
            const tokens = String(parsed.rel || '').toLowerCase().split(/\s+/).filter(Boolean);
            const loading = tokens.find((t) => LOADING_REL_TOKENS.has(t));
            if (loading && parsed.href) {
                refs.push({ tag, attr: 'href', value: parsed.href, rel: loading });
                if (followStylesheets && loading === 'stylesheet' && baseDir && !isExternal(parsed.href)) {
                    const local = resolve(baseDir, parsed.href.split(/[?#]/)[0]);
                    if (existsSync(local)) {
                        const css = readFileSync(local, 'utf8');
                        for (const r of scanCss(css, local)) refs.push({ ...r, via: parsed.href });
                    }
                }
            }
            continue;
        }
        for (const attr of LOAD_ATTRS) {
            if (!parsed[attr]) continue;
            if (attr === 'srcset') {
                for (const part of parsed[attr].split(',')) {
                    const url = part.trim().split(/\s+/)[0];
                    if (url) refs.push({ tag, attr, value: url });
                }
            } else refs.push({ tag, attr, value: parsed[attr] });
        }
    }
    for (const style of html.matchAll(/<style\b[^>]*>([\s\S]*?)<\/style>/gi)) {
        for (const r of scanCss(style[1], null)) refs.push(r);
    }
    for (const script of html.matchAll(/<script\b(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/gi)) {
        // Fail-closed: any fully-written external URL literal in inline script text
        // is reported (a script could pass it to fetch/import/Worker at load). The
        // static scan cannot evaluate runtime-computed URLs; the runtime trap remains
        // owed and is recorded in the lab README.
        for (const lit of script[1].matchAll(SCRIPT_URL_LITERAL_RE)) {
            const value = lit[2];
            // Hostlike guard: prose/comment fragments that start with // are not URLs.
            const isAbsolute = /^https?:\/\//i.test(value);
            const isProtocolRelativeHost = /^\/\/[A-Za-z0-9.-]+\.[A-Za-z]{2,}/.test(value);
            if (isAbsolute || isProtocolRelativeHost) refs.push({ tag: 'script', attr: 'url-literal', value });
        }
    }
    return refs;
}

function scanCss(css, baseDir) {
    const refs = [];
    for (const m of css.matchAll(STYLE_URL_RE)) refs.push({ tag: 'css', attr: 'url()', value: m[1] });
    for (const m of css.matchAll(STYLE_IMPORT_STRING_RE)) refs.push({ tag: 'css', attr: '@import', value: m[2] });
    if (baseDir) {
        for (const m of css.matchAll(/@import\s+url\(\s*['"]?([^'")]+)['"]?\s*\)/gi)) refs.push({ tag: 'css', attr: '@import url()', value: m[1] });
    }
    return refs;
}

export function isExternal(value) {
    const v = String(value).trim();
    if (!v) return false;
    if (v.startsWith('#') || v.startsWith('data:') || v.startsWith('blob:')) return false;
    if (v.startsWith('mailto:') || v.startsWith('tel:')) return false;
    if (v.startsWith('//')) return true;
    if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(v)) return true;
    return false;
}

function scanPage(name, html, baseDir) {
    return loadReferences(html, { baseDir, followStylesheets: true })
        .filter((r) => isExternal(r.value))
        .map((r) => ({ page: name, tag: r.tag, attr: r.attr, value: r.value, ...(r.via ? { via: r.via } : {}) }));
}

function selfTest() {
    const fixture = [
        '<link rel="stylesheet" href="./local.css">',
        '<link rel="preload stylesheet" href="https://cdn.example.invalid/multi.css">',
        '<link rel="preconnect" href="https://cdn.example.invalid">',
        '<link rel="dns-prefetch" href="https://dns.example.invalid">',
        '<base href="https://base.example.invalid/app/">',
        '<script src="//cdn.example.invalid/x.js"></script>',
        '<img srcset="./a.png 1x, https://cdn.example.invalid/b.png 2x">',
        '<a href="https://github.com/UOR-Foundation/uor-r4">navigation is fine</a>',
        '<style>@import "https://css.example.invalid/evil.css"; .x{background:url(\'https://cdn.example.invalid/bg.png\')}</style>',
        '<script type="module">import x from "https://js.example.invalid/mod.js"; fetch("https://api.example.invalid/data");</script>',
        '<script>const u = ["https:", "", "api.github.com"].join("/"); window.open("https://github.com/x");</script>',
    ].join('\n');
    const found = scanPage('fixture.html', fixture, null).map((f) => f.value).sort();
    const expected = [
        '//cdn.example.invalid/x.js',
        'https://api.example.invalid/data',
        'https://base.example.invalid/app/',
        'https://cdn.example.invalid',
        'https://cdn.example.invalid/b.png',
        'https://cdn.example.invalid/bg.png',
        'https://cdn.example.invalid/multi.css',
        'https://css.example.invalid/evil.css',
        'https://dns.example.invalid',
        'https://github.com/x',
        'https://js.example.invalid/mod.js',
    ].sort();
    if (JSON.stringify(found) !== JSON.stringify(expected)) {
        console.error('FAIL: scanner self-test mismatch:', JSON.stringify({ found, expected }, null, 2));
        process.exit(1);
    }
    // Inline-script literals are fail-closed by design (see comment in loadReferences).
}

selfTest();

const strictRouter = process.argv.includes('--strict-router');
const pages = readdirSync(__dirname).filter((f) => f.endsWith('.html')).sort();
const strictPages = new Set(['index.html']);
if (strictRouter) strictPages.add('geometric_prime_router_webapp.html');

let failed = false;
const report = { strictPages: [...strictPages], clean: [], legacyResidual: [], failures: [], note: 'static scan; runtime page-load trap in verify_studio_page_load.mjs' };

for (const page of pages) {
    const html = readFileSync(resolve(__dirname, page), 'utf8');
    const offenders = scanPage(page, html, __dirname);
    if (offenders.length === 0) { report.clean.push(page); continue; }
    if (strictPages.has(page)) { failed = true; report.failures.push({ page, offenders }); }
    else report.legacyResidual.push({ page, note: 'known legacy page: load refs tracked, outside the page-load bar unless the PM rules otherwise', offenders });
}

if (failed) {
    console.error('FAIL: load-time external references in a page under the zero-external-load bar:');
    for (const f of report.failures) for (const o of f.offenders) console.error('  ' + f.page + ' ' + o.tag + '[' + o.attr + '] = ' + o.value + (o.via ? ' (via ' + o.via + ')' : ''));
    process.exit(1);
}
console.log(JSON.stringify({ verdict: 'PAGE-LOAD CHECKS PASSED', ...report }, null, 2));
