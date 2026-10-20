//! Trait generation: OWL class → Rust trait, OWL property → trait method.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as FmtWrite;

use uor_ontology::model::iris::*;
use uor_ontology::model::{Class, Property, PropertyKind};
use uor_ontology::NamespaceModule;

use crate::emit::{normalize_comment, RustFile};
use crate::mapping::{
    class_trait_path, local_name, namespace_mappings, to_snake_case, xsd_is_unsized,
    xsd_to_primitives_type, NamespaceMapping,
};

/// Set of class local names that skip trait generation.
/// Most are enum classes; WittLevel is a struct but also skips trait generation.
/// The authoritative list is [`uor_ontology::Ontology::enum_class_names()`].
fn enum_class_names() -> HashSet<&'static str> {
    uor_ontology::Ontology::enum_class_names()
        .iter()
        .copied()
        .collect()
}

/// Maps an enum class local name to its enum type name.
/// When an ObjectProperty's range is one of these, we return the enum directly
/// instead of generating an associated type with a trait bound.
/// All current enum classes use identity mapping (name → name).
fn object_property_enum_override(range_local: &str) -> Option<&'static str> {
    uor_ontology::Ontology::enum_class_names()
        .iter()
        .find(|&&name| name == range_local)
        .copied()
}

/// Collects associated type names that parent traits already declare,
/// so that child traits do not re-declare them (which causes E0221 ambiguity).
fn collect_inherited_assoc_types(
    class: &Class,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
) -> HashSet<String> {
    let mut result = HashSet::new();
    for parent_iri in class.subclass_of {
        if *parent_iri == OWL_THING {
            continue;
        }
        let parent_local = local_name(parent_iri);
        if let Some(props) = all_props_by_domain.get(*parent_iri) {
            for prop in props {
                if prop.kind == PropertyKind::Object {
                    let range_local = local_name(prop.range);
                    if object_property_enum_override(range_local).is_none()
                        && prop.range != OWL_THING
                        && prop.range != OWL_CLASS
                        && prop.range != RDF_LIST
                    {
                        let assoc_name = if range_local == parent_local {
                            format!("{range_local}Target")
                        } else {
                            range_local.to_string()
                        };
                        result.insert(assoc_name);
                    }
                }
            }
        }
    }
    result
}

/// Generates a single namespace module file.
///
/// Returns the Rust source code for the module. Also emits Phase-2 Null
/// stubs (`Null{Class}<H>` + `impl Trait<H>` for every class classified
/// `Path1HandleResolver`) after the trait section of each namespace. The
/// classification is looked up via `uor_codegen::classification`.
pub fn generate_namespace_module(
    module: &NamespaceModule,
    ns_map: &HashMap<&str, NamespaceMapping>,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
) -> String {
    let ns = &module.namespace;
    let space_str = format!("{:?}", ns.space);

    let mut f = RustFile::new(&format!(
        "`{}/` namespace — {}.\n//!\n//! Space: {space_str}",
        ns.prefix,
        normalize_comment(ns.comment)
    ));

    let skip_classes = enum_class_names();

    // Determine imports needed. Phase B (target §4.1 W10): the `H: HostTypes`
    // parameter replaces the deleted `P: Primitives`. The same heuristic
    // applies — any trait whose class has a property or a non-owl-Thing
    // supertrait receives the generic parameter.
    let mut needs_host_types = false;
    for prop in &module.properties {
        if prop.domain.is_some() && prop.kind != PropertyKind::Annotation {
            needs_host_types = true;
            break;
        }
    }
    for class in &module.classes {
        if skip_classes.contains(local_name(class.id)) {
            continue;
        }
        for _parent in class.subclass_of {
            if *_parent != OWL_THING {
                needs_host_types = true;
            }
        }
    }

    // Collect enum imports needed (only for properties that generate methods,
    // i.e., properties whose domain is in the current namespace)
    let mut enum_imports: Vec<&str> = Vec::new();
    for prop in &module.properties {
        // Skip cross-namespace domain properties — they don't generate methods
        if let Some(domain) = prop.domain {
            if !domain.starts_with(ns.iri) {
                continue;
            }
        }
        if let Some(override_name) = datatype_enum_override(prop) {
            if !enum_imports.contains(&override_name) {
                enum_imports.push(override_name);
            }
        }
        // Also check object property ranges that are enum classes
        if prop.kind == PropertyKind::Object {
            let range_local = local_name(prop.range);
            if let Some(enum_name) = object_property_enum_override(range_local) {
                if !enum_imports.contains(&enum_name) {
                    enum_imports.push(enum_name);
                }
            }
        }
    }

    // Phase 7c (cross-namespace enum imports). Null stubs impl every
    // transitive supertrait of each class, and those supertraits may live
    // in other namespaces and return enums. Walk each class's parents and
    // collect enum ranges from their properties — without this pass the
    // Null impl's method body references `MyEnum` without a matching `use`.
    //
    // Apply the same "cross-namespace domain" filter that `generate_trait`
    // uses: only consider properties declared in the parent trait's own
    // namespace. Properties declared in a different namespace that name a
    // parent as their domain are not emitted as trait methods, so their
    // enum ranges don't need imports.
    let ontology_for_enums = uor_ontology::Ontology::full();
    for class in &module.classes {
        if skip_classes.contains(local_name(class.id)) {
            continue;
        }
        for parent_iri in transitive_supertraits(class, ontology_for_enums) {
            let parent_ns = namespace_of(parent_iri, ns_map);
            if let Some(props) = all_props_by_domain.get(parent_iri) {
                for prop in props {
                    if prop.kind == PropertyKind::Annotation {
                        continue;
                    }
                    // Same filter as trait generation: skip properties whose
                    // declaring namespace differs from the parent's.
                    if !prop.id.starts_with(parent_ns) {
                        continue;
                    }
                    if let Some(override_name) = datatype_enum_override(prop) {
                        if !enum_imports.contains(&override_name) {
                            enum_imports.push(override_name);
                        }
                    }
                    if prop.kind == PropertyKind::Object {
                        let range_local = local_name(prop.range);
                        if let Some(enum_name) = object_property_enum_override(range_local) {
                            if !enum_imports.contains(&enum_name) {
                                enum_imports.push(enum_name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Emit imports in alphabetical order (enum imports before HostTypes).
    enum_imports.sort_unstable();
    for imp in &enum_imports {
        let _ = writeln!(f.buf, "use crate::enums::{imp};");
    }
    if needs_host_types {
        f.line("use crate::HostTypes;");
    }
    f.blank();

    // Build property-to-domain lookup
    let props_by_domain = build_props_by_domain(&module.properties);

    // Generate traits for each class (skip enum-represented classes)
    for class in &module.classes {
        if skip_classes.contains(local_name(class.id)) {
            continue;
        }
        generate_trait(
            &mut f,
            class,
            &props_by_domain,
            all_props_by_domain,
            ns_map,
            ns.iri,
        );
    }

    // Phase 2 (orphan-closure): emit Null stubs for every Path-1 class in
    // this namespace. Each stub impls its ontology trait (and every
    // transitive non-Thing supertrait) with absent-sentinel defaults.
    emit_null_stubs_for_namespace(&mut f, module, all_props_by_domain, ns_map);

    // Phase 8 (orphan-closure): emit `{Foo}Handle` / `{Foo}Resolver` /
    // `{Foo}Record` / `Resolved{Foo}` for every Path-1 class in this
    // namespace.
    crate::resolved_wrapper::emit_resolved_wrappers_for_namespace(
        &mut f,
        module,
        all_props_by_domain,
        ns_map,
    );

    // Generate individual constants
    generate_individuals(&mut f, module);

    f.finish()
}

/// Phase 2 emission: walks `module.classes`, classifies each, and emits a
/// `Null{Class}<H>` stub + `impl Trait<H>` for every class whose
/// classification is `Path1HandleResolver` AND every supertrait impl
/// required to satisfy the trait hierarchy.
fn emit_null_stubs_for_namespace(
    f: &mut RustFile,
    module: &NamespaceModule,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
) {
    let ontology = uor_ontology::Ontology::full();
    let enum_names = enum_class_names();
    let emitable = emitable_null_set(ontology, &enum_names);
    for class in &module.classes {
        if !emitable.contains(class.id) {
            continue;
        }
        emit_null_stub(
            f,
            class,
            ontology,
            all_props_by_domain,
            ns_map,
            &enum_names,
            &emitable,
        );
    }
}

/// Returns the set of class IRIs for which Phase-2 will emit `Null{Class}<H>`
/// stubs. Computed as a fixed point: a class is in the set iff every class
/// referenced as a property range (directly + via transitive parents) is
/// either (a) in the set, (b) an enum class (skipped — traits return
/// enums directly; filtered separately), (c) owl:Thing / owl:Class /
/// rdf:List (trait returns `&H::HostString` — no Null needed), (d) an XSD
/// primitive, or (e) a class with a known existing Null stub
/// (`NullPartition<H>` from the Product/Coproduct Amendment).
fn emitable_null_set<'a>(
    ontology: &'a uor_ontology::Ontology,
    enum_names: &HashSet<&'static str>,
) -> HashSet<&'a str> {
    // Seed: every candidate per `should_emit_null_stub`. Iterate; drop any
    // candidate whose transitive references aren't satisfied.
    let mut candidates: HashSet<&str> = HashSet::new();
    for module in &ontology.namespaces {
        for class in &module.classes {
            if should_emit_null_stub(class, ontology, enum_names) {
                candidates.insert(class.id);
            }
        }
    }
    let existing_nulls = existing_null_class_iris();
    loop {
        let snapshot = candidates.clone();
        candidates.retain(|iri| {
            let class = match ontology.find_class(iri) {
                Some(c) => c,
                None => return false,
            };
            transitive_references(class, ontology)
                .into_iter()
                .all(|ref_iri| {
                    is_reference_satisfied(ref_iri, &snapshot, &existing_nulls, enum_names)
                })
        });
        if candidates.len() == snapshot.len() {
            break;
        }
    }
    candidates
}

/// Every class IRI referenced (directly or via transitive supertraits) by
/// some property of `class`. Used to verify that `emitable_null_set` can
/// satisfy all references.
fn transitive_references<'a>(
    class: &'a Class,
    ontology: &'a uor_ontology::Ontology,
) -> Vec<&'a str> {
    let all = all_properties_by_domain(ontology);
    let mut refs: Vec<&str> = Vec::new();
    let mut record = |iri: &'a str| {
        if !refs.contains(&iri) {
            refs.push(iri);
        }
    };
    let mut visit = |domain_iri: &'a str| {
        if let Some(props) = all.get(domain_iri) {
            for p in props {
                if p.kind != PropertyKind::Object {
                    continue;
                }
                record(p.range);
            }
        }
    };
    visit(class.id);
    for parent_iri in transitive_supertraits(class, ontology) {
        visit(parent_iri);
    }
    refs
}

/// Resolves whether a range IRI is a "satisfied" reference at emission time.
fn is_reference_satisfied(
    range_iri: &str,
    emitable: &HashSet<&str>,
    existing_nulls: &HashMap<&'static str, &'static str>,
    enum_names: &HashSet<&'static str>,
) -> bool {
    // Generic pointers → trait returns `&H::HostString`; no Null needed.
    if range_iri == OWL_THING || range_iri == OWL_CLASS || range_iri == RDF_LIST {
        return true;
    }
    // XSD primitives — handled as scalars.
    if range_iri.starts_with("http://www.w3.org/2001/XMLSchema#") {
        return true;
    }
    // Enum classes — trait returns the enum; no Null stub referenced.
    // (Classes with enum accessors are already filtered in `should_emit_null_stub`.)
    if enum_names.contains(local_name(range_iri)) {
        return true;
    }
    // In the emitable set, or has a known existing Null stub
    // (Product/Coproduct Amendment NullPartition family).
    emitable.contains(range_iri) || existing_nulls.contains_key(range_iri)
}

/// Class IRIs whose Null stubs already exist in `foundation/src/enforcement.rs`
/// (emitted by the Product/Coproduct Amendment §D1.2). Mapped to their
/// stub-type name; Phase 2 references these by full `crate::*` path.
fn existing_null_class_iris() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "https://uor.foundation/partition/Partition",
        "crate::enforcement::NullPartition",
    );
    m
}

/// Filters a class for Null-stub emission.
///
/// Emit iff the class:
///   - classifies `Path1HandleResolver` (Phase 2: resolver-absent default),
///     `Path2TheoremWitness` (Phase 3: witness trait orphan until a concrete
///     type impls it), or `Path4TheoryDeferred` (Phase 7d: reference-
///     satisfaction stub with `#[doc(hidden)]` + THEORY-DEFERRED banner);
///     AND
///   - is not itself an enum class (enums don't have traits).
///
/// Phase 7 removed the earlier "no enum-typed accessors" filter — enum
/// variants now derive `Default` (Phase 7a) and cross-namespace enum imports
/// are pre-collected (Phase 7c), so a Null stub returning an enum defaults
/// to the spec-canonical first variant via `<Enum>::default()`.
fn should_emit_null_stub(
    class: &Class,
    ontology: &uor_ontology::Ontology,
    enum_names: &HashSet<&'static str>,
) -> bool {
    if enum_names.contains(local_name(class.id)) {
        return false;
    }
    let path_kind = crate::classification::classify(class, ontology).path_kind;
    // Phase 11: Path-3 (primitive-backed) classes ALSO get a Null stub.
    // The Null stub closes the orphan for resolver-absent contexts;
    // the Phase-11 hand-written blanket impl on `Validated<T, Phase>`
    // closes it for primitive-backed contexts. Both coexist via
    // mutually-disjoint concrete carriers.
    matches!(
        path_kind,
        crate::classification::PathKind::Path1HandleResolver
            | crate::classification::PathKind::Path2TheoremWitness { .. }
            | crate::classification::PathKind::Path3PrimitiveBacked { .. }
            | crate::classification::PathKind::Path4TheoryDeferred
    )
}

/// Returns all non-owl:Thing transitive supertraits of `class`, deduplicated.
/// Excludes enum-class parents (they don't generate traits).
fn transitive_supertraits<'a>(
    class: &'a Class,
    ontology: &'a uor_ontology::Ontology,
) -> Vec<&'a str> {
    let enum_names = enum_class_names();
    let mut result: Vec<&'a str> = Vec::new();
    let mut frontier: Vec<&'a str> = class.subclass_of.to_vec();
    while let Some(parent_iri) = frontier.pop() {
        if parent_iri == OWL_THING {
            continue;
        }
        if enum_names.contains(local_name(parent_iri)) {
            continue;
        }
        if result.contains(&parent_iri) {
            continue;
        }
        result.push(parent_iri);
        if let Some(parent_class) = ontology.find_class(parent_iri) {
            for pp in parent_class.subclass_of {
                frontier.push(pp);
            }
        }
    }
    result
}

/// Cached `all_props_by_domain` lookup — rebuilt per call (cheap; the
/// ontology is static). Keeps `emit_null_stub` pure without threading the
/// caller's map through.
fn all_properties_by_domain<'a>(
    ontology: &'a uor_ontology::Ontology,
) -> HashMap<&'a str, Vec<&'a Property>> {
    let mut map: HashMap<&'a str, Vec<&'a Property>> = HashMap::new();
    for module in &ontology.namespaces {
        for prop in &module.properties {
            if let Some(domain) = prop.domain {
                map.entry(domain).or_default().push(prop);
            }
        }
    }
    map
}

/// Emits `Null{Class}<H>` struct, `Default` impl, `ABSENT` const, and every
/// required `impl Trait<H> for Null{Class}<H>` (class itself + transitive
/// supertraits).
fn emit_null_stub(
    f: &mut RustFile,
    class: &Class,
    ontology: &uor_ontology::Ontology,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    enum_names: &HashSet<&'static str>,
    emitable: &HashSet<&str>,
) {
    let class_local = local_name(class.id);
    let null_type = format!("Null{class_local}");
    let class_iri_ns = namespace_of(class.id, ns_map);

    // Phase 7d: Path-4 classes are emitted with the `#[doc(hidden)]`
    // THEORY-DEFERRED banner. The struct shape is identical to Path-1/2
    // stubs; only the banner differs.
    let path_kind = crate::classification::classify(class, ontology).path_kind;
    let is_theory_deferred = matches!(
        path_kind,
        crate::classification::PathKind::Path4TheoryDeferred
    );

    // ── Struct + Default + ABSENT const ───────────────────────────────
    if is_theory_deferred {
        // Banner is deliberately the exact string the conformance validator
        // greps for. Any drift breaks `rust/theory_deferred_register`.
        f.line("#[doc(hidden)]");
        f.line(
            "#[doc = \"THEORY-DEFERRED \\u{2014} not a valid implementation; \
             see [docs/theory_deferred.md]. Exists only to satisfy downstream \
             trait-bound references.\"]",
        );
    } else {
        f.doc_comment(&format!(
            "Phase 2 (orphan-closure) — resolver-absent default impl of `{class_local}<H>`."
        ));
        f.doc_comment("Every accessor returns `H::EMPTY_*` sentinels (for scalar / host-typed");
        f.doc_comment("returns) or a `'static`-lifetime reference to a sibling `Null*`'s `ABSENT`");
        f.doc_comment("const (for trait-typed returns).  Downstream provides concrete impls;");
        f.doc_comment("this stub closes the ontology-derived trait orphan.");
    }
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    let _ = writeln!(f.buf, "pub struct {null_type}<H: HostTypes> {{");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    let _ = writeln!(f.buf, "impl<H: HostTypes> Default for {null_type}<H> {{");
    f.line("    fn default() -> Self {");
    f.line("        Self {");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    let _ = writeln!(f.buf, "impl<H: HostTypes> {null_type}<H> {{");
    f.indented_doc_comment(
        "Absent-value sentinel. `&Self::ABSENT` gives every trait-typed \
         accessor a `'static`-lifetime reference target.",
    );
    // rustfmt wraps `pub const ABSENT: Foo<H> = Foo {` when the prefix
    // exceeds the 100-char line budget. Mirror that here so regen output
    // is fmt-clean without a follow-up `cargo fmt` pass.
    let prefix_len = format!("    pub const ABSENT: {null_type}<H> = {null_type} {{").len();
    if prefix_len > 96 {
        let _ = writeln!(f.buf, "    pub const ABSENT: {null_type}<H> =");
        let _ = writeln!(f.buf, "        {null_type} {{");
        f.line("            _phantom: core::marker::PhantomData,");
        f.line("        };");
    } else {
        let _ = writeln!(
            f.buf,
            "    pub const ABSENT: {null_type}<H> = {null_type} {{"
        );
        f.line("        _phantom: core::marker::PhantomData,");
        f.line("    };");
    }
    f.line("}");

    // ── Trait impls: transitive supertraits first, then class itself ──
    let existing_nulls = existing_null_class_iris();
    let parents = transitive_supertraits(class, ontology);
    // Phase 7b: assoc-type names that parent traits introduce. When emitting
    // `impl Child<H> for Null{X}<H>`, child must not re-declare `type Foo = ..`
    // if a supertrait already declares it (E0202 / semantic drift). Each
    // parent-trait impl has its own `emitted_assoc` counter; the inherited
    // set guards the child-trait impl.
    let inherited_for_class = collect_inherited_assoc_types(class, all_props_by_domain);
    for parent_iri in parents.iter().rev() {
        let parent_class_opt = ontology.find_class(parent_iri);
        let inherited_for_parent = match parent_class_opt {
            Some(pc) => collect_inherited_assoc_types(pc, all_props_by_domain),
            None => HashSet::new(),
        };
        emit_null_impl_for_trait(
            f,
            &null_type,
            parent_iri,
            all_props_by_domain,
            ns_map,
            class_iri_ns,
            enum_names,
            emitable,
            &existing_nulls,
            &inherited_for_parent,
        );
    }
    emit_null_impl_for_trait(
        f,
        &null_type,
        class.id,
        all_props_by_domain,
        ns_map,
        class_iri_ns,
        enum_names,
        emitable,
        &existing_nulls,
        &inherited_for_class,
    );
    f.blank();
}

/// Emits a single `impl Trait<H> for Null{Class}<H>` block, with one method
/// body per direct property of `trait_iri` and an associated type per
/// object-property range.
#[allow(clippy::too_many_arguments)]
fn emit_null_impl_for_trait(
    f: &mut RustFile,
    null_type: &str,
    trait_iri: &str,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    enum_names: &HashSet<&'static str>,
    emitable: &HashSet<&str>,
    existing_nulls: &HashMap<&'static str, &'static str>,
    inherited_assocs: &HashSet<String>,
) {
    let trait_local = local_name(trait_iri);
    let trait_path = if trait_iri.starts_with(current_ns_iri) {
        trait_local.to_string()
    } else {
        class_trait_path(trait_iri, ns_map).unwrap_or_else(|| trait_local.to_string())
    };

    // Mirror the trait-generation filter (CLAUDE.md: "Cross-namespace domain
    // properties are not generated"). A trait's method set is the properties
    // declared IN THE SAME NAMESPACE as the trait, whose domain is the trait.
    // Properties declared in another namespace with a cross-namespace domain
    // pointing at this trait never become trait methods, so the Null stub
    // must not try to impl them.
    let ontology = uor_ontology::Ontology::full();
    let trait_ns_iri = namespace_of(trait_iri, ns_map);
    let declaring_module = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.iri == trait_ns_iri);
    let _ = all_props_by_domain;
    let direct_props: Vec<&Property> = match declaring_module {
        Some(m) => m
            .properties
            .iter()
            .filter(|p| p.kind != PropertyKind::Annotation && p.domain == Some(trait_iri))
            .collect(),
        None => Vec::new(),
    };

    // rustfmt wraps a too-long `impl<...> {trait_path}<H> for {null_type}<H>`
    // header onto two lines and forces empty bodies to `{\n}\n`. Mirror that
    // so regen is fmt-clean.
    let header_len = format!("impl<H: HostTypes> {trait_path}<H> for {null_type}<H> {{}}").len();
    let header_wraps = header_len > 100;

    if direct_props.is_empty() {
        if header_wraps {
            let _ = writeln!(f.buf, "impl<H: HostTypes> {trait_path}<H>");
            let _ = writeln!(f.buf, "    for {null_type}<H>");
            f.line("{");
            f.line("}");
        } else {
            let _ = writeln!(
                f.buf,
                "impl<H: HostTypes> {trait_path}<H> for {null_type}<H> {{}}"
            );
        }
        return;
    }

    if header_wraps {
        let _ = writeln!(f.buf, "impl<H: HostTypes> {trait_path}<H>");
        let _ = writeln!(f.buf, "    for {null_type}<H>");
        f.line("{");
    } else {
        let _ = writeln!(
            f.buf,
            "impl<H: HostTypes> {trait_path}<H> for {null_type}<H> {{"
        );
    }

    // Inherited associated-type declarations come from parent traits, so only
    // emit an associated type if this trait is the one that introduces it.
    let mut emitted_assoc: HashSet<String> = HashSet::new();
    for prop in &direct_props {
        emit_null_method_body(
            f,
            prop,
            ns_map,
            current_ns_iri,
            trait_local,
            enum_names,
            &mut emitted_assoc,
            emitable,
            existing_nulls,
            inherited_assocs,
        );
    }
    f.line("}");
}

/// Emits one method (and, if needed, one associated type) inside the
/// `impl Trait<H> for Null{Class}<H>` block currently being built. Returns
/// without emitting if the property is annotation-only or uses an enum
/// range (callers pre-filter but we guard defensively).
#[allow(clippy::too_many_arguments)]
fn emit_null_method_body(
    f: &mut RustFile,
    prop: &Property,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    owner_trait_name: &str,
    _enum_names: &HashSet<&'static str>,
    emitted_assoc: &mut HashSet<String>,
    emitable: &HashSet<&str>,
    existing_nulls: &HashMap<&'static str, &'static str>,
    inherited_assocs: &HashSet<String>,
) {
    let method_name = to_snake_case(local_name(prop.id));

    // Helper: emit a method body of the form `fn name(&self) -> RetTy { expr }`
    // as the multi-line block rustfmt prefers. Keeps every Null-stub method
    // emission fmt-clean without a follow-up `cargo fmt` pass.
    let emit_fn = |f: &mut RustFile, sig: &str, body: &str| {
        let _ = writeln!(f.buf, "    {sig} {{");
        let _ = writeln!(f.buf, "        {body}");
        f.line("    }");
    };

    match prop.kind {
        PropertyKind::Datatype => {
            // Enum-typed datatype: trait returns the enum by value; Null stub
            // defaults to the spec-canonical first variant via `Enum::default()`.
            if let Some(enum_t) = datatype_enum_override(prop) {
                if prop.functional {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> {enum_t}"),
                        &format!("<{enum_t}>::default()"),
                    );
                } else {
                    emit_fn(f, &format!("fn {method_name}(&self) -> &[{enum_t}]"), "&[]");
                }
                return;
            }
            let prim = xsd_to_primitives_type(prop.range);
            match prim {
                Some(t) => {
                    if prop.functional {
                        let body = match t {
                            "H::HostString" => "H::EMPTY_HOST_STRING".to_string(),
                            "H::WitnessBytes" => "H::EMPTY_WITNESS_BYTES".to_string(),
                            "H::Decimal" => "H::EMPTY_DECIMAL".to_string(),
                            "bool" => "false".to_string(),
                            _ => "0".to_string(), // u64 / i64 / u32 / i32
                        };
                        if xsd_is_unsized(prop.range) {
                            emit_fn(f, &format!("fn {method_name}(&self) -> &{t}"), &body);
                        } else {
                            emit_fn(f, &format!("fn {method_name}(&self) -> {t}"), &body);
                        }
                    } else if xsd_is_unsized(prop.range) {
                        let body = match t {
                            "H::HostString" => "H::EMPTY_HOST_STRING",
                            "H::WitnessBytes" => "H::EMPTY_WITNESS_BYTES",
                            _ => "H::EMPTY_HOST_STRING",
                        };
                        emit_fn(f, &format!("fn {method_name}_count(&self) -> usize"), "0");
                        emit_fn(
                            f,
                            &format!("fn {method_name}_at(&self, _index: usize) -> &{t}"),
                            body,
                        );
                    } else {
                        emit_fn(f, &format!("fn {method_name}(&self) -> &[{t}]"), "&[]");
                    }
                }
                None => {
                    // Unknown XSD: the trait emits `&H::HostString`; mirror here.
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> &H::HostString"),
                        "H::EMPTY_HOST_STRING",
                    );
                }
            }
        }
        PropertyKind::Object => {
            let range_local = local_name(prop.range);
            let is_owl_thing = prop.range == OWL_THING;
            let is_owl_class = prop.range == OWL_CLASS;
            let is_rdf_list = prop.range == RDF_LIST;

            // Enum-typed object range: trait returns the enum directly; Null
            // stub defaults to the spec-canonical first variant.
            if let Some(enum_name) = object_property_enum_override(range_local) {
                if prop.functional {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> {enum_name}"),
                        &format!("<{enum_name}>::default()"),
                    );
                } else {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> &[{enum_name}]"),
                        "&[]",
                    );
                }
                return;
            }

            if is_owl_thing || is_owl_class || is_rdf_list {
                // Trait emits `&H::HostString` (or count+at). Mirror.
                if prop.functional {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> &H::HostString"),
                        "H::EMPTY_HOST_STRING",
                    );
                } else {
                    emit_fn(f, &format!("fn {method_name}_count(&self) -> usize"), "0");
                    emit_fn(
                        f,
                        &format!("fn {method_name}_at(&self, _index: usize) -> &H::HostString"),
                        "H::EMPTY_HOST_STRING",
                    );
                }
                return;
            }

            // Ontology class range — associated type + reference to the
            // sibling Null stub's ABSENT const.
            let assoc_name = if range_local == owner_trait_name {
                format!("{range_local}Target")
            } else {
                range_local.to_string()
            };
            // If the range has an existing hand-written Null stub
            // (Product/Coproduct Amendment NullPartition family), use that
            // path. Otherwise, the range must be in the emitable set and
            // we construct `Null{Range}` at the appropriate module path.
            let null_path = if let Some(path) = existing_nulls.get(prop.range) {
                (*path).to_string()
            } else if emitable.contains(prop.range) {
                let null_range = format!("Null{range_local}");
                let is_cross_ns = !prop.range.starts_with(current_ns_iri);
                if is_cross_ns {
                    let module = class_trait_path(prop.range, ns_map).unwrap_or_default();
                    // class_trait_path returns e.g. `crate::bridge::partition::Foo`
                    // — replace the class suffix with our Null<class> path.
                    if let Some(prefix_end) = module.rfind("::") {
                        format!("{}::{null_range}", &module[..prefix_end])
                    } else {
                        null_range
                    }
                } else {
                    null_range
                }
            } else {
                // Caller's `emitable_null_set` should have filtered this out
                // already. Defensive: emit a compile-error marker so drift
                // doesn't produce silently-broken code.
                let _ = writeln!(
                    f.buf,
                    "    // ORPHAN_CLOSURE_EMISSION_ERROR: range {} not in emitable set",
                    prop.range
                );
                return;
            };

            // Phase 7b: skip the `type {assoc_name} = ..;` declaration if
            // (a) a parent trait already declared it (`inherited_assocs`) —
            //     the assoc type lives on the parent's impl block — OR
            // (b) this impl block already emitted it (`emitted_assoc`).
            // The method body (`fn m(&self) -> &Self::{assoc_name}`) is
            // still emitted unconditionally; only the `type =` line is
            // deduplicated.
            if !inherited_assocs.contains(&assoc_name) && !emitted_assoc.contains(&assoc_name) {
                let _ = writeln!(f.buf, "    type {assoc_name} = {null_path}<H>;");
                emitted_assoc.insert(assoc_name.clone());
            }

            if prop.functional {
                if is_by_value_partition_factor(prop.id) {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> Self::{assoc_name}"),
                        &format!("<{null_path}<H>>::default()"),
                    );
                } else {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> &Self::{assoc_name}"),
                        &format!("&<{null_path}<H>>::ABSENT"),
                    );
                }
            } else {
                emit_fn(
                    f,
                    &format!("fn {method_name}(&self) -> &[Self::{assoc_name}]"),
                    "&[]",
                );
            }
        }
        PropertyKind::Annotation => {}
    }
}

/// Returns the namespace IRI that contains `class_iri`.
fn namespace_of<'a>(class_iri: &'a str, ns_map: &HashMap<&str, NamespaceMapping>) -> &'a str {
    for ns in ns_map.keys() {
        if class_iri.starts_with(*ns) {
            // `starts_with` on &str returns bool; we need the slice of class_iri
            // matching the namespace. But class_iri's static str slice IS what
            // we want — namespace IRI itself, if we had the matching owned ref.
            // Simpler: return a prefix of class_iri by finding the last "/" or
            // matching against ns.
            return &class_iri[..ns.len()];
        }
    }
    class_iri
}

/// Builds a map from domain class IRI → list of properties.
fn build_props_by_domain(properties: &[Property]) -> HashMap<&str, Vec<&Property>> {
    let mut map: HashMap<&str, Vec<&Property>> = HashMap::new();
    for prop in properties {
        if let Some(domain) = prop.domain {
            map.entry(domain).or_default().push(prop);
        }
    }
    map
}

/// Generates a single trait for a class.
fn generate_trait(
    f: &mut RustFile,
    class: &Class,
    props_by_domain: &HashMap<&str, Vec<&Property>>,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
) {
    let trait_name = local_name(class.id);
    let comment = normalize_comment(class.comment);

    // Doc comment
    f.doc_comment(&comment);

    // Disjoint-with note
    if !class.disjoint_with.is_empty() {
        f.doc_comment("");
        let disjoints: Vec<&str> = class.disjoint_with.iter().map(|d| local_name(d)).collect();
        let _ = writeln!(f.buf, "/// Disjoint with: {}.", disjoints.join(", "));
    }

    // Phase B: every trait takes `<H: HostTypes>` for consistency. The
    // `Primitives` trait is deleted; `HostTypes` is the only host-environment
    // slot carrier (three slots: `Decimal`, `HostString`, `WitnessBytes`).
    let p_param = "<H: HostTypes>";

    // Supertrait bounds
    let supertraits = build_supertrait_bounds(class, ns_map, current_ns_iri);

    if supertraits.is_empty() {
        let _ = writeln!(f.buf, "pub trait {trait_name}{p_param} {{");
    } else {
        let bounds = supertraits.join(" + ");
        let one_line = format!("pub trait {trait_name}{p_param}: {bounds} {{");
        // rustfmt wraps trait bounds more aggressively than the nominal
        // 100-char max_width — use 92 to match its heuristic and avoid
        // drift between codegen output and `cargo fmt`.
        if one_line.chars().count() <= 92 {
            let _ = writeln!(f.buf, "{one_line}");
        } else {
            let _ = writeln!(f.buf, "pub trait {trait_name}{p_param}:\n    {bounds}\n{{");
        }
    }

    // Associated types and methods from properties
    let props = props_by_domain.get(class.id).cloned().unwrap_or_default();
    let non_annotation_props: Vec<&&Property> = props
        .iter()
        .filter(|p| p.kind != PropertyKind::Annotation)
        .collect();

    if non_annotation_props.is_empty() {
        // Empty trait body — emit `{}` on the same line for single-line
        // traits, or `{\n}\n` for multi-line traits.
        if f.buf.ends_with("{\n") {
            // Check if this is a multi-line trait (brace on its own line)
            let before_brace = &f.buf[..f.buf.len() - 2];
            if before_brace.ends_with('\n') {
                // Multi-line: keep `{` on its own line, add `}`
                f.buf.push_str("}\n");
            } else {
                // Single-line: collapse to `{}`
                f.buf.truncate(f.buf.len() - 2);
                f.buf.push_str("{}\n");
            }
        }
    } else {
        // Pre-populate with associated types already declared in parent traits
        // to avoid E0221 ambiguous-associated-type errors.
        let mut associated_types = collect_inherited_assoc_types(class, all_props_by_domain);
        for prop in &non_annotation_props {
            generate_property_method(
                f,
                prop,
                ns_map,
                current_ns_iri,
                trait_name,
                &mut associated_types,
            );
        }
        f.line("}");
    }
    f.blank();
}

/// Generates a method (and possibly an associated type) for a property.
fn generate_property_method(
    f: &mut RustFile,
    prop: &Property,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    owner_trait_name: &str,
    associated_types: &mut HashSet<String>,
) {
    let method_name = to_snake_case(local_name(prop.id));
    let comment = normalize_comment(prop.comment);

    match prop.kind {
        PropertyKind::Datatype => {
            // Check if the range maps to a known enum type
            let enum_type = datatype_enum_override(prop);
            if let Some(enum_t) = enum_type {
                f.indented_doc_comment(&comment);
                let _ = writeln!(f.buf, "    fn {method_name}(&self) -> {enum_t};");
                return;
            }

            let prim_type = xsd_to_primitives_type(prop.range);
            match prim_type {
                Some(t) => {
                    f.indented_doc_comment(&comment);
                    if prop.functional {
                        if xsd_is_unsized(prop.range) {
                            let _ = writeln!(f.buf, "    fn {method_name}(&self) -> &{t};");
                        } else {
                            let _ = writeln!(f.buf, "    fn {method_name}(&self) -> {t};");
                        }
                    } else if xsd_is_unsized(prop.range) {
                        // Non-functional unsized: can't have &[str], emit count + indexed getter
                        let _ = writeln!(f.buf, "    fn {method_name}_count(&self) -> usize;");
                        f.indented_doc_comment(&format!(
                            "Returns the item at `index`. Must satisfy `index < self.{method_name}_count()`."
                        ));
                        let _ = writeln!(
                            f.buf,
                            "    fn {method_name}_at(&self, index: usize) -> &{t};"
                        );
                    } else {
                        // Non-functional sized: return slice
                        let _ = writeln!(f.buf, "    fn {method_name}(&self) -> &[{t}];");
                    }
                }
                None => {
                    // Unknown XSD type — fall back to host-string ref.
                    f.indented_doc_comment(&comment);
                    let _ = writeln!(f.buf, "    fn {method_name}(&self) -> &H::HostString;");
                }
            }
        }
        PropertyKind::Object => {
            let range_local = local_name(prop.range);
            let is_owl_thing = prop.range == OWL_THING;
            let is_owl_class = prop.range == OWL_CLASS;
            let is_rdf_list = prop.range == RDF_LIST;

            // Check if range is an enum class — return enum type directly
            if let Some(enum_path) = object_property_enum_override(range_local) {
                f.indented_doc_comment(&comment);
                if prop.functional {
                    let _ = writeln!(f.buf, "    fn {method_name}(&self) -> {enum_path};");
                } else {
                    let _ = writeln!(f.buf, "    fn {method_name}(&self) -> &[{enum_path}];");
                }
            } else if is_owl_thing || is_owl_class || is_rdf_list {
                // Generic object — use a host-string IRI reference.
                f.indented_doc_comment(&comment);
                if prop.functional {
                    let _ = writeln!(f.buf, "    fn {method_name}(&self) -> &H::HostString;");
                } else {
                    // Non-functional unsized: emit count + indexed getter.
                    let _ = writeln!(f.buf, "    fn {method_name}_count(&self) -> usize;");
                    f.indented_doc_comment(&format!(
                        "Returns the item at `index`. Must satisfy `index < self.{method_name}_count()`."
                    ));
                    let _ = writeln!(
                        f.buf,
                        "    fn {method_name}_at(&self, index: usize) -> &H::HostString;"
                    );
                }
            } else {
                // Generate associated type + method
                // Disambiguate if the associated type name matches the owning trait
                let assoc_name = if range_local == owner_trait_name {
                    format!("{range_local}Target")
                } else {
                    range_local.to_string()
                };

                // Avoid duplicate associated types
                if !associated_types.contains(&assoc_name) {
                    // Determine the trait bound path
                    let is_cross_ns = !prop.range.starts_with(current_ns_iri);
                    let trait_bound = if is_cross_ns {
                        class_trait_path(prop.range, ns_map)
                            .map(|p| format!("{p}<H>"))
                            .unwrap_or_else(|| format!("{range_local}<H>"))
                    } else {
                        format!("{range_local}<H>")
                    };

                    let _ = writeln!(f.buf, "    /// Associated type for `{range_local}`.");
                    let _ = writeln!(f.buf, "    type {assoc_name}: {trait_bound};");
                    associated_types.insert(assoc_name.clone());
                }

                f.indented_doc_comment(&comment);
                if prop.functional {
                    if is_by_value_partition_factor(prop.id) {
                        // Product/Coproduct Completion Amendment §1d: the six
                        // partition-algebra factor accessors return by value
                        // so that witness types (PartitionProductWitness,
                        // PartitionCoproductWitness, CartesianProductWitness)
                        // can hand out a freshly constructed PartitionHandle
                        // with no backing storage. PartitionHandle is Copy
                        // and small, so by-value return is efficient.
                        let _ =
                            writeln!(f.buf, "    fn {method_name}(&self) -> Self::{assoc_name};");
                    } else {
                        let _ =
                            writeln!(f.buf, "    fn {method_name}(&self) -> &Self::{assoc_name};");
                    }
                } else {
                    let _ = writeln!(
                        f.buf,
                        "    fn {method_name}(&self) -> &[Self::{assoc_name}];"
                    );
                }
            }
        }
        PropertyKind::Annotation => {
            // Skip annotation properties in trait generation
        }
    }
}

/// Returns an enum type override for special datatype properties.
///
/// All former overrides have been removed by property retypings
/// (siteState in Amendment 90, geometricCharacter in Amendment 23).
fn datatype_enum_override(_prop: &Property) -> Option<&'static str> {
    None
}

/// Phase 8 escape hatch: re-export for `crate::resolved_wrapper`. Remove
/// when the helper moves into a shared location.
pub fn datatype_enum_override_pub(prop: &Property) -> Option<&'static str> {
    datatype_enum_override(prop)
}

/// Phase 8 escape hatch: re-export for `crate::resolved_wrapper`. Remove
/// when the helper moves into a shared location.
pub fn object_property_enum_override_pub(range_local: &str) -> Option<&'static str> {
    object_property_enum_override(range_local)
}

/// Phase 8 escape hatch: re-export for `crate::resolved_wrapper`. Returns
/// the set of associated-type names already declared by a class's
/// transitive supertraits.
pub fn collect_inherited_assoc_types_pub(
    class: &Class,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
) -> HashSet<String> {
    collect_inherited_assoc_types(class, all_props_by_domain)
}

/// Product/Coproduct Completion Amendment §1d: returns true for the six
/// partition-algebra factor accessor properties whose traits return the
/// associated Partition type by value rather than by reference.
///
/// These six property IRIs identify the left/right accessors on
/// PartitionProduct, PartitionCoproduct, and CartesianPartitionProduct.
/// Witness impls (PartitionProductWitness, PartitionCoproductWitness,
/// CartesianProductWitness) need the by-value return so they can
/// construct a fresh PartitionHandle — a Copy, register-sized value
/// type — without needing to hold a reference to persistent storage
/// inside the witness.
fn is_by_value_partition_factor(prop_id: &str) -> bool {
    matches!(
        prop_id,
        "https://uor.foundation/partition/leftFactor"
            | "https://uor.foundation/partition/rightFactor"
            | "https://uor.foundation/partition/leftSummand"
            | "https://uor.foundation/partition/rightSummand"
            | "https://uor.foundation/partition/leftCartesianFactor"
            | "https://uor.foundation/partition/rightCartesianFactor"
    )
}

/// Builds supertrait bounds for a class.
fn build_supertrait_bounds(
    class: &Class,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
) -> Vec<String> {
    let mut bounds = Vec::new();
    let skip = enum_class_names();

    for parent_iri in class.subclass_of {
        // Skip owl:Thing — it's the universal superclass
        if *parent_iri == OWL_THING {
            continue;
        }

        let parent_local = local_name(parent_iri);

        // Skip if the parent is an enum class
        if skip.contains(parent_local) {
            continue;
        }

        let is_cross_ns = !parent_iri.starts_with(current_ns_iri);

        if is_cross_ns {
            if let Some(path) = class_trait_path(parent_iri, ns_map) {
                bounds.push(format!("{path}<H>"));
            } else {
                bounds.push(format!("{parent_local}<H>"));
            }
        } else {
            bounds.push(format!("{parent_local}<H>"));
        }
    }

    bounds
}

/// Generates named individual constant modules.
fn generate_individuals(f: &mut RustFile, module: &NamespaceModule) {
    use uor_ontology::IndividualValue;

    for ind in &module.individuals {
        let type_local = local_name(ind.type_);

        // Skip individuals that are part of enums (operations, metric axes)
        // Skip individuals whose types are codegen-internal enums (PrimitiveOp
        // variants) or OWL enum classes whose individuals carry no property
        // assertions worth exposing as constant modules.  Other enum classes
        // (e.g. WittLevel, VerificationDomain) retain constant modules
        // because their individuals have data properties.
        if type_local == "UnaryOp"
            || type_local == "BinaryOp"
            || type_local == "Involution"
            || type_local == "MetricAxis"
        {
            continue;
        }

        let mod_name = to_snake_case(local_name(ind.id));
        let comment = normalize_comment(ind.comment);

        f.doc_comment(&comment);

        // Empty modules (no property assertions) → single-line `pub mod name {}`
        if ind.properties.is_empty() {
            let _ = writeln!(f.buf, "pub mod {mod_name} {{}}");
            f.blank();
            continue;
        }

        let _ = writeln!(f.buf, "pub mod {mod_name} {{");

        // Group property assertions by IRI (preserving insertion order)
        let mut grouped: BTreeMap<&str, Vec<&IndividualValue>> = BTreeMap::new();
        for (prop_iri, value) in ind.properties {
            grouped.entry(prop_iri).or_default().push(value);
        }

        for (prop_iri, values) in &grouped {
            let prop_local = local_name(prop_iri);
            let base_const = to_snake_case(prop_local).to_uppercase();

            // If any value is a List, emit from the List (subsumes IriRef entries)
            if let Some(list_val) = values.iter().find_map(|v| match v {
                IndividualValue::List(items) => Some(items),
                _ => None,
            }) {
                let _ = writeln!(f.buf, "    /// `{prop_local}`");
                emit_str_slice(&mut f.buf, &base_const, list_val);
                continue;
            }

            // Multiple IriRef values → emit as slice
            if values.len() > 1 {
                if values
                    .iter()
                    .all(|v| matches!(v, IndividualValue::IriRef(_)))
                {
                    let items: Vec<&str> = values
                        .iter()
                        .filter_map(|v| match v {
                            IndividualValue::IriRef(iri) => Some(*iri),
                            _ => None,
                        })
                        .collect();
                    let _ = writeln!(f.buf, "    /// `{prop_local}`");
                    emit_str_slice(&mut f.buf, &base_const, &items);
                    continue;
                }
                if values.iter().all(|v| matches!(v, IndividualValue::Str(_))) {
                    let items: Vec<&str> = values
                        .iter()
                        .filter_map(|v| match v {
                            IndividualValue::Str(s) => Some(*s),
                            _ => None,
                        })
                        .collect();
                    let _ = writeln!(f.buf, "    /// `{prop_local}`");
                    emit_str_slice(&mut f.buf, &base_const, &items);
                    continue;
                }
            }

            // Single value — emit scalar const
            match values[0] {
                IndividualValue::Str(s) => {
                    let _ = writeln!(f.buf, "    /// `{prop_local}`");
                    let line = format!("    pub const {base_const}: &str = \"{s}\";");
                    if line.chars().count() <= 100 {
                        let _ = writeln!(f.buf, "{line}");
                    } else {
                        let _ = writeln!(f.buf, "    pub const {base_const}: &str =");
                        let _ = writeln!(f.buf, "        \"{s}\";");
                    }
                }
                IndividualValue::Int(n) => {
                    let _ = writeln!(f.buf, "    /// `{prop_local}`");
                    let _ = writeln!(f.buf, "    pub const {base_const}: i64 = {n};");
                }
                IndividualValue::Bool(b) => {
                    let _ = writeln!(f.buf, "    /// `{prop_local}`");
                    let _ = writeln!(f.buf, "    pub const {base_const}: bool = {b};");
                }
                IndividualValue::Float(x) => {
                    // Phase 9: emit individual decimal values as IEEE-754
                    // bit patterns so the constant carries no f64 type
                    // signature in source. Consumers convert via
                    // `H::Decimal::from_bits` at use time. The bit pattern
                    // is identical to `f64::to_bits(x)`.
                    let bits = x.to_bits();
                    let _ = writeln!(
                        f.buf,
                        "    /// `{prop_local}` (IEEE-754 f64 bit pattern of `{x:?}`)."
                    );
                    let _ = writeln!(f.buf, "    pub const {base_const}_BITS: u64 = {bits}_u64;");
                }
                IndividualValue::IriRef(iri) => {
                    let ref_local = local_name(iri);
                    let _ = writeln!(f.buf, "    /// `{prop_local}` -> `{ref_local}`");
                    let line = format!("    pub const {base_const}: &str = \"{iri}\";");
                    if line.chars().count() <= 100 {
                        let _ = writeln!(f.buf, "{line}");
                    } else {
                        let _ = writeln!(f.buf, "    pub const {base_const}: &str =");
                        let _ = writeln!(f.buf, "        \"{iri}\";");
                    }
                }
                IndividualValue::List(_) => unreachable!(),
            }
        }

        f.line("}");
        f.blank();
    }
}

/// Emits a `pub const NAME: &[&str] = &[...];` with multi-line formatting for long items.
fn emit_str_slice(buf: &mut String, const_name: &str, items: &[&str]) {
    use std::fmt::Write as _;
    // Format each item on its own line for readability
    let _ = writeln!(buf, "    pub const {const_name}: &[&str] = &[");
    for item in items {
        let _ = writeln!(buf, "        \"{item}\",");
    }
    let _ = writeln!(buf, "    ];");
}

/// Returns the set of all namespace IRIs used by the ontology.
pub fn all_namespace_iris() -> HashMap<&'static str, NamespaceMapping> {
    namespace_mappings()
}
