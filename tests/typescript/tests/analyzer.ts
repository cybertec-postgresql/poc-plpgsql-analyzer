// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, DboMetaData, DboType } from 'poc-plpgsql-analyzer';

const PROCEDURE_HEADINGS_DIR = '../procedure/heading';
const FUNCTION_HEADINGS_DIR = '../function/heading';

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
