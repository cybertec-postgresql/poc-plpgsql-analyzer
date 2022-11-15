// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL

pub mod builtins;
pub mod procedure;

use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, Function, Param, Procedure, Root};
use crate::parser::{parse_any, ParseError};
use crate::syntax::{SqlProcedureLang, SyntaxElement, SyntaxNode, SyntaxToken};
use rowan::{TextRange, TokenAtOffset};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Range;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

lazy_static::lazy_static! {
    static ref ANALYZER_RULES: HashMap<&'static str, Box<dyn RuleDefinition + Send + Sync>> = {
        let mut m = HashMap::new();
        m.insert("CYAR-0001", Box::new(procedure::AddParamlistParenthesis) as Box<dyn RuleDefinition + Send + Sync>);
        m.insert("CYAR-0002", Box::new(procedure::ReplacePrologue) as Box<dyn RuleDefinition + Send + Sync>);
        m.insert("CYAR-0003", Box::new(procedure::ReplaceEpilogue) as Box<dyn RuleDefinition + Send + Sync>);
        m.insert("CYAR-0004", Box::new(builtins::FixTrunc) as Box<dyn RuleDefinition + Send + Sync>);
        m
    };
}

#[derive(Debug, Eq, thiserror::Error, PartialEq, Serialize, TypescriptDefinition)]
#[serde(rename_all = "camelCase")]
pub enum RuleError {
    #[error("Item not found: {0}")]
    NoSuchItem(&'static str),
    #[error("No change")]
    NoChange,
    #[error("Table column definition for '{0}' not found")]
    NoTableInfo(String),
    #[error("Invalid type reference: {0}")]
    InvalidTypeRef(String),
    #[error("Invalid location: {0}")]
    InvalidLocation(RuleLocation),
    #[error("Failed to parse replacement: {0}")]
    ParseError(String),
}

trait RuleDefinition {
    fn short_desc(&self) -> &'static str;
    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError>;
    fn find(&self, node: &SyntaxNode, ctx: &DboAnalyzeContext)
        -> Result<Vec<TextRange>, RuleError>;
    fn apply(
        &self,
        node: &SyntaxNode,
        location: TextRange,
        ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError>;
}

#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
pub struct RuleLocation {
    offset: Range<usize>,
}

impl RuleLocation {
    fn new(offset: Range<usize>) -> Self {
        Self { offset }
    }
}

impl From<TextRange> for RuleLocation {
    fn from(text_range: TextRange) -> Self {
        Self::new(text_range.start().into()..text_range.end().into())
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
pub struct RuleHint {
    name: String,
    locations: Vec<RuleLocation>,
    short_desc: &'static str,
}

impl From<ParseError> for RuleError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error.to_string())
    }
}

pub fn find_applicable_rules(root: &Root, ctx: &DboAnalyzeContext) -> Vec<RuleHint> {
    ANALYZER_RULES
        .iter()
        .filter_map(|(name, rule)| {
            rule.get_node(root)
                .and_then(|node| {
                    rule.find(&node, ctx).map(|ranges| RuleHint {
                        name: (*name).to_owned(),
                        locations: ranges.into_iter().map(Into::into).collect(),
                        short_desc: rule.short_desc(),
                    })
                })
                .ok()
        })
        .collect()
}

/// Finds all child tokens within a syntax tree.
///
/// # Arguments
///
/// `node`: The parent node to find children token(s) in.
///
/// `token_pred`: A closure returning `true` for all tokens to replace.
///
/// `map_location`: Transforms the token location to the same format as
/// `location`.
fn find_token<P, M>(
    node: &dyn AstNode<Language = SqlProcedureLang>,
    token_pred: P,
    map_location: M,
) -> Vec<TextRange>
where
    P: Fn(&SyntaxToken) -> bool,
    M: Fn(SyntaxToken) -> TextRange,
{
    node.syntax()
        .children_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .filter(token_pred)
        .map(map_location)
        .collect()
}

/// Replaces a child token with an updated syntax tree.
///
/// # Arguments
///
/// `node`: The parent node to find and replace some children token in.
///
/// `location`: Specifies the exact token to replace based on it's position.
///
/// `replacement`: A parsable string to replace the token(s) with.
///
/// `replace_offset`: The offset and range from the found token to delete some
/// tokens at. If the range is empty, no tokens are deleted.
fn replace_token(
    node: &SyntaxNode,
    location: TextRange,
    replacement: &str,
    to_delete: Range<usize>,
) -> Result<TextRange, RuleError> {
    let replacement = parse_any(replacement)?.syntax().clone_for_update();

    let start = match node.token_at_offset(location.start()) {
        TokenAtOffset::None => return Err(RuleError::InvalidLocation(location.into())),
        TokenAtOffset::Single(t) => t.index(),
        TokenAtOffset::Between(_, t) => t.index(),
    };
    let end = match node.token_at_offset(location.start()) {
        TokenAtOffset::None => return Err(RuleError::InvalidLocation(location.into())),
        TokenAtOffset::Single(t) => t.index(),
        TokenAtOffset::Between(_, t) => t.index(),
    };

    node.splice_children(
        start + to_delete.start..end + to_delete.end,
        vec![SyntaxElement::Node(replacement.clone())],
    );
    Ok(replacement.text_range())
}

fn check_parameter_types(node: &SyntaxNode, ctx: &DboAnalyzeContext) -> Result<(), RuleError> {
    if let Some(params) = Procedure::cast(node.clone())
        .and_then(|p| p.header())
        .and_then(|p| p.param_list())
    {
        return check_parameter_types_lower(&params.params(), ctx);
    }

    if let Some(params) = Function::cast(node.clone())
        .and_then(|f| f.header())
        .and_then(|f| f.param_list())
    {
        return check_parameter_types_lower(&params.params(), ctx);
    }

    Ok(())
}

fn check_parameter_types_lower(params: &[Param], ctx: &DboAnalyzeContext) -> Result<(), RuleError> {
    for param in params {
        if let Some(ident) = param.type_reference() {
            if let (Some(t), Some(c)) = (ident.qualifier(), ident.name()) {
                if ctx.table_column(&t, &c).is_none() {
                    return Err(RuleError::NoTableInfo(t.to_string()));
                }
            } else {
                return Err(RuleError::InvalidTypeRef(ident.text()));
            }
        }
    }

    Ok(())
}
