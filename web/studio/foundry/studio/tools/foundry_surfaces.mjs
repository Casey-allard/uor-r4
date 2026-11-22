// What each of Foundry's screens asks of the back end, read from the screen's own code.
//
// A renderer screen reaches the host only through the typed IPC client (`ipc.<domain>.<method>`,
// vendor/prism/src/ipc/types). Each method is a contract with one channel, and the command registry
// (CommandRegistry.commandRegistryJson, projected to shell/command-registry-data.mjs) names the
// back-end feature that owns every channel the host answers. So the back-end features a screen needs
// are read, not asserted: walk the screen's imports, collect the IPC calls on the way, resolve each to
// its channel and each channel to its registry feature. foundry_model.mjs then refuses a screen that
// needs a registry feature none of its back-end features owns.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const studio = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const src = path.join(studio, "vendor/prism/src");
const EXTS = ["", ".ts", ".tsx", "/index.ts", "/index.tsx"];

class SurfaceError extends Error {}
const refuse = (message) => {
  throw new SurfaceError(message);
};

function resolveImport(from, spec) {
  let base;
  if (spec.startsWith("@/")) base = path.join(src, spec.slice(2));
  else if (spec.startsWith(".")) base = path.join(path.dirname(from), spec);
  else return null;
  for (const e of EXTS) {
    const p = base + e;
    if (fs.existsSync(p) && fs.statSync(p).isFile()) return p;
  }
  return null;
}

const parsed = new Map();
function fileInfo(file) {
  if (parsed.has(file)) return parsed.get(file);
  const text = fs.readFileSync(file, "utf8");
  const deps = [...text.matchAll(/(?:import|export)\s[^;]*?from\s+["']([^"']+)["']|import\(\s*["']([^"']+)["']\s*\)/g)].map((m) => resolveImport(file, m[1] ?? m[2])).filter(Boolean);
  // Calls the screen makes. Event subscriptions (ipc.events.*) are the host speaking to the screen.
  const calls = [...text.matchAll(/\bipc\.(\w+)\.(\w+)\b/g)].filter((m) => m[1] !== "events").map((m) => `${m[1]}.${m[2]}`);
  const info = { deps, calls };
  parsed.set(file, info);
  return info;
}

/** Every file a screen's entry reaches, not descending into the IPC type definitions themselves. */
function reach(entry) {
  const seen = new Set();
  const stack = [entry];
  while (stack.length) {
    const f = stack.pop();
    if (seen.has(f)) continue;
    seen.add(f);
    if (f.includes(`${path.sep}ipc${path.sep}types${path.sep}`)) continue;
    stack.push(...fileInfo(f).deps);
  }
  return seen;
}

let channelsOnce = null;
/** `domain.method` -> channel, from the contract maps and the `ipc` object of ipc/types. */
function channels() {
  if (channelsOnce) return channelsOnce;
  const dir = path.join(src, "ipc/types");
  const byContracts = {};
  const clientOf = {};
  for (const f of fs.readdirSync(dir).filter((x) => x.endsWith(".ts") && !x.includes(".test."))) {
    const t = fs.readFileSync(path.join(dir, f), "utf8");
    for (const m of t.matchAll(/export const (\w+) = \{([\s\S]*?)\n\}(?: as const)?;/g)) {
      const map = {};
      for (const x of m[2].matchAll(/\n  (\w+): define\w+\(\{\s*channel: "([^"]+)"/g)) map[x[1]] = x[2];
      if (Object.keys(map).length) byContracts[m[1]] = map;
    }
    // A stream contract is one channel the screen starts; its events are the host speaking back.
    for (const m of t.matchAll(/export const (\w+) = defineStream\(\{\s*channel: "([^"]+)"/g)) byContracts[m[1]] = { start: m[2] };
    for (const m of t.matchAll(/export const (\w+) = create\w*Client\(\s*(\w+)/g)) clientOf[m[1]] = m[2];
  }
  const ipc = fs.readFileSync(path.join(dir, "index.ts"), "utf8").match(/export const ipc = \{([\s\S]*?)\n\} as const;/)?.[1] ?? refuse("ipc/types/index.ts declares no ipc object");
  const out = new Map();
  let inEvents = false;
  let spread = null;
  for (const line of ipc.split("\n")) {
    if (/^\s*events: \{/.test(line)) inEvents = true;
    if (inEvents) continue;
    const nested = line.match(/^\s*(\w+): \{$/);
    if (nested) spread = nested[1];
    const direct = line.match(/^\s*(\w+): (\w+),$/);
    if (direct && !spread) for (const [method, channel] of Object.entries(byContracts[clientOf[direct[2]]] ?? {})) out.set(`${direct[1]}.${method}`, channel);
    const spreadOf = line.match(/^\s*\.\.\.(\w+),$/);
    if (spreadOf && spread) for (const [method, channel] of Object.entries(byContracts[clientOf[spreadOf[1]]] ?? {})) out.set(`${spread}.${method}`, channel);
    const picked = line.match(/^\s*(\w+): (\w+)\.(\w+),$/);
    if (picked && spread) {
      const channel = byContracts[clientOf[picked[2]]]?.[picked[3]];
      if (channel) out.set(`${spread}.${picked[1]}`, channel);
    }
    if (/^\s*\},$/.test(line)) spread = null;
  }
  return (channelsOnce = out);
}

/**
 * The registry features a screen reaches, by feature id, with the channels that reach each, and the
 * channels it reaches that the registry does not name (the host answers them itself). `entries`
 * are paths under vendor/prism/src; `exclude` are entries whose calls belong to another surface (the
 * shell around every screen), so each call is counted where it is made.
 */
export function surfaceCommands(entries, registry, { exclude = [] } = {}) {
  const byChannel = new Map(registry.commands.map((c) => [c.channel, c]));
  const files = (list) => new Set(list.flatMap((e) => [...reach(path.join(src, e))]));
  const shared = new Set([...files(exclude)].flatMap((f) => fileInfo(f).calls));
  const reached = new Map();
  const direct = [];
  const unresolved = [];
  for (const e of entries) if (!fs.existsSync(path.join(src, e))) refuse(`the screen entry vendor/prism/src/${e} does not exist`);
  for (const call of new Set([...files(entries)].flatMap((f) => fileInfo(f).calls))) {
    if (shared.has(call)) continue;
    const channel = channels().get(call);
    if (!channel) {
      unresolved.push(call);
      continue;
    }
    const command = byChannel.get(channel);
    if (!command) {
      direct.push(channel);
      continue;
    }
    if (!reached.has(command.feature)) reached.set(command.feature, []);
    reached.get(command.feature).push(channel);
  }
  return { reached, direct, unresolved };
}

export { SurfaceError };
