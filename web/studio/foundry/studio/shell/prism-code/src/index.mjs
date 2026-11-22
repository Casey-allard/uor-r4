/**
 * prism-code — the JavaScript runtime for Prism SDK (PrismPM) applications.
 *
 * `load(source)` opens what PrismPM generates, a Hologram v4 `.holo` archive or
 * the six-file browser closure, verifies its identities, and returns an
 * application object whose `dispatch(bytes)` runs the modeled guest and whose
 * `acceptance()` replays the modeled vectors. Nothing here compiles, proves or
 * produces an artifact; those stay in the SDK image.
 */
import { parseArchive, resolveArchive, asBytes } from './archive.mjs';
import { attestationSection, readAttestations } from './attestations.mjs';
import { decodeHoloview } from './view.mjs';
import { compileGuest, checkRequest, instantiateGuest, runInstance, runAcceptance } from './core.mjs';
import { isSessionContract, openSession, runSessionAcceptance, runStep } from './session.mjs';
import { parseCapabilities, grantOf, DEFAULT_SET } from './capabilities.mjs';
import { verifyClosure, closureDispatch, initializeGlue } from './closure.mjs';
import { CapabilityDeniedError, DigestMismatchError, PrismCodeError, ReferenceInvalidError, fail } from './errors.mjs';
import { inspect } from './inspect.mjs';
import { experimental_verify } from './verify.mjs';
import { acceptanceHistory, experimental_recorder, experimental_replay } from './replay.mjs';
import { createMiddlewareStack } from './middleware.mjs';
import { worldPolicy } from './vendor/verify.mjs';
import { createContainers } from './containers.mjs';
import { createNetworks } from './networks.mjs';
import { createVolumes } from './volumes.mjs';
import { createEventBus, createSystem } from './system.mjs';
import { verify as verifyNamespace } from './verify-namespace.mjs';
import { errors as errorsNamespace } from './errors-namespace.mjs';
import * as canonical from './canonical/index.js';

export {
  PrismError,
  PrismCodeError,
  VerificationError,
  ProofMismatchError,
  CapabilityDeniedError,
  WorldMismatchError,
  DeterminismViolationError,
  ReplayError,
  SigningError,
  SignatureInvalidError,
  IdentityMismatchError,
  TransparencyError,
  SubjectMismatchError,
  AttestationNotFoundError,
  TrustRootError,
  RegistryError,
  RegistryAuthError,
  ReferrersUnsupportedError,
  ManifestNotFoundError,
  DigestMismatchError,
  PullVerificationError,
  ReferenceInvalidError,
  ProviderError,
  ProviderNotFoundError,
  ProviderReplyError,
  ProviderFailedError,
  ExportMismatchError,
  MiddlewareError,
  ERROR_CLASSES,
  PolicyParseError,
  PolicyEvaluationError,
  PolicyDeniedError,
  classForCheck,
  errorForReason,
  errorFromVerdict,
  EXIT_CODES,
  exitCodeFor,
} from './errors.mjs';
export { experimental_verify, experimental_createClient, MINIMUM_LEVEL_ENV } from './verify.mjs';
export {
  HISTORY_SCHEMA,
  SESSION_HISTORY_SCHEMA,
  historyEvents,
  REPLAY_PREDICATE_TYPE,
  parseHistory,
  historyBytes,
  historyDigest,
  historyJson,
  acceptanceHistory,
  experimental_recorder,
  experimental_replay,
} from './replay.mjs';
export { CONTAINER_SCHEMA, memoryContainerStore } from './containers.mjs';
export { parseArchive, resolveArchive, canonicalJson } from './archive.mjs';
export { decodeHoloview, encodeHoloview } from './view.mjs';
export { compileGuest, runGuest, runAcceptance } from './core.mjs';
export { SESSION_CONTRACT, decodeResponses, openSession, runStep, runSessionAcceptance } from './session.mjs';
export { MIDDLEWARE_STEPS } from './middleware.mjs';
export { experimental_tracing } from './tracing.mjs';
export { parseCapabilities, grantOf, CAPABILITIES_SCHEMA, CAPABILITY_KINDS, DEFAULT_SET } from './capabilities.mjs';
export {
  parseManifest,
  validateManifest,
  lintManifest,
  mergeOverlay,
  manifestCapabilities,
  manifestProviders,
  resolveVariables,
  MANIFEST_FILE,
  MANIFEST_TARGETS,
  MANIFEST_VERSION,
} from './manifest.mjs';
export { parseToml } from './toml.mjs';
export { verifyClosure, closureDispatch, initializeGlue } from './closure.mjs';
export { blake3, blake3Hex, kappa } from './blake3.mjs';
export { inspect, formatInspection, INSPECTION_SCHEMA } from './inspect.mjs';
export {
  ATTESTATION_EXTENSION,
  BUNDLE_MEDIA_TYPE,
  attestationSection,
  readAttestations,
  decodeBundle,
  appendAttestations,
  stripAttestations,
  encodeArchive,
} from './attestations.mjs';
export {
  PROVENANCE_PREDICATE_TYPE,
  HOLO_BUILD_TYPE,
  PROVENANCE_MODES,
  resolvedDependencies,
  provenanceStatement,
  statementBytes,
  provenanceBundle,
  attachProvenance,
  readProvenance,
} from './provenance.mjs';

const decoder = new TextDecoder('utf-8', { fatal: true });

function isNode() {
  return typeof process !== 'undefined' && process.versions != null && process.versions.node != null;
}

function limitsOf(application) {
  return {
    request: application.request_maximum,
    response: application.response_maximum,
    guestAllocation: application.guest_allocation_maximum,
  };
}

function vectorsOf(application) {
  const list = Array.isArray(application.acceptance_vectors) ? application.acceptance_vectors : [];
  return list.map((v) => ({ request: Uint8Array.from(v.request), response: Uint8Array.from(v.response) }));
}

/**
 * The bounds of a session guest (#629): the request is the allocation bound,
 * and the session member of the application declares the rest.
 */
function sessionLimitsOf(application) {
  const session = application.session ?? {};
  return {
    request: application.request_maximum,
    guestAllocation: application.request_maximum,
    response: session.response_maximum,
    responseCount: session.response_count_maximum,
    state: session.state_maximum,
  };
}

/** The acceptance of a session guest: ordered sessions of `{ request, responses }` steps (#629). */
function sessionsOf(application) {
  const list = Array.isArray(application.acceptance_sessions) ? application.acceptance_sessions : [];
  return list.map((steps) =>
    steps.map((s) => ({ request: Uint8Array.from(s.request), responses: s.responses.map((r) => Uint8Array.from(r)) })),
  );
}

function profileOf(modelDocument) {
  return modelDocument.application?.profile ?? modelDocument.schema ?? null;
}

function evidenceOf(provenance) {
  return {
    leanManifestSha256: provenance.lean_manifest_sha256 ?? null,
    lcnfManifestSha256: provenance.lcnf_manifest_sha256 ?? null,
    lexleanCommit: provenance.lexlean_commit ?? null,
    lean4ProdCommit: provenance.lean4_prod_commit ?? null,
    hologramLiveCommit: provenance.hologram_live_commit ?? null,
    uorHologramCommit: provenance.uor_hologram_commit ?? null,
    generatedCoreSha256: provenance.generated_core_sha256 ?? null,
    compilerSemanticsId: provenance.compiler_semantics_id ?? null,
    stdlibSemanticsId: provenance.stdlib_semantics_id ?? null,
  };
}

/** Run one step of `app` through its middleware stack, around the runtime's own `terminal`. */
function stepper(stack, application) {
  return (step, terminal, args) => stack.resolve(step, terminal, { application: application() })(args);
}

/**
 * The grant for `options.capabilities` (a capability file as text, bytes or a
 * parsed value) and `options.capabilitySet`. No file grants nothing (#279).
 */
function grantFor({ capabilities = null, capabilitySet = DEFAULT_SET } = {}) {
  if (capabilities === null || capabilities === undefined) {
    if (capabilitySet !== DEFAULT_SET) fail('capabilities.invalid', `capability set ${capabilitySet} was named, but no capability file was given`);
    return grantOf(null);
  }
  return grantOf(parseCapabilities(capabilities), capabilitySet);
}

/**
 * The static world-within-policy check (issue #283): the world the guest's
 * contract names, compared with the granted set before anything is compiled
 * or instantiated. A world that imports a capability kind the grant gives
 * none of fails with `CapabilityDeniedError` (`world.capabilities`). The same
 * function decides `world.capabilities` in `experimental_verify`, so the two
 * cannot disagree. Returns `{ contract, within, pure, effects }`, frozen;
 * `within`, `pure` and `effects` are null for a contract this reader has no
 * world for.
 */
function worldWithinGrant(directory, grant) {
  const contract = directory?.layers?.find((l) => l.kind === 'wasm')?.contract ?? null;
  if (contract === null) return Object.freeze({ contract: null, within: null, pure: null, effects: null, session: false });
  const result = worldPolicy(contract, grant.members);
  if (result.known && !result.within) {
    throw new CapabilityDeniedError(`the world ${contract} exceeds the granted capabilities: ${result.problems.join('; ')}`, {
      check: 'world.capabilities',
      details: { contract, problems: result.problems, set: grant.set },
    });
  }
  return Object.freeze({
    contract,
    within: result.within,
    pure: result.pure,
    effects: result.effects === null ? null : Object.freeze([...result.effects]),
    /** True for the session world (#629): `open()` takes steps, `dispatch` is refused. */
    session: isSessionContract(contract),
  });
}

/**
 * Open a `.holo` archive from bytes. `options.capabilities` is the capability
 * file the guest runs under and `options.capabilitySet` the set to apply,
 * `default` unless named; without a file the holo is granted nothing.
 */
export async function loadArchive(source, options = {}) {
  const parsed = parseArchive(asBytes(source));
  // An altered archive is refused before anything inside it is read, so damage is
  // named as damage, not as whichever document it happened to break.
  if (!parsed.verification.footer) refuseUnverified('integrity failure: the holo was altered: the footer is not the BLAKE3 of the body', options);
  const resolved = resolveArchive(parsed);
  // A malformed attestation row fails the load; the bundles decode lazily.
  attestationSection(parsed);
  const { modelDocument, provenance, directory } = resolved;
  const application = modelDocument.application ?? fail('archive.model', 'model document has no application');
  const capabilities = grantFor(options);
  const world = worldWithinGrant(resolved.directory, capabilities);
  const module = await compileGuest(resolved.guest.content, capabilities, world.contract ?? undefined);
  const limits = world.session ? sessionLimitsOf(application) : limitsOf(application);
  const vectors = world.session ? [] : vectorsOf(application);
  const sessions = world.session ? sessionsOf(application) : [];
  const view = decodeHoloview(resolved.view.content);
  let metadata = null;
  if (parsed.metadata) {
    try {
      metadata = JSON.parse(parsed.metadata.text);
    } catch {
      metadata = parsed.metadata.text;
    }
  }
  refuseUnverified(unverifiedArchive(parsed, resolved.identities), options);
  const middleware = createMiddlewareStack();
  let app = null;
  const step = stepper(middleware, () => app);
  const dispatch = async (input) => {
    if (world.session) fail('session.required', `the world ${world.contract} keeps state across steps; open a session with app.open() and dispatch on it`);
    const request = asBytes(input);
    checkRequest(request, limits);
    const instance = await step('instantiate', (args) => instantiateGuest(args.module, args.imports), { module, imports: {} });
    const response = await step('dispatch', (args) => runInstance(args.instance, args.request), { request, instance });
    return step('record', (args) => args.response, { request, response });
  };
  /** A new session on a fresh instance (#629). Each step runs the middleware `dispatch` and `record` steps. */
  const open = () => {
    if (!world.session) fail('session.unsupported', `the world ${world.contract} is stateless; it has no sessions, so dispatch on the application`);
    return openSession({
      instantiate: () => step('instantiate', (args) => instantiateGuest(args.module, args.imports), { module, imports: {} }),
      check: (request) => checkRequest(request, limits),
      step: async (instance, input) => {
        const request = asBytes(input);
        const responses = await step('dispatch', (args) => runStep(args.instance, args.request, limits), { request, instance });
        return step('record', (args) => args.response, { request, response: responses });
      },
    });
  };
  let inspection = null;
  app = Object.freeze({
    kind: 'holo',
    name: application.name ?? provenance.cargo_name ?? null,
    model: modelDocument,
    profile: profileOf(modelDocument),
    limits,
    vectors,
    provenance: { model: provenance, browser: null },
    directory,
    metadata,
    identities: resolved.identities,
    evidence: evidenceOf(provenance),
    view,
    archive: parsed,
    /** The capability set in force (#279): `allows(kind, resource)`, `assert(kind, resource)`, `set`. */
    capabilities,
    /** The declared world against the grant, checked before the guest compiled (#283): contract, within, pure, effects. */
    world,
    /** The acceptance of a session world, as ordered sessions of steps (#629); empty for a stateless world. */
    sessions,
    dispatch,
    open,
    attestations: () => readAttestations(parsed),
    acceptance: () => (world.session ? runSessionAcceptance(sessions, open) : runAcceptance(vectors, dispatch)),
    /** What the holo says about itself, read from the section table; the guest is not run. */
    inspect: () => (inspection ??= inspect(parsed.bytes)),
    /**
     * The verdict `prism check` returns for this holo; see `experimental_verify`.
     * The capability set the holo was loaded with is the default `capabilities`
     * option, so `world.capabilities` judges the grant it runs under (#283).
     */
    experimental_verify: (options) =>
      step('verify', (args) => experimental_verify(parsed.bytes, { capabilities: capabilities.members, ...(args.options ?? {}) }), { options }),
    /** The middleware stack around verify, instantiate, dispatch and record (#295). */
    experimental_middleware: middleware,
    /** The acceptance vectors as a history, this application's first (#284). */
    history: () => acceptanceHistory(app),
    /** Record dispatches as they run (#284). */
    experimental_recorder: () => experimental_recorder(app),
    /** Re-execute histories against this build; see `experimental_replay`. */
    experimental_replay: (histories, options) => experimental_replay(app, histories, options),
  });
  return app;
}

/**
 * Why a holo's identities do not verify, or null when they do: the footer,
 * a blob whose content does not hash to its label, or a manifest, directory
 * and provenance that name different blobs.
 */
function unverifiedArchive(parsed, identities) {
  if (identities.verified) return null;
  const problems = [];
  if (!parsed.verification.footer) problems.push('the footer is not the BLAKE3 of the body');
  if (!parsed.verification.labels) problems.push('a content blob does not hash to its label');
  if (!identities.consistent) problems.push('the manifest, directory and provenance do not name the same blobs');
  return `integrity failure: the holo was altered: ${problems.join('; ')}`;
}

/**
 * Refuse an application whose bytes are not the ones its identities name
 * (#583), unless the caller asked for it with `allowUnverified: true`. Nothing
 * has run: the checks before this one only decode.
 */
function refuseUnverified(problem, { allowUnverified = false } = {}) {
  if (problem === null || allowUnverified === true) return;
  throw new DigestMismatchError(`${problem}; to open it anyway, pass allowUnverified: true (--allow-unverified on the command line)`, { check: 'holo.integrity' });
}

/**
 * Open a six-file closure. `files` maps file names to bytes; `importGlue(name)`
 * imports the wasm-bindgen glue module by file name (a file URL in Node, a
 * URL in browsers). `model` is optional and only needed to dispatch through a
 * numeric-profile closure, whose wire format lives in the model document.
 * `capabilities` and `capabilitySet` are as for `loadArchive`; a closure's
 * imports belong to its glue, so only `app.capabilities` carries the grant.
 */
export async function loadClosure(files, importGlue, { model = null, capabilities = null, capabilitySet = DEFAULT_SET, allowUnverified = false } = {}) {
  const grant = grantFor({ capabilities, capabilitySet });
  const { stem, glueName, wasmName, provenance, verification } = await verifyClosure(files);
  if (!verification.verified) {
    refuseUnverified(`the closure's files do not match its provenance: ${verification.mismatches.map((m) => m.field).join(', ')}`, { allowUnverified });
  }
  const glue = initializeGlue(await importGlue(glueName), files.get(wasmName));
  const modelDocument = model ? (model instanceof Uint8Array ? JSON.parse(decoder.decode(model)) : model) : null;
  const application = modelDocument?.application ?? null;
  const limits = application ? limitsOf(application) : null;
  const vectors = application ? vectorsOf(application) : [];
  const profile = modelDocument ? profileOf(modelDocument) : typeof glue.invoke_bytes === 'function' ? 'text' : 'numeric';
  const dispatch = closureDispatch(glue, { profile, model: modelDocument, limits });
  const view = { entry: 'index.html', files };
  // The glue instantiated its module once, above; a closure runs the verify, dispatch and record steps.
  const middleware = createMiddlewareStack();
  let app = null;
  const step = stepper(middleware, () => app);
  const dispatchThrough = async (request) => {
    const response = await step('dispatch', (args) => dispatch(args.request), { request });
    return step('record', (args) => args.response, { request, response });
  };
  app = Object.freeze({
    kind: 'closure',
    name: stem,
    model: modelDocument,
    profile,
    limits,
    vectors,
    provenance: { model: null, browser: provenance },
    directory: null,
    metadata: null,
    identities: { ...verification, stem, glueName, wasmName },
    evidence: {
      modelId: provenance.model_id ?? null,
      viewModelId: provenance.view_model_id ?? null,
      generatedCoreSha256: provenance.generated_core_sha256 ?? null,
    },
    view,
    capabilities: grant,
    /** A closure names no guest contract, so there is no world to check (#283). */
    world: Object.freeze({ contract: null, within: null, pure: null, effects: null }),
    dispatch: async (request) => dispatchThrough(asBytes(request)),
    attestations: () => [],
    acceptance: () => runAcceptance(vectors, (r) => dispatchThrough(r)),
    inspect: async () => fail('inspect.kind', 'inspect reads a .holo archive; a closure has no section table'),
    experimental_verify: (options) => step('verify', (args) => experimental_verify({ kind: 'closure' }, args.options), { options }),
    experimental_middleware: middleware,
    history: () => acceptanceHistory(app),
    experimental_recorder: () => experimental_recorder(app),
    experimental_replay: (histories, options) => experimental_replay(app, histories, options),
  });
  return app;
}

/**
 * Open an application from bytes, a path (Node) or a URL. Bytes and `.holo`
 * paths or URLs open an archive; a directory path or a URL ending in `/`
 * opens a closure.
 */
export async function load(source, options = {}) {
  if (source instanceof Uint8Array || source instanceof ArrayBuffer || ArrayBuffer.isView(source)) {
    return loadArchive(source, options);
  }
  if (typeof Blob !== 'undefined' && source instanceof Blob) {
    return loadArchive(new Uint8Array(await source.arrayBuffer()), options);
  }
  if (typeof source === 'string' || source instanceof URL) {
    if (isNode()) {
      const node = await import('./node.mjs');
      return node.fromFile(source, options);
    }
    const browser = await import('./browser.mjs');
    return browser.load(source, options);
  }
  fail('load.source', 'load() takes bytes, a Blob, a path or a URL');
}

/** The nine modules of #440, in the order the client exposes them. */
export const MODULES = Object.freeze(['client', 'images', 'containers', 'networks', 'volumes', 'system', 'auth', 'errors', 'verify']);

const CLIENT_OPTIONS = ['store', 'warn', 'capabilities', 'capabilityStore', 'capabilitySet', 'policy', 'world', 'declarations', 'runner', 'docker', 'spawn', 'readLock', 'auth', 'events', 'imageStore'];

/** A capability file kept in memory, which `prism.networks` reads and writes when no store is given. */
export function memoryCapabilityStore(initial = null) {
  let text = initial === null || initial === undefined ? null : typeof initial === 'string' ? initial : initial instanceof Uint8Array ? new TextDecoder().decode(initial) : JSON.stringify(initial);
  return Object.freeze({
    read: () => text,
    write: (next) => {
      text = next;
    },
  });
}

const IMAGE_VERBS = Object.freeze(['build', 'pull', 'push', 'list', 'get', 'tag', 'remove', 'prune', 'inspect', 'verify', 'usage']);

/**
 * `prism.images`, loaded on first use. It reads files and registries, so a bundler
 * leaves `images-namespace.mjs` out of a browser build (package.json `browser`),
 * and there every verb raises `images.unavailable`. In Node it is `createImages`.
 */
function lazyImages(dependencies) {
  let made = null;
  const namespace = () => {
    made ??= import('./images-namespace.mjs').then((module) => {
      if (typeof module?.createImages !== 'function') fail('images.unavailable', 'prism.images reads files and registries, which this build of prism-code leaves out; use it in Node');
      return module.createImages(dependencies);
    });
    return made;
  };
  return Object.freeze(Object.fromEntries(IMAGE_VERBS.map((verb) => [verb, async (...args) => (await namespace())[verb](...args)])));
}

/** `prism.auth` where there is no credential store or key: in a browser, or a client made without one. Every verb says how to get the real one. */
const unavailable = (verb) => () => fail('auth.unavailable', `prism.auth.${verb} needs Node's credential stores and keys; use Prism.fromEnv() from prism-code/node, or pass createPrism({ auth })`);
const NO_AUTH = Object.freeze({
  login: unavailable('login'),
  logout: unavailable('logout'),
  inspect: unavailable('inspect'),
  stores: Object.freeze({ list: unavailable('stores.list'), get: unavailable('stores.get') }),
  identities: Object.freeze({ fromKey: unavailable('identities.fromKey'), keyless: unavailable('identities.keyless') }),
});

/**
 * A client: the nine modules of #440 as one object (issue #441). Every option is
 * optional, and each goes to the module that owns it:
 *
 * - `containers`: `store` keeps the records, in memory unless given; `warn` receives
 *   the warning of an unverified run, `console.warn` unless given.
 * - `networks` and `volumes`: `capabilities` is the capability file, and none grants
 *   nothing; `capabilityStore` holds it for `networks`, in memory unless given, and
 *   overrides `capabilities` there. `capabilitySet`, `world`, `policy`,
 *   `declarations` and `runner` are `createVolumes`'s. `policy` also bounds `networks`.
 * - `images`: `docker` builds the Docker invocation for `build`, and `spawn` runs it.
 *   Without `docker`, `build` raises `PP5008` and the other verbs work. `imageStore`
 *   is the local image store's backend, in memory unless given; the images a
 *   container record references are held there, so `remove` and `prune` keep them.
 * - `system`: `readLock(path)` returns a project's `prismpm.lock`, for `info` and
 *   `version`; `events` is the bus containers report to, a new one unless given.
 * - `auth`: a namespace from `createAuth`. It needs Node, so without one every verb
 *   raises `auth.unavailable`; `Prism.fromEnv()` in `prism-code/node` gives it.
 *
 * `createPrism()` stays the documented path in a browser. In Node, `Prism.fromEnv()`
 * makes the same object from this machine's configuration.
 */
export function createPrism(options = {}) {
  const prototype = options === null || typeof options !== 'object' ? undefined : Object.getPrototypeOf(options);
  if (prototype !== Object.prototype && prototype !== null) fail('client.option', 'createPrism options must be a plain object');
  const unknown = Object.keys(options).filter((name) => !CLIENT_OPTIONS.includes(name));
  if (unknown.length > 0) fail('client.option', `createPrism has no option ${unknown.join(', ')}; it takes ${CLIENT_OPTIONS.join(', ')}`);
  const { store, warn, capabilities = null, capabilityStore, capabilitySet, policy = null, world = null, declarations, runner = null, docker = null, spawn, readLock = null, auth = NO_AUTH, imageStore } = options;
  const events = options.events ?? createEventBus();
  // One volume scope for `volumes` and for the volumes a container asks for (#445), so both refuse alike.
  const volumeScope = { capabilities, ...(capabilitySet === undefined ? {} : { set: capabilitySet }), world, policy, ...(declarations === undefined ? {} : { declarations }) };
  const networks = createNetworks({ store: capabilityStore ?? memoryCapabilityStore(capabilities), policy: { effects: policy?.effects ?? null, capabilities: policy?.capabilities ?? null } });
  const volumes = createVolumes({ ...volumeScope, runner });
  if (docker !== null && typeof docker !== 'function') fail('client.option', 'docker must be a function that builds the Docker invocation, or null');
  // What holds an image is a container record that names it; `images` may not import `containers`, so it is told.
  const references = async () => Promise.all((await containers.list({ all: true })).map(async (container) => {
    const record = await container.inspect();
    return { digest: record.image.digest, running: record.state.status === 'running' };
  }));
  const images = lazyImages({ docker, references, ...(spawn === undefined ? {} : { spawn }), ...(imageStore === undefined ? {} : { store: imageStore }) });
  // A name the image store keeps opens that image, as `docker run calc:latest` does (#347); a path that exists wins.
  const loadImage = async (source, loadOptions) => {
    if (typeof source === 'string' && !(isNode() && (await (await import('./node.mjs')).pathExists(source)))) {
      const image = await images.get(source).catch((error) => {
        if (error instanceof ReferenceInvalidError || /^image(s)?\.(missing|name|unavailable)$/.test(error?.code ?? '')) return null;
        throw error;
      });
      if (image !== null) return load(image.bytes, loadOptions);
    }
    return load(source, loadOptions);
  };
  const containers = createContainers({ load: loadImage, ...(store === undefined ? {} : { store }), ...(warn === undefined ? {} : { warn }), onEvent: events.emit, volumes: volumeScope });
  const system = createSystem({ containers, events, readLock, images });
  return Object.freeze({ images, containers, networks, volumes, system, auth, errors: errorsNamespace, verify: verifyNamespace });
}

/** The default client: `prism.containers.run(image, input)`, `prism.images.pull(ref)` and the other modules. Its records live in memory until removed. */
export const prism = createPrism();

// 0.1.0 compatibility: the canonicalization API stays at `prism-code/canonical`.
// These top-level re-exports are deprecated and will be removed in 0.3.0.
let warned = false;
function deprecated(name, fn) {
  return function deprecatedCanonical(...args) {
    if (!warned) {
      warned = true;
      const message = `prism-code: top-level ${name}() is deprecated; import it from 'prism-code/canonical'`;
      if (isNode() && typeof process.emitWarning === 'function') process.emitWarning(message, 'DeprecationWarning');
      else if (typeof console !== 'undefined') console.warn(message);
    }
    return fn(...args);
  };
}
/** @deprecated import from 'prism-code/canonical' */
export const canonicalize = deprecated('canonicalize', canonical.canonicalize);
/** @deprecated import from 'prism-code/canonical' */
export const codeHash = deprecated('codeHash', canonical.codeHash);
/** @deprecated import from 'prism-code/canonical' */
export const describe = deprecated('describe', canonical.describe);
/** @deprecated import from 'prism-code/canonical' */
export const differs = deprecated('differs', canonical.differs);
