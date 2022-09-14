// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

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

    const SECURE_DML: &str = r#"
        CREATE PROCEDURE secure_dml
        IS
        BEGIN
          IF TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
            RAISE_APPLICATION_ERROR (-20205,
                'You may only make changes during normal office hours');
          END IF;
        END secure_dml;
    "#;

    /// Created code by extracting PL/SQL trigger body into PL/pgSQL function.
    /// Meaning, we are going to end up with neither valid PL/SQL nor PL/pgSQL
    /// code.
    const UPDATE_JOB_HISTORY_TRIGGER_FUNCTION: &str = r#"
        CREATE FUNCTION update_job_history()
          RETURNS TRIGGER
          LANGUAGE PLPGSQL
        AS $$
        BEGIN
          add_job_history(:old.employee_id, :old.hire_date, sysdate,
                          :old.job_id, :old.department_id);
        END;
        $$
    "#;

    const ADD_JOB_HISTORY: &str = r#"
        CREATE OR REPLACE PROCEDURE add_job_history
          (  p_emp_id          job_history.employee_id%type
           , p_start_date      job_history.start_date%type
           , p_end_date        job_history.end_date%type
           , p_job_id          job_history.job_id%type
           , p_department_id   job_history.department_id%type
           )
        IS
        BEGIN
          INSERT INTO job_history (employee_id, start_date, end_date,
                                   job_id, department_id)
            VALUES(p_emp_id, p_start_date, p_end_date, p_job_id, p_department_id);
        END add_job_history;
    "#;

    #[test]
    fn test_lines_of_code() {
        assert_eq!(
            analyze(Type::Procedure, ADD_JOB_HISTORY)
                .unwrap()
                .lines_of_code,
            3,
        );

        assert_eq!(
            analyze(Type::TriggerBody, UPDATE_JOB_HISTORY_TRIGGER_FUNCTION)
                .unwrap()
                .lines_of_code,
            2,
        );

        assert_eq!(
            analyze(Type::Procedure, SECURE_DML).unwrap().lines_of_code,
            5,
        );
    }

    #[test]
    fn test_number_of_statements() {
        assert_eq!(
            analyze(Type::Procedure, ADD_JOB_HISTORY)
                .unwrap()
                .sql_statements
                .len(),
            1,
        );
    }
}
