// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use std::fs;
use std::path::Path;

use poc_plpgsql_analyzer::{
    parse_function, parse_procedure, parse_query, parse_trigger, parse_view,
};

fn test_parse_coverage(path: &Path) -> datatest_stable::Result<()> {
    let components = path.components().collect::<Vec<_>>();
    let typ = components
        .get(1)
        .expect("Failed to get second component from path");
    let content = fs::read_to_string(path)?;

    let result = match typ.as_os_str().to_str().unwrap() {
        "dql" => parse_query(&content),
        "function" => parse_function(&content),
        "procedure" => parse_procedure(&content),
        "trigger" => parse_trigger(&content),
        "view" => parse_view(&content),
        typ => panic!("Can not parse typ {}", typ),
    };
    assert!(result.is_ok(), "{result:#?}");

    let parse = result.unwrap();
    assert!(
        parse.errors.is_empty(),
        "\n{:#?}\n{:?}",
        parse.syntax(),
        parse.errors
    );
    Ok(())
}

datatest_stable::harness! {
    { test = test_parse_coverage, root = "tests/procedure", pattern = r"^(.*).ora\.sql$" },
    { test = test_parse_coverage, root = "tests/function", pattern = r"^(.*)\.sql$" },
    { test = test_parse_coverage, root = "tests/dql", pattern = r"(.*)\.sql$" },
    { test = test_parse_coverage, root = "tests/trigger", pattern = r"(.*)\.sql$" },
    { test = test_parse_coverage, root = "tests/view", pattern = r"(.*)\.sql$" },
}
