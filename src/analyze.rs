// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use std::fmt;
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
    lines_of_code: usize,
    sql_statements: Vec<()>,
}

/// Possible errors that might occur during analyzing.
#[derive(Debug, Eq, PartialEq)]
pub enum AnalyzeError {
    Unknown,
}

// Needed for conversion into a [`JsError`][`wasm_bindgen::JsError`].
impl std::error::Error for AnalyzeError {}

// impl [`std::error::Error`] for `AnalyzeError` requires
// the [`std::fmt::Display`] trait to be implemented.
impl fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Main entry point into the analyzer.
pub fn analyze(typ: Type, sql: &str) -> Result<DboMetaData, AnalyzeError> {
    Err(AnalyzeError::Unknown)
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
    analyze(typ, sql).map_err(|err| err.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECURE_DML: &str = include_str!("../tests/fixtures/secure_dml.sql");
    /// Automatically created code by extracting PL/SQL trigger body into a
    /// PL/pgSQL function. Meaning, this is neither valid PL/SQL nor PL/pgSQL
    /// code.
    const UPDATE_JOB_HISTORY: &str = include_str!("../tests/fixtures/update_job_history.sql");
    const ADD_JOB_HISTORY: &str = include_str!("../tests/fixtures/add_job_history.sql");

    #[test]
    fn test_lines_of_code() {
        let result = analyze(Type::Procedure, ADD_JOB_HISTORY);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines_of_code, 3);

        let result = analyze(Type::TriggerBody, UPDATE_JOB_HISTORY);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines_of_code, 2);

        let result = analyze(Type::Procedure, SECURE_DML);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines_of_code, 5);
    }

    #[test]
    fn test_number_of_statements() {
        let result = analyze(Type::Procedure, ADD_JOB_HISTORY);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sql_statements.len(), 1);
    }
}
