// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use crate::ast::{AstNode, Root};
use crate::parser::*;
use crate::syntax::SyntaxKind;
use crate::util::SqlIdent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

/// Different types the analyzer can possibly examine.
///
/// Some types may be only available for specific frontends, e.g.
/// [`Package`][`Type::Package`] is only available for Oracle databases.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[wasm_bindgen]
pub enum DboType {
    CheckConstraint,
    DefaultExpr,
    Function,
    IndexExpr,
    Package,
    Procedure,
    Query,
    TriggerBody,
    View,
}

/// The result of parsing and analyzing a piece of SQL code.
#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub enum DboMetaData {
    #[serde(rename_all = "camelCase")]
    Function {
        name: String,
        body: String,
        lines_of_code: usize,
    },
    #[serde(rename_all = "camelCase")]
    Procedure {
        name: String,
        body: String,
        lines_of_code: usize,
    },
    #[serde(rename_all = "camelCase")]
    Query {
        // For now, we only report how many OUTER JOINs there are, but not any
        // other info about them yet.
        outer_joins: usize,
    },
}

/// List of possible datatypes for tuple fields.
///
/// Mainly derived from <https://www.postgresql.org/docs/current/datatype.html>,
/// but furter extensible as needed. Keep alphabetically sorted.
#[derive(Debug, Eq, PartialEq, Deserialize, TypescriptDefinition)]
#[wasm_bindgen]
pub enum DboColumnType {
    BigInt,
    Date,
    DoublePrecision,
    Integer,
    Real,
    SmallInt,
    Text,
    Time,
    Timestamp,
    TimestampWithTz,
    TimeWithTz,
}

#[derive(Debug, Eq, PartialEq, Deserialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct DboTableColumn {
    typ: DboColumnType,
}

impl DboTableColumn {
    pub fn new(typ: DboColumnType) -> Self {
        Self { typ }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct DboTable {
    pub(crate) columns: HashMap<SqlIdent, DboTableColumn>,
}

impl DboTable {
    pub fn new(columns: HashMap<SqlIdent, DboTableColumn>) -> Self {
        Self { columns }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct DboAnalyzeContext {
    pub(crate) tables: HashMap<SqlIdent, DboTable>,
}

impl DboAnalyzeContext {
    pub fn new(tables: HashMap<SqlIdent, DboTable>) -> Self {
        Self { tables }
    }

    pub(crate) fn table_column(
        &self,
        table: &SqlIdent,
        column: &SqlIdent,
    ) -> Option<&DboTableColumn> {
        self.tables.get(table).and_then(|t| t.columns.get(column))
    }
}

/// Possible errors that might occur during analyzing.
#[derive(Debug, Eq, thiserror::Error, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub enum AnalyzeError {
    #[error("Language construct unsupported: {0:?}")]
    Unsupported(DboType),
    #[error("Error during parsing: {0}")]
    ParseError(String),
    #[error("Expected {0} node, got {1}")]
    NodeError(String, String),
    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    #[error("Failed to deserialize DBO context: {0}")]
    InvalidContext(String),
}

impl From<ParseError> for AnalyzeError {
    fn from(error: ParseError) -> Self {
        AnalyzeError::ParseError(error.to_string())
    }
}

pub fn analyze(
    typ: DboType,
    sql: &str,
    ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    match typ {
        DboType::Function => analyze_function(parse_function(sql)?, ctx),
        DboType::Procedure => analyze_procedure(parse_procedure(sql)?, ctx),
        DboType::Query => analyze_query(parse_query(sql)?, ctx),
        _ => Err(AnalyzeError::Unsupported(typ)),
    }
}

fn analyze_function(parse: Parse, _ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
    let function = Root::cast(parse.syntax())
        .and_then(|p| p.function())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function".to_owned()))?;

    let body = function
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function body".to_owned()))?;

    let name = function.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count();

    Ok(DboMetaData::Function {
        name,
        body,
        lines_of_code,
    })
}

fn analyze_procedure(parse: Parse, _ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
    let procedure = Root::cast(parse.syntax())
        .and_then(|r| r.procedure())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure".to_owned()))?;

    let body = procedure
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure body".to_owned()))?;

    let name = procedure.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count();

    Ok(DboMetaData::Procedure {
        name,
        body,
        lines_of_code,
    })
}

fn analyze_query(parse: Parse, _ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
    let query = Root::cast(parse.syntax())
        .and_then(|r| r.query())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find query".to_owned()))?;

    let outer_joins = query
        .where_clause()
        .and_then(|wc| wc.expression())
        .map(|expr| {
            expr.filter_tokens(|t| t.kind() == SyntaxKind::Keyword && t.text() == "(+)")
                .count()
        })
        .unwrap_or(0);

    Ok(DboMetaData::Query { outer_joins })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_analyze_function() {
        const INPUT: &str =
            include_str!("../tests/function/heading/function_heading_example.ora.sql");

        let result = analyze(DboType::Function, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData::Function {
                name,
                lines_of_code,
                ..
            } => {
                assert_eq!(name, "function_heading_example");
                assert_eq!(lines_of_code, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_analyze_procedure() {
        const ADD_JOB_HISTORY: &str = include_str!("../tests/fixtures/add_job_history.sql");
        let result = analyze(
            DboType::Procedure,
            ADD_JOB_HISTORY,
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData::Procedure {
                name,
                lines_of_code,
                ..
            } => {
                assert_eq!(name, "add_job_history");
                assert_eq!(lines_of_code, 3);
            }
            _ => unreachable!(),
        }

        const SECURE_DML: &str = include_str!("../tests/fixtures/secure_dml.sql");
        let result = analyze(
            DboType::Procedure,
            SECURE_DML,
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData::Procedure {
                name,
                lines_of_code,
                ..
            } => {
                assert_eq!(name, "secure_dml");
                assert_eq!(lines_of_code, 5);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_analyze_query() {
        const INPUT: &str = include_str!("../tests/dql/select_left_join.ora.sql");
        let result = analyze(DboType::Query, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData::Query { outer_joins, .. } => {
                assert_eq!(outer_joins, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[ignore]
    fn test_triggerbody_lines_of_code() {
        const UPDATE_JOB_HISTORY: &str = include_str!("../tests/fixtures/update_job_history.sql");
        let result = analyze(
            DboType::TriggerBody,
            UPDATE_JOB_HISTORY,
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);
        unreachable!();
    }
}
