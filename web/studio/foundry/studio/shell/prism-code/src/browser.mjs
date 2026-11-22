/**
 * prism-code/browser — open a `.holo` fetched over HTTP, open a closure by URL,
 * or mount a closure in an iframe. Mounting a HOLOVIEW page in the browser
 * needs a service worker for `/_hologram/intent` and is not in this release.
 */
import { loadArchive, loadClosure } from './index.mjs';
import { fail } from './errors.mjs';

const CLOSURE_FILES = ['index.html', 'app.js', 'app.css', 'provenance.json'];

async function fetchBytes(url) {
  const response = await fetch(url);
  if (!response.ok) fail('load.fetch', `${url} answered ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
}

/**
 * Open an application by URL. A URL ending in `.holo` is fetched as an
 * archive. Any other URL is treated as a closure directory: its
 * `provenance.json` names nothing about the stem, so `options.stem` (the
 * `<stem>_bg.wasm` prefix) must be given, or `options.files` listing the file
 * names, or the glue is discovered from `app.js`'s import line.
 */
export async function load(source, options = {}) {
  const base = new URL(source, typeof document !== 'undefined' ? document.baseURI : undefined);
  const grant = { capabilities: options.capabilities ?? null, capabilitySet: options.capabilitySet, allowUnverified: options.allowUnverified === true };
  if (base.pathname.endsWith('.holo')) return loadArchive(await fetchBytes(base), grant);
  const directory = base.href.endsWith('/') ? base : new URL(`${base.href}/`);
  const names = new Set(CLOSURE_FILES);
  let stem = options.stem ?? null;
  if (!stem) {
    const appJs = new TextDecoder().decode(await fetchBytes(new URL('app.js', directory)));
    const match = appJs.match(/import\(['"]\.\/([^'"]+)\.js['"]\)|from\s*['"]\.\/([^'"]+)\.js['"]/);
    stem = match ? match[1] ?? match[2] : null;
  }
  if (!stem) fail('closure.files', 'cannot determine the closure stem; pass { stem }');
  names.add(`${stem}.js`);
  names.add(`${stem}_bg.wasm`);
  for (const extra of options.files ?? []) names.add(extra);
  const files = new Map();
  await Promise.all(
    [...names].map(async (name) => {
      try {
        files.set(name, await fetchBytes(new URL(name, directory)));
      } catch (error) {
        if (CLOSURE_FILES.includes(name) || name.startsWith(stem)) throw error;
      }
    }),
  );
  // The glue is at a URL known only now; without the comment webpack looks it up among its own modules and fails.
  return loadClosure(files, (name) => import(/* webpackIgnore: true */ new URL(name, directory).href), { model: options.model ?? null, ...grant });
}

/** Place a closure's `index.html` in an iframe inside `container`; the page's own wasm does the work. */
export function mount(container, closureUrl, { title = 'Prism application' } = {}) {
  if (typeof document === 'undefined') fail('mount.environment', 'mount() needs a DOM');
  const frame = document.createElement('iframe');
  frame.src = new URL(closureUrl, document.baseURI).href;
  frame.title = title;
  frame.setAttribute('sandbox', 'allow-scripts allow-same-origin');
  frame.style.border = '0';
  frame.style.width = '100%';
  frame.style.height = '100%';
  container.appendChild(frame);
  return frame;
}
