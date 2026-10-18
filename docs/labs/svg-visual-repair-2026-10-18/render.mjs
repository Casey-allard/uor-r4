// Documentation-only browser QA. Dependencies are development tools, not product
// dependencies: marked and playwright-core (Chromium supplied by the operator).
// Run from the repository root. Optional MODULE paths support an existing install.
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';

const load = async (env, name) => import(process.env[env] ? pathToFileURL(process.env[env]).href : name);
const { marked } = await load('MARKED_MODULE', 'marked');
const { chromium } = await load('PLAYWRIGHT_MODULE', 'playwright-core');
const sha256 = data => createHash('sha256').update(data).digest('hex');
const root = process.cwd();
const output = process.env.PREVIEW_DIR || path.join(root, 'local/svg-review/embedded');
fs.mkdirSync(output, { recursive: true });
const before = '9fd1b8baec6a7586a2595cd4be94f6df77fd1c66';
const versions = ['before', 'after'];
const sources = {
  'architecture': 'README.md',
  'quaternion-rotation': 'docs/geometry.md',
  '600-cell-icosians': 'docs/geometry.md',
  'zeta-phases': 'docs/geometry.md',
  'connections': 'docs/geometry.md',
  'hopf-fibration': 'docs/geometry.md',
  'geometry-stack': 'docs/history/geometry-float-form-2026-10.md',
};
const css = `*{box-sizing:border-box}body{margin:0;background:#f6f8fa;color:#1f2328}
article{max-width:1012px;margin:24px auto;padding:32px;border:1px solid #d0d7de;border-radius:6px;background:white;font:16px/1.5 -apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;overflow-wrap:break-word}
h1,h2,h3,h4{font-weight:600;line-height:1.25;margin:24px 0 16px}h1{font-size:32px}h2{font-size:24px}h3{font-size:20px}h1,h2{padding-bottom:.3em;border-bottom:1px solid #d8dee4}p,ul,ol,table,pre,details{margin-top:0;margin-bottom:16px}ul,ol{padding-left:2em}li+li{margin-top:.25em}a{color:#0969da;text-decoration:none}img{max-width:100%;height:auto;vertical-align:middle}picture{display:block}pre{padding:16px;background:#f6f8fa;overflow:auto;border-radius:6px;font-size:85%}code{font:85% ui-monospace,SFMono-Regular,Consolas,monospace;background:#afb8c133;border-radius:6px;padding:.2em .4em}pre code{padding:0;background:none}table{display:block;max-width:100%;overflow:auto;border-spacing:0;border-collapse:collapse}td,th{padding:6px 13px;border:1px solid #d0d7de}tr:nth-child(2n){background:#f6f8fa}blockquote{margin:0 0 16px;padding:0 1em;border-left:.25em solid #d0d7de;color:#57606a}hr{height:.25em;border:0;background:#d0d7de;margin:24px 0}summary{cursor:pointer}@media(max-width:600px){article{margin:0;padding:16px;border:0;border-radius:0}}`;

function read(version, file) {
  const safe = path.resolve(root, file);
  if (!safe.startsWith(root + path.sep)) throw new Error('Outside repository');
  return version === 'before' ? execFileSync('git', ['show', `${before}:${file}`], { maxBuffer: 16e6 }) : fs.readFileSync(safe);
}
const server = http.createServer((req, res) => {
  try {
    const parts = decodeURIComponent(new URL(req.url, 'http://localhost').pathname).slice(1).split('/');
    const version = parts.shift();
    if (!versions.includes(version)) throw new Error('Unknown version');
    const file = parts.join('/');
    let body = read(version, file);
    if (file.endsWith('.md')) {
      body = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><style>${css}</style><article class="markdown-body">${marked.parse(body.toString(), { gfm: true })}</article>`;
      res.setHeader('Content-Type', 'text/html; charset=utf-8');
    } else res.setHeader('Content-Type', file.endsWith('.svg') ? 'image/svg+xml' : file.endsWith('.png') ? 'image/png' : 'application/octet-stream');
    res.end(body);
  } catch { res.writeHead(404); res.end('Not found'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
const results = { before, content_width_desktop: 946, viewports: [1280, 390], embedded: [], svg: [] };
try {
  for (const version of versions) for (const width of results.viewports) {
    const page = await browser.newPage({ viewport: { width, height: 1000 }, deviceScaleFactor: 1 });
    await page.route('**/*', route => new URL(route.request().url()).origin === base ? route.continue() : route.abort());
    for (const file of new Set(Object.values(sources))) {
      await page.goto(`${base}/${version}/${file}`);
      for (const [name, source] of Object.entries(sources).filter(([, source]) => source === file)) {
        const img = page.locator(`img[src$="/${name}.svg"]`).first();
        await img.evaluate(e => e.decode());
        await img.scrollIntoViewIfNeeded();
        await img.evaluate(e => {
          const block = e.closest('picture') || e.closest('p') || e;
          const prev = block.previousElementSibling;
          const context = (prev?.matches('img,picture') || prev?.querySelector('img')) ? block : (prev || block);
          scrollTo(0, Math.max(0, context.getBoundingClientRect().top + scrollY - 16));
        });
        const details = await img.evaluate(e => ({ src: new URL(e.currentSrc).pathname, width: e.width, height: e.height, naturalWidth: e.naturalWidth, naturalHeight: e.naturalHeight, alt: e.alt, pageOverflow: document.documentElement.scrollWidth > innerWidth }));
        const rect = await img.boundingBox();
        const article = await page.locator('article').boundingBox();
        const top = await img.evaluate(e => {
          const block = e.closest('picture') || e.closest('p') || e;
          const prev = block.previousElementSibling;
          return ((prev?.matches('img,picture') || prev?.querySelector('img')) ? block : (prev || block)).getBoundingClientRect().top;
        });
        const scroll = await page.evaluate(() => scrollY);
        const clip = { x: article.x, y: Math.max(0, top + scroll - 8), width: article.width, height: rect.y + rect.height + 70 - Math.max(0, top - 8) };
        const preview = `${name}-${version}-${width}.png`;
        await page.screenshot({ path: path.join(output, preview), fullPage: true, clip });
        const pixels = fs.readFileSync(path.join(output, preview));
        const capture_height = pixels.readUInt32BE(20);
        if (capture_height < details.height) throw new Error(`${preview}: screenshot truncates the figure`);
        results.embedded.push({ version, page: file, viewport: width, figure: name, preview, capture_height, preview_sha256: sha256(pixels), ...details });
      }
    }
    await page.close();
  }
  // Native SVG accessibility, vector-only content and actual browser text bounds.
  const figures = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', 'docs/figures/*.svg', 'docs/figures/geometry/*.svg'], { encoding: 'utf8' }).trim().split('\n');
  const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
  for (const file of figures) {
    await page.goto(`${base}/after/${file}`);
    const bounds = await page.evaluate(async () => {
      await document.fonts.ready;
      const svg = document.querySelector('svg'), v = svg.viewBox.baseVal;
      const clipped = [...svg.querySelectorAll('text')].flatMap(t => {
        if (!t.textContent.trim()) return [];
        const b = t.getBBox(), m = svg.getCTM().inverse().multiply(t.getCTM());
        const pts = [[b.x, b.y], [b.x + b.width, b.y], [b.x, b.y + b.height], [b.x + b.width, b.y + b.height]].map(([x, y]) => new DOMPoint(x, y).matrixTransform(m));
        return pts.some(p => p.x < v.x - .5 || p.y < v.y - .5 || p.x > v.x + v.width + .5 || p.y > v.y + v.height + .5) ? [t.textContent] : [];
      });
      return { viewBox: [v.width, v.height], clipped, raster: svg.querySelectorAll('image,foreignObject').length, role: svg.getAttribute('role'), title: svg.querySelector('title')?.textContent, description: Boolean(svg.querySelector('desc')?.textContent) };
    });
    results.svg.push({ file, sha256: sha256(fs.readFileSync(file)), ...bounds });
  }
  results.preserved_geometry = [];
  for (const name of ['quaternion-rotation', '600-cell-icosians', 'zeta-phases', 'hopf-fibration', 'prime-addressing', 'connections']) {
    const hashes = {};
    for (const version of versions) {
      await page.goto(`${base}/${version}/docs/figures/geometry/${name}.svg`);
      const primitives = await page.evaluate(() => [...document.querySelector('svg').children].filter(e => !['text', 'title', 'desc'].includes(e.localName)).map(e => e.outerHTML).join('\n'));
      hashes[version] = sha256(primitives);
    }
    results.preserved_geometry.push({ figure: name, ...hashes, equal: hashes.before === hashes.after });
  }
  await page.goto(`${base}/after/docs/figures/geometry/zeta-phases.svg`);
  results.zeta_label_overlaps = await page.evaluate(() => {
    const ts = [...document.querySelectorAll('text')].filter(t => +t.getAttribute('y') > 80 && +t.getAttribute('y') < 425);
    const overlaps = [];
    for (let i = 0; i < ts.length; i++) for (let j = i + 1; j < ts.length; j++) {
      const a = ts[i].getBBox(), b = ts[j].getBBox();
      if (Math.min(a.x + a.width, b.x + b.width) > Math.max(a.x, b.x) && Math.min(a.y + a.height, b.y + b.height) > Math.max(a.y, b.y)) overlaps.push([ts[i].textContent, ts[j].textContent]);
    }
    return overlaps;
  });
  await page.close();
  fs.writeFileSync(path.join(output, 'render-results.json'), JSON.stringify(results, null, 2) + '\n');
  console.log(`${results.embedded.length} actual Markdown-embedded captures; ${results.svg.length} SVGs checked. Output: ${output}`);
  if (results.embedded.some(r => r.pageOverflow || !r.alt) || results.svg.some(r => r.clipped.length || r.raster || r.role !== 'img' || !r.title || !r.description) || results.preserved_geometry.some(r => !r.equal) || results.zeta_label_overlaps.length) process.exitCode = 1;
} finally {
  await browser.close();
  server.close();
}
