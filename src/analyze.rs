// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use crate::ast::{AstNode, Root};
use crate::parser::*;
use serde::Serialize;
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
pub enum DboMetaData {
    Function {
        name: String,
        body: String,
        lines_of_code: usize,
    },
    Procedure {
        name: String,
        body: String,
        lines_of_code: usize,
    },
    Query {
        outer_joins: usize, // For now, we only report how many OUTER JOINs there are, but not any other info about them yet.
    },
}

/// Possible errors that might occur during analyzing.
#[derive(Debug, Eq, thiserror::Error, PartialEq, Serialize, TypescriptDefinition)]
pub enum AnalyzeError {
    #[error("Language construct unsupported: {0:?}")]
    Unsupported(DboType),
    #[error("Error during parsing: {0}")]
    ParseError(String),
    #[error("Expected {0} node, got {1}")]
    NodeError(String, String),
}

impl From<ParseError> for AnalyzeError {
    fn from(error: ParseError) -> Self {
        AnalyzeError::ParseError(error.to_string())
    }
}

pub fn analyze(typ: DboType, sql: &str) -> Result<DboMetaData, AnalyzeError> {
    match typ {
        DboType::Function => analyze_function(parse_function(sql)?),
        DboType::Procedure => analyze_procedure(parse_procedure(sql)?),
        DboType::Query => analyze_query(parse_query(sql)?),
        _ => Err(AnalyzeError::Unsupported(typ)),
    }
}

/// WASM export of [`analyze()`]. Should _never_ be called from other Rust code.
///
/// A second, WASM-specific function is needed here. Since the only allowed
/// [`Result`] type to return to JS is a [`Result<T, JsError>`], we just call
/// the actual [`analyze()`] function and map the error type.
///
/// For one, the main [`analyze()`] function shouldn't return a
/// [`JsError`][`wasm_bindgen::JsError`], since it should represent the "normal"
/// entry point into the library (e.g. from other Rust code). And secondly,
/// [`JsError`][`wasm_bindgen::JsError`] doesn't implement the
/// [`Debug`][`std::fmt::Debug`] trait, which just complicates unit tests.
#[wasm_bindgen(js_name = "analyze")]
pub fn analyze_js(typ: DboType, sql: &str) -> Result<JsValue, JsValue> {
    match analyze(typ, sql) {
        Ok(metadata) => Ok(serde_wasm_bindgen::to_value(&metadata)?),
        Err(err) => Err(serde_wasm_bindgen::to_value(&err)?),
    }
}

fn analyze_function(parse: Parse) -> Result<DboMetaData, AnalyzeError> {
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

fn analyze_procedure(parse: Parse) -> Result<DboMetaData, AnalyzeError> {
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

fn analyze_query(parse: Parse) -> Result<DboMetaData, AnalyzeError> {
    let query = Root::cast(parse.syntax())
        .and_then(|r| r.query())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find query".to_owned()))?;

    Ok(DboMetaData::Query {
        outer_joins: query
            .where_clauses()
            .into_iter()
            .filter(|wc| wc.is_outer_join())
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const SECURE_DML: &str = include_str!("../tests/fixtures/secure_dml.sql");
    /// Automatically created code by extracting PL/SQL trigger body into a
    /// PL/pgSQL function. Meaning, this is neither valid PL/SQL nor PL/pgSQL
    /// code.
    const UPDATE_JOB_HISTORY: &str = include_str!("../tests/fixtures/update_job_history.sql");
    const ADD_JOB_HISTORY: &str = include_str!("../tests/fixtures/add_job_history.sql");

    #[test]
    fn test_analyze_procedure() {
        let result = analyze(DboType::Procedure, ADD_JOB_HISTORY);
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

        let result = analyze(DboType::Procedure, SECURE_DML);
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
    fn test_analyze_function() {
        const INPUT: &str =
            include_str!("../tests/function/heading/function_heading_example.ora.sql");

        let result = analyze(DboType::Function, INPUT);
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData::Function {
                name,
                body,
                lines_of_code,
                ..
            } => {
                println!("{:?}", body);
                assert_eq!(name, "function_heading_example");
                assert_eq!(lines_of_code, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[ignore]
    fn test_triggerbody_lines_of_code() {
        let result = analyze(DboType::TriggerBody, UPDATE_JOB_HISTORY);
        assert!(result.is_ok(), "{:#?}", result);
        unreachable!();
    }
}
