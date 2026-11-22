// Prometheus without egress. A browser tab cannot be scraped (it listens on nothing) and must not push
// (that would be egress), so interoperability is by file, in Prometheus's own formats, saved only when
// the user asks:
//
// - snapshot(): the text exposition format 0.0.4, no timestamps. Dropped into node_exporter's textfile
//   collector directory, it is scraped by an existing Prometheus like any other target.
// - history(): OpenMetrics 1.0 text, one point per turn at the turn's own time, ending in `# EOF`.
//   `promtool tsdb create-blocks-from openmetrics` turns it into TSDB blocks a Prometheus loads.
//
// Values are written exactly: microseconds become seconds by moving the decimal point, never through a
// float. Bucket bounds and thresholds are the model's (bucketMicros, budgetMicros).
import { METRICS } from "./words.mjs";

export const seconds = (us) => {
  if (!Number.isSafeInteger(us) || us < 0) throw new Error("not a whole number of microseconds: " + us);
  const whole = Math.floor(us / 1e6), frac = String(us % 1e6).padStart(6, "0").replace(/0+$/, "");
  return frac ? `${whole}.${frac}` : String(whole);
};
const escape = (v) => String(v).replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n");
const labels = (o) => { const e = Object.entries(o); return e.length ? "{" + e.map(([k, v]) => `${k}="${escape(v)}"`).join(",") + "}" : ""; };

// `data` is what telemetry.mjs's dataset() returns:
// { kpis: [{ name, time, budget|null, threshold|null, buckets|null }], turns: [{ at, model, kpis: {name: value|null} }], suggestions: [{ kpi, lever }] }

function cumulative(data) {
  // Per turn, the running totals a counter and a histogram carry.
  const state = { reloads: 0, reads: new Map(), refusals: new Map(), checks: { true: 0, false: 0 }, hist: new Map(), turns: new Map(), over: new Map() };
  const points = [];
  for (const turn of data.turns) {
    const k = turn.kpis;
    state.reloads += k.reloads ?? 0;
    const m = turn.model;
    for (const [attempt, key] of [["first", "firstTry"], ["repair", "repaired"]]) {
      const id = m + "\u0000" + attempt;
      state.reads.set(id, (state.reads.get(id) ?? 0) + (k[key] ?? 0));
    }
    state.refusals.set(m, (state.refusals.get(m) ?? 0) + (k.refused ?? 0));
    state.checks.true += k.reused ?? 0;
    state.checks.false += k.rechecked ?? 0;
    for (const kpi of data.kpis) {
      const v = k[kpi.name];
      if (v == null || kpi.budget == null) continue;
      state.turns.set(kpi.name, (state.turns.get(kpi.name) ?? 0) + 1);
      state.over.set(kpi.name, (state.over.get(kpi.name) ?? 0) + (turn.over[kpi.name] ? 1 : 0));
      if (!kpi.buckets) continue;
      const h = state.hist.get(kpi.name) ?? { counts: kpi.buckets.map(() => 0), count: 0, sum: 0 };
      kpi.buckets.forEach((le, i) => { if (v <= le) h.counts[i]++; });
      h.count++; h.sum += v;
      state.hist.set(kpi.name, h);
    }
    points.push({ at: turn.at, turn, reloads: state.reloads, reads: new Map(state.reads), refusals: new Map(state.refusals), checks: { ...state.checks },
      hist: new Map([...state.hist].map(([n, h]) => [n, { counts: [...h.counts], count: h.count, sum: h.sum }])), turns: new Map(state.turns), over: new Map(state.over) });
  }
  return points;
}

// One renderer for both formats; `om` selects OpenMetrics (typed families, timestamps, # EOF).
function render(data, om) {
  const points = cumulative(data);
  if (!points.length) return om ? "# EOF\n" : "";
  const shown = om ? points : [points[points.length - 1]];
  const ts = (p) => (om ? " " + seconds(p.at) : "");
  const out = [];
  const family = (metric, type, unit) => {
    const name = metric.name;
    const sampleBase = !om && type === "counter" ? name + "_total" : !om && type === "info" ? name + "_info" : name;
    out.push(`# HELP ${sampleBase} ${metric.help}`);
    out.push(`# TYPE ${sampleBase} ${!om && type === "info" ? "gauge" : type}`);
    if (om && unit) out.push(`# UNIT ${name} ${unit}`);
  };
  const timeKpis = data.kpis.filter((k) => k.time);

  family(METRICS.kpi, "gauge", "seconds");
  for (const kpi of timeKpis) for (const p of shown) { const v = p.turn.kpis[kpi.name]; if (v != null) out.push(`${METRICS.kpi.name}${labels({ kpi: kpi.name })} ${seconds(v)}${ts(p)}`); }

  family(METRICS.threshold, "gauge", "seconds");
  for (const kpi of timeKpis.filter((k) => k.threshold != null)) for (const p of shown) out.push(`${METRICS.threshold.name}${labels({ kpi: kpi.name })} ${seconds(kpi.threshold)}${ts(p)}`);

  family(METRICS.duration, "histogram", "seconds");
  for (const kpi of timeKpis.filter((k) => k.buckets)) for (const p of shown) {
    const h = p.hist.get(kpi.name);
    if (!h) continue;
    kpi.buckets.forEach((le, i) => out.push(`${METRICS.duration.name}_bucket${labels({ kpi: kpi.name, le: seconds(le) })} ${h.counts[i]}${ts(p)}`));
    out.push(`${METRICS.duration.name}_bucket${labels({ kpi: kpi.name, le: "+Inf" })} ${h.count}${ts(p)}`);
    out.push(`${METRICS.duration.name}_count${labels({ kpi: kpi.name })} ${h.count}${ts(p)}`);
    out.push(`${METRICS.duration.name}_sum${labels({ kpi: kpi.name })} ${seconds(h.sum)}${ts(p)}`);
  }

  const counter = (metric, rows) => {
    family(metric, "counter");
    const keys = [...new Set(shown.flatMap((p) => rows(p).map(([l]) => JSON.stringify(l))))];
    for (const key of keys) for (const p of shown) { const row = rows(p).find(([l]) => JSON.stringify(l) === key); if (row) out.push(`${metric.name}_total${labels(row[0])} ${row[1]}${ts(p)}`); }
  };
  counter(METRICS.turns, (p) => [...p.turns].map(([kpi, n]) => [{ kpi }, n]));
  counter(METRICS.overTurns, (p) => [...p.over].map(([kpi, n]) => [{ kpi }, n]));
  counter(METRICS.reloads, (p) => [[{}, p.reloads]]);
  counter(METRICS.reads, (p) => [...p.reads].map(([id, n]) => { const [model, attempt] = id.split("\u0000"); return [{ model, attempt }, n]; }));
  counter(METRICS.refusals, (p) => [...p.refusals].map(([model, n]) => [{ model }, n]));
  counter(METRICS.checks, (p) => [[{ reused: "true" }, p.checks.true], [{ reused: "false" }, p.checks.false]]);

  family(METRICS.suggestion, "info");
  const last = points[points.length - 1];
  const infoName = METRICS.suggestion.name + "_info";
  for (const s of data.suggestions) out.push(`${infoName}${labels({ kpi: s.kpi, lever: s.lever })} 1${om ? " " + seconds(last.at) : ""}`);

  if (om) out.push("# EOF");
  return out.join("\n") + "\n";
}

export const snapshot = (data) => render(data, false);
export const history = (data) => render(data, true);

// Alerting and recording rules that judge exactly as the kernel does: a value is over when it reaches
// the threshold (budget + 1 ms). `promtool check rules` and `promtool test rules` hold this file.
export const RULES = `groups:
  - name: prism-kpi
    rules:
      - record: prism:kpi_over_budget
        expr: ${METRICS.kpi.name} >= on(kpi) ${METRICS.threshold.name}
      - alert: PrismKpiOverBudget
        expr: ${METRICS.kpi.name} >= on(kpi) ${METRICS.threshold.name}
        labels:
          severity: warning
        annotations:
          summary: "Prism KPI {{ $labels.kpi }} is over its kernel budget"
      - alert: PrismPreviewReloaded
        expr: increase(${METRICS.reloads.name}_total[1h]) > 0
        labels:
          severity: warning
        annotations:
          summary: "A Prism build turn reloaded the preview"
`;
