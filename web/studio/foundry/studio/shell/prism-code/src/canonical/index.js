/**
 * prism-code/canonical — deterministic canonicalization and content addressing
 * for source code. This is prism-code 0.1.0, unchanged: two files that differ
 * only in line endings, BOM, trailing whitespace or trailing blank lines get
 * the same hash.
 */
import { fail } from '../errors.mjs';
import { sha256HexSync } from '../sha256.mjs';

const BOM = '﻿';
const encoder = new TextEncoder();

/**
 * The digest as hex. `node:crypto` where the host has one, reached without an
 * import so that this module and `prism-code` load in a browser. Elsewhere
 * SHA-256 is computed here, because WebCrypto's digest is async and this API is
 * not, and no other algorithm is available.
 */
function digestHex(algorithm, input) {
  const crypto = globalThis.process?.getBuiltinModule?.('node:crypto');
  if (crypto) return crypto.createHash(algorithm).update(input, 'utf8').digest('hex');
  if (algorithm.toLowerCase() !== 'sha256') fail('canonical.algorithm', `${algorithm} needs node:crypto, which this host does not have; sha256 is available`);
  return sha256HexSync(encoder.encode(input));
}

/**
 * Reduce source text to its canonical form.
 *
 * Normalizes `\r\n` and lone `\r` to `\n`, drops a leading BOM, trims trailing
 * whitespace per line and collapses trailing blank lines into a single final
 * newline. Input with no content — `''` or only newlines — canonicalizes to
 * `''`, never to a lone newline.
 */
export function canonicalize(source, options = {}) {
  const { eol = '\n', stripTrailingWhitespace = true, finalNewline = true, stripBom = true } = options;
  let out = source;
  if (stripBom && out.startsWith(BOM)) out = out.slice(BOM.length);
  out = out.replace(/\r\n?/g, '\n');
  if (stripTrailingWhitespace) out = out.replace(/[ \t]+$/gm, '');
  const body = out.replace(/\n+$/, '');
  out = body === '' ? '' : finalNewline ? `${body}\n` : body;
  if (eol === '\r\n' && out !== '') out = out.replace(/\n/g, '\r\n');
  return out;
}

/**
 * Content address of source text.
 *
 * Two inputs that differ only in line endings, BOM, trailing whitespace or
 * trailing blank lines produce the same hash by default.
 */
export function codeHash(source, options = {}) {
  const { algorithm = 'sha256', canonical = true, prefix = true, ...canonicalizeOptions } = options;
  const input = canonical ? canonicalize(source, canonicalizeOptions) : source;
  const hex = digestHex(algorithm, input);
  return prefix ? `${algorithm}:${hex}` : hex;
}

/**
 * Cheap structural facts about source text, including its content address.
 */
export function describe(source, options = {}) {
  const hadBom = source.startsWith(BOM);
  const body = hadBom ? source.slice(BOM.length) : source;
  const lf = (body.match(/\n/g) ?? []).length;
  const crlf = (body.match(/\r\n/g) ?? []).length;
  const cr = (body.match(/\r(?!\n)/g) ?? []).length;
  let eol;
  if (body === '') eol = 'none';
  else if (crlf > 0 && lf === crlf && cr === 0) eol = 'crlf';
  else if (crlf === 0 && cr === 0) eol = 'lf';
  else eol = 'mixed';
  return {
    bytes: encoder.encode(source).length,
    lines: body === '' ? 0 : body.replace(/\n$/, '').split('\n').length,
    eol,
    hadBom,
    finalNewline: /\n$/.test(body),
    hash: codeHash(source, options),
  };
}

/**
 * Whether two sources differ in substance rather than in formatting.
 *
 * Returns `false` when the inputs share a canonical form — the check to run
 * before reporting a formatting-only change as a real change.
 */
export function differs(a, b, options = {}) {
  return canonicalize(a, options) !== canonicalize(b, options);
}
