use crate::Parser;
use source_gen::syntax::SyntaxKind;

pub fn parse_raise_stmt(p: &mut Parser) {
    p.start(SyntaxKind::RaiseStmt);
    p.finish();
}
