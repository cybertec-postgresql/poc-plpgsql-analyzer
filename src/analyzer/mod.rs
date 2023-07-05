// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements the main analyzer functionality.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::analyzer::function::{analyze_function, DboFunctionMetaData};
use crate::analyzer::procedure::{analyze_procedure, DboProcedureMetaData};
use crate::analyzer::query::{analyze_query, DboQueryMetaData};
use crate::analyzer::trigger::{analyze_trigger, DboTriggerMetaData};
use crate::ast::{AstNode, Root};
use crate::parser::*;
use crate::SqlIdent;

mod function;
mod procedure;
mod query;
mod trigger;

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
    Trigger,
    View,
}

/// The result of parsing and analyzing a piece of SQL code.
#[derive(Tsify, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DboMetaData {
    pub function: Option<DboFunctionMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedure: Option<DboProcedureMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<DboQueryMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<DboTriggerMetaData>,
}

/// List of possible datatypes for tuple fields.
///
/// Mainly derived from <https://www.postgresql.org/docs/current/datatype.html>,
/// but further extensible as needed. Keep alphabetically sorted.
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
    TimeWithTz,
    Timestamp,
    TimestampWithTz,
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

    pub fn table_column(&self, table: &SqlIdent, column: &SqlIdent) -> Option<&DboTableColumn> {
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
    _ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    let cast_to_root = |p: Parse| {
        Root::cast(p.syntax())
            .ok_or_else(|| AnalyzeError::ParseError("failed to find root node".to_owned()))
    };

    match typ {
        DboType::Function => analyze_function(cast_to_root(parse_function(sql)?)?),
        DboType::Procedure => analyze_procedure(cast_to_root(parse_procedure(sql)?)?),
        DboType::Query => analyze_query(cast_to_root(parse_query(sql)?)?),
        DboType::Trigger => analyze_trigger(cast_to_root(parse_trigger(sql)?)?),
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
