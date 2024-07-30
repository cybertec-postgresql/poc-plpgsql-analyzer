// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generates the [`crate::TokenKind`] enum

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Literal, Punct, Spacing, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::{format_ident, quote};

use crate::data::TOKENS;
use crate::{add_preamble, guarantee_file_content, project_path, rustfmt_content};

pub struct Token<'a> {
    pub shorthand: &'a str,
    pub name: &'a str,
    pub syntax_kind: Option<&'a str>,
    pub regex: Option<&'a str>,
    pub priority: Option<u8>,
}

impl Token<'_> {
    pub fn is_keyword(&self) -> bool {
        self.shorthand == self.name && self.regex.is_none()
    }

    pub fn to_ident(&self) -> Ident {
        format_ident!(
            "{}{}",
            self.name.to_upper_camel_case(),
            if self.is_keyword() { "Kw" } else { "" }
        )
    }

    fn to_enum_variant(&self) -> TokenStream {
        let priority = if let Some(priority) = self.priority {
            let priority = Literal::u8_unsuffixed(priority);
            quote!(, priority = #priority)
        } else {
            quote!()
        };

        let logos_macro = if let Some(regex) = self.regex {
            quote!(#[regex(#regex #priority)])
        } else {
            let token = self.shorthand;
            quote!(#[token(#token, ignore(case) #priority)])
        };

        let ident = self.to_ident();
        quote! {
            #logos_macro
            #ident,
        }
    }

    fn to_macro_variant(&self) -> TokenStream {
        let rule = {
            if self.shorthand == "$$" {
                quote! {"$$"}
            } else if "()".contains(self.shorthand) {
                let char = self.shorthand.next().unwrap();
                quote! {#char}
            } else {
                let char = self
                    .shorthand
                    .chars()
                    .map(|c| Punct::new(c, Spacing::Joint));
                quote! { #(#char)* }
            }
        };
        let variant = self.to_ident();
        quote! {
            [#rule] => { TokenKind::#variant };
        }
    }
}

macro_rules! T {
    ($shorthand:literal) => {
        T!($shorthand, $shorthand, None, None, None)
    };
    ($shorthand:literal, $name:literal) => {
        T!($shorthand, $name, None, None, None)
    };
    ($shorthand:literal, $name:literal, $syntax_kind:literal) => {
        T!($shorthand, $name, Some($syntax_kind), None, None)
    };
    ($shorthand:literal, $name:literal, $syntax_kind:literal, $regex:literal) => {
        T!($shorthand, $name, Some($syntax_kind), Some($regex), None)
    };
    ($shorthand:literal, $name:literal, $syntax_kind:literal, $regex:literal, $priority:literal) => {
        T!(
            $shorthand,
            $name,
            Some($syntax_kind),
            Some($regex),
            Some($priority)
        )
    };
    ($shorthand:literal, $name:literal, $syntax_kind:expr, $regex:expr, $priority:expr) => {
        crate::sourcegen::token::lib::Token {
            shorthand: $shorthand,
            name: $name,
            syntax_kind: $syntax_kind,
            regex: $regex,
            priority: $priority,
        }
    };
}
pub(crate) use T;

pub(crate) struct Tokens<'a> {
    pub(crate) trivia: &'a [Token<'a>],
    pub(crate) punctuation: &'a [Token<'a>],
    pub(crate) literals: &'a [Token<'a>],
    pub(crate) keywords: &'a [Token<'a>],
}

impl Tokens<'static> {
    pub fn iter(&self) -> impl Iterator<Item = &'static Token<'static>> {
        TOKENS
            .trivia
            .iter()
            .chain(TOKENS.punctuation.iter())
            .chain(TOKENS.literals.iter())
            .chain(TOKENS.keywords.iter())
    }
}

fn sourcegen_token() {
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

            #[error]
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
        macro_rules! T {
            #rules
            [EOF] => { TokenKind::Eof };
        }
        pub(crate) use T;
    }
}
