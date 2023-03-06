// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Typed AST nodes for PL/SQL procedures.

use crate::ast::{AstNode, IdentGroup, ParamList};

use super::typed_syntax_node;

typed_syntax_node!(Procedure, ProcedureHeader, ProcedureBody);

impl Procedure {
    /// Returns the name of the procedure.
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children()
            .find_map(ProcedureHeader::cast)?
            .name()
    }

    pub fn header(&self) -> Option<ProcedureHeader> {
        self.syntax.children().find_map(ProcedureHeader::cast)
    }

    /// Returns the name of the procedure.
    pub fn body(&self) -> Option<ProcedureBody> {
        self.syntax.children().find_map(ProcedureBody::cast)
    }
}

impl ProcedureHeader {
    /// Returns the name of the procedure.
    pub fn name(&self) -> Option<String> {
        self.syntax.children().find_map(IdentGroup::cast)?.name()
    }

    pub fn param_list(&self) -> Option<ParamList> {
        self.syntax.children().find_map(ParamList::cast)
    }
}

impl ProcedureBody {
    pub fn text(&self) -> String {
        self.syntax.text().to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Root;

    use super::*;

    #[test]
    fn check_ast_node_to_procedure() {
        const INPUT: &str = r#"
            CREATE OR REPLACE PROCEDURE multiple_parameters(
                p1 VARCHAR2
                , p2 VARCHAR2
            )
            IS
            BEGIN
                NULL;
            END multiple_parameters;
        "#;
        let result = crate::parse_procedure(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let procedure = root.unwrap().procedure();
        assert!(procedure.is_some());
        assert_eq!(
            procedure.unwrap().name(),
            Some("multiple_parameters".to_string())
        );
    }
}
