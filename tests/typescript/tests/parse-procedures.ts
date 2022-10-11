// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>

import fs from 'node:fs';
import path from 'node:path';
import { analyze, Type } from 'poc-plpgsql-analyzer';

const PROCEDURE_HEADINGS_DIR = '../procedure/heading';

describe('try to parse Oracle procedures', () => {
  const files = fs.readdirSync(PROCEDURE_HEADINGS_DIR)
    .filter((name) => name.match(/^(.+)\.ora\.sql$/))
    .map((name) => path.join(PROCEDURE_HEADINGS_DIR, name));

  it.skip.each(files)('should parse %s successfully', (path) => {
    const content = fs.readFileSync(path, 'utf8');

    let error;
    try {
      analyze(Type.Procedure, content);
    } catch (err) {
      error = err;
    }

    expect(error).toEqual(undefined);
  });

  it('should count the lines of code correctly', () => {
    const content = fs.readFileSync('../fixtures/add_job_history.sql', 'utf8');

    const metaData = analyze(Type.Procedure, content);
    expect(metaData.lines_of_code).toEqual(3);
  });
});
