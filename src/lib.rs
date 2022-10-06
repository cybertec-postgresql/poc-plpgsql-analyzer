// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

mod analyze;
mod ast;
mod lexer;
mod parser;
mod grammar;

pub use analyze::*;
pub use ast::{SyntaxElement, SyntaxKind};
pub use lexer::{Lexer, Token};
pub use parser::{Parser, ParseError};
