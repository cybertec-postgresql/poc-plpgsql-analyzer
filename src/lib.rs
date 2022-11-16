// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

mod analyze;
mod ast;
mod grammar;
mod lexer;
mod parser;
mod rules;
mod syntax;
mod util;

pub use analyze::*;
pub use parser::*;
pub use rules::apply_rule;
pub use util::SqlIdent;
