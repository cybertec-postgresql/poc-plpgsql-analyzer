// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use crate::ast::{AstNode, Root};
use crate::parser::*;
use crate::rules::{find_applicable_rules, RuleHint};
use crate::syntax::SyntaxKind;
use crate::util::SqlIdent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

/// Different types the analyzer can possibly examine.
///
/// Some types may be only available for specific frontends, e.g.
/// [`Package`][`DboType::Package`] is only available for Oracle databases.
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

#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct DboFunctionMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct DboProcedureMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct DboQueryMetaData {
    // For now, we only report how many OUTER JOINs there are, but not any
    // other info about them yet.
    pub outer_joins: usize,
}

/// The result of parsing and analyzing a piece of SQL code.
#[derive(Debug, Default, Eq, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct DboMetaData {
    pub rules: Vec<RuleHint>,
    pub function: Option<DboFunctionMetaData>,
    pub procedure: Option<DboProcedureMetaData>,
    pub query: Option<DboQueryMetaData>,
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

#[derive(Debug, Default, Eq, PartialEq, Deserialize)]
pub struct DboTable {
    columns: HashMap<SqlIdent, DboTableColumn>,
}

impl DboTable {
    pub fn new(columns: HashMap<SqlIdent, DboTableColumn>) -> Self {
        Self { columns }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Deserialize)]
pub struct DboAnalyzeContext {
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
    let cast_to_root = |p: Parse| {
        Root::cast(p.syntax())
            .ok_or_else(|| AnalyzeError::ParseError("failed to find root node".to_owned()))
    };

    match typ {
        DboType::Function => analyze_function(cast_to_root(parse_function(sql)?)?, ctx),
        DboType::Procedure => analyze_procedure(cast_to_root(parse_procedure(sql)?)?, ctx),
        DboType::Query => analyze_query(cast_to_root(parse_query(sql)?)?, ctx),
        _ => Err(AnalyzeError::Unsupported(typ)),
    }
}

/// WASM export of [`analyze()`]. Should _never_ be called from other Rust code.
///
/// A second, WASM-specific function is needed here. Since the only allowed
/// [`Result`] type to return to JS is a [`Result<T, JsValue>`], we just call
/// the actual [`analyze()`] function and map the error type.
///
/// For one, the main [`analyze()`] function shouldn't return a
/// [`JsValue`][`wasm_bindgen::JsValue`], since it should represent the "normal"
/// entry point into the library (e.g. from other Rust code). And secondly,
/// [`JsValue`][`wasm_bindgen::JsValue`] doesn't implement the
/// [`Debug`][`std::fmt::Debug`] trait, which just complicates unit tests.
/// And secondly, we want to transparently parse
/// [`DboAnalyzeContext`][`self::DboAnalyzeContext`] from the raw JS value
/// and pass it on.
#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
#[wasm_bindgen(js_name = "analyze")]
pub fn js_analyze(typ: DboType, sql: &str, ctx: JsValue) -> Result<JsValue, JsValue> {
    let ctx = serde_wasm_bindgen::from_value(ctx)?;

    match analyze(typ, sql, &ctx) {
        Ok(metadata) => Ok(serde_wasm_bindgen::to_value(&metadata)?),
        Err(err) => Err(serde_wasm_bindgen::to_value(&err)?),
    }
}

fn analyze_function(root: Root, ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
    let function = root
        .function()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function".to_owned()))?;

    let body = function
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function body".to_owned()))?;

    let name = function.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count();

    Ok(DboMetaData {
        rules: find_applicable_rules(&root, ctx),
        function: Some(DboFunctionMetaData {
            name,
            body,
            lines_of_code,
        }),
        ..Default::default()
    })
}

fn analyze_procedure(root: Root, ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
    let procedure = root
        .procedure()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure".to_owned()))?;

    let body = procedure
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure body".to_owned()))?;

    let name = procedure.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count();

    Ok(DboMetaData {
        rules: find_applicable_rules(&root, ctx),
        procedure: Some(DboProcedureMetaData {
            name,
            body,
            lines_of_code,
        }),
        ..Default::default()
    })
}

fn analyze_query(root: Root, ctx: &DboAnalyzeContext) -> Result<DboMetaData, AnalyzeError> {
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
        rules: find_applicable_rules(&root, ctx),
        query: Some(DboQueryMetaData { outer_joins }),
        ..Default::default()
    })
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
