/**
 * Containers: one run of a holo as an object with Docker's states (issue #348,
 * docs/proposals/348-container-lifecycle.md). A container is created from an
 * image and an input, is started once or again, and is kept with its exit code
 * and output until it is removed.
 *
 *   created --start--> running --guest returned or threw--> exited --start--> running
 *      |                                                      ^
 *      +----- check refused the holo, the guest never ran ----+
 *
 * This is the developer view of an instance (#346); the model's terms do not
 * change. The gate is the verdict of `check` (#282), so no container runs a
 * holo that check refuses unless the caller says `allowUnverified`.
 *
 * Experimental until the v0.6 freeze (STABILITY.md section 6).
 */
import { PrismError, VerificationError, errorFromVerdict, exitCodeFor, fail } from './errors.mjs';
import { attestationBlocker } from './verify.mjs';
import { ASSURANCE } from './vendor/verify.mjs';
import { attachVolume, refusalRecord } from './volumes.mjs';

export const CONTAINER_SCHEMA = 'prism/container/1';

// Not part of the API: how `get` hands a cached container the record in the store.
const REFRESH = Symbol('refresh');

/** The volume names `create` takes: an array of strings, each attached through `volumes` (#445). */
function volumeNames(volumes) {
  if (volumes === undefined) return [];
  if (!Array.isArray(volumes) || !volumes.every((name) => typeof name === 'string')) fail('container.option', 'volumes must be an array of volume names');
  return [...new Set(volumes)];
}

/**
 * Attach each volume, and return the first refusal with the record of every one.
 * Every attach is refused until host effects land (#216, #217); a refusal that is
 * not the volumes module's, such as an invalid capability file, is thrown.
 */
async function attachAll(scope, names) {
  let first = null;
  const rows = [];
  for (const name of names) {
    try {
      await attachVolume(scope, name);
      rows.push({ name, attached: true, refusal: null });
    } catch (error) {
      const refusal = refusalRecord(error);
      if (refusal === null) throw error;
      first ??= error;
      rows.push({ name, attached: false, refusal: structuredClone(refusal) });
    }
  }
  return { error: first, rows };
}

const utf8 = new TextDecoder('utf-8', { fatal: true });
const HEX_ID = /^[0-9a-f]{1,64}$/;

const hex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');

function inputBytes(input) {
  if (typeof input === 'string') return new TextEncoder().encode(input);
  if (input instanceof Uint8Array) return input;
  if (input instanceof ArrayBuffer) return new Uint8Array(input);
  if (ArrayBuffer.isView(input)) return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  return fail('container.input', 'a container takes one input: a string or bytes');
}

function encodeOutput(bytes) {
  try {
    return { encoding: 'utf-8', data: utf8.decode(bytes) };
  } catch {
    return { encoding: 'hex', data: hex(bytes) };
  }
}

function decodeOutput(output) {
  if (output === null) return null;
  if (output.encoding === 'utf-8') return new TextEncoder().encode(output.data);
  return Uint8Array.from(output.data.match(/../g) ?? [], (pair) => parseInt(pair, 16));
}

function errorSummary(error) {
  const summary = { name: error?.name ?? 'Error', code: error?.code ?? null, message: String(error?.message ?? error) };
  if (error?.check !== undefined) summary.check = error.check;
  if (error?.diagnostic !== undefined) summary.diagnostic = error.diagnostic;
  return summary;
}

/** The warning a run under `allowUnverified` gives, on every start. `error` explains a null verdict. */
export function unverifiedWarning(verdict, error) {
  if (verdict === null) return `no verdict (${error.check ?? error.code}: ${error.message}); running it unverified`;
  const failed = verdict.reasons.filter((r) => r.check !== 'policy.level').map((r) => r.check).join(', ') || 'policy.level';
  return `the holo failed check (${verdict.level}, policy ${verdict.policy.name} requires ${verdict.policy.minimum}): ${failed}; running it unverified`;
}

/**
 * The verdict the holo runs under, and the error that explains a refusal.
 * `policy` is the members `experimental_verify` takes, or a function that
 * returns them, so that a policy file that cannot be read is no verdict and
 * not a failure to create. Whatever else goes wrong is no verdict too, as in
 * `check`: the gate fails closed.
 */
async function judge(app, policy) {
  try {
    const members = typeof policy === 'function' ? await policy() : policy;
    const blocked = app.archive?.bytes ? attestationBlocker(app.archive.bytes) : null;
    if (blocked) throw blocked;
    const verdict = await app.experimental_verify(members ?? undefined);
    return { verdict, error: errorFromVerdict(verdict, ASSURANCE.RANK) };
  } catch (error) {
    if (error instanceof PrismError) return { verdict: null, error };
    return { verdict: null, error: new VerificationError(error?.message ?? String(error), { check: 'verify.input', cause: error }) };
  }
}

/** Records in memory: the default store, in Node and in a browser. A record lives as long as the process. */
export function memoryContainerStore() {
  const records = new Map();
  return Object.freeze({
    put: async (record) => void records.set(record.id, structuredClone(record)),
    get: async (id) => structuredClone(records.get(id)),
    find: async (prefix) => [...records.keys()].filter((id) => id.startsWith(prefix)),
    remove: async (id) => void records.delete(id),
  });
}

class Container {
  #record;
  #app;
  #input;
  #policy;
  #allowUnverified;
  #store;
  #warn;
  #forget;
  #emit;
  #volumeScope;
  #output;
  #error = null;
  #inFlight = null;
  #removed = false;

  constructor({ record, app = null, input = null, policy, allowUnverified = false, store, warn, forget, emit, volumeScope = {} }) {
    this.#record = record;
    this.#app = app;
    this.#input = input;
    this.#policy = policy;
    this.#allowUnverified = allowUnverified;
    this.#store = store;
    this.#warn = warn;
    this.#forget = forget;
    this.#emit = emit;
    this.#volumeScope = volumeScope;
    this.#output = decodeOutput(record.output);
  }

  get id() {
    return this.#record.id;
  }

  /** `created`, `running` or `exited`. */
  get status() {
    return this.#record.state.status;
  }

  /** The exit code once exited, otherwise null. */
  get exitCode() {
    return this.#record.state.exitCode;
  }

  /** The bytes the guest returned, or null when it returned none. */
  get output() {
    return this.#output;
  }

  /** The typed error that ended this run, or null. A container read back from a store has the summary in `inspect()` instead. */
  get error() {
    return this.#error;
  }

  /** The output the guest returned, as `docker logs` serves it: from the record, so a container read back from its store has it too. Empty before any run. */
  async logs(options) {
    if (options !== undefined && (options === null || typeof options !== 'object' || Object.keys(options).length > 0)) fail('container.option', 'logs takes no options: a run returns one output, kept on the record');
    if (this.#removed) fail('container.missing', `no such container: ${this.id}`);
    return decodeOutput(this.#record.output) ?? new Uint8Array(0);
  }

  /**
   * Take the record another client wrote to the shared store, so a long-lived client
   * sees what others did. A run in this process is the writer of its own record and
   * keeps it.
   */
  [REFRESH](record) {
    if (this.#inFlight === null && !this.#removed) {
      this.#record = record;
      this.#output = decodeOutput(record.output);
      if (record.state.status !== 'exited' || record.state.error === null) this.#error = null;
    }
    return this;
  }

  /** The output as text, or null when the guest returned none. */
  text() {
    return this.#output === null ? null : new TextDecoder().decode(this.#output);
  }

  /**
   * Run the holo, and resolve to this container once it has exited. A second
   * call while it runs joins that run; a call after it exited runs it again
   * from the same input, as `docker start` does. A failure of the run is the
   * exit code, never a rejection.
   */
  start() {
    this.#inFlight ??= this.#run().finally(() => {
      this.#inFlight = null;
    });
    return this.#inFlight;
  }

  async #run() {
    if (this.#removed) fail('container.missing', `no such container: ${this.id}`);
    if (this.#app === null) fail('container.state', `container ${this.id} was read back from its store, which does not keep its input; run the holo again instead`);
    // A volume is refused before check runs, and allowUnverified does not bypass it.
    if ((this.#record.volumes ?? []).length > 0) {
      const attached = await attachAll(this.#volumeScope, this.#record.volumes.map((row) => row.name));
      this.#record.volumes = attached.rows;
      if (attached.error !== null) return this.#exit({ error: attached.error, startedAt: null });
    }
    const { verdict, error } = await judge(this.#app, this.#policy);
    this.#record.verdict = verdict;
    this.#record.unverified = null;
    if (error !== null) {
      if (!this.#allowUnverified) return this.#exit({ error, startedAt: null });
      const warning = unverifiedWarning(verdict, error);
      // The bypass is on the record, so whoever reads the container sees that it ran unverified, and why.
      this.#record.unverified = { at: new Date().toISOString(), warning, code: error.code ?? null, check: error.check ?? null };
      this.#warn(warning);
    }
    const startedAt = new Date().toISOString();
    this.#record.state = { status: 'running', exitCode: null, error: null, startedAt, finishedAt: null };
    this.#record.output = null;
    this.#output = null;
    this.#error = null;
    await this.#store.put(this.#record);
    this.#emit('start', this.#record);
    let output;
    try {
      output = await this.#app.dispatch(this.#input);
    } catch (thrown) {
      return this.#exit({ error: thrown, startedAt });
    }
    return this.#exit({ error: null, startedAt, output });
  }

  async #exit({ error, startedAt, output = null }) {
    this.#error = error;
    this.#output = output;
    this.#record.state = {
      status: 'exited',
      exitCode: exitCodeFor(error),
      error: error === null ? null : errorSummary(error),
      startedAt,
      finishedAt: new Date().toISOString(),
    };
    this.#record.output = output === null ? null : encodeOutput(output);
    this.#emit('die', this.#record);
    if (this.#record.rm) await this.#drop();
    else await this.#store.put(this.#record);
    return this;
  }

  async #drop() {
    await this.#store.remove(this.id);
    this.#removed = true;
    this.#forget(this.id);
    this.#emit('destroy', this.#record);
  }

  /**
   * The record: state, exit code, image digest, verdict and output. Rejects
   * with `container.missing` once the container is removed; what you hold
   * keeps `status`, `exitCode`, `error` and `output`.
   */
  async inspect() {
    if (this.#removed) fail('container.missing', `no such container: ${this.id}`);
    return structuredClone(this.#record);
  }

  /** Remove the container and its record. A running container is refused, as Docker refuses it without `--force`. */
  async remove() {
    if (this.#removed) fail('container.missing', `no such container: ${this.id}`);
    if (this.#inFlight !== null) fail('container.running', `container ${this.id} is running and cannot be removed`);
    await this.#drop();
  }
}

const utf8Encoder = new TextEncoder();

/**
 * The `containers` namespace over a loader and a store. `load` opens an image;
 * `store` keeps the records; `warn` receives the warning of an unverified run.
 * `onEvent` receives each lifecycle event, `{ action, id, image, exitCode }` with
 * Docker's actions `create`, `start`, `die` and `destroy`; the client passes the
 * bus `system.events` reads (#446). An `onEvent` that throws does not stop a run.
 */
export function createContainers({ load, store = memoryContainerStore(), warn = (message) => console.warn(`prism-code: ${message}`), onEvent = null, volumes: volumeScope = {} }) {
  if (onEvent !== null && typeof onEvent !== 'function') fail('container.option', 'onEvent must be a function, or null');
  const live = new Map();
  const forget = (id) => live.delete(id);
  const emit = (action, record) => {
    if (onEvent === null) return;
    try {
      onEvent({ action, id: record.id, image: record.image.digest, exitCode: record.state.exitCode });
    } catch {
      // An observer is told; it does not decide.
    }
  };

  /**
   * Open the image and make a container in state `created`. An image that
   * cannot be opened makes no container: this rejects with its typed error, as
   * `docker create` fails on an image it cannot find.
   */
  async function create(image, input, options = {}) {
    const { policy, allowUnverified = false, rm = false, volumes, ...loadOptions } = options;
    if (typeof allowUnverified !== 'boolean') fail('container.option', 'allowUnverified must be true or false');
    // A copy: a later `start` runs the input as it was given here.
    const bytes = inputBytes(input).slice();
    const names = volumeNames(volumes);
    const app = typeof image?.dispatch === 'function' ? image : await load(image, { ...loadOptions, allowUnverified });
    const fingerprint = app.identities?.archiveFingerprint;
    const record = {
      schema: CONTAINER_SCHEMA,
      id: hex(crypto.getRandomValues(new Uint8Array(32))),
      image: { name: app.name ?? null, kind: app.kind ?? null, digest: fingerprint ? `blake3:${fingerprint}` : null },
      created: new Date().toISOString(),
      rm,
      allowUnverified,
      // Each volume asked for, and the typed refusal that keeps it from the guest (#445).
      volumes: (await attachAll(volumeScope, names)).rows,
      state: { status: 'created', exitCode: null, error: null, startedAt: null, finishedAt: null },
      verdict: null,
      // Set when a start ran past a refusing check under allowUnverified: when, why, and the warning given.
      unverified: null,
      output: null,
    };
    await store.put(record);
    const container = new Container({ record, app, input: bytes, policy, allowUnverified, store, warn, forget, emit, volumeScope });
    live.set(record.id, container);
    emit('create', record);
    return container;
  }

  /** Create and start: resolves to the container once it has exited, whatever its exit code. */
  async function run(image, input, options) {
    const container = await create(image, input, options);
    return container.start();
  }

  /** The container with this id, or with this unique id prefix. */
  async function get(id) {
    const ids = typeof id === 'string' && HEX_ID.test(id) ? await store.find(id) : [];
    if (ids.length === 0) fail('container.missing', `no such container: ${id}`);
    if (ids.length > 1) fail('container.ambiguous', `${id} names ${ids.length} containers; give more of the id`);
    const record = await store.get(ids[0]);
    // Removed by another client between find and get.
    if (record === undefined) {
      forget(ids[0]);
      fail('container.missing', `no such container: ${id}`);
    }
    const known = live.get(ids[0]);
    if (known) return known[REFRESH](record);
    const container = new Container({ record, store, warn, forget, emit, volumeScope });
    live.set(ids[0], container);
    return container;
  }

  /** Every container in the store, oldest first: running ones only, as `docker ps`, unless `all`. */
  async function list(options = {}) {
    const { all = false, ...rest } = options;
    if (Object.keys(rest).length > 0) fail('container.option', `containers.list has no option ${Object.keys(rest).join(', ')}; it takes all`);
    if (typeof all !== 'boolean') fail('container.option', 'containers.list option all must be true or false');
    const found = await Promise.all((await store.find('')).map((id) => get(id)));
    const records = await Promise.all(found.map(async (container) => [container, await container.inspect()]));
    return records
      .filter(([, record]) => all || record.state.status === 'running')
      .sort(([, a], [, b]) => (a.created < b.created ? -1 : a.created > b.created ? 1 : a.id < b.id ? -1 : 1))
      .map(([container]) => container);
  }

  /**
   * Remove every exited container, or those that finished before `until`
   * (milliseconds since the epoch), as `docker container prune` does. A created or
   * running container is kept. `spaceReclaimed` is the bytes of the records' JSON.
   */
  async function prune(options = {}) {
    const { until, ...rest } = options;
    if (Object.keys(rest).length > 0) fail('container.option', `containers.prune has no option ${Object.keys(rest).join(', ')}; it takes until`);
    if (until !== undefined && !Number.isFinite(until)) fail('container.option', 'containers.prune option until must be milliseconds since the epoch');
    const containersDeleted = [];
    let spaceReclaimed = 0;
    for (const container of await list({ all: true })) {
      const record = await container.inspect();
      if (record.state.status !== 'exited') continue;
      if (until !== undefined && Date.parse(record.state.finishedAt) >= until) continue;
      try {
        await container.remove();
      } catch (error) {
        // Started again since it was listed: it is running now, and is kept.
        if (error?.code === 'container.running') continue;
        throw error;
      }
      containersDeleted.push(record.id);
      spaceReclaimed += utf8Encoder.encode(JSON.stringify(record)).length;
    }
    return { containersDeleted, spaceReclaimed };
  }

  /**
   * Run the holo once and return what it returned, as a call: the container is
   * removed on exit unless `rm: false`, and a failed run rejects with its typed
   * error instead of resolving to an exit code. A refused image never runs.
   */
  async function dispatch(image, input, options = {}) {
    const container = await run(image, input, { rm: true, ...options });
    if (container.error !== null) throw container.error;
    return container.output ?? new Uint8Array(0);
  }

  return Object.freeze({
    run,
    create,
    dispatch,
    get,
    list,
    prune,
    start: async (id) => (await get(id)).start(),
    logs: async (id, options) => (await get(id)).logs(options),
    inspect: async (id) => (await get(id)).inspect(),
    remove: async (id) => (await get(id)).remove(),
  });
}
