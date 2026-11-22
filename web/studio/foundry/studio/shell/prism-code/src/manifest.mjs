/**
 * The `prism.toml` manifest (issue #280, docs/proposals/280-prism-toml.md).
 *
 * A manifest names the application, says where its model comes from, grants
 * it capabilities and declares the variables it reads:
 *
 *   [application]  name, version (semver), description, authors
 *   [model]        source, build, watch
 *   [capabilities] the members of one `prism/capabilities/1` set, inline,
 *                  or `file` (and `set`) naming a capability file; any other
 *                  table declares a capability a provider provisions (#294)
 *   [variables]    name = "literal" | "env(NAME)" | { value, secret }
 *
 * Per-target overlays, `prism.browser.toml` and `prism.node.toml`, are merged
 * into the main manifest before it is checked: a table merges key by key, and
 * any other value, arrays included, replaces the one it overrides.
 *
 * The parsed manifest is a plain object that `@prism-code/spec`'s
 * `schemas/manifest/manifest-v1.schema.json` describes; a test runs the
 * schema's fixtures through `validateManifest`, so the two cannot drift.
 * The lint rules a schema cannot state (a secret written as a literal, a
 * path that leaves the project) are here, and are errors.
 */
import { fail } from './errors.mjs';
import { parseToml } from './toml.mjs';
import { parseCapabilities, grantOf, CAPABILITIES_SCHEMA, CAPABILITY_KINDS, DEFAULT_SET } from './capabilities.mjs';

export const MANIFEST_FILE = 'prism.toml';
export const MANIFEST_TARGETS = Object.freeze(['browser', 'node']);
export const MANIFEST_VERSION = 1;

const NAME = /^[a-z][a-z0-9-]{0,62}$/;
// Semantic Versioning 2.0.0, the regular expression semver.org gives.
const SEMVER =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
const VARIABLE = /^[a-z][a-z0-9_]*$/;
const SET_NAME = /^[a-z][a-z0-9-]*$/;
const ENV_REFERENCE = /^env\(([A-Za-z_][A-Za-z0-9_]*)\)$/;
const SET_MEMBERS = CAPABILITY_KINDS.flatMap((kind) => [`allow-${kind}`, `deny-${kind}`]);
const CAPABILITY_MEMBERS = Object.freeze(['file', 'set', ...SET_MEMBERS]);
// No hyphen: a misspelt allow-variables stays an error, and a name maps to one environment prefix.
const CAPABILITY_NAME = /^[a-z][a-z0-9_]*$/;
const PROVIDER_TYPE = /^[a-z][a-z0-9-]*$/;
const PROVIDER_OPTION = /^[a-z][a-z0-9_-]*$/;
const CONTROL_CHARACTER = /[\u0000-\u001f\u007f]/;

/**
 * Literals that are credentials whatever key holds them. Each pattern is
 * specific enough that a false match is a string nobody should commit anyway.
 */
const CREDENTIAL_PATTERNS = Object.freeze([
  ['a private key', /-----BEGIN [A-Z ]*PRIVATE KEY-----/],
  ['a GitHub token', /\b(gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{22,})\b/],
  ['an AWS access key id', /\b(AKIA|ASIA)[0-9A-Z]{16}\b/],
  ['a Slack token', /\bxox[abprs]-[A-Za-z0-9-]{10,}\b/],
  ['an API secret key', /\bsk-[A-Za-z0-9_-]{20,}\b/],
  ['a Google API key', /\bAIza[0-9A-Za-z_-]{35}\b/],
]);

const isTable = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const invalid = (message, details) => fail('manifest.invalid', message, details);

function onlyMembers(where, value, allowed) {
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) invalid(`${where} has an unknown member ${key}; it has ${allowed.join(', ')}`, { at: where, member: key });
  }
}

function string(where, value, { pattern, describe } = {}) {
  if (typeof value !== 'string' || value === '') invalid(`${where} must be a non-empty string`, { at: where });
  if (pattern && !pattern.test(value)) invalid(`${where} ${JSON.stringify(value)} must be ${describe}`, { at: where });
  return value;
}

function stringList(where, value, check = (w, v) => string(w, v)) {
  if (!Array.isArray(value)) invalid(`${where} must be an array of strings`, { at: where });
  value.forEach((item, i) => check(`${where}[${i}]`, item));
  if (new Set(value).size !== value.length) invalid(`${where} repeats an entry`, { at: where });
  return value;
}

/** A path inside the project: relative, forward slashes, never `..`. */
function projectPath(where, value) {
  string(where, value);
  if (/^([A-Za-z]:|[/\\])/.test(value) || value.includes('\\')) invalid(`${where} ${JSON.stringify(value)} must be a relative path with forward slashes`, { at: where });
  if (value.split('/').includes('..')) invalid(`${where} ${JSON.stringify(value)} must stay inside the project; it has a .. segment`, { at: where });
  return value;
}

function checkApplication(value) {
  if (!isTable(value)) invalid('the manifest has no [application] table', { at: 'application' });
  onlyMembers('application', value, ['name', 'version', 'description', 'authors']);
  string('application.name', value.name, { pattern: NAME, describe: 'lower case letters, digits and -, starting with a letter, at most 63 characters' });
  string('application.version', value.version, { pattern: SEMVER, describe: 'a semantic version such as 0.1.0' });
  if (value.description !== undefined && typeof value.description !== 'string') invalid('application.description must be a string', { at: 'application.description' });
  if (value.authors !== undefined) stringList('application.authors', value.authors);
}

function checkModel(value) {
  if (!isTable(value)) invalid('[model] must be a table', { at: 'model' });
  onlyMembers('model', value, ['source', 'build', 'watch']);
  if (value.source !== undefined) projectPath('model.source', value.source);
  if (value.build !== undefined) string('model.build', value.build);
  if (value.watch !== undefined) stringList('model.watch', value.watch, projectPath);
}

function checkProviderOption(where, value) {
  if (typeof value === 'boolean' || Number.isInteger(value)) return;
  if (typeof value !== 'string') invalid(`${where} must be a string, an integer, a boolean or an array of those`, { at: where });
  // cmd.exe ends a command line at a newline, and an npm shim on Windows is cmd.exe.
  if (CONTROL_CHARACTER.test(value)) invalid(`${where} holds a control character, which a command line cannot carry safely`, { at: where });
  if (ENV_REFERENCE.test(value)) invalid(`${where} is env(NAME), and every process on the machine can read a command line; the provider reads its secrets from its own environment`, { at: where });
}

function checkProviderCapability(name, value) {
  const where = `capabilities.${name}`;
  if (!CAPABILITY_NAME.test(name)) invalid(`capabilities has an unknown member ${name}; it has ${CAPABILITY_MEMBERS.join(', ')}, or a capability named ${CAPABILITY_NAME.source}`, { at: where, member: name });
  if (!isTable(value)) invalid(`${where} must be a table with a provider`, { at: where });
  onlyMembers(where, value, ['provider']);
  if (!isTable(value.provider)) invalid(`${where} has no provider = { type, options }`, { at: `${where}.provider` });
  onlyMembers(`${where}.provider`, value.provider, ['type', 'options']);
  string(`${where}.provider.type`, value.provider.type, { pattern: PROVIDER_TYPE, describe: `a provider type, ${PROVIDER_TYPE.source}` });
  const { options } = value.provider;
  if (options === undefined) return;
  if (!isTable(options)) invalid(`${where}.provider.options must be a table`, { at: `${where}.provider.options` });
  for (const [key, option] of Object.entries(options)) {
    const at = `${where}.provider.options.${key}`;
    if (!PROVIDER_OPTION.test(key)) invalid(`${at} must be named ${PROVIDER_OPTION.source}, because it becomes a flag`, { at });
    if (Array.isArray(option)) option.forEach((item, i) => checkProviderOption(`${at}[${i}]`, item));
    else checkProviderOption(at, option);
  }
}

function checkCapabilities(value) {
  if (!isTable(value)) invalid('[capabilities] must be a table', { at: 'capabilities' });
  for (const [name, entry] of Object.entries(value)) {
    if (!CAPABILITY_MEMBERS.includes(name)) checkProviderCapability(name, entry);
  }
  const inline = SET_MEMBERS.filter((m) => Object.hasOwn(value, m));
  if (value.file !== undefined) {
    projectPath('capabilities.file', value.file);
    if (inline.length > 0) invalid(`[capabilities] names a file and also lists ${inline.join(', ')}; use one or the other`, { at: 'capabilities' });
  }
  if (value.set !== undefined) {
    if (value.file === undefined) invalid('capabilities.set names a set of a capability file, so it needs capabilities.file', { at: 'capabilities.set' });
    string('capabilities.set', value.set, { pattern: SET_NAME, describe: `a set name, ${SET_NAME.source}` });
  }
  // The inline members are a capability set; the capability reader checks their entries.
  if (inline.length > 0) parseCapabilities({ schema: CAPABILITIES_SCHEMA, sets: { [DEFAULT_SET]: Object.fromEntries(inline.map((m) => [m, value[m]])) } });
}

function checkVariables(value) {
  if (!isTable(value)) invalid('[variables] must be a table', { at: 'variables' });
  for (const [name, entry] of Object.entries(value)) {
    const where = `variables.${name}`;
    if (!VARIABLE.test(name)) invalid(`variable name ${JSON.stringify(name)} must match ${VARIABLE.source}, as a capability set names variables`, { at: where });
    if (typeof entry === 'string') continue;
    if (!isTable(entry)) invalid(`${where} must be a string or a table { value, secret }`, { at: where });
    onlyMembers(where, entry, ['value', 'secret']);
    if (entry.value !== undefined && typeof entry.value !== 'string') invalid(`${where}.value must be a string`, { at: `${where}.value` });
    if (entry.secret !== undefined && typeof entry.secret !== 'boolean') invalid(`${where}.secret must be true or false`, { at: `${where}.secret` });
    if (entry.secret !== true && entry.value === undefined) invalid(`${where} has no value; give one, or mark it secret = true to read it from the environment`, { at: where });
  }
}

/**
 * Check a parsed manifest (after overlays are merged) against the manifest
 * rules the schema states: tables, members, types and patterns. Throws
 * `PrismCodeError` `manifest.invalid` naming the first member that is wrong.
 */
export function validateManifest(manifest) {
  if (!isTable(manifest)) invalid('a manifest is a table');
  onlyMembers('the manifest', manifest, ['manifest_version', 'application', 'model', 'capabilities', 'variables']);
  if (manifest.manifest_version !== undefined && manifest.manifest_version !== MANIFEST_VERSION) {
    invalid(`manifest_version ${JSON.stringify(manifest.manifest_version)} is not one this reader knows; it reads ${MANIFEST_VERSION}`, { at: 'manifest_version' });
  }
  checkApplication(manifest.application);
  if (manifest.model !== undefined) checkModel(manifest.model);
  if (manifest.capabilities !== undefined) checkCapabilities(manifest.capabilities);
  if (manifest.variables !== undefined) checkVariables(manifest.variables);
  return manifest;
}

function* strings(value, path) {
  if (typeof value === 'string') yield [path, value];
  else if (Array.isArray(value)) for (const [i, item] of value.entries()) yield* strings(item, `${path}[${i}]`);
  else if (isTable(value)) for (const [key, item] of Object.entries(value)) yield* strings(item, path ? `${path}.${key}` : key);
}

/**
 * The lint rules: what a schema cannot say. Returns `{ errors, warnings }`,
 * each a list of `{ rule, at, message }`. Errors:
 *
 * - `secret-literal`: a variable marked `secret = true` whose value is not
 *   `env(NAME)`. A secret is read from the environment, never written down.
 * - `credential`: any string in the manifest that is a credential by its
 *   shape (a private key, a GitHub, AWS, Slack or API token).
 *
 * Warnings:
 *
 * - `variable-not-granted`: a declared variable the capability set does not
 *   allow, so the holo can never read it. Only checked for inline sets; a
 *   capability file is read at run time.
 * - `provider-not-granted`: a declared provider the capability set does not
 *   allow, so it can never run. Also only checked for inline sets.
 */
export function lintManifest(manifest) {
  const errors = [];
  const warnings = [];
  for (const [name, entry] of Object.entries(manifest.variables ?? {})) {
    if (isTable(entry) && entry.secret === true && entry.value !== undefined && !ENV_REFERENCE.test(entry.value)) {
      errors.push({ rule: 'secret-literal', at: `variables.${name}.value`, message: `variables.${name} is a secret, so its value must be env(NAME), not a literal` });
    }
  }
  for (const [at, value] of strings(manifest, '')) {
    for (const [what, pattern] of CREDENTIAL_PATTERNS) {
      if (pattern.test(value)) errors.push({ rule: 'credential', at, message: `${at} holds what looks like ${what}; declare a secret variable and read it with env(NAME)` });
    }
  }
  const caps = manifest.capabilities;
  if (!caps || caps.file === undefined) {
    const grant = grantOf(capabilitiesOfInline(caps), DEFAULT_SET);
    for (const { name, type } of manifestProviders(manifest)) {
      if (!grant.allows('providers', type)) warnings.push({ rule: 'provider-not-granted', at: `capabilities.${name}`, message: `capabilities.${name} needs provider ${type}, and [capabilities] does not allow it, so it cannot run` });
    }
    for (const name of Object.keys(manifest.variables ?? {})) {
      if (!grant.allows('variables', name)) warnings.push({ rule: 'variable-not-granted', at: `variables.${name}`, message: `variables.${name} is declared but [capabilities] does not allow it, so the holo cannot read it` });
    }
  }
  return { errors, warnings };
}

function capabilitiesOfInline(caps) {
  const members = SET_MEMBERS.filter((m) => caps && Object.hasOwn(caps, m));
  return { schema: CAPABILITIES_SCHEMA, sets: { [DEFAULT_SET]: Object.fromEntries(members.map((m) => [m, caps[m]])) } };
}

/**
 * Merge an overlay into a manifest: a table merges key by key, and any other
 * value, arrays included, replaces the one it overrides. Neither input is
 * changed.
 */
export function mergeOverlay(base, overlay) {
  if (!isTable(overlay)) fail('manifest.overlay', 'an overlay is a table');
  const out = { ...base };
  for (const [key, value] of Object.entries(overlay)) {
    out[key] = isTable(value) && isTable(base[key]) ? mergeOverlay(base[key], value) : value;
  }
  return out;
}

function deepFreeze(value) {
  if (Array.isArray(value) || isTable(value)) {
    for (const item of Object.values(value)) deepFreeze(item);
    Object.freeze(value);
  }
  return value;
}

function textOf(source, label) {
  if (source === undefined || source === null) return null;
  if (typeof source === 'string' || source instanceof Uint8Array) return parseToml(source);
  if (isTable(source)) return source;
  fail('manifest.invalid', `${label} must be TOML text, UTF-8 bytes or a parsed table`);
}

/**
 * Read a manifest: parse `prism.toml` (text, bytes or a parsed table), merge
 * the overlay for `target` when one is given, validate, and lint. A lint
 * error throws `PrismCodeError` `manifest.secret`; lint warnings are returned
 * on the result. Returns a deeply frozen object:
 *
 *   { manifest, target, warnings }
 *
 * @param {string | Uint8Array | object} source the text of prism.toml
 * @param {{ target?: string, overlay?: string | Uint8Array | object | null }} [options]
 */
export function parseManifest(source, { target = null, overlay = null } = {}) {
  if (target !== null && !MANIFEST_TARGETS.includes(target)) fail('manifest.overlay', `target ${JSON.stringify(target)} is not one of ${MANIFEST_TARGETS.join(', ')}`);
  if (overlay !== null && target === null) fail('manifest.overlay', 'an overlay was given without the target it is for');
  let manifest = textOf(source, 'the manifest');
  if (manifest === null) fail('manifest.invalid', 'no manifest was given');
  const layer = textOf(overlay, `prism.${target}.toml`);
  if (layer !== null) {
    if (Object.hasOwn(layer, 'manifest_version')) fail('manifest.overlay', `prism.${target}.toml may not set manifest_version; the main manifest does`);
    manifest = mergeOverlay(manifest, layer);
  }
  validateManifest(manifest);
  const { errors, warnings } = lintManifest(manifest);
  if (errors.length > 0) {
    fail('manifest.secret', errors.map((e) => e.message).join('; '), { errors });
  }
  return deepFreeze({ manifest: structuredClone(manifest), target, warnings });
}

/**
 * The capability grant a manifest makes. An inline set is the `default` set
 * of a one-set capability file. A `file` reference is returned unread, as
 * `{ file, set }`, for the caller to open relative to the manifest. With no
 * `[capabilities]` table the manifest grants nothing.
 *
 * @returns {{ capabilities: object | null, capabilitySet: string, file: string | null }}
 */
export function manifestCapabilities(manifest) {
  const caps = manifest.capabilities;
  if (!caps) return { capabilities: null, capabilitySet: DEFAULT_SET, file: null };
  if (caps.file !== undefined) return { capabilities: null, capabilitySet: caps.set ?? DEFAULT_SET, file: caps.file };
  return { capabilities: parseCapabilities(capabilitiesOfInline(caps)), capabilitySet: DEFAULT_SET, file: null };
}

/**
 * The providers a manifest declares, in order of capability name: `{ name,
 * type, options, flags }`. `flags` are the options as command-line arguments,
 * `--<key>=<value>`, sorted by key; an array repeats the flag. Running them is
 * `resolveProviders` in `prism-code/node`.
 *
 * @param {object} manifest a validated manifest
 */
export function manifestProviders(manifest) {
  const caps = manifest.capabilities ?? {};
  const names = Object.keys(caps).filter((name) => !CAPABILITY_MEMBERS.includes(name)).sort();
  return Object.freeze(
    names.map((name) => {
      const { type, options = {} } = caps[name].provider;
      const flags = Object.keys(options)
        .sort()
        .flatMap((key) => [options[key]].flat().map((value) => `--${key}=${value}`));
      return Object.freeze({ name, type, options, flags: Object.freeze(flags) });
    }),
  );
}

/**
 * Resolve the manifest's variables against an environment. A literal is its
 * own value; `env(NAME)` reads `env[NAME]`; a secret with no value reads the
 * variable's name in upper case. An `env(NAME)` that is undefined throws
 * `PrismCodeError` `manifest.env`, naming the variable and the environment
 * name but never a value. Returns a frozen object of name to value.
 *
 * @param {object} manifest a validated manifest
 * @param {Record<string, string | undefined>} env usually `process.env`
 */
export function resolveVariables(manifest, env) {
  const out = {};
  for (const [name, entry] of Object.entries(manifest.variables ?? {})) {
    const value = typeof entry === 'string' ? entry : (entry.value ?? `env(${name.toUpperCase()})`);
    const reference = ENV_REFERENCE.exec(value);
    if (!reference) {
      out[name] = value;
      continue;
    }
    const resolved = env?.[reference[1]];
    if (resolved === undefined) {
      fail('manifest.env', `variables.${name} reads env(${reference[1]}), and ${reference[1]} is not set`, { variable: name, env: reference[1] });
    }
    out[name] = resolved;
  }
  return Object.freeze(out);
}
