// Turns synthesized from the owner's hand measurements (2026-11): Enter to preview frame 1.3 s, Enter to
// first declaration 99 to 290 s, first declaration to first pixel 31 to 64 ms, work per update 4.8 to
// 8.5 ms, edit to screen 16 to 25 ms, module complete to Checked 35 to 70 ms, Enter to Checked 167 to
// 196 s, no preview reloads. They are NOT captured turns: no analytics existed to capture them. They seed
// the history so the loop has something to judge; the replayed turn is the measured one.
// `kind` codes are the model's, passed in, so a fixture cannot disagree with the model.

const S = 1e6, MS = 1e3;

// think: Enter to first token; lead: token to first declaration; write: declaration to complete.
const TURNS = [
  { model: "model-a", think: 80 * S, lead: 19 * S, write: 70 * S, check: 41 * MS, pixel: 31 * MS, updates: [4.8, 6.1, 7.2], edit: 16 * MS, attempts: [1] },
  { model: "model-a", think: 120 * S, lead: 25 * S, write: 40 * S, check: 70 * MS, pixel: 64 * MS, updates: [5.0, 8.5], edit: 25 * MS, attempts: [2] },
  { model: "model-b", think: 70 * S, lead: 29 * S, write: 72 * S, check: 35 * MS, pixel: 40 * MS, updates: [4.9, 5.5], edit: 18 * MS, attempts: [1] },
  { model: "model-a", think: 95 * S, lead: 20 * S, write: 60 * S, check: 52 * MS, pixel: 44 * MS, updates: [6.0, 7.7, 8.1], edit: 21 * MS, attempts: [2] },
  { model: "model-b", think: 90 * S, lead: 15 * S, write: 75 * S, check: 47 * MS, pixel: 38 * MS, updates: [5.2], edit: 17 * MS, attempts: [1] },
  { model: "model-a", think: 110 * S, lead: 24 * S, write: 50 * S, check: 60 * MS, pixel: 50 * MS, updates: [7.0, 7.9], edit: 23 * MS, attempts: [1] },
  { model: "model-b", think: 85 * S, lead: 18 * S, write: 80 * S, check: 39 * MS, pixel: 35 * MS, updates: [4.8, 6.6], edit: 16 * MS, attempts: [1] },
  { model: "model-b", think: 100 * S, lead: 20 * S, write: 66 * S, check: 45 * MS, pixel: 42 * MS, updates: [5.9], edit: 19 * MS, attempts: [1] },
  { model: "model-a", think: 105 * S, lead: 22 * S, write: 55 * S, check: 66 * MS, pixel: 58 * MS, updates: [8.0, 8.4], edit: 24 * MS, attempts: [2] },
  { model: "model-b", think: 92 * S, lead: 17 * S, write: 71 * S, check: 43 * MS, pixel: 37 * MS, updates: [5.1, 5.8], edit: 17 * MS, attempts: [1] },
  { model: "model-a", think: 115 * S, lead: 26 * S, write: 45 * S, check: 69 * MS, pixel: 61 * MS, updates: [7.5], edit: 22 * MS, attempts: [1] },
];

export function fixtureTurns(KIND, start = Date.UTC(2026, 10, 10, 9) * 1000) {
  return TURNS.map((t, i) => {
    const enter = start + i * 3600 * S, events = [];
    const mark = (kind, at, value = 0) => events.push([KIND[kind], Math.round(at), Math.round(value), 0]);
    mark("enter", enter);
    mark("frame", enter + 1.3 * S);
    mark("token", enter + t.think);
    const decl = enter + t.think + t.lead;
    mark("declaration", decl);
    mark("pixel", decl + t.pixel);
    t.updates.forEach((ms, j) => { mark("update", decl + t.pixel + j * 20 * MS, ms * MS); mark("pixel", decl + t.pixel + j * 20 * MS + 1); });
    const complete = decl + t.write;
    mark("complete", complete);
    t.attempts.forEach((a) => mark("read", complete + t.check - 1, a));
    if (t.attempts.includes(2)) mark("refused", complete + 1);
    mark("rechecked", complete + t.check - 1);
    mark("checked", complete + t.check);
    mark("edit", complete + 5 * S);
    mark("editScreen", complete + 5 * S + t.edit);
    return { model: t.model, events, dropped: 0 };
  });
}
