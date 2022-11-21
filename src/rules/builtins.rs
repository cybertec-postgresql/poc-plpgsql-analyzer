// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{check_parameter_types, replace_token, RuleDefinition, RuleError};
use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, AstToken, Ident, Root};
use crate::syntax::{SyntaxElement, SyntaxNode};
use rowan::TextRange;

/// Dummy rule for demonstrating passing in analyzer context and type checking.
///
/// TODO: Make generic over procedures and functions
pub(super) struct FixTrunc;

impl RuleDefinition for FixTrunc {
    fn short_desc(&self) -> &'static str {
        "Fix `trunc()` usage based on type"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        root.procedure()
            .map(|p| p.syntax().clone())
            .ok_or(RuleError::NoSuchItem("Procedure"))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        check_parameter_types(node, ctx)?;

        Err(RuleError::NoChange)
    }

    fn apply(
        &self,
        _node: &SyntaxNode,
        _location: TextRange,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        Err(RuleError::NoChange)
    }
}

pub(super) struct ReplaceSysdate;

impl RuleDefinition for ReplaceSysdate {
    fn short_desc(&self) -> &'static str {
        "Replace `SYSDATE` with PostgreSQL's `clock_timestamp()`"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        if let Some(body) = root.procedure().and_then(|p| p.body()) {
            return Ok(body.syntax().clone());
        }

        if let Some(body) = root.function().and_then(|p| p.body()) {
            return Ok(body.syntax().clone());
        }

        if let Some(clause) = root.query().and_then(|p| p.select_clause()) {
            return Ok(clause.syntax().clone());
        }

        Err(RuleError::NoSuchItem(
            "Procedure body, function body or SELECT clause",
        ))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        let locations = node
            .descendants_with_tokens()
            .filter_map(|el| {
                if let SyntaxElement::Token(t) = el {
                    Ident::cast(t)
                } else {
                    None
                }
            })
            .filter(|ident| ident.text().to_lowercase() == "sysdate")
            .map(|ident| ident.syntax().text_range())
            .collect::<Vec<TextRange>>();

        if locations.is_empty() {
            Err(RuleError::NoChange)
        } else {
            Ok(locations)
        }
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: TextRange,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "clock_timestamp()", 0..1)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AstNode;
    use crate::syntax::SyntaxNode;
    use crate::{DboAnalyzeContext, DboColumnType, DboTable, DboTableColumn};
    use expect_test::{expect, Expect};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use super::*;

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
    }

    #[test]
    fn replace_trunc_with_date_trunc() {
        const INPUT: &str = include_str!("../../tests/fixtures/log_last_login_fuzzy.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = FixTrunc;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert_eq!(result, Err(RuleError::NoTableInfo("persons".to_owned())));

        let mut columns = HashMap::new();
        columns.insert("id".into(), DboTableColumn::new(DboColumnType::Integer));
        columns.insert("name".into(), DboTableColumn::new(DboColumnType::Text));
        columns.insert(
            "number_of_logins".into(),
            DboTableColumn::new(DboColumnType::Integer),
        );
        columns.insert(
            "last_login".into(),
            DboTableColumn::new(DboColumnType::Date),
        );

        let mut tables = HashMap::new();
        tables.insert("persons".into(), DboTable::new(columns));
        let ctx = DboAnalyzeContext::new(tables);
        let result = rule.find(&node, &ctx);
        assert_eq!(result, Err(RuleError::NoChange));
    }

    #[test]
    fn test_replace_sysdate() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = ReplaceSysdate;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0], TextRange::new(51.into(), 58.into()));
        assert_eq!(&root.syntax().to_string()[locations[0]], "SYSDATE");
        assert_eq!(locations[1], TextRange::new(123.into(), 130.into()));
        assert_eq!(&root.syntax().to_string()[locations[0]], "SYSDATE");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(&node, locations[0], &DboAnalyzeContext::default());
        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
                CREATE PROCEDURE secure_dml
                IS
                BEGIN
                  IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
                    RAISE_APPLICATION_ERROR (-20205,
                        'You may only make changes during normal office hours');
                  END IF;
                END secure_dml;
            "#]],
        );
        assert_eq!(location, TextRange::new(51.into(), 68.into()));
        assert_eq!(&root.syntax().to_string()[location], "clock_timestamp()");

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0], TextRange::new(133.into(), 140.into()));

        let result = rule.apply(&node, locations[0], &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
                CREATE PROCEDURE secure_dml
                IS
                BEGIN
                  IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                        OR TO_CHAR (clock_timestamp(), 'DY') IN ('SAT', 'SUN') THEN
                    RAISE_APPLICATION_ERROR (-20205,
                        'You may only make changes during normal office hours');
                  END IF;
                END secure_dml;
            "#]],
        );
        assert_eq!(location, TextRange::new(133.into(), 150.into()));
        assert_eq!(&root.syntax().to_string()[location], "clock_timestamp()");
    }
}
