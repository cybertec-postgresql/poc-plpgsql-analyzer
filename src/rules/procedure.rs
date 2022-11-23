// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements procedure-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{
    find_children_tokens, find_sibling_token, next_token, replace_token, RuleDefinition, RuleError,
};
use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstNode, Procedure, ProcedureHeader, Root};
use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::TextRange;

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
                    find_children_tokens(header.syntax(), |t| t.kind() == SyntaxKind::Ident)
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
        location: TextRange,
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
        location: TextRange,
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
                    && t.text().to_lowercase() == "end"
                    && next_token(t)
                        .map(|t| t.kind() == SyntaxKind::Ident)
                        .unwrap_or(true)
            })
            .map(|t| {
                let end = find_sibling_token(&t, |t| t.kind() == SyntaxKind::Ident)
                    .unwrap_or_else(|| t.clone());
                TextRange::new(t.text_range().start(), end.text_range().end())
            })
            .collect::<Vec<TextRange>>();

            if locations.is_empty() {
                return Err(RuleError::NoChange);
            } else {
                locations.truncate(1);
                return Ok(locations);
            }
        }

        Err(RuleError::NoSuchItem("Procedure epilogue"))
    }

    fn apply(
        &self,
        node: &SyntaxNode,
        location: TextRange,
        _ctx: &DboAnalyzeContext,
    ) -> Result<TextRange, RuleError> {
        replace_token(node, location, ";\n$$ LANGUAGE plpgsql", None, 1..3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;
    use crate::syntax::SyntaxNode;
    use expect_test::{expect, Expect};
    use pretty_assertions::assert_eq;

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
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

        let result = rule.apply(&node, location, &DboAnalyzeContext::default());
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

        let result = rule.apply(&node, location, &DboAnalyzeContext::default());
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
        assert_eq!(location, TextRange::new(273.into(), 287.into()));
        assert_eq!(&root.syntax().to_string()[location], "END secure_dml");

        let root = root.clone_for_update();

        let result = rule.get_node(&root);
        assert!(result.is_ok(), "{:#?}", result);
        let node = result.unwrap();

        let result = rule.apply(&node, location, &DboAnalyzeContext::default());
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
        assert_eq!(location, TextRange::new(276.into(), 297.into()));
        assert_eq!(
            &root.syntax().to_string()[location],
            ";\n$$ LANGUAGE plpgsql"
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

        let result = rule.apply(&node, location, &DboAnalyzeContext::default());
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
