// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL.

#![allow(dead_code)]

pub mod builtins;
pub mod procedure;

use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, Param, Root};
use crate::parser::{parse_any, ParseError};
use crate::syntax::{SqlProcedureLang, SyntaxElement, SyntaxToken};
use rowan::TextRange;
use serde::Serialize;
use std::ops::Range;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

static ANALYZER_RULES: phf::OrderedMap<&'static str, RuleDefinition> = phf::phf_ordered_map! {
    "CYAR-0001" => RuleDefinition {
        short_desc: "Add parameter list parentheses",
        apply: procedure::add_paramlist_parens,
    },
    "CYAR-0002" => RuleDefinition {
        short_desc: "Replace procedure prologue",
        apply: procedure::replace_procedure_prologue,
    },
    "CYAR-0003" => RuleDefinition {
        short_desc: "Replace procedure epilogue",
        apply: procedure::replace_procedure_epilogue,
    },
    "CYAR-0004" => RuleDefinition {
        short_desc: "Fix `trunc()` usage based on type",
        apply: builtins::fix_trunc,
    },
};

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
    #[error("Failed to parse replacement: {0}")]
    ParseError(String),
}

struct RuleDefinition {
    short_desc: &'static str,
    apply: fn(&Root, Option<TextRange>, &DboAnalyzeContext) -> Result<TextRange, RuleError>,
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
    location: RuleLocation,
    short_desc: &'static str,
}

impl From<ParseError> for RuleError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error.to_string())
    }
}

impl RuleHint {
    fn new(loc: Range<usize>, short_desc: &'static str) -> Self {
        Self {
            name: String::new(),
            short_desc,
            location: RuleLocation::new(loc),
        }
    }
}

pub fn find_applicable_rules(root: &Root, ctx: &DboAnalyzeContext) -> Vec<RuleHint> {
    ANALYZER_RULES
        .into_iter()
        .filter_map(|(name, rule)| {
            (rule.apply)(root, None, ctx).ok().map(|range| RuleHint {
                name: (*name).to_owned(),
                location: range.into(),
                short_desc: rule.short_desc,
            })
        })
        .collect()
}

/// Replaces a child token with an updated syntax tree.
///
/// If no `location` is specified, it returns all locations (appropriatly
/// transformed using `map_location`) suitable tokens to replace were found.
///
/// # Arguments
///
/// `node`: The parent node to replace some children token in.
///
/// `token_pred`: A closure returning `true` for all tokens to replace.
///
/// `location`: If not `None`, specifies the exact token to replace based on
/// it's position.
///
/// `map_location`: Transforms the token location to the same format as
/// `location`.
///
/// `item_name`: The name of the item to search for. Used for errors if the
/// searched-for token could not be found.
///
/// `replacement`: A parsable string to replace the token(s) with.
///
/// `replace_offset`: The offset and range to delete tokens at.
fn replace_child<P, M>(
    node: &dyn AstNode<Language = SqlProcedureLang>,
    token_pred: P,
    location: Option<TextRange>,
    map_location: M,
    item_name: &'static str,
    replacement: &str,
    replace_offset: Range<usize>,
) -> Result<TextRange, RuleError>
where
    P: Fn(&SyntaxToken) -> bool,
    M: FnOnce(&SyntaxToken) -> TextRange,
{
    let replacement = parse_any(replacement)?.syntax().clone_for_update();

    let token = node
        .syntax()
        .children_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .find(token_pred)
        .ok_or(RuleError::NoSuchItem(item_name))?;

    let range = map_location(&token);

    match location {
        // Only replace the node if a) the user requested replacement by specifying
        // an location and b) if that location actually matches where we found our
        // node to replace.
        Some(loc) if loc == range => {
            let index = token.index();
            node.syntax().splice_children(
                index + replace_offset.start..index + replace_offset.end,
                vec![SyntaxElement::Node(replacement.clone())],
            );
            Ok(replacement.text_range())
        }
        Some(_) => Err(RuleError::NoSuchItem(item_name)),
        None => Ok(range),
    }
}

fn check_parameter_types(root: &Root, ctx: &DboAnalyzeContext) -> Result<(), RuleError> {
    if let Some(params) = root
        .procedure()
        .and_then(|p| p.header())
        .and_then(|p| p.param_list())
    {
        return check_parameter_types_lower(&params.params(), ctx);
    }

    if let Some(params) = root
        .function()
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
