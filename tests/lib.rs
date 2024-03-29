// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use poc_plpgsql_analyzer::parse_procedure;

const ADD_JOB_HISTORY: &str = include_str!("fixtures/add_job_history.sql");

#[test]
fn check_parse_procedure() {
    let result = parse_procedure(ADD_JOB_HISTORY);
    assert!(result.is_ok(), "{result:#?}");
    assert!(result.unwrap().errors.is_empty());
}
