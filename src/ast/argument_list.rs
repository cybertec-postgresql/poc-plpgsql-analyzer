// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for an argument list and its arguments.

use crate::ast::AstNode;

use super::typed_syntax_node;

typed_syntax_node!(ArgumentList, Argument);

impl ArgumentList {
    pub fn arguments(&self) -> Vec<Argument> {
        self.syntax()
            .children()
            .filter_map(Argument::cast)
            .collect::<Vec<Argument>>()
    }
}

impl Argument {
    pub fn text(&self) -> String {
        self.syntax.text().to_string()
    }
}
