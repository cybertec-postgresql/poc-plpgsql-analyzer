use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::from_utf8;

use definitions::data::SYNTAX_NODES;
use definitions::data::TOKENS;
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use quote::quote;

/// Adds the license header and a edit warning preamble
pub fn add_preamble(generator: &str, mut content: String) -> String {
    let preamble = format!(
        r#"// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generated by `{generator}`, do not edit manually.

"#
    );
    content.insert_str(0, &preamble);
    content
}

/// Verifies that the `file` has the specified `content`.
/// If not, it updates the file and fails the test.
pub fn guarantee_file_content(file: &Path, content: &str) {
    // Check if the file is up-to-date.
    if let Ok(old_content) = fs::read_to_string(file) {
        if old_content == content {
            return;
        }
    }

    fs::write(file, content).unwrap();
    println!("cargo:warning=A file was not up-to-date and has been updated!");
}

/// Formats the specified `content` using the `rustfmt` binary
pub fn rustfmt_content(content: String) -> String {
    let config_path = project_path().join(".rustfmt.toml");

    let mut cmd = Command::new("rustfmt")
        .arg("--config-path")
        .arg(config_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn rustfmt");

    let cmd_stdin = cmd.stdin.as_mut().unwrap();
    cmd_stdin.write_all(content.as_bytes()).unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());
    from_utf8(&output.stdout).unwrap().to_owned()
}

/// Returns the path of the project root
pub fn project_path() -> PathBuf {
    let path = std::env::current_dir().unwrap();
    // let path = path.parent().unwrap().parent().unwrap().parent().unwrap();

    assert!(path.join("Cargo.toml").exists());

    path.to_owned()
}

mod syntax {
    use super::*;

    pub fn sourcegen_syntax() {
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

mod lexer {
    use super::*;

    pub fn sourcegen_token() {
        let file = project_path().join("src/lexer/generated.rs");
        let content = rustfmt_content(add_preamble(file!(), generate_content()));
        guarantee_file_content(&file, content.as_str());
    }

    fn generate_content() -> String {
        let token_kind_enum = generate_token_kind_enum();
        let token_kind_impl = generate_token_kind_impl();
        let token_kind_macro = generate_token_kind_macro();

        let content = quote! {
            #token_kind_enum
            #token_kind_impl
            #token_kind_macro
        };
        content.to_string()
    }

    fn generate_token_kind_enum() -> TokenStream {
        let tokens: TokenStream = TOKENS.iter().map(|t| t.to_enum_variant()).collect();

        quote! {
            #[derive(logos::Logos, Debug, Copy, Clone, PartialEq, Eq)]
            pub enum TokenKind {
                #tokens

                Error,

                /// Marker token to indicate end of input, not used by lexer directly.
                Eof,
            }
        }
    }

    fn generate_token_kind_impl() -> TokenStream {
        let trivia: Vec<Ident> = TOKENS.trivia.iter().map(|t| t.to_ident()).collect();
        let punctuation: Vec<Ident> = TOKENS.punctuation.iter().map(|t| t.to_ident()).collect();
        let literals: Vec<Ident> = TOKENS.literals.iter().map(|t| t.to_ident()).collect();
        quote! {
            impl TokenKind {
                pub fn is_trivia(self) -> bool {
                    matches!(self, #( Self :: #trivia )|*)
                }

                pub fn is_punct(self) -> bool {
                    matches!(self, #(Self::#punctuation)|*)
                }

                pub fn is_literal(self) -> bool {
                    matches!(self, #(Self::#literals)|*)
                }

                pub fn is_ident(self) -> bool {
                    matches!(self, Self::UnquotedIdent | Self::QuotedIdent | Self::BindVar)
                        || !(self.is_trivia()
                            || self.is_punct()
                            || self.is_literal()
                            || matches!(self, Self::Eof | Self::Error))
                }
            }

            impl std::fmt::Display for TokenKind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{self:?}")
                }
            }
        }
    }

    fn generate_token_kind_macro() -> TokenStream {
        let rules: TokenStream = TOKENS.iter().map(|t| t.to_macro_variant()).collect();

        quote! {
            #[macro_export]
            macro_rules! T {
                #rules
                [EOF] => { TokenKind::Eof };
            }
        }
    }
}

fn main() -> () {
    lexer::sourcegen_token();
    syntax::sourcegen_syntax();
}
