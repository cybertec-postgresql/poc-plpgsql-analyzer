// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022-2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL function invocations.

use crate::ast::{AstNode, IdentGroup};
use crate::{Argument, ArgumentList};

use super::typed_syntax_node;

typed_syntax_node!(FunctionInvocation);

impl FunctionInvocation {
    /// Returns the name of the function.
    #[allow(unused)]
    pub fn ident(&self) -> Option<IdentGroup> {
        self.syntax.children().find_map(IdentGroup::cast)
    }

    pub fn arguments(&self) -> Option<Vec<Argument>> {
        self.syntax
            .children()
            .find_map(ArgumentList::cast)
            .map(|l| l.arguments())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{ColumnExpr, Root};

    use super::*;

    fn find_function_invocation(root: Option<Root>) -> Option<FunctionInvocation> {
        root.unwrap()
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
            .find_map(FunctionInvocation::cast)
    }

    #[test]
    fn test_unqualified_function() {
        const INPUT: &str = "SELECT NVL(first_name, 'John') FROM DUAL";
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let function_invocation = find_function_invocation(root);

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

        let function_invocation = find_function_invocation(root);

        assert!(function_invocation.is_some());
        assert_eq!(
            function_invocation.unwrap().ident().unwrap().name(),
            Some("JOHN.NVL".to_string())
        );
    }

    #[test]
    fn extract_function_arguments() {
        const INPUT: &str = "SELECT NVL2(col1, col2 + 1, col3) FROM DUAL";
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let function_invocation = find_function_invocation(root);

        assert!(function_invocation.is_some());
        assert_eq!(
            function_invocation
                .unwrap()
                .arguments()
                .unwrap()
                .iter()
                .map(|a| a.text())
                .collect::<Vec<String>>(),
            vec!["col1", "col2 + 1", "col3"]
        );
    }
}
