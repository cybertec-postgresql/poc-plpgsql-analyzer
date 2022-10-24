// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, DboMetaData, DboType } from 'poc-plpgsql-analyzer';

const FUNCTION_HEADINGS_DIR = '../function/heading';
const PROCEDURE_HEADINGS_DIR = '../procedure/heading';
const QUERYS_DIR = '../dql';

describe('try to parse and analyze Oracle function', () => {
  const files = fs.readdirSync(FUNCTION_HEADINGS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(FUNCTION_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Function, content);

    expect(result.Function).toEqual(expect.anything());
  });

  it('should return the correct function name', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');

    const metaData = analyze(DboType.Function, content);
    expect(metaData.Function.name).toEqual('function_heading_example');
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../function/heading/function_heading_example.ora.sql', 'utf8');

    const metaData = analyze(DboType.Function, content);
    expect(metaData.Function.lines_of_code).toEqual(1);
  });
});

describe('try to parse and analyze Oracle procedures', () => {
  const files = fs.readdirSync(PROCEDURE_HEADINGS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(PROCEDURE_HEADINGS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Procedure, content);

    expect(result.Procedure).toEqual(expect.anything());
  });

  it('should return the correct procedure name', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');

    const metaData = analyze(DboType.Procedure, content);
    expect(metaData.Procedure.name).toEqual('add_job_history');
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');

    const metaData = analyze(DboType.Procedure, content);
    expect(metaData.Procedure.lines_of_code).toEqual(3);
  });
});

describe('try to parse and analyze Oracle `SELECT` querys', () => {
  const files = fs.readdirSync(QUERYS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(QUERYS_DIR, name));

  it.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');
    const result = analyze(DboType.Query, content);

    expect(result.Query).toEqual(expect.anything());
  });

  it('should return the correct function name', () => {
    const content = fs.readFileSync('../dql/select_left_join.ora.sql', 'utf8');

    const metaData = analyze(DboType.Query, content);
    expect(metaData.Query.outer_joins).toEqual(1);
  });
});
