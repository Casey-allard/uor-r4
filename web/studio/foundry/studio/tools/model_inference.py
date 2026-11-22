"""Declarations of the freeinference-prism model, as data. Imported by tools/author.py, which partitions them into
LexLean modules, qualifies cross module references and writes src/*.lex.tex. Not run on its own."""
import json, pathlib

NL = "\n"
BS = "\\"

# Exact axiom sets Lean observes per declaration, pinned by tools/pin_axioms.py.
# LexLean treats the `axioms` field as an exact policy; anything else is refused.
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
def utf8(x): return prim("utf8_encode", BYTES, x)
def eq(l, r): return {"kind": "eq", "left": l, "right": r}
# Arithmetic and comparison as LexLean spells them: `add` is a term kind; multiply and quotient are
# primitives (quotient carries the zero divisor case); ble is the boolean order.
def add(l, r): return {"kind": "add", "left": l, "right": r}
NAT = {"kind": "nat"}
def nat(v): return {"kind": "nat", "value": str(v)}
def mul(l, r): return prim("multiply", NAT, l, r)
def quot(l, r): return prim("quotient", NAT, l, r, nat(0))
def ble(l, r): return {"kind": "ble", "left": l, "right": r}
def band(l, r): return {"kind": "and", "left": l, "right": r}
# An owned copy of a string expression: split on newline, join with newline. Identity on every
# string; rendered by the generator as split(...).map(String::from).collect() then join.
def owned(expr):
    return match(prim("split_exact", opt(lst(STRING)), expr, s(NL), u32(2147483647)),
                 branch("Option.none", [], s("")),
                 branch("Option.some", ["fields"], join(var("fields"), NL)))

# The wire: every byte of an OpenAI compatible response is built here. JSON escaping is split and
# join, each step owned, in the order backslash first; created and temperature arrive spelled.
def esc_step(name, needle, replacement, inner):
    return definition(name, [("value", STRING)], STRING,
        match(prim("split_exact", opt(lst(STRING)), inner, s(needle), u32(2147483647)),
              branch("Option.none", [], s("")),
              branch("Option.some", ["parts"], join(var("parts"), replacement))))
def field_of(type_name, field):
    return definition(field + "Of", [("value", named(type_name))], STRING, owned(project(field, var("value"))))
def q(expr): return join(strings(s('"'), call("escapeJson", expr), s('"')))
def completion_head(c):
    return [s('{"id":'), q(call("idOf", c)), s(',"object":"chat.completion","created":'), call("createdOf", c), s(',"model":'), q(call("modelOf", c)), s(',"system_fingerprint":'), q(call("fingerprintOf", c))]
def chunk_head(c):
    return [s('{"id":'), q(call("idOf", c)), s(',"object":"chat.completion.chunk","created":'), call("createdOf", c), s(',"model":'), q(call("modelOf", c)), s(',"system_fingerprint":'), q(call("fingerprintOf", c))]
def completion_record():
    return record("Completion", id=s("i"), created=s("1"), model=s("m"), text=s("t"), fingerprint=s("f"), receipt=s("r"))

# The route: who answers a request. Written once here so the definition and its theorems share the term.
def route_body(hit, gpu):
    return if_(hit, ctor("Route.Serve"), if_(gpu, ctor("Route.Local"), ctor("Route.NoGpu")))
# The endpoint: ready when the local model is resident, or a paid key is kept and the device is online.
# Written once here so the definition and its theorems share the term.
def endpoint_body(resident, key, online):
    return if_(resident, b(True), if_(key, online, b(False)))
# The bytes OpenRouter receives: only these fields, in this order, nothing else, no key. Usage is asked
# for so the cost is known; reasoning is off because the product asks for answers, not thinking; only
# providers that honor every field may answer (measured: one provider ignored the reasoning field and
# returned an empty answer).
def openrouter_body(model, req, stream):
    return join(strings(s('{"model":'), q(var(model) if isinstance(model, str) else model), s(',"messages":['), call("orMessages", project("messages", req)), s("]"),
                        call("maxTokensField", req), call("seedField", req), call("temperatureField", req),
                        s(',"stream":'), call("streamText", stream), s(',"usage":{"include":true},"reasoning":{"enabled":false},"provider":{"require_parameters":true}}')))

# The κ object's addressing rule, written once so definitions and theorems share the term.
def expert_range(start, length, experts, expert):
    return record("Range", start=add(start, mul(expert, call("stride", length, experts))),
                  stop=add(start, mul(add(expert, nat(1)), call("stride", length, experts))))
def obj_entry(o):
    return join(strings(s('["'), call("escapeJson", call("objKind", o)), s('","'), call("escapeJson", call("objLabel", o)),
                        s('","'), call("escapeJson", call("objKappa", o)), s('",'), prim("format_decimal", STRING, project("bytes", o)), s("]")))
def shard_entry(sh):
    return join(strings(s('{"bytes":'), prim("format_decimal", STRING, project("bytes", sh)), s(',"kappa":'), q(call("shardKappa", sh)),
                        s(',"name":'), q(call("shardLabel", sh)), s(',"objects":'), q(call("shardObjects", sh)),
                        s(',"sha256":'), call("sha256Text", sh), s("}")))
def manifest_preimage(m):
    return join(strings(s('{"experts":'), prim("format_decimal", STRING, project("experts", m)), s(',"repo":'), q(call("manifestRepo", m)),
                        s(',"revision":'), q(call("manifestRevision", m)), s(',"shards":['), call("shardEntries", project("shards", m)), s("]"),
                        s(',"spec":'), q(call("manifestSpec", m)), s(',"table_rows":'), prim("format_decimal", STRING, project("tableRows", m)), s("}")))
def small_manifest():
    return record("Manifest", spec=s("hologram/kappa-object/2"), repo=s("r"), revision=s("v"), experts=u64(2), tableRows=u64(64),
                  shards=cons(record("Shard", label=s("a"), bytes=u64(10), sha256=s(""), kappa=s("blake3:aa"), objects=s("blake3:bb")),
                              nil(named("Shard"))))

# ---- declarations
def inductive(name, *ctors): return {"constructors": [{"fields": [], "name": c} for c in ctors], "kind": "inductive", "name": name, "parameters": [], "type_parameters": []}
def structure(name, **fields): return {"fields": [{"name": k, "type": v} for k, v in fields.items()], "kind": "structure", "name": name, "parameters": [], "type_parameters": []}
def definition(name, params, result, body, recursive=None):
    node = {"axioms": AXIOMS.get(name, []), "body": body, "kind": "definition", "name": name, "parameters": [{"name": k, "type": v} for k, v in params], "result": result}
    if recursive: node["recursive_argument"] = recursive
    return node
def theorem(name, statement, proof="reflexivity"): return {"axioms": AXIOMS.get(name, []), "kind": "theorem", "name": name, "parameters": [], "proof": {"kind": proof}, "statement": statement}

def message(role, content): return record("Message", role=s(role), content=s(content))
def request(messages, max_tokens=None, seed=None, temperature=""):
    return record("Request", model=s("webgpu:Qwen"), messages=messages,
                  maxTokens=ctor("Option.none", targs=[U64]) if max_tokens is None else ctor("Option.some", u64(max_tokens), targs=[U64]),
                  seed=ctor("Option.none", targs=[U64]) if seed is None else ctor("Option.some", u64(seed), targs=[U64]),
                  temperature=s(temperature))
def memo(*models): return record("Memo", model=strings(*[s(m) for m in models]), engineKappa=s("e"), promptKappa=s("p"), paramsKappa=s("q"), outputKappa=s("o"), receipt=s("r"))

# The exact params bytes for a request: keys sorted, nulls spelled null, temperature as the wire spells it
# (the empty string is absent). Written once here so the definition and its theorems share the term.
def params_body(req):
    return join(strings(
        s('{"max_tokens":'), call("optionalDecimal", project("maxTokens", req)),
        s(',"seed":'), call("optionalDecimal", project("seed", req)),
        s(',"temperature":'), call("temperatureText", req),
        s("}")))
def message_body(msg):
    return join(strings(call("roleOf", msg), s(": "), call("contentOf", msg)))

decls = [
    # The transcript one OpenAI request carries, as the daemon renders it.
    structure("Message", role=STRING, content=STRING),
    structure("Request", model=STRING, messages=lst(named("Message")), maxTokens=opt(U64), seed=opt(U64), temperature=STRING),
    # What a later request needs to find a sealed answer without executing.
    structure("Memo", model=lst(STRING), engineKappa=STRING, promptKappa=STRING, paramsKappa=STRING, outputKappa=STRING, receipt=STRING),
    # The bytes the hash adapter addresses; the model owns preimages, not digests.
    structure("Preimages", prompt=BYTES, params=BYTES),
    inductive("Decision", "Serve", "Execute", "Refuse"),
    # Who runs a request: this device, always; no route reaches a paid or remote model.
    inductive("Route", "Serve", "Local", "NoGpu"),
    # Every visible word of the page. The page is projected from this record by core/src/bin/project-site.rs;
    # no copy is written in HTML. Zero hyphens in any string, as the product's site rule requires.
    # A curated backdrop: an Unsplash photo vendored with the page, credited as the Unsplash License asks.
    structure("Wallpaper", file=STRING, label=STRING, author=STRING, authorUrl=STRING),
    structure("View", headline=STRING, lede=STRING, promptPlaceholder=STRING, sendLabel=STRING,
              loadingLabel=STRING, servedLabel=STRING, sealedLabel=STRING, rederiveLabel=STRING,
              identicalLabel=STRING, noGpuLabel=STRING, offlineLabel=STRING,
              modelLabel=STRING, appearanceLabel=STRING, darkLabel=STRING, lightLabel=STRING, immersiveLabel=STRING,
              wallpapers=lst(named("Wallpaper")),
              localLabel=STRING,
              ladderLabel=STRING, seedModelLabel=STRING, largerLoadingLabel=STRING, promotedLabel=STRING,
              connectLabel=STRING, connectedLabel=STRING, listeningLabel=STRING, notConnectedLabel=STRING,
              runLabel=STRING, verifyLabel=STRING, baseUrlLabel=STRING, anyKeyLabel=STRING, modelIdLabel=STRING,
              testLabel=STRING, stayOpenLabel=STRING, askLabel=STRING, secondTabLabel=STRING,
              copyLabel=STRING, copiedLabel=STRING, macLabel=STRING, windowsLabel=STRING),

    # Owned copies of projected strings live in their own record taking definitions: the generator
    # borrows a record parameter and returns an owned string, and a match nested inside a list literal
    # would lower to a closure the exporter refuses.
    definition("roleOf", [("message", named("Message"))], STRING, owned(project("role", var("message")))),
    definition("contentOf", [("message", named("Message"))], STRING, owned(project("content", var("message")))),
    definition("temperatureText", [("request", named("Request"))], STRING,
        if_(equal(project("temperature", var("request")), s("")), s("null"), owned(project("temperature", var("request"))))),
    definition("renderMessage", [("message", named("Message"))], STRING, message_body(var("message"))),
    # No intermediate list: the transcript is folded directly, one line per message, joined by newline.
    definition("renderPrompt", [("messages", lst(named("Message")))], STRING,
        match(var("messages"),
            branch("List.nil", [], s("")),
            branch("List.cons", ["message", "rest"],
                match(var("rest"),
                    branch("List.nil", [], call("renderMessage", var("message"))),
                    branch("List.cons", ["next", "more"], join(strings(call("renderMessage", var("message")), call("renderPrompt", var("rest"))), NL))))),
        recursive="messages"),
    definition("optionalDecimal", [("value", opt(U64))], STRING,
        match(var("value"),
            branch("Option.none", [], s("null")),
            branch("Option.some", ["number"], prim("format_decimal", STRING, var("number"))))),
    definition("paramsCanonical", [("request", named("Request"))], STRING, params_body(var("request"))),
    definition("preimages", [("request", named("Request"))], named("Preimages"),
        record("Preimages",
               prompt=utf8(call("renderPrompt", project("messages", var("request")))),
               params=utf8(call("paramsCanonical", var("request"))))),

    definition("anyEqual", [("candidates", lst(STRING)), ("model", STRING)], BOOL,
        match(var("candidates"),
            branch("List.nil", [], b(False)),
            branch("List.cons", ["candidate", "rest"], if_(equal(var("candidate"), var("model")), b(True), call("anyEqual", var("rest"), var("model"))))),
        recursive="candidates"),
    definition("intersects", [("models", lst(STRING)), ("candidates", lst(STRING))], BOOL,
        match(var("models"),
            branch("List.nil", [], b(False)),
            branch("List.cons", ["model", "rest"], if_(call("anyEqual", var("candidates"), var("model")), b(True), call("intersects", var("rest"), var("candidates"))))),
        recursive="models"),
    definition("memoMatches", [("memo", named("Memo")), ("candidates", lst(STRING)), ("promptKappa", STRING), ("paramsKappa", STRING)], BOOL,
        if_(equal(project("promptKappa", var("memo")), var("promptKappa")),
            if_(equal(project("paramsKappa", var("memo")), var("paramsKappa")),
                call("intersects", project("model", var("memo")), var("candidates")),
                b(False)),
            b(False))),
    definition("view", [], named("View"), record("View",
        headline=s("Own Your Ideas"),
        lede=s("Seamlessly build, run, share and earn from your serverless AI applications."),
        promptPlaceholder=s("Ask anything"),
        sendLabel=s("Ask"),
        loadingLabel=s("getting the model, once"),
        servedLabel=s("Instant, from the seal"),
        sealedLabel=s("Sealed"),
        rederiveLabel=s("Check again"),
        identicalLabel=s("Checked, identical"),
        noGpuLabel=s("This browser cannot run the model. Try Chrome or Edge on a computer."),
        offlineLabel=s("offline, working from your device"),
        modelLabel=s("Qwen2.5 1.5B, on your device"),
        appearanceLabel=s("Appearance"),
        darkLabel=s("Dark"),
        lightLabel=s("Light"),
        immersiveLabel=s("Immersive"),
        wallpapers=cons(record("Wallpaper", file=s("alps.jpg"), label=s("Alpine Dawn"), author=s("Unsplash"), authorUrl=s("https://unsplash.com/?utm_source=Hologram_AI&utm_medium=referral")),
                   cons(record("Wallpaper", file=s("galaxy.jpg"), label=s("Galaxy"), author=s("Tiago Ferreira"), authorUrl=s("https://unsplash.com/@tiago_f_ferreira?utm_source=Hologram_AI&utm_medium=referral")),
                   cons(record("Wallpaper", file=s("aurora.jpg"), label=s("Aurora"), author=s("Lightscape"), authorUrl=s("https://unsplash.com/@lightscape?utm_source=Hologram_AI&utm_medium=referral")),
                   nil(named("Wallpaper"))))),
        localLabel=s("On your device"),
        # The ladder's words: the small model answers while the larger one loads, and the swap is announced once.
        ladderLabel=s("The best model your device has ready"), seedModelLabel=s("SmolLM2 360M, on your device"),
        largerLoadingLabel=s("getting Qwen2.5 1.5B behind it"), promotedLabel=s("Now answering with Qwen2.5 1.5B"),
        connectLabel=s("Connect"), connectedLabel=s("Connected"), listeningLabel=s("Listening for the relay"),
        notConnectedLabel=s("Not connected"), runLabel=s("Run this once, on this computer"), verifyLabel=s("Verify the file"),
        baseUrlLabel=s("Base URL"), anyKeyLabel=s("Any key works, for example local"), modelIdLabel=s("Model"),
        testLabel=s("Send a test request"), stayOpenLabel=s("Nothing leaves your device. Keep this tab open."),
        askLabel=s("Your browser may ask to let this page reach your computer."), secondTabLabel=s("Another tab is already serving"),
        copyLabel=s("Copy"), copiedLabel=s("Copied"), macLabel=s("macOS / Linux"), windowsLabel=s("Windows"))),
    # One answer as the wire sees it: created is a decimal string the adapter spells.
    structure("Completion", id=STRING, created=STRING, model=STRING, text=STRING, fingerprint=STRING, receipt=STRING),
    field_of("Completion", "id"), field_of("Completion", "created"), field_of("Completion", "model"),
    field_of("Completion", "text"), field_of("Completion", "fingerprint"), field_of("Completion", "receipt"),
    esc_step("escapeBackslash", BS, BS + BS, var("value")),
    esc_step("escapeQuote", '"', BS + '"', call("escapeBackslash", var("value"))),
    esc_step("escapeNewline", NL, BS + "n", call("escapeQuote", var("value"))),
    esc_step("escapeReturn", "\r", BS + "r", call("escapeNewline", var("value"))),
    esc_step("escapeJson", "\t", BS + "t", call("escapeReturn", var("value"))),
    definition("encodeCompletion", [("completion", named("Completion"))], STRING,
        join(strings(*completion_head(var("completion")),
                     s(',"choices":[{"index":0,"message":{"role":"assistant","content":'), q(call("textOf", var("completion"))),
                     s(',"refusal":null},"logprobs":null,"finish_reason":"stop"}],"usage":null}')))),
    definition("encodeRole", [("completion", named("Completion"))], STRING,
        join(strings(*chunk_head(var("completion")), s(',"choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}')))),
    definition("encodeDelta", [("completion", named("Completion")), ("delta", STRING)], STRING,
        join(strings(*chunk_head(var("completion")), s(',"choices":[{"index":0,"delta":{"content":'), q(var("delta")), s('},"finish_reason":null}]}')))),
    definition("encodeFinal", [("completion", named("Completion"))], STRING,
        join(strings(*chunk_head(var("completion")), s(',"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"hologram":{"receipt":'), q(call("receiptOf", var("completion"))), s("}}")))),
    definition("done", [], STRING, s("[DONE]")),
    definition("encodeError", [("message", STRING), ("kind", STRING)], STRING,
        join(strings(s('{"error":{"message":'), q(var("message")), s(',"type":'), q(var("kind")), s(',"param":null,"code":null}}')))),
    definition("modelEntry", [("id", STRING)], STRING,
        join(strings(s('{"id":'), q(var("id")), s(',"object":"model","created":0,"owned_by":"browser"}')))),
    definition("modelEntries", [("ids", lst(STRING))], STRING,
        match(var("ids"),
            branch("List.nil", [], s("")),
            branch("List.cons", ["id", "rest"],
                match(var("rest"),
                    branch("List.nil", [], call("modelEntry", owned(var("id")))),
                    branch("List.cons", ["next", "more"], join(strings(call("modelEntry", owned(var("id"))), call("modelEntries", var("rest"))), ","))))),
        recursive="ids"),
    definition("encodeModels", [("ids", lst(STRING))], STRING,
        join(strings(s('{"object":"list","data":['), call("modelEntries", var("ids")), s("]}")))),
    definition("route", [("hit", BOOL), ("gpuReady", BOOL)], named("Route"),
        route_body(var("hit"), var("gpuReady"))),
    # The endpoint control: shown when a request could be answered, resident model or paid key online.
    definition("endpointReady", [("resident", BOOL), ("keyPresent", BOOL), ("online", BOOL)], BOOL,
        endpoint_body(var("resident"), var("keyPresent"), var("online"))),
    definition("orMessage", [("message", named("Message"))], STRING,
        join(strings(s('{"role":'), q(call("roleOf", var("message"))), s(',"content":'), q(call("contentOf", var("message"))), s("}")))),
    definition("orMessages", [("messages", lst(named("Message")))], STRING,
        match(var("messages"),
            branch("List.nil", [], s("")),
            branch("List.cons", ["message", "rest"],
                match(var("rest"),
                    branch("List.nil", [], call("orMessage", var("message"))),
                    branch("List.cons", ["next", "more"], join(strings(call("orMessage", var("message")), call("orMessages", var("rest"))), ","))))),
        recursive="messages"),
    definition("maxTokensField", [("request", named("Request"))], STRING,
        match(project("maxTokens", var("request")),
            branch("Option.none", [], s("")),
            branch("Option.some", ["number"], join(strings(s(',"max_tokens":'), prim("format_decimal", STRING, var("number"))))))),
    definition("seedField", [("request", named("Request"))], STRING,
        match(project("seed", var("request")),
            branch("Option.none", [], s("")),
            branch("Option.some", ["number"], join(strings(s(',"seed":'), prim("format_decimal", STRING, var("number"))))))),
    definition("temperatureField", [("request", named("Request"))], STRING,
        if_(equal(project("temperature", var("request")), s("")), s(""), join(strings(s(',"temperature":'), owned(project("temperature", var("request"))))))),
    definition("streamText", [("stream", BOOL)], STRING, if_(var("stream"), s("true"), s("false"))),
    definition("encodeOpenRouterRequest", [("model", STRING), ("request", named("Request")), ("stream", BOOL)], STRING,
        openrouter_body(var("model"), var("request"), var("stream"))),
    # ---- Address: the κ object. Every tensor a κ, every expert a page, every table page a κ, one root.
    structure("Range", start=NAT, stop=NAT),
    structure("Obj", kind=STRING, label=STRING, kappa=STRING, bytes=U64),
    structure("Shard", label=STRING, bytes=U64, sha256=STRING, kappa=STRING, objects=STRING),
    structure("Manifest", spec=STRING, repo=STRING, revision=STRING, experts=U64, tableRows=U64, shards=lst(named("Shard"))),
    # The rows of one expert inside a stacked expert tensor: stride is the tensor's bytes over the expert count.
    definition("stride", [("length", NAT), ("experts", NAT)], NAT, quot(var("length"), var("experts"))),
    definition("expertPage", [("start", NAT), ("length", NAT), ("experts", NAT), ("expert", NAT)], named("Range"),
        expert_range(var("start"), var("length"), var("experts"), var("expert"))),
    # One fixed page of the n gram table: whole rows, the last page shorter.
    definition("pageBytes", [("rowBytes", NAT), ("rows", NAT)], NAT, mul(var("rowBytes"), var("rows"))),
    definition("pageStart", [("start", NAT), ("rowBytes", NAT), ("rows", NAT), ("index", NAT)], NAT,
        add(var("start"), mul(var("index"), call("pageBytes", var("rowBytes"), var("rows"))))),
    definition("tablePage", [("start", NAT), ("stop", NAT), ("rowBytes", NAT), ("rows", NAT), ("index", NAT)], named("Range"),
        record("Range", start=call("pageStart", var("start"), var("rowBytes"), var("rows"), var("index")),
               stop=if_(ble(add(call("pageStart", var("start"), var("rowBytes"), var("rows"), var("index")), call("pageBytes", var("rowBytes"), var("rows"))), var("stop")),
                       add(call("pageStart", var("start"), var("rowBytes"), var("rows"), var("index")), call("pageBytes", var("rowBytes"), var("rows"))),
                       var("stop")))),
    # The root preimage: canonical JSON of the manifest, keys sorted, no spaces, an absent sha256 spelled null.
    # Owned copies of the records' strings, each in its own record taking definition (a copy inside a
    # list literal lowers to a closure the exporter refuses).
    definition("objKind", [("obj", named("Obj"))], STRING, owned(project("kind", var("obj")))),
    definition("objLabel", [("obj", named("Obj"))], STRING, owned(project("label", var("obj")))),
    definition("objKappa", [("obj", named("Obj"))], STRING, owned(project("kappa", var("obj")))),
    definition("shardLabel", [("shard", named("Shard"))], STRING, owned(project("label", var("shard")))),
    definition("shardKappa", [("shard", named("Shard"))], STRING, owned(project("kappa", var("shard")))),
    definition("shardObjects", [("shard", named("Shard"))], STRING, owned(project("objects", var("shard")))),
    definition("shardSha256", [("shard", named("Shard"))], STRING, owned(project("sha256", var("shard")))),
    definition("manifestRepo", [("manifest", named("Manifest"))], STRING, owned(project("repo", var("manifest")))),
    definition("manifestRevision", [("manifest", named("Manifest"))], STRING, owned(project("revision", var("manifest")))),
    definition("manifestSpec", [("manifest", named("Manifest"))], STRING, owned(project("spec", var("manifest")))),
    definition("sha256Text", [("shard", named("Shard"))], STRING,
        if_(equal(project("sha256", var("shard")), s("")), s("null"), q(call("shardSha256", var("shard"))))),
    definition("objEntry", [("obj", named("Obj"))], STRING, obj_entry(var("obj"))),
    # The lists fold with an accumulator so the recursion sits in tail position: a real manifest has
    # tens of thousands of objects, and a descent per element would exhaust any stack.
    # One object is one line of its shard's object list; the list itself is a κ object the shard names.
    # (A per element recursion over a real list of 156,256 objects is a descent per element in the
    # generated code, so the list is not folded in the model: finding 8 in VERIFICATION.md.)
    definition("shardEntry", [("shard", named("Shard"))], STRING, shard_entry(var("shard"))),
    definition("shardEntriesFrom", [("shards", lst(named("Shard"))), ("acc", STRING)], STRING,
        match(var("shards"),
            branch("List.nil", [], var("acc")),
            branch("List.cons", ["shard", "rest"],
                call("shardEntriesFrom", var("rest"),
                     if_(equal(var("acc"), s("")), call("shardEntry", var("shard")), join(strings(var("acc"), call("shardEntry", var("shard"))), ","))))),
        recursive="shards"),
    definition("shardEntries", [("shards", lst(named("Shard")))], STRING, call("shardEntriesFrom", var("shards"), s(""))),
    definition("rootPreimage", [("manifest", named("Manifest"))], STRING, manifest_preimage(var("manifest"))),
    # A page binds only if the root lists its κ and the bytes derive that κ.
    definition("admitPage", [("listed", lst(STRING)), ("kappa", STRING), ("derived", STRING)], BOOL,
        band(call("anyEqual", var("listed"), owned(var("kappa"))), equal(var("kappa"), var("derived")))),
    # ---- Pool and Stage: the streaming expert pool's rules, as edge0 runs them, in verified form.
    # TrueRouting: the router decides and a page missing from the slots is fetched before the step.
    # StagedReplace: the prerouter's prediction is the routing, and a missing page is dropped (the overflow row).
    inductive("Staging", "TrueRouting", "StagedReplace"),
    inductive("PageAction", "Bind", "Fetch", "Drop"),
    definition("pageAction", [("resident", BOOL), ("staging", named("Staging"))], named("PageAction"),
        if_(var("resident"), ctor("PageAction.Bind"),
            match(var("staging"), branch("Staging.TrueRouting", [], ctor("PageAction.Fetch")), branch("Staging.StagedReplace", [], ctor("PageAction.Drop"))))),
    # Admission into the LRU pool: a present page is touched, a free slot takes a new page, else the oldest is evicted first.
    inductive("Admission", "Touch", "Insert", "EvictThenInsert"),
    definition("poolAdmit", [("present", BOOL), ("spaceLeft", BOOL)], named("Admission"),
        if_(var("present"), ctor("Admission.Touch"), if_(var("spaceLeft"), ctor("Admission.Insert"), ctor("Admission.EvictThenInsert")))),
    # Where a missing page comes from: the device store first, then a peer when it is faster, then a mirror, else nowhere.
    inductive("Source", "Device", "Peer", "Mirror", "Nowhere"),
    definition("fetchSource", [("onDevice", BOOL), ("onMirror", BOOL), ("peerFaster", BOOL)], named("Source"),
        if_(var("onDevice"), ctor("Source.Device"),
            if_(var("peerFaster"), ctor("Source.Peer"), if_(var("onMirror"), ctor("Source.Mirror"), ctor("Source.Nowhere"))))),
    # What the prefetcher pulls next: a predicted page first, a popular page to fill, nothing otherwise.
    inductive("Priority", "First", "Fill", "Skip"),
    definition("prefetchOrder", [("predicted", BOOL), ("popular", BOOL)], named("Priority"),
        if_(var("predicted"), ctor("Priority.First"), if_(var("popular"), ctor("Priority.Fill"), ctor("Priority.Skip")))),
    # ---- Pack, Ladder, Loader: the archive's first use order, which model answers, how a visit starts.
    # Pack: sections in the order a first visit needs them; the n gram table is never packed (it is pages).
    inductive("Section", "Header", "Tokenizer", "Spine", "Expert", "Table"),
    definition("packRank", [("part", named("Section"))], NAT,
        match(var("part"), branch("Section.Header", [], nat(0)), branch("Section.Tokenizer", [], nat(1)), branch("Section.Spine", [], nat(2)),
              branch("Section.Expert", [], nat(3)), branch("Section.Table", [], nat(4)))),
    definition("packed", [("part", named("Section"))], BOOL,
        match(var("part"), branch("Section.Header", [], b(True)), branch("Section.Tokenizer", [], b(True)), branch("Section.Spine", [], b(True)),
              branch("Section.Expert", [], b(True)), branch("Section.Table", [], b(False)))),
    # The first token needs the spine and the prompt's own expert pages, nothing more.
    definition("firstTokenReady", [("spinePresent", BOOL), ("promptPagesPresent", BOOL)], BOOL, band(var("spinePresent"), var("promptPagesPresent"))),
    # Ladder: the small resident model answers until the large one is resident and measured fast enough; never demote silently.
    inductive("Tier", "Small", "Large"),
    definition("promote", [("current", named("Tier")), ("largeResident", BOOL), ("largeFast", BOOL)], named("Tier"),
        match(var("current"),
              branch("Tier.Large", [], ctor("Tier.Large")),
              branch("Tier.Small", [], if_(band(var("largeResident"), var("largeFast")), ctor("Tier.Large"), ctor("Tier.Small"))))),
    # Loader: how a visit starts. A session snapshot resumes; a shell on the device is warm; else cold.
    inductive("Start", "Cold", "Warm", "Resume"),
    definition("loaderStart", [("shellOnDevice", BOOL), ("snapshotOnDevice", BOOL)], named("Start"),
        if_(var("snapshotOnDevice"), ctor("Start.Resume"), if_(var("shellOnDevice"), ctor("Start.Warm"), ctor("Start.Cold")))),
    # Warm up: while the local model is not yet resident, a held key (the visitor's or the site's)
    # answers through OpenRouter and says so; without a key, or offline, the local model is waited for.
    definition("warmup", [("localReady", BOOL), ("keyPresent", BOOL), ("online", BOOL)], BOOL,
        if_(var("localReady"), b(False), band(var("keyPresent"), var("online")))),
    definition("decide", [("hit", BOOL), ("workerAttached", BOOL)], named("Decision"),
        if_(var("hit"), ctor("Decision.Serve"), if_(var("workerAttached"), ctor("Decision.Execute"), ctor("Decision.Refuse")))),

    # Closed facts checked by Lean. String literals are not kernel reducible in Lean 4.32 (String is byte
    # based; rfl and decide both stall on them), so each theorem pins the RULE definitionally: what the
    # function unfolds to for a concrete input. The literal bytes are then pinned by the execution corpus.
    theorem("renderMessage_shape",
        eq(call("renderMessage", message("user", "hello")), message_body(message("user", "hello")))),
    theorem("renderPrompt_singleUser",
        eq(call("renderPrompt", cons(message("user", "hello"), nil(named("Message")))), call("renderMessage", message("user", "hello")))),
    theorem("renderPrompt_twoTurns",
        eq(call("renderPrompt", cons(message("user", "a"), cons(message("assistant", "b"), nil(named("Message"))))),
           join(strings(call("renderMessage", message("user", "a")), call("renderPrompt", cons(message("assistant", "b"), nil(named("Message"))))), NL))),
    theorem("paramsCanonical_empty",
        eq(call("paramsCanonical", request(nil(named("Message")))), params_body(request(nil(named("Message")))))),
    theorem("paramsCanonical_full",
        eq(call("paramsCanonical", request(nil(named("Message")), 32, 1, "0.0")), params_body(request(nil(named("Message")), 32, 1, "0.0")))),
    theorem("view_headline", eq(project("headline", call("view")), s("Own Your Ideas"))),
    theorem("done_frame", eq(call("done"), s("[DONE]"))),
    theorem("encodeError_shape",
        eq(call("encodeError", s("m"), s("k")),
           join(strings(s('{"error":{"message":'), q(s("m")), s(',"type":'), q(s("k")), s(',"param":null,"code":null}}'))))),
    theorem("encodeCompletion_shape",
        eq(call("encodeCompletion", completion_record()),
           join(strings(*completion_head(completion_record()),
                        s(',"choices":[{"index":0,"message":{"role":"assistant","content":'), q(call("textOf", completion_record())),
                        s(',"refusal":null},"logprobs":null,"finish_reason":"stop"}],"usage":null}'))))),
    theorem("encodeModels_empty", eq(call("encodeModels", nil(STRING)), join(strings(s('{"object":"list","data":['), call("modelEntries", nil(STRING)), s("]}"))))),
    # The route table, every row. A hit serves; the device runs a request only with a GPU.
    theorem("route_hitLocal", eq(call("route", b(True), b(False)), ctor("Route.Serve"))),
    theorem("route_local", eq(call("route", b(False), b(True)), ctor("Route.Local"))),
    theorem("route_noGpu", eq(call("route", b(False), b(False)), ctor("Route.NoGpu"))),
    # The endpoint readiness table, every row: a resident model is enough; a key needs the network.
    theorem("endpointReady_resident", eq(call("endpointReady", b(True), b(False), b(False)), b(True))),
    theorem("endpointReady_paid", eq(call("endpointReady", b(False), b(True), b(True)), b(True))),
    theorem("endpointReady_paidOffline", eq(call("endpointReady", b(False), b(True), b(False)), b(False))),
    theorem("endpointReady_nothing", eq(call("endpointReady", b(False), b(False), b(True)), b(False))),
    theorem("orMessages_empty", eq(call("orMessages", nil(named("Message"))), s(""))),
    theorem("streamText_true", eq(call("streamText", b(True)), s("true"))),
    theorem("encodeOpenRouterRequest_shape",
        eq(call("encodeOpenRouterRequest", s("m"), request(nil(named("Message")), 32, 1, "0.7"), b(True)),
           openrouter_body(s("m"), request(nil(named("Message")), 32, 1, "0.7"), b(True)))),
    # The addressing rule's rows.
    theorem("expertPage_shape", eq(call("expertPage", nat(100), nat(80), nat(4), nat(1)), expert_range(nat(100), nat(80), nat(4), nat(1)))),
    theorem("stride_shape", eq(call("stride", nat(80), nat(4)), quot(nat(80), nat(4)))),
    theorem("pageStart_shape", eq(call("pageStart", nat(7), nat(2), nat(3), nat(5)), add(nat(7), mul(nat(5), call("pageBytes", nat(2), nat(3)))))),
    theorem("rootPreimage_shape", eq(call("rootPreimage", small_manifest()), manifest_preimage(small_manifest()))),
    theorem("objEntry_shape", eq(call("objEntry", record("Obj", kind=s("tensor"), label=s("t"), kappa=s("blake3:bb"), bytes=u64(4))), obj_entry(record("Obj", kind=s("tensor"), label=s("t"), kappa=s("blake3:bb"), bytes=u64(4))))),
    # admitPage pins its shape definitionally (its string copy is split and join, which decide cannot reduce);
    # the rows are pinned by the corpus through the crate and the guest.
    theorem("admitPage_shape",
        eq(call("admitPage", strings(s("blake3:a")), s("blake3:a"), s("blake3:b")),
           band(call("anyEqual", strings(s("blake3:a")), owned(s("blake3:a"))), equal(s("blake3:a"), s("blake3:b"))))),
    # The pool and stage tables, every row.
    theorem("pageAction_residentTrue", eq(call("pageAction", b(True), ctor("Staging.TrueRouting")), ctor("PageAction.Bind"))),
    theorem("pageAction_residentStaged", eq(call("pageAction", b(True), ctor("Staging.StagedReplace")), ctor("PageAction.Bind"))),
    theorem("pageAction_missTrue", eq(call("pageAction", b(False), ctor("Staging.TrueRouting")), ctor("PageAction.Fetch"))),
    theorem("pageAction_missStaged", eq(call("pageAction", b(False), ctor("Staging.StagedReplace")), ctor("PageAction.Drop"))),
    theorem("poolAdmit_present", eq(call("poolAdmit", b(True), b(False)), ctor("Admission.Touch"))),
    theorem("poolAdmit_space", eq(call("poolAdmit", b(False), b(True)), ctor("Admission.Insert"))),
    theorem("poolAdmit_full", eq(call("poolAdmit", b(False), b(False)), ctor("Admission.EvictThenInsert"))),
    theorem("fetchSource_device", eq(call("fetchSource", b(True), b(True), b(True)), ctor("Source.Device"))),
    theorem("fetchSource_peer", eq(call("fetchSource", b(False), b(True), b(True)), ctor("Source.Peer"))),
    theorem("fetchSource_mirror", eq(call("fetchSource", b(False), b(True), b(False)), ctor("Source.Mirror"))),
    theorem("fetchSource_nowhere", eq(call("fetchSource", b(False), b(False), b(False)), ctor("Source.Nowhere"))),
    theorem("prefetchOrder_predicted", eq(call("prefetchOrder", b(True), b(False)), ctor("Priority.First"))),
    theorem("prefetchOrder_popular", eq(call("prefetchOrder", b(False), b(True)), ctor("Priority.Fill"))),
    theorem("prefetchOrder_skip", eq(call("prefetchOrder", b(False), b(False)), ctor("Priority.Skip"))),
    # Pack, Ladder and Loader, every row.
    theorem("packRank_header", eq(call("packRank", ctor("Section.Header")), nat(0))),
    theorem("packRank_spine", eq(call("packRank", ctor("Section.Spine")), nat(2))),
    theorem("packRank_expert", eq(call("packRank", ctor("Section.Expert")), nat(3))),
    theorem("packed_table", eq(call("packed", ctor("Section.Table")), b(False))),
    theorem("packed_spine", eq(call("packed", ctor("Section.Spine")), b(True))),
    theorem("firstTokenReady_both", eq(call("firstTokenReady", b(True), b(True)), b(True))),
    theorem("firstTokenReady_noPages", eq(call("firstTokenReady", b(True), b(False)), b(False))),
    theorem("promote_stays", eq(call("promote", ctor("Tier.Large"), b(False), b(False)), ctor("Tier.Large"))),
    theorem("promote_up", eq(call("promote", ctor("Tier.Small"), b(True), b(True)), ctor("Tier.Large"))),
    theorem("promote_notResident", eq(call("promote", ctor("Tier.Small"), b(False), b(True)), ctor("Tier.Small"))),
    theorem("promote_notFast", eq(call("promote", ctor("Tier.Small"), b(True), b(False)), ctor("Tier.Small"))),
    theorem("loaderStart_resume", eq(call("loaderStart", b(True), b(True)), ctor("Start.Resume"))),
    theorem("loaderStart_warm", eq(call("loaderStart", b(True), b(False)), ctor("Start.Warm"))),
    theorem("loaderStart_cold", eq(call("loaderStart", b(False), b(False)), ctor("Start.Cold"))),
    theorem("warmup_notReadyKeyOnline", eq(call("warmup", b(False), b(True), b(True)), b(True))),
    theorem("warmup_ready", eq(call("warmup", b(True), b(True), b(True)), b(False))),
    theorem("warmup_noKey", eq(call("warmup", b(False), b(False), b(True)), b(False))),
    theorem("warmup_offline", eq(call("warmup", b(False), b(True), b(False)), b(False))),
    theorem("decide_serve", eq(call("decide", b(True), b(False)), ctor("Decision.Serve"))),
    theorem("decide_execute", eq(call("decide", b(False), b(True)), ctor("Decision.Execute"))),
    theorem("decide_refuse", eq(call("decide", b(False), b(False)), ctor("Decision.Refuse"))),
    theorem("memoMatches_ownKey",
        eq(call("memoMatches", memo("webgpu:Qwen"), strings(s("webgpu:Qwen")), s("p"), s("q")), b(True)), proof="decide"),
    theorem("memoMatches_otherPrompt",
        eq(call("memoMatches", memo("webgpu:Qwen"), strings(s("webgpu:Qwen")), s("x"), s("q")), b(False)), proof="decide"),
    theorem("memoMatches_otherModel",
        eq(call("memoMatches", memo("webgpu:Qwen"), strings(s("smollm2")), s("p"), s("q")), b(False)), proof="decide"),
]

