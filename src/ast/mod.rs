// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements a typed AST for PL/SQL.

pub use rowan::ast::AstNode;

pub use datatype::*;
pub use expressions::*;
pub use function::*;
pub use function_invocation::*;
pub use procedure::*;
pub use query::*;
pub use trigger::*;

use crate::syntax::{SyntaxKind, SyntaxToken};

mod datatype;
mod expressions;
mod function;
mod function_invocation;
mod procedure;
mod query;
mod trigger;

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
pub(crate) use {typed_syntax, typed_syntax_node, typed_syntax_token};

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

typed_syntax_node!(Root, IdentGroup, ParamList, Param, Block);
typed_syntax_token!(ComparisonOp, Ident);

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

    /// Finds the (next) trigger query in this root node.
    pub fn trigger(&self) -> Option<Trigger> {
        self.syntax.children().find_map(Trigger::cast)
    }
}

impl IdentGroup {
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .filter(|it| it.kind() == SyntaxKind::Ident || it.kind() == SyntaxKind::Dot)
            .map(|it| Some(it.text().to_string()))
            .collect()
    }

    // TODO: implement a `last_nth` method
    pub fn nth(&self, n: usize) -> Option<Ident> {
        self.syntax
            .children_with_tokens()
            .filter_map(|t| t.into_token())
            .filter_map(Ident::cast)
            .nth(n)
    }
}

impl Ident {
    /// Returns the full identifier name itself.
    pub fn text(&self) -> String {
        self.syntax.text().to_string()
    }
}

impl ParamList {
    pub fn params(&self) -> Vec<Param> {
        self.syntax.children().filter_map(Param::cast).collect()
    }
}

impl Param {
    #[allow(unused)]
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Ident::cast)
            .map(|id| id.text())
    }

    pub fn datatype(&self) -> Option<Datatype> {
        self.syntax.children().find_map(Datatype::cast)
    }

    pub fn type_reference(&self) -> Option<IdentGroup> {
        self.datatype()?.referenced_type()
    }
}

impl Block {
    pub fn text(&self) -> String {
        self.syntax.text().to_string()
    }
}
