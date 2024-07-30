// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL datatypes.

use crate::ast::IdentGroup;
use rowan::ast::AstNode;
use source_gen::syntax::SyntaxKind;

use super::typed_syntax_node;

typed_syntax_node!(Datatype);

impl Datatype {
    /// Returns the identifiers referenced by the %TYPE attribute of the datatype.
    pub fn referenced_type(&self) -> Option<IdentGroup> {
        let type_attribute_exists = self
            .syntax
            .children()
            .find(|t| t.kind() == SyntaxKind::TypeAttribute);

        match type_attribute_exists {
            Some(_) => self
                .syntax
                .children()
                .find(|t| t.kind() != SyntaxKind::TypeAttribute)
                .map(IdentGroup::cast)?,
            None => None,
        }
    }
}
