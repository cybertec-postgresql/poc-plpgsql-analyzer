// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parameter-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{check_parameter_types, RuleChanges, RuleError};
use crate::ast::{AstNode, Root};
use crate::DboAnalyzeContext;

/// Dummy rule for demonstrating passing in analyzer context.
pub fn fix_trunc(root: &Root, ctx: &DboAnalyzeContext) -> Result<RuleChanges, RuleError> {
    if !check_parameter_types(root, ctx) {
        return Err("Parameter type information needed".to_owned());
    }

    let replacement = root.clone_for_update();

    Ok(RuleChanges {
        replacement: replacement.syntax().clone(),
        hints: Vec::new(),
    })
}
