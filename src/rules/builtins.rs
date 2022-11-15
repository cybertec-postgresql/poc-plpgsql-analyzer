// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{check_parameter_types, RuleError};
use crate::analyze::DboAnalyzeContext;
use crate::ast::Root;
use rowan::TextRange;

/// Dummy rule for demonstrating passing in analyzer context and type checking.
pub(super) fn fix_trunc(
    root: &Root,
    _location: Option<TextRange>,
    ctx: &DboAnalyzeContext,
) -> Result<TextRange, RuleError> {
    check_parameter_types(root, ctx)?;

    Err(RuleError::NoChange)
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
        let change = fix_trunc(&root, None, &DboAnalyzeContext::default());
        assert_eq!(change, Err(RuleError::NoTableInfo("persons".to_owned())));

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
        let change = fix_trunc(&root, None, &context);
        assert_eq!(change, Err(RuleError::NoChange));
    }
}
