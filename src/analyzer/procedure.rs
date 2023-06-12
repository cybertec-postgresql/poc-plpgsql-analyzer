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
pub struct DboProcedureMetaData {
    pub name: String,
    pub body: String,
    pub lines_of_code: usize,
}

pub(super) fn analyze_procedure(
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::analyzer::{analyze, DboType};

    use super::*;

    #[test]
    fn test_analyze_procedure() {
        const ADD_JOB_HISTORY: &str = include_str!("../../tests/fixtures/add_job_history.sql");
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

        const SECURE_DML: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");
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
}
