use cairo_lang_debug::DebugWithDb;
use cairo_lang_defs::ids::{ImplDefId, TraitId};
use cairo_lang_diagnostics::Maybe;
use cairo_lang_syntax::node::ast::ExprPath;
use cairo_lang_utils::try_extract_matches;
use itertools::Itertools;

use crate::db::SemanticGroup;
use crate::diagnostic::SemanticDiagnosticKind::NotATrait;
use crate::diagnostic::{NotFoundItemType, SemanticDiagnostics};
use crate::resolve::{ResolvedGenericItem, Resolver};
use crate::GenericParam;

pub mod attribute;
pub mod constant;
pub mod enm;
pub mod extern_function;
pub mod extern_type;
pub mod fmt;
pub mod free_function;
pub mod function_with_body;
pub mod functions;
pub mod generics;
pub mod imp;
pub mod impl_alias;
pub mod modifiers;
pub mod module;
pub mod module_type_alias;
pub mod structure;
pub mod trt;
pub mod type_aliases;
pub mod us;
pub mod visibility;

#[cfg(test)]
mod test;

/// Tries to resolve a trait path. Reports a diagnostic if the path doesn't point to a trait.
fn resolve_trait_path(
    diagnostics: &mut SemanticDiagnostics,
    resolver: &mut Resolver<'_>,
    trait_path_syntax: &ExprPath,
) -> Maybe<TraitId> {
    try_extract_matches!(
        resolver.resolve_generic_path_with_args(
            diagnostics,
            trait_path_syntax,
            NotFoundItemType::Trait,
        )?,
        ResolvedGenericItem::Trait
    )
    .ok_or_else(|| diagnostics.report(trait_path_syntax, NotATrait))
}

/// A context of a trait, if in a trait. This is used in the resolver to resolve
/// "Self::" paths.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TraitContext {
    pub trait_id: TraitId,
    pub generic_parameters: Vec<GenericParam>,
}
impl DebugWithDb<dyn SemanticGroup> for TraitContext {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &(dyn SemanticGroup + 'static),
    ) -> std::fmt::Result {
        // TODO(yg): fix
        write!(f, "{:?}::<", db.lookup_intern_trait(self.trait_id).debug(db));
        self.generic_parameters.iter().map(|p| {
            p.fmt(f, db);
            write!(f, ", ");
        });
        write!(f, ">")
    }
}

/// A context of an impl, if in an impl. This is used in the resolver to resolve
/// "Self::" paths and in implizations.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImplContext {
    pub impl_def_id: ImplDefId,
    pub generic_parameters: Vec<GenericParam>,
}
impl DebugWithDb<dyn SemanticGroup> for ImplContext {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &(dyn SemanticGroup + 'static),
    ) -> std::fmt::Result {
        // TODO(yg): fix
        write!(f, "{:?}::<", db.lookup_intern_impl(self.impl_def_id).debug(db),);
        self.generic_parameters.iter().map(|p| {
            p.fmt(f, db);
            write!(f, ", ");
        });
        write!(f, ">")
    }
}

/// A context of a trait or an impl, if in any of those. This is used in the resolver to resolve
/// "Self::" paths.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TraitOrImplContext {
    /// No trait/impl context.
    None,
    /// The context is of a trait.
    Trait(TraitContext),
    /// The context is of an impl.
    Impl(ImplContext),
}
impl TraitOrImplContext {
    // TODO(yg): for both, consider returning a reference, and panic if wrong.
    /// Returns the context as a trait context, if the context is indeed a trait context, or None
    /// otherwise.
    pub fn trait_context(&self) -> Option<TraitContext> {
        if let TraitOrImplContext::Trait(ctx) = self { Some(ctx.clone()) } else { None }
    }
    /// Returns the context as an impl context, if the context is indeed an impl context, or None
    /// otherwise.
    pub fn impl_context(&self) -> Option<ImplContext> {
        if let TraitOrImplContext::Impl(ctx) = self { Some(ctx.clone()) } else { None }
    }
}
impl DebugWithDb<dyn SemanticGroup> for TraitOrImplContext {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &(dyn SemanticGroup + 'static),
    ) -> std::fmt::Result {
        match self {
            TraitOrImplContext::None => write!(f, "None"),
            TraitOrImplContext::Trait(trait_ctx) => write!(f, "{:?}", trait_ctx.debug(db)),
            TraitOrImplContext::Impl(impl_ctx) => write!(f, "{:?}", impl_ctx.debug(db)),
        }
    }
}
