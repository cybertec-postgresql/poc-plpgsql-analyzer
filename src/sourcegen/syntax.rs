// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generates the [`SyntaxKind`] enum and the mapping of the [`TokenKind`] enum

#[cfg(test)]
pub mod lib {
    use heck::ToUpperCamelCase;
    use proc_macro2::{Ident, TokenStream};
    use quote::{format_ident, quote};

    use crate::sourcegen::data::lib::SYNTAX_NODES;
    use crate::sourcegen::data::lib::TOKENS;
    use crate::sourcegen::lib::{
        add_preamble, guarantee_file_content, project_path, rustfmt_content,
    };

    pub struct SyntaxNode<'a> {
        pub name: &'a str,
        pub explanation: &'a str,
        pub tokens: &'a [&'a str],
    }

    impl SyntaxNode<'_> {
        fn to_ident(&self) -> Ident {
            format_ident!("{}", self.name.to_upper_camel_case(),)
        }

        fn to_enum_variant(&self) -> TokenStream {
            let ident = self.to_ident();
            let doc = self.explanation;
            quote! {
                #[doc = #doc]
                #ident,
            }
        }
    }

    macro_rules! S {
        ($name:literal, $explanation:literal) => {
            SyntaxNode {
                name: $name,
                explanation: $explanation,
                tokens: &[],
            }
        };
        ($name:literal, $explanation:literal, $tokens:expr) => {
            SyntaxNode {
                name: $name,
                explanation: $explanation,
                tokens: $tokens,
            }
        };
    }
    pub(crate) use S;

    #[test]
    fn sourcegen_syntax() {
        let file = project_path().join("src/syntax/generated.rs");
        let content = rustfmt_content(add_preamble(file!(), generate_content()));
        guarantee_file_content(&file, content.as_str());
    }

    fn generate_content() -> String {
        // All variants of the `SyntaxKind` enum
        let syntax_nodes: TokenStream = SYNTAX_NODES.iter().map(|t| t.to_enum_variant()).collect();

        let tokens: TokenStream = TOKENS
            .iter()
            .map(|t| {
                let token = t.to_ident();
                let syntax = if let Some(syntax_kind) = t.syntax_kind {
                    format_ident!("{}", syntax_kind.to_upper_camel_case())
                } else {
                    format_ident!("Keyword")
                };
                quote! { TokenKind::#token => SyntaxKind::#syntax, }
            })
            .collect();

        let content = quote! {
            use num_derive::{FromPrimitive, ToPrimitive};
            use num_traits::ToPrimitive;

            use crate::lexer::TokenKind;

            /// Represents all possible kind of syntax items the parser can process.
            ///
            /// Examples
            /// * <https://blog.kiranshila.com/blog/easy_cst.md>
            /// * <https://arzg.github.io/lang/10/>
            /// * <https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs>
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
            #[repr(u16)]
            pub enum SyntaxKind {
                #syntax_nodes
            }

            impl From<SyntaxKind> for rowan::SyntaxKind {
                fn from(kind: SyntaxKind) -> Self {
                    rowan::SyntaxKind(kind.to_u16().unwrap())
                }
            }

            impl From<TokenKind> for SyntaxKind {
                fn from(kind: TokenKind) -> Self {
                    match kind {
                        #tokens
                        TokenKind::Error => SyntaxKind::Error,
                        TokenKind::Eof => unreachable!(),
                    }
                }
            }
        };
        content.to_string()
    }
}
