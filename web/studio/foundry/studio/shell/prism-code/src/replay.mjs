/**
 * Histories and replay (issue #284, docs/proposals/284-histories-replay.md).
 *
 * A history, `prism/history/1`, is the dispatches of one application in order,
 * each with its recorded outcome. Acceptance vectors are its first source: a
 * vector is an event with a response. Replay re-executes every event against a
 * build and compares outcomes byte for byte. A difference is a
 * `DeterminismViolationError` (`replay.divergence`); a replay that cannot be
 * carried out is a `ReplayError`. Either way nothing is decoded, so the same
 * code replays any profile.
 *
 * A replay of a holo also produces a `prism/replay/v1` statement about its
 * guest. The statement holds no clock, so the same replay writes the same
 * bytes, and signing it is a separate step (`prism-code attest`).
 *
 * Experimental until the v0.6 freeze (STABILITY.md section 6).
 */
import { asBytes, canonicalJson, resolveArchive } from './archive.mjs';
import { STATEMENT_TYPE } from './attestations.mjs';
import { blake3Hex } from './blake3.mjs';
import { DeterminismViolationError, ReplayError } from './errors.mjs';
import { sha256Hex } from './sha256.mjs';
import { VERSION } from './version.mjs';

export const HISTORY_SCHEMA = 'prism/history/1';
/** The history of a session world (#629, docs/proposals/629-session-world.md). */
export const SESSION_HISTORY_SCHEMA = 'prism/history/2';
export const REPLAY_PREDICATE_TYPE = 'https://uor.foundation/prism/replay/v1';
/** The registered diagnostic of `replay.divergence`. */
export const DIVERGENCE_CODE = 'PP5701';

const HEX = /^(?:[0-9a-f]{2})*$/;
const HEX64 = /^[0-9a-f]{64}$/;
const ERROR_CODE = /^[a-z][a-z0-9-]*(?:\.[a-z0-9-]+)+$/;
const TRAP = 'core.trap';

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });

const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

function invalid(message, schema = HISTORY_SCHEMA) {
  return new ReplayError(`the history is not ${schema}: ${message}`, { check: 'replay.history' });
}

function toHex(bytes) {
  let s = '';
  for (const b of bytes) s += b.toString(16).padStart(2, '0');
  return s;
}

function onlyMembers(value, allowed, where, schema = HISTORY_SCHEMA) {
  const unknown = Object.keys(value).find((key) => !allowed.includes(key));
  if (unknown !== undefined) throw invalid(`${where} has the unknown member ${unknown}`, schema);
}

function digestSet(value, where, schema = HISTORY_SCHEMA) {
  if (!isObject(value)) throw invalid(`${where} is not a digest set`, schema);
  onlyMembers(value, ['blake3', 'sha256'], where, schema);
  for (const key of ['blake3', 'sha256']) {
    if (typeof value[key] !== 'string' || !HEX64.test(value[key])) throw invalid(`${where}.${key} is not 64 lowercase hex characters`, schema);
  }
  return { blake3: value.blake3, sha256: value.sha256 };
}

function hexField(value, where, schema = HISTORY_SCHEMA) {
  if (typeof value !== 'string' || !HEX.test(value)) throw invalid(`${where} is not lowercase hex`, schema);
  return value;
}

/** The members a history of either schema shares: `application` and `recordedFrom`. */
function headerOf(value, schema) {
  if (!Object.hasOwn(value, 'application')) throw invalid('application is missing', schema);
  if (value.application !== null && (typeof value.application !== 'string' || value.application === '')) throw invalid('application must be a name or null', schema);
  const header = { schema, application: value.application };
  if (Object.hasOwn(value, 'recordedFrom')) {
    if (!isObject(value.recordedFrom)) throw invalid('recordedFrom is not an object', schema);
    onlyMembers(value.recordedFrom, ['guest', 'model'], 'recordedFrom', schema);
    header.recordedFrom = { guest: digestSet(value.recordedFrom.guest, 'recordedFrom.guest', schema), model: digestSet(value.recordedFrom.model, 'recordedFrom.model', schema) };
  }
  return header;
}

const MAX_RESPONSES = 65535;

/** One session of a `prism/history/2` history: its steps, then the failure that ended it, if any. */
function sessionOf(value, index) {
  const S = SESSION_HISTORY_SCHEMA;
  const where = `sessions[${index}]`;
  if (!isObject(value)) throw invalid(`${where} is not an object`, S);
  onlyMembers(value, ['steps', 'failure'], where, S);
  if (!Array.isArray(value.steps)) throw invalid(`${where}.steps must be a list`, S);
  const steps = value.steps.map((step, i) => {
    const at = `${where}.steps[${i}]`;
    if (!isObject(step)) throw invalid(`${at} is not an object`, S);
    onlyMembers(step, ['request', 'responses'], at, S);
    if (!Array.isArray(step.responses) || step.responses.length > MAX_RESPONSES) throw invalid(`${at}.responses must be a list of at most ${MAX_RESPONSES}`, S);
    return { request: hexField(step.request, `${at}.request`, S), responses: step.responses.map((r, k) => hexField(r, `${at}.responses[${k}]`, S)) };
  });
  const session = { steps };
  if (Object.hasOwn(value, 'failure')) {
    const at = `${where}.failure`;
    const failure = value.failure;
    if (!isObject(failure)) throw invalid(`${at} is not an object`, S);
    onlyMembers(failure, ['request', 'error'], at, S);
    if (typeof failure.error !== 'string' || !ERROR_CODE.test(failure.error)) throw invalid(`${at}.error is not a stable error code`, S);
    session.failure = { request: hexField(failure.request, `${at}.request`, S), error: failure.error };
  }
  if (steps.length === 0 && session.failure === undefined) throw invalid(`${where} records neither a step nor a failure`, S);
  return session;
}

function parseSessionHistory(value) {
  const S = SESSION_HISTORY_SCHEMA;
  onlyMembers(value, ['schema', 'application', 'recordedFrom', 'sessions'], 'the history', S);
  const history = headerOf(value, S);
  if (!Array.isArray(value.sessions) || value.sessions.length === 0) throw invalid('sessions must hold at least one session', S);
  history.sessions = value.sessions.map(sessionOf);
  return history;
}

/** The records of a parsed history in order: events of `/1`, or every step and failure of every session of `/2`. */
export function historyEvents(history) {
  if (history.schema !== SESSION_HISTORY_SCHEMA) return history.events;
  return history.sessions.flatMap((session) => [...session.steps, ...(session.failure ? [session.failure] : [])]);
}

function eventOf(value, index) {
  const where = `events[${index}]`;
  if (!isObject(value)) throw invalid(`${where} is not an object`);
  const request = hexField(value.request, `${where}.request`);
  const hasResponse = Object.hasOwn(value, 'response');
  const hasError = Object.hasOwn(value, 'error');
  if (hasResponse === hasError) throw invalid(`${where} must hold exactly one of response and error`);
  if (hasResponse) {
    onlyMembers(value, ['request', 'response'], where);
    return { request, response: hexField(value.response, `${where}.response`) };
  }
  onlyMembers(value, ['request', 'error'], where);
  if (typeof value.error !== 'string' || !ERROR_CODE.test(value.error)) throw invalid(`${where}.error is not a stable error code`);
  return { request, error: value.error };
}

/**
 * Read a history from text, UTF-8 bytes or a parsed value. Rejects with a
 * `ReplayError` (`replay.history`) for anything the schema rejects, and returns
 * the history with its members in a fixed order.
 *
 * @param {string | Uint8Array | ArrayBuffer | object} source
 */
export function parseHistory(source) {
  let value = source;
  if (source instanceof Uint8Array || source instanceof ArrayBuffer || ArrayBuffer.isView(source)) {
    try {
      value = decoder.decode(asBytes(source));
    } catch {
      throw invalid('it is not UTF-8');
    }
  }
  if (typeof value === 'string') {
    try {
      value = JSON.parse(value);
    } catch (error) {
      throw invalid(`it is not JSON: ${error.message}`);
    }
  }
  if (!isObject(value)) throw invalid('it is not a JSON object');
  if (value.schema === SESSION_HISTORY_SCHEMA) return parseSessionHistory(value);
  onlyMembers(value, ['schema', 'application', 'recordedFrom', 'events'], 'the history');
  if (value.schema !== HISTORY_SCHEMA) throw invalid(`schema is ${JSON.stringify(value.schema)}`);
  const history = headerOf(value, HISTORY_SCHEMA);
  if (!Array.isArray(value.events) || value.events.length === 0) throw invalid('events must hold at least one event');
  history.events = value.events.map(eventOf);
  return history;
}

/** The bytes both digests of a history cover: its canonical JSON, with no trailing newline. */
export function historyBytes(history) {
  return encoder.encode(canonicalJson(parseHistory(history)));
}

async function digestsOf(bytes) {
  return { blake3: blake3Hex(bytes), sha256: await sha256Hex(bytes) };
}

/** Both digests of a history, as `prism/replay/v1` names it (ADR-015). */
export function historyDigest(history) {
  return digestsOf(historyBytes(history));
}

/** A history as a file: canonical JSON and one LF. */
export function historyJson(history) {
  return `${canonicalJson(parseHistory(history))}\n`;
}

/** The guest and model blobs of an opened holo with both digests of their bytes, or null for a closure. */
async function buildOf(app) {
  if (app.kind !== 'holo') return null;
  const { guest, model } = resolveArchive(app.archive);
  return { guest: await digestsOf(guest.content), model: await digestsOf(model.content) };
}

/** The stable code of a failed dispatch. A `PrismError` names its own; a guest trap has no stable message, so it has one code. */
function errorCodeOf(error) {
  return typeof error?.code === 'string' && ERROR_CODE.test(error.code) ? error.code : TRAP;
}

async function sessionHistoryFrom(app, sessions) {
  const recorded = sessions.filter((s) => s.steps.length > 0 || s.failure !== undefined);
  if (recorded.length === 0) throw new ReplayError('a history needs at least one session with a step, and none was recorded', { check: 'replay.history' });
  const build = await buildOf(app);
  return parseHistory({ schema: SESSION_HISTORY_SCHEMA, application: app.name ?? null, ...(build ? { recordedFrom: build } : {}), sessions: recorded });
}

/** A session that records its steps into `into`, a `prism/history/2` session. */
function recordingSession(session, into) {
  return Object.freeze({
    dispatch: async (request) => {
      const bytes = asBytes(request);
      const hex = toHex(bytes);
      const open = !session.closed;
      const slot = into.steps.push(null) - 1;
      try {
        const responses = await session.dispatch(bytes);
        into.steps[slot] = { request: hex, responses: responses.map(toHex) };
        return responses;
      } catch (error) {
        into.steps.splice(slot, 1);
        // Only the failure that ended the session is part of it. A step refused because the session was
        // already closed, or because another step was running, never reached the guest and changed nothing.
        if (open && session.closed && into.failure === undefined) into.failure = { request: hex, error: errorCodeOf(error) };
        throw error;
      }
    },
    async *stream(request) {
      for (const response of await this.dispatch(request)) yield response;
    },
    close: () => session.close(),
    get closed() {
      return session.closed;
    },
    get steps() {
      return session.steps;
    },
  });
}

async function historyFrom(app, events) {
  if (events.length === 0) throw new ReplayError('a history needs at least one event, and none was recorded', { check: 'replay.history' });
  const build = await buildOf(app);
  return parseHistory({ schema: HISTORY_SCHEMA, application: app.name ?? null, ...(build ? { recordedFrom: build } : {}), events });
}

/**
 * The acceptance vectors of an application as a history, its first history.
 * Rejects with `replay.history` when it carries none.
 */
export function acceptanceHistory(app) {
  if (app.world?.session) {
    return sessionHistoryFrom(
      app,
      app.sessions.map((steps) => ({ steps: steps.map((s) => ({ request: toHex(s.request), responses: s.responses.map(toHex) })) })),
    );
  }
  return historyFrom(
    app,
    app.vectors.map((vector) => ({ request: toHex(vector.request), response: toHex(vector.response) })),
  );
}

/**
 * Record dispatches as they run. `dispatch` behaves as `app.dispatch` does, a
 * refusal included; `history()` resolves to what has finished, in call order.
 *
 * @returns {{ dispatch(request: Uint8Array | ArrayBuffer): Promise<Uint8Array>, history(): Promise<object>, events(): number }}
 */
export function experimental_recorder(app) {
  if (app.world?.session) {
    // A session world records sessions (#629): `open()` returns a session whose steps are recorded in order.
    const sessions = [];
    const done = () => sessions.map((s) => ({ steps: s.steps.filter((step) => step !== null), ...(s.failure ? { failure: s.failure } : {}) }));
    return Object.freeze({
      dispatch: (request) => app.dispatch(request),
      open: async () => {
        const into = { steps: [] };
        sessions.push(into);
        return recordingSession(await app.open(), into);
      },
      history: () => sessionHistoryFrom(app, done()),
      events: () => done().reduce((sum, s) => sum + s.steps.length + (s.failure ? 1 : 0), 0),
    });
  }
  const slots = [];
  const finished = () => slots.filter((event) => event !== null);
  return Object.freeze({
    dispatch: async (request) => {
      const bytes = asBytes(request);
      const hex = toHex(bytes);
      const slot = slots.push(null) - 1;
      try {
        const response = await app.dispatch(bytes);
        slots[slot] = { request: hex, response: toHex(response) };
        return response;
      } catch (error) {
        slots[slot] = { request: hex, error: errorCodeOf(error) };
        throw error;
      }
    },
    history: () => historyFrom(app, finished()),
    events: () => finished().length,
  });
}

/** Run one event and compare its outcome with the recording. Returns the divergence, or null. */
async function replayEvent(app, event, index) {
  const expected = event.error === undefined ? { response: event.response } : { error: event.error };
  let actual;
  try {
    actual = { response: toHex(await app.dispatch(fromHex(event.request))) };
  } catch (error) {
    actual = { error: errorCodeOf(error) };
  }
  if (actual.response === expected.response && actual.error === expected.error) return null;
  return { index, request: event.request, expected, actual };
}

/**
 * Replay one recorded session on a fresh session. Every record runs, in order:
 * after the replay fails, the later records meet the closed session and their
 * outcome is what the runtime gives them, so every difference is counted.
 * `first` numbers the records across the whole history.
 */
async function replaySession(app, recorded, number, first) {
  const running = await app.open();
  const divergences = [];
  const records = [...recorded.steps, ...(recorded.failure ? [recorded.failure] : [])];
  try {
    for (const [offset, record] of records.entries()) {
      const expected = record.error === undefined ? { responses: record.responses } : { error: record.error };
      let actual;
      try {
        actual = { responses: (await running.dispatch(fromHex(record.request))).map(toHex) };
      } catch (error) {
        actual = { error: errorCodeOf(error) };
      }
      if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        divergences.push({ index: first + offset, session: number, step: offset, request: record.request, expected, actual });
      }
    }
  } finally {
    running.close();
  }
  return divergences;
}

function fromHex(hex) {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return bytes;
}

/**
 * The `prism/replay/v1` statement of a replay (docs/proposals/284-histories-replay.md).
 * Its one subject is the guest blob, named `guest` as `prism-code attest` names it.
 */
function statementOf(build, histories, runtime) {
  return {
    _type: STATEMENT_TYPE,
    subject: [{ name: 'guest', digest: build.guest }],
    predicateType: REPLAY_PREDICATE_TYPE,
    predicate: {
      runtime,
      model: build.model,
      histories: histories.map(({ digest, events, divergences }) => ({ digest, events, divergences: divergences.length })),
      divergenceCount: histories.reduce((sum, h) => sum + h.divergences.length, 0),
    },
  };
}

/** Bytes as a person reads them: the text when they are UTF-8, otherwise `0x` and hex. */
export function displayBytes(hex) {
  try {
    return JSON.stringify(decoder.decode(fromHex(hex)));
  } catch {
    return `0x${hex}`;
  }
}

/** A recorded or replayed outcome as a person reads it. */
export function displayOutcome(outcome) {
  if (outcome.error !== undefined) return `error ${outcome.error}`;
  if (outcome.responses !== undefined) return `[${outcome.responses.map(displayBytes).join(', ')}]`;
  return displayBytes(outcome.response);
}

/**
 * Re-execute histories against an opened application. Every event of every
 * history runs, so one replay reports every divergence and the statement
 * counts them all.
 *
 * Resolves to the result when nothing diverged. On a divergence it rejects with
 * `DeterminismViolationError` (`replay.divergence`), whose `details` is the
 * result, statement included; with `throw: false` it resolves to the result
 * instead. A history that does not parse, a repeated history and an archive
 * that fails its own integrity checks reject with `ReplayError`, always.
 *
 * @param {object} app an application from `load`
 * @param {string | Uint8Array | object | Array<string | Uint8Array | object>} histories
 * @param {{ throw?: boolean, runtime?: { name: string, version: string } }} [options]
 */
export async function experimental_replay(app, histories, options = {}) {
  const unknown = Object.keys(options ?? {}).find((key) => !['throw', 'runtime'].includes(key));
  if (unknown !== undefined) throw new ReplayError(`unknown replay option ${unknown}; expected throw, runtime`, { check: 'replay.options' });
  const { throw: throwOnDivergence = true, runtime = { name: 'prism-code', version: VERSION } } = options ?? {};
  const list = Array.isArray(histories) ? histories : [histories];
  if (list.length === 0) throw new ReplayError('replay takes at least one history', { check: 'replay.history' });
  if (!app.identities?.verified) throw new ReplayError('the archive fails its own integrity checks; a replay of it would prove nothing', { check: 'replay.holo' });
  const parsed = list.map(parseHistory);
  const digests = await Promise.all(parsed.map(historyDigest));
  if (new Set(digests.map((d) => d.blake3)).size !== digests.length) throw new ReplayError('the same history was given twice; the statement names each one once', { check: 'replay.history' });

  const session = Boolean(app.world?.session);
  for (const history of parsed) {
    // A session history says nothing about a stateless guest, and a stateless history nothing about a session (#629).
    if ((history.schema === SESSION_HISTORY_SCHEMA) !== session) {
      throw new ReplayError(
        `a ${history.schema} history cannot be replayed against the world ${app.world?.contract ?? 'of this application'}; ${session ? SESSION_HISTORY_SCHEMA : HISTORY_SCHEMA} records it`,
        { check: 'replay.history' },
      );
    }
  }

  const replayed = [];
  for (const [position, history] of parsed.entries()) {
    const divergences = [];
    if (session) {
      let index = 0;
      for (const [number, recorded] of history.sessions.entries()) {
        for (const divergence of await replaySession(app, recorded, number, index)) divergences.push(divergence);
        index += recorded.steps.length + (recorded.failure ? 1 : 0);
      }
    } else {
      for (const [index, event] of history.events.entries()) {
        const divergence = await replayEvent(app, event, index);
        if (divergence) divergences.push(divergence);
      }
    }
    replayed.push({ digest: digests[position], application: history.application, events: historyEvents(history).length, divergences });
  }
  const build = await buildOf(app);
  const divergenceCount = replayed.reduce((sum, h) => sum + h.divergences.length, 0);
  const result = {
    application: app.name ?? null,
    runtime,
    events: replayed.reduce((sum, h) => sum + h.events, 0),
    divergenceCount,
    histories: replayed,
    // A closure has no guest blob to name, so it has no statement.
    statement: build ? statementOf(build, replayed, runtime) : null,
  };
  if (divergenceCount > 0 && throwOnDivergence) {
    const first = replayed.flatMap((h) => h.divergences)[0];
    throw new DeterminismViolationError(
      `the replay diverged from its history on ${divergenceCount} of ${result.events} events; first at event ${first.index}: recorded ${displayOutcome(first.expected)}, replayed ${displayOutcome(first.actual)}`,
      { check: 'replay.divergence', diagnostic: DIVERGENCE_CODE, details: result },
    );
  }
  return result;
}
