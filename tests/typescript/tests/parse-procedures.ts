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

  test.skip.each(files)('%s', async (path) => {
    const content = fs.readFileSync(path, 'utf8');

    let error;
    try {
      analyze(Type.Procedure, content);
    } catch (err) {
      error = err;
    }

    expect(error).toEqual(undefined);
  });
});
