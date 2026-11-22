/**
 * Capability sets, deny by default (issue #279, docs/proposals/279-capability-sets.md).
 *
 * A capability file, `prism/capabilities/1`, grants a holo outbound hosts,
 * key-value stores, variables and providers (#294). Nothing is granted unless an `allow-` entry
 * of the resource's kind matches it and no `deny-` entry does. The runtime
 * applies the set named `default` unless the caller names another; with no
 * file at all, every resource and every guest import is denied.
 *
 * The rules here are `@prism-code/spec`'s `schemas/capabilities/capabilities-v1.schema.json`
 * in code; a test runs the schema's fixtures through `parseCapabilities`, so
 * the two cannot drift.
 */
import { CapabilityDeniedError, fail } from './errors.mjs';

export const CAPABILITIES_SCHEMA = 'prism/capabilities/1';
export const DEFAULT_SET = 'default';
export const CAPABILITY_KINDS = Object.freeze(['outbound-hosts', 'key-value-stores', 'variables', 'providers']);

const SET_NAME = /^[a-z][a-z0-9-]*$/;
const ENTRY = Object.freeze({
  'outbound-hosts': /^([*]|[a-z][a-z0-9+.-]*):\/\/([*]|([*][.])?[a-z0-9]([a-z0-9-]*[a-z0-9])?([.][a-z0-9]([a-z0-9-]*[a-z0-9])?)*)(:([*]|[0-9]{1,5}))?$/,
  'key-value-stores': /^([*]|[a-z0-9][a-z0-9_-]*)$/,
  variables: /^([*]|[a-z][a-z0-9_]*)$/,
  providers: /^([*]|[a-z][a-z0-9-]*)$/,
});
const MEMBERS = CAPABILITY_KINDS.flatMap((kind) => [`allow-${kind}`, `deny-${kind}`]);
const DEFAULT_PORTS = Object.freeze({ http: '80', https: '443', ws: '80', wss: '443' });

const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const invalid = (message) => fail('capabilities.invalid', message);

function checkSet(name, set) {
  if (!isObject(set)) invalid(`set ${name} must be an object`);
  for (const member of Object.keys(set)) {
    if (!MEMBERS.includes(member)) invalid(`set ${name} has an unknown member ${member}; a set has ${MEMBERS.join(', ')}`);
    const entries = set[member];
    const kind = member.slice(member.indexOf('-') + 1);
    if (!Array.isArray(entries)) invalid(`${name}.${member} must be an array`);
    for (const entry of entries) {
      if (typeof entry !== 'string' || !ENTRY[kind].test(entry)) invalid(`${name}.${member} has a malformed entry ${JSON.stringify(entry)}`);
    }
    if (new Set(entries).size !== entries.length) invalid(`${name}.${member} repeats an entry`);
  }
}

/**
 * Validate a capability file (its text, bytes or parsed value) and return it,
 * deeply frozen. Throws `PrismCodeError` `capabilities.invalid` naming the
 * first member that is wrong; an unknown member is always an error.
 */
export function parseCapabilities(source) {
  let value = source;
  if (source instanceof Uint8Array) value = new TextDecoder('utf-8', { fatal: true }).decode(source);
  if (typeof value === 'string') {
    try {
      value = JSON.parse(value);
    } catch (error) {
      invalid(`capability file is not JSON: ${error.message}`);
    }
  }
  if (!isObject(value)) invalid('a capability file is a JSON object');
  for (const member of Object.keys(value)) {
    if (member !== 'schema' && member !== 'sets') invalid(`unknown member ${member}; a capability file has schema and sets`);
  }
  if (value.schema !== CAPABILITIES_SCHEMA) invalid(`schema must be ${CAPABILITIES_SCHEMA}, not ${JSON.stringify(value.schema)}`);
  if (!isObject(value.sets)) invalid('sets must be an object of named sets');
  if (!Object.hasOwn(value.sets, DEFAULT_SET)) invalid(`sets has no ${DEFAULT_SET} set`);
  for (const [name, set] of Object.entries(value.sets)) {
    if (!SET_NAME.test(name)) invalid(`set name ${JSON.stringify(name)} must match ${SET_NAME.source}`);
    checkSet(name, set);
  }
  return Object.freeze({
    schema: value.schema,
    sets: Object.freeze(
      Object.fromEntries(Object.entries(value.sets).map(([name, set]) => [name, Object.freeze(Object.fromEntries(Object.entries(set).map(([m, e]) => [m, Object.freeze([...e])])))])),
    ),
  });
}

function parseHostEntry(entry) {
  const [scheme, rest] = entry.split('://');
  const colon = rest.indexOf(':');
  return colon < 0 ? { scheme, host: rest, port: null } : { scheme, host: rest.slice(0, colon), port: rest.slice(colon + 1) };
}

function hostMatches(entry, url) {
  const e = parseHostEntry(entry);
  if (e.scheme !== '*' && e.scheme !== url.scheme) return false;
  if (e.host.startsWith('*.')) {
    if (!url.host.endsWith(e.host.slice(1))) return false;
  } else if (e.host !== '*' && e.host !== url.host) return false;
  if (e.port === '*') return true;
  if (e.port === null) return url.port === (DEFAULT_PORTS[url.scheme] ?? '');
  return e.port === url.port;
}

/** A requested URL as scheme, host and port (the default port when it has none), or null. */
function requestedUrl(resource) {
  let url;
  try {
    url = new URL(resource);
  } catch {
    return null;
  }
  const scheme = url.protocol.slice(0, -1);
  if (!url.hostname) return null;
  return { scheme, host: url.hostname.toLowerCase(), port: url.port || DEFAULT_PORTS[scheme] || '' };
}

function matches(kind, entry, resource) {
  if (kind === 'outbound-hosts') return hostMatches(entry, resource);
  return entry === '*' || entry === resource;
}

/**
 * The grant a runtime enforces: one set of a parsed capability file, or
 * nothing at all when `capabilities` is null. `allows(kind, resource)` is
 * true only when an allow entry matches and no deny entry does; `assert`
 * throws `CapabilityDeniedError` instead of returning false.
 *
 * @param {object | null} capabilities the result of `parseCapabilities`, or null
 * @param {string} [setName]
 */
export function grantOf(capabilities, setName = DEFAULT_SET) {
  if (capabilities !== null && capabilities !== undefined && !Object.hasOwn(capabilities.sets, setName)) {
    invalid(`the capability file has no set ${setName}; it has ${Object.keys(capabilities.sets).join(', ')}`);
  }
  const set = capabilities ? capabilities.sets[setName] : {};
  const list = (member) => set[member] ?? [];
  const allows = (kind, resource) => {
    if (!CAPABILITY_KINDS.includes(kind)) fail('capabilities.kind', `unknown capability kind ${kind}; the kinds are ${CAPABILITY_KINDS.join(', ')}`);
    if (typeof resource !== 'string') return false;
    const subject = kind === 'outbound-hosts' ? requestedUrl(resource) : resource;
    if (subject === null) return false;
    return list(`allow-${kind}`).some((e) => matches(kind, e, subject)) && !list(`deny-${kind}`).some((e) => matches(kind, e, subject));
  };
  const name = capabilities ? setName : null;
  return Object.freeze({
    /** The set in force, or null when no capability file was given. */
    set: name,
    /** The members of that set (frozen), or null when no capability file was given (#283 compares the world with them). */
    members: capabilities ? set : null,
    allows,
    /** Throws `CapabilityDeniedError` (`capabilities.<kind>`) unless `allows(kind, resource)`. */
    assert(kind, resource) {
      if (allows(kind, resource)) return;
      throw new CapabilityDeniedError(
        name === null ? `${kind} ${JSON.stringify(resource)} is denied: no capability file grants anything` : `${kind} ${JSON.stringify(resource)} is denied by capability set ${name}`,
        { check: `capabilities.${kind}`, details: { kind, resource, set: name } },
      );
    },
    /**
     * Throws `CapabilityDeniedError` (`capabilities.import`) for a guest import
     * the grant does not link. The world `hologram:guest/core-wasm@1` lists no
     * imports, so no import names a capability kind and every import is denied.
     */
    linkImport(entry) {
      throw new CapabilityDeniedError(
        `guest import ${entry.module}.${entry.name} (${entry.kind}) is denied: the world hologram:guest/core-wasm@1 lists no imports, so no capability set can grant one`,
        { check: 'capabilities.import', details: { import: { module: entry.module, name: entry.name, kind: entry.kind }, set: name } },
      );
    },
  });
}

/** The grant when the caller gives no capability file: nothing. */
export const NO_CAPABILITIES = grantOf(null);
