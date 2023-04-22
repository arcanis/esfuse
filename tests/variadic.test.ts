import {makeAppRunner, makeTestApplication} from './helpers';

describe(`Variadic imports`, () => {
  it(`should support lazily importing a file through a variadic pattern`, async () => {
    const app = makeTestApplication();
    try {
      const {runtime, query} = await makeAppRunner(app);

      const {run} = await query.get(`/_dev/bundle/app/fixtures/variadic-modules/import-lazy.js`);

      expect(runtime.getDefinedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-lazy.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/a.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/b.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/c.js`,
        `/_dev/internal/lazy/app/fixtures/variadic-modules/simple/[...t0].js`,
      ]);

      expect(runtime.getEvaluatedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-lazy.js`,
      ]);

      await expect(run(`a`)).resolves.toMatchObject({
        val: `a`,
      });

      expect(runtime.getEvaluatedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-lazy.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/a.js`,
        `/_dev/internal/lazy/app/fixtures/variadic-modules/simple/[...t0].js`,
      ]);

      await expect(run(`c`)).resolves.toMatchObject({
        val: `c`,
      });

      expect(runtime.getEvaluatedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-lazy.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/a.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/c.js`,
        `/_dev/internal/lazy/app/fixtures/variadic-modules/simple/[...t0].js`,
      ]);
    } finally {
      app.dispose();
    }
  });

  it(`should support eagerly importing a file through a variadic pattern`, async () => {
    const app = makeTestApplication();
    try {
      const {runtime, query} = await makeAppRunner(app);

      const {run} = await query.get(`/_dev/bundle/app/fixtures/variadic-modules/import-eager.js`);

      expect(runtime.getDefinedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-eager.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/a.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/b.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/c.js`,
        `/_dev/internal/eager/app/fixtures/variadic-modules/simple/[val].js`,
      ]);

      expect(runtime.getEvaluatedModules()).toEqual([
        `/_dev/file/app/fixtures/variadic-modules/import-eager.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/a.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/b.js`,
        `/_dev/file/app/fixtures/variadic-modules/simple/c.js`,
        `/_dev/internal/eager/app/fixtures/variadic-modules/simple/[val].js`,
      ]);

      expect(run(`a`)).toMatchObject({
        val: `a`,
      });

      expect(run(`c`)).toMatchObject({
        val: `c`,
      });
    } finally {
      app.dispose();
    }
  });
});
