// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements miscellaneous types and helper.

use serde::Serialize;
use std::fmt;
use std::hash::{Hash, Hasher};
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::TypescriptDefinition;

#[derive(Clone, Debug, Eq, Serialize, TypescriptDefinition)]
pub struct SqlIdent {
    name: String,
    is_quoted: bool,
}

impl SqlIdent {
    pub fn new<S>(name: S, is_quoted: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            is_quoted,
        }
    }
}

impl From<&str> for SqlIdent {
    fn from(s: &str) -> Self {
        Self::new(s, s.starts_with('"') && s.ends_with('"'))
    }
}

impl fmt::Display for SqlIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escape = |s: &str| s.replace('\"', "\"\"");

        if self.is_quoted {
            write!(f, "\"{}\"", escape(&self.name))
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl PartialEq for SqlIdent {
    fn eq(&self, other: &Self) -> bool {
        // If the quote-status is different, don't even try further.
        if self.is_quoted != other.is_quoted {
            return false;
        }

        if self.is_quoted {
            // And if the are indeed quoted, the must be *exactly* equal.
            self.name == other.name
        } else {
            // Otherwise, do a case-insenstive compare.
            self.name.to_lowercase() == other.name.to_lowercase()
        }
    }
}

impl Hash for SqlIdent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_quoted.hash(state);

        if self.is_quoted {
            self.name.hash(state);
        } else {
            self.name.to_lowercase().hash(state);
        }
    }
}
