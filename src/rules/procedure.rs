// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements procedure-specific rules for transpiling PL/SQL to PL/pgSQL.

use rowan::NodeOrToken::Token;
use rowan::{Direction, TextRange};

use crate::analyzer::DboAnalyzeContext;
use crate::ast::{AstNode, Function, Procedure, Root};
use crate::rules::{filter_map_descendant_nodes, RuleMatch};
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use super::{next_token, replace_token, RuleDefinition, RuleError, RuleLocation};

pub(super) struct AddParamlistParenthesis;

impl RuleDefinition for AddParamlistParenthesis {
    fn short_desc(&self) -> String {
        "Add parameter list parentheses".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, Procedure::cast)
            .filter_map(|p| p.header())
            .filter(|h| h.param_list().is_none())
            .map(|h| {
                RuleMatch::new(
                    h.syntax().clone(),
                    TextRange::at(
                        h.identifier().unwrap().syntax().text_range().end(),
                        0.into(),
                    ),
                )
            })
            .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "()", Some(SyntaxKind::ParamList), 0..0)
    }
}

pub(super) struct ReplacePrologue;

impl RuleDefinition for ReplacePrologue {
    fn short_desc(&self) -> String {
        "Replace procedure prologue".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, Procedure::cast)
            .filter_map(|p| {
                p.syntax()
                    .children_with_tokens()
                    .filter_map(SyntaxElement::into_token)
                    .find(|t| {
                        t.kind() == SyntaxKind::Keyword
                            && ["is", "as"].contains(&t.text().to_lowercase().as_str())
                            && next_token(t)
                                .map(|t| t.kind() != SyntaxKind::DollarQuote)
                                .unwrap_or(true)
                    })
                    .map(|t| RuleMatch::new(p.syntax().clone(), t.text_range()))
            })
            .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "AS $$", None, 0..1)
    }
}

pub(super) struct ReplaceEpilogue;

impl RuleDefinition for ReplaceEpilogue {
    fn short_desc(&self) -> String {
        "Replace procedure epilogue".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, Procedure::cast)
            .filter_map(|p| p.body())
            .filter_map(|p| {
                p.syntax()
                    .children_with_tokens()
                    .filter_map(SyntaxElement::into_token)
                    .find(|t| {
                        // Find an `END` keyword
                        if t.kind() == SyntaxKind::Keyword
                            && t.text().to_string().to_lowercase() == "end"
                        {
                            // Grab the next four syntax tokens
                            let syntax_tokens = t
                                .parent()
                                .unwrap()
                                .siblings_with_tokens(Direction::Next)
                                .skip(1)
                                .take(4)
                                .collect::<Vec<_>>();

                            // If the token sequence represents `\n$$ LANGUAGE`, the rule has already been applied
                            return if syntax_tokens.iter().map(|t| t.kind()).collect::<Vec<_>>()
                                != [
                                    SyntaxKind::Whitespace,
                                    SyntaxKind::DollarQuote,
                                    SyntaxKind::Whitespace,
                                    SyntaxKind::Ident,
                                ] {
                                true
                            } else {
                                syntax_tokens[3].to_string().to_lowercase() != "language"
                            };
                        }
                        false
                    })
                    .map(|t| {
                        let start = t.clone();
                        let end = t
                            .siblings_with_tokens(Direction::Next)
                            .find(|t| t.kind() == SyntaxKind::IdentGroup)
                            .unwrap_or(Token(t));
                        RuleMatch::new(
                            p.syntax().clone(),
                            TextRange::new(start.text_range().end(), end.text_range().end()),
                        )
                    })
            })
            .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        // removes the block identifier if present
        if location.offset.start < location.offset.end {
            replace_token(node, location, "", None, 0..2)?;
        }

        // find the the offset for the end of the block
        let block_end = node.text_range().end();

        let text_range = replace_token(
            &node.parent().unwrap(),
            &RuleLocation {
                offset: block_end.into()..block_end.into(),
                start: location.start,
                end: location.start,
            },
            "\n$$ LANGUAGE plpgsql;",
            None,
            0..0,
        )?;
        Ok(text_range)
    }
}

pub(super) struct RemoveEditionable;

impl RuleDefinition for RemoveEditionable {
    fn short_desc(&self) -> String {
        "Remove `EDITIONABLE` or `NONEDITIONABLE`".to_string()
    }

    fn find_rules(
        &self,
        root: &Root,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<RuleMatch>, RuleError> {
        let locations: Vec<RuleMatch> = filter_map_descendant_nodes(root, |n| {
            if let Some(procedure) = Procedure::cast(n.clone()) {
                procedure.header().map(|p| p.syntax().clone())
            } else if let Some(function) = Function::cast(n) {
                function.header().map(|f| f.syntax().clone())
            } else {
                None
            }
        })
        .filter_map(|n| {
            n.children_with_tokens()
                .filter_map(SyntaxElement::into_token)
                .find(|t| {
                    t.kind() == SyntaxKind::Keyword
                        && (t.text().to_lowercase() == "editionable"
                            || t.text().to_lowercase() == "noneditionable")
                })
                .map(|t| RuleMatch::new(n.clone(), t.text_range()))
        })
        .collect();

        Ok(locations)
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: &RuleLocation,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, "", None, 0..2)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use pretty_assertions::assert_eq;

    use crate::rules::tests::{apply_first_rule, check_node, parse_root};

    use super::*;

    #[test]
    fn test_replace_editionable() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_editionable.ora.sql");
        let mut root = parse_root(INPUT, crate::parse_procedure);
        let rule = RemoveEditionable;

        apply_first_rule(&rule, &mut root).expect("Failed to apply rule");
        check_node(
            &root,
            expect![[r#"
    -- test: ignore EDITIONABLE keyword, there is no equivalent in PostgreSQL
    CREATE OR REPLACE PROCEDURE ignore_editionable
    IS
    BEGIN
        NULL;
    END ignore_editionable;
    "#]],
        );
    }

    #[test]
    fn test_add_paramlist_parens() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");
        let mut root = parse_root(INPUT, crate::parse_procedure);
        let rule = AddParamlistParenthesis;

        apply_first_rule(&rule, &mut root).expect("Failed to apply rule");
        check_node(
            &root,
            expect![[r#"
                CREATE PROCEDURE secure_dml()
                IS
                BEGIN
                  IF TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
                    RAISE_APPLICATION_ERROR (-20205,
                        'You may only make changes during normal office hours');
                  END IF;
                END secure_dml;
            "#]],
        );
    }

    #[test]
    fn test_replace_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");
        let mut root = parse_root(INPUT, crate::parse_procedure);
        let rule = ReplacePrologue;

        apply_first_rule(&rule, &mut root).expect("Failed to apply rule");
        check_node(
            &root,
            expect![[r#"
                CREATE PROCEDURE secure_dml
                AS $$
                BEGIN
                  IF TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
                    RAISE_APPLICATION_ERROR (-20205,
                        'You may only make changes during normal office hours');
                  END IF;
                END secure_dml;
            "#]],
        );
    }

    #[test]
    fn test_replace_procedure_epilogue() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");
        let mut root = parse_root(INPUT, crate::parse_procedure);
        let rule = ReplaceEpilogue;

        apply_first_rule(&rule, &mut root).expect("Failed to apply rule");
        check_node(
            &root,
            expect![[r#"
                CREATE PROCEDURE secure_dml
                IS
                BEGIN
                  IF TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
                        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
                    RAISE_APPLICATION_ERROR (-20205,
                        'You may only make changes during normal office hours');
                  END IF;
                END;
                $$ LANGUAGE plpgsql;
            "#]],
        );
    }

    #[test]
    fn dont_add_second_pair_of_parentheses_for_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/empty_parameter_list.sql");
        let root = parse_root(INPUT, crate::parse_procedure);
        let rule = AddParamlistParenthesis;

        let result = rule.find_rules(&root, &DboAnalyzeContext::default());
        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn accept_either_is_or_as_in_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/procedure/heading/procedure_as.ora.sql");
        let mut root = parse_root(INPUT, crate::parse_procedure);
        let rule = ReplacePrologue;

        apply_first_rule(&rule, &mut root).expect("Failed to apply rule");
        check_node(
            &root,
            expect![[r#"
                -- test: use of AS instead of IS
                CREATE OR REPLACE PROCEDURE procedure_as
                AS $$
                BEGIN
                    NULL;
                END procedure_as;
            "#]],
        );
    }
}
