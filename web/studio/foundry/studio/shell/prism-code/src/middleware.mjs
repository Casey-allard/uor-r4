/**
 * The middleware stack (issue #295): named, tagged middleware over the four
 * steps the runtime performs, ordered by step, priority and position relative
 * to another named middleware. The precedent is the AWS SDK for JavaScript v3
 * stack; the difference is that each step here wraps one operation of its own
 * rather than one request passing through every step:
 *
 * - `verify` wraps `Application.experimental_verify`: `{ options }` in, the verdict out.
 * - `instantiate` wraps creating a guest instance: `{ module, imports }` in, the
 *   `WebAssembly.Instance` out. Import linking happens here, so a capability
 *   check (#279) can be a middleware at this step.
 * - `dispatch` wraps running one request: `{ request, instance }` in, the
 *   response bytes out (`instance` is absent for a closure).
 * - `record` wraps keeping the exchange: `{ request, response }` in, the
 *   response out. The runtime keeps nothing itself; histories (#284) hook here.
 *
 * A middleware is `(next, context) => async (args) => result`. The first one
 * in a step's order is the outermost: it sees the arguments first and the
 * result last. A middleware that throws anything but a `PrismError` raises a
 * `MiddlewareError` naming it; an error thrown by the runtime or by a
 * middleware further in passes through unchanged.
 *
 * Experimental (STABILITY.md).
 */
import { MiddlewareError, PrismError, fail } from './errors.mjs';

export const MIDDLEWARE_STEPS = Object.freeze(['verify', 'instantiate', 'dispatch', 'record']);
const PRIORITIES = ['high', 'normal', 'low'];
const RELATIONS = ['before', 'after'];

function checkMiddleware(middleware) {
  if (typeof middleware !== 'function') fail('middleware.shape', 'a middleware is a function (next, context) => handler');
}

function checkTags(tags) {
  if (!Array.isArray(tags) || tags.some((t) => typeof t !== 'string')) fail('middleware.shape', 'tags must be an array of strings');
  return [...tags];
}

/** Call `middleware` around `next`, telling its own failures apart from the ones it passes on. */
function wrap(entry, next, context) {
  return async (args) => {
    let passed = {};
    const inner = async (innerArgs) => {
      try {
        return await next(innerArgs);
      } catch (error) {
        passed = { error };
        throw error;
      }
    };
    try {
      const handler = entry.middleware(inner, context);
      if (typeof handler !== 'function') fail('middleware.shape', `middleware ${entry.name ?? '(anonymous)'} did not return a handler`);
      return await handler(args);
    } catch (error) {
      if (error instanceof PrismError || ('error' in passed && passed.error === error)) throw error;
      const label = entry.name ?? '(anonymous)';
      throw new MiddlewareError(`middleware ${label} at step ${entry.step} failed: ${error?.message ?? String(error)}`, {
        details: { name: entry.name ?? null, step: entry.step, tags: [...entry.tags] },
        cause: error,
      });
    }
  };
}

export class MiddlewareStack {
  #absolute = [];
  #relative = [];

  #named(name) {
    return this.#absolute.find((e) => e.name === name) ?? this.#relative.find((e) => e.name === name);
  }

  #claim(name, override) {
    if (name === undefined) return;
    if (typeof name !== 'string' || name === '') fail('middleware.shape', 'a middleware name is a non-empty string');
    if (!this.#named(name)) return;
    if (!override) fail('middleware.name', `a middleware named ${name} is already on the stack; pass override: true to replace it`);
    this.remove(name);
  }

  /** Add a middleware at a step: `{ step, name?, tags?, priority?: 'high' | 'normal' | 'low', override? }`. */
  add(middleware, options = {}) {
    checkMiddleware(middleware);
    const { step, name, tags = [], priority = 'normal', override = false } = options;
    if (!MIDDLEWARE_STEPS.includes(step)) fail('middleware.step', `step must be one of ${MIDDLEWARE_STEPS.join(', ')}, not ${JSON.stringify(step)}`);
    if (!PRIORITIES.includes(priority)) fail('middleware.shape', `priority must be one of ${PRIORITIES.join(', ')}`);
    const entry = { middleware, step, name, tags: checkTags(tags), priority };
    this.#claim(name, override);
    this.#absolute.push(entry);
    return this;
  }

  /**
   * Add a middleware next to a named one, at that one's step:
   * `{ relation: 'before' | 'after', toMiddleware, name?, tags?, override? }`.
   * The target may be added later; it must be on the stack when a step runs.
   */
  addRelativeTo(middleware, options = {}) {
    checkMiddleware(middleware);
    const { relation, toMiddleware, name, tags = [], override = false } = options;
    if (!RELATIONS.includes(relation)) fail('middleware.shape', `relation must be before or after, not ${JSON.stringify(relation)}`);
    if (typeof toMiddleware !== 'string') fail('middleware.shape', 'toMiddleware names the middleware to sit next to');
    const entry = { middleware, name, tags: checkTags(tags), relation, toMiddleware };
    this.#claim(name, override);
    this.#relative.push(entry);
    return this;
  }

  /** Remove a middleware by name or by the function itself. Returns whether anything was removed. */
  remove(nameOrMiddleware) {
    const match = typeof nameOrMiddleware === 'string' ? (e) => e.name === nameOrMiddleware : (e) => e.middleware === nameOrMiddleware;
    const before = this.#absolute.length + this.#relative.length;
    this.#absolute = this.#absolute.filter((e) => !match(e));
    this.#relative = this.#relative.filter((e) => !match(e));
    return this.#absolute.length + this.#relative.length !== before;
  }

  /** Remove every middleware carrying `tag`. Returns whether anything was removed. */
  removeByTag(tag) {
    const before = this.#absolute.length + this.#relative.length;
    this.#absolute = this.#absolute.filter((e) => !e.tags.includes(tag));
    this.#relative = this.#relative.filter((e) => !e.tags.includes(tag));
    return this.#absolute.length + this.#relative.length !== before;
  }

  /** Apply a `Pluggable`, an object with `applyToStack(stack)`. */
  use(pluggable) {
    if (!pluggable || typeof pluggable.applyToStack !== 'function') fail('middleware.shape', 'a pluggable has an applyToStack(stack) method');
    pluggable.applyToStack(this);
    return this;
  }

  /** A copy with the same entries; changing one stack leaves the other alone. */
  clone() {
    const copy = new MiddlewareStack();
    for (const e of this.#absolute) copy.add(e.middleware, { step: e.step, name: e.name, tags: e.tags, priority: e.priority });
    for (const e of this.#relative) copy.addRelativeTo(e.middleware, { relation: e.relation, toMiddleware: e.toMiddleware, name: e.name, tags: e.tags });
    return copy;
  }

  /** Every entry in the order it runs: by step, then outermost first. */
  identify() {
    return this.#ordered().map((e) => ({ step: e.step, name: e.name ?? null, tags: [...e.tags] }));
  }

  #ordered() {
    const pending = new Map();
    for (const e of this.#relative) {
      if (!pending.has(e.toMiddleware)) pending.set(e.toMiddleware, []);
      pending.get(e.toMiddleware).push(e);
    }
    const placed = new Set();
    const expand = (entry, step) => {
      const resolved = { ...entry, step };
      placed.add(entry);
      const around = entry.name === undefined ? [] : pending.get(entry.name) ?? [];
      const before = around.filter((e) => e.relation === 'before').flatMap((e) => expand(e, step));
      const after = around.filter((e) => e.relation === 'after').flatMap((e) => expand(e, step));
      return [...before, resolved, ...after];
    };
    const order = [];
    for (const step of MIDDLEWARE_STEPS) {
      const here = this.#absolute.filter((e) => e.step === step);
      for (const priority of PRIORITIES) {
        for (const e of here.filter((x) => x.priority === priority)) order.push(...expand(e, step));
      }
    }
    const stray = this.#relative.find((e) => !placed.has(e));
    if (stray) {
      fail('middleware.relative', `middleware ${stray.name ?? '(anonymous)'} is ${stray.relation} ${stray.toMiddleware}, which is not on the stack`);
    }
    return order;
  }

  /** The handler for one step: every middleware at that step around `terminal`. */
  resolve(step, terminal, context = {}) {
    if (!MIDDLEWARE_STEPS.includes(step)) fail('middleware.step', `step must be one of ${MIDDLEWARE_STEPS.join(', ')}, not ${JSON.stringify(step)}`);
    const here = this.#ordered().filter((e) => e.step === step);
    const stepContext = { ...context, step };
    let handler = terminal;
    for (let i = here.length - 1; i >= 0; i--) handler = wrap(here[i], handler, stepContext);
    return handler;
  }
}

export function createMiddlewareStack() {
  return new MiddlewareStack();
}
