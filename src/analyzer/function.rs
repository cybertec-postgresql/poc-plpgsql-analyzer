// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::analyzer::{AnalyzeError, DboAnalyzeContext, DboMetaData};
use crate::ast::Root;
use crate::rules::find_applicable_rules;

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboFunctionMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

pub(super) fn analyze_function(
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::analyzer::{analyze, DboType};

    use super::*;

    #[test]
    fn test_analyze_function() {
        const INPUT: &str =
            include_str!("../../tests/function/heading/function_heading_example.ora.sql");

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
}
