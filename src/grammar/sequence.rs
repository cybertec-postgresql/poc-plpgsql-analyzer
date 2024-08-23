use crate::{safe_loop, Parser};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::parse_ident;

#[allow(unused)]
pub(crate) fn parse_sequence(p: &mut Parser) {
    p.start(SyntaxKind::SequenceStmt);
    p.expect(T![create]);
    p.expect(T![sequence]);
    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }
    parse_ident(p, 1..2);
    if p.eat(T![sharing]) {
        p.expect(T![=]);
        p.expect_one_of(&[T![metadata], T![data], T![none]]);
    }
    parse_sequence_params(p);
    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_sequence_params(p: &mut Parser) {
    p.start(SyntaxKind::SequenceParameters);
    safe_loop!(p, {
        match p.current() {
            T![increment] => {
                p.expect(T![increment]);
                p.expect(T![by]);
                p.expect(T![int_literal]);
            }
            T![start] => {
                p.expect(T![start]);
                p.expect(T![with]);
                p.expect(T![int_literal]);
            }
            T![maxvalue] | T![minvalue] | T![cache] => {
                p.bump_any();
                p.expect(T![int_literal]);
            }
            T![nomaxvalue]
            | T![nominvalue]
            | T![cycle]
            | T![nocycle]
            | T![nocache]
            | T![order]
            | T![noorder]
            | T![keep]
            | T![nokeep]
            | T![noscale]
            | T![noshard]
            | T![session]
            | T![global] => {
                p.bump_any();
            }
            T![scale] | T![shard] => {
                p.bump_any();
                p.expect_one_of(&[T![extend], T![noextend]]);
            }

            _ => {
                break;
            }
        }
    });

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::*;

    #[test]
    fn test_parse_sequence() {
        check(
            parse(
                "CREATE SEQUENCE customers_seq
 START WITH     1000
 INCREMENT BY   1
 NOCACHE
 NOCYCLE;",
                parse_sequence,
            ),
            expect![[r#"
Root@0..87
  SequenceStmt@0..87
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..15 "SEQUENCE"
    Whitespace@15..16 " "
    IdentGroup@16..29
      Ident@16..29 "customers_seq"
    Whitespace@29..31 "\n "
    SequenceParameters@31..86
      Keyword@31..36 "START"
      Whitespace@36..37 " "
      Keyword@37..41 "WITH"
      Whitespace@41..46 "     "
      Integer@46..50 "1000"
      Whitespace@50..52 "\n "
      Keyword@52..61 "INCREMENT"
      Whitespace@61..62 " "
      Keyword@62..64 "BY"
      Whitespace@64..67 "   "
      Integer@67..68 "1"
      Whitespace@68..70 "\n "
      Keyword@70..77 "NOCACHE"
      Whitespace@77..79 "\n "
      Keyword@79..86 "NOCYCLE"
    Semicolon@86..87 ";"
"#]],
            vec![],
        );
    }
}
