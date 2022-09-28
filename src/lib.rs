// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Proof of concept interface and implementation for a PL/SQL parser.

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod analyze;
mod parser;

pub use analyze::*;
