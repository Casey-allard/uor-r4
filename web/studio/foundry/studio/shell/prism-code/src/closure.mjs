/**
 * The browser closure `prismpm export-browser` writes. v1: `index.html`, `app.js`,
 * `app.css`, `<stem>.js`, `<stem>_bg.wasm` and the anchor `provenance.json`. v2 adds
 * the brand files and manifest (`brand.rs`); its provenance names every file digest.
 *
 * The closure is loaded from a map of file name to bytes plus a way to import
 * the glue module, so the same code serves Node (files on disk) and browsers
 * (files fetched from a URL).
 */
import { fail } from './errors.mjs';
import { sha256Hex } from './sha256.mjs';

export const PROVENANCE_SCHEMA = 'prismpm/browser-provenance/1';
export const PROVENANCE_SCHEMA_V2 = 'prismpm/browser-provenance/2';
export const FIXED_FILES = ['index.html', 'app.js', 'app.css', 'provenance.json'];
export const BRAND_FILES = Object.freeze(['Archivo-SemiBold.woff2', 'Geist-Medium.woff2', 'Geist-Regular.woff2',
  'Geist-SemiBold.woff2', 'GeistMono-Regular.woff2', 'OFL-Archivo.txt', 'OFL-Geist.txt', 'icon-512.png', 'icon.svg',
  'manifest.webmanifest']);

export function closureProfiles(stem) {
  const v1 = [...FIXED_FILES, `${stem}.js`, `${stem}_bg.wasm`].sort();
  return { 1: v1, 2: [...v1, ...BRAND_FILES].sort() };
}
const decoder = new TextDecoder('utf-8', { fatal: true });
const encoder = new TextEncoder();

export function closureStem(fileNames) {
  const wasm = fileNames.filter((n) => n.endsWith('_bg.wasm'));
  if (wasm.length !== 1) fail('closure.files', `expected exactly one *_bg.wasm, found ${JSON.stringify(wasm)}`);
  return wasm[0].slice(0, -'_bg.wasm'.length);
}

/**
 * Verify the closure's provenance against the files given. Returns the
 * provenance and a verification record; a mismatch is a finding, not an
 * exception, because hand-copied sites exist (see calculator-example's
 * `public/`), and the caller decides whether to trust them.
 */
export async function verifyClosure(files) {
  const names = [...files.keys()];
  for (const name of FIXED_FILES) if (!files.has(name)) fail('closure.files', `closure lacks ${name}`);
  const stem = closureStem(names);
  const glueName = `${stem}.js`;
  const wasmName = `${stem}_bg.wasm`;
  if (!files.has(glueName)) fail('closure.files', `closure lacks ${glueName}`);
  let provenance;
  try {
    provenance = JSON.parse(decoder.decode(files.get('provenance.json')));
  } catch (error) {
    fail('closure.provenance', `provenance.json is not JSON: ${error.message}`);
  }
  if (provenance.schema !== PROVENANCE_SCHEMA && provenance.schema !== PROVENANCE_SCHEMA_V2) {
    fail('closure.provenance', `provenance schema is ${provenance.schema}, expected ${PROVENANCE_SCHEMA} or ${PROVENANCE_SCHEMA_V2}`);
  }
  const checks = {
    adapter_wasm_sha256: await sha256Hex(files.get(wasmName)),
    binding_javascript_sha256: await sha256Hex(files.get(glueName)),
  };
  if (files.has('view-manifest.json')) checks.view_manifest_sha256 = await sha256Hex(files.get('view-manifest.json'));
  const mismatches = [];
  for (const [field, actual] of Object.entries(checks)) {
    if (provenance[field] !== actual) mismatches.push({ field, declared: provenance[field], actual });
  }
  const unverifiable = ['model_id', 'view_model_id', 'generated_core_sha256', 'adapter_lock_sha256'].filter(
    (f) => typeof provenance[f] === 'string',
  );
  if (provenance.schema === PROVENANCE_SCHEMA_V2) {
    // A supplied file must be listed and match; a listed file not supplied stays unverified.
    const listed = provenance.files && typeof provenance.files === 'object' ? provenance.files : {};
    for (const name of names) {
      if (name === 'provenance.json') continue;
      const actual = await sha256Hex(files.get(name));
      if (listed[name] !== actual) mismatches.push({ field: `files.${name}`, declared: listed[name], actual });
    }
    for (const name of Object.keys(listed)) if (!files.has(name)) unverifiable.push(`files.${name}`);
  }
  return { stem, glueName, wasmName, provenance, verification: { verified: mismatches.length === 0, mismatches, unverifiable } };
}

const NUMERIC_VERSION = '1';

/** Parse a numeric-profile request (`1\tadd\t20\t22`) the way the guest does; null when malformed. */
function numericRequest(text, operations) {
  const parts = text.split('\t');
  if (parts.length !== 4) return { error: 'malformed-request' };
  const [version, name, left, right] = parts;
  if (version !== NUMERIC_VERSION) return { error: 'unsupported-version' };
  const operation = operations.find((o) => o.request_name === name);
  if (!operation) return { error: 'unknown-operation' };
  const integer = /^(?:0|-?[1-9][0-9]*)$/;
  if (!integer.test(left) || !integer.test(right)) return { error: 'malformed-request' };
  const min = -(2n ** 63n);
  const max = 2n ** 63n - 1n;
  for (const v of [left, right]) {
    const n = BigInt(v);
    if (n < min || n > max) return { error: 'malformed-request' };
  }
  return { operation, left, right };
}

/**
 * Build `dispatch(bytes) -> bytes` over an initialized wasm-bindgen glue module.
 * Text profiles export `invoke_bytes`; the numeric profile exports only
 * `calculate(operation, left, right)`, so its wire format is reproduced here
 * from the model document's operation table (`application.view.operations`).
 */
export function closureDispatch(glue, { profile, model, limits }) {
  if (typeof glue.invoke_bytes === 'function') {
    // Requests above the request maximum get the modeled refusal from the
    // adapter, as from the guest; only the allocation bound is enforced here.
    return (request) => {
      if (limits?.guestAllocation !== undefined && request.length > limits.guestAllocation) {
        fail('core.allocation', `request of ${request.length} bytes exceeds the guest allocation maximum ${limits.guestAllocation}`);
      }
      const out = glue.invoke_bytes(request);
      return out instanceof Uint8Array ? out : new Uint8Array(out);
    };
  }
  if (typeof glue.calculate === 'function') {
    const operations = model?.application?.view?.operations;
    if (!Array.isArray(operations)) {
      fail('closure.model', 'numeric closure dispatch needs the model document (application.view.operations); pass { model }');
    }
    return (request) => {
      let text;
      try {
        text = decoder.decode(request);
      } catch {
        return encoder.encode(`${NUMERIC_VERSION}\terror\tmalformed-request`);
      }
      const parsed = numericRequest(text, operations);
      if (parsed.error) return encoder.encode(`${NUMERIC_VERSION}\terror\t${parsed.error}`);
      const answer = glue.calculate(parsed.operation.discriminant, parsed.left, parsed.right);
      if (answer && answer.kind === 'ok') return encoder.encode(`${NUMERIC_VERSION}\tok\t${answer.value}`);
      return encoder.encode(`${NUMERIC_VERSION}\terror\t${answer?.error ?? 'malformed-request'}`);
    };
  }
  fail('closure.glue', `glue module exports neither invoke_bytes nor calculate (${profile ?? 'unknown profile'})`);
}

/** Initialize a wasm-bindgen glue module with explicit wasm bytes (no fetch). */
export function initializeGlue(glue, wasmBytes) {
  if (typeof glue.initSync === 'function') {
    glue.initSync({ module: wasmBytes });
    return glue;
  }
  fail('closure.glue', 'glue module has no initSync export');
}
