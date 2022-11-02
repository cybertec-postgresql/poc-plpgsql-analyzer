// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements a typed AST for PL/SQL.

mod expressions;
mod function;
mod procedure;
mod query;

use crate::syntax::{SyntaxKind, SyntaxToken};
pub use expressions::*;
pub use function::*;
pub use procedure::*;
pub use query::*;
pub use rowan::ast::AstNode;

macro_rules! typed_syntax {
    ($synty:ty, $astty:ty, $name:ident $(; { $( $additional:item )+ } )? ) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $name {
            pub(crate) syntax: $synty,
        }

        impl $astty for $name {
            $( $( $additional )+ )?

            fn can_cast(kind: crate::syntax::SyntaxKind) -> bool {
                kind == crate::syntax::SyntaxKind::$name
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
            crate::ast::typed_syntax!(crate::syntax::SyntaxNode, crate::ast::AstNode, $name; {
                type Language = crate::syntax::SqlProcedureLang;
            });
        )+
    };
}

/// Automatically generate `struct`s and implementation of the [`AstToken`]
/// trait for [`SyntaxKind`] variants.
macro_rules! typed_syntax_token {
    ($( $name:ident ),+ $(,)?) => {
        $( crate::ast::typed_syntax!(crate::syntax::SyntaxToken, crate::ast::AstToken, $name); )+
    };
}

// Needed so that submodules can import [`typed_syntax_node`] and
// [`typed_syntax_token`]
/// as `super::typed_syntax_{node,token}`.
pub(self) use {typed_syntax, typed_syntax_node, typed_syntax_token};

/// Represents a interface for typed AST tokens, akin to [`AstNode`].
pub trait AstToken {
    /// Returns whether the passed [`SyntaxKind`] can be casted to this type of
    /// token or not.
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    /// Tries to cast the passed (generic) token to a typed token. Might
    /// fail if the syntax kind is not compatible (see
    /// [`can_cast()`](Self::can_cast())).
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
typed_syntax_token!(Ident, ComparisonOp);

impl Root {
    /// Finds the (next) function in this root node.
    pub fn function(&self) -> Option<Function> {
        self.syntax.children().find_map(Function::cast)
    }

    /// Finds the (next) procedure in this root node.
    pub fn procedure(&self) -> Option<Procedure> {
        self.syntax.children().find_map(Procedure::cast)
    }

    /// Finds the (next) `SELECT` query in this root node.
    pub fn query(&self) -> Option<SelectStmt> {
        self.syntax.children().find_map(SelectStmt::cast)
    }
}

impl Ident {
    /// Returns the identifier name itself.
    pub fn name(&self) -> String {
        self.syntax.text().to_string()
    }
}
