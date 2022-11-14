// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements procedure-specific rules for transpiling PL/SQL to PL/pgSQL.

use super::{define_rule, RuleChanges, RuleError, RuleHint};
use crate::ast::{AstNode, Procedure, ProcedureHeader, Root};
use crate::syntax::SyntaxKind;

fn add_paramlist_parens(header: &ProcedureHeader) -> Result<RuleHint, RuleError> {
    define_rule(
        header,
        |t| t.kind() == SyntaxKind::Ident,
        "Failed to find procedure identifier",
        "()",
        1..1,
        "Add parameter parentheses",
    )
}

fn replace_procedure_prologue(procedure: &Procedure) -> Result<RuleHint, RuleError> {
    define_rule(
        procedure,
        |t| {
            t.kind() == SyntaxKind::Keyword
                && ["is", "as"].contains(&t.text().to_lowercase().as_str())
        },
        "Failed to find procedure prologue",
        "AS $$",
        0..1,
        "Replace procedure prologue",
    )
}

fn replace_procedure_epilogue(procedure: &Procedure) -> Result<RuleHint, RuleError> {
    define_rule(
        procedure,
        |t| t.kind() == SyntaxKind::Keyword && t.text().to_lowercase() == "end",
        "Failed to find procedure epilogue",
        ";\n$$ LANGUAGE plpgsql",
        1..3,
        "Replace procedure epilogue",
    )
}

pub fn fix_header(root: &Root) -> Result<RuleChanges, RuleError> {
    let replacement = root.clone_for_update();
    let mut hints = Vec::new();

    if let Some(procedure) = replacement.procedure() {
        if let Some(header) = procedure.header() {
            if header.param_list().is_none() {
                hints.push(add_paramlist_parens(&header)?);
            }
        }

        hints.push(replace_procedure_prologue(&procedure)?);
        hints.push(replace_procedure_epilogue(&procedure)?);
    }

    Ok(RuleChanges {
        replacement: replacement.syntax().clone(),
        hints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::SyntaxNode;
    use expect_test::{expect, Expect};

    fn check(node: SyntaxNode, expect: Expect) {
        expect.assert_eq(&node.to_string());
    }

    #[test]
    fn replace_procedure_heading_with_plpgsql() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let change = fix_header(&root);
        assert!(change.is_ok());

        let change = change.unwrap();
        check(
            change.replacement,
            expect![[r#"
                CREATE PROCEDURE secure_dml()
                AS $$
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

        assert_eq!(
            change.hints,
            vec![
                RuleHint::new(5..5, "Add parameter parentheses"),
                RuleHint::new(1..2, "Replace procedure prologue"),
                RuleHint::new(7..9, "Replace procedure epilogue"),
            ]
        );
    }

    #[test]
    fn dont_add_second_pair_of_parentheses_for_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/empty_parameter_list.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let change = fix_header(&root);
        assert!(change.is_ok());

        let change = change.unwrap();
        check(
            change.replacement,
            expect![[r#"
                CREATE PROCEDURE example()
                AS $$
                BEGIN
                    NULL;
                END;
                $$ LANGUAGE plpgsql;
            "#]],
        );

        assert_eq!(
            change.hints,
            vec![
                RuleHint::new(1..2, "Replace procedure prologue"),
                RuleHint::new(7..9, "Replace procedure epilogue"),
            ]
        );
    }

    #[test]
    fn accept_either_is_or_as_in_procedure_prologue() {
        const INPUT: &str = include_str!("../../tests/procedure/heading/procedure_as.ora.sql");

        let parse = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(parse.syntax()).unwrap();
        let change = fix_header(&root);
        assert!(change.is_ok());

        let change = change.unwrap();
        check(
            change.replacement,
            expect![[r#"
                -- test: use of AS instead of IS
                CREATE OR REPLACE PROCEDURE procedure_as()
                AS $$
                BEGIN
                    NULL;
                END;
                $$ LANGUAGE plpgsql;
            "#]],
        );

        assert_eq!(
            change.hints,
            vec![
                RuleHint::new(11..11, "Add parameter parentheses"),
                RuleHint::new(1..2, "Replace procedure prologue"),
                RuleHint::new(7..9, "Replace procedure epilogue"),
            ]
        );
    }
}
