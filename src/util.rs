// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements miscellaneous types and helper.

use std::fmt;
use std::hash::{Hash, Hasher};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Eq)]
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

impl From<String> for SqlIdent {
    fn from(s: String) -> Self {
        let is_quoted = s.starts_with('"') && s.ends_with('"');
        Self::new(s, is_quoted)
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

struct SqlIdentVisitor;

impl<'de> Visitor<'de> for SqlIdentVisitor {
    type Value = SqlIdent;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a valid SQL identifier")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
}

impl<'de> Deserialize<'de> for SqlIdent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SqlIdentVisitor)
    }
}
