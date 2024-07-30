// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements grammar parsing of the token tree from the lexer.

use std::ops::Range;

pub(crate) use block::*;
pub(crate) use constraint::*;
pub(crate) use datatype::*;
pub(crate) use expressions::*;
pub(crate) use function::*;
pub(crate) use function_invocation::*;
pub(crate) use package::*;
pub(crate) use procedure::*;
pub(crate) use query::*;
pub(crate) use trigger::*;
pub(crate) use view::*;

use crate::parser::{safe_loop, Parser};
use crate::ParseErrorType;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

mod block;
mod call_spec;
mod constraint;
mod datatype;
mod declare_section;
mod expressions;
mod function;
mod function_invocation;
mod package;
mod procedure;
mod query;
mod trigger;
mod view;

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    if p.at(T!["("]) {
        p.start(SyntaxKind::ParamList);
        p.bump(T!["("]);

        safe_loop!(p, {
            match p.current() {
                T![,] => {
                    p.bump(T![,]);
                }
                T![")"] | T![EOF] => {
                    break;
                }
                _ => {
                    parse_param(p);
                }
            }
        });

        p.expect(T![")"]);
        p.finish();
    }
}

/// Parses a single parameter in a parameter list of a procedure.
/// Refer to https://docs.oracle.com/en/database/oracle/oracle-database/21/lnpls/formal-parameter-declaration.html#GUID-5BA8E033-96B9-439A-A4FC-4844FEC14AD8.
///
/// Example:
///   p2 VARCHAR2 := 'not empty'
fn parse_param(p: &mut Parser) {
    p.start(SyntaxKind::Param);
    parse_ident(p, 1..1);

    if !p.at(T![")"]) && !p.at(T![,]) && !p.at(T![EOF]) {
        p.eat(T![in]);

        if p.eat(T![out]) {
            p.eat(T![nocopy]);
            parse_datatype(p);
        } else {
            parse_datatype(p);
            if p.eat_one_of(&[T![:=], T![default]]) {
                parse_expr(p);
            }
        }
    }

    p.finish();
}

/// Parses a qualified SQL identifier.
///
/// # Arguments
///
/// * `p`: The parser struct
/// * `expected_components`: A range of the minimum and maximum expected components that should be present in the identifier.
///     To allow an optional identifier, pass a range starting with `0`.
///
/// returns: ()
///
/// # Examples
///
/// ```
/// // Matches [identifier].<identifier>
/// // parse_qualified_ident(p, 1..2);
/// ```
///
/// ```
/// // Matches <identifier>.<identifier>.<identifier>
/// // parse_qualified_ident(p, 3..3);
/// ```
fn parse_ident(p: &mut Parser, expected_components: Range<u8>) {
    assert!(expected_components.end > 0);
    assert!(expected_components.start <= expected_components.end);

    if expected_components.start == 0 && !p.current().is_ident() {
        return;
    }

    p.start(SyntaxKind::IdentGroup);

    parse_single_ident(p);

    let mut i: u8 = 1;
    while i < expected_components.end {
        if i < expected_components.start {
            p.expect(T![.]);
        } else if p.nth(0) != Some(T![.]) {
            break;
        }

        p.expect(T![.]);
        parse_single_ident(p);
        i += 1;
    }

    p.finish();
}

/// Helper function for [`parse_ident`]
fn parse_single_ident(p: &mut Parser) {
    if p.current().is_ident() {
        if !p.eat(T![bind_var]) {
            p.bump_any_map(SyntaxKind::Ident)
        }
    } else {
        p.error(ParseErrorType::ExpectedIdent)
    }
}

/// Parses a column list, e.g. `(col1, col2)`
fn parse_column_list(p: &mut Parser) {
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_ident(p, 1..1);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
}

fn parse_ident_or_function_invocation(p: &mut Parser) {
    if !opt_function_invocation(p) {
        parse_ident(p, 1..3);
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use crate::parser::{Parse, Parser};
    use crate::ParseError;

    use super::*;

    /// A helper to allow to call the different parse functions.
    pub fn parse<F>(input: &str, f: F) -> Parse
    where
        F: Fn(&mut Parser),
    {
        let mut parser = Parser::new(input);
        f(&mut parser);
        parser.build()
    }

    /// Helper function to compare the build syntax tree with the expected
    /// output.
    #[track_caller]
    pub fn check(parse: Parse, expected_tree: Expect, expected_errors: Vec<ParseError>) {
        expected_tree.assert_eq(&format!("{:#?}", parse.syntax()));
        assert_eq!(parse.errors, expected_errors);
    }

    #[test]
    fn test_parse_ident() {
        check(
            parse("hello", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..5
  IdentGroup@0..5
    Ident@0..5 "hello"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_bindvar() {
        check(
            parse(":old.employee_id", |p| parse_ident(p, 1..2)),
            expect![[r#"
Root@0..16
  IdentGroup@0..16
    BindVar@0..4 ":old"
    Dot@4..5 "."
    Ident@5..16 "employee_id"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_keyword_as_ident() {
        check(
            parse("procedure", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..9
  IdentGroup@0..9
    Ident@0..9 "procedure"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_ident_with_trivia() {
        check(
            parse(" -- hello\n  foo", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..15
  Whitespace@0..1 " "
  Comment@1..9 "-- hello"
  Whitespace@9..12 "\n  "
  IdentGroup@12..15
    Ident@12..15 "foo"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_param() {
        assert!(parse("p_1 VARCHAR2", parse_param).ok());
        assert!(parse("p_2 NUMBER", parse_param).ok());
        assert!(parse("p_3 IN BOOLEAN := FALSE", parse_param).ok());
        assert!(parse("p_4 IN OUT NOCOPY DATE", parse_param).ok());
        assert!(parse("p_5", parse_param).ok());

        check(
            parse("p_1 VARCHAR2", parse_param),
            expect![[r#"
Root@0..12
  Param@0..12
    IdentGroup@0..3
      Ident@0..3 "p_1"
    Whitespace@3..4 " "
    Datatype@4..12
      Keyword@4..12 "VARCHAR2"
"#]],
            vec![],
        );

        check(
            parse("  foo bar%type", parse_param),
            expect![[r#"
Root@0..14
  Whitespace@0..2 "  "
  Param@2..14
    IdentGroup@2..5
      Ident@2..5 "foo"
    Whitespace@5..6 " "
    Datatype@6..14
      IdentGroup@6..9
        Ident@6..9 "bar"
      TypeAttribute@9..14
        Percentage@9..10 "%"
        Keyword@10..14 "type"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_param_with_default_value() {
        check(
            parse("p2 VARCHAR2 := 'not empty'", parse_param),
            expect![[r#"
Root@0..26
  Param@0..26
    IdentGroup@0..2
      Ident@0..2 "p2"
    Whitespace@2..3 " "
    Datatype@3..12
      Keyword@3..11 "VARCHAR2"
      Whitespace@11..12 " "
    Assign@12..14 ":="
    Whitespace@14..15 " "
    Expression@15..26
      QuotedLiteral@15..26 "'not empty'"
"#]],
            vec![],
        );
    }
}
