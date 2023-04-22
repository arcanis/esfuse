import assert     from 'assert/strict';
import {foo, bar} from 'package-exports/export-cjs-constants';

assert.equal(foo, 42);
assert.equal(bar, 21);
