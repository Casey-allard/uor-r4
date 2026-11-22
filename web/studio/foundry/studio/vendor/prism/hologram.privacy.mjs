// FoundrySpecification.browserTelemetryBoundary. Imported source is never edited in place.
import ts from "typescript";
const refuse = () => { throw Object.assign(new Error("Browser telemetry projection does not match the imported renderer"), { code: "browserTelemetryProjectionMismatch" }); };
const source = (file, text) => {
  const parsed = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  if (parsed.parseDiagnostics.length) refuse(); return parsed;
};
const visit = (node, predicate) => { const found = []; const walk = n => { if (predicate(n)) found.push(n); ts.forEachChild(n, walk); }; walk(node); return found; };
export function privacySource(file, text) {
  if (file === "src/renderer.tsx") {
    const parsed = source(file, text);
    const calls = visit(parsed, n => ts.isCallExpression(n) && n.expression.getText(parsed) === "posthog.init");
    const declarations = visit(parsed, n => ts.isVariableDeclaration(n) && ts.isIdentifier(n.name) && n.name.text === "posthogClient");
    if (calls.length !== 1 || declarations.length !== 1 || declarations[0].initializer !== calls[0] || calls[0].arguments.length !== 2 || !ts.isStringLiteral(calls[0].arguments[0]) || !ts.isObjectLiteralExpression(calls[0].arguments[1])) refuse();
    const call = calls[0]; return text.slice(0, call.getStart(parsed)) + "posthog" + text.slice(call.getEnd());
  }
  if (file === "src/pages/settings.tsx") {
    const parsed = source(file, text);
    const sections = visit(parsed, n => ts.isJsxElement(n) && n.openingElement.tagName.getText(parsed) === "SettingsSection" && n.openingElement.attributes.properties.some(a => ts.isJsxAttribute(a) && a.name.getText(parsed) === "id" && a.initializer && ts.isJsxExpression(a.initializer) && a.initializer.expression?.getText(parsed) === "SECTION_IDS.telemetry"));
    const imports = visit(parsed, n => ts.isImportDeclaration(n) && ts.isStringLiteral(n.moduleSpecifier) && n.moduleSpecifier.text === "@/components/TelemetrySwitch");
    if (sections.length !== 1 || imports.length !== 1 || !sections[0].getText(parsed).includes("<TelemetrySwitch />")) refuse();
    const section = sections[0], replacement = `<SettingsSection id={SECTION_IDS.telemetry} title="Privacy" description="Browser telemetry is disabled.">
            <p id={SETTING_IDS.telemetry} className={hint}>This browser app does not send usage events, session recordings, or diagnostic reports to the renderer vendor. Requests you make to AI providers and publishing services are separate.</p>
          </SettingsSection>`;
    const edits = [[section.getStart(parsed), section.getEnd(), replacement], [imports[0].getStart(parsed), imports[0].getEnd(), ""]].sort((a, b) => b[0] - a[0]);
    let out = text; for (const [start, end, value] of edits) out = out.slice(0, start) + value + out.slice(end); return out;
  }
  if (file === "src/lib/settingsSearchIndex.ts") {
    const parsed = source(file, text);
    const rows = visit(parsed, n => ts.isObjectLiteralExpression(n) && n.properties.some(p => ts.isPropertyAssignment(p) && p.name.getText(parsed) === "id" && p.initializer.getText(parsed) === "SETTING_IDS.telemetry"));
    if (rows.length !== 1) refuse();
    const values = { label: "Privacy", description: "Browser vendor telemetry is disabled", sectionLabel: "Privacy" };
    const edits = [];
    for (const [name, value] of Object.entries(values)) {
      const properties = rows[0].properties.filter(p => ts.isPropertyAssignment(p) && p.name.getText(parsed) === name);
      if (properties.length !== 1 || !ts.isStringLiteral(properties[0].initializer)) refuse();
      const node = properties[0].initializer; edits.push([node.getStart(parsed), node.getEnd(), JSON.stringify(value)]);
    }
    let out = text; for (const [start, end, value] of edits.sort((a, b) => b[0] - a[0])) out = out.slice(0, start) + value + out.slice(end); return out;
  }
  return text;
}
