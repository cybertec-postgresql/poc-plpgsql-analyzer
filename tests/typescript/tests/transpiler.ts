// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH <office@cybertec.at>
import fs from 'node:fs';
import { analyze, applyRule, DboAnalyzeContext } from 'poc-plpgsql-analyzer';

describe('apply transpiler rules', () => {
  it('CYAR-0006', () => {
    const content = fs.readFileSync('../dql/nvl-coalesce.ora.sql', 'utf8');
    const context: DboAnalyzeContext = { tables: {} };
    let metaData = analyze('query', content, context);

    expect(metaData.rules).toBeInstanceOf(Array);
    expect(metaData.rules.length).toEqual(1);
    expect(metaData.rules[0].name).toEqual('CYAR-0006');

    let original = content;
    const doApply = rule => {
      original = applyRule('query', original, rule.name, rule.locations[0], context).original;
      return analyze('query', original, context);
    };

    expect(metaData.rules[0].name).toEqual('CYAR-0006');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(2);
    metaData = doApply(metaData.rules[0]);

    expect(metaData.rules[0].name).toEqual('CYAR-0006');
    expect(metaData.rules[0].locations).toBeInstanceOf(Array);
    expect(metaData.rules[0].locations.length).toEqual(1);

    metaData = doApply(metaData.rules[0]);
    expect(metaData.rules).toHaveLength(0);

    expect(original).toEqual(fs.readFileSync('../dql/nvl-coalesce.pg.sql', 'utf8'));
  });
});
