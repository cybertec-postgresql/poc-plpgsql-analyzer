// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements procedure-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{replace_child, RuleError};
use crate::analyze::DboAnalyzeContext;
use crate::ast::{AstToken, Ident, Root};
use crate::syntax::SyntaxKind;
use rowan::TextRange;

pub(super) fn add_paramlist_parens(
    root: &Root,
    location: Option<TextRange>,
    _ctx: &DboAnalyzeContext,
) -> Result<TextRange, RuleError> {
    if let Some(header) = root.procedure().and_then(|p| p.header()) {
        if header.param_list().is_none() {
            return replace_child(
                &header,
                |t| t.kind() == SyntaxKind::Ident,
                location,
                |t| TextRange::at(t.text_range().end(), 0.into()),
                "Procedure identifier",
                "()",
                1..1,
            );
        } else {
            return Err(RuleError::NoChange);
        }
    }

    Err(RuleError::NoSuchItem("Procedure header"))
}

pub(super) fn replace_procedure_prologue(
    root: &Root,
    location: Option<TextRange>,
    _ctx: &DboAnalyzeContext,
) -> Result<TextRange, RuleError> {
    if let Some(procedure) = root.procedure() {
        return replace_child(
            &procedure,
            |t| {
                t.kind() == SyntaxKind::Keyword
                    && ["is", "as"].contains(&t.text().to_lowercase().as_str())
            },
            location,
            |t| t.text_range(),
            "Procedure prologue",
            "AS $$",
            0..1,
        );
    }

    Err(RuleError::NoSuchItem("Procedure prologue"))
}

pub(super) fn replace_procedure_epilogue(
    root: &Root,
    location: Option<TextRange>,
    _ctx: &DboAnalyzeContext,
) -> Result<TextRange, RuleError> {
    if let Some(procedure) = root.procedure() {
        return replace_child(
            &procedure,
            |t| t.kind() == SyntaxKind::Keyword && t.text().to_lowercase() == "end",
            location,
            |t| {
                let end = t
                    .siblings_with_tokens(rowan::Direction::Next)
                    .filter_map(|it| it.into_token())
                    .find(|t| Ident::cast(t.clone()).is_some())
                    .map(|e| e.text_range().end())
                    .unwrap_or_else(|| t.text_range().end());

                TextRange::new(t.text_range().start(), end)
            },
            "Procedure epilogue",
            ";\n$$ LANGUAGE plpgsql",
            1..3,
        );
    }

    Err(RuleError::NoSuchItem("Procedure epilogue"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;
    use crate::syntax::SyntaxNode;
    use expect_test::{expect, expect_file, Expect, ExpectFile};
    use pretty_assertions::assert_eq;

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
    }

    fn check_file(node: SyntaxNode, expect: ExpectFile) {
        expect.assert_eq(&node.to_string());
    }

    #[test]
    fn test_add_paramlist_parens() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let result = add_paramlist_parens(&root, None, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check_file(
            root.syntax().clone(),
            expect_file!["../../tests/fixtures/secure_dml.sql"],
        );
        assert_eq!(location, TextRange::new(27.into(), 27.into()));
        assert_eq!(&root.syntax().to_string()[location], "");

        let root = root.clone_for_update();
        let result = add_paramlist_parens(&root, Some(location), &DboAnalyzeContext::default());
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
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let result = replace_procedure_prologue(&root, None, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check_file(
            root.syntax().clone(),
            expect_file!["../../tests/fixtures/secure_dml.sql"],
        );
        assert_eq!(location, TextRange::new(28.into(), 30.into()));
        assert_eq!(&root.syntax().to_string()[location], "IS");

        let root = root.clone_for_update();
        let result =
            replace_procedure_prologue(&root, Some(location), &DboAnalyzeContext::default());
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
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let result = replace_procedure_epilogue(&root, None, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check_file(
            root.syntax().clone(),
            expect_file!["../../tests/fixtures/secure_dml.sql"],
        );
        assert_eq!(location, TextRange::new(273.into(), 287.into()));
        assert_eq!(&root.syntax().to_string()[location], "END secure_dml");

        let root = root.clone_for_update();
        let result =
            replace_procedure_epilogue(&root, Some(location), &DboAnalyzeContext::default());
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
        let result = add_paramlist_parens(&root, None, &DboAnalyzeContext::default());
        assert_eq!(result, Err(RuleError::NoChange));
        check_file(
            root.syntax().clone(),
            expect_file!["../../tests/fixtures/empty_parameter_list.sql"],
        );
    }

    #[test]
    fn accept_either_is_or_as_in_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/procedure/heading//procedure_as.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let result = replace_procedure_prologue(&root, None, &DboAnalyzeContext::default());
        assert!(result.is_ok(), "{:#?}", result);

        let location = result.unwrap();
        check_file(
            root.syntax().clone(),
            expect_file!["../../tests/procedure/heading//procedure_as.ora.sql"],
        );
        assert_eq!(location, TextRange::new(74.into(), 76.into()));
        assert_eq!(&root.syntax().to_string()[location], "AS");

        let root = root.clone_for_update();
        let result =
            replace_procedure_prologue(&root, Some(location), &DboAnalyzeContext::default());
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
