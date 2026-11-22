"""Authoring the hologram-forge model: five LexLean modules from three declaration sources.

The authority is the set of .lex.tex files this script writes; LexLean and PrismPM read only those.
The declarations live as data in tools/model_workspace.py (the builder: records, snapshot preimage,
restore, capability, versions, preview) and tools/model_inference.py (freeinference: transcript,
preimages, memo, the route table, the OpenAI wire, the OpenRouter request, the κ object addressing
rule, pool and stage tables, pack, ladder and loader) and tools/model_publish.py (the published
application: its model document, source manifest, decision and path). This script partitions them
into modules, qualifies every reference that crosses a module boundary, merges the two View records
into one, and writes src/{Workspace,Inference,Object,Publish,Forge}.lex.tex, plus
model/roots.txt (Module.name, strictly sorted, as the exporter demands).

Run from the project root: python3 tools/author.py
Check without rewriting: python3 tools/author.py --check

The authoring/projection obligation is FoundrySpecification.authority and
FoundrySpecification.authoringBoundary in app/src/FoundrySpecification.lex.tex.

Rules met on the way (all hit): glossary imports precede module imports in a header; a cross module
reference is the member's "module" field; declaration names are unique across modules because the
generated Rust is one flat crate; the escape chain is freeinference's five steps (tab included).
"""
import argparse, importlib.util, json, pathlib, sys

sys.dont_write_bytecode = True
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--check", action="store_true", help="compare every projection without writing")
args = parser.parse_args()

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent


def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


inf = load("model_inference")
ws = load("model_workspace")
pub = load("model_publish")

# ---- what each source contributes
ESC = {"escapeBackslash", "escapeQuote", "escapeNewline", "escapeReturn", "escapeJson"}
DROP_INF = {"Decision", "decide", "decide_serve", "decide_execute", "decide_refuse", "View", "view", "view_headline"}
DROP_WS = {"View", "view", "view_headline"} | ESC
OBJECT_TYPES = {"Range", "Obj", "Shard", "Manifest", "Staging", "PageAction", "Admission", "Source", "Priority", "Section", "Tier", "Start"}
OBJECT_DEFS = {"stride", "expertPage", "pageBytes", "pageStart", "tablePage", "objKind", "objLabel", "objKappa", "objEntry",
               "shardEntry", "shardEntries", "shardEntriesFrom", "shardKappa", "shardLabel", "shardObjects", "shardSha256", "sha256Text",
               "manifestRepo", "manifestRevision", "manifestSpec", "rootPreimage", "admitPage",
               "pageAction", "poolAdmit", "fetchSource", "prefetchOrder", "packRank", "packed", "firstTokenReady", "promote", "loaderStart"}
TOP_TYPES = {"Wallpaper"}

inf_decls = [d for d in inf.decls if d["name"] not in DROP_INF]
ws_decls = [d for d in ws.decls if d["name"] not in DROP_WS]


def by_name(decls, name):
    for d in decls:
        if d["name"] == name:
            return d
    raise KeyError(name)


# ---- one View: the builder's words for the hero, freeinference's words for everything else
ws_view, inf_view = by_name(ws.decls, "View"), by_name(inf.decls, "View")
ws_words, inf_words = by_name(ws.decls, "view")["body"], by_name(inf.decls, "view")["body"]
fields = list(ws_view["fields"])
values = list(ws_words["fields"])
have = {f["name"] for f in fields}
for f in inf_view["fields"]:
    if f["name"] not in have:
        fields.append(f)
        have.add(f["name"])
inf_values = {a["field"]: a["value"] for a in inf_words["fields"]}
for f in fields[len(ws_view["fields"]):]:
    values.append({"field": f["name"], "value": inf_values[f["name"]]})
view_structure = {"fields": fields, "kind": "structure", "name": "View", "parameters": [], "type_parameters": []}
view_definition = dict(by_name(ws.decls, "view"))
view_definition["body"] = {"fields": values, "kind": "record", "type": {"name": "View"}}
view_theorem = by_name(ws.decls, "view_headline")

# ---- modules
modules = {"Workspace": [], "Inference": [], "Object": [], "Publish": [], "Forge": []}
name2mod = {}


def place(module, decl):
    modules[module].append(decl)
    name2mod[decl["name"]] = module


# A module admits no forward reference, so the escape chain comes first in Workspace.
for d in inf_decls:
    if d["name"] in ESC:
        place("Workspace", d)
for d in ws_decls:
    place("Workspace", d)
for d in inf_decls:
    if d["kind"] == "theorem" or d["name"] in ESC:
        continue
    if False:
        pass
    elif d["name"] in OBJECT_TYPES or d["name"] in OBJECT_DEFS:
        place("Object", d)
    elif d["name"] in TOP_TYPES:
        place("Forge", d)
    else:
        place("Inference", d)
for d in pub.decls:
    place("Publish", d)
for d in (view_structure, view_definition, view_theorem):
    place("Forge", d)


def mentioned(term, out):
    if isinstance(term, dict):
        for key in ("function", "constructor", "type", "member"):
            m = term.get(key)
            if isinstance(m, dict) and "name" in m:
                out.add(m["name"].split(".")[0])
        for v in term.values():
            mentioned(v, out)
    elif isinstance(term, list):
        for v in term:
            mentioned(v, out)
    return out


ORDER = ["Forge", "Publish", "Object", "Inference", "Workspace"]
for d in inf_decls:
    if d["kind"] != "theorem":
        continue
    names = mentioned(d["statement"], set())
    mods = {name2mod[n] for n in names if n in name2mod}
    home = next((m for m in ORDER if m in mods), "Inference")
    place(home, d)


# ---- qualify every reference that crosses a module boundary, and collect imports
def qualify(term, current, imports):
    if isinstance(term, dict):
        for key in ("function", "constructor", "type", "member"):
            m = term.get(key)
            if isinstance(m, dict) and "name" in m:
                base = m["name"].split(".")[0]
                home = name2mod.get(base)
                if home and home != current:
                    m["module"] = home
                    imports.add(home)
        for v in term.values():
            qualify(v, current, imports)
    elif isinstance(term, list):
        for v in term:
            qualify(v, current, imports)


BS = chr(92)
NL = "\n"


def document(name, imports, decls):
    head = BS + "begin{lexlean}{" + name + "}" + NL
    head += BS + "useglossary{lexlean.std.bool@1.1.0}" + NL + BS + "useglossary{lexlean.std.nat@1.1.0}" + NL
    for i in imports:
        head += BS + "importmodule{" + i + "}" + NL
    head += BS + "title{Boolean}" + NL + BS + "begin{semanticmodule}" + NL + BS + "semanticdata{"
    body = json.dumps({"declarations": decls, "spec": "lexlean/semantic-module/1"}, separators=(",", ":"), sort_keys=True, ensure_ascii=False)
    return head + body + "}" + NL + BS + "end{semanticmodule}" + NL + BS + "end{lexlean}" + NL


outputs = {}
roots = []
DEPENDENCY_ORDER = ["Workspace", "Inference", "Object", "Publish", "Forge"]
for name in DEPENDENCY_ORDER:
    decls = modules[name]
    imports = set()
    qualify(decls, name, imports)
    imports = [m for m in DEPENDENCY_ORDER if m in imports]
    text = document(name, imports, decls)
    outputs["src/" + name + ".lex.tex"] = text.encode("utf-8")
    roots += [name + "." + d["name"] for d in decls if d["kind"] == "definition"]

roots.sort()
outputs["model/roots.txt"] = ("\n".join(roots) + "\n").encode("utf-8")

failures = []
for relative, expected in outputs.items():
    path = ROOT / relative
    if args.check:
        try:
            actual = path.read_bytes()
        except FileNotFoundError:
            failures.append(("missingProjection", relative))
            continue
        except OSError:
            failures.append(("unreadableProjection", relative))
            continue
        if actual != expected:
            failures.append(("divergedProjection", relative))
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(expected)
        print(f"wrote {relative} ({len(expected)} bytes)")

for diagnostic, relative in failures:
    print(f"author:{diagnostic}: {relative}", file=sys.stderr)
if failures:
    raise SystemExit(1)
if args.check:
    print(f"author: current ({len(outputs)} projections)")
