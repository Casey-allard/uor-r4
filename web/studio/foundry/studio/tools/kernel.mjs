// The Forge kernel, pinned into the shell.
//
// The kernel is a Prism application published by UOR-Foundation/foundry: a LexLean model whose
// guarantees Lean proves (the build phases, FK-01 to FK-04, and the capability gate, FK-05 to FK-09),
// built into a .holo whose guest is that model compiled to Core-Wasm. Forge does not trust a URL for
// it. KERNEL_REV names the holo by its archive fingerprint, the BLAKE3 of every byte before the
// footer, which is the digest `prism-code check` prints and the kernel's page attests; the footer
// must carry the same value. shell/kernel/kernel.holo holds those bytes.
// This tool refuses any other bytes, opens the holo through prism-code (every identity checked),
// replays every acceptance vector through the guest, and writes what the worker needs to run it:
//
//   shell/kernel/guest.wasm   the guest, byte for byte as the holo carries it
//   shell/kernel/kernel.json  the holo's address, the guest's SHA-256, and every vector
//
// At run time shell/kernel.js checks the guest's SHA-256 again and replays every vector again before
// the worker asks it anything.
//
//   node tools/kernel.mjs            regenerate from the committed holo
//   node tools/kernel.mjs --check    the Kernel gate: the committed files are what the holo says
//   node tools/kernel.mjs --update   fetch the live holo, pin its address in KERNEL_REV, regenerate
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const PRISM_CODE = "0.8.0";
const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const rev = path.join(root, "KERNEL_REV");
const dir = path.join(root, "shell", "kernel");
const holoPath = path.join(dir, "kernel.holo");
const mode = process.argv[2] || "";

function prismCode() {
  const prefix = path.join(os.tmpdir(), "hologram-forge-prism-code-" + PRISM_CODE);
  const entry = path.join(prefix, "node_modules", "prism-code", "src", "index.mjs");
  if (!fs.existsSync(entry)) {
    execFileSync("npm", ["install", "--no-save", "--no-audit", "--no-fund", "--prefix", prefix, "prism-code@" + PRISM_CODE], { stdio: "inherit", shell: process.platform === "win32" });
  }
  return import(pathToFileURL(entry).href);
}

function readRev() {
  const [source, digest] = fs.readFileSync(rev, "utf8").split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
  if (!source || !/^blake3:[0-9a-f]{64}$/.test(digest || "")) throw new Error("KERNEL_REV must hold the holo's URL, then its blake3:<64 hex> address");
  return { source, digest };
}

const b64 = (bytes) => Buffer.from(bytes).toString("base64");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
// The holo's identity as Prism names it: its archive fingerprint, checked against its own footer.
function identity(pc, bytes) {
  const parsed = pc.parseArchive(bytes);
  if (parsed.footer !== parsed.fingerprint) throw new Error("the holo's footer does not match its body");
  return "blake3:" + parsed.fingerprint;
}

async function generate(pc, bytes, rec) {
  const digest = identity(pc, bytes);
  if (digest !== rec.digest) throw new Error(`kernel.holo is ${digest}, KERNEL_REV pins ${rec.digest}`);
  const app = await pc.loadArchive(bytes);
  const acceptance = await app.acceptance();
  if (acceptance.passed !== acceptance.total) throw new Error(`kernel acceptance: ${acceptance.passed} of ${acceptance.total} vectors passed`);
  const guest = pc.resolveArchive(pc.parseArchive(bytes)).guest.content;
  const meta = {
    spec: "hologram-forge/kernel/1",
    application: app.name,
    source: rec.source,
    digest,
    guest: { path: "kernel/guest.wasm", sha256: sha256(guest), bytes: guest.length },
    limits: app.limits,
    vectors: app.vectors.map((v) => [b64(v.request), b64(v.response)]),
  };
  return { guest, json: JSON.stringify(meta, null, 1) + "\n", total: acceptance.total, name: app.name };
}

const pc = await prismCode();
if (mode === "--update") {
  const rec = readRev();
  const response = await fetch(rec.source, { cache: "no-store" });
  if (!response.ok) throw new Error(`${rec.source} answered ${response.status}`);
  const bytes = new Uint8Array(await response.arrayBuffer());
  const digest = identity(pc, bytes);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(holoPath, bytes);
  fs.writeFileSync(rev, rec.source + "\n" + digest + "\n");
  console.log(`kernel: pinned ${digest} from ${rec.source}`);
}
const rec = readRev();
const bytes = new Uint8Array(fs.readFileSync(holoPath));
const out = await generate(pc, bytes, rec);
const guestFile = path.join(dir, "guest.wasm");
const jsonFile = path.join(dir, "kernel.json");
if (mode === "--check") {
  const bad = [];
  if (!fs.existsSync(guestFile) || !Buffer.from(out.guest).equals(fs.readFileSync(guestFile))) bad.push("shell/kernel/guest.wasm");
  if (!fs.existsSync(jsonFile) || fs.readFileSync(jsonFile, "utf8").replace(/\r\n/g, "\n") !== out.json) bad.push("shell/kernel/kernel.json");
  if (bad.length) { console.error("kernel gate refused: " + bad.join(", ") + " differ from what " + rec.digest + " says; run node tools/kernel.mjs"); process.exit(1); }
} else {
  fs.writeFileSync(guestFile, out.guest);
  fs.writeFileSync(jsonFile, out.json);
}
console.log(`kernel: ${out.name} ${rec.digest}, ${out.total} of ${out.total} vectors replayed, guest ${out.guest.length} bytes${mode === "--check" ? ", committed files match" : ""}`);
