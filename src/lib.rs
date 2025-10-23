// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

#![forbid(unsafe_code)]
#![deny(warnings)]

pub use analyzer::*;
pub use ast::*;
pub use parser::*;
pub use source_gen::syntax::*;
pub use util::SqlIdent;

mod analyzer;
mod ast;
mod grammar;
mod parser;
mod util;
