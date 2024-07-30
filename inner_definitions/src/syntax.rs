// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generates the [`crate::SyntaxKind`] enum and the mapping of the [`crate::TokenKind`] enum

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::data::SYNTAX_NODES;
use crate::data::TOKENS;

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
