use super::parse_where_clause;
use crate::parser::Parser;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

#[allow(unused)]
pub(crate) fn parse_dml(p: &mut Parser) {
    p.start(SyntaxKind::DeleteStmt);
    p.expect(T![delete]);
    parse_column_expr(p);
    p.expect(T![from]);
    p.expect_one_of(&[T![quoted_ident], T![unquoted_ident]]);
    parse_where_clause(p);

    p.eat(T![;]);
    p.finish();
}

#[allow(unused)]
fn parse_column_expr(p: &mut Parser) {
    p.start(SyntaxKind::DeleteClause);
    // Leaving this possibility for pl/sql "hint"
    p.finish();
}
