// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements a syntax-level representation of the input.

use num_traits::{FromPrimitive, ToPrimitive};

pub use generated::SyntaxKind;

mod generated;

/// Dummy type for our PL/SQL language definition, for use with rowan.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SqlProcedureLang {}

impl rowan::Language for SqlProcedureLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

/// Typed [`SyntaxNode`] with our [`SqlProcedureLang`] language definition.
pub type SyntaxNode = rowan::SyntaxNode<SqlProcedureLang>;
/// Typed [`SyntaxToken`] with our [`SqlProcedureLang`] language definition.
pub type SyntaxToken = rowan::SyntaxToken<SqlProcedureLang>;
/// Typed [`SyntaxElement`] with our [`SqlProcedureLang`] language definition.
#[allow(unused)]
pub type SyntaxElement = rowan::SyntaxElement<SqlProcedureLang>;
