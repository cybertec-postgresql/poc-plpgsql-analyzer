// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL

pub mod builtins;
pub mod procedure;

use crate::analyze::{DboAnalyzeContext, DboType};
use crate::ast::{AstNode, Function, Param, Procedure, Root};
use crate::parser::*;
use crate::syntax::{SqlProcedureLang, SyntaxElement, SyntaxNode, SyntaxToken};
use indexmap::IndexMap;
use rowan::{TextRange, TokenAtOffset};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

macro_rules! rule_list {
    ( $( $name:literal => $ty:path ),+ $(,)? ) => {
        indexmap::indexmap! {
            $( $name => Box::new($ty) as Box<dyn RuleDefinition + Send + Sync>, )+
        }
    };
}

lazy_static::lazy_static! {
    static ref ANALYZER_RULES: IndexMap<&'static str, Box<dyn RuleDefinition + Send + Sync>> = {
        rule_list! {
            "CYAR-0001" => procedure::AddParamlistParenthesis,
            "CYAR-0002" => procedure::ReplacePrologue,
            "CYAR-0003" => procedure::ReplaceEpilogue,
            "CYAR-0004" => builtins::FixTrunc,
            "CYAR-0005" => builtins::ReplaceSysdate,
        }
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
    #[error("Rule '{0}' not found")]
    RuleNotFound(String),
    #[error("Failed to parse replacement: {0}")]
    ParseError(String),
    #[error("Language construct unsupported: {0:?}")]
    Unsupported(DboType),
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, TypescriptDefinition)]
pub struct RuleLocation {
    offset: Range<u32>,
}

impl RuleLocation {
    fn new(offset: Range<u32>) -> Self {
        Self { offset }
    }
}

impl From<TextRange> for RuleLocation {
    fn from(text_range: TextRange) -> Self {
        Self::new(text_range.start().into()..text_range.end().into())
    }
}

impl From<&RuleLocation> for TextRange {
    fn from(location: &RuleLocation) -> Self {
        TextRange::at(
            location.offset.start.into(),
            (location.offset.len() as u32).into(),
        )
    }
}

impl fmt::Display for RuleLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.offset.start, self.offset.end)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, TypescriptDefinition)]
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

pub fn apply_rule(
    typ: DboType,
    sql: &str,
    rule_name: &str,
    location: &RuleLocation,
    ctx: &DboAnalyzeContext,
) -> Result<(String, RuleLocation), RuleError> {
    let apply = |p: Parse| {
        let rule = ANALYZER_RULES
            .get(rule_name)
            .ok_or_else(|| RuleError::RuleNotFound(rule_name.to_owned()))?;

        let root = Root::cast(p.syntax())
            .ok_or_else(|| RuleError::ParseError("failed to find root node".to_owned()))?
            .clone_for_update();

        let node = rule.get_node(&root)?;
        let location = rule.apply(&node, location.into(), ctx)?;

        Ok((root.syntax().to_string(), location.into()))
    };

    match typ {
        DboType::Function => apply(parse_function(sql)?),
        DboType::Procedure => apply(parse_procedure(sql)?),
        DboType::Query => apply(parse_query(sql)?),
        _ => Err(RuleError::Unsupported(typ)),
    }
}

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
#[wasm_bindgen(js_name = "applyRule")]
pub fn js_apply_rule(
    typ: DboType,
    sql: &str,
    rule: &str,
    location: JsValue,
    ctx: JsValue,
) -> Result<JsValue, JsValue> {
    let location = serde_wasm_bindgen::from_value(location)?;
    let ctx = serde_wasm_bindgen::from_value(ctx)?;

    match apply_rule(typ, sql, rule, &location, &ctx) {
        Ok(location) => Ok(serde_wasm_bindgen::to_value(&location)?),
        Err(err) => Err(serde_wasm_bindgen::to_value(&err)?),
    }
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

    if !node.text_range().contains_range(location) {
        return Err(RuleError::InvalidLocation(location.into()));
    }

    let last = node.last_child_or_token().map(|e| e.index()).unwrap_or(0);

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

    let to_delete = start + to_delete.start..end + to_delete.end;

    // Check carefully that we also have a valid index range, as
    // `.splice_children()` will straight up panic with out-of-bounds indices.
    if to_delete.end > last {
        return Err(RuleError::InvalidLocation(location.into()));
    }

    node.splice_children(to_delete, vec![SyntaxElement::Node(replacement.clone())]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_apply_all_applicable_rules_on_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let context = DboAnalyzeContext::default();
        let result = crate::analyze(DboType::Procedure, INPUT, &context);
        assert!(result.is_ok(), "{:#?}", result);
        let mut metadata = result.unwrap();

        assert_eq!(metadata.rules.len(), 4);
        assert_eq!(metadata.rules[0].name, "CYAR-0001");
        assert_eq!(metadata.rules[1].name, "CYAR-0002");
        assert_eq!(metadata.rules[2].name, "CYAR-0003");
        assert_eq!(metadata.rules[3].name, "CYAR-0005");

        let mut transpiled = INPUT.to_owned();

        let mut do_apply = |rule: &RuleHint| {
            let result = apply_rule(
                DboType::Procedure,
                &transpiled,
                &rule.name,
                &rule.locations[0],
                &context,
            );
            assert!(result.is_ok(), "{:#?}", result);
            transpiled = result.unwrap().0;

            let result = crate::analyze(DboType::Procedure, &transpiled, &context);
            assert!(result.is_ok(), "{:#?}", result);
            result.unwrap()
        };

        assert_eq!(metadata.rules[0].name, "CYAR-0001");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0002");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0003");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0005");
        assert_eq!(metadata.rules[0].locations.len(), 2);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0005");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        do_apply(&metadata.rules[0]);

        expect![[r#"
            CREATE PROCEDURE secure_dml()
            AS $$
            BEGIN
              IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                    OR TO_CHAR (clock_timestamp(), 'DY') IN ('SAT', 'SUN') THEN
                RAISE_APPLICATION_ERROR (-20205,
                    'You may only make changes during normal office hours');
              END IF;
            END;
            $$ LANGUAGE plpgsql;
        "#]]
        .assert_eq(&transpiled);
    }
}
