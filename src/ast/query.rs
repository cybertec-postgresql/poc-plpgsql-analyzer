// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL procedures.

use super::typed_syntax_node;
use super::{Ident, Operator};
use crate::ast::{AstNode, AstToken};
use crate::syntax::SyntaxKind;

typed_syntax_node!(
    SelectStmt,
    ColumnExprList,
    ColumnExpr,
    WhereClauseList,
    WhereClause
);

impl SelectStmt {
    pub fn where_clauses(&self) -> Vec<WhereClause> {
        self.syntax
            .children()
            .find_map(WhereClauseList::cast)
            .map(|wcl| {
                wcl.syntax()
                    .children()
                    .filter_map(WhereClause::cast)
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }
}

impl WhereClause {
    #[allow(unused)]
    pub fn left_side(&self) -> Option<Ident> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Ident::cast)
    }

    #[allow(unused)]
    pub fn right_side(&self) -> Option<Ident> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .filter_map(Ident::cast)
            .nth(1)
    }

    #[allow(unused)]
    pub fn operator(&self) -> Option<Operator> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Operator::cast)
    }

    pub fn is_outer_join(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .any(|t| t.kind() == SyntaxKind::Keyword && t.text() == "(+)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Root;
    use pretty_assertions::assert_eq;

    #[test]
    fn check_ast_node_to_select_stmt() {
        const INPUT: &str = include_str!("../../tests/dql/select_left_join.ora.sql");
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let query = root.unwrap().query();
        assert!(query.is_some());

        let clauses = query.unwrap().where_clauses();
        assert!(!clauses.is_empty() && clauses.first().is_some());

        let clause = clauses.first().unwrap();
        assert_eq!(
            clause.left_side().unwrap().text(),
            "places.person_id".to_owned(),
        );
        assert_eq!(
            clause.operator().unwrap().syntax().kind(),
            SyntaxKind::Equals,
        );
        assert_eq!(clause.right_side().unwrap().text(), "persons.id".to_owned(),);
    }
}
