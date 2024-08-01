use super::typed_syntax_node;
use crate::{ast::AstNode, WhereClause};

typed_syntax_node!(DeleteStmt, UpdateStmt, SetClause);

impl DeleteStmt {
    pub fn where_clause(&self) -> Option<WhereClause> {
        self.syntax.children().find_map(WhereClause::cast)
    }
}

impl UpdateStmt {
    pub fn where_clause(&self) -> Option<WhereClause> {
        self.syntax.children().find_map(WhereClause::cast)
    }

    pub fn set_clause(&self) -> Option<SetClause> {
        self.syntax.children().find_map(SetClause::cast)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{AstNode, Root};

    #[test]
    fn check_ast_node_to_delete_stmt() {
        const TEST_STRING: &str = r#"DELETE FROM emp WHERE emp_id = 69;"#;

        let result = crate::parse_dml(TEST_STRING).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let delete = root.unwrap().dml();
        assert!(delete.is_some());
        let delete = delete.unwrap();

        let where_clause = delete.where_clause();
        assert!(where_clause.is_some());
    }
}
