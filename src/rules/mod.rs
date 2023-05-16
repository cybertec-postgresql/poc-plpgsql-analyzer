// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements rules for transpiling PL/SQL to PL/pgSQL

extern crate unicode_width;

use std::fmt;
use std::ops::Range;

use indexmap::IndexMap;
use rowan::{Direction, GreenNode, GreenToken, NodeOrToken, TextRange, TokenAtOffset};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

use crate::analyze::{DboAnalyzeContext, DboType};
use crate::ast::{AstNode, Function, Param, Procedure, Root};
use crate::parser::*;
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub mod builtins;
pub mod procedure;

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
            "CYAR-0006" => builtins::ReplaceNvl,
            "CYAR-0007" => procedure::RemoveEditionable,
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
    fn find(&self, root: &Root, ctx: &DboAnalyzeContext) -> Result<Vec<RuleMatch>, RuleError>;
    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError>;
}

#[derive(Debug, Eq, PartialEq)]
struct RuleMatch {
    node: SyntaxNode,
    range: TextRange,
}

impl RuleMatch {
    pub fn new(node: SyntaxNode, range: TextRange) -> Self {
        Self { node, range }
    }
    pub fn from_node(node: &SyntaxNode) -> Self {
        Self {
            node: node.clone(),
            range: node.text_range(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, TypescriptDefinition)]
pub struct RuleLocation {
    offset: Range<u32>,
    start: LineCol,
    end: LineCol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, TypescriptDefinition)]
pub struct LineCol {
    line: u32,
    col: u32,
}

impl RuleLocation {
    /// The caller must take care that the offset is always valid for the given
    /// text.
    fn from(text: &str, range: TextRange) -> Self {
        Self {
            offset: range.into(),
            start: position_to_line_col(text, range.start().into()),
            end: position_to_line_col(text, range.end().into()),
        }
    }

    fn text_range(&self) -> TextRange {
        TextRange::new(self.offset.start.into(), self.offset.end.into())
    }
}

/// Determine the Line:Column for an offset in a &str
fn position_to_line_col(text: &str, pos: usize) -> LineCol {
    let line = text[0..pos].matches('\n').count() as u32 + 1;

    let line_start = text[0..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = (UnicodeWidthStr::width(&text[line_start..pos])) as u32 + 1;

    LineCol { line, col }
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

pub fn find_applicable_rules(input: &str, root: &Root, ctx: &DboAnalyzeContext) -> Vec<RuleHint> {
    ANALYZER_RULES
        .iter()
        .filter_map(|(name, rule)| {
            rule.find(root, ctx)
                .map(|ranges| RuleHint {
                    name: (*name).to_owned(),
                    locations: ranges
                        .into_iter()
                        .map(|r| RuleLocation::from(input, r.range))
                        .collect(),
                    short_desc: rule.short_desc(),
                })
                .ok()
        })
        .collect()
}

pub fn apply_rule(
    typ: DboType,
    sql: &str,
    rule_name: &str,
    location: Option<&RuleLocation>,
    ctx: &DboAnalyzeContext,
) -> Result<(String, Vec<RuleLocation>), RuleError> {
    let apply = |p: Parse| {
        let rule = ANALYZER_RULES
            .get(rule_name)
            .ok_or_else(|| RuleError::RuleNotFound(rule_name.to_owned()))?;

        let root = Root::cast(p.syntax())
            .ok_or_else(|| RuleError::ParseError("failed to find root node".to_owned()))?
            .clone_for_update();

        if let Some(location) = location {
            // find the node that matches the given location
            let occurrences = &rule.find(&root, ctx).unwrap();
            let node = occurrences
                .iter()
                .find(|p| (p.range.start().into()..p.range.end().into()) == location.offset);
            let range = rule.apply(&node.unwrap().node, location, ctx)?;
            let text = root.syntax().to_string();
            let location = RuleLocation::from(&text, range);
            Ok((root.syntax().to_string(), vec![location]))
        } else {
            let mut result = Vec::new();
            while let Ok(locations) = rule.find(&root, ctx) {
                if locations.is_empty() {
                    break;
                }

                let location = RuleLocation::from(&root.syntax().to_string(), locations[0].range);
                let range = rule.apply(&locations[0].node, &location, ctx)?;
                result.push(range);
            }

            let text = root.syntax().to_string();
            let result = result
                .into_iter()
                .map(|r| RuleLocation::from(&text, r))
                .collect();
            Ok((text, result))
        }
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
    let location: Option<RuleLocation> = serde_wasm_bindgen::from_value(location)?;
    let ctx = serde_wasm_bindgen::from_value(ctx)?;

    match apply_rule(typ, sql, rule, location.as_ref(), &ctx) {
        Ok(location) => Ok(serde_wasm_bindgen::to_value(&location)?),
        Err(err) => Err(serde_wasm_bindgen::to_value(&err)?),
    }
}

/// Finds all descendant nodes within an AST node that fulfill the predicate
fn filter_map_descendant_nodes<B, F>(root: &Root, f: F) -> impl Iterator<Item = B>
where
    F: Fn(SyntaxNode) -> Option<B>,
{
    root.syntax().descendants().filter_map(f)
}

/// Returns the next non-whitespace sibling token that follows.
///
/// # Arguments
///
/// `token`: The token to find the next non-whitespace sibling token of.
fn next_token(token: &SyntaxToken) -> Option<SyntaxToken> {
    token
        .siblings_with_tokens(Direction::Next)
        .filter_map(SyntaxElement::into_token)
        .filter(|t| t.kind() != SyntaxKind::Whitespace)
        .nth(1)
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
/// `kind`: If set, becomes the root node of the inserted AST subtree. If unset,
/// all children of of the AST subtree are inserted instead, without a node
/// nesting them in the existing AST.
///
/// `replace_offset`: The offset and range from the found token to delete some
/// tokens at. If the range is empty, no tokens are deleted.
fn replace_token(
    node: &SyntaxNode,
    location: &RuleLocation,
    replacement: &str,
    kind: Option<SyntaxKind>,
    to_delete: Range<usize>,
) -> Result<TextRange, RuleError> {
    let replacement = parse_any(replacement)?.syntax().clone_for_update();

    let text_range = location.text_range();
    if !node.text_range().contains_range(text_range) {
        return Err(RuleError::InvalidLocation(location.clone()));
    }

    let last = node.last_child_or_token().map(|e| e.index()).unwrap_or(0);

    let start = match node.token_at_offset(text_range.start()) {
        TokenAtOffset::None => return Err(RuleError::InvalidLocation(location.clone())),
        TokenAtOffset::Single(t) => t.index(),
        TokenAtOffset::Between(_, t) => t.index(),
    };
    let end = match node.token_at_offset(text_range.start()) {
        TokenAtOffset::None => return Err(RuleError::InvalidLocation(location.clone())),
        TokenAtOffset::Single(t) => t.index(),
        TokenAtOffset::Between(_, t) => t.index(),
    };

    let to_delete = start + to_delete.start..end + to_delete.end;

    // Check carefully that we also have a valid index range, as
    // `.splice_children()` will straight up panic with out-of-bounds indices.
    if to_delete.end > last + 1 {
        return Err(RuleError::InvalidLocation(location.clone()));
    }

    if let Some(kind) = kind {
        // If we have a syntax kind, we have to first construct a new green tree to then
        // create a new green node of the specified type. And to do that, we
        // have to convert all children syntax elements into their green tree
        // equvilant first.
        let children = replacement
            .children_with_tokens()
            .map(|elem| match elem {
                SyntaxElement::Node(n) => NodeOrToken::Node(n.green().into_owned()),
                SyntaxElement::Token(t) => NodeOrToken::Token(t.green().to_owned()),
            })
            .collect::<Vec<NodeOrToken<GreenNode, GreenToken>>>();

        let child = SyntaxNode::new_root(GreenNode::new(kind.into(), children)).clone_for_update();

        // Now insert it as one single node into the original AST.
        node.splice_children(to_delete, vec![SyntaxElement::Node(child.clone())]);

        // The child node was modified, so it's location now represents it in the AST we
        // inserted it into.
        Ok(child.text_range())
    } else {
        // Just insert all the children elements without a nesting node.
        let children = replacement
            .children_with_tokens()
            .collect::<Vec<SyntaxElement>>();

        node.splice_children(to_delete, children.clone());

        // The rule might just delete some token, without inserting any. Return the
        // correct location in any case.
        if let (Some(first), Some(last)) = (children.first(), children.last()) {
            Ok(TextRange::new(
                first.text_range().start(),
                last.text_range().end(),
            ))
        } else {
            // TODO: Is this what we want? Also, completely untested as we do not have
            // any rule yet which only deletes tokens, without also inserting some.
            Ok(TextRange::new(text_range.start(), text_range.start()))
        }
    }
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
            if let (Some(t), Some(c)) = (ident.nth(0), ident.nth(1)) {
                if ctx
                    .table_column(&t.text().to_string().into(), &c.text().to_string().into())
                    .is_none()
                {
                    return Err(RuleError::NoTableInfo(t.text()));
                }
            } else {
                return Err(RuleError::InvalidTypeRef(ident.name().unwrap()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use pretty_assertions::assert_eq;

    use super::*;

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
                Some(&rule.locations[0]),
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
        assert_eq!(metadata.rules[0].locations[0].offset, 27..27);
        assert_eq!(metadata.rules[0].locations[0].start.line, 1);
        assert_eq!(metadata.rules[0].locations[0].start.col, 28);
        assert_eq!(metadata.rules[0].locations[0].end.line, 1);
        assert_eq!(metadata.rules[0].locations[0].end.col, 28);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0002");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        assert_eq!(metadata.rules[0].locations[0].offset, 30..32);
        assert_eq!(metadata.rules[0].locations[0].start.line, 2);
        assert_eq!(metadata.rules[0].locations[0].start.col, 1);
        assert_eq!(metadata.rules[0].locations[0].end.line, 2);
        assert_eq!(metadata.rules[0].locations[0].end.col, 3);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0003");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        assert_eq!(metadata.rules[0].locations[0].offset, 281..292);
        assert_eq!(metadata.rules[0].locations[0].start.line, 9);
        assert_eq!(metadata.rules[0].locations[0].start.col, 4);
        assert_eq!(metadata.rules[0].locations[0].end.line, 9);
        assert_eq!(metadata.rules[0].locations[0].end.col, 15);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0005");
        assert_eq!(metadata.rules[0].locations.len(), 2);
        assert_eq!(metadata.rules[0].locations[0].offset, 56..63);
        assert_eq!(metadata.rules[0].locations[0].start.line, 4);
        assert_eq!(metadata.rules[0].locations[0].start.col, 15);
        assert_eq!(metadata.rules[0].locations[0].end.line, 4);
        assert_eq!(metadata.rules[0].locations[0].end.col, 22);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0005");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        assert_eq!(metadata.rules[0].locations[0].offset, 138..145);
        assert_eq!(metadata.rules[0].locations[0].start.line, 5);
        assert_eq!(metadata.rules[0].locations[0].start.col, 21);
        assert_eq!(metadata.rules[0].locations[0].end.line, 5);
        assert_eq!(metadata.rules[0].locations[0].end.col, 28);
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

    #[test]
    fn test_apply_all_applicable_rules_on_procedure_without_location() {
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
            let result = apply_rule(DboType::Procedure, &transpiled, &rule.name, None, &context);
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

    #[test]
    fn test_unicode_width_in_rule_location() {
        const INPUT: &str = include_str!("../../tests/fixtures/unicode_characters.ora.sql");

        let context = DboAnalyzeContext::default();
        let result = crate::analyze(DboType::Procedure, INPUT, &context);
        assert!(result.is_ok(), "{:#?}", result);
        let mut metadata = result.unwrap();

        assert_eq!(metadata.rules.len(), 3);
        assert_eq!(metadata.rules[0].name, "CYAR-0001");
        assert_eq!(metadata.rules[1].name, "CYAR-0002");
        assert_eq!(metadata.rules[2].name, "CYAR-0003");

        let mut transpiled = INPUT.to_owned();

        let mut do_apply = |rule: &RuleHint| {
            let result = apply_rule(
                DboType::Procedure,
                &transpiled,
                &rule.name,
                Some(&rule.locations[0]),
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
        assert_eq!(metadata.rules[0].locations[0].offset, 40..40);
        assert_eq!(metadata.rules[0].locations[0].start.line, 1);
        assert_eq!(metadata.rules[0].locations[0].start.col, 30);
        assert_eq!(metadata.rules[0].locations[0].end.line, 1);
        assert_eq!(metadata.rules[0].locations[0].end.col, 30);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0002");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        assert_eq!(metadata.rules[0].locations[0].offset, 43..45);
        assert_eq!(metadata.rules[0].locations[0].start.line, 2);
        assert_eq!(metadata.rules[0].locations[0].start.col, 1);
        assert_eq!(metadata.rules[0].locations[0].end.line, 2);
        assert_eq!(metadata.rules[0].locations[0].end.col, 3);
        metadata = do_apply(&metadata.rules[0]);

        assert_eq!(metadata.rules[0].name, "CYAR-0003");
        assert_eq!(metadata.rules[0].locations.len(), 1);
        assert_eq!(metadata.rules[0].locations[0].offset, 77..101);
        assert_eq!(metadata.rules[0].locations[0].start.line, 4);
        assert_eq!(metadata.rules[0].locations[0].start.col, 4);
        assert_eq!(metadata.rules[0].locations[0].end.line, 4);
        assert_eq!(metadata.rules[0].locations[0].end.col, 17);
        do_apply(&metadata.rules[0]);

        expect![[r#"CREATE PROCEDURE "读文👩🏼‍🔬"()
AS $$ BEGIN
  NULL; -- メ メ
END;
$$ LANGUAGE plpgsql;
"#]]
        .assert_eq(&transpiled);
    }
}
