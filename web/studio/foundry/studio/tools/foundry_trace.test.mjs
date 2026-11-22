// PM-02: the front end drives the back end. Each test plants one defect in a copy of
// app/product.json and expects the projector to refuse it, so the trace checks in foundry_model.mjs
// are each shown to bite, and the model as committed is shown to pass all of them.
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { deriveFoundry, ModelError } from "./foundry_model.mjs";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const product = () => JSON.parse(fs.readFileSync(path.join(root, "app/product.json"), "utf8"));
const refused = (mutate, pattern) => {
  const p = product();
  mutate(p);
  assert.throws(() => deriveFoundry({ product: p }), (e) => e instanceof ModelError && pattern.test(e.message));
};
const ids = () => [...fs.readFileSync(path.join(root, "model/ids.toml"), "utf8").matchAll(/^id = "([^"]+)"$/gm)].map((m) => m[1]);

test("the committed model traces every screen, ID, registry feature and component", () => {
  const m = deriveFoundry();
  const p = product();
  const backend = m.features.features.slice(0, p.features.length);
  const carried = backend.flatMap((f) => f.statement.match(/\(([^)]*)\)/)[1].split(", ").filter(Boolean));
  assert.deepEqual([...carried].sort(), ids().sort(), "every register ID is carried by exactly one back-end feature");
  for (const f of backend) assert.ok(f.acceptance.length > 0 && f.calls.length > 0, `${f.name} calls and is accepted`);
  assert.equal(m.users.goals.length, p.journeys.length);
  assert.equal(m.users.groups.length, p.stakeholders.length);
  assert.ok(m.design.views.some((v) => v.diagram === "opmObjectProcess"));
  assert.equal(m.design.views.filter((v) => v.diagram === "umlSequence").length, p.journeys.length);
  assert.equal(m.quality.characteristics.length, 9);
  assert.equal(m.quality.subcharacteristics.length, 40);
});

test("a screen that calls a registry feature it does not require is refused", () => {
  refused((p) => {
    const home = p.surfaces.find((s) => s.name === "Home");
    home.requires = home.requires.filter((f) => f !== "Libraries and settings");
  }, /Home reaches (theme|prompt)-library/);
});

test("a screen that starts a host channel no required feature owns is refused", () => {
  refused((p) => delete p.features.find((f) => f.name === "Chat").channels, /host channel chat:stream, which no feature owns/);
});

test("an ID carried by two features, or by none, is refused", () => {
  refused((p) => p.features[1].ids.push(p.features[0].ids[0]), /is carried by .* and .*; one feature carries an ID/);
  refused((p) => p.features[0].ids.pop(), /is carried by no feature/);
});

test("a back-end feature no screen requires is refused", () => {
  refused((p) => {
    for (const s of p.surfaces) s.requires = s.requires.filter((f) => f !== "Stateful apps");
  }, /Stateful apps is required by no screen/);
});

test("a screen in no journey, and a stakeholder with none, are refused", () => {
  refused((p) => (p.journeys = p.journeys.filter((j) => j.name !== "Shape the studio")), /screen (Settings|Library) is in no journey/);
  refused((p) => (p.journeys = p.journeys.filter((j) => j.stakeholder !== "Verifier")), /Verifier has no journey/);
});

test("a threat on a category STRIDE per element does not open is refused", () => {
  refused((p) => (p.threats.find((t) => t.name === "Altered project record").stride = "spoofing"), /which STRIDE per element does not open/);
});
