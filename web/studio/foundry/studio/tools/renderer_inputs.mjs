// Execute the production bridge and the imported schemas, not a schema imitation.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "../vendor/prism/node_modules/esbuild/lib/main.js";
import { invokeHost, unwrapHost } from "../shell/host-boundary.mjs";
const root = fileURLToPath(new URL("../vendor/prism/", import.meta.url));
const out = path.join(root, "node_modules/.cache-renderer-inputs.mjs");
await build({ stdin: { contents: 'import "./hologram.contracts.ts"; export { appContracts } from "./src/ipc/types/app"; export { createClient } from "./src/ipc/contracts/core";', resolveDir: root, loader: "ts" }, outfile: out, bundle: true, format: "esm", platform: "node", logLevel: "silent", alias: { "@": path.join(root, "src") }, loader: { ".png": "empty", ".svg": "empty", ".css": "empty" } });
globalThis.window = {};
const { appContracts, createClient } = await import(pathToFileURL(out));
const outputs = window.__contracts, inputs = window.__inputContracts;
assert.ok(Object.keys(outputs).length > 100);
for (const channel of Object.keys(outputs)) assert.equal(typeof inputs[channel]?.safeParse, "function", `missing imported input: ${channel}`);
let effects = 0, accepted;
const contracts = Object.values(appContracts);
const get = contracts.find(c => c.channel === "get-app");
assert.ok(get);
const invoke = (channel, ...args) => invokeHost(channel, args, value => { effects++; accepted = value; return null; }, null, { schema: inputs[channel] });
window.electron = { ipcRenderer: { invokeEnvelope: (channel, ...args) => invoke(channel, ...args) } };
// The production bridge's input object is the imported schema object itself.
assert.equal(inputs[get.channel], get.input);
for (const value of ["private", null, {}, NaN, Infinity]) {
  const result = await invoke(get.channel, value);
  assert.equal(result.ok, false); assert.equal(result.error.code, "invalidInput");
}
assert.equal(effects, 0);
assert.equal(unwrapHost(await invoke(get.channel, 7)), null);
assert.equal(accepted, 7); assert.equal(effects, 1);
const read = createClient({ get });
await assert.rejects(read.get("private"), error => error.code === "invalidInput");
assert.equal(effects, 1);
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
assert.match(host, /window\.__inputContracts/);
console.log(JSON.stringify({ outputContracts: Object.keys(outputs).length, inputContracts: Object.keys(inputs).length, invalidCommands: "refused before effects", validCommand: "admitted" }));
