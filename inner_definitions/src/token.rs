// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generates the [`crate::TokenKind`] enum

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Literal, Punct, Spacing, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::{format_ident, quote};

use crate::data::TOKENS;

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
        crate::token::Token {
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
