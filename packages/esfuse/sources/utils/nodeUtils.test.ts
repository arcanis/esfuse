import {getRegExpFromPath} from 'esfuse/sources/utils/nodeUtils';

describe(`nodeUtils`, () => {
  const TESTS = [
    [`dist`, `dist/index.js`, true],
    [`dist`, `lib/index.js`, false],
    [`./dist`, `dist/index.js`, true],
    [`./dist`, `lib/index.js`, false],
    [`dist/`, `dist/index.js`, true],
    [`{}/dist`, `hello/dist/index.js`, true],
    [`{}/dist`, `hello/world/dist/index.js`, false],
    [`path/{}/dist`, `path/hello/dist/index.js`, true],
    [`path/{}/dist`, `path/hello/world/dist/index.js`, false],
    [`./{}/dist`, `hello/dist/index.js`, true],
    [`./{}/dist`, `hello/world/dist/index.js`, false],
  ] as const;

  for (const [pattern, subject, expectation] of TESTS) {
    it(`supports ${pattern} / ${subject}`, async () => {
      const m = subject.match(getRegExpFromPath(pattern));
      expect(!!m).toEqual(expectation);
    });
  }
});
