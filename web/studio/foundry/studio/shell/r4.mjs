// Prism R4 in the studio: whether one model answer may be shown.
//
// The decision is the Prism R4 spec (r4/spec.json, afflom/me app/specs/r4/spec.json at 28dca52),
// evaluated with prism-code's own judge, first match decides. Foundry's kernel holds the same spec as
// a LexLean module whose guarantees Lean proves (app/src/PrismR4.lex.tex); this file holds no rule of
// its own, only the facts a turn reports.
//
// Foundry answers with its own language and reasoning only: its reader, compiled with Prism, and
// nothing routed. Its grant is empty, so
// a route to any other model is ungranted and R4 refuses it before a byte of the request leaves the tab.
import { judge } from "./prism/spec.mjs";

let specOnce = null;
export function r4Spec() {
  specOnce ??= fetch(new URL("./r4/spec.json", import.meta.url)).then((r) => {
    if (!r.ok) throw Object.assign(new Error("The Prism R4 decision could not be read"), { code: "r4Unavailable" });
    return r.json();
  });
  return specOnce;
}

/** Foundry grants no route: every route to another model is ungranted. */
export const GRANT = Object.freeze([]);

/**
 * The R4 request a turn makes. `provider` is the chat's route ("local" for Foundry's own reader). The
 * reader is compiled with Prism into the kernel guest (IdeaReader in app/src): it carries no weights and
 * routes nothing, and its engine matches because the guest answers only after its pin and every
 * acceptance vector passed (kernel.js).
 */
export function r4Request({ provider, engine = "match", weights = "match", boundary = "allow" }) {
  if (provider === "uor-r4") {
    return { mode: "inprism", engine, weights, route: "none", boundary };
  }
  if (provider !== "local") {
    return {
      mode: "routed",
      engine: "missing",
      weights: "none",
      route: GRANT.includes(provider) ? "granted" : "ungranted",
      boundary,
    };
  }
  return { mode: "compiled", engine: "match", weights: "none", route: "none", boundary };
}

/** R4's outcome for one request: answer, receipt, ask or refuse, with the rule that decided it. */
export async function r4Decide(request) {
  const spec = await r4Spec();
  return judge(spec, request);
}

/** The words a refused turn shows, by the rule that refused it. */
export function r4Refusal(decision, request) {
  if (request?.mode === "routed" || decision.rule === "REQ-007") {
    return "UOR-R4 Foundry answers only in-browser with verified UOR-R4 WASM or Prism IdeaReader. External model routing is not granted.";
  }
  if (request?.mode === "inprism" && request?.weights !== "match") {
    return "UOR-R4 weight artifact SHA-256 verification failed or no artifact is loaded.";
  }
  if (decision.rule === "REQ-001") {
    return "The browser engine fingerprint did not match its lock, so the answer was refused.";
  }
  return "Prism R4 did not allow this answer to be shown.";
}
