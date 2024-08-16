use crate::{
    grammar::{parse_expr, parse_stmt},
    safe_loop, Parser,
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::parse_ident;

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
