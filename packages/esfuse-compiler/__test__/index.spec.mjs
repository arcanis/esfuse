import test from 'ava';

import {transform} from '../index.js';

test('simple transform', (t) => {
  t.is(transform(`file.ts`, `console.log('foo');`), [
    `$esfuse$.define("<file.ts>", (module, exports, require)=>{\n`,
    `    console.log("foo");\n`,
    `});\n`,
  ].join(``));
});
