// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Additional functions and types needed for a clean Rust <-> JS interface.

#![cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]

use crate::{analyze, DboAnalyzeContext, DboTable, DboTableColumn, DboType};
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

#[derive(Debug, Default, Eq, PartialEq, Deserialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct JsDboTable {
    columns: HashMap<String, DboTableColumn>,
}

impl From<JsDboTable> for DboTable {
    fn from(from: JsDboTable) -> Self {
        let mut columns = HashMap::new();

        for (k, v) in from.columns {
            columns.insert(k.into(), v);
        }

        Self { columns }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Deserialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub struct JsDboAnalyzeContext {
    tables: HashMap<String, JsDboTable>,
}

impl From<JsDboAnalyzeContext> for DboAnalyzeContext {
    fn from(from: JsDboAnalyzeContext) -> Self {
        let mut tables = HashMap::new();

        for (k, v) in from.tables {
            tables.insert(k.into(), v.into());
        }

        Self { tables }
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
/// [`JsDboAnalyzeContext`][`self::JsDboAnalyzeContext`] from the raw JS value
/// and pass it on.
#[wasm_bindgen(js_name = "analyze")]
pub fn js_analyze(typ: DboType, sql: &str, ctx: JsValue) -> Result<JsValue, JsValue> {
    let ctx = serde_wasm_bindgen::from_value::<JsDboAnalyzeContext>(ctx)?;

    match analyze(typ, sql, &ctx.into()) {
        Ok(metadata) => Ok(serde_wasm_bindgen::to_value(&metadata)?),
        Err(err) => Err(serde_wasm_bindgen::to_value(&err)?),
    }
}
