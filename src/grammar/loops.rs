use crate::{
    grammar::{parse_expr, parse_stmt},
    safe_loop, Parser,
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{opt_expr, parse_ident};

#[allow(unused)]
pub(crate) fn parse_loop(p: &mut Parser) {
    p.start(SyntaxKind::Loop);

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
                todo!("The rest of cursor_object, dynamic_sql, sql_statements");
                // p.expect(T![")"]);
            } else if opt_expr(p) {
            } else {
                parse_ident(p, 1..2);
            }
        }
        _ => {
            if p.eat(T![repeat]) {
                parse_expr(p);
            } else if p.eat(T!["("]) {
                todo!("The rest of cursor_iteration_control");
                // p.expect(T![")"]);
            } else {
                parse_expr(p);
                if p.eat(T![..]) {
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
        BlockStatement@69..74
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
}
