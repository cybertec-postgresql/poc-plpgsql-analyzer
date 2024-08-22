use crate::{
    grammar::{parse_execute_immediate, parse_expr, parse_query, parse_stmt},
    safe_loop, Parser,
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{opt_expr, parse_ident};

pub(crate) fn parse_loop(p: &mut Parser) {
    p.start(SyntaxKind::Loop);
    p.eat(T![loop_label]);

    match p.current() {
        T![loop] => parse_basic_loop(p),
        T![for] => parse_for_loop(p),
        T![while] => parse_while_loop(p),
        _ => (),
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_basic_loop(p: &mut Parser) {
    p.start(SyntaxKind::BasicLoop);
    p.expect(T![loop]);
    safe_loop!(p, {
        parse_stmt(p);

        if p.at(T![end]) {
            break;
        }
    });
    p.expect(T![end]);
    p.expect(T![loop]);
    p.eat_one_of(&[T![unquoted_ident], T![quoted_ident]]);
    p.finish();
}

fn parse_for_loop(p: &mut Parser) {
    p.start(SyntaxKind::ForLoop);
    p.expect(T![for]);
    parse_iterator(p);
    p.expect(T![loop]);
    safe_loop!(p, {
        parse_stmt(p);

        if p.at(T![end]) {
            break;
        }
    });
    p.expect(T![end]);
    p.expect(T![loop]);
    p.eat_one_of(&[T![unquoted_ident], T![quoted_ident]]);
    p.finish();
}

fn parse_while_loop(p: &mut Parser) {
    p.start(SyntaxKind::WhileLoop);
    p.expect(T![while]);
    parse_expr(p);
    p.expect(T![loop]);
    safe_loop!(p, {
        parse_stmt(p);

        if p.at(T![end]) {
            break;
        }
    });
    p.expect(T![end]);
    p.expect(T![loop]);
    p.eat_one_of(&[T![unquoted_ident], T![quoted_ident]]);
    p.finish();
}

fn parse_iterator(p: &mut Parser) {
    p.start(SyntaxKind::Iterator);
    parse_iterand_decl(p);
    if p.eat(T![,]) {
        parse_iterand_decl(p);
    }
    p.expect(T![in]);
    parse_iteration_ctl_seq(p);
    p.finish();
}

fn parse_iterand_decl(p: &mut Parser) {
    parse_ident(p, 1..1);
    p.eat_one_of(&[T![mutable], T![immutable]]);
}

fn parse_iteration_ctl_seq(p: &mut Parser) {
    safe_loop!(p, {
        parse_qual_iteration_ctl(p);
        if !p.eat(T![,]) {
            break;
        }
    });
}

fn parse_qual_iteration_ctl(p: &mut Parser) {
    p.eat(T![reverse]);
    parse_iteration_control(p);
    parse_pred_clause_seq(p);
}

fn parse_iteration_control(p: &mut Parser) {
    p.start(SyntaxKind::IterationControl);
    match p.current() {
        T![values] | T![indices] | T![pairs] => {
            p.expect_one_of(&[T![values], T![indices], T![pairs]]);
            p.expect(T![of]);
            if p.eat(T!["("]) {
                if [T![with], T![select]].contains(&p.current()) {
                    parse_query(p, false);
                } else if p.at(T![execute]) {
                    parse_execute_immediate(p);
                } else {
                    parse_ident(p, 1..2);
                }
                p.expect(T![")"]);
            } else if opt_expr(p) {
            } else {
                parse_ident(p, 1..2);
            }
        }
        _ => {
            if p.eat(T![repeat]) {
                parse_expr(p);
            } else if p.eat(T!["("]) {
                if [T![with], T![select]].contains(&p.current()) {
                    parse_query(p, false);
                } else if p.at(T![ref]) {
                    p.eat(T![ref]);
                    parse_ident(p, 1..2);
                } else if p.at(T![execute]) {
                    parse_execute_immediate(p);
                } else {
                    parse_ident(p, 1..2);
                }
                p.expect(T![")"]);
            } else {
                parse_expr(p);
                if p.eat(T![..]) {
                    parse_expr(p);
                }
                if p.eat(T![by]) {
                    parse_expr(p);
                }
            }
        }
    }

    p.finish();
}

fn parse_pred_clause_seq(p: &mut Parser) {
    if p.eat_one_of(&[T![while], T![when]]) {
        parse_expr(p);
    }
}

pub(crate) fn parse_exit_stmt(p: &mut Parser) {
    p.start(SyntaxKind::ExitStmt);
    p.expect(T![exit]);
    p.eat_one_of(&[T![loop_label], T![unquoted_ident]]);
    if p.eat(T![when]) {
        parse_expr(p);
    }
    p.eat(T![;]);

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::parse_loop;

    #[test]
    fn test_parse_simple_loop() {
        check(
            parse(
                "LOOP
    l_counter := l_counter + 1;
    IF l_counter > 3 THEN
      EXIT;
    END IF;
  END LOOP;",
                parse_loop,
            ),
            expect![[r#"
Root@0..98
  Loop@0..98
    BasicLoop@0..97
      Keyword@0..4 "LOOP"
      Whitespace@4..9 "\n    "
      BlockStatement@9..36
        IdentGroup@9..18
          Ident@9..18 "l_counter"
        Whitespace@18..19 " "
        Assign@19..21 ":="
        Whitespace@21..22 " "
        Expression@22..35
          IdentGroup@22..31
            Ident@22..31 "l_counter"
          Whitespace@31..32 " "
          ArithmeticOp@32..33 "+"
          Whitespace@33..34 " "
          Integer@34..35 "1"
        Semicolon@35..36 ";"
      Whitespace@36..41 "\n    "
      BlockStatement@41..86
        Keyword@41..43 "IF"
        Whitespace@43..44 " "
        Expression@44..58
          IdentGroup@44..53
            Ident@44..53 "l_counter"
          Whitespace@53..54 " "
          ComparisonOp@54..55 ">"
          Whitespace@55..56 " "
          Integer@56..57 "3"
          Whitespace@57..58 " "
        Keyword@58..62 "THEN"
        Whitespace@62..69 "\n      "
        BlockStatement@69..79
          ExitStmt@69..74
            Keyword@69..73 "EXIT"
            Semicolon@73..74 ";"
          Whitespace@74..79 "\n    "
        Keyword@79..82 "END"
        Whitespace@82..83 " "
        Keyword@83..85 "IF"
        Semicolon@85..86 ";"
      Whitespace@86..89 "\n  "
      Keyword@89..92 "END"
      Whitespace@92..93 " "
      Keyword@93..97 "LOOP"
    Semicolon@97..98 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_nested_loop() {
        check(
            parse(
                "<<outer_loop>>
  LOOP
    l_i := l_i + 1;
    EXIT outer_loop WHEN l_i > 2;    
    dbms_output.put_line('Outer counter ' || l_i);
    -- reset inner counter
    l_j := 0;
      <<inner_loop>> LOOP
      l_j := l_j + 1;
      EXIT inner_loop WHEN l_j > 3;
      dbms_output.put_line(' Inner counter ' || l_j);
    END LOOP inner_loop;
  END LOOP outer_loop;",
                parse_loop,
            ),
            expect![[r#"
Root@0..357
  Loop@0..357
    Ident@0..14 "<<outer_loop>>"
    Whitespace@14..17 "\n  "
    BasicLoop@17..356
      Keyword@17..21 "LOOP"
      Whitespace@21..26 "\n    "
      BlockStatement@26..41
        IdentGroup@26..29
          Ident@26..29 "l_i"
        Whitespace@29..30 " "
        Assign@30..32 ":="
        Whitespace@32..33 " "
        Expression@33..40
          IdentGroup@33..36
            Ident@33..36 "l_i"
          Whitespace@36..37 " "
          ArithmeticOp@37..38 "+"
          Whitespace@38..39 " "
          Integer@39..40 "1"
        Semicolon@40..41 ";"
      Whitespace@41..46 "\n    "
      BlockStatement@46..84
        ExitStmt@46..75
          Keyword@46..50 "EXIT"
          Whitespace@50..51 " "
          Ident@51..61 "outer_loop"
          Whitespace@61..62 " "
          Keyword@62..66 "WHEN"
          Whitespace@66..67 " "
          Expression@67..74
            IdentGroup@67..70
              Ident@67..70 "l_i"
            Whitespace@70..71 " "
            ComparisonOp@71..72 ">"
            Whitespace@72..73 " "
            Integer@73..74 "2"
          Semicolon@74..75 ";"
        Whitespace@75..84 "    \n    "
      BlockStatement@84..130
        FunctionInvocation@84..129
          IdentGroup@84..104
            Ident@84..95 "dbms_output"
            Dot@95..96 "."
            Ident@96..104 "put_line"
          LParen@104..105 "("
          ArgumentList@105..128
            Argument@105..128
              Expression@105..128
                QuotedLiteral@105..121 "'Outer counter '"
                Whitespace@121..122 " "
                Concat@122..124 "||"
                Whitespace@124..125 " "
                IdentGroup@125..128
                  Ident@125..128 "l_i"
          RParen@128..129 ")"
        Semicolon@129..130 ";"
      Whitespace@130..135 "\n    "
      Comment@135..157 "-- reset inner counter"
      Whitespace@157..162 "\n    "
      BlockStatement@162..171
        IdentGroup@162..165
          Ident@162..165 "l_j"
        Whitespace@165..166 " "
        Assign@166..168 ":="
        Whitespace@168..169 " "
        Expression@169..170
          Integer@169..170 "0"
        Semicolon@170..171 ";"
      Whitespace@171..178 "\n      "
      BlockStatement@178..337
        Loop@178..334
          Ident@178..192 "<<inner_loop>>"
          Whitespace@192..193 " "
          BasicLoop@193..333
            Keyword@193..197 "LOOP"
            Whitespace@197..204 "\n      "
            BlockStatement@204..219
              IdentGroup@204..207
                Ident@204..207 "l_j"
              Whitespace@207..208 " "
              Assign@208..210 ":="
              Whitespace@210..211 " "
              Expression@211..218
                IdentGroup@211..214
                  Ident@211..214 "l_j"
                Whitespace@214..215 " "
                ArithmeticOp@215..216 "+"
                Whitespace@216..217 " "
                Integer@217..218 "1"
              Semicolon@218..219 ";"
            Whitespace@219..226 "\n      "
            BlockStatement@226..262
              ExitStmt@226..255
                Keyword@226..230 "EXIT"
                Whitespace@230..231 " "
                Ident@231..241 "inner_loop"
                Whitespace@241..242 " "
                Keyword@242..246 "WHEN"
                Whitespace@246..247 " "
                Expression@247..254
                  IdentGroup@247..250
                    Ident@247..250 "l_j"
                  Whitespace@250..251 " "
                  ComparisonOp@251..252 ">"
                  Whitespace@252..253 " "
                  Integer@253..254 "3"
                Semicolon@254..255 ";"
              Whitespace@255..262 "\n      "
            BlockStatement@262..309
              FunctionInvocation@262..308
                IdentGroup@262..282
                  Ident@262..273 "dbms_output"
                  Dot@273..274 "."
                  Ident@274..282 "put_line"
                LParen@282..283 "("
                ArgumentList@283..307
                  Argument@283..307
                    Expression@283..307
                      QuotedLiteral@283..300 "' Inner counter '"
                      Whitespace@300..301 " "
                      Concat@301..303 "||"
                      Whitespace@303..304 " "
                      IdentGroup@304..307
                        Ident@304..307 "l_j"
                RParen@307..308 ")"
              Semicolon@308..309 ";"
            Whitespace@309..314 "\n    "
            Keyword@314..317 "END"
            Whitespace@317..318 " "
            Keyword@318..322 "LOOP"
            Whitespace@322..323 " "
            Ident@323..333 "inner_loop"
          Semicolon@333..334 ";"
        Whitespace@334..337 "\n  "
      Keyword@337..340 "END"
      Whitespace@340..341 " "
      Keyword@341..345 "LOOP"
      Whitespace@345..346 " "
      Ident@346..356 "outer_loop"
    Semicolon@356..357 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_for_loop() {
        check(
            parse(
                "FOR l_counter IN 1..5
  LOOP
    DBMS_OUTPUT.PUT_LINE( l_counter );
  END LOOP;
",
                parse_loop,
            ),
            expect![[r#"
Root@0..80
  Loop@0..79
    ForLoop@0..78
      Keyword@0..3 "FOR"
      Whitespace@3..4 " "
      Iterator@4..24
        IdentGroup@4..13
          Ident@4..13 "l_counter"
        Whitespace@13..14 " "
        Keyword@14..16 "IN"
        Whitespace@16..17 " "
        IterationControl@17..24
          IterRange@17..21 "1..5"
          Whitespace@21..24 "\n  "
      Keyword@24..28 "LOOP"
      Whitespace@28..33 "\n    "
      BlockStatement@33..67
        FunctionInvocation@33..66
          IdentGroup@33..53
            Ident@33..44 "DBMS_OUTPUT"
            Dot@44..45 "."
            Ident@45..53 "PUT_LINE"
          LParen@53..54 "("
          Whitespace@54..55 " "
          ArgumentList@55..65
            Argument@55..65
              IdentGroup@55..64
                Ident@55..64 "l_counter"
              Whitespace@64..65 " "
          RParen@65..66 ")"
        Semicolon@66..67 ";"
      Whitespace@67..70 "\n  "
      Keyword@70..73 "END"
      Whitespace@73..74 " "
      Keyword@74..78 "LOOP"
    Semicolon@78..79 ";"
  Whitespace@79..80 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_for_step_by() {
        check(
            parse(
                "FOR l_conuter IN 1..10 BY 2
    LOOP
        DBMS_OUTPUT.PUT_LINE( l_counter );
    END LOOP;",
                parse_loop,
            ),
            expect![[r#"
Root@0..93
  Loop@0..93
    ForLoop@0..92
      Keyword@0..3 "FOR"
      Whitespace@3..4 " "
      Iterator@4..32
        IdentGroup@4..13
          Ident@4..13 "l_conuter"
        Whitespace@13..14 " "
        Keyword@14..16 "IN"
        Whitespace@16..17 " "
        IterationControl@17..32
          IterRange@17..22 "1..10"
          Whitespace@22..23 " "
          Keyword@23..25 "BY"
          Whitespace@25..26 " "
          Integer@26..27 "2"
          Whitespace@27..32 "\n    "
      Keyword@32..36 "LOOP"
      Whitespace@36..45 "\n        "
      BlockStatement@45..79
        FunctionInvocation@45..78
          IdentGroup@45..65
            Ident@45..56 "DBMS_OUTPUT"
            Dot@56..57 "."
            Ident@57..65 "PUT_LINE"
          LParen@65..66 "("
          Whitespace@66..67 " "
          ArgumentList@67..77
            Argument@67..77
              IdentGroup@67..76
                Ident@67..76 "l_counter"
              Whitespace@76..77 " "
          RParen@77..78 ")"
        Semicolon@78..79 ";"
      Whitespace@79..84 "\n    "
      Keyword@84..87 "END"
      Whitespace@87..88 " "
      Keyword@88..92 "LOOP"
    Semicolon@92..93 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_for_loop_reverse() {
        check(
            parse(
                "FOR l_counter IN REVERSE 1..3
  LOOP
    DBMS_OUTPUT.PUT_LINE( l_counter );
  END LOOP;",
                parse_loop,
            ),
            expect![[r#"
Root@0..87
  Loop@0..87
    ForLoop@0..86
      Keyword@0..3 "FOR"
      Whitespace@3..4 " "
      Iterator@4..32
        IdentGroup@4..13
          Ident@4..13 "l_counter"
        Whitespace@13..14 " "
        Keyword@14..16 "IN"
        Whitespace@16..17 " "
        Keyword@17..24 "REVERSE"
        Whitespace@24..25 " "
        IterationControl@25..32
          IterRange@25..29 "1..3"
          Whitespace@29..32 "\n  "
      Keyword@32..36 "LOOP"
      Whitespace@36..41 "\n    "
      BlockStatement@41..75
        FunctionInvocation@41..74
          IdentGroup@41..61
            Ident@41..52 "DBMS_OUTPUT"
            Dot@52..53 "."
            Ident@53..61 "PUT_LINE"
          LParen@61..62 "("
          Whitespace@62..63 " "
          ArgumentList@63..73
            Argument@63..73
              IdentGroup@63..72
                Ident@63..72 "l_counter"
              Whitespace@72..73 " "
          RParen@73..74 ")"
        Semicolon@74..75 ";"
      Whitespace@75..78 "\n  "
      Keyword@78..81 "END"
      Whitespace@81..82 " "
      Keyword@82..86 "LOOP"
    Semicolon@86..87 ";"
"#]],
            vec![],
        );
    }
}
