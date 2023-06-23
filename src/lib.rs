// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

#![forbid(unsafe_code)]

pub use analyzer::*;
pub use ast::IdentGroup;
pub use parser::*;
pub use syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
pub use util::SqlIdent;

mod analyzer;
mod ast;
mod grammar;
mod lexer;
mod parser;
mod sourcegen;
mod syntax;
mod util;
