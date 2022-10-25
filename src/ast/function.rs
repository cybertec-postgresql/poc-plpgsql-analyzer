// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL functions.

use super::typed_syntax_node;
use super::Ident;
use crate::ast::{AstNode, AstToken};

typed_syntax_node!(Function, FunctionHeader, FunctionBody);

impl Function {
    /// Returns the name of the function.
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children()
            .find_map(FunctionHeader::cast)?
            .name()
    }

    /// Returns the body of the function.
    pub fn body(&self) -> Option<FunctionBody> {
        self.syntax.children().find_map(FunctionBody::cast)
    }
}

impl FunctionHeader {
    /// Returns the name of the function.
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Ident::cast)
            .map(|ident| ident.name())
    }
}

impl FunctionBody {
    pub fn text(&self) -> String {
        self.syntax.text().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Root;

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
