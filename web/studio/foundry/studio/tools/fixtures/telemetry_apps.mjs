// What the worker would note about four apps: the kernel's decisions as it serves each one. They are
// synthesized, NOT captured: they show each standing the model can give (an app that reaches nothing, one
// that reaches only sites it was allowed, one stopped from a site, one stopped from opening). The shape is
// the worker's message (studio/shell/sw.template.js, "telemetry/gate"); decision kinds are the model's names.

const MIN = 60e6, HOUR = 60 * MIN;

export function fixtureApps(now = Date.now() * 1000) {
  const opened = (at, value, ...sites) => [{ kind: "opened", at, value }, ...sites.map(([site, ok]) => ({ kind: ok ? "allowed" : "blocked", at, site }))];
  const notes = [];
  // A to-do list built here: React and Tailwind from their two sites, every time it opened.
  for (let i = 0; i < 12; i++) notes.push({ id: "p:todo-" + (i % 3), key: "app:1", name: "Grocery list", decisions: opened(now - (12 - i) * 3 * HOUR, 184_320, ["https://esm.sh", true], ["https://cdn.tailwindcss.com", true]) });
  // A weather app that asked for a site the user denied.
  for (let i = 0; i < 3; i++) notes.push({ id: "p:weather", key: "app:2", name: "Weather now", decisions: opened(now - (3 - i) * 26 * HOUR, 96_000, ["https://esm.sh", true], ["https://api.open-meteo.com", false]) });
  // An installed app that asks for nothing.
  for (let i = 0; i < 5; i++) notes.push({ id: "holo:notes", name: "Field notes", installed: true, decisions: opened(now - (5 - i) * 40 * MIN, 52_224) });
  // A preview whose permission had lapsed: stopped before it opened.
  notes.push({ id: "p:budget", key: "app:3", name: "Budget planner", decisions: [{ kind: "notOpened", at: now - 50 * HOUR }] });
  return notes;
}
