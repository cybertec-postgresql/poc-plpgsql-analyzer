// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::analyzer::{AnalyzeError, DboMetaData};
use crate::ast::Root;
use crate::syntax::SyntaxKind;

#[derive(Tsify, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DboQueryMetaData {
    // For now, we only report how many OUTER JOINs there are, but not any
    // other info about them yet.
    pub outer_joins: usize,
}

pub(super) fn analyze_query(root: Root) -> Result<DboMetaData, AnalyzeError> {
    let query = root
        .query()
        .ok_or_else(|| AnalyzeError::ParseError("failed to find query".to_owned()))?;

    let outer_joins = query
        .where_clause()
        .and_then(|wc| wc.expression())
        .map(|expr| {
            expr.filter_tokens(|t| t.kind() == SyntaxKind::Keyword && t.text() == "(+)")
                .count()
        })
        .unwrap_or(0);

    Ok(DboMetaData {
        query: Some(DboQueryMetaData { outer_joins }),
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
    fn test_analyze_query() {
        const INPUT: &str = include_str!("../../tests/dql/select_left_join.ora.sql");
        let result = analyze(DboType::Query, INPUT, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{result:#?}");
        let result = result.unwrap();

        match result {
            DboMetaData {
                function,
                procedure,
                query: Some(DboQueryMetaData { outer_joins, .. }),
                ..
            } => {
                assert_eq!(function, None);
                assert_eq!(procedure, None);
                assert_eq!(outer_joins, 1);
            }
            _ => unreachable!(),
        }
    }
}
