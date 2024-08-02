// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of SQL blocks from a token tree.

use crate::grammar::declare_section::parse_declare_section;
use crate::grammar::{
    opt_expr, opt_function_invocation, parse_expr, parse_ident, parse_insert, parse_query,
};
use crate::parser::{safe_loop, Parser};
use crate::ParseErrorType;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

use super::{parse_dml, parse_into_clause};

/// Parses a complete block.
pub fn parse_block(p: &mut Parser) {
    p.start(SyntaxKind::Block);

    let checkpoint = p.checkpoint();
    if p.eat(T![declare]) || p.current() != T![begin] {
        parse_declare_section(p, Some(checkpoint));
    }

    p.expect(T![begin]);

    safe_loop!(p, {
        parse_stmt(p);
        if p.at(T![end]) {
            break;
        }
    });

    p.expect(T![end]);
    parse_ident(p, 0..1);
    p.expect(T![;]);

    p.finish();
}

pub(super) fn parse_stmt(p: &mut Parser) {
    p.start(SyntaxKind::BlockStatement);

    match p.current() {
        T![declare] | T![begin] => parse_block(p),
        T![execute] => parse_execute_immediate_stmt(p),
        T![if] => parse_if_stmt(p),
        T![insert] => parse_insert(p),
        T![null] => parse_null_stmt(p),
        T![return] => parse_return_stmt(p),
        T![select] => parse_query(p, true),
        T![delete] | T![update] => parse_dml(p),
        current_token => {
            if !(opt_assignment_stmt(p) || opt_procedure_call(p)) {
                p.error(ParseErrorType::ExpectedStatement(current_token));
                p.bump_any();
            }
        }
    }

    p.finish();
}

fn parse_execute_immediate_stmt(p: &mut Parser) {
    p.start(SyntaxKind::ExecuteImmediateStmt);
    p.expect(T![execute]);
    p.expect(T![immediate]);
    // Parse String
    if !p.eat(T![unquoted_ident]) {
        p.expect(T![quoted_literal]);
    }
    // handle INTO, USING and RETURN/RETURNING clauses
    if p.at(T![into]) {
        parse_into_clause(p, true);
    }
    if p.at(T![using]) {
        // parse using clause
        parse_using_clause(p);
    }
    if [T![return], T![returning]].contains(&p.current()) {
        // parse return into stuff
        parse_return_into_clause(p);
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_using_clause(p: &mut Parser) {
    p.start(SyntaxKind::UsingClause);
    p.expect(T![using]);
    safe_loop!(p, {
        p.eat(T![in]);
        p.eat(T![out]);
        p.expect(T![unquoted_ident]);
        if [T![return], T![returning], T![;]].contains(&p.current()) {
            break;
        }
        p.eat(T![,]);
    });
    p.finish();
}

fn parse_return_into_clause(p: &mut Parser) {
    p.start(SyntaxKind::ReturnIntoClause);
    p.expect_one_of(&[T![return], T![returning]]);
    parse_into_clause(p, false);
}

fn parse_if_stmt(p: &mut Parser) {
    p.expect(T![if]);
    parse_expr(p);
    p.expect(T![then]);

    safe_loop!(p, {
        parse_stmt(p);
        if [T![elsif], T![else], T![end]].contains(&p.current()) {
            break;
        }
    });

    safe_loop!(p, {
        if !p.eat(T![elsif]) {
            break;
        }

        parse_expr(p);
        p.expect(T![then]);

        safe_loop!(p, {
            parse_stmt(p);
            if [T![elsif], T![else], T![end]].contains(&p.current()) {
                break;
            }
        });
    });

    if p.eat(T![else]) {
        safe_loop!(p, {
            parse_stmt(p);
            if p.at(T![end]) {
                break;
            }
        });
    }

    p.expect(T![end]);
    p.expect(T![if]);
    p.expect(T![;]);
}

fn parse_null_stmt(p: &mut Parser) {
    p.expect(T![null]);
    p.expect(T![;]);
}

fn parse_return_stmt(p: &mut Parser) {
    p.expect(T![return]);
    opt_expr(p);
    p.expect(T![;]);
}

fn opt_procedure_call(p: &mut Parser) -> bool {
    if opt_function_invocation(p) {
        p.expect(T![;]);
        true
    } else {
        false
    }
}

fn opt_assignment_stmt(p: &mut Parser) -> bool {
    if (p.current().is_ident() && p.nth(1).unwrap_or(T![EOF]) == T![:=])
        || (p.current().is_ident()
            && p.nth(1).unwrap_or(T![EOF]) == T![.]
            && p.nth(2).unwrap_or(T![EOF]).is_ident()
            && p.nth(3).unwrap_or(T![EOF]) == T![:=])
        || (p.current().is_ident()
            && p.nth(1).unwrap_or(T![EOF]) == T!["("]
            && p.nth(2).unwrap_or(T![EOF]) == T![int_literal]
            && p.nth(3).unwrap_or(T![EOF]) == T![")"]
            && p.nth(4).unwrap_or(T![EOF]) == T![:=])
    {
        parse_ident(p, 1..2);
        if p.eat(T!["("]) {
            p.expect(T![int_literal]);
            p.expect(T![")"]);
        }
        p.expect(T![:=]);
        parse_expr(p);
        p.expect(T![;]);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::ParseError;
    use crate::ParseErrorType::{ExpectedStatement, ExpectedToken};
    use source_gen::lexer::TokenKind::{IntoKw, UnquotedIdent};

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_block_with_unexpected_token() {
        check(
            parse(r#"BEGIN ABC END;"#, parse_block),
            expect![[r#"
Root@0..14
  Block@0..14
    Keyword@0..5 "BEGIN"
    Whitespace@5..6 " "
    BlockStatement@6..9
      Ident@6..9 "ABC"
    Whitespace@9..10 " "
    Keyword@10..13 "END"
    Semicolon@13..14 ";"
"#]],
            vec![ParseError::new(ExpectedStatement(UnquotedIdent), 6..9)],
        );
    }

    #[test]
    fn test_block_with_null_stmt() {
        check(
            parse(r#"BEGIN NULL; END;"#, parse_block),
            expect![[r#"
Root@0..16
  Block@0..16
    Keyword@0..5 "BEGIN"
    Whitespace@5..6 " "
    BlockStatement@6..11
      Keyword@6..10 "NULL"
      Semicolon@10..11 ";"
    Whitespace@11..12 " "
    Keyword@12..15 "END"
    Semicolon@15..16 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_exhaustive_block() {
        check(
            parse(
                r#"
DECLARE
    formatted_output VARCHAR2(100);
BEGIN
    -- SELECT INTO
    SELECT 'name: ' || dummy || ', last login: '  INTO formatted_output FROM DUAL;
    -- Nested block
    BEGIN NULL; END;
    -- Procedure call
    DBMS_OUTPUT.PUT_LINE(formatted_output);
    -- IF statement
        IF TRUE THEN NULL;
        ELSIF TRUE THEN NULL;
        ELSIF TRUE THEN NULL;
        ELSE NULL;
    END IF;
    -- Assignment operation
    formatted_output := 'abc';
    -- Return statement
    RETURN 1;
END log_last_login_fuzzy;"#,
                parse_block,
            ),
            expect![[r#"
Root@0..520
  Whitespace@0..1 "\n"
  Block@1..520
    DeclareSection@1..45
      Keyword@1..8 "DECLARE"
      Whitespace@8..13 "\n    "
      IdentGroup@13..29
        Ident@13..29 "formatted_output"
      Whitespace@29..30 " "
      Datatype@30..43
        Keyword@30..38 "VARCHAR2"
        LParen@38..39 "("
        Integer@39..42 "100"
        RParen@42..43 ")"
      Semicolon@43..44 ";"
      Whitespace@44..45 "\n"
    Keyword@45..50 "BEGIN"
    Whitespace@50..55 "\n    "
    Comment@55..69 "-- SELECT INTO"
    Whitespace@69..74 "\n    "
    BlockStatement@74..177
      SelectStmt@74..152
        Keyword@74..80 "SELECT"
        Whitespace@80..81 " "
        SelectClause@81..120
          ColumnExpr@81..120
            Expression@81..120
              Expression@81..99
                QuotedLiteral@81..89 "'name: '"
                Whitespace@89..90 " "
                Concat@90..92 "||"
                Whitespace@92..93 " "
                IdentGroup@93..98
                  Ident@93..98 "dummy"
                Whitespace@98..99 " "
              Concat@99..101 "||"
              Whitespace@101..102 " "
              QuotedLiteral@102..118 "', last login: '"
              Whitespace@118..120 "  "
        IntoClause@120..142
          Keyword@120..124 "INTO"
          Whitespace@124..125 " "
          IdentGroup@125..141
            Ident@125..141 "formatted_output"
          Whitespace@141..142 " "
        Keyword@142..146 "FROM"
        Whitespace@146..147 " "
        IdentGroup@147..151
          Ident@147..151 "DUAL"
        Semicolon@151..152 ";"
      Whitespace@152..157 "\n    "
      Comment@157..172 "-- Nested block"
      Whitespace@172..177 "\n    "
    BlockStatement@177..220
      Block@177..193
        Keyword@177..182 "BEGIN"
        Whitespace@182..183 " "
        BlockStatement@183..188
          Keyword@183..187 "NULL"
          Semicolon@187..188 ";"
        Whitespace@188..189 " "
        Keyword@189..192 "END"
        Semicolon@192..193 ";"
      Whitespace@193..198 "\n    "
      Comment@198..215 "-- Procedure call"
      Whitespace@215..220 "\n    "
    BlockStatement@220..259
      FunctionInvocation@220..258
        IdentGroup@220..240
          Ident@220..231 "DBMS_OUTPUT"
          Dot@231..232 "."
          Ident@232..240 "PUT_LINE"
        LParen@240..241 "("
        ArgumentList@241..257
          Argument@241..257
            IdentGroup@241..257
              Ident@241..257 "formatted_output"
        RParen@257..258 ")"
      Semicolon@258..259 ";"
    Whitespace@259..264 "\n    "
    Comment@264..279 "-- IF statement"
    Whitespace@279..288 "\n        "
    BlockStatement@288..397
      Keyword@288..290 "IF"
      Whitespace@290..291 " "
      IdentGroup@291..295
        Ident@291..295 "TRUE"
      Whitespace@295..296 " "
      Keyword@296..300 "THEN"
      Whitespace@300..301 " "
      BlockStatement@301..306
        Keyword@301..305 "NULL"
        Semicolon@305..306 ";"
      Whitespace@306..315 "\n        "
      Keyword@315..320 "ELSIF"
      Whitespace@320..321 " "
      IdentGroup@321..325
        Ident@321..325 "TRUE"
      Whitespace@325..326 " "
      Keyword@326..330 "THEN"
      Whitespace@330..331 " "
      BlockStatement@331..336
        Keyword@331..335 "NULL"
        Semicolon@335..336 ";"
      Whitespace@336..345 "\n        "
      Keyword@345..350 "ELSIF"
      Whitespace@350..351 " "
      IdentGroup@351..355
        Ident@351..355 "TRUE"
      Whitespace@355..356 " "
      Keyword@356..360 "THEN"
      Whitespace@360..361 " "
      BlockStatement@361..366
        Keyword@361..365 "NULL"
        Semicolon@365..366 ";"
      Whitespace@366..375 "\n        "
      Keyword@375..379 "ELSE"
      Whitespace@379..380 " "
      BlockStatement@380..385
        Keyword@380..384 "NULL"
        Semicolon@384..385 ";"
      Whitespace@385..390 "\n    "
      Keyword@390..393 "END"
      Whitespace@393..394 " "
      Keyword@394..396 "IF"
      Semicolon@396..397 ";"
    Whitespace@397..402 "\n    "
    Comment@402..425 "-- Assignment operation"
    Whitespace@425..430 "\n    "
    BlockStatement@430..456
      IdentGroup@430..446
        Ident@430..446 "formatted_output"
      Whitespace@446..447 " "
      Assign@447..449 ":="
      Whitespace@449..450 " "
      Expression@450..455
        QuotedLiteral@450..455 "'abc'"
      Semicolon@455..456 ";"
    Whitespace@456..461 "\n    "
    Comment@461..480 "-- Return statement"
    Whitespace@480..485 "\n    "
    BlockStatement@485..494
      Keyword@485..491 "RETURN"
      Whitespace@491..492 " "
      Expression@492..493
        Integer@492..493 "1"
      Semicolon@493..494 ";"
    Whitespace@494..495 "\n"
    Keyword@495..498 "END"
    Whitespace@498..499 " "
    IdentGroup@499..519
      Ident@499..519 "log_last_login_fuzzy"
    Semicolon@519..520 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_select_without_into_clause() {
        check(
            parse(r#"BEGIN SELECT 1 FROM dual; END ;"#, parse_block),
            expect![[r#"
Root@0..31
  Block@0..31
    Keyword@0..5 "BEGIN"
    Whitespace@5..6 " "
    BlockStatement@6..26
      SelectStmt@6..25
        Keyword@6..12 "SELECT"
        Whitespace@12..13 " "
        SelectClause@13..15
          ColumnExpr@13..15
            Integer@13..14 "1"
            Whitespace@14..15 " "
        Keyword@15..19 "FROM"
        Whitespace@19..20 " "
        IdentGroup@20..24
          Ident@20..24 "dual"
        Semicolon@24..25 ";"
      Whitespace@25..26 " "
    Keyword@26..29 "END"
    Whitespace@29..30 " "
    Semicolon@30..31 ";"
"#]],
            vec![ParseError::new(ExpectedToken(IntoKw), 15..19)],
        );
    }

    #[test]
    fn test_assignment_operations() {
        check(
            parse(
                r#"BEGIN
                a := 1;
                a.b := 1;
                :a := 1;
                a(1) := 1;
            END;"#,
                parse_block,
            ),
            expect![[r#"
Root@0..124
  Block@0..124
    Keyword@0..5 "BEGIN"
    Whitespace@5..22 "\n                "
    BlockStatement@22..29
      IdentGroup@22..23
        Ident@22..23 "a"
      Whitespace@23..24 " "
      Assign@24..26 ":="
      Whitespace@26..27 " "
      Expression@27..28
        Integer@27..28 "1"
      Semicolon@28..29 ";"
    Whitespace@29..46 "\n                "
    BlockStatement@46..55
      IdentGroup@46..49
        Ident@46..47 "a"
        Dot@47..48 "."
        Ident@48..49 "b"
      Whitespace@49..50 " "
      Assign@50..52 ":="
      Whitespace@52..53 " "
      Expression@53..54
        Integer@53..54 "1"
      Semicolon@54..55 ";"
    Whitespace@55..72 "\n                "
    BlockStatement@72..80
      IdentGroup@72..74
        BindVar@72..74 ":a"
      Whitespace@74..75 " "
      Assign@75..77 ":="
      Whitespace@77..78 " "
      Expression@78..79
        Integer@78..79 "1"
      Semicolon@79..80 ";"
    Whitespace@80..97 "\n                "
    BlockStatement@97..107
      IdentGroup@97..98
        Ident@97..98 "a"
      LParen@98..99 "("
      Integer@99..100 "1"
      RParen@100..101 ")"
      Whitespace@101..102 " "
      Assign@102..104 ":="
      Whitespace@104..105 " "
      Expression@105..106
        Integer@105..106 "1"
      Semicolon@106..107 ";"
    Whitespace@107..120 "\n            "
    Keyword@120..123 "END"
    Semicolon@123..124 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_simple_execute_immediate() {
        check(
            parse(
                r#"BEGIN EXECUTE IMMEDIATE 'SELECT * FROM emp;'; END;"#,
                parse_block,
            ),
            expect![[r#"
Root@0..50
  Block@0..50
    Keyword@0..5 "BEGIN"
    Whitespace@5..6 " "
    BlockStatement@6..46
      ExecuteImmediateStmt@6..45
        Keyword@6..13 "EXECUTE"
        Whitespace@13..14 " "
        Keyword@14..23 "IMMEDIATE"
        Whitespace@23..24 " "
        QuotedLiteral@24..44 "'SELECT * FROM emp;'"
        Semicolon@44..45 ";"
      Whitespace@45..46 " "
    Keyword@46..49 "END"
    Semicolon@49..50 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_complex_execute_immediate() {
        check(
            parse(
                r#"BEGIN
   EXECUTE IMMEDIATE sql_stmt USING emp_id RETURNING INTO salary;
END;"#,
                parse_block,
            ),
            expect![[r#"
Block@0..76
  Keyword@0..5 "BEGIN"
  Whitespace@5..9 "\n   "
  BlockStatement@9..76
    ExecuteImmediateStmt@9..72
      Keyword@9..16 "EXECUTE"
      Whitespace@16..17 " "
      Keyword@17..26 "IMMEDIATE"
      Whitespace@26..27 " "
      Ident@27..35 "sql_stmt"
      Whitespace@35..36 " "
      UsingClause@36..49
        Keyword@36..41 "USING"
        Whitespace@41..42 " "
        Ident@42..48 "emp_id"
        Whitespace@48..49 " "
      ReturnIntoClause@49..71
        Keyword@49..58 "RETURNING"
        Whitespace@58..59 " "
        IntoClause@59..70
          Keyword@59..63 "INTO"
          Whitespace@63..64 " "
          IdentGroup@64..70
            Ident@64..70 "salary"
        Semicolon@70..71 ";"
      Whitespace@71..72 "\n"
    Keyword@72..75 "END"
    Semicolon@75..76 ";"
"#]],
            vec![],
        );
    }
}
