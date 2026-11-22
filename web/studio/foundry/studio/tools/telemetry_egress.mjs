// Run: node studio/tools/telemetry_egress.mjs   (exit 1 on any finding)
// The static half of zero egress for build telemetry. The kernel's half is FK-25 to FK-27 (proved: a
// record may be stored, shown, saved when the user asks, or read by an on-device model, and nothing
// else); the runtime half is the tripwire test in telemetry.test.mjs and the request log of the browser
// test. This gate holds the source:
//
// - every file in studio/shell/telemetry is scanned (comments stripped first; strings are not): no
//   network API, no URL with a scheme, no postMessage to "*", no `.src`/`href` set to anything but a
//   blob or relative URL, no import but its siblings and the shell's interpreter;
// - nothing in the repository outside the telemetry reads a record: only the readers named below
//   import the store or the telemetry module;
// - its own blind spots are refused: a file it neither scans nor names, a missing file, an empty scan.
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../..", import.meta.url));
const DIR = "studio/shell/telemetry";
export const SHIPPED = ["probe.mjs", "model.mjs", "store.mjs", "telemetry.mjs", "prometheus.mjs", "words.mjs", "panel.mjs"];
const DATA = ["data/Telemetry.json", "data/TelemetryProofs.json", "data/modules.json", "data/source.json"];
// Who may hold a record: the host records turns and opens the panel, and the tests.
const READERS = ["studio/shell/host.js", "studio/tools/telemetry.test.mjs", "studio/tools/telemetry_defects.mjs"];
const ALLOWED_IMPORTS = /^(\.\/[a-z-]+\.mjs|\.\.\/live\/v2\/interp\.mjs)$/;

const NETWORK = [
  [/\bfetch\s*\(/, "fetch"], [/\bXMLHttpRequest\b/, "XMLHttpRequest"], [/\bWebSocket\b/, "WebSocket"], [/\bEventSource\b/, "EventSource"],
  [/\bsendBeacon\b/, "sendBeacon"], [/\bRTCPeerConnection\b/, "RTCPeerConnection"], [/\bWebTransport\b/, "WebTransport"], [/\bimportScripts\b/, "importScripts"],
  [/\bserviceWorker\b/, "serviceWorker"], [/\bwindow\.open\s*\(/, "window.open"], [/\bnew\s+(Shared)?Worker\s*\(/, "Worker"], [/\blocation\s*\.\s*(href|assign|replace)\b/, "navigation"],
  [/\b[a-z][a-z0-9+.-]*:\/\//i, "a URL with a scheme"], [/postMessage\([^)]*["']\*["']/, "postMessage to any origin"],
  [/\.(src|href|action)\s*=(?!\s*(url|URL\.createObjectURL\())/, "src/href/action set to something other than a blob URL"],
  [/\bsetAttribute\s*\(\s*["'](src|href|action)["']/, "src/href/action set by attribute"], [/\bnavigator\s*\.\s*(sendBeacon|connection)\b/, "navigator network"],
  [/\bimport\s*\(/, "a dynamic import"],
];
const stripComments = (text) => text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:"'\\])\/\/[^\n]*/g, "$1");

export function scan(root = repo) {
  const findings = [];
  const dir = join(root, DIR);
  const at = (file, message) => findings.push(`${DIR}/${file}: ${message}`);
  if (!existsSync(dir)) return [`${DIR}: missing`];
  for (const name of readdirSync(dir)) {
    if (statSync(join(dir, name)).isDirectory()) { if (name !== "data") at(name, "a directory the gate does not know"); continue; }
    if (!SHIPPED.includes(name)) at(name, "a file the gate neither scans nor names");
  }
  for (const name of readdirSync(join(dir, "data"))) if (!DATA.includes("data/" + name)) at("data/" + name, "a data file the gate does not name");
  for (const name of DATA) if (!existsSync(join(dir, name))) at(name, "named but missing");
  let scanned = 0;
  for (const name of SHIPPED) {
    const path = join(dir, name);
    if (!existsSync(path)) { at(name, "named but missing"); continue; }
    const text = stripComments(readFileSync(path, "utf8"));
    scanned++;
    text.split("\n").forEach((line, i) => { for (const [re, what] of NETWORK) if (re.test(line)) at(name, `line ${i + 1}: ${what}`); });
    for (const m of text.matchAll(/(?:^|\n)\s*(?:import|export)\s[^;]*?from\s*["']([^"']+)["']|\bimport\s*\(\s*["']([^"']+)["']|\bawait import\(\s*["']([^"']+)["']/g)) {
      const spec = m[1] ?? m[2] ?? m[3];
      if (!ALLOWED_IMPORTS.test(spec)) at(name, `imports ${spec}`);
    }
  }
  if (scanned === 0) findings.push("nothing scanned");
  // Outside the telemetry, nothing but the named readers holds a record.
  const walk = (d) => readdirSync(d).flatMap((n) => {
    const p = join(d, n);
    if (["node_modules", ".git", "target", ".lexlean", ".lake", "vendor", "out"].includes(n)) return [];
    return statSync(p).isDirectory() ? walk(p) : /\.(m?js|jsx|ts|tsx|html)$/.test(n) ? [p] : [];
  });
  for (const top of ["studio", "v2", "site", "app"].map((d) => join(root, d)).filter(existsSync)) {
    for (const file of walk(top)) {
      const rel = relative(root, file).split(sep).join("/");
      if (READERS.includes(rel) || rel.startsWith(DIR + "/")) continue;
      if (/telemetry\/(store|telemetry)\.mjs/.test(readFileSync(file, "utf8"))) findings.push(`${rel}: reads telemetry records`);
    }
  }
  return findings;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const findings = scan();
  for (const f of findings) console.log("refused: " + f);
  console.log(findings.length ? `telemetry egress gate: ${findings.length} finding(s)` : `telemetry egress gate: ${SHIPPED.length} files clean`);
  process.exit(findings.length ? 1 : 0);
}
