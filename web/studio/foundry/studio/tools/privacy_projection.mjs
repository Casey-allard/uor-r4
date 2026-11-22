import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { privacySource } from "../vendor/prism/hologram.privacy.mjs";
import { screenSource } from "../vendor/prism/hologram.screens.mjs";
const root = fileURLToPath(new URL("../vendor/prism/", import.meta.url));
const renderer = fs.readFileSync(path.join(root, "src/renderer.tsx"), "utf8");
const settings = fs.readFileSync(path.join(root, "src/pages/settings.tsx"), "utf8");
const projected = privacySource("src/renderer.tsx", renderer);
assert.ok(!projected.includes("posthog.init(")); assert.ok(projected.includes("const posthogClient = posthog;")); assert.ok(!projected.includes("phc_5Vxx0XT8Ug3eWROhP6mm4D6D2DgIIKT232q4AKxC2ab"));
const page = privacySource("src/pages/settings.tsx", settings);
assert.ok(page.includes("Browser telemetry is disabled.")); assert.ok(!page.includes("<TelemetrySwitch")); assert.ok(!page.includes("Telemetry ID")); assert.ok(!page.includes('from "@/components/TelemetrySwitch"'));
for (const file of ["src/renderer.tsx", "src/pages/settings.tsx"]) {
  const input = fs.readFileSync(path.join(root, file), "utf8"), actual = screenSource(path.join(root, file), input);
  assert.ok(file.endsWith("renderer.tsx") ? actual.includes("const posthogClient = posthog;") : actual.includes("Browser telemetry is disabled."));
}
const bad = [renderer.replace("posthog.init(", "posthog.initialize("), renderer + '\nposthog.init("other", {});', renderer.replace('const posthogClient = posthog.init(', 'const otherClient = posthog.init('), 'const posthogClient = posthog.init("x");', 'const posthogClient = posthog.init("x", makeOptions());', 'const posthogClient = ('];
for (const text of bad) assert.throws(() => privacySource("src/renderer.tsx", text), { code: "browserTelemetryProjectionMismatch" });
for (const text of [settings.replace("SECTION_IDS.telemetry", "SECTION_IDS.unknown"), settings.replace("<TelemetrySwitch />", "<DifferentSwitch />"), settings + settings]) assert.throws(() => privacySource("src/pages/settings.tsx", text), { code: "browserTelemetryProjectionMismatch" });
const search = fs.readFileSync(path.join(root, "src/lib/settingsSearchIndex.ts"), "utf8");
const indexed = screenSource(path.join(root, "src/lib/settingsSearchIndex.ts"), search);
assert.ok(indexed.includes("Browser vendor telemetry is disabled")); assert.ok(!indexed.includes("Enable or disable anonymous usage data collection"));
assert.throws(() => privacySource("src/lib/settingsSearchIndex.ts", search.replace("id: SETTING_IDS.telemetry,", "id: SETTING_IDS.other,")), { code: "browserTelemetryProjectionMismatch" });
assert.equal(privacySource("src/unrelated.tsx", "unchanged"), "unchanged");
console.log("Privacy projection: actual renderer and settings, ten drift refusals and build/typecheck integration pass");
