use super::typed_syntax_node;
use crate::{ast::AstNode, WhereClause};

typed_syntax_node!(DeleteStmt, DeleteClause);

impl DeleteStmt {
    #[allow(unused)]
    pub fn delete_clause(&self) -> Option<DeleteClause> {
        self.syntax.children().find_map(DeleteClause::cast)
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        self.syntax.children().find_map(WhereClause::cast)
    }
}

#[cfg(test)]
mod tests {}
