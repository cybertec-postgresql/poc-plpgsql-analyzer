// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{check_parameter_types, RuleDefinition, RuleError};
use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, Root};
use crate::syntax::SyntaxNode;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;
    use crate::{DboAnalyzeContext, DboColumnType, DboTable, DboTableColumn};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

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
}
