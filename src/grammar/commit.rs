use crate::parser::Parser;
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

#[allow(unused)]
pub(crate) fn parse_commit(p: &mut Parser) {
    p.start(SyntaxKind::CommitStmt);
    p.expect(T![commit]);
    p.eat(T![work]);
    match p.current() {
        T![force] => parse_force(p),
        T![comment] => parse_comment(p),
        T![write] => parse_write(p),
        _ => (),
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_force(p: &mut Parser) {
    p.expect(T![force]);
    p.expect(T![quoted_literal]);
    if p.eat(T![,]) {
        p.expect(T![int_literal]);
    }
}

fn parse_comment(p: &mut Parser) {
    p.expect(T![comment_word]);
    p.expect(T![quoted_literal]);
    if p.at(T![write]) {
        parse_write(p);
    }
}

fn parse_write(p: &mut Parser) {
    p.expect(T![write]);
    p.eat_one_of(&[T![wait], T![nowait]]);
    p.eat_one_of(&[T![immediate], T![batch]]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::parse_commit;

    #[test]
    fn parse_simple_commit() {
        check(
            parse("COMMIT;", parse_commit),
            expect![[r#"
Root@0..7
  CommitStmt@0..7
    Keyword@0..6 "COMMIT"
    Semicolon@6..7 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_commit_work() {
        check(
            parse("COMMIT WORK;", parse_commit),
            expect![[r#"
Root@0..12
  CommitStmt@0..12
    Keyword@0..6 "COMMIT"
    Whitespace@6..7 " "
    Keyword@7..11 "WORK"
    Semicolon@11..12 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_force_commit() {
        check(
            parse("COMMIT FORCE 'i am pleb', 69;", parse_commit),
            expect![[r#"
Root@0..29
  CommitStmt@0..29
    Keyword@0..6 "COMMIT"
    Whitespace@6..7 " "
    Keyword@7..12 "FORCE"
    Whitespace@12..13 " "
    QuotedLiteral@13..24 "'i am pleb'"
    Comma@24..25 ","
    Whitespace@25..26 " "
    Integer@26..28 "69"
    Semicolon@28..29 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_commit_comment() {
        check(
            parse("COMMIT COMMENT 'i am pleb' WRITE WAIT BATCH;", parse_commit),
            expect![[r#"
Root@0..7
  CommitStmt@0..7
    Keyword@0..6 "COMMIT"
    Whitespace@6..7 " "
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_commit_write() {
        check(
            parse("COMMIT WRITE NOWAIT IMMEDIATE;", parse_commit),
            expect![[r#"
Root@0..30
  CommitStmt@0..30
    Keyword@0..6 "COMMIT"
    Whitespace@6..7 " "
    Keyword@7..12 "WRITE"
    Whitespace@12..13 " "
    Keyword@13..19 "NOWAIT"
    Whitespace@19..20 " "
    Keyword@20..29 "IMMEDIATE"
    Semicolon@29..30 ";"
"#]],
            vec![],
        );
    }
}
