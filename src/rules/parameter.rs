// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{check_parameter_types, RuleChanges, RuleError};
use crate::ast::{AstNode, Root};
use crate::DboAnalyzeContext;

/// Dummy rule for demonstrating passing in analyzer context.
pub fn fix_trunc(root: &Root, ctx: &DboAnalyzeContext) -> Result<RuleChanges, RuleError> {
    if !check_parameter_types(root, ctx) {
        return Err("Parameter type information needed".to_owned());
    }

    let replacement = root.clone_for_update();

    Ok(RuleChanges {
        replacement: replacement.syntax().clone(),
        hints: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DboAnalyzeContext, DboColumnType, DboTable, DboTableColumn};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn replace_trunc_with_date_trunc() {
        const INPUT: &str = include_str!("../../tests/fixtures/log_last_login_fuzzy.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let change = fix_trunc(&root, &DboAnalyzeContext::default());
        assert_eq!(change, Err("Parameter type information needed".to_owned()));

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
        let context = DboAnalyzeContext::new(tables);
        let change = fix_trunc(&root, &context);
        assert!(change.is_ok(), "{:#?}", change);
    }
}
