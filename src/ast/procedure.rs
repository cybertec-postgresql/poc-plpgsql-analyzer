// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Typed AST nodes for PL/SQL procedures.

use crate::{syntax::SyntaxToken, AstNode, AstToken, SyntaxKind, SyntaxNode};

/// Automatically generate `struct`s and implementation of the [`AstNode`] or [`AstToken`] trait for [`SyntaxKind`] variants.
macro_rules! typed_syntax {
    ($synty:ty [ $astty:ty ] { $( $name:ident ),+ $(,)? }) => {
        $(
            pub struct $name {
                pub(crate) syntax: $synty,
            }

            impl $astty for $name {
                fn can_cast(kind: SyntaxKind) -> bool {
                    kind == SyntaxKind::$name
                }

                fn cast(syntax: $synty) -> Option<Self> {
                    if Self::can_cast(syntax.kind()) {
                        Some(Self { syntax })
                    } else {
                        None
                    }
                }

                fn syntax(&self) -> &$synty {
                    &self.syntax
                }
            }
        )+
    };
}

typed_syntax!(SyntaxNode[AstNode] { Procedure, ProcedureHeader, ProcedureBody });
typed_syntax!(SyntaxToken[AstToken] { Ident });

impl Procedure {
    /// Returns the name of the procedure.
    #[allow(unused)]
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children()
            .find_map(ProcedureHeader::cast)?
            .name()
    }

    /// Returns the name of the procedure.
    pub fn body(&self) -> Option<ProcedureBody> {
        self.syntax.children().find_map(ProcedureBody::cast)
    }
}

impl ProcedureHeader {
    /// Returns the name of the procedure.
    #[allow(unused)]
    pub fn name(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(Ident::cast)
            .map(|ident| ident.name())
    }
}

impl Ident {
    /// Returns the identifier name itself.
    #[allow(unused)]
    pub fn name(&self) -> String {
        self.syntax.text().to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, AstNode, Root};

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
        let result = parse(INPUT).unwrap();
        let root = Root::cast(result.syntax()).expect("Failed to get root");
        assert!(root.procedure().is_some());
        let procedure = root.procedure().unwrap();
        assert_eq!(Some("multiple_parameters".to_string()), procedure.name());
    }
}
