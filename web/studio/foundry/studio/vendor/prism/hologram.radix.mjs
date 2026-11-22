// hologram-forge (ours, not upstream's; see VENDORED.md): Upstream's screens call shadcn on Base UI; the renderer's
// ui components are the brand kit's shadcn/ui new-york-v4, which is Radix (hologram.ui.mjs). This rewrites
// each call to a ui component from Base UI's props to Radix's, at build time, so upstream's files stay verbatim
// and an upstream update needs no hand edits. Only elements whose tag is imported from @/components/ui/*
// are touched. The typecheck (hologram.typecheck.mjs) checks the rewritten sources, so a pattern this does
// not know shows up as a type error, never as a silent difference.
//
//   render={<El/>}                     -> asChild, with El as the only child (the element's children move into El)
//   as="span"                          -> asChild, with <span> as the only child
//   <Accordion multiple>               -> type="multiple"; without it, type="single" collapsible (or multiple
//                                         when it is given an array value)
//   <ToggleGroup>                      -> type="multiple" (Base UI's values are arrays)
//   <TooltipProvider delay>            -> delayDuration
//   <DropdownMenuItem closeOnClick={false}> -> onSelect={(e) => e.preventDefault()}
//   <TabsContent keepMounted>          -> forceMount
//   <SelectValue>{(v) => label}</SelectValue> -> {((v) => label)(value)}, value being its Select's
//                                         (Radix's SelectValue takes the label, not a function)
//   MiniSelectTrigger                  -> the kit's SelectTrigger, size="sm"
//   import from "./ui/button"           -> from "@/components/ui/button"
//   <CommandDialog filter={f}>         -> the filter on a cmdk Command inside
//   <Select onValueChange={f}>         -> onValueChange={(value: string) => (f)(value as never)}
//   render={button}                    -> asChild, with {button} as the only child
import path from "node:path";
import ts from "typescript";

const RENAMED_IMPORTS = { MiniSelectTrigger: "SelectTrigger" };
// Radix's Select hands its handler a string; Base UI types the value by the `value` prop. The strings are
// the same, so the handler is called unchanged, its own parameter type kept (an immediately called
// function's parameters take the argument's type).
const SELECT_HANDLER = "(value: string) => ";

function uiNames(sf) {
  const names = new Set();
  for (const st of sf.statements) {
    if (!ts.isImportDeclaration(st) || !ts.isStringLiteral(st.moduleSpecifier)) continue;
    const from = st.moduleSpecifier.text;
    if (!from.startsWith("@/components/ui/")) continue;
    const b = st.importClause?.namedBindings;
    if (b && ts.isNamedImports(b)) for (const el of b.elements) names.add(el.name.text);
  }
  return names;
}

const unparen = (e) => { while (e && ts.isParenthesizedExpression(e)) e = e.expression; return e; };
const isJsx = (e) => e && (ts.isJsxElement(e) || ts.isJsxSelfClosingElement(e));
const tagText = (n) => (ts.isJsxElement(n) ? n.openingElement : n).tagName.getText();
const attrsOf = (n) => (ts.isJsxElement(n) ? n.openingElement : n).attributes.properties;
const attr = (n, name) => attrsOf(n).find((a) => ts.isJsxAttribute(a) && a.name.getText() === name);
const meaningful = (children) => children.some((c) => !(ts.isJsxText(c) && c.containsOnlyTriviaWhiteSpaces));

// One pass: collect non-overlapping edits, outermost first. The caller re-parses and repeats until stable,
// so an element nested inside a moved one is rewritten on the next pass.
function pass(code, file) {
  const sf = ts.createSourceFile(file, code, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  const names = uiNames(sf);
  if (!names.size) return { code, n: 0, problems: [] };
  const edits = [];
  const problems = [];
  const taken = [];
  const free = (s, e) => !taken.some(([a, b]) => s < b && a < e);
  const add = (list) => {
    if (!list.every(([s, e]) => free(s, e))) return false;
    for (const ed of list) { edits.push(ed); taken.push([ed[0], ed[1]]); }
    return true;
  };
  let needsCommand = false;
  const at = (node) => `${file}:${sf.getLineAndCharacterOfPosition(node.getStart()).line + 1}`;

  // Imports renamed to the kit's names, kept under the old local name.
  for (const st of sf.statements) {
    if (!ts.isImportDeclaration(st) || !st.moduleSpecifier.getText().includes("@/components/ui/")) continue;
    const b = st.importClause?.namedBindings;
    if (!b || !ts.isNamedImports(b)) continue;
    for (const el of b.elements) {
      const kit = RENAMED_IMPORTS[el.name.text];
      if (kit && !el.propertyName) add([[el.getStart(), el.getEnd(), `${kit} as ${el.name.text}`]]);
    }
  }

  const visit = (node) => {
    if (isJsx(node) && names.has(tagText(node))) {
      const tag = tagText(node);
      const done = rewrite(node, tag);
      if (done === true) return; // children are rewritten on the next pass
    }
    ts.forEachChild(node, visit);
  };

  function moveInto(node, childText, dropAttr) {
    const [s, e] = [dropAttr.getStart(), dropAttr.getEnd()];
    if (ts.isJsxElement(node)) {
      const kids = node.children;
      const inner = meaningful(kids) ? code.slice(node.openingElement.getEnd(), node.closingElement.getStart()) : null;
      let child = childText;
      const el = unparenChild(childText);
      if (inner !== null) {
        if (el.selfClosing) child = el.open + inner + `</${el.tag}>`;
        else { problems.push(`${at(node)}: <${tagText(node)}> has children and its render element has its own`); return false; }
      }
      return add([[s, e, "asChild"], [node.openingElement.getEnd(), node.closingElement.getStart(), child]]);
    }
    const whole = code.slice(node.getStart(), node.getEnd());
    const open = (code.slice(node.getStart(), s) + "asChild" + code.slice(e, node.getEnd())).replace(/\s*\/>$/, ">");
    void whole;
    return add([[node.getStart(), node.getEnd(), open + childText + `</${tagText(node)}>`]]);
  }

  function unparenChild(text) {
    const m = text.match(/^<([A-Za-z_][\w.]*)/);
    const tag = m ? m[1] : "";
    const selfClosing = /\/>\s*$/.test(text);
    const open = selfClosing ? text.replace(/\s*\/>\s*$/, ">") : text;
    return { tag, selfClosing, open };
  }

  function rewrite(node, tag) {
    const render = attr(node, "render");
    if (render) {
      const init = render.initializer;
      const expr = init && ts.isJsxExpression(init) ? unparen(init.expression) : null;
      if (expr && ts.isIdentifier(expr)) {
        if (ts.isJsxElement(node) && meaningful(node.children)) { problems.push(`${at(node)}: <${tag} render={${expr.text}}> has children`); return false; }
        return moveInto(node, `{${expr.text}}`, render);
      }
      if (!isJsx(expr)) { problems.push(`${at(node)}: <${tag} render> is not an element`); return false; }
      return moveInto(node, expr.getText(), render);
    }
    const as = attr(node, "as");
    if (as && as.initializer && ts.isStringLiteral(as.initializer)) {
      return moveInto(node, `<${as.initializer.text} />`, as);
    }
    const edits = [];
    const rename = (a, text) => edits.push([a.getStart(), a.getEnd(), text]);
    const append = (text) => { const o = ts.isJsxElement(node) ? node.openingElement : node; edits.push([o.tagName.getEnd(), o.tagName.getEnd(), " " + text]); };
    if (tag === "Accordion" || tag === "ToggleGroup") {
      const multiple = attr(node, "multiple");
      if (multiple) rename(multiple, 'type="multiple"');
      else if (!attr(node, "type")) {
        const arrayValue = tag === "ToggleGroup" || attr(node, "defaultValue") || attr(node, "value");
        append(arrayValue ? 'type="multiple"' : 'type="single" collapsible');
      }
    }
    if (tag === "TooltipProvider") { const d = attr(node, "delay"); if (d) rename(d.name, "delayDuration"); }
    if (tag === "DropdownMenuItem") {
      const c = attr(node, "closeOnClick");
      if (c && c.initializer?.getText() === "{false}") rename(c, "onSelect={(e) => e.preventDefault()}");
    }
    if (tag === "TabsContent") { const k = attr(node, "keepMounted"); if (k) rename(k, "forceMount"); }
    if (tag === "SelectValue" && ts.isJsxElement(node)) {
      const fn = node.children.find((c) => ts.isJsxExpression(c) && c.expression && (ts.isArrowFunction(unparen(c.expression)) || ts.isFunctionExpression(unparen(c.expression))));
      if (fn) {
        let up = node.parent;
        while (up && !(ts.isJsxElement(up) && tagText(up) === "Select")) up = up.parent;
        const v = up && attr(up, "value");
        const ve = v?.initializer && ts.isJsxExpression(v.initializer) ? v.initializer.expression : null;
        if (!ve) problems.push(`${at(node)}: <SelectValue> renders a function and its <Select> has no value`);
        else edits.push([fn.getStart(), fn.getEnd(), `{(${fn.expression.getText()})(${ve.getText()} as never)}`]);
      }
    }
    if (tag in RENAMED_IMPORTS && !attr(node, "size")) append('size="sm"');
    // The kit's CommandDialog wraps its children in its own cmdk Command and forwards nothing to it, so a
    // custom filter goes on a Command of its own inside, laid out as if it were not there.
    if (tag === "CommandDialog" && ts.isJsxElement(node)) {
      const f = attr(node, "filter");
      if (f) {
        rename(f, "");
        edits.push([node.openingElement.getEnd(), node.openingElement.getEnd(), `<HologramCommand ${f.getText()} className="contents">`]);
        edits.push([node.closingElement.getStart(), node.closingElement.getStart(), "</HologramCommand>"]);
        needsCommand = true;
      }
    }
    if (tag === "Select") {
      const h = attr(node, "onValueChange");
      const x = h?.initializer && ts.isJsxExpression(h.initializer) ? h.initializer.expression : null;
      if (x && !x.getText().startsWith(SELECT_HANDLER)) rename(h, `onValueChange={${SELECT_HANDLER}(${x.getText()})(value as never)}`);
    }
    if (edits.length) add(edits);
    return false;
  }

  visit(sf);
  if (needsCommand) edits.push([0, 0, 'import { Command as HologramCommand } from "@/components/ui/command";\n']);
  edits.sort((a, b) => b[0] - a[0]);
  let out = code;
  for (const [s, e, t] of edits) out = out.slice(0, s) + t + out.slice(e);
  return { code: out, n: edits.length, problems };
}

/** Upstream's source rewritten for the kit's Radix components; `problems` lists what it could not rewrite. */
export function toRadix(code, file) {
  // A screen that names a ui component by a relative path (`./ui/button`) names it as @/components/ui/button,
  // so one rule decides where every ui import resolves (hologram.ui.mjs) and which calls are rewritten here.
  const dir = path.posix.dirname(file.replace(/\\/g, "/"));
  code = code.replace(/(from\s+["'])(\.{1,2}\/[^"']+)(["'])/g, (m, a, spec, b) => {
    const target = path.posix.normalize(path.posix.join(dir, spec));
    return target.startsWith("src/components/ui/") ? a + "@/components/ui/" + target.slice("src/components/ui/".length) + b : m;
  });
  if (!code.includes("@/components/ui/")) return { code, problems: [] };
  let out = code;
  const problems = [];
  for (let i = 0; i < 12; i++) {
    const r = pass(out, file);
    problems.push(...r.problems);
    if (!r.n) return { code: out, problems: [...new Set(problems)] };
    out = r.code;
  }
  throw new Error(`${file}: the Radix rewrite did not settle`);
}
