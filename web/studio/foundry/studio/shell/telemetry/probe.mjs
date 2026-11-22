// The probe: where a measurement is taken. It is deliberately the smallest thing in the design, because
// it runs on the hot path of the shell, the live frame and the kernel worker.
//
// Clock: performance.timeOrigin + performance.now(), in whole microseconds. performance.now() is monotonic
// and is not moved by the wall clock; adding timeOrigin puts the page, the frame and a worker, which each
// have their own origin, on one time line, so a span between two contexts is a plain subtraction.
// Resolution is what the browser grants: 5 us when the page is cross-origin isolated, 100 us otherwise
// (Chromium), more under fingerprinting protection. The value is rounded, never interpolated.
//
// Cost: a mark writes four numbers into a preallocated Float64Array. No allocation, no promise, no
// message, no storage on the hot path; the ring is drained once per turn. A mark that finds the ring full
// is counted in `dropped`, never written over an earlier one, so a span is never computed from a stale
// event. Nothing here can reach the network: this file imports nothing and calls no I/O.
//
// Capacity is 16,384 events a turn (512 KB); a replayed turn records about 130, a long one a few
// thousand. A turn that fills the ring says so in `dropped`.

// The Telemetry model's event kinds, in its code order (Telemetry.lex.tex, kind…). Held to the model by
// telemetry.test.mjs, so a name here can never mean another code there.
export const KINDS = ["enter", "token", "frame", "declaration", "pixel", "update", "edit", "editScreen", "complete", "checked", "reload", "read", "refused", "reused", "rechecked"];
export const K = Object.freeze(Object.fromEntries(KINDS.map((k, i) => [k, i])));

export const now = () => Math.round((performance.timeOrigin + performance.now()) * 1000);

const FIELDS = 4;

export const CAPACITY = 16384;

export function createProbe(capacity = CAPACITY) {
  const ring = new Float64Array(capacity * FIELDS);
  let count = 0, dropped = 0;
  return {
    // kind: one of Telemetry's kind codes; at: microseconds on the common clock (a frame or worker
    // passes its own reading, taken where the thing happened); value: a kind's quantity; model: index.
    mark(kind, value = 0, model = 0, at = now()) {
      if (count === capacity) { dropped++; return; }
      const i = count++ * FIELDS;
      ring[i] = kind; ring[i + 1] = at; ring[i + 2] = value; ring[i + 3] = model;
    },
    // The turn's events, in the order they were marked, and an empty ring for the next turn.
    drain() {
      const events = [];
      for (let i = 0; i < count; i++) events.push([ring[i * FIELDS], ring[i * FIELDS + 1], ring[i * FIELDS + 2], ring[i * FIELDS + 3]]);
      const out = { events, dropped };
      count = 0; dropped = 0;
      return out;
    },
    get size() { return count; },
  };
}
