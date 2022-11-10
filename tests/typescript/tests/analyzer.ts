// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, DboMetaData, DboType, DboColumnType } from 'poc-plpgsql-analyzer';

const FUNCTION_HEADINGS_DIR = '../function/heading';
const PROCEDURE_HEADINGS_DIR = '../procedure/heading';
const QUERYS_DIR = '../dql';

describe('try to parse and analyze Oracle function', () => {
  const files = fs.readdirSync(FUNCTION_HEADINGS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(FUNCTION_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Function, content, { tables: {} });

    expect(result.function).toEqual(expect.anything());
  });

  it('should return the correct function name', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');

    const metaData = analyze(DboType.Function, content, { tables: {} });
    expect(metaData.function.name).toEqual('function_heading_example');
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');

    const metaData = analyze(DboType.Function, content, { tables: {} });
    expect(metaData.function.linesOfCode).toEqual(1);
  });
});

describe('try to parse and analyze Oracle procedures', () => {
  const files = fs.readdirSync(PROCEDURE_HEADINGS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(PROCEDURE_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Procedure, content, { tables: {} });

    expect(result.procedure).toEqual(expect.anything());
  });

  it('should return the correct procedure name', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');

    const metaData = analyze(DboType.Procedure, content, { tables: {} });
    expect(metaData.procedure.name).toEqual('add_job_history');
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');

    const metaData = analyze(DboType.Procedure, content, { tables: {} });
    expect(metaData.procedure.linesOfCode).toEqual(3);
  });
});

describe('try to parse and analyze Oracle `SELECT` querys', () => {
  const files = fs.readdirSync(QUERYS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(QUERYS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Query, content, { tables: {} });

    expect(result.query).toEqual(expect.anything());
  });

  it('should return the correct amount of outer joins', () => {
    const content = fs.readFileSync('../dql/select_left_join.ora.sql', 'utf8');

    const metaData = analyze(DboType.Query, content, { tables: {} });
    expect(metaData.query.outerJoins).toEqual(1);
  });
});

describe('passing type context information into analyzer', () => {
    it('should be able to analyze procedure with `%TYPE` parameters', () => {
      const content = fs.readFileSync('../fixtures/log_last_login_fuzzy.ora.sql','utf8');
      const context = {
        tables: {
          persons: {
            columns: {
              // TODO: This should be usable as `id: { typ: DboColumnType.Integer }`
              // But somehow `serde_wasw_bindgen` messes up and can only deserialize
              // this type from strings.
              id: { typ: 'Integer' },
              name: { typ: 'Text' },
              number_of_logins: { typ: 'Integer' },
              last_login: { typ: 'Date' },
            },
          },
        },
      };

      const metaData = analyze(DboType.Procedure, content, context);
      console.log(metaData);
    });
});
