// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Typed AST nodes for PL/SQL triggers.

use crate::ast::{AstNode, Block, IdentGroup};

use super::typed_syntax_node;

typed_syntax_node!(Trigger, TriggerHeader);

impl Trigger {
    /// Returns the name of the trigger.
    pub fn name(&self) -> Option<String> {
        self.header()?.identifier()?.name()
    }

    pub fn header(&self) -> Option<TriggerHeader> {
        self.syntax.children().find_map(TriggerHeader::cast)
    }

    /// Returns the body of the trigger.
    pub fn body(&self) -> Option<Block> {
        self.syntax.children().find_map(Block::cast)
    }
}

impl TriggerHeader {
    /// Returns the name of the trigger.
    pub fn identifier(&self) -> Option<IdentGroup> {
        self.syntax.children().find_map(IdentGroup::cast)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Root;

    use super::*;

    #[test]
    fn check_ast_node_to_trigger() {
        const INPUT: &str = include_str!("../../tests/trigger/after_trigger.ora.sql");
        let result = crate::parse_trigger(INPUT).unwrap();
        let root = Root::cast(result.syntax());
        assert!(root.is_some());

        let trigger = root.unwrap().trigger();
        assert!(trigger.is_some());
        assert_eq!(
            trigger.unwrap().name(),
            Some("store.after_trigger".to_string())
        );
    }
}
