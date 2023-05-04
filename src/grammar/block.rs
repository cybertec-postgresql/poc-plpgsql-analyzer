// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of SQL blocks from a token tree.

use crate::grammar::{
    opt_expr, opt_function_invocation, opt_insert, opt_query, parse_datatype, parse_expr,
    parse_ident,
};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

/// Looks ahead and parses a block if applicable
pub(crate) fn opt_block(p: &mut Parser) -> bool {
    if p.at(TokenKind::DeclareKw) || p.at(TokenKind::BeginKw) {
        parse_block(p);
        return true;
    }
    false
}

/// Parses a complete block.
pub fn parse_block(p: &mut Parser) {
    p.start(SyntaxKind::Block);
    parse_declare_section(p);

    p.expect(TokenKind::BeginKw);

    while !p.at(TokenKind::EndKw) {
        parse_stmt(p);
    }

    p.expect(TokenKind::EndKw);
    parse_ident(p, 0..1);
    p.expect(TokenKind::SemiColon);

    p.finish();
}

/// <https://docs.oracle.com/cd/B28359_01/appdev.111/b28370/block.htm#CJAIABJJ>
fn parse_declare_section(p: &mut Parser) {
    if p.at(TokenKind::BeginKw) {
        return;
    }

    p.start(SyntaxKind::DeclareSection);
    p.eat(TokenKind::DeclareKw);
    while !p.at(TokenKind::BeginKw) && !p.at(TokenKind::Eof) {
        p.start(SyntaxKind::VariableDecl);

        if !p.expect_one_of(&[TokenKind::UnquotedIdent, TokenKind::QuotedIdent]) {
            break;
        }

        while !p.at(TokenKind::SemiColon) && !p.at(TokenKind::Eof) {
            parse_datatype(p);
        }

        p.finish();

        if !p.eat(TokenKind::SemiColon) {
            break;
        }
    }
    p.finish();
}

fn parse_stmt(p: &mut Parser) {
    p.start(SyntaxKind::BlockStatement);

    if !(opt_block(p)
        || opt_if_stmt(p)
        || opt_null_stmt(p)
        || opt_return_stmt(p)
        || opt_insert(p)
        || opt_query(p, true)
        || opt_procedure_call(p))
    {
        p.error(ParseError::ExpectedStatement);
        p.bump_any();
    }

    p.finish();
}

fn opt_if_stmt(p: &mut Parser) -> bool {
    if p.eat(TokenKind::IfKw) {
        parse_expr(p);
        p.expect(TokenKind::ThenKw);

        while ![TokenKind::ElsifKw, TokenKind::ElseKw, TokenKind::EndKw].contains(&p.current()) {
            parse_stmt(p);
        }

        while p.eat(TokenKind::ElsifKw) {
            parse_expr(p);
            p.expect(TokenKind::ThenKw);

            while ![TokenKind::ElsifKw, TokenKind::ElseKw, TokenKind::EndKw].contains(&p.current())
            {
                parse_stmt(p);
            }
        }

        if p.eat(TokenKind::ElseKw) {
            while !p.at(TokenKind::EndKw) {
                parse_stmt(p);
            }
        }

        p.expect(TokenKind::EndKw);
        p.expect(TokenKind::IfKw);
        p.expect(TokenKind::SemiColon);

        true
    } else {
        false
    }
}

fn opt_null_stmt(p: &mut Parser) -> bool {
    if p.eat(TokenKind::NullKw) {
        p.expect(TokenKind::SemiColon);
        true
    } else {
        false
    }
}

fn opt_procedure_call(p: &mut Parser) -> bool {
    if opt_function_invocation(p) {
        p.expect(TokenKind::SemiColon);
        true
    } else {
        false
    }
}

fn opt_return_stmt(p: &mut Parser) -> bool {
    if p.eat(TokenKind::ReturnKw) {
        opt_expr(p);
        p.expect(TokenKind::SemiColon);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

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
      SemiColon@10..11 ";"
    Whitespace@11..12 " "
    Keyword@12..15 "END"
    SemiColon@15..16 ";"
"#]],
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
    -- Return statement
    RETURN 1;
END log_last_login_fuzzy;"#,
                parse_block,
            ),
            expect![[r#"
Root@0..461
  Block@0..461
    Whitespace@0..1 "\n"
    DeclareSection@1..45
      Keyword@1..8 "DECLARE"
      Whitespace@8..13 "\n    "
      VariableDecl@13..43
        Ident@13..29 "formatted_output"
        Whitespace@29..30 " "
        Datatype@30..43
          Keyword@30..38 "VARCHAR2"
          LParen@38..39 "("
          Integer@39..42 "100"
          RParen@42..43 ")"
      SemiColon@43..44 ";"
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
        Ident@147..151 "DUAL"
        SemiColon@151..152 ";"
      Whitespace@152..157 "\n    "
      Comment@157..172 "-- Nested block"
      Whitespace@172..177 "\n    "
    BlockStatement@177..220
      Block@177..193
        Keyword@177..182 "BEGIN"
        Whitespace@182..183 " "
        BlockStatement@183..188
          Keyword@183..187 "NULL"
          SemiColon@187..188 ";"
        Whitespace@188..189 " "
        Keyword@189..192 "END"
        SemiColon@192..193 ";"
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
      SemiColon@258..259 ";"
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
        SemiColon@305..306 ";"
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
        SemiColon@335..336 ";"
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
        SemiColon@365..366 ";"
      Whitespace@366..375 "\n        "
      Keyword@375..379 "ELSE"
      Whitespace@379..380 " "
      BlockStatement@380..385
        Keyword@380..384 "NULL"
        SemiColon@384..385 ";"
      Whitespace@385..390 "\n    "
      Keyword@390..393 "END"
      Whitespace@393..394 " "
      Keyword@394..396 "IF"
      SemiColon@396..397 ";"
    Whitespace@397..402 "\n    "
    Comment@402..421 "-- Return statement"
    Whitespace@421..426 "\n    "
    BlockStatement@426..435
      Keyword@426..432 "RETURN"
      Expression@432..434
        Whitespace@432..433 " "
        Integer@433..434 "1"
      SemiColon@434..435 ";"
    Whitespace@435..436 "\n"
    Keyword@436..439 "END"
    Whitespace@439..440 " "
    IdentGroup@440..460
      Ident@440..460 "log_last_login_fuzzy"
    SemiColon@460..461 ";"
"#]],
        );
    }

    #[test]
    fn test_select_without_into_clause() {
        check(
            parse(r#"BEGIN SELECT 1 FROM dual; END ;"#, parse_block),
            expect![[r#"
Root@0..54
  Block@0..54
    Keyword@0..5 "BEGIN"
    Whitespace@5..6 " "
    BlockStatement@6..49
      SelectStmt@6..48
        Keyword@6..12 "SELECT"
        Whitespace@12..13 " "
        SelectClause@13..15
          ColumnExpr@13..15
            Integer@13..14 "1"
            Whitespace@14..15 " "
        Error@15..38
          Text@15..38 "Expected token 'IntoKw'"
        Keyword@38..42 "FROM"
        Whitespace@42..43 " "
        Ident@43..47 "dual"
        SemiColon@47..48 ";"
      Whitespace@48..49 " "
    Keyword@49..52 "END"
    Whitespace@52..53 " "
    SemiColon@53..54 ";"
"#]],
        );
    }

    #[test]
    fn test_declare_section() {
        const INPUT: &str = "
    l_credit_limit NUMBER (10,0);
    l_contact_name VARCHAR2(255);";

        check(
            parse(INPUT, parse_declare_section),
            expect![[r#"
Root@0..68
  Whitespace@0..5 "\n    "
  DeclareSection@5..68
    VariableDecl@5..33
      Ident@5..19 "l_credit_limit"
      Whitespace@19..20 " "
      Datatype@20..33
        Keyword@20..26 "NUMBER"
        Whitespace@26..27 " "
        LParen@27..28 "("
        Integer@28..30 "10"
        Comma@30..31 ","
        Integer@31..32 "0"
        RParen@32..33 ")"
    SemiColon@33..34 ";"
    Whitespace@34..39 "\n    "
    VariableDecl@39..67
      Ident@39..53 "l_contact_name"
      Whitespace@53..54 " "
      Datatype@54..67
        Keyword@54..62 "VARCHAR2"
        LParen@62..63 "("
        Integer@63..66 "255"
        RParen@66..67 ")"
    SemiColon@67..68 ";"
"#]],
        );
    }
}
