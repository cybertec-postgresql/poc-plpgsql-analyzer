// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements a typed AST for PL/SQL.

mod procedure;

use crate::{syntax::SyntaxToken, SyntaxKind, SyntaxNode};
pub use procedure::*;

/// Represents a interface for typed AST nodes.
pub trait AstNode {
    /// Returns whether the passed [`SyntaxKind`] can be casted to this type of node or
    /// not.
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    /// Tries to cast the passed (generic) node to a typed node. Might
    /// fail if the syntax kind is not compatible (see [`can_cast()`](Self::can_cast())).
    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    /// Returns the [`SyntaxNode`] for this typed node.
    fn syntax(&self) -> &SyntaxNode;
}

/// Represents a interface for typed AST tokens.
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

/// Represents the root node for a parsed input.
#[derive(Debug)]
pub struct Root {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Root {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Root
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl Root {
    /// Finds the (next) procedure in this root node.
    pub fn procedure(&self) -> Option<Procedure> {
        self.syntax.children().find_map(Procedure::cast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_root_ast_node() {
        const INPUT: &str = r#"
            CREATE OR REPLACE PROCEDURE multiple_parameters(
                p1 VARCHAR2
                , p2 VARCHAR2
            )
            IS
            BEGIN
                NULL;
            END multiple_parameters;
        "#;
        let result = crate::parse(INPUT).unwrap();
        let root = result.syntax();
        assert!(Root::cast(root).is_some());
    }
}
