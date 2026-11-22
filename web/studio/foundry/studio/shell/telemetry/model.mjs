// Binds the Telemetry LexLean model (studio/telemetry/src, run here as its data form, data/*.json) to
// JavaScript values. Every decision is the model's: KPI values, verdicts, observations, suggestions, the
// model switch, the loop, retention and histogram buckets. This file converts numbers, puts a turn's
// events in time order and folds the model's `step` over them; it decides nothing.
//
// The model has no recursion: a turn is a fold of `step` over its events, so a turn of any length costs
// one interpreter call per event and no stack depth.
import { loadProject } from "../live/v2/interp.mjs";

const big = (n) => BigInt(Math.round(n));
const num = (b) => Number(b);
const opt = (o) => (o.c === "Option.some" ? num(o.a[0]) : null);

// `modules` maps a module name to its data form, as data/*.json hold it.
export function fromData(modules) {
  const project = loadProject(modules);
  if (project.missingPrimitives.length) throw new Error("interpreter lacks " + project.missingPrimitives.join(", "));
  return bind(project, modules.Telemetry);
}

function bind(project, data) {
  const call = (name, ...args) => project.call("Telemetry::" + name, ...args);
  // The model's named codes: its definitions without parameters.
  const names = data.declarations.filter((d) => d.kind === "definition" && d.parameters.length === 0).map((d) => d.name);
  const constant = (name) => num(call(name));
  const table = (prefix) => Object.fromEntries(names.filter((n) => n.startsWith(prefix) && /^[A-Z]/.test(n.slice(prefix.length))).map((n) => [n.slice(prefix.length, prefix.length + 1).toLowerCase() + n.slice(prefix.length + 1), constant(n)]));
  const KIND = table("kind"), KPI = table("kpi"), LEVER = table("lever"), LOOP = table("loop"), NET = table("net"), STANDING = table("standing");
  delete KPI.count;
  const event = ([kind, at, value, model]) => ({ kind: big(kind), at: big(at), value: big(value), model: big(model) });
  const observation = (o) => ({ kpi: num(o.kpi), turns: num(o.turns), over: num(o.over), worst: num(o.worst), total: num(o.total), last: num(o.last) });
  const toObservation = (o) => ({ kpi: big(o.kpi), turns: big(o.turns), over: big(o.over), worst: big(o.worst), total: big(o.total), last: big(o.last) });
  const stages = (s) => ({ think: big(s.think), lead: big(s.lead), write: big(s.write), check: big(s.check) });
  const stat = (s) => ({ model: big(s.model), reads: big(s.reads), firstTry: big(s.firstTry) });
  const loop = (l) => ({ applied: big(l.applied), pending: big(l.pending), baseline: big(l.baseline) });
  const kpiCount = constant("kpiCount");
  const decision = ({ kind, at, value = 0 }) => ({ kind: BigInt(kind), at: big(at), value: big(value) });
  const appNet = (a) => ({ opened: num(a.opened), notOpened: num(a.notOpened), allowed: num(a.allowed), blocked: num(a.blocked), size: num(a.size), last: num(a.last) });
  const toAppNet = (a) => ({ opened: big(a.opened), notOpened: big(a.notOpened), allowed: big(a.allowed), blocked: big(a.blocked), size: big(a.size), last: big(a.last) });
  const reach = (r) => ({ allowed: num(r.allowed), blocked: num(r.blocked), last: num(r.last) });
  const toReach = (r) => ({ allowed: big(r.allowed), blocked: big(r.blocked), last: big(r.last) });
  return {
    KIND, KPI, LEVER, LOOP, NET, STANDING, kpiCount,
    appKeepLimit: constant("appKeepLimit"), siteKeepLimit: constant("siteKeepLimit"),
    keepLimit: constant("keepLimit"), reloadBudget: constant("reloadBudget"),
    // One turn's KPIs: { code: microseconds or a count, or null when the turn has no such journey }.
    // Events from the page, the frame and the worker arrive out of order; the fold reads them in time
    // order (a stable sort keeps marks of the same microsecond in the order they were made).
    kpis(events) {
      let state = call("empty");
      for (const e of [...events].sort((a, b) => a[1] - b[1])) state = call("step", state, event(e));
      const out = {};
      for (let code = 0; code < kpiCount; code++) out[code] = opt(call("kpiOf", state, BigInt(code)));
      return out;
    },
    budgeted: (code) => call("budgeted", BigInt(code)),
    over: (code, value, budget) => num(call("overCode", BigInt(code), big(value), BigInt(budget))) === 1,
    budgetMicros: (budget) => num(call("budgetMicros", BigInt(budget))),
    observe(code, values, budget) {
      let o = call("observeStart", BigInt(code));
      for (const v of values) o = call("observeStep", o, big(v), BigInt(budget));
      return observation(o);
    },
    persistent: (o) => call("persistent", toObservation(o)),
    lever: (o, s) => num(call("lever", toObservation(o), stages(s))),
    severity: (o) => num(call("severity", toObservation(o))),
    firstTryPermille: (s) => num(call("firstTryPermille", stat(s))),
    // The model the loop would suggest instead of `current`, or null.
    switchTo(current, others, built) {
      let best = 0n, rate = call("firstTryPermille", stat(current));
      for (const o of others) {
        const [b, r] = [call("pickBetter", best, rate, stat(current), stat(o), toObservation(built)), call("pickRate", best, rate, stat(current), stat(o), toObservation(built))];
        best = b; rate = r;
      }
      return best === 0n ? null : num(best) - 1;
    },
    loopStep: (l, ev, offered, mean) => { const r = call("loopStep", loop(l), BigInt(ev), BigInt(offered), big(mean)); return { applied: num(r.applied), pending: num(r.pending), baseline: num(r.baseline) }; },
    assess: (before, after) => num(call("assess", big(before), big(after))),
    kept: (count) => num(call("kept", BigInt(count))),
    evicted: (count) => num(call("evicted", BigInt(count))),
    bucketMicros: (budget, index) => num(call("bucketMicros", BigInt(budget), BigInt(index))),
    // One app's network decisions, folded in time order into its totals, and the same for one site.
    appEmpty: () => appNet(call("appEmpty")),
    appStep: (a, d) => appNet(call("appStep", toAppNet(a), decision(d))),
    reachEmpty: () => reach(call("reachEmpty")),
    reachStep: (r, d) => reach(call("reachStep", toReach(r), decision(d))),
    standing: (a) => num(call("standing", toAppNet(a))),
    reachStanding: (r) => num(call("reachStanding", toReach(r))),
    appsEvicted: (count) => num(call("appsEvicted", BigInt(count))),
    sitesEvicted: (count) => num(call("sitesEvicted", BigInt(count))),
    theorems: project.theorems,
  };
}
