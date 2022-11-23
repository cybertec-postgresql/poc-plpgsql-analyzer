// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, applyRule, DboMetaData, DboType, DboColumnType } from 'poc-plpgsql-analyzer';

const FUNCTION_HEADINGS_DIR = '../function/heading';
const PROCEDURE_HEADINGS_DIR = '../procedure/heading';
const QUERYS_DIR = '../dql';

describe('try to parse and analyze Oracle function', () => {
  const files = fs
    .readdirSync(FUNCTION_HEADINGS_DIR)
    .filter(name => name.match(/^(.+)\.ora\.sql$/))
    .map(name => path.join(FUNCTION_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', path => {
    const content = fs.readFileSync(path, 'utf8');
    const metaData = analyze(DboType.Function, content, { tables: {} });

    expect(metaData.function).toEqual(expect.anything());
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.procedure).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should return the correct function name', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');
    const metaData = analyze(DboType.Function, content, { tables: {} });

    expect(metaData.function.name).toEqual('function_heading_example');
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.procedure).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');
    const metaData = analyze(DboType.Function, content, { tables: {} });

    expect(metaData.function.linesOfCode).toEqual(1);
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.procedure).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });
});

describe('try to parse and analyze Oracle procedures', () => {
  const files = fs
    .readdirSync(PROCEDURE_HEADINGS_DIR)
    .filter(name => name.match(/^(.+)\.ora\.sql$/))
    .map(name => path.join(PROCEDURE_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', path => {
    const content = fs.readFileSync(path, 'utf8');
    const metaData = analyze(DboType.Procedure, content, { tables: {} });

    expect(metaData.procedure).toEqual(expect.anything());
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should return the correct procedure name', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');
    const metaData = analyze(DboType.Procedure, content, { tables: {} });

    expect(metaData.procedure.name).toEqual('add_job_history');
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');
    const metaData = analyze(DboType.Procedure, content, { tables: {} });

    expect(metaData.procedure.linesOfCode).toEqual(3);
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });
});

describe('try to parse and analyze Oracle `SELECT` querys', () => {
  const files = fs
    .readdirSync(QUERYS_DIR)
    .filter(name => name.match(/^(.+)\.ora\.sql$/))
    .map(name => path.join(QUERYS_DIR, name));

  it.each(files)('should parse %s successfully', path => {
    const content = fs.readFileSync(path, 'utf8');
    const metaData = analyze(DboType.Query, content, { tables: {} });

    expect(metaData.query).toEqual(expect.anything());
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.function).toBeUndefined();
    expect(metaData.procedure).toBeUndefined();
  });

  it('should return the correct amount of outer joins', () => {
    const content = fs.readFileSync('../dql/select_left_join.ora.sql', 'utf8');

    const metaData = analyze(DboType.Query, content, { tables: {} });
    expect(metaData.query.outerJoins).toEqual(1);
    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.function).toBeUndefined();
    expect(metaData.procedure).toBeUndefined();
  });
});

describe('passing type context information into analyzer', () => {
  it('should be able to analyze procedure with `%TYPE` parameters', () => {
    const content = fs.readFileSync('../fixtures/log_last_login_fuzzy.ora.sql', 'utf8');
    const context = {
      tables: {
        persons: {
          columns: {
            id: { typ: DboColumnType.Integer },
            name: { typ: DboColumnType.Text },
            number_of_logins: { typ: DboColumnType.Integer },
            last_login: { typ: DboColumnType.Date },
          },
        },
      },
    };

    const metaData = analyze(DboType.Procedure, content, context);
    expect(metaData.procedure.name).toEqual('log_last_login_fuzzy');
    expect(metaData.procedure.linesOfCode).toEqual(2);

    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.rules.length).toEqual(2);
    expect(metaData.rules[0]).toEqual({
      name: 'CYAR-0002',
      locations: [{ offset: { start: 315, end: 317 } }],
      short_desc: 'Replace procedure prologue',
    });
    expect(content.substring(315, 317)).toEqual('IS');

    expect(metaData.rules[1]).toEqual({
      name: 'CYAR-0003',
      locations: [{ offset: { start: 571, end: 595 } }],
      short_desc: 'Replace procedure epilogue',
    });
    expect(content.substring(571, 595)).toEqual('END log_last_login_fuzzy');

    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });
});

describe('trying to apply some transpiler rules', () => {
  it('should transform the PL/SQL code correctly', () => {
    const content = fs.readFileSync('../fixtures/secure_dml.ora.sql', 'utf8');
    const context = { tables: {} };
    let metaData = analyze(DboType.Procedure, content, context);

    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.rules.length).toEqual(4);
    expect(metaData.rules[0].name).toEqual('CYAR-0001');
    expect(metaData.rules[1].name).toEqual('CYAR-0002');
    expect(metaData.rules[2].name).toEqual('CYAR-0003');
    expect(metaData.rules[3].name).toEqual('CYAR-0005');

    let transpiled = content;
    const doApply = rule => {
      let location;
      [transpiled, location] = applyRule(DboType.Procedure, transpiled, rule.name, rule.locations[0], context);

      return analyze(DboType.Procedure, transpiled, context);
    };

    expect(metaData.rules[0].name).toEqual('CYAR-0001');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0002');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0003');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0005');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(2);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0005');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(transpiled).toEqual(`CREATE PROCEDURE secure_dml()
AS $$
BEGIN
  IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
        OR TO_CHAR (clock_timestamp(), 'DY') IN ('SAT', 'SUN') THEN
    RAISE_APPLICATION_ERROR (-20205,
        'You may only make changes during normal office hours');
  END IF;
END;
$$ LANGUAGE plpgsql;
`);
  });

  it('should transform the PL/SQL code correctly without locations', () => {
    const content = fs.readFileSync('../fixtures/secure_dml.ora.sql', 'utf8');
    const context = { tables: {} };
    let metaData = analyze(DboType.Procedure, content, context);

    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.rules.length).toEqual(4);
    expect(metaData.rules[0].name).toEqual('CYAR-0001');
    expect(metaData.rules[1].name).toEqual('CYAR-0002');
    expect(metaData.rules[2].name).toEqual('CYAR-0003');
    expect(metaData.rules[3].name).toEqual('CYAR-0005');

    let transpiled = content;
    const doApply = rule => {
      let location;
      [transpiled, location] = applyRule(DboType.Procedure, transpiled, rule.name, null, context);

      return analyze(DboType.Procedure, transpiled, context);
    };

    expect(metaData.rules[0].name).toEqual('CYAR-0001');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0002');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0003');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0005');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(2);
    doApply(metaData.rules[0]);

    expect(transpiled).toEqual(`CREATE PROCEDURE secure_dml()
AS $$
BEGIN
  IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
        OR TO_CHAR (clock_timestamp(), 'DY') IN ('SAT', 'SUN') THEN
    RAISE_APPLICATION_ERROR (-20205,
        'You may only make changes during normal office hours');
  END IF;
END;
$$ LANGUAGE plpgsql;
`);
  });
});
