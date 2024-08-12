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
    p.expect(T![comment]);
    p.expect(T![quoted_literal]);
    if p.at(T![write]) {
        parse_write(p);
    }
}

fn parse_write(p: &mut Parser) {
    p.expect(T![write]);
    p.eat_one_of(&[T![wait], T![nowait]]);
    p.eat_one_of(&[T![wait], T![nowait]]);
}

#[cfg(test)]
mod tests {}
