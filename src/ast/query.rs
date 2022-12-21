// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL procedures.

use crate::ast::AstNode;

use super::typed_syntax_node;
use super::Expression;

typed_syntax_node!(SelectClause, SelectStmt, ColumnExpr, WhereClause);

impl SelectStmt {
    pub fn select_clause(&self) -> Option<SelectClause> {
        self.syntax.children().find_map(SelectClause::cast)
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        self.syntax.children().find_map(WhereClause::cast)
    }
}

impl WhereClause {
    pub fn expression(&self) -> Option<Expression> {
        self.syntax.children().find_map(Expression::cast)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::ast::{ComparisonOpType, Root};
    use crate::syntax::SyntaxKind;

    use super::*;

    #[test]
    fn check_ast_node_to_select_stmt() {
        const INPUT: &str = include_str!("../../tests/dql/select_left_join.ora.sql");
        let result = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let query = root.unwrap().query();

        assert!(query.is_some());
        let clause = query.unwrap().where_clause();

        assert!(clause.is_some());
        let clause = clause.unwrap();

        assert!(clause.expression().is_some());
        let expr = clause.expression().unwrap();

        assert_eq!(
            expr.filter_tokens(|t| t.kind() == SyntaxKind::Ident)
                .next()
                .map(|t| t.text().to_owned()),
            Some("places.person_id".to_owned()),
        );

        assert_eq!(
            expr.filter_tokens(|t| t.kind() == SyntaxKind::ComparisonOp)
                .next()
                .and_then(|t| t.text().parse().ok()),
            Some(ComparisonOpType::Equal),
        );

        assert_eq!(
            expr.filter_tokens(|t| t.kind() == SyntaxKind::Ident)
                .nth(1)
                .map(|t| t.text().to_owned()),
            Some("persons.id".to_owned()),
        );
    }
}
