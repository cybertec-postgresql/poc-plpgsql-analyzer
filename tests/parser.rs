use poc_plpgsql_analyzer::parse_procedure;
use std::fs;
use std::path::Path;

fn test_parse_procedure_coverage(path: &Path) -> datatest_stable::Result<()> {
    let input = fs::read_to_string(path)?;
    let result = parse_procedure(&input);
    assert!(result.is_ok(), "{:#?}", result);
    Ok(())
}

datatest_stable::harness!(
    test_parse_procedure_coverage,
    "tests/procedure",
    r"^(.*).ora\.sql$"
);
