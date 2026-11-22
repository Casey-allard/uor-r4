// Exercise the imported renderer's real client, decoder and query state against
// the shipped host adapter. Uses the renderer's locked build dependencies.
import assert from "node:assert/strict";
import { build } from "../vendor/prism/node_modules/esbuild/lib/main.js";
import { fileURLToPath } from "node:url";
import { invokeHost, unwrapHost } from "../shell/host-boundary.mjs";
const root = fileURLToPath(new URL("../vendor/prism/", import.meta.url));
const bundle = await build({
  stdin: {
    contents: 'export { createClient, defineContract } from "./src/ipc/contracts/core"; export { z } from "zod"; export { QueryClient } from "@tanstack/react-query";',
    resolveDir: root, loader: "ts",
  },
  bundle: true, write: false, platform: "node", format: "esm", logLevel: "silent",
});
const { createClient, defineContract, z, QueryClient } = await import("data:text/javascript;base64," + Buffer.from(bundle.outputFiles[0].contents).toString("base64"));
const output = z.array(z.string());
const contract = defineContract({ channel: "boundary:test", input: z.void(), output });
for (const envelopeMode of [true, false]) {
  let handler;
  const invoke = (channel, ...args) => invokeHost(channel, args, handler, output);
  globalThis.window = { electron: { ipcRenderer: envelopeMode ? { invokeEnvelope: invoke } : { invoke: (...args) => invoke(...args).then(unwrapHost) } } };
  const client = createClient({ read: contract });
  const query = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const options = { queryKey: ["boundary"], queryFn: () => client.read() };
  await assert.rejects(query.fetchQuery(options), { code: "unsupportedChannel", kind: "not_found" });
  assert.equal(query.getQueryState(options.queryKey).status, "error");
  assert.equal(query.getQueryData(options.queryKey), undefined);
  handler = () => ({ items: [] });
  await assert.rejects(query.fetchQuery(options), { code: "invalidResponse", kind: "internal" });
  handler = () => ["actual result"];
  assert.deepEqual(await query.fetchQuery(options), ["actual result"]);
  assert.equal(query.getQueryState(options.queryKey).status, "success");
  query.clear();
}
delete globalThis.window;
console.log("renderer boundary: both real client entrypoints refuse, retain error state and recover");
