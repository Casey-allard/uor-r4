"""Declarations of the hologram-forge model, as data. Imported by tools/author.py, which partitions them into
LexLean modules, qualifies cross module references and writes src/*.lex.tex. Not run on its own."""
import json, pathlib

NL = "\n"
BS = "\\"

# Exact axiom sets Lean observes per declaration, pinned by tools/pin_axioms.py (exact policy).
_PINS = pathlib.Path(__file__).resolve().parent / "axioms.json"
AXIOMS = json.loads(_PINS.read_text(encoding="utf-8")) if _PINS.exists() else {}

# ---- types
STRING = {"kind": "string"}
BYTES = {"kind": "bytes"}
BOOL = {"kind": "bool"}
U64 = {"kind": "uint64"}
def named(n): return {"arguments": [], "kind": "named", "member": {"name": n}}
def lst(t): return {"element": t, "kind": "list"}
def opt(t): return {"kind": "option", "value": t}

# ---- expressions
def var(n): return {"kind": "var", "name": n}
def s(v): return {"kind": "string", "value": v}
def u64(v): return {"kind": "integer", "representation": "uint64", "value": str(v)}
def u32(v): return {"kind": "integer", "representation": "uint32", "value": str(v)}
def b(v): return {"kind": "bool", "value": v}
def call(f, *args): return {"arguments": list(args), "function": {"name": f}, "kind": "call"}
def prim(op, result, *args): return {"arguments": list(args), "kind": "primitive", "operation": op, "result": result}
def ctor(name, *args, targs=()):
    return {"arguments": list(args), "constructor": {"name": name}, "kind": "constructor", "type_arguments": list(targs)}
def project(field, value): return {"field": field, "kind": "project", "value": value}
def if_(cond, then, else_): return {"condition": cond, "kind": "if", "then_value": then, "else_value": else_}
def match(scrutinee, *branches): return {"branches": list(branches), "kind": "match", "scrutinee": scrutinee}
def branch(ctor_name, binders, body): return {"binders": binders, "body": body, "constructor": {"name": ctor_name}}
def record(type_name, **fields): return {"fields": [{"field": k, "value": v} for k, v in fields.items()], "kind": "record", "type": {"name": type_name}}
def cons(head, tail): return {"head": head, "kind": "cons", "tail": tail}
def nil(t): return {"element": t, "kind": "nil"}
def strings(*items):
    out = nil(STRING)
    for item in reversed(items): out = cons(item, out)
    return out
def join(list_expr, sep=""): return prim("join", STRING, list_expr, s(sep))
def equal(a, c): return prim("equal", BOOL, a, c)
def eq(l, r): return {"kind": "eq", "left": l, "right": r}
def owned(expr):
    return match(prim("split_exact", opt(lst(STRING)), expr, s(NL), u32(2147483647)),
                 branch("Option.none", [], s("")),
                 branch("Option.some", ["fields"], join(var("fields"), NL)))
def esc_step(name, needle, replacement, inner):
    return definition(name, [("value", STRING)], STRING,
        match(prim("split_exact", opt(lst(STRING)), inner, s(needle), u32(2147483647)),
              branch("Option.none", [], s("")),
              branch("Option.some", ["parts"], join(var("parts"), replacement))))
def q(expr): return join(strings(s('"'), call("escapeJson", expr), s('"')))

# ---- declarations
def inductive(name, *ctors): return {"constructors": [{"fields": [], "name": c} for c in ctors], "kind": "inductive", "name": name, "parameters": [], "type_parameters": []}
def structure(name, **fields): return {"fields": [{"name": k, "type": v} for k, v in fields.items()], "kind": "structure", "name": name, "parameters": [], "type_parameters": []}
def definition(name, params, result, body, recursive=None):
    node = {"axioms": AXIOMS.get(name, []), "body": body, "kind": "definition", "name": name, "parameters": [{"name": k, "type": v} for k, v in params], "result": result}
    if recursive: node["recursive_argument"] = recursive
    return node
def theorem(name, statement, proof="reflexivity"): return {"axioms": AXIOMS.get(name, []), "kind": "theorem", "name": name, "parameters": [], "proof": {"kind": proof}, "statement": statement}

# The snapshot preimage, written once so the definition and the corpus share the term: entries in the
# order the host sorted them (by path, bytewise), each ["path","kappa",bytes], then the name, then the
# parent address ("" for the first snapshot). Keys sorted, no spaces, no trailing newline.
def preimage_body(project_, parent):
    return join(strings(s('{"entries":['), call("entryLines", project("entries", project_)), s('],"name":'), q(call("projectName", project_)),
                        s(',"parent":'), q(parent), s("}")))

decls = [
    # ---- records the host keeps
    structure("Entry", path=STRING, kappa=STRING, bytes=U64),
    structure("Project", label=STRING, entries=lst(named("Entry"))),
    inductive("Decision", "Accept", "Refuse"),
    # A network grant: one origin or one https prefix, as hologram-live's capabilities.json spells it.
    structure("Grant", endpoint=STRING),
    structure("Capabilities", endpoints=lst(named("Grant"))),
    # Every visible word of the shell; the projector renders the page from this record, nothing is
    # written in HTML by hand. No hyphens in any string.
    structure("View", headline=STRING, lede=STRING, promptPlaceholder=STRING, sendLabel=STRING,
              buildingLabel=STRING, previewLabel=STRING, snapshotLabel=STRING, rollbackLabel=STRING,
              refusedLabel=STRING, offlineLabel=STRING,
              publishLabel=STRING, publishingLabel=STRING, publishedLabel=STRING, publishRefusedLabel=STRING,
              holoLabel=STRING, downloadLabel=STRING, openLabel=STRING, runsLabel=STRING,
              holoTitle=STRING, holoLede=STRING, pickLabel=STRING, verifyingLabel=STRING),

    # ---- JSON escaping, split and join, backslash first
    esc_step("escapeBackslash", BS, BS + BS, var("value")),
    esc_step("escapeQuote", '"', BS + '"', call("escapeBackslash", var("value"))),
    esc_step("escapeNewline", NL, BS + "n", call("escapeQuote", var("value"))),
    esc_step("escapeReturn", "\r", BS + "r", call("escapeNewline", var("value"))),
    definition("escapeJson", [("value", STRING)], STRING, call("escapeReturn", var("value"))),

    # ---- the address preimage of a project state
    definition("entryPath", [("entry", named("Entry"))], STRING, owned(project("path", var("entry")))),
    definition("entryKappa", [("entry", named("Entry"))], STRING, owned(project("kappa", var("entry")))),
    definition("projectName", [("project", named("Project"))], STRING, owned(project("label", var("project")))),
    definition("entryLine", [("entry", named("Entry"))], STRING,
        join(strings(s("["), q(call("entryPath", var("entry"))), s(","), q(call("entryKappa", var("entry"))), s(","),
                     prim("format_decimal", STRING, project("bytes", var("entry"))), s("]")))),
    definition("entryLines", [("entries", lst(named("Entry")))], STRING,
        match(var("entries"),
            branch("List.nil", [], s("")),
            branch("List.cons", ["entry", "rest"],
                match(var("rest"),
                    branch("List.nil", [], call("entryLine", var("entry"))),
                    branch("List.cons", ["next", "more"], join(strings(call("entryLine", var("entry")), call("entryLines", var("rest"))), ","))))),
        recursive="entries"),
    definition("snapshotPreimage", [("project", named("Project")), ("parent", STRING)], STRING,
        preimage_body(var("project"), var("parent"))),

    # ---- restore: the stored bytes must re-derive to the address they were stored under
    definition("restoreDecision", [("derived", STRING), ("expected", STRING)], named("Decision"),
        if_(equal(var("derived"), var("expected")), ctor("Decision.Accept"), ctor("Decision.Refuse"))),

    # ---- capabilities: admitted only when listed; nothing listed, nothing admitted
    definition("grantEndpoint", [("grant", named("Grant"))], STRING, owned(project("endpoint", var("grant")))),
    definition("admits", [("endpoints", lst(named("Grant"))), ("origin", STRING)], BOOL,
        match(var("endpoints"),
            branch("List.nil", [], b(False)),
            branch("List.cons", ["grant", "rest"],
                if_(equal(call("grantEndpoint", var("grant")), var("origin")), b(True), call("admits", var("rest"), var("origin"))))),
        recursive="endpoints"),
    definition("networkDecision", [("capabilities", named("Capabilities")), ("origin", STRING)], named("Decision"),
        if_(call("admits", project("endpoints", var("capabilities")), var("origin")), ctor("Decision.Accept"), ctor("Decision.Refuse"))),

    # ---- versions: a branch is a name pointing at an address; the first ref for a branch is its head
    structure("Ref", branch=STRING, kappa=STRING),
    definition("refBranch", [("ref", named("Ref"))], STRING, owned(project("branch", var("ref")))),
    definition("refKappa", [("ref", named("Ref"))], STRING, owned(project("kappa", var("ref")))),
    definition("headOf", [("refs", lst(named("Ref"))), ("branch", STRING)], opt(STRING),
        match(var("refs"),
            branch("List.nil", [], ctor("Option.none", targs=[STRING])),
            branch("List.cons", ["ref", "rest"],
                if_(equal(call("refBranch", var("ref")), var("branch")), ctor("Option.some", call("refKappa", var("ref")), targs=[STRING]), call("headOf", var("rest"), var("branch"))))),
        recursive="refs"),
    # ---- preview: a project state is served under its own address on the page origin
    definition("previewPath", [("kappa", STRING)], STRING, join(strings(s("/p/"), var("kappa"), s("/")))),

    # ---- the words
    definition("view", [], named("View"), record("View",
        headline=s("Own Your Ideas"),
        lede=s("Describe it. It builds, runs and stays yours, in this browser, on no server."),
        promptPlaceholder=s("What do you want to build?"),
        sendLabel=s("Build"),
        buildingLabel=s("building in your browser"),
        previewLabel=s("Preview"),
        snapshotLabel=s("Sealed"),
        rollbackLabel=s("Go back"),
        refusedLabel=s("Refused: not what it claims to be"),
        offlineLabel=s("offline, working from your device"),
        publishLabel=s("Publish as a Hologram application"),
        publishingLabel=s("cooking your application through PrismPM"),
        publishedLabel=s("Published"),
        publishRefusedLabel=s("Refused: the evidence does not match this shell"),
        holoLabel=s("Your .holo application"),
        downloadLabel=s("Download"),
        openLabel=s("Open"),
        runsLabel=s("Runs on Hologram Desktop, Hologram Server and in any browser, with no server"),
        holoTitle=s("Run a Hologram application"),
        holoLede=s("The file is checked by PrismPM's own validator before it runs. Nothing leaves this browser."),
        pickLabel=s("Choose a .holo file"),
        verifyingLabel=s("verifying"))),

    # ---- theorems: the shape of the rules, by unfolding; bytes are pinned by the corpus
    theorem("view_headline", eq(project("headline", call("view")), s("Own Your Ideas"))),
    theorem("admits_nothing_without_grants", eq(call("admits", nil(named("Grant")), s("https://example.com")), b(False))),
    theorem("refuses_without_grants",
        eq(call("networkDecision", record("Capabilities", endpoints=nil(named("Grant"))), s("https://example.com")), ctor("Decision.Refuse"))),
    theorem("no_head_without_refs", eq(call("headOf", nil(named("Ref")), s("main")), ctor("Option.none", targs=[STRING]))),
    theorem("preview_path_shape", eq(call("previewPath", s("k")), join(strings(s("/p/"), s("k"), s("/"))))),
    theorem("empty_project_preimage",
        eq(call("snapshotPreimage", record("Project", label=s("a"), entries=nil(named("Entry"))), s("")),
           join(strings(s('{"entries":['), call("entryLines", nil(named("Entry"))), s('],"name":'), q(call("projectName", record("Project", label=s("a"), entries=nil(named("Entry"))))), s(',"parent":'), q(s("")), s("}"))))),
]

