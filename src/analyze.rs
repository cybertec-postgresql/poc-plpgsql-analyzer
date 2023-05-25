// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::ast::{AstNode, Root};
use crate::parser::*;
use crate::rules::{find_applicable_rules, RuleHint};
use crate::syntax::SyntaxKind;
use crate::SqlIdent;

/// Different types the analyzer can possibly examine.
///
/// Some types may be only available for specific frontends, e.g.
/// [`Package`][`DboType::Package`] is only available for Oracle databases.
#[derive(Tsify, Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
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

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboFunctionMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboProcedureMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboQueryMetaData {
    // For now, we only report how many OUTER JOINs there are, but not any
    // other info about them yet.
    pub outer_joins: usize,
}

/// The result of parsing and analyzing a piece of SQL code.
#[derive(Tsify, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DboMetaData {
    pub rules: Vec<RuleHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DboFunctionMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedure: Option<DboProcedureMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<DboQueryMetaData>,
}

/// List of possible datatypes for tuple fields.
///
/// Mainly derived from <https://www.postgresql.org/docs/current/datatype.html>,
/// but furter extensible as needed. Keep alphabetically sorted.
#[derive(Tsify, Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
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

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboTableColumn {
    typ: DboColumnType,
}

impl DboTableColumn {
    pub fn new(typ: DboColumnType) -> Self {
        Self { typ }
    }
}

#[derive(Tsify, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DboTable {
    #[tsify(type = "Record<string, DboTableColumn>")]
    columns: HashMap<SqlIdent, DboTableColumn>,
}

impl DboTable {
    pub fn new(columns: HashMap<SqlIdent, DboTableColumn>) -> Self {
        Self { columns }
    }
}

#[derive(Tsify, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DboAnalyzeContext {
    #[tsify(type = "Record<string, DboTable>")]
    tables: HashMap<SqlIdent, DboTable>,
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
#[derive(Debug, Eq, thiserror::Error, PartialEq, Serialize)]
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
    let cast_to_root = |p: Parse| {
        Root::cast(p.syntax())
            .ok_or_else(|| AnalyzeError::ParseError("failed to find root node".to_owned()))
    };

    match typ {
        DboType::Function => analyze_function(sql, cast_to_root(parse_function(sql)?)?, ctx),
        DboType::Procedure => analyze_procedure(sql, cast_to_root(parse_procedure(sql)?)?, ctx),
        DboType::Query => analyze_query(sql, cast_to_root(parse_query(sql)?)?, ctx),
        _ => Err(AnalyzeError::Unsupported(typ)),
    }
}

/// WASM export of [`analyze()`]. Should _never_ be called from other Rust code.
///
/// A second, WASM-specific function is required here, as the only allowed [`Result`] type for
/// returning to JS is [`Result<T, JsValue>`]. We simply call the actual [`analyze()`] function and
/// map the error type.
///
/// The main [`analyze()`] function should not return a [`JsValue`][`wasm_bindgen::JsValue`],
/// since it represents the "normal" entry point into the library (e.g. from other Rust code).
/// Furthermore, [`JsValue`][`wasm_bindgen::JsValue`] does not implement the
/// [`Debug`][`std::fmt::Debug`] trait, making unit tests unnecessarily complex.
#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
#[wasm_bindgen(js_name = "analyze")]
pub fn js_analyze(typ: DboType, sql: &str, ctx: DboAnalyzeContext) -> Result<DboMetaData, JsValue> {
    analyze(typ, sql, &ctx).or_else(|err| Err(serde_wasm_bindgen::to_value(&err)?))
}

fn analyze_function(
    input: &str,
    root: Root,
    ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    let function = root
        .function()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function".to_owned()))?;

    let body = function
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function body".to_owned()))?;

    let name = function.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count() + 1;

    Ok(DboMetaData {
        rules: find_applicable_rules(input, &root, ctx),
        function: Some(DboFunctionMetaData {
            name,
            body,
            lines_of_code,
        }),
        ..Default::default()
    })
}

fn analyze_procedure(
    input: &str,
    root: Root,
    ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    let procedure = root
        .procedure()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure".to_owned()))?;

    let body = procedure
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure body".to_owned()))?;

    let name = procedure.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count() + 1;

    Ok(DboMetaData {
        rules: find_applicable_rules(input, &root, ctx),
        procedure: Some(DboProcedureMetaData {
            name,
            body,
            lines_of_code,
        }),
        ..Default::default()
    })
}

fn analyze_query(
    input: &str,
    root: Root,
    ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    let query = root
        .query()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find query".to_owned()))?;

    let outer_joins = query
        .where_clause()
        .and_then(|wc| wc.expression())
        .map(|expr| {
            expr.filter_tokens(|t| t.kind() == SyntaxKind::Keyword && t.text() == "(+)")
                .count()
        })
        .unwrap_or(0);

    Ok(DboMetaData {
        rules: find_applicable_rules(input, &root, ctx),
        query: Some(DboQueryMetaData { outer_joins }),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_analyze_function() {
        const INPUT: &str =
            include_str!("../tests/function/heading/function_heading_example.ora.sql");

        let result = analyze(DboType::Function, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData {
                function:
                    Some(DboFunctionMetaData {
                        name,
                        lines_of_code,
                        ..
                    }),
                procedure,
                query,
                ..
            } => {
                assert_eq!(procedure, None);
                assert_eq!(query, None);
                assert_eq!(name, "function_heading_example");
                assert_eq!(lines_of_code, 3);
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
            DboMetaData {
                function,
                procedure:
                    Some(DboProcedureMetaData {
                        name,
                        lines_of_code,
                        ..
                    }),
                query,
                ..
            } => {
                assert_eq!(function, None);
                assert_eq!(query, None);
                assert_eq!(name, "add_job_history");
                assert_eq!(lines_of_code, 5);
            }
            _ => unreachable!(),
        }

        const SECURE_DML: &str = include_str!("../tests/fixtures/secure_dml.ora.sql");
        let result = analyze(
            DboType::Procedure,
            SECURE_DML,
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData {
                function,
                procedure:
                    Some(DboProcedureMetaData {
                        name,
                        lines_of_code,
                        ..
                    }),
                query,
                ..
            } => {
                assert_eq!(function, None);
                assert_eq!(query, None);
                assert_eq!(name, "secure_dml");
                assert_eq!(lines_of_code, 7);
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
            DboMetaData {
                function,
                procedure,
                query: Some(DboQueryMetaData { outer_joins, .. }),
                ..
            } => {
                assert_eq!(function, None);
                assert_eq!(procedure, None);
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
