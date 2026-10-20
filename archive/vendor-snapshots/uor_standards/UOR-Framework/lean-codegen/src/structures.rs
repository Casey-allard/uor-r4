//! Lean 4 structure generator.
//!
//! Generates a single `UOR/Structures.lean` file containing every
//! non-enum OWL class as a `structure` declaration. Structures are
//! emitted in topological order of the direct dependency graph
//! (Tarjan's SCC + Kahn ordering on the condensation). Singleton SCCs
//! go inside reopenable `namespace UOR.<Space>.<Module>` blocks.
//! Non-trivial SCCs go inside a synthetic
//! `namespace UOR.Mutual.Cluster<N>` block containing a `mutual ... end`
//! block, with `abbrev` re-exports in their original namespaces so the
//! user-facing paths are preserved.

use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::fmt::Write as FmtWrite;

use uor_ontology::model::iris::*;
use uor_ontology::model::{Class, Ontology, Property, PropertyKind};

use crate::emit::{normalize_lean_comment, LeanFile};
use crate::enums::enum_class_names;
use crate::mapping::{
    lean_qualified_name, local_name, to_lean_field_name, xsd_to_lean_type, LeanNamespaceMapping,
};

/// Emission scope for a structure: either emitted inside a singleton
/// namespace block, or inside a synthetic mutual-cluster namespace.
///
/// Used by `resolve_object_type` and `build_extends_clause` to decide
/// whether a referenced IRI should be rendered as a bare local name
/// (same scope) or as a fully-qualified `UOR.<Space>.<Module>.Name` path.
pub(crate) enum EmissionScope<'a> {
    /// Singleton emission: the structure lives in `ns_iri`'s namespace.
    /// Bare names are used for same-namespace references.
    Namespace(&'a str),
    /// Mutual-cluster emission: the structure lives in a synthetic
    /// cluster namespace. Bare names are used only for SCC-internal
    /// references (other members of the same cluster).
    MutualCluster(&'a HashSet<&'a str>),
}

impl<'a> EmissionScope<'a> {
    /// Returns true iff `iri` is reachable by a bare local name in the
    /// current emission scope.
    pub(crate) fn is_local(&self, iri: &str) -> bool {
        match self {
            EmissionScope::Namespace(ns) => iri.starts_with(*ns),
            EmissionScope::MutualCluster(set) => set.contains(iri),
        }
    }
}

/// A class collected for single-file emission.
pub(crate) struct ClassEntry<'a> {
    pub(crate) class: &'a Class,
    pub(crate) ns_iri: &'a str,
    pub(crate) space_module: &'static str,
    pub(crate) file_module: &'static str,
    pub(crate) class_local: &'a str,
    pub(crate) full_ns: String,
}

impl<'a> ClassEntry<'a> {
    fn stable_key(&self) -> (&'static str, &'static str, &'a str) {
        (self.space_module, self.file_module, self.class_local)
    }
}

/// One declared field of a generated structure, shared between the
/// structure emitter (`generate_structure`) and the individual emitter
/// in `individuals.rs`. Keeping a single source of truth for the field
/// list guarantees typed individual struct literals use the same names
/// and order that the structure declaration produced, so Lean's
/// type-checker accepts them.
pub(crate) struct StructureField<'a> {
    /// Lean field name, already keyword-escaped via `to_lean_field_name`.
    pub(crate) name: String,
    /// Source OWL property this field is generated from.
    pub(crate) property: &'a Property,
    /// Resolved Lean type expression (e.g. `"P.NonNegativeInteger"`,
    /// `"Option (Operation P)"`, `"UOR.Kernel.Schema.TermExpression P"`).
    pub(crate) lean_type: String,
}

/// Returns the ordered list of a class's **own** (non-inherited) fields,
/// in the canonical emission order. Every call site — structure
/// emission and individual emission — must use this helper so the
/// field list stays consistent.
pub(crate) fn compute_structure_fields<'a>(
    class: &'a Class,
    ns_iri: &'a str,
    all_props_by_domain: &'a HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
    ns_map: &HashMap<&'a str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&'a str>,
) -> Vec<StructureField<'a>> {
    let class_local = local_name(class.id);
    // Synthetic entry; only `class.id` and `ns_iri` are read.
    let entry = ClassEntry {
        class,
        ns_iri,
        space_module: "",
        file_module: "",
        class_local,
        full_ns: String::new(),
    };
    let owner_props = owned_non_annotation_props(&entry, all_props_by_domain);
    let inherited =
        collect_inherited_fields(class, all_props_by_domain, all_classes_by_iri, skip_classes);

    let mut out = Vec::new();
    for prop in owner_props {
        let field_name = to_lean_field_name(local_name(prop.id));
        if inherited.contains(&field_name) {
            continue;
        }
        let lean_type = resolve_lean_type(prop, class_local, ns_map, scope, skip_classes);
        out.push(StructureField {
            name: field_name,
            property: prop,
            lean_type,
        });
    }
    out
}

/// Returns the non-annotation object/datatype properties whose domain is
/// `entry.class.id` AND that are defined in the same namespace as `entry`
/// (matching the original per-namespace codegen rule: cross-namespace
/// domain properties are not generated).
fn owned_non_annotation_props<'a>(
    entry: &ClassEntry,
    all_props_by_domain: &'a HashMap<&str, Vec<&'a Property>>,
) -> Vec<&'a Property> {
    all_props_by_domain
        .get(entry.class.id)
        .map(|v| {
            v.iter()
                .copied()
                .filter(|p| p.kind != PropertyKind::Annotation && p.id.starts_with(entry.ns_iri))
                .collect()
        })
        .unwrap_or_default()
}

/// Collects all non-enum OWL classes across every namespace, in the
/// ontology's canonical order, into `ClassEntry` records.
fn collect_class_entries<'a>(
    ontology: &'a Ontology,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    skip_classes: &HashSet<&str>,
) -> Vec<ClassEntry<'a>> {
    let mut out = Vec::new();
    for module in &ontology.namespaces {
        let ns_iri = module.namespace.iri;
        let mapping = match ns_map.get(ns_iri) {
            Some(m) => m,
            None => continue,
        };
        for class in &module.classes {
            let class_local = local_name(class.id);
            if skip_classes.contains(class_local) {
                continue;
            }
            out.push(ClassEntry {
                class,
                ns_iri,
                space_module: mapping.space_module,
                file_module: mapping.file_module,
                class_local,
                full_ns: format!("UOR.{}.{}", mapping.space_module, mapping.file_module),
            });
        }
    }
    out
}

/// Builds a direct dependency graph over non-enum classes.
///
/// Edge A -> B iff A's emitted declaration references B by structure name:
///   1. `A extends B` via `class.subclass_of`
///   2. A has a declared non-annotation object property with range == B
///      (only properties whose field is actually emitted in A's body —
///      inherited fields are filtered out)
///
/// Excluded:
///   - Self-edges (self-references are handled by `Option`-wrapping
///     in `resolve_object_type`).
///   - Enum class ranges/parents (they are `inductive`, not structures).
///   - `owl:Thing`, `owl:Class`, `rdf:List` (these map to `P.String`).
///
/// Fields of type `Option (B P)` or `Array (B P)` still contribute an
/// edge to B — Lean 4 requires B to be defined or co-defined in the
/// same `mutual` block even when it appears under a wrapper.
fn build_dependency_graph(
    entries: &[ClassEntry],
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &Class>,
    skip_classes: &HashSet<&str>,
) -> Vec<Vec<usize>> {
    let iri_to_idx: HashMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.class.id, i))
        .collect();

    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); entries.len()];

    for (a_idx, entry) in entries.iter().enumerate() {
        let mut deps: BTreeSet<usize> = BTreeSet::new();

        // (1) extends edges
        for parent_iri in entry.class.subclass_of {
            if *parent_iri == OWL_THING {
                continue;
            }
            let parent_local = local_name(parent_iri);
            if skip_classes.contains(parent_local) {
                continue;
            }
            if let Some(&b_idx) = iri_to_idx.get(parent_iri) {
                if a_idx != b_idx {
                    deps.insert(b_idx);
                }
            }
        }

        // (2) field range edges — only declared, non-inherited,
        //     same-defining-namespace object properties.
        let inherited = collect_inherited_fields(
            entry.class,
            all_props_by_domain,
            all_classes_by_iri,
            skip_classes,
        );
        for prop in owned_non_annotation_props(entry, all_props_by_domain) {
            if prop.kind != PropertyKind::Object {
                continue;
            }
            let field_name = to_lean_field_name(local_name(prop.id));
            if inherited.contains(&field_name) {
                continue;
            }
            if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
                continue;
            }
            let range_local = local_name(prop.range);
            if skip_classes.contains(range_local) {
                continue;
            }
            if let Some(&b_idx) = iri_to_idx.get(prop.range) {
                if a_idx != b_idx {
                    deps.insert(b_idx);
                }
            }
        }

        graph[a_idx] = deps.into_iter().collect();
    }
    graph
}

/// Iterative Tarjan's strongly connected components.
///
/// Input: adjacency list `graph`, where `graph[i]` is the out-neighbors of node i.
/// Output: `Vec<Vec<usize>>` where each inner vector is one SCC. The outer
/// order is Tarjan's natural reverse-topological order (sinks first).
///
/// Deterministic for deterministic input — iteration over neighbors follows
/// the adjacency list order.
pub(crate) fn tarjan_scc(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = graph.len();
    let mut index = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut result: Vec<Vec<usize>> = Vec::new();
    let mut next_index = 0usize;

    enum Frame {
        Enter(usize),
        Resume(usize, usize),
    }

    let mut work: Vec<Frame> = Vec::new();

    for start in 0..n {
        if index[start] != usize::MAX {
            continue;
        }
        work.push(Frame::Enter(start));
        while let Some(frame) = work.pop() {
            match frame {
                Frame::Enter(v) => {
                    index[v] = next_index;
                    lowlink[v] = next_index;
                    next_index += 1;
                    stack.push(v);
                    on_stack[v] = true;
                    work.push(Frame::Resume(v, 0));
                }
                Frame::Resume(v, k) => {
                    if k < graph[v].len() {
                        let w = graph[v][k];
                        // Schedule continuation for the next neighbor.
                        work.push(Frame::Resume(v, k + 1));
                        if index[w] == usize::MAX {
                            work.push(Frame::Enter(w));
                        } else if on_stack[w] {
                            lowlink[v] = lowlink[v].min(index[w]);
                        }
                        // Lowlink propagation from tree-children is handled
                        // in the finalization branch below (final k == len)
                        // by re-scanning all on-stack neighbors.
                    } else {
                        // All neighbors processed; propagate lowlink from
                        // every still-on-stack neighbor (covers tree-child
                        // and cross/back edges uniformly).
                        for &w in &graph[v] {
                            if on_stack[w] {
                                lowlink[v] = lowlink[v].min(lowlink[w]);
                            }
                        }
                        if lowlink[v] == index[v] {
                            // v is the root of an SCC; pop until v.
                            let mut scc = Vec::new();
                            while let Some(w) = stack.pop() {
                                on_stack[w] = false;
                                scc.push(w);
                                if w == v {
                                    break;
                                }
                            }
                            result.push(scc);
                        }
                    }
                }
            }
        }
    }
    result
}

/// Given SCCs (in any order) and the original graph, produces a
/// deterministic forward topological order of SCC indices using Kahn's
/// algorithm with a min-heap keyed on the minimum `stable_key` of each
/// SCC. Ties between SCCs with equal minimum keys are broken by the
/// SCC index, guaranteeing a total order.
fn order_sccs_deterministically(
    sccs: &[Vec<usize>],
    graph: &[Vec<usize>],
    entries: &[ClassEntry],
) -> Vec<usize> {
    order_sccs_by_key(sccs, graph, |node_idx| {
        let k = entries[node_idx].stable_key();
        (k.0.to_string(), k.1.to_string(), k.2.to_string())
    })
}

/// Generic Kahn-ordering over SCCs using a caller-supplied stable
/// key. Edge direction is flipped relative to the dependency graph so
/// that Kahn processes **dependencies before dependents**. Each SCC's
/// position in the output is determined by its *minimum* stable key,
/// so callers get a deterministic total order as long as the key
/// function is itself deterministic.
///
/// Type parameters:
/// - `K`: key type. Must be `Ord + Clone` to work with the min-heap.
/// - `F`: closure `|node_idx| -> K`.
pub(crate) fn order_sccs_by_key<K, F>(
    sccs: &[Vec<usize>],
    graph: &[Vec<usize>],
    key_of: F,
) -> Vec<usize>
where
    K: Ord + Clone,
    F: Fn(usize) -> K,
{
    let num_sccs = sccs.len();

    // node_to_scc[i] = index of the SCC containing node i.
    let mut node_to_scc = vec![0usize; graph.len()];
    for (scc_idx, scc) in sccs.iter().enumerate() {
        for &node in scc {
            node_to_scc[node] = scc_idx;
        }
    }

    // Condensation graph — see `order_sccs_deterministically` for the
    // edge-direction rationale.
    let mut cond_out: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_sccs];
    let mut in_degree = vec![0usize; num_sccs];
    for (u, neighbors) in graph.iter().enumerate() {
        let su = node_to_scc[u];
        for &v in neighbors {
            let sv = node_to_scc[v];
            if su != sv && cond_out[sv].insert(su) {
                in_degree[su] += 1;
            }
        }
    }

    // Minimum stable key per SCC for deterministic tie-breaking.
    let min_key: Vec<K> = sccs
        .iter()
        .map(|scc| {
            let mut best: Option<K> = None;
            for &node in scc {
                let k = key_of(node);
                best = Some(match best {
                    None => k,
                    Some(cur) => {
                        if k < cur {
                            k
                        } else {
                            cur
                        }
                    }
                });
            }
            best.unwrap_or_else(|| {
                // SCCs are non-empty by Tarjan's definition; the
                // fallback is defensive and unused in practice.
                key_of(0)
            })
        })
        .collect();

    // Min-heap over (min_key, scc_idx). Rust's BinaryHeap is a max-heap,
    // so we wrap in `std::cmp::Reverse` to flip ordering.
    use std::cmp::Reverse;
    let mut heap: BinaryHeap<Reverse<(K, usize)>> = BinaryHeap::new();
    for scc_idx in 0..num_sccs {
        if in_degree[scc_idx] == 0 {
            heap.push(Reverse((min_key[scc_idx].clone(), scc_idx)));
        }
    }

    let mut order: Vec<usize> = Vec::with_capacity(num_sccs);
    while let Some(Reverse((_, scc_idx))) = heap.pop() {
        order.push(scc_idx);
        for &next_scc in &cond_out[scc_idx] {
            in_degree[next_scc] -= 1;
            if in_degree[next_scc] == 0 {
                heap.push(Reverse((min_key[next_scc].clone(), next_scc)));
            }
        }
    }
    order
}

/// Generates the complete content of `UOR/Structures.lean`.
///
/// Returns `(content, structure_count, field_count)`.
pub fn generate_all_structures(
    ontology: &Ontology,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &Class>,
) -> (String, usize, usize) {
    let skip_classes: HashSet<&str> = enum_class_names().iter().copied().collect();

    // 1. Collect all non-enum classes.
    let entries = collect_class_entries(ontology, ns_map, &skip_classes);

    // 2. Build dependency graph.
    let graph = build_dependency_graph(
        &entries,
        all_props_by_domain,
        all_classes_by_iri,
        &skip_classes,
    );

    // 3. Tarjan + Kahn deterministic ordering.
    let sccs = tarjan_scc(&graph);
    let order = order_sccs_deterministically(&sccs, &graph, &entries);

    // 3a. Compute the set of class IRIs that CANNOT receive an
    //     `Inhabited (<Name> UOR.Prims.Standard)` instance. The base
    //     set is every class in a non-trivial (size >= 2) SCC — those
    //     are the mutual cluster members with cyclic fields that
    //     cannot be constructed as finite data. The set is then closed
    //     transitively: any class that extends a blocked class or has
    //     a non-Option/non-Array field whose range is a blocked class
    //     is itself blocked. Individuals typed with blocked classes
    //     will be reported as unproven in Phase 6.
    let inhabited_blocked = compute_inhabited_blocked(
        &entries,
        &sccs,
        all_props_by_domain,
        all_classes_by_iri,
        &skip_classes,
    );

    // 4. Emit the file.
    let mut f = LeanFile::new(
        "UOR Foundation \u{2014} all structure declarations (single compilation unit).",
    );
    f.line("import UOR.Primitives");
    f.line("import UOR.Enums");
    f.blank();
    f.line("open UOR.Primitives");
    f.blank();

    let mut structure_count = 0usize;
    let mut field_count = 0usize;
    let mut current_open_ns: Option<String> = None;
    let mut cluster_counter: usize = 0;

    for scc_idx in order {
        let scc = &sccs[scc_idx];

        if scc.len() >= 2 {
            // Non-trivial SCC: emit inside a synthetic mutual-cluster namespace.
            if let Some(ns) = current_open_ns.take() {
                let _ = writeln!(f.buf, "end {ns}");
                f.blank();
            }
            emit_mutual_cluster(
                &mut f,
                scc,
                &entries,
                cluster_counter,
                all_props_by_domain,
                all_classes_by_iri,
                ns_map,
                &skip_classes,
                &inhabited_blocked,
                &mut structure_count,
                &mut field_count,
            );
            cluster_counter += 1;
        } else {
            // Singleton SCC: emit inside the target namespace, reopening
            // as necessary.
            let idx = scc[0];
            let entry = &entries[idx];
            let target_ns = entry.full_ns.clone();
            if current_open_ns.as_deref() != Some(target_ns.as_str()) {
                if let Some(ns) = current_open_ns.take() {
                    let _ = writeln!(f.buf, "end {ns}");
                    f.blank();
                }
                let _ = writeln!(f.buf, "namespace {target_ns}");
                f.blank();
                current_open_ns = Some(target_ns);
            }

            let scope = EmissionScope::Namespace(entry.ns_iri);
            let fc = generate_structure(
                &mut f,
                entry.class,
                entry.ns_iri,
                all_props_by_domain,
                all_classes_by_iri,
                ns_map,
                &scope,
                &skip_classes,
            );
            structure_count += 1;
            field_count += fc;

            // Per-Standard Inhabited instance, emitted only if the
            // class is not in the blocked set. Blocked classes are
            // cluster members or transitively depend on cluster
            // members and cannot have a finite default.
            if !inhabited_blocked.contains(entry.class.id) {
                emit_inhabited_instance(
                    &mut f,
                    entry.class,
                    entry.ns_iri,
                    all_props_by_domain,
                    all_classes_by_iri,
                    ns_map,
                    &scope,
                    &skip_classes,
                );
            }
        }
    }

    if let Some(ns) = current_open_ns.take() {
        let _ = writeln!(f.buf, "end {ns}");
    }

    (f.finish(), structure_count, field_count)
}

/// Emits a non-trivial SCC as a synthetic `UOR.Mutual.Cluster<N>`
/// namespace containing a `mutual ... end` block, followed by `abbrev`
/// re-exports in each member's original namespace and — for cluster
/// members not in the `inhabited_blocked` set — per-Standard Inhabited
/// instances.
#[allow(clippy::too_many_arguments)]
fn emit_mutual_cluster(
    f: &mut LeanFile,
    scc: &[usize],
    entries: &[ClassEntry],
    cluster_idx: usize,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &Class>,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    skip_classes: &HashSet<&str>,
    inhabited_blocked: &HashSet<&str>,
    structure_count: &mut usize,
    field_count: &mut usize,
) {
    let cluster_ns = format!("UOR.Mutual.Cluster{cluster_idx}");

    // Sort members deterministically.
    let mut members: Vec<usize> = scc.to_vec();
    members.sort_by(|&a, &b| entries[a].stable_key().cmp(&entries[b].stable_key()));

    // Set of IRIs that are in-cluster — these get bare names inside the
    // mutual block (resolved via the synthetic namespace).
    let in_cluster: HashSet<&str> = members.iter().map(|&i| entries[i].class.id).collect();
    let scope = EmissionScope::MutualCluster(&in_cluster);

    let _ = writeln!(f.buf, "namespace {cluster_ns}");
    f.blank();
    f.line("mutual");

    for &i in &members {
        let entry = &entries[i];
        let fc = generate_structure(
            f,
            entry.class,
            entry.ns_iri,
            all_props_by_domain,
            all_classes_by_iri,
            ns_map,
            &scope,
            skip_classes,
        );
        *structure_count += 1;
        *field_count += fc;
    }

    f.line("end");
    f.blank();
    let _ = writeln!(f.buf, "end {cluster_ns}");
    f.blank();

    // Abbrev re-exports so users reference the types via their
    // original namespace paths (e.g., `UOR.Kernel.Op.ComposedOperation`).
    for &i in &members {
        let entry = &entries[i];
        let _ = writeln!(f.buf, "namespace {}", entry.full_ns);
        let _ = writeln!(
            f.buf,
            "abbrev {} := {}.{}",
            entry.class_local, cluster_ns, entry.class_local
        );
        let _ = writeln!(f.buf, "end {}", entry.full_ns);
        f.blank();
    }

    // Per-Standard Inhabited instances for cluster members NOT in the
    // blocked set. A "soft" cluster (one where the SCC exists only
    // because of Option-wrapped fields) can still have Inhabited
    // instances: Lean resolves the `default` expression through the
    // Option wrapper and `none`. Instances are emitted inside the
    // cluster namespace so the bare type name resolves.
    let emit_any = members
        .iter()
        .any(|&i| !inhabited_blocked.contains(entries[i].class.id));
    if emit_any {
        let _ = writeln!(f.buf, "namespace {cluster_ns}");
        f.blank();
        let scope_in_cluster = EmissionScope::MutualCluster(&in_cluster);
        for &i in &members {
            let entry = &entries[i];
            if inhabited_blocked.contains(entry.class.id) {
                continue;
            }
            emit_inhabited_instance(
                f,
                entry.class,
                entry.ns_iri,
                all_props_by_domain,
                all_classes_by_iri,
                ns_map,
                &scope_in_cluster,
                skip_classes,
            );
        }
        let _ = writeln!(f.buf, "end {cluster_ns}");
        f.blank();
    }
}

/// Computes the set of class IRIs that cannot receive an Inhabited
/// instance at the `UOR.Prims.Standard` instantiation.
///
/// Base set: every class participating in a non-trivial SCC of the
/// **hard** dependency graph — edges restricted to required+functional
/// object properties (plus extends chains). Non-required functional
/// fields wrap in `Option`, which is always Inhabited (`none`), so they
/// don't create hard dependencies. Non-functional fields wrap in `Array`,
/// also always Inhabited (`#[]`).
///
/// Closure: iteratively add any class that (a) transitively extends a
/// blocked class, or (b) has a required+functional field whose range
/// IRI is already blocked. Iterates to a fixed point.
pub(crate) fn compute_inhabited_blocked<'a>(
    entries: &[ClassEntry<'a>],
    _sccs: &[Vec<usize>],
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &'a Class>,
    skip_classes: &HashSet<&str>,
) -> HashSet<&'a str> {
    let mut blocked: HashSet<&'a str> = HashSet::new();

    // Build the hard-dependency graph: required+functional object
    // property edges + extends edges.
    let iri_to_idx: HashMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.class.id, i))
        .collect();
    let mut hard_graph: Vec<Vec<usize>> = vec![Vec::new(); entries.len()];
    for (a_idx, entry) in entries.iter().enumerate() {
        let mut deps: BTreeSet<usize> = BTreeSet::new();
        // Extends edges are always hard.
        for parent_iri in entry.class.subclass_of {
            if *parent_iri == OWL_THING {
                continue;
            }
            let parent_local = local_name(parent_iri);
            if skip_classes.contains(parent_local) {
                continue;
            }
            if let Some(&b_idx) = iri_to_idx.get(parent_iri) {
                if a_idx != b_idx {
                    deps.insert(b_idx);
                }
            }
        }
        // Required+functional object property edges are hard.
        if let Some(props) = all_props_by_domain.get(entry.class.id) {
            for prop in props {
                if prop.kind != PropertyKind::Object {
                    continue;
                }
                if !prop.id.starts_with(entry.ns_iri) {
                    continue;
                }
                if !prop.functional || !prop.required {
                    continue;
                }
                if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
                    continue;
                }
                let range_local = local_name(prop.range);
                if skip_classes.contains(range_local) {
                    continue;
                }
                if let Some(&b_idx) = iri_to_idx.get(prop.range) {
                    if a_idx != b_idx {
                        deps.insert(b_idx);
                    }
                }
            }
        }
        hard_graph[a_idx] = deps.into_iter().collect();
    }

    // Base: all classes in non-trivial SCCs of the hard graph. Those
    // are the truly-cyclic structures with no Inhabited escape.
    let hard_sccs = tarjan_scc(&hard_graph);
    for scc in &hard_sccs {
        if scc.len() >= 2 {
            for &idx in scc {
                blocked.insert(entries[idx].class.id);
            }
        }
    }

    // Fixed-point closure.
    loop {
        let mut added_any = false;
        for entry in entries {
            if blocked.contains(entry.class.id) {
                continue;
            }
            // (a) extends chain hits a blocked class.
            if entry.class.subclass_of.iter().any(|p| blocked.contains(*p)) {
                blocked.insert(entry.class.id);
                added_any = true;
                continue;
            }
            // (b) has a required, functional field whose range is
            //     blocked. Only required+functional object fields emit
            //     as bare structure types; non-required ones wrap in
            //     `Option`, which is always Inhabited regardless of
            //     the target's blockedness. Non-functional fields wrap
            //     in `Array`, same story.
            if let Some(props) = all_props_by_domain.get(entry.class.id) {
                for prop in props {
                    if prop.kind != PropertyKind::Object {
                        continue;
                    }
                    if !prop.id.starts_with(entry.ns_iri) {
                        continue;
                    }
                    if !prop.functional {
                        continue; // Array → always Inhabited
                    }
                    if !prop.required {
                        continue; // Option → always Inhabited
                    }
                    if prop.range == entry.class.id {
                        continue; // self-ref → Option wrapped in resolve_object_type
                    }
                    let range_local = local_name(prop.range);
                    if skip_classes.contains(range_local) {
                        continue;
                    }
                    if blocked.contains(prop.range) {
                        blocked.insert(entry.class.id);
                        added_any = true;
                        break;
                    }
                }
            }
        }
        if !added_any {
            break;
        }
    }

    // Transitively also block subclasses via the extends chain (one
    // more pass after the field-based closure, covering cases where a
    // class extends something that only got blocked by a field-based
    // rule in the previous iteration).
    loop {
        let mut added_any = false;
        for entry in entries {
            if blocked.contains(entry.class.id) {
                continue;
            }
            for parent in entry.class.subclass_of {
                if blocked.contains(*parent) {
                    blocked.insert(entry.class.id);
                    added_any = true;
                    break;
                }
            }
            // Also check if any required, functional field references
            // a newly-blocked class.
            if !blocked.contains(entry.class.id) {
                if let Some(_guard) = all_classes_by_iri.get(entry.class.id) {
                    if let Some(props) = all_props_by_domain.get(entry.class.id) {
                        for prop in props {
                            if prop.kind != PropertyKind::Object || !prop.functional {
                                continue;
                            }
                            if !prop.required {
                                continue;
                            }
                            if !prop.id.starts_with(entry.ns_iri) {
                                continue;
                            }
                            if prop.range == entry.class.id {
                                continue;
                            }
                            if blocked.contains(prop.range) {
                                blocked.insert(entry.class.id);
                                added_any = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !added_any {
            break;
        }
    }

    blocked
}

/// Computes the inhabited-blocked set for the full ontology. Public
/// wrapper for use by `individuals.rs`, which needs to know whether a
/// given structure type has an `Inhabited` instance available to
/// discharge default values in typed individual emission.
pub fn compute_inhabited_blocked_for_ontology<'a>(
    ontology: &'a Ontology,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    all_props_by_domain: &HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
) -> HashSet<&'a str> {
    let skip_classes: HashSet<&str> = enum_class_names().iter().copied().collect();
    let entries = collect_class_entries(ontology, ns_map, &skip_classes);
    let graph = build_dependency_graph(
        &entries,
        all_props_by_domain,
        all_classes_by_iri,
        &skip_classes,
    );
    let sccs = tarjan_scc(&graph);
    compute_inhabited_blocked(
        &entries,
        &sccs,
        all_props_by_domain,
        all_classes_by_iri,
        &skip_classes,
    )
}

/// Returns a concrete Lean expression of a canonical default value for
/// a field whose Lean type is `lean_type`. Unlike the generic `default`
/// keyword, these literals sidestep Lean's inability to synthesize
/// `Inhabited` for `P.String`-style field-projection types by producing
/// values at the concrete `UOR.Prims.Standard` instantiation.
pub(crate) fn default_expr_for_type(lean_type: &str) -> String {
    // Wrapper types get priority: `Option _` → `none`; `Array _` → `#[]`.
    if lean_type.starts_with("Option ") {
        return "none".to_string();
    }
    if lean_type.starts_with("Array ") {
        return "#[]".to_string();
    }
    match lean_type {
        "P.String" => "(\"\" : String)".to_string(),
        "P.Integer" => "(0 : Int)".to_string(),
        "P.NonNegativeInteger" => "(0 : Nat)".to_string(),
        "P.PositiveInteger" => "(1 : Nat)".to_string(),
        "P.Decimal" => "(0.0 : Float)".to_string(),
        "P.Boolean" => "false".to_string(),
        // Anything else (enum type or structure type) uses Lean's
        // `default`, which requires an `Inhabited` instance to exist
        // on the concrete type. Enums derive `Inhabited`; structures
        // get their per-`Standard` instance emitted by
        // `emit_inhabited_instance`, in topological order, so the
        // instance is already in scope at every use site.
        _ => "default".to_string(),
    }
}

/// Emits an `instance : Inhabited (<Name> UOR.Prims.Standard)` block for
/// a structure. The body populates every field (own + inherited via the
/// `extends` chain) with a concrete default expression from
/// `default_expr_for_type`. Assumes every referenced non-primitive
/// field type already has an `Inhabited` instance in scope — the
/// caller must emit instances in topological order.
#[allow(clippy::too_many_arguments)]
fn emit_inhabited_instance<'a>(
    f: &mut LeanFile,
    class: &'a Class,
    ns_iri: &'a str,
    all_props_by_domain: &'a HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
    ns_map: &HashMap<&'a str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&'a str>,
) {
    let class_local = local_name(class.id);

    // Collect every field the typed instance must populate: own fields
    // from `compute_structure_fields` plus all inherited fields via the
    // `extends` chain. Lean's `extends` requires the struct literal to
    // include inherited fields because the child's default must define
    // them (the parent's `default` instance isn't accessible via
    // struct-literal shorthand alone).
    let mut all_fields = compute_structure_fields(
        class,
        ns_iri,
        all_props_by_domain,
        all_classes_by_iri,
        ns_map,
        scope,
        skip_classes,
    );
    let inherited_fields = collect_all_inherited_structure_fields(
        class,
        ns_iri,
        all_props_by_domain,
        all_classes_by_iri,
        ns_map,
        scope,
        skip_classes,
    );
    all_fields.extend(inherited_fields);

    if all_fields.is_empty() {
        let _ = writeln!(
            f.buf,
            "instance : Inhabited ({class_local} UOR.Prims.Standard) where"
        );
        let _ = writeln!(f.buf, "  default := {{}}");
    } else {
        let _ = writeln!(
            f.buf,
            "instance : Inhabited ({class_local} UOR.Prims.Standard) where"
        );
        f.line("  default := {");
        for field in &all_fields {
            let expr = default_expr_for_type(&field.lean_type);
            let _ = writeln!(f.buf, "    {} := {expr}", field.name);
        }
        f.line("  }");
    }
    f.blank();
}

/// Public-facing version of `collect_all_inherited_structure_fields`
/// for use by `individuals.rs`'s typed-instance emission.
pub(crate) fn collect_all_inherited_structure_fields_public<'a>(
    class: &'a Class,
    ns_iri: &'a str,
    all_props_by_domain: &'a HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
    ns_map: &HashMap<&'a str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&'a str>,
) -> Vec<StructureField<'a>> {
    collect_all_inherited_structure_fields(
        class,
        ns_iri,
        all_props_by_domain,
        all_classes_by_iri,
        ns_map,
        scope,
        skip_classes,
    )
}

/// Collects inherited fields from the `extends` chain as a full
/// `Vec<StructureField>` rather than the name-only set returned by
/// `collect_inherited_fields`. Needed for struct-literal emission where
/// we must supply every parent field.
#[allow(clippy::too_many_arguments)]
fn collect_all_inherited_structure_fields<'a>(
    class: &'a Class,
    _ns_iri: &'a str,
    all_props_by_domain: &'a HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
    ns_map: &HashMap<&'a str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&'a str>,
) -> Vec<StructureField<'a>> {
    let mut result: Vec<StructureField<'a>> = Vec::new();
    let mut seen_field_names: HashSet<String> = HashSet::new();
    let mut stack: Vec<&'a str> = class.subclass_of.to_vec();
    let mut visited: HashSet<&'a str> = HashSet::new();

    while let Some(parent_iri) = stack.pop() {
        if parent_iri == OWL_THING {
            continue;
        }
        let parent_local = local_name(parent_iri);
        if skip_classes.contains(parent_local) {
            continue;
        }
        if !visited.insert(parent_iri) {
            continue;
        }
        let Some(parent_class) = all_classes_by_iri.get(parent_iri) else {
            continue;
        };
        // Determine the parent's owning namespace IRI by finding its
        // prefix in ns_map.
        let parent_ns_iri = ns_map
            .keys()
            .copied()
            .find(|ns| parent_iri.starts_with(*ns))
            .unwrap_or("");

        let parent_fields = compute_structure_fields(
            parent_class,
            parent_ns_iri,
            all_props_by_domain,
            all_classes_by_iri,
            ns_map,
            scope,
            skip_classes,
        );
        for field in parent_fields {
            if seen_field_names.insert(field.name.clone()) {
                result.push(field);
            }
        }
        // Recurse into grandparents.
        for gp in parent_class.subclass_of {
            stack.push(*gp);
        }
    }
    result
}

/// Generates a single `structure` declaration. Returns the number of
/// fields emitted.
#[allow(clippy::too_many_arguments)]
fn generate_structure<'a>(
    f: &mut LeanFile,
    class: &'a Class,
    ns_iri: &'a str,
    all_props_by_domain: &'a HashMap<&'a str, Vec<&'a Property>>,
    all_classes_by_iri: &HashMap<&'a str, &'a Class>,
    ns_map: &HashMap<&'a str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&'a str>,
) -> usize {
    let class_local = local_name(class.id);
    let comment = normalize_lean_comment(class.comment);

    f.doc_comment(&comment);

    // Build extends clause.
    let extends = build_extends_clause(class, ns_map, scope, skip_classes);
    let extends_str = if extends.is_empty() {
        String::new()
    } else {
        format!(" extends {}", extends.join(", "))
    };

    // Shared field introspection: the struct literal emitter in
    // individuals.rs uses the same helper, guaranteeing consistent
    // field names and order.
    let fields = compute_structure_fields(
        class,
        ns_iri,
        all_props_by_domain,
        all_classes_by_iri,
        ns_map,
        scope,
        skip_classes,
    );

    if fields.is_empty() {
        let _ = writeln!(
            f.buf,
            "structure {class_local} (P : Primitives){extends_str}"
        );
    } else {
        let _ = writeln!(
            f.buf,
            "structure {class_local} (P : Primitives){extends_str} where"
        );
        for field in &fields {
            let comment = normalize_lean_comment(field.property.comment);
            f.indented_doc_comment(&comment);
            let _ = writeln!(f.buf, "  {} : {}", field.name, field.lean_type);
        }
    }
    f.blank();

    fields.len()
}

/// Resolves the Lean type expression for a property.
fn resolve_lean_type(
    prop: &Property,
    owner_class: &str,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&str>,
) -> String {
    match prop.kind {
        PropertyKind::Datatype => resolve_datatype(prop),
        PropertyKind::Object => resolve_object_type(prop, owner_class, ns_map, scope, skip_classes),
        PropertyKind::Annotation => "P.String".to_string(),
    }
}

/// Resolves a datatype property to its Lean type.
///
/// Functional properties marked `required: false` are wrapped in
/// `Option` so a missing assertion becomes `none` (proven-by-absence)
/// rather than a hard conformance gap. Non-functional properties
/// always emit as `Array T`, which is already Inhabited via `#[]`.
fn resolve_datatype(prop: &Property) -> String {
    let base = xsd_to_lean_type(prop.range).unwrap_or("P.String");
    if prop.functional {
        if prop.required {
            base.to_string()
        } else {
            format!("Option {base}")
        }
    } else {
        format!("Array {base}")
    }
}

/// Resolves an object property to its Lean type.
fn resolve_object_type(
    prop: &Property,
    owner_class: &str,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&str>,
) -> String {
    let range_local = local_name(prop.range);

    // Enum range → return the enum type directly (always in scope via
    // `UOR.Enums`). Non-required functional enum fields wrap in
    // `Option` so a missing assertion is proven-by-absence.
    if skip_classes.contains(range_local) {
        return if prop.functional {
            if prop.required {
                range_local.to_string()
            } else {
                format!("Option {range_local}")
            }
        } else {
            format!("Array {range_local}")
        };
    }

    // Generic OWL/RDF placeholders map to `P.String`.
    if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
        return if prop.functional {
            if prop.required {
                "P.String".to_string()
            } else {
                "Option P.String".to_string()
            }
        } else {
            "Array P.String".to_string()
        };
    }

    // Self-reference detection: compare local names of the class being
    // emitted vs. the property range.
    let is_self_ref = range_local == owner_class;

    // Resolve the structure type expression.
    let struct_type = if scope.is_local(prop.range) {
        format!("{range_local} P")
    } else {
        match lean_qualified_name(prop.range, ns_map) {
            Some(qualified) => format!("{qualified} P"),
            None => format!("{range_local} P"),
        }
    };

    if is_self_ref && prop.functional {
        // Self-referential functional → wrap in `Option` to break recursion.
        format!("Option ({struct_type})")
    } else if prop.functional {
        // Functional structure field: wrap in `Option` when the
        // property is not required. Lean's `Option` is Inhabited
        // (`none`), so missing assertions resolve as proven-by-absence.
        if prop.required {
            struct_type
        } else {
            format!("Option ({struct_type})")
        }
    } else {
        format!("Array ({struct_type})")
    }
}

/// Builds the `extends` clause for a class, as a vector of parent type
/// expressions (e.g., `["Parent1 P", "UOR.User.Type_.Parent2 P"]`).
fn build_extends_clause(
    class: &Class,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
    scope: &EmissionScope<'_>,
    skip_classes: &HashSet<&str>,
) -> Vec<String> {
    let mut parents = Vec::new();

    for parent_iri in class.subclass_of {
        if *parent_iri == OWL_THING {
            continue;
        }
        let parent_local = local_name(parent_iri);
        if skip_classes.contains(parent_local) {
            continue;
        }

        if scope.is_local(parent_iri) {
            parents.push(format!("{parent_local} P"));
        } else {
            match lean_qualified_name(parent_iri, ns_map) {
                Some(qualified) => parents.push(format!("{qualified} P")),
                None => parents.push(format!("{parent_local} P")),
            }
        }
    }

    parents
}

/// Collects field names inherited from parent classes (transitive closure).
fn collect_inherited_fields(
    class: &Class,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &Class>,
    skip_classes: &HashSet<&str>,
) -> HashSet<String> {
    let mut result = HashSet::new();
    let mut visited = HashSet::new();
    collect_inherited_fields_recursive(
        class.subclass_of,
        all_props_by_domain,
        all_classes_by_iri,
        skip_classes,
        &mut result,
        &mut visited,
    );
    result
}

/// Recursively walks parent classes to collect inherited field names.
fn collect_inherited_fields_recursive(
    parents: &[&str],
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_classes_by_iri: &HashMap<&str, &Class>,
    skip_classes: &HashSet<&str>,
    result: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) {
    for parent_iri in parents {
        if *parent_iri == OWL_THING {
            continue;
        }
        let parent_local = local_name(parent_iri);
        if skip_classes.contains(parent_local) {
            continue;
        }
        if !visited.insert(parent_iri.to_string()) {
            continue;
        }

        if let Some(props) = all_props_by_domain.get(parent_iri) {
            for prop in props {
                if prop.kind != PropertyKind::Annotation {
                    result.insert(to_lean_field_name(local_name(prop.id)));
                }
            }
        }

        if let Some(parent_class) = all_classes_by_iri.get(parent_iri) {
            collect_inherited_fields_recursive(
                parent_class.subclass_of,
                all_props_by_domain,
                all_classes_by_iri,
                skip_classes,
                result,
                visited,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scc_empty() {
        let g: Vec<Vec<usize>> = Vec::new();
        assert!(tarjan_scc(&g).is_empty());
    }

    #[test]
    fn scc_single_no_edges() {
        let g = vec![vec![]];
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0], vec![0]);
    }

    #[test]
    fn scc_two_cycle() {
        let g = vec![vec![1], vec![0]];
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 1);
        let mut s = sccs[0].clone();
        s.sort();
        assert_eq!(s, vec![0, 1]);
    }

    #[test]
    fn scc_chain() {
        let g = vec![vec![1], vec![2], vec![]];
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 3);
        for scc in &sccs {
            assert_eq!(scc.len(), 1);
        }
    }

    #[test]
    fn scc_diamond() {
        // 0 -> {1,2}; 1 -> 3; 2 -> 3
        let g = vec![vec![1, 2], vec![3], vec![3], vec![]];
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 4);
    }

    #[test]
    fn scc_3cycle_in_chain() {
        // 4 -> 0; 0 -> 1; 1 -> 2; 2 -> 0; 2 -> 3
        let g = vec![vec![1], vec![2], vec![0, 3], vec![], vec![0]];
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 3);
        let mut sizes: Vec<usize> = sccs.iter().map(|s| s.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![1, 1, 3]);
    }

    #[test]
    fn self_referential_wraps_option() {
        let ontology = uor_ontology::Ontology::full();
        let ns_map = crate::mapping::lean_namespace_mappings();
        let all_props = crate::build_all_props_by_domain(ontology);
        let all_classes = crate::build_all_classes_by_iri(ontology);
        let (content, sc, _fc) =
            generate_all_structures(ontology, &ns_map, &all_props, &all_classes);
        assert!(
            content.contains("Option (Operation P)"),
            "Operation.inverse should be wrapped in Option"
        );
        assert!(sc > 0);
    }

    #[test]
    fn deterministic_regeneration() {
        let ontology = uor_ontology::Ontology::full();
        let ns_map = crate::mapping::lean_namespace_mappings();
        let all_props = crate::build_all_props_by_domain(ontology);
        let all_classes = crate::build_all_classes_by_iri(ontology);
        let (a, _, _) = generate_all_structures(ontology, &ns_map, &all_props, &all_classes);
        let (b, _, _) = generate_all_structures(ontology, &ns_map, &all_props, &all_classes);
        assert_eq!(a, b, "regeneration must be byte-identical");
    }

    #[test]
    fn structures_file_has_imports_and_open() {
        let ontology = uor_ontology::Ontology::full();
        let ns_map = crate::mapping::lean_namespace_mappings();
        let all_props = crate::build_all_props_by_domain(ontology);
        let all_classes = crate::build_all_classes_by_iri(ontology);
        let (content, _, _) = generate_all_structures(ontology, &ns_map, &all_props, &all_classes);
        assert!(content.contains("import UOR.Primitives"));
        assert!(content.contains("import UOR.Enums"));
        assert!(content.contains("open UOR.Primitives"));
    }
}
