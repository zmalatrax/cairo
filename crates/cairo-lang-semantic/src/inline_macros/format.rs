use cairo_lang_defs::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_defs::plugin::{
    InlineMacroExprPlugin, InlinePluginResult, PluginDiagnostic, PluginGeneratedFile,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};
use itertools::Itertools;

use super::unsupported_bracket_diagnostic;
use crate::extract_macro_unnamed_args;

/// Macro for formatting.
#[derive(Debug, Default)]
pub struct FormatMacro;
impl FormatMacro {
    pub const NAME: &'static str = "format";
}
impl InlineMacroExprPlugin for FormatMacro {
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
            [ast::ArgClause::Unnamed(arg)] => {
                builder.add_modified(RewriteNode::interpolate_patched(
                    "{ core::fmt::display_format(@$arg$) }",
                    &[("arg".to_string(), RewriteNode::new_trimmed(arg.as_syntax_node()))].into(),
                ));
            }
            [ast::ArgClause::Unnamed(format_string), ast::ArgClause::Unnamed(arg)] => {
                if format_string.as_syntax_node().get_text(db) != r#""{}""# {
                    let diagnostics = vec![PluginDiagnostic {
                        stable_ptr: syntax.stable_ptr().untyped(),
                        message: format!(
                            r#"Currently `{}` macro only supports `"{{}}"` as the first argument (format string)."#,
                            FormatMacro::NAME
                        ),
                    }];
                    return InlinePluginResult { code: None, diagnostics };
                }

                builder.add_modified(RewriteNode::interpolate_patched(
                    "{ core::fmt::display_format(@$arg$) }",
                    &[("arg".to_string(), RewriteNode::new_trimmed(arg.as_syntax_node()))].into(),
                ));
            }
            _ => {
                let diagnostics = vec![PluginDiagnostic {
                    stable_ptr: syntax.stable_ptr().untyped(),
                    message: format!(
                        "`{}` macro can have up to 2 unnamed arguments.",
                        FormatMacro::NAME
                    ),
                }];
                return InlinePluginResult { code: None, diagnostics };
            }
        }

        // TODO(yuval): allow more

        InlinePluginResult {
            code: Some(PluginGeneratedFile {
                name: "format_macro".into(),
                content: builder.code,
                diagnostics_mappings: builder.diagnostics_mappings,
                aux_data: None,
            }),
            diagnostics: vec![],
        }
    }
}
