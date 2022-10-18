// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use crate::ast::{AstNode, Root};
use crate::parser::*;
use wasm_bindgen::prelude::*;

/// Different types the analyzer can possibly examine.
///
/// Some types may be only available for specific frontends, e.g.
/// [`Package`][`Type::Package`] is only available for Oracle databases.
#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    CheckConstraint,
    DefaultExpr,
    Function,
    IndexExpr,
    Procedure,
    TriggerBody,
    View,
    Package,
}

/// The result of parsing and analyzing a piece of SQL code.
#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq)]
pub struct DboMetaData {
    pub lines_of_code: usize,
    sql_statements: Vec<()>,
}

/// Possible errors that might occur during analyzing.
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum AnalyzeError {
    #[error("Language construct not yet unsupported: {0:?}")]
    Unsupported(Type),
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

/// Main entry point into the analyzer.
pub fn analyze(typ: Type, sql: &str) -> Result<DboMetaData, AnalyzeError> {
    match typ {
        Type::Function => analyze_function(parse_function(sql)?),
        Type::Procedure => analyze_procedure(parse_procedure(sql)?),
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
pub fn analyze_js(typ: Type, sql: &str) -> Result<DboMetaData, JsError> {
    analyze(typ, sql).map_err(Into::into)
}

fn analyze_function(parse: Parse) -> Result<DboMetaData, AnalyzeError> {
    let body = Root::cast(parse.syntax())
        .and_then(|p| p.function())
        .and_then(|p| p.body())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find function body".to_owned()))?;

    Ok(DboMetaData {
        lines_of_code: body.syntax().text().to_string().matches('\n').count(),
        sql_statements: vec![()],
    })
}

fn analyze_procedure(parse: Parse) -> Result<DboMetaData, AnalyzeError> {
    let body = Root::cast(parse.syntax())
        .and_then(|p| p.procedure())
        .and_then(|p| p.body())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find procedure body".to_owned()))?;

    Ok(DboMetaData {
        lines_of_code: body.syntax().text().to_string().matches('\n').count(),
        sql_statements: vec![()],
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
    fn test_procedure_lines_of_code() {
        let result = analyze(Type::Procedure, ADD_JOB_HISTORY);
        assert!(result.is_ok(), "{:#?}", result);
        assert_eq!(result.unwrap().lines_of_code, 3);

        let result = analyze(Type::Procedure, SECURE_DML);
        assert!(result.is_ok(), "{:#?}", result);
        assert_eq!(result.unwrap().lines_of_code, 5);
    }

    #[test]
    #[ignore]
    fn test_triggerbody_lines_of_code() {
        let result = analyze(Type::TriggerBody, UPDATE_JOB_HISTORY);
        assert!(result.is_ok(), "{:#?}", result);
        assert_eq!(result.unwrap().lines_of_code, 2);
    }

    #[test]
    fn test_number_of_statements() {
        let result = analyze(Type::Procedure, ADD_JOB_HISTORY);
        assert!(result.is_ok(), "{:#?}", result);
        assert_eq!(result.unwrap().sql_statements.len(), 1);
    }
}
