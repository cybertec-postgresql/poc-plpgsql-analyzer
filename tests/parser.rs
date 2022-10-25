// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use poc_plpgsql_analyzer::{parse_function, parse_procedure};
use std::fs;
use std::path::Path;

fn test_parse_procedure_coverage(path: &Path) -> datatest_stable::Result<()> {
    let input = fs::read_to_string(path)?;
    let result = parse_procedure(&input);
    assert!(result.is_ok(), "{:#?}", result);
    let parse = result.unwrap();
    assert!(
        parse.errors.is_empty(),
        "\n{}\n{:?}",
        parse.tree(),
        parse.errors
    );
    Ok(())
}

fn test_parse_function_coverage(path: &Path) -> datatest_stable::Result<()> {
    let input = fs::read_to_string(path)?;
    let result = parse_function(&input);
    assert!(result.is_ok(), "{:#?}", result);
    let parse = result.unwrap();
    assert!(
        parse.errors.is_empty(),
        "\n{}\n{:?}",
        parse.tree(),
        parse.errors
    );
    Ok(())
}

datatest_stable::harness!(
    test_parse_procedure_coverage,
    "tests/procedure",
    r"^(.*).ora\.sql$",
    test_parse_function_coverage,
    "tests/function",
    r"^(.*).ora\.sql$",
);
