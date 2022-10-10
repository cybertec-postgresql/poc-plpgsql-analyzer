use poc_plpgsql_analyzer::parse;
use std::fs;
use std::path::Path;

fn test_parse_procedure_coverage(path: &Path) -> datatest_stable::Result<()> {
    let input = fs::read_to_string(path)?;
    let result = parse(&input);
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
    r"^(.*).ora\.sql$"
);