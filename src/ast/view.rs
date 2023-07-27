// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL views.

use crate::ast::{AstNode, IdentGroup};

use super::typed_syntax_node;

typed_syntax_node!(View);

impl View {
    /// Returns the name of the view.
    pub fn name(&self) -> Option<String> {
        self.syntax.children().find_map(IdentGroup::cast)?.name()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Root;

    use super::*;

    #[test]
    fn check_ast_node_to_view() {
        const INPUT: &str = "CREATE VIEW store_view AS SELECT name FROM stores";
        let result = crate::parse_view(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let view = root.unwrap().view();
        assert!(view.is_some());
        assert_eq!(view.unwrap().name(), Some("store_view".to_string()));
    }
}
