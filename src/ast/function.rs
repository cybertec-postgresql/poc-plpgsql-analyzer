// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL functions.

use crate::ast::{AstNode, Block, IdentGroup, ParamList};

use super::typed_syntax_node;

typed_syntax_node!(Function, FunctionHeader);

impl Function {
    /// Returns the name of the function.
    pub fn name(&self) -> Option<String> {
        self.header()?.identifier()?.name()
    }

    pub fn header(&self) -> Option<FunctionHeader> {
        self.syntax.children().find_map(FunctionHeader::cast)
    }

    /// Returns the body of the function.
    pub fn body(&self) -> Option<Block> {
        self.syntax.children().find_map(Block::cast)
    }
}

impl FunctionHeader {
    /// Returns the identifier of the function.
    pub fn identifier(&self) -> Option<IdentGroup> {
        self.syntax.children().find_map(IdentGroup::cast)
    }

    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().find_map(ParamList::cast)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Root;

    use super::*;

    #[test]
    fn check_ast_node_to_function() {
        const INPUT: &str = r#"
            CREATE OR REPLACE FUNCTION deterministic_function
            RETURN NUMBER DETERMINISTIC
            IS
            BEGIN
                RETURN 1;
            END deterministic_function;
        "#;
        let result = crate::parse_function(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let function = root.unwrap().function();
        assert!(function.is_some());
        assert_eq!(
            function.unwrap().name(),
            Some("deterministic_function".to_string())
        );
    }
}
