// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use rowan::TextRange;

use crate::analyzer::DboAnalyzeContext;
use crate::ast::{AstNode, FunctionInvocation, IdentGroup, Root};
use crate::rules::{filter_map_descendant_nodes, RuleMatch};
use crate::syntax::SyntaxNode;

use super::{check_parameter_types, replace_token, RuleDefinition, RuleError, RuleLocation};

/// Dummy rule for demonstrating passing in analyzer context and type checking.
///
/// TODO: Make generic over procedures and functions
pub(super) struct FixTrunc;

impl RuleDefinition for FixTrunc {
    fn short_desc(&self) -> String {
        "Fix `trunc()` usage based on type".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let node = &root
            .procedure()
            .map(|p| p.syntax().clone())
            .ok_or(RuleError::NoSuchItem("Procedure"))?;
        check_parameter_types(node, ctx)?;

        Err(RuleError::Unsupported("CYAR-0004: FixTrunc".to_string()))
    }

    fn apply(
        &self,
        _node: &SyntaxNode,
        _location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        Err(RuleError::Unsupported("CYAR-0004: FixTrunc".to_string()))
    }
}

pub(super) struct ReplaceSysdate;

impl RuleDefinition for ReplaceSysdate {
    fn short_desc(&self) -> String {
        "Replace `SYSDATE` with PostgreSQL's `clock_timestamp()`".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, IdentGroup::cast)
            .filter(|i| i.name().unwrap().to_lowercase() == "sysdate")
            .map(|i| RuleMatch::from_node(i.syntax()))
            .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "clock_timestamp()", None, 0..1)
    }
}

pub(super) struct ReplaceNvl;

impl RuleDefinition for ReplaceNvl {
    fn short_desc(&self) -> String {
        "Replace `NVL` with PostgreSQL's `coalesce`".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, FunctionInvocation::cast)
            .filter_map(|f| f.ident())
            .filter(|i| i.name().unwrap().to_lowercase() == "nvl")
            .map(|i| RuleMatch::from_node(i.syntax()))
            .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "coalesce", None, 0..1)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use expect_test::{expect, Expect};
    use pretty_assertions::assert_eq;

    use crate::ast::AstNode;
    use crate::syntax::SyntaxNode;
    use crate::{DboAnalyzeContext, DboColumnType, DboTable, DboTableColumn};

    use super::*;

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
    }

    #[test]
    fn replace_trunc_with_date_trunc() {
        const INPUT: &str = include_str!("../../tests/fixtures/log_last_login_fuzzy.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap().clone_for_update();
        let rule = FixTrunc;

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
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
        let result = rule.find_rules(&root, &ctx);
        assert_eq!(
            result,
            Err(RuleError::Unsupported("CYAR-0004: FixTrunc".to_string()))
        );
    }

    #[test]
    fn test_replace_sysdate() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap().clone_for_update();
        let rule = ReplaceSysdate;

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].range, TextRange::new(51.into(), 58.into()));
        assert_eq!(&root.syntax().to_string()[locations[0].range], "SYSDATE");
        assert_eq!(locations[1].range, TextRange::new(123.into(), 130.into()));
        assert_eq!(&root.syntax().to_string()[locations[0].range], "SYSDATE");

        let result = rule.apply(
            &locations[0].node,
            &RuleLocation::from(INPUT, locations[0].range),
            &DboAnalyzeContext::default(),
        );
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

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range, TextRange::new(133.into(), 140.into()));

        let result = rule.apply(
            &locations[0].node,
            &RuleLocation::from(&root.syntax().to_string(), locations[0].range),
            &DboAnalyzeContext::default(),
        );
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

    #[test]
    fn test_replace_nvl() {
        const INPUT: &str = include_str!("../../tests/dql/nvl-coalesce.ora.sql");

        let parse = crate::parse_query(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap().clone_for_update();
        let rule = ReplaceNvl;

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].range, TextRange::new(7.into(), 10.into()));
        assert_eq!(locations[1].range, TextRange::new(11.into(), 14.into()));
        assert_eq!(&root.syntax().to_string()[locations[0].range], "NVL");

        let result = rule.apply(
            &locations[0].node,
            &RuleLocation::from(INPUT, locations[0].range),
            &DboAnalyzeContext::default(),
        );
        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
                SELECT coalesce(NVL(dummy, dummy), 'John'), JOHN.NVL() from dual;
            "#]],
        );

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range, TextRange::new(16.into(), 19.into()));
        assert_eq!(&root.syntax().to_string()[locations[0].range], "NVL");

        assert_eq!(location, TextRange::new(7.into(), 15.into()));
        assert_eq!(&root.syntax().to_string()[location], "coalesce");

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].range, TextRange::new(16.into(), 19.into()));

        let result = rule.apply(
            &locations[0].node,
            &RuleLocation::from(&root.syntax().to_string(), locations[0].range),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
                SELECT coalesce(coalesce(dummy, dummy), 'John'), JOHN.NVL() from dual;
            "#]],
        );
        assert_eq!(location, TextRange::new(16.into(), 24.into()));
        assert_eq!(&root.syntax().to_string()[location], "coalesce");
    }
}
