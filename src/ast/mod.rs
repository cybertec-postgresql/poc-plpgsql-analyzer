// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements a typed AST for PL/SQL.

mod procedure;

use crate::{syntax::SyntaxToken, SyntaxKind, SyntaxNode};
pub use procedure::*;
pub use rowan::ast::AstNode;

macro_rules! typed_syntax {
    ($synty:ty, $astty:ty, $name:ident $(; { $( $additional:item )+ } )? ) => {
        pub struct $name {
            pub(crate) syntax: $synty,
        }

        impl $astty for $name {
            $( $( $additional )+ )?

            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$name
            }

            fn cast(syntax: $synty) -> Option<Self> {
                if Self::can_cast(syntax.kind()) {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &$synty {
                &self.syntax
            }
        }
    };
}

/// Automatically generate `struct`s and implementation of the [`AstNode`] trait
/// for [`SyntaxKind`] variants.
macro_rules! typed_syntax_node {
    ($( $name:ident ),+ $(,)?) => {
        $(
            crate::ast::typed_syntax!(SyntaxNode, AstNode, $name; {
                type Language = crate::syntax::SqlProcedureLang;
            });
        )+
    };
}

/// Automatically generate `struct`s and implementation of the [`AstToken`] trait
/// for [`SyntaxKind`] variants.
macro_rules! typed_syntax_token {
    ($( $name:ident ),+ $(,)?) => {
        $( crate::ast::typed_syntax!(SyntaxToken, AstToken, $name); )+
    };
}

// Needed so that submodules can import [`typed_syntax_node`] and [`typed_syntax_token`]
/// as `super::typed_syntax_{node,token}`.
use {typed_syntax, typed_syntax_node, typed_syntax_token};

/// Represents a interface for typed AST tokens, akin to [`AstNode`].
pub trait AstToken {
    /// Returns whether the passed [`SyntaxKind`] can be casted to this type of token or
    /// not.
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    /// Tries to cast the passed (generic) token to a typed token. Might
    /// fail if the syntax kind is not compatible (see [`can_cast()`](Self::can_cast())).
    fn cast(token: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    /// Returns the [`SyntaxToken`] for this typed node.
    fn syntax(&self) -> &SyntaxToken;

    /// Returns the original representation of the token.
    fn text(&self) -> &str {
        self.syntax().text()
    }
}

typed_syntax_node!(Root);

impl Root {
    /// Finds the (next) procedure in this root node.
    pub fn procedure(&self) -> Option<Procedure> {
        self.syntax.children().find_map(Procedure::cast)
    }
}
