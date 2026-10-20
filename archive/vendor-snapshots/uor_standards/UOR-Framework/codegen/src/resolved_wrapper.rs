//! Phase 8 (orphan-closure) — Resolved-wrapper infrastructure.
//!
//! Per Path-1 class `Foo`, emit four types into the namespace module:
//!
//! 1. `{Foo}Handle<H>` — content-fingerprint pointer.
//! 2. `{Foo}Resolver<H>` — resolver trait with `resolve(handle) -> Option<Record>`.
//! 3. `{Foo}Record<H>` — typed record with fields for every functional
//!    accessor of `Foo` (object properties carry `{Range}Handle<H>`).
//! 4. `Resolved{Foo}<'r, R, H>` — wrapper carrying `(handle, resolver,
//!    cached record)`; impls `Foo<H>` (and every transitive supertrait)
//!    by delegating to the record.
//!
//! Non-functional accessors are NOT stored on the record; the Resolved
//! impl always returns `&[]` for them. Hosts that need real iteration use
//! the per-accessor chain-resolver methods (`resolve_{m}`) generated on
//! `Resolved{Foo}` for functional object accessors.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;

use uor_ontology::model::iris::{OWL_CLASS, OWL_THING, RDF_LIST};
use uor_ontology::model::{Class, Property, PropertyKind};
use uor_ontology::{NamespaceModule, Ontology};

use crate::emit::RustFile;
use crate::mapping::{
    class_trait_path, local_name, to_snake_case, xsd_is_unsized, xsd_to_primitives_type,
    NamespaceMapping,
};

/// Returns the absolute module path (e.g., `crate::bridge::partition`) that
/// holds the given class IRI's emitted code, or `None` for unknown ranges.
fn module_path(class_iri: &str, ns_map: &HashMap<&str, NamespaceMapping>) -> Option<String> {
    let trait_path = class_trait_path(class_iri, ns_map)?;
    let prefix_end = trait_path.rfind("::")?;
    Some(trait_path[..prefix_end].to_string())
}

/// Returns the fully-qualified path to `{Class}{Suffix}` (e.g.,
/// `crate::bridge::partition::PartitionHandle`) for cross-namespace
/// references; for same-namespace classes, returns just `{Class}{Suffix}`.
fn cross_ns_type_path(
    class_iri: &str,
    suffix: &str,
    current_ns_iri: &str,
    ns_map: &HashMap<&str, NamespaceMapping>,
) -> String {
    let local = local_name(class_iri);
    let bare = format!("{local}{suffix}");
    if class_iri.starts_with(current_ns_iri) {
        bare
    } else {
        match module_path(class_iri, ns_map) {
            Some(prefix) => format!("{prefix}::{bare}"),
            None => bare,
        }
    }
}

/// Returns the fully-qualified path to `Null{Class}` for cross-namespace
/// references — used by Resolved impls to return absent sentinels. Honors
/// the hand-written-NullPartition family in `enforcement.rs`.
fn null_type_path(
    class_iri: &str,
    current_ns_iri: &str,
    ns_map: &HashMap<&str, NamespaceMapping>,
) -> String {
    if let Some(existing) = existing_null_class_iris().get(class_iri) {
        return (*existing).to_string();
    }
    cross_ns_type_path(class_iri, "", current_ns_iri, ns_map).replacen(
        local_name(class_iri),
        &format!("Null{}", local_name(class_iri)),
        1,
    )
}

/// Mirrors `crate::traits::existing_null_class_iris` — class IRIs whose Null
/// stubs are hand-written in `enforcement.rs` rather than emitted by the
/// namespace-module codegen.
fn existing_null_class_iris() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "https://uor.foundation/partition/Partition",
        "crate::enforcement::NullPartition",
    );
    m
}

/// Returns every transitive non-Thing supertrait IRI of `class`, excluding
/// enum-class parents.
fn transitive_supertraits<'a>(class: &'a Class, ontology: &'a Ontology) -> Vec<&'a str> {
    let enum_set: HashSet<&'static str> = Ontology::enum_class_names().iter().copied().collect();
    let mut result: Vec<&'a str> = Vec::new();
    let mut frontier: Vec<&'a str> = class.subclass_of.to_vec();
    while let Some(parent_iri) = frontier.pop() {
        if parent_iri == OWL_THING {
            continue;
        }
        if enum_set.contains(local_name(parent_iri)) {
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

/// Phase 8 emission entry point: walks `module.classes`, emitting
/// `{Foo}Handle<H>` + `{Foo}Resolver<H>` + `{Foo}Record<H>` +
/// `Resolved{Foo}<'r, R, H>` for every Path-1 class.
pub fn emit_resolved_wrappers_for_namespace(
    f: &mut RustFile,
    module: &NamespaceModule,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
) {
    let ontology = uor_ontology::Ontology::full();
    let enum_set: HashSet<&'static str> = Ontology::enum_class_names().iter().copied().collect();
    for class in &module.classes {
        if enum_set.contains(local_name(class.id)) {
            continue;
        }
        let path_kind = crate::classification::classify(class, ontology).path_kind;
        // Phase 11: Path-3 (primitive-backed) classes ALSO get the
        // Phase-8 Handle / Resolver / Record / Resolved scaffold —
        // the primitive backing is an additional impl on top, not a
        // substitute. Drop only Skip / AlreadyImplemented / Path-2 /
        // Path-4 from the Phase-8 emission.
        if !matches!(
            path_kind,
            crate::classification::PathKind::Path1HandleResolver
                | crate::classification::PathKind::Path3PrimitiveBacked { .. }
        ) {
            continue;
        }
        let class_local = local_name(class.id);
        emit_handle(f, class_local);
        emit_resolver(f, class_local);
        emit_record(
            f,
            class,
            all_props_by_domain,
            ns_map,
            module.namespace.iri,
            ontology,
        );
        emit_resolved(
            f,
            class,
            ontology,
            all_props_by_domain,
            ns_map,
            module.namespace.iri,
        );
    }
}

fn emit_handle(f: &mut RustFile, class_local: &str) {
    let _ = writeln!(
        f.buf,
        "/// Phase 8 (orphan-closure) — content-addressed handle for `{class_local}<H>`."
    );
    f.line("///");
    f.line("/// Pairs a [`crate::enforcement::ContentFingerprint`] with a phantom");
    f.line("/// `H` so type-state checks can't mix handles across `HostTypes` impls.");
    f.line("#[derive(Debug)]");
    let _ = writeln!(f.buf, "pub struct {class_local}Handle<H: HostTypes> {{");
    f.line("    /// Content fingerprint identifying the resolved record.");
    f.line("    pub fingerprint: crate::enforcement::ContentFingerprint,");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    // Manual Copy / Clone / Eq / PartialEq / Hash impls — `#[derive]`'s
    // auto-bounds would require `H: Copy + Clone + ...`, which we don't
    // want to require on the host type. The fields already satisfy each
    // bound regardless of `H` (via `ContentFingerprint` + `PhantomData<H>`).
    let _ = writeln!(
        f.buf,
        "impl<H: HostTypes> Copy for {class_local}Handle<H> {{}}"
    );
    let _ = writeln!(
        f.buf,
        "impl<H: HostTypes> Clone for {class_local}Handle<H> {{"
    );
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self {");
    f.line("        *self");
    f.line("    }");
    f.line("}");
    let _ = writeln!(
        f.buf,
        "impl<H: HostTypes> PartialEq for {class_local}Handle<H> {{"
    );
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        self.fingerprint == other.fingerprint");
    f.line("    }");
    f.line("}");
    let _ = writeln!(
        f.buf,
        "impl<H: HostTypes> Eq for {class_local}Handle<H> {{}}"
    );
    let _ = writeln!(
        f.buf,
        "impl<H: HostTypes> core::hash::Hash for {class_local}Handle<H> {{"
    );
    f.line("    #[inline]");
    f.line("    fn hash<S: core::hash::Hasher>(&self, state: &mut S) {");
    f.line("        self.fingerprint.hash(state);");
    f.line("    }");
    f.line("}");
    let _ = writeln!(f.buf, "impl<H: HostTypes> {class_local}Handle<H> {{");
    f.line("    /// Construct a handle from its content fingerprint.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(fingerprint: crate::enforcement::ContentFingerprint) -> Self {");
    f.line("        Self {");
    f.line("            fingerprint,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn emit_resolver(f: &mut RustFile, class_local: &str) {
    let _ = writeln!(
        f.buf,
        "/// Phase 8 (orphan-closure) — resolver trait for `{class_local}<H>`."
    );
    f.line("///");
    f.line("/// Hosts implement this trait to map a handle into a typed record.");
    f.line("/// The default Null stub does not implement this trait — it carries");
    f.line("/// no record. Resolution is the responsibility of the host pipeline.");
    let _ = writeln!(f.buf, "pub trait {class_local}Resolver<H: HostTypes> {{");
    f.line("    /// Resolve a handle into its record. Returns `None` when the");
    f.line("    /// handle does not correspond to known content.");
    // rustfmt's `fn_call_width = 60` (and the trait-method line uses the
    // method-call layout) wraps when the inline form exceeds ~60 chars
    // worth of arg list. For the simple `(&self, handle: X) -> Option<Y>`
    // shape, the threshold is roughly 100 chars total — wrap above that.
    let inline = format!(
        "    fn resolve(&self, handle: {class_local}Handle<H>) -> Option<{class_local}Record<H>>;"
    );
    // Empirical rustfmt thresholds (rustc 1.94 default config):
    // - line ≤ 99   → single line
    // - 100..=101   → wrap before `->` (args fit on first line)
    // - ≥ 102       → fully wrap (each arg on its own line)
    if inline.len() <= 99 {
        let _ = writeln!(f.buf, "{inline}");
    } else if inline.len() <= 101 {
        let _ = writeln!(
            f.buf,
            "    fn resolve(&self, handle: {class_local}Handle<H>)"
        );
        let _ = writeln!(f.buf, "        -> Option<{class_local}Record<H>>;");
    } else {
        f.line("    fn resolve(");
        f.line("        &self,");
        let _ = writeln!(f.buf, "        handle: {class_local}Handle<H>,");
        let _ = writeln!(f.buf, "    ) -> Option<{class_local}Record<H>>;");
    }
    f.line("}");
    f.blank();
}

/// Returns true iff the class IRI classifies as Path-1 (has Handle / Resolver /
/// Resolved emitted). Used to gate Record handle fields and chain-resolver
/// methods — non-Path-1 ranges have no Handle to point at.
fn is_path1_range(range_iri: &str, ontology: &Ontology) -> bool {
    match ontology.find_class(range_iri) {
        None => false,
        Some(c) => matches!(
            crate::classification::classify(c, ontology).path_kind,
            crate::classification::PathKind::Path1HandleResolver
        ),
    }
}

/// Returns true iff this property has a per-record field on the Record
/// struct: functional accessors with non-trivial range (i.e., not
/// owl:Thing/owl:Class/rdf:List which all degrade to `&H::HostString`).
fn record_field_for(
    prop: &Property,
    current_ns_iri: &str,
    ns_map: &HashMap<&str, NamespaceMapping>,
    ontology: &Ontology,
) -> Option<RecordField> {
    if prop.kind == PropertyKind::Annotation {
        return None;
    }
    if !prop.functional {
        // Non-functional accessors aren't stored on Record. The Resolved
        // wrapper returns `&[]` for them; iteration goes through chain
        // resolution.
        return None;
    }
    let field_name = to_snake_case(local_name(prop.id));
    match prop.kind {
        PropertyKind::Datatype => {
            if let Some(enum_t) = crate::traits::datatype_enum_override_pub(prop) {
                return Some(RecordField {
                    name: field_name,
                    ty: enum_t.to_string(),
                });
            }
            let prim = xsd_to_primitives_type(prop.range);
            match prim {
                Some(t) => {
                    if xsd_is_unsized(prop.range) {
                        Some(RecordField {
                            name: field_name,
                            ty: format!("&'static {t}"),
                        })
                    } else {
                        Some(RecordField {
                            name: field_name,
                            ty: t.to_string(),
                        })
                    }
                }
                None => Some(RecordField {
                    name: field_name,
                    ty: "&'static H::HostString".to_string(),
                }),
            }
        }
        PropertyKind::Object => {
            let range_local = local_name(prop.range);
            if let Some(enum_name) = crate::traits::object_property_enum_override_pub(range_local) {
                return Some(RecordField {
                    name: field_name,
                    ty: enum_name.to_string(),
                });
            }
            if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
                return Some(RecordField {
                    name: field_name,
                    ty: "&'static H::HostString".to_string(),
                });
            }
            // Only Path-1 ranges have `{Range}Handle<H>` — skip the field
            // for Path-2 (theorem-witness), Path-3 (primitive-backed),
            // Path-4 (theory-deferred), and AlreadyImplemented ranges. The
            // accessor still emits, but as an absent-sentinel return.
            if !is_path1_range(prop.range, ontology) {
                return None;
            }
            let handle_path = cross_ns_type_path(prop.range, "Handle", current_ns_iri, ns_map);
            Some(RecordField {
                name: format!("{field_name}_handle"),
                ty: format!("{handle_path}<H>"),
            })
        }
        PropertyKind::Annotation => None,
    }
}

struct RecordField {
    /// Field name (already snake_case).
    name: String,
    /// Field type as written in the struct definition.
    ty: String,
}

fn emit_record(
    f: &mut RustFile,
    class: &Class,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    ontology: &Ontology,
) {
    let class_local = local_name(class.id);
    let fields =
        collect_record_fields(class, all_props_by_domain, ns_map, current_ns_iri, ontology);

    let _ = writeln!(
        f.buf,
        "/// Phase 8 (orphan-closure) — typed record for `{class_local}<H>`."
    );
    f.line("///");
    f.line("/// Carries a field per functional accessor of the trait. Object");
    f.line("/// fields hold `{Range}Handle<H>`; iterate via the Resolved wrapper");
    f.line("/// chain-resolver methods.");
    f.line("#[derive(Clone, Debug, PartialEq, Eq, Hash)]");
    let _ = writeln!(f.buf, "pub struct {class_local}Record<H: HostTypes> {{");
    for field in &fields {
        let _ = writeln!(f.buf, "    pub {}: {},", field.name, field.ty);
    }
    // Always include the phantom so the struct compiles when its fields
    // don't reference `H` (e.g., a record carrying only `u64` / `bool`
    // scalars). Public-named so hosts can construct via `new {}` without
    // a constructor wrapper if they prefer.
    f.line("    #[doc(hidden)]");
    f.line("    pub _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.blank();
}

fn collect_record_fields(
    class: &Class,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    ontology: &Ontology,
) -> Vec<RecordField> {
    // Record carries fields for the class's OWN direct properties — same
    // filter as trait method generation (no cross-namespace-domain props).
    let mut out: Vec<RecordField> = Vec::new();
    if let Some(props) = all_props_by_domain.get(class.id) {
        for prop in props {
            if !prop.id.starts_with(current_ns_iri) {
                // Cross-namespace-domain: not a trait method, no field.
                continue;
            }
            if let Some(field) = record_field_for(prop, current_ns_iri, ns_map, ontology) {
                out.push(field);
            }
        }
    }
    out
}

fn emit_resolved(
    f: &mut RustFile,
    class: &Class,
    ontology: &Ontology,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
) {
    let class_local = local_name(class.id);
    let resolved_type = format!("Resolved{class_local}");

    let _ = writeln!(
        f.buf,
        "/// Phase 8 (orphan-closure) — content-addressed wrapper for `{class_local}<H>`."
    );
    f.line("///");
    f.line("/// Caches the resolver's lookup at construction. Accessors return");
    f.line("/// the cached record's fields when present, falling back to the");
    f.line("/// `Null{Class}<H>` absent sentinels when the resolver returned");
    f.line("/// `None`. Object accessors always return absent sentinels — use");
    f.line("/// the `resolve_{m}` chain methods to descend into sub-records.");
    let struct_header_line =
        format!("pub struct {resolved_type}<'r, R: {class_local}Resolver<H>, H: HostTypes> {{");
    if struct_header_line.len() > 100 {
        let _ = writeln!(
            f.buf,
            "pub struct {resolved_type}<'r, R: {class_local}Resolver<H>, H: HostTypes>"
        );
        f.line("{");
    } else {
        let _ = writeln!(
            f.buf,
            "pub struct {resolved_type}<'r, R: {class_local}Resolver<H>, H: HostTypes> {{"
        );
    }
    let _ = writeln!(f.buf, "    handle: {class_local}Handle<H>,");
    f.line("    resolver: &'r R,");
    let _ = writeln!(f.buf, "    record: Option<{class_local}Record<H>>,");
    f.line("}");

    let inherent_header_line =
        format!("impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {resolved_type}<'r, R, H> {{");
    if inherent_header_line.len() > 100 {
        let _ = writeln!(f.buf, "impl<'r, R: {class_local}Resolver<H>, H: HostTypes>");
        let _ = writeln!(f.buf, "    {resolved_type}<'r, R, H>");
        f.line("{");
    } else {
        let _ = writeln!(
            f.buf,
            "impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {resolved_type}<'r, R, H> {{"
        );
    }
    f.line("    /// Construct the wrapper, eagerly resolving the handle.");
    f.line("    #[inline]");
    let _ = writeln!(
        f.buf,
        "    pub fn new(handle: {class_local}Handle<H>, resolver: &'r R) -> Self {{"
    );
    f.line("        let record = resolver.resolve(handle);");
    f.line("        Self {");
    f.line("            handle,");
    f.line("            resolver,");
    f.line("            record,");
    f.line("        }");
    f.line("    }");
    f.line("    /// The handle this wrapper resolves.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    let _ = writeln!(
        f.buf,
        "    pub const fn handle(&self) -> {class_local}Handle<H> {{"
    );
    f.line("        self.handle");
    f.line("    }");
    f.line("    /// The resolver supplied at construction.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn resolver(&self) -> &'r R {");
    f.line("        self.resolver");
    f.line("    }");
    f.line("    /// The cached record, or `None` when the resolver returned `None`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    let _ = writeln!(
        f.buf,
        "    pub const fn record(&self) -> Option<&{class_local}Record<H>> {{"
    );
    f.line("        self.record.as_ref()");
    f.line("    }");
    f.line("}");

    // Trait impls — class itself + every transitive supertrait.
    let inherited_for_class =
        crate::traits::collect_inherited_assoc_types_pub(class, all_props_by_domain);
    let parents = transitive_supertraits(class, ontology);
    for parent_iri in parents.iter().rev() {
        let parent_class_opt = ontology.find_class(parent_iri);
        let inherited_for_parent = match parent_class_opt {
            Some(pc) => crate::traits::collect_inherited_assoc_types_pub(pc, all_props_by_domain),
            None => HashSet::new(),
        };
        emit_resolved_impl_for_trait(
            f,
            &resolved_type,
            parent_iri,
            class.id,
            class_local,
            ontology,
            all_props_by_domain,
            ns_map,
            current_ns_iri,
            &inherited_for_parent,
        );
    }
    emit_resolved_impl_for_trait(
        f,
        &resolved_type,
        class.id,
        class.id,
        class_local,
        ontology,
        all_props_by_domain,
        ns_map,
        current_ns_iri,
        &inherited_for_class,
    );

    // Chain-resolver inherent methods — one per functional object accessor
    // across the class + every transitive supertrait.
    emit_chain_resolvers(
        f,
        &resolved_type,
        class,
        ontology,
        all_props_by_domain,
        ns_map,
        current_ns_iri,
    );

    f.blank();
}

#[allow(clippy::too_many_arguments)]
fn emit_resolved_impl_for_trait(
    f: &mut RustFile,
    resolved_type: &str,
    trait_iri: &str,
    class_iri: &str,
    class_local: &str,
    ontology: &Ontology,
    _all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
    inherited_assocs: &HashSet<String>,
) {
    let trait_local = local_name(trait_iri);
    let trait_path = if trait_iri.starts_with(current_ns_iri) {
        trait_local.to_string()
    } else {
        class_trait_path(trait_iri, ns_map).unwrap_or_else(|| trait_local.to_string())
    };

    // Same filter as `emit_null_impl_for_trait`: only properties declared in
    // the same namespace as the trait become trait methods.
    let trait_ns_iri = namespace_of(trait_iri, ns_map);
    let declaring_module = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.iri == trait_ns_iri);
    let direct_props: Vec<&Property> = match declaring_module {
        Some(m) => m
            .properties
            .iter()
            .filter(|p| p.kind != PropertyKind::Annotation && p.domain == Some(trait_iri))
            .collect(),
        None => Vec::new(),
    };

    // Only the class's OWN direct trait impl emits the record-backed bodies;
    // supertrait impls return absent sentinels (because the supertrait's
    // properties don't have record fields on the child class's record). To
    // keep the impl simple, look up record fields from the CLASS's record,
    // not the trait's properties — supertrait properties may not be on the
    // child record.
    let is_own_trait = trait_iri == class_iri;

    let header = format!(
        "impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {trait_path}<H> for {resolved_type}<'r, R, H>"
    );
    // Check the ACTUAL one-liner the codegen would emit: `{header} {` (with
    // a body opening brace) for non-empty impls, or `{header} {}` for empty
    // ones. rustfmt accepts ≤ 100 chars; wrap only when strictly > 100.
    let header_with_body_open = format!("{header} {{");
    let header_with_empty_body = format!("{header} {{}}");
    let header_wraps = header_with_body_open.len() > 100
        || (direct_props.is_empty() && header_with_empty_body.len() > 100);

    // Empty trait → `{}` or wrapped form.
    if direct_props.is_empty() {
        if header_wraps {
            // Mirror rustfmt's `<H> Trait<H>` / `for ...` layout: keep the
            // `<H>` and trait on the same line (the `for ...` clause is what
            // moves to the next line).
            let _ = writeln!(
                f.buf,
                "impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {trait_path}<H>"
            );
            let _ = writeln!(f.buf, "    for {resolved_type}<'r, R, H>");
            f.line("{");
            f.line("}");
        } else {
            let _ = writeln!(f.buf, "{header} {{}}");
        }
        return;
    }

    if header_wraps {
        let _ = writeln!(
            f.buf,
            "impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {trait_path}<H>"
        );
        let _ = writeln!(f.buf, "    for {resolved_type}<'r, R, H>");
        f.line("{");
    } else {
        let _ = writeln!(f.buf, "{header} {{");
    }

    let mut emitted_assoc: HashSet<String> = HashSet::new();
    for prop in &direct_props {
        emit_resolved_method_body(
            f,
            prop,
            trait_local,
            class_local,
            current_ns_iri,
            ns_map,
            ontology,
            is_own_trait,
            &mut emitted_assoc,
            inherited_assocs,
        );
    }
    f.line("}");
}

#[allow(clippy::too_many_arguments)]
fn emit_resolved_method_body(
    f: &mut RustFile,
    prop: &Property,
    owner_trait_name: &str,
    _class_local: &str,
    current_ns_iri: &str,
    ns_map: &HashMap<&str, NamespaceMapping>,
    ontology: &Ontology,
    is_own_trait: bool,
    emitted_assoc: &mut HashSet<String>,
    inherited_assocs: &HashSet<String>,
) {
    let method_name = to_snake_case(local_name(prop.id));

    // Helper for multi-line method emission.
    let emit_fn = |f: &mut RustFile, sig: &str, body: &str| {
        let _ = writeln!(f.buf, "    {sig} {{");
        let _ = writeln!(f.buf, "        {body}");
        f.line("    }");
    };
    // Helper for multi-line `match &self.record { Some(r) => ..., None => ... }`.
    let emit_match_record = |f: &mut RustFile, sig: &str, some_arm: &str, none_arm: &str| {
        let _ = writeln!(f.buf, "    {sig} {{");
        f.line("        match &self.record {");
        let _ = writeln!(f.buf, "            Some(r) => {some_arm},");
        let _ = writeln!(f.buf, "            None => {none_arm},");
        f.line("        }");
        f.line("    }");
    };

    match prop.kind {
        PropertyKind::Datatype => {
            if let Some(enum_t) = crate::traits::datatype_enum_override_pub(prop) {
                if prop.functional {
                    if is_own_trait {
                        emit_match_record(
                            f,
                            &format!("fn {method_name}(&self) -> {enum_t}"),
                            &format!("r.{method_name}"),
                            &format!("<{enum_t}>::default()"),
                        );
                    } else {
                        emit_fn(
                            f,
                            &format!("fn {method_name}(&self) -> {enum_t}"),
                            &format!("<{enum_t}>::default()"),
                        );
                    }
                } else {
                    emit_fn(f, &format!("fn {method_name}(&self) -> &[{enum_t}]"), "&[]");
                }
                return;
            }
            let prim = xsd_to_primitives_type(prop.range);
            match prim {
                Some(t) => {
                    if prop.functional {
                        let empty = match t {
                            "H::HostString" => "H::EMPTY_HOST_STRING".to_string(),
                            "H::WitnessBytes" => "H::EMPTY_WITNESS_BYTES".to_string(),
                            "H::Decimal" => "H::EMPTY_DECIMAL".to_string(),
                            "bool" => "false".to_string(),
                            _ => "0".to_string(),
                        };
                        // Reading from `&self.record` requires the field type
                        // to be Copy. Phase 9's `DecimalTranscendental`
                        // supertrait bound now forces `H::Decimal: Copy`, so
                        // it joins integers/bools/references as a record-
                        // readable type. `H::WitnessBytes` is `?Sized` and
                        // accessed by reference — also OK to read from
                        // record (the field is `&'static H::WitnessBytes`).
                        let owner_copy = true;
                        let read = is_own_trait && owner_copy;
                        if xsd_is_unsized(prop.range) {
                            if read {
                                emit_match_record(
                                    f,
                                    &format!("fn {method_name}(&self) -> &{t}"),
                                    &format!("r.{method_name}"),
                                    &empty,
                                );
                            } else {
                                emit_fn(f, &format!("fn {method_name}(&self) -> &{t}"), &empty);
                            }
                        } else if read {
                            emit_match_record(
                                f,
                                &format!("fn {method_name}(&self) -> {t}"),
                                &format!("r.{method_name}"),
                                &empty,
                            );
                        } else {
                            emit_fn(f, &format!("fn {method_name}(&self) -> {t}"), &empty);
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
            // Enum-typed object range.
            if let Some(enum_name) = crate::traits::object_property_enum_override_pub(range_local) {
                if prop.functional {
                    if is_own_trait {
                        emit_match_record(
                            f,
                            &format!("fn {method_name}(&self) -> {enum_name}"),
                            &format!("r.{method_name}"),
                            &format!("<{enum_name}>::default()"),
                        );
                    } else {
                        emit_fn(
                            f,
                            &format!("fn {method_name}(&self) -> {enum_name}"),
                            &format!("<{enum_name}>::default()"),
                        );
                    }
                } else {
                    emit_fn(
                        f,
                        &format!("fn {method_name}(&self) -> &[{enum_name}]"),
                        "&[]",
                    );
                }
                return;
            }
            if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
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
            // Ontology class range. Use the same Null path as emit_null_method_body.
            let assoc_name = if range_local == owner_trait_name {
                format!("{range_local}Target")
            } else {
                range_local.to_string()
            };
            let null_path = null_type_path(prop.range, current_ns_iri, ns_map);

            // Defensive: only declare assoc when (a) the trait is the one
            // that introduces it (not in `inherited_assocs`) AND (b) we
            // haven't emitted it earlier in this impl block. The Null path
            // is also gated on the range class actually existing in the
            // ontology — `null_type_path` produces a usable name for any
            // emitable range (Path-1/2/4) thanks to Phase 7d.
            let _ = ontology;
            if !inherited_assocs.contains(&assoc_name) && !emitted_assoc.contains(&assoc_name) {
                let _ = writeln!(f.buf, "    type {assoc_name} = {null_path}<H>;");
                emitted_assoc.insert(assoc_name.clone());
            }

            // Resolved always returns the absent sentinel for object accessors.
            // Real iteration uses chain-resolver methods.
            if prop.functional {
                emit_fn(
                    f,
                    &format!("fn {method_name}(&self) -> &Self::{assoc_name}"),
                    &format!("&<{null_path}<H>>::ABSENT"),
                );
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

fn emit_chain_resolvers(
    f: &mut RustFile,
    resolved_type: &str,
    class: &Class,
    ontology: &Ontology,
    all_props_by_domain: &HashMap<&str, Vec<&Property>>,
    ns_map: &HashMap<&str, NamespaceMapping>,
    current_ns_iri: &str,
) {
    let class_local = local_name(class.id);
    let mut chain_methods: Vec<(String, String, String, String)> = Vec::new();
    if let Some(props) = all_props_by_domain.get(class.id) {
        for prop in props {
            if !prop.id.starts_with(current_ns_iri) {
                continue;
            }
            if prop.kind != PropertyKind::Object || !prop.functional {
                continue;
            }
            let range_local = local_name(prop.range);
            if crate::traits::object_property_enum_override_pub(range_local).is_some() {
                continue;
            }
            if prop.range == OWL_THING || prop.range == OWL_CLASS || prop.range == RDF_LIST {
                continue;
            }
            // Only Path-1 ranges have Resolver / Resolved emitted.
            if !is_path1_range(prop.range, ontology) {
                continue;
            }
            let method = to_snake_case(local_name(prop.id));
            let resolver_path = cross_ns_type_path(prop.range, "Resolver", current_ns_iri, ns_map);
            // Build the Resolved type path: same module as the trait, but the
            // class name is prefixed with `Resolved`.
            let resolved_path = if prop.range.starts_with(current_ns_iri) {
                format!("Resolved{range_local}")
            } else {
                match module_path(prop.range, ns_map) {
                    Some(prefix) => format!("{prefix}::Resolved{range_local}"),
                    None => format!("Resolved{range_local}"),
                }
            };
            chain_methods.push((
                method.clone(),
                resolver_path,
                resolved_path,
                format!("{method}_handle"),
            ));
        }
    }

    if chain_methods.is_empty() {
        return;
    }

    let chain_header_line =
        format!("impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {resolved_type}<'r, R, H> {{");
    if chain_header_line.len() > 100 {
        let _ = writeln!(f.buf, "impl<'r, R: {class_local}Resolver<H>, H: HostTypes>");
        let _ = writeln!(f.buf, "    {resolved_type}<'r, R, H>");
        f.line("{");
    } else {
        let _ = writeln!(
            f.buf,
            "impl<'r, R: {class_local}Resolver<H>, H: HostTypes> {resolved_type}<'r, R, H> {{"
        );
    }
    for (method, resolver_path, resolved_path, handle_field) in &chain_methods {
        let _ = writeln!(
            f.buf,
            "    /// Promote the `{method}` handle on the cached record into a"
        );
        f.line("    /// resolved wrapper, given a resolver for the range class.");
        f.line("    /// Returns `None` if no record was resolved at construction.");
        f.line("    #[inline]");
        let _ = writeln!(
            f.buf,
            "    pub fn resolve_{method}<'r2, R2: {resolver_path}<H>>("
        );
        f.line("        &self,");
        f.line("        r: &'r2 R2,");
        let _ = writeln!(f.buf, "    ) -> Option<{resolved_path}<'r2, R2, H>> {{");
        f.line("        let record = self.record.as_ref()?;");
        // rustfmt's `fn_call_width` default is 60 chars — wrap the
        // constructor call accordingly. Compare against the inner call
        // (`{resolved_path}::new(record.{handle_field}, r)`) length.
        let call_only = format!("{resolved_path}::new(record.{handle_field}, r)");
        if call_only.len() > 60 {
            let _ = writeln!(f.buf, "        Some({resolved_path}::new(");
            let _ = writeln!(f.buf, "            record.{handle_field},");
            f.line("            r,");
            f.line("        ))");
        } else {
            let _ = writeln!(
                f.buf,
                "        Some({resolved_path}::new(record.{handle_field}, r))"
            );
        }
        f.line("    }");
    }
    f.line("}");
}

/// Returns the namespace IRI that contains `class_iri`.
fn namespace_of<'a>(class_iri: &'a str, ns_map: &HashMap<&str, NamespaceMapping>) -> &'a str {
    for ns in ns_map.keys() {
        if class_iri.starts_with(*ns) {
            return &class_iri[..ns.len()];
        }
    }
    class_iri
}
