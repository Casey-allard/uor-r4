/**
 * Tracing, the first middleware (#295): one OpenTelemetry span per step the
 * runtime performs, named `prism.<step>`.
 *
 * The caller passes the tracer, `trace.getTracer(...)` from
 * `@opentelemetry/api` or anything with the same `startActiveSpan`. The
 * package therefore depends on nothing: an application that traces already
 * has the API and its SDK, and one that does not pays nothing.
 *
 * Experimental (STABILITY.md).
 */
import { MIDDLEWARE_STEPS } from './middleware.mjs';
import { fail } from './errors.mjs';

/** `SpanStatusCode.ERROR` and `SpanKind.INTERNAL` of `@opentelemetry/api`, which this file does not import. */
const STATUS_ERROR = 2;
const KIND_INTERNAL = 0;

export const TRACING_TAG = 'tracing';

function attributesBefore(step, args, context) {
  const attributes = { 'prism.step': step };
  const app = context.application;
  if (app?.name) attributes['prism.application.name'] = app.name;
  if (app?.kind) attributes['prism.application.kind'] = app.kind;
  if (args?.request instanceof Uint8Array) attributes['prism.request.size'] = args.request.length;
  return attributes;
}

function attributesAfter(step, result) {
  if (step === 'verify' && result && typeof result === 'object') {
    return { 'prism.verdict.level': result.level, 'prism.verdict.passed': result.passed };
  }
  if ((step === 'dispatch' || step === 'record') && result instanceof Uint8Array) return { 'prism.response.size': result.length };
  return {};
}

function tracingMiddleware(tracer, step) {
  return (next, context) => async (args) =>
    tracer.startActiveSpan(`prism.${step}`, { kind: KIND_INTERNAL, attributes: attributesBefore(step, args, context) }, async (span) => {
      try {
        const result = await next(args);
        span.setAttributes(attributesAfter(step, result));
        return result;
      } catch (error) {
        span.recordException(error);
        span.setStatus({ code: STATUS_ERROR, message: error?.message ?? String(error) });
        if (error?.code) span.setAttribute('prism.error.code', error.code);
        throw error;
      } finally {
        span.end();
      }
    });
}

/**
 * A `Pluggable` that adds one span per step: `stack.use(experimental_tracing({ tracer }))`.
 * Each middleware is named `tracing.<step>`, tagged `tracing`, and sits first
 * (outermost) in its step so its span covers the other middleware too.
 *
 * @param {{ tracer: { startActiveSpan: Function }, steps?: string[] }} options
 */
export function experimental_tracing({ tracer, steps = MIDDLEWARE_STEPS } = {}) {
  if (!tracer || typeof tracer.startActiveSpan !== 'function') {
    fail('middleware.shape', 'experimental_tracing needs a tracer with startActiveSpan, such as trace.getTracer() from @opentelemetry/api');
  }
  return Object.freeze({
    applyToStack(stack) {
      for (const step of steps) {
        stack.add(tracingMiddleware(tracer, step), { step, name: `tracing.${step}`, tags: [TRACING_TAG], priority: 'high' });
      }
    },
  });
}
