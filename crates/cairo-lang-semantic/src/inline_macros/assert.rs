use cairo_lang_defs::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_defs::plugin::{
    InlineMacroExprPlugin, InlinePluginResult, PluginDiagnostic, PluginGeneratedFile,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};
use cairo_lang_utils::try_extract_matches;
use itertools::Itertools;

use super::unsupported_bracket_diagnostic;

/// Macro for asserting conditions.
#[derive(Debug, Default)]
pub struct AssertMacro;
impl AssertMacro {
    pub const NAME: &'static str = "assert";
}
impl InlineMacroExprPlugin for AssertMacro {
    fn generate_code(
        &self,
        db: &dyn SyntaxGroup,
        syntax: &ast::ExprInlineMacro,
    ) -> InlinePluginResult {
        let macro_args =
            if let ast::WrappedArgList::ParenthesizedArgList(args) = syntax.arguments(db) {
                args.arguments(db)
            } else {
                return unsupported_bracket_diagnostic(db, syntax);
            };

        let arguments = macro_args.elements(db).iter().map(|arg| arg.arg_clause(db)).collect_vec();
        let mut builder = PatchBuilder::new(db);
        match arguments.as_slice() {
            [ast::ArgClause::Unnamed(condition), tail @ ..] => {
                let Some(tail) = tail
                    .iter()
                    .map(|arg| {
                        Some(RewriteNode::new_trimmed(
                            try_extract_matches!(arg, ast::ArgClause::Unnamed)?.as_syntax_node(),
                        ))
                    })
                    .collect::<Option<Vec<_>>>()
                else {
                    let diagnostics = vec![PluginDiagnostic {
                        stable_ptr: syntax.stable_ptr().untyped(),
                        message: format!(
                            "All arguments of the `{}` macro must be unnamed.",
                            AssertMacro::NAME
                        ),
                    }];
                    return InlinePluginResult { code: None, diagnostics };
                };
                if tail.len() == 0 {
                    builder.add_modified(RewriteNode::interpolate_patched(
                        r#"{
                            if !($condition$) {
                                let ba: ByteArray = "Assertion failed";
                                panic!(ba);
                            }
                        }"#,
                        &[(
                            "condition".to_string(),
                            RewriteNode::new_trimmed(condition.as_syntax_node()),
                        )]
                        .into(),
                    ));
                } else {
                    let panic_args =
                        RewriteNode::interspersed(tail.into_iter(), RewriteNode::text(", "));
                    builder.add_modified(RewriteNode::interpolate_patched(
                        r#"{
                            if !($condition$) {
                                let ba: ByteArray = "Assertion failed: " + format!($panic_args$);
                                panic!(ba);
                            }
                        }"#,
                        &[
                            (
                                "condition".to_string(),
                                RewriteNode::new_trimmed(condition.as_syntax_node()),
                            ),
                            ("panic_args".to_string(), panic_args),
                        ]
                        .into(),
                    ));
                }
            }
            _ => {
                let diagnostics = vec![PluginDiagnostic {
                    stable_ptr: syntax.stable_ptr().untyped(),
                    message: format!(
                        "`{}` macro must have at least one unnamed argument (condition).",
                        AssertMacro::NAME
                    ),
                }];
                return InlinePluginResult { code: None, diagnostics };
            }
        }

        InlinePluginResult {
            code: Some(PluginGeneratedFile {
                name: "print_macro".into(),
                content: builder.code,
                diagnostics_mappings: builder.diagnostics_mappings,
                aux_data: None,
            }),
            diagnostics: vec![],
        }
    }
}
