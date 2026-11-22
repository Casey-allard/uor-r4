// The View's words for the model's codes: what the panel shows and what Prometheus calls each series.
// Words only; no decision lives here. Keys are the model's names (Telemetry.lex.tex), read at load.
// The wording follows the shell's surface standard (FoundrySpecification.foundrySurfaceExperienceBoundary):
// plain words, why then how then what, no technical terms. Prometheus names stay as Prometheus spells them.

export const KPI_WORDS = {
  preview: { label: "Enter to the app's first look", unit: "time" },
  declaration: { label: "Enter to the first line of the app", unit: "time" },
  pixel: { label: "First line to first drawing", unit: "time" },
  update: { label: "Longest redraw as the app is written", unit: "time" },
  edit: { label: "Longest wait for a change to show", unit: "time" },
  checked: { label: "Finished app to its check", unit: "time" },
  built: { label: "Enter to a checked app", unit: "time" },
  reloads: { label: "Times the preview reloaded", unit: "count" },
  think: { label: "Enter to the model's first word", unit: "time" },
  lead: { label: "First word to the first line of the app", unit: "time" },
  write: { label: "First line to the finished app", unit: "time" },
  firstTry: { label: "Apps that read at the first try", unit: "count" },
  repaired: { label: "Apps that read after one repair", unit: "count" },
  refused: { label: "Apps that did not read", unit: "count" },
  reused: { label: "Checks answered from earlier work", unit: "count" },
  rechecked: { label: "Checks run in full", unit: "count" },
};

// Each says why it matters, then how to change it, then what to expect.
export const LEVER_WORDS = {
  warmFrame: "The first look at the app is slow because the preview loads after you press Enter. Keep the preview ready before Enter, and the first look becomes a message instead of a page load.",
  reasoning: "Most of the wait is the model thinking before it writes. Limit its thinking for build requests, or choose a model that thinks less, and the first line arrives sooner.",
  declarationsFirst: "The first line of the app comes late in the answer. Ask the model to write the app's lines before any explanation, and it appears sooner.",
  projectEarly: "The first drawing waits for more than it needs. Draw the first line as soon as it is whole, and the app appears sooner.",
  coalesce: "Redraws take longer than one screen refresh. Combine redraws into one per refresh and move the table work out of the way, and the preview stays smooth.",
  editPath: "A change takes more than two screen refreshes to show. Send an edit on its own refresh instead of with the model's batch, and it shows sooner.",
  reuseChecks: "The check is where the time goes. Reuse earlier checks for what did not change, and an edit checks only what it touched.",
  shorterModule: "Writing the app is where the time goes. Ask for a smaller first version and add to it in later requests.",
  reloadTrigger: "The preview reloaded during a build. Find what changed its address; it should load once for each app.",
};

export const SWITCH_WORDS = (from, to, fromRate, toRate) =>
  `Builds run slow, and ${from} writes apps that read at the first try ${fromRate / 10}% of the time, against ${toRate / 10}% for ${to}. Try ${to} for build requests.`;

// Prometheus metric families. Names follow Prometheus's conventions: base unit seconds, counters end in
// _total, one family per meaning, the KPI as a label.
export const METRICS = {
  kpi: { name: "prism_kpi_seconds", help: "Latest value of a Prism build KPI in seconds, per turn." },
  threshold: { name: "prism_kpi_threshold_seconds", help: "Smallest value the Forge kernel judges over budget (budget + 1 ms), in seconds." },
  duration: { name: "prism_kpi_duration_seconds", help: "Distribution of a Prism build KPI over turns, in seconds; buckets are 1/4, 1/2, 1, 2 and 4 times the threshold." },
  turns: { name: "prism_kpi_turns", help: "Turns in which a KPI was observed." },
  overTurns: { name: "prism_kpi_over_budget_turns", help: "Turns in which a KPI was over its kernel budget." },
  reloads: { name: "prism_preview_reloads", help: "Preview reloads during build turns." },
  reads: { name: "prism_module_reads", help: "Modules that read, by model and by attempt (first, or after one repair)." },
  refusals: { name: "prism_module_refusals", help: "Modules the reader refused, by model." },
  checks: { name: "prism_checks", help: "Checks after a turn, by whether the result was reused." },
  suggestion: { name: "prism_suggestion", help: "A suggestion the on-device loop makes now; 1 while it stands." },
};

// The apps view. Keys are the model's standings (standingQuiet, standingReaches, standingBlocked); the
// standing itself is the model's decision, these are only its words. Plain words: a site, opened, reached,
// allowed, stopped; never the network's own terms.
const times = (n) => (n === 1 ? "once" : n === 2 ? "twice" : `${n} times`);
const plural = (n, one, many = one + "s") => `${n} ${n === 1 ? one : many}`;
export const APP_WORDS = {
  quiet: () => "Reaches nothing outside this device",
  reaches: (a) => `Reaches ${plural(a.sites.length, "site")}, all allowed`,
  blocked: (a) => {
    const stopped = a.sites.filter((s) => s.blocked > 0).length;
    if (a.notOpened && !stopped) return `Stopped from opening ${times(a.notOpened)}`;
    return `Stopped from reaching ${plural(stopped, "site")}` + (a.notOpened ? `, and from opening ${times(a.notOpened)}` : "");
  },
};
export const SITE_WORDS = {
  reaches: (s) => `Allowed ${times(s.allowed)}`,
  blocked: (s) => (s.allowed ? `Stopped ${times(s.blocked)}, allowed ${times(s.allowed)}` : `Stopped ${times(s.blocked)}`),
};
// The sentence that sums every app up; it leads the panel.
export const APPS_SUMMARY = (apps) => {
  if (!apps.length) return "No app has opened on this device yet. Build or open one, and what it reaches shows here.";
  const reach = apps.filter((a) => a.standing !== "quiet" && a.sites.some((s) => s.allowed > 0)).length;
  const stopped = apps.filter((a) => a.standing === "blocked").length;
  let s = `${plural(apps.length, "app")} opened on this device. `;
  s += reach ? `${reach === apps.length ? (apps.length === 1 ? "It" : "All") : reach} reached the internet` : "None reached the internet";
  s += stopped ? `, and ${stopped === 1 && apps.length === 1 ? "it" : stopped} ${stopped === 1 ? "was" : "were"} stopped at least once.`: ", and nothing was stopped.";
  return s;
};
