#!/usr/bin/env python3
"""Independent coupled episode saved-arithmetic core.
Producer adapter is NOT_READY until its exact journal schema is frozen.
No encoder, backward, autoregressive rollout, model dependency, or new proposal.
Arithmetic may be executed only under the parent's subsequent audit admission.
"""
import array
import hashlib
import importlib.util
import json
import math
import pathlib
import struct

AUTHORITY = pathlib.Path("/workspace/uor-r4/codex/sol-generate-episode-learning/generate-independent-reader.py")
AUTHORITY_SHA = "8e5383e5ac93133c071ab540cf85cece0edc6e70d95367a4230d8574906ec8f4"
PREFIX = "prefix.coefficients"
GENERATE = "generate.unary"
U20 = 1 << 20
U22 = 1 << 22
N = 4096

def need(ok, why):
    if not ok:
        raise ValueError(why)

def file_sha(path):
    h = hashlib.sha256()
    with pathlib.Path(path).open("rb") as stream:
        for b in iter(lambda: stream.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()

def authenticated_kernel():
    need(file_sha(AUTHORITY) == AUTHORITY_SHA, "immutable fixture-verified sparse kernel authority")
    spec = importlib.util.spec_from_file_location("immutable_generate_saved_arithmetic", AUTHORITY)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]

def code(m):
    need(math.isfinite(m) and -1.75 <= m <= 1.75, "finite original admitted Q4 master")
    z = m * 4.
    return math.floor(z + .5) if z >= 0. else math.ceil(z - .5)

def total_key(x):
    need(math.isfinite(x), "finite widened actual-displacement utility")
    return (x, 0 if math.copysign(1., x) < 0. else 1)

def mixed_coordinate_order(masters, gradients):
    """Priority from original f32 values; zero/saturation retain fractional bits."""
    need(set(masters) == set(gradients) == {PREFIX, GENERATE}, "exact two active families")
    result = []
    for family in sorted(masters):
        need(len(masters[family]) == len(gradients[family]) == 960, "each family960")
        for index, (m, g) in enumerate(zip(masters[family], gradients[family])):
            need(f32(m) == m and f32(g) == g, "original exact widened f32 values")
            q = code(m)
            if family == PREFIX:
                c = max(-7, min(7, q + (1 if g < 0. else -1 if g > 0. else 0)))
                noop = c == q
                target = m if noop else f32(c * .25)
                utility = g * (target - m)
            else:
                c, utility = min(((c, g * (f32(c * .25) - m))
                                  for c in range(-7, 8) if c != q),
                                 key=lambda pair: (total_key(pair[1]), pair[0]))
                target, noop = f32(c * .25), False
            result.append({"family": family, "index": index, "initial_code": q,
                           "ranking_code": c, "ranking_target": target,
                           "ranking_actual_delta": target - m, "utility": utility,
                           "noop": noop})
    result.sort(key=lambda r: (total_key(r["utility"]), r["family"], r["index"]))
    for order, r in enumerate(result):
        r["order"] = order
    return result

class RowIncidence:
    """960 offsets plus sentinel, sorted u16 tokens; no global CSR clones."""
    def __init__(self, token_keys):
        need(len(token_keys) == N * 8, "all4096 Generate tokens eight native factor keys")
        counts = [0] * 960
        for token in range(N):
            keys = token_keys[token*8:token*8+8]
            need(all(isinstance(k, int) and lane*120 <= k < (lane+1)*120
                     for lane, k in enumerate(keys)), "one legal key per lane, every token")
            for k in keys:
                counts[k] += 1
        self.offsets = array.array("I", [0])
        for n in counts:
            self.offsets.append(self.offsets[-1] + n)
        self.tokens = array.array("H", [0]) * (N * 8)
        cursors = list(self.offsets[:-1])
        for token in range(N):
            for k in token_keys[token*8:token*8+8]:
                self.tokens[cursors[k]] = token
                cursors[k] += 1
        need(self.offsets[-1] == N*8 and all(cursors[k] == self.offsets[k+1]
             for k in range(960)), "complete incidence multiplicity")
    def for_key(self, key):
        need(0 <= key < 960, "legal unary coordinate")
        return memoryview(self.tokens)[self.offsets[key]:self.offsets[key+1]]
    def mathematical_digest(self):
        return integer_row_digest({"offsets":list(self.offsets), "tokens":list(self.tokens)})

def integer_row_digest(rows):
    """Only integers/bools in canonical ordered objects; never float JSON hashes."""
    def check(v):
        if isinstance(v, dict):
            return all(isinstance(k, str) and check(x) for k,x in v.items())
        if isinstance(v, (tuple,list)):
            return all(check(x) for x in v)
        return v is None or isinstance(v, (int,bool))
    need(check(rows), "integer mathematical digest only")
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(",",":")).encode()).hexdigest()

class CoupledPools:
    """Independent current-epoch pools; rejected Prefix replacements never mutate.
    frame['base'] and frame['keys'] are ORIGINAL physical BASECopy and Prefix keys.
    original_generate(post) supplies frozen original packed-factor raw Generate.
    token_keys(post) supplies all4096*8 native prototype-relative unary keys.
    bridge(frame, donor) supplies authenticated post-state from immutable bank states.
    Those pure saved finite callbacks are bound by the future producer adapter.
    """
    def __init__(self, kernel, frames, prefix_initial, unary_initial, weights,
                 original_generate, token_keys, bridge, legal_token_ids):
        need(tuple(legal_token_ids) == tuple(range(N)),
             "authenticated admission exactly all4096; sparse holes UNSUPPORTED")
        self.kernel, self.frames, self.weights = kernel, frames, weights
        self.prefix_initial = tuple(prefix_initial)
        self.unary_initial = tuple(unary_initial)
        need(len(prefix_initial) == len(unary_initial) == 960, "two original packed families")
        self.prefix = list(prefix_initial)
        self.unary = list(unary_initial)
        self.original_generate, self.token_keys, self.bridge = original_generate, token_keys, bridge
        self.epoch = 0
        self.rows = [self.build_row(i, self.prefix) for i in range(len(frames))]
        # Frame list is immutable; absence of a key proves unchanged physical Copy.
        self.prefix_rows = [array.array("H") for _ in range(960)]
        for row, f in enumerate(frames):
            need(len(f["keys"]) == len(f["ids"])*8, "every physical alias has eight Prefix keys")
            for k in sorted(set(f["keys"])):
                need(isinstance(k,int) and 0 <= k < 960, "unmasked legal Prefix key")
                self.prefix_rows[k].append(row)

    def build_row(self, row, prefix):
        f = self.frames[row]
        base = list(f["base"])
        need(len(base) == len(f["ids"]), "original full physical BASECopy")
        for ordinal in range(len(base)):
            base[ordinal] += sum((prefix[k] - self.prefix_initial[k])*U22
                                 for k in f["keys"][ordinal*8:ordinal*8+8])
        need(base, "nonempty captured physical source bank")
        donor = max(range(len(base)), key=lambda i: (base[i], -i))
        post = tuple(self.bridge(f, donor))
        previous = self.rows[row] if hasattr(self, "rows") else None
        copied = [base[o] + f["u"][t] for o,t in enumerate(f["ids"])]
        if previous is not None and previous["post"] == post:
            # Post, unary epoch and frozen U are unchanged: reuse immutable scores.
            # Fresh row identity plus copied small containers prevents foreign stage commit.
            incidence = previous["incidence"]
            old = previous["state"]
            state = self.kernel.PoolState.__new__(self.kernel.PoolState)
            state.scores = old.scores
            state.copy_ids = tuple(f["ids"])
            state.copy_scores = tuple(copied)
            state.copy_clipped = tuple(self.kernel.clipped(x) for x in copied)
            state.copy_max = max(state.copy_clipped, default=-self.kernel.CLIP)
            state.weights = self.weights
            state.counts = old.counts[:]
            state.members = old.members[:]
            state.occupied = old.occupied
            state.epoch = 0
            state._identity = object()
            state.current = state.stage((),None,initial=True)["summary"]
        else:
            keys = self.token_keys(post)
            incidence = RowIncidence(keys)
            raw = self.original_generate(post)
            need(len(raw) == len(f["u"]) == N, "complete original factors and frozen U")
            generated = [raw[t] + f["u"][t] +
                         sum((self.unary[k] - self.unary_initial[k])*U20
                             for k in keys[t*8:t*8+8]) for t in range(N)]
            state = self.kernel.PoolState(generated, f["ids"], copied, self.weights)
        return {"state":state, "donor":donor, "post":post,
                "base_copy":array.array("q",base), "incidence":incidence}

    def stage_prefix(self, index, destination):
        need(0 <= index < 960 and -7 <= destination <= 7, "legal Prefix destination")
        next_prefix = self.prefix[:]
        next_prefix[index] = destination
        affected = tuple(self.prefix_rows[index])
        # Every affected row constructed before any commit/veto receipt.
        replacements = {r:self.build_row(r,next_prefix) for r in affected}
        return {"owner":self, "epoch":self.epoch, "family":PREFIX,
                "index":index, "before":self.prefix[index], "after":destination,
                "replacements":replacements}

    def stage_unary(self, index, destination, targets):
        need(0 <= index < 960 and -7 <= destination <= 7, "legal Generate destination")
        need(len(targets) == len(self.rows), "one objective/guard label per captured row")
        delta = (destination-self.unary[index])*U20
        stages = {}
        for row, current in enumerate(self.rows):
            tokens = current["incidence"].for_key(index)
            if not tokens:
                continue
            state = current["state"]
            stages[row] = state.stage(((t,state.scores[t]+delta) for t in tokens),targets[row])
        return {"owner":self, "epoch":self.epoch, "family":GENERATE,
                "index":index, "before":self.unary[index], "after":destination,
                "stages":stages}

    def summary_at(self, row, stage, target):
        need(stage["owner"] is self and stage["epoch"] == self.epoch, "current owned transaction")
        if stage["family"] == PREFIX and row in stage["replacements"]:
            state = stage["replacements"][row]["state"]
            return state.stage((),target)["summary"]
        if stage["family"] == GENERATE and row in stage["stages"]:
            return stage["stages"][row]["summary"]
        return self.rows[row]["state"].stage((),target)["summary"]

    def commit(self, stage):
        need(stage["owner"] is self and stage["epoch"] == self.epoch, "foreign/stale coupled stage")
        family, index = stage["family"], stage["index"]
        values = self.prefix if family == PREFIX else self.unary
        need(values[index] == stage["before"], "current original coordinate before-value")
        if family == GENERATE:
            batch = [(self.rows[r]["state"],s) for r,s in stage["stages"].items()]
            # Opaque per-row identity and every epoch validated before any mutation.
            self.kernel.commit_batch(batch)
        else:
            for row in stage["replacements"]:
                need(0 <= row < len(self.rows), "owned replacement row")
            for row,new in stage["replacements"].items():
                self.rows[row] = new
        values[index] = stage["after"]
        self.epoch += 1

def read_raw960(path):
    raw = pathlib.Path(path).read_bytes()
    need(len(raw) == 3840, "exact960 f32 raw bytes")
    values = list(struct.unpack("<960f",raw))
    need(all(math.isfinite(x) for x in values), "finite saved raw960")
    return raw,values

def audit_inherited_learning(run,cfg,report,args,p,inventories):
    mode=cfg["coupled_episode_learning"]
    retained=mode.get("retained_gradient") or (mode.get("retained_export") or {}).get("inherited_gradients")
    if retained is None:
        return None
    old=pathlib.Path(retained["retained_failed_root"])
    obs=pathlib.Path(retained["retained_learning_observation_root"])
    runtime=pathlib.Path(retained["retained_learning_runtime_root"])
    for root,leaf,key in [(old,"report.json","expected_report_sha256"),
                          (old,"manifest.json","expected_manifest_sha256"),
                          (obs,"manifest.json","expected_learning_observation_manifest_sha256"),
                          (runtime,"runtime-identity.json","expected_learning_runtime_identity_sha256")]:
        need(file_sha(root/leaf)==retained[key],"exact inherited failed learning authority")
    inventories["inherited_failed_learning"]=p.inventory(old)
    inventories["inherited_learning_observations"]=p.inventory(obs)
    failed=read_json(old/"report.json")
    need(failed["status"]=="FAILED" and failed["error"]=="guard Prefix trace missing",
         "specific execution failure, not model negative")
    oldcfg=read_json(old/"config.json")
    need(file_sha(old/"config.json")==retained["expected_learning_config_sha256"] and
         oldcfg["checkpoint"]==cfg["checkpoint"] and
         oldcfg["coupled_episode_learning"]["original_inputs"]==cfg["coupled_episode_learning"]["original_inputs"] and
         oldcfg["coupled_episode_learning"].get("retained_gradient") is None,
         "same original initializer/objective; old raw config separately authenticated")
    attempt=read_json(old/"attempt.json");launch=read_json(obs/"launch.json");execution=read_json(obs/"execution.json")
    identity=read_json(runtime/"runtime-identity.json")
    binding=read_json(old/"external-config-binding.json")
    need(identity["source_commit"]==retained["expected_learning_source_commit"] and
         identity["binary_sha256"]==retained["expected_learning_binary_sha256"] and
         file_sha(runtime/"geometric-frozen-map-fit")==retained["expected_learning_binary_sha256"] and
         launch["argv"]==attempt["argv"] and launch["pid"]==attempt["pid"] and
         launch["started_utc"]==execution["started_utc"] and execution["exit_code"]==1 and
         launch["binary_sha256"]==execution["binary_sha256"]==retained["expected_learning_binary_sha256"] and
         launch["config_sha256"]==execution["config_sha256"]==retained["expected_learning_config_sha256"] and
         binding["sha256"]==retained["expected_learning_config_sha256"] and binding["attempt_argv"]==attempt["argv"],
         "distinct original learning launch/config/source/binary")
    authority=read_json(run/"inherited-gradient-authority.json")
    need(report["inherited_learning"]==authority and authority["constructor_restart_from_original"] is True and
         authority["runtime_identity"]==identity and authority["recorded_launch"]==launch and
         authority["recorded_execution"]==execution and authority["completion_source_commit"]==args.expected_source and
         authority["completion_attempt"]==read_json(run/"attempt.json") and
         authority["completion_external_config_binding"]==read_json(run/"external-config-binding.json"),
         "new continuation identity does not replace inherited science")
    for field,key in [("root","retained_failed_root"),("source_commit","expected_learning_source_commit"),
                      ("report_sha256","expected_report_sha256"),("manifest_sha256","expected_manifest_sha256"),
                      ("config_sha256","expected_learning_config_sha256"),("binary_sha256","expected_learning_binary_sha256"),
                      ("gradient_receipt_sha256","expected_gradient_receipt_sha256"),("order_sha256","expected_coordinate_order_sha256"),
                      ("observations","retained_learning_observation_root"),
                      ("observation_manifest_sha256","expected_learning_observation_manifest_sha256")]:
        need(authority[field]==retained[key],"complete inherited authority field "+field)
    for leaf,key in [("coupled-gradient-receipt.json","expected_gradient_receipt_sha256"),
                     ("coupled-coordinate-order.json","expected_coordinate_order_sha256"),
                     ("coupled-forward-parity.json","expected_forward_parity_sha256")]:
        need(file_sha(old/leaf)==retained[key] and (old/leaf).read_bytes()==(run/leaf).read_bytes(),
             "completed original science byte copied "+leaf)
    gr=read_json(old/"coupled-gradient-receipt.json")
    expected={entry["file"] for term in gr["perterm"] for entry in term["families"]}
    expected.update(entry["file"] for entry in gr["files"])
    expected.update(["coupled-gradient-receipt.json","coupled-coordinate-order.json","coupled-forward-parity.json"])
    expected.update(x.name for x in old.glob("original-*.json"))
    extra={"inherited-partial-construction.json":old/"coupled-construction.json",
           "inherited-learning-report.json":old/"report.json","inherited-learning-manifest.json":old/"manifest.json",
           "inherited-learning-config.json":old/"config.json",
           "inherited-learning-external-config-binding.json":old/"external-config-binding.json",
           "inherited-learning-launch.json":obs/"launch.json","inherited-learning-execution.json":obs/"execution.json"}
    expected.update(extra)
    copied=authority["copied_scientific_files"]
    need(len(copied)==len(expected) and {x["file"] for x in copied}==expected,
         "complete unique inherited science inventory")
    for entry in copied:
        leaf=entry["file"];source=extra.get(leaf,old/leaf)
        need(pathlib.Path(entry["source_file"])==source and entry["bytes"]==source.stat().st_size==(run/leaf).stat().st_size and
             entry["sha256"]==file_sha(source)==file_sha(run/leaf),
             "all inherited original and current bytes preserved after helpers")
    partial=authority["original_partial"]
    need(partial=={"coordinate_records":retained["inherited_partial_coordinate_records"],
                   "alternatives":retained["inherited_partial_alternatives"],"commits":retained["inherited_partial_commits"],
                   "raw_journal_sha256":retained["expected_partial_journal_sha256"],"format":"INCOMPLETE_JSON",
                   "scope":"retained and charged owner-authenticated partial counters; not parsed as a resumable incumbent"} and
         file_sha(old/"coupled-construction.json")==retained["expected_partial_journal_sha256"],
         "partial original arithmetic retained separately, never parsed as resumable state")
    need(authority["raw_family_files"]==62 and authority["physical_backward_calls"]==31 and
         authority["new_training_graph_forwards"]==authority["new_backward_calls"]==authority["new_gradient_context_encoder_calls"]==0 and
         report["constructor_restart_from_original"] == bool(mode.get("retained_gradient")) and report["new_training_graph_forwards"]==report["new_backward_calls"]==0 and
         report["fresh_gradient_files"]==0 and report["logical_gradient_producer"]==retained["expected_learning_source_commit"] and
         report["prior_partial_alternatives_charged"]==retained["inherited_partial_alternatives"],
         "inherited31 backwards and fresh0/0; finite work separately counted")
    return {"status":"PASS_SAVED_PROVENANCE","learning_source":retained["expected_learning_source_commit"],
            "completion_source":args.expected_source,"copied_scientific_files":len(copied),
            "inherited_backward_calls":31,"new_backward_calls":0,"new_training_graph_forwards":0,
            "partial_ancestor":partial,"scope":"No old partial decision replay or backward replication."}

def audit_inherited_construction(run,cfg,report,args,p,inventories):
    retained=cfg["coupled_episode_learning"].get("retained_export")
    if retained is None:return None
    old=pathlib.Path(retained["retained_failed_root"])
    obs=pathlib.Path(retained["retained_observation_root"])
    runtime=pathlib.Path(retained["retained_runtime_root"])
    for root,leaf,key in [(old,"report.json","expected_report_sha256"),
                          (old,"manifest.json","expected_manifest_sha256"),
                          (old,"config.json","expected_config_sha256"),
                          (obs,"manifest.json","expected_observation_manifest_sha256"),
                          (runtime,"runtime-identity.json","expected_runtime_identity_sha256"),
                          (old,"coupled-construction.json","expected_complete_journal_sha256"),
                          (old,"final-objective.json","expected_final_objective_sha256")]:
        need(file_sha(root/leaf)==retained[key],"exact completed constructor authority "+leaf)
    inventories["completed_failed_constructor"]=p.inventory(old)
    inventories["completed_constructor_observations"]=p.inventory(obs)
    failed=read_json(old/"report.json")
    need(failed["status"]=="FAILED" and failed["error"]=='invalid reference request: Binding("continuation Generate dimensions/metadata")',
         "specific unfinished coherent-export boundary, not new finite search")
    oldcfg=read_json(old/"config.json");oldmode=oldcfg["coupled_episode_learning"]
    need(oldcfg["checkpoint"]==cfg["checkpoint"] and oldmode["original_inputs"]==cfg["coupled_episode_learning"]["original_inputs"] and
         oldmode.get("retained_export") is None and oldmode["retained_gradient"]==retained["inherited_gradients"],
         "completed constructor uses same original initializer and exact inherited learning")
    attempt=read_json(old/"attempt.json");launch=read_json(obs/"launch.json");execution=read_json(obs/"execution.json")
    identity=read_json(runtime/"runtime-identity.json");binding=read_json(old/"external-config-binding.json")
    need(identity["source_commit"]==retained["expected_source_commit"] and identity["config_sha256"]==retained["expected_config_sha256"] and
         identity["binary_sha256"]==retained["expected_binary_sha256"]==file_sha(runtime/"geometric-frozen-map-fit") and
         launch["argv"]==binding["attempt_argv"]==attempt["argv"] and launch["pid"]==attempt["pid"] and
         launch["started_utc"]==execution["started_utc"] and execution["exit_code"]==1 and
         launch["binary_sha256"]==execution["binary_sha256"]==retained["expected_binary_sha256"] and
         launch["config_sha256"]==execution["config_sha256"]==binding["sha256"]==retained["expected_config_sha256"],
         "completed constructor source/config/binary/observations separate from export completion")
    authority=read_json(run/"inherited-construction-authority.json")
    need(report["inherited_construction"]==authority and authority["runtime_identity"]==identity and
         authority["recorded_launch"]==launch and authority["recorded_execution"]==execution and
         authority["completion_source_commit"]==args.expected_source and
         authority["completion_attempt"]==read_json(run/"attempt.json") and
         authority["completion_external_config_binding"]==read_json(run/"external-config-binding.json"),
         "distinct newest completion identity")
    for field,key in [("root","retained_failed_root"),("report_sha256","expected_report_sha256"),
                      ("manifest_sha256","expected_manifest_sha256"),("source_commit","expected_source_commit"),
                      ("config_sha256","expected_config_sha256"),("binary_sha256","expected_binary_sha256"),
                      ("journal_sha256","expected_complete_journal_sha256"),("final_objective_sha256","expected_final_objective_sha256"),
                      ("final_prefix_master_sha256","expected_final_prefix_master_sha256"),
                      ("final_generate_master_sha256","expected_final_generate_master_sha256")]:
        need(authority[field]==retained[key],"complete constructor authority field "+field)
    renamed={"attempt.json","config.json","report.json","manifest.json","external-config-binding.json",
             "inherited-gradient-authority.json","coupled-pregradient-resource-projection.json","coupled-constructor-resource-projection.json"}
    expected={("completed-constructor-"+x.name if x.name in renamed else x.name):x for x in old.iterdir() if x.is_file()}
    expected["inherited-partial-checkpoint-receipt.json"]=old/"checkpoint-0001/receipt.json"
    copied=authority["copied_scientific_files"]
    need(len(copied)==len(expected) and {x["file"] for x in copied}==set(expected),
         "complete unique finished scientific bytecopy inventory")
    for entry in copied:
        source=expected[entry["file"]]
        need(pathlib.Path(entry["source_file"])==source and entry["bytes"]==source.stat().st_size==(run/entry["file"]).stat().st_size and
             entry["sha256"]==file_sha(source)==file_sha(run/entry["file"]),
             "completed science remains exact after normalization and final export")
    for family,leaf,key in [(PREFIX,"prefix/prefix-source-f32.bin","expected_final_prefix_master_sha256"),
                            (GENERATE,"generate-source/generate.unary.f32le","expected_final_generate_master_sha256")]:
        partial=old/"checkpoint-0001"/leaf
        need(file_sha(partial)==retained[key] and partial.read_bytes()==(run/"checkpoint-0001"/leaf).read_bytes(),
             "completed journal-selected actual masters byteequal to partial and final export")
    need(authority["inherited_constructor_calls"]==1 and authority["inherited_proposals"]==report["construction_summary"]["evaluated_alternatives"] and
         authority["new_constructor_calls"]==authority["new_proposals"]==authority["new_order_selection_calls"]==
         authority["new_training_graph_forwards"]==authority["new_backward_calls"]==0 and
         report["export_completion_only"] is True and report["new_constructor_calls"]==report["new_proposals"]==
         report["new_constructor_proposals"]==report["new_order_selection_calls"]==0 and report["final_cache_preparations"]==391,
         "zero new decisions;391 expected reconstruction and391 native separately")
    receipt=read_json(run/"checkpoint-0001/receipt.json")
    need(receipt["new_constructor_calls"]==receipt["new_proposals"]==receipt["new_order_selection_calls"]==0 and
         receipt["inherited_constructor_source_commit"]==retained["expected_source_commit"],
         "candidate receipt preserves constructor provenance and zero new search")
    return {"status":"PASS_SAVED_PROVENANCE","constructor_source":retained["expected_source_commit"],
            "export_source":args.expected_source,"inherited_proposals":authority["inherited_proposals"],
            "new_constructor_calls":0,"new_proposals":0,"final_cache_preparations":391,
            "copied_scientific_files":len(copied),"scope":"Full journal decision replay follows independently; no new production decisions."}

def audit_fresh_gradients(run, frames, original_master_paths):
    """Draft adapter for the written coupled-gradient-receipt schema.
    Must be rebound to final source before execution. No backward replication.
    """
    receipt = json.loads((run/"coupled-gradient-receipt.json").read_bytes())
    need(receipt["weighted_roles"] == 32 and receipt["physical_backward_calls"] == 31
         and receipt["prebackward_native_parity_graph_forwards"] == 31
         and receipt["gradient_graph_forwards"] == 31
         and receipt["extracted_families"] == [PREFIX,GENERATE]
         and receipt["context_gradient"] == "NOT_RUN"
         and len(frames) == len(receipt["perterm"]) == 31,
         "one31 loss/backward sequence, two extracted families and32 weighted roles")
    aggregates = {family:[0.]*960 for family in [PREFIX,GENERATE]}
    all_files = []
    for index,(frame,entry) in enumerate(zip(frames,receipt["perterm"])):
        row=frame["row"]
        need(entry["physical_index"] == index and
             entry["input_index"] == row["input_index"] and
             entry["position"] == row["position"] and
             entry["target"] == row["target_label_only"] and entry["weight"] == row["weight"],
             "raw physical/coalesced role identity")
        need([f["family"] for f in entry["families"]] == [PREFIX,GENERATE],
             "both actual family arrays per same physical backward")
        for f in entry["families"]:
            family=f["family"]
            name=f"coupled-gradient-{family}-term-{index:02}.f32le"
            need(f["file"] == name and f["shape"] == [960] and f["bytes"] == 3840
                 and f["status"] == "PRESENT" and f["missing_gradient_filled_zero"] is False,
                 "canonical raw family inventory with honest missing policy")
            raw,values=read_raw960(run/name)
            need(hashlib.sha256(raw).hexdigest() == f["sha256"],"actual perterm rawSHA")
            norm=math.sqrt(sum(x*x for x in values))
            need(f["all_zero"] == all(x == 0. for x in values)
                 and abs(f["l2_norm"]-norm) <= 1e-11*(1+norm),
                 "PRESENT zero distinct from MISSING, actual f32norm")
            aggregates[family]=[f32(a+x) for a,x in zip(aggregates[family],values)]
            need(all(math.isfinite(x) for x in aggregates[family]),"finite ordered aggregate")
            all_files.append(name)
    masters={}
    need(len(receipt["files"]) == 4,"two aggregate and two original master files")
    for family in [PREFIX,GENERATE]:
        for kind in ["gradient","initial-master"]:
            matches=[x for x in receipt["files"] if x["family"] == family and x["kind"] == kind]
            need(len(matches) == 1,"unique actual aggregate/master family entry")
            f=matches[0];name=f"coupled-{family}-{kind}.f32le"
            need(f["file"] == name and f["shape"] == [960] and f["bytes"] == 3840,
                 "canonical aggregate/master file and shape")
            raw,values=read_raw960(run/name)
            need(hashlib.sha256(raw).hexdigest() == f["sha256"],"actual aggregate/master SHA")
            if kind == "gradient":
                need(raw == struct.pack("<960f",*aggregates[family]),
                     "ordered31 f32 sum exactly equals saved family aggregate")
            else:
                original=pathlib.Path(original_master_paths[family]).read_bytes()
                need(raw == original,"entire original actual fractional master bytes")
                masters[family]=values
            all_files.append(name)
    actual={p.name for p in run.glob("coupled-*.f32le")}
    need(actual == set(all_files),"complete raw62+four aggregate/master file set")
    return masters,aggregates,receipt

def f32_bits(value):
    return struct.unpack("<I",struct.pack("<f",value))[0]

def f64_bits(value):
    return struct.unpack("<Q",struct.pack("<d",value))[0]

def coupled_objective(corpus, mapping, roles, stage=None):
    states=[r["state"] for r in corpus.rows]
    staged={}
    if stage is not None:
        if stage["family"] == PREFIX:
            for row,replacement in stage["replacements"].items():
                states[row]=replacement["state"]
        else:
            staged=stage["stages"]
    return corpus.kernel.objective(corpus.frames,states,mapping,roles,staged)

def audit_journal(journal, corpus, masters, gradients, mapping, roles):
    """Draft exact written construct() schema; final frozen-source binding pending."""
    ranked=mixed_coordinate_order(masters,gradients)
    records=journal["coordinate_records"]
    need(len(records) == 1920 and len(corpus.rows) == 391 and len(mapping) == 31,
         "complete coupled1920 order/391 population/31 physical objective frames")
    current=coupled_objective(corpus,mapping,roles)
    baseline=current
    final_masters={name:list(values) for name,values in masters.items()}
    counts={PREFIX:0,GENERATE:0}
    alternatives_count=0
    for order,(rank,record) in enumerate(zip(ranked,records)):
        family,index=rank["family"],rank["index"]
        need(record["order"] == order and record["family"] == family and record["index"] == index
             and record["incumbent_epoch"] == corpus.epoch,"frozen mixed coordinate/epoch")
        need(record["original_master_bits"] == f32_bits(masters[family][index])
             and record["gradient_bits"] == f32_bits(gradients[family][index])
             and record["priority_bits"] == f64_bits(rank["utility"])
             and record["rank_code"] == rank["ranking_code"],
             "exact f32 original/gradient and signed-zero f64 priority bits")
        expected_status=("all14" if family==GENERATE else
                         "zero_gradient" if gradients[family][index] == 0. else
                         "saturated" if rank["noop"] else "eligible")
        need(record["rank_status"] == expected_status,"explicit legal/noop rank status")
        codes=([rank["ranking_code"]] if family==PREFIX and not rank["noop"] else
               [] if family==PREFIX else
               [q for q in range(-7,8) if q != corpus.unary[index]])
        need([v["code"] for v in record["alternatives"]] == codes,"complete same-epoch legal alternatives")
        best=None
        for alternative,q in zip(record["alternatives"],codes):
            alternatives_count+=1
            before=corpus.prefix[index] if family==PREFIX else corpus.unary[index]
            stage=(corpus.stage_prefix(index,q) if family==PREFIX else
                   corpus.stage_unary(index,q,[f["row"]["target_label_only"] for f in corpus.frames]))
            next_objective=coupled_objective(corpus,mapping,roles,stage)
            corpus.kernel.compare_objective(alternative["objective"],next_objective)
            objective_gate=(current["combined"]-next_objective["combined"] >
                            1e-10*(1+abs(current["combined"])) and
                            next_objective["correct_reference_frames"] == 17)
            need(alternative["incumbent_epoch"] == corpus.epoch and
                 alternative["strict_current_CE_and17"] is objective_gate,
                 "same incumbent coupled epoch and strict current objective gate")
            changed=(stage["replacements"] if family==PREFIX else stage["stages"])
            affected=sorted(i for i in changed if i<380)
            need(alternative["affected_guard_indices"] == affected,"all incidence-affected380 guards")
            checked=[];failure=None
            if objective_gate:
                for row in affected:
                    checked.append(row)
                    target=corpus.frames[row]["row"]["target_label_only"]
                    chosen=corpus.summary_at(row,stage,target)["chosen_token_id"]
                    if chosen != target:
                        failure={"guard_index":row,"required":target,"chosen":chosen}
                        break
            status=("NOT_CHECKED_OBJECTIVE_GATE_FALSE" if not objective_gate else
                    "FIRST_VETO" if failure is not None else "FULL_PASS")
            feasible=objective_gate and failure is None
            need(alternative["checked_guard_indices"] == checked and
                 alternative["first_failure"] == failure and alternative["guard_status"] == status
                 and alternative["feasible"] is feasible,"ordered first veto vs whole380 pass")
            # Producer stages affected objective rows before its first-veto guard scan.
            staged_ids=sorted((set(mapping)&set(changed))|set(checked))
            summaries=[]
            for row in staged_ids:
                value=corpus.summary_at(row,stage,corpus.frames[row]["row"]["target_label_only"])
                summaries.append(dict(row=row,**{k:value[k] for k in
                    ["reference_q24","total_weight_q31","chosen_token_id","chosen_weight_q31","target_mass"]}))
            digest=integer_row_digest(summaries)
            need(alternative["staged_summary_digest"] == digest,"integer mathematical staged-row digest")
            if family==PREFIX:
                changes=alternative["changed_rows"]
                need([x[0] for x in changes] == staged_ids,"exact actually staged Prefix rows")
                for saved in changes:
                    need(isinstance(saved,list) and len(saved)==6,"explicit compact Prefix change array")
                    row,before_donor,after_donor,after_post,replaced,inc_digest=saved
                    old=corpus.rows[row];new=stage["replacements"][row]
                    need(before_donor == old["donor"] and after_donor == new["donor"]
                         and after_post == list(new["post"])
                         and replaced == (old["post"] != new["post"])
                         and inc_digest == new["incidence"].mathematical_digest(),
                         "BASE donor/current Generate post/dynamic row incidence")
            else:
                need(alternative["incumbent_code"] == before and
                     alternative["delta_code"] == q-before and
                     alternative["actual_master_delta_from_original"] == f32(q*.25)-masters[family][index],
                     "current code vs original fractional displacement")
                atoms=sum(len(row["incidence"].for_key(index)) for row in corpus.rows)
                need(alternative["changed_atom_count"] == atoms,"all4096 current-row incidence atoms")
            if feasible and (best is None or
                (next_objective["combined"],q)<(best[0]["combined"],best[1])):
                best=(next_objective,q,stage,digest)
        selected=record["selected"]
        if best is None:
            need(selected["status"] == ("noop" if family==PREFIX and rank["noop"] else "unchanged"),
                 "no feasible best retains actual fractional master bits")
            if family==PREFIX and rank["noop"]:
                need(selected["reason"] == expected_status,"explicit no-op reason")
        else:
            objective,q,stage,digest=best
            need(selected["status"] == "committed" and selected["code"] == q and
                 selected["staged_summary_digest"] == digest,"minimum feasible CE/signed-code selected")
            corpus.commit(stage)
            final_masters[family][index]=f32(q*.25)
            current=objective
            counts[family]+=1
        need(record["epoch_after"] == corpus.epoch,"transactional coupled epoch")
    summary=journal["summary"]
    need(summary["coordinates"] == 1920 and summary["maximum_alternatives"] == 14400 and
         summary["evaluated_alternatives"] == alternatives_count and
         summary["accepted_prefix"] == counts[PREFIX] and
         summary["accepted_generate"] == counts[GENERATE] and
         summary["accepted_epoch"] == corpus.epoch and summary["revisited"] == 0 and
         summary["selected_restage_count"] == 0,"complete frozen pass and committed state counts")
    corpus.kernel.compare_objective(summary["initial"],baseline)
    corpus.kernel.compare_objective(summary["final"],current)
    return final_masters,current,{"alternatives":alternatives_count,"accepted":counts,"epoch":corpus.epoch}

def read_json(path):
    return json.loads(pathlib.Path(path).read_bytes())

def audit_coupled_population(run, frames, pools, mapping, roles, corpus):
    saved=read_json(run/"coupled-population.json")
    need(saved["guards"] == 380 and saved["unique_union"] == 391 and
         saved["physical_frames"] == 31 and saved["weighted_roles"] == 32 and
         saved["objective_row_map"] == mapping and saved["roles"] == roles,
         "authenticated complete380 / unique391 / coalesced31 / weighted32 mapping")
    rows=[]
    for index,(frame,pool) in enumerate(zip(frames,pools)):
        rr=frame["row"]
        rows.append({"row":index,"input_index":rr["input_index"],"position":rr["position"],
                     "id":rr["id"],"actual_prefix_ids":rr["actual_prefix_ids"],
                     "target_label_only":rr["target_label_only"],"zero_weight_guard":index<380,
                     "guard_weight":0.,"coalesced_objective_weight":rr["weight"],
                     "donor":pool["donor"],"post_state":pool["post"]})
        need(corpus.rows[index]["donor"] == pool["donor"] and
             list(corpus.rows[index]["post"]) == pool["post"] and
             list(corpus.rows[index]["state"].scores) == pool["generate"] and
             list(corpus.rows[index]["state"].copy_scores) == pool["copy"],
             "all391 original native vectors/BASE donors before construction")
    need(saved["rows"] == rows,"rowwise original required winners and prefix authority")
    original_guard_authority=saved["guard_population"]
    # Full source guard provenance already authenticated by immutable corpus loader.
    need(original_guard_authority["guards"] == 380,"full selected guard authority count")
    incidence=read_json(run/"coupled-incidence.json")
    need(incidence["legal_generate_ids"] == list(range(N)) and
         incidence["total_postings"] == 391*N*8 and
         incidence["row_incidence_digests"] ==
            [row["incidence"].mathematical_digest() for row in corpus.rows],
         "legal4096/all391 dynamic-key initial incidence")
    return saved

def audit_coupled_export(cp, original, masters, base, p, inherited=None):
    need(base.directory(cp/"native") == base.directory(original/"native") and
         base.directory(cp/"source") == base.directory(original/"source") and
         base.directory(cp/"cue") == base.directory(original/"cue"),
         "entire frozen Source/native/Cue payloads")
    for folder in ["cue-source"]:
        if (original/folder).exists():
            need(base.directory(cp/folder) == base.directory(original/folder),
                 "frozen Cue floating sidecar")
    for leaf in ["read-state-bridge.bin","read-state-bridge-categorical.bin"]:
        need((cp/leaf).read_bytes() == (original/leaf).read_bytes(),"frozen source bridge")
    base.frozen_raw(original/"read-state-bridge-source",cp/"read-state-bridge-source")
    base.frozen_raw(original/"continuation-source",cp/"continuation-source")
    original_g=read_json(original/"generate.bin");candidate_g=read_json(cp/"generate.bin")
    expected=json.loads(json.dumps(original_g["payload"]))
    packed=expected["energy"]["unary_packed"][:]
    q=[code(m) for m in masters[GENERATE]]
    for lane in range(8):
        for k in range(120):
            i=lane*128+k;shift=(i%2)*4
            packed[i//2]=(packed[i//2]&~(15<<shift))|((q[lane*120+k]&15)<<shift)
    expected["energy"]["unary_packed"]=packed
    need(candidate_g["payload"] == expected,"whole Generate payload differs ONLY selected960 unary codes")
    gm=dict(original_g["metadata"]);gm["payload_sha256"]=candidate_g["metadata"]["payload_sha256"]
    need(candidate_g["metadata"] == gm,"fixed Generate geometry/protocol/score_shift20")
    om=read_json(original/"generate-source/metadata.json");cm=read_json(cp/"generate-source/metadata.json")
    need(set(om["parameters"]) == set(cm["parameters"]) ==
         {GENERATE,"generate.pair","generate.bias","generate.prototype_choices"},
         "exact actual Generate family inventory")
    for family,info in cm["parameters"].items():
        raw=(cp/"generate-source"/(family+".f32le")).read_bytes()
        need(len(raw) == info["bytes"] and hashlib.sha256(raw).hexdigest() == info["sha256"],
             "actual floating Generate parameter identity")
        if family == GENERATE:
            need(info["shape"] == [8,120] and raw == struct.pack("<960f",*masters[family]),
                 "all960 selected/untouched original-fractional unary master bits")
        else:
            need(info == om["parameters"][family] and
                 raw == (original/"generate-source"/(family+".f32le")).read_bytes(),
                 "frozen actual Generate pair/bias/prototype master bits")
    em=dict(om);em["parameters"]=cm["parameters"]
    need(cm == em,"all floating Generate metadata outside active unary identity frozen")
    raw=(cp/"prefix/prefix-source-f32.bin").read_bytes()
    need(raw == struct.pack("<960f",*masters[PREFIX]),"all960 selected/untouched fractional Prefix master bits")
    prefix_codes=[code(m) for m in masters[PREFIX]]
    packed_prefix=bytearray(480)
    for i,c in enumerate(prefix_codes):
        packed_prefix[i//2]|=(c&15)<<((i%2)*4)
    need((cp/"prefix/prefix-q4.bin").read_bytes() == packed_prefix,"exact legal quarter Prefix packing")
    original_pn=read_json(original/"prefix/native-metadata.json")
    candidate_pn=read_json(cp/"prefix/native-metadata.json")
    expected_pn=dict(original_pn)
    expected_pn["potential_packed_sha256"]=file_sha(cp/"prefix/prefix-q4.bin")
    need(candidate_pn == expected_pn and
         candidate_pn["frozen_cue"] == read_json(cp/"cue/native-metadata.json"),
         "Prefix immutable source/geometry/frozen-Cue binding with only packed digest changed")
    original_pm=read_json(original/"prefix/metadata.json");candidate_pm=read_json(cp/"prefix/metadata.json")
    expected_pm=dict(original_pm)
    expected_pm["source_sha256"]=hashlib.sha256(raw).hexdigest()
    expected_pm["packed_sha256"]=file_sha(cp/"prefix/prefix-q4.bin")
    expected_pm["native_metadata"]=candidate_pn
    need(candidate_pm == expected_pm,"whole Prefix floating metadata updates only master/packed/native identities")
    orig_u=read_json(original/"continuation-field.bin");cand_u=read_json(cp/"continuation-field.bin")
    expected_u=json.loads(json.dumps(orig_u))
    expected_u["metadata"]["generate_sha256"]=file_sha(cp/"generate.bin")
    expected_u["metadata"]["generate_metadata"]=candidate_g["metadata"]
    need(cand_u == expected_u,"whole frozen U numerical artifact with only current Generate binding")
    receipt=read_json(cp/"receipt.json")
    need(receipt == read_json(cp/"continuation-source/metadata.json") and
         receipt["mode"] == "coupled_episode_learning" and
         receipt["active_parameter_names"] == [PREFIX,GENERATE] and
         receipt["fresh_adam"] is False and receipt["optimizer_updates"] == 0 and
         receipt["new_gradients"] == (0 if inherited else 1) and receipt["coefficient_backward_calls"] == 31 and
         receipt["extracted_family_gradients"] == 2 and receipt["candidate_native_steps"] == 391 and
         receipt["expected_finite_pool_reductions"] == 391 and
         receipt["native_independently_reloaded"] is True and
         receipt["generate_sha256"] == file_sha(cp/"generate.bin") and
         receipt["prefix_sha256"] == file_sha(cp/"prefix/prefix-q4.bin"),
         "exact coupled export/reload receipt; in-RAM restore is producer witness")
    if inherited:
        need(receipt["new_backward_calls"]==receipt["new_training_graph_forwards"]==0 and
             receipt["inherited_learning_source_commit"]==inherited["learning_source"],
             "export inherited31/fresh0 counters separately")
    return receipt

def audit_coupled_snapshots(run, cp, corpus, e, base):
    Arithmetic=e.build_arithmetic(base)
    final=Arithmetic(cp)
    expected_files={f"candidate-row-{f['row']['input_index']:04}-position-{f['row']['position']:02}.json"
                    for f in corpus.frames}
    need({x.name for x in run.glob("candidate-row-*-position-*.json")} == expected_files and
         len(expected_files) == 391,"exact unique391 final snapshot file set")
    for frame,row in zip(corpus.frames,corpus.rows):
        rr=frame["row"]
        saved=read_json(run/f"candidate-row-{rr['input_index']:04}-position-{rr['position']:02}.json")
        need((run/f"candidate-row-{rr['input_index']:04}-position-{rr['position']:02}.json").stat().st_size <= 535837+16384,
             "final snapshot within admitted serialized envelope")
        identity={k:v for k,v in saved.items() if k != "native"}
        need(identity == {k:v for k,v in rr.items() if not k.startswith("_")},
             "final391 identity, conditional prefix, target-label and coalesced weight")
        ff,pool=final.snapshot_checked(identity,saved["native"])
        state=row["state"]
        need(pool["generate"] == list(state.scores) and
             pool["copy"] == list(state.copy_scores) and
             pool["post"] == list(row["post"]) and pool["donor"] == row["donor"] and
             ff["base"] == list(row["base_copy"]) and
             ff["bank"] == frame["bank"] and ff["keys"] == frame["keys"] and
             ff["ids"] == frame["ids"] and ff["u"] == frame["u"] and
             ff["u_witness"] == frame["u_witness"],
             "complete final source states/Prefix bins/current donor/G/U/full aliases")
        for k,v in [("reference_q24",pool["summary"]["max_score_q24"]),
                    ("total_weight_q31",pool["summary"]["total_weight_q31"]),
                    ("chosen_token_id",pool["summary"]["chosen_token_id"]),
                    ("chosen_weight_q31",pool["summary"]["chosen_weight_q31"])]:
            need(state.current[k] == v,"sparse/full allalias final pool parity")
        del saved,ff,pool
    return {"native_snapshots":391,"complete_snapshot_pool_validations":final.snapshot_validations,
            "full_saved_factor_pool_reconstructions":final.reductions}

RETAINED_EPISODE = pathlib.Path("/workspace/uor-r4/codex/sol-episode-progression-learning/runs/episode-0001-attempt1")
RETAINED_REPORT = "54ffd2ae5c81b00e0474aa8e815d82af75cb0cccf36efc6c8a24b1755f4258b6"
RETAINED_SEAL = "3e0c502da4952b177e9485e18303b88df33d36e08d27c99a405619fc8c077df2"
PACKET = pathlib.Path("/workspace/uor-r4/codex/sol-coupled-episode-learning/implementation-packet.json")
PACKET_SHA = "350e3c24845936a635831c55e8af20a90bcfdfb49bb15d1edf781b2a9a6fa316"
EXPECTED_SOURCE = "a6be8f699eef3841a1a4dd7a51faefc06d83dd7c"
ADAPTER_READY = True

def audit_capacity(cap, max_copy, legal):
    row_cache=3*N*8+4*max_copy*8+8*N+2048
    row_incidence=961*8+8*legal*2+1024
    pending=max(N*32,3*N*8)+2048
    incumbent=391*(row_cache+row_incidence)
    prefix_stage=incumbent
    generate_stage=2*391*pending
    vectors=391*N*16
    cache=incumbent+max(prefix_stage,generate_stage+vectors)+(16<<20)+(8<<20)
    expected={"row_cache_bytes":row_cache,"row_incidence_bytes":row_incidence,
              "pending_row_bytes":pending,"persistent391_cache_incidence":incumbent,
              "prefix_all391_replacement_stage":prefix_stage,
              "generate_best_and_current_stage":generate_stage,
              "immutable_post_cache_cap":16<<20,"containers_margin":8<<20,
              "all391_changed_atom_vector_bytes":vectors,
              "cache_upper_bound":cache,"cache_cap":256<<20,
              "max_copy_aliases":max_copy,"legal_generate_ids":legal}
    need(cap == expected and cache <= 256<<20,"all persistent/best/current/replacement/vector capacity components")
    return cache

def audit_coupled_resources(run,cfg,frames,original_inputs):
    pre=read_json(run/"coupled-pregradient-resource-projection.json")
    post=read_json(run/"coupled-constructor-resource-projection.json")
    old_root=pathlib.Path(original_inputs["episode"]["retained_projection"]["root"])
    old=read_json(old_root/"trajectory-resource-projection.json")
    oldpre=read_json(old_root/"resource-projection.json")
    need(pre["original_resource_receipt"] == old and
         pre["original_pregradient_receipt"] == oldpre,"authenticated retained complete resource authority")
    max_copy=max(512,max(len(f["ids"]) for f in frames))
    cache=audit_capacity(pre["capacity"],max_copy,N)
    mode=cfg["coupled_episode_learning"]
    export=mode.get("retained_export")
    retained=bool(export or mode.get("retained_gradient"))
    stage="BEFORE_RETAINED_EXPORT_ADMISSION" if export else "BEFORE_RETAINED_GRADIENT_ADMISSION_AND_FINITE_RESTART" if retained else "BEFORE_ANY_BACKWARD"
    need(pre["stage"] == stage and
         pre["fresh_backward_calls_completed"] == 0 and
         pre["role_count"] == 32 and pre["physical_graph_count"] == 31 and
         pre["episode_length"] == 15 and pre["fresh_family_gradient_bytes"] == (0 if retained else 2*32*3840) and
         pre["retained_family_gradient_bytes"] == (2*32*3840 if retained else 0),
         "resource admission precedes any fresh backward")
    full=pre["actual31_full_serialized_native_bytes"]
    need(isinstance(full,int) and full>0,"producer Rust-serialized full31 size witness; no float JSON recomputation")
    typed=old["typed_guard_frame_bound"]
    copy_peak=0
    if export:
        original_journal=pathlib.Path(export["retained_failed_root"])/"coupled-construction.json"
        need(original_journal.is_file() and file_sha(original_journal)==export["expected_complete_journal_sha256"],
             "authenticated complete original journal resource identity")
        copy_peak=original_journal.stat().st_size+(4<<20)
    need(pre["retained_export_copy_peak_bytes"]==copy_peak,"exact journal-plus4MiB copy/typed-decoder peak")
    numeric=typed+cache+full*2+(32<<20)+copy_peak
    need(oldpre["process_ram_cap"] == 4<<30,"retained phase process cap authority")
    process=max(2489696,1540696)*1024+(512<<20)+full*8+(128<<20)+copy_peak
    journal=pre["streamed_journal_reserve"]
    report=(535837+16384)*391+journal+57780353+(48<<20)+(32<<20)
    need(pre["typed_guard_frame_bound"] == typed and pre["numeric_upper_bound"] == numeric and
         pre["numeric_cap"] == 512<<20 and pre["process_ram_projection_bytes"] == process and
         pre["process_ram_cap"] == 4<<30 and pre["report_upper_bound"] == report and
         pre["report_cap"] == cfg["maximum_report_bytes"] and
         pre["native391_snapshot_max_measured_bytes"] == 535837 and
         numeric <= 512<<20 and process <= 4<<30 and report+(1<<20)<cfg["maximum_report_bytes"] and
         (run/"coupled-construction.json").stat().st_size <= journal,
         "exact integer complete pregradient formulas and actual journal under serialized bound")
    need(post["stage"] == ("BEFORE_FINAL391_EXPECTED_POOL_RECONSTRUCTION_AND_NATIVE_RELOAD" if cfg["coupled_episode_learning"].get("retained_export") else "AFTER_INHERITED31_BACKWARDS_BEFORE_FINITE_RESTART" if cfg["coupled_episode_learning"].get("retained_gradient") else "AFTER31_BACKWARDS_BEFORE_CONSTRUCTOR") and
         post["model_graph_and_prepared_device_tensors_dropped"] is True and
         post["all380_authority_complete"] is True and
         post["actual_max_copy_aliases"] == max(len(f["ids"]) for f in frames) <= 512,
         "construction allocation after graph teardown and actual380 alias coverage")
    audit_capacity(post["capacity"],512,N)
    return {"numeric_upper_bound":numeric,"report_upper_bound":report,
            "process_ram_projection_bytes":process,"retained_export_copy_peak_bytes":copy_peak,"actual_journal_bytes":(run/"coupled-construction.json").stat().st_size,
            "scope":"Integer capacity formulas independently reconstructed. Rust native JSON sizes and worst-case serialized journal formatter bound remain source-reviewed producer receipts."}

def audit_initial_rich(saved,baseline):
    for k in ["combined","task","reference"]:
        need(abs(saved[k]-baseline[k])<=1e-12*(1+abs(baseline[k])),"original32-role component CE")
    need(len(saved["phases"]) == 15 and saved["correct_reference_frames"] == 17,
         "complete original fifteen phase witnesses/seventeen references")
    for a,b in zip(saved["phases"],baseline["phases"]):
        need(a["position"] == b["position"] and a["target"] == b["target"] and
             a["target_mass"] == b["target_mass"] and a["total_mass"] == b["total_mass"] and
             a["pool"]["chosen_token_id"] == b["chosen"],"initial native phase masses and winners")

def run_adapter(args):
    import resource,time,collections
    start=time.monotonic()
    kernel=authenticated_kernel()
    e,base,p=kernel.immutable_helpers()
    need(file_sha(PACKET) == PACKET_SHA,"final reviewed frozen producer packet")
    need(file_sha(args.config) == args.config_sha256 and
         file_sha(args.binary) == args.binary_sha256,"external raw config and immutable binary SHA")
    run=args.run;cfg=read_json(args.config)
    need(pathlib.Path(cfg["out"]) == run and read_json(run/"config.json") == cfg,
         "semantic config equality; pinned submitted raw bytes remain independent")
    attempt=read_json(run/"attempt.json");execution=read_json(args.execution)
    launch=read_json(args.execution.with_name("launch.json"))
    need(attempt["schema"] == "uor-r4.report-attempt/1" and len(attempt["argv"]) == 2 and
         pathlib.Path(attempt["argv"][1]) == args.config and launch["argv"] == attempt["argv"] and
         launch["binary"] == attempt["argv"][0] and launch["pid"] == attempt["pid"],
         "actual launch path/argv/PID; immutable executable relocation supported")
    need(launch["binary_sha256"] == execution["binary_sha256"] == args.binary_sha256 and
         launch["config_sha256"] == execution["config_sha256"] == args.config_sha256 and
         launch["started_utc"] == execution["started_utc"] and execution["exit_code"] == 0,
         "actual launch/execution/source-config identity")
    need(read_json(run/"external-config-binding.json") ==
         {"path":str(args.config.resolve()),"sha256":args.config_sha256,
          "bytes":args.config.stat().st_size,"attempt_argv":attempt["argv"]},
         "exact submitted config byte receipt")
    report=read_json(run/"report.json")
    need(report["schema"] == "uor-r4.coupled-episode-report/1" and
         report["status"] == "COMPLETED" and report["mode"] == "coupled_episode_learning" and
         report["source_commit"] == args.expected_source,"actual exact completed coupled producer")
    c=cfg["coupled_episode_learning"];original_inputs=c["original_inputs"]
    need(file_sha(RETAINED_EPISODE/"report.json") == RETAINED_REPORT and
         file_sha(RETAINED_EPISODE/"manifest.json") == RETAINED_SEAL and
         read_json(RETAINED_EPISODE/"config.json")["prefix_fragment_learning"] == original_inputs,
         "unchanged original complete episode inputs and fixed selected initializer")
    need(cfg["credit"] == "raw_identity" and cfg["updates"] == 1 and
         cfg["loss_scope"] == "all" and cfg["mode"] == "joint_continuation",
         "fixed hard anchored scalar objective and no optimizer sweep")
    inventories={"model":p.inventory(run),"retained_original_episode":p.inventory(RETAINED_EPISODE)}
    parent=pathlib.Path(original_inputs["retained_intermediate_root"]);cp0=parent/"checkpoint-0001"
    need(pathlib.Path(cfg["checkpoint"]) == cp0 and file_sha(cp0/"native/metadata.json") == e.SOURCE_SHA and
         file_sha(cp0/"generate.bin") == e.G_SHA and file_sha(cp0/"continuation-field.bin") == e.U_SHA,
         "original selected Source/G/U; no negative checkpoint initializer")
    need(file_sha(pathlib.Path(cfg["training_inputs"])) ==
         "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a" and
         file_sha(pathlib.Path(cfg["training_labels"])) ==
         "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155",
         "frozen authored panel data")
    roots=[(parent,base.P_REPORT,base.P_SEAL),
           (pathlib.Path(original_inputs["retained_probe_root"]),base.B_REPORT,base.B_SEAL),
           (pathlib.Path(original_inputs["episode"]["retained_supplement_root"]),e.SUP_REPORT,e.SUP_SEAL)]
    roots.extend((pathlib.Path(w["capture"]["root"]),w["capture"]["expected_report_sha256"],
                  w["capture"]["expected_manifest_sha256"]) for w in original_inputs["episode"]["phases"])
    rp=original_inputs["episode"]["retained_projection"]
    roots.append((pathlib.Path(rp["root"]),rp["expected_report_sha256"],rp["expected_manifest_sha256"]))
    for root,rh,mh in roots:
        need(file_sha(root/"report.json") == rh and file_sha(root/"manifest.json") == mh,
             "original witness report/seal bytes")
        if str(root) not in inventories:
            inventories[str(root)]=p.inventory(root)
    inherited=audit_inherited_learning(run,cfg,report,args,p,inventories)
    inherited_construction=audit_inherited_construction(run,cfg,report,args,p,inventories)
    frames,pools,mapping,roles,objective_frames,original,ar=kernel.load_original_corpus(
        RETAINED_EPISODE,original_inputs,e,base)
    masters,gradients,gradient_receipt=audit_fresh_gradients(
        run,objective_frames,{PREFIX:original/"prefix/prefix-source-f32.bin",
                             GENERATE:original/"generate-source/generate.unary.f32le"})
    prefix_initial=[code(m) for m in masters[PREFIX]]
    unary_initial=[code(m) for m in masters[GENERATE]]
    need(prefix_initial == p.codes(original)[p.FAMILIES[1]] and unary_initial ==
         [p.nib(ar.a.e["unary_packed"],lane*128+k) for lane in range(8) for k in range(120)],
         "original actual fractional masters and native packed quantization")
    incidence=read_json(run/"coupled-incidence.json")
    need(incidence["legal_generate_ids"] == list(range(N)),"authenticate4096 admission before kernel")
    def token_keys(post):
        return array.array("H",(lane*120+ar.a.rel(post[lane],ar.a.p["prototypes"][t*8+lane])
                               for t in range(N) for lane in range(8)))
    def bridge(frame,donor):
        bank=frame["bank"]
        return ar.a.bridge(bank["context"]["states"][-1],
                           bank["context"]["states"][bank["candidates"][donor]["context_position"]])[0]
    weights=kernel.IntegerWeights(ar.exp)
    corpus=CoupledPools(kernel,frames,prefix_initial,unary_initial,weights,
                       ar.a.generate,token_keys,bridge,incidence["legal_generate_ids"])
    population=audit_coupled_population(run,frames,pools,mapping,roles,corpus)
    baseline=coupled_objective(corpus,mapping,roles)
    kernel.compare_objective(report["baseline_objective"],baseline)
    audit_initial_rich(read_json(run/"initial-original-objective.json"),baseline)
    need(abs(gradient_receipt["weighted_graph_ce"]-baseline["combined"])<=1e-5,
         "native hard-anchored weighted graph CE; no independent backward claim")
    parity=read_json(run/"coupled-forward-parity.json")
    need(parity == {"all_before_any_backward":True,"physical_frames":31,
                    "raw_G_Copy_U_donor_post_full_alias_pool":True,"device":"cuda",
                    "captured_objective_encoder_calls":0,"gradient_context_backward_calls":0},
         "all factual31 hard forward parity before any backward producer receipt")
    projection=audit_coupled_resources(run,cfg,frames,original_inputs)
    journal=read_json(run/"coupled-construction.json")
    need(read_json(run/"coupled-coordinate-order.json") ==
         [{"family":r["family"],"index":r["index"],"gradient":gradients[r["family"]][r["index"]],
           "original_master":masters[r["family"]][r["index"]],"rank_code":r["ranking_code"],
           "original_code":r["initial_code"],"actual_master_delta":r["ranking_actual_delta"],
           "priority":r["utility"],"status":("all14" if r["family"]==GENERATE else
             "zero_gradient" if gradients[r["family"]][r["index"]]==0. else
             "saturated" if r["noop"] else "eligible")}
          for r in mixed_coordinate_order(masters,gradients)],
         "entire frozen1920 coordinate manifest")
    ar.cache.clear();ar.a.generate_cache.clear()
    final_masters,current,counters=audit_journal(journal,corpus,masters,gradients,mapping,roles)
    del journal
    kernel.compare_objective(report["candidate_objective"],current)
    kernel.compare_objective(read_json(run/"final-objective.json"),current)
    guards=all(r["state"].current["chosen_token_id"] == frames[i]["row"]["target_label_only"]
               for i,r in enumerate(corpus.rows[:380]))
    gate=kernel.final_gate(baseline,current,guards)
    need(report["final_gate"] == gate and report["finite_episode_positive"] == gate["passed"] and
         report["all_original380_preserved"] is True and guards,"final original15/17/380 gate")
    fg=read_json(run/"final-trajectory-guards.json")
    need(fg["guards"] == 380 and fg["all_original_winners"] is True and len(fg["terms"]) == 380,
         "complete final380 retention receipt")
    for i,term in enumerate(fg["terms"]):
        rr=frames[i]["row"];state=corpus.rows[i]["state"]
        need(term["guard_index"] == i and term["input_index"] == rr["input_index"] and
             term["position"] == rr["position"] and term["required_original_winner"] ==
             term["chosen"] == rr["target_label_only"],"rowwise required original380 winners")
        for k in ["reference_q24","total_weight_q31","chosen_token_id","chosen_weight_q31"]:
            need(term["pool"][k] == state.current[k],"full final380 pooled summaries")
    cp=run/"checkpoint-0001"
    exported=audit_coupled_export(cp,original,final_masters,base,p,inherited)
    need(report["candidate_receipt"] == exported,"coherent exported/report receipt")
    snapshots=audit_coupled_snapshots(run,cp,corpus,e,base)
    need(report["selected_model"] is False and report["useful_candidate"] is False and
         report["weighted_roles"] == 32 and report["unique_objective_frames"] == 31 and
         report["physical_backward_calls"] == 31 and report["extracted_family_gradients"] == 2 and
         report["optimizer_updates"] == 0 and report["candidate_native_steps"] == 391 and
         report["expected_final_pool_reductions"] == 391 and report["full512"] == "NOT_RUN" and
         report["parent_master_bits_restored"] is True,
         "actual bounded coupled scope; RAM restore remains producer witness")
    if gate["passed"]:
        need(args.qualification is not None and args.qualification_report_sha256 and
             args.qualification_manifest_sha256,"positive conditional gate requires separate actual9")
        qualification=kernel.audit_actual_qualification(args,run,parent,cp,p,cfg,report)
    else:
        need(args.qualification is None,"negative gate has no undeclared autoregressive rollout")
        qualification={"status":"NOT_RUN","reason":"complete conditional episode construction negative"}
    rss=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss*1024
    need(rss <= 4<<30,"4GiB saved audit process ceiling")
    return {"schema":"uor-r4.coupled-episode-independent-audit/1","status":"PASS",
            "source":args.expected_source,"report_sha256":file_sha(run/"report.json"),
            "reader_sha256":file_sha(pathlib.Path(__file__)),"immutable_kernel_sha256":AUTHORITY_SHA,
            "raw_gradients":62,"physical_backwards":31,"roles":32,"coordinates":1920,
            "Generate_alternatives":13440,"protected_population":380,"native_union":391,
            "constructor":counters,"snapshots":snapshots,"projection":projection,
            "complete_inventories":inventories,"inherited_learning":inherited,"inherited_construction":inherited_construction,"final_gate":gate,"actual_qualification":qualification,
            "elapsed_seconds":time.monotonic()-start,"peak_rss_bytes":rss,
            "scope":"Independent saved integer factor/Prefix physical incidence/current donor-current unary/allalias pool arithmetic and raw62/frozen1920/all14 choice/380/391. No encoder/backward replication, autoregressive execution, new gradient/proposal or global feasibility/capacity proof. Rust complete BLAKE3 verification separate; operational graph/restore fields are producer receipts."}

def main():
    import argparse
    parser=argparse.ArgumentParser()
    parser.add_argument("run",type=pathlib.Path)
    for name in ["config","binary","execution"]:
        parser.add_argument("--"+name,type=pathlib.Path,required=True)
    for name in ["config-sha256","binary-sha256","expected-source"]:
        parser.add_argument("--"+name,required=True)
    parser.add_argument("--qualification",type=pathlib.Path)
    parser.add_argument("--qualification-report-sha256")
    parser.add_argument("--qualification-manifest-sha256")
    args=parser.parse_args()
    need(ADAPTER_READY and PACKET_SHA is not None and EXPECTED_SOURCE is not None,
         "NOT_READY: final producer/source packet and peer review not frozen; no audit admitted")
    need(len(args.expected_source)==40 and all(c in "0123456789abcdef" for c in args.expected_source),
         "actual exact producer source hash before input load")
    need(args.expected_source == EXPECTED_SOURCE,"reviewed exact producer source/packet binding")
    print(json.dumps(run_adapter(args)))

if __name__ == "__main__":
    main()
