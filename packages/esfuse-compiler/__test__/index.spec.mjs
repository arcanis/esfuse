import test from 'ava'

import {bundle} from '../index.js'

test('sum from native', (t) => {
  t.is(sum(1, 2), 3)
})
