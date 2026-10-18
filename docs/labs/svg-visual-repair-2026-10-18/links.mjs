// Local Markdown targets, heading anchors, HTML images and responsive sources.
// Uses marked as a QA dependency only. Run from the repository root.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const { marked } = await import(process.env.MARKED_MODULE ? pathToFileURL(process.env.MARKED_MODULE).href : 'marked');
const files = ['README.md', 'docs/geometry.md', 'docs/architecture.md', 'docs/bookkeeping.md', 'docs/labs/svg-visual-repair-2026-10-18/README.md'];
const anchors = new Map();
function ids(file) {
  if (anchors.has(file)) return anchors.get(file);
  const text = fs.readFileSync(file, 'utf8'), found = new Set(), used = new Map();
  marked.walkTokens(marked.lexer(text), t => {
    if (t.type !== 'heading') return;
    const s = t.text.replace(/<[^>]*>/g, '').replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1').replace(/&amp;/g, '&').toLowerCase();
    const slug = [...s].filter(c => c === '-' || c === '_' || !/[\p{P}\p{S}]/u.test(c)).join('').replace(/ /g, '-');
    const count = used.get(slug) || 0;
    found.add(count ? `${slug}-${count}` : slug); used.set(slug, count + 1);
  });
  for (const m of text.matchAll(/\b(?:id|name)=["']([^"']+)/g)) found.add(m[1]);
  anchors.set(file, found); return found;
}
const failures = []; let checked = 0;
for (const file of files) {
  const body = fs.readFileSync(file, 'utf8'), links = [];
  marked.walkTokens(marked.lexer(body), t => {
    if (t.type === 'link' || t.type === 'image') links.push(t.href);
    if (t.type === 'html') for (const m of t.text.matchAll(/\b(?:href|src|srcset)=["']([^"']+)/g)) links.push(m[1]);
  });
  for (const href of links) {
    if (/^(?:[a-z]+:|\/\/)/i.test(href)) continue;
    const [rel, fragment] = href.split('#');
    const target = rel ? path.resolve(path.dirname(file), decodeURIComponent(rel.split('?')[0])) : path.resolve(file);
    checked++;
    if (!fs.existsSync(target)) failures.push({ file, href, reason: 'missing target' });
    else if (fragment && target.endsWith('.md') && !ids(target).has(decodeURIComponent(fragment))) failures.push({ file, href, reason: 'missing heading anchor' });
  }
}
console.log(JSON.stringify({ files, checked, failures }, null, 2));
if (failures.length) process.exitCode = 1;
