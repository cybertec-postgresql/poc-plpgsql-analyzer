// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL tables.

use crate::ast::{AstNode, AstToken, Datatype, QualifiedIdent};

use super::typed_syntax_node;
use super::Ident;

typed_syntax_node!(Table, ColumnList, Column, DefaultExpression);

impl Table {
    /// Returns the name of the table.
    #[allow(unused)]
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children()
            .find_map(QualifiedIdent::cast)
            .map(|ident| ident.text())
    }

    #[allow(unused)]
    pub fn column_list(&self) -> Option<ColumnList> {
        self.syntax.children().find_map(ColumnList::cast)
    }
}

impl ColumnList {
    #[allow(unused)]
    pub fn columns(&self) -> Vec<Column> {
        self.syntax.children().filter_map(Column::cast).collect()
    }
}

impl Column {
    #[allow(unused)]
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Ident::cast)
            .map(|id| id.text())
    }

    #[allow(unused)]
    pub fn datatype(&self) -> Option<String> {
        self.syntax.children().find_map(Datatype::cast).map(|d| {
            d.syntax
                .children_with_tokens()
                .filter_map(|t| t.into_token())
                .filter(|t| !t.kind().is_trivia())
                .map(|t| t.text().to_string())
                .collect()
        })
    }

    #[allow(unused)]
    pub fn default_expr(&self) -> Option<DefaultExpression> {
        self.syntax.children().find_map(DefaultExpression::cast)
    }
}

// TODO: not exactly happy with this.
//   is there a more elegant way to traverse the AST, or should we introduce more syntax kinds?
impl DefaultExpression {
    #[allow(unused)]
    pub fn on_null(&self) -> bool {
        let mut tokens = self
            .syntax
            .children_with_tokens()
            .filter_map(|t| t.into_token())
            .filter(|t| !t.kind().is_trivia());

        tokens.next();
        if let Some(on_kw) = &tokens.next() {
            if let Some(null_kw) = &tokens.next() {
                return on_kw.text().to_lowercase() == "on"
                    && null_kw.text().to_lowercase() == "null";
            }
        }
        false
    }

    #[allow(unused)]
    pub fn expr(&self) -> Option<String> {
        self.syntax
            .children()
            .filter(|t| !t.kind().is_trivia())
            .last()
            .map(|t| t.text().to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Root;

    use super::*;

    #[test]
    fn check_ast_node_to_table() {
        const INPUT: &str = r#"
            CREATE TABLE "MY_SCHEMA"."EMP" (
                "EMP_ID" NUMBER ( 6 , 0 ) DEFAULT ON NULL MY_SCHEMA.EMP_SEQ.nextval,
                "CREATED_AT" TIMESTAMP WITH TIME ZONE,
                email VARCHAR2(25 BYTE)
            );
        "#;
        let result = crate::parse_table(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let table = root.unwrap().table();
        assert!(table.is_some());
        assert_eq!(
            table.as_ref().unwrap().name(),
            Some(r#" "MY_SCHEMA"."EMP""#.to_string())
        );

        let column_list = table.unwrap().column_list();
        assert!(column_list.is_some());
        assert_eq!(column_list.as_ref().unwrap().columns().len(), 3);

        let columns = column_list.unwrap().columns();
        let column = columns.get(0);
        assert!(column.is_some());
        assert_eq!(column.unwrap().name(), Some("\"EMP_ID\"".to_string()));
        assert_eq!(
            column.unwrap().datatype().unwrap(),
            "NUMBER(6,0)".to_string()
        );
        assert!(column.unwrap().default_expr().unwrap().on_null());
        assert_eq!(
            column.unwrap().default_expr().unwrap().expr(),
            Some("MY_SCHEMA.EMP_SEQ.nextval".to_string()),
        );
        assert!(columns.get(1).is_some());
    }
}
