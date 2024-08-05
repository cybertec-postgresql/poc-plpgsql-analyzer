use crate::{safe_loop, Parser};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{parse_datatype, parse_expr, parse_ident, parse_query};

pub fn parse_cursor(p: &mut Parser) {
    p.start(SyntaxKind::CursorStmt);
    p.expect(T![cursor]);
    parse_ident(p, 0..1);
    if p.at(T!["("]) {
        parse_cursor_param_declarations(p);
    }
    if p.eat(T![return]) {
        parse_rowtype_clause(p);
    }
    if p.eat(T![is]) {
        parse_query(p, false);
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_cursor_param_declarations(p: &mut Parser) {
    p.start(SyntaxKind::CursorParameterDeclarations);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_cursor_param_declaration(p);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

fn parse_cursor_param_declaration(p: &mut Parser) {
    p.start(SyntaxKind::CursorParameterDeclaration);
    parse_ident(p, 1..1);
    p.eat(T![in]);
    parse_datatype(p);
    if [T![:=], T![default]].contains(&p.current()) {
        p.eat(T![:=]);
        p.eat(T![default]);
        parse_expr(p);
    }
    p.finish();
}

fn parse_rowtype_clause(p: &mut Parser) {
    p.start(SyntaxKind::RowtypeClause);
    parse_ident(p, 1..2);
    if p.eat(T![%]) {
        if !p.eat(T![rowtype]) {
            parse_datatype(p);
        }
    }

    p.finish();
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_simple_cursor() {}
}
