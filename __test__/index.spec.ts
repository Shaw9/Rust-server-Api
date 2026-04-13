import test from 'ava'

import { add } from '../index'

test('sync function from native code', (t) => {
  const fixture = 42
  t.is(add(fixture, 100), fixture + 100)
})
