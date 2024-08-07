// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements a typed AST for PL/SQL.

use cursor::CursorStmt;
pub use rowan::ast::AstNode;

pub use argument_list::*;
pub use datatype::*;
pub use dml::*;
pub use expressions::*;
pub use function::*;
pub use function_invocation::*;
pub use procedure::*;
pub use query::*;
pub use trigger::*;
pub use view::*;

use source_gen::syntax::{SyntaxKind, SyntaxToken};

mod argument_list;
mod cursor;
mod datatype;
mod dml;
mod expressions;
mod function;
mod function_invocation;
mod procedure;
mod query;
mod trigger;
mod view;

macro_rules! typed_syntax {
    ($synty:ty, $astty:ty, $name:ident $(; { $( $additional:item )+ } )? ) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $name {
            pub(crate) syntax: $synty,
        }

        impl $astty for $name {
            $( $( $additional )+ )?

            fn can_cast(kind: source_gen::syntax::SyntaxKind) -> bool {
                kind == source_gen::syntax::SyntaxKind::$name
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
            crate::ast::typed_syntax!(source_gen::syntax::SyntaxNode, crate::ast::AstNode, $name; {
                type Language = source_gen::syntax::SqlProcedureLang;
            });
        )+
    };
}

/// Automatically generate `struct`s and implementation of the [`AstToken`]
/// trait for [`SyntaxKind`] variants.
macro_rules! typed_syntax_token {
    ($( $name:ident ),+ $(,)?) => {
        $( crate::ast::typed_syntax!(source_gen::syntax::SyntaxToken, crate::ast::AstToken, $name); )+
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
    pub fn dml(&self) -> Option<DeleteStmt> {
        self.syntax.children().find_map(DeleteStmt::cast)
    }

    pub fn cursor(&self) -> Option<CursorStmt> {
        self.syntax.children().find_map(CursorStmt::cast)
    }

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

    /// Finds the (next) view in this root node.
    pub fn view(&self) -> Option<View> {
        self.syntax.children().find_map(View::cast)
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
