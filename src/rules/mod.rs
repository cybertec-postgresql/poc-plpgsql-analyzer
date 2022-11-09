// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL.

#![allow(dead_code)]

pub mod parameter;
pub mod procedure;

use crate::ast::{AstNode, ParamList, Root};
use crate::parser::parse_any;
use crate::parser::ParseError;
use crate::syntax::{SqlProcedureLang, SyntaxElement, SyntaxNode, SyntaxToken};
use crate::DboAnalyzeContext;
use rowan::TextRange;
use std::ops::Range;

type RuleError = String;

#[derive(Debug, Eq, PartialEq)]
pub struct RuleHint {
    location: TextRange,
    text: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RuleChanges {
    replacement: SyntaxNode,
    hints: Vec<RuleHint>,
}

impl From<ParseError> for RuleError {
    fn from(error: ParseError) -> Self {
        error.to_string()
    }
}

impl RuleHint {
    fn new<S: Into<String>>(loc: Range<usize>, text: S) -> Self {
        Self {
            location: TextRange::new((loc.start as u32).into(), (loc.end as u32).into()),
            text: text.into(),
        }
    }
}

fn define_rule<P>(
    node: &dyn AstNode<Language = SqlProcedureLang>,
    token_pred: P,
    token_err: &str,
    replacement: &str,
    offset: Range<usize>,
    hint: &str,
) -> Result<RuleHint, RuleError>
where
    P: FnMut(&SyntaxToken) -> bool,
{
    let replacement = parse_any(replacement)?.syntax();

    let index = node
        .syntax()
        .children_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .find(token_pred)
        .map(|t| t.index())
        .ok_or_else(|| token_err.to_owned())?;

    let location = index + offset.start..index + offset.end;
    node.syntax().splice_children(
        location.clone(),
        vec![SyntaxElement::Node(replacement.clone_for_update())],
    );

    Ok(RuleHint::new(location, hint))
}

fn check_parameter_types(root: &Root, ctx: &DboAnalyzeContext) -> bool {
    if let Some(params) = root
        .procedure()
        .and_then(|p| p.header())
        .and_then(|p| p.param_list())
    {
        return check_parameter_types_lower(&params, ctx);
    }

    if let Some(params) = root
        .function()
        .and_then(|f| f.header())
        .and_then(|f| f.param_list())
    {
        return check_parameter_types_lower(&params, ctx);
    }

    true
}

fn check_parameter_types_lower(params: &ParamList, ctx: &DboAnalyzeContext) -> bool {
    for param in params.params() {
        if let Some(ident) = param.type_reference() {
            if let (Some(t), Some(c)) = (ident.qualifier(), ident.name()) {
                if ctx.table_column(&t, &c).is_none() {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    true
}
