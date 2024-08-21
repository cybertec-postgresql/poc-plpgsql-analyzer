use crate::{grammar::parse_ident, safe_loop, Parser};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{call_spec::parse_call_spec, parse_datatype};

#[allow(unused)]
pub(crate) fn parse_element_spec(p: &mut Parser) {
    p.start(SyntaxKind::ElementSpec);
    p.eat(T![not]);
    p.eat_one_of(&[T![overriding], T![final], T![instantiable]]);
    safe_loop!(p, {
        match p.current() {
            T![member] | T![static] => parse_subprogram_spec(p),
            T![final] | T![instantiable] | T![constructor] => parse_constructor_spec(p),
            T![map] | T![order] => parse_map_order_function_spec(p),
            _ => break,
        }

        if p.at(T![pragma]) {
            break;
        }
    });
    if p.at(T![pragma]) {
        parse_restrict_references_pragma(p);
    }
    p.finish();
}

fn parse_subprogram_spec(p: &mut Parser) {
    p.expect_one_of(&[T![member], T![static]]);
    match p.current() {
        T![procedure] => parse_procedure_spec(p),
        T![function] => parse_function_spec(p),
        _ => p.error(crate::ParseErrorType::ExpectedOneOfTokens(vec![
            T![function],
            T![procedure],
        ])),
    }
}

fn parse_constructor_spec(p: &mut Parser) {
    p.eat(T![final]);
    p.eat(T![instantiable]);
    p.expect(T![constructor]);
    p.expect(T![function]);
    parse_datatype(p);
    if p.eat(T!["("]) {
        if p.eat(T![self]) {
            p.expect(T![in]);
            p.expect(T![out]);
            parse_datatype(p);
            p.expect(T![,]);
        }
        safe_loop!(p, {
            parse_ident(p, 1..1);
            parse_datatype(p);
            if !p.at(T![,]) || p.at(T![")"]) {
                break;
            }
        });
        p.expect(T![")"]);
    }
    p.expect(T![return]);
    p.expect(T![self]);
    p.expect(T![as]);
    p.expect(T![result]);
    if p.eat_one_of(&[T![is], T![as]]) {
        parse_call_spec(p);
    }
}

fn parse_map_order_function_spec(p: &mut Parser) {
    p.expect_one_of(&[T![map], T![order]]);
    p.expect(T![member]);
    parse_function_spec(p);
}

fn parse_procedure_spec(p: &mut Parser) {
    p.start(SyntaxKind::ProcedureSpec);
    p.expect(T![procedure]);
    parse_ident(p, 1..1);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_ident(p, 1..1);
        parse_datatype(p);
        if p.at(T![")"]) || p.at(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    if p.eat_one_of(&[T![is], T![as]]) {
        parse_call_spec(p);
    }
    p.finish();
}

fn parse_function_spec(p: &mut Parser) {
    p.start(SyntaxKind::FunctionSpec);
    p.expect(T![function]);
    parse_ident(p, 1..1);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_ident(p, 1..1);
        parse_datatype(p);
        if p.at(T![")"]) || p.at(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.expect(T![return]);
    parse_datatype(p);
    if p.eat_one_of(&[T![is], T![as]]) {
        parse_call_spec(p);
    }
    p.finish();
}

fn parse_restrict_references_pragma(p: &mut Parser) {
    p.expect(T![pragma]);
    p.expect(T![restricted_references]);
    p.expect(T!["("]);
    parse_ident(p, 1..1);
    safe_loop!(p, {
        p.expect_one_of(&[T![rnds], T![rnps], T![wnds], T![wnps], T![trust]]);
        if !p.at(T![,]) || p.at(T![")"]) {
            break;
        }
    });
    p.expect(T![")"]);
}
