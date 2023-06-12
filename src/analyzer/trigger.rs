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
pub struct DboTriggerMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

pub(super) fn analyze_trigger(
    input: &str,
    root: Root,
    ctx: &DboAnalyzeContext,
) -> Result<DboMetaData, AnalyzeError> {
    let trigger = root
        .trigger()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find trigger".to_owned()))?;

    let body = trigger
        .body()
        .map(|b| b.text())
        .ok_or_else(|| AnalyzeError::ParseError("failed to find trigger body".to_owned()))?;

    let name = trigger.name().unwrap_or_else(|| "<unknown>".to_string());
    let lines_of_code = body.matches('\n').count() + 1;

    Ok(DboMetaData {
        rules: find_applicable_rules(input, &root, ctx),
        trigger: Some(DboTriggerMetaData {
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
    fn test_analyze_trigger() {
        const INPUT: &str = include_str!("../../tests/trigger/after_trigger.ora.sql");

        let result = analyze(DboType::Trigger, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);
        let result = result.unwrap();

        match result {
            DboMetaData {
                trigger:
                    Some(DboTriggerMetaData {
                        name,
                        lines_of_code,
                        ..
                    }),
                ..
            } => {
                assert_eq!(name, "store.after_trigger");
                assert_eq!(lines_of_code, 4);
            }
            _ => unreachable!(),
        }
    }
}
