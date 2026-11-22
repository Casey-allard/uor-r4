// prism-prism (ours, not upstream's; see VENDORED.md): the contracts, published to the page so the host
// can validate commands before effects and check its own answers.
//
// Upstream's renderer types every call it makes but validates no answer at runtime, so a handler that
// returns an object where the contract declares an array crashes a screen with "is not iterable" the
// first time a user opens it, far from the cause. The whole contract set is small and already in the
// bundle, so it is published here as {channel: output schema}: `shell/host.js` parses every answer
// against it and returns a typed refusal on disagreement. Input schemas are
// published separately so the host uses the imported contract before any effect.
import * as types from "@/ipc/types";

type Schema = { safeParse: (value: unknown) => { success: boolean; error?: unknown } };
type Contract = { channel?: unknown; input?: unknown; output?: unknown };

const shapes: Record<string, Schema> = {};
const inputs: Record<string, Schema> = {};
for (const group of Object.values(types as Record<string, unknown>)) {
  if (!group || typeof group !== "object") continue;
  for (const contract of Object.values(group as Record<string, Contract>)) {
    if (!contract || typeof contract !== "object") continue;
    const { channel, input, output } = contract;
    if (typeof channel === "string" && input && typeof (input as Schema).safeParse === "function") {
      inputs[channel] = input as Schema;
    }
    if (
      typeof channel === "string" &&
      output &&
      typeof (output as Schema).safeParse === "function"
    ) {
      shapes[channel] = output as Schema;
    }
  }
}

(window as unknown as { __contracts?: Record<string, Schema> }).__contracts = shapes;
(window as unknown as { __inputContracts?: Record<string, Schema> }).__inputContracts = inputs;
