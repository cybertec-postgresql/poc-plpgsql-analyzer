// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL function invocations.

use crate::ast::{AstNode, IdentGroup};

use super::typed_syntax_node;

typed_syntax_node!(FunctionInvocation);

impl FunctionInvocation {
    /// Returns the name of the function.
    #[allow(unused)]
    pub fn ident(&self) -> Option<IdentGroup> {
        self.syntax.children().find_map(IdentGroup::cast)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{ColumnExpr, Root};

    use super::*;

    #[test]
    fn test_unqualified_function() {
        const INPUT: &str = "SELECT NVL(first_name, 'John') FROM DUAL";
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let function_invocation = root
            .unwrap()
            .query()
            .unwrap()
            .select_clause()
            .unwrap()
            .syntax()
            .children()
            .find_map(ColumnExpr::cast)
            .unwrap()
            .syntax()
            .children()
            .find_map(FunctionInvocation::cast);

        assert!(function_invocation.is_some());
        assert_eq!(
            function_invocation.unwrap().ident().unwrap().name(),
            Some("NVL".to_string())
        );
    }

    #[test]
    fn test_qualified_function() {
        const INPUT: &str = "SELECT JOHN.NVL(first_name, 'John') FROM DUAL";
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let function_invocation = root
            .unwrap()
            .query()
            .unwrap()
            .select_clause()
            .unwrap()
            .syntax()
            .children()
            .find_map(ColumnExpr::cast)
            .unwrap()
            .syntax()
            .children()
            .find_map(FunctionInvocation::cast);

        assert!(function_invocation.is_some());
        assert_eq!(
            function_invocation.unwrap().ident().unwrap().name(),
            Some("JOHN.NVL".to_string())
        );
    }
}