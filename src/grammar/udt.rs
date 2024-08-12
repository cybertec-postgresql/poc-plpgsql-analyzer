use crate::Parser;
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

#[allow(unused)]
pub(crate) fn parse_create_type(p: &mut Parser) {
    p.start(SyntaxKind::UdtDefinitionStmt);
    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }
    p.eat_one_of(&[T![editionable], T![noneditionable]]);
    p.expect(T![type]);
    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }
    parse_plsql_type_source(p);
    p.eat(T![;]);
    p.finish();
}
