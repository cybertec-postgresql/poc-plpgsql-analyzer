// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::analyzer::{AnalyzeError, DboMetaData};
use crate::ast::Root;

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboViewMetaData {
    pub name: String,
}

pub(super) fn analyze_view(root: Root) -> Result<DboMetaData, AnalyzeError> {
    let view = root
        .view()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find view".to_owned()))?;

    let name = view.name().unwrap_or_else(|| "<unknown>".to_string());

    Ok(DboMetaData {
        view: Some(DboViewMetaData { name }),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::analyzer::{analyze, DboType};
    use crate::DboAnalyzeContext;

    use super::*;

    #[test]
    fn test_analyze_view() {
        const INPUT: &str = "CREATE VIEW store_view AS SELECT name FROM stores";
        let result = analyze(DboType::View, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{result:#?}");
        let result = result.unwrap();

        match result {
            DboMetaData {
                view: Some(view), ..
            } => {
                assert_eq!(view.name, "store_view");
            }
            _ => unreachable!(),
        }
    }
}
