// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements procedure-specific rules for transpiling PL/SQL to PL/pgSQL.

use rowan::NodeOrToken::Token;
use rowan::{Direction, TextRange};

use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, Procedure, ProcedureHeader, Root};
use crate::rules::find_children_nodes;
use crate::syntax::{SyntaxKind, SyntaxNode};

use super::{
    find_children_tokens, next_token, replace_token, RuleDefinition, RuleError, RuleLocation,
};

pub(super) struct AddParamlistParenthesis;

impl RuleDefinition for AddParamlistParenthesis {
    fn short_desc(&self) -> &'static str {
        "Add parameter list parentheses"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        root.procedure()
            .and_then(|p| p.header())
            .map(|h| h.syntax().clone())
            .ok_or(RuleError::NoSuchItem("Procedure header"))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        if let Some(header) = ProcedureHeader::cast(node.clone()) {
            if header.param_list().is_none() {
                let mut locations =
                    find_children_nodes(header.syntax(), |t| t.kind() == SyntaxKind::IdentGroup)
                        .map(|t| TextRange::at(t.text_range().end(), 0.into()))
                        .collect::<Vec<TextRange>>();

                if locations.is_empty() {
                    return Err(RuleError::NoSuchItem("Procedure identifier"));
                } else {
                    locations.truncate(1);
                    return Ok(locations);
                }
            } else {
                return Err(RuleError::NoChange);
            }
        }

        Err(RuleError::NoSuchItem("Procedure header"))
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
    fn short_desc(&self) -> &'static str {
        "Replace procedure prologue"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        root.procedure()
            .map(|p| p.syntax().clone())
            .ok_or(RuleError::NoSuchItem("Procedure"))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        if let Some(procedure) = Procedure::cast(node.clone()) {
            let mut locations = find_children_tokens(procedure.syntax(), |t| {
                t.kind() == SyntaxKind::Keyword
                    && ["is", "as"].contains(&t.text().to_lowercase().as_str())
                    && next_token(t)
                        .map(|t| t.kind() != SyntaxKind::DollarQuote)
                        .unwrap_or(true)
            })
            .map(|t| t.text_range())
            .collect::<Vec<TextRange>>();

            if locations.is_empty() {
                return Err(RuleError::NoChange);
            } else {
                locations.truncate(1);
                return Ok(locations);
            }
        }

        Err(RuleError::NoSuchItem("Procedure prologue"))
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
    fn short_desc(&self) -> &'static str {
        "Replace procedure epilogue"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        root.procedure()
            .and_then(|p| p.body())
            .map(|p| p.syntax().clone())
            .ok_or(RuleError::NoSuchItem("Procedure"))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        let locations = find_children_tokens(node, |t| {
            // find an `END` keyword
            if !(t.kind() == SyntaxKind::Keyword && t.text().to_string().to_lowercase() == "end") {
                return false;
            }
            dbg!(&t
                .parent()
                .unwrap()
                .next_sibling_or_token()
                .and_then(|t| match t.kind() {
                    SyntaxKind::Whitespace => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::DollarQuote => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::Whitespace => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::Ident => Some(t.to_string().to_lowercase() == "language"),
                    _ => None,
                }));
            // determine if the `LANGUAGE` epilogue has already been applied
            t.parent()
                .unwrap()
                .next_sibling_or_token()
                .and_then(|t| match t.kind() {
                    SyntaxKind::Whitespace => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::DollarQuote => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::Whitespace => t.next_sibling_or_token(),
                    _ => None,
                })
                .and_then(|t| match t.kind() {
                    SyntaxKind::Ident => Some(t.to_string().to_lowercase() == "language"),
                    _ => None,
                })
                .is_none()
        })
        .map(|t| {
            let start = t.clone();
            let end = t
                .siblings_with_tokens(Direction::Next)
                .find(|t| t.kind() == SyntaxKind::IdentGroup)
                .unwrap_or(Token(t));
            TextRange::new(start.text_range().end(), end.text_range().end())
        })
        .collect::<Vec<TextRange>>();

        if locations.is_empty() {
            Err(RuleError::NoChange)
        } else {
            Ok(locations)
        }
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
                start: location.clone().start,
                end: location.clone().start,
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
    fn short_desc(&self) -> &'static str {
        "Remove `EDITIONABLE` or `NONEDITIONABLE`"
    }

    fn get_node(&self, root: &Root) -> Result<SyntaxNode, RuleError> {
        if let Some(header) = root.procedure().and_then(|p| p.header()) {
            return Ok(header.syntax().clone());
        }

        if let Some(header) = root.function().and_then(|p| p.header()) {
            return Ok(header.syntax().clone());
        }

        Err(RuleError::NoSuchItem("Procedure or function"))
    }

    fn find(
        &self,
        node: &SyntaxNode,
        _ctx: &DboAnalyzeContext,
    ) -> Result<Vec<TextRange>, RuleError> {
        let locations = find_children_tokens(node, |t| {
            t.kind() == SyntaxKind::Keyword
                && (t.text().to_lowercase() == "editionable"
                    || t.text().to_lowercase() == "noneditionable")
        })
        .map(|t| t.text_range())
        .collect::<Vec<TextRange>>();

        if locations.is_empty() {
            Err(RuleError::NoChange)
        } else {
            Ok(locations)
        }
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
    use expect_test::{expect, Expect};
    use pretty_assertions::assert_eq;

    use crate::ast::AstNode;
    use crate::syntax::SyntaxNode;

    use super::*;

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
    }

    #[test]
    fn test_replace_editionable() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_editionable.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = RemoveEditionable;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let mut locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        let location = locations.pop().unwrap();
        assert_eq!(location, TextRange::new(92.into(), 103.into()));
        assert_eq!(&root.syntax().to_string()[location], "EDITIONABLE");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(
            &node,
            &RuleLocation::from(INPUT, location),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
-- test: ignore EDITIONABLE keyword, there is no equivalent in PostgreSQL
CREATE OR REPLACE PROCEDURE ignore_editionable
IS
BEGIN
    NULL;
END ignore_editionable;
"#]],
        );
        assert_eq!(location, TextRange::new(92.into(), 92.into()));
        assert_eq!(&root.syntax().to_string()[location], "");
    }

    #[test]
    fn test_add_paramlist_parens() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = AddParamlistParenthesis;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let mut locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        let location = locations.pop().unwrap();
        assert_eq!(location, TextRange::new(27.into(), 27.into()));
        assert_eq!(&root.syntax().to_string()[location], "");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(
            &node,
            &RuleLocation::from(INPUT, location),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
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
        assert_eq!(location, TextRange::new(27.into(), 29.into()));
        assert_eq!(&root.syntax().to_string()[location], "()");
    }

    #[test]
    fn test_replace_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = ReplacePrologue;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let mut locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        let location = locations.pop().unwrap();
        assert_eq!(location, TextRange::new(28.into(), 30.into()));
        assert_eq!(&root.syntax().to_string()[location], "IS");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(
            &node,
            &RuleLocation::from(INPUT, location),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
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
        assert_eq!(location, TextRange::new(28.into(), 33.into()));
        assert_eq!(&root.syntax().to_string()[location], "AS $$");
    }

    #[test]
    fn test_replace_procedure_epilogue() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = ReplaceEpilogue;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let mut locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        let location = locations.pop().unwrap();
        assert_eq!(location, TextRange::new(276.into(), 287.into()));
        assert_eq!(&root.syntax().to_string()[location], " secure_dml");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();
        let result = rule.apply(
            &node,
            &RuleLocation::from(INPUT, location),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
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
        assert_eq!(location, TextRange::new(277.into(), 298.into()));
        assert_eq!(
            &root.syntax().to_string()[location],
            "\n$$ LANGUAGE plpgsql;"
        );
    }

    #[test]
    fn dont_add_second_pair_of_parentheses_for_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/empty_parameter_list.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = AddParamlistParenthesis;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert_eq!(result, Err(RuleError::NoChange));
    }

    #[test]
    fn accept_either_is_or_as_in_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/procedure/heading/procedure_as.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let rule = ReplacePrologue;

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.find(&node, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let mut locations = result.unwrap();
        assert_eq!(locations.len(), 1);
        let location = locations.pop().unwrap();
        assert_eq!(location, TextRange::new(74.into(), 76.into()));
        assert_eq!(&root.syntax().to_string()[location], "AS");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(
            &node,
            &RuleLocation::from(INPUT, location),
            &DboAnalyzeContext::default(),
        );
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check(
            root.syntax().clone(),
            expect![[r#"
                -- test: use of AS instead of IS
                CREATE OR REPLACE PROCEDURE procedure_as
                AS $$
                BEGIN
                    NULL;
                END procedure_as;
            "#]],
        );
        assert_eq!(location, TextRange::new(74.into(), 79.into()));
        assert_eq!(&root.syntax().to_string()[location], "AS $$");
    }
}
