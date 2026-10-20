// Smoke test for the transpiled npm package. Mints κ-labels for one
// input per realization and asserts each is a well-formed 71-byte
// ASCII sha256:<64hex>.
//
// The reference byte-for-byte κ-labels are pinned by the Rust
// cross-realization test suite (tests/all_realizations.rs); this test
// only confirms the npm package wires up to the same wasm component
// and the format-specific entry points are reachable.

import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const distEntry = resolve(__dirname, "..", "dist", "uor-addr.js");

if (!existsSync(distEntry)) {
  console.error("error: dist/ not built. run `npm run build` first.");
  process.exit(1);
}

const mod = await import(distEntry);
const kappa = mod.kappa ?? mod["uor:addr/kappa"] ?? mod;

const cases = [
  ["jsonAddress",     new TextEncoder().encode('{"foo":"bar"}')],
  ["sexpAddress",     new TextEncoder().encode("(a b c)")],
  ["xmlAddress",      new TextEncoder().encode("<root/>")],
  ["ringAddress",     new Uint8Array([0, 0x42])],
];

const witnessCases = [
  ["jsonAddressWithWitness",     new TextEncoder().encode('{"foo":"bar"}')],
  ["sexpAddressWithWitness",     new TextEncoder().encode("(a b c)")],
  ["xmlAddressWithWitness",      new TextEncoder().encode("<root/>")],
  ["ringAddressWithWitness",     new Uint8Array([0, 0x42])],
];

const KAPPA_LABEL_RE = /^sha256:[0-9a-f]{64}$/;
let failed = 0;

for (const [fnName, input] of cases) {
  const fn = kappa[fnName];
  if (typeof fn !== "function") {
    console.error(`fail: ${fnName} is not a function`);
    failed += 1;
    continue;
  }
  let result;
  try {
    result = fn(input);
  } catch (e) {
    console.error(`fail: ${fnName} threw: ${e.message}`);
    failed += 1;
    continue;
  }
  if (typeof result !== "string" || result.length !== 71 || !KAPPA_LABEL_RE.test(result)) {
    console.error(`fail: ${fnName} returned ${JSON.stringify(result)} (not a 71-byte sha256:<64hex>)`);
    failed += 1;
    continue;
  }
  console.log(`ok:   ${fnName} → ${result}`);
}

// TC-05 cross-language round-trip: each *AddressWithWitness call
// returns a Grounded whose verify() returns the same κ-label
// byte-for-byte (without re-invoking SHA-256).
for (const [fnName, input] of witnessCases) {
  const fn = kappa[fnName];
  if (typeof fn !== "function") {
    console.error(`fail: ${fnName} is not a function`);
    failed += 1;
    continue;
  }
  let grounded;
  try {
    grounded = fn(input);
  } catch (e) {
    console.error(`fail: ${fnName} threw: ${e.message}`);
    failed += 1;
    continue;
  }
  let mintLabel, verifyLabel, fingerprint;
  try {
    mintLabel = grounded.kappaLabel();
    verifyLabel = grounded.verify();
    fingerprint = grounded.contentFingerprint();
  } catch (e) {
    console.error(`fail: ${fnName} resource accessor threw: ${e.message}`);
    failed += 1;
    continue;
  }
  if (!KAPPA_LABEL_RE.test(mintLabel)) {
    console.error(`fail: ${fnName} kappaLabel returned ${JSON.stringify(mintLabel)}`);
    failed += 1;
    continue;
  }
  if (mintLabel !== verifyLabel) {
    console.error(`fail: ${fnName}: TC-05 round-trip mismatch — mint=${mintLabel}, verify=${verifyLabel}`);
    failed += 1;
    continue;
  }
  if (!(fingerprint instanceof Uint8Array) || fingerprint.length !== 32) {
    console.error(`fail: ${fnName}: contentFingerprint expected 32-byte Uint8Array, got ${fingerprint?.length}`);
    failed += 1;
    continue;
  }
  console.log(`ok:   ${fnName} → mint==verify (TC-05) → ${mintLabel}`);
}

// ADR-061 κ-label composition: compose two operand κ-labels through
// each of the five categorical operations, asserting the named laws.
let composeChecks = 0;
{
  const a = kappa.jsonAddress(new TextEncoder().encode('{"role":"left"}'));
  const b = kappa.jsonAddress(new TextEncoder().encode('{"role":"right"}'));

  // CS-G2 commutativity.
  try {
    const ab = kappa.composeG2(a, b, "sha256");
    const ba = kappa.composeG2(b, a, "sha256");
    if (!KAPPA_LABEL_RE.test(ab)) throw new Error(`g2 not well-formed: ${ab}`);
    if (ab !== ba) throw new Error(`g2 not commutative: ${ab} != ${ba}`);
    console.log(`ok:   composeG2 (commutative) → ${ab}`);
    composeChecks += 1;
  } catch (e) {
    console.error(`fail: composeG2: ${e.message}`);
    failed += 1;
  }

  // The four unary ops, each well-formed + witness round-trip.
  const unary = [
    ["composeF4", "composeF4WithWitness"],
    ["composeE6", "composeE6WithWitness"],
    ["composeE7", "composeE7WithWitness"],
    ["composeE8", "composeE8WithWitness"],
  ];
  for (const [labelFn, witnessFn] of unary) {
    try {
      const label = kappa[labelFn](a, "sha256");
      if (!KAPPA_LABEL_RE.test(label)) throw new Error(`not well-formed: ${label}`);
      const g = kappa[witnessFn](a, "sha256");
      if (g.kappaLabel() !== label) throw new Error("witness label mismatch");
      if (g.verify() !== label) throw new Error("TC-05 round-trip mismatch");
      console.log(`ok:   ${labelFn} → mint==verify (TC-05) → ${label}`);
      composeChecks += 1;
    } catch (e) {
      console.error(`fail: ${labelFn}: ${e.message}`);
      failed += 1;
    }
  }
}

if (failed > 0) {
  console.error(`\n${failed} failure(s)`);
  process.exit(1);
}

const total = cases.length + witnessCases.length + composeChecks;
console.log(`\nall ${total} smoke tests passed`);
