/**
 * Build provenance of a `.holo` (issue #268, design
 * `docs/proposals/268-build-provenance.md`).
 *
 * `provenanceStatement` makes the in-toto Statement v1 with a SLSA provenance
 * v1 predicate that every build writes: subjects are the four content blobs,
 * `resolvedDependencies` is derived from `prismpm.lock`, and the builder is the
 * SDK image the lock pins. `provenanceBundle` wraps it in an unsigned Sigstore
 * bundle line and `attachProvenance` appends that line to the archive's
 * attestation section (#267).
 *
 * Min mode, the default, holds nothing that varies between machines or runs, so
 * two builds of the same inputs give the same statement bytes. Max mode adds
 * the build arguments, platform, invocation id and timestamps and is opt-in.
 *
 * Nothing here signs. The bundle carries one signature with an empty `sig`,
 * which no verifier accepts; `prism attest` (#272) replaces it. Nothing here
 * verifies either; that is `prism verify` (#272, #273).
 */
import { fail } from './errors.mjs';
import { asBytes, parseArchive, resolveArchive, canonicalJson } from './archive.mjs';
import { appendAttestations, readAttestations, BUNDLE_MEDIA_TYPE, STATEMENT_TYPE, DSSE_PAYLOAD_TYPE } from './attestations.mjs';
import { blake3Hex } from './blake3.mjs';
import { sha256Hex } from './sha256.mjs';

export const PROVENANCE_PREDICATE_TYPE = 'https://slsa.dev/provenance/v1';
export const HOLO_BUILD_TYPE = 'https://uor.foundation/prism/holo-build/v1';
export const PROVENANCE_MODES = Object.freeze({ MIN: 'min', MAX: 'max' });
export const LOCK_URI = 'urn:prismpm:sdk-lock';
export const STANDARDS_LOCK_URI = 'urn:prismpm:standards-lock';
export const TOOLCHAIN_URI = 'urn:prismpm:toolchain:';
export const GENERATOR_URI = 'urn:prismpm:generator:';
const LOCK_MEDIA_TYPES = Object.freeze({
  'prismpm/sdk-lock/2': 'application/vnd.prismpm.sdk-lock.v2+json',
  'prismpm/sdk-lock/3': 'application/vnd.prismpm.sdk-lock.v3+json',
});

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });
const HEX64 = /^[0-9a-f]{64}$/;
const IMAGE_DIGEST = /@sha256:([0-9a-f]{64})$/;
const TIMESTAMP = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/;
const PLATFORM = /^[a-z0-9]+\/[a-z0-9]+(\/[a-z0-9]+)?$/;

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function parsedOf(source) {
  if (source && typeof source === 'object' && Array.isArray(source.sections) && source.bytes instanceof Uint8Array) return source;
  return parseArchive(asBytes(source));
}

function base64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

async function dualDigest(bytes) {
  return { blake3: blake3Hex(bytes), sha256: await sha256Hex(bytes) };
}

/** `sha256:<hex>` to hex, or a typed failure naming the lock member. */
function sha256Of(value, member) {
  if (typeof value !== 'string' || !value.startsWith('sha256:') || !HEX64.test(value.slice(7))) {
    fail('provenance.lock', `the lock's ${member} is not a sha256: digest`);
  }
  return value.slice(7);
}

function hexOf(value, member) {
  if (typeof value !== 'string' || !HEX64.test(value)) fail('provenance.lock', `the lock's ${member} is not 64 lowercase hex characters`);
  return value;
}

function parseLock(lock) {
  const bytes = typeof lock === 'string' ? encoder.encode(lock) : lock instanceof Uint8Array || lock instanceof ArrayBuffer || ArrayBuffer.isView(lock) ? asBytes(lock) : null;
  if (!bytes) fail('provenance.lock', 'the lock is the bytes of prismpm.lock, as a string or UTF-8 bytes');
  let document;
  try {
    document = JSON.parse(decoder.decode(bytes));
  } catch (error) {
    fail('provenance.lock', `prismpm.lock is not UTF-8 JSON: ${error.message}`);
  }
  if (!isObject(document) || !(document.schema in LOCK_MEDIA_TYPES)) {
    fail('provenance.lock', `prismpm.lock schema is ${JSON.stringify(document?.schema)}, expected prismpm/sdk-lock/2 or prismpm/sdk-lock/3`);
  }
  return { bytes, document };
}

/**
 * The resolved dependency rows a lock gives, sorted by `uri` then `name`.
 *
 * Every lock gives the lock itself (both digests of its bytes), the SDK image
 * and the standards lock. A `prismpm/sdk-lock/3` lock adds the guest world, the
 * toolchain versions and the two generator identities. `inputs.model` and
 * `inputs.spec` are not rows: they are the build's external parameters.
 */
export async function resolvedDependencies(lock) {
  const { bytes, document } = parseLock(lock);
  const rows = [
    {
      name: 'sdk-lock',
      uri: LOCK_URI,
      digest: await dualDigest(bytes),
      mediaType: LOCK_MEDIA_TYPES[document.schema],
    },
    { name: 'standards-lock', uri: STANDARDS_LOCK_URI, digest: { sha256: sha256Of(document.standards_lock, 'standards_lock') } },
  ];
  const image = typeof document.sdk_image === 'string' ? IMAGE_DIGEST.exec(document.sdk_image) : null;
  if (!image) fail('provenance.lock', "the lock's sdk_image is not an image reference pinned by @sha256:<digest>");
  rows.push({ name: 'sdk-image', uri: document.sdk_image, digest: { sha256: image[1] } });
  if (document.schema === 'prismpm/sdk-lock/3') {
    const { inputs } = document;
    if (!isObject(inputs)) fail('provenance.lock', 'a prismpm/sdk-lock/3 lock has no inputs');
    if (isObject(inputs.world)) {
      rows.push({
        name: 'world',
        uri: inputs.world.contract,
        digest: { blake3: hexOf(inputs.world.blake3, 'inputs.world.blake3'), sha256: hexOf(inputs.world.sha256, 'inputs.world.sha256') },
      });
    }
    for (const [tool, version] of Object.entries(isObject(inputs.toolchain) ? inputs.toolchain : {})) {
      if (typeof version !== 'string' || version.length === 0) fail('provenance.lock', `inputs.toolchain.${tool} is not a version string`);
      const name = tool.replaceAll('_', '-');
      rows.push({ name, uri: `${TOOLCHAIN_URI}${name}:${version}` });
    }
    const generation = isObject(inputs.generation) ? inputs.generation : {};
    for (const kind of ['application', 'library']) {
      rows.push({
        name: `${kind}-generator`,
        uri: `${GENERATOR_URI}${kind}`,
        digest: { sha256: hexOf(generation[`${kind}_generator_sha256`], `inputs.generation.${kind}_generator_sha256`) },
      });
    }
  }
  return rows.sort((a, b) => (a.uri === b.uri ? (a.name < b.name ? -1 : 1) : a.uri < b.uri ? -1 : 1));
}

function metadataOf(options) {
  const { invocationId, startedOn, finishedOn } = options;
  if (typeof invocationId !== 'string' || invocationId.length === 0) fail('provenance.input', 'max mode needs an invocationId');
  for (const [name, value] of [['startedOn', startedOn], ['finishedOn', finishedOn]]) {
    if (typeof value !== 'string' || !TIMESTAMP.test(value)) fail('provenance.input', `max mode needs ${name} as UTC RFC 3339 without fractions, for example 2026-10-25T14:47:36Z`);
  }
  return { invocationId, startedOn, finishedOn };
}

/**
 * The provenance statement of a holo.
 *
 * @param {Uint8Array | ArrayBuffer | object} source the archive, or its `parseArchive` result; its footer must verify
 * @param {object} options
 * @param {Uint8Array | string} options.lock the bytes of `prismpm.lock`, schema `/2` or `/3`
 * @param {'min' | 'max'} [options.mode] `min` unless told otherwise
 * @param {string[]} [options.arguments] max only: the command line after `prismpm`
 * @param {string} [options.platform] max only: the OCI platform of the builder, for example `linux/amd64`
 * @param {string} [options.invocationId] max only
 * @param {string} [options.startedOn] max only, UTC RFC 3339
 * @param {string} [options.finishedOn] max only, UTC RFC 3339
 * @param {{ guest?: string, view?: string, model?: string, capabilities?: string }} [options.subjectNames] the build-manifest path of each blob; the defaults are the paths `prismpm build` writes
 */
export async function provenanceStatement(source, options = {}) {
  const mode = options.mode ?? PROVENANCE_MODES.MIN;
  if (mode !== PROVENANCE_MODES.MIN && mode !== PROVENANCE_MODES.MAX) fail('provenance.mode', `provenance mode is ${JSON.stringify(mode)}, expected min or max`);
  const maxOnly = ['arguments', 'platform', 'invocationId', 'startedOn', 'finishedOn'].filter((key) => options[key] !== undefined);
  if (mode === PROVENANCE_MODES.MIN && maxOnly.length > 0) {
    fail('provenance.mode', `min mode records no ${maxOnly.join(', ')}; pass mode max to record them`);
  }
  const parsed = parsedOf(source);
  if (!parsed.verification.footer || !parsed.verification.labels) fail('provenance.archive', 'the archive does not verify; refusing to describe it');
  const resolved = resolveArchive(parsed);

  const names = {
    guest: `core-wasm/${String(resolved.provenance.cargo_name ?? '').replaceAll('-', '_')}_core_wasm.wasm`,
    view: 'view/hologram/view.holoview',
    model: 'model.prism.json',
    capabilities: 'application/capability-request.bin',
    ...options.subjectNames,
  };
  if (typeof resolved.provenance.cargo_name !== 'string' || resolved.provenance.cargo_name.length === 0) {
    if (!options.subjectNames?.guest) fail('provenance.input', 'the model extension names no cargo_name; pass subjectNames.guest');
  }
  const blobs = [
    ['guest', resolved.guest],
    ['view', resolved.view],
    ['model', resolved.model],
    ['capabilities', resolved.capabilities],
  ];
  const subject = [];
  for (const [role, blob] of blobs) {
    if (typeof names[role] !== 'string' || names[role].length === 0) fail('provenance.input', `subjectNames.${role} is not a path`);
    subject.push({ name: names[role], digest: await dualDigest(blob.content) });
  }
  if (new Set(subject.map((s) => s.name)).size !== subject.length) fail('provenance.input', 'two subjects share a name');

  const lock = parseLock(options.lock);
  const model = subject[2].digest;
  const externalParameters = { model };
  let version;
  if (lock.document.schema === 'prismpm/sdk-lock/3') {
    const locked = lock.document.inputs?.model;
    if (!isObject(locked) || locked.blake3 !== model.blake3 || locked.sha256 !== model.sha256) {
      fail('provenance.lock', "the lock's inputs.model is not the model document of this archive; the lock does not describe this build");
    }
    externalParameters.spec = lock.document.inputs.spec;
    const prismpm = lock.document.inputs.toolchain?.prismpm;
    if (typeof prismpm === 'string') version = { prismpm };
  }
  const builder = { id: lock.document.sdk_image, ...(version ? { version } : {}) };

  const predicate = {
    buildDefinition: {
      buildType: HOLO_BUILD_TYPE,
      externalParameters,
      internalParameters: {},
      resolvedDependencies: await resolvedDependencies(lock.bytes),
    },
    runDetails: { builder },
  };
  if (mode === PROVENANCE_MODES.MAX) {
    const { arguments: args, platform } = options;
    if (!Array.isArray(args) || args.some((a) => typeof a !== 'string')) fail('provenance.input', 'max mode needs arguments, an array of strings');
    if (typeof platform !== 'string' || !PLATFORM.test(platform)) fail('provenance.input', 'max mode needs platform as an OCI platform, for example linux/amd64');
    externalParameters.arguments = [...args];
    predicate.buildDefinition.internalParameters = { platform };
    predicate.runDetails.metadata = metadataOf(options);
  }
  return { _type: STATEMENT_TYPE, subject, predicateType: PROVENANCE_PREDICATE_TYPE, predicate };
}

/** The statement as the bytes a DSSE envelope signs: canonical JSON, keys sorted, no whitespace. */
export function statementBytes(statement) {
  return encoder.encode(canonicalJson(statement));
}

/**
 * One unsigned bundle line for a statement, as canonical JSON bytes. The
 * envelope payload is `statementBytes(statement)`; the signature is empty.
 */
export function provenanceBundle(statement) {
  return encoder.encode(
    canonicalJson({
      mediaType: BUNDLE_MEDIA_TYPE,
      verificationMaterial: { publicKey: {} },
      dsseEnvelope: {
        payload: base64(statementBytes(statement)),
        payloadType: DSSE_PAYLOAD_TYPE,
        signatures: [{ sig: '' }],
      },
    }),
  );
}

/** The provenance statements an archive carries, in section order, each with its mode and whether any signature is present. */
export function readProvenance(source) {
  const parsed = parsedOf(source);
  return readAttestations(parsed)
    .filter((a) => a.predicateType === PROVENANCE_PREDICATE_TYPE)
    .map((a) => {
      const predicate = a.statement.predicate ?? {};
      return {
        index: a.index,
        mode: predicate.buildDefinition?.externalParameters?.arguments === undefined ? PROVENANCE_MODES.MIN : PROVENANCE_MODES.MAX,
        signed: a.signed,
        statement: a.statement,
      };
    });
}

/**
 * Return a new archive that carries its provenance statement: the statement is
 * made by `provenanceStatement`, wrapped by `provenanceBundle` and appended to
 * the attestation section. Refuses an archive that already has one. Pure: the
 * input is not modified.
 */
export async function attachProvenance(source, options = {}) {
  const parsed = parsedOf(source);
  if (readProvenance(parsed).length > 0) fail('provenance.duplicate', 'the archive already carries a provenance statement');
  const statement = await provenanceStatement(parsed, options);
  return appendAttestations(parsed, [provenanceBundle(statement)]);
}
