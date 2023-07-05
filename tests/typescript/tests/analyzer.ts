// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, DboAnalyzeContext } from 'poc-plpgsql-analyzer';

const FUNCTION_HEADINGS_DIR = '../function/heading';
const PROCEDURE_HEADINGS_DIR = '../procedure/heading';
const TRIGGERS_DIR = '../trigger';
const QUERYS_DIR = '../dql';

describe('try to parse and analyze Oracle function', () => {
  const files = fs
    .readdirSync(FUNCTION_HEADINGS_DIR)
    .filter(name => name.match(/^(.+)\.ora\.sql$/))
    .map(name => path.join(FUNCTION_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', path => {
    const content = fs.readFileSync(path, 'utf8');
    const metaData = analyze('function', content, { tables: {} });

    expect(metaData.function).toEqual(expect.anything());
    expect(metaData.procedure).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should return the correct function name', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');
    const metaData = analyze('function', content, { tables: {} });

    expect(metaData.function.name).toEqual('function_heading_example');
    expect(metaData.procedure).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');
    const metaData = analyze('function', content, { tables: {} });

    expect(metaData.function.linesOfCode).toEqual(3);
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
    const metaData = analyze('procedure', content, { tables: {} });

    expect(metaData.procedure).toEqual(expect.anything());
    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should return the correct procedure name', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');
    const metaData = analyze('procedure', content, { tables: {} });

    expect(metaData.procedure.name).toEqual('add_job_history');
    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');
    const metaData = analyze('procedure', content, { tables: {} });

    expect(metaData.procedure.linesOfCode).toEqual(5);
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
    const metaData = analyze('query', content, { tables: {} });

    expect(metaData.query).toEqual(expect.anything());
    expect(metaData.function).toBeUndefined();
    expect(metaData.procedure).toBeUndefined();
  });

  it('should return the correct amount of outer joins', () => {
    const content = fs.readFileSync('../dql/select_left_join.ora.sql', 'utf8');

    const metaData = analyze('query', content, { tables: {} });
    expect(metaData.query.outerJoins).toEqual(1);
    expect(metaData.function).toBeUndefined();
    expect(metaData.procedure).toBeUndefined();
  });
});

describe('try to parse and analyze Oracle triggers', () => {
  const files = fs
    .readdirSync(TRIGGERS_DIR)
    .filter(name => name.match(/^(.+)\.ora\.sql$/))
    .map(name => path.join(TRIGGERS_DIR, name));

  it.each(files)('should parse %s successfully', path => {
    const content = fs.readFileSync(path, 'utf8');
    const metaData = analyze('trigger', content, { tables: {} });

    expect(metaData.trigger).toEqual(expect.anything());
  });

  it('should return the correct trigger name and lines of code', () => {
    const content = fs.readFileSync('../trigger/after_trigger.ora.sql', 'utf8');
    const metaData = analyze('trigger', content, { tables: {} });

    expect(metaData.trigger.name).toEqual('store.after_trigger');
    expect(metaData.trigger.linesOfCode).toEqual(4);
  });
});

describe('passing type context information into analyzer', () => {
  it('should be able to analyze procedure with `%TYPE` parameters', () => {
    const content = fs.readFileSync('../fixtures/log_last_login_fuzzy.ora.sql', 'utf8');
    const context: DboAnalyzeContext = {
      tables: {
        persons: {
          columns: {
            id: { typ: 'integer' },
            name: { typ: 'text' },
            number_of_logins: { typ: 'integer' },
            last_login: { typ: 'date' },
          },
        },
      },
    };

    const metaData = analyze('procedure', content, context);
    expect(metaData.procedure.name).toEqual('log_last_login_fuzzy');
    expect(metaData.procedure.linesOfCode).toEqual(5);

    expect(content.substring(315, 317)).toEqual('IS');
    expect(content.substring(571, 595)).toEqual('END log_last_login_fuzzy');

    expect(metaData.function).toBeUndefined();
    expect(metaData.query).toBeUndefined();
  });
});
