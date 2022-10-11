use poc_plpgsql_analyzer::parse;

const ADD_JOB_HISTORY: &str = include_str!("fixtures/add_job_history.sql");

#[test]
fn check_parse_procedure() {
    let result = parse(ADD_JOB_HISTORY);
    assert!(result.is_ok(), "{:#?}", result);
    assert!(result.unwrap().errors.is_empty());
}
