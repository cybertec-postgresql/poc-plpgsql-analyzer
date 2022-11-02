// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements a typed AST node for SQL expressions.

use super::typed_syntax_node;
use crate::syntax::SyntaxToken;
use std::str::FromStr;

typed_syntax_node!(Expression);

#[derive(Debug, Eq, PartialEq)]
pub enum ComparisonOpType {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
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
}

impl FromStr for ComparisonOpType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(Self::Equal),
            "<>" => Ok(Self::NotEqual),
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEqual),
            ">" => Ok(Self::GreaterThan),
            ">=" => Ok(Self::GreaterThanOrEqual),
            _ => Err(()),
        }
    }
}
