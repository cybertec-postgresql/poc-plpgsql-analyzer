// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements a typed AST node for SQL expressions.

use std::str::FromStr;

use inner_source_gen::syntax::{SyntaxNode, SyntaxToken};

use super::typed_syntax_node;

typed_syntax_node!(Expression);

#[derive(Debug, Eq, PartialEq)]
pub enum ComparisonOpType {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Like,
    ILike,
}

impl Expression {
    pub fn filter_tokens<F>(&self, filter: F) -> impl Iterator<Item = SyntaxToken>
    where
        F: Fn(&SyntaxToken) -> bool,
    {
        self.syntax
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .filter(filter)
    }

    #[allow(unused)]
    pub fn filter_nodes<F>(&self, filter: F) -> impl Iterator<Item = SyntaxNode>
    where
        F: Fn(&SyntaxNode) -> bool,
    {
        self.syntax.children().filter(filter)
    }
}

impl FromStr for ComparisonOpType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "=" => Ok(Self::Equal),
            "<>" => Ok(Self::NotEqual),
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEqual),
            ">" => Ok(Self::GreaterThan),
            ">=" => Ok(Self::GreaterThanOrEqual),
            "like" => Ok(Self::Like),
            "ilike" => Ok(Self::ILike),
            _ => Err(()),
        }
    }
}
