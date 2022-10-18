// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL.

#![allow(dead_code)]

pub mod procedure;

use crate::ast::AstNode;
use crate::parser::parse_any;
use crate::parser::ParseError;
use crate::syntax::{SqlProcedureLang, SyntaxElement, SyntaxNode, SyntaxToken};
use rowan::TextRange;
use std::ops::Range;

type RuleError = String;

#[derive(Debug, Eq, PartialEq)]
pub struct RuleHint {
    location: TextRange,
    text: String,
}

#[derive(Debug)]
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
